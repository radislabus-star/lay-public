#[cfg(test)]
use std::io::Cursor;
use std::io::{self, Read};
use std::path::Path;
use std::sync::mpsc::sync_channel;
use std::thread;

use serde::Serialize;

use super::online::{
    l2_lattice_competitors, L2ProbePool, OnlineContextPhaseConfig, OnlineContextPhaseLearner,
    L2_PROBE_BATCH_FRAGMENTS,
};
use super::{ContextPhaseDisposition, ContextPhaseMode, ContextPhasePackage};

const MAX_COMPETITORS: usize = 4;
const HELDOUT_MODULUS: usize = 5;
const HELDOUT_REMAINDER: usize = 4;
const MIN_SUPPORT_COVERAGE_PPM: u32 = 100_000;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ContextPhaseProofReport {
    pub(crate) kind: &'static str,
    pub(crate) architecture: &'static str,
    pub(crate) train_passes: u8,
    pub(crate) heldout_passes: u8,
    pub(crate) train_fragments: usize,
    pub(crate) heldout_fragments: usize,
    /// Fixed denominator: every heldout transition where L2 produced a real
    /// lattice. This remains comparable across package variants.
    pub(crate) lattice_transitions: usize,
    /// A missing target profile is a coverage miss, not a reason to erase the
    /// transition from the proof denominator.
    pub(crate) target_profile_missing: usize,
    pub(crate) evaluated_transitions: usize,
    pub(crate) context_evidence_cases: usize,
    pub(crate) full_supports: usize,
    pub(crate) full_top1: usize,
    pub(crate) full_top1_provisional_positive: usize,
    pub(crate) full_top1_reinforced_positive: usize,
    pub(crate) full_false_supports: usize,
    pub(crate) full_false_top1: usize,
    pub(crate) full_false_top1_close_competition: usize,
    pub(crate) full_false_top1_separated_competition: usize,
    pub(crate) full_false_top1_weak_context: usize,
    pub(crate) full_false_top1_context_ready: usize,
    pub(crate) full_false_top1_edit_distance_one: usize,
    pub(crate) full_false_top1_edit_distance_two: usize,
    pub(crate) full_false_top1_edit_distance_three_or_more: usize,
    pub(crate) full_false_top1_correct_supported: usize,
    pub(crate) full_false_top1_correct_not_supported: usize,
    pub(crate) full_false_top1_correct_profile_missing: usize,
    pub(crate) full_false_top1_correct_profile_present_not_supported: usize,
    pub(crate) full_false_top1_any_competitor_profile_missing: usize,
    pub(crate) full_false_top1_winner_profile_missing: usize,
    pub(crate) full_false_top1_winner_with_anti: usize,
    pub(crate) full_false_top1_winner_without_anti: usize,
    pub(crate) full_false_top1_winner_anti_active: usize,
    pub(crate) full_false_top1_winner_anti_inactive: usize,
    pub(crate) full_false_top1_winner_provisional_positive: usize,
    pub(crate) full_false_top1_winner_reinforced_positive: usize,
    pub(crate) no_phase_supports: usize,
    pub(crate) no_anti_top1: usize,
    pub(crate) no_anti_false_supports: usize,
    pub(crate) no_anti_false_top1: usize,
    pub(crate) no_semantic_top1: usize,
    pub(crate) no_pairwise_top1: usize,
    pub(crate) no_pairwise_false_top1: usize,
    pub(crate) no_hard_pairwise_top1: usize,
    pub(crate) no_hard_pairwise_false_top1: usize,
    pub(crate) pairwise_improved_cases: usize,
    pub(crate) pairwise_worsened_cases: usize,
    pub(crate) hard_pairwise_improved_cases: usize,
    pub(crate) hard_pairwise_worsened_cases: usize,
    pub(crate) shuffled_pair_direction_top1: usize,
    pub(crate) shuffled_pair_direction_false_top1: usize,
    pub(crate) shuffled_pair_scene_top1: usize,
    pub(crate) shuffled_pair_scene_false_top1: usize,
    pub(crate) magnitude_only_pairwise_top1: usize,
    pub(crate) magnitude_only_pairwise_false_top1: usize,
    pub(crate) candidate_permutation_mismatches: usize,
    pub(crate) pairwise_known_edges: u64,
    pub(crate) pairwise_unknown_edges: u64,
    pub(crate) pairwise_cycle_members: u64,
    pub(crate) pairwise_correct_blocked: usize,
    pub(crate) pairwise_false_candidates_blocked: usize,
    pub(crate) pairwise_worsened_correct_blocked: usize,
    pub(crate) pairwise_certificate_supports: usize,
    pub(crate) pairwise_certificate_correct: usize,
    pub(crate) pairwise_certificate_false: usize,
    pub(crate) phase_improved_cases: usize,
    pub(crate) phase_worsened_cases: usize,
    pub(crate) anti_improved_cases: usize,
    pub(crate) anti_worsened_cases: usize,
    pub(crate) phase_ablation_drop: usize,
    pub(crate) anti_ablation_drop: usize,
    pub(crate) anti_false_support_reduction: usize,
    pub(crate) anti_false_top1_reduction: usize,
    pub(crate) semantic_ablation_drop: usize,
    pub(crate) pairwise_ablation_drop: usize,
    pub(crate) pairwise_false_top1_reduction: usize,
    pub(crate) hard_pairwise_ablation_drop: usize,
    pub(crate) hard_pairwise_false_top1_reduction: usize,
    pub(crate) shuffled_pair_direction_drop: usize,
    pub(crate) shuffled_pair_scene_drop: usize,
    pub(crate) magnitude_only_pairwise_drop: usize,
    pub(crate) support_precision_ppm: u32,
    /// Support coverage over the fixed L2-lattice denominator.
    pub(crate) global_support_coverage_ppm: u32,
    pub(crate) support_coverage_ppm: u32,
    pub(crate) min_support_coverage_ppm: u32,
    pub(crate) raw_words_stored: bool,
    pub(crate) l2_lattice_unchanged: bool,
    pub(crate) l3_apply_authority: bool,
    pub(crate) min_profile_support: u32,
    pub(crate) verdict: &'static str,
}

