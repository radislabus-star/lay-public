use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};
use std::io::{self, Read};
use std::path::Path;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::calibrate::{CandidateRankOriginV1, ProductiveCalibratedVerdictV1};
use super::corpus::load_axis_schema;
use super::events::{
    decode_verified_spool_record, TypedProductiveEventV1, VerifiedSpoolShardReaderV1,
};
use super::packaged_runtime::{
    base_surface_projection_preserved_v1, ColdBindingDerivationDiagnosticsV1, ColdLemmaBindingV1,
    ColdLemmaSourceV1, PackagedGroundedLemmaV1, PackagedProductiveCandidateV1,
    PackagedProductiveReadoutV1, PackagedProductiveRuntimeV1, ProductiveEvaluationTelemetryV1,
    ProductiveTargetProbeIdentityV1, ProductiveTargetProbeV1, RecoveryIdentityAnchorRefV1,
    SharedHypothesisReplayAuditV1,
};
use super::types::CanonicalL2BindingIdentityV1;
use crate::nanda_wave::lexical_grokking::{split_damages, DamageExample};

const EXPECTED_DAMAGE_CLASSES: usize = 13;
const TARGET_PROBE_PARITY_ERROR: &str = "productive target probe changed runtime readout";
const FROZEN_MANIFEST_SCHEMA_VERSION: u16 = 1;
const FROZEN_MANIFEST_HELDOUT_PER_CLASS: usize = 100;
const FROZEN_MANIFEST_ENTRY_COUNT: usize = 1_300;
const FROZEN_MANIFEST_H_COUNT: usize = 1_280;
const FROZEN_V64_PACKAGE_SHA256: &str =
    "9fd8c950398fb8ba47a2c9f2236880239d9f4376b191a691b0d01c47ddd3e438";
const FROZEN_PROOF_SPOOL_SHA256: &str =
    "6e282474b26bf90dc61ee21c93c9dd7dd727c29a2b02650c513ffdd06746e807";
const FROZEN_PROOF_SPOOL_BYTES: u64 = 1_154_794_811;
const FROZEN_DAMAGE_GENERATOR_ID: &str = "lexical_grokking::split_damages:v1";
const FROZEN_MANIFEST_FILE: &str = "frozen-v64-hypothesis-manifest-v1.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FrozenHypothesisManifestV1 {
    schema_version: u16,
    v64_package_sha256: String,
    proof_spool_sha256: String,
    proof_spool_bytes: u64,
    l11_package_sha256: String,
    canonical_l2_package_sha256: String,
    axis_schema_sha256: String,
    heldout_per_class: usize,
    cohorts: Vec<String>,
    damage_generator_id: String,
    entry_count: usize,
    h_count: usize,
    payload_sha256: String,
    entries: Vec<FrozenHypothesisEntryV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct FrozenHypothesisEntryV1 {
    proof_identity: [u8; 32],
    damage_class: String,
    damage_identity: [u8; 32],
    target_lemma_id: u32,
    target_pos_domain: u16,
    oracle_paradigm_ids: Vec<u32>,
}

#[derive(Clone, Debug)]
struct FrozenHypothesisIndexV1 {
    path: std::path::PathBuf,
    payload_sha256: String,
    entries: BTreeMap<([u8; 32], [u8; 32]), FrozenHypothesisEntryV1>,
    h_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ProofCohortV1 {
    SeenExact,
    LemmaHeldout,
}

impl ProofCohortV1 {
    const fn name(self) -> &'static str {
        match self {
            Self::SeenExact => "SEEN_EXACT",
            Self::LemmaHeldout => "LEMMA_HELDOUT",
        }
    }
}

#[derive(Clone, Debug)]
struct SampledProofEventV1 {
    key: [u8; 32],
    event: super::events::ProofEventV1,
}

impl PartialEq for SampledProofEventV1 {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.event.proof_identity == other.event.proof_identity
    }
}

impl Eq for SampledProofEventV1 {}

impl PartialOrd for SampledProofEventV1 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SampledProofEventV1 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key
            .cmp(&other.key)
            .then_with(|| self.event.proof_identity.cmp(&other.event.proof_identity))
    }
}

#[derive(Clone, Debug)]
struct ProofCaseV1 {
    cohort: ProofCohortV1,
    class: &'static str,
    damaged_surface: String,
    event: super::events::ProofEventV1,
}

#[derive(Clone, Debug)]
pub(super) struct FixedSharedReplayAuditCaseV1 {
    pub(super) class: &'static str,
    pub(super) proof_identity: [u8; 32],
    pub(super) audits: Vec<SharedHypothesisReplayAuditV1>,
    pub(super) binding_parity_exact: bool,
    pub(super) legacy_grounding_us: u64,
    pub(super) semantic_grounding_us: u64,
    pub(super) legacy_binding_count: usize,
    pub(super) semantic_binding_count: usize,
}

#[derive(Clone, Debug)]
pub(super) struct FixedSharedReplayAuditCorpusV1 {
    pub(super) cases: Vec<FixedSharedReplayAuditCaseV1>,
    pub(super) scanned_proof_events: usize,
    pub(super) sampled_events_by_cohort: BTreeMap<&'static str, usize>,
    pub(super) workers: usize,
    pub(super) cold_load_us: u64,
    pub(super) sampling_us: u64,
    pub(super) preparation_us: u64,
    pub(super) binding_parity_comparisons: usize,
    pub(super) binding_parity_failures: usize,
    pub(super) legacy_grounding_us: u64,
    pub(super) semantic_grounding_us: u64,
}

#[derive(Clone, Debug, Default, Serialize)]
struct ClassMetricsV1 {
    cases: usize,
    oracle_applicable_cases: usize,
    hypothesis_covered: usize,
    compatible_binding_retained: usize,
    target_slot_in_binding: usize,
    target_exact_pre_slot_bound: usize,
    target_exact_post_slot_bound: usize,
    target_exact_post_surface_basin_bound: usize,
    target_lemma_born: usize,
    target_slot_born: usize,
    target_exact_born: usize,
    target_top1: usize,
    base_target_top1: usize,
    target_top16: usize,
    readout_target_retained: usize,
    clean_target_retained: usize,
    winner: usize,
    tied: usize,
    abstain: usize,
    empty_lattice: usize,
    shadow_false_singleton: usize,
    integrity_errors: usize,
    base_projection_comparisons: usize,
    base_projection_failures: usize,
    demotions_without_certificate: usize,
    hypothesis_covered_percent: f64,
    compatible_binding_retained_percent: f64,
    target_slot_in_binding_percent: f64,
    target_exact_pre_slot_bound_percent: f64,
    target_exact_post_slot_bound_percent: f64,
    target_exact_post_surface_basin_bound_percent: f64,
    target_lemma_born_percent: f64,
    target_slot_born_percent: f64,
    target_exact_born_percent: f64,
    target_top1_percent: f64,
    target_top16_percent: f64,
    readout_target_retained_percent: f64,
    clean_target_retained_percent: f64,
    latency_p50_us: u64,
    latency_p95_us: u64,
    latency_p99_us: u64,
    latency_max_us: u64,
    #[serde(skip)]
    latency_us: Vec<u64>,
}

impl ClassMetricsV1 {
    fn merge(&mut self, mut other: Self) {
        self.cases += other.cases;
        self.oracle_applicable_cases += other.oracle_applicable_cases;
        self.hypothesis_covered += other.hypothesis_covered;
        self.compatible_binding_retained += other.compatible_binding_retained;
        self.target_slot_in_binding += other.target_slot_in_binding;
        self.target_exact_pre_slot_bound += other.target_exact_pre_slot_bound;
        self.target_exact_post_slot_bound += other.target_exact_post_slot_bound;
        self.target_exact_post_surface_basin_bound += other.target_exact_post_surface_basin_bound;
        self.target_lemma_born += other.target_lemma_born;
        self.target_slot_born += other.target_slot_born;
        self.target_exact_born += other.target_exact_born;
        self.target_top1 += other.target_top1;
        self.base_target_top1 += other.base_target_top1;
        self.target_top16 += other.target_top16;
        self.readout_target_retained += other.readout_target_retained;
        self.clean_target_retained += other.clean_target_retained;
        self.winner += other.winner;
        self.tied += other.tied;
        self.abstain += other.abstain;
        self.empty_lattice += other.empty_lattice;
        self.shadow_false_singleton += other.shadow_false_singleton;
        self.integrity_errors += other.integrity_errors;
        self.base_projection_comparisons += other.base_projection_comparisons;
        self.base_projection_failures += other.base_projection_failures;
        self.demotions_without_certificate += other.demotions_without_certificate;
        self.latency_us.append(&mut other.latency_us);
    }

    fn finish(&mut self) {
        self.hypothesis_covered_percent =
            percent(self.hypothesis_covered, self.oracle_applicable_cases);
        self.compatible_binding_retained_percent = percent(
            self.compatible_binding_retained,
            self.oracle_applicable_cases,
        );
        self.target_slot_in_binding_percent =
            percent(self.target_slot_in_binding, self.oracle_applicable_cases);
        self.target_exact_pre_slot_bound_percent = percent(
            self.target_exact_pre_slot_bound,
            self.oracle_applicable_cases,
        );
        self.target_exact_post_slot_bound_percent = percent(
            self.target_exact_post_slot_bound,
            self.oracle_applicable_cases,
        );
        self.target_exact_post_surface_basin_bound_percent = percent(
            self.target_exact_post_surface_basin_bound,
            self.oracle_applicable_cases,
        );
        self.target_lemma_born_percent = percent(self.target_lemma_born, self.cases);
        self.target_slot_born_percent = percent(self.target_slot_born, self.cases);
        self.target_exact_born_percent = percent(self.target_exact_born, self.cases);
        self.target_top1_percent = percent(self.target_top1, self.cases);
        self.target_top16_percent = percent(self.target_top16, self.cases);
        self.readout_target_retained_percent = percent(self.readout_target_retained, self.cases);
        self.clean_target_retained_percent = percent(self.clean_target_retained, self.cases);
        self.latency_us.sort_unstable();
        self.latency_p50_us = percentile(&self.latency_us, 50);
        self.latency_p95_us = percentile(&self.latency_us, 95);
        self.latency_p99_us = percentile(&self.latency_us, 99);
        self.latency_max_us = self.latency_us.last().copied().unwrap_or_default();
    }
}

