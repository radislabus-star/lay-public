use crate::stable_hash::mix64_golden;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::Path;
use std::sync::Arc;

const SENTENCE_ANCHOR: &str = "__lay_l3_sentence_v1";
const MARKER_PREFIX: &str = "__lay_l3_";
pub(crate) const PAIR_VIEW_LEFT_EXACT: usize = 14;
pub(crate) const PAIR_VIEW_RIGHT_EXACT: usize = 15;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SentenceContextScene {
    left: Vec<String>,
    right: Vec<String>,
    punctuation_before: PunctuationClass,
    punctuation_after: PunctuationClass,
    position: SlotPosition,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum PunctuationClass {
    #[default]
    None,
    Comma,
    Terminal,
    Clause,
    Bracket,
    Quote,
    Other,
}

impl PunctuationClass {
    fn from_separator(separator: &str) -> Self {
        if separator.contains(',') {
            Self::Comma
        } else if separator
            .chars()
            .any(|ch| matches!(ch, '.' | '!' | '?' | '…'))
        {
            Self::Terminal
        } else if separator.chars().any(|ch| matches!(ch, ':' | ';')) {
            Self::Clause
        } else if separator
            .chars()
            .any(|ch| matches!(ch, '(' | ')' | '[' | ']' | '{' | '}'))
        {
            Self::Bracket
        } else if separator
            .chars()
            .any(|ch| matches!(ch, '"' | '\'' | '«' | '»' | '“' | '”' | '„'))
        {
            Self::Quote
        } else if separator.chars().any(|ch| !ch.is_whitespace()) {
            Self::Other
        } else {
            Self::None
        }
    }

    fn id(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Comma => 1,
            Self::Terminal => 2,
            Self::Clause => 3,
            Self::Bracket => 4,
            Self::Quote => 5,
            Self::Other => 6,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotPosition {
    Start,
    Middle,
    End,
    Only,
}

impl SlotPosition {
    fn id(self) -> u8 {
        match self {
            Self::Start => 1,
            Self::Middle => 2,
            Self::End => 3,
            Self::Only => 4,
        }
    }
}

#[derive(Clone, Debug)]
struct SurfaceToken {
    text: String,
    start: usize,
    end: usize,
}

#[derive(Clone, Debug)]
struct TokenizedSurface {
    tokens: Vec<SurfaceToken>,
    separators: Vec<String>,
}

pub(crate) struct SentenceCandidateProjection {
    pub(crate) scene: SentenceContextScene,
    pub(crate) candidates: Vec<Option<String>>,
}

#[derive(Clone, Debug)]
struct SentenceProofCase {
    split: String,
    class: String,
    original: String,
    candidates: Vec<String>,
    expected: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
struct SentenceClassProof {
    cases: usize,
    target_lattice: usize,
    unique_top1: usize,
    ambiguity_cases: usize,
    ambiguity_retained: usize,
    false_authority: usize,
    target_lattice_ppm: u32,
    unique_top1_ppm: u32,
    ambiguity_retention_ppm: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct SentenceCaseOutcome {
    expected_target: bool,
    passed: bool,
}

#[derive(Debug, Default)]
struct SentenceProofEvaluation {
    classes: BTreeMap<String, SentenceClassProof>,
    failures: Vec<serde_json::Value>,
    outcomes: Vec<SentenceCaseOutcome>,
    false_authority: usize,
}

pub(crate) fn compile_supervised_relation_delta(
    projection_base: &super::ContextPhasePackage,
    scenes: &[String],
    target: &str,
    competitors: &[String],
    min_profile_support: u32,
    signature_schema: u32,
) -> io::Result<(super::ContextPhasePackage, serde_json::Value)> {
    if scenes.is_empty() || target.trim().is_empty() || competitors.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "supervised relation delta requires scenes, target, and competitors",
        ));
    }
    let config = super::online::OnlineContextPhaseConfig::production_with_signature_schema(
        min_profile_support,
        signature_schema,
    );
    let mut learner = super::online::OnlineContextPhaseLearner::new_with_projection_base(
        config,
        Arc::new(super::SurfaceMutationField::default()),
        projection_base,
    );
    let mut distinct_scenes = BTreeSet::new();
    let mut projected = Vec::new();
    for (index, surface) in scenes.iter().enumerate() {
        if !distinct_scenes.insert(surface.clone()) {
            continue;
        }
        let Some((scene, projected_target, projected_competitors)) =
            project_supervised_tail_relation(surface, target, competitors)
        else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "supervised relation scene {} does not end in the labelled target",
                    index + 1
                ),
            ));
        };
        projected.push((scene, projected_target, projected_competitors));
    }
    if distinct_scenes.len() < config.min_profile_support as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "supervised relation delta has {} distinct scenes; {} required",
                distinct_scenes.len(),
                config.min_profile_support
            ),
        ));
    }

    // Build one stationary semantic basis for the bounded relation batch, then
    // reduce every independently labelled episode into that same coordinate
    // system. Reading the caller-provided scenes remains a single bounded pass.
    for (scene, _, _) in &projected {
        learner.prepare_supervised_sentence_basis(&scene.encoded_tokens());
    }
    for (scene, projected_target, projected_competitors) in &projected {
        learner.ingest_supervised_sentence_on_prepared_basis(
            &scene.encoded_tokens(),
            &scene.pair_views(),
            projected_target,
            projected_competitors,
        );
    }

    let package = learner.snapshot();
    let (exact_pair_profiles, generalized_pair_profiles) = package.pair_profile_counts();
    let report = serde_json::json!({
        "kind": "l3_supervised_relation_delta_compile",
        "architecture": "sentence_slot_directional_relation_v1",
        "signature_schema": package.signature_schema,
        "corpus_passes": 1,
        "source_scenes": scenes.len(),
        "distinct_scenes": distinct_scenes.len(),
        "supervised_relations": 1,
        "competitors": competitors.len(),
        "raw_words_stored": false,
        "emitted_semantic_states": package.semantic_states.len(),
        "emitted_candidate_profiles": package.profiles.len(),
        "emitted_signature_profiles": package.signature_profiles.len(),
        "emitted_pair_profiles": package.pair_profiles.len(),
        "emitted_exact_pair_profiles": exact_pair_profiles,
        "emitted_generalized_pair_profiles": generalized_pair_profiles,
        "runtime_authority": false,
    });
    Ok((package, report))
}