#[derive(Default)]
struct ProofTotals {
    lattice_transitions: usize,
    target_profile_missing: usize,
    evaluated: usize,
    context_evidence_cases: usize,
    full_supports: usize,
    full_top1: usize,
    full_top1_provisional_positive: usize,
    full_top1_reinforced_positive: usize,
    full_false_supports: usize,
    full_false_top1: usize,
    full_false_top1_close_competition: usize,
    full_false_top1_separated_competition: usize,
    full_false_top1_weak_context: usize,
    full_false_top1_context_ready: usize,
    full_false_top1_edit_distance_one: usize,
    full_false_top1_edit_distance_two: usize,
    full_false_top1_edit_distance_three_or_more: usize,
    full_false_top1_correct_supported: usize,
    full_false_top1_correct_not_supported: usize,
    full_false_top1_correct_profile_missing: usize,
    full_false_top1_correct_profile_present_not_supported: usize,
    full_false_top1_any_competitor_profile_missing: usize,
    full_false_top1_winner_profile_missing: usize,
    full_false_top1_winner_with_anti: usize,
    full_false_top1_winner_without_anti: usize,
    full_false_top1_winner_anti_active: usize,
    full_false_top1_winner_anti_inactive: usize,
    full_false_top1_winner_provisional_positive: usize,
    full_false_top1_winner_reinforced_positive: usize,
    no_phase_supports: usize,
    no_anti_top1: usize,
    no_anti_false_supports: usize,
    no_anti_false_top1: usize,
    no_semantic_top1: usize,
    no_pairwise_top1: usize,
    no_pairwise_false_top1: usize,
    no_hard_pairwise_top1: usize,
    no_hard_pairwise_false_top1: usize,
    pairwise_improved_cases: usize,
    pairwise_worsened_cases: usize,
    hard_pairwise_improved_cases: usize,
    hard_pairwise_worsened_cases: usize,
    shuffled_pair_direction_top1: usize,
    shuffled_pair_direction_false_top1: usize,
    shuffled_pair_scene_top1: usize,
    shuffled_pair_scene_false_top1: usize,
    magnitude_only_pairwise_top1: usize,
    magnitude_only_pairwise_false_top1: usize,
    candidate_permutation_mismatches: usize,
    pairwise_known_edges: u64,
    pairwise_unknown_edges: u64,
    pairwise_cycle_members: u64,
    pairwise_correct_blocked: usize,
    pairwise_false_candidates_blocked: usize,
    pairwise_worsened_correct_blocked: usize,
    pairwise_certificate_supports: usize,
    pairwise_certificate_correct: usize,
    pairwise_certificate_false: usize,
    phase_improved_cases: usize,
    phase_worsened_cases: usize,
    anti_improved_cases: usize,
    anti_worsened_cases: usize,
}