#[derive(Default)]
struct ProofShardV1 {
    classes: BTreeMap<(ProofCohortV1, &'static str), ClassMetricsV1>,
    failure_examples: Vec<serde_json::Value>,
    first_loss_diagnostics: FirstLossDiagnosticsV1,
    probe_parity_comparisons: usize,
    probe_parity_failures: usize,
    bounded_recovery: BoundedRecoveryTotalsV1,
    stage_telemetry: ProductiveStageTelemetryTotalsV1,
    slow_calls: BTreeMap<(ProofCohortV1, &'static str), Vec<ProductiveSlowCallV1>>,
}

struct WorkerProofPartialV1 {
    worker_id: usize,
    case_count: usize,
    elapsed_us: u64,
    shard: ProofShardV1,
}

impl ProofShardV1 {
    fn merge(&mut self, other: Self) {
        self.probe_parity_comparisons += other.probe_parity_comparisons;
        self.probe_parity_failures += other.probe_parity_failures;
        self.first_loss_diagnostics
            .merge(other.first_loss_diagnostics);
        self.bounded_recovery.merge(other.bounded_recovery);
        self.stage_telemetry.merge(other.stage_telemetry);
        for (key, calls) in other.slow_calls {
            for call in calls {
                record_slow_call(self.slow_calls.entry(key).or_default(), call);
            }
        }
        for (key, metrics) in other.classes {
            self.classes.entry(key).or_default().merge(metrics);
        }
        for example in other.failure_examples {
            if self.failure_examples.len() < 64 {
                self.failure_examples.push(example);
            }
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct ProductiveSlowCallV1 {
    proof_identity: [u8; 32],
    damaged_surface: String,
    target_surface: String,
    elapsed_us: u64,
    stages: ProductiveEvaluationTelemetryV1,
}

fn record_slow_call(calls: &mut Vec<ProductiveSlowCallV1>, call: ProductiveSlowCallV1) {
    calls.push(call);
    calls.sort_unstable_by(|left, right| {
        right
            .elapsed_us
            .cmp(&left.elapsed_us)
            .then_with(|| left.proof_identity.cmp(&right.proof_identity))
    });
    calls.truncate(3);
}

#[derive(Default, Serialize)]
struct ProductiveStageTelemetryTotalsV1 {
    profiled_calls: u64,
    setup_us: u64,
    binding_preparation_us: u64,
    traversal_us: u64,
    surface_reduce_us: u64,
    final_readout_us: u64,
    maximum_setup_us: u64,
    maximum_binding_preparation_us: u64,
    maximum_traversal_us: u64,
    maximum_surface_reduce_us: u64,
    maximum_final_readout_us: u64,
    active_binding_count: u64,
    logical_terminal_count: u64,
    logical_surface_basin_count: u64,
    selected_candidate_count: u64,
    #[serde(skip)]
    setup_samples_us: Vec<u64>,
    #[serde(skip)]
    binding_preparation_samples_us: Vec<u64>,
    #[serde(skip)]
    traversal_samples_us: Vec<u64>,
    #[serde(skip)]
    surface_reduce_samples_us: Vec<u64>,
    #[serde(skip)]
    final_readout_samples_us: Vec<u64>,
}

impl ProductiveStageTelemetryTotalsV1 {
    fn record(&mut self, telemetry: ProductiveEvaluationTelemetryV1) {
        self.profiled_calls += 1;
        self.setup_us += telemetry.setup_us;
        self.binding_preparation_us += telemetry.binding_preparation_us;
        self.traversal_us += telemetry.traversal_us;
        self.surface_reduce_us += telemetry.surface_reduce_us;
        self.final_readout_us += telemetry.final_readout_us;
        self.maximum_setup_us = self.maximum_setup_us.max(telemetry.setup_us);
        self.maximum_binding_preparation_us = self
            .maximum_binding_preparation_us
            .max(telemetry.binding_preparation_us);
        self.maximum_traversal_us = self.maximum_traversal_us.max(telemetry.traversal_us);
        self.maximum_surface_reduce_us = self
            .maximum_surface_reduce_us
            .max(telemetry.surface_reduce_us);
        self.maximum_final_readout_us = self
            .maximum_final_readout_us
            .max(telemetry.final_readout_us);
        self.active_binding_count += telemetry.active_binding_count;
        self.logical_terminal_count += telemetry.logical_terminal_count;
        self.logical_surface_basin_count += telemetry.logical_surface_basin_count;
        self.selected_candidate_count += telemetry.selected_candidate_count;
        self.setup_samples_us.push(telemetry.setup_us);
        self.binding_preparation_samples_us
            .push(telemetry.binding_preparation_us);
        self.traversal_samples_us.push(telemetry.traversal_us);
        self.surface_reduce_samples_us
            .push(telemetry.surface_reduce_us);
        self.final_readout_samples_us
            .push(telemetry.final_readout_us);
    }

    fn merge(&mut self, other: Self) {
        self.profiled_calls += other.profiled_calls;
        self.setup_us += other.setup_us;
        self.binding_preparation_us += other.binding_preparation_us;
        self.traversal_us += other.traversal_us;
        self.surface_reduce_us += other.surface_reduce_us;
        self.final_readout_us += other.final_readout_us;
        self.maximum_setup_us = self.maximum_setup_us.max(other.maximum_setup_us);
        self.maximum_binding_preparation_us = self
            .maximum_binding_preparation_us
            .max(other.maximum_binding_preparation_us);
        self.maximum_traversal_us = self.maximum_traversal_us.max(other.maximum_traversal_us);
        self.maximum_surface_reduce_us = self
            .maximum_surface_reduce_us
            .max(other.maximum_surface_reduce_us);
        self.maximum_final_readout_us = self
            .maximum_final_readout_us
            .max(other.maximum_final_readout_us);
        self.active_binding_count += other.active_binding_count;
        self.logical_terminal_count += other.logical_terminal_count;
        self.logical_surface_basin_count += other.logical_surface_basin_count;
        self.selected_candidate_count += other.selected_candidate_count;
        self.setup_samples_us.extend(other.setup_samples_us);
        self.binding_preparation_samples_us
            .extend(other.binding_preparation_samples_us);
        self.traversal_samples_us.extend(other.traversal_samples_us);
        self.surface_reduce_samples_us
            .extend(other.surface_reduce_samples_us);
        self.final_readout_samples_us
            .extend(other.final_readout_samples_us);
    }

    fn percentiles(&mut self) -> serde_json::Value {
        let summarize = |samples: &mut Vec<u64>| {
            samples.sort_unstable();
            serde_json::json!({
                "p50_us": percentile(samples, 50),
                "p95_us": percentile(samples, 95),
                "p99_us": percentile(samples, 99),
                "maximum_us": samples.last().copied().unwrap_or_default(),
            })
        };
        serde_json::json!({
            "setup": summarize(&mut self.setup_samples_us),
            "binding_preparation": summarize(&mut self.binding_preparation_samples_us),
            "traversal": summarize(&mut self.traversal_samples_us),
            "surface_reduce": summarize(&mut self.surface_reduce_samples_us),
            "final_readout": summarize(&mut self.final_readout_samples_us),
        })
    }
}

#[derive(Default, Serialize)]
struct BoundedRecoveryTotalsV1 {
    cases: u64,
    source_count: u64,
    observed_slot_count: u64,
    posting_lookup_count: u64,
    posting_miss_count: u64,
    structural_eligible_paradigm_count: u64,
    recovery_lookup_count: u64,
    recovery_path_count: u64,
    recovery_post_intersection_count: u64,
    recovery_program_execution_count: u64,
    recovered_anchor_count: u64,
    recovery_unique_anchor_count: u64,
    recovery_post_frontier_anchor_count: u64,
    recovery_frontier_dropped_count: u64,
    recovery_max_independent_source_count: u64,
    identity_bridge_candidate_count: u64,
    exact_replay_program_execution_count: u64,
    shared_hypothesis_observation_count: u64,
    shared_hypothesis_unique_count: u64,
    shared_hypothesis_join_attempt_count: u64,
    shared_hypothesis_exact_count: u64,
    shared_hypothesis_replay_execution_count: u64,
    transition_equivalence_class_count: u64,
    transition_equivalence_owner_count: u64,
    transition_equivalence_max_class_size: u64,
    transition_equivalence_representative_replay_count: u64,
    transition_equivalence_exact_class_count: u64,
    transition_equivalence_exact_owner_fanout_count: u64,
    recovery_exact_reconstructing_count: u64,
    retained_binding_count: u64,
}

impl BoundedRecoveryTotalsV1 {
    fn record(&mut self, diagnostics: &ColdBindingDerivationDiagnosticsV1) {
        self.cases += 1;
        self.source_count += diagnostics.source_count as u64;
        self.observed_slot_count += diagnostics.observed_slot_count as u64;
        self.posting_lookup_count += diagnostics.posting_lookup_count as u64;
        self.posting_miss_count += diagnostics.posting_miss_count as u64;
        self.structural_eligible_paradigm_count +=
            diagnostics.structural_eligible_paradigm_count as u64;
        self.recovery_lookup_count += diagnostics.recovery_lookup_count as u64;
        self.recovery_path_count += diagnostics.recovery_path_count as u64;
        self.recovery_post_intersection_count +=
            diagnostics.recovery_post_intersection_count as u64;
        self.recovery_program_execution_count +=
            diagnostics.recovery_program_execution_count as u64;
        self.recovered_anchor_count += diagnostics.recovered_anchor_count as u64;
        self.recovery_unique_anchor_count += diagnostics.recovery_unique_anchor_count as u64;
        self.recovery_post_frontier_anchor_count +=
            diagnostics.recovery_post_frontier_anchor_count as u64;
        self.recovery_frontier_dropped_count += diagnostics.recovery_frontier_dropped_count as u64;
        self.recovery_max_independent_source_count = self
            .recovery_max_independent_source_count
            .max(diagnostics.recovery_max_independent_source_count as u64);
        self.identity_bridge_candidate_count += diagnostics.identity_bridge_candidate_count as u64;
        self.exact_replay_program_execution_count +=
            diagnostics.exact_replay_program_execution_count as u64;
        self.shared_hypothesis_observation_count +=
            diagnostics.shared_hypothesis_observation_count as u64;
        self.shared_hypothesis_unique_count += diagnostics.shared_hypothesis_unique_count as u64;
        self.shared_hypothesis_join_attempt_count +=
            diagnostics.shared_hypothesis_join_attempt_count as u64;
        self.shared_hypothesis_exact_count += diagnostics.shared_hypothesis_exact_count as u64;
        self.shared_hypothesis_replay_execution_count +=
            diagnostics.shared_hypothesis_replay_execution_count as u64;
        self.transition_equivalence_class_count +=
            diagnostics.transition_equivalence_class_count as u64;
        self.transition_equivalence_owner_count +=
            diagnostics.transition_equivalence_owner_count as u64;
        self.transition_equivalence_max_class_size = self
            .transition_equivalence_max_class_size
            .max(diagnostics.transition_equivalence_max_class_size as u64);
        self.transition_equivalence_representative_replay_count +=
            diagnostics.transition_equivalence_representative_replay_count as u64;
        self.transition_equivalence_exact_class_count +=
            diagnostics.transition_equivalence_exact_class_count as u64;
        self.transition_equivalence_exact_owner_fanout_count +=
            diagnostics.transition_equivalence_exact_owner_fanout_count as u64;
        self.recovery_exact_reconstructing_count +=
            diagnostics.recovery_exact_reconstructing_count as u64;
        self.retained_binding_count += diagnostics.retained_binding_count as u64;
    }

    fn merge(&mut self, other: Self) {
        self.cases += other.cases;
        self.source_count += other.source_count;
        self.observed_slot_count += other.observed_slot_count;
        self.posting_lookup_count += other.posting_lookup_count;
        self.posting_miss_count += other.posting_miss_count;
        self.structural_eligible_paradigm_count += other.structural_eligible_paradigm_count;
        self.recovery_lookup_count += other.recovery_lookup_count;
        self.recovery_path_count += other.recovery_path_count;
        self.recovery_post_intersection_count += other.recovery_post_intersection_count;
        self.recovery_program_execution_count += other.recovery_program_execution_count;
        self.recovered_anchor_count += other.recovered_anchor_count;
        self.recovery_unique_anchor_count += other.recovery_unique_anchor_count;
        self.recovery_post_frontier_anchor_count += other.recovery_post_frontier_anchor_count;
        self.recovery_frontier_dropped_count += other.recovery_frontier_dropped_count;
        self.recovery_max_independent_source_count = self
            .recovery_max_independent_source_count
            .max(other.recovery_max_independent_source_count);
        self.identity_bridge_candidate_count += other.identity_bridge_candidate_count;
        self.exact_replay_program_execution_count += other.exact_replay_program_execution_count;
        self.shared_hypothesis_observation_count += other.shared_hypothesis_observation_count;
        self.shared_hypothesis_unique_count += other.shared_hypothesis_unique_count;
        self.shared_hypothesis_join_attempt_count += other.shared_hypothesis_join_attempt_count;
        self.shared_hypothesis_exact_count += other.shared_hypothesis_exact_count;
        self.shared_hypothesis_replay_execution_count +=
            other.shared_hypothesis_replay_execution_count;
        self.transition_equivalence_class_count += other.transition_equivalence_class_count;
        self.transition_equivalence_owner_count += other.transition_equivalence_owner_count;
        self.transition_equivalence_max_class_size = self
            .transition_equivalence_max_class_size
            .max(other.transition_equivalence_max_class_size);
        self.transition_equivalence_representative_replay_count +=
            other.transition_equivalence_representative_replay_count;
        self.transition_equivalence_exact_class_count +=
            other.transition_equivalence_exact_class_count;
        self.transition_equivalence_exact_owner_fanout_count +=
            other.transition_equivalence_exact_owner_fanout_count;
        self.recovery_exact_reconstructing_count += other.recovery_exact_reconstructing_count;
        self.retained_binding_count += other.retained_binding_count;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ValidTargetV1 {
    lemma_id: u32,
    pos_domain: u16,
    slot_id: u32,
    surface: String,
}

#[derive(Default)]
struct ProductiveGroundingsV1 {
    grounded: Vec<PackagedGroundedLemmaV1>,
    cold: Vec<ColdLemmaBindingV1>,
    base_cold: Vec<ColdLemmaBindingV1>,
    oracle_cold: Vec<ColdLemmaBindingV1>,
    target_cold_diagnostics: Option<TargetColdGroundingDiagnosticsV1>,
}

#[derive(Clone, Debug, Default)]
struct TargetColdGroundingDiagnosticsV1 {
    observation_found: bool,
    target_form_observed: bool,
    target_slot_observed: bool,
    target_pos_observed_after_mask: bool,
    oracle_observed_principal_parts: usize,
    target_blind_observed_principal_parts: usize,
    oracle: ColdBindingDerivationDiagnosticsV1,
    target_blind: ColdBindingDerivationDiagnosticsV1,
    oracle_paradigm_count: usize,
    target_blind_paradigm_count: usize,
    oracle_intersection_count: usize,
    tracked_paradigm_count: usize,
    tracked_pos_without_source_count: usize,
    tracked_in_postings_count: usize,
    tracked_slot_compatible_count: usize,
    tracked_exact_reconstructing_count: usize,
    tracked_in_recovery_postings_count: usize,
    tracked_recovered_anchor_count: usize,
    tracked_recovery_post_frontier_count: usize,
    tracked_recovery_exact_reconstructing_count: usize,
    tracked_identity_anchor_pre_frontier_count: usize,
    tracked_identity_anchor_exact_pre_frontier_count: usize,
    tracked_identity_anchor_post_frontier_count: usize,
    tracked_identity_anchor_exact_count: usize,
    oracle_target_slot_paradigm_count: usize,
    intersection_target_slot_paradigm_count: usize,
}

#[derive(Default, Serialize)]
struct FirstLossDiagnosticsV1 {
    outside_h: LossStageDiagnosticsV1,
    h_to_b: LossStageDiagnosticsV1,
    b_to_s0: LossStageDiagnosticsV1,
}

impl FirstLossDiagnosticsV1 {
    fn merge(&mut self, other: Self) {
        self.outside_h.merge(other.outside_h);
        self.h_to_b.merge(other.h_to_b);
        self.b_to_s0.merge(other.b_to_s0);
    }

    fn record(
        &mut self,
        proof_identity: [u8; 32],
        target_lemma_id: u32,
        hypothesis_covered: bool,
        compatible_binding_retained: bool,
        target_slot_in_binding: bool,
        diagnostics: &TargetColdGroundingDiagnosticsV1,
    ) {
        let (stage, mechanism) = if !hypothesis_covered {
            (&mut self.outside_h, outside_h_mechanism(diagnostics))
        } else if !compatible_binding_retained {
            (&mut self.h_to_b, h_to_b_mechanism(diagnostics))
        } else if !target_slot_in_binding {
            (&mut self.b_to_s0, b_to_s0_mechanism(diagnostics))
        } else {
            return;
        };
        stage.record(proof_identity, target_lemma_id, mechanism, diagnostics);
    }

    fn finish(&mut self) {
        self.outside_h.finish();
        self.h_to_b.finish();
        self.b_to_s0.finish();
    }
}

#[derive(Default, Serialize)]
struct LossStageDiagnosticsV1 {
    cases: usize,
    unique_proof_events: usize,
    unique_target_lemmas: usize,
    mechanisms: BTreeMap<&'static str, usize>,
    target_form_observed_cases: usize,
    target_slot_observed_cases: usize,
    target_pos_observed_after_mask_cases: usize,
    oracle_observed_principal_parts: BTreeMap<usize, usize>,
    target_blind_observed_principal_parts: BTreeMap<usize, usize>,
    oracle_observed_slots: BTreeMap<usize, usize>,
    target_blind_observed_slots: BTreeMap<usize, usize>,
    oracle_posting_paradigms: BTreeMap<usize, usize>,
    target_blind_posting_paradigms: BTreeMap<usize, usize>,
    oracle_slot_compatible_paradigms: BTreeMap<usize, usize>,
    target_blind_slot_compatible_paradigms: BTreeMap<usize, usize>,
    oracle_exact_reconstructing_paradigms: BTreeMap<usize, usize>,
    target_blind_exact_reconstructing_paradigms: BTreeMap<usize, usize>,
    oracle_paradigms: BTreeMap<usize, usize>,
    target_blind_paradigms: BTreeMap<usize, usize>,
    oracle_intersection: BTreeMap<usize, usize>,
    tracked_pos_without_source: BTreeMap<usize, usize>,
    tracked_in_postings: BTreeMap<usize, usize>,
    tracked_slot_compatible: BTreeMap<usize, usize>,
    tracked_exact_reconstructing: BTreeMap<usize, usize>,
    target_blind_recovery_paths: BTreeMap<usize, usize>,
    target_blind_recovered_anchors: BTreeMap<usize, usize>,
    target_blind_recovery_exact_reconstructions: BTreeMap<usize, usize>,
    tracked_in_recovery_postings: BTreeMap<usize, usize>,
    tracked_recovered_anchors: BTreeMap<usize, usize>,
    tracked_recovery_post_frontier: BTreeMap<usize, usize>,
    tracked_recovery_exact_reconstructions: BTreeMap<usize, usize>,
    tracked_identity_anchors_pre_frontier: BTreeMap<usize, usize>,
    tracked_identity_anchors_exact_pre_frontier: BTreeMap<usize, usize>,
    tracked_identity_anchors_post_frontier: BTreeMap<usize, usize>,
    tracked_identity_anchors_exact: BTreeMap<usize, usize>,
    oracle_target_slot_paradigms: BTreeMap<usize, usize>,
    intersection_target_slot_paradigms: BTreeMap<usize, usize>,
    #[serde(skip)]
    proof_identities: BTreeSet<[u8; 32]>,
    #[serde(skip)]
    target_lemma_ids: BTreeSet<u32>,
}

impl LossStageDiagnosticsV1 {
    fn record(
        &mut self,
        proof_identity: [u8; 32],
        target_lemma_id: u32,
        mechanism: &'static str,
        diagnostics: &TargetColdGroundingDiagnosticsV1,
    ) {
        self.cases += 1;
        self.proof_identities.insert(proof_identity);
        self.target_lemma_ids.insert(target_lemma_id);
        *self.mechanisms.entry(mechanism).or_default() += 1;
        self.target_form_observed_cases += usize::from(diagnostics.target_form_observed);
        self.target_slot_observed_cases += usize::from(diagnostics.target_slot_observed);
        self.target_pos_observed_after_mask_cases +=
            usize::from(diagnostics.target_pos_observed_after_mask);
        record_histogram(
            &mut self.oracle_observed_principal_parts,
            diagnostics.oracle_observed_principal_parts,
        );
        record_histogram(
            &mut self.target_blind_observed_principal_parts,
            diagnostics.target_blind_observed_principal_parts,
        );
        record_histogram(
            &mut self.oracle_observed_slots,
            diagnostics.oracle.observed_slot_count,
        );
        record_histogram(
            &mut self.target_blind_observed_slots,
            diagnostics.target_blind.observed_slot_count,
        );
        record_histogram(
            &mut self.oracle_posting_paradigms,
            diagnostics.oracle.posting_paradigm_count,
        );
        record_histogram(
            &mut self.target_blind_posting_paradigms,
            diagnostics.target_blind.posting_paradigm_count,
        );
        record_histogram(
            &mut self.oracle_slot_compatible_paradigms,
            diagnostics.oracle.slot_compatible_paradigm_count,
        );
        record_histogram(
            &mut self.target_blind_slot_compatible_paradigms,
            diagnostics.target_blind.slot_compatible_paradigm_count,
        );
        record_histogram(
            &mut self.oracle_exact_reconstructing_paradigms,
            diagnostics.oracle.exact_reconstructing_paradigm_count,
        );
        record_histogram(
            &mut self.target_blind_exact_reconstructing_paradigms,
            diagnostics.target_blind.exact_reconstructing_paradigm_count,
        );
        record_histogram(
            &mut self.oracle_paradigms,
            diagnostics.oracle_paradigm_count,
        );
        record_histogram(
            &mut self.target_blind_paradigms,
            diagnostics.target_blind_paradigm_count,
        );
        record_histogram(
            &mut self.oracle_intersection,
            diagnostics.oracle_intersection_count,
        );
        record_histogram(
            &mut self.tracked_pos_without_source,
            diagnostics.tracked_pos_without_source_count,
        );
        record_histogram(
            &mut self.tracked_in_postings,
            diagnostics.tracked_in_postings_count,
        );
        record_histogram(
            &mut self.tracked_slot_compatible,
            diagnostics.tracked_slot_compatible_count,
        );
        record_histogram(
            &mut self.tracked_exact_reconstructing,
            diagnostics.tracked_exact_reconstructing_count,
        );
        record_histogram(
            &mut self.target_blind_recovery_paths,
            diagnostics.target_blind.recovery_path_count,
        );
        record_histogram(
            &mut self.target_blind_recovered_anchors,
            diagnostics.target_blind.recovered_anchor_count,
        );
        record_histogram(
            &mut self.target_blind_recovery_exact_reconstructions,
            diagnostics.target_blind.recovery_exact_reconstructing_count,
        );
        record_histogram(
            &mut self.tracked_in_recovery_postings,
            diagnostics.tracked_in_recovery_postings_count,
        );
        record_histogram(
            &mut self.tracked_recovered_anchors,
            diagnostics.tracked_recovered_anchor_count,
        );
        record_histogram(
            &mut self.tracked_recovery_post_frontier,
            diagnostics.tracked_recovery_post_frontier_count,
        );
        record_histogram(
            &mut self.tracked_recovery_exact_reconstructions,
            diagnostics.tracked_recovery_exact_reconstructing_count,
        );
        record_histogram(
            &mut self.tracked_identity_anchors_pre_frontier,
            diagnostics.tracked_identity_anchor_pre_frontier_count,
        );
        record_histogram(
            &mut self.tracked_identity_anchors_exact_pre_frontier,
            diagnostics.tracked_identity_anchor_exact_pre_frontier_count,
        );
        record_histogram(
            &mut self.tracked_identity_anchors_post_frontier,
            diagnostics.tracked_identity_anchor_post_frontier_count,
        );
        record_histogram(
            &mut self.tracked_identity_anchors_exact,
            diagnostics.tracked_identity_anchor_exact_count,
        );
        record_histogram(
            &mut self.oracle_target_slot_paradigms,
            diagnostics.oracle_target_slot_paradigm_count,
        );
        record_histogram(
            &mut self.intersection_target_slot_paradigms,
            diagnostics.intersection_target_slot_paradigm_count,
        );
    }

    fn merge(&mut self, other: Self) {
        self.cases += other.cases;
        self.target_form_observed_cases += other.target_form_observed_cases;
        self.target_slot_observed_cases += other.target_slot_observed_cases;
        self.target_pos_observed_after_mask_cases += other.target_pos_observed_after_mask_cases;
        self.proof_identities.extend(other.proof_identities);
        self.target_lemma_ids.extend(other.target_lemma_ids);
        merge_histogram(&mut self.mechanisms, other.mechanisms);
        merge_histogram(
            &mut self.oracle_observed_principal_parts,
            other.oracle_observed_principal_parts,
        );
        merge_histogram(
            &mut self.target_blind_observed_principal_parts,
            other.target_blind_observed_principal_parts,
        );
        merge_histogram(&mut self.oracle_observed_slots, other.oracle_observed_slots);
        merge_histogram(
            &mut self.target_blind_observed_slots,
            other.target_blind_observed_slots,
        );
        merge_histogram(
            &mut self.oracle_posting_paradigms,
            other.oracle_posting_paradigms,
        );
        merge_histogram(
            &mut self.target_blind_posting_paradigms,
            other.target_blind_posting_paradigms,
        );
        merge_histogram(
            &mut self.oracle_slot_compatible_paradigms,
            other.oracle_slot_compatible_paradigms,
        );
        merge_histogram(
            &mut self.target_blind_slot_compatible_paradigms,
            other.target_blind_slot_compatible_paradigms,
        );
        merge_histogram(
            &mut self.oracle_exact_reconstructing_paradigms,
            other.oracle_exact_reconstructing_paradigms,
        );
        merge_histogram(
            &mut self.target_blind_exact_reconstructing_paradigms,
            other.target_blind_exact_reconstructing_paradigms,
        );
        merge_histogram(&mut self.oracle_paradigms, other.oracle_paradigms);
        merge_histogram(
            &mut self.target_blind_paradigms,
            other.target_blind_paradigms,
        );
        merge_histogram(&mut self.oracle_intersection, other.oracle_intersection);
        merge_histogram(
            &mut self.tracked_pos_without_source,
            other.tracked_pos_without_source,
        );
        merge_histogram(&mut self.tracked_in_postings, other.tracked_in_postings);
        merge_histogram(
            &mut self.tracked_slot_compatible,
            other.tracked_slot_compatible,
        );
        merge_histogram(
            &mut self.tracked_exact_reconstructing,
            other.tracked_exact_reconstructing,
        );
        merge_histogram(
            &mut self.target_blind_recovery_paths,
            other.target_blind_recovery_paths,
        );
        merge_histogram(
            &mut self.target_blind_recovered_anchors,
            other.target_blind_recovered_anchors,
        );
        merge_histogram(
            &mut self.target_blind_recovery_exact_reconstructions,
            other.target_blind_recovery_exact_reconstructions,
        );
        merge_histogram(
            &mut self.tracked_in_recovery_postings,
            other.tracked_in_recovery_postings,
        );
        merge_histogram(
            &mut self.tracked_recovered_anchors,
            other.tracked_recovered_anchors,
        );
        merge_histogram(
            &mut self.tracked_recovery_post_frontier,
            other.tracked_recovery_post_frontier,
        );
        merge_histogram(
            &mut self.tracked_recovery_exact_reconstructions,
            other.tracked_recovery_exact_reconstructions,
        );
        merge_histogram(
            &mut self.tracked_identity_anchors_pre_frontier,
            other.tracked_identity_anchors_pre_frontier,
        );
        merge_histogram(
            &mut self.tracked_identity_anchors_exact_pre_frontier,
            other.tracked_identity_anchors_exact_pre_frontier,
        );
        merge_histogram(
            &mut self.tracked_identity_anchors_post_frontier,
            other.tracked_identity_anchors_post_frontier,
        );
        merge_histogram(
            &mut self.tracked_identity_anchors_exact,
            other.tracked_identity_anchors_exact,
        );
        merge_histogram(
            &mut self.oracle_target_slot_paradigms,
            other.oracle_target_slot_paradigms,
        );
        merge_histogram(
            &mut self.intersection_target_slot_paradigms,
            other.intersection_target_slot_paradigms,
        );
    }

    fn finish(&mut self) {
        self.unique_proof_events = self.proof_identities.len();
        self.unique_target_lemmas = self.target_lemma_ids.len();
    }
}

fn record_histogram<K: Ord>(histogram: &mut BTreeMap<K, usize>, value: K) {
    *histogram.entry(value).or_default() += 1;
}

fn merge_histogram<K: Ord>(target: &mut BTreeMap<K, usize>, source: BTreeMap<K, usize>) {
    for (key, count) in source {
        *target.entry(key).or_default() += count;
    }
}

fn outside_h_mechanism(diagnostics: &TargetColdGroundingDiagnosticsV1) -> &'static str {
    if !diagnostics.observation_found {
        "NO_LEXICAL_OBSERVATION"
    } else if diagnostics.oracle.source_count == 0 {
        "NO_MAPPED_SOURCE_SLOT"
    } else if diagnostics.oracle.posting_paradigm_count == 0 {
        "NO_COMPATIBILITY_POSTING"
    } else if diagnostics.oracle.slot_compatible_paradigm_count == 0 {
        "NO_PARADIGM_COVERS_OBSERVED_SLOTS"
    } else if diagnostics.oracle.exact_reconstructing_paradigm_count == 0 {
        "NO_PARADIGM_RECONSTRUCTS_EXPOSED_FORMS"
    } else if diagnostics.oracle_paradigm_count == 0 {
        "NO_TARGET_POS_PARADIGM_RECONSTRUCTS_EXPOSED_FORMS"
    } else {
        "TARGET_LEMMA_BINDING_MISSING_AFTER_RECONSTRUCTION"
    }
}

fn h_to_b_mechanism(diagnostics: &TargetColdGroundingDiagnosticsV1) -> &'static str {
    let tracked = diagnostics.tracked_paradigm_count;
    if diagnostics.target_blind.source_count == 0 {
        "TARGET_MASK_REMOVED_ALL_PRINCIPAL_PARTS"
    } else if tracked > 0 && diagnostics.tracked_pos_without_source_count == tracked {
        "ORACLE_POS_HAS_NO_TARGET_BLIND_SOURCE"
    } else if diagnostics.tracked_identity_anchor_exact_pre_frontier_count > 0
        && diagnostics.tracked_recovery_exact_reconstructing_count == 0
    {
        "ORACLE_EXACT_IDENTITY_ANCHOR_DROPPED_BY_FRONTIER"
    } else if diagnostics.tracked_in_recovery_postings_count == 0
        && diagnostics.tracked_identity_anchor_exact_pre_frontier_count == 0
    {
        "ORACLE_EXACT_RECOVERY_ANCHOR_NOT_GENERATED"
    } else if diagnostics.tracked_recovered_anchor_count > 0
        && diagnostics.tracked_recovery_post_frontier_count == 0
    {
        "ORACLE_PARADIGM_RECOVERY_CANDIDATES_DROPPED_BY_FRONTIER"
    } else if diagnostics.tracked_recovery_post_frontier_count > 0
        && diagnostics.tracked_recovery_exact_reconstructing_count == 0
    {
        "ORACLE_PARADIGM_REACHES_FRONTIER_WITHOUT_EXACT_BINDING"
    } else if diagnostics.tracked_in_recovery_postings_count > 0
        && diagnostics.tracked_recovered_anchor_count == 0
    {
        "ORACLE_RECOVERY_PROGRAM_NOT_APPLICABLE"
    } else if diagnostics.tracked_in_postings_count == 0 {
        "ORACLE_PARADIGM_ABSENT_FROM_SOURCE_SLOT_POSTINGS"
    } else if diagnostics.tracked_slot_compatible_count == 0 {
        "ORACLE_PARADIGM_MISSES_AN_OBSERVED_SLOT"
    } else if diagnostics.tracked_exact_reconstructing_count == 0 {
        "ORACLE_PARADIGM_FAILS_EXPOSED_FORM_RECONSTRUCTION"
    } else {
        "ORACLE_BINDING_MISSING_AFTER_EXACT_RECONSTRUCTION"
    }
}

fn b_to_s0_mechanism(diagnostics: &TargetColdGroundingDiagnosticsV1) -> &'static str {
    if diagnostics.oracle_target_slot_paradigm_count == 0 {
        if diagnostics.target_slot_observed {
            "OBSERVED_TARGET_SLOT_ABSENT_FROM_ORACLE_PARADIGMS"
        } else {
            "UNOBSERVED_TARGET_SLOT_ABSENT_FROM_ORACLE_PARADIGMS"
        }
    } else if diagnostics.intersection_target_slot_paradigm_count == 0 {
        "TARGET_SLOT_LOST_ACROSS_ORACLE_BINDING_INTERSECTION"
    } else {
        "S0_PROOF_INVARIANT_MISMATCH"
    }
}

#[allow(clippy::too_many_arguments)]
fn load_or_build_frozen_hypothesis_manifest(
    path: &Path,
    l11_sha256: [u8; 32],
    canonical_l2_sha256: [u8; 32],
    productive_sha256: [u8; 32],
    axis_schema_sha256: [u8; 32],
    spool_path: &Path,
    frozen_runtime: &PackagedProductiveRuntimeV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    axis_schema: &super::transition_reduce::MorphologyAxisSchemaV1,
) -> io::Result<FrozenHypothesisIndexV1> {
    let spool_metadata = std::fs::metadata(spool_path)?;
    let proof_spool_sha256 = sha256_file(spool_path)?;
    validate_frozen_manifest_identities(
        productive_sha256,
        proof_spool_sha256,
        spool_metadata.len(),
    )?;
    let expected = (
        hex_sha256(l11_sha256),
        hex_sha256(canonical_l2_sha256),
        hex_sha256(axis_schema_sha256),
    );
    let manifest = if path.is_file() {
        serde_json::from_slice::<FrozenHypothesisManifestV1>(&std::fs::read(path)?)
            .map_err(io::Error::other)?
    } else {
        let (cases, _, _) = sample_cases(
            frozen_runtime,
            spool_path,
            FROZEN_MANIFEST_HELDOUT_PER_CLASS,
        )?;
        let mut oracle_cache = BTreeMap::<([u8; 32], u32, u16), Vec<u32>>::new();
        let mut entries = Vec::with_capacity(FROZEN_MANIFEST_ENTRY_COUNT);
        for case in cases
            .iter()
            .filter(|case| case.cohort == ProofCohortV1::LemmaHeldout)
        {
            let targets = valid_targets(
                &case.event.valid_targets,
                frozen_runtime,
                canonical_l2,
                axis_schema,
            )
            .map_err(io::Error::other)?;
            let target = targets.first().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "frozen manifest case has no valid target",
                )
            })?;
            if targets.iter().any(|candidate| {
                candidate.lemma_id != target.lemma_id || candidate.pos_domain != target.pos_domain
            }) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "frozen manifest case spans multiple target lemma/POS identities",
                ));
            }
            let cache_key = (
                case.event.proof_identity,
                target.lemma_id,
                target.pos_domain,
            );
            let oracle_paradigm_ids = if let Some(cached) = oracle_cache.get(&cache_key) {
                cached.clone()
            } else {
                let observation = canonical_l2
                    .lexical_lemma_observation_v1(target.lemma_id)
                    .map_err(io::Error::other)?
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "frozen manifest target lacks a lexical observation",
                        )
                    })?;
                let sources =
                    cold_sources_from_observation(&observation, frozen_runtime, axis_schema)
                        .map_err(io::Error::other)?;
                let mut paradigms = frozen_runtime
                    .derive_cold_lemma_bindings(target.lemma_id, &sources)
                    .map_err(io::Error::other)?
                    .into_iter()
                    .filter(|binding| binding.pos_domain() == target.pos_domain)
                    .map(|binding| binding.paradigm_id())
                    .collect::<Vec<_>>();
                paradigms.sort_unstable();
                paradigms.dedup();
                oracle_cache.insert(cache_key, paradigms.clone());
                paradigms
            };
            entries.push(FrozenHypothesisEntryV1 {
                proof_identity: case.event.proof_identity,
                damage_class: case.class.to_string(),
                damage_identity: damage_identity(
                    &case.event.proof_identity,
                    case.class,
                    &case.damaged_surface,
                ),
                target_lemma_id: target.lemma_id,
                target_pos_domain: target.pos_domain,
                oracle_paradigm_ids,
            });
        }
        entries.sort();
        let h_count = entries
            .iter()
            .filter(|entry| !entry.oracle_paradigm_ids.is_empty())
            .count();
        if entries.len() != FROZEN_MANIFEST_ENTRY_COUNT || h_count != FROZEN_MANIFEST_H_COUNT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "frozen manifest denominator mismatch entries={} H={h_count}",
                    entries.len()
                ),
            ));
        }
        let payload_sha256 = hex_sha256(
            Sha256::digest(serde_json::to_vec(&entries).map_err(io::Error::other)?).into(),
        );
        let manifest = FrozenHypothesisManifestV1 {
            schema_version: FROZEN_MANIFEST_SCHEMA_VERSION,
            v64_package_sha256: hex_sha256(productive_sha256),
            proof_spool_sha256: hex_sha256(proof_spool_sha256),
            proof_spool_bytes: spool_metadata.len(),
            l11_package_sha256: expected.0.clone(),
            canonical_l2_package_sha256: expected.1.clone(),
            axis_schema_sha256: expected.2.clone(),
            heldout_per_class: FROZEN_MANIFEST_HELDOUT_PER_CLASS,
            cohorts: vec![ProofCohortV1::LemmaHeldout.name().to_string()],
            damage_generator_id: FROZEN_DAMAGE_GENERATOR_ID.to_string(),
            entry_count: entries.len(),
            h_count,
            payload_sha256,
            entries,
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let temporary = path.with_extension("json.tmp");
        std::fs::write(
            &temporary,
            serde_json::to_vec_pretty(&manifest).map_err(io::Error::other)?,
        )?;
        std::fs::rename(temporary, path)?;
        manifest
    };
    validate_frozen_manifest(&manifest, &expected)?;
    let mut entries = BTreeMap::new();
    for entry in manifest.entries {
        let key = (entry.proof_identity, entry.damage_identity);
        if entries.insert(key, entry).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "frozen manifest contains a duplicate case identity",
            ));
        }
    }
    Ok(FrozenHypothesisIndexV1 {
        path: path.to_path_buf(),
        payload_sha256: manifest.payload_sha256,
        entries,
        h_count: manifest.h_count,
    })
}

