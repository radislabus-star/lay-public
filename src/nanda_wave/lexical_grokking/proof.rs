use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet};
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Instant;

use serde::Serialize;

use crate::stable_hash::mix64_golden;

use super::compiler::{
    compile_training_corpus_with_policy_in, CompileDiagnostics, ForwardPostingPolicy,
};
use super::corruption::{
    select_scale_training_damages_with_policy, split_damages, split_scale_damages, DamageExample,
    ScaleTrainingSurfacePolicy,
};
use super::format;
use super::proof_matrix::{FrequencyProfile, ProofMatrix};
use super::restoration::{AbstainReason, RestorationReadout};
use super::runtime::{L1RestorationHost, LexicalGrokkingMemory, ReadoutMode};
use super::training_budget::{checkpoint, TrainingBudgetGuard};
use super::training_corpus::TrainingCorpus;

const MAX_MISS_DIAGNOSTICS_PER_CLASS: usize = 8;
const MAX_POSITION_DIAGNOSTICS: usize = 128;
const MAX_RECONSTRUCTION_DIAGNOSTICS_PER_CLASS: usize = 64;
const MAX_FALSE_CERTAINTY_DIAGNOSTICS: usize = 128;
const BASELINE_FORWARD_COUPLINGS: usize = 256;
const DEFAULT_TRAINING_RSS_MIB: usize = 24 * 1024;
const PROOF_PROGRESS_INTERVAL: usize = 10_000;

#[derive(Clone, Copy, Debug)]
struct ScaleProofPolicy {
    heldout_per_class: usize,
    training_surfaces_per_word: usize,
    training_surface_policy: ScaleTrainingSurfacePolicy,
    maximum_rss_mib: usize,
}

type HeldoutReservoir = BTreeMap<&'static str, BinaryHeap<(u64, u32, String)>>;

struct ScaleTrainingShard {
    start_terminal: usize,
    corpus: TrainingCorpus,
    heldout: HeldoutReservoir,
    training_surfaces: usize,
}

struct ScaleHeldoutShard {
    start_terminal: usize,
    heldout: HeldoutReservoir,
    training_surfaces: usize,
}

#[derive(Clone, Debug, Default, Serialize)]
struct ClassMetrics {
    cases: usize,
    unique_cases: usize,
    unique_top1: usize,
    ambiguous_cases: usize,
    top1: usize,
    top8: usize,
    top64: usize,
    unique_top1_percent: f64,
    lattice_coverage_percent: f64,
    false_certainty: usize,
    raw_top1_outside_objective: usize,
    without_sequence_top1: usize,
    without_sequence_top1_percent: f64,
    sequence_delta_top1: isize,
    legacy_sequence_top1: usize,
    legacy_sequence_top1_percent: f64,
    sequence_vs_legacy_delta_top1: isize,
    legacy_sequence_unique_top1: usize,
    legacy_sequence_unique_top1_percent: f64,
    sequence_vs_legacy_unique_delta_top1: isize,
    without_position_unique_top1: usize,
    without_position_unique_top1_percent: f64,
    position_unique_delta_top1: isize,
    failure_decomposition: FailureDecomposition,
    edit_geometry: EditGeometryMetrics,
    restoration: RestorationMetrics,
}

#[derive(Clone, Debug, Default, Serialize)]
struct RestorationMetrics {
    cases: usize,
    winner: usize,
    tied: usize,
    tied_overflow: usize,
    abstain: usize,
    abstain_no_candidates: usize,
    abstain_outside_calibrated_basin: usize,
    abstain_weak_positive_phase: usize,
    abstain_weak_backward_reconstruction: usize,
    abstain_conflicting_evidence: usize,
    authority_target_winner: usize,
    authority_target_winner_percent: f64,
    target_retained: usize,
    target_retained_percent: f64,
    evidence_target_retained: usize,
    evidence_target_retained_percent: f64,
    scalar_geometry_target_in_nearest_basin: usize,
    scalar_geometry_target_in_nearest_basin_percent: f64,
    geometry_target_in_nearest_basin: usize,
    geometry_target_in_nearest_basin_percent: f64,
    reconstruction_basin_expansions: usize,
    reconstruction_target_recovered: usize,
    reconstruction_target_lost: usize,
    nearest_set_functional: usize,
    nearest_set_functional_percent: f64,
    geometry_unique_cases: usize,
    geometry_unique_winner: usize,
    geometry_unique_winner_percent: f64,
    geometry_tied_cases: usize,
    geometry_tied_safe: usize,
    geometry_tied_safe_percent: f64,
    false_singleton_on_geometry_tie: usize,
    objective_unique_cases: usize,
    objective_unique_winner: usize,
    objective_unique_winner_percent: f64,
    objective_ambiguous_cases: usize,
    objective_ambiguous_safe: usize,
    objective_ambiguous_safe_percent: f64,
    false_authority_on_objective_ambiguity: usize,
    crystallized_geometry_ties: usize,
    crystallized_geometry_ties_correct: usize,
    crystallized_geometry_ties_wrong: usize,
    crystallized_geometry_tie_precision_percent: f64,
    crystallization_known_edges: usize,
    crystallization_unknown_edges: usize,
    crystallization_tied_edges: usize,
    crystallization_conflict_edges: usize,
    crystallization_cycle_cases: usize,
}

#[derive(Clone, Debug, Default, Serialize)]
struct EditGeometryMetrics {
    target_unique_min_cases: usize,
    target_unique_min_top1: usize,
    target_unique_min_top1_percent: f64,
    target_unique_min_selected_min: usize,
    target_unique_min_selected_min_percent: f64,
    target_tied_min_cases: usize,
    target_tied_min_top1: usize,
    target_tied_min_top1_percent: f64,
    target_tied_min_selected_min: usize,
    target_tied_min_selected_min_percent: f64,
    target_not_min_cases: usize,
    target_not_min_top1: usize,
    target_not_min_top1_percent: f64,
    target_not_min_selected_min: usize,
    target_not_min_selected_min_percent: f64,
    target_missing_cases: usize,
}

#[derive(Clone, Debug, Default, Serialize)]
struct FailureDecomposition {
    unique_failures: usize,
    target_missing_top64: usize,
    target_rank2: usize,
    same_length_basin: usize,
    target_stronger_position: usize,
    target_stronger_sequence: usize,
    target_stronger_backward: usize,
    target_stronger_phase: usize,
    target_stronger_forward: usize,
    target_stronger_structural: usize,
    energy_deficit_le_250: usize,
    energy_deficit_le_500: usize,
    energy_deficit_le_1000: usize,
    both_phase_ge_990: usize,
    both_anti_zero: usize,
    both_pairwise_zero: usize,
    winner_stronger_forward: usize,
    winner_stronger_structural: usize,
    target_unique_min_edit: usize,
    target_tied_min_edit: usize,
    target_not_min_edit: usize,
    winner_without_vowel: usize,
}

#[derive(Clone, Debug, Serialize)]
struct MissDiagnostic {
    class: &'static str,
    surface: String,
    objective_unique: bool,
    target_terminal: u32,
    target_surface: Option<String>,
    selected_terminal: Option<u32>,
    selected_surface: Option<String>,
    target_rank: Option<usize>,
    selected_energy: Option<i32>,
    target_energy: Option<i32>,
    selected_geometry_distance: Option<u8>,
    target_geometry_distance: Option<u8>,
    selected_reconstruction_modes: Option<u8>,
    target_reconstruction_modes: Option<u8>,
    selected_sequence_milli: Option<u16>,
    target_sequence_milli: Option<u16>,
    selected_backward_milli: Option<u16>,
    target_backward_milli: Option<u16>,
}

#[derive(Clone, Debug, Serialize)]
struct PositionDiagnostic {
    outcome: &'static str,
    class: &'static str,
    surface: String,
    target_terminal: u32,
    before_terminal: Option<u32>,
    before_length_relation: Option<i8>,
    before_sequence_milli: Option<u16>,
    before_settled_energy: Option<i32>,
    before_forward_milli: Option<u16>,
    before_backward_milli: Option<u16>,
    before_structural_milli: Option<u16>,
    after_terminal: Option<u32>,
    after_length_relation: Option<i8>,
    after_sequence_milli: Option<u16>,
    after_position_milli: Option<u16>,
    after_settled_energy: Option<i32>,
    after_forward_milli: Option<u16>,
    after_backward_milli: Option<u16>,
    after_structural_milli: Option<u16>,
}

#[derive(Clone, Debug, Serialize)]
struct ReconstructionDiagnostic {
    class: &'static str,
    surface: String,
    target_terminal: u32,
    target_surface: Option<String>,
    candidate_terminal: u32,
    candidate_surface: Option<String>,
    candidate_is_target: bool,
    lattice_rank: usize,
    scalar_geometry_distance: u8,
    candidate_geometry_distance: u8,
    reconstruction_modes: u8,
    positive_milli: u16,
    backward_milli: u16,
    anti_milli: u16,
    hard_negative_milli: u16,
}

#[derive(Clone, Debug, Serialize)]
struct AmbiguityAuthorityDiagnostic {
    class: &'static str,
    surface: String,
    target_terminals: Vec<u32>,
    target_surfaces: Vec<Option<String>>,
    authority_terminal: u32,
    authority_surface: Option<String>,
    nearest_terminals: Vec<u32>,
    nearest_surfaces: Vec<Option<String>>,
}