impl ProofTotals {
    fn merge(&mut self, other: Self) {
        self.lattice_transitions += other.lattice_transitions;
        self.target_profile_missing += other.target_profile_missing;
        self.evaluated += other.evaluated;
        self.context_evidence_cases += other.context_evidence_cases;
        self.full_supports += other.full_supports;
        self.full_top1 += other.full_top1;
        self.full_top1_provisional_positive += other.full_top1_provisional_positive;
        self.full_top1_reinforced_positive += other.full_top1_reinforced_positive;
        self.full_false_supports += other.full_false_supports;
        self.full_false_top1 += other.full_false_top1;
        self.full_false_top1_close_competition += other.full_false_top1_close_competition;
        self.full_false_top1_separated_competition += other.full_false_top1_separated_competition;
        self.full_false_top1_weak_context += other.full_false_top1_weak_context;
        self.full_false_top1_context_ready += other.full_false_top1_context_ready;
        self.full_false_top1_edit_distance_one += other.full_false_top1_edit_distance_one;
        self.full_false_top1_edit_distance_two += other.full_false_top1_edit_distance_two;
        self.full_false_top1_edit_distance_three_or_more +=
            other.full_false_top1_edit_distance_three_or_more;
        self.full_false_top1_correct_supported += other.full_false_top1_correct_supported;
        self.full_false_top1_correct_not_supported += other.full_false_top1_correct_not_supported;
        self.full_false_top1_correct_profile_missing +=
            other.full_false_top1_correct_profile_missing;
        self.full_false_top1_correct_profile_present_not_supported +=
            other.full_false_top1_correct_profile_present_not_supported;
        self.full_false_top1_any_competitor_profile_missing +=
            other.full_false_top1_any_competitor_profile_missing;
        self.full_false_top1_winner_profile_missing += other.full_false_top1_winner_profile_missing;
        self.full_false_top1_winner_with_anti += other.full_false_top1_winner_with_anti;
        self.full_false_top1_winner_without_anti += other.full_false_top1_winner_without_anti;
        self.full_false_top1_winner_anti_active += other.full_false_top1_winner_anti_active;
        self.full_false_top1_winner_anti_inactive += other.full_false_top1_winner_anti_inactive;
        self.full_false_top1_winner_provisional_positive +=
            other.full_false_top1_winner_provisional_positive;
        self.full_false_top1_winner_reinforced_positive +=
            other.full_false_top1_winner_reinforced_positive;
        self.no_phase_supports += other.no_phase_supports;
        self.no_anti_top1 += other.no_anti_top1;
        self.no_anti_false_supports += other.no_anti_false_supports;
        self.no_anti_false_top1 += other.no_anti_false_top1;
        self.no_semantic_top1 += other.no_semantic_top1;
        self.no_pairwise_top1 += other.no_pairwise_top1;
        self.no_pairwise_false_top1 += other.no_pairwise_false_top1;
        self.no_hard_pairwise_top1 += other.no_hard_pairwise_top1;
        self.no_hard_pairwise_false_top1 += other.no_hard_pairwise_false_top1;
        self.pairwise_improved_cases += other.pairwise_improved_cases;
        self.pairwise_worsened_cases += other.pairwise_worsened_cases;
        self.hard_pairwise_improved_cases += other.hard_pairwise_improved_cases;
        self.hard_pairwise_worsened_cases += other.hard_pairwise_worsened_cases;
        self.shuffled_pair_direction_top1 += other.shuffled_pair_direction_top1;
        self.shuffled_pair_direction_false_top1 += other.shuffled_pair_direction_false_top1;
        self.shuffled_pair_scene_top1 += other.shuffled_pair_scene_top1;
        self.shuffled_pair_scene_false_top1 += other.shuffled_pair_scene_false_top1;
        self.magnitude_only_pairwise_top1 += other.magnitude_only_pairwise_top1;
        self.magnitude_only_pairwise_false_top1 += other.magnitude_only_pairwise_false_top1;
        self.candidate_permutation_mismatches += other.candidate_permutation_mismatches;
        self.pairwise_known_edges += other.pairwise_known_edges;
        self.pairwise_unknown_edges += other.pairwise_unknown_edges;
        self.pairwise_cycle_members += other.pairwise_cycle_members;
        self.pairwise_correct_blocked += other.pairwise_correct_blocked;
        self.pairwise_false_candidates_blocked += other.pairwise_false_candidates_blocked;
        self.pairwise_worsened_correct_blocked += other.pairwise_worsened_correct_blocked;
        self.pairwise_certificate_supports += other.pairwise_certificate_supports;
        self.pairwise_certificate_correct += other.pairwise_certificate_correct;
        self.pairwise_certificate_false += other.pairwise_certificate_false;
        self.phase_improved_cases += other.phase_improved_cases;
        self.phase_worsened_cases += other.phase_worsened_cases;
        self.anti_improved_cases += other.anti_improved_cases;
        self.anti_worsened_cases += other.anti_worsened_cases;
    }
}

#[cfg(test)]
pub(super) fn prove_context_phase_bytes(
    corpus_text: &str,
    max_fragments: usize,
    min_profile_support: u32,
) -> ContextPhaseProofReport {
    train_and_prove(
        || Ok(Cursor::new(corpus_text.as_bytes())),
        max_fragments,
        min_profile_support,
    )
    .expect("in-memory L3 proof reader cannot fail")
    .1
}

pub(crate) fn prove_context_phase_path(
    corpus_path: &Path,
    max_fragments: usize,
    min_profile_support: u32,
) -> io::Result<ContextPhaseProofReport> {
    train_and_prove(
        || std::fs::File::open(corpus_path),
        max_fragments,
        min_profile_support,
    )
    .map(|(_, report)| report)
}

pub(crate) fn build_and_prove_context_phase_path(
    corpus_path: &Path,
    max_fragments: usize,
    min_profile_support: u32,
) -> io::Result<(ContextPhasePackage, ContextPhaseProofReport)> {
    train_and_prove(
        || std::fs::File::open(corpus_path),
        max_fragments,
        min_profile_support,
    )
}

/// Evaluates an already compiled package against a separate corpus surface.
/// This path never trains, mutates, or reloads package state, so a
/// cross-surface merge cannot accidentally prove itself on its source rows.
pub(crate) fn prove_context_phase_package_path(
    corpus_path: &Path,
    package_path: &Path,
    max_fragments: usize,
    min_profile_support: u32,
) -> io::Result<ContextPhaseProofReport> {
    let package = super::read_package(package_path)?;
    let (totals, heldout_fragments) =
        evaluate_heldout_stream(std::fs::File::open(corpus_path)?, max_fragments, &package)?;
    Ok(report_from_totals(
        totals,
        0,
        heldout_fragments,
        min_profile_support.max(2),
        0,
    ))
}

