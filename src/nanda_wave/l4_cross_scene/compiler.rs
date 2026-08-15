use std::collections::{BTreeMap, BTreeSet};

use super::encoder::encode_scene;
use super::format::{canonical_runtime_package, encode_package};
use super::model::{
    CrossSceneCompileReport, L4CrossSceneObservation, L4CrossScenePackage, L4CrossScenePairProfile,
    L4CrossSceneProfile, L4CrossSceneProfileKey,
};
use super::{
    MAX_AMBIGUITY_CENTERS_PER_BANK, MAX_CENTERS_PER_BANK, MAX_HARD_CENTERS_PER_BANK,
    MAX_PAIR_PROFILES, MAX_PROFILES, SPLIT_COHERENCE,
};
use crate::nanda_wave::phase_field::{add_cluster, max_coherence, phase_micro, PhaseCell};
use crate::typing_memory::TypingMemoryOutcome;

#[derive(Clone, Copy, Debug)]
pub(crate) struct CrossSceneCompileConfig {
    pub(crate) max_profiles: usize,
    pub(crate) max_pair_profiles: usize,
    pub(crate) include_anti_centers: bool,
}

impl Default for CrossSceneCompileConfig {
    fn default() -> Self {
        Self {
            max_profiles: MAX_PROFILES,
            max_pair_profiles: MAX_PAIR_PROFILES,
            include_anti_centers: true,
        }
    }
}

#[derive(Clone, Debug)]
struct ConsolidatedScene {
    key: L4CrossSceneProfileKey,
    family_fingerprint: u64,
    vector: Vec<PhaseCell>,
    candidate_relation_id: u64,
    keep_relation_id: u64,
    positive: u32,
    negative: u32,
    reverted: u32,
    ambiguity: u32,
    censored: u32,
}

impl ConsolidatedScene {
    fn conflicting(&self) -> bool {
        let candidate_wins = self.positive > 0;
        let candidate_loses = self.negative > 0 || self.reverted > 0;
        self.ambiguity > 0 || (candidate_wins && candidate_loses)
    }

    fn total(&self) -> u32 {
        self.positive
            .saturating_add(self.negative)
            .saturating_add(self.reverted)
            .saturating_add(self.ambiguity)
            .saturating_add(self.censored)
    }
}