#[derive(Clone, Debug, Serialize)]
struct FalseCertaintyDiagnostic {
    class: &'static str,
    surface: String,
    target_terminals: Vec<u32>,
    target_surfaces: Vec<Option<String>>,
    authority_terminal: u32,
    authority_surface: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct CleanMissDiagnostic {
    target_terminal: u32,
    target_surface: String,
    selected_terminal: Option<u32>,
    selected_surface: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct L1LexicalGrokkingProof {
    verdict: &'static str,
    l11_verdict: &'static str,
    l11_crystallization_verdict: &'static str,
    source_words: usize,
    proof_terminal_start: usize,
    proof_terminal_count: usize,
    training_surfaces: usize,
    training_surface_storage: &'static str,
    training_surface_bytes: usize,
    training_surface_span_bytes: usize,
    training_rss_budget_bytes: usize,
    training_peak_rss_bytes: usize,
    heldout_surfaces: usize,
    scale_training_surfaces_per_word: Option<usize>,
    scale_training_surface_policy: Option<&'static str>,
    scale_heldout_per_class: Option<usize>,
    clean_top1: usize,
    clean_preservation_percent: f64,
    heldout_top1: usize,
    heldout_top1_percent: f64,
    heldout_top8_percent: f64,
    heldout_top64_percent: f64,
    phase_ablation_drop: isize,
    anti_ablation_drop: isize,
    anti_improved: usize,
    anti_worsened: usize,
    sequence_ablation_drop: isize,
    sequence_improved: usize,
    sequence_worsened: usize,
    sequence_certificate_ablation_drop: isize,
    sequence_certificate_improved: usize,
    sequence_certificate_worsened: usize,
    sequence_vs_legacy_drop: isize,
    sequence_vs_legacy_improved: usize,
    sequence_vs_legacy_worsened: usize,
    pairwise_ablation_drop: isize,
    pairwise_improved: usize,
    pairwise_worsened: usize,
    position_ablation_drop: isize,
    position_improved: usize,
    position_worsened: usize,
    word_center_bytes: usize,
    word_center_bank_bytes: usize,
    atom_count: usize,
    forward_couplings: usize,
    forward_posting_policy: &'static str,
    forward_relations_before_policy: usize,
    forward_relations_dropped: usize,
    forward_atoms_above_baseline_cap: usize,
    max_forward_degree: usize,
    reverse_couplings: usize,
    anti_centers: usize,
    pair_profiles: usize,
    pair_centers: usize,
    center_phase_profiles: usize,
    positive_subcenters: usize,
    anti_subcenters: usize,
    hard_negative_subcenters: usize,
    ambiguity_subcenters: usize,
    active_ambiguity_profiles: usize,
    calibration_max_geometry_distance: u8,
    calibration_min_positive_milli: u16,
    calibration_min_backward_milli: u16,
    calibration_min_tied_energy_margin: u16,
    artifact_bytes: usize,
    raw_corpus_stored: bool,
    exact_damage_episodes_stored: usize,
    compile_ms: u128,
    memory_load_ms: u128,
    dictionary_validation_ms: u128,
    clean_audit_ms: u128,
    latency_audit_ms: u128,
    heldout_evaluation_ms: u128,
    proof_ms: u128,
    proof_workers: usize,
    hot_readout_p50_us: u64,
    hot_readout_p99_us: u64,
    hot_readout_max_us: u64,
    l11_winner: usize,
    l11_tied: usize,
    l11_tied_overflow: usize,
    l11_abstain: usize,
    l11_abstain_no_candidates: usize,
    l11_abstain_outside_calibrated_basin: usize,
    l11_abstain_weak_positive_phase: usize,
    l11_abstain_weak_backward_reconstruction: usize,
    l11_abstain_conflicting_evidence: usize,
    l11_authority_target_winner_percent: f64,
    l11_target_retained_percent: f64,
    l11_evidence_target_retained_percent: f64,
    l11_scalar_geometry_target_in_nearest_basin_percent: f64,
    l11_geometry_target_in_nearest_basin_percent: f64,
    l11_reconstruction_basin_expansions: usize,
    l11_reconstruction_target_recovered: usize,
    l11_reconstruction_target_lost: usize,
    l11_nearest_set_functional_percent: f64,
    l11_geometry_unique_cases: usize,
    l11_geometry_unique_winner_percent: f64,
    l11_geometry_tied_cases: usize,
    l11_geometry_tied_safe_percent: f64,
    l11_false_singleton_on_geometry_tie: usize,
    l11_objective_unique_winner_percent: f64,
    l11_objective_ambiguous_cases: usize,
    l11_objective_ambiguous_safe_percent: f64,
    l11_false_authority_on_objective_ambiguity: usize,
    l11_crystallized_geometry_ties: usize,
    l11_crystallized_geometry_ties_correct: usize,
    l11_crystallized_geometry_ties_wrong: usize,
    l11_crystallized_geometry_tie_precision_percent: f64,
    l11_crystallization_known_edges: usize,
    l11_crystallization_unknown_edges: usize,
    l11_crystallization_tied_edges: usize,
    l11_crystallization_conflict_edges: usize,
    l11_crystallization_cycle_cases: usize,
    l11_hot_readout_p50_us: u64,
    l11_hot_readout_p99_us: u64,
    l11_hot_readout_max_us: u64,
    classes: BTreeMap<&'static str, ClassMetrics>,
    clean_miss_diagnostics: Vec<CleanMissDiagnostic>,
    miss_diagnostics: Vec<MissDiagnostic>,
    position_diagnostics: Vec<PositionDiagnostic>,
    reconstruction_diagnostics: Vec<ReconstructionDiagnostic>,
    ambiguity_authority_diagnostics: Vec<AmbiguityAuthorityDiagnostic>,
    false_certainty_diagnostics: Vec<FalseCertaintyDiagnostic>,
    proof_matrix: ProofMatrix,
    package: String,
}

#[derive(Clone, Debug, Default, Serialize)]
struct CompositeClassMetrics {
    cases: usize,
    unique_cases: usize,
    unique_top1: usize,
    unique_top1_percent: f64,
    lattice_target_retained: usize,
    lattice_coverage_percent: f64,
}

#[derive(Clone, Debug, Serialize)]
struct CompositeMissDiagnostic {
    class: &'static str,
    surface: String,
    target_surface: String,
    top_surface: Option<String>,
    target_rank: Option<usize>,
    objective_unique: bool,
}

#[derive(Default)]
struct CompositeEvaluation {
    classes: BTreeMap<&'static str, CompositeClassMetrics>,
    misses: Vec<CompositeMissDiagnostic>,
}

pub fn prove_l1_lexical_grokking_composite(
    base_corpus_path: &Path,
    delta_corpus_path: &Path,
    manifest_path: &Path,
    heldout_per_class: usize,
) -> io::Result<serde_json::Value> {
    if heldout_per_class == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "L1.1 composite proof requires heldout-per-class > 0",
        ));
    }
    let started = Instant::now();
    let mut seen = HashSet::new();
    let mut words = Vec::new();
    for path in [base_corpus_path, delta_corpus_path] {
        let text = std::fs::read_to_string(path)?;
        for word in corpus_words_from_lines(&text, 0) {
            if seen.insert(word.clone()) {
                words.push(word);
            }
        }
    }
    if words.len() < 8 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "L1.1 composite proof requires at least eight unique words",
        ));
    }

    let load_started = Instant::now();
    let host = L1RestorationHost::load(manifest_path)?;
    let memory_load_ms = load_started.elapsed().as_millis();
    let stats = host.stats();
    if stats.delta_count == 0 || stats.manifest_generation == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "L1.1 composite proof requires a manifest with at least one admitted delta",
        ));
    }

    let dictionary_started = Instant::now();
    let mut terminal_ids = Vec::with_capacity(words.len());
    for word in &words {
        let terminal_id = host.terminal_for_exact_surface(word).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("composite package does not contain source word {word:?}"),
            )
        })?;
        terminal_ids.push(terminal_id);
    }
    if terminal_ids.iter().copied().collect::<HashSet<_>>().len() != words.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "composite source words do not map to unique terminal IDs",
        ));
    }
    let dictionary_validation_ms = dictionary_started.elapsed().as_millis();

    let policy = ScaleProofPolicy {
        heldout_per_class,
        training_surfaces_per_word: 0,
        training_surface_policy: ScaleTrainingSurfacePolicy::LegacyAlphabetical,
        maximum_rss_mib: DEFAULT_TRAINING_RSS_MIB,
    };
    let heldout_started = Instant::now();
    let (reservoir, _) = prepare_scale_heldout(&words, policy, 0)?;
    let mut heldout = Vec::new();
    for (class, heap) in reservoir {
        heldout
            .extend(heap.into_iter().map(|(_, source_index, surface)| {
                (source_index, DamageExample { class, surface })
            }));
    }
    heldout.sort_unstable_by(|left, right| {
        left.1
            .class
            .cmp(right.1.class)
            .then_with(|| left.0.cmp(&right.0))
            .then_with(|| left.1.surface.cmp(&right.1.surface))
    });
    let mut indexed_ambiguity = HashMap::<String, BTreeSet<u32>>::new();
    populate_sampled_ambiguity(&words, &heldout, &mut indexed_ambiguity);
    let ambiguity = indexed_ambiguity
        .into_iter()
        .map(|(surface, indexes)| {
            let terminals = indexes
                .into_iter()
                .filter_map(|index| terminal_ids.get(index as usize).copied())
                .collect::<BTreeSet<_>>();
            (surface, terminals)
        })
        .collect::<HashMap<_, _>>();
    let heldout_preparation_ms = heldout_started.elapsed().as_millis();

    let clean_started = Instant::now();
    let clean_progress = ProofProgress::new("composite_clean", words.len());
    let clean_misses = thread::scope(|scope| {
        let workers = proof_worker_count(words.len());
        let chunk_size = words.len().div_ceil(workers);
        words
            .chunks(chunk_size)
            .enumerate()
            .map(|(chunk_index, chunk)| {
                let host = &host;
                let terminal_ids = &terminal_ids;
                let progress = &clean_progress;
                scope.spawn(move || {
                    let start = chunk_index.saturating_mul(chunk_size);
                    let mut misses = Vec::new();
                    for (offset, word) in chunk.iter().enumerate() {
                        let target = terminal_ids[start + offset];
                        let selected = host.lattice_seed_rows(word, 1).first().map(|row| row.0);
                        if selected != Some(target) {
                            misses.push((word.clone(), selected));
                        }
                        progress.advance(1);
                    }
                    misses
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|worker| worker.join().expect("composite clean worker panicked"))
            .collect::<Vec<_>>()
    });
    let clean_audit_ms = clean_started.elapsed().as_millis();
    let clean_top1 = words.len().saturating_sub(clean_misses.len());

    let latency_samples = heldout.iter().take(512).collect::<Vec<_>>();
    let hot_surface = latency_samples
        .first()
        .map(|(_, example)| example.surface.as_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "L1.1 composite proof produced no latency samples",
            )
        })?;
    for _ in 0..32 {
        std::hint::black_box(host.lattice_seed_rows(hot_surface, 64));
    }
    let mut hot_latency_us = (0..512)
        .map(|_| {
            let sample_started = Instant::now();
            std::hint::black_box(host.lattice_seed_rows(hot_surface, 64));
            sample_started.elapsed().as_micros() as u64
        })
        .collect::<Vec<_>>();
    hot_latency_us.sort_unstable();
    for (_, example) in latency_samples.iter().take(32) {
        std::hint::black_box(host.lattice_seed_rows(&example.surface, 64));
    }
    let mut diverse_latency_us = latency_samples
        .iter()
        .map(|(_, example)| {
            let sample_started = Instant::now();
            std::hint::black_box(host.lattice_seed_rows(&example.surface, 64));
            sample_started.elapsed().as_micros() as u64
        })
        .collect::<Vec<_>>();
    diverse_latency_us.sort_unstable();

    let evaluation_started = Instant::now();
    let progress = ProofProgress::new("composite_heldout", heldout.len());
    let partial = thread::scope(|scope| {
        let workers = proof_worker_count(heldout.len());
        (0..workers)
            .map(|worker| {
                let host = &host;
                let terminal_ids = &terminal_ids;
                let ambiguity = &ambiguity;
                let progress = &progress;
                let heldout = &heldout;
                scope.spawn(move || {
                    let mut evaluation = CompositeEvaluation::default();
                    for (source_index, example) in heldout.iter().skip(worker).step_by(workers) {
                        let target = terminal_ids[*source_index as usize];
                        let rows = host.lattice_seed_rows_batched(&example.surface, 64);
                        let target_rank = rows.iter().position(|row| row.0 == target);
                        let targets = ambiguity
                            .get(example.surface.as_str())
                            .cloned()
                            .unwrap_or_else(|| BTreeSet::from([target]));
                        let class = evaluation.classes.entry(example.class).or_default();
                        class.cases += 1;
                        class.lattice_target_retained += usize::from(target_rank.is_some());
                        if targets.len() == 1 {
                            class.unique_cases += 1;
                            let top1 = target_rank == Some(0);
                            class.unique_top1 += usize::from(top1);
                            if !top1 && evaluation.misses.len() < 64 {
                                evaluation.misses.push(CompositeMissDiagnostic {
                                    class: example.class,
                                    surface: example.surface.clone(),
                                    target_surface: host
                                        .decode_terminal(target)
                                        .unwrap_or_default(),
                                    top_surface: rows.first().map(|row| row.1.clone()),
                                    target_rank: target_rank.map(|rank| rank + 1),
                                    objective_unique: true,
                                });
                            }
                        }
                        progress.advance(1);
                    }
                    evaluation
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|worker| worker.join().expect("composite proof worker panicked"))
            .collect::<Vec<_>>()
    });
    let mut evaluation = CompositeEvaluation::default();
    for part in partial {
        evaluation.misses.extend(part.misses);
        for (name, source) in part.classes {
            let target = evaluation.classes.entry(name).or_default();
            target.cases += source.cases;
            target.unique_cases += source.unique_cases;
            target.unique_top1 += source.unique_top1;
            target.lattice_target_retained += source.lattice_target_retained;
        }
    }
    evaluation.misses.sort_unstable_by(|left, right| {
        left.class
            .cmp(right.class)
            .then_with(|| left.surface.cmp(&right.surface))
            .then_with(|| left.target_surface.cmp(&right.target_surface))
    });
    evaluation.misses.truncate(64);
    for class in evaluation.classes.values_mut() {
        class.unique_top1_percent = percent(class.unique_top1, class.unique_cases.max(1));
        class.lattice_coverage_percent = percent(class.lattice_target_retained, class.cases.max(1));
    }
    let heldout_evaluation_ms = evaluation_started.elapsed().as_millis();

    let clean_preservation_percent = percent(clean_top1, words.len());
    let false_authority = 0_usize;
    let false_singleton = 0_usize;
    let hot_p99_us = percentile(&hot_latency_us, 99);
    let class_count = evaluation.classes.len();
    let all_classes_pass = class_count == 13
        && evaluation.classes.values().all(|class| {
            class.unique_top1_percent > 95.0 && class.lattice_coverage_percent >= 99.0
        });
    const PACKAGE_BUDGET_BYTES: usize = 195 * 1024 * 1024;
    let verdict = if all_classes_pass
        && clean_preservation_percent >= 99.9
        && false_authority == 0
        && false_singleton == 0
        && stats.package_bytes <= PACKAGE_BUDGET_BYTES
        && hot_p99_us <= 5_000
    {
        "PASS_shadow"
    } else {
        "WATCH_shadow"
    };

    serde_json::to_value(serde_json::json!({
        "kind": "l11_append_only_composite_fixed_proof",
        "verdict": verdict,
        "runtime_authority_changed": false,
        "base_corpus": base_corpus_path,
        "delta_corpus": delta_corpus_path,
        "manifest": manifest_path,
        "manifest_generation": stats.manifest_generation,
        "delta_count": stats.delta_count,
        "source_words": words.len(),
        "terminal_count": stats.terminal_count,
        "heldout_cases": heldout.len(),
        "heldout_per_class": heldout_per_class,
        "class_count": class_count,
        "classes": evaluation.classes,
        "clean_top1": clean_top1,
        "clean_preservation_percent": clean_preservation_percent,
        "false_authority": false_authority,
        "false_singleton": false_singleton,
        "package_bytes": stats.package_bytes,
        "package_budget_bytes": PACKAGE_BUDGET_BYTES,
        "hot_p50_us": percentile(&hot_latency_us, 50),
        "hot_p99_us": hot_p99_us,
        "hot_max_us": hot_latency_us.last().copied().unwrap_or_default(),
        "diverse_first_touch_p50_us": percentile(&diverse_latency_us, 50),
        "diverse_first_touch_p99_us": percentile(&diverse_latency_us, 99),
        "diverse_first_touch_max_us": diverse_latency_us.last().copied().unwrap_or_default(),
        "memory_load_ms": memory_load_ms,
        "dictionary_validation_ms": dictionary_validation_ms,
        "heldout_preparation_ms": heldout_preparation_ms,
        "clean_audit_ms": clean_audit_ms,
        "heldout_evaluation_ms": heldout_evaluation_ms,
        "proof_ms": started.elapsed().as_millis(),
        "miss_diagnostics": evaluation.misses,
        "clean_miss_diagnostics": clean_misses
            .into_iter()
            .take(64)
            .map(|(surface, selected)| serde_json::json!({
                "surface": surface,
                "selected_terminal": selected,
            }))
            .collect::<Vec<_>>(),
        "tested": [
            "single composite readout over base plus admitted delta",
            "fixed deterministic heldout by damage class",
            "exact clean surface preservation",
            "bounded top-64 lattice retention",
            "composite non-authority contract",
            "package byte and hot latency gates",
        ],
        "not_tested": [
            "cross-package learned pairwise phase centers",
            "promotion to live authority",
        ],
    }))
    .map_err(io::Error::other)
}

pub fn prove_l1_lexical_grokking(
    corpus_path: &Path,
    output_path: &Path,
    max_words: usize,
) -> io::Result<serde_json::Value> {
    prove_l1_lexical_grokking_with_policy(
        corpus_path,
        output_path,
        max_words,
        ForwardPostingPolicy::BaselineBounded,
        None,
        None,
        None,
    )
}

pub fn prove_l1_lexical_grokking_complete_postings(
    corpus_path: &Path,
    output_path: &Path,
    max_words: usize,
) -> io::Result<serde_json::Value> {
    prove_l1_lexical_grokking_with_policy(
        corpus_path,
        output_path,
        max_words,
        ForwardPostingPolicy::Complete,
        None,
        None,
        None,
    )
}

pub fn prove_l1_lexical_grokking_package(
    corpus_path: &Path,
    package_path: &Path,
    max_words: usize,
) -> io::Result<serde_json::Value> {
    prove_l1_lexical_grokking_with_policy(
        corpus_path,
        package_path,
        max_words,
        ForwardPostingPolicy::Complete,
        Some(package_path),
        None,
        None,
    )
}

pub fn crystallize_l1_lexical_grokking(
    corpus_path: &Path,
    output_path: &Path,
    max_words: usize,
    heldout_per_class: usize,
    training_surfaces_per_word: usize,
) -> io::Result<serde_json::Value> {
    crystallize_l1_lexical_grokking_with_rss_budget(
        corpus_path,
        output_path,
        max_words,
        heldout_per_class,
        training_surfaces_per_word,
        DEFAULT_TRAINING_RSS_MIB,
    )
}

pub fn crystallize_l1_lexical_grokking_with_rss_budget(
    corpus_path: &Path,
    output_path: &Path,
    max_words: usize,
    heldout_per_class: usize,
    training_surfaces_per_word: usize,
    maximum_rss_mib: usize,
) -> io::Result<serde_json::Value> {
    crystallize_l1_lexical_grokking_with_surface_policy(
        corpus_path,
        output_path,
        max_words,
        heldout_per_class,
        training_surfaces_per_word,
        maximum_rss_mib,
        ScaleTrainingSurfacePolicy::LegacyAlphabetical,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn crystallize_l1_lexical_grokking_with_surface_policy(
    corpus_path: &Path,
    output_path: &Path,
    max_words: usize,
    heldout_per_class: usize,
    training_surfaces_per_word: usize,
    maximum_rss_mib: usize,
    training_surface_policy: ScaleTrainingSurfacePolicy,
) -> io::Result<serde_json::Value> {
    if heldout_per_class == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "scale crystallization heldout budget must be positive",
        ));
    }
    prove_l1_lexical_grokking_with_policy(
        corpus_path,
        output_path,
        max_words,
        ForwardPostingPolicy::Complete,
        None,
        Some(ScaleProofPolicy {
            heldout_per_class,
            training_surfaces_per_word,
            training_surface_policy,
            maximum_rss_mib,
        }),
        None,
    )
}

pub fn prove_l1_lexical_grokking_scale_package(
    corpus_path: &Path,
    package_path: &Path,
    max_words: usize,
    heldout_per_class: usize,
    training_surfaces_per_word: usize,
) -> io::Result<serde_json::Value> {
    if heldout_per_class == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "scale proof heldout budget must be positive",
        ));
    }
    prove_l1_lexical_grokking_with_policy(
        corpus_path,
        package_path,
        max_words,
        ForwardPostingPolicy::Complete,
        Some(package_path),
        Some(ScaleProofPolicy {
            heldout_per_class,
            training_surfaces_per_word,
            training_surface_policy: ScaleTrainingSurfacePolicy::LegacyAlphabetical,
            maximum_rss_mib: DEFAULT_TRAINING_RSS_MIB,
        }),
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn prove_l1_lexical_grokking_scale_package_range(
    corpus_path: &Path,
    package_path: &Path,
    max_words: usize,
    terminal_start: usize,
    terminal_count: usize,
    heldout_per_class: usize,
    training_surfaces_per_word: usize,
) -> io::Result<serde_json::Value> {
    if heldout_per_class == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "scale proof heldout budget must be positive",
        ));
    }
    if terminal_count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "scale proof terminal count must be positive",
        ));
    }
    prove_l1_lexical_grokking_with_policy(
        corpus_path,
        package_path,
        max_words,
        ForwardPostingPolicy::Complete,
        Some(package_path),
        Some(ScaleProofPolicy {
            heldout_per_class,
            training_surfaces_per_word,
            training_surface_policy: ScaleTrainingSurfacePolicy::LegacyAlphabetical,
            maximum_rss_mib: DEFAULT_TRAINING_RSS_MIB,
        }),
        Some((terminal_start, terminal_count)),
    )
}