fn validate_frozen_manifest_identities(
    productive_sha256: [u8; 32],
    proof_spool_sha256: [u8; 32],
    proof_spool_bytes: u64,
) -> io::Result<()> {
    if hex_sha256(productive_sha256) != FROZEN_V64_PACKAGE_SHA256
        || hex_sha256(proof_spool_sha256) != FROZEN_PROOF_SPOOL_SHA256
        || proof_spool_bytes != FROZEN_PROOF_SPOOL_BYTES
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "V66 frozen manifest package or proof spool identity mismatch",
        ));
    }
    Ok(())
}

fn validate_frozen_manifest(
    manifest: &FrozenHypothesisManifestV1,
    expected: &(String, String, String),
) -> io::Result<()> {
    let payload_sha256 = hex_sha256(
        Sha256::digest(serde_json::to_vec(&manifest.entries).map_err(io::Error::other)?).into(),
    );
    let sorted_unique = manifest.entries.windows(2).all(|pair| pair[0] < pair[1]);
    let measured_h = manifest
        .entries
        .iter()
        .filter(|entry| !entry.oracle_paradigm_ids.is_empty())
        .count();
    if manifest.schema_version != FROZEN_MANIFEST_SCHEMA_VERSION
        || manifest.v64_package_sha256 != FROZEN_V64_PACKAGE_SHA256
        || manifest.proof_spool_sha256 != FROZEN_PROOF_SPOOL_SHA256
        || manifest.proof_spool_bytes != FROZEN_PROOF_SPOOL_BYTES
        || manifest.l11_package_sha256 != expected.0
        || manifest.canonical_l2_package_sha256 != expected.1
        || manifest.axis_schema_sha256 != expected.2
        || manifest.heldout_per_class != FROZEN_MANIFEST_HELDOUT_PER_CLASS
        || manifest.cohorts != vec![ProofCohortV1::LemmaHeldout.name().to_string()]
        || manifest.damage_generator_id != FROZEN_DAMAGE_GENERATOR_ID
        || manifest.entry_count != FROZEN_MANIFEST_ENTRY_COUNT
        || manifest.entries.len() != FROZEN_MANIFEST_ENTRY_COUNT
        || manifest.h_count != FROZEN_MANIFEST_H_COUNT
        || measured_h != FROZEN_MANIFEST_H_COUNT
        || manifest.payload_sha256 != payload_sha256
        || !sorted_unique
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frozen V64 hypothesis manifest validation failed",
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(in crate::nanda_wave::l2_field) fn prove_productive_paradigm_field_v1(
    l1_package_path: &Path,
    l2_package_path: &Path,
    productive_package_path: &Path,
    axis_schema_path: &Path,
    work_dir: &Path,
    heldout_per_class: usize,
    requested_workers: usize,
) -> io::Result<serde_json::Value> {
    prove_productive_paradigm_field_v1_inner(
        l1_package_path,
        l2_package_path,
        productive_package_path,
        axis_schema_path,
        work_dir,
        heldout_per_class,
        requested_workers,
        false,
    )
}

pub(in crate::nanda_wave::l2_field) fn prove_productive_paradigm_field_v1_semantic(
    l1_package_path: &Path,
    l2_package_path: &Path,
    productive_package_path: &Path,
    axis_schema_path: &Path,
    work_dir: &Path,
    heldout_per_class: usize,
    requested_workers: usize,
) -> io::Result<serde_json::Value> {
    prove_productive_paradigm_field_v1_inner(
        l1_package_path,
        l2_package_path,
        productive_package_path,
        axis_schema_path,
        work_dir,
        heldout_per_class,
        requested_workers,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn prove_productive_paradigm_field_v1_inner(
    l1_package_path: &Path,
    l2_package_path: &Path,
    productive_package_path: &Path,
    axis_schema_path: &Path,
    work_dir: &Path,
    heldout_per_class: usize,
    requested_workers: usize,
    semantic_proof_authority: bool,
) -> io::Result<serde_json::Value> {
    if heldout_per_class == 0 || requested_workers == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "productive V1 proof requires heldout-per-class and workers > 0",
        ));
    }
    let l11_sha256 = sha256_file(l1_package_path)?;
    let canonical_l2_sha256 = sha256_file(l2_package_path)?;
    let productive_sha256 = sha256_file(productive_package_path)?;
    let axis_schema_sha256 = sha256_file(axis_schema_path)?;
    let canonical_l2 = super::super::runtime::StandaloneL2Field::load(l2_package_path)
        .map_err(io::Error::other)?;
    let axis_schema = load_axis_schema(axis_schema_path).map_err(io::Error::other)?;
    let load_started = Instant::now();
    let runtime = if semantic_proof_authority {
        PackagedProductiveRuntimeV1::load_with_semantic_proof_authority(
            productive_package_path,
            l11_sha256,
            canonical_l2_sha256,
        )
    } else {
        PackagedProductiveRuntimeV1::load(productive_package_path, l11_sha256, canonical_l2_sha256)
    }
    .map_err(io::Error::other)?;
    let frozen_oracle_runtime = PackagedProductiveRuntimeV1::load_without_anchor_recovery(
        productive_package_path,
        l11_sha256,
        canonical_l2_sha256,
    )
    .map_err(io::Error::other)?;
    let cold_load_us = load_started.elapsed().as_micros() as u64;

    let spool_path = work_dir.join("context-sorted/sorted-events-global.p2s");
    let frozen_manifest = load_or_build_frozen_hypothesis_manifest(
        &work_dir.join(FROZEN_MANIFEST_FILE),
        l11_sha256,
        canonical_l2_sha256,
        productive_sha256,
        axis_schema_sha256,
        &spool_path,
        &frozen_oracle_runtime,
        &canonical_l2,
        &axis_schema,
    )?;
    let sampling_started = Instant::now();
    let (cases, scanned_proof_events, sampled_by_cohort) =
        sample_cases(&runtime, &spool_path, heldout_per_class)?;
    let sampling_us = sampling_started.elapsed().as_micros() as u64;
    let workers = requested_workers
        .min(
            std::thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1),
        )
        .min(cases.len().max(1));
    let mut worker_cases = vec![Vec::<usize>::new(); workers];
    for case_index in 0..cases.len() {
        worker_cases[case_index % workers].push(case_index);
    }
    let worker_case_counts = worker_cases.iter().map(Vec::len).collect::<Vec<_>>();
    let proof_started = Instant::now();
    let partials = std::thread::scope(|scope| {
        let cases = &cases;
        worker_cases
            .iter()
            .enumerate()
            .map(|(worker_id, case_indices)| {
                let runtime = &runtime;
                let frozen_oracle_runtime = &frozen_oracle_runtime;
                let canonical_l2 = &canonical_l2;
                let axis_schema = &axis_schema;
                let frozen_manifest = &frozen_manifest;
                scope.spawn(move || {
                    let started = Instant::now();
                    let shard = evaluate_cases(
                        case_indices,
                        cases,
                        runtime,
                        frozen_oracle_runtime,
                        canonical_l2,
                        axis_schema,
                        frozen_manifest,
                    );
                    WorkerProofPartialV1 {
                        worker_id,
                        case_count: case_indices.len(),
                        elapsed_us: started.elapsed().as_micros() as u64,
                        shard,
                    }
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|worker| worker.join().expect("productive V1 proof worker panicked"))
            .collect::<Vec<_>>()
    });
    let mut proof = ProofShardV1::default();
    let mut worker_telemetry = Vec::with_capacity(partials.len());
    for partial in partials {
        worker_telemetry.push(serde_json::json!({
            "worker_id": partial.worker_id,
            "case_count": partial.case_count,
            "elapsed_us": partial.elapsed_us,
        }));
        proof.merge(partial.shard);
    }
    worker_telemetry.sort_by_key(|row| {
        row.get("worker_id")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default()
    });
    let worker_elapsed = worker_telemetry
        .iter()
        .filter_map(|row| row.get("elapsed_us").and_then(serde_json::Value::as_u64))
        .collect::<Vec<_>>();
    let worker_tail_us = worker_elapsed
        .iter()
        .max()
        .copied()
        .unwrap_or_default()
        .saturating_sub(worker_elapsed.iter().min().copied().unwrap_or_default());
    let proof_us = proof_started.elapsed().as_micros() as u64;
    for metrics in proof.classes.values_mut() {
        metrics.finish();
    }
    proof.first_loss_diagnostics.finish();
    let stage_telemetry_percentiles = proof.stage_telemetry.percentiles();
    let slow_calls = std::mem::take(&mut proof.slow_calls)
        .into_iter()
        .map(|((cohort, class), calls)| (format!("{}::{class}", cohort.name()), calls))
        .collect::<BTreeMap<_, _>>();

    let class_count_by_cohort = [ProofCohortV1::SeenExact, ProofCohortV1::LemmaHeldout]
        .into_iter()
        .map(|cohort| {
            (
                cohort.name(),
                proof
                    .classes
                    .keys()
                    .filter(|(candidate, _)| *candidate == cohort)
                    .count(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let classes = proof
        .classes
        .into_iter()
        .map(|((cohort, class), metrics)| (format!("{}::{class}", cohort.name()), metrics))
        .collect::<BTreeMap<_, _>>();
    let evaluated = classes.values().map(|metrics| metrics.cases).sum::<usize>();
    let lemma_heldout = classes
        .iter()
        .filter(|(key, _)| key.starts_with("LEMMA_HELDOUT::"))
        .map(|(_, metrics)| metrics)
        .collect::<Vec<_>>();
    let sampled_h = lemma_heldout
        .iter()
        .map(|metrics| metrics.hypothesis_covered)
        .sum::<usize>();
    let sampled_b = lemma_heldout
        .iter()
        .map(|metrics| metrics.compatible_binding_retained)
        .sum::<usize>();
    let sampled_s0 = lemma_heldout
        .iter()
        .map(|metrics| metrics.target_slot_in_binding)
        .sum::<usize>();
    let raw_top1 = lemma_heldout
        .iter()
        .map(|metrics| metrics.target_top1)
        .sum::<usize>();
    let base_raw_top1 = lemma_heldout
        .iter()
        .map(|metrics| metrics.base_target_top1)
        .sum::<usize>();
    let maximum_class_p99_us = classes
        .values()
        .map(|metrics| metrics.latency_p99_us)
        .max()
        .unwrap_or_default();
    let base_projection_comparisons = classes
        .values()
        .map(|metrics| metrics.base_projection_comparisons)
        .sum::<usize>();
    let base_projection_failures = classes
        .values()
        .map(|metrics| metrics.base_projection_failures)
        .sum::<usize>();
    let demotions_without_certificate = classes
        .values()
        .map(|metrics| metrics.demotions_without_certificate)
        .sum::<usize>();
    let false_singleton = classes
        .values()
        .map(|metrics| metrics.shadow_false_singleton)
        .sum::<usize>();
    let integrity_errors = classes
        .values()
        .map(|metrics| metrics.integrity_errors)
        .sum::<usize>();
    let probe_parity_complete = proof.probe_parity_comparisons == evaluated;
    let fixed_full_denominator = heldout_per_class == 100;
    let denominator_gate = !fixed_full_denominator || sampled_h == 1_280;
    let base_top1_gate =
        raw_top1 >= base_raw_top1 && (!fixed_full_denominator || base_raw_top1 == 267);
    let measured_quality_gate = class_count_by_cohort
        .values()
        .all(|count| *count == EXPECTED_DAMAGE_CLASSES)
        && lemma_heldout
            .iter()
            .all(|metrics| metrics.target_top16_percent > 95.0)
        && sampled_h.saturating_sub(sampled_b) == 0
        && sampled_b.saturating_sub(sampled_s0) == 0
        && denominator_gate
        && base_top1_gate
        && base_projection_failures == 0
        && demotions_without_certificate == 0
        && maximum_class_p99_us <= 5_000
        && probe_parity_complete
        && proof.probe_parity_failures == 0;
    let v66_bounded_recovery = serde_json::json!({
        "sampled_h": sampled_h,
        "sampled_b": sampled_b,
        "sampled_s0": sampled_s0,
        "h_to_b_losses": sampled_h.saturating_sub(sampled_b),
        "b_to_s0_losses": sampled_b.saturating_sub(sampled_s0),
        "raw_top1": raw_top1,
        "base_v64_raw_top1": base_raw_top1,
        "base_projection_comparisons": base_projection_comparisons,
        "base_projection_failures": base_projection_failures,
        "demotions_without_independent_certificate": demotions_without_certificate,
        "maximum_class_p99_us": maximum_class_p99_us,
    });
    let v66_gates = serde_json::json!({
        "each_top16_retained_strictly_above_percent": 95.0,
        "full_frozen_h_equals": 1280,
        "h_to_b_losses_equals": 0,
        "b_to_s0_losses_equals": 0,
        "full_base_raw_top1_equals": 267,
        "experimental_raw_top1_at_least_base": true,
        "base_projection_failures_equals": 0,
        "demotions_without_independent_certificate_equals": 0,
        "maximum_class_p99_us_at_most": 5000,
        "false_singleton_equals": 0,
        "integrity_errors_equals": 0
    });

    Ok(serde_json::json!({
        "kind": "l2_productive_paradigm_field_v1_fixed_shadow_proof",
        "verdict": if measured_quality_gate && false_singleton == 0 && integrity_errors == 0 {
            "PASS_measured_cohorts_but_promotion_incomplete"
        } else {
            "FAIL_measured_shadow_gates"
        },
        "promotion_eligible": false,
        "runtime_authority_changed": false,
        "shared_replay_owner": if semantic_proof_authority { "semantic_proof_authority" } else { "legacy" },
        "semantic_authority_scope": if semantic_proof_authority { "proof process only; not daemon or installed runtime" } else { "disabled" },
        "scope": "conditional packaged productive readout with target and explicit competitor lemma identities grounded from immutable proof events",
        "v64_change_scope": {
            "runtime": "surface-equivalent candidates are coalesced before the global top-32 by lemma_id, target_slot_id, and normalized_surface",
            "proof": "read-only H/B/S0/S1/S2/S3/R probes; probed and unprobed readouts are executed independently",
            "unchanged": [
                "raw corpus",
                "transition induction",
                "physical slot limit 16",
                "physical global limit 32",
                "calibrated coefficients",
                "authority thresholds",
                "SafetyGate and verifier"
            ]
        },
        "post_v64_anchor_recovery_scope": {
            "runtime": "optional mmap sidecar recovers a canonical anchor from an exposed source slot, then reuses the unchanged forward paradigm trie",
            "admission": "every recovered anchor must replay every exposed form exactly; recovery never grants Winner authority",
            "evidence": "reverse programs require at least two independent train lemmas inside the same learned paradigm",
            "base_productive_package_unchanged": true
        },
        "frozen_hypothesis_scope": {
            "oracle_runtime": "base productive package with anchor recovery explicitly disabled",
            "experimental_runtime_can_change_h": false,
            "base_package_sha256": hex_sha256(sha256_file(productive_package_path)?),
            "manifest_path": frozen_manifest.path,
            "manifest_payload_sha256": frozen_manifest.payload_sha256,
            "manifest_entry_count": frozen_manifest.entries.len(),
            "manifest_h_count": frozen_manifest.h_count,
        },
        "v66_bounded_recovery": v66_bounded_recovery,
        "stage_definitions": {
            "H": "true heldout paradigm is representable by the train-learned paradigm hypothesis class",
            "B": "an oracle-compatible paradigm remains in the target-blind cold binding set",
            "S0": "target slot exists in a retained compatible binding before candidate execution",
            "S1": "exact target surface is executed before the per-binding slot frontier",
            "S2": "exact target surface survives the per-binding 16-slot frontier",
            "S3": "exact target surface survives surface-basin coalescing and the global 32-basin frontier",
            "R": "exact target surface survives Winner, Tied, or ABSTAIN readout"
        },
        "packages": {
            "l11": l1_package_path,
            "canonical_l2": l2_package_path,
            "productive_v1": productive_package_path,
            "productive_bytes": runtime.package_bytes(),
            "anchor_recovery_bytes": runtime.anchor_recovery_package_bytes(),
            "anchor_recovery_paths": runtime.anchor_recovery_path_count(),
            "productive_mmap_backed": runtime.mmap_backed(),
            "productive_constant_cache_bytes": runtime.resident_cache_bytes(),
        },
        "proof_spool": spool_path,
        "heldout_per_class": heldout_per_class,
        "workers": workers,
        "proof_scheduler": {
            "kind": "deterministic_round_robin_v1",
            "worker_case_counts": worker_case_counts,
            "worker_telemetry": worker_telemetry,
            "worker_tail_us": worker_tail_us,
            "contiguous_chunks": false,
        },
        "scanned_proof_events": scanned_proof_events,
        "sampled_events_by_cohort": sampled_by_cohort,
        "evaluated": evaluated,
        "class_count_by_cohort": class_count_by_cohort,
        "classes": classes,
        "unique_resolvable": {
            "cases": evaluated,
            "note": "each immutable V1 proof event currently carries one valid target identity"
        },
        "multi_label": {
            "cases": 0,
            "gate_evaluated": false
        },
        "unsupported": {
            "cases": 0,
            "gate_evaluated": false
        },
        "false_singleton": false_singleton,
        "integrity_errors": integrity_errors,
        "probe_parity": {
            "comparison": "full structural equality of independently executed probed and unprobed readouts",
            "expected_comparisons": evaluated,
            "completed_comparisons": proof.probe_parity_comparisons,
            "failures": proof.probe_parity_failures,
            "exact": probe_parity_complete && proof.probe_parity_failures == 0
        },
        "failure_examples": proof.failure_examples,
        "bounded_recovery_diagnostics": proof.bounded_recovery,
        "first_loss_diagnostics": proof.first_loss_diagnostics,
        "stage_telemetry": {
            "enabled": std::env::var_os("LAY_PRODUCTIVE_STAGE_TELEMETRY").is_some(),
            "scope": "proof-only coarse stages for the independently timed productive readout; disabled in the normative latency gate",
            "totals": proof.stage_telemetry,
            "percentiles": stage_telemetry_percentiles,
            "three_slowest_calls_per_class": slow_calls,
        },
        "latency": {
            "cold_productive_mmap_load_us": cold_load_us,
            "sampling_us": sampling_us,
            "proof_us": proof_us,
            "measurement_scope": "closed evaluation time without queue or product IPC"
        },
        "memory": {
            "rss_kib": proc_status_kib("VmRSS:"),
            "peak_rss_kib": proc_status_kib("VmHWM:")
        },
        "gates": v66_gates,
        "measured": [
            "SEEN_EXACT and LEMMA_HELDOUT, 13 damage classes each",
            "H/B/S0/S1/S2/S3/R per class",
            "surface-basin candidate and readout retention",
            "probed versus unprobed exact parity",
            "shadow false singleton and package integrity",
            "closed-call latency, mmap package bytes, process RSS and peak RSS"
        ],
        "not_tested": [
            "BANK_UNSEEN: proof targets are canonical L2 exact identities by construction",
            "SLOT_HELDOUT: current compiler split is lemma-owned and has no frozen slot-heldout manifest",
            "MULTI_LABEL: current proof events carry one valid target identity each",
            "UNSUPPORTED false authority: no unsupported proof-event cohort exists",
            "L1.1 damage-to-lemma birth and full grounded lattice retention",
            "L3/L4/DecisionCore/verifier authority transfer",
            "queue-inclusive single-client and 20-client latency",
            "physical product matrix"
        ]
    }))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_fixed_shared_replay_audits_v1(
    l1_package_path: &Path,
    l2_package_path: &Path,
    productive_package_path: &Path,
    axis_schema_path: &Path,
    work_dir: &Path,
    heldout_per_class: usize,
    requested_workers: usize,
) -> io::Result<FixedSharedReplayAuditCorpusV1> {
    if heldout_per_class == 0 || requested_workers == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "productive semantic audit requires heldout-per-class and workers > 0",
        ));
    }
    let l11_sha256 = sha256_file(l1_package_path)?;
    let canonical_l2_sha256 = sha256_file(l2_package_path)?;
    let productive_sha256 = sha256_file(productive_package_path)?;
    let axis_schema_sha256 = sha256_file(axis_schema_path)?;
    let canonical_l2 = super::super::runtime::StandaloneL2Field::load(l2_package_path)
        .map_err(io::Error::other)?;
    let axis_schema = load_axis_schema(axis_schema_path).map_err(io::Error::other)?;
    let load_started = Instant::now();
    let runtime =
        PackagedProductiveRuntimeV1::load(productive_package_path, l11_sha256, canonical_l2_sha256)
            .map_err(io::Error::other)?;
    let semantic_runtime = PackagedProductiveRuntimeV1::load_with_semantic_proof_authority(
        productive_package_path,
        l11_sha256,
        canonical_l2_sha256,
    )
    .map_err(io::Error::other)?;
    let frozen_oracle_runtime = PackagedProductiveRuntimeV1::load_without_anchor_recovery(
        productive_package_path,
        l11_sha256,
        canonical_l2_sha256,
    )
    .map_err(io::Error::other)?;
    let cold_load_us = load_started.elapsed().as_micros() as u64;
    let spool_path = work_dir.join("context-sorted/sorted-events-global.p2s");
    let frozen_manifest = load_or_build_frozen_hypothesis_manifest(
        &work_dir.join(FROZEN_MANIFEST_FILE),
        l11_sha256,
        canonical_l2_sha256,
        productive_sha256,
        axis_schema_sha256,
        &spool_path,
        &frozen_oracle_runtime,
        &canonical_l2,
        &axis_schema,
    )?;
    let sampling_started = Instant::now();
    let (cases, scanned_proof_events, sampled_events_by_cohort) =
        sample_cases(&runtime, &spool_path, heldout_per_class)?;
    let sampling_us = sampling_started.elapsed().as_micros() as u64;
    let heldout_indices = cases
        .iter()
        .enumerate()
        .filter_map(|(index, case)| (case.cohort == ProofCohortV1::LemmaHeldout).then_some(index))
        .collect::<Vec<_>>();
    let workers = requested_workers
        .min(
            std::thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1),
        )
        .min(heldout_indices.len().max(1));
    let mut worker_cases = vec![Vec::new(); workers];
    for (ordinal, case_index) in heldout_indices.iter().copied().enumerate() {
        worker_cases[ordinal % workers].push(case_index);
    }
    let preparation_started = Instant::now();
    let partials = std::thread::scope(|scope| {
        let cases = &cases;
        let runtime = &runtime;
        let semantic_runtime = &semantic_runtime;
        let frozen_oracle_runtime = &frozen_oracle_runtime;
        let canonical_l2 = &canonical_l2;
        let axis_schema = &axis_schema;
        let frozen_manifest = &frozen_manifest;
        worker_cases
            .iter()
            .map(|indices| {
                scope.spawn(move || {
                    indices
                        .iter()
                        .map(|case_index| {
                            collect_case_shared_replay_audits(
                                &cases[*case_index],
                                &runtime,
                                semantic_runtime,
                                &frozen_oracle_runtime,
                                &canonical_l2,
                                &axis_schema,
                                &frozen_manifest,
                            )
                            .map(|case| (*case_index, case))
                        })
                        .collect::<Result<Vec<_>, String>>()
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .map_err(|_| "productive semantic audit worker panicked".to_string())?
            })
            .collect::<Result<Vec<_>, String>>()
    });
    let mut collected = Vec::with_capacity(heldout_indices.len());
    for partial in partials.map_err(io::Error::other)? {
        collected.extend(partial);
    }
    let preparation_us = preparation_started.elapsed().as_micros() as u64;
    collected.sort_by_key(|(case_index, _)| *case_index);
    let cases = collected
        .into_iter()
        .map(|(_, case)| case)
        .collect::<Vec<_>>();
    let binding_parity_comparisons = cases.len();
    let binding_parity_failures = cases
        .iter()
        .filter(|case| !case.binding_parity_exact)
        .count();
    let legacy_grounding_us = cases.iter().map(|case| case.legacy_grounding_us).sum();
    let semantic_grounding_us = cases.iter().map(|case| case.semantic_grounding_us).sum();
    Ok(FixedSharedReplayAuditCorpusV1 {
        cases,
        scanned_proof_events,
        sampled_events_by_cohort,
        workers,
        cold_load_us,
        sampling_us,
        preparation_us,
        binding_parity_comparisons,
        binding_parity_failures,
        legacy_grounding_us,
        semantic_grounding_us,
    })
}

fn collect_case_shared_replay_audits(
    case: &ProofCaseV1,
    runtime: &PackagedProductiveRuntimeV1,
    semantic_runtime: &PackagedProductiveRuntimeV1,
    frozen_oracle_runtime: &PackagedProductiveRuntimeV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    axis_schema: &super::transition_reduce::MorphologyAxisSchemaV1,
    frozen_manifest: &FrozenHypothesisIndexV1,
) -> Result<FixedSharedReplayAuditCaseV1, String> {
    let (targets, frozen_entry) =
        case_targets_and_frozen_entry(case, runtime, canonical_l2, axis_schema, frozen_manifest)?;
    let mut identities = case.event.valid_targets.clone();
    identities.extend(case.event.explicit_invalid_competitors.iter().copied());
    identities.sort_unstable();
    identities.dedup();
    let mut audits = Vec::new();
    let legacy_started = Instant::now();
    let legacy = build_groundings(
        &identities,
        &case.event.valid_targets,
        &targets,
        runtime,
        frozen_oracle_runtime,
        canonical_l2,
        axis_schema,
        frozen_entry,
        Some(&mut audits),
    )?;
    let legacy_grounding_us = legacy_started.elapsed().as_micros() as u64;
    let semantic_started = Instant::now();
    let semantic = build_groundings(
        &identities,
        &case.event.valid_targets,
        &targets,
        semantic_runtime,
        frozen_oracle_runtime,
        canonical_l2,
        axis_schema,
        frozen_entry,
        None,
    )?;
    let semantic_grounding_us = semantic_started.elapsed().as_micros() as u64;
    Ok(FixedSharedReplayAuditCaseV1 {
        class: case.class,
        proof_identity: case.event.proof_identity,
        audits,
        binding_parity_exact: legacy.cold == semantic.cold,
        legacy_grounding_us,
        semantic_grounding_us,
        legacy_binding_count: legacy.cold.len(),
        semantic_binding_count: semantic.cold.len(),
    })
}

fn sample_cases(
    runtime: &PackagedProductiveRuntimeV1,
    spool_path: &Path,
    heldout_per_class: usize,
) -> io::Result<(Vec<ProofCaseV1>, usize, BTreeMap<&'static str, usize>)> {
    let capacity = heldout_per_class
        .saturating_mul(48)
        .max(heldout_per_class.saturating_add(128));
    let mut reservoirs = BTreeMap::<ProofCohortV1, BinaryHeap<SampledProofEventV1>>::new();
    let mut lemma_seen = BTreeMap::<u32, bool>::new();
    let mut reader = VerifiedSpoolShardReaderV1::open(spool_path).map_err(io::Error::other)?;
    let mut scanned = 0_usize;
    while let Some(record) = reader.next_record().map_err(io::Error::other)? {
        let event = decode_verified_spool_record(&record, runtime.split_seed())
            .map_err(io::Error::other)?;
        let TypedProductiveEventV1::Proof(event) = event else {
            continue;
        };
        scanned += 1;
        let Some(target) = event.valid_targets.first() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "productive proof event has no target",
            ));
        };
        let packaged = match lemma_seen.get(&target.lemma_ref.0) {
            Some(value) => *value,
            None => {
                let value = runtime
                    .has_packaged_lemma(target.lemma_ref.0)
                    .map_err(io::Error::other)?;
                lemma_seen.insert(target.lemma_ref.0, value);
                value
            }
        };
        let cohort = if packaged {
            ProofCohortV1::SeenExact
        } else {
            ProofCohortV1::LemmaHeldout
        };
        let mut hasher = Sha256::new();
        hasher.update(b"lay-productive-v1-fixed-proof-sample\0");
        hasher.update([cohort as u8]);
        hasher.update(event.proof_identity);
        let sampled = SampledProofEventV1 {
            key: hasher.finalize().into(),
            event,
        };
        let reservoir = reservoirs.entry(cohort).or_default();
        if reservoir.len() < capacity {
            reservoir.push(sampled);
        } else if reservoir.peek().is_some_and(|largest| sampled < *largest) {
            reservoir.pop();
            reservoir.push(sampled);
        }
    }

    let mut cases = Vec::new();
    let mut sampled_by_cohort = BTreeMap::new();
    for cohort in [ProofCohortV1::SeenExact, ProofCohortV1::LemmaHeldout] {
        let mut seeds = reservoirs
            .remove(&cohort)
            .unwrap_or_default()
            .into_sorted_vec();
        seeds.sort_unstable();
        sampled_by_cohort.insert(cohort.name(), seeds.len());
        let mut counts = BTreeMap::<&'static str, usize>::new();
        let mut seen_requests = BTreeSet::new();
        for seed in seeds {
            let (_, heldout) = split_damages(&seed.event.observed_surface);
            let mut by_class = BTreeMap::<&'static str, DamageExample>::new();
            for damage in heldout {
                by_class
                    .entry(damage.class)
                    .and_modify(|current| {
                        if damage_key(&seed.event.proof_identity, &damage)
                            < damage_key(&seed.event.proof_identity, current)
                        {
                            *current = damage.clone();
                        }
                    })
                    .or_insert(damage);
            }
            for (class, damage) in by_class {
                if counts.get(class).copied().unwrap_or_default() >= heldout_per_class {
                    continue;
                }
                let request_key = (class, damage.surface.clone(), seed.event.proof_identity);
                if !seen_requests.insert(request_key) {
                    continue;
                }
                cases.push(ProofCaseV1 {
                    cohort,
                    class,
                    damaged_surface: damage.surface,
                    event: seed.event.clone(),
                });
                *counts.entry(class).or_default() += 1;
            }
            if counts.len() == EXPECTED_DAMAGE_CLASSES
                && counts.values().all(|count| *count == heldout_per_class)
            {
                break;
            }
        }
        if counts.len() != EXPECTED_DAMAGE_CLASSES
            || counts.values().any(|count| *count != heldout_per_class)
        {
            let counts = counts
                .into_iter()
                .map(|(class, count)| format!("{class}={count}"))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "productive V1 proof cannot fill {}::{heldout_per_class}: {counts}",
                    cohort.name()
                ),
            ));
        }
    }
    Ok((cases, scanned, sampled_by_cohort))
}

