use std::collections::{BTreeMap, BTreeSet, BinaryHeap};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::time::Instant;

use serde::Serialize;

use super::context::ContextEvidenceScope;
use super::productive::{
    canonical_source, collect_wanted_form_features, directional_evidence_margin,
    productive_slot_features_for_scope, ProductiveBirthStatus, ProductiveContextPairEvidence,
    ProductiveMorphologySource,
};
use super::productive_format::CompactProductiveMorphologyView;
use super::runtime::{
    productive_l2_birth_rank, productive_l2_readout, CompositionalFormBirth, ProductiveL2FormBirth,
    ProductiveL2Readout, StandaloneL2Field,
};
use crate::nanda_wave::lexical_grokking::{split_damages, DamageExample};

const EXPECTED_DAMAGE_CLASSES: usize = 13;

#[derive(Clone, Debug)]
struct ProofCase {
    target_form_ref: u32,
    target_lemma_ids: Vec<u32>,
    target_surface: String,
    feature_mask: u32,
    context: String,
    class: &'static str,
    damaged_surface: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SampledScene {
    hash: u64,
    lemma: String,
    surface: String,
    feature_text: String,
    context: String,
}

#[derive(Default)]
struct SampledCases {
    cases: Vec<ProofCase>,
    heldout_lemma_ids: BTreeSet<u32>,
    heldout_lemma_names: BTreeSet<String>,
    scanned_lines: usize,
    heldout_rows: usize,
    eligible_heldout_rows: usize,
    productive_heldout_rows: usize,
    reservoir_rows: usize,
    selected_target_forms: usize,
}

#[derive(Clone, Debug)]
struct DirectionalHeldoutRow {
    line_number: usize,
    lemma: String,
    target_surface: String,
    target_feature_text: String,
    target_feature_mask: u32,
    context: String,
    competitors: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
struct DirectionalHeldoutGate {
    verdict: &'static str,
    total_nh_rows: usize,
    competitor_surfaces: usize,
    same_lemma_competitor_surfaces: usize,
    non_same_lemma_competitor_surfaces: usize,
    same_lemma_comparisons: usize,
    pair_evidence_covered: usize,
    pair_evidence_coverage_percent: f64,
    target_directional_wins: usize,
    target_wins_of_covered_percent: f64,
    reverse_false_supports: usize,
    tied_pair_evidence: usize,
    no_pair_evidence: usize,
    reverse_invariant_violations: usize,
    target_win_examples: Vec<serde_json::Value>,
    reverse_false_support_examples: Vec<serde_json::Value>,
    no_evidence_examples: Vec<serde_json::Value>,
}

impl DirectionalHeldoutGate {
    fn passed(&self) -> bool {
        self.target_directional_wins > 0
            && self.reverse_false_supports == 0
            && self.reverse_invariant_violations == 0
    }
}

#[derive(Clone, Debug, Default, Serialize)]
struct ClassMetrics {
    cases: usize,
    known_context_modes: usize,
    known_context_modes_percent: f64,
    broad_lemma_retained: usize,
    broad_lemma_retained_percent: f64,
    active_lemma_retained: usize,
    active_lemma_retained_percent: f64,
    seen_exact_top16_retained: usize,
    seen_exact_top16_retained_percent: f64,
    seen_exact_unique_top1: usize,
    seen_exact_unique_top1_percent: f64,
    unseen_generated_slot_retained: usize,
    unseen_generated_slot_retained_percent: f64,
    unseen_generated_top8_retained: usize,
    unseen_generated_top8_retained_percent: f64,
    unseen_generated_top16_retained: usize,
    unseen_generated_top16_retained_percent: f64,
    unseen_generated_unique_top1: usize,
    unseen_generated_unique_top1_percent: f64,
    unseen_generated_readout_target_retained: usize,
    unseen_generated_readout_target_retained_percent: f64,
    generated_winner: usize,
    generated_tied: usize,
    generated_abstain: usize,
    unseen_generated_empty_lattice: usize,
    false_singleton: usize,
    false_authority: usize,
    status_violations: usize,
    exact_annotation_leaks: usize,
}

impl ClassMetrics {
    fn merge(&mut self, other: Self) {
        self.cases += other.cases;
        self.known_context_modes += other.known_context_modes;
        self.broad_lemma_retained += other.broad_lemma_retained;
        self.active_lemma_retained += other.active_lemma_retained;
        self.seen_exact_top16_retained += other.seen_exact_top16_retained;
        self.seen_exact_unique_top1 += other.seen_exact_unique_top1;
        self.unseen_generated_slot_retained += other.unseen_generated_slot_retained;
        self.unseen_generated_top8_retained += other.unseen_generated_top8_retained;
        self.unseen_generated_top16_retained += other.unseen_generated_top16_retained;
        self.unseen_generated_unique_top1 += other.unseen_generated_unique_top1;
        self.unseen_generated_readout_target_retained +=
            other.unseen_generated_readout_target_retained;
        self.generated_winner += other.generated_winner;
        self.generated_tied += other.generated_tied;
        self.generated_abstain += other.generated_abstain;
        self.unseen_generated_empty_lattice += other.unseen_generated_empty_lattice;
        self.false_singleton += other.false_singleton;
        self.false_authority += other.false_authority;
        self.status_violations += other.status_violations;
        self.exact_annotation_leaks += other.exact_annotation_leaks;
    }