fn project_supervised_tail_relation(
    surface: &str,
    target: &str,
    competitors: &[String],
) -> Option<(SentenceContextScene, String, Vec<String>)> {
    let surface = tokenize_surface(surface);
    let slot = surface.tokens.len().checked_sub(1)?;
    let target = target.trim().to_lowercase();
    if surface.tokens[slot].text.to_lowercase() != target {
        return None;
    }
    let mut seen = BTreeSet::new();
    let competitors = competitors
        .iter()
        .map(|competitor| competitor.trim().to_lowercase())
        .filter(|competitor| !competitor.is_empty() && competitor != &target)
        .filter(|competitor| seen.insert(competitor.clone()))
        .collect::<Vec<_>>();
    if competitors.is_empty() {
        return None;
    }
    let left = surface.tokens[..slot]
        .iter()
        .map(|token| token.text.clone())
        .collect::<Vec<_>>();
    let right = surface.tokens[slot + 1..]
        .iter()
        .map(|token| token.text.clone())
        .collect::<Vec<_>>();
    let position = match (left.is_empty(), right.is_empty()) {
        (true, true) => SlotPosition::Only,
        (true, false) => SlotPosition::Start,
        (false, true) => SlotPosition::End,
        (false, false) => SlotPosition::Middle,
    };
    Some((
        SentenceContextScene {
            left,
            right,
            punctuation_before: PunctuationClass::from_separator(&surface.separators[slot]),
            punctuation_after: PunctuationClass::from_separator(&surface.separators[slot + 1]),
            position,
        },
        target,
        competitors,
    ))
}

pub(crate) fn build_and_prove_sentence_context_path(
    cases_path: &Path,
    output_package: &Path,
) -> io::Result<serde_json::Value> {
    let cases = parse_cases(&std::fs::read_to_string(cases_path)?)?;
    let training = cases
        .iter()
        .filter(|case| case.split == "train")
        .collect::<Vec<_>>();
    let heldout = cases
        .iter()
        .filter(|case| case.split == "heldout")
        .collect::<Vec<_>>();
    if training.is_empty() || heldout.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "sentence proof requires train and heldout rows",
        ));
    }
    let training_scenes = training
        .iter()
        .map(|case| case.original.clone())
        .collect::<BTreeSet<_>>();
    let heldout_scene_overlap = heldout
        .iter()
        .filter(|case| training_scenes.contains(&case.original))
        .count();

    let mut learner = super::online::OnlineContextPhaseLearner::new_with_surface_field(
        super::online::OnlineContextPhaseConfig::production(2),
        Arc::new(super::SurfaceMutationField::default()),
    );
    const TRAIN_PASSES: usize = 4;
    for _ in 0..TRAIN_PASSES {
        for case in &training {
            let Some(expected) = case.expected.as_deref() else {
                continue;
            };
            let references = case
                .candidates
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();
            let projection =
                project_candidate_lattice(&case.original, &references).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("invalid sentence training lattice for class {}", case.class),
                    )
                })?;
            let Some(target_index) = case
                .candidates
                .iter()
                .position(|candidate| candidate == expected)
            else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "expected target is absent from class {} lattice",
                        case.class
                    ),
                ));
            };
            let target = projection.candidates[target_index]
                .as_deref()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "target changed context")
                })?;
            let competitors = projection
                .candidates
                .iter()
                .enumerate()
                .filter_map(|(index, candidate)| (index != target_index).then_some(candidate))
                .flatten()
                .cloned()
                .collect::<Vec<_>>();
            learner.ingest_supervised_sentence(
                &projection.scene.encoded_tokens(),
                &projection.scene.pair_views(),
                target,
                &competitors,
            );
        }
    }
    let package = learner.snapshot();
    super::write_package(output_package, &package)?;

    let evaluation = evaluate_heldout(&package, &heldout);
    let passed = heldout_scene_overlap == 0 && sentence_gate(&evaluation.classes);
    Ok(serde_json::json!({
        "kind": "l3_sentence_context_fixed_proof",
        "architecture": "sentence_slot_phase_v1",
        "cases": cases_path,
        "output_package": output_package,
        "train_rows": training.len(),
        "train_passes": TRAIN_PASSES,
        "heldout_rows": heldout.len(),
        "heldout_scene_overlap": heldout_scene_overlap,
        "candidate_source": "fixed_bounded_l2_lattice_fixture",
        "left_context": true,
        "right_context": true,
        "punctuation": true,
        "word_order": true,
        "morphology_slot": true,
        "classes": evaluation.classes,
        "failures": evaluation.failures,
        "false_authority": evaluation.false_authority,
        "raw_words_stored": false,
        "runtime_authority": false,
        "verdict": if passed { "PASS" } else { "WATCH" },
    }))
}