fn train_and_prove<R, F>(
    mut open: F,
    max_fragments: usize,
    min_profile_support: u32,
) -> io::Result<(ContextPhasePackage, ContextPhaseProofReport)>
where
    R: Read,
    F: FnMut() -> io::Result<R>,
{
    let config = OnlineContextPhaseConfig::production(min_profile_support);
    let mut learner = OnlineContextPhaseLearner::new(config);
    let l2_pool = L2ProbePool::new();
    let mut pending_l2 = Vec::new();
    let mut batch_fragments = 0_usize;
    let train_stats =
        super::stream::visit_tokenized_fragments(open()?, max_fragments, |ordinal, tokens| {
            if !is_heldout(ordinal) {
                pending_l2.extend(learner.ingest_fragment_positive(tokens));
                batch_fragments = batch_fragments.saturating_add(1);
                if batch_fragments >= L2_PROBE_BATCH_FRAGMENTS {
                    learner.apply_l2_probe_batch(&l2_pool, &mut pending_l2)?;
                    batch_fragments = 0;
                }
            }
            Ok(())
        })?;
    learner.apply_l2_probe_batch(&l2_pool, &mut pending_l2)?;
    let package = learner.snapshot();
    let (totals, heldout_fragments) = evaluate_heldout_stream(open()?, max_fragments, &package)?;
    let train_fragments = train_stats
        .accepted_fragments
        .saturating_sub(heldout_fragments);
    let report = report_from_totals(
        totals,
        train_fragments,
        heldout_fragments,
        config.min_profile_support,
        1,
    );
    Ok((package, report))
}

fn evaluate_heldout_stream<R: Read>(
    reader: R,
    max_fragments: usize,
    package: &ContextPhasePackage,
) -> io::Result<(ProofTotals, usize)> {
    let workers = proof_worker_count();
    thread::scope(|scope| {
        let mut senders = Vec::with_capacity(workers);
        let mut handles = Vec::with_capacity(workers);
        for _ in 0..workers {
            let (sender, receiver) = sync_channel::<Vec<String>>(2);
            senders.push(sender);
            handles.push(scope.spawn(move || {
                let mut totals = ProofTotals::default();
                while let Ok(tokens) = receiver.recv() {
                    evaluate_fragment(package, &tokens, &mut totals);
                }
                totals
            }));
        }

        let mut heldout_fragments = 0_usize;
        let mut next_worker = 0_usize;
        let stream_result =
            super::stream::visit_tokenized_fragments(reader, max_fragments, |ordinal, tokens| {
                if !is_heldout(ordinal) {
                    return Ok(());
                }
                heldout_fragments = heldout_fragments.saturating_add(1);
                senders[next_worker]
                    .send(tokens.to_vec())
                    .map_err(|_| io::Error::other("L3 heldout proof worker stopped"))?;
                next_worker = (next_worker + 1) % workers;
                Ok(())
            });
        drop(senders);

        let mut totals = ProofTotals::default();
        for handle in handles {
            totals.merge(
                handle
                    .join()
                    .map_err(|_| io::Error::other("L3 heldout proof worker panicked"))?,
            );
        }
        stream_result?;
        Ok((totals, heldout_fragments))
    })
}

fn proof_worker_count() -> usize {
    if cfg!(test) {
        return 1;
    }
    thread::available_parallelism()
        .map(|workers| workers.get())
        .unwrap_or(1)
        .max(1)
}

fn is_heldout(ordinal: usize) -> bool {
    ordinal % HELDOUT_MODULUS == HELDOUT_REMAINDER
}

