use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io;
use std::path::Path;
use std::thread;
use std::time::Instant;

use serde::Serialize;

use super::compiler::{
    compile_with_policy, CompileDiagnostics, ForwardPostingPolicy, TrainingWord,
};
use super::corruption::{split_damages, DamageExample};
use super::format;
use super::restoration::{AbstainReason, RestorationReadout};
use super::runtime::{LexicalGrokkingMemory, ReadoutMode};

const MAX_MISS_DIAGNOSTICS_PER_CLASS: usize = 8;
const MAX_POSITION_DIAGNOSTICS: usize = 128;
const MAX_RECONSTRUCTION_DIAGNOSTICS_PER_CLASS: usize = 64;
const BASELINE_FORWARD_COUPLINGS: usize = 256;

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
    target_terminal: u32,
    selected_terminal: Option<u32>,
    target_rank: Option<usize>,
    selected_energy: Option<i32>,
    target_energy: Option<i32>,
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
pub struct L1LexicalGrokkingProof {
    verdict: &'static str,
    l11_verdict: &'static str,
    source_words: usize,
    training_surfaces: usize,
    heldout_surfaces: usize,
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
    calibration_max_geometry_distance: u8,
    calibration_min_positive_milli: u16,
    calibration_min_backward_milli: u16,
    artifact_bytes: usize,
    raw_corpus_stored: bool,
    exact_damage_episodes_stored: usize,
    compile_ms: u128,
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
    l11_hot_readout_p50_us: u64,
    l11_hot_readout_p99_us: u64,
    l11_hot_readout_max_us: u64,
    classes: BTreeMap<&'static str, ClassMetrics>,
    miss_diagnostics: Vec<MissDiagnostic>,
    position_diagnostics: Vec<PositionDiagnostic>,
    reconstruction_diagnostics: Vec<ReconstructionDiagnostic>,
    package: String,
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
    )
}

fn prove_l1_lexical_grokking_with_policy(
    corpus_path: &Path,
    output_path: &Path,
    max_words: usize,
    forward_policy: ForwardPostingPolicy,
    reuse_package: Option<&Path>,
) -> io::Result<serde_json::Value> {
    let text = std::fs::read_to_string(corpus_path)?;
    let words = corpus_words(&text, max_words);
    if words.len() < 8 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "L1 crystal proof requires at least eight unique words",
        ));
    }
    let mut training_words = Vec::with_capacity(words.len());
    let mut heldout = Vec::new();
    let mut ambiguity = HashMap::<String, BTreeSet<u32>>::new();
    let mut training_surfaces = 0;
    for (terminal_id, word) in words.iter().enumerate() {
        let (training, test) = split_damages(word);
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
        training_surfaces += training.len();
        heldout.extend(
            test.into_iter()
                .map(|example| (terminal_id as u32, example)),
        );
        training_words.push(TrainingWord {
            terminal_id: terminal_id as u32,
            surface: word.clone(),
            training_surfaces: training.into_iter().map(|item| item.surface).collect(),
        });
    }
    extend_sparse_omission_ambiguity(&words, &heldout, &mut ambiguity);

    let (package, bytes, compile_ms, diagnostics) = if let Some(package_path) = reuse_package {
        let bytes = std::fs::read(package_path)?;
        let package = format::decode(&bytes).map_err(io::Error::other)?;
        let max_forward_degree = package
            .atoms
            .iter()
            .map(|atom| usize::from(atom.coupling_count))
            .max()
            .unwrap_or_default();
        let diagnostics = CompileDiagnostics {
            forward_relations_before_policy: package.forward_couplings.len(),
            forward_relations_dropped: 0,
            forward_atoms_above_baseline_cap: package
                .atoms
                .iter()
                .filter(|atom| usize::from(atom.coupling_count) > BASELINE_FORWARD_COUPLINGS)
                .count(),
            max_forward_degree,
        };
        (package, bytes, 0, diagnostics)
    } else {
        let compile_started = Instant::now();
        let compiled =
            compile_with_policy(&training_words, forward_policy).map_err(io::Error::other)?;
        let package = compiled.package;
        let bytes = format::encode(&package).map_err(io::Error::other)?;
        let compile_ms = compile_started.elapsed().as_millis();
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let temporary = output_path.with_extension("tmp");
        std::fs::write(&temporary, &bytes)?;
        std::fs::rename(&temporary, output_path)?;
        (package, bytes, compile_ms, compiled.diagnostics)
    };

    let proof_started = Instant::now();
    let memory = LexicalGrokkingMemory::from_bytes(&bytes).map_err(io::Error::other)?;
    if reuse_package.is_some()
        && (package.terminal_count() as usize != words.len()
            || words.iter().enumerate().any(|(terminal_id, word)| {
                memory.decode_terminal(terminal_id as u32).as_deref() != Some(word.as_str())
            }))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L1 package terminal dictionary does not match proof corpus",
        ));
    }
    let clean_top1 = words
        .iter()
        .enumerate()
        .filter(|(target, word)| top_is(&memory, word, *target as u32, ReadoutMode::Full))
        .count();
    // Measure the hot path before proof-only decoder and edit-geometry audits
    // disturb caches or contend with the parallel evaluator.
    let latency = measure_hot_readout(&memory, &heldout);
    let l11_latency = measure_hot_restoration(&memory, &heldout);
    let evaluation = evaluate_parallel(&memory, &heldout, &ambiguity);
    let position_diagnostics = evaluation.position_diagnostics.clone();
    let metrics = evaluation.metrics;
    let restoration = aggregate_restoration(&metrics.classes);
    let clean_percent = percent(clean_top1, words.len());
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
    let all_l11_classes_pass = metrics.classes.values().all(|class| {
        class.restoration.geometry_unique_winner_percent > 95.0
            && class.restoration.geometry_tied_safe_percent == 100.0
            && class.restoration.false_singleton_on_geometry_tie == 0
    });
    let l11_verdict = if clean_percent >= 99.9
        && l11_geometry_unique_winner_percent > 98.8
        && l11_geometry_tied_safe_percent == 100.0
        && restoration.false_singleton_on_geometry_tie == 0
        && percentile(&l11_latency, 99) <= 5_000
        && all_l11_classes_pass
    {
        "PASS_shadow"
    } else {
        "WATCH_shadow"
    };
    let proof = L1LexicalGrokkingProof {
        verdict,
        l11_verdict,
        source_words: words.len(),
        training_surfaces,
        heldout_surfaces: heldout.len(),
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
        word_center_bank_bytes: package.centers.len() * 64,
        atom_count: package.atoms.len(),
        forward_couplings: package.forward_couplings.len(),
        forward_posting_policy: forward_policy.name(),
        forward_relations_before_policy: diagnostics.forward_relations_before_policy,
        forward_relations_dropped: diagnostics.forward_relations_dropped,
        forward_atoms_above_baseline_cap: diagnostics.forward_atoms_above_baseline_cap,
        max_forward_degree: diagnostics.max_forward_degree,
        reverse_couplings: package.reverse_couplings.len(),
        anti_centers: package.anti_centers.len(),
        pair_profiles: package.pair_profiles.len(),
        pair_centers: package.pair_centers.len(),
        center_phase_profiles: package.center_phase_profiles.len(),
        positive_subcenters: package.positive_subcenters.len(),
        anti_subcenters: package.anti_subcenters.len(),
        hard_negative_subcenters: package.hard_negative_subcenters.len(),
        calibration_max_geometry_distance: package.restoration_calibration.max_geometry_distance,
        calibration_min_positive_milli: package.restoration_calibration.min_positive_milli,
        calibration_min_backward_milli: package.restoration_calibration.min_backward_milli,
        artifact_bytes: bytes.len(),
        raw_corpus_stored: false,
        exact_damage_episodes_stored: 0,
        compile_ms,
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
        l11_hot_readout_p50_us: percentile(&l11_latency, 50),
        l11_hot_readout_p99_us: percentile(&l11_latency, 99),
        l11_hot_readout_max_us: l11_latency.last().copied().unwrap_or_default(),
        classes: metrics.classes,
        miss_diagnostics: metrics.misses,
        position_diagnostics,
        reconstruction_diagnostics: evaluation.reconstruction_diagnostics,
        package: output_path.display().to_string(),
    };
    serde_json::to_value(proof).map_err(io::Error::other)
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
}