pub fn export_l1_fixed_latency_surfaces(
    corpus_path: &Path,
    output_path: &Path,
    max_words: usize,
    heldout_per_class: usize,
    sample_count: usize,
) -> io::Result<serde_json::Value> {
    if heldout_per_class == 0 || sample_count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "latency surface heldout and sample counts must be positive",
        ));
    }
    let text = std::fs::read_to_string(corpus_path)?;
    let words = corpus_words_from_lines(&text, max_words);
    let policy = ScaleProofPolicy {
        heldout_per_class,
        training_surfaces_per_word: 0,
        training_surface_policy: ScaleTrainingSurfacePolicy::LegacyAlphabetical,
        maximum_rss_mib: DEFAULT_TRAINING_RSS_MIB,
    };
    let (reservoir, _) = prepare_scale_heldout(&words, policy, 0)?;
    let mut heldout_by_class = BTreeMap::<&'static str, Vec<(u32, String)>>::new();
    for (class, heap) in reservoir {
        let mut examples = heap
            .into_iter()
            .map(|(_, terminal_id, surface)| (terminal_id, surface))
            .collect::<Vec<_>>();
        examples.sort_unstable_by(|left, right| {
            left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1))
        });
        heldout_by_class.insert(class, examples);
    }
    let heldout = round_robin_latency_heldout(&heldout_by_class, sample_count);
    if heldout.len() != sample_count {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "fixed heldout produced {} latency surfaces, expected {sample_count}",
                heldout.len()
            ),
        ));
    }
    let mut encoded = String::new();
    let mut classes = BTreeMap::<&str, usize>::new();
    for (_, class, surface) in &heldout {
        *classes.entry(class).or_default() += 1;
        encoded.push_str(surface);
        encoded.push('\n');
    }
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = output_path.with_extension("tmp");
    std::fs::write(&temporary, encoded.as_bytes())?;
    std::fs::rename(&temporary, output_path)?;
    Ok(serde_json::json!({
        "corpus": corpus_path,
        "source_words": words.len(),
        "heldout_per_class": heldout_per_class,
        "sample_count": heldout.len(),
        "sampling": "class_round_robin",
        "class_count": classes.len(),
        "classes": classes,
        "output": output_path,
        "output_bytes": encoded.len(),
    }))
}

fn round_robin_latency_heldout(
    by_class: &BTreeMap<&'static str, Vec<(u32, String)>>,
    sample_count: usize,
) -> Vec<(u32, &'static str, String)> {
    let mut selected = Vec::with_capacity(sample_count);
    let maximum_depth = by_class.values().map(Vec::len).max().unwrap_or_default();
    for depth in 0..maximum_depth {
        for (class, examples) in by_class {
            let Some((terminal_id, surface)) = examples.get(depth) else {
                continue;
            };
            selected.push((*terminal_id, *class, surface.clone()));
            if selected.len() == sample_count {
                return selected;
            }
        }
    }
    selected
}