pub(crate) fn prove_sentence_context_delta_path(
    manifest_path: &Path,
    delta_path: &Path,
    cases_path: &Path,
    receipt_path: &Path,
) -> io::Result<serde_json::Value> {
    let cases = parse_cases(&std::fs::read_to_string(cases_path)?)?;
    let training_scenes = cases
        .iter()
        .filter(|case| case.split == "train")
        .map(|case| case.original.clone())
        .collect::<BTreeSet<_>>();
    let heldout = cases
        .iter()
        .filter(|case| case.split == "heldout")
        .collect::<Vec<_>>();
    if heldout.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "sentence delta proof requires heldout rows",
        ));
    }
    let heldout_scene_overlap = heldout
        .iter()
        .filter(|case| training_scenes.contains(&case.original))
        .count();
    let baseline = super::L3CompositeMemory::load_manifest(manifest_path)?;
    let candidate = baseline.compose_delta_path(delta_path)?;
    let baseline_sha256 = super::package_sha256(baseline.package());
    let delta_sha256 = super::package_path_sha256(delta_path)?;
    let baseline_evaluation = evaluate_heldout(baseline.package(), &heldout);
    let candidate_evaluation = evaluate_heldout(&candidate, &heldout);
    let improved = baseline_evaluation
        .outcomes
        .iter()
        .zip(&candidate_evaluation.outcomes)
        .filter(|(before, after)| !before.passed && after.passed)
        .count();
    let regressions = baseline_evaluation
        .outcomes
        .iter()
        .zip(&candidate_evaluation.outcomes)
        .filter(|(before, after)| before.passed && !after.passed)
        .count();
    let improve_cases = candidate_evaluation
        .outcomes
        .iter()
        .filter(|outcome| outcome.expected_target)
        .count();
    let target_failures = candidate_evaluation
        .outcomes
        .iter()
        .filter(|outcome| outcome.expected_target && !outcome.passed)
        .count();
    let safety_cases = candidate_evaluation
        .outcomes
        .iter()
        .filter(|outcome| !outcome.expected_target)
        .count();
    let passed = heldout_scene_overlap == 0
        && improved > 0
        && regressions == 0
        && target_failures == 0
        && candidate_evaluation.false_authority == 0
        && sentence_gate(&candidate_evaluation.classes);
    let report = serde_json::json!({
        "kind": "l3_context_delta_targeted_proof",
        "proof_mode": "sentence_multiview_v1",
        "architecture": "sentence_slot_phase_v1",
        "manifest": manifest_path,
        "delta": delta_path,
        "delta_bytes": std::fs::metadata(delta_path)?.len(),
        "baseline_sha256": baseline_sha256,
        "delta_sha256": delta_sha256,
        "cases": cases_path,
        "heldout_rows": heldout.len(),
        "heldout_scene_overlap": heldout_scene_overlap,
        "improve_cases": improve_cases,
        "improved": improved,
        "target_failures": target_failures,
        "safety_cases": safety_cases,
        "false_supports": candidate_evaluation.false_authority,
        "regressions": regressions,
        "baseline_passed_cases": baseline_evaluation.outcomes.iter().filter(|outcome| outcome.passed).count(),
        "candidate_passed_cases": candidate_evaluation.outcomes.iter().filter(|outcome| outcome.passed).count(),
        "baseline_classes": baseline_evaluation.classes,
        "classes": candidate_evaluation.classes,
        "failures": candidate_evaluation.failures,
        "base_rewritten": false,
        "full_corpus_recompiled": false,
        "runtime_authority": false,
        "verdict": if passed { "PASS" } else { "WATCH" },
    });
    let mut bytes = serde_json::to_vec_pretty(&report).map_err(io::Error::other)?;
    bytes.push(b'\n');
    crate::private_file::write_private_bytes(receipt_path, &bytes)?;
    Ok(report)
}

fn evaluate_heldout(
    package: &super::ContextPhasePackage,
    heldout: &[&SentenceProofCase],
) -> SentenceProofEvaluation {
    let mut evaluation = SentenceProofEvaluation::default();
    for case in heldout {
        let mut outcome = SentenceCaseOutcome {
            expected_target: case.expected.is_some(),
            passed: false,
        };
        let proof = evaluation.classes.entry(case.class.clone()).or_default();
        proof.cases += 1;
        let references = case
            .candidates
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let Some(projection) = project_candidate_lattice(&case.original, &references) else {
            evaluation.outcomes.push(outcome);
            continue;
        };
        let valid = projection
            .candidates
            .iter()
            .filter_map(Option::as_deref)
            .collect::<Vec<_>>();
        let readouts = package.score_sentence_candidates(&projection.scene, &valid);
        let valid_indexes = projection
            .candidates
            .iter()
            .enumerate()
            .filter_map(|(index, candidate)| candidate.as_ref().map(|_| index))
            .collect::<Vec<_>>();
        let support_winners = readouts
            .iter()
            .enumerate()
            .filter(|(_, readout)| readout.disposition == super::ContextPhaseDisposition::Support)
            .collect::<Vec<_>>();
        match case.expected.as_deref() {
            Some(expected) => {
                let target = case
                    .candidates
                    .iter()
                    .position(|candidate| candidate == expected);
                if target.is_some_and(|index| projection.candidates[index].is_some()) {
                    proof.target_lattice += 1;
                }
                if support_winners.len() == 1 {
                    let winner = valid_indexes[support_winners[0].0];
                    if Some(winner) == target {
                        proof.unique_top1 += 1;
                        outcome.passed = true;
                    } else {
                        proof.false_authority += 1;
                    }
                }
                if support_winners.len() != 1
                    || support_winners
                        .first()
                        .is_some_and(|winner| Some(valid_indexes[winner.0]) != target)
                {
                    push_failure(
                        &mut evaluation.failures,
                        case,
                        &valid,
                        &readouts,
                        package,
                        &projection.scene,
                    );
                }
            }
            None => {
                proof.ambiguity_cases += 1;
                if support_winners.is_empty() {
                    proof.ambiguity_retained += 1;
                    outcome.passed = true;
                } else {
                    proof.false_authority += support_winners.len();
                    push_failure(
                        &mut evaluation.failures,
                        case,
                        &valid,
                        &readouts,
                        package,
                        &projection.scene,
                    );
                }
            }
        }
        evaluation.outcomes.push(outcome);
    }
    for proof in evaluation.classes.values_mut() {
        let target_cases = proof.cases.saturating_sub(proof.ambiguity_cases);
        proof.target_lattice_ppm = ratio_ppm(proof.target_lattice, target_cases);
        proof.unique_top1_ppm = ratio_ppm(proof.unique_top1, target_cases);
        proof.ambiguity_retention_ppm = ratio_ppm(proof.ambiguity_retained, proof.ambiguity_cases);
    }
    evaluation.false_authority = evaluation
        .classes
        .values()
        .map(|proof| proof.false_authority)
        .sum();
    evaluation
}

