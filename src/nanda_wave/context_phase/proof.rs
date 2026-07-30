#[cfg(test)]
use std::io::Cursor;
use std::io::{self, Read};
use std::path::Path;
use std::sync::mpsc::sync_channel;
use std::thread;

use serde::Serialize;

use super::online::{
    l2_lattice_probe, L2ProbePool, OnlineContextPhaseConfig, OnlineContextPhaseLearner,
    L2_PROBE_BATCH_FRAGMENTS,
};
use super::{ContextPhaseDisposition, ContextPhaseMode, ContextPhasePackage, SurfaceMutationField};
use std::sync::Arc;

const MAX_COMPETITORS: usize = 4;
const HELDOUT_MODULUS: usize = 5;
const HELDOUT_REMAINDER: usize = 4;
const MIN_SUPPORT_COVERAGE_PPM: u32 = 100_000;
const MAX_COUNTEREXAMPLES: usize = 64;

/// Cold-proof evidence only. Hashes identify a repeated phase conflict without
/// putting corpus text or lexical strings into the hot package.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ContextPhaseCounterexample {
    pub(crate) context_tail: String,
    pub(crate) target: String,
    pub(crate) false_winner: String,
    pub(crate) scene_hash: u64,
    pub(crate) target_hash: u64,
    pub(crate) false_winner_hash: u64,
    pub(crate) target_margin_micro: i64,
    pub(crate) false_margin_micro: i64,
    pub(crate) target_profile_present: bool,
    pub(crate) target_signature_profile_present: bool,
    pub(crate) target_positive_micro: i64,
    pub(crate) target_anti_micro: i64,
    pub(crate) target_pairwise_known_edges: u16,
    pub(crate) target_pairwise_unknown_edges: u16,
    pub(crate) target_pairwise_blocked: bool,
    pub(crate) target_pairwise_certified: bool,
    pub(crate) false_profile_present: bool,
    pub(crate) false_signature_profile_present: bool,
    pub(crate) false_positive_micro: i64,
    pub(crate) false_anti_micro: i64,
    pub(crate) false_pairwise_known_edges: u16,
    pub(crate) false_pairwise_unknown_edges: u16,
    pub(crate) false_pairwise_blocked: bool,
    pub(crate) false_pairwise_certified: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ContextPhaseProofReport {
    pub(crate) kind: &'static str,
    pub(crate) architecture: &'static str,
    pub(crate) train_passes: u8,
    pub(crate) heldout_passes: u8,
    pub(crate) train_fragments: usize,
    pub(crate) heldout_fragments: usize,
    pub(crate) training_l2_lattice_probes: u64,
    pub(crate) training_l2_lattice_negative_examples: u64,
    pub(crate) training_l2_lattice_empty_results: u64,
    pub(crate) training_l2_lattice_max_competitors: u32,
    pub(crate) training_l2_probe_workers: usize,
    pub(crate) training_l2_target_not_retained: u64,
    /// Fixed denominator: every heldout transition where L2 produced a real
    /// lattice. This remains comparable across package variants.
    pub(crate) lattice_transitions: usize,
    pub(crate) l2_target_not_retained: usize,
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
    pub(crate) counterexamples: Vec<ContextPhaseCounterexample>,
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
    pub(crate) no_signature_top1: usize,
    pub(crate) no_signature_false_top1: usize,
    pub(crate) signature_improved_cases: usize,
    pub(crate) signature_worsened_cases: usize,
    pub(crate) signature_assisted_top1: usize,
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
    pub(crate) signature_ablation_drop: usize,
    pub(crate) signature_false_top1_reduction: usize,
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

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ContextPhaseDifferentialProofReport {
    pub(crate) kind: &'static str,
    pub(crate) heldout_fragments: usize,
    pub(crate) lattice_transitions: usize,
    pub(crate) compared_transitions: usize,
    pub(crate) baseline_target_profiles: usize,
    pub(crate) candidate_target_profiles: usize,
    pub(crate) lost_target_profiles: usize,
    pub(crate) baseline_supports: usize,
    pub(crate) candidate_supports: usize,
    pub(crate) lost_supports: usize,
    pub(crate) gained_supports: usize,
    pub(crate) baseline_top1: usize,
    pub(crate) candidate_top1: usize,
    pub(crate) lost_top1: usize,
    pub(crate) gained_top1: usize,
    pub(crate) baseline_false_supports: usize,
    pub(crate) candidate_false_supports: usize,
    pub(crate) new_false_supports: usize,
    pub(crate) baseline_false_top1: usize,
    pub(crate) candidate_false_top1: usize,
    pub(crate) new_false_top1: usize,
    pub(crate) raw_words_stored: bool,
    pub(crate) runtime_authority: bool,
    pub(crate) verdict: &'static str,
}

#[derive(Default)]
struct DifferentialTotals {
    lattice_transitions: usize,
    compared_transitions: usize,
    baseline_target_profiles: usize,
    candidate_target_profiles: usize,
    lost_target_profiles: usize,
    baseline_supports: usize,
    candidate_supports: usize,
    lost_supports: usize,
    gained_supports: usize,
    baseline_top1: usize,
    candidate_top1: usize,
    lost_top1: usize,
    gained_top1: usize,
    baseline_false_supports: usize,
    candidate_false_supports: usize,
    new_false_supports: usize,
    baseline_false_top1: usize,
    candidate_false_top1: usize,
    new_false_top1: usize,
}

impl DifferentialTotals {
    fn merge(&mut self, other: Self) {
        self.lattice_transitions += other.lattice_transitions;
        self.compared_transitions += other.compared_transitions;
        self.baseline_target_profiles += other.baseline_target_profiles;
        self.candidate_target_profiles += other.candidate_target_profiles;
        self.lost_target_profiles += other.lost_target_profiles;
        self.baseline_supports += other.baseline_supports;
        self.candidate_supports += other.candidate_supports;
        self.lost_supports += other.lost_supports;
        self.gained_supports += other.gained_supports;
        self.baseline_top1 += other.baseline_top1;
        self.candidate_top1 += other.candidate_top1;
        self.lost_top1 += other.lost_top1;
        self.gained_top1 += other.gained_top1;
        self.baseline_false_supports += other.baseline_false_supports;
        self.candidate_false_supports += other.candidate_false_supports;
        self.new_false_supports += other.new_false_supports;
        self.baseline_false_top1 += other.baseline_false_top1;
        self.candidate_false_top1 += other.candidate_false_top1;
        self.new_false_top1 += other.new_false_top1;
    }
}

#[derive(Default)]
struct ProofTotals {
    lattice_transitions: usize,
    l2_target_not_retained: usize,
    target_profile_missing: usize,
    evaluated: usize,
    context_evidence_cases: usize,
    full_supports: usize,
    full_top1: usize,
    full_top1_provisional_positive: usize,
    full_top1_reinforced_positive: usize,
    full_false_supports: usize,
    full_false_top1: usize,
    counterexamples: Vec<ContextPhaseCounterexample>,
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
    no_signature_top1: usize,
    no_signature_false_top1: usize,
    signature_improved_cases: usize,
    signature_worsened_cases: usize,
    signature_assisted_top1: usize,
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
        self.l2_target_not_retained += other.l2_target_not_retained;
        self.target_profile_missing += other.target_profile_missing;
        self.evaluated += other.evaluated;
        self.context_evidence_cases += other.context_evidence_cases;
        self.full_supports += other.full_supports;
        self.full_top1 += other.full_top1;
        self.full_top1_provisional_positive += other.full_top1_provisional_positive;
        self.full_top1_reinforced_positive += other.full_top1_reinforced_positive;
        self.full_false_supports += other.full_false_supports;
        self.full_false_top1 += other.full_false_top1;
        let remaining = MAX_COUNTEREXAMPLES.saturating_sub(self.counterexamples.len());
        self.counterexamples
            .extend(other.counterexamples.into_iter().take(remaining));
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
        self.no_signature_top1 += other.no_signature_top1;
        self.no_signature_false_top1 += other.no_signature_false_top1;
        self.signature_improved_cases += other.signature_improved_cases;
        self.signature_worsened_cases += other.signature_worsened_cases;
        self.signature_assisted_top1 += other.signature_assisted_top1;
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
        Arc::new(SurfaceMutationField::default()),
    )
    .expect("in-memory L3 proof reader cannot fail")
    .1
}