fn evaluate_cases(
    case_indices: &[usize],
    cases: &[ProofCaseV1],
    runtime: &PackagedProductiveRuntimeV1,
    frozen_oracle_runtime: &PackagedProductiveRuntimeV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    axis_schema: &super::transition_reduce::MorphologyAxisSchemaV1,
    frozen_manifest: &FrozenHypothesisIndexV1,
) -> ProofShardV1 {
    let mut shard = ProofShardV1::default();
    for case in case_indices.iter().map(|index| &cases[*index]) {
        let result = evaluate_case(
            case,
            runtime,
            frozen_oracle_runtime,
            canonical_l2,
            axis_schema,
            frozen_manifest,
        );
        let class_key = (case.cohort, case.class);
        let metrics = shard.classes.entry(class_key).or_default();
        match result {
            Ok(result) => {
                shard.probe_parity_comparisons += 1;
                metrics.cases += 1;
                metrics.oracle_applicable_cases += usize::from(result.oracle_applicable);
                metrics.hypothesis_covered += usize::from(result.hypothesis_covered);
                metrics.compatible_binding_retained +=
                    usize::from(result.compatible_binding_retained);
                metrics.target_slot_in_binding += usize::from(result.target_slot_in_binding);
                metrics.target_exact_pre_slot_bound +=
                    usize::from(result.target_exact_pre_slot_bound);
                metrics.target_exact_post_slot_bound +=
                    usize::from(result.target_exact_post_slot_bound);
                metrics.target_exact_post_surface_basin_bound +=
                    usize::from(result.target_exact_post_surface_basin_bound);
                metrics.target_lemma_born += usize::from(result.target_lemma_born);
                metrics.target_slot_born += usize::from(result.target_slot_born);
                metrics.target_exact_born += usize::from(result.target_exact_born);
                metrics.target_top1 += usize::from(result.target_top1);
                metrics.base_target_top1 += usize::from(result.base_target_top1);
                metrics.target_top16 += usize::from(result.target_top16);
                metrics.readout_target_retained += usize::from(result.readout_target_retained);
                metrics.clean_target_retained += usize::from(result.clean_target_retained);
                metrics.winner += usize::from(result.winner);
                metrics.tied += usize::from(result.tied);
                metrics.abstain += usize::from(result.abstain);
                metrics.empty_lattice += usize::from(result.empty_lattice);
                metrics.shadow_false_singleton += usize::from(result.shadow_false_singleton);
                metrics.base_projection_comparisons += 1;
                metrics.base_projection_failures += usize::from(!result.base_projection_exact);
                metrics.demotions_without_certificate += result.demotions_without_certificate;
                metrics.latency_us.push(result.latency_us);
                if let Some(telemetry) = result.stage_telemetry {
                    shard.stage_telemetry.record(telemetry);
                    record_slow_call(
                        shard.slow_calls.entry(class_key).or_default(),
                        ProductiveSlowCallV1 {
                            proof_identity: case.event.proof_identity,
                            damaged_surface: case.damaged_surface.clone(),
                            target_surface: case.event.observed_surface.clone(),
                            elapsed_us: result.latency_us,
                            stages: telemetry,
                        },
                    );
                }
                if case.cohort == ProofCohortV1::LemmaHeldout {
                    if let Some(diagnostics) = result.target_cold_diagnostics.as_ref() {
                        shard.bounded_recovery.record(&diagnostics.target_blind);
                        shard.first_loss_diagnostics.record(
                            case.event.proof_identity,
                            result.target_lemma_id,
                            result.hypothesis_covered,
                            result.compatible_binding_retained,
                            result.target_slot_in_binding,
                            diagnostics,
                        );
                    }
                }
                if ((case.cohort == ProofCohortV1::LemmaHeldout
                    && (!result.hypothesis_covered
                        || !result.compatible_binding_retained
                        || !result.target_slot_in_binding
                        || !result.target_exact_pre_slot_bound
                        || !result.target_exact_post_slot_bound
                        || !result.target_exact_post_surface_basin_bound))
                    || !result.target_top16
                    || !result.readout_target_retained)
                    && shard.failure_examples.len() < 64
                {
                    shard.failure_examples.push(serde_json::json!({
                        "cohort": case.cohort.name(),
                        "class": case.class,
                        "damaged_surface": case.damaged_surface,
                        "target_surface": case.event.observed_surface,
                        "candidate_surfaces": result.candidate_surfaces,
                        "oracle_applicable": result.oracle_applicable,
                        "hypothesis_covered": result.hypothesis_covered,
                        "compatible_binding_retained": result.compatible_binding_retained,
                        "target_slot_in_binding": result.target_slot_in_binding,
                        "target_exact_pre_slot_bound": result.target_exact_pre_slot_bound,
                        "target_exact_post_slot_bound": result.target_exact_post_slot_bound,
                        "target_exact_post_surface_basin_bound": result.target_exact_post_surface_basin_bound,
                        "target_lemma_born": result.target_lemma_born,
                        "target_slot_born": result.target_slot_born,
                        "target_exact_born": result.target_exact_born,
                        "target_top16": result.target_top16,
                        "readout_target_retained": result.readout_target_retained,
                        "candidate_diagnostics": result.candidate_diagnostics,
                    }));
                }
            }
            Err(error) => {
                if error == TARGET_PROBE_PARITY_ERROR {
                    shard.probe_parity_comparisons += 1;
                    shard.probe_parity_failures += 1;
                }
                metrics.cases += 1;
                metrics.integrity_errors += 1;
                if shard.failure_examples.len() < 64 {
                    shard.failure_examples.push(serde_json::json!({
                        "cohort": case.cohort.name(),
                        "class": case.class,
                        "error": error,
                    }));
                }
            }
        }
    }
    shard
}