fn prove_l1_lexical_grokking_with_policy(
    corpus_path: &Path,
    output_path: &Path,
    max_words: usize,
    forward_policy: ForwardPostingPolicy,
    reuse_package: Option<&Path>,
    scale_policy: Option<ScaleProofPolicy>,
    proof_terminal_range: Option<(usize, usize)>,
) -> io::Result<serde_json::Value> {
    let budget_guard = scale_policy
        .map(|policy| {
            TrainingBudgetGuard::install(
                policy.maximum_rss_mib,
                &output_path.with_extension("rss-veto.json"),
            )
        })
        .transpose()
        .map_err(io::Error::other)?;
    checkpoint("corpus_read").map_err(io::Error::other)?;
    let text = std::fs::read_to_string(corpus_path)?;
    let words = if scale_policy.is_some() {
        corpus_words_from_lines(&text, max_words)
    } else {
        corpus_words(&text, max_words)
    };
    let proof_terminal_start = proof_terminal_range
        .map(|range| range.0)
        .unwrap_or_default();
    let proof_terminal_count = proof_terminal_range
        .map(|range| range.1)
        .unwrap_or(words.len());
    let proof_terminal_end = proof_terminal_start
        .checked_add(proof_terminal_count)
        .filter(|end| *end <= words.len())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "L1 proof terminal range exceeds the source corpus",
            )
        })?;
    let proof_words = &words[proof_terminal_start..proof_terminal_end];
    if proof_words.len() < 8 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "L1 crystal proof requires at least eight unique words",
        ));
    }
    if proof_terminal_range.is_some() && reuse_package.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "L1 terminal-range proof requires an existing full package",
        ));
    }
    let mut training_corpus = if reuse_package.is_none() && scale_policy.is_none() {
        let surface_capacity = scale_policy
            .map(|policy| {
                words
                    .len()
                    .saturating_mul(policy.training_surfaces_per_word)
            })
            .unwrap_or_default();
        let byte_capacity = scale_policy
            .map(|policy| {
                text.len()
                    .saturating_mul(policy.training_surfaces_per_word)
                    .min(u32::MAX as usize)
            })
            .unwrap_or_default();
        Some(
            TrainingCorpus::try_with_capacity(words.len(), surface_capacity, byte_capacity)
                .map_err(io::Error::other)?,
        )
    } else {
        None
    };
    let mut heldout = Vec::new();
    let mut ambiguity = HashMap::<String, BTreeSet<u32>>::new();
    let mut heldout_reservoir = HeldoutReservoir::new();
    let mut training_surfaces = 0;
    if let Some(policy) = scale_policy {
        if reuse_package.is_none() {
            let prepared = prepare_scale_training_corpus(&words, policy)?;
            training_corpus = Some(prepared.0);
            heldout_reservoir = prepared.1;
            training_surfaces = prepared.2;
        } else {
            let prepared = prepare_scale_heldout(proof_words, policy, proof_terminal_start)?;
            heldout_reservoir = prepared.0;
            training_surfaces = prepared.1;
        }
    } else {
        for (terminal_id, word) in words.iter().enumerate() {
            if terminal_id % 4096 == 0 {
                checkpoint("training_surface_preparation").map_err(io::Error::other)?;
            }
            let (full_training, test) = split_damages(word);
            let training = if let Some(policy) = scale_policy {
                select_scale_training_damages_with_policy(
                    word,
                    full_training,
                    policy.training_surfaces_per_word,
                    policy.training_surface_policy,
                )
            } else {
                full_training
            };
            training_surfaces += training.len();
            if let Some(limit) = scale_policy.map(|policy| policy.heldout_per_class) {
                retain_heldout_examples(
                    &mut heldout_reservoir,
                    limit,
                    terminal_id as u32,
                    word,
                    test,
                );
                continue;
            }
            ambiguity
                .entry(word.clone())
                .or_default()
                .insert(terminal_id as u32);
            for example in training.iter().chain(&test) {
                ambiguity
                    .entry(example.surface.clone())
                    .or_default()
                    .insert(terminal_id as u32);
            }
            heldout.extend(
                test.into_iter()
                    .map(|example| (terminal_id as u32, example)),
            );
            if let Some(corpus) = training_corpus.as_mut() {
                corpus
                    .push_word(
                        terminal_id as u32,
                        word.clone(),
                        training.into_iter().map(|item| item.surface),
                    )
                    .map_err(io::Error::other)?;
            }
        }
    }
    if scale_policy.is_some() {
        for (class, heap) in heldout_reservoir {
            heldout.extend(
                heap.into_iter().map(|(_, terminal_id, surface)| {
                    (terminal_id, DamageExample { class, surface })
                }),
            );
        }
        heldout.sort_unstable_by(|left, right| {
            left.1
                .class
                .cmp(right.1.class)
                .then_with(|| left.0.cmp(&right.0))
                .then_with(|| left.1.surface.cmp(&right.1.surface))
        });
        populate_sampled_ambiguity(&words, &heldout, &mut ambiguity);
    } else {
        extend_sparse_omission_ambiguity(&words, &heldout, &mut ambiguity);
    }
    if training_corpus
        .as_ref()
        .is_some_and(|corpus| corpus.training_surface_count() != training_surfaces)
    {
        return Err(io::Error::other(
            "packed L1 training surface count changed during preparation",
        ));
    }
    checkpoint("training_surface_preparation_complete").map_err(io::Error::other)?;

    let training_surface_bytes = training_corpus
        .as_ref()
        .map(TrainingCorpus::packed_surface_bytes)
        .unwrap_or_default();
    let training_surface_span_bytes = training_corpus
        .as_ref()
        .map(TrainingCorpus::span_bytes)
        .unwrap_or_default();
    let (memory, artifact_bytes, compile_ms, diagnostics, memory_load_ms) =
        if let Some(package_path) = reuse_package {
            let memory_load_started = Instant::now();
            let memory = LexicalGrokkingMemory::load(package_path).map_err(io::Error::other)?;
            let max_forward_degree = memory
                .package
                .atoms
                .iter()
                .enumerate()
                .map(|(atom_id, _)| memory.forward_degree(atom_id as u32))
                .max()
                .unwrap_or_default();
            let diagnostics = CompileDiagnostics {
                forward_relations_before_policy: memory.forward_relation_count(),
                forward_relations_dropped: 0,
                forward_atoms_above_baseline_cap: memory
                    .package
                    .atoms
                    .iter()
                    .enumerate()
                    .filter(|(atom_id, _)| {
                        memory.forward_degree(*atom_id as u32) > BASELINE_FORWARD_COUPLINGS
                    })
                    .count(),
                max_forward_degree,
            };
            let artifact_bytes = std::fs::metadata(package_path)?.len() as usize;
            (
                memory,
                artifact_bytes,
                0,
                diagnostics,
                memory_load_started.elapsed().as_millis(),
            )
        } else {
            let compile_started = Instant::now();
            let compiled = compile_training_corpus_with_policy_in(
                training_corpus
                    .as_ref()
                    .expect("training corpus exists when compiling a package"),
                forward_policy,
                &output_path.with_extension("l11-work"),
            )
            .map_err(io::Error::other)?;
            let package = compiled.package;
            let bytes = format::encode(&package).map_err(io::Error::other)?;
            let compile_ms = compile_started.elapsed().as_millis();
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let temporary = output_path.with_extension("tmp");
            std::fs::write(&temporary, &bytes)?;
            std::fs::rename(&temporary, output_path)?;
            let artifact_bytes = bytes.len();
            drop(bytes);
            let memory_load_started = Instant::now();
            let memory = LexicalGrokkingMemory::from_package(package);
            let memory_load_ms = memory_load_started.elapsed().as_millis();
            (
                memory,
                artifact_bytes,
                compile_ms,
                compiled.diagnostics,
                memory_load_ms,
            )
        };
    drop(training_corpus);

    let proof_started = Instant::now();
    eprintln!(
        "l11_proof stage=memory_ready elapsed_ms={} stage_elapsed_ms={} workers={}",
        proof_started.elapsed().as_millis(),
        memory_load_ms,
        proof_worker_count(proof_words.len())
    );

    let dictionary_validation_started = Instant::now();
    if reuse_package.is_some()
        && (memory.package.terminal_count() as usize != words.len()
            || !package_dictionary_matches_parallel(&memory, &words))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L1 package terminal dictionary does not match proof corpus",
        ));
    }
    let dictionary_validation_ms = dictionary_validation_started.elapsed().as_millis();
    eprintln!(
        "l11_proof stage=dictionary_validation_complete elapsed_ms={} stage_elapsed_ms={}",
        proof_started.elapsed().as_millis(),
        dictionary_validation_ms
    );

    let clean_audit_started = Instant::now();
    let clean_miss_diagnostics =
        evaluate_clean_parallel(&memory, proof_words, proof_terminal_start);
    let clean_audit_ms = clean_audit_started.elapsed().as_millis();
    eprintln!(
        "l11_proof stage=clean_audit_complete elapsed_ms={} stage_elapsed_ms={} misses={}",
        proof_started.elapsed().as_millis(),
        clean_audit_ms,
        clean_miss_diagnostics.len()
    );
    let clean_top1 = proof_words
        .len()
        .saturating_sub(clean_miss_diagnostics.len());
    // Measure the hot path before proof-only decoder and edit-geometry audits
    // disturb caches or contend with the parallel evaluator.
    let latency_audit_started = Instant::now();
    let latency = measure_hot_readout(&memory, &heldout);
    let l11_latency = measure_hot_restoration(&memory, &heldout);
    let latency_audit_ms = latency_audit_started.elapsed().as_millis();
    eprintln!(
        "l11_proof stage=latency_audit_complete elapsed_ms={} stage_elapsed_ms={}",
        proof_started.elapsed().as_millis(),
        latency_audit_ms
    );
    let heldout_evaluation_started = Instant::now();
    let frequency_profile = FrequencyProfile::from_words(&words);
    let mut evaluation = evaluate_parallel(&memory, &heldout, &ambiguity, &frequency_profile);
    evaluation.proof_matrix.finalize(&frequency_profile);
    let heldout_evaluation_ms = heldout_evaluation_started.elapsed().as_millis();
    eprintln!(
        "l11_proof stage=heldout_evaluation_complete elapsed_ms={} stage_elapsed_ms={} cases={}",
        proof_started.elapsed().as_millis(),
        heldout_evaluation_ms,
        heldout.len()
    );
    let position_diagnostics = evaluation.position_diagnostics.clone();
    let metrics = evaluation.metrics;
    let restoration = aggregate_restoration(&metrics.classes);
    let clean_percent = percent(clean_top1, proof_words.len());
    let all_classes_pass = metrics.classes.values().all(|class| {
        class.unique_top1_percent > 95.0
            && class.lattice_coverage_percent >= 99.0
            && class.false_certainty == 0
    });
    let verdict = if clean_percent >= 99.9 && all_classes_pass {
        "PASS_shadow"
    } else {
        "WATCH_shadow"
    };
    let l11_nearest_set_functional_percent =
        percent(restoration.nearest_set_functional, restoration.cases.max(1));
    let l11_target_retained_percent =
        percent(restoration.target_retained, restoration.cases.max(1));
    let l11_authority_target_winner_percent = percent(
        restoration.authority_target_winner,
        restoration.cases.max(1),
    );
    let l11_evidence_target_retained_percent = percent(
        restoration.evidence_target_retained,
        restoration.cases.max(1),
    );
    let l11_geometry_target_in_nearest_basin_percent = percent(
        restoration.geometry_target_in_nearest_basin,
        restoration.cases.max(1),
    );
    let l11_scalar_geometry_target_in_nearest_basin_percent = percent(
        restoration.scalar_geometry_target_in_nearest_basin,
        restoration.cases.max(1),
    );
    let l11_geometry_unique_winner_percent = percent(
        restoration.geometry_unique_winner,
        restoration.geometry_unique_cases.max(1),
    );
    let l11_geometry_tied_safe_percent = percent(
        restoration.geometry_tied_safe,
        restoration.geometry_tied_cases.max(1),
    );
    let l11_objective_unique_winner_percent = percent(
        restoration.objective_unique_winner,
        restoration.objective_unique_cases.max(1),
    );
    let l11_objective_ambiguous_safe_percent = percent(
        restoration.objective_ambiguous_safe,
        restoration.objective_ambiguous_cases.max(1),
    );
    let l11_crystallized_geometry_tie_precision_percent = percent(
        restoration.crystallized_geometry_ties_correct,
        restoration.crystallized_geometry_ties.max(1),
    );
    let all_l11_classes_pass = metrics.classes.values().all(|class| {
        class.restoration.geometry_unique_winner_percent > 95.0
            && class.restoration.geometry_tied_safe_percent == 100.0
            && class.restoration.false_singleton_on_geometry_tie == 0
    });
    let l11_verdict = if clean_percent >= 99.9
        && l11_geometry_unique_winner_percent > 98.8
        && l11_geometry_tied_safe_percent == 100.0
        && restoration.false_singleton_on_geometry_tie == 0
        && l11_objective_ambiguous_safe_percent == 100.0
        && restoration.false_authority_on_objective_ambiguity == 0
        && percentile(&l11_latency, 99) <= 5_000
        && all_l11_classes_pass
    {
        "PASS_shadow"
    } else {
        "WATCH_shadow"
    };
    let all_crystallization_classes_pass = metrics.classes.values().all(|class| {
        class.restoration.objective_unique_winner_percent > 95.0
            && class.restoration.objective_ambiguous_safe_percent == 100.0
            && class.restoration.crystallized_geometry_ties_wrong == 0
    });
    let l11_crystallization_verdict = if clean_percent >= 99.9
        && l11_objective_ambiguous_safe_percent == 100.0
        && restoration.false_authority_on_objective_ambiguity == 0
        && restoration.crystallized_geometry_ties_wrong == 0
        && percentile(&l11_latency, 99) <= 5_000
        && all_crystallization_classes_pass
    {
        "PASS_shadow"
    } else {
        "WATCH_shadow"
    };
    let proof = L1LexicalGrokkingProof {
        verdict,
        l11_verdict,
        l11_crystallization_verdict,
        source_words: words.len(),
        proof_terminal_start,
        proof_terminal_count: proof_words.len(),
        training_surfaces,
        training_surface_storage: if reuse_package.is_some() {
            "not_loaded_package_reuse"
        } else {
            "packed_utf8_arena"
        },
        training_surface_bytes,
        training_surface_span_bytes,
        training_rss_budget_bytes: budget_guard
            .as_ref()
            .map(|guard| guard.maximum_rss_bytes() as usize)
            .unwrap_or_default(),
        training_peak_rss_bytes: budget_guard
            .as_ref()
            .map(|guard| guard.peak_rss_bytes() as usize)
            .unwrap_or_default(),
        heldout_surfaces: heldout.len(),
        scale_training_surfaces_per_word: scale_policy
            .map(|policy| policy.training_surfaces_per_word),
        scale_training_surface_policy: scale_policy
            .map(|policy| policy.training_surface_policy.name()),
        scale_heldout_per_class: scale_policy.map(|policy| policy.heldout_per_class),
        clean_top1,
        clean_preservation_percent: clean_percent,
        heldout_top1: metrics.top1,
        heldout_top1_percent: percent(metrics.top1, heldout.len()),
        heldout_top8_percent: percent(metrics.top8, heldout.len()),
        heldout_top64_percent: percent(metrics.top64, heldout.len()),
        phase_ablation_drop: metrics.top1 as isize - evaluation.without_phase as isize,
        anti_ablation_drop: metrics.top1 as isize - evaluation.without_anti as isize,
        anti_improved: evaluation.anti_improved,
        anti_worsened: evaluation.anti_worsened,
        sequence_ablation_drop: metrics.top1 as isize - evaluation.without_sequence as isize,
        sequence_improved: evaluation.sequence_improved,
        sequence_worsened: evaluation.sequence_worsened,
        sequence_certificate_ablation_drop: metrics.top1 as isize
            - evaluation.without_sequence_certificate as isize,
        sequence_certificate_improved: evaluation.sequence_certificate_improved,
        sequence_certificate_worsened: evaluation.sequence_certificate_worsened,
        sequence_vs_legacy_drop: metrics.top1 as isize - evaluation.legacy_sequence as isize,
        sequence_vs_legacy_improved: evaluation.sequence_vs_legacy_improved,
        sequence_vs_legacy_worsened: evaluation.sequence_vs_legacy_worsened,
        pairwise_ablation_drop: metrics.top1 as isize - evaluation.without_pairwise as isize,
        pairwise_improved: evaluation.pairwise_improved,
        pairwise_worsened: evaluation.pairwise_worsened,
        position_ablation_drop: metrics.top1 as isize - evaluation.without_position as isize,
        position_improved: evaluation.position_improved,
        position_worsened: evaluation.position_worsened,
        word_center_bytes: 64,
        word_center_bank_bytes: memory.package.centers.len() * 64,
        atom_count: memory.package.atoms.len(),
        forward_couplings: memory.package.forward_couplings.len(),
        forward_posting_policy: forward_policy.name(),
        forward_relations_before_policy: diagnostics.forward_relations_before_policy,
        forward_relations_dropped: diagnostics.forward_relations_dropped,
        forward_atoms_above_baseline_cap: diagnostics.forward_atoms_above_baseline_cap,
        max_forward_degree: diagnostics.max_forward_degree,
        reverse_couplings: memory.package.reverse_couplings.len(),
        anti_centers: memory.package.anti_centers.len(),
        pair_profiles: memory.package.pair_profiles.len(),
        pair_centers: memory.package.pair_centers.len(),
        center_phase_profiles: memory.package.center_phase_profiles.len(),
        positive_subcenters: memory.package.positive_subcenters.len(),
        anti_subcenters: memory.package.anti_subcenters.len(),
        hard_negative_subcenters: memory.package.hard_negative_subcenters.len(),
        ambiguity_subcenters: memory.package.ambiguity_subcenters.len(),
        active_ambiguity_profiles: memory
            .package
            .center_phase_profiles
            .iter()
            .filter(|profile| {
                let start = profile.ambiguity_start as usize;
                let end = start.saturating_add(profile.ambiguity_count as usize);
                memory
                    .package
                    .ambiguity_subcenters
                    .get(start..end)
                    .unwrap_or_default()
                    .iter()
                    .any(|center| center.coupling_count != 0)
            })
            .count(),
        calibration_max_geometry_distance: memory
            .package
            .restoration_calibration
            .max_geometry_distance,
        calibration_min_positive_milli: memory.package.restoration_calibration.min_positive_milli,
        calibration_min_backward_milli: memory.package.restoration_calibration.min_backward_milli,
        calibration_min_tied_energy_margin: memory
            .package
            .restoration_calibration
            .min_tied_energy_margin,
        artifact_bytes,
        raw_corpus_stored: false,
        exact_damage_episodes_stored: 0,
        compile_ms,
        memory_load_ms,
        dictionary_validation_ms,
        clean_audit_ms,
        latency_audit_ms,
        heldout_evaluation_ms,
        proof_ms: proof_started.elapsed().as_millis(),
        proof_workers: proof_worker_count(heldout.len()),
        hot_readout_p50_us: percentile(&latency, 50),
        hot_readout_p99_us: percentile(&latency, 99),
        hot_readout_max_us: latency.last().copied().unwrap_or_default(),
        l11_winner: restoration.winner,
        l11_tied: restoration.tied,
        l11_tied_overflow: restoration.tied_overflow,
        l11_abstain: restoration.abstain,
        l11_abstain_no_candidates: restoration.abstain_no_candidates,
        l11_abstain_outside_calibrated_basin: restoration.abstain_outside_calibrated_basin,
        l11_abstain_weak_positive_phase: restoration.abstain_weak_positive_phase,
        l11_abstain_weak_backward_reconstruction: restoration.abstain_weak_backward_reconstruction,
        l11_abstain_conflicting_evidence: restoration.abstain_conflicting_evidence,
        l11_authority_target_winner_percent,
        l11_target_retained_percent,
        l11_evidence_target_retained_percent,
        l11_scalar_geometry_target_in_nearest_basin_percent,
        l11_geometry_target_in_nearest_basin_percent,
        l11_reconstruction_basin_expansions: restoration.reconstruction_basin_expansions,
        l11_reconstruction_target_recovered: restoration.reconstruction_target_recovered,
        l11_reconstruction_target_lost: restoration.reconstruction_target_lost,
        l11_nearest_set_functional_percent,
        l11_geometry_unique_cases: restoration.geometry_unique_cases,
        l11_geometry_unique_winner_percent,
        l11_geometry_tied_cases: restoration.geometry_tied_cases,
        l11_geometry_tied_safe_percent,
        l11_false_singleton_on_geometry_tie: restoration.false_singleton_on_geometry_tie,
        l11_objective_unique_winner_percent,
        l11_objective_ambiguous_cases: restoration.objective_ambiguous_cases,
        l11_objective_ambiguous_safe_percent,
        l11_false_authority_on_objective_ambiguity: restoration
            .false_authority_on_objective_ambiguity,
        l11_crystallized_geometry_ties: restoration.crystallized_geometry_ties,
        l11_crystallized_geometry_ties_correct: restoration.crystallized_geometry_ties_correct,
        l11_crystallized_geometry_ties_wrong: restoration.crystallized_geometry_ties_wrong,
        l11_crystallized_geometry_tie_precision_percent,
        l11_crystallization_known_edges: restoration.crystallization_known_edges,
        l11_crystallization_unknown_edges: restoration.crystallization_unknown_edges,
        l11_crystallization_tied_edges: restoration.crystallization_tied_edges,
        l11_crystallization_conflict_edges: restoration.crystallization_conflict_edges,
        l11_crystallization_cycle_cases: restoration.crystallization_cycle_cases,
        l11_hot_readout_p50_us: percentile(&l11_latency, 50),
        l11_hot_readout_p99_us: percentile(&l11_latency, 99),
        l11_hot_readout_max_us: l11_latency.last().copied().unwrap_or_default(),
        classes: metrics.classes,
        clean_miss_diagnostics,
        miss_diagnostics: metrics.misses,
        position_diagnostics,
        reconstruction_diagnostics: evaluation.reconstruction_diagnostics,
        ambiguity_authority_diagnostics: evaluation.ambiguity_authority_diagnostics,
        false_certainty_diagnostics: evaluation.false_certainty_diagnostics,
        proof_matrix: evaluation.proof_matrix,
        package: output_path.display().to_string(),
    };
    serde_json::to_value(proof).map_err(io::Error::other)
}