pub(crate) fn prove_context_phase_path(
    corpus_path: &Path,
    max_fragments: usize,
    min_profile_support: u32,
) -> io::Result<ContextPhaseProofReport> {
    prove_context_phase_path_with_surface_field(
        corpus_path,
        max_fragments,
        min_profile_support,
        Arc::new(SurfaceMutationField::default()),
    )
}

pub(crate) fn prove_context_phase_path_with_surface_field(
    corpus_path: &Path,
    max_fragments: usize,
    min_profile_support: u32,
    surface_field: Arc<SurfaceMutationField>,
) -> io::Result<ContextPhaseProofReport> {
    train_and_prove(
        || std::fs::File::open(corpus_path),
        max_fragments,
        min_profile_support,
        surface_field,
    )
    .map(|(_, report)| report)
}

pub(crate) fn build_and_prove_context_phase_path(
    corpus_path: &Path,
    max_fragments: usize,
    min_profile_support: u32,
) -> io::Result<(ContextPhasePackage, ContextPhaseProofReport)> {
    build_and_prove_context_phase_path_with_surface_field(
        corpus_path,
        max_fragments,
        min_profile_support,
        Arc::new(SurfaceMutationField::default()),
    )
}

pub(crate) fn build_and_prove_context_phase_path_with_surface_field(
    corpus_path: &Path,
    max_fragments: usize,
    min_profile_support: u32,
    surface_field: Arc<SurfaceMutationField>,
) -> io::Result<(ContextPhasePackage, ContextPhaseProofReport)> {
    train_and_prove(
        || std::fs::File::open(corpus_path),
        max_fragments,
        min_profile_support,
        surface_field,
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
    prove_context_phase_package_path_with_surface_field(
        corpus_path,
        package_path,
        max_fragments,
        min_profile_support,
        &SurfaceMutationField::default(),
    )
}

pub(crate) fn prove_context_phase_package_path_with_surface_field(
    corpus_path: &Path,
    package_path: &Path,
    max_fragments: usize,
    min_profile_support: u32,
    surface_field: &SurfaceMutationField,
) -> io::Result<ContextPhaseProofReport> {
    let package = super::read_package(package_path)?;
    let (totals, heldout_fragments) = evaluate_heldout_stream(
        std::fs::File::open(corpus_path)?,
        max_fragments,
        &package,
        surface_field,
    )?;
    Ok(report_from_totals(
        totals,
        0,
        heldout_fragments,
        min_profile_support.max(2),
        0,
    ))
}

pub(crate) fn prove_context_phase_package_delta_path(
    corpus_path: &Path,
    baseline_path: &Path,
    candidate_path: &Path,
    max_fragments: usize,
    surface_field: &SurfaceMutationField,
) -> io::Result<ContextPhaseDifferentialProofReport> {
    let baseline = super::read_package(baseline_path)?;
    let candidate = super::read_package(candidate_path)?;
    if baseline.signature_schema != candidate.signature_schema {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "L3 differential proof requires matching signature schemas",
        ));
    }
    let workers = proof_worker_count();
    let (totals, heldout_fragments) = thread::scope(|scope| {
        let mut senders = Vec::with_capacity(workers);
        let mut handles = Vec::with_capacity(workers);
        for _ in 0..workers {
            let (sender, receiver) = sync_channel::<Vec<String>>(2);
            senders.push(sender);
            let baseline = &baseline;
            let candidate = &candidate;
            handles.push(scope.spawn(move || {
                let mut totals = DifferentialTotals::default();
                while let Ok(tokens) = receiver.recv() {
                    evaluate_fragment_delta(
                        baseline,
                        candidate,
                        &tokens,
                        surface_field,
                        &mut totals,
                    );
                }
                totals
            }));
        }

        let mut heldout_fragments = 0_usize;
        let mut next_worker = 0_usize;
        let stream_result = super::stream::visit_tokenized_fragments(
            std::fs::File::open(corpus_path)?,
            max_fragments,
            |ordinal, tokens| {
                if !is_heldout(ordinal) {
                    return Ok(());
                }
                heldout_fragments = heldout_fragments.saturating_add(1);
                senders[next_worker]
                    .send(tokens.to_vec())
                    .map_err(|_| io::Error::other("L3 differential proof worker stopped"))?;
                next_worker = (next_worker + 1) % workers;
                Ok(())
            },
        );
        drop(senders);

        let mut totals = DifferentialTotals::default();
        for handle in handles {
            totals.merge(
                handle
                    .join()
                    .map_err(|_| io::Error::other("L3 differential proof worker panicked"))?,
            );
        }
        stream_result?;
        Ok::<_, io::Error>((totals, heldout_fragments))
    })?;

    let passed = totals.compared_transitions > 0
        && totals.lost_target_profiles == 0
        && totals.lost_supports == 0
        && totals.lost_top1 == 0
        && totals.new_false_supports == 0
        && totals.new_false_top1 == 0;
    Ok(ContextPhaseDifferentialProofReport {
        kind: "l3_context_phase_full_differential_proof",
        heldout_fragments,
        lattice_transitions: totals.lattice_transitions,
        compared_transitions: totals.compared_transitions,
        baseline_target_profiles: totals.baseline_target_profiles,
        candidate_target_profiles: totals.candidate_target_profiles,
        lost_target_profiles: totals.lost_target_profiles,
        baseline_supports: totals.baseline_supports,
        candidate_supports: totals.candidate_supports,
        lost_supports: totals.lost_supports,
        gained_supports: totals.gained_supports,
        baseline_top1: totals.baseline_top1,
        candidate_top1: totals.candidate_top1,
        lost_top1: totals.lost_top1,
        gained_top1: totals.gained_top1,
        baseline_false_supports: totals.baseline_false_supports,
        candidate_false_supports: totals.candidate_false_supports,
        new_false_supports: totals.new_false_supports,
        baseline_false_top1: totals.baseline_false_top1,
        candidate_false_top1: totals.candidate_false_top1,
        new_false_top1: totals.new_false_top1,
        raw_words_stored: false,
        runtime_authority: false,
        verdict: if passed { "PASS" } else { "WATCH" },
    })
}