struct CaseResultV1 {
    target_lemma_id: u32,
    target_cold_diagnostics: Option<TargetColdGroundingDiagnosticsV1>,
    oracle_applicable: bool,
    hypothesis_covered: bool,
    compatible_binding_retained: bool,
    target_slot_in_binding: bool,
    target_exact_pre_slot_bound: bool,
    target_exact_post_slot_bound: bool,
    target_exact_post_surface_basin_bound: bool,
    target_lemma_born: bool,
    target_slot_born: bool,
    target_exact_born: bool,
    target_top1: bool,
    base_target_top1: bool,
    target_top16: bool,
    readout_target_retained: bool,
    clean_target_retained: bool,
    winner: bool,
    tied: bool,
    abstain: bool,
    empty_lattice: bool,
    shadow_false_singleton: bool,
    base_projection_exact: bool,
    demotions_without_certificate: usize,
    latency_us: u64,
    stage_telemetry: Option<ProductiveEvaluationTelemetryV1>,
    candidate_surfaces: Vec<String>,
    candidate_diagnostics: Vec<serde_json::Value>,
}

fn case_targets_and_frozen_entry<'a>(
    case: &ProofCaseV1,
    runtime: &PackagedProductiveRuntimeV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    axis_schema: &super::transition_reduce::MorphologyAxisSchemaV1,
    frozen_manifest: &'a FrozenHypothesisIndexV1,
) -> Result<(Vec<ValidTargetV1>, Option<&'a FrozenHypothesisEntryV1>), String> {
    let targets = valid_targets(
        &case.event.valid_targets,
        runtime,
        canonical_l2,
        axis_schema,
    )?;
    let frozen_entry = if case.cohort == ProofCohortV1::LemmaHeldout {
        let key = (
            case.event.proof_identity,
            damage_identity(
                &case.event.proof_identity,
                case.class,
                &case.damaged_surface,
            ),
        );
        let entry = frozen_manifest
            .entries
            .get(&key)
            .ok_or_else(|| "sampled case is absent from the frozen H manifest".to_string())?;
        let target = targets
            .first()
            .ok_or_else(|| "sampled frozen case has no target".to_string())?;
        if entry.damage_class != case.class
            || entry.target_lemma_id != target.lemma_id
            || entry.target_pos_domain != target.pos_domain
        {
            return Err("sampled case disagrees with its frozen H manifest entry".to_string());
        }
        Some(entry)
    } else {
        None
    };
    Ok((targets, frozen_entry))
}