fn sentence_gate(classes: &BTreeMap<String, SentenceClassProof>) -> bool {
    !classes.is_empty()
        && classes.values().all(|proof| {
            let target_gate = proof.ambiguity_cases == proof.cases
                || (proof.target_lattice_ppm >= 990_000 && proof.unique_top1_ppm > 950_000);
            let ambiguity_gate =
                proof.ambiguity_cases == 0 || proof.ambiguity_retention_ppm >= 990_000;
            target_gate && ambiguity_gate && proof.false_authority == 0
        })
}

impl SentenceContextScene {
    pub(crate) fn encoded_tokens(&self) -> Vec<String> {
        // Keep the sentence scene position-stable and bounded. Raw sentence
        // prefixes split one grammatical mode into unrelated phase centers;
        // local roles and endings carry the transferable evidence, while two
        // directional exact anchors retain strong lexical governors.
        let mut tokens = vec![SENTENCE_ANCHOR.to_string()];
        tokens.push(format!("{MARKER_PREFIX}slot_{}", self.position.id()));
        tokens.push(format!(
            "{MARKER_PREFIX}punct_before_{}",
            self.punctuation_before.id()
        ));
        tokens.push(format!(
            "{MARKER_PREFIX}punct_after_{}",
            self.punctuation_after.id()
        ));

        let left_0 = self.left.last().map(String::as_str);
        let left_1 = self.left.iter().rev().nth(1).map(String::as_str);
        let right_0 = self.right.first().map(String::as_str);
        append_neighbor_shape(&mut tokens, "left_0", left_0);
        append_neighbor_shape(&mut tokens, "left_1", left_1);
        append_neighbor_shape(&mut tokens, "right_0", right_0);

        if let Some(left) = left_0 {
            tokens.push(format!(
                "{MARKER_PREFIX}left_exact_0_{}",
                left.to_lowercase()
            ));
        }
        if let Some(left) = left_1 {
            tokens.push(format!(
                "{MARKER_PREFIX}left_exact_1_{}",
                left.to_lowercase()
            ));
        }
        if let Some(right) = right_0 {
            tokens.push(format!(
                "{MARKER_PREFIX}right_exact_0_{}",
                right.to_lowercase()
            ));
        }
        if let Some(right) = self.right.get(1) {
            tokens.push(format!(
                "{MARKER_PREFIX}right_exact_1_{}",
                right.to_lowercase()
            ));
        }
        tokens
    }