fn evaluate_fragment(package: &ContextPhasePackage, tokens: &[String], totals: &mut ProofTotals) {
    for index in 1..tokens.len() {
        let target = &tokens[index];
        let competitors = l2_lattice_competitors(&tokens[..index], target, MAX_COMPETITORS);
        if competitors.is_empty() {
            continue;
        }
        totals.lattice_transitions = totals.lattice_transitions.saturating_add(1);
        let mut candidates = Vec::with_capacity(competitors.len() + 1);
        candidates.push(target.as_str());
        candidates.extend(competitors.iter().map(String::as_str));
        let full = package.score_candidates_with_mode(
            &tokens[..index],
            &candidates,
            ContextPhaseMode::Full,
        );
        if !full.first().is_some_and(|readout| readout.profile_present) {
            totals.target_profile_missing = totals.target_profile_missing.saturating_add(1);
            continue;
        }
        totals.evaluated = totals.evaluated.saturating_add(1);
        totals.context_evidence_cases += full
            .first()
            .is_some_and(|readout| readout.context_known_tokens > 0 && readout.positive_centers > 0)
            as usize;
        totals.full_supports += full
            .first()
            .is_some_and(|readout| readout.disposition == ContextPhaseDisposition::Support)
            as usize;
        let full_correct = correct_is_unique_top(&full);
        totals.pairwise_known_edges += u64::from(full[0].pairwise_known_edges);
        totals.pairwise_unknown_edges += u64::from(full[0].pairwise_unknown_edges);
        totals.pairwise_cycle_members += u64::from(full[0].pairwise_cycle_members);
        totals.pairwise_correct_blocked += full[0].pairwise_blocked as usize;
        totals.pairwise_false_candidates_blocked += full
            .iter()
            .skip(1)
            .filter(|readout| readout.pairwise_blocked)
            .count();
        totals.pairwise_certificate_supports += full
            .iter()
            .filter(|readout| readout.pairwise_certified)
            .count();
        totals.pairwise_certificate_correct +=
            full.first()
                .is_some_and(|readout| readout.pairwise_certified) as usize;
        totals.pairwise_certificate_false += full
            .iter()
            .skip(1)
            .filter(|readout| readout.pairwise_certified)
            .count();
        totals.full_top1 += full_correct as usize;
        if full_correct {
            if full[0].positive_center_support < 2 {
                totals.full_top1_provisional_positive += 1;
            } else {
                totals.full_top1_reinforced_positive += 1;
            }
        }
        totals.full_false_supports += full
            .iter()
            .skip(1)
            .filter(|readout| readout.disposition == ContextPhaseDisposition::Support)
            .count();
        classify_false_winner(package, target, &candidates, &full, totals);

        let no_phase = package.score_candidates_with_mode(
            &tokens[..index],
            &candidates,
            ContextPhaseMode::NoPhase,
        );
        totals.no_phase_supports += no_phase
            .first()
            .is_some_and(|readout| readout.disposition == ContextPhaseDisposition::Support)
            as usize;
        let no_phase_correct = correct_is_unique_top(&no_phase);
        totals.phase_improved_cases += (full_correct && !no_phase_correct) as usize;
        totals.phase_worsened_cases += (!full_correct && no_phase_correct) as usize;

        let no_anti = package.score_candidates_with_mode(
            &tokens[..index],
            &candidates,
            ContextPhaseMode::NoAnti,
        );
        let no_anti_correct = correct_is_unique_top(&no_anti);
        totals.no_anti_top1 += no_anti_correct as usize;
        totals.no_anti_false_supports += no_anti
            .iter()
            .skip(1)
            .filter(|readout| readout.disposition == ContextPhaseDisposition::Support)
            .count();
        totals.no_anti_false_top1 += false_candidate_wins(&no_anti) as usize;
        totals.anti_improved_cases += (full_correct && !no_anti_correct) as usize;
        totals.anti_worsened_cases += (!full_correct && no_anti_correct) as usize;

        let no_semantic = package.score_candidates_with_mode(
            &tokens[..index],
            &candidates,
            ContextPhaseMode::NoSemanticState,
        );
        totals.no_semantic_top1 += correct_is_unique_top(&no_semantic) as usize;

        let no_pairwise = package.score_candidates_with_mode(
            &tokens[..index],
            &candidates,
            ContextPhaseMode::NoPairwise,
        );
        let no_pairwise_correct = correct_is_unique_top(&no_pairwise);
        totals.no_pairwise_top1 += no_pairwise_correct as usize;
        totals.no_pairwise_false_top1 += false_candidate_wins(&no_pairwise) as usize;
        totals.pairwise_improved_cases += (full_correct && !no_pairwise_correct) as usize;
        totals.pairwise_worsened_cases += (!full_correct && no_pairwise_correct) as usize;
        totals.pairwise_worsened_correct_blocked +=
            (!full_correct && no_pairwise_correct && full[0].pairwise_blocked) as usize;

        let no_hard_pairwise = package.score_candidates_with_mode(
            &tokens[..index],
            &candidates,
            ContextPhaseMode::NoHardPairwise,
        );
        let no_hard_pairwise_correct = correct_is_unique_top(&no_hard_pairwise);
        totals.no_hard_pairwise_top1 += no_hard_pairwise_correct as usize;
        totals.no_hard_pairwise_false_top1 += false_candidate_wins(&no_hard_pairwise) as usize;
        totals.hard_pairwise_improved_cases += (full_correct && !no_hard_pairwise_correct) as usize;
        totals.hard_pairwise_worsened_cases += (!full_correct && no_hard_pairwise_correct) as usize;

        let mut permutation = candidates.clone();
        permutation.sort_by_key(|candidate| super::super::phase_field::hash_text(candidate));
        let permuted = package.score_candidates_with_mode(
            &tokens[..index],
            &permutation,
            ContextPhaseMode::Full,
        );
        let full_winner = unique_support_winner(&full).map(|winner| candidates[winner]);
        let permuted_winner = unique_support_winner(&permuted).map(|winner| permutation[winner]);
        totals.candidate_permutation_mismatches += (full_winner != permuted_winner) as usize;

        let shuffled_direction = package.score_candidates_with_mode(
            &tokens[..index],
            &candidates,
            ContextPhaseMode::ShuffledPairDirection,
        );
        totals.shuffled_pair_direction_top1 += correct_is_unique_top(&shuffled_direction) as usize;
        totals.shuffled_pair_direction_false_top1 +=
            false_candidate_wins(&shuffled_direction) as usize;

        let shuffled_scene = package.score_candidates_with_mode(
            &tokens[..index],
            &candidates,
            ContextPhaseMode::ShuffledPairScene,
        );
        totals.shuffled_pair_scene_top1 += correct_is_unique_top(&shuffled_scene) as usize;
        totals.shuffled_pair_scene_false_top1 += false_candidate_wins(&shuffled_scene) as usize;

        let magnitude_only = package.score_candidates_with_mode(
            &tokens[..index],
            &candidates,
            ContextPhaseMode::MagnitudeOnlyPairwise,
        );
        totals.magnitude_only_pairwise_top1 += correct_is_unique_top(&magnitude_only) as usize;
        totals.magnitude_only_pairwise_false_top1 += false_candidate_wins(&magnitude_only) as usize;
    }
}