fn evaluate_case(
    case: &ProofCaseV1,
    runtime: &PackagedProductiveRuntimeV1,
    frozen_oracle_runtime: &PackagedProductiveRuntimeV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    axis_schema: &super::transition_reduce::MorphologyAxisSchemaV1,
    frozen_manifest: &FrozenHypothesisIndexV1,
) -> Result<CaseResultV1, String> {
    let (targets, frozen_entry) =
        case_targets_and_frozen_entry(case, runtime, canonical_l2, axis_schema, frozen_manifest)?;
    let mut identities = case.event.valid_targets.clone();
    identities.extend(case.event.explicit_invalid_competitors.iter().copied());
    identities.sort_unstable();
    identities.dedup();
    let groundings = build_groundings(
        &identities,
        &case.event.valid_targets,
        &targets,
        runtime,
        frozen_oracle_runtime,
        canonical_l2,
        axis_schema,
        frozen_entry,
        None,
    )?;
    let oracle_applicable = case.cohort == ProofCohortV1::LemmaHeldout;
    let target_domains = targets
        .iter()
        .map(|target| (target.lemma_id, target.pos_domain))
        .collect::<BTreeSet<_>>();
    let hypothesis_covered = oracle_applicable
        && groundings
            .oracle_cold
            .iter()
            .any(|binding| target_domains.contains(&(binding.lemma_id(), binding.pos_domain())));
    let oracle_paradigms = groundings
        .oracle_cold
        .iter()
        .filter(|binding| target_domains.contains(&(binding.lemma_id(), binding.pos_domain())))
        .map(|binding| (binding.lemma_id(), binding.paradigm_id()))
        .collect::<BTreeSet<_>>();
    let compatible_binding_retained = oracle_applicable
        && groundings
            .cold
            .iter()
            .any(|binding| oracle_paradigms.contains(&(binding.lemma_id(), binding.paradigm_id())));
    let mut target_slot_in_binding = false;
    if oracle_applicable {
        for binding in &groundings.cold {
            for target in &targets {
                if oracle_paradigms.contains(&(binding.lemma_id(), binding.paradigm_id()))
                    && binding.lemma_id() == target.lemma_id
                    && runtime.cold_binding_has_slot(binding, target.slot_id)?
                {
                    target_slot_in_binding = true;
                    break;
                }
            }
            if target_slot_in_binding {
                break;
            }
        }
    }
    let mut scene = case.event.scene.clone();
    scene.current_token = case.damaged_surface.clone();
    scene.current_normalized_scalars = case.damaged_surface.chars().map(u32::from).collect();
    let mut target_probe = ProductiveTargetProbeV1::new(
        targets
            .iter()
            .map(|target| ProductiveTargetProbeIdentityV1 {
                lemma_id: target.lemma_id,
                target_slot_id: target.slot_id,
                normalized_surface: target.surface.clone(),
            })
            .collect(),
    );
    let base_readout = frozen_oracle_runtime.evaluate_shadow_with_cold_bindings(
        &case.damaged_surface,
        &scene,
        &groundings.grounded,
        &groundings.base_cold,
        false,
    );
    if let Some(error) = base_readout.integrity_error.as_ref() {
        return Err(error.clone());
    }
    let probed = runtime.evaluate_shadow_with_cold_bindings_probed(
        &case.damaged_surface,
        &scene,
        &groundings.grounded,
        &groundings.cold,
        false,
        &mut target_probe,
    );
    if let Some(error) = probed.integrity_error.as_ref() {
        return Err(error.clone());
    }
    let stage_telemetry_enabled = std::env::var_os("LAY_PRODUCTIVE_STAGE_TELEMETRY").is_some();
    let started = Instant::now();
    let (readout, stage_telemetry) = if stage_telemetry_enabled {
        let (readout, telemetry) = runtime.evaluate_shadow_with_cold_bindings_profiled(
            &case.damaged_surface,
            &scene,
            &groundings.grounded,
            &groundings.cold,
            false,
        );
        (readout, Some(telemetry))
    } else {
        (
            runtime.evaluate_shadow_with_cold_bindings(
                &case.damaged_surface,
                &scene,
                &groundings.grounded,
                &groundings.cold,
                false,
            ),
            None,
        )
    };
    let latency_us = started.elapsed().as_micros() as u64;
    if let Some(error) = readout.integrity_error.as_ref() {
        return Err(error.clone());
    }
    if probed != readout {
        return Err(TARGET_PROBE_PARITY_ERROR.to_string());
    }
    let base_projection = readout
        .candidates
        .iter()
        .filter(|candidate| candidate.rank_origin == CandidateRankOriginV1::BaseV64)
        .cloned()
        .collect::<Vec<_>>();
    let base_projection_exact =
        base_surface_projection_preserved_v1(&base_readout.candidates, &base_projection);
    let mut uncertified_recovered_seen = false;
    let mut demotions_without_certificate = 0_usize;
    for candidate in &readout.candidates {
        match candidate.rank_origin {
            CandidateRankOriginV1::BaseV64 if uncertified_recovered_seen => {
                demotions_without_certificate += 1;
            }
            CandidateRankOriginV1::RecoveredV66 if !candidate.cross_lane_certified => {
                uncertified_recovered_seen = true;
            }
            CandidateRankOriginV1::RecoveredV66 => {}
            CandidateRankOriginV1::BaseV64 => {}
        }
    }
    let target_lemma_born = readout.candidates.iter().any(|candidate| {
        targets.iter().any(|target| {
            candidate
                .equivalent_identities
                .iter()
                .any(|identity| identity.lemma_id == target.lemma_id)
        })
    });
    let target_slot_born = readout.candidates.iter().any(|candidate| {
        targets.iter().any(|target| {
            candidate.equivalent_identities.iter().any(|identity| {
                identity.lemma_id == target.lemma_id && identity.target_slot_id == target.slot_id
            })
        })
    });
    let target_exact_born = readout
        .candidates
        .iter()
        .any(|candidate| candidate_is_target(candidate, &targets));
    if target_probe.exact_post_surface_basin_bound != target_exact_born {
        return Err("productive target probe disagrees with final basin birth".to_string());
    }
    let target_top16 = readout
        .candidates
        .iter()
        .take(16)
        .any(|candidate| candidate_is_target(candidate, &targets));
    let target_top1 = unique_target_leader(&readout, &targets);
    let base_target_top1 = unique_target_leader(&base_readout, &targets);
    let readout_target_retained = verdict_retains_target(&readout, &targets);
    let shadow_false_singleton = matches!(
        &readout.verdict,
        ProductiveCalibratedVerdictV1::Winner { candidate, .. }
            if !readout_candidate_is_target(candidate, &targets)
    );
    let (winner, tied, abstain) = match &readout.verdict {
        ProductiveCalibratedVerdictV1::Winner { .. } => (true, false, false),
        ProductiveCalibratedVerdictV1::Tied { .. } => (false, true, false),
        ProductiveCalibratedVerdictV1::Abstain { .. } => (false, false, true),
    };

    let mut clean_scene = case.event.scene.clone();
    clean_scene.current_token = case.event.observed_surface.clone();
    clean_scene.current_normalized_scalars =
        case.event.observed_surface.chars().map(u32::from).collect();
    let clean = runtime.evaluate_shadow_with_cold_bindings(
        &case.event.observed_surface,
        &clean_scene,
        &groundings.grounded,
        &groundings.cold,
        false,
    );
    if let Some(error) = clean.integrity_error {
        return Err(error);
    }
    Ok(CaseResultV1 {
        target_lemma_id: targets
            .first()
            .map(|target| target.lemma_id)
            .unwrap_or_default(),
        target_cold_diagnostics: groundings.target_cold_diagnostics,
        oracle_applicable,
        hypothesis_covered,
        compatible_binding_retained,
        target_slot_in_binding,
        target_exact_pre_slot_bound: oracle_applicable && target_probe.exact_pre_slot_bound,
        target_exact_post_slot_bound: oracle_applicable && target_probe.exact_post_slot_bound,
        target_exact_post_surface_basin_bound: oracle_applicable
            && target_probe.exact_post_surface_basin_bound,
        target_lemma_born,
        target_slot_born,
        target_exact_born,
        target_top1,
        base_target_top1,
        target_top16,
        readout_target_retained,
        clean_target_retained: clean
            .candidates
            .iter()
            .take(16)
            .any(|candidate| candidate_is_target(candidate, &targets)),
        winner,
        tied,
        abstain,
        empty_lattice: readout.candidates.is_empty(),
        shadow_false_singleton,
        base_projection_exact,
        demotions_without_certificate,
        latency_us,
        stage_telemetry,
        candidate_surfaces: readout
            .candidates
            .iter()
            .take(8)
            .map(|candidate| candidate.normalized_surface.to_string())
            .collect(),
        candidate_diagnostics: readout
            .candidates
            .iter()
            .enumerate()
            .map(|(index, candidate)| {
                serde_json::json!({
                    "rank": index + 1,
                    "target_match": candidate_is_target(candidate, &targets),
                    "surface": candidate.normalized_surface.as_ref(),
                    "lemma_id": candidate.identity.lemma_id,
                    "paradigm_id": candidate.identity.paradigm_id,
                    "program_id": candidate.identity.program_id,
                    "target_slot_id": candidate.identity.target_slot_id,
                    "normalized_surface_id": candidate.identity.normalized_surface_id,
                    "variant_id": candidate.identity.variant_id,
                    "equivalent_identities": candidate.equivalent_identities.iter().map(|identity| serde_json::json!({
                        "lemma_id": identity.lemma_id,
                        "paradigm_id": identity.paradigm_id,
                        "program_id": identity.program_id,
                        "target_slot_id": identity.target_slot_id,
                        "normalized_surface_id": identity.normalized_surface_id,
                        "variant_id": identity.variant_id,
                    })).collect::<Vec<_>>(),
                    "score_q16": candidate.score_q16,
                    "rank_origin": format!("{:?}", candidate.rank_origin),
                    "cross_lane_certified": candidate.cross_lane_certified,
                    "provenance": format!("{:?}", candidate.provenance),
                    "minimum_independent_support": candidate.minimum_independent_support,
                    "grounded_support": candidate.grounded_support,
                    "character_distance": candidate.geometry.character_distance,
                    "keyboard_distance": candidate.geometry.keyboard_distance,
                    "equivalent_identity_count": candidate.equivalent_identity_count,
                    "equivalent_paradigm_count": candidate.equivalent_paradigm_count,
                    "minimum_equivalent_support": candidate.minimum_equivalent_support,
                    "maximum_equivalent_support": candidate.maximum_equivalent_support,
                })
            })
            .collect(),
    })
}