    pub(crate) fn pair_views(&self) -> Vec<Vec<String>> {
        let left_0 = self.left.last().map(String::as_str);
        let left_1 = self.left.iter().rev().nth(1).map(String::as_str);
        let right_0 = self.right.first().map(String::as_str);
        let right_1 = self.right.get(1).map(String::as_str);
        let base = || {
            vec![
                SENTENCE_ANCHOR.to_string(),
                format!("{MARKER_PREFIX}slot_{}", self.position.id()),
            ]
        };

        let mut punctuation = base();
        punctuation.push(format!(
            "{MARKER_PREFIX}punct_before_{}",
            self.punctuation_before.id()
        ));
        punctuation.push(format!(
            "{MARKER_PREFIX}punct_after_{}",
            self.punctuation_after.id()
        ));
        punctuation.push(format!("{MARKER_PREFIX}pair_view_punctuation"));

        let punctuation_joint = vec![
            format!(
                "{MARKER_PREFIX}punct_pair_joint_{}_{}",
                self.punctuation_before.id(),
                self.punctuation_after.id()
            ),
            format!("{MARKER_PREFIX}pair_view_punctuation_joint"),
        ];
        let punctuation_crystal = vec![
            format!(
                "{MARKER_PREFIX}punct_crystal_joint_{}_{}",
                self.punctuation_before.id(),
                self.punctuation_after.id()
            ),
            format!("{MARKER_PREFIX}pair_view_punctuation_crystal"),
        ];

        let mut immediate_left = base();
        append_neighbor_shape(&mut immediate_left, "left_0", left_0);
        append_left_exact(&mut immediate_left, 0, left_0);
        immediate_left.push(format!("{MARKER_PREFIX}pair_view_left_0"));

        let mut governing_left = base();
        append_neighbor_shape(&mut governing_left, "left_1", left_1);
        append_left_exact(&mut governing_left, 1, left_1);
        governing_left.push(format!("{MARKER_PREFIX}pair_view_left_1"));

        let mut immediate_right = base();
        append_neighbor_shape(&mut immediate_right, "right_0", right_0);
        immediate_right.push(format!("{MARKER_PREFIX}pair_view_right_0"));

        let mut following_right = base();
        append_neighbor_shape(&mut following_right, "right_1", right_1);
        following_right.push(format!("{MARKER_PREFIX}pair_view_right_1"));

        let mut bridge = base();
        append_neighbor_role_and_tail(&mut bridge, "left_0", left_0, 1);
        append_neighbor_role_and_tail(&mut bridge, "right_0", right_0, 1);
        append_neighbor_role_and_tail(&mut bridge, "right_1", right_1, 1);
        bridge.push(format!("{MARKER_PREFIX}pair_view_bridge"));

        let mut morphology = base();
        append_morphology_role(&mut morphology, "left_0", left_0);
        append_morphology_role(&mut morphology, "right_0", right_0);
        append_morphology_role(&mut morphology, "right_1", right_1);
        morphology.push(format!("{MARKER_PREFIX}pair_view_morphology"));

        let left_morphology = vec![
            format!(
                "{MARKER_PREFIX}left_0_morph_crystal_{}",
                left_0.map(token_morphology_class).unwrap_or("none")
            ),
            format!("{MARKER_PREFIX}pair_view_left_0_morphology"),
        ];

        let mut left_tail = base();
        append_neighbor_tail(&mut left_tail, "left_0", left_0);
        left_tail.push(format!("{MARKER_PREFIX}pair_view_left_0_tail"));

        let mut right_tail = base();
        append_neighbor_tail(&mut right_tail, "right_0", right_0);
        right_tail.push(format!("{MARKER_PREFIX}pair_view_right_0_tail"));

        let mut following_tail = base();
        append_neighbor_tail(&mut following_tail, "right_1", right_1);
        following_tail.push(format!("{MARKER_PREFIX}pair_view_right_1_tail"));

        let left_exact = vec![
            format!(
                "{MARKER_PREFIX}left_exact_crystal_{}",
                left_0
                    .map(str::to_lowercase)
                    .unwrap_or_else(|| "none".to_string())
            ),
            format!("{MARKER_PREFIX}pair_view_left_exact_crystal"),
        ];
        let right_exact = vec![
            format!(
                "{MARKER_PREFIX}right_exact_crystal_{}",
                right_0
                    .map(str::to_lowercase)
                    .unwrap_or_else(|| "none".to_string())
            ),
            format!("{MARKER_PREFIX}pair_view_right_exact_crystal"),
        ];

        let mut full = self.encoded_tokens();
        full.push(format!("{MARKER_PREFIX}pair_view_full"));

        vec![
            full,
            punctuation,
            punctuation_joint,
            punctuation_crystal,
            immediate_left,
            governing_left,
            immediate_right,
            following_right,
            bridge,
            morphology,
            left_morphology,
            left_tail,
            right_tail,
            following_tail,
            left_exact,
            right_exact,
        ]
    }

    pub(crate) fn direct_pair_view_indices(&self) -> Vec<usize> {
        let mut indices = Vec::new();
        if self.punctuation_before != PunctuationClass::None
            || self.punctuation_after != PunctuationClass::None
        {
            indices.extend(
                self.pair_views()
                    .into_iter()
                    .enumerate()
                    .filter_map(|(index, view)| {
                        view.iter()
                            .any(|token| token.contains("pair_view_punctuation"))
                            .then_some(index)
                    })
                    .collect::<Vec<_>>(),
            );
        } else {
            // Punctuation crystals and coarse morphology views transfer useful
            // ranking pressure, but they are correlated and cannot independently
            // ground sentence authority. Require agreement from views carrying an
            // actual neighbouring surface, tail, or cross-slot bridge.
            indices.push(0); // full scene
            if !self.left.is_empty() {
                indices.extend([4, 11]); // immediate left and its bounded tail
            }
            if self.left.len() >= 2 {
                indices.push(5); // governing left
            }
            if !self.right.is_empty() {
                indices.extend([6, 12]); // immediate right and its bounded tail
            }
            if self.right.len() >= 2 {
                indices.extend([7, 13]); // following right and its bounded tail
            }
            if !self.left.is_empty() && !self.right.is_empty() {
                indices.push(8); // cross-slot bridge
            }
        }
        indices.extend(self.required_anchor_pair_view_indices());
        indices.sort_unstable();
        indices.dedup();
        indices
    }

    pub(crate) fn required_anchor_pair_view_indices(&self) -> Vec<usize> {
        let mut indices = Vec::with_capacity(2);
        if !self.left.is_empty() {
            indices.push(PAIR_VIEW_LEFT_EXACT);
        }
        if !self.right.is_empty() {
            indices.push(PAIR_VIEW_RIGHT_EXACT);
        }
        indices
    }

    pub(crate) fn legacy_left_tokens(&self) -> &[String] {
        &self.left
    }

    pub(crate) fn has_directional_context(&self) -> bool {
        !self.left.is_empty()
            || !self.right.is_empty()
            || self.punctuation_before != PunctuationClass::None
            || self.punctuation_after != PunctuationClass::None
    }

    pub(crate) fn anchor_hash() -> u64 {
        super::context_exact_hash(SENTENCE_ANCHOR)
    }
}

pub(crate) fn is_sentence_marker(token: &str) -> bool {
    token.starts_with(MARKER_PREFIX)
}

pub(crate) fn is_sentence_structural_marker(token: &str) -> bool {
    is_sentence_marker(token)
        && !token.starts_with("__lay_l3_left_exact_")
        && !token.starts_with("__lay_l3_right_exact_")
}