fn classify_false_winner(
    package: &ContextPhasePackage,
    target: &str,
    candidates: &[&str],
    full: &[super::ContextPhaseReadout],
    totals: &mut ProofTotals,
) {
    let Some(false_index) = false_candidate_winner_index(full) else {
        return;
    };
    let false_winner = &full[false_index];
    totals.full_false_top1 = totals.full_false_top1.saturating_add(1);
    let correct = &full[0];
    if correct.disposition == ContextPhaseDisposition::Support {
        totals.full_false_top1_correct_supported += 1;
    } else {
        totals.full_false_top1_correct_not_supported += 1;
    }
    if correct.profile_present {
        if correct.disposition != ContextPhaseDisposition::Support {
            totals.full_false_top1_correct_profile_present_not_supported += 1;
        }
    } else {
        totals.full_false_top1_correct_profile_missing += 1;
    }
    if full.iter().skip(1).any(|readout| !readout.profile_present) {
        totals.full_false_top1_any_competitor_profile_missing += 1;
    }
    if !false_winner.profile_present {
        totals.full_false_top1_winner_profile_missing += 1;
    }
    if false_winner.anti_centers > 0 {
        totals.full_false_top1_winner_with_anti += 1;
    } else {
        totals.full_false_top1_winner_without_anti += 1;
    }
    if false_winner.anti_micro > 0 {
        totals.full_false_top1_winner_anti_active += 1;
    } else {
        totals.full_false_top1_winner_anti_inactive += 1;
    }
    if false_winner.positive_center_support < 2 {
        totals.full_false_top1_winner_provisional_positive += 1;
    } else {
        totals.full_false_top1_winner_reinforced_positive += 1;
    }
    let loss = false_winner
        .margin_micro
        .saturating_sub(correct.margin_micro);
    let competition_scale = i64::from(package.competition_threshold_micro.max(1));
    if loss <= competition_scale {
        totals.full_false_top1_close_competition += 1;
    } else {
        totals.full_false_top1_separated_competition += 1;
    }
    let context_ready = correct.context_known_tokens >= 2
        && usize::from(correct.context_known_tokens) * 2 >= usize::from(correct.context_tokens);
    if context_ready {
        totals.full_false_top1_context_ready += 1;
    } else {
        totals.full_false_top1_weak_context += 1;
    }
    match crate::text_metrics::damerau_levenshtein(target, candidates[false_index]) {
        0 => {}
        1 => totals.full_false_top1_edit_distance_one += 1,
        2 => totals.full_false_top1_edit_distance_two += 1,
        _ => totals.full_false_top1_edit_distance_three_or_more += 1,
    }
}