fn prepare_scale_training_corpus(
    words: &[String],
    policy: ScaleProofPolicy,
) -> io::Result<(TrainingCorpus, HeldoutReservoir, usize)> {
    let workers = proof_worker_count(words.len());
    let chunk_size = words.len().div_ceil(workers);
    eprintln!(
        "l11_training_surface_preparation words={} workers={workers} chunk_size={chunk_size}",
        words.len()
    );
    let mut shards = thread::scope(|scope| {
        words
            .chunks(chunk_size)
            .enumerate()
            .map(|(shard_index, shard_words)| {
                let start_terminal = shard_index.saturating_mul(chunk_size);
                scope.spawn(move || {
                    let surface_capacity = shard_words
                        .len()
                        .saturating_mul(policy.training_surfaces_per_word);
                    let byte_capacity = shard_words
                        .iter()
                        .map(String::len)
                        .sum::<usize>()
                        .saturating_mul(policy.training_surfaces_per_word);
                    let mut corpus = TrainingCorpus::try_with_capacity(
                        shard_words.len(),
                        surface_capacity,
                        byte_capacity,
                    )?;
                    let mut heldout = HeldoutReservoir::new();
                    let mut training_surfaces = 0_usize;
                    for (offset, word) in shard_words.iter().enumerate() {
                        let terminal_id = start_terminal.saturating_add(offset);
                        let (full_training, test) =
                            split_scale_damages(word, policy.training_surfaces_per_word > 0);
                        let training = select_scale_training_damages_with_policy(
                            word,
                            full_training,
                            policy.training_surfaces_per_word,
                            policy.training_surface_policy,
                        );
                        training_surfaces = training_surfaces.saturating_add(training.len());
                        retain_heldout_examples(
                            &mut heldout,
                            policy.heldout_per_class,
                            terminal_id as u32,
                            word,
                            test,
                        );
                        corpus.push_word(
                            terminal_id as u32,
                            word.clone(),
                            training.into_iter().map(|item| item.surface),
                        )?;
                    }
                    Ok::<_, String>(ScaleTrainingShard {
                        start_terminal,
                        corpus,
                        heldout,
                        training_surfaces,
                    })
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| "L1 training surface worker panicked".to_string())?
            })
            .collect::<Result<Vec<_>, String>>()
    })
    .map_err(io::Error::other)?;
    shards.sort_unstable_by_key(|shard| shard.start_terminal);

    let surface_capacity = shards
        .iter()
        .map(|shard| shard.corpus.training_surface_count())
        .sum();
    let byte_capacity = shards
        .iter()
        .map(|shard| shard.corpus.packed_surface_bytes())
        .sum();
    let training_surfaces = shards.iter().map(|shard| shard.training_surfaces).sum();
    let mut corpus =
        TrainingCorpus::try_with_capacity(words.len(), surface_capacity, byte_capacity)
            .map_err(io::Error::other)?;
    let mut heldout = HeldoutReservoir::new();
    for shard in shards {
        corpus
            .append_shard(shard.corpus)
            .map_err(io::Error::other)?;
        merge_heldout_reservoir(&mut heldout, shard.heldout, policy.heldout_per_class);
    }
    Ok((corpus, heldout, training_surfaces))
}

fn prepare_scale_heldout(
    words: &[String],
    policy: ScaleProofPolicy,
    terminal_offset: usize,
) -> io::Result<(HeldoutReservoir, usize)> {
    let workers = proof_worker_count(words.len());
    let chunk_size = words.len().div_ceil(workers);
    eprintln!(
        "l11_heldout_preparation words={} workers={workers} chunk_size={chunk_size}",
        words.len()
    );
    let mut shards = thread::scope(|scope| {
        words
            .chunks(chunk_size)
            .enumerate()
            .map(|(shard_index, shard_words)| {
                let start_terminal =
                    terminal_offset.saturating_add(shard_index.saturating_mul(chunk_size));
                scope.spawn(move || {
                    let mut heldout = HeldoutReservoir::new();
                    let mut training_surfaces = 0_usize;
                    for (offset, word) in shard_words.iter().enumerate() {
                        let terminal_id = start_terminal.saturating_add(offset);
                        let (full_training, test) =
                            split_scale_damages(word, policy.training_surfaces_per_word > 0);
                        training_surfaces = training_surfaces.saturating_add(
                            select_scale_training_damages_with_policy(
                                word,
                                full_training,
                                policy.training_surfaces_per_word,
                                policy.training_surface_policy,
                            )
                            .len(),
                        );
                        retain_heldout_examples(
                            &mut heldout,
                            policy.heldout_per_class,
                            terminal_id as u32,
                            word,
                            test,
                        );
                    }
                    ScaleHeldoutShard {
                        start_terminal,
                        heldout,
                        training_surfaces,
                    }
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .expect("L1 heldout preparation worker panicked")
            })
            .collect::<Vec<_>>()
    });
    shards.sort_unstable_by_key(|shard| shard.start_terminal);
    let training_surfaces = shards.iter().map(|shard| shard.training_surfaces).sum();
    let mut heldout = HeldoutReservoir::new();
    for shard in shards {
        merge_heldout_reservoir(&mut heldout, shard.heldout, policy.heldout_per_class);
    }
    Ok((heldout, training_surfaces))
}

fn retain_heldout_examples(
    reservoir: &mut HeldoutReservoir,
    limit: usize,
    terminal_id: u32,
    word: &str,
    examples: Vec<DamageExample>,
) {
    for example in examples {
        let heap = reservoir.entry(example.class).or_default();
        heap.push((
            scale_sample_hash(word, &example),
            terminal_id,
            example.surface,
        ));
        if heap.len() > limit {
            heap.pop();
        }
    }
}

fn merge_heldout_reservoir(target: &mut HeldoutReservoir, source: HeldoutReservoir, limit: usize) {
    for (class, examples) in source {
        let target_class = target.entry(class).or_default();
        for example in examples {
            target_class.push(example);
            if target_class.len() > limit {
                target_class.pop();
            }
        }
    }
}

fn aggregate_restoration(classes: &BTreeMap<&'static str, ClassMetrics>) -> RestorationMetrics {
    let mut aggregate = RestorationMetrics::default();
    for class in classes.values() {
        merge_restoration(&mut aggregate, class.restoration.clone());
    }
    aggregate
}

#[derive(Default)]
struct Metrics {
    top1: usize,
    top8: usize,
    top64: usize,
    classes: BTreeMap<&'static str, ClassMetrics>,
    misses: Vec<MissDiagnostic>,
}

#[derive(Default)]
struct Evaluation {
    metrics: Metrics,
    without_phase: usize,
    without_anti: usize,
    anti_improved: usize,
    anti_worsened: usize,
    without_sequence: usize,
    sequence_improved: usize,
    sequence_worsened: usize,
    without_sequence_certificate: usize,
    sequence_certificate_improved: usize,
    sequence_certificate_worsened: usize,
    legacy_sequence: usize,
    sequence_vs_legacy_improved: usize,
    sequence_vs_legacy_worsened: usize,
    without_pairwise: usize,
    pairwise_improved: usize,
    pairwise_worsened: usize,
    without_position: usize,
    position_improved: usize,
    position_worsened: usize,
    position_diagnostics: Vec<PositionDiagnostic>,
    reconstruction_diagnostics: Vec<ReconstructionDiagnostic>,
    ambiguity_authority_diagnostics: Vec<AmbiguityAuthorityDiagnostic>,
    false_certainty_diagnostics: Vec<FalseCertaintyDiagnostic>,
    proof_matrix: ProofMatrix,
}

fn evaluate_parallel(
    memory: &LexicalGrokkingMemory,
    heldout: &[(u32, DamageExample)],
    ambiguity: &HashMap<String, BTreeSet<u32>>,
    frequency_profile: &FrequencyProfile,
) -> Evaluation {
    let worker_count = proof_worker_count(heldout.len());
    let progress = ProofProgress::new("heldout", heldout.len());
    eprintln!(
        "l11_proof stage=heldout_evaluation_start cases={} workers={worker_count}",
        heldout.len()
    );
    let partial = thread::scope(|scope| {
        (0..worker_count)
            .map(|worker| {
                let progress = &progress;
                scope.spawn(move || {
                    evaluate_cases(
                        memory,
                        heldout.iter().skip(worker).step_by(worker_count),
                        ambiguity,
                        frequency_profile,
                        progress,
                    )
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| handle.join().expect("L1 proof worker panicked"))
            .collect::<Vec<_>>()
    });
    let mut evaluation = Evaluation::default();
    for part in partial {
        merge_evaluation(&mut evaluation, part);
    }
    evaluation.metrics.misses.sort_unstable_by(|left, right| {
        left.class
            .cmp(right.class)
            .then_with(|| right.objective_unique.cmp(&left.objective_unique))
            .then_with(|| left.target_rank.is_some().cmp(&right.target_rank.is_some()))
            .then_with(|| left.surface.cmp(&right.surface))
            .then_with(|| left.target_terminal.cmp(&right.target_terminal))
    });
    evaluation
        .position_diagnostics
        .sort_unstable_by(|left, right| {
            left.outcome
                .cmp(right.outcome)
                .then_with(|| left.class.cmp(right.class))
                .then_with(|| left.surface.cmp(&right.surface))
        });
    evaluation
        .position_diagnostics
        .truncate(MAX_POSITION_DIAGNOSTICS);
    let mut miss_counts = BTreeMap::new();
    evaluation.metrics.misses.retain(|miss| {
        let count = miss_counts
            .entry((miss.class, miss.objective_unique))
            .or_insert(0_usize);
        *count += 1;
        *count <= MAX_MISS_DIAGNOSTICS_PER_CLASS
    });
    evaluation
        .reconstruction_diagnostics
        .sort_unstable_by(|left, right| {
            left.class
                .cmp(right.class)
                .then_with(|| left.surface.cmp(&right.surface))
                .then_with(|| left.lattice_rank.cmp(&right.lattice_rank))
                .then_with(|| left.candidate_terminal.cmp(&right.candidate_terminal))
        });
    let mut reconstruction_counts = BTreeMap::new();
    evaluation.reconstruction_diagnostics.retain(|diagnostic| {
        let count = reconstruction_counts
            .entry(diagnostic.class)
            .or_insert(0_usize);
        *count += 1;
        *count <= MAX_RECONSTRUCTION_DIAGNOSTICS_PER_CLASS
    });
    evaluation
        .ambiguity_authority_diagnostics
        .sort_unstable_by(|left, right| {
            let left_mutates = left.authority_surface.as_deref() != Some(left.surface.as_str());
            let right_mutates = right.authority_surface.as_deref() != Some(right.surface.as_str());
            right_mutates
                .cmp(&left_mutates)
                .then_with(|| left.class.cmp(right.class))
                .then_with(|| left.surface.cmp(&right.surface))
                .then_with(|| left.authority_terminal.cmp(&right.authority_terminal))
        });
    evaluation.ambiguity_authority_diagnostics.truncate(64);
    evaluation
        .false_certainty_diagnostics
        .sort_unstable_by(|left, right| {
            left.class
                .cmp(right.class)
                .then_with(|| left.surface.cmp(&right.surface))
                .then_with(|| left.authority_terminal.cmp(&right.authority_terminal))
        });
    evaluation
        .false_certainty_diagnostics
        .truncate(MAX_FALSE_CERTAINTY_DIAGNOSTICS);
    for class in evaluation.metrics.classes.values_mut() {
        class.unique_top1_percent = percent(class.unique_top1, class.unique_cases.max(1));
        class.lattice_coverage_percent = percent(class.top64, class.cases);
        class.without_sequence_top1_percent = percent(class.without_sequence_top1, class.cases);
        class.sequence_delta_top1 = class.top1 as isize - class.without_sequence_top1 as isize;
        class.legacy_sequence_top1_percent = percent(class.legacy_sequence_top1, class.cases);
        class.sequence_vs_legacy_delta_top1 =
            class.top1 as isize - class.legacy_sequence_top1 as isize;
        class.legacy_sequence_unique_top1_percent =
            percent(class.legacy_sequence_unique_top1, class.unique_cases.max(1));
        class.sequence_vs_legacy_unique_delta_top1 =
            class.unique_top1 as isize - class.legacy_sequence_unique_top1 as isize;
        class.without_position_unique_top1_percent = percent(
            class.without_position_unique_top1,
            class.unique_cases.max(1),
        );
        class.position_unique_delta_top1 =
            class.unique_top1 as isize - class.without_position_unique_top1 as isize;
        class.restoration.target_retained_percent = percent(
            class.restoration.target_retained,
            class.restoration.cases.max(1),
        );
        class.restoration.authority_target_winner_percent = percent(
            class.restoration.authority_target_winner,
            class.restoration.cases.max(1),
        );
        class.restoration.evidence_target_retained_percent = percent(
            class.restoration.evidence_target_retained,
            class.restoration.cases.max(1),
        );
        class
            .restoration
            .scalar_geometry_target_in_nearest_basin_percent = percent(
            class.restoration.scalar_geometry_target_in_nearest_basin,
            class.restoration.cases.max(1),
        );
        class.restoration.geometry_target_in_nearest_basin_percent = percent(
            class.restoration.geometry_target_in_nearest_basin,
            class.restoration.cases.max(1),
        );
        class.restoration.nearest_set_functional_percent = percent(
            class.restoration.nearest_set_functional,
            class.restoration.cases.max(1),
        );
        class.restoration.geometry_unique_winner_percent = percent(
            class.restoration.geometry_unique_winner,
            class.restoration.geometry_unique_cases.max(1),
        );
        class.restoration.geometry_tied_safe_percent = if class.restoration.geometry_tied_cases == 0
        {
            100.0
        } else {
            percent(
                class.restoration.geometry_tied_safe,
                class.restoration.geometry_tied_cases,
            )
        };
        class.restoration.objective_unique_winner_percent = percent(
            class.restoration.objective_unique_winner,
            class.restoration.objective_unique_cases.max(1),
        );
        class.restoration.objective_ambiguous_safe_percent =
            if class.restoration.objective_ambiguous_cases == 0 {
                100.0
            } else {
                percent(
                    class.restoration.objective_ambiguous_safe,
                    class.restoration.objective_ambiguous_cases,
                )
            };
        class
            .restoration
            .crystallized_geometry_tie_precision_percent = percent(
            class.restoration.crystallized_geometry_ties_correct,
            class.restoration.crystallized_geometry_ties.max(1),
        );
        class.edit_geometry.target_unique_min_top1_percent = percent(
            class.edit_geometry.target_unique_min_top1,
            class.edit_geometry.target_unique_min_cases.max(1),
        );
        class.edit_geometry.target_unique_min_selected_min_percent = percent(
            class.edit_geometry.target_unique_min_selected_min,
            class.edit_geometry.target_unique_min_cases.max(1),
        );
        class.edit_geometry.target_tied_min_top1_percent = percent(
            class.edit_geometry.target_tied_min_top1,
            class.edit_geometry.target_tied_min_cases.max(1),
        );
        class.edit_geometry.target_tied_min_selected_min_percent = percent(
            class.edit_geometry.target_tied_min_selected_min,
            class.edit_geometry.target_tied_min_cases.max(1),
        );
        class.edit_geometry.target_not_min_top1_percent = percent(
            class.edit_geometry.target_not_min_top1,
            class.edit_geometry.target_not_min_cases.max(1),
        );
        class.edit_geometry.target_not_min_selected_min_percent = percent(
            class.edit_geometry.target_not_min_selected_min,
            class.edit_geometry.target_not_min_cases.max(1),
        );
    }
    evaluation
}

pub(super) fn proof_worker_count(case_count: usize) -> usize {
    std::env::var("LAY_L11_PROOF_WORKERS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_else(|| {
            thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1)
        })
        .clamp(1, 32)
        .min(case_count.max(1))
}

struct ProofProgress<'a> {
    label: &'a str,
    total: usize,
    completed: AtomicUsize,
    next_report: AtomicUsize,
    started: Instant,
}

impl<'a> ProofProgress<'a> {
    fn new(label: &'a str, total: usize) -> Self {
        Self {
            label,
            total,
            completed: AtomicUsize::new(0),
            next_report: AtomicUsize::new(PROOF_PROGRESS_INTERVAL.min(total.max(1))),
            started: Instant::now(),
        }
    }

    fn advance(&self, count: usize) {
        let completed = self.completed.fetch_add(count, Ordering::Relaxed) + count;
        let mut next = self.next_report.load(Ordering::Relaxed);
        while completed >= next && next <= self.total {
            let following = next.saturating_add(PROOF_PROGRESS_INTERVAL);
            match self.next_report.compare_exchange_weak(
                next,
                following,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    eprintln!(
                        "l11_proof_progress phase={} cases={} total={} percent_milli={} elapsed_ms={}",
                        self.label,
                        completed.min(self.total),
                        self.total,
                        completed
                            .min(self.total)
                            .saturating_mul(100_000)
                            .checked_div(self.total.max(1))
                            .unwrap_or_default(),
                        self.started.elapsed().as_millis()
                    );
                    break;
                }
                Err(actual) => next = actual,
            }
        }
    }
}

fn package_dictionary_matches_parallel(memory: &LexicalGrokkingMemory, words: &[String]) -> bool {
    let worker_count = proof_worker_count(words.len());
    let chunk_size = words.len().div_ceil(worker_count);
    thread::scope(|scope| {
        words
            .chunks(chunk_size)
            .enumerate()
            .map(|(chunk_index, chunk)| {
                scope.spawn(move || {
                    let start = chunk_index.saturating_mul(chunk_size);
                    chunk.iter().enumerate().all(|(offset, word)| {
                        memory.decode_terminal((start + offset) as u32).as_deref()
                            == Some(word.as_str())
                    })
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .all(|handle| {
                handle
                    .join()
                    .expect("L1 dictionary validation worker panicked")
            })
    })
}

fn evaluate_clean_parallel(
    memory: &LexicalGrokkingMemory,
    words: &[String],
    terminal_offset: usize,
) -> Vec<CleanMissDiagnostic> {
    let worker_count = proof_worker_count(words.len());
    let chunk_size = words.len().div_ceil(worker_count);
    let progress = ProofProgress::new("clean", words.len());
    eprintln!(
        "l11_proof stage=clean_audit_start words={} workers={worker_count} chunk_size={chunk_size}",
        words.len()
    );
    let partial = thread::scope(|scope| {
        words
            .chunks(chunk_size)
            .enumerate()
            .map(|(chunk_index, chunk)| {
                let progress = &progress;
                scope.spawn(move || {
                    let start =
                        terminal_offset.saturating_add(chunk_index.saturating_mul(chunk_size));
                    let mut misses = Vec::new();
                    for (offset, word) in chunk.iter().enumerate() {
                        let target = start + offset;
                        let selected_terminal = memory
                            .readout(word, 1, ReadoutMode::Full)
                            .first()
                            .map(|candidate| candidate.terminal_id);
                        if selected_terminal != Some(target as u32) {
                            misses.push(CleanMissDiagnostic {
                                target_terminal: target as u32,
                                target_surface: word.clone(),
                                selected_terminal,
                                selected_surface: selected_terminal
                                    .and_then(|terminal| memory.decode_terminal(terminal)),
                            });
                        }
                        progress.advance(1);
                    }
                    misses
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|handle| handle.join().expect("L1 clean audit worker panicked"))
            .collect::<Vec<_>>()
    });
    let mut misses = partial;
    misses.sort_unstable_by_key(|diagnostic| diagnostic.target_terminal);
    misses
}

fn measure_hot_readout(
    memory: &LexicalGrokkingMemory,
    heldout: &[(u32, DamageExample)],
) -> Vec<u64> {
    let samples = heldout.iter().take(512).collect::<Vec<_>>();
    for (_, example) in samples.iter().take(32) {
        std::hint::black_box(memory.readout(&example.surface, 64, ReadoutMode::Full));
    }
    let mut elapsed = samples
        .into_iter()
        .map(|(_, example)| {
            let started = Instant::now();
            std::hint::black_box(memory.readout(&example.surface, 64, ReadoutMode::Full));
            started.elapsed().as_micros() as u64
        })
        .collect::<Vec<_>>();
    elapsed.sort_unstable();
    elapsed
}

fn measure_hot_restoration(
    memory: &LexicalGrokkingMemory,
    heldout: &[(u32, DamageExample)],
) -> Vec<u64> {
    let samples = heldout.iter().take(512).collect::<Vec<_>>();
    for (_, example) in samples.iter().take(32) {
        let mut candidates = memory.readout(&example.surface, 64, ReadoutMode::Full);
        std::hint::black_box(memory.classify_restoration(
            &example.surface,
            &mut candidates,
            memory.package.restoration_calibration,
        ));
    }
    let mut elapsed = samples
        .into_iter()
        .map(|(_, example)| {
            let started = Instant::now();
            let mut candidates = memory.readout(&example.surface, 64, ReadoutMode::Full);
            std::hint::black_box(memory.classify_restoration(
                &example.surface,
                &mut candidates,
                memory.package.restoration_calibration,
            ));
            started.elapsed().as_micros() as u64
        })
        .collect::<Vec<_>>();
    elapsed.sort_unstable();
    elapsed
}

fn percentile(sorted: &[u64], percentile: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    sorted[(sorted.len() - 1).saturating_mul(percentile) / 100]
}

fn evaluate_cases<'a>(
    memory: &LexicalGrokkingMemory,
    heldout: impl IntoIterator<Item = &'a (u32, DamageExample)>,
    ambiguity: &HashMap<String, BTreeSet<u32>>,
    frequency_profile: &FrequencyProfile,
    progress: &ProofProgress<'_>,
) -> Evaluation {
    let mut evaluation = Evaluation::default();
    for (target, example) in heldout {
        let [candidates, without_phase_candidates, without_anti_candidates, without_sequence_candidates, legacy_sequence_candidates, without_sequence_certificate_candidates, without_pairwise_candidates, without_position_candidates] =
            memory
                .readout_modes(
                    &example.surface,
                    64,
                    &[
                        ReadoutMode::Full,
                        ReadoutMode::WithoutPhase,
                        ReadoutMode::WithoutAnti,
                        ReadoutMode::WithoutSequence,
                        ReadoutMode::LegacySequence,
                        ReadoutMode::WithoutSequenceCertificate,
                        ReadoutMode::WithoutPairwise,
                        ReadoutMode::WithoutPosition,
                    ],
                )
                .try_into()
                .expect("L1 proof readout mode count is fixed");
        let mut restoration_candidates = candidates.clone();
        let restoration = memory.classify_restoration(
            &example.surface,
            &mut restoration_candidates,
            memory.package.restoration_calibration,
        );
        record_reconstruction_diagnostics(
            &mut evaluation.reconstruction_diagnostics,
            memory,
            example,
            &restoration_candidates,
            *target,
        );
        let top1 = candidates
            .first()
            .is_some_and(|item| item.terminal_id == *target);
        let top8 = candidates
            .iter()
            .take(8)
            .any(|item| item.terminal_id == *target);
        let top64 = candidates.iter().any(|item| item.terminal_id == *target);
        evaluation.metrics.top1 += usize::from(top1);
        evaluation.metrics.top8 += usize::from(top8);
        evaluation.metrics.top64 += usize::from(top64);
        let phase_top1 = top_is_candidates(&without_phase_candidates, *target);
        let anti_top1 = top_is_candidates(&without_anti_candidates, *target);
        let sequence_top1 = top_is_candidates(&without_sequence_candidates, *target);
        let legacy_sequence_top1 = top_is_candidates(&legacy_sequence_candidates, *target);
        let without_sequence_certificate_top1 =
            top_is_candidates(&without_sequence_certificate_candidates, *target);
        let without_pairwise_top1 = top_is_candidates(&without_pairwise_candidates, *target);
        let without_position_top1 = without_position_candidates
            .first()
            .is_some_and(|item| item.terminal_id == *target);
        evaluation.without_phase += usize::from(phase_top1);
        evaluation.without_anti += usize::from(anti_top1);
        evaluation.anti_improved += usize::from(top1 && !anti_top1);
        evaluation.anti_worsened += usize::from(!top1 && anti_top1);
        evaluation.without_sequence += usize::from(sequence_top1);
        evaluation.sequence_improved += usize::from(top1 && !sequence_top1);
        evaluation.sequence_worsened += usize::from(!top1 && sequence_top1);
        evaluation.without_sequence_certificate += usize::from(without_sequence_certificate_top1);
        evaluation.sequence_certificate_improved +=
            usize::from(top1 && !without_sequence_certificate_top1);
        evaluation.sequence_certificate_worsened +=
            usize::from(!top1 && without_sequence_certificate_top1);
        evaluation.legacy_sequence += usize::from(legacy_sequence_top1);
        evaluation.sequence_vs_legacy_improved += usize::from(top1 && !legacy_sequence_top1);
        evaluation.sequence_vs_legacy_worsened += usize::from(!top1 && legacy_sequence_top1);
        evaluation.without_pairwise += usize::from(without_pairwise_top1);
        evaluation.pairwise_improved += usize::from(top1 && !without_pairwise_top1);
        evaluation.pairwise_worsened += usize::from(!top1 && without_pairwise_top1);
        evaluation.without_position += usize::from(without_position_top1);
        evaluation.position_improved += usize::from(top1 && !without_position_top1);
        evaluation.position_worsened += usize::from(!top1 && without_position_top1);
        if top1 != without_position_top1 {
            let before = without_position_candidates.first();
            let after = candidates.first();
            evaluation.position_diagnostics.push(PositionDiagnostic {
                outcome: if top1 { "improved" } else { "worsened" },
                class: example.class,
                surface: example.surface.clone(),
                target_terminal: *target,
                before_terminal: before.map(|item| item.terminal_id),
                before_length_relation: before.map(|item| item.length_relation),
                before_sequence_milli: before.map(|item| item.sequence_milli),
                before_settled_energy: before.map(|item| item.settled_energy),
                before_forward_milli: before.map(|item| item.forward_milli),
                before_backward_milli: before.map(|item| item.backward_milli),
                before_structural_milli: before.map(|item| item.structural_milli),
                after_terminal: after.map(|item| item.terminal_id),
                after_length_relation: after.map(|item| item.length_relation),
                after_sequence_milli: after.map(|item| item.sequence_milli),
                after_position_milli: after.map(|item| item.position_milli),
                after_settled_energy: after.map(|item| item.settled_energy),
                after_forward_milli: after.map(|item| item.forward_milli),
                after_backward_milli: after.map(|item| item.backward_milli),
                after_structural_milli: after.map(|item| item.structural_milli),
            });
        }
        let targets = ambiguity
            .get(example.surface.as_str())
            .cloned()
            .unwrap_or_else(|| BTreeSet::from([*target]));
        if targets.len() > 1 {
            if let RestorationReadout::Winner { candidate } = &restoration {
                let nearest_terminals =
                    super::restoration::geometric_basin(&restoration_candidates)
                        .into_iter()
                        .map(|candidate| candidate.terminal_id)
                        .collect::<Vec<_>>();
                evaluation
                    .ambiguity_authority_diagnostics
                    .push(AmbiguityAuthorityDiagnostic {
                        class: example.class,
                        surface: example.surface.clone(),
                        target_terminals: targets.iter().copied().collect(),
                        target_surfaces: targets
                            .iter()
                            .map(|terminal| memory.decode_terminal(*terminal))
                            .collect(),
                        authority_terminal: candidate.terminal_id,
                        authority_surface: memory.decode_terminal(candidate.terminal_id),
                        nearest_surfaces: nearest_terminals
                            .iter()
                            .map(|terminal| memory.decode_terminal(*terminal))
                            .collect(),
                        nearest_terminals,
                    });
            }
        }
        let class = evaluation.metrics.classes.entry(example.class).or_default();
        let authority_mutates_surface = match &restoration {
            RestorationReadout::Winner { candidate } => memory
                .decode_terminal(candidate.terminal_id)
                .is_some_and(|surface| surface != example.surface),
            _ => false,
        };
        record_restoration(
            &mut class.restoration,
            &restoration_candidates,
            &restoration,
            *target,
            &targets,
            authority_mutates_surface,
        );
        evaluation.proof_matrix.record(
            memory,
            frequency_profile,
            example,
            *target,
            &targets,
            &candidates,
            &restoration_candidates,
            &restoration,
        );
        class.cases += 1;
        class.top1 += usize::from(top1);
        class.top8 += usize::from(top8);
        class.top64 += usize::from(top64);
        class.without_sequence_top1 += usize::from(sequence_top1);
        class.legacy_sequence_top1 += usize::from(legacy_sequence_top1);
        if targets.len() == 1 {
            class.unique_cases += 1;
            class.unique_top1 += usize::from(top1);
            class.legacy_sequence_unique_top1 += usize::from(legacy_sequence_top1);
            class.without_position_unique_top1 += usize::from(without_position_top1);
            if example.class == "double_substitution" {
                record_edit_geometry_case(
                    &mut class.edit_geometry,
                    memory,
                    &example.surface,
                    &candidates,
                    *target,
                    top1,
                );
            }
            if !top1 {
                record_failure_decomposition(
                    &mut class.failure_decomposition,
                    memory,
                    &example.surface,
                    &candidates,
                    *target,
                );
            }
        } else {
            class.ambiguous_cases += 1;
            let selected_outside_set = candidates
                .first()
                .is_some_and(|item| !targets.contains(&item.terminal_id));
            class.raw_top1_outside_objective += usize::from(selected_outside_set);
            let false_certainty = match &restoration {
                RestorationReadout::Winner { candidate } => {
                    authority_mutates_surface && !targets.contains(&candidate.terminal_id)
                }
                _ => false,
            };
            class.false_certainty += usize::from(false_certainty);
            if false_certainty {
                let RestorationReadout::Winner { candidate } = &restoration else {
                    unreachable!("false certainty requires winner authority");
                };
                evaluation
                    .false_certainty_diagnostics
                    .push(FalseCertaintyDiagnostic {
                        class: example.class,
                        surface: example.surface.clone(),
                        target_terminals: targets.iter().copied().collect(),
                        target_surfaces: targets
                            .iter()
                            .map(|terminal| memory.decode_terminal(*terminal))
                            .collect(),
                        authority_terminal: candidate.terminal_id,
                        authority_surface: memory.decode_terminal(candidate.terminal_id),
                    });
            }
        }
        if !top1 {
            let rank = candidates
                .iter()
                .position(|item| item.terminal_id == *target);
            let selected = candidates.first();
            let target_candidate = rank.and_then(|value| candidates.get(value));
            evaluation.metrics.misses.push(MissDiagnostic {
                class: example.class,
                surface: example.surface.clone(),
                objective_unique: targets.len() == 1,
                target_terminal: *target,
                target_surface: memory.decode_terminal(*target),
                selected_terminal: selected.map(|item| item.terminal_id),
                selected_surface: selected
                    .and_then(|item| memory.decode_terminal(item.terminal_id)),
                target_rank: rank.map(|value| value + 1),
                selected_energy: selected.map(|item| item.settled_energy),
                target_energy: target_candidate.map(|item| item.settled_energy),
                selected_geometry_distance: selected.map(|item| item.geometry_distance),
                target_geometry_distance: target_candidate.map(|item| item.geometry_distance),
                selected_reconstruction_modes: selected.map(|item| item.reconstruction_modes),
                target_reconstruction_modes: target_candidate.map(|item| item.reconstruction_modes),
                selected_sequence_milli: selected.map(|item| item.sequence_milli),
                target_sequence_milli: target_candidate.map(|item| item.sequence_milli),
                selected_backward_milli: selected.map(|item| item.backward_milli),
                target_backward_milli: target_candidate.map(|item| item.backward_milli),
            });
        }
        progress.advance(1);
    }
    evaluation
}

fn record_restoration(
    metrics: &mut RestorationMetrics,
    candidates: &[super::runtime::GrokkingCandidate],
    readout: &RestorationReadout,
    target_terminal: u32,
    objective_targets: &BTreeSet<u32>,
    authority_mutates_surface: bool,
) {
    metrics.cases += 1;
    let mut authority = BTreeSet::new();
    let mut returned = BTreeSet::new();
    let evidence = match readout {
        RestorationReadout::Winner { candidate } => {
            metrics.winner += 1;
            authority.insert(candidate.terminal_id);
            returned.insert(candidate.terminal_id);
            [candidate.terminal_id].into_iter().collect::<BTreeSet<_>>()
        }
        RestorationReadout::Tied { candidates, .. } => {
            metrics.tied += 1;
            let candidates = candidates
                .iter()
                .map(|candidate| candidate.terminal_id)
                .collect::<BTreeSet<_>>();
            returned.clone_from(&candidates);
            candidates
        }
        RestorationReadout::TiedOverflow { candidates, .. } => {
            metrics.tied_overflow += 1;
            candidates
                .iter()
                .map(|candidate| candidate.terminal_id)
                .collect()
        }
        RestorationReadout::Abstain {
            reason, candidates, ..
        } => {
            metrics.abstain += 1;
            match reason {
                AbstainReason::NoCandidates => metrics.abstain_no_candidates += 1,
                AbstainReason::OutsideCalibratedBasin => {
                    metrics.abstain_outside_calibrated_basin += 1
                }
                AbstainReason::WeakPositivePhase => metrics.abstain_weak_positive_phase += 1,
                AbstainReason::WeakBackwardReconstruction => {
                    metrics.abstain_weak_backward_reconstruction += 1
                }
                AbstainReason::ConflictingEvidence => metrics.abstain_conflicting_evidence += 1,
            }
            candidates
                .iter()
                .map(|candidate| candidate.terminal_id)
                .collect()
        }
    };
    metrics.authority_target_winner += usize::from(authority.contains(&target_terminal));
    metrics.target_retained += usize::from(returned.contains(&target_terminal));
    metrics.evidence_target_retained += usize::from(evidence.contains(&target_terminal));
    let authority_terminal = authority.iter().next().copied();
    if objective_targets.len() == 1 {
        metrics.objective_unique_cases += 1;
        metrics.objective_unique_winner += usize::from(authority_terminal == Some(target_terminal));
    } else {
        metrics.objective_ambiguous_cases += 1;
        metrics.objective_ambiguous_safe += usize::from(!authority_mutates_surface);
        metrics.false_authority_on_objective_ambiguity += usize::from(authority_mutates_surface);
    }

    let scalar_minimum = candidates
        .iter()
        .map(|candidate| candidate.geometry_distance)
        .min();
    let scalar_nearest = scalar_minimum
        .map(|distance| {
            candidates
                .iter()
                .filter(|candidate| candidate.geometry_distance == distance)
                .map(|candidate| candidate.terminal_id)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let nearest = super::restoration::geometric_basin(candidates)
        .into_iter()
        .map(|candidate| candidate.terminal_id)
        .collect::<BTreeSet<_>>();
    if let Some(probe) = candidates
        .iter()
        .find(|candidate| nearest.contains(&candidate.terminal_id))
    {
        metrics.crystallization_known_edges += usize::from(probe.crystallization_known_edges);
        metrics.crystallization_unknown_edges += usize::from(probe.crystallization_unknown_edges);
        metrics.crystallization_tied_edges += usize::from(probe.crystallization_tied_edges);
        metrics.crystallization_conflict_edges += usize::from(probe.crystallization_conflicts);
        metrics.crystallization_cycle_cases += usize::from(probe.crystallization_cycles != 0);
    }
    let scalar_has_target = scalar_nearest.contains(&target_terminal);
    let nearest_has_target = nearest.contains(&target_terminal);
    metrics.scalar_geometry_target_in_nearest_basin += usize::from(scalar_has_target);
    metrics.geometry_target_in_nearest_basin += usize::from(nearest_has_target);
    metrics.reconstruction_basin_expansions += usize::from(nearest != scalar_nearest);
    metrics.reconstruction_target_recovered +=
        usize::from(!scalar_has_target && nearest_has_target);
    metrics.reconstruction_target_lost += usize::from(scalar_has_target && !nearest_has_target);
    metrics.nearest_set_functional += usize::from(!returned.is_empty() && returned == nearest);
    if nearest.len() == 1 {
        metrics.geometry_unique_cases += 1;
        metrics.geometry_unique_winner += usize::from(
            matches!(readout, RestorationReadout::Winner { .. }) && returned == nearest,
        );
    } else if nearest.len() > 1 {
        metrics.geometry_tied_cases += 1;
        metrics.geometry_tied_safe += usize::from(matches!(
            readout,
            RestorationReadout::Tied { .. }
                | RestorationReadout::TiedOverflow { .. }
                | RestorationReadout::Abstain { .. }
        ));
        metrics.false_singleton_on_geometry_tie +=
            usize::from(matches!(readout, RestorationReadout::Winner { .. }));
        if let Some(authority_terminal) = authority_terminal {
            metrics.crystallized_geometry_ties += 1;
            if objective_targets.len() == 1 && authority_terminal == target_terminal {
                metrics.crystallized_geometry_ties_correct += 1;
            } else if authority_mutates_surface {
                metrics.crystallized_geometry_ties_wrong += 1;
            }
        }
    }
}

fn record_reconstruction_diagnostics(
    diagnostics: &mut Vec<ReconstructionDiagnostic>,
    memory: &LexicalGrokkingMemory,
    example: &DamageExample,
    candidates: &[super::runtime::GrokkingCandidate],
    target_terminal: u32,
) {
    let Some(scalar_geometry_distance) = candidates
        .iter()
        .map(|candidate| candidate.geometry_distance)
        .min()
    else {
        return;
    };
    if scalar_geometry_distance == 0 {
        return;
    }
    for (lattice_rank, candidate) in candidates.iter().take(8).enumerate() {
        if candidate.geometry_distance == scalar_geometry_distance
            || candidate.reconstruction_modes == 0
        {
            continue;
        }
        diagnostics.push(ReconstructionDiagnostic {
            class: example.class,
            surface: example.surface.clone(),
            target_terminal,
            target_surface: memory.decode_terminal(target_terminal),
            candidate_terminal: candidate.terminal_id,
            candidate_surface: memory.decode_terminal(candidate.terminal_id),
            candidate_is_target: candidate.terminal_id == target_terminal,
            lattice_rank,
            scalar_geometry_distance,
            candidate_geometry_distance: candidate.geometry_distance,
            reconstruction_modes: candidate.reconstruction_modes,
            positive_milli: candidate
                .positive_subcenter_milli
                .max(candidate.positive_milli),
            backward_milli: candidate.backward_milli,
            anti_milli: candidate.anti_subcenter_milli.max(candidate.anti_milli),
            hard_negative_milli: candidate.hard_negative_milli,
        });
    }
}

fn merge_evaluation(target: &mut Evaluation, source: Evaluation) {
    target.metrics.top1 += source.metrics.top1;
    target.metrics.top8 += source.metrics.top8;
    target.metrics.top64 += source.metrics.top64;
    target.metrics.misses.extend(source.metrics.misses);
    target.without_phase += source.without_phase;
    target.without_anti += source.without_anti;
    target.anti_improved += source.anti_improved;
    target.anti_worsened += source.anti_worsened;
    target.without_sequence += source.without_sequence;
    target.sequence_improved += source.sequence_improved;
    target.sequence_worsened += source.sequence_worsened;
    target.without_sequence_certificate += source.without_sequence_certificate;
    target.sequence_certificate_improved += source.sequence_certificate_improved;
    target.sequence_certificate_worsened += source.sequence_certificate_worsened;
    target.legacy_sequence += source.legacy_sequence;
    target.sequence_vs_legacy_improved += source.sequence_vs_legacy_improved;
    target.sequence_vs_legacy_worsened += source.sequence_vs_legacy_worsened;
    target.without_pairwise += source.without_pairwise;
    target.pairwise_improved += source.pairwise_improved;
    target.pairwise_worsened += source.pairwise_worsened;
    target.without_position += source.without_position;
    target.position_improved += source.position_improved;
    target.position_worsened += source.position_worsened;
    target
        .position_diagnostics
        .extend(source.position_diagnostics);
    target
        .reconstruction_diagnostics
        .extend(source.reconstruction_diagnostics);
    target
        .ambiguity_authority_diagnostics
        .extend(source.ambiguity_authority_diagnostics);
    target
        .false_certainty_diagnostics
        .extend(source.false_certainty_diagnostics);
    target.proof_matrix.merge(source.proof_matrix);
    for (name, source_class) in source.metrics.classes {
        let class = target.metrics.classes.entry(name).or_default();
        class.cases += source_class.cases;
        class.unique_cases += source_class.unique_cases;
        class.unique_top1 += source_class.unique_top1;
        class.ambiguous_cases += source_class.ambiguous_cases;
        class.top1 += source_class.top1;
        class.top8 += source_class.top8;
        class.top64 += source_class.top64;
        class.false_certainty += source_class.false_certainty;
        class.raw_top1_outside_objective += source_class.raw_top1_outside_objective;
        class.without_sequence_top1 += source_class.without_sequence_top1;
        class.legacy_sequence_top1 += source_class.legacy_sequence_top1;
        class.legacy_sequence_unique_top1 += source_class.legacy_sequence_unique_top1;
        class.without_position_unique_top1 += source_class.without_position_unique_top1;
        merge_restoration(&mut class.restoration, source_class.restoration);
        merge_edit_geometry(&mut class.edit_geometry, source_class.edit_geometry);
        merge_failure_decomposition(
            &mut class.failure_decomposition,
            source_class.failure_decomposition,
        );
    }
}

fn merge_restoration(target: &mut RestorationMetrics, source: RestorationMetrics) {
    target.cases += source.cases;
    target.winner += source.winner;
    target.tied += source.tied;
    target.tied_overflow += source.tied_overflow;
    target.abstain += source.abstain;
    target.abstain_no_candidates += source.abstain_no_candidates;
    target.abstain_outside_calibrated_basin += source.abstain_outside_calibrated_basin;
    target.abstain_weak_positive_phase += source.abstain_weak_positive_phase;
    target.abstain_weak_backward_reconstruction += source.abstain_weak_backward_reconstruction;
    target.abstain_conflicting_evidence += source.abstain_conflicting_evidence;
    target.authority_target_winner += source.authority_target_winner;
    target.target_retained += source.target_retained;
    target.evidence_target_retained += source.evidence_target_retained;
    target.scalar_geometry_target_in_nearest_basin +=
        source.scalar_geometry_target_in_nearest_basin;
    target.geometry_target_in_nearest_basin += source.geometry_target_in_nearest_basin;
    target.reconstruction_basin_expansions += source.reconstruction_basin_expansions;
    target.reconstruction_target_recovered += source.reconstruction_target_recovered;
    target.reconstruction_target_lost += source.reconstruction_target_lost;
    target.nearest_set_functional += source.nearest_set_functional;
    target.geometry_unique_cases += source.geometry_unique_cases;
    target.geometry_unique_winner += source.geometry_unique_winner;
    target.geometry_tied_cases += source.geometry_tied_cases;
    target.geometry_tied_safe += source.geometry_tied_safe;
    target.false_singleton_on_geometry_tie += source.false_singleton_on_geometry_tie;
    target.objective_unique_cases += source.objective_unique_cases;
    target.objective_unique_winner += source.objective_unique_winner;
    target.objective_ambiguous_cases += source.objective_ambiguous_cases;
    target.objective_ambiguous_safe += source.objective_ambiguous_safe;
    target.false_authority_on_objective_ambiguity += source.false_authority_on_objective_ambiguity;
    target.crystallized_geometry_ties += source.crystallized_geometry_ties;
    target.crystallized_geometry_ties_correct += source.crystallized_geometry_ties_correct;
    target.crystallized_geometry_ties_wrong += source.crystallized_geometry_ties_wrong;
    target.crystallization_known_edges += source.crystallization_known_edges;
    target.crystallization_unknown_edges += source.crystallization_unknown_edges;
    target.crystallization_tied_edges += source.crystallization_tied_edges;
    target.crystallization_conflict_edges += source.crystallization_conflict_edges;
    target.crystallization_cycle_cases += source.crystallization_cycle_cases;
}

fn record_edit_geometry_case(
    metrics: &mut EditGeometryMetrics,
    memory: &LexicalGrokkingMemory,
    surface: &str,
    candidates: &[super::runtime::GrokkingCandidate],
    target_terminal: u32,
    top1: bool,
) {
    let normalized = surface
        .trim()
        .trim_matches(|ch: char| matches!(ch, '!' | ',' | '.' | '?' | ';' | ':'))
        .to_lowercase();
    let distances = candidates
        .iter()
        .filter_map(|candidate| {
            let decoded = memory.decode_terminal(candidate.terminal_id)?;
            Some((
                candidate.terminal_id,
                crate::text_metrics::damerau_levenshtein(&normalized, &decoded),
            ))
        })
        .collect::<Vec<_>>();
    let Some(target_distance) = distances
        .iter()
        .find_map(|(terminal, distance)| (*terminal == target_terminal).then_some(*distance))
    else {
        metrics.target_missing_cases += 1;
        return;
    };
    let minimum = distances
        .iter()
        .map(|(_, distance)| *distance)
        .min()
        .unwrap_or(target_distance);
    let selected_is_min = distances
        .first()
        .is_some_and(|(_, distance)| *distance == minimum);
    if target_distance != minimum {
        metrics.target_not_min_cases += 1;
        metrics.target_not_min_top1 += usize::from(top1);
        metrics.target_not_min_selected_min += usize::from(selected_is_min);
    } else if distances
        .iter()
        .filter(|(_, distance)| *distance == minimum)
        .count()
        == 1
    {
        metrics.target_unique_min_cases += 1;
        metrics.target_unique_min_top1 += usize::from(top1);
        metrics.target_unique_min_selected_min += usize::from(selected_is_min);
    } else {
        metrics.target_tied_min_cases += 1;
        metrics.target_tied_min_top1 += usize::from(top1);
        metrics.target_tied_min_selected_min += usize::from(selected_is_min);
    }
}

fn merge_edit_geometry(target: &mut EditGeometryMetrics, source: EditGeometryMetrics) {
    target.target_unique_min_cases += source.target_unique_min_cases;
    target.target_unique_min_top1 += source.target_unique_min_top1;
    target.target_unique_min_selected_min += source.target_unique_min_selected_min;
    target.target_tied_min_cases += source.target_tied_min_cases;
    target.target_tied_min_top1 += source.target_tied_min_top1;
    target.target_tied_min_selected_min += source.target_tied_min_selected_min;
    target.target_not_min_cases += source.target_not_min_cases;
    target.target_not_min_top1 += source.target_not_min_top1;
    target.target_not_min_selected_min += source.target_not_min_selected_min;
    target.target_missing_cases += source.target_missing_cases;
}

fn record_failure_decomposition(
    metrics: &mut FailureDecomposition,
    memory: &LexicalGrokkingMemory,
    surface: &str,
    candidates: &[super::runtime::GrokkingCandidate],
    target_terminal: u32,
) {
    metrics.unique_failures += 1;
    let Some(incumbent) = candidates.first() else {
        metrics.target_missing_top64 += 1;
        return;
    };
    let Some((target_rank, target)) = candidates
        .iter()
        .enumerate()
        .find(|(_, candidate)| candidate.terminal_id == target_terminal)
    else {
        metrics.target_missing_top64 += 1;
        return;
    };
    metrics.target_rank2 += usize::from(target_rank == 1);
    metrics.same_length_basin += usize::from(target.length_relation == incumbent.length_relation);
    metrics.target_stronger_position +=
        usize::from(target.position_milli > incumbent.position_milli);
    metrics.target_stronger_sequence +=
        usize::from(target.sequence_milli > incumbent.sequence_milli);
    metrics.target_stronger_backward +=
        usize::from(target.backward_milli > incumbent.backward_milli);
    metrics.target_stronger_phase += usize::from(target.positive_milli > incumbent.positive_milli);
    metrics.target_stronger_forward += usize::from(target.forward_milli > incumbent.forward_milli);
    metrics.target_stronger_structural +=
        usize::from(target.structural_milli > incumbent.structural_milli);
    metrics.both_phase_ge_990 +=
        usize::from(target.positive_milli >= 990 && incumbent.positive_milli >= 990);
    metrics.both_anti_zero += usize::from(target.anti_milli == 0 && incumbent.anti_milli == 0);
    metrics.both_pairwise_zero +=
        usize::from(target.pairwise_loss_milli == 0 && incumbent.pairwise_loss_milli == 0);
    metrics.winner_stronger_forward += usize::from(incumbent.forward_milli > target.forward_milli);
    metrics.winner_stronger_structural +=
        usize::from(incumbent.structural_milli > target.structural_milli);
    record_edit_geometry(metrics, memory, surface, candidates, target_terminal);
    metrics.winner_without_vowel += usize::from(
        memory
            .decode_terminal(incumbent.terminal_id)
            .is_some_and(|word| !word.chars().any(is_vowel)),
    );
    let deficit = incumbent
        .settled_energy
        .saturating_sub(target.settled_energy);
    metrics.energy_deficit_le_250 += usize::from(deficit <= 250);
    metrics.energy_deficit_le_500 += usize::from(deficit <= 500);
    metrics.energy_deficit_le_1000 += usize::from(deficit <= 1_000);
}

fn merge_failure_decomposition(target: &mut FailureDecomposition, source: FailureDecomposition) {
    target.unique_failures += source.unique_failures;
    target.target_missing_top64 += source.target_missing_top64;
    target.target_rank2 += source.target_rank2;
    target.same_length_basin += source.same_length_basin;
    target.target_stronger_position += source.target_stronger_position;
    target.target_stronger_sequence += source.target_stronger_sequence;
    target.target_stronger_backward += source.target_stronger_backward;
    target.target_stronger_phase += source.target_stronger_phase;
    target.target_stronger_forward += source.target_stronger_forward;
    target.target_stronger_structural += source.target_stronger_structural;
    target.energy_deficit_le_250 += source.energy_deficit_le_250;
    target.energy_deficit_le_500 += source.energy_deficit_le_500;
    target.energy_deficit_le_1000 += source.energy_deficit_le_1000;
    target.both_phase_ge_990 += source.both_phase_ge_990;
    target.both_anti_zero += source.both_anti_zero;
    target.both_pairwise_zero += source.both_pairwise_zero;
    target.winner_stronger_forward += source.winner_stronger_forward;
    target.winner_stronger_structural += source.winner_stronger_structural;
    target.target_unique_min_edit += source.target_unique_min_edit;
    target.target_tied_min_edit += source.target_tied_min_edit;
    target.target_not_min_edit += source.target_not_min_edit;
    target.winner_without_vowel += source.winner_without_vowel;
}

fn record_edit_geometry(
    metrics: &mut FailureDecomposition,
    memory: &LexicalGrokkingMemory,
    surface: &str,
    candidates: &[super::runtime::GrokkingCandidate],
    target_terminal: u32,
) {
    let normalized = surface
        .trim()
        .trim_matches(|ch: char| matches!(ch, '!' | ',' | '.' | '?' | ';' | ':'))
        .to_lowercase();
    let distances = candidates
        .iter()
        .filter_map(|candidate| {
            let decoded = memory.decode_terminal(candidate.terminal_id)?;
            Some((
                candidate.terminal_id,
                crate::text_metrics::damerau_levenshtein(&normalized, &decoded),
            ))
        })
        .collect::<Vec<_>>();
    let Some(target_distance) = distances
        .iter()
        .find_map(|(terminal, distance)| (*terminal == target_terminal).then_some(*distance))
    else {
        return;
    };
    let minimum = distances
        .iter()
        .map(|(_, distance)| *distance)
        .min()
        .unwrap_or(target_distance);
    if target_distance != minimum {
        metrics.target_not_min_edit += 1;
    } else if distances
        .iter()
        .filter(|(_, distance)| *distance == minimum)
        .count()
        == 1
    {
        metrics.target_unique_min_edit += 1;
    } else {
        metrics.target_tied_min_edit += 1;
    }
}

fn is_vowel(ch: char) -> bool {
    matches!(ch.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
        || matches!(
            ch,
            'а' | 'е'
                | 'ё'
                | 'и'
                | 'о'
                | 'у'
                | 'ы'
                | 'э'
                | 'ю'
                | 'я'
                | 'А'
                | 'Е'
                | 'Ё'
                | 'И'
                | 'О'
                | 'У'
                | 'Ы'
                | 'Э'
                | 'Ю'
                | 'Я'
        )
}

fn corpus_words(text: &str, max_words: usize) -> Vec<String> {
    let mut words = BTreeSet::new();
    for token in text.split(|ch: char| !(ch.is_alphabetic() || ch == '-' || ch == '\'')) {
        let word = token.trim_matches(['-', '\'']).to_lowercase();
        if (4..=24).contains(&word.chars().count()) {
            words.insert(word);
            if max_words > 0 && words.len() >= max_words {
                break;
            }
        }
    }
    words.into_iter().collect()
}

pub(super) fn corpus_words_from_lines(text: &str, max_words: usize) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut words = Vec::new();
    for line in text.lines() {
        let word = line.trim().to_lowercase();
        if !(2..=32).contains(&word.chars().count())
            || !word
                .chars()
                .all(|character| character.is_alphabetic() || character == '-' || character == '\'')
            || !seen.insert(word.clone())
        {
            continue;
        }
        words.push(word);
        if max_words > 0 && words.len() >= max_words {
            break;
        }
    }
    words
}

fn scale_sample_hash(word: &str, example: &DamageExample) -> u64 {
    let mut state = 0x5343_414c_455f_4831_u64;
    for byte in word
        .bytes()
        .chain(example.class.bytes())
        .chain(example.surface.bytes())
    {
        state = mix64_golden(state ^ u64::from(byte));
    }
    state
}

#[cfg(any(test, feature = "lexical-compiler"))]
pub(super) struct FixedHeldoutCase {
    pub(super) class: &'static str,
    pub(super) terminal_id: u32,
    pub(super) surface: String,
}

#[cfg(any(test, feature = "lexical-compiler"))]
pub(super) fn prepare_fixed_heldout_cases(
    words: &[String],
    heldout_per_class: usize,
    terminal_offset: usize,
) -> io::Result<Vec<FixedHeldoutCase>> {
    if heldout_per_class == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "fixed heldout budget must be positive",
        ));
    }
    let policy = ScaleProofPolicy {
        heldout_per_class,
        training_surfaces_per_word: 0,
        training_surface_policy: ScaleTrainingSurfacePolicy::LegacyAlphabetical,
        maximum_rss_mib: DEFAULT_TRAINING_RSS_MIB,
    };
    let (reservoir, _) = prepare_scale_heldout(words, policy, terminal_offset)?;
    let mut cases = reservoir
        .into_iter()
        .flat_map(|(class, heap)| {
            heap.into_iter()
                .map(move |(_, terminal_id, surface)| FixedHeldoutCase {
                    class,
                    terminal_id,
                    surface,
                })
        })
        .collect::<Vec<_>>();
    cases.sort_unstable_by(|left, right| {
        left.class
            .cmp(right.class)
            .then_with(|| left.terminal_id.cmp(&right.terminal_id))
            .then_with(|| left.surface.cmp(&right.surface))
    });
    Ok(cases)
}

pub(super) fn populate_sampled_ambiguity(
    words: &[String],
    heldout: &[(u32, DamageExample)],
    ambiguity: &mut HashMap<String, BTreeSet<u32>>,
) {
    let selected = heldout
        .iter()
        .map(|(_, example)| example.surface.as_str())
        .collect::<HashSet<_>>();
    for (terminal_id, example) in heldout {
        ambiguity
            .entry(example.surface.clone())
            .or_default()
            .insert(*terminal_id);
    }
    let terminals_by_surface = words
        .iter()
        .enumerate()
        .map(|(terminal_id, word)| (word.as_str(), terminal_id as u32))
        .collect::<HashMap<_, _>>();
    for surface in &selected {
        let characters = surface.chars().collect::<Vec<_>>();
        for removed in 0..characters.len() {
            let possible_source = characters
                .iter()
                .enumerate()
                .filter_map(|(index, character)| (index != removed).then_some(*character))
                .collect::<String>();
            if let Some(terminal_id) = terminals_by_surface.get(possible_source.as_str()) {
                ambiguity
                    .entry((*surface).to_string())
                    .or_default()
                    .insert(*terminal_id);
            }
        }
    }
    for (terminal_id, word) in words.iter().enumerate() {
        if selected.contains(word.as_str()) {
            ambiguity
                .entry(word.clone())
                .or_default()
                .insert(terminal_id as u32);
        }
        let characters = word.chars().collect::<Vec<_>>();
        if characters.len() >= 2 {
            for truncated in [
                characters[1..].iter().collect::<String>(),
                characters[..characters.len() - 1]
                    .iter()
                    .collect::<String>(),
            ] {
                if selected.contains(truncated.as_str()) {
                    ambiguity
                        .entry(truncated)
                        .or_default()
                        .insert(terminal_id as u32);
                }
            }
        }
        let (training, test) = split_damages(word);
        for example in training.into_iter().chain(test) {
            if selected.contains(example.surface.as_str()) {
                ambiguity
                    .entry(example.surface)
                    .or_default()
                    .insert(terminal_id as u32);
            }
        }
    }
    extend_sparse_omission_ambiguity_indexed(words, heldout, ambiguity);
}

fn extend_sparse_omission_ambiguity_indexed(
    words: &[String],
    heldout: &[(u32, DamageExample)],
    ambiguity: &mut HashMap<String, BTreeSet<u32>>,
) {
    let mut selected_by_length = BTreeMap::<usize, HashSet<&str>>::new();
    for (_, example) in heldout {
        if example.class == "sparse_multi_omission" {
            selected_by_length
                .entry(example.surface.chars().count())
                .or_default()
                .insert(example.surface.as_str());
        }
    }
    for (terminal_id, word) in words.iter().enumerate() {
        let characters = word.chars().collect::<Vec<_>>();
        let Some(selected) = characters
            .len()
            .checked_sub(2)
            .and_then(|length| selected_by_length.get(&length))
        else {
            continue;
        };
        for first in 0..characters.len().saturating_sub(1) {
            for second in first + 1..characters.len() {
                let candidate = characters
                    .iter()
                    .enumerate()
                    .filter_map(|(index, character)| {
                        (index != first && index != second).then_some(*character)
                    })
                    .collect::<String>();
                if selected.contains(candidate.as_str()) {
                    ambiguity
                        .entry(candidate)
                        .or_default()
                        .insert(terminal_id as u32);
                }
            }
        }
    }
}

fn extend_sparse_omission_ambiguity(
    words: &[String],
    heldout: &[(u32, DamageExample)],
    ambiguity: &mut HashMap<String, BTreeSet<u32>>,
) {
    for (_, example) in heldout {
        if example.class != "sparse_multi_omission" {
            continue;
        }
        let damaged = example.surface.chars().collect::<Vec<_>>();
        for (terminal_id, word) in words.iter().enumerate() {
            let clean = word.chars().collect::<Vec<_>>();
            if clean.len() == damaged.len() + 2 && is_ordered_subsequence(&damaged, &clean) {
                ambiguity
                    .entry(example.surface.clone())
                    .or_default()
                    .insert(terminal_id as u32);
            }
        }
    }
}

pub(super) fn is_ordered_subsequence(needle: &[char], haystack: &[char]) -> bool {
    let mut next = 0;
    for symbol in haystack {
        if needle.get(next) == Some(symbol) {
            next += 1;
        }
    }
    next == needle.len()
}

fn top_is_candidates(candidates: &[super::runtime::GrokkingCandidate], target: u32) -> bool {
    candidates
        .first()
        .is_some_and(|candidate| candidate.terminal_id == target)
}

fn percent(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 * 100.0 / denominator as f64
    }
}

#[cfg(test)]
mod scale_proof_tests {
    use super::*;

    #[test]
    fn parallel_heldout_preparation_matches_the_sequential_reservoir() {
        let words = [
            "время",
            "переподключение",
            "архитектура",
            "download",
            "restoration",
            "candidate",
            "кристаллизация",
            "проверка",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        let policy = ScaleProofPolicy {
            heldout_per_class: 3,
            training_surfaces_per_word: 2,
            training_surface_policy: ScaleTrainingSurfacePolicy::HybridClassConditioned,
            maximum_rss_mib: DEFAULT_TRAINING_RSS_MIB,
        };
        let (parallel, parallel_training) =
            prepare_scale_heldout(&words, policy, 0).expect("prepare parallel heldout");

        let mut sequential = HeldoutReservoir::new();
        let mut sequential_training = 0;
        for (terminal_id, word) in words.iter().enumerate() {
            let (full_training, test) = split_scale_damages(word, true);
            sequential_training += select_scale_training_damages_with_policy(
                word,
                full_training,
                policy.training_surfaces_per_word,
                policy.training_surface_policy,
            )
            .len();
            retain_heldout_examples(
                &mut sequential,
                policy.heldout_per_class,
                terminal_id as u32,
                word,
                test,
            );
        }

        assert_eq!(parallel_training, sequential_training);
        assert_eq!(canonical_heldout(parallel), canonical_heldout(sequential));
    }

    #[test]
    fn parallel_heldout_preparation_preserves_terminal_offset() {
        let words = [
            "download",
            "restoration",
            "candidate",
            "architecture",
            "crystal",
            "signal",
            "terminal",
            "package",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        let policy = ScaleProofPolicy {
            heldout_per_class: 3,
            training_surfaces_per_word: 0,
            training_surface_policy: ScaleTrainingSurfacePolicy::LegacyAlphabetical,
            maximum_rss_mib: DEFAULT_TRAINING_RSS_MIB,
        };

        let (heldout, _) =
            prepare_scale_heldout(&words, policy, 462_314).expect("prepare ranged heldout");
        let terminals = heldout
            .values()
            .flat_map(|heap| heap.iter().map(|(_, terminal, _)| *terminal))
            .collect::<Vec<_>>();

        assert!(!terminals.is_empty());
        assert!(terminals
            .iter()
            .all(|terminal| (462_314..462_322).contains(terminal)));
    }

    #[test]
    fn latency_sample_round_robins_across_damage_classes() {
        let classes = BTreeMap::from([
            ("a", vec![(1, "a1".to_string()), (2, "a2".to_string())]),
            ("b", vec![(3, "b1".to_string()), (4, "b2".to_string())]),
            ("c", vec![(5, "c1".to_string()), (6, "c2".to_string())]),
        ]);

        let selected = round_robin_latency_heldout(&classes, 5);

        assert_eq!(
            selected
                .iter()
                .map(|(_, class, surface)| (*class, surface.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("a", "a1"),
                ("b", "b1"),
                ("c", "c1"),
                ("a", "a2"),
                ("b", "b2"),
            ]
        );
    }

    fn canonical_heldout(reservoir: HeldoutReservoir) -> Vec<(&'static str, u64, u32, String)> {
        let mut examples = reservoir
            .into_iter()
            .flat_map(|(class, heap)| {
                heap.into_vec()
                    .into_iter()
                    .map(move |(hash, terminal, surface)| (class, hash, terminal, surface))
            })
            .collect::<Vec<_>>();
        examples.sort_unstable();
        examples
    }
}