fn train_and_prove<R, F>(
    mut open: F,
    max_fragments: usize,
    min_profile_support: u32,
    surface_field: Arc<SurfaceMutationField>,
) -> io::Result<(ContextPhasePackage, ContextPhaseProofReport)>
where
    R: Read,
    F: FnMut() -> io::Result<R>,
{
    let config = OnlineContextPhaseConfig::production(min_profile_support);
    let mut learner =
        OnlineContextPhaseLearner::new_with_surface_field(config, Arc::clone(&surface_field));
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
    let (totals, heldout_fragments) =
        evaluate_heldout_stream(open()?, max_fragments, &package, surface_field.as_ref())?;
    let train_fragments = train_stats
        .accepted_fragments
        .saturating_sub(heldout_fragments);
    let mut report = report_from_totals(
        totals,
        train_fragments,
        heldout_fragments,
        config.min_profile_support,
        1,
    );
    let training_stats = learner.stats();
    report.training_l2_lattice_probes = training_stats.l2_lattice_probes;
    report.training_l2_lattice_negative_examples = training_stats.l2_lattice_negative_examples;
    report.training_l2_lattice_empty_results = training_stats.l2_lattice_empty_results;
    report.training_l2_lattice_max_competitors = training_stats.l2_lattice_max_competitors;
    report.training_l2_probe_workers = l2_pool.worker_count();
    report.training_l2_target_not_retained = training_stats.l2_target_not_retained;
    Ok((package, report))
}