fn report_from_totals(
    totals: ProofTotals,
    train_fragments: usize,
    heldout_fragments: usize,
    min_profile_support: u32,
    train_passes: u8,
) -> ContextPhaseProofReport {
    let phase_ablation_drop = totals
        .full_supports
        .saturating_sub(totals.no_phase_supports);
    let anti_ablation_drop = totals.full_top1.saturating_sub(totals.no_anti_top1);
    let anti_false_support_reduction = totals
        .no_anti_false_supports
        .saturating_sub(totals.full_false_supports);
    let anti_false_top1_reduction = totals
        .no_anti_false_top1
        .saturating_sub(totals.full_false_top1);
    let semantic_ablation_drop = totals.full_top1.saturating_sub(totals.no_semantic_top1);
    let pairwise_ablation_drop = totals.full_top1.saturating_sub(totals.no_pairwise_top1);
    let pairwise_false_top1_reduction = totals
        .no_pairwise_false_top1
        .saturating_sub(totals.full_false_top1);
    let hard_pairwise_ablation_drop = totals
        .full_top1
        .saturating_sub(totals.no_hard_pairwise_top1);
    let hard_pairwise_false_top1_reduction = totals
        .no_hard_pairwise_false_top1
        .saturating_sub(totals.full_false_top1);
    let shuffled_pair_direction_drop = totals
        .full_top1
        .saturating_sub(totals.shuffled_pair_direction_top1);
    let shuffled_pair_scene_drop = totals
        .full_top1
        .saturating_sub(totals.shuffled_pair_scene_top1);
    let magnitude_only_pairwise_drop = totals
        .full_top1
        .saturating_sub(totals.magnitude_only_pairwise_top1);
    let support_precision_ppm = ((totals.full_supports as u64 * 1_000_000)
        / (totals.full_supports + totals.full_false_supports).max(1) as u64)
        .min(u64::from(u32::MAX)) as u32;
    let support_coverage_ppm = ((totals.full_supports as u64 * 1_000_000)
        / totals.evaluated.max(1) as u64)
        .min(u64::from(u32::MAX)) as u32;
    let global_support_coverage_ppm = ((totals.full_supports as u64 * 1_000_000)
        / totals.lattice_transitions.max(1) as u64)
        .min(u64::from(u32::MAX)) as u32;
    let verdict = promotion_verdict(&totals, global_support_coverage_ppm);
    ContextPhaseProofReport {
        kind: "l3_context_phase_heldout_proof",
        architecture: "online_relation_phase_v3_pairwise_lattice",
        train_passes,
        heldout_passes: 1,
        train_fragments,
        heldout_fragments,
        lattice_transitions: totals.lattice_transitions,
        target_profile_missing: totals.target_profile_missing,
        evaluated_transitions: totals.evaluated,
        context_evidence_cases: totals.context_evidence_cases,
        full_supports: totals.full_supports,
        full_top1: totals.full_top1,
        full_top1_provisional_positive: totals.full_top1_provisional_positive,
        full_top1_reinforced_positive: totals.full_top1_reinforced_positive,
        full_false_supports: totals.full_false_supports,
        full_false_top1: totals.full_false_top1,
        full_false_top1_close_competition: totals.full_false_top1_close_competition,
        full_false_top1_separated_competition: totals.full_false_top1_separated_competition,
        full_false_top1_weak_context: totals.full_false_top1_weak_context,
        full_false_top1_context_ready: totals.full_false_top1_context_ready,
        full_false_top1_edit_distance_one: totals.full_false_top1_edit_distance_one,
        full_false_top1_edit_distance_two: totals.full_false_top1_edit_distance_two,
        full_false_top1_edit_distance_three_or_more: totals
            .full_false_top1_edit_distance_three_or_more,
        full_false_top1_correct_supported: totals.full_false_top1_correct_supported,
        full_false_top1_correct_not_supported: totals.full_false_top1_correct_not_supported,
        full_false_top1_correct_profile_missing: totals.full_false_top1_correct_profile_missing,
        full_false_top1_correct_profile_present_not_supported: totals
            .full_false_top1_correct_profile_present_not_supported,
        full_false_top1_any_competitor_profile_missing: totals
            .full_false_top1_any_competitor_profile_missing,
        full_false_top1_winner_profile_missing: totals.full_false_top1_winner_profile_missing,
        full_false_top1_winner_with_anti: totals.full_false_top1_winner_with_anti,
        full_false_top1_winner_without_anti: totals.full_false_top1_winner_without_anti,
        full_false_top1_winner_anti_active: totals.full_false_top1_winner_anti_active,
        full_false_top1_winner_anti_inactive: totals.full_false_top1_winner_anti_inactive,
        full_false_top1_winner_provisional_positive: totals
            .full_false_top1_winner_provisional_positive,
        full_false_top1_winner_reinforced_positive: totals
            .full_false_top1_winner_reinforced_positive,
        no_phase_supports: totals.no_phase_supports,
        no_anti_top1: totals.no_anti_top1,
        no_anti_false_supports: totals.no_anti_false_supports,
        no_anti_false_top1: totals.no_anti_false_top1,
        no_semantic_top1: totals.no_semantic_top1,
        no_pairwise_top1: totals.no_pairwise_top1,
        no_pairwise_false_top1: totals.no_pairwise_false_top1,
        no_hard_pairwise_top1: totals.no_hard_pairwise_top1,
        no_hard_pairwise_false_top1: totals.no_hard_pairwise_false_top1,
        pairwise_improved_cases: totals.pairwise_improved_cases,
        pairwise_worsened_cases: totals.pairwise_worsened_cases,
        hard_pairwise_improved_cases: totals.hard_pairwise_improved_cases,
        hard_pairwise_worsened_cases: totals.hard_pairwise_worsened_cases,
        shuffled_pair_direction_top1: totals.shuffled_pair_direction_top1,
        shuffled_pair_direction_false_top1: totals.shuffled_pair_direction_false_top1,
        shuffled_pair_scene_top1: totals.shuffled_pair_scene_top1,
        shuffled_pair_scene_false_top1: totals.shuffled_pair_scene_false_top1,
        magnitude_only_pairwise_top1: totals.magnitude_only_pairwise_top1,
        magnitude_only_pairwise_false_top1: totals.magnitude_only_pairwise_false_top1,
        candidate_permutation_mismatches: totals.candidate_permutation_mismatches,
        pairwise_known_edges: totals.pairwise_known_edges,
        pairwise_unknown_edges: totals.pairwise_unknown_edges,
        pairwise_cycle_members: totals.pairwise_cycle_members,
        pairwise_correct_blocked: totals.pairwise_correct_blocked,
        pairwise_false_candidates_blocked: totals.pairwise_false_candidates_blocked,
        pairwise_worsened_correct_blocked: totals.pairwise_worsened_correct_blocked,
        pairwise_certificate_supports: totals.pairwise_certificate_supports,
        pairwise_certificate_correct: totals.pairwise_certificate_correct,
        pairwise_certificate_false: totals.pairwise_certificate_false,
        phase_improved_cases: totals.phase_improved_cases,
        phase_worsened_cases: totals.phase_worsened_cases,
        anti_improved_cases: totals.anti_improved_cases,
        anti_worsened_cases: totals.anti_worsened_cases,
        phase_ablation_drop,
        anti_ablation_drop,
        anti_false_support_reduction,
        anti_false_top1_reduction,
        semantic_ablation_drop,
        pairwise_ablation_drop,
        pairwise_false_top1_reduction,
        hard_pairwise_ablation_drop,
        hard_pairwise_false_top1_reduction,
        shuffled_pair_direction_drop,
        shuffled_pair_scene_drop,
        magnitude_only_pairwise_drop,
        support_precision_ppm,
        global_support_coverage_ppm,
        support_coverage_ppm,
        min_support_coverage_ppm: MIN_SUPPORT_COVERAGE_PPM,
        raw_words_stored: false,
        l2_lattice_unchanged: true,
        l3_apply_authority: false,
        min_profile_support,
        verdict,
    }
}