fn evaluate_parallel(
    memory: &LexicalGrokkingMemory,
    heldout: &[(u32, DamageExample)],
    ambiguity: &HashMap<String, BTreeSet<u32>>,
) -> Evaluation {
    let worker_count = proof_worker_count(heldout.len());
    let partial = thread::scope(|scope| {
        (0..worker_count)
            .map(|worker| {
                scope.spawn(move || {
                    evaluate_cases(
                        memory,
                        heldout.iter().skip(worker).step_by(worker_count),
                        ambiguity,
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
        let count = miss_counts.entry(miss.class).or_insert(0_usize);
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

fn proof_worker_count(case_count: usize) -> usize {
    thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(case_count.max(1))
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
) -> Evaluation {
    let mut evaluation = Evaluation::default();
    for (target, example) in heldout {
        let candidates = memory.readout(&example.surface, 64, ReadoutMode::Full);
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
        let phase_top1 = top_is(memory, &example.surface, *target, ReadoutMode::WithoutPhase);
        let anti_top1 = top_is(memory, &example.surface, *target, ReadoutMode::WithoutAnti);
        let sequence_top1 = top_is(
            memory,
            &example.surface,
            *target,
            ReadoutMode::WithoutSequence,
        );
        let legacy_sequence_top1 = top_is(
            memory,
            &example.surface,
            *target,
            ReadoutMode::LegacySequence,
        );
        let without_sequence_certificate_top1 = top_is(
            memory,
            &example.surface,
            *target,
            ReadoutMode::WithoutSequenceCertificate,
        );
        let without_pairwise_top1 = top_is(
            memory,
            &example.surface,
            *target,
            ReadoutMode::WithoutPairwise,
        );
        let without_position_candidates =
            memory.readout(&example.surface, 64, ReadoutMode::WithoutPosition);
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
        let targets = &ambiguity[example.surface.as_str()];
        let class = evaluation.metrics.classes.entry(example.class).or_default();
        record_restoration(
            &mut class.restoration,
            &restoration_candidates,
            &restoration,
            *target,
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
            class.false_certainty += usize::from(selected_outside_set);
        }
        if !top1 {
            let rank = candidates
                .iter()
                .position(|item| item.terminal_id == *target);
            evaluation.metrics.misses.push(MissDiagnostic {
                class: example.class,
                surface: example.surface.clone(),
                target_terminal: *target,
                selected_terminal: candidates.first().map(|item| item.terminal_id),
                target_rank: rank.map(|value| value + 1),
                selected_energy: candidates.first().map(|item| item.settled_energy),
                target_energy: rank
                    .and_then(|value| candidates.get(value))
                    .map(|item| item.settled_energy),
            });
        }
    }
    evaluation
}

fn record_restoration(
    metrics: &mut RestorationMetrics,
    candidates: &[super::runtime::GrokkingCandidate],
    readout: &RestorationReadout,
    target_terminal: u32,
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

fn top_is(memory: &LexicalGrokkingMemory, surface: &str, target: u32, mode: ReadoutMode) -> bool {
    memory
        .readout(surface, 1, mode)
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