fn evaluate_heldout_stream<R: Read>(
    reader: R,
    max_fragments: usize,
    package: &ContextPhasePackage,
    surface_field: &SurfaceMutationField,
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
                    evaluate_fragment(package, &tokens, surface_field, &mut totals);
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

fn evaluate_fragment(
    package: &ContextPhasePackage,
    tokens: &[String],
    surface_field: &SurfaceMutationField,
    totals: &mut ProofTotals,
) {
    for index in 1..tokens.len() {
        let target = &tokens[index];
        let lattice = l2_lattice_probe(&tokens[..index], target, MAX_COMPETITORS, surface_field);
        if lattice.competitors.is_empty() {
            continue;
        }
        totals.lattice_transitions = totals.lattice_transitions.saturating_add(1);
        if !lattice.target_retained {
            totals.l2_target_not_retained = totals.l2_target_not_retained.saturating_add(1);
            continue;
        }
        let competitors = lattice.competitors;
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
        totals.signature_assisted_top1 += (full_correct
            && full[0].signature_profile_present
            && full[0].signature_positive_micro >= full[0].positive_micro)
            as usize;
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
        classify_false_winner(
            package,
            &tokens[..index],
            target,
            &candidates,
            &full,
            totals,
        );

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

        let no_signature = package.score_candidates_with_mode(
            &tokens[..index],
            &candidates,
            ContextPhaseMode::NoSignatureProfile,
        );
        let no_signature_correct = correct_is_unique_top(&no_signature);
        totals.no_signature_top1 += no_signature_correct as usize;
        totals.no_signature_false_top1 += false_candidate_wins(&no_signature) as usize;
        totals.signature_improved_cases += (full_correct && !no_signature_correct) as usize;
        totals.signature_worsened_cases += (!full_correct && no_signature_correct) as usize;

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
        permutation.sort_by_key(|candidate| super::context_exact_hash(candidate));
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

fn evaluate_fragment_delta(
    baseline: &ContextPhasePackage,
    candidate: &ContextPhasePackage,
    tokens: &[String],
    surface_field: &SurfaceMutationField,
    totals: &mut DifferentialTotals,
) {
    for index in 1..tokens.len() {
        let target = &tokens[index];
        let lattice = l2_lattice_probe(&tokens[..index], target, MAX_COMPETITORS, surface_field);
        if lattice.competitors.is_empty() || !lattice.target_retained {
            continue;
        }
        totals.lattice_transitions = totals.lattice_transitions.saturating_add(1);
        let mut candidates = Vec::with_capacity(lattice.competitors.len() + 1);
        candidates.push(target.as_str());
        candidates.extend(lattice.competitors.iter().map(String::as_str));
        let baseline_readouts = baseline.score_candidates_with_mode(
            &tokens[..index],
            &candidates,
            ContextPhaseMode::Full,
        );
        let candidate_readouts = candidate.score_candidates_with_mode(
            &tokens[..index],
            &candidates,
            ContextPhaseMode::Full,
        );
        let baseline_profile = baseline_readouts
            .first()
            .is_some_and(|readout| readout.profile_present);
        let candidate_profile = candidate_readouts
            .first()
            .is_some_and(|readout| readout.profile_present);
        totals.baseline_target_profiles += baseline_profile as usize;
        totals.candidate_target_profiles += candidate_profile as usize;
        totals.lost_target_profiles += (baseline_profile && !candidate_profile) as usize;
        if !baseline_profile && !candidate_profile {
            continue;
        }
        totals.compared_transitions = totals.compared_transitions.saturating_add(1);

        let baseline_support = baseline_readouts
            .first()
            .is_some_and(|readout| readout.disposition == ContextPhaseDisposition::Support);
        let candidate_support = candidate_readouts
            .first()
            .is_some_and(|readout| readout.disposition == ContextPhaseDisposition::Support);
        totals.baseline_supports += baseline_support as usize;
        totals.candidate_supports += candidate_support as usize;
        totals.lost_supports += (baseline_support && !candidate_support) as usize;
        totals.gained_supports += (!baseline_support && candidate_support) as usize;

        let baseline_top1 = correct_is_unique_top(&baseline_readouts);
        let candidate_top1 = correct_is_unique_top(&candidate_readouts);
        totals.baseline_top1 += baseline_top1 as usize;
        totals.candidate_top1 += candidate_top1 as usize;
        totals.lost_top1 += (baseline_top1 && !candidate_top1) as usize;
        totals.gained_top1 += (!baseline_top1 && candidate_top1) as usize;

        let baseline_false_supports = baseline_readouts
            .iter()
            .skip(1)
            .filter(|readout| readout.disposition == ContextPhaseDisposition::Support)
            .count();
        let candidate_false_supports = candidate_readouts
            .iter()
            .skip(1)
            .filter(|readout| readout.disposition == ContextPhaseDisposition::Support)
            .count();
        totals.baseline_false_supports += baseline_false_supports;
        totals.candidate_false_supports += candidate_false_supports;
        totals.new_false_supports += baseline_readouts
            .iter()
            .zip(&candidate_readouts)
            .skip(1)
            .filter(|(baseline, candidate)| {
                baseline.disposition != ContextPhaseDisposition::Support
                    && candidate.disposition == ContextPhaseDisposition::Support
            })
            .count();

        let baseline_false_top1 = false_candidate_winner_index(&baseline_readouts);
        let candidate_false_top1 = false_candidate_winner_index(&candidate_readouts);
        totals.baseline_false_top1 += baseline_false_top1.is_some() as usize;
        totals.candidate_false_top1 += candidate_false_top1.is_some() as usize;
        totals.new_false_top1 +=
            candidate_false_top1.is_some_and(|winner| Some(winner) != baseline_false_top1) as usize;
    }
}

fn classify_false_winner(
    package: &ContextPhasePackage,
    context: &[String],
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
    if totals.counterexamples.len() < MAX_COUNTEREXAMPLES {
        let scene_hash = context.iter().fold(0_u64, |state, token| {
            crate::stable_hash::mix64_golden(state ^ super::context_exact_hash(token))
        });
        totals.counterexamples.push(ContextPhaseCounterexample {
            context_tail: context
                .iter()
                .rev()
                .take(8)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join(" "),
            target: target.to_string(),
            false_winner: candidates[false_index].to_string(),
            scene_hash,
            target_hash: super::context_exact_hash(target),
            false_winner_hash: super::context_exact_hash(candidates[false_index]),
            target_margin_micro: correct.margin_micro,
            false_margin_micro: false_winner.margin_micro,
            target_profile_present: correct.profile_present,
            target_signature_profile_present: correct.signature_profile_present,
            target_positive_micro: correct.positive_micro,
            target_anti_micro: correct.anti_micro,
            target_pairwise_known_edges: correct.pairwise_known_edges,
            target_pairwise_unknown_edges: correct.pairwise_unknown_edges,
            target_pairwise_blocked: correct.pairwise_blocked,
            target_pairwise_certified: correct.pairwise_certified,
            false_profile_present: false_winner.profile_present,
            false_signature_profile_present: false_winner.signature_profile_present,
            false_positive_micro: false_winner.positive_micro,
            false_anti_micro: false_winner.anti_micro,
            false_pairwise_known_edges: false_winner.pairwise_known_edges,
            false_pairwise_unknown_edges: false_winner.pairwise_unknown_edges,
            false_pairwise_blocked: false_winner.pairwise_blocked,
            false_pairwise_certified: false_winner.pairwise_certified,
        });
    }
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
    let minimum_known_tokens = if package.signature_schema >= super::SIGNATURE_SCHEMA_RELATION_ROLES
    {
        1
    } else {
        2
    };
    let context_ready = usize::from(correct.context_known_tokens) >= minimum_known_tokens
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
    let signature_ablation_drop = totals.full_top1.saturating_sub(totals.no_signature_top1);
    let signature_false_top1_reduction = totals
        .no_signature_false_top1
        .saturating_sub(totals.full_false_top1);
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
        architecture: "online_relation_phase_v4_signature_pairwise_lattice",
        train_passes,
        heldout_passes: 1,
        train_fragments,
        heldout_fragments,
        training_l2_lattice_probes: 0,
        training_l2_lattice_negative_examples: 0,
        training_l2_lattice_empty_results: 0,
        training_l2_lattice_max_competitors: 0,
        training_l2_probe_workers: 0,
        training_l2_target_not_retained: 0,
        lattice_transitions: totals.lattice_transitions,
        l2_target_not_retained: totals.l2_target_not_retained,
        target_profile_missing: totals.target_profile_missing,
        evaluated_transitions: totals.evaluated,
        context_evidence_cases: totals.context_evidence_cases,
        full_supports: totals.full_supports,
        full_top1: totals.full_top1,
        full_top1_provisional_positive: totals.full_top1_provisional_positive,
        full_top1_reinforced_positive: totals.full_top1_reinforced_positive,
        full_false_supports: totals.full_false_supports,
        full_false_top1: totals.full_false_top1,
        counterexamples: totals.counterexamples,
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
        no_signature_top1: totals.no_signature_top1,
        no_signature_false_top1: totals.no_signature_false_top1,
        signature_improved_cases: totals.signature_improved_cases,
        signature_worsened_cases: totals.signature_worsened_cases,
        signature_assisted_top1: totals.signature_assisted_top1,
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
        signature_ablation_drop,
        signature_false_top1_reduction,
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

    #[test]
    fn identical_packages_have_zero_differential_regressions() {
        let corpus = [
            "на улице снова идет дождь",
            "вечером на улице идет дождь",
            "утром на улице идет дождь",
            "сегодня на улице идет дождь",
            "завтра на улице идет дождь",
        ]
        .join("\n");
        let (package, _) =
            super::super::compile_context_phase(super::super::ContextPhaseCompileInput {
                corpus_text: &corpus,
                max_fragments: 0,
                min_profile_support: 2,
            });
        let field =
            SurfaceMutationField::from_corrections_jsonl(r#"{"from":"дожь","to":"дождь"}"#, 1)
                .expect("valid differential surface field");
        let tokens = ["сегодня", "на", "улице", "идет", "дождь"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let mut totals = DifferentialTotals::default();
        evaluate_fragment_delta(&package, &package, &tokens, &field, &mut totals);
        assert_eq!(totals.lost_target_profiles, 0);
        assert_eq!(totals.lost_supports, 0);
        assert_eq!(totals.lost_top1, 0);
        assert_eq!(totals.new_false_supports, 0);
        assert_eq!(totals.new_false_top1, 0);
        assert_eq!(totals.baseline_supports, totals.candidate_supports);
        assert_eq!(totals.baseline_top1, totals.candidate_top1);
    }
}