/// v3 puts destructive corpus evidence in directed pairwise banks. Unary anti
/// centers deliberately do not receive authority from a single corpus or a
/// dismissed live suggestion, so their ablation cannot be a promotion gate.
/// Promotion instead requires that the pairwise mechanism changes a heldout
/// decision causally, never worsens one, and loses that effect when its phase
/// representation is destroyed.
fn promotion_verdict(totals: &ProofTotals, global_support_coverage_ppm: u32) -> &'static str {
    let pairwise_causal = totals.pairwise_improved_cases > totals.pairwise_worsened_cases
        && totals.pairwise_known_edges > 0
        && totals.pairwise_certificate_false == 0
        && totals.shuffled_pair_direction_top1 < totals.full_top1
        && totals.shuffled_pair_scene_top1 < totals.full_top1
        && totals.magnitude_only_pairwise_top1 < totals.full_top1;
    if totals.evaluated > 0
        && totals.context_evidence_cases > 0
        && totals.full_supports > 0
        && totals.full_false_top1 == 0
        && totals.full_false_supports == 0
        && totals.candidate_permutation_mismatches == 0
        && totals.pairwise_worsened_cases == 0
        && totals.hard_pairwise_worsened_cases == 0
        && totals.phase_improved_cases > totals.phase_worsened_cases
        && totals.full_false_top1 <= totals.no_pairwise_false_top1
        && global_support_coverage_ppm >= MIN_SUPPORT_COVERAGE_PPM
        && pairwise_causal
    {
        "PASS"
    } else {
        "WATCH"
    }
}

fn correct_is_unique_top(readouts: &[super::ContextPhaseReadout]) -> bool {
    let Some(correct) = readouts.first() else {
        return false;
    };
    correct.disposition == ContextPhaseDisposition::Support
        && readouts
            .iter()
            .skip(1)
            .all(|readout| readout.margin_micro < correct.margin_micro)
}

fn false_candidate_wins(readouts: &[super::ContextPhaseReadout]) -> bool {
    false_candidate_winner_index(readouts).is_some()
}

fn false_candidate_winner_index(readouts: &[super::ContextPhaseReadout]) -> Option<usize> {
    let correct = readouts.first()?;
    readouts
        .iter()
        .enumerate()
        .skip(1)
        .filter(|candidate| {
            candidate.1.disposition == ContextPhaseDisposition::Support
                && candidate.1.margin_micro >= correct.margin_micro
        })
        .max_by_key(|(_, candidate)| candidate.margin_micro)
        .map(|(index, _)| index)
}

fn unique_support_winner(readouts: &[super::ContextPhaseReadout]) -> Option<usize> {
    let (index, winner) = readouts
        .iter()
        .enumerate()
        .filter(|(_, readout)| readout.disposition == ContextPhaseDisposition::Support)
        .max_by_key(|(_, readout)| readout.margin_micro)?;
    readouts
        .iter()
        .enumerate()
        .find(|(other, readout)| {
            *other != index
                && readout.disposition == ContextPhaseDisposition::Support
                && readout.margin_micro == winner.margin_micro
        })
        .is_none()
        .then_some(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_partition_is_stable() {
        let heldout = (0..20)
            .filter(|ordinal| is_heldout(*ordinal))
            .collect::<Vec<_>>();
        assert_eq!(heldout, vec![4, 9, 14, 19]);
    }

    #[test]
    fn proof_never_grants_l3_apply_authority() {
        let corpus = concat!(
            "на улице снова идет дождь. ",
            "вечером на улице идет дождь. ",
            "утром на улице идет дождь. ",
            "сегодня на улице идет дождь. ",
            "завтра на улице идет дождь."
        );
        let report = prove_context_phase_bytes(corpus, 0, 2);
        assert!(!report.raw_words_stored);
        assert!(!report.l3_apply_authority);
        assert!(report.l2_lattice_unchanged);
        assert_eq!(report.train_fragments, 4);
        assert_eq!(report.heldout_fragments, 1);
        assert!(report.lattice_transitions >= report.evaluated_transitions);
        assert_eq!(
            report.lattice_transitions,
            report.evaluated_transitions + report.target_profile_missing
        );
    }

    #[test]
    fn pairwise_promotion_does_not_require_unsafe_unary_anti_authority() {
        let totals = ProofTotals {
            lattice_transitions: 1,
            evaluated: 1,
            context_evidence_cases: 1,
            full_supports: 1,
            full_top1: 2,
            no_pairwise_top1: 1,
            no_pairwise_false_top1: 0,
            pairwise_improved_cases: 1,
            pairwise_known_edges: 1,
            phase_improved_cases: 1,
            shuffled_pair_direction_top1: 1,
            shuffled_pair_scene_top1: 1,
            magnitude_only_pairwise_top1: 1,
            // No unary anti outcome is present. This is the normal v3 case
            // when the corpus has no causal single-candidate rejection.
            ..ProofTotals::default()
        };
        assert_eq!(promotion_verdict(&totals, MIN_SUPPORT_COVERAGE_PPM), "PASS");
    }
}