pub(crate) fn compile_observations(
    observations: &[L4CrossSceneObservation],
    config: CrossSceneCompileConfig,
) -> (L4CrossScenePackage, CrossSceneCompileReport) {
    let mut report = CrossSceneCompileReport {
        source_observations: observations.len().min(u32::MAX as usize) as u32,
        ..CrossSceneCompileReport::default()
    };
    let mut seen = BTreeSet::new();
    let mut scenes = BTreeMap::<(L4CrossSceneProfileKey, u64, u64, u64), ConsolidatedScene>::new();

    for observation in observations {
        if !observation.complete_chain {
            report.orphan_observations = report.orphan_observations.saturating_add(1);
            continue;
        }
        let encoded = encode_scene(observation.input());
        let mut family_input = observation.input();
        family_input.candidate_relation_id = 0;
        family_input.keep_relation_id = 0;
        let family_fingerprint = encode_scene(family_input).fingerprint;
        let dedup_key = (
            observation.receipt_id,
            encoded.fingerprint,
            encoded.candidate_relation_id,
            observation.outcome.code(),
        );
        if !seen.insert(dedup_key) {
            continue;
        }
        report.joined_observations = report.joined_observations.saturating_add(1);
        match observation.outcome {
            TypingMemoryOutcome::ConfirmedPositive => {
                report.positive_observations = report.positive_observations.saturating_add(1)
            }
            TypingMemoryOutcome::ConfirmedNegative => {
                report.negative_observations = report.negative_observations.saturating_add(1)
            }
            TypingMemoryOutcome::Reverted => {
                report.reverted_observations = report.reverted_observations.saturating_add(1)
            }
            TypingMemoryOutcome::Ambiguous => {
                report.ambiguity_observations = report.ambiguity_observations.saturating_add(1)
            }
            TypingMemoryOutcome::Censored => {
                report.censored_observations = report.censored_observations.saturating_add(1)
            }
        }
        let scene = scenes
            .entry((
                observation.profile,
                encoded.fingerprint,
                encoded.candidate_relation_id,
                encoded.keep_relation_id,
            ))
            .or_insert_with(|| ConsolidatedScene {
                key: observation.profile,
                family_fingerprint,
                vector: encoded.vector,
                candidate_relation_id: encoded.candidate_relation_id,
                keep_relation_id: encoded.keep_relation_id,
                positive: 0,
                negative: 0,
                reverted: 0,
                ambiguity: 0,
                censored: 0,
            });
        match observation.outcome {
            TypingMemoryOutcome::ConfirmedPositive => {
                scene.positive = scene.positive.saturating_add(1)
            }
            TypingMemoryOutcome::ConfirmedNegative => {
                scene.negative = scene.negative.saturating_add(1)
            }
            TypingMemoryOutcome::Reverted => scene.reverted = scene.reverted.saturating_add(1),
            TypingMemoryOutcome::Ambiguous => scene.ambiguity = scene.ambiguity.saturating_add(1),
            TypingMemoryOutcome::Censored => scene.censored = scene.censored.saturating_add(1),
        }
    }

    let cyclic_pairs = cyclic_scene_pairs(scenes.values());
    report.consolidated_scenes = scenes.len().min(u32::MAX as usize) as u32;
    report.conflict_scenes = scenes
        .values()
        .filter(|scene| {
            scene.conflicting()
                || cyclic_pairs.contains(&(
                    scene.key,
                    scene.family_fingerprint,
                    scene.candidate_relation_id.min(scene.keep_relation_id),
                    scene.candidate_relation_id.max(scene.keep_relation_id),
                ))
        })
        .count()
        .min(u32::MAX as usize) as u32;

    let mut profiles = BTreeMap::<L4CrossSceneProfileKey, L4CrossSceneProfile>::new();
    let mut pairs = BTreeMap::<(L4CrossSceneProfileKey, u64, u64), L4CrossScenePairProfile>::new();
    for scene in scenes.values() {
        if !profiles.contains_key(&scene.key) && profiles.len() >= config.max_profiles {
            continue;
        }
        let profile = profiles
            .entry(scene.key)
            .or_insert_with(|| empty_profile(scene.key));
        profile.positive_examples = profile.positive_examples.saturating_add(scene.positive);
        profile.negative_examples = profile.negative_examples.saturating_add(scene.negative);
        profile.reverted_examples = profile.reverted_examples.saturating_add(scene.reverted);
        profile.ambiguity_examples = profile.ambiguity_examples.saturating_add(scene.ambiguity);
        profile.censored_examples = profile.censored_examples.saturating_add(scene.censored);

        let low = scene.candidate_relation_id.min(scene.keep_relation_id);
        let high = scene.candidate_relation_id.max(scene.keep_relation_id);
        let conflict = scene.conflicting()
            || cyclic_pairs.contains(&(scene.key, scene.family_fingerprint, low, high));
        if conflict {
            observe(
                &mut profile.ambiguity,
                &scene.vector,
                scene.total(),
                MAX_AMBIGUITY_CENTERS_PER_BANK,
            );
        } else if scene.positive > 0 {
            observe(
                &mut profile.positive,
                &scene.vector,
                scene.positive,
                MAX_CENTERS_PER_BANK,
            );
        } else if config.include_anti_centers && scene.reverted > 0 {
            observe(
                &mut profile.hard_negative,
                &scene.vector,
                scene.reverted,
                MAX_HARD_CENTERS_PER_BANK,
            );
        } else if config.include_anti_centers && scene.negative > 0 {
            observe(
                &mut profile.negative,
                &scene.vector,
                scene.negative,
                MAX_CENTERS_PER_BANK,
            );
        }

        if scene.candidate_relation_id == scene.keep_relation_id {
            continue;
        }
        let pair_key = (scene.key, low, high);
        if !pairs.contains_key(&pair_key) && pairs.len() >= config.max_pair_profiles {
            continue;
        }
        let pair = pairs
            .entry(pair_key)
            .or_insert_with(|| empty_pair(scene.key, low, high));
        pair.observations = pair.observations.saturating_add(scene.total());
        if conflict {
            observe(
                &mut pair.ambiguity,
                &scene.vector,
                scene.total(),
                MAX_AMBIGUITY_CENTERS_PER_BANK,
            );
            continue;
        }
        let candidate_is_low = scene.candidate_relation_id == low;
        if scene.positive > 0 {
            let bank = if candidate_is_low {
                &mut pair.low_wins
            } else {
                &mut pair.high_wins
            };
            observe(bank, &scene.vector, scene.positive, MAX_CENTERS_PER_BANK);
        }
        if config.include_anti_centers && scene.negative > 0 {
            let bank = if candidate_is_low {
                &mut pair.high_wins
            } else {
                &mut pair.low_wins
            };
            observe(bank, &scene.vector, scene.negative, MAX_CENTERS_PER_BANK);
        }
        if config.include_anti_centers && scene.reverted > 0 {
            let bank = if candidate_is_low {
                &mut pair.hard_high_wins
            } else {
                &mut pair.hard_low_wins
            };
            observe(
                bank,
                &scene.vector,
                scene.reverted,
                MAX_HARD_CENTERS_PER_BANK,
            );
        }
    }

    let mut package = L4CrossScenePackage {
        encoder_version: super::ENCODER_VERSION,
        encoder_hash: super::ENCODER_HASH,
        applied_segment: 0,
        symbols: observations
            .iter()
            .flat_map(|observation| observation.scene_symbols.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        profiles: profiles.into_values().collect(),
        pair_profiles: pairs.into_values().collect(),
        source_observations: report.source_observations,
        joined_observations: report.joined_observations,
        positive_observations: report.positive_observations,
        negative_observations: report.negative_observations,
        reverted_observations: report.reverted_observations,
        ambiguity_observations: report.ambiguity_observations,
        censored_observations: report.censored_observations,
    };
    package = canonical_runtime_package(&package)
        .expect("bounded L4 cross-scene package must canonicalize");

    for profile in &mut package.profiles {
        let key = profile.key;
        let relevant = scenes.values().filter(|scene| scene.key == key);
        profile.threshold_micro = calibrate_profile_threshold(profile, relevant);
    }
    for pair in &mut package.pair_profiles {
        let key = pair.key;
        let low = pair.low_relation;
        let high = pair.high_relation;
        let relevant = scenes.values().filter(|scene| {
            scene.key == key
                && scene.candidate_relation_id.min(scene.keep_relation_id) == low
                && scene.candidate_relation_id.max(scene.keep_relation_id) == high
        });
        pair.threshold_micro = calibrate_pair_threshold(pair, relevant);
    }
    report.profiles = package.profiles.len().min(u32::MAX as usize) as u32;
    report.pair_profiles = package.pair_profiles.len().min(u32::MAX as usize) as u32;
    report.symbols = package.symbols.len().min(u32::MAX as usize) as u32;
    report.logical_center_bytes = encode_package(&package).len() as u64;
    report.raw_text_stored = false;
    report.runtime_authority_changed = false;
    (package, report)
}

fn empty_profile(key: L4CrossSceneProfileKey) -> L4CrossSceneProfile {
    L4CrossSceneProfile {
        key,
        threshold_micro: 0,
        positive: Vec::new(),
        negative: Vec::new(),
        hard_negative: Vec::new(),
        ambiguity: Vec::new(),
        positive_examples: 0,
        negative_examples: 0,
        reverted_examples: 0,
        ambiguity_examples: 0,
        censored_examples: 0,
    }
}

fn empty_pair(
    key: L4CrossSceneProfileKey,
    low_relation: u64,
    high_relation: u64,
) -> L4CrossScenePairProfile {
    L4CrossScenePairProfile {
        key,
        low_relation,
        high_relation,
        threshold_micro: 0,
        low_wins: Vec::new(),
        high_wins: Vec::new(),
        hard_low_wins: Vec::new(),
        hard_high_wins: Vec::new(),
        ambiguity: Vec::new(),
        observations: 0,
    }
}

fn observe(
    centers: &mut Vec<crate::nanda_wave::phase_field::PhaseCenter>,
    vector: &[PhaseCell],
    count: u32,
    maximum: usize,
) {
    for _ in 0..count.clamp(1, 16) {
        add_cluster(centers, vector, maximum, SPLIT_COHERENCE);
    }
}

fn calibrate_profile_threshold<'a>(
    profile: &L4CrossSceneProfile,
    scenes: impl Iterator<Item = &'a ConsolidatedScene>,
) -> i32 {
    let mut correct = Vec::new();
    for scene in scenes.filter(|scene| !scene.conflicting()) {
        let positive = max_coherence(&scene.vector, &profile.positive).unwrap_or_default();
        let negative = max_coherence(&scene.vector, &profile.negative).unwrap_or_default();
        let hard = max_coherence(&scene.vector, &profile.hard_negative).unwrap_or_default();
        if scene.positive > 0 {
            correct.push(positive - negative.max(hard));
        } else if scene.negative > 0 || scene.reverted > 0 {
            correct.push(negative.max(hard) - positive);
        }
    }
    evidence_threshold(&mut correct)
}

