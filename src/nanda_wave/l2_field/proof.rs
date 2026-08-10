use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io;
use std::path::Path;
use std::time::Instant;

use super::runtime::{L2LexicalSeed, L2LexicalSeedOrigin, L2LocalVerdict, StandaloneL2Field};
use super::teacher::{L2TeacherCorpus, TeacherScene};

#[derive(Default)]
struct SlotScore {
    total: usize,
    winner_correct: usize,
    tied_contains: usize,
    abstain: usize,
    false_authority: usize,
}

impl SlotScore {
    fn merge(&mut self, other: Self) {
        self.total += other.total;
        self.winner_correct += other.winner_correct;
        self.tied_contains += other.tied_contains;
        self.abstain += other.abstain;
        self.false_authority += other.false_authority;
    }
}

#[derive(Default)]
struct ProofShard {
    aggregate: SlotScore,
    per_slot: BTreeMap<u32, SlotScore>,
    per_pos: BTreeMap<u16, SlotScore>,
    unresolved_per_pos: BTreeMap<u16, usize>,
    latency_us: Vec<u64>,
    unresolved: usize,
    evaluated: usize,
    failure_examples: Vec<serde_json::Value>,
    coverage_failure_examples: Vec<serde_json::Value>,
    false_authority_examples: Vec<serde_json::Value>,
}

impl ProofShard {
    fn merge(&mut self, other: Self) {
        self.aggregate.merge(other.aggregate);
        for (feature_mask, score) in other.per_slot {
            self.per_slot.entry(feature_mask).or_default().merge(score);
        }
        for (pos, score) in other.per_pos {
            self.per_pos.entry(pos).or_default().merge(score);
        }
        for (pos, unresolved) in other.unresolved_per_pos {
            *self.unresolved_per_pos.entry(pos).or_default() += unresolved;
        }
        self.latency_us.extend(other.latency_us);
        self.unresolved += other.unresolved;
        self.evaluated += other.evaluated;
        self.failure_examples.extend(other.failure_examples);
        self.failure_examples.truncate(32);
        self.coverage_failure_examples
            .extend(other.coverage_failure_examples);
        self.coverage_failure_examples.truncate(32);
        self.false_authority_examples
            .extend(other.false_authority_examples);
        self.false_authority_examples.truncate(32);
    }
}