fn valid_targets(
    identities: &[CanonicalL2BindingIdentityV1],
    runtime: &PackagedProductiveRuntimeV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    axis_schema: &super::transition_reduce::MorphologyAxisSchemaV1,
) -> Result<Vec<ValidTargetV1>, String> {
    identities
        .iter()
        .map(|identity| {
            let labels = crate::nanda_wave::morphology_phase::canonical_feature_labels(
                identity.legacy_feature_mask,
            )?;
            let key = axis_schema.parse_feature_labels(&labels.join(":"))?;
            let slot_id = runtime.slot_id(key)?.ok_or_else(|| {
                "productive proof target slot is absent from the package".to_string()
            })?;
            let surface = canonical_l2
                .imported_surface_for_form(identity.form_ref.0)
                .ok_or_else(|| {
                    "productive proof target form has no canonical surface".to_string()
                })?;
            Ok(ValidTargetV1 {
                lemma_id: identity.lemma_ref.0,
                pos_domain: u16::from(key.pos_domain()),
                slot_id,
                surface,
            })
        })
        .collect()
}

fn build_groundings(
    identities: &[CanonicalL2BindingIdentityV1],
    masked_targets: &[CanonicalL2BindingIdentityV1],
    targets: &[ValidTargetV1],
    runtime: &PackagedProductiveRuntimeV1,
    frozen_oracle_runtime: &PackagedProductiveRuntimeV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    axis_schema: &super::transition_reduce::MorphologyAxisSchemaV1,
    frozen_entry: Option<&FrozenHypothesisEntryV1>,
    mut shared_replay_audits: Option<&mut Vec<SharedHypothesisReplayAuditV1>>,
) -> Result<ProductiveGroundingsV1, String> {
    let masked_forms = masked_targets
        .iter()
        .map(|identity| (identity.lemma_ref.0, identity.form_ref.0))
        .collect::<BTreeSet<_>>();
    let masked_slots = masked_targets
        .iter()
        .map(|identity| (identity.lemma_ref.0, identity.legacy_feature_mask))
        .collect::<BTreeSet<_>>();
    let mut output = ProductiveGroundingsV1::default();
    let mut cold_lemmas = BTreeSet::new();
    for identity in identities {
        let descriptors = runtime.grounding_descriptors(identity.lemma_ref.0)?;
        if descriptors.is_empty() {
            if !cold_lemmas.insert(identity.lemma_ref.0) {
                continue;
            }
            let target = targets
                .iter()
                .find(|target| target.lemma_id == identity.lemma_ref.0);
            let Some(observation) =
                canonical_l2.lexical_lemma_observation_v1(identity.lemma_ref.0)?
            else {
                if target.is_some() {
                    output.target_cold_diagnostics =
                        Some(TargetColdGroundingDiagnosticsV1::default());
                }
                continue;
            };
            let oracle_observed_principal_parts = observation.exact_source_forms.len();
            let target_form_observed = masked_forms.iter().any(|(lemma_id, form_ref)| {
                *lemma_id == observation.lemma_id
                    && observation
                        .exact_source_forms
                        .iter()
                        .any(|source| source.form_ref == *form_ref)
            });
            let oracle_sources = cold_sources_from_observation(&observation, runtime, axis_schema)?;
            let (oracle_bindings, oracle_diagnostics) = if target.is_some() {
                frozen_oracle_runtime.derive_cold_lemma_bindings_with_diagnostics(
                    observation.lemma_id,
                    &oracle_sources,
                )?
            } else {
                (
                    frozen_oracle_runtime
                        .derive_cold_lemma_bindings(observation.lemma_id, &oracle_sources)?,
                    ColdBindingDerivationDiagnosticsV1::default(),
                )
            };
            let measured_oracle_paradigms = oracle_bindings
                .iter()
                .filter(|binding| {
                    target.is_none_or(|target| binding.pos_domain() == target.pos_domain)
                })
                .map(ColdLemmaBindingV1::paradigm_id)
                .collect::<BTreeSet<_>>();
            let oracle_paradigms = if let Some(entry) =
                frozen_entry.filter(|entry| entry.target_lemma_id == observation.lemma_id)
            {
                let frozen = entry
                    .oracle_paradigm_ids
                    .iter()
                    .copied()
                    .collect::<BTreeSet<_>>();
                if frozen != measured_oracle_paradigms {
                    return Err(
                        "base-only oracle replay disagrees with frozen H manifest".to_string()
                    );
                }
                frozen
            } else {
                measured_oracle_paradigms
            };
            let observation_lemma_id = observation.lemma_id;
            let mut masked_observation = observation;
            masked_observation.exact_source_forms.retain(|source| {
                !masked_forms.contains(&(observation_lemma_id, source.form_ref))
                    && !masked_slots.contains(&(observation_lemma_id, source.feature_mask))
            });
            let target_blind_observed_principal_parts = masked_observation.exact_source_forms.len();
            let sources = cold_sources_from_observation(&masked_observation, runtime, axis_schema)?;
            let base_cold_bindings = frozen_oracle_runtime
                .derive_cold_lemma_bindings(masked_observation.lemma_id, &sources)?;
            let (cold_bindings, target_blind_diagnostics) = if let Some(audits) =
                shared_replay_audits.as_deref_mut()
            {
                let (bindings, diagnostics, lemma_audits) = runtime
                    .derive_cold_lemma_bindings_with_shared_replay_audit(
                        masked_observation.lemma_id,
                        &sources,
                    )?;
                audits.extend(lemma_audits);
                (bindings, diagnostics)
            } else if target.is_some() {
                runtime.derive_cold_lemma_bindings_with_anchor_trace(
                    masked_observation.lemma_id,
                    &sources,
                )?
            } else {
                (
                    runtime.derive_cold_lemma_bindings(masked_observation.lemma_id, &sources)?,
                    ColdBindingDerivationDiagnosticsV1::default(),
                )
            };
            if let Some(target) = target {
                let cold_paradigms = cold_bindings
                    .iter()
                    .filter(|binding| binding.pos_domain() == target.pos_domain)
                    .map(ColdLemmaBindingV1::paradigm_id)
                    .collect::<BTreeSet<_>>();
                let oracle_intersection = oracle_paradigms
                    .intersection(&cold_paradigms)
                    .copied()
                    .collect::<BTreeSet<_>>();
                let tracked_paradigm_count = oracle_paradigms.len();
                let tracked_pos_without_source_count = if target_blind_diagnostics
                    .source_pos_domains
                    .contains(&target.pos_domain)
                {
                    0
                } else {
                    tracked_paradigm_count
                };
                let tracked_in_postings_count = oracle_paradigms
                    .intersection(&target_blind_diagnostics.posting_paradigm_ids)
                    .count();
                let tracked_slot_compatible_count = oracle_paradigms
                    .intersection(&target_blind_diagnostics.slot_compatible_paradigm_ids)
                    .count();
                let tracked_exact_reconstructing_count = oracle_paradigms
                    .intersection(&target_blind_diagnostics.exact_reconstructing_paradigm_ids)
                    .count();
                let tracked_in_recovery_postings_count = oracle_paradigms
                    .intersection(&target_blind_diagnostics.recovery_posting_paradigm_ids)
                    .count();
                let tracked_recovered_anchor_count = oracle_paradigms
                    .intersection(&target_blind_diagnostics.recovered_anchor_paradigm_ids)
                    .count();
                let tracked_recovery_post_frontier_count = oracle_paradigms
                    .intersection(&target_blind_diagnostics.recovery_post_frontier_paradigm_ids)
                    .count();
                let tracked_recovery_exact_reconstructing_count = oracle_paradigms
                    .intersection(&target_blind_diagnostics.recovery_exact_paradigm_ids)
                    .count();
                let tracked_identity_anchor_pre_frontier_count = target_blind_diagnostics
                    .identity_anchors_pre_frontier
                    .iter()
                    .filter(|anchor| {
                        identity_anchor_matches_oracle_binding(
                            anchor,
                            &oracle_bindings,
                            target,
                            canonical_l2,
                        )
                    })
                    .count();
                let mut tracked_identity_anchor_exact_pre_frontier_count = 0;
                for anchor in &target_blind_diagnostics.identity_anchors_pre_frontier {
                    if !oracle_paradigms.contains(&anchor.paradigm_id) {
                        continue;
                    }
                    let anchor_surface = canonical_l2
                        .imported_surface_for_form(anchor.canonical_source_form_ref)
                        .ok_or_else(|| {
                            "identity-anchor trace has no canonical source surface".to_string()
                        })?;
                    tracked_identity_anchor_exact_pre_frontier_count +=
                        usize::from(runtime.identity_anchor_reconstructs_exposed_forms(
                            *anchor,
                            &anchor_surface,
                            &sources,
                        )?);
                }
                let tracked_identity_anchor_post_frontier_count = target_blind_diagnostics
                    .identity_anchors_post_frontier
                    .iter()
                    .filter(|anchor| {
                        identity_anchor_matches_oracle_binding(
                            anchor,
                            &oracle_bindings,
                            target,
                            canonical_l2,
                        )
                    })
                    .count();
                let tracked_identity_anchor_exact_count = target_blind_diagnostics
                    .identity_anchors_exact
                    .iter()
                    .filter(|anchor| {
                        identity_anchor_matches_oracle_binding(
                            anchor,
                            &oracle_bindings,
                            target,
                            canonical_l2,
                        )
                    })
                    .count();
                let mut oracle_target_slot_paradigm_count = 0;
                for binding in &oracle_bindings {
                    if binding.pos_domain() == target.pos_domain {
                        oracle_target_slot_paradigm_count +=
                            usize::from(runtime.cold_binding_has_slot(binding, target.slot_id)?);
                    }
                }
                let mut intersection_target_slot_paradigm_count = 0;
                for binding in &cold_bindings {
                    if binding.pos_domain() == target.pos_domain
                        && oracle_intersection.contains(&binding.paradigm_id())
                        && runtime.cold_binding_has_slot(binding, target.slot_id)?
                    {
                        intersection_target_slot_paradigm_count += 1;
                    }
                }
                output.target_cold_diagnostics = Some(TargetColdGroundingDiagnosticsV1 {
                    observation_found: true,
                    target_form_observed,
                    target_slot_observed: oracle_sources
                        .iter()
                        .any(|source| source.source_slot_id == target.slot_id),
                    target_pos_observed_after_mask: sources
                        .iter()
                        .any(|source| source.pos_domain == target.pos_domain),
                    oracle_observed_principal_parts,
                    target_blind_observed_principal_parts,
                    oracle: oracle_diagnostics,
                    target_blind: target_blind_diagnostics,
                    oracle_paradigm_count: oracle_paradigms.len(),
                    target_blind_paradigm_count: cold_paradigms.len(),
                    oracle_intersection_count: oracle_intersection.len(),
                    tracked_paradigm_count,
                    tracked_pos_without_source_count,
                    tracked_in_postings_count,
                    tracked_slot_compatible_count,
                    tracked_exact_reconstructing_count,
                    tracked_in_recovery_postings_count,
                    tracked_recovered_anchor_count,
                    tracked_recovery_post_frontier_count,
                    tracked_recovery_exact_reconstructing_count,
                    tracked_identity_anchor_pre_frontier_count,
                    tracked_identity_anchor_exact_pre_frontier_count,
                    tracked_identity_anchor_post_frontier_count,
                    tracked_identity_anchor_exact_count,
                    oracle_target_slot_paradigm_count,
                    intersection_target_slot_paradigm_count,
                });
            }
            output.oracle_cold.extend(oracle_bindings);
            output.base_cold.extend(base_cold_bindings);
            output.cold.extend(cold_bindings);
            continue;
        }
        for descriptor in descriptors {
            let surface = canonical_l2
                .imported_surface_for_form(descriptor.canonical_source_form_ref)
                .ok_or_else(|| {
                    "productive packaged grounding has no canonical source surface".to_string()
                })?;
            output.grounded.push(PackagedGroundedLemmaV1 {
                lemma_id: descriptor.lemma_id,
                pos_domain: descriptor.pos_domain,
                canonical_source_form_ref: descriptor.canonical_source_form_ref,
                source_slot_id: descriptor.source_slot_id,
                normalized_source: surface,
                grounded_support: descriptor.grounded_support,
            });
        }
    }
    output.grounded.sort_by(|left, right| {
        (left.lemma_id, left.pos_domain, left.source_slot_id).cmp(&(
            right.lemma_id,
            right.pos_domain,
            right.source_slot_id,
        ))
    });
    output.grounded.dedup_by(|left, right| {
        (left.lemma_id, left.pos_domain) == (right.lemma_id, right.pos_domain)
    });
    Ok(output)
}