pub(crate) fn project_candidate_lattice(
    original: &str,
    replacements: &[&str],
) -> Option<SentenceCandidateProjection> {
    let original = tokenize_surface(original);
    if original.tokens.is_empty() {
        return None;
    }
    let mut scene = None;
    let mut candidates = Vec::with_capacity(replacements.len());
    for replacement in replacements {
        let replacement = tokenize_surface(replacement);
        let Some((candidate_scene, candidate)) = project_candidate(&original, &replacement) else {
            candidates.push(None);
            continue;
        };
        if scene
            .as_ref()
            .is_some_and(|existing| existing != &candidate_scene)
        {
            candidates.push(None);
            continue;
        }
        scene.get_or_insert(candidate_scene);
        candidates.push(Some(candidate));
    }
    Some(SentenceCandidateProjection {
        scene: scene?,
        candidates,
    })
}

fn project_candidate(
    original: &TokenizedSurface,
    replacement: &TokenizedSurface,
) -> Option<(SentenceContextScene, String)> {
    if original.tokens.len() != replacement.tokens.len()
        || original.separators != replacement.separators
    {
        return None;
    }
    let changed = original
        .tokens
        .iter()
        .zip(&replacement.tokens)
        .enumerate()
        .filter_map(|(index, (left, right))| (left.text != right.text).then_some(index))
        .collect::<Vec<_>>();
    let [slot] = changed.as_slice() else {
        return None;
    };
    for (index, (left, right)) in original.tokens.iter().zip(&replacement.tokens).enumerate() {
        if index != *slot && left.text != right.text {
            return None;
        }
    }
    let left = original.tokens[..*slot]
        .iter()
        .map(|token| token.text.clone())
        .collect::<Vec<_>>();
    let right = original.tokens[*slot + 1..]
        .iter()
        .map(|token| token.text.clone())
        .collect::<Vec<_>>();
    let position = match (left.is_empty(), right.is_empty()) {
        (true, true) => SlotPosition::Only,
        (true, false) => SlotPosition::Start,
        (false, true) => SlotPosition::End,
        (false, false) => SlotPosition::Middle,
    };
    Some((
        SentenceContextScene {
            left,
            right,
            punctuation_before: PunctuationClass::from_separator(&original.separators[*slot]),
            punctuation_after: PunctuationClass::from_separator(&original.separators[*slot + 1]),
            position,
        },
        replacement.tokens[*slot].text.clone(),
    ))
}

fn tokenize_surface(text: &str) -> TokenizedSurface {
    let mut tokens = Vec::new();
    let mut token_start = None;
    for (index, ch) in text.char_indices() {
        if ch.is_alphanumeric() {
            token_start.get_or_insert(index);
        } else if let Some(start) = token_start.take() {
            tokens.push(SurfaceToken {
                text: text[start..index].to_string(),
                start,
                end: index,
            });
        }
    }
    if let Some(start) = token_start {
        tokens.push(SurfaceToken {
            text: text[start..].to_string(),
            start,
            end: text.len(),
        });
    }
    let mut separators = Vec::with_capacity(tokens.len() + 1);
    let mut cursor = 0;
    for token in &tokens {
        separators.push(text[cursor..token.start].to_string());
        cursor = token.end;
    }
    separators.push(text[cursor..].to_string());
    TokenizedSurface { tokens, separators }
}

fn role_marker(direction: &str, token: &str) -> String {
    format!("{MARKER_PREFIX}{direction}_role_{:04x}", token_role(token))
}

fn append_neighbor_shape(tokens: &mut Vec<String>, direction: &str, token: Option<&str>) {
    let Some(token) = token else {
        tokens.push(format!("{MARKER_PREFIX}{direction}_role_none"));
        tokens.push(format!("{MARKER_PREFIX}{direction}_tail_1_none"));
        tokens.push(format!("{MARKER_PREFIX}{direction}_tail_2_none"));
        return;
    };
    tokens.push(role_marker(direction, token));
    let lower = token.to_lowercase();
    tokens.push(format!(
        "{MARKER_PREFIX}{direction}_tail_1_{}",
        token_tail(&lower, 1)
    ));
    tokens.push(format!(
        "{MARKER_PREFIX}{direction}_tail_2_{}",
        token_tail(&lower, 2)
    ));
}

fn append_neighbor_role_and_tail(
    tokens: &mut Vec<String>,
    direction: &str,
    token: Option<&str>,
    tail_width: usize,
) {
    let Some(token) = token else {
        tokens.push(format!("{MARKER_PREFIX}{direction}_role_none"));
        tokens.push(format!("{MARKER_PREFIX}{direction}_tail_{tail_width}_none"));
        return;
    };
    tokens.push(role_marker(direction, token));
    tokens.push(format!(
        "{MARKER_PREFIX}{direction}_tail_{tail_width}_{}",
        token_tail(&token.to_lowercase(), tail_width)
    ));
}

fn append_neighbor_tail(tokens: &mut Vec<String>, direction: &str, token: Option<&str>) {
    let lower = token.map(str::to_lowercase);
    for width in [1, 2] {
        tokens.push(format!(
            "{MARKER_PREFIX}{direction}_tail_only_{width}_{}",
            lower
                .as_deref()
                .map(|token| token_tail(token, width))
                .unwrap_or_else(|| "none".to_string())
        ));
    }
}

fn append_morphology_role(tokens: &mut Vec<String>, direction: &str, token: Option<&str>) {
    tokens.push(format!(
        "{MARKER_PREFIX}{direction}_morph_{}",
        token.map(token_morphology_class).unwrap_or("none")
    ));
}