fn calibrate_pair_threshold<'a>(
    pair: &L4CrossScenePairProfile,
    scenes: impl Iterator<Item = &'a ConsolidatedScene>,
) -> i32 {
    let mut correct = Vec::new();
    for scene in scenes.filter(|scene| !scene.conflicting()) {
        let low = max_coherence(&scene.vector, &pair.low_wins).unwrap_or_default();
        let high = max_coherence(&scene.vector, &pair.high_wins).unwrap_or_default();
        let hard_low = max_coherence(&scene.vector, &pair.hard_low_wins).unwrap_or_default();
        let hard_high = max_coherence(&scene.vector, &pair.hard_high_wins).unwrap_or_default();
        let low_score = low.max(hard_low);
        let high_score = high.max(hard_high);
        let candidate_is_low = scene.candidate_relation_id == pair.low_relation;
        let candidate_wins = scene.positive > 0;
        let expected_low = candidate_is_low == candidate_wins;
        correct.push(if expected_low {
            low_score - high_score
        } else {
            high_score - low_score
        });
    }
    evidence_threshold(&mut correct)
}

fn evidence_threshold(values: &mut [f32]) -> i32 {
    if values.is_empty() {
        return phase_micro(0.18) as i32;
    }
    values.sort_by(f32::total_cmp);
    let lower_decile = values[values.len().saturating_sub(1) / 10].max(0.0);
    phase_micro((lower_decile * 0.45).clamp(0.04, 0.35)) as i32
}