pub(crate) fn prove_package(
    l1_package_path: &Path,
    l2_package_path: &Path,
    morphology_corpus_path: &Path,
    limit: usize,
) -> io::Result<serde_json::Value> {
    let cold_started = Instant::now();
    let field = StandaloneL2Field::load(l2_package_path).map_err(io::Error::other)?;
    let cold_load_us = cold_started.elapsed().as_micros() as u64;
    let l1 = crate::nanda_wave::L1RestorationHost::load(l1_package_path)?;
    let l1_fingerprint = l1.corpus_fingerprint();
    if field.l1_package_fingerprint() != l1_fingerprint {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L2 package was compiled for a different L1.1 corpus fingerprint",
        ));
    }
    let source = std::fs::read_to_string(morphology_corpus_path)?;
    let corpus = L2TeacherCorpus::parse_tsv(&source).map_err(io::Error::other)?;
    let bound_forms = field.bound_form_refs().collect::<Vec<_>>();
    let resolver_workers = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(bound_forms.len().max(1));
    let chunk_size = bound_forms.len().div_ceil(resolver_workers);
    let decoded = std::thread::scope(|scope| {
        let handles = bound_forms
            .chunks(chunk_size.max(1))
            .map(|chunk| {
                let l1 = &l1;
                let field = &field;
                scope.spawn(move || {
                    chunk
                        .iter()
                        .filter_map(|(form_ref, terminal_id)| {
                            let l1_surface = l1.decode_terminal(*terminal_id)?;
                            let l2_surface = field.decode_form_ref(*form_ref)?;
                            (l1_surface == l2_surface)
                                .then(|| (l2_surface.to_string(), *terminal_id))
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("L2 proof decoder worker"))
            .collect::<BTreeMap<_, _>>()
    });
    if decoded.len() != bound_forms.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L2 package contains undecodable or surface-mismatched L1.1 terminal IDs",
        ));
    }
    let teacher_surfaces = corpus
        .forms
        .iter()
        .map(|form| form.surface.as_str())
        .collect::<BTreeSet<_>>();
    if let Some(surface) = decoded
        .keys()
        .find(|surface| !teacher_surfaces.contains(surface.as_str()))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("L2 package surface {surface:?} is absent from the teacher"),
        ));
    }
    let terminal_by_surface = decoded;
    let seeds_by_lemma = corpus
        .forms
        .iter()
        .filter_map(|form| Some((form.lemma.clone(), *terminal_by_surface.get(&form.surface)?)))
        .fold(
            BTreeMap::<String, Vec<u32>>::new(),
            |mut seeds, (lemma, terminal)| {
                seeds.entry(lemma).or_default().push(terminal);
                seeds
            },
        );

    let heldout_scenes = corpus
        .scenes
        .iter()
        .filter(|scene| scene.heldout)
        .collect::<Vec<_>>();
    let proof_workers = if limit == 0 {
        std::env::var("LAY_L2_PROOF_WORKERS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(resolver_workers)
            .clamp(1, heldout_scenes.len().max(1))
    } else {
        1
    };
    let proof_chunk_size = heldout_scenes.len().div_ceil(proof_workers).max(1);
    let partials = std::thread::scope(|scope| {
        heldout_scenes
            .chunks(proof_chunk_size)
            .map(|scenes| {
                let field = &field;
                let seeds_by_lemma = &seeds_by_lemma;
                scope
                    .spawn(move || evaluate_morphology_scenes(scenes, field, seeds_by_lemma, limit))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|worker| worker.join().expect("L2 proof worker"))
            .collect::<Vec<_>>()
    });
    let mut proof = ProofShard::default();
    for partial in partials {
        proof.merge(partial);
    }
    for pos in 1_u16..=4 {
        proof.per_pos.entry(pos).or_default();
    }
    let ProofShard {
        aggregate,
        per_slot,
        per_pos,
        unresolved_per_pos,
        mut latency_us,
        unresolved,
        evaluated,
        failure_examples,
        coverage_failure_examples,
        false_authority_examples,
    } = proof;
    let mut near_neighbor = SlotScore::default();
    let mut near_neighbor_examples = Vec::new();
    for scene in corpus.neighbor_scenes.iter().filter(|scene| scene.heldout) {
        let Some(target) = field.form_ref_for_surface(&scene.surface) else {
            continue;
        };
        let mut seed_surfaces = std::iter::once(scene.surface.as_str())
            .chain(scene.competitors.iter().map(String::as_str))
            .collect::<Vec<_>>();
        seed_surfaces.sort_unstable();
        seed_surfaces.dedup();
        let seeds = seed_surfaces
            .iter()
            .map(|surface| L2LexicalSeed {
                terminal_id: terminal_by_surface.get(*surface).copied(),
                surface: Some((*surface).to_string()),
                evidence_milli: 1_000,
                origin: L2LexicalSeedOrigin::GroundedL11,
            })
            .collect::<Vec<_>>();
        let readout = field.readout(&scene.context, &seeds, 32);
        score_verdict(&mut near_neighbor, &readout.verdict, target);
        let target_retained = match &readout.verdict {
            L2LocalVerdict::Winner { form_ref } => *form_ref == target,
            L2LocalVerdict::Tied { form_refs } => form_refs.contains(&target),
            L2LocalVerdict::Abstain => false,
        };
        if !target_retained && near_neighbor_examples.len() < 32 {
            near_neighbor_examples.push(serde_json::json!({
                "lemma": scene.lemma,
                "expected": scene.surface,
                "context": scene.context,
                "competitors": scene.competitors,
                "verdict": format!("{:?}", readout.verdict),
                "candidates": readout.candidates.iter().take(16).map(|candidate| {
                    serde_json::json!({
                        "form_ref": candidate.form_ref,
                        "l1_terminal_id": candidate.l1_terminal_id,
                        "surface": candidate.surface,
                        "score": candidate.local_score,
                        "slot": candidate.slot_phase_milli,
                        "neighbor": candidate.neighbor_pressure,
                        "competition": candidate.competition_pressure,
                        "explicit_competition": candidate.explicit_competition_pressure,
                    })
                }).collect::<Vec<_>>(),
            }));
        }
    }
    latency_us.sort_unstable();
    let package_bytes = std::fs::metadata(l2_package_path)?.len();
    let per_slot = per_slot
        .into_iter()
        .map(|(feature_mask, score)| {
            (
                feature_mask.to_string(),
                serde_json::json!(score_json(&score)),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let per_pos = per_pos
        .into_iter()
        .map(|(pos, score)| {
            let name = match pos {
                1 => "noun",
                2 => "verb",
                3 => "adjective",
                4 => "pronoun",
                _ => "unknown",
            };
            (name.to_string(), serde_json::json!(score_json(&score)))
        })
        .collect::<serde_json::Map<_, _>>();
    let unresolved_per_pos = unresolved_per_pos
        .into_iter()
        .map(|(pos, unresolved)| {
            let name = match pos {
                1 => "noun",
                2 => "verb",
                3 => "adjective",
                4 => "pronoun",
                _ => "unknown",
            };
            (name.to_string(), serde_json::json!(unresolved))
        })
        .collect::<serde_json::Map<_, _>>();
    Ok(serde_json::json!({
        "kind": "canonical_l2_fixed_heldout_proof",
        "l1_package": l1_package_path,
        "l2_package": l2_package_path,
        "morphology_corpus": morphology_corpus_path,
        "heldout_available": corpus.scenes.iter().filter(|scene| scene.heldout).count(),
        "evaluated": evaluated,
        "unresolved": unresolved,
        "unresolved_per_pos": unresolved_per_pos,
        "same_lemma": score_json(&aggregate),
        "morphology_slot": score_json(&aggregate),
        "per_pos": per_pos,
        "per_feature_mask": per_slot,
        "failure_examples": failure_examples,
        "coverage_failure_examples": coverage_failure_examples,
        "false_authority_examples": false_authority_examples,
        "near_neighbor": score_json(&near_neighbor),
        "near_neighbor_tested": near_neighbor.total > 0,
        "near_neighbor_failure_examples": near_neighbor_examples,
        "package_bytes": package_bytes,
        "cold_load_us": cold_load_us,
        "hot_p50_us": percentile(&latency_us, 50),
        "hot_p99_us": percentile(&latency_us, 99),
        "proof_workers": proof_workers,
        "runtime_authority_changed": false,
    }))
}

fn evaluate_morphology_scenes(
    scenes: &[&TeacherScene],
    field: &StandaloneL2Field,
    seeds_by_lemma: &BTreeMap<String, Vec<u32>>,
    limit: usize,
) -> ProofShard {
    let mut shard = ProofShard::default();
    for scene in scenes {
        if limit != 0 && shard.evaluated >= limit {
            break;
        }
        let (Some(target), Some(seed_terminals)) = (
            field.form_ref_for_surface(&scene.surface),
            seeds_by_lemma.get(&scene.lemma),
        ) else {
            shard.unresolved += 1;
            *shard
                .unresolved_per_pos
                .entry(crate::nanda_wave::morphology_phase::feature_primary_pos(
                    scene.feature_mask,
                ))
                .or_default() += 1;
            continue;
        };
        let mut seed_terminals = seed_terminals.clone();
        seed_terminals.sort_unstable();
        seed_terminals.dedup();
        let seeds = seed_terminals
            .iter()
            .take(32)
            .map(|terminal_id| L2LexicalSeed {
                terminal_id: Some(*terminal_id),
                surface: None,
                evidence_milli: 1_000,
                origin: L2LexicalSeedOrigin::GroundedL11,
            })
            .collect::<Vec<_>>();
        let started = Instant::now();
        let readout = field.readout(&scene.context, &seeds, 32);
        shard.latency_us.push(started.elapsed().as_micros() as u64);
        shard.evaluated += 1;
        score_verdict(&mut shard.aggregate, &readout.verdict, target);
        score_verdict(
            shard.per_slot.entry(scene.feature_mask).or_default(),
            &readout.verdict,
            target,
        );
        score_verdict(
            shard
                .per_pos
                .entry(crate::nanda_wave::morphology_phase::feature_primary_pos(
                    scene.feature_mask,
                ))
                .or_default(),
            &readout.verdict,
            target,
        );
        let correct = matches!(
            &readout.verdict,
            L2LocalVerdict::Winner { form_ref } if *form_ref == target
        );
        let target_retained = correct
            || matches!(
                &readout.verdict,
                L2LocalVerdict::Tied { form_refs } if form_refs.contains(&target)
            );
        if !correct && shard.failure_examples.len() < 32 {
            shard.failure_examples.push(serde_json::json!({
                "lemma": scene.lemma,
                "expected": scene.surface,
                "feature_mask": scene.feature_mask,
                "context": scene.context,
                "seed_terminals": seed_terminals,
                "verdict": format!("{:?}", readout.verdict),
                "context_mode_id": readout.context_mode_id,
                "candidates": readout.candidates.iter().take(8).map(|candidate| {
                    serde_json::json!({
                        "form_ref": candidate.form_ref,
                        "l1_terminal_id": candidate.l1_terminal_id,
                        "surface": candidate.surface,
                        "score": candidate.local_score,
                        "l1": candidate.l1_evidence_milli,
                        "slot": candidate.slot_phase_milli,
                        "neighbor": candidate.neighbor_pressure,
                        "competition": candidate.competition_pressure,
                        "explicit_competition": candidate.explicit_competition_pressure,
                        "features": candidate.feature_masks,
                    })
                }).collect::<Vec<_>>(),
            }));
        }
        if !target_retained && shard.coverage_failure_examples.len() < 32 {
            shard.coverage_failure_examples.push(serde_json::json!({
                "lemma": scene.lemma,
                "expected": scene.surface,
                "feature_mask": scene.feature_mask,
                "context": scene.context,
                "seed_terminals": seed_terminals,
                "verdict": format!("{:?}", readout.verdict),
                "context_mode_id": readout.context_mode_id,
                "candidates": readout.candidates.iter().take(16).map(|candidate| {
                    serde_json::json!({
                        "form_ref": candidate.form_ref,
                        "l1_terminal_id": candidate.l1_terminal_id,
                        "surface": candidate.surface,
                        "score": candidate.local_score,
                        "l1": candidate.l1_evidence_milli,
                        "slot": candidate.slot_phase_milli,
                        "neighbor": candidate.neighbor_pressure,
                        "competition": candidate.competition_pressure,
                        "explicit_competition": candidate.explicit_competition_pressure,
                        "features": candidate.feature_masks,
                        "lemmas": candidate.lemma_ids,
                    })
                }).collect::<Vec<_>>(),
            }));
        }
        if matches!(&readout.verdict, L2LocalVerdict::Winner { form_ref } if *form_ref != target)
            && shard.false_authority_examples.len() < 32
        {
            shard.false_authority_examples.push(serde_json::json!({
                "lemma": scene.lemma,
                "expected": scene.surface,
                "feature_mask": scene.feature_mask,
                "context": scene.context,
                "verdict": format!("{:?}", readout.verdict),
                "candidates": readout.candidates.iter().take(16).map(|candidate| {
                    serde_json::json!({
                        "form_ref": candidate.form_ref,
                        "l1_terminal_id": candidate.l1_terminal_id,
                        "surface": candidate.surface,
                        "score": candidate.local_score,
                        "l1": candidate.l1_evidence_milli,
                        "slot": candidate.slot_phase_milli,
                        "neighbor": candidate.neighbor_pressure,
                        "competition": candidate.competition_pressure,
                        "explicit_competition": candidate.explicit_competition_pressure,
                        "features": candidate.feature_masks,
                        "lemmas": candidate.lemma_ids,
                    })
                }).collect::<Vec<_>>(),
            }));
        }
    }
    shard
}

fn score_verdict(score: &mut SlotScore, verdict: &L2LocalVerdict, target: u32) {
    score.total += 1;
    match verdict {
        L2LocalVerdict::Winner { form_ref } if *form_ref == target => {
            score.winner_correct += 1;
        }
        L2LocalVerdict::Winner { .. } => {
            score.false_authority += 1;
        }
        L2LocalVerdict::Tied { form_refs } if form_refs.contains(&target) => {
            score.tied_contains += 1;
        }
        L2LocalVerdict::Tied { .. } => {}
        L2LocalVerdict::Abstain => {
            score.abstain += 1;
        }
    }
}

fn score_json(score: &SlotScore) -> serde_json::Value {
    serde_json::json!({
        "total": score.total,
        "winner_correct": score.winner_correct,
        "winner_top1_percent": percent(score.winner_correct, score.total),
        "target_coverage_percent": percent(
            score.winner_correct.saturating_add(score.tied_contains),
            score.total,
        ),
        "tied_contains": score.tied_contains,
        "abstain": score.abstain,
        "false_authority": score.false_authority,
        "false_authority_percent": percent(score.false_authority, score.total),
    })
}

fn percent(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 * 100.0 / denominator as f64
    }
}

fn percentile(values: &[u64], percentile: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let index = (values.len() - 1).saturating_mul(percentile) / 100;
    values[index]
}