fn token_morphology_class(token: &str) -> &'static str {
    let lower = token.to_lowercase();
    if lower.chars().all(|ch| ch.is_ascii_digit()) {
        return "number";
    }
    if lower.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return if crate::word_recognizer::is_ascii_titlecase_token(token)
            || crate::word_recognizer::is_ascii_technical_or_brand_token(token)
        {
            "latin_entity"
        } else {
            "latin_word"
        };
    }
    if !lower.chars().all(crate::keyboard::is_cyrillic_letter) {
        return "mixed";
    }
    if crate::lexicon::is_ru_short_preposition(&lower)
        || crate::phrase_lexicon::is_short_russian_function_word(&lower)
    {
        return "ru_function";
    }
    if lower.ends_with("ть") || lower.ends_with("ти") || lower.ends_with("чь") {
        return "ru_infinitive";
    }
    const FINITE_VERB_TAILS: &[&str] = &[
        "ешь", "ишь", "ете", "ите", "ает", "яет", "ует", "ют", "ут", "ат", "ят", "ем", "им", "ет",
        "ит",
    ];
    if FINITE_VERB_TAILS.iter().any(|tail| lower.ends_with(tail)) {
        return "ru_finite_verb";
    }
    if ["ые", "ие", "ых", "их", "ыми", "ими"]
        .iter()
        .any(|tail| lower.ends_with(tail))
    {
        return "ru_plural_modifier";
    }
    if ["ый", "ий", "ой", "ая", "яя", "ое", "ее"]
        .iter()
        .any(|tail| lower.ends_with(tail))
    {
        return "ru_singular_modifier";
    }
    if lower.chars().last().is_some_and(|ch| {
        matches!(
            ch,
            'б' | 'в'
                | 'г'
                | 'д'
                | 'ж'
                | 'з'
                | 'й'
                | 'к'
                | 'л'
                | 'м'
                | 'н'
                | 'п'
                | 'р'
                | 'с'
                | 'т'
                | 'ф'
                | 'х'
                | 'ц'
                | 'ч'
                | 'ш'
                | 'щ'
        )
    }) {
        "ru_consonant"
    } else {
        "ru_vowel"
    }
}

fn append_left_exact(tokens: &mut Vec<String>, index: usize, token: Option<&str>) {
    if let Some(token) = token {
        tokens.push(format!(
            "{MARKER_PREFIX}left_exact_{index}_{}",
            token.to_lowercase()
        ));
    }
}

fn token_tail(token: &str, width: usize) -> String {
    let mut tail = token.chars().rev().take(width).collect::<Vec<_>>();
    tail.reverse();
    if tail.is_empty() {
        "none".to_string()
    } else {
        tail.into_iter().collect()
    }
}

fn token_role(token: &str) -> u16 {
    let lower = token.to_lowercase();
    let script = if lower.chars().all(crate::keyboard::is_cyrillic_letter) {
        1_u64
    } else if lower.chars().all(|ch| ch.is_ascii_alphabetic()) {
        2
    } else if lower.chars().all(|ch| ch.is_ascii_digit()) {
        3
    } else {
        4
    };
    let length = match lower.chars().count() {
        0 => 0,
        1 => 1,
        2..=3 => 2,
        4..=7 => 3,
        _ => 4,
    };
    let functional = u64::from(
        crate::lexicon::is_ru_short_preposition(&lower)
            || crate::phrase_lexicon::is_short_russian_function_word(&lower),
    );
    (mix64_golden(script | (length << 4) | (functional << 8)) & 0xffff) as u16
}

fn parse_cases(text: &str) -> io::Result<Vec<SentenceProofCase>> {
    let mut cases = Vec::new();
    for (line_number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 5 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "sentence proof line {} must have 5 TSV columns",
                    line_number + 1
                ),
            ));
        }
        let candidates = columns[3]
            .split('|')
            .map(str::trim)
            .filter(|candidate| !candidate.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        if candidates.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "sentence proof line {} needs at least two candidates",
                    line_number + 1
                ),
            ));
        }
        cases.push(SentenceProofCase {
            split: columns[0].to_string(),
            class: columns[1].to_string(),
            original: columns[2].to_string(),
            candidates,
            expected: (columns[4] != "-").then(|| columns[4].to_string()),
        });
    }
    Ok(cases)
}

fn ratio_ppm(numerator: usize, denominator: usize) -> u32 {
    if denominator == 0 {
        return 1_000_000;
    }
    ((numerator as u64 * 1_000_000) / denominator as u64).min(u64::from(u32::MAX)) as u32
}