fn cyclic_scene_pairs<'a>(
    scenes: impl Iterator<Item = &'a ConsolidatedScene>,
) -> BTreeSet<(L4CrossSceneProfileKey, u64, u64, u64)> {
    let mut graphs = BTreeMap::<(L4CrossSceneProfileKey, u64), BTreeMap<u64, BTreeSet<u64>>>::new();
    for scene in scenes.filter(|scene| !scene.conflicting()) {
        let candidate_wins = scene.positive > 0;
        let candidate_loses = scene.negative > 0 || scene.reverted > 0;
        if candidate_wins == candidate_loses
            || scene.candidate_relation_id == scene.keep_relation_id
        {
            continue;
        }
        let (winner, loser) = if candidate_wins {
            (scene.candidate_relation_id, scene.keep_relation_id)
        } else {
            (scene.keep_relation_id, scene.candidate_relation_id)
        };
        graphs
            .entry((scene.key, scene.family_fingerprint))
            .or_default()
            .entry(winner)
            .or_default()
            .insert(loser);
    }

    let mut cyclic = BTreeSet::new();
    for ((profile, family), graph) in graphs {
        for (&winner, losers) in &graph {
            for &loser in losers {
                if path_exists(&graph, loser, winner) {
                    cyclic.insert((profile, family, winner.min(loser), winner.max(loser)));
                }
            }
        }
    }
    cyclic
}