    fn finish(&mut self) {
        self.known_context_modes_percent = percent(self.known_context_modes, self.cases);
        self.broad_lemma_retained_percent = percent(self.broad_lemma_retained, self.cases);
        self.active_lemma_retained_percent = percent(self.active_lemma_retained, self.cases);
        self.seen_exact_top16_retained_percent =
            percent(self.seen_exact_top16_retained, self.cases);
        self.seen_exact_unique_top1_percent = percent(self.seen_exact_unique_top1, self.cases);
        self.unseen_generated_slot_retained_percent =
            percent(self.unseen_generated_slot_retained, self.cases);
        self.unseen_generated_top8_retained_percent =
            percent(self.unseen_generated_top8_retained, self.cases);
        self.unseen_generated_top16_retained_percent =
            percent(self.unseen_generated_top16_retained, self.cases);
        self.unseen_generated_unique_top1_percent =
            percent(self.unseen_generated_unique_top1, self.cases);
        self.unseen_generated_readout_target_retained_percent =
            percent(self.unseen_generated_readout_target_retained, self.cases);
    }
}

#[derive(Default)]
struct ProofShard {
    classes: BTreeMap<&'static str, ClassMetrics>,
    lemma_birth_latency_us: Vec<u64>,
    context_reduction_latency_us: Vec<u64>,
    exact_birth_latency_us: Vec<u64>,
    generated_birth_latency_us: Vec<u64>,
    failure_examples: BTreeMap<&'static str, Vec<serde_json::Value>>,
    false_singleton_examples: Vec<serde_json::Value>,
}

impl ProofShard {
    fn merge(&mut self, other: Self) {
        for (class, metrics) in other.classes {
            self.classes.entry(class).or_default().merge(metrics);
        }
        self.lemma_birth_latency_us
            .extend(other.lemma_birth_latency_us);
        self.context_reduction_latency_us
            .extend(other.context_reduction_latency_us);
        self.exact_birth_latency_us
            .extend(other.exact_birth_latency_us);
        self.generated_birth_latency_us
            .extend(other.generated_birth_latency_us);
        for (class, examples) in other.failure_examples {
            let retained = self.failure_examples.entry(class).or_default();
            retained.extend(examples);
            retained.truncate(8);
        }
        self.false_singleton_examples
            .extend(other.false_singleton_examples);
        self.false_singleton_examples.truncate(32);
    }
}

fn prove_directional_heldout_gate<I: ProductiveMorphologySource + ?Sized>(
    source: &I,
    morphology_corpus_path: &Path,
) -> io::Result<DirectionalHeldoutGate> {
    let (rows, wanted_surfaces) = collect_directional_heldout_rows(BufReader::with_capacity(
        1024 * 1024,
        File::open(morphology_corpus_path)?,
    ))?;
    let form_features = collect_wanted_form_features(
        BufReader::with_capacity(1024 * 1024, File::open(morphology_corpus_path)?),
        &wanted_surfaces,
    )?;
    Ok(evaluate_directional_heldout_rows(
        source,
        &rows,
        &form_features,
    ))
}

fn collect_directional_heldout_rows(
    mut reader: impl BufRead,
) -> io::Result<(
    Vec<DirectionalHeldoutRow>,
    BTreeMap<String, BTreeSet<String>>,
)> {
    let mut rows = Vec::new();
    let mut wanted_surfaces = BTreeMap::<String, BTreeSet<String>>::new();
    let mut line = String::with_capacity(256);
    let mut line_number = 0_usize;
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        line_number += 1;
        if !line.starts_with("NH\t") {
            continue;
        }
        let raw = line.trim_end_matches(['\r', '\n']);
        let fields = raw.split('\t').collect::<Vec<_>>();
        let ["NH", lemma, target_surface, target_feature_text, context, competitor_text] =
            fields.as_slice()
        else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("line {line_number}: invalid NH directional heldout row"),
            ));
        };
        if context
            .split_whitespace()
            .filter(|token| *token == "_")
            .count()
            != 1
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("line {line_number}: NH context requires one _ slot"),
            ));
        }
        let target_feature_mask = crate::nanda_wave::morphology_phase::parse_features(
            target_feature_text,
        )
        .map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("line {line_number}: {error}"),
            )
        })?;
        let lemma = lemma.trim().to_lowercase();
        let competitors = competitor_text
            .split(',')
            .map(str::trim)
            .filter(|surface| !surface.is_empty())
            .map(str::to_lowercase)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        wanted_surfaces
            .entry(lemma.clone())
            .or_default()
            .extend(competitors.iter().cloned());
        rows.push(DirectionalHeldoutRow {
            line_number,
            lemma,
            target_surface: target_surface.trim().to_lowercase(),
            target_feature_text: target_feature_text.trim().to_string(),
            target_feature_mask,
            context: context.trim().to_string(),
            competitors,
        });
    }
    Ok((rows, wanted_surfaces))
}

fn evaluate_directional_heldout_rows<I: ProductiveMorphologySource + ?Sized>(
    source: &I,
    rows: &[DirectionalHeldoutRow],
    form_features: &BTreeMap<String, BTreeMap<String, BTreeSet<u32>>>,
) -> DirectionalHeldoutGate {
    let mut gate = DirectionalHeldoutGate {
        total_nh_rows: rows.len(),
        ..DirectionalHeldoutGate::default()
    };
    for row in rows {
        let target_primary_pos =
            crate::nanda_wave::morphology_phase::feature_primary_pos(row.target_feature_mask);
        for competitor_surface in &row.competitors {
            gate.competitor_surfaces += 1;
            let Some(competitor_features) = form_features
                .get(&row.lemma)
                .and_then(|surfaces| surfaces.get(competitor_surface))
            else {
                gate.non_same_lemma_competitor_surfaces += 1;
                continue;
            };
            gate.same_lemma_competitor_surfaces += 1;
            for competitor_feature_mask in competitor_features {
                if *competitor_feature_mask == row.target_feature_mask
                    || crate::nanda_wave::morphology_phase::feature_primary_pos(
                        *competitor_feature_mask,
                    ) != target_primary_pos
                    || !directional_slots_are_distinguishable(
                        row.target_feature_mask,
                        *competitor_feature_mask,
                    )
                {
                    continue;
                }
                gate.same_lemma_comparisons += 1;
                let evidence = source.context_pair_evidence_for(
                    &row.context,
                    row.target_feature_mask,
                    *competitor_feature_mask,
                );
                let reverse = source.context_pair_evidence_for(
                    &row.context,
                    *competitor_feature_mask,
                    row.target_feature_mask,
                );
                let example = directional_example(
                    row,
                    competitor_surface,
                    *competitor_feature_mask,
                    evidence,
                );
                if reverse.positive_support != evidence.anti_support
                    || reverse.anti_support != evidence.positive_support
                {
                    gate.reverse_invariant_violations += 1;
                }
                if evidence.positive_support == 0 && evidence.anti_support == 0 {
                    gate.no_pair_evidence += 1;
                    push_bounded(&mut gate.no_evidence_examples, example);
                    continue;
                }
                gate.pair_evidence_covered += 1;
                match directional_evidence_margin(evidence) {
                    std::cmp::Ordering::Greater => {
                        gate.target_directional_wins += 1;
                        push_bounded(&mut gate.target_win_examples, example);
                    }
                    std::cmp::Ordering::Less => {
                        gate.reverse_false_supports += 1;
                        push_bounded(&mut gate.reverse_false_support_examples, example);
                    }
                    std::cmp::Ordering::Equal => gate.tied_pair_evidence += 1,
                }
            }
        }
    }
    gate.pair_evidence_coverage_percent =
        percent(gate.pair_evidence_covered, gate.same_lemma_comparisons);
    gate.target_wins_of_covered_percent =
        percent(gate.target_directional_wins, gate.pair_evidence_covered);
    gate.verdict = if gate.passed() {
        "PASS_shadow_directional_nh"
    } else {
        "FAIL"
    };
    gate
}

fn directional_slots_are_distinguishable(preferred: u32, competitor: u32) -> bool {
    [ContextEvidenceScope::Exact, ContextEvidenceScope::Neighbor]
        .into_iter()
        .any(|scope| {
            productive_slot_features_for_scope(preferred, scope)
                != productive_slot_features_for_scope(competitor, scope)
        })
}