fn identity_anchor_matches_oracle_binding(
    anchor: &RecoveryIdentityAnchorRefV1,
    oracle_bindings: &[ColdLemmaBindingV1],
    target: &ValidTargetV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
) -> bool {
    let Some(anchor_surface) =
        canonical_l2.imported_surface_for_form(anchor.canonical_source_form_ref)
    else {
        return false;
    };
    oracle_bindings.iter().any(|binding| {
        binding.pos_domain() == target.pos_domain
            && binding.paradigm_id() == anchor.paradigm_id
            && binding.source_slot_id() == anchor.source_slot_id
            && binding.normalized_source() == anchor_surface
    })
}

fn cold_sources_from_observation(
    observation: &super::super::runtime::LexicalLemmaObservationV1,
    runtime: &PackagedProductiveRuntimeV1,
    axis_schema: &super::transition_reduce::MorphologyAxisSchemaV1,
) -> Result<Vec<ColdLemmaSourceV1>, String> {
    let canonical_source_form_ref = observation
        .exact_source_forms
        .first()
        .map(|source| source.form_ref);
    let mut sources = Vec::with_capacity(observation.exact_source_forms.len());
    for source in &observation.exact_source_forms {
        let labels =
            crate::nanda_wave::morphology_phase::canonical_feature_labels(source.feature_mask)?;
        let key = axis_schema.parse_feature_labels(&labels.join(":"))?;
        let Some(source_slot_id) = runtime.slot_id(key)? else {
            continue;
        };
        sources.push(ColdLemmaSourceV1 {
            pos_domain: u16::from(key.pos_domain()),
            canonical_source_form_ref: source.form_ref,
            source_slot_id,
            normalized_source: source.normalized_surface.clone(),
            grounded_support: u32::from(source.support.max(1)),
            canonical_preference: source.canonical_preference,
            canonical_source: Some(source.form_ref) == canonical_source_form_ref,
        });
    }
    Ok(sources)
}

fn candidate_is_target(
    candidate: &PackagedProductiveCandidateV1,
    targets: &[ValidTargetV1],
) -> bool {
    targets.iter().any(|target| {
        candidate.equivalent_identities.iter().any(|identity| {
            identity.lemma_id == target.lemma_id && identity.target_slot_id == target.slot_id
        }) && candidate.normalized_surface.as_ref() == target.surface
    })
}

fn unique_target_leader(readout: &PackagedProductiveReadoutV1, targets: &[ValidTargetV1]) -> bool {
    let Some(leader) = readout.candidates.first() else {
        return false;
    };
    candidate_is_target(leader, targets)
        && readout
            .candidates
            .iter()
            .skip(1)
            .find(|candidate| candidate.rank_origin == leader.rank_origin)
            .is_none_or(|second| leader.score_q16 > second.score_q16)
}

fn readout_candidate_is_target(
    candidate: &super::calibrate::ReadoutCandidateV1,
    targets: &[ValidTargetV1],
) -> bool {
    targets.iter().any(|target| {
        candidate.equivalent_identities.iter().any(|identity| {
            identity.lemma_id == target.lemma_id && identity.target_slot_id == target.slot_id
        }) && candidate.normalized_surface.as_ref() == target.surface
    })
}

fn verdict_retains_target(
    readout: &PackagedProductiveReadoutV1,
    targets: &[ValidTargetV1],
) -> bool {
    match &readout.verdict {
        ProductiveCalibratedVerdictV1::Winner { candidate, .. } => {
            readout_candidate_is_target(candidate, targets)
        }
        ProductiveCalibratedVerdictV1::Tied { candidates, .. } => candidates
            .iter()
            .any(|candidate| readout_candidate_is_target(candidate, targets)),
        ProductiveCalibratedVerdictV1::Abstain { suggestions, .. } => suggestions
            .iter()
            .any(|candidate| readout_candidate_is_target(candidate, targets)),
    }
}

fn damage_key(identity: &[u8; 32], damage: &DamageExample) -> [u8; 32] {
    damage_identity(identity, damage.class, &damage.surface)
}

fn damage_identity(identity: &[u8; 32], class: &str, surface: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"lay-productive-v1-fixed-damage\0");
    hasher.update(identity);
    hasher.update(class.as_bytes());
    hasher.update([0]);
    hasher.update(surface.as_bytes());
    hasher.finalize().into()
}

fn sha256_file(path: &Path) -> io::Result<[u8; 32]> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().into())
}

fn hex_sha256(value: [u8; 32]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn percent(value: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        value as f64 * 100.0 / total as f64
    }
}

fn percentile(sorted: &[u64], percentile: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let rank = (sorted.len() * percentile).div_ceil(100).max(1);
    sorted[rank.saturating_sub(1).min(sorted.len() - 1)]
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