fn path_exists(graph: &BTreeMap<u64, BTreeSet<u64>>, start: u64, target: u64) -> bool {
    let mut frontier = vec![start];
    let mut visited = BTreeSet::new();
    while let Some(node) = frontier.pop() {
        if node == target {
            return true;
        }
        if !visited.insert(node) {
            continue;
        }
        if let Some(next) = graph.get(&node) {
            frontier.extend(next.iter().copied());
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::super::encoder::{
        candidate_relation_id, keep_relation_id, relation_class_from_context,
    };
    use super::*;
    use crate::nanda_wave::l4_cross_scene::model::L4CrossSceneL2Signal;
    use crate::transition_relation::{TransitionOperatorKind, TransitionRelationAtoms};
    use crate::typing_memory::{
        LayoutProjectionDirection, LayoutProjectionScope, TypingTransitionIdentity,
    };
    use crate::typing_scene::SentenceLanguageEvidence;

    fn observation(outcome: TypingMemoryOutcome, receipt_id: u64) -> L4CrossSceneObservation {
        let from = "ghbdtn";
        let to = "привет";
        let relation = TransitionRelationAtoms::for_operator(
            from,
            to,
            TransitionOperatorKind::LayoutProjection,
        );
        let context = vec!["мы".to_string(), "пишем".to_string()];
        let identity = TypingTransitionIdentity::observed(from, to, "replacement");
        let sentence_language = SentenceLanguageEvidence::script_only(&context, to);
        L4CrossSceneObservation {
            receipt_id,
            complete_chain: true,
            profile: L4CrossSceneProfileKey::new(
                TransitionOperatorKind::LayoutProjection,
                Some(LayoutProjectionDirection::EnToRu),
                Some(LayoutProjectionScope::CurrentToken),
            )
            .with_scene(identity.scene, sentence_language),
            context: context.clone(),
            from_text: from.to_string(),
            to_text: to.to_string(),
            relation_atoms: relation.atoms().to_vec(),
            candidate_relation_id: candidate_relation_id(relation.atoms()),
            keep_relation_id: keep_relation_id(),
            l3_relation_class: relation_class_from_context(&context, to),
            context_signal: super::super::model::L4CrossSceneContextSignal::Support,
            l2_signal: L4CrossSceneL2Signal::Support,
            sentence_language,
            scene_symbols: identity.scene.known_symbols(),
            outcome,
        }
    }

    #[test]
    fn causal_only_compiler_excludes_orphans_and_censored_centers() {
        let mut orphan = observation(TypingMemoryOutcome::ConfirmedPositive, 2);
        orphan.complete_chain = false;
        let observations = vec![
            observation(TypingMemoryOutcome::ConfirmedPositive, 1),
            observation(TypingMemoryOutcome::Censored, 3),
            orphan,
        ];
        let (package, report) = compile_observations(&observations, Default::default());

        assert_eq!(report.joined_observations, 2);
        assert_eq!(report.orphan_observations, 1);
        assert_eq!(package.profiles[0].positive_examples, 1);
        assert_eq!(package.profiles[0].censored_examples, 1);
        assert_eq!(package.profiles[0].positive.len(), 1);
    }

    #[test]
    fn contradictory_receipts_form_ambiguity_instead_of_authority() {
        let observations = vec![
            observation(TypingMemoryOutcome::ConfirmedPositive, 1),
            observation(TypingMemoryOutcome::Reverted, 2),
        ];
        let (package, report) = compile_observations(&observations, Default::default());

        assert_eq!(report.conflict_scenes, 1);
        assert!(package.profiles[0].positive.is_empty());
        assert!(package.profiles[0].hard_negative.is_empty());
        assert_eq!(package.profiles[0].ambiguity.len(), 1);
    }

    #[test]
    fn directed_relation_cycle_is_quarantined_as_ambiguity() {
        let mut a_over_b = observation(TypingMemoryOutcome::ConfirmedPositive, 1);
        a_over_b.candidate_relation_id = 10;
        a_over_b.keep_relation_id = 20;
        let mut b_over_c = observation(TypingMemoryOutcome::ConfirmedPositive, 2);
        b_over_c.candidate_relation_id = 20;
        b_over_c.keep_relation_id = 30;
        let mut c_over_a = observation(TypingMemoryOutcome::ConfirmedPositive, 3);
        c_over_a.candidate_relation_id = 30;
        c_over_a.keep_relation_id = 10;

        let (package, report) = compile_observations(
            &[a_over_b, b_over_c, c_over_a],
            CrossSceneCompileConfig::default(),
        );

        assert_eq!(report.conflict_scenes, 3);
        assert!(package.profiles[0].positive.is_empty());
        assert!(!package.profiles[0].ambiguity.is_empty());
        assert!(package
            .pair_profiles
            .iter()
            .all(|pair| !pair.ambiguity.is_empty()));
    }

    #[test]
    fn compiled_package_is_already_runtime_canonical() {
        let observations = vec![
            observation(TypingMemoryOutcome::ConfirmedPositive, 1),
            observation(TypingMemoryOutcome::ConfirmedNegative, 2),
        ];
        let (package, _) = compile_observations(&observations, Default::default());

        let first = encode_package(&package);
        let restored = canonical_runtime_package(&package).expect("canonical package roundtrip");
        assert_eq!(first, encode_package(&restored));
    }
}