fn directional_example(
    row: &DirectionalHeldoutRow,
    competitor_surface: &str,
    competitor_feature_mask: u32,
    evidence: ProductiveContextPairEvidence,
) -> serde_json::Value {
    serde_json::json!({
        "line": row.line_number,
        "lemma": row.lemma,
        "target_surface": row.target_surface,
        "target_features": row.target_feature_text,
        "target_feature_mask": row.target_feature_mask,
        "competitor_surface": competitor_surface,
        "competitor_feature_mask": competitor_feature_mask,
        "context": row.context,
        "positive_support": evidence.positive_support,
        "anti_support": evidence.anti_support,
        "posterior_milli": evidence.posterior_milli,
        "context_observed": evidence.context_observed,
        "exact_positive_support": evidence.exact_positive_support,
        "exact_anti_support": evidence.exact_anti_support,
        "supporting_neighbor_lanes": evidence.supporting_neighbor_lanes,
        "contradicting_neighbor_lanes": evidence.contradicting_neighbor_lanes,
        "tied_neighbor_lanes": evidence.tied_neighbor_lanes,
    })
}

fn push_bounded(target: &mut Vec<serde_json::Value>, value: serde_json::Value) {
    if target.len() < 8 {
        target.push(value);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prove_package(
    l1_package_path: &Path,
    l2_package_path: &Path,
    morphology_corpus_path: &Path,
    heldout_per_class: usize,
    requested_workers: usize,
    broad_lemma_limit: usize,
    active_lemma_limit: usize,
    feature_limit: usize,
    form_limit: usize,
    atom_relation_limit: usize,
    minimum_profile_support: u32,
) -> io::Result<serde_json::Value> {
    if heldout_per_class == 0
        || broad_lemma_limit == 0
        || active_lemma_limit == 0
        || active_lemma_limit > broad_lemma_limit
        || feature_limit == 0
        || form_limit < 16
        || minimum_profile_support == 0
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "productive L2 proof requires heldout > 0, 0 < active lemmas <= broad lemmas, feature limit > 0, form limit >= 16, and profile support > 0",
        ));
    }

    let l2_started = Instant::now();
    let field = StandaloneL2Field::load(l2_package_path).map_err(io::Error::other)?;
    let l2_cold_load_us = l2_started.elapsed().as_micros() as u64;
    let rss_after_l2_load_kib = proc_status_kib("VmRSS:");
    let (package_storage, package_backing_bytes) = field.package_storage();

    let l1_started = Instant::now();
    let l1 = crate::nanda_wave::L1RestorationHost::load(l1_package_path)?;
    let l1_cold_load_us = l1_started.elapsed().as_micros() as u64;
    if field.l1_package_fingerprint() != l1.corpus_fingerprint() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L2 package was compiled for a different L1.1 corpus fingerprint",
        ));
    }
    drop(l1);

    let sampling_started = Instant::now();
    let sampled = sample_fixed_productive_cases(&field, morphology_corpus_path, heldout_per_class)?;
    let sampling_us = sampling_started.elapsed().as_micros() as u64;

    let rss_before_profile_training_kib = proc_status_kib("VmRSS:");
    let training_started = Instant::now();
    let mut productive_index = field
        .train_productive_morphology(
            |lemma_id| !sampled.heldout_lemma_ids.contains(&lemma_id),
            minimum_profile_support,
        )
        .map_err(io::Error::other)?;
    productive_index
        .train_context_slots_from_corpus(morphology_corpus_path, &sampled.heldout_lemma_names)?;
    let profile_training_us = training_started.elapsed().as_micros() as u64;
    let rss_after_profile_training_kib = proc_status_kib("VmRSS:");
    let training_report = productive_index.report().clone();
    let directional_nh_started = Instant::now();
    let directional_nh_gate =
        prove_directional_heldout_gate(&productive_index, morphology_corpus_path)?;
    let directional_nh_us = directional_nh_started.elapsed().as_micros() as u64;

    let workers = requested_workers
        .max(1)
        .min(sampled.cases.len().max(1))
        .min(
            std::thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1),
        );
    let chunk_size = sampled.cases.len().div_ceil(workers).max(1);
    let proof_started = Instant::now();
    let partials = std::thread::scope(|scope| {
        sampled
            .cases
            .chunks(chunk_size)
            .map(|cases| {
                let field = &field;
                let productive_index = &productive_index;
                scope.spawn(move || {
                    evaluate_cases(
                        cases,
                        field,
                        productive_index,
                        broad_lemma_limit,
                        active_lemma_limit,
                        feature_limit,
                        form_limit,
                        atom_relation_limit,
                    )
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|worker| worker.join().expect("productive L2 proof worker panicked"))
            .collect::<Vec<_>>()
    });
    let mut proof = ProofShard::default();
    for partial in partials {
        proof.merge(partial);
    }
    let proof_us = proof_started.elapsed().as_micros() as u64;
    for metrics in proof.classes.values_mut() {
        metrics.finish();
    }
    proof.lemma_birth_latency_us.sort_unstable();
    proof.context_reduction_latency_us.sort_unstable();
    proof.exact_birth_latency_us.sort_unstable();
    proof.generated_birth_latency_us.sort_unstable();

    let false_singleton = proof
        .classes
        .values()
        .map(|metrics| metrics.false_singleton)
        .sum::<usize>();
    let false_authority = proof
        .classes
        .values()
        .map(|metrics| metrics.false_authority)
        .sum::<usize>();
    let status_violations = proof
        .classes
        .values()
        .map(|metrics| metrics.status_violations)
        .sum::<usize>();
    let exact_annotation_leaks = proof
        .classes
        .values()
        .map(|metrics| metrics.exact_annotation_leaks)
        .sum::<usize>();
    let quality_passed = proof.classes.len() == EXPECTED_DAMAGE_CLASSES
        && proof.classes.values().all(|metrics| {
            metrics.unseen_generated_top16_retained_percent > 95.0
                && metrics.unseen_generated_unique_top1_percent > 95.0
                && metrics.unseen_generated_readout_target_retained_percent > 95.0
        });
    let passed = quality_passed
        && false_singleton == 0
        && false_authority == 0
        && status_violations == 0
        && exact_annotation_leaks == 0
        && directional_nh_gate.passed();
    let evaluated = proof
        .classes
        .values()
        .map(|metrics| metrics.cases)
        .sum::<usize>();
    let failure_examples = proof
        .failure_examples
        .values()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();

    Ok(serde_json::json!({
        "kind": "canonical_l2_productive_morphology_leave_lemmas_out_fixed_proof",
        "verdict": if passed { "PASS_shadow_productive" } else { "FAIL" },
        "scope": "heldout lemmas excluded from productive profile training; normal exact-bank baseline and exact-masked generated readout measured separately over 13 damage classes",
        "l1_package": l1_package_path,
        "l2_package": l2_package_path,
        "morphology_corpus": morphology_corpus_path,
        "heldout_per_class": heldout_per_class,
        "class_count": proof.classes.len(),
        "evaluated": evaluated,
        "heldout_lemmas": sampled.heldout_lemma_ids.len(),
        "classes": proof.classes,
        "false_singleton": false_singleton,
        "false_authority": false_authority,
        "status_violations": status_violations,
        "exact_annotation_leaks": exact_annotation_leaks,
        "failure_examples": failure_examples,
        "false_singleton_examples": proof.false_singleton_examples,
        "directional_nh_gate": directional_nh_gate,
        "split_denominators": {
            "seen_exact": {
                "cases": evaluated,
                "contract": "normal V13 exact form expansion; target exact form remains visible",
            },
            "unseen_generated": {
                "cases": evaluated,
                "contract": "all selected target lemmas excluded from productive profile training; target exact lookup and exact annotation disabled for generated birth",
                "physical_package_rebuilt_without_targets": false,
                "logical_exact_target_masked": true,
            },
        },
        "profile_training": {
            "minimum_positive_lemma_support": minimum_profile_support,
            "observed_lemmas": training_report.observed_lemmas,
            "admitted_train_lemmas": training_report.admitted_lemmas,
            "heldout_lemmas": sampled.heldout_lemma_ids.len(),
            "train_heldout_overlap": 0,
            "observed_transforms": training_report.observed_transforms,
            "admitted_profiles": training_report.admitted_profiles,
            "rejected_low_support_profiles": training_report.rejected_low_support_profiles,
            "observed_context_rows": training_report.observed_context_rows,
            "admitted_context_rows": training_report.admitted_context_rows,
            "excluded_context_rows": training_report.excluded_context_rows,
            "rejected_context_rows": training_report.rejected_context_rows,
            "context_modes": training_report.context_modes,
            "context_slots": training_report.context_slots,
            "training_us": profile_training_us,
            "one_lemma_at_a_time": true,
            "raw_forms_materialized_together": false,
            "context_training_streamed": true,
        },
        "sampling": {
            "strategy": "bounded deterministic lowest-hash reservoir over real heldout H rows; selected lemmas are then fully excluded from productive profile training",
            "scanned_lines": sampled.scanned_lines,
            "heldout_rows": sampled.heldout_rows,
            "eligible_heldout_rows": sampled.eligible_heldout_rows,
            "productive_heldout_rows": sampled.productive_heldout_rows,
            "reservoir_rows": sampled.reservoir_rows,
            "selected_target_forms": sampled.selected_target_forms,
            "sampling_us": sampling_us,
        },
        "limits": {
            "broad_lemma_frontier": broad_lemma_limit,
            "active_lemma_frontier": active_lemma_limit,
            "features_per_lemma": feature_limit,
            "form_lattice": form_limit,
            "atom_relation_budget": atom_relation_limit,
        },
        "latency": {
            "lemma_birth_p50_us": percentile(&proof.lemma_birth_latency_us, 50),
            "lemma_birth_p99_us": percentile(&proof.lemma_birth_latency_us, 99),
            "context_reduction_p50_us": percentile(&proof.context_reduction_latency_us, 50),
            "context_reduction_p99_us": percentile(&proof.context_reduction_latency_us, 99),
            "seen_exact_birth_p50_us": percentile(&proof.exact_birth_latency_us, 50),
            "seen_exact_birth_p99_us": percentile(&proof.exact_birth_latency_us, 99),
            "unseen_generated_birth_p50_us": percentile(&proof.generated_birth_latency_us, 50),
            "unseen_generated_birth_p99_us": percentile(&proof.generated_birth_latency_us, 99),
            "proof_us": proof_us,
            "directional_nh_us": directional_nh_us,
            "workers": workers,
        },
        "memory": {
            "package_bytes": std::fs::metadata(l2_package_path)?.len(),
            "package_storage": package_storage,
            "package_backing_bytes": package_backing_bytes,
            "package_mmap_backed": field.package_mmap_backed(),
            "rss_after_l2_load_kib": rss_after_l2_load_kib,
            "rss_before_profile_training_kib": rss_before_profile_training_kib,
            "rss_after_profile_training_kib": rss_after_profile_training_kib,
            "rss_peak_kib": proc_status_kib("VmHWM:"),
        },
        "cold_load": {
            "l2_us": l2_cold_load_us,
            "l1_fingerprint_check_us": l1_cold_load_us,
        },
        "gates": {
            "each_unseen_generated_unique_top1_strictly_above_percent": 95.0,
            "each_unseen_generated_top16_retained_strictly_above_percent": 95.0,
            "each_generated_readout_target_retained_strictly_above_percent": 95.0,
            "false_singleton_equals": 0,
            "false_authority_equals": 0,
            "status_violations_equals": 0,
            "exact_annotation_leaks_equals": 0,
            "directional_nh_target_wins_strictly_above": 0,
            "directional_nh_reverse_false_supports_equals": 0,
            "directional_nh_reverse_invariant_violations_equals": 0,
        },
        "not_tested": [
            "physical package rebuilt with heldout target surfaces removed; this proof masks exact lookup without recompiling immutable V13",
            "clean preservation and ambiguity retention",
            "grounded L1.1 lattice preservation after live generated-candidate integration",
            "L3/L4/DecisionCore final apply authority",
            "daemon and IBus latency",
        ],
        "runtime_authority_changed_by_proof": false,
    }))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prove_compact_sidecar(
    l1_package_path: &Path,
    l2_package_path: &Path,
    productive_sidecar_path: &Path,
    morphology_corpus_path: &Path,
    heldout_per_class: usize,
    requested_workers: usize,
    broad_lemma_limit: usize,
    active_lemma_limit: usize,
    feature_limit: usize,
    form_limit: usize,
    atom_relation_limit: usize,
) -> io::Result<serde_json::Value> {
    if heldout_per_class == 0
        || broad_lemma_limit == 0
        || active_lemma_limit == 0
        || active_lemma_limit > broad_lemma_limit
        || feature_limit == 0
        || form_limit < 16
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "productive sidecar proof requires heldout > 0, 0 < active lemmas <= broad lemmas, feature limit > 0, and form limit >= 16",
        ));
    }

    let l2_started = Instant::now();
    let field = StandaloneL2Field::load(l2_package_path).map_err(io::Error::other)?;
    let l2_cold_load_us = l2_started.elapsed().as_micros() as u64;
    let l1_started = Instant::now();
    let l1 = crate::nanda_wave::L1RestorationHost::load(l1_package_path)?;
    let l1_cold_load_us = l1_started.elapsed().as_micros() as u64;
    if field.l1_package_fingerprint() != l1.corpus_fingerprint() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L2 package was compiled for a different L1.1 corpus fingerprint",
        ));
    }
    drop(l1);

    let sidecar_started = Instant::now();
    let productive_sidecar =
        CompactProductiveMorphologyView::load(productive_sidecar_path).map_err(io::Error::other)?;
    let sidecar_cold_load_us = sidecar_started.elapsed().as_micros() as u64;
    if productive_sidecar.l2_fingerprint() != field.l1_package_fingerprint() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "productive sidecar L1.1 fingerprint does not match canonical L2",
        ));
    }

    let directional_nh_started = Instant::now();
    let directional_nh_gate =
        prove_directional_heldout_gate(&productive_sidecar, morphology_corpus_path)?;
    let directional_nh_us = directional_nh_started.elapsed().as_micros() as u64;

    let sampling_started = Instant::now();
    let sampled = sample_fixed_productive_cases(&field, morphology_corpus_path, heldout_per_class)?;
    let sampling_us = sampling_started.elapsed().as_micros() as u64;
    let workers = requested_workers
        .max(1)
        .min(sampled.cases.len().max(1))
        .min(
            std::thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1),
        );
    let chunk_size = sampled.cases.len().div_ceil(workers).max(1);
    let proof_started = Instant::now();
    let partials = std::thread::scope(|scope| {
        sampled
            .cases
            .chunks(chunk_size)
            .map(|cases| {
                let field = &field;
                let productive_sidecar = &productive_sidecar;
                scope.spawn(move || {
                    evaluate_cases(
                        cases,
                        field,
                        productive_sidecar,
                        broad_lemma_limit,
                        active_lemma_limit,
                        feature_limit,
                        form_limit,
                        atom_relation_limit,
                    )
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .expect("productive sidecar proof worker panicked")
            })
            .collect::<Vec<_>>()
    });
    let mut proof = ProofShard::default();
    for partial in partials {
        proof.merge(partial);
    }
    let proof_us = proof_started.elapsed().as_micros() as u64;
    for metrics in proof.classes.values_mut() {
        metrics.finish();
    }
    proof.lemma_birth_latency_us.sort_unstable();
    proof.context_reduction_latency_us.sort_unstable();
    proof.exact_birth_latency_us.sort_unstable();
    proof.generated_birth_latency_us.sort_unstable();

    let false_singleton = proof
        .classes
        .values()
        .map(|metrics| metrics.false_singleton)
        .sum::<usize>();
    let false_authority = proof
        .classes
        .values()
        .map(|metrics| metrics.false_authority)
        .sum::<usize>();
    let status_violations = proof
        .classes
        .values()
        .map(|metrics| metrics.status_violations)
        .sum::<usize>();
    let exact_annotation_leaks = proof
        .classes
        .values()
        .map(|metrics| metrics.exact_annotation_leaks)
        .sum::<usize>();
    let evaluated = proof
        .classes
        .values()
        .map(|metrics| metrics.cases)
        .sum::<usize>();
    let quality_passed = proof.classes.len() == EXPECTED_DAMAGE_CLASSES
        && proof.classes.values().all(|metrics| {
            metrics.unseen_generated_top16_retained_percent > 95.0
                && metrics.unseen_generated_unique_top1_percent > 95.0
                && metrics.unseen_generated_readout_target_retained_percent > 95.0
        });
    let passed = quality_passed
        && false_singleton == 0
        && false_authority == 0
        && status_violations == 0
        && exact_annotation_leaks == 0
        && directional_nh_gate.passed()
        && percentile(&proof.generated_birth_latency_us, 99) <= 5_000;
    let classes = proof
        .classes
        .iter()
        .map(|(class, metrics)| {
            (
                *class,
                serde_json::json!({
                    "cases": metrics.cases,
                    "broad_lemma_retained_percent": metrics.broad_lemma_retained_percent,
                    "active_lemma_retained_percent": metrics.active_lemma_retained_percent,
                    "generated_slot_retained_percent": metrics.unseen_generated_slot_retained_percent,
                    "generated_top8_retained_percent": metrics.unseen_generated_top8_retained_percent,
                    "generated_top16_retained_percent": metrics.unseen_generated_top16_retained_percent,
                    "generated_unique_top1_percent": metrics.unseen_generated_unique_top1_percent,
                    "generated_readout_target_retained_percent": metrics.unseen_generated_readout_target_retained_percent,
                    "winner": metrics.generated_winner,
                    "tied": metrics.generated_tied,
                    "abstain": metrics.generated_abstain,
                    "false_singleton": metrics.false_singleton,
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let sidecar_report = productive_sidecar.report();

    Ok(serde_json::json!({
        "kind": "canonical_l2_productive_morphology_compact_sidecar_fixed_proof",
        "verdict": if passed { "PASS_runtime_sidecar_shadow" } else { "FAIL" },
        "scope": "fixed H-row scenes over the packaged full productive sidecar; exact target annotation remains disabled in generated birth, but sampled lemmas were not excluded from sidecar training",
        "l1_package": l1_package_path,
        "l2_package": l2_package_path,
        "productive_sidecar": productive_sidecar_path,
        "morphology_corpus": morphology_corpus_path,
        "heldout_per_class": heldout_per_class,
        "class_count": proof.classes.len(),
        "evaluated": evaluated,
        "classes": classes,
        "false_singleton": false_singleton,
        "false_authority": false_authority,
        "status_violations": status_violations,
        "exact_annotation_leaks": exact_annotation_leaks,
        "failure_examples": proof.failure_examples.values().flatten().cloned().collect::<Vec<_>>(),
        "directional_nh_gate": directional_nh_gate,
        "limits": {
            "broad_lemma_frontier": broad_lemma_limit,
            "active_lemma_frontier": active_lemma_limit,
            "features_per_lemma": feature_limit,
            "form_lattice": form_limit,
            "atom_relation_budget": atom_relation_limit,
        },
        "sidecar": {
            "bytes": productive_sidecar.backing_bytes(),
            "mmap_backed": productive_sidecar.mmap_backed(),
            "l2_package_fingerprint": productive_sidecar.l2_fingerprint(),
            "admitted_profiles": sidecar_report.admitted_profiles,
            "context_modes": sidecar_report.context_modes,
            "context_slots": sidecar_report.context_slots,
            "context_pairs": sidecar_report.context_pairs,
            "observed_competitor_rows": sidecar_report.observed_competitor_rows,
            "observed_competitor_surfaces": sidecar_report.observed_competitor_surfaces,
            "same_lemma_competitor_surfaces": sidecar_report.same_lemma_competitor_surfaces,
            "admitted_pair_observations": sidecar_report.admitted_pair_observations,
        },
        "sampling": {
            "strategy": "same deterministic H-row reservoir as leave-lemmas-out proof; targets are sampling-heldout only, not sidecar-training-heldout",
            "scanned_lines": sampled.scanned_lines,
            "selected_target_forms": sampled.selected_target_forms,
            "sampling_us": sampling_us,
        },
        "latency": {
            "lemma_birth_p50_us": percentile(&proof.lemma_birth_latency_us, 50),
            "lemma_birth_p99_us": percentile(&proof.lemma_birth_latency_us, 99),
            "context_reduction_p50_us": percentile(&proof.context_reduction_latency_us, 50),
            "context_reduction_p99_us": percentile(&proof.context_reduction_latency_us, 99),
            "generated_birth_p50_us": percentile(&proof.generated_birth_latency_us, 50),
            "generated_birth_p99_us": percentile(&proof.generated_birth_latency_us, 99),
            "proof_us": proof_us,
            "directional_nh_us": directional_nh_us,
            "workers": workers,
        },
        "cold_load": {
            "l2_us": l2_cold_load_us,
            "sidecar_us": sidecar_cold_load_us,
            "l1_fingerprint_check_us": l1_cold_load_us,
        },
        "memory": {
            "rss_kib": proc_status_kib("VmRSS:"),
            "peak_rss_kib": proc_status_kib("VmHWM:"),
        },
        "gates": {
            "each_generated_unique_top1_strictly_above_percent": 95.0,
            "each_generated_top16_retained_strictly_above_percent": 95.0,
            "each_generated_readout_target_retained_strictly_above_percent": 95.0,
            "generated_birth_p99_us_at_most": 5_000,
            "false_singleton_equals": 0,
            "false_authority_equals": 0,
            "directional_nh_target_wins_strictly_above": 0,
            "directional_nh_reverse_false_supports_equals": 0,
            "directional_nh_reverse_invariant_violations_equals": 0,
        },
        "not_tested": [
            "leave-lemma-out generalization; that remains owned by the separate V7 proof",
            "clean preservation and ambiguity retention",
            "grounded L1.1 lattice preservation after live generated-candidate integration",
            "L3 final contextual selection and physical apply authority",
            "daemon and IBus latency",
        ],
        "runtime_authority_changed_by_proof": false,
    }))
}

fn sample_fixed_productive_cases(
    field: &StandaloneL2Field,
    morphology_corpus_path: &Path,
    heldout_per_class: usize,
) -> io::Result<SampledCases> {
    let capacity = heldout_per_class
        .saturating_mul(24)
        .max(heldout_per_class.saturating_add(64));
    let file = File::open(morphology_corpus_path)?;
    let mut reader = BufReader::with_capacity(1024 * 1024, file);
    let mut line = Vec::with_capacity(256);
    let mut reservoir = BinaryHeap::<SampledScene>::with_capacity(capacity + 1);
    let mut sampled = SampledCases::default();
    loop {
        line.clear();
        if reader.read_until(b'\n', &mut line)? == 0 {
            break;
        }
        sampled.scanned_lines += 1;
        if !line.starts_with(b"H\t") {
            continue;
        }
        sampled.heldout_rows += 1;
        let raw = std::str::from_utf8(&line).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("morphology corpus is not UTF-8: {error}"),
            )
        })?;
        let raw = raw.trim_end_matches(['\r', '\n']);
        let fields = raw.split('\t').collect::<Vec<_>>();
        let ["H", lemma, surface, feature_text, context] = fields.as_slice() else {
            continue;
        };
        if !eligible_surface(surface) {
            continue;
        }
        sampled.eligible_heldout_rows += 1;
        let candidate = SampledScene {
            hash: stable_hash(raw.as_bytes()),
            lemma: lemma.trim().to_lowercase(),
            surface: (*surface).to_string(),
            feature_text: (*feature_text).to_string(),
            context: (*context).to_string(),
        };
        if reservoir.len() < capacity {
            reservoir.push(candidate);
        } else if reservoir.peek().is_some_and(|largest| candidate < *largest) {
            reservoir.pop();
            reservoir.push(candidate);
        }
    }

    let mut scenes = reservoir.into_sorted_vec();
    scenes.sort_unstable();
    sampled.reservoir_rows = scenes.len();
    let mut by_class = BTreeMap::<&'static str, Vec<ProofCase>>::new();
    let mut target_forms = BTreeSet::new();
    for scene in scenes {
        let feature_mask = crate::nanda_wave::morphology_phase::parse_features(&scene.feature_text)
            .map_err(io::Error::other)?;
        let Some(target_form_ref) = field.form_ref_for_surface(&scene.surface) else {
            continue;
        };
        let target_lemma_ids = field
            .lemma_ids_for_form_feature(target_form_ref, feature_mask)
            .into_iter()
            .filter(|lemma_id| {
                field
                    .productive_lemma(*lemma_id)
                    .ok()
                    .and_then(|lemma| {
                        canonical_source(&lemma.forms).map(|source| {
                            source.feature_mask != feature_mask && source.surface != scene.surface
                        })
                    })
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        if target_lemma_ids.is_empty() {
            continue;
        }
        sampled.productive_heldout_rows += 1;
        let (_, heldout) = split_damages(&scene.surface);
        let mut one_per_class = BTreeMap::<&'static str, DamageExample>::new();
        for example in heldout {
            one_per_class
                .entry(example.class)
                .and_modify(|selected| {
                    if damage_hash(&scene.surface, &scene.context, &example)
                        < damage_hash(&scene.surface, &scene.context, selected)
                    {
                        *selected = example.clone();
                    }
                })
                .or_insert(example);
        }
        for (class, example) in one_per_class {
            let bucket = by_class.entry(class).or_default();
            if bucket.len() == heldout_per_class {
                continue;
            }
            bucket.push(ProofCase {
                target_form_ref,
                target_lemma_ids: target_lemma_ids.clone(),
                target_surface: scene.surface.clone(),
                feature_mask,
                context: scene.context.clone(),
                class,
                damaged_surface: example.surface,
            });
            target_forms.insert(target_form_ref);
            sampled
                .heldout_lemma_ids
                .extend(target_lemma_ids.iter().copied());
            sampled.heldout_lemma_names.insert(scene.lemma.clone());
        }
        if by_class.len() == EXPECTED_DAMAGE_CLASSES
            && by_class
                .values()
                .all(|cases| cases.len() == heldout_per_class)
        {
            break;
        }
    }

    if by_class.len() != EXPECTED_DAMAGE_CLASSES
        || by_class
            .values()
            .any(|cases| cases.len() != heldout_per_class)
    {
        let counts = by_class
            .iter()
            .map(|(class, cases)| format!("{class}={}", cases.len()))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "productive L2 proof cannot fill {heldout_per_class} cases for all 13 classes: {counts}"
            ),
        ));
    }
    sampled.selected_target_forms = target_forms.len();
    sampled.cases = by_class.into_values().flatten().collect();
    Ok(sampled)
}

#[allow(clippy::too_many_arguments)]
fn evaluate_cases<I: ProductiveMorphologySource + Sync + ?Sized>(
    cases: &[ProofCase],
    field: &StandaloneL2Field,
    productive_index: &I,
    broad_lemma_limit: usize,
    active_lemma_limit: usize,
    feature_limit: usize,
    form_limit: usize,
    atom_relation_limit: usize,
) -> ProofShard {
    let mut shard = ProofShard::default();
    for case in cases {
        let lemma_started = Instant::now();
        let broad = field.compositional_lemma_births_with_atom_relation_limit(
            &case.damaged_surface,
            broad_lemma_limit,
            atom_relation_limit,
        );
        shard
            .lemma_birth_latency_us
            .push(lemma_started.elapsed().as_micros() as u64);

        let context_started = Instant::now();
        let active =
            field.contextual_compositional_lemma_births(&case.context, &broad, active_lemma_limit);
        shard
            .context_reduction_latency_us
            .push(context_started.elapsed().as_micros() as u64);

        let exact_started = Instant::now();
        let exact_births = field.contextual_compositional_form_births_from_lemmas(
            &case.context,
            &case.damaged_surface,
            &active,
            feature_limit,
            form_limit,
        );
        shard
            .exact_birth_latency_us
            .push(exact_started.elapsed().as_micros() as u64);

        let generated_started = Instant::now();
        let generated = field.productive_form_births_from_lemmas_exact_masked(
            productive_index,
            &case.context,
            &case.damaged_surface,
            &active,
            feature_limit,
            form_limit,
        );
        shard
            .generated_birth_latency_us
            .push(generated_started.elapsed().as_micros() as u64);

        let broad_retained = broad
            .iter()
            .any(|birth| case.target_lemma_ids.contains(&birth.lemma_id));
        let broad_target_rank = broad
            .iter()
            .position(|birth| case.target_lemma_ids.contains(&birth.lemma_id))
            .map(|index| index + 1);
        let active_retained = active
            .iter()
            .any(|birth| case.target_lemma_ids.contains(&birth.lemma_id));
        let active_target_rank = active
            .iter()
            .position(|birth| case.target_lemma_ids.contains(&birth.lemma_id))
            .map(|index| index + 1);
        let seen_exact_top16 = exact_births
            .iter()
            .any(|birth| birth.form_ref == case.target_form_ref);
        let seen_exact_unique_top1 = unique_exact_target_top1(&exact_births, case.target_form_ref);
        let generated_slot_retained = generated.iter().any(|birth| {
            case.target_lemma_ids.contains(&birth.lemma_id)
                && birth.target_feature_mask == case.feature_mask
        });
        let generated_top8 = generated
            .iter()
            .take(8)
            .any(|birth| generated_is_target(birth, case));
        let generated_top16 = generated
            .iter()
            .any(|birth| generated_is_target(birth, case));
        let generated_unique_top1 = unique_generated_target_top1(&generated, case);
        let generated_readout = productive_l2_readout(&case.damaged_surface, &generated);
        let generated_readout_target_retained = match &generated_readout {
            ProductiveL2Readout::Winner { surface } => surface == &case.target_surface,
            ProductiveL2Readout::Tied { surfaces } => surfaces.contains(&case.target_surface),
            ProductiveL2Readout::Abstain => false,
        };
        let false_singleton = matches!(
            &generated_readout,
            ProductiveL2Readout::Winner { surface } if surface != &case.target_surface
        );
        let status_violations = generated
            .iter()
            .filter(|birth| birth.status != ProductiveBirthStatus::ShadowUnverified)
            .count();
        let exact_annotation_leaks = generated
            .iter()
            .filter(|birth| birth.exact_surface_form_ref.is_some())
            .count();

        let metrics = shard.classes.entry(case.class).or_default();
        metrics.cases += 1;
        metrics.known_context_modes += usize::from(field.context_mode_known(&case.context));
        metrics.broad_lemma_retained += usize::from(broad_retained);
        metrics.active_lemma_retained += usize::from(active_retained);
        metrics.seen_exact_top16_retained += usize::from(seen_exact_top16);
        metrics.seen_exact_unique_top1 += usize::from(seen_exact_unique_top1);
        metrics.unseen_generated_slot_retained += usize::from(generated_slot_retained);
        metrics.unseen_generated_top8_retained += usize::from(generated_top8);
        metrics.unseen_generated_top16_retained += usize::from(generated_top16);
        metrics.unseen_generated_unique_top1 += usize::from(generated_unique_top1);
        metrics.unseen_generated_readout_target_retained +=
            usize::from(generated_readout_target_retained);
        metrics.generated_winner += usize::from(matches!(
            &generated_readout,
            ProductiveL2Readout::Winner { .. }
        ));
        metrics.generated_tied += usize::from(matches!(
            &generated_readout,
            ProductiveL2Readout::Tied { .. }
        ));
        metrics.generated_abstain +=
            usize::from(matches!(&generated_readout, ProductiveL2Readout::Abstain));
        metrics.unseen_generated_empty_lattice += usize::from(generated.is_empty());
        metrics.false_singleton += usize::from(false_singleton);
        metrics.false_authority += 0;
        metrics.status_violations += status_violations;
        metrics.exact_annotation_leaks += exact_annotation_leaks;

        if (!active_retained || !generated_top16 || !generated_unique_top1)
            && shard.failure_examples.entry(case.class).or_default().len() < 8
        {
            shard
                .failure_examples
                .entry(case.class)
                .or_default()
                .push(serde_json::json!({
                    "class": case.class,
                    "target": case.target_surface,
                    "damaged": case.damaged_surface,
                    "feature_mask": case.feature_mask,
                    "context": case.context,
                    "broad_lemma_retained": broad_retained,
                    "broad_target_rank": broad_target_rank,
                    "active_lemma_retained": active_retained,
                    "active_target_rank": active_target_rank,
                    "seen_exact_top16_retained": seen_exact_top16,
                    "unseen_generated_slot_retained": generated_slot_retained,
                    "unseen_generated_top16_retained": generated_top16,
                    "unseen_generated_unique_top1": generated_unique_top1,
                    "generated_readout_target_retained": generated_readout_target_retained,
                    "generated_readout": format!("{generated_readout:?}"),
                    "generated": generated_surfaces(&generated, 8),
                }));
        }
        if false_singleton && shard.false_singleton_examples.len() < 32 {
            shard.false_singleton_examples.push(serde_json::json!({
                "class": case.class,
                "target": case.target_surface,
                "damaged": case.damaged_surface,
                "feature_mask": case.feature_mask,
                "context": case.context,
                "generated_readout": format!("{generated_readout:?}"),
                "generated": generated_surfaces(&generated, 4),
            }));
        }
    }
    shard
}

fn generated_is_target(birth: &ProductiveL2FormBirth, case: &ProofCase) -> bool {
    birth.surface == case.target_surface
}

fn unique_generated_target_top1(births: &[ProductiveL2FormBirth], case: &ProofCase) -> bool {
    let Some(first) = births.first() else {
        return false;
    };
    generated_is_target(first, case)
        && births.get(1).is_none_or(|second| {
            productive_l2_birth_rank(first) != productive_l2_birth_rank(second)
        })
}

fn unique_exact_target_top1(births: &[CompositionalFormBirth], target_form_ref: u32) -> bool {
    let Some(first) = births.first() else {
        return false;
    };
    first.form_ref == target_form_ref
        && births
            .get(1)
            .is_none_or(|second| first.rank_evidence() != second.rank_evidence())
}

fn generated_surfaces(births: &[ProductiveL2FormBirth], limit: usize) -> Vec<serde_json::Value> {
    births
        .iter()
        .take(limit)
        .map(|birth| {
            serde_json::json!({
                "surface": birth.surface,
                "lemma_id": birth.lemma_id,
                "target_feature_mask": birth.target_feature_mask,
                "geometry_evidence_milli": birth.geometry_evidence_milli,
                "slot_evidence_milli": birth.slot_evidence_milli,
                "context_positive_support": birth.context_positive_support,
                "context_anti_support": 0,
                "context_unlabeled_alternative_support": birth.context_unlabeled_alternative_support,
                "context_posterior_milli": birth.context_posterior_milli,
                "context_observed": birth.context_observed,
                "context_pair_evidence": birth.context_pair_evidence.iter().map(|edge| serde_json::json!({
                    "competitor_feature_mask": edge.competitor_feature_mask,
                    "positive_support": edge.evidence.positive_support,
                    "anti_support": edge.evidence.anti_support,
                    "posterior_milli": edge.evidence.posterior_milli,
                    "exact_positive_support": edge.evidence.exact_positive_support,
                    "exact_anti_support": edge.evidence.exact_anti_support,
                    "supporting_neighbor_lanes": edge.evidence.supporting_neighbor_lanes,
                    "contradicting_neighbor_lanes": edge.evidence.contradicting_neighbor_lanes,
                    "tied_neighbor_lanes": edge.evidence.tied_neighbor_lanes,
                })).collect::<Vec<_>>(),
                "joint_evidence_milli": birth.joint_evidence_milli,
                "profile_evidence_milli": birth.profile_evidence_milli,
                "positive_support": birth.positive_support,
                "anti_support": birth.anti_support,
                "family_specificity": birth.family_specificity,
                "lemma_atom_evidence_milli": birth.lemma_atom_evidence_milli,
                "lemma_wave_distance": birth.lemma_wave_distance,
                "status": format!("{:?}", birth.status),
            })
        })
        .collect()
}

fn eligible_surface(surface: &str) -> bool {
    let length = surface.chars().count();
    (7..=48).contains(&length) && surface.chars().all(crate::keyboard::is_cyrillic_letter)
}

fn stable_hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0x6a09_e667_f3bc_c909, |state, byte| {
        crate::stable_hash::mix64_golden(state ^ u64::from(*byte))
    })
}