fn push_failure(
    failures: &mut Vec<serde_json::Value>,
    case: &SentenceProofCase,
    candidates: &[&str],
    readouts: &[super::ContextPhaseReadout],
    package: &super::ContextPhasePackage,
    scene: &SentenceContextScene,
) {
    if failures.len() >= 64 {
        return;
    }
    failures.push(serde_json::json!({
        "class": case.class,
        "original": case.original,
        "expected": case.expected,
        "pair_views": package.sentence_pair_debug(scene, candidates),
        "candidates": candidates.iter().zip(readouts).map(|(candidate, readout)| serde_json::json!({
            "candidate": candidate,
            "disposition": format!("{:?}", readout.disposition),
            "margin_micro": readout.margin_micro,
            "competition_margin_micro": readout.competition_margin_micro,
            "threshold_micro": readout.threshold_micro,
            "positive_examples": readout.positive_examples,
            "positive_center_support": readout.positive_center_support,
            "context_tokens": readout.context_tokens,
            "context_known_tokens": readout.context_known_tokens,
            "pairwise_blocked": readout.pairwise_blocked,
            "pairwise_conflict": readout.pairwise_conflict,
            "pairwise_certified": readout.pairwise_certified,
        })).collect::<Vec<_>>(),
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::context_phase::online::{
        OnlineContextPhaseConfig, OnlineContextPhaseLearner,
    };
    use crate::nanda_wave::context_phase::{ContextPhaseDisposition, SurfaceMutationField};
    use std::sync::Arc;

    #[test]
    fn projection_retains_right_context_order_and_punctuation() {
        let projection = project_candidate_lattice(
            "мы пишем код, сегодня",
            &["мы пишем кода, сегодня", "мы пишем коду, сегодня"],
        )
        .unwrap();

        assert_eq!(
            projection.candidates,
            [Some("кода".to_string()), Some("коду".to_string())]
        );
        let encoded = projection.scene.encoded_tokens();
        assert!(encoded.iter().any(|token| token == SENTENCE_ANCHOR));
        assert!(encoded
            .iter()
            .any(|token| token == "__lay_l3_punct_after_1"));
        assert!(encoded
            .iter()
            .any(|token| token == "__lay_l3_right_0_tail_2_ня"));
    }

    #[test]
    fn candidate_that_changes_surrounding_context_is_excluded() {
        let projection = project_candidate_lattice(
            "мы пишем код сегодня",
            &["мы пишем кода сегодня", "они пишут коду сегодня"],
        )
        .unwrap();

        assert_eq!(projection.candidates[0].as_deref(), Some("кода"));
        assert_eq!(projection.candidates[1], None);
    }

    #[test]
    fn right_context_direction_changes_the_scene() {
        let first = project_candidate_lattice("мы пишем код", &["мы пишем кода"])
            .unwrap()
            .scene
            .encoded_tokens();
        let second = project_candidate_lattice("код пишем мы", &["кода пишем мы"])
            .unwrap()
            .scene
            .encoded_tokens();

        assert_ne!(first, second);
    }

    #[test]
    fn supervised_relation_delta_emits_directional_pair_evidence() {
        let base = super::super::ContextPhasePackage {
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
            ..super::super::ContextPhasePackage::default()
        };
        let scenes = vec![
            "нам нужно посмотреть".to_string(),
            "здесь пора посмотреть".to_string(),
        ];
        let (package, report) = compile_supervised_relation_delta(
            &base,
            &scenes,
            "посмотреть",
            &["посмотри".to_string()],
            2,
            super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
        )
        .unwrap();

        assert_eq!(report["corpus_passes"], 1);
        assert_eq!(report["distinct_scenes"], 2);
        assert!(report["emitted_exact_pair_profiles"].as_u64().unwrap() > 0);
        let projection = project_candidate_lattice(
            "нам нужно посмот",
            &["нам нужно посмотреть", "нам нужно посмотри"],
        )
        .unwrap();
        let candidates = projection
            .candidates
            .iter()
            .filter_map(Option::as_deref)
            .collect::<Vec<_>>();
        let readouts = package.score_sentence_candidates(&projection.scene, &candidates);
        assert!(readouts[0].pairwise_known_edges > 0, "{readouts:#?}");
        assert!(readouts[0].pairwise_certified, "{readouts:#?}");
        assert_eq!(
            readouts[0].disposition,
            ContextPhaseDisposition::Support,
            "{readouts:#?}"
        );
        assert_ne!(
            readouts[1].disposition,
            ContextPhaseDisposition::Support,
            "{readouts:#?}"
        );
    }

    #[test]
    fn supervised_sentence_field_settles_opposite_modes_of_one_candidate_pair() {
        let mut learner = OnlineContextPhaseLearner::new_with_surface_field(
            OnlineContextPhaseConfig::production(2),
            Arc::new(SurfaceMutationField::default()),
        );
        for _ in 0..2 {
            for prefix in ["сейчас нужно", "нам нужно", "здесь нужно", "всем нужно"]
            {
                ingest_case(
                    &mut learner,
                    &format!("{prefix} посмот код"),
                    &format!("{prefix} посмотреть код"),
                    &[&format!("{prefix} посмотри код")],
                );
            }
            for prefix in ["ты быстро", "ты срочно", "ты сразу", "ты внимательно"]
            {
                ingest_case(
                    &mut learner,
                    &format!("{prefix} посмот код"),
                    &format!("{prefix} посмотри код"),
                    &[&format!("{prefix} посмотреть код")],
                );
            }
        }
        let package = learner.snapshot();

        for (original, expected, other) in [
            (
                "завтра нужно посмот код",
                "завтра нужно посмотреть код",
                "завтра нужно посмотри код",
            ),
            (
                "ты срочно посмот код",
                "ты срочно посмотри код",
                "ты срочно посмотреть код",
            ),
        ] {
            let projection = project_candidate_lattice(original, &[expected, other]).unwrap();
            let candidate_tokens = projection
                .candidates
                .iter()
                .filter_map(Option::as_deref)
                .collect::<Vec<_>>();
            let pair_debug = package.sentence_pair_debug(&projection.scene, &candidate_tokens);
            let readouts = super::super::readout_candidates_with_package(
                &package,
                original,
                &[expected, other],
            );
            assert_eq!(
                readouts[0].disposition,
                ContextPhaseDisposition::Support,
                "{readouts:#?}\n{pair_debug:#}"
            );
            assert!(readouts[0].pairwise_certified);
            assert_ne!(readouts[1].disposition, ContextPhaseDisposition::Support);
        }
    }

    fn ingest_case(
        learner: &mut OnlineContextPhaseLearner,
        original: &str,
        expected: &str,
        competitors: &[&str],
    ) {
        let mut replacements = Vec::with_capacity(competitors.len() + 1);
        replacements.push(expected);
        replacements.extend_from_slice(competitors);
        let projection = project_candidate_lattice(original, &replacements).unwrap();
        let mut candidates = projection.candidates.into_iter();
        let target = candidates.next().flatten().unwrap();
        let competitors = candidates.flatten().collect::<Vec<_>>();
        learner.ingest_supervised_sentence(
            &projection.scene.encoded_tokens(),
            &projection.scene.pair_views(),
            &target,
            &competitors,
        );
    }
}