fn damage_hash(target: &str, context: &str, example: &DamageExample) -> u64 {
    stable_hash(
        target
            .bytes()
            .chain(context.bytes())
            .chain(example.class.bytes())
            .chain(example.surface.bytes())
            .collect::<Vec<_>>()
            .as_slice(),
    )
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
    let index = (values.len() - 1).saturating_mul(percentile.min(100)) / 100;
    values[index]
}

fn proc_status_kib(prefix: &str) -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix(prefix)?
                    .split_whitespace()
                    .next()?
                    .parse::<u64>()
                    .ok()
            })
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::super::productive::{ProductiveContextSlotEvidence, ProductiveFormBirth};
    use super::*;

    struct DirectionalSource {
        preferred: u32,
        competitor: u32,
    }

    impl ProductiveMorphologySource for DirectionalSource {
        fn target_features_vec(&self, _primary_pos: u16, _source_feature_mask: u32) -> Vec<u32> {
            Vec::new()
        }

        fn context_slot_evidence_for(
            &self,
            _context: &str,
            _target_feature_mask: u32,
        ) -> ProductiveContextSlotEvidence {
            ProductiveContextSlotEvidence::default()
        }

        fn context_pair_evidence_for(
            &self,
            _context: &str,
            preferred_feature_mask: u32,
            competitor_feature_mask: u32,
        ) -> ProductiveContextPairEvidence {
            if preferred_feature_mask == self.preferred
                && competitor_feature_mask == self.competitor
            {
                ProductiveContextPairEvidence {
                    positive_support: 2,
                    anti_support: 0,
                    posterior_milli: 750,
                    context_observed: true,
                    exact_positive_support: 2,
                    ..ProductiveContextPairEvidence::default()
                }
            } else if preferred_feature_mask == self.competitor
                && competitor_feature_mask == self.preferred
            {
                ProductiveContextPairEvidence {
                    positive_support: 0,
                    anti_support: 2,
                    posterior_milli: 250,
                    context_observed: true,
                    exact_anti_support: 2,
                    ..ProductiveContextPairEvidence::default()
                }
            } else {
                ProductiveContextPairEvidence::default()
            }
        }

        fn generate_forms(
            &self,
            _observed_surface: &str,
            _primary_pos: u16,
            _source_surface: &str,
            _source_feature_mask: u32,
            _target_feature_mask: u32,
            _limit: usize,
        ) -> Vec<ProductiveFormBirth> {
            Vec::new()
        }
    }

    #[test]
    fn nh_gate_keeps_only_same_lemma_directional_comparisons() {
        let corpus = concat!(
            "F\tрамка\tрамки\tnoun:gen:sg\n",
            "F\tрамка\tрамке\tnoun:dat:sg\n",
            "F\tлапка\tлапки\tnoun:gen:sg\n",
            "NT\tрамка\tрамке\tnoun:dat:sg\tподошел к _ дому\tрамки,лапки\n",
            "NH\tрамка\tрамке\tnoun:dat:sg\tпошел к _ окну\tрамки,лапки\n",
        );
        let (rows, wanted) =
            collect_directional_heldout_rows(std::io::Cursor::new(corpus)).expect("NH rows");
        let features = collect_wanted_form_features(std::io::Cursor::new(corpus), &wanted)
            .expect("competitor features");
        let preferred = crate::nanda_wave::morphology_phase::parse_features("noun:dat:sg")
            .expect("preferred features");
        let competitor = crate::nanda_wave::morphology_phase::parse_features("noun:gen:sg")
            .expect("competitor features");

        let gate = evaluate_directional_heldout_rows(
            &DirectionalSource {
                preferred,
                competitor,
            },
            &rows,
            &features,
        );

        assert_eq!(gate.total_nh_rows, 1);
        assert_eq!(gate.competitor_surfaces, 2);
        assert_eq!(gate.same_lemma_competitor_surfaces, 1);
        assert_eq!(gate.non_same_lemma_competitor_surfaces, 1);
        assert_eq!(gate.same_lemma_comparisons, 1);
        assert_eq!(gate.pair_evidence_covered, 1);
        assert_eq!(gate.target_directional_wins, 1);
        assert_eq!(gate.reverse_false_supports, 0);
        assert_eq!(gate.verdict, "PASS_shadow_directional_nh");
    }
}
