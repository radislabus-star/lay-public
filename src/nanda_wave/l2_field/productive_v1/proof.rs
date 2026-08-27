use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};
use std::io::{self, Read};
use std::path::Path;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::boundary_birth::enumerate_typed_boundary_births_from_packages;
use super::calibrate::{CandidateRankOriginV1, ProductiveCalibratedVerdictV1};
use super::candidate_state::{
    derive_candidate_validity_shadow, derive_original_preservation_shadow,
    TargetNamespaceSettlementV1, WitnessFrameAssessmentV1,
};
use super::cohort_compare::{
    CohortCompareStatusV1, CohortFirstDivergenceV1, LexicalVerdictObservationV1,
};
use super::conflict_cohort::derive_conflict_cohort_shadow;
use super::contour_birth::{
    enumerate_typed_contour_births, enumerate_typed_contour_births_with_l11,
};
use super::corpus::load_axis_schema;
use super::events::{
    decode_verified_spool_record, TypedProductiveEventV1, VerifiedSpoolShardReaderV1,
};
use super::material_frame::{
    bind_exact_frame_target, prepare_context_neutral_productive_material_with_contours,
    prepare_context_neutral_productive_material_with_contours_and_boundaries, ExactInputFrameV1,
    ExactPackageTupleV1, PreparedMaterialLeaseArenaV1, FROZEN_V90_ENUMERATION_WORK_BUDGET,
};
use super::packaged_runtime::{
    base_surface_projection_preserved_v1, ColdBindingDerivationDiagnosticsV1, ColdLemmaBindingV1,
    ColdLemmaSourceV1, ContextNeutralProductiveEnumerationV1, PackagedGroundedLemmaV1,
    PackagedProductiveCandidateV1, PackagedProductiveReadoutV1, PackagedProductiveRuntimeV1,
    ProductiveEvaluationTelemetryV1, ProductiveTargetProbeIdentityV1, ProductiveTargetProbeV1,
    RecoveryIdentityAnchorRefV1, SharedHypothesisReplayAuditV1,
};
use super::types::CanonicalL2BindingIdentityV1;
use crate::nanda_wave::l2_field::runtime::{L2FieldAuthority, L2FieldAvailability};
use crate::nanda_wave::lexical_grokking::{split_damages, DamageExample, ExactL11SurfaceIndexV1};
use crate::typing_transition::target_evidence::{
    CandidateStateV1, CohortVerdictV1, EnumerationStateV1,
    EnumerationWorkCountersV1 as MaterialWorkCountersV1, FrameOriginalPreservationVerdictV1,
    IncompletenessReasonV1, LeaseConsumerStateV1, TargetRelationV1, VerdictMembershipV1,
};

const EXPECTED_DAMAGE_CLASSES: usize = 13;
const TARGET_PROBE_PARITY_ERROR: &str = "productive target probe changed runtime readout";
const FROZEN_MANIFEST_SCHEMA_VERSION: u16 = 1;
const FROZEN_MANIFEST_HELDOUT_PER_CLASS: usize = 100;
const FROZEN_MANIFEST_ENTRY_COUNT: usize = 1_300;
const FROZEN_MANIFEST_H_COUNT: usize = 1_280;
const FROZEN_V64_PACKAGE_SHA256: &str =
    "9fd8c950398fb8ba47a2c9f2236880239d9f4376b191a691b0d01c47ddd3e438";
const FROZEN_V64_L11_PACKAGE_SHA256: &str =
    "47fa757acac03b0f76e5397e965b9127884e245e9845ce0f1ca0896fb40f33e9";
const V9_BOUND_V90_PACKAGE_SHA256: &str =
    "40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44";
const ACTIVE_V9_L11_PACKAGE_SHA256: &str =
    "bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7";
const FROZEN_CANONICAL_L2_PACKAGE_SHA256: &str =
    "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b";
const FROZEN_AXIS_SCHEMA_SHA256: &str =
    "b5b24f952e83e1e9738db0f89a9d2e9e16eaf7af754990114a562d42be3c060b";
const FROZEN_PROOF_SPOOL_SHA256: &str =
    "6e282474b26bf90dc61ee21c93c9dd7dd727c29a2b02650c513ffdd06746e807";
const FROZEN_PROOF_SPOOL_BYTES: u64 = 1_154_794_811;
const FROZEN_MANIFEST_PAYLOAD_SHA256: &str =
    "2f54844d7f7900734049d2ed2ae53150eead60da3223c0efc4256ba804b7f89b";
const FROZEN_DAMAGE_GENERATOR_ID: &str = "lexical_grokking::split_damages:v1";
const FROZEN_MANIFEST_FILE: &str = "frozen-v64-hypothesis-manifest-v1.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FrozenProofGenerationV1 {
    FrozenV64ManifestSource,
    ActiveL11V9,
}

impl FrozenProofGenerationV1 {
    const fn name(self) -> &'static str {
        match self {
            Self::FrozenV64ManifestSource => "frozen_v64_manifest_source",
            Self::ActiveL11V9 => "active_l11_v9",
        }
    }

    const fn permits_manifest_creation(self) -> bool {
        matches!(self, Self::FrozenV64ManifestSource)
    }
}

#[derive(Clone, Copy, Debug)]
struct FrozenProofGenerationBindingV1 {
    generation: FrozenProofGenerationV1,
    productive_package_sha256: &'static str,
    l11_package_sha256: &'static str,
}

const FROZEN_PROOF_GENERATION_BINDINGS: [FrozenProofGenerationBindingV1; 2] = [
    FrozenProofGenerationBindingV1 {
        generation: FrozenProofGenerationV1::FrozenV64ManifestSource,
        productive_package_sha256: FROZEN_V64_PACKAGE_SHA256,
        l11_package_sha256: FROZEN_V64_L11_PACKAGE_SHA256,
    },
    FrozenProofGenerationBindingV1 {
        generation: FrozenProofGenerationV1::ActiveL11V9,
        productive_package_sha256: V9_BOUND_V90_PACKAGE_SHA256,
        l11_package_sha256: ACTIVE_V9_L11_PACKAGE_SHA256,
    },
];

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
    generation: FrozenProofGenerationV1,
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
    enumeration_work_cases: Vec<EnumerationWorkCaseV1>,
    enumeration_work_errors: Vec<String>,
    material_frame_cases: Vec<MaterialFrameCaseV1>,
    material_frame_errors: Vec<String>,
    live_cohort_compare_cases: Vec<LiveCohortCompareCaseV1>,
    live_cohort_no_field_cases: Vec<LiveCohortNoFieldCaseV1>,
    live_cohort_compare_errors: Vec<LiveCohortCompareErrorV1>,
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
        self.enumeration_work_cases
            .extend(other.enumeration_work_cases);
        self.enumeration_work_errors
            .extend(other.enumeration_work_errors);
        self.material_frame_cases.extend(other.material_frame_cases);
        self.material_frame_errors
            .extend(other.material_frame_errors);
        self.live_cohort_compare_cases
            .extend(other.live_cohort_compare_cases);
        self.live_cohort_no_field_cases
            .extend(other.live_cohort_no_field_cases);
        self.live_cohort_compare_errors
            .extend(other.live_cohort_compare_errors);
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
    relation_replay_count: u64,
    operator_step_count: u64,
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
        self.relation_replay_count += telemetry.relation_replay_count;
        self.operator_step_count += telemetry.operator_step_count;
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
        self.relation_replay_count += other.relation_replay_count;
        self.operator_step_count += other.operator_step_count;
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
    posting_visit_count: u64,
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
    operator_step_count: u64,
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
        self.posting_visit_count += diagnostics.posting_visit_count as u64;
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
        self.operator_step_count += diagnostics.operator_step_count;
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
        self.posting_visit_count += other.posting_visit_count;
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
        self.operator_step_count += other.operator_step_count;
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
struct EnumerationWorkCountersV1 {
    posting_visits: u64,
    relation_replays: u64,
    grounding_lookups: u64,
    generated_logical_targets: u64,
    operator_steps: u64,
}

impl EnumerationWorkCountersV1 {
    fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            posting_visits: self.posting_visits.checked_add(other.posting_visits)?,
            relation_replays: self.relation_replays.checked_add(other.relation_replays)?,
            grounding_lookups: self
                .grounding_lookups
                .checked_add(other.grounding_lookups)?,
            generated_logical_targets: self
                .generated_logical_targets
                .checked_add(other.generated_logical_targets)?,
            operator_steps: self.operator_steps.checked_add(other.operator_steps)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct EnumerationWorkCaseV1 {
    proof_identity: [u8; 32],
    damage_identity: [u8; 32],
    damage_class: &'static str,
    canonical_grounding: EnumerationWorkCountersV1,
    cold_binding: EnumerationWorkCountersV1,
    productive_traversal: EnumerationWorkCountersV1,
    aggregate: EnumerationWorkCountersV1,
}

impl EnumerationWorkCaseV1 {
    fn new(
        case: &ProofCaseV1,
        grounding_lookups: u64,
        diagnostics: &ColdBindingDerivationDiagnosticsV1,
        telemetry: ProductiveEvaluationTelemetryV1,
    ) -> Result<Self, String> {
        let canonical_grounding = EnumerationWorkCountersV1 {
            grounding_lookups,
            ..EnumerationWorkCountersV1::default()
        };
        let cold_binding = EnumerationWorkCountersV1 {
            posting_visits: diagnostics.posting_visit_count as u64,
            relation_replays: (diagnostics.recovery_program_execution_count as u64)
                .checked_add(diagnostics.exact_replay_program_execution_count as u64)
                .ok_or_else(|| "cold-binding replay count overflow".to_string())?,
            operator_steps: diagnostics.operator_step_count,
            ..EnumerationWorkCountersV1::default()
        };
        let productive_traversal = EnumerationWorkCountersV1 {
            relation_replays: telemetry.relation_replay_count,
            generated_logical_targets: telemetry.logical_terminal_count,
            operator_steps: telemetry.operator_step_count,
            ..EnumerationWorkCountersV1::default()
        };
        let aggregate = canonical_grounding
            .checked_add(cold_binding)
            .and_then(|value| value.checked_add(productive_traversal))
            .ok_or_else(|| "aggregate enumeration work count overflow".to_string())?;
        Ok(Self {
            proof_identity: case.event.proof_identity,
            damage_identity: damage_identity(
                &case.event.proof_identity,
                case.class,
                &case.damaged_surface,
            ),
            damage_class: case.class,
            canonical_grounding,
            cold_binding,
            productive_traversal,
            aggregate,
        })
    }
}

#[derive(Clone, Debug)]
struct MaterialFrameCaseV1 {
    proof_identity: [u8; 32],
    damage_identity: [u8; 32],
    damage_class: &'static str,
    hypothesis_covered: bool,
    target_member: bool,
    complete_material: bool,
    explicit_incompleteness: bool,
    incompleteness_reason: IncompletenessReasonV1,
    work_budget_respected: bool,
    contour_births: usize,
    contour_target_members: usize,
    contour_overflow: bool,
    contour_work: EnumerationWorkCountersV1,
    boundary_births: usize,
    boundary_overflow: bool,
    boundary_work: EnumerationWorkCountersV1,
    bindable_target: bool,
    context_count: usize,
    context_bindings: usize,
    context_material_digest_exact: bool,
    stale_reuse_attempts: usize,
    stale_reuse_accepts: usize,
    candidate_state_derivations: usize,
    candidate_born: usize,
    candidate_grounded: usize,
    candidate_rejected: usize,
    candidate_false_grounding: usize,
    candidate_cross_context_mismatches: usize,
    stale_candidate_state_accepts: usize,
    original_preserve: usize,
    original_replace_permitted: usize,
    original_unresolved: usize,
    cohort_derivations: usize,
    cohort_context_mismatches: usize,
    cohort_winners: usize,
    cohort_ties: usize,
    cohort_abstains: usize,
    cohort_incomplete_winners: usize,
    cohort_false_singletons: usize,
    cohort_lost_grounded_targets: usize,
    cohort_multiple_component_authority: usize,
    cohort_preservation_bypass: usize,
}

fn summarize_material_frame_shadow(
    mut cases: Vec<MaterialFrameCaseV1>,
    errors: Vec<String>,
    expected_samples: usize,
    enabled: bool,
) -> serde_json::Value {
    cases.sort_unstable_by(|left, right| {
        (left.proof_identity, left.damage_identity)
            .cmp(&(right.proof_identity, right.damage_identity))
    });
    let unique_samples = cases
        .iter()
        .map(|case| (case.proof_identity, case.damage_identity))
        .collect::<BTreeSet<_>>()
        .len();
    let hypothesis_cases = cases.iter().filter(|case| case.hypothesis_covered).count();
    let hypothesis_target_members = cases
        .iter()
        .filter(|case| case.hypothesis_covered && case.target_member)
        .count();
    let complete_material = cases.iter().filter(|case| case.complete_material).count();
    let explicit_incompleteness = cases
        .iter()
        .filter(|case| case.explicit_incompleteness)
        .count();
    let failed_material = cases
        .iter()
        .filter(|case| {
            (!case.complete_material && !case.explicit_incompleteness)
                || case.incompleteness_reason == IncompletenessReasonV1::IntegrityFailure
        })
        .count();
    let incompleteness_reasons = cases.iter().fold(
        BTreeMap::<&'static str, usize>::new(),
        |mut reasons, case| {
            let reason = match case.incompleteness_reason {
                IncompletenessReasonV1::None => "NONE",
                IncompletenessReasonV1::StorageCapacity => "STORAGE_CAPACITY",
                IncompletenessReasonV1::WorkBudgetExceeded => "WORK_BUDGET_EXCEEDED",
                IncompletenessReasonV1::UpstreamIncomplete => "UPSTREAM_INCOMPLETE",
                IncompletenessReasonV1::IntegrityFailure => "INTEGRITY_FAILURE",
            };
            *reasons.entry(reason).or_default() += 1;
            reasons
        },
    );
    let budget_respected = cases
        .iter()
        .filter(|case| case.work_budget_respected)
        .count();
    let contour_births = cases.iter().map(|case| case.contour_births).sum::<usize>();
    let contour_target_members = cases
        .iter()
        .map(|case| case.contour_target_members)
        .sum::<usize>();
    let contour_overflows = cases.iter().filter(|case| case.contour_overflow).count();
    let contour_work = work_counter_summary(cases.iter().map(|case| case.contour_work));
    let boundary_births = cases.iter().map(|case| case.boundary_births).sum::<usize>();
    let boundary_overflows = cases.iter().filter(|case| case.boundary_overflow).count();
    let boundary_work = work_counter_summary(cases.iter().map(|case| case.boundary_work));
    let context_comparisons = cases.iter().map(|case| case.context_count).sum::<usize>();
    let context_bindings = cases
        .iter()
        .map(|case| case.context_bindings)
        .sum::<usize>();
    let expected_context_bindings = cases
        .iter()
        .filter(|case| case.bindable_target)
        .map(|case| case.context_count)
        .sum::<usize>();
    let context_digest_failures = cases
        .iter()
        .filter(|case| !case.context_material_digest_exact)
        .count();
    let stale_reuse_attempts = cases
        .iter()
        .map(|case| case.stale_reuse_attempts)
        .sum::<usize>();
    let stale_reuse_accepts = cases
        .iter()
        .map(|case| case.stale_reuse_accepts)
        .sum::<usize>();
    let candidate_state_derivations = cases
        .iter()
        .map(|case| case.candidate_state_derivations)
        .sum::<usize>();
    let candidate_born = cases.iter().map(|case| case.candidate_born).sum::<usize>();
    let candidate_grounded = cases
        .iter()
        .map(|case| case.candidate_grounded)
        .sum::<usize>();
    let candidate_rejected = cases
        .iter()
        .map(|case| case.candidate_rejected)
        .sum::<usize>();
    let candidate_false_grounding = cases
        .iter()
        .map(|case| case.candidate_false_grounding)
        .sum::<usize>();
    let candidate_cross_context_mismatches = cases
        .iter()
        .map(|case| case.candidate_cross_context_mismatches)
        .sum::<usize>();
    let stale_candidate_state_accepts = cases
        .iter()
        .map(|case| case.stale_candidate_state_accepts)
        .sum::<usize>();
    let original_preserve = cases
        .iter()
        .map(|case| case.original_preserve)
        .sum::<usize>();
    let original_replace_permitted = cases
        .iter()
        .map(|case| case.original_replace_permitted)
        .sum::<usize>();
    let original_unresolved = cases
        .iter()
        .map(|case| case.original_unresolved)
        .sum::<usize>();
    let cohort_derivations = cases
        .iter()
        .map(|case| case.cohort_derivations)
        .sum::<usize>();
    let cohort_context_mismatches = cases
        .iter()
        .map(|case| case.cohort_context_mismatches)
        .sum::<usize>();
    let cohort_winners = cases.iter().map(|case| case.cohort_winners).sum::<usize>();
    let cohort_ties = cases.iter().map(|case| case.cohort_ties).sum::<usize>();
    let cohort_abstains = cases.iter().map(|case| case.cohort_abstains).sum::<usize>();
    let cohort_incomplete_winners = cases
        .iter()
        .map(|case| case.cohort_incomplete_winners)
        .sum::<usize>();
    let cohort_false_singletons = cases
        .iter()
        .map(|case| case.cohort_false_singletons)
        .sum::<usize>();
    let cohort_lost_grounded_targets = cases
        .iter()
        .map(|case| case.cohort_lost_grounded_targets)
        .sum::<usize>();
    let cohort_multiple_component_authority = cases
        .iter()
        .map(|case| case.cohort_multiple_component_authority)
        .sum::<usize>();
    let cohort_preservation_bypass = cases
        .iter()
        .map(|case| case.cohort_preservation_bypass)
        .sum::<usize>();
    let by_class = cases.iter().fold(
        BTreeMap::<&'static str, (usize, usize, usize)>::new(),
        |mut classes, case| {
            let metrics = classes.entry(case.damage_class).or_default();
            metrics.0 += 1;
            metrics.1 += usize::from(case.hypothesis_covered);
            metrics.2 += usize::from(case.hypothesis_covered && case.target_member);
            classes
        },
    );
    let by_class = by_class
        .into_iter()
        .map(|(class, (class_cases, class_h, class_members))| {
            (
                class,
                serde_json::json!({
                    "cases": class_cases,
                    "hypothesis_cases": class_h,
                    "hypothesis_target_members": class_members,
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let pass = enabled
        && cases.len() == expected_samples
        && unique_samples == expected_samples
        && hypothesis_target_members == hypothesis_cases
        && complete_material.saturating_add(explicit_incompleteness) == expected_samples
        && failed_material == 0
        && budget_respected == expected_samples
        && context_comparisons >= expected_samples.saturating_mul(3)
        && context_bindings == expected_context_bindings
        && context_digest_failures == 0
        && stale_reuse_attempts == expected_context_bindings
        && stale_reuse_accepts == 0
        && candidate_state_derivations == expected_context_bindings
        && candidate_born
            .saturating_add(candidate_grounded)
            .saturating_add(candidate_rejected)
            == candidate_state_derivations
        && candidate_false_grounding == 0
        && candidate_cross_context_mismatches == 0
        && stale_candidate_state_accepts == 0
        && original_preserve
            .saturating_add(original_replace_permitted)
            .saturating_add(original_unresolved)
            == context_comparisons
        && cohort_derivations == context_comparisons
        && cohort_winners
            .saturating_add(cohort_ties)
            .saturating_add(cohort_abstains)
            == cohort_derivations
        && cohort_context_mismatches == 0
        && cohort_incomplete_winners == 0
        && cohort_false_singletons == 0
        && cohort_lost_grounded_targets == 0
        && cohort_multiple_component_authority == 0
        && cohort_preservation_bypass == 0
        && errors.is_empty();
    serde_json::json!({
        "schema": "lay.context-neutral-material-frame-shadow.v1",
        "enabled": enabled,
        "verdict": if pass && explicit_incompleteness == 0 {
            "PASS_SLICE2_SHADOW_SOFTWARE_COMPLETE"
        } else if pass {
            "PASS_SLICE2_SHADOW_FAIL_CLOSED_INCOMPLETENESS_RECORDED"
        } else {
            "FAIL_OR_UNACCOUNTED_SLICE2_SHADOW"
        },
        "pass": pass,
        "expected_material_pairs": expected_samples,
        "material_pairs": cases.len(),
        "unique_material_pairs": unique_samples,
        "hypothesis_cases": hypothesis_cases,
        "hypothesis_target_members": hypothesis_target_members,
        "complete_material": complete_material,
        "explicit_incompleteness": explicit_incompleteness,
        "failed_material": failed_material,
        "incompleteness_reasons": incompleteness_reasons,
        "work_budget_respected": budget_respected,
        "contour_birth": {
            "births": contour_births,
            "target_members": contour_target_members,
            "overflows": contour_overflows,
            "work": contour_work,
        },
        "boundary_birth": {
            "births": boundary_births,
            "overflows": boundary_overflows,
            "work": boundary_work,
        },
        "contexts_per_pair_minimum": 3,
        "context_comparisons": context_comparisons,
        "context_bindings": context_bindings,
        "expected_context_bindings": expected_context_bindings,
        "context_material_digest_failures": context_digest_failures,
        "stale_reuse_attempts": stale_reuse_attempts,
        "stale_reuse_accepts": stale_reuse_accepts,
        "candidate_state": {
            "derivations": candidate_state_derivations,
            "born": candidate_born,
            "grounded": candidate_grounded,
            "rejected": candidate_rejected,
            "false_grounding": candidate_false_grounding,
            "cross_context_mismatches": candidate_cross_context_mismatches,
            "stale_frame_accepts": stale_candidate_state_accepts,
        },
        "original_preservation": {
            "preserve": original_preserve,
            "replace_permitted": original_replace_permitted,
            "unresolved": original_unresolved,
            "outside_target_set": true,
        },
        "conflict_cohort": {
            "derivations": cohort_derivations,
            "context_mismatches": cohort_context_mismatches,
            "winner": cohort_winners,
            "tied": cohort_ties,
            "abstain": cohort_abstains,
            "incomplete_winners": cohort_incomplete_winners,
            "false_singletons": cohort_false_singletons,
            "lost_grounded_targets": cohort_lost_grounded_targets,
            "multiple_component_authority": cohort_multiple_component_authority,
            "preservation_bypass": cohort_preservation_bypass,
        },
        "errors": errors,
        "by_damage_class": by_class,
        "runtime_authority_changed": false,
    })
}

fn summarize_slice5_contour_birth_shadow(
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    exact_l11: Option<&ExactL11SurfaceIndexV1>,
    package_tuple: ExactPackageTupleV1,
    enabled: bool,
) -> serde_json::Value {
    const CASES: [(&str, &str, &str); 8] = [
        ("active-ru-layout-known-ascii", "зва", "pdf"),
        ("duplicate-layout-prefix", "fавтозамена", "автозамена"),
        ("missing-layout-initial", "dnjpfvtyf", "автозамена"),
        ("dual-layout-boundary", "вщцутдщфв", "download"),
        ("accidental-final-consonant", "читайл", "читай"),
        ("local-compound-edit", "автозаменет", "автозамена"),
        ("short-layout-function-word", "yt", "не"),
        ("sequential-short-layout-word", "yt", "не"),
    ];

    let mut rows = Vec::with_capacity(CASES.len());
    let mut birth_hits = 0_usize;
    let mut retention_hits = 0_usize;
    let mut born_only_hits = 0_usize;
    let mut authority_grants = 0_usize;
    let mut overflows = 0_usize;
    let mut work_budget_hits = 0_usize;
    for (case_id, observed, target) in CASES {
        let contours = exact_l11.map_or_else(
            || enumerate_typed_contour_births(observed, canonical_l2),
            |l11| enumerate_typed_contour_births_with_l11(observed, canonical_l2, l11),
        );
        let birth_hit = contours
            .births
            .iter()
            .any(|birth| birth.normalized_surface == target);
        let birth_count = contours.births.len();
        let contour_work = contours.work;
        let contour_overflow = contours.overflow_reason.is_some();
        let work_within_budget = contours.work_within_budget();
        let material = prepare_context_neutral_productive_material_with_contours(
            observed,
            package_tuple,
            ContextNeutralProductiveEnumerationV1 {
                readout: PackagedProductiveReadoutV1 {
                    verdict: ProductiveCalibratedVerdictV1::Abstain {
                        suggestions: Vec::new(),
                        productive_overflow: false,
                    },
                    candidates: Vec::new(),
                    logical_terminal_count: 0,
                    logical_surface_basin_count: 0,
                    integrity_error: None,
                },
                productive_work: MaterialWorkCountersV1::default(),
                aggregate_work: MaterialWorkCountersV1::default(),
                work_budget_exceeded: false,
            },
            contours,
        );
        let (retained, born_only, authority_granted, material_state, material_reason) =
            match material {
                Ok(material) => {
                    let target_ref = material
                        .exact_target_surfaces()
                        .position(|surface| surface == target);
                    let witnesses = target_ref
                        .and_then(|index| material.compact().targets.as_slice().get(index))
                        .map(|target| target.witnesses.witnesses())
                        .unwrap_or_default();
                    let born_only = !witnesses.is_empty()
                        && witnesses
                            .iter()
                            .all(|witness| witness.verdict_membership == VerdictMembershipV1::Born);
                    let authority_granted = witnesses.iter().any(|witness| {
                        matches!(
                            witness.verdict_membership,
                            VerdictMembershipV1::Grounded
                                | VerdictMembershipV1::L11Winner
                                | VerdictMembershipV1::L11Tied
                        )
                    });
                    (
                        target_ref.is_some(),
                        born_only,
                        authority_granted,
                        material.completeness().state(),
                        material.completeness().reason(),
                    )
                }
                Err(_) => (
                    false,
                    false,
                    false,
                    EnumerationStateV1::Failed,
                    IncompletenessReasonV1::IntegrityFailure,
                ),
            };
        birth_hits += usize::from(birth_hit);
        retention_hits += usize::from(retained);
        born_only_hits += usize::from(born_only);
        authority_grants += usize::from(authority_granted);
        overflows += usize::from(contour_overflow);
        work_budget_hits += usize::from(work_within_budget);
        rows.push(serde_json::json!({
            "case_id": case_id,
            "observed": observed,
            "target": target,
            "birth_hit": birth_hit,
            "retained": retained,
            "born_only": born_only,
            "authority_granted": authority_granted,
            "birth_count": birth_count,
            "contour_overflow": contour_overflow,
            "material_state": format!("{material_state:?}"),
            "material_reason": format!("{material_reason:?}"),
            "work": {
                "grounding_lookups": contour_work.grounding_lookups,
                "operator_steps": contour_work.operator_steps,
            },
            "work_within_budget": work_within_budget,
        }));
    }
    let pass = enabled
        && birth_hits == CASES.len()
        && retention_hits == CASES.len()
        && born_only_hits == CASES.len()
        && authority_grants == 0
        && overflows == 0
        && work_budget_hits == CASES.len();
    serde_json::json!({
        "schema": "lay.slice5-contour-birth-shadow.v1",
        "enabled": enabled,
        "verdict": if pass { "PASS_SLICE5_FIXED_BIRTH_RETENTION_SHADOW" } else { "FAIL_SLICE5_FIXED_BIRTH_RETENTION_SHADOW" },
        "pass": pass,
        "birth_denominator": CASES.len(),
        "birth_hits": birth_hits,
        "retention_hits": retention_hits,
        "born_only_hits": born_only_hits,
        "authority_denominator": CASES.len(),
        "authority_grants": authority_grants,
        "overflows": overflows,
        "work_budget_respected": work_budget_hits,
        "cases": rows,
        "runtime_authority_changed": false,
    })
}

fn summarize_slice6_boundary_internalization_shadow(
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    exact_l11: Option<&ExactL11SurfaceIndexV1>,
    package_tuple: ExactPackageTupleV1,
    enabled: bool,
) -> serde_json::Value {
    const MAX_SCAN_FORMS: usize = 100_000;
    let mut scanned_forms = 0_usize;
    let mut short_parts = Vec::<String>::new();
    let mut merge_case = None::<(String, String)>;
    let mut false_split_case = None::<String>;
    let mut overflow_case = None::<String>;

    if enabled {
        for form_ref in 0..canonical_l2.form_count().min(MAX_SCAN_FORMS) {
            let Some(surface) = canonical_l2.decode_form_ref(form_ref as u32) else {
                continue;
            };
            let surface = surface.into_owned().to_lowercase();
            let scalar_count = surface.chars().count();
            if !(2..=32).contains(&scalar_count) || !surface.chars().all(char::is_alphabetic) {
                continue;
            }
            scanned_forms += 1;
            if scalar_count <= 8 && short_parts.len() < 24 && !short_parts.contains(&surface) {
                short_parts.push(surface.clone());
            }
            let births =
                enumerate_typed_boundary_births_from_packages(&surface, canonical_l2, exact_l11);
            if false_split_case.is_none() && births.births.is_empty() {
                false_split_case = Some(surface.clone());
            }
            if merge_case.is_none() {
                merge_case = births
                    .births
                    .iter()
                    .find(|birth| birth.relation == TargetRelationV1::BoundarySplit)
                    .map(|birth| (birth.normalized_surface.clone(), surface.clone()));
            }
            if overflow_case.is_none() && births.logical_match_count > 2 {
                overflow_case = Some(surface);
            }
            if merge_case.is_some()
                && false_split_case.is_some()
                && overflow_case.is_some()
                && short_parts.len() >= 8
            {
                break;
            }
        }
    }

    let split_case = short_parts
        .iter()
        .enumerate()
        .find_map(|(left_index, left)| {
            short_parts.iter().skip(left_index + 1).find_map(|right| {
                let observed = format!("{left}{right}");
                if observed.chars().count() > 32 {
                    return None;
                }
                let expected = format!("{left} {right}");
                let births = enumerate_typed_boundary_births_from_packages(
                    &observed,
                    canonical_l2,
                    exact_l11,
                );
                (births.logical_match_count <= 2
                    && births.births.iter().any(|birth| {
                        birth.relation == TargetRelationV1::BoundarySplit
                            && birth.normalized_surface == expected
                    }))
                .then_some((observed, expected))
            })
        });

    let split = split_case.as_ref().map(|(observed, expected)| {
        evaluate_fixed_boundary_case(
            observed,
            expected,
            TargetRelationV1::BoundarySplit,
            canonical_l2,
            exact_l11,
            package_tuple,
            false,
        )
    });
    let merge = merge_case.as_ref().map(|(observed, expected)| {
        evaluate_fixed_boundary_case(
            observed,
            expected,
            TargetRelationV1::BoundaryMerge,
            canonical_l2,
            exact_l11,
            package_tuple,
            false,
        )
    });
    let overflow = overflow_case.as_ref().map(|observed| {
        evaluate_fixed_boundary_case(
            observed,
            "",
            TargetRelationV1::BoundarySplit,
            canonical_l2,
            exact_l11,
            package_tuple,
            true,
        )
    });
    let false_split_births = false_split_case.as_ref().map(|observed| {
        enumerate_typed_boundary_births_from_packages(observed, canonical_l2, exact_l11)
    });
    let false_split_pass = false_split_births
        .as_ref()
        .is_some_and(|births| births.births.is_empty() && births.overflow_reason.is_none());
    let legacy_split = split_case
        .as_ref()
        .and_then(|(observed, _)| crate::phrase_reader::correct_glued_russian_phrase(observed));
    let legacy_split_exact = split_case
        .as_ref()
        .is_some_and(|(_, expected)| legacy_split.as_deref() == Some(expected.as_str()));
    let legacy_merge = merge_case
        .as_ref()
        .and_then(|(observed, _)| crate::phrase_reader::correct_glued_russian_phrase(observed));
    let legacy_merge_exact = merge_case
        .as_ref()
        .is_some_and(|(_, expected)| legacy_merge.as_deref() == Some(expected.as_str()));
    let pass = enabled
        && split.as_ref().is_some_and(fixed_boundary_case_passes)
        && merge.as_ref().is_some_and(fixed_boundary_case_passes)
        && overflow.as_ref().is_some_and(|case| {
            case.get("overflow_pass")
                .and_then(serde_json::Value::as_bool)
                == Some(true)
        })
        && false_split_pass;

    serde_json::json!({
        "schema": "lay.slice6-boundary-internalization-shadow.v1",
        "enabled": enabled,
        "pass": pass,
        "verdict": if pass { "PASS_SLICE6_BOUNDARY_SHADOW" } else { "FAIL_SLICE6_BOUNDARY_SHADOW" },
        "selection": {
            "source": "exact installed package identities only",
            "maximum_scanned_forms": MAX_SCAN_FORMS,
            "scanned_eligible_forms": scanned_forms,
            "short_part_pool": short_parts.len(),
        },
        "true_split": split,
        "true_merge": merge,
        "multi_split_overflow": overflow,
        "false_split": {
            "observed": false_split_case,
            "birth_count": false_split_births.as_ref().map_or(0, |births| births.births.len()),
            "pass": false_split_pass,
        },
        "legacy_live_route_observation": {
            "authority_changed": false,
            "split_output": legacy_split,
            "split_exact_match": legacy_split_exact,
            "merge_output": legacy_merge,
            "merge_exact_match": legacy_merge_exact,
            "parity_is_not_a_promotion_gate": true,
        },
        "runtime_authority_changed": false,
    })
}

fn evaluate_fixed_boundary_case(
    observed: &str,
    expected: &str,
    relation: TargetRelationV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    exact_l11: Option<&ExactL11SurfaceIndexV1>,
    package_tuple: ExactPackageTupleV1,
    expect_overflow: bool,
) -> serde_json::Value {
    let births = enumerate_typed_boundary_births_from_packages(observed, canonical_l2, exact_l11);
    let birth_count = births.births.len();
    let logical_surfaces = births.logical_match_count;
    let expected_born = births.births.iter().any(|birth| {
        birth.relation == relation && (expected.is_empty() || birth.normalized_surface == expected)
    });
    let material = prepare_context_neutral_productive_material_with_contours_and_boundaries(
        observed,
        package_tuple,
        empty_productive_material_enumeration(),
        super::contour_birth::TypedContourBirthEnumerationV1::complete_empty(),
        births,
    );
    let (retained, born_only, composite_records, material_state, material_reason) = material
        .as_ref()
        .map(|material| {
            let retained = expected.is_empty()
                || material
                    .exact_target_surfaces()
                    .any(|surface| surface == expected);
            let born_only = material.compact().targets.as_slice().iter().all(|target| {
                target.witnesses.witnesses().iter().all(|witness| {
                    witness.grounding_namespace
                        != crate::typing_transition::target_evidence::GroundingNamespaceV1::CompositeBoundary
                        || witness.verdict_membership == VerdictMembershipV1::Born
                })
            });
            (
                retained,
                born_only,
                material.boundary_groundings().len(),
                format!("{:?}", material.completeness().state()),
                format!("{:?}", material.completeness().reason()),
            )
        })
        .unwrap_or((false, false, 0, "ERROR".to_string(), "ERROR".to_string()));
    let geometry_exact = match relation {
        TargetRelationV1::BoundarySplit => {
            expected.is_empty() || expected.replace(' ', "") == observed
        }
        TargetRelationV1::BoundaryMerge => observed.replace(' ', "") == expected,
        _ => false,
    };
    let overflow_pass = expect_overflow
        && logical_surfaces > 2
        && material.as_ref().is_ok_and(|material| {
            material.completeness().state() == EnumerationStateV1::Overflow
                && material.completeness().reason() == IncompletenessReasonV1::StorageCapacity
                && material.exact_target_surfaces().count() == 2
        });
    serde_json::json!({
        "observed": observed,
        "expected": expected,
        "relation": format!("{relation:?}"),
        "birth_count": birth_count,
        "logical_surfaces": logical_surfaces,
        "expected_born": expected_born,
        "retained": retained,
        "born_only": born_only,
        "composite_records": composite_records,
        "geometry_exact": geometry_exact,
        "material_state": material_state,
        "material_reason": material_reason,
        "overflow_pass": overflow_pass,
        "authority_granted": false,
    })
}

fn fixed_boundary_case_passes(case: &serde_json::Value) -> bool {
    ["expected_born", "retained", "born_only", "geometry_exact"]
        .into_iter()
        .all(|field| case.get(field).and_then(serde_json::Value::as_bool) == Some(true))
        && case
            .get("composite_records")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|count| count > 0)
        && case
            .get("authority_granted")
            .and_then(serde_json::Value::as_bool)
            == Some(false)
}

fn empty_productive_material_enumeration() -> ContextNeutralProductiveEnumerationV1 {
    ContextNeutralProductiveEnumerationV1 {
        readout: PackagedProductiveReadoutV1 {
            verdict: ProductiveCalibratedVerdictV1::Abstain {
                suggestions: Vec::new(),
                productive_overflow: false,
            },
            candidates: Vec::new(),
            logical_terminal_count: 0,
            logical_surface_basin_count: 0,
            integrity_error: None,
        },
        productive_work: MaterialWorkCountersV1::default(),
        aggregate_work: MaterialWorkCountersV1::default(),
        work_budget_exceeded: false,
    }
}

fn work_metric_summary(mut samples: Vec<u64>) -> serde_json::Value {
    samples.sort_unstable();
    serde_json::json!({
        "total": samples.iter().copied().sum::<u64>(),
        "p50": percentile(&samples, 50),
        "p95": percentile(&samples, 95),
        "p99": percentile(&samples, 99),
        "maximum": samples.last().copied().unwrap_or_default(),
    })
}

fn work_counter_summary(
    counters: impl Iterator<Item = EnumerationWorkCountersV1>,
) -> serde_json::Value {
    let counters = counters.collect::<Vec<_>>();
    serde_json::json!({
        "posting_visits": work_metric_summary(counters.iter().map(|row| row.posting_visits).collect()),
        "relation_replays": work_metric_summary(counters.iter().map(|row| row.relation_replays).collect()),
        "grounding_lookups": work_metric_summary(counters.iter().map(|row| row.grounding_lookups).collect()),
        "generated_logical_targets": work_metric_summary(counters.iter().map(|row| row.generated_logical_targets).collect()),
        "operator_steps": work_metric_summary(counters.iter().map(|row| row.operator_steps).collect()),
    })
}

fn summarize_enumeration_work(
    mut cases: Vec<EnumerationWorkCaseV1>,
    errors: Vec<String>,
    expected_samples: usize,
    enabled: bool,
) -> serde_json::Value {
    cases.sort_unstable_by(|left, right| {
        (left.proof_identity, left.damage_identity)
            .cmp(&(right.proof_identity, right.damage_identity))
    });
    let unique_samples = cases
        .iter()
        .map(|case| (case.proof_identity, case.damage_identity))
        .collect::<BTreeSet<_>>()
        .len();
    let aggregate_exact = cases.iter().all(|case| {
        case.canonical_grounding
            .checked_add(case.cold_binding)
            .and_then(|value| value.checked_add(case.productive_traversal))
            == Some(case.aggregate)
    });
    let sample_digest_sha256 = hex_sha256(
        Sha256::digest(serde_json::to_vec(&cases).expect("enumeration work cases serialize"))
            .into(),
    );
    let mut classes = BTreeMap::<&'static str, Vec<EnumerationWorkCountersV1>>::new();
    for case in &cases {
        classes
            .entry(case.damage_class)
            .or_default()
            .push(case.aggregate);
    }
    let by_class = classes
        .into_iter()
        .map(|(class, counters)| (class, work_counter_summary(counters.into_iter())))
        .collect::<BTreeMap<_, _>>();
    let complete = enabled
        && cases.len() == expected_samples
        && unique_samples == expected_samples
        && aggregate_exact
        && errors.is_empty();
    serde_json::json!({
        "schema": "lay.enumeration-work-measurement.v1",
        "enabled": enabled,
        "verdict": if complete { "PASS_COMPLETE_MEASUREMENT_ONLY" } else { "INCOMPLETE_NOT_A_BUDGET" },
        "complete": complete,
        "expected_samples": expected_samples,
        "samples": cases.len(),
        "unique_samples": unique_samples,
        "aggregate_exact": aggregate_exact,
        "errors": errors,
        "sample_digest_sha256": sample_digest_sha256,
        "scope": "one target-blind grounding/binding preparation plus one profiled damaged-surface Productive readout; oracle, baseline, probe and clean executions excluded",
        "counter_semantics": {
            "posting_visits": "direct and recovery posting records visited by target-blind cold binding",
            "relation_replays": "cold recovery/direct program executions plus Productive selected program or trie-arc visits",
            "grounding_lookups": "Productive descriptor, canonical observation, target-blind slot and canonical surface lookups",
            "generated_logical_targets": "Productive logical terminals before surface-basin coalescing and storage frontiers",
            "operator_steps": "scheduled morph-operation records for executed programs plus trie arc steps"
        },
        "producers": {
            "canonical_grounding": work_counter_summary(cases.iter().map(|case| case.canonical_grounding)),
            "cold_binding": work_counter_summary(cases.iter().map(|case| case.cold_binding)),
            "productive_traversal": work_counter_summary(cases.iter().map(|case| case.productive_traversal)),
            "aggregate": work_counter_summary(cases.iter().map(|case| case.aggregate)),
        },
        "aggregate_by_damage_class": by_class,
        "runtime_authority_changed": false,
        "numeric_budget_frozen": false,
    })
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
    enumeration_grounding_lookup_count: u64,
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
    let generation = validate_frozen_proof_generation(
        l11_sha256,
        canonical_l2_sha256,
        productive_sha256,
        axis_schema_sha256,
        proof_spool_sha256,
        spool_metadata.len(),
    )?;
    let manifest = if path.is_file() {
        serde_json::from_slice::<FrozenHypothesisManifestV1>(&std::fs::read(path)?)
            .map_err(io::Error::other)?
    } else {
        if !generation.permits_manifest_creation() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "the active L1.1 V9 proof generation requires the existing frozen V64 manifest",
            ));
        }
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
            v64_package_sha256: FROZEN_V64_PACKAGE_SHA256.to_string(),
            proof_spool_sha256: FROZEN_PROOF_SPOOL_SHA256.to_string(),
            proof_spool_bytes: FROZEN_PROOF_SPOOL_BYTES,
            l11_package_sha256: FROZEN_V64_L11_PACKAGE_SHA256.to_string(),
            canonical_l2_package_sha256: FROZEN_CANONICAL_L2_PACKAGE_SHA256.to_string(),
            axis_schema_sha256: FROZEN_AXIS_SCHEMA_SHA256.to_string(),
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
    validate_frozen_manifest(&manifest)?;
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
        generation,
        payload_sha256: manifest.payload_sha256,
        entries,
        h_count: manifest.h_count,
    })
}

fn frozen_proof_generation_for_pair(
    productive_sha256: &str,
    l11_sha256: &str,
) -> Option<FrozenProofGenerationV1> {
    FROZEN_PROOF_GENERATION_BINDINGS
        .iter()
        .find(|binding| {
            binding.productive_package_sha256 == productive_sha256
                && binding.l11_package_sha256 == l11_sha256
        })
        .map(|binding| binding.generation)
}

fn validate_frozen_proof_generation(
    l11_sha256: [u8; 32],
    canonical_l2_sha256: [u8; 32],
    productive_sha256: [u8; 32],
    axis_schema_sha256: [u8; 32],
    proof_spool_sha256: [u8; 32],
    proof_spool_bytes: u64,
) -> io::Result<FrozenProofGenerationV1> {
    if hex_sha256(canonical_l2_sha256) != FROZEN_CANONICAL_L2_PACKAGE_SHA256
        || hex_sha256(axis_schema_sha256) != FROZEN_AXIS_SCHEMA_SHA256
        || hex_sha256(proof_spool_sha256) != FROZEN_PROOF_SPOOL_SHA256
        || proof_spool_bytes != FROZEN_PROOF_SPOOL_BYTES
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frozen proof spool, canonical L2, or axis-schema identity mismatch",
        ));
    }
    let productive_sha256 = hex_sha256(productive_sha256);
    let l11_sha256 = hex_sha256(l11_sha256);
    frozen_proof_generation_for_pair(&productive_sha256, &l11_sha256).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "frozen proof Productive/L1.1 generation pair mismatch",
        )
    })
}

fn validate_frozen_manifest(manifest: &FrozenHypothesisManifestV1) -> io::Result<()> {
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
        || manifest.l11_package_sha256 != FROZEN_V64_L11_PACKAGE_SHA256
        || manifest.canonical_l2_package_sha256 != FROZEN_CANONICAL_L2_PACKAGE_SHA256
        || manifest.axis_schema_sha256 != FROZEN_AXIS_SCHEMA_SHA256
        || manifest.heldout_per_class != FROZEN_MANIFEST_HELDOUT_PER_CLASS
        || manifest.cohorts != vec![ProofCohortV1::LemmaHeldout.name().to_string()]
        || manifest.damage_generator_id != FROZEN_DAMAGE_GENERATOR_ID
        || manifest.entry_count != FROZEN_MANIFEST_ENTRY_COUNT
        || manifest.entries.len() != FROZEN_MANIFEST_ENTRY_COUNT
        || manifest.h_count != FROZEN_MANIFEST_H_COUNT
        || measured_h != FROZEN_MANIFEST_H_COUNT
        || manifest.payload_sha256 != FROZEN_MANIFEST_PAYLOAD_SHA256
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
    let material_package_tuple = ExactPackageTupleV1 {
        l11_sha256,
        canonical_l2_sha256,
        productive_sha256,
    };
    let canonical_l2 = super::super::runtime::StandaloneL2Field::load(l2_package_path)
        .map_err(io::Error::other)?;
    let contour_birth_enabled = std::env::var_os("LAY_PRODUCTIVE_CONTOUR_BIRTH_PROOF").is_some();
    let boundary_birth_enabled = std::env::var_os("LAY_PRODUCTIVE_BOUNDARY_PROOF").is_some();
    let live_cohort_compare_enabled =
        std::env::var_os("LAY_PRODUCTIVE_LIVE_COHORT_COMPARE_PROOF").is_some();
    let exact_l11 =
        (contour_birth_enabled || boundary_birth_enabled || live_cohort_compare_enabled)
            .then(|| ExactL11SurfaceIndexV1::load(l1_package_path))
            .transpose()
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
    let live_cohort_preload_us = if live_cohort_compare_enabled {
        let started = Instant::now();
        super::super::preload_installed_l2_field().map_err(|error| {
            io::Error::other(format!(
                "live cohort proof could not preload canonical L2 and Productive V90: {error}"
            ))
        })?;
        started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
    } else {
        0
    };
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
                let exact_l11 = exact_l11.as_ref();
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
                        exact_l11,
                        axis_schema,
                        frozen_manifest,
                        material_package_tuple,
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
    let enumeration_work_enabled = std::env::var_os("LAY_PRODUCTIVE_WORK_MEASUREMENT").is_some();
    let enumeration_work_measurement = summarize_enumeration_work(
        std::mem::take(&mut proof.enumeration_work_cases),
        std::mem::take(&mut proof.enumeration_work_errors),
        EXPECTED_DAMAGE_CLASSES.saturating_mul(heldout_per_class),
        enumeration_work_enabled,
    );
    let material_frame_enabled = std::env::var_os("LAY_PRODUCTIVE_MATERIAL_FRAME_PROOF").is_some();
    let material_frame_shadow = summarize_material_frame_shadow(
        std::mem::take(&mut proof.material_frame_cases),
        std::mem::take(&mut proof.material_frame_errors),
        EXPECTED_DAMAGE_CLASSES.saturating_mul(heldout_per_class),
        material_frame_enabled,
    );
    let live_cohort_compare_shadow = summarize_live_cohort_compare_shadow(
        std::mem::take(&mut proof.live_cohort_compare_cases),
        std::mem::take(&mut proof.live_cohort_no_field_cases),
        std::mem::take(&mut proof.live_cohort_compare_errors),
        EXPECTED_DAMAGE_CLASSES.saturating_mul(heldout_per_class),
        live_cohort_compare_enabled,
        live_cohort_preload_us,
    );
    let contour_birth_shadow = summarize_slice5_contour_birth_shadow(
        &canonical_l2,
        exact_l11.as_ref(),
        material_package_tuple,
        contour_birth_enabled,
    );
    let boundary_internalization_shadow = summarize_slice6_boundary_internalization_shadow(
        &canonical_l2,
        exact_l11.as_ref(),
        material_package_tuple,
        boundary_birth_enabled,
    );
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
    let semantic_non_latency_gate = class_count_by_cohort
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
        && probe_parity_complete
        && proof.probe_parity_failures == 0;
    let measured_quality_gate = semantic_non_latency_gate && maximum_class_p99_us <= 5_000;
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

    let mut report = serde_json::json!({
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
            "proof_generation": frozen_manifest.generation.name(),
            "oracle_runtime": "base productive package with anchor recovery explicitly disabled",
            "experimental_runtime_can_change_h": false,
            "base_package_sha256": hex_sha256(sha256_file(productive_package_path)?),
            "active_l11_sha256": hex_sha256(l11_sha256),
            "canonical_l2_sha256": hex_sha256(canonical_l2_sha256),
            "axis_schema_sha256": hex_sha256(axis_schema_sha256),
            "proof_spool_sha256": FROZEN_PROOF_SPOOL_SHA256,
            "proof_spool_bytes": FROZEN_PROOF_SPOOL_BYTES,
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
        "enumeration_work_measurement": enumeration_work_measurement,
        "stage_telemetry": {
            "enabled": std::env::var_os("LAY_PRODUCTIVE_STAGE_TELEMETRY").is_some() || enumeration_work_enabled,
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
    });
    report
        .as_object_mut()
        .expect("productive proof report is an object")
        .insert("material_frame_shadow".to_string(), material_frame_shadow);
    report
        .as_object_mut()
        .expect("productive proof report is an object")
        .insert(
            "live_cohort_compare_shadow".to_string(),
            live_cohort_compare_shadow,
        );
    report
        .as_object_mut()
        .expect("productive proof report is an object")
        .insert("contour_birth_shadow".to_string(), contour_birth_shadow);
    report
        .as_object_mut()
        .expect("productive proof report is an object")
        .insert(
            "boundary_internalization_shadow".to_string(),
            boundary_internalization_shadow,
        );
    report
        .as_object_mut()
        .expect("productive proof report is an object")
        .insert(
            "semantic_non_latency_gate".to_string(),
            serde_json::Value::Bool(
                semantic_non_latency_gate && false_singleton == 0 && integrity_errors == 0,
            ),
        );
    Ok(report)
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
    exact_l11: Option<&ExactL11SurfaceIndexV1>,
    axis_schema: &super::transition_reduce::MorphologyAxisSchemaV1,
    frozen_manifest: &FrozenHypothesisIndexV1,
    material_package_tuple: ExactPackageTupleV1,
) -> ProofShardV1 {
    let mut shard = ProofShardV1::default();
    for case in case_indices.iter().map(|index| &cases[*index]) {
        let result = evaluate_case(
            case,
            runtime,
            frozen_oracle_runtime,
            canonical_l2,
            exact_l11,
            axis_schema,
            frozen_manifest,
            material_package_tuple,
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
                if let Some(material_frame) = result.material_frame {
                    match material_frame {
                        Ok(case) => shard.material_frame_cases.push(case),
                        Err(error) if shard.material_frame_errors.len() < 64 => {
                            shard.material_frame_errors.push(error);
                        }
                        Err(_) => {}
                    }
                }
                if case.cohort == ProofCohortV1::LemmaHeldout
                    && std::env::var_os("LAY_PRODUCTIVE_LIVE_COHORT_COMPARE_PROOF").is_some()
                {
                    match result.live_cohort_compare {
                        Some(Ok(LiveCohortProofOutcomeV1::ProducedField(case))) => {
                            shard.live_cohort_compare_cases.push(case);
                        }
                        Some(Ok(LiveCohortProofOutcomeV1::NoField(case))) => {
                            shard.live_cohort_no_field_cases.push(case);
                        }
                        Some(Err(error)) => shard
                            .live_cohort_compare_errors
                            .push(LiveCohortCompareErrorV1::new(case, error)),
                        None => {
                            shard
                                .live_cohort_compare_errors
                                .push(LiveCohortCompareErrorV1::new(
                                    case,
                                    "live cohort proof produced no typed outcome".to_string(),
                                ))
                        }
                    }
                }
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
                        if std::env::var_os("LAY_PRODUCTIVE_WORK_MEASUREMENT").is_some() {
                            if let Some(telemetry) = result.stage_telemetry {
                                match EnumerationWorkCaseV1::new(
                                    case,
                                    result.enumeration_grounding_lookup_count,
                                    &diagnostics.target_blind,
                                    telemetry,
                                ) {
                                    Ok(sample) => shard.enumeration_work_cases.push(sample),
                                    Err(error) if shard.enumeration_work_errors.len() < 64 => {
                                        shard.enumeration_work_errors.push(error);
                                    }
                                    Err(_) => {}
                                }
                            }
                        }
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
                if case.cohort == ProofCohortV1::LemmaHeldout
                    && std::env::var_os("LAY_PRODUCTIVE_LIVE_COHORT_COMPARE_PROOF").is_some()
                {
                    shard
                        .live_cohort_compare_errors
                        .push(LiveCohortCompareErrorV1::new(case, error.clone()));
                }
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

#[derive(Clone, Debug)]
enum LiveCohortProofOutcomeV1 {
    ProducedField(LiveCohortCompareCaseV1),
    NoField(LiveCohortNoFieldCaseV1),
}

#[derive(Clone, Debug)]
struct LiveCohortNoFieldCaseV1 {
    class: &'static str,
    proof_identity: [u8; 32],
    damaged_surface: String,
    target_surface: String,
    availability: &'static str,
    field_producer_count: u64,
    cache_disposition: &'static str,
    l11_us: u64,
    productive_v90_us: u64,
    bridge_total_us: u64,
    latency_us: u64,
    target_exact_l11: bool,
    target_exact_v13: bool,
    productive_hypothesis_covered: bool,
    productive_exact_born: bool,
    provenance_complete: bool,
}

impl LiveCohortNoFieldCaseV1 {
    fn bind_target_provenance(
        &mut self,
        exact_l11: &ExactL11SurfaceIndexV1,
        canonical_l2: &super::super::runtime::StandaloneL2Field,
        productive_hypothesis_covered: bool,
        productive_exact_born: bool,
    ) -> Result<(), String> {
        if self.provenance_complete {
            return Err("NoField target provenance was bound twice".to_string());
        }
        self.target_exact_l11 = exact_l11
            .terminal_for_surface(&self.target_surface)
            .is_some();
        self.target_exact_v13 = canonical_l2
            .form_ref_for_surface(&self.target_surface)
            .is_some();
        self.productive_hypothesis_covered = productive_hypothesis_covered;
        self.productive_exact_born = productive_exact_born;
        self.provenance_complete = true;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
struct LiveCohortCompareErrorV1 {
    class: &'static str,
    proof_identity: [u8; 32],
    damaged_surface: String,
    target_surface: String,
    error: String,
}

impl LiveCohortCompareErrorV1 {
    fn new(case: &ProofCaseV1, error: String) -> Self {
        Self {
            class: case.class,
            proof_identity: case.event.proof_identity,
            damaged_surface: case.damaged_surface.clone(),
            target_surface: case.event.observed_surface.clone(),
            error,
        }
    }
}

#[derive(Clone, Debug)]
struct LiveCohortCompareCaseV1 {
    class: &'static str,
    proof_identity: [u8; 32],
    damaged_surface: String,
    target_surface: String,
    status: &'static str,
    material_scope: &'static str,
    legacy_kind: &'static str,
    cohort_kind: &'static str,
    first_divergence: Option<&'static str>,
    field_candidate_count: usize,
    material_target_count: usize,
    retained_field_candidate_count: usize,
    grounded_l11_loss_count: usize,
    unretained_field_candidate_surfaces: Vec<String>,
    lost_grounded_l11_surfaces: Vec<String>,
    complete_for_authority: bool,
    legacy_decision_parity_exact: bool,
    field_producer_count: u64,
    cache_disposition: &'static str,
    l11_us: u64,
    productive_v90_us: u64,
    bridge_total_us: u64,
    latency_us: u64,
}

fn evaluate_live_cohort_compare_case(
    case: &ProofCaseV1,
) -> Result<LiveCohortProofOutcomeV1, String> {
    let context_prefix = proof_live_context_prefix(case);
    let original = format!("{context_prefix}{}", case.damaged_surface);
    let Some((lexical_context, observed_token)) =
        super::super::bridge::canonical_input_lexical_parts(&original)
    else {
        let started = Instant::now();
        let observed =
            super::super::bridge::canonical_text_readout_observed_with_frame(&original, None);
        let latency_us = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
        if observed.cohort_compare.is_some() {
            return Err("unsupported production input unexpectedly produced a cohort".to_string());
        }
        return live_no_field_outcome(
            case,
            observed.readout.availability,
            &observed.readout.authority,
            observed.telemetry.field_producer_count,
            observed.telemetry.cache_disposition.as_str(),
            observed.telemetry.l11_us,
            observed.telemetry.productive_v90_us,
            observed.telemetry.total_us,
            latency_us,
        );
    };
    let frame = proof_lexical_authority_frame(case, &original, lexical_context, observed_token)?;
    let started = Instant::now();
    let observed =
        super::super::bridge::canonical_text_readout_observed_with_frame(&original, Some(&frame));
    let latency_us = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    let Some(compare) = observed.cohort_compare.as_ref() else {
        return live_no_field_outcome(
            case,
            observed.readout.availability,
            &observed.readout.authority,
            observed.telemetry.field_producer_count,
            observed.telemetry.cache_disposition.as_str(),
            observed.telemetry.l11_us,
            observed.telemetry.productive_v90_us,
            observed.telemetry.total_us,
            latency_us,
        );
    };
    Ok(LiveCohortProofOutcomeV1::ProducedField(
        LiveCohortCompareCaseV1 {
            class: case.class,
            proof_identity: case.event.proof_identity,
            damaged_surface: case.damaged_surface.clone(),
            target_surface: case.event.observed_surface.clone(),
            status: cohort_compare_status_name(compare.status),
            material_scope: material_scope_name(compare.material_scope),
            legacy_kind: lexical_observation_kind(&compare.legacy),
            cohort_kind: lexical_observation_kind(&compare.cohort),
            first_divergence: compare.first_divergence.map(cohort_divergence_name),
            field_candidate_count: compare.field_candidate_count,
            material_target_count: compare.material_target_count,
            retained_field_candidate_count: compare.retained_field_candidate_count,
            grounded_l11_loss_count: compare.grounded_l11_loss_count,
            unretained_field_candidate_surfaces: compare
                .unretained_field_candidate_surfaces
                .clone(),
            lost_grounded_l11_surfaces: compare.lost_grounded_l11_surfaces.clone(),
            complete_for_authority: compare.complete_for_authority,
            legacy_decision_parity_exact: legacy_observation_matches_authority(
                &compare.legacy,
                &observed.readout.authority,
            ),
            field_producer_count: observed.telemetry.field_producer_count,
            cache_disposition: observed.telemetry.cache_disposition.as_str(),
            l11_us: observed.telemetry.l11_us,
            productive_v90_us: observed.telemetry.productive_v90_us,
            bridge_total_us: observed.telemetry.total_us,
            latency_us,
        },
    ))
}

#[allow(clippy::too_many_arguments)]
fn live_no_field_outcome(
    case: &ProofCaseV1,
    availability: L2FieldAvailability,
    authority: &L2FieldAuthority,
    field_producer_count: u64,
    cache_disposition: &'static str,
    l11_us: u64,
    productive_v90_us: u64,
    bridge_total_us: u64,
    latency_us: u64,
) -> Result<LiveCohortProofOutcomeV1, String> {
    if !matches!(
        availability,
        L2FieldAvailability::UnsupportedInput | L2FieldAvailability::EmptyL11Lattice
    ) {
        return Err(format!(
            "live cohort comparator missing for {}::{}",
            case.class,
            l2_field_availability_name(availability)
        ));
    }
    if !matches!(
        authority,
        L2FieldAuthority::Abstain | L2FieldAuthority::Unavailable
    ) || field_producer_count != 0
    {
        return Err(format!(
            "no-field observation carried authority or a producer for {}::{}",
            case.class,
            l2_field_availability_name(availability)
        ));
    }
    Ok(LiveCohortProofOutcomeV1::NoField(LiveCohortNoFieldCaseV1 {
        class: case.class,
        proof_identity: case.event.proof_identity,
        damaged_surface: case.damaged_surface.clone(),
        target_surface: case.event.observed_surface.clone(),
        availability: l2_field_availability_name(availability),
        field_producer_count,
        cache_disposition,
        l11_us,
        productive_v90_us,
        bridge_total_us,
        latency_us,
        target_exact_l11: false,
        target_exact_v13: false,
        productive_hypothesis_covered: false,
        productive_exact_born: false,
        provenance_complete: false,
    }))
}

fn proof_live_context_prefix(case: &ProofCaseV1) -> String {
    let joined = case
        .event
        .scene
        .left_tokens
        .iter()
        .flatten()
        .map(|token| token.normalized_surface.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    if joined.is_empty() {
        joined
    } else {
        format!("{joined} ")
    }
}

fn proof_lexical_authority_frame(
    case: &ProofCaseV1,
    committed_tail: &str,
    context_prefix: &str,
    observed_token: &str,
) -> Result<crate::lexical_authority_frame::LexicalAuthorityFrameV1, String> {
    let scalar_count = u32::try_from(observed_token.chars().count())
        .map_err(|_| "proof token exceeds u32 scalar width".to_string())?;
    let first = u64::from_le_bytes(
        case.event.proof_identity[..8]
            .try_into()
            .expect("fixed proof identity prefix"),
    )
    .max(1);
    let second = u64::from_le_bytes(
        case.event.proof_identity[8..16]
            .try_into()
            .expect("fixed proof identity suffix"),
    )
    .max(1);
    let config = crate::config::LayConfig::default();
    let coordinates = crate::lexical_authority_frame::LexicalAuthorityCoordinatesV1::new(
        first,
        [first, second],
        second,
        observed_token.to_string(),
        context_prefix.to_string(),
        scalar_count,
        (scalar_count, scalar_count),
        observed_token.to_string(),
        scalar_count,
        1,
        1,
    )
    .ok_or_else(|| "fixed proof could not construct exact lexical coordinates".to_string())?;
    Ok(
        crate::lexical_authority_frame::LexicalAuthorityFrameV1::from_exact_parts(
            "productive-fixed-live-proof".to_string(),
            Some(format!("{:016x}", first)),
            first,
            committed_tail.to_string(),
            context_prefix.to_string(),
            observed_token.to_string(),
            true,
            true,
            crate::exact_layout_authority::FactoryEngineProfile::Ru,
            None,
            first,
            second,
            crate::lexical_authority_frame::LexicalAuthorityConfigIdentityV1::from_config(&config),
        )
        .with_coordinates(Some(coordinates)),
    )
}

const fn cohort_compare_status_name(status: CohortCompareStatusV1) -> &'static str {
    match status {
        CohortCompareStatusV1::MissingFrame => "MISSING_FRAME",
        CohortCompareStatusV1::MissingCoordinates => "MISSING_COORDINATES",
        CohortCompareStatusV1::FrameMismatch => "FRAME_MISMATCH",
        CohortCompareStatusV1::LeaseUnavailable => "LEASE_UNAVAILABLE",
        CohortCompareStatusV1::SettlementFailed => "SETTLEMENT_FAILED",
        CohortCompareStatusV1::Ready => "READY",
    }
}

const fn l2_field_availability_name(availability: L2FieldAvailability) -> &'static str {
    match availability {
        L2FieldAvailability::Ready => "READY",
        L2FieldAvailability::UnsupportedInput => "UNSUPPORTED_INPUT",
        L2FieldAvailability::EmptyL11Lattice => "EMPTY_L11_LATTICE",
        L2FieldAvailability::L11ServiceUnavailable => "L11_SERVICE_UNAVAILABLE",
        L2FieldAvailability::CanonicalPackageUnavailable => "CANONICAL_PACKAGE_UNAVAILABLE",
        L2FieldAvailability::ProductivePackageUnavailable => "PRODUCTIVE_PACKAGE_UNAVAILABLE",
        L2FieldAvailability::ProductiveReadoutError => "PRODUCTIVE_READOUT_ERROR",
    }
}

const fn material_scope_name(scope: super::live::PreparedFieldMaterialScopeV1) -> &'static str {
    match scope {
        super::live::PreparedFieldMaterialScopeV1::ContextNeutral => "CONTEXT_NEUTRAL",
        super::live::PreparedFieldMaterialScopeV1::ContextShapedObservation => {
            "CONTEXT_SHAPED_OBSERVATION"
        }
    }
}

const fn cohort_divergence_name(divergence: CohortFirstDivergenceV1) -> &'static str {
    match divergence {
        CohortFirstDivergenceV1::CandidateRetention => "CANDIDATE_RETENTION",
        CohortFirstDivergenceV1::VerdictKind => "VERDICT_KIND",
        CohortFirstDivergenceV1::WinnerSurface => "WINNER_SURFACE",
    }
}

const fn lexical_observation_kind(observation: &LexicalVerdictObservationV1) -> &'static str {
    match observation {
        LexicalVerdictObservationV1::Winner(_) => "WINNER",
        LexicalVerdictObservationV1::Tied(_) => "TIED",
        LexicalVerdictObservationV1::Abstain => "ABSTAIN",
        LexicalVerdictObservationV1::Unavailable => "UNAVAILABLE",
    }
}

fn legacy_observation_matches_authority(
    observation: &LexicalVerdictObservationV1,
    authority: &L2FieldAuthority,
) -> bool {
    match (observation, authority) {
        (
            LexicalVerdictObservationV1::Winner(left),
            L2FieldAuthority::Winner { surface: right },
        ) => left == right,
        (LexicalVerdictObservationV1::Tied(left), L2FieldAuthority::Tied { surfaces: right }) => {
            left == right
        }
        (LexicalVerdictObservationV1::Abstain, L2FieldAuthority::Abstain)
        | (LexicalVerdictObservationV1::Unavailable, L2FieldAuthority::Unavailable) => true,
        _ => false,
    }
}

fn summarize_live_cohort_compare_shadow(
    mut cases: Vec<LiveCohortCompareCaseV1>,
    mut no_field_cases: Vec<LiveCohortNoFieldCaseV1>,
    mut errors: Vec<LiveCohortCompareErrorV1>,
    expected_cases: usize,
    enabled: bool,
    preload_us: u64,
) -> serde_json::Value {
    if !enabled {
        return serde_json::json!({
            "schema": "lay.live-cohort-compare-shadow.v3",
            "enabled": false,
            "verdict": "NOT_RUN",
            "runtime_authority_changed": false,
        });
    }
    cases.sort_unstable_by(|left, right| {
        left.class
            .cmp(right.class)
            .then_with(|| left.proof_identity.cmp(&right.proof_identity))
    });
    no_field_cases.sort_unstable_by(|left, right| {
        left.class
            .cmp(right.class)
            .then_with(|| left.proof_identity.cmp(&right.proof_identity))
    });
    errors.sort_unstable_by(|left, right| {
        left.class
            .cmp(right.class)
            .then_with(|| left.proof_identity.cmp(&right.proof_identity))
    });
    let mut status_counts = BTreeMap::<&'static str, usize>::new();
    let mut material_scope_counts = BTreeMap::<&'static str, usize>::new();
    let mut legacy_verdict_counts = BTreeMap::<&'static str, usize>::new();
    let mut cohort_verdict_counts = BTreeMap::<&'static str, usize>::new();
    let mut divergence_counts = BTreeMap::<&'static str, usize>::new();
    let mut cache_dispositions = BTreeMap::<&'static str, usize>::new();
    let mut no_field_availability_counts = BTreeMap::<&'static str, usize>::new();
    let mut no_field_class_counts = BTreeMap::<&'static str, usize>::new();
    let mut no_field_provenance_buckets = BTreeMap::<String, usize>::new();
    let mut no_field_provenance_by_class = BTreeMap::<&'static str, BTreeMap<String, usize>>::new();
    let mut no_field_provenance_by_availability =
        BTreeMap::<&'static str, BTreeMap<String, usize>>::new();
    let mut no_field_target_exact_l11 = 0_usize;
    let mut no_field_target_exact_v13 = 0_usize;
    let mut no_field_productive_hypothesis_covered = 0_usize;
    let mut no_field_productive_exact_born = 0_usize;
    let mut no_field_incomplete_provenance = 0_usize;
    let mut latency_us = Vec::with_capacity(cases.len());
    let mut no_field_latency_us = Vec::with_capacity(no_field_cases.len());
    for case in &cases {
        *status_counts.entry(case.status).or_default() += 1;
        *material_scope_counts
            .entry(case.material_scope)
            .or_default() += 1;
        *legacy_verdict_counts.entry(case.legacy_kind).or_default() += 1;
        *cohort_verdict_counts.entry(case.cohort_kind).or_default() += 1;
        *divergence_counts
            .entry(case.first_divergence.unwrap_or("NONE"))
            .or_default() += 1;
        *cache_dispositions
            .entry(case.cache_disposition)
            .or_default() += 1;
        latency_us.push(case.latency_us);
    }
    for case in &no_field_cases {
        *no_field_availability_counts
            .entry(case.availability)
            .or_default() += 1;
        *no_field_class_counts.entry(case.class).or_default() += 1;
        no_field_target_exact_l11 += usize::from(case.target_exact_l11);
        no_field_target_exact_v13 += usize::from(case.target_exact_v13);
        no_field_productive_hypothesis_covered += usize::from(case.productive_hypothesis_covered);
        no_field_productive_exact_born += usize::from(case.productive_exact_born);
        no_field_incomplete_provenance += usize::from(!case.provenance_complete);
        let provenance_key = format!(
            "l11_exact={};v13_exact={};v90_hypothesis={};v90_exact_born={}",
            case.target_exact_l11,
            case.target_exact_v13,
            case.productive_hypothesis_covered,
            case.productive_exact_born,
        );
        *no_field_provenance_buckets
            .entry(provenance_key.clone())
            .or_default() += 1;
        *no_field_provenance_by_class
            .entry(case.class)
            .or_default()
            .entry(provenance_key.clone())
            .or_default() += 1;
        *no_field_provenance_by_availability
            .entry(case.availability)
            .or_default()
            .entry(provenance_key)
            .or_default() += 1;
        no_field_latency_us.push(case.latency_us);
    }
    latency_us.sort_unstable();
    no_field_latency_us.sort_unstable();
    let candidate_retention_failures = cases
        .iter()
        .filter(|case| case.retained_field_candidate_count != case.field_candidate_count)
        .count();
    let grounded_l11_losses = cases
        .iter()
        .map(|case| case.grounded_l11_loss_count)
        .sum::<usize>();
    let legacy_decision_parity_failures = cases
        .iter()
        .filter(|case| !case.legacy_decision_parity_exact)
        .count();
    let context_shaped_cases = cases
        .iter()
        .filter(|case| case.material_scope == "CONTEXT_SHAPED_OBSERVATION")
        .count();
    let complete_for_authority = cases
        .iter()
        .filter(|case| case.complete_for_authority)
        .count();
    let ready = status_counts.get("READY").copied().unwrap_or_default();
    let field_producer_count = cases
        .iter()
        .map(|case| case.field_producer_count)
        .sum::<u64>();
    let no_field_unexpected_producers = no_field_cases
        .iter()
        .map(|case| case.field_producer_count)
        .sum::<u64>();
    let attempted_cases = cases
        .len()
        .saturating_add(no_field_cases.len())
        .saturating_add(errors.len());
    let mut attempt_identities = BTreeSet::<(&'static str, [u8; 32])>::new();
    attempt_identities.extend(cases.iter().map(|case| (case.class, case.proof_identity)));
    attempt_identities.extend(
        no_field_cases
            .iter()
            .map(|case| (case.class, case.proof_identity)),
    );
    attempt_identities.extend(errors.iter().map(|case| (case.class, case.proof_identity)));
    let unique_attempt_identities = attempt_identities.len();
    let duplicate_attempt_identities = attempted_cases.saturating_sub(unique_attempt_identities);
    let attempt_denominator_conserved = attempted_cases == expected_cases
        && unique_attempt_identities == expected_cases
        && duplicate_attempt_identities == 0;
    let produced_field_identity_parity = field_producer_count == cases.len() as u64;
    let scoped_pass = attempt_denominator_conserved
        && errors.is_empty()
        && !cases.is_empty()
        && ready == cases.len()
        && produced_field_identity_parity
        && no_field_unexpected_producers == 0
        && no_field_incomplete_provenance == 0
        && candidate_retention_failures == 0
        && grounded_l11_losses == 0
        && legacy_decision_parity_failures == 0
        && context_shaped_cases == cases.len()
        && complete_for_authority == 0;
    let production_field_coverage_complete = scoped_pass && no_field_cases.is_empty();
    let overall_pass = scoped_pass && production_field_coverage_complete;
    let slowest = cases.iter().max_by_key(|case| case.latency_us).map(|case| {
        serde_json::json!({
            "class": case.class,
            "proof_identity": case.proof_identity,
            "latency_us": case.latency_us,
            "field_candidates": case.field_candidate_count,
            "material_targets": case.material_target_count,
            "l11_us": case.l11_us,
            "productive_v90_us": case.productive_v90_us,
            "bridge_total_us": case.bridge_total_us,
        })
    });
    let failure_cases = cases
        .iter()
        .filter(|case| {
            case.status != "READY"
                || !case.unretained_field_candidate_surfaces.is_empty()
                || !case.lost_grounded_l11_surfaces.is_empty()
                || !case.legacy_decision_parity_exact
        })
        .map(|case| {
            serde_json::json!({
                "class": case.class,
                "proof_identity": case.proof_identity,
                "damaged_surface": case.damaged_surface,
                "target_surface": case.target_surface,
                "status": case.status,
                "field_candidate_count": case.field_candidate_count,
                "material_target_count": case.material_target_count,
                "retained_field_candidate_count": case.retained_field_candidate_count,
                "unretained_field_candidate_surfaces": case.unretained_field_candidate_surfaces,
                "lost_grounded_l11_surfaces": case.lost_grounded_l11_surfaces,
                "l11_us": case.l11_us,
                "productive_v90_us": case.productive_v90_us,
                "bridge_total_us": case.bridge_total_us,
                "wall_us": case.latency_us,
            })
        })
        .collect::<Vec<_>>();
    let no_field_samples = no_field_cases
        .iter()
        .take(16)
        .map(|case| {
            serde_json::json!({
                "class": case.class,
                "proof_identity": case.proof_identity,
                "damaged_surface": case.damaged_surface,
                "target_surface": case.target_surface,
                "availability": case.availability,
                "cache_disposition": case.cache_disposition,
                "l11_us": case.l11_us,
                "productive_v90_us": case.productive_v90_us,
                "bridge_total_us": case.bridge_total_us,
                "wall_us": case.latency_us,
                "target_provenance": {
                    "l11_exact_support": case.target_exact_l11,
                    "v13_exact_support": case.target_exact_v13,
                    "productive_hypothesis_covered": case.productive_hypothesis_covered,
                    "productive_exact_born": case.productive_exact_born,
                    "complete": case.provenance_complete,
                },
            })
        })
        .collect::<Vec<_>>();
    let no_field_records = no_field_cases
        .iter()
        .map(|case| {
            serde_json::json!({
                "class": case.class,
                "proof_identity": case.proof_identity,
                "damaged_surface": case.damaged_surface,
                "target_surface": case.target_surface,
                "availability": case.availability,
                "target_provenance": {
                    "l11_exact_support": case.target_exact_l11,
                    "v13_exact_support": case.target_exact_v13,
                    "productive_hypothesis_covered": case.productive_hypothesis_covered,
                    "productive_exact_born": case.productive_exact_born,
                    "complete": case.provenance_complete,
                },
            })
        })
        .collect::<Vec<_>>();
    let verdict = if scoped_pass {
        if production_field_coverage_complete {
            "PASS_SLICE8B_PRODUCED_FIELD_COMPARATOR_COVERAGE_COMPLETE"
        } else {
            "PASS_SLICE8B_PRODUCED_FIELD_COMPARATOR_UPSTREAM_COVERAGE_OPEN"
        }
    } else {
        "FAIL_SLICE8B_PRODUCED_FIELD_COMPARATOR"
    };
    let production_field_coverage_verdict = if production_field_coverage_complete {
        "PASS_COMPLETE"
    } else {
        "FAIL_UPSTREAM_NO_FIELD"
    };
    let mut summary = serde_json::Map::new();
    for section in [
        serde_json::json!({
            "schema": "lay.live-cohort-compare-shadow.v3",
            "enabled": true,
            "verdict": verdict,
            "pass": scoped_pass,
            "scoped_comparator_pass": scoped_pass,
            "overall_pass": overall_pass,
            "promotion_eligible": false,
            "expected_cases": expected_cases,
            "attempted_cases": attempted_cases,
            "unique_attempt_identities": unique_attempt_identities,
            "duplicate_attempt_identities": duplicate_attempt_identities,
            "attempt_denominator_conserved": attempt_denominator_conserved,
            "cases": cases.len(),
            "produced_field_cases": cases.len(),
            "no_field_cases": no_field_cases.len(),
            "error_cases": errors.len(),
            "errors": errors,
        }),
        serde_json::json!({
            "status_counts": status_counts,
            "material_scope_counts": material_scope_counts,
            "legacy_verdict_counts": legacy_verdict_counts,
            "cohort_verdict_counts": cohort_verdict_counts,
            "first_divergence_counts": divergence_counts,
            "cache_dispositions": cache_dispositions,
            "field_producer_count": field_producer_count,
            "produced_field_identity_parity": produced_field_identity_parity,
            "no_field_unexpected_producers": no_field_unexpected_producers,
            "production_field_coverage_complete": production_field_coverage_complete,
            "production_field_coverage_verdict": production_field_coverage_verdict,
            "no_field_availability_counts": no_field_availability_counts,
            "no_field_class_counts": no_field_class_counts,
            "no_field_samples": no_field_samples,
            "no_field_provenance": {
                "records": no_field_records.len(),
                "incomplete_records": no_field_incomplete_provenance,
                "target_exact_l11": no_field_target_exact_l11,
                "target_absent_l11": no_field_cases.len().saturating_sub(no_field_target_exact_l11),
                "target_exact_v13": no_field_target_exact_v13,
                "target_absent_v13": no_field_cases.len().saturating_sub(no_field_target_exact_v13),
                "productive_hypothesis_covered": no_field_productive_hypothesis_covered,
                "productive_hypothesis_absent": no_field_cases.len().saturating_sub(no_field_productive_hypothesis_covered),
                "productive_exact_born": no_field_productive_exact_born,
                "productive_exact_not_born": no_field_cases.len().saturating_sub(no_field_productive_exact_born),
                "joint_buckets": no_field_provenance_buckets,
                "by_damage_class": no_field_provenance_by_class,
                "by_availability": no_field_provenance_by_availability,
            },
            "no_field_records": no_field_records,
        }),
        serde_json::json!({
            "candidate_retention_failures": candidate_retention_failures,
            "grounded_l11_losses": grounded_l11_losses,
            "legacy_decision_parity_failures": legacy_decision_parity_failures,
            "context_shaped_cases": context_shaped_cases,
            "complete_for_authority": complete_for_authority,
            "failure_cases": failure_cases,
            "preload": {
                "status": "READY_BEFORE_TIMED_WORKERS",
                "elapsed_us": preload_us,
                "scope": "canonical L2 and Productive V90 package admission outside queue-inclusive request latency",
            },
            "latency": {
                "scope": "queue-inclusive produced-field bridge call with L1.1 service, cache and comparator",
                "p50_us": percentile(&latency_us, 50),
                "p95_us": percentile(&latency_us, 95),
                "p99_us": percentile(&latency_us, 99),
                "maximum_us": latency_us.last().copied().unwrap_or_default(),
                "slowest": slowest,
            },
            "no_field_latency": {
                "scope": "queue-inclusive production bridge call that returned no canonical field",
                "p50_us": percentile(&no_field_latency_us, 50),
                "p95_us": percentile(&no_field_latency_us, 95),
                "p99_us": percentile(&no_field_latency_us, 99),
                "maximum_us": no_field_latency_us.last().copied().unwrap_or_default(),
            },
            "runtime_authority_changed": false,
        }),
    ] {
        let serde_json::Value::Object(section) = section else {
            unreachable!("live cohort summary section must be a JSON object");
        };
        for (key, value) in section {
            assert!(
                summary.insert(key.clone(), value).is_none(),
                "duplicate live cohort summary key: {key}"
            );
        }
    }
    serde_json::Value::Object(summary)
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
    enumeration_grounding_lookup_count: u64,
    candidate_surfaces: Vec<String>,
    candidate_diagnostics: Vec<serde_json::Value>,
    material_frame: Option<Result<MaterialFrameCaseV1, String>>,
    live_cohort_compare: Option<Result<LiveCohortProofOutcomeV1, String>>,
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
    exact_l11: Option<&ExactL11SurfaceIndexV1>,
    axis_schema: &super::transition_reduce::MorphologyAxisSchemaV1,
    frozen_manifest: &FrozenHypothesisIndexV1,
    material_package_tuple: ExactPackageTupleV1,
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
    let material_frame = (case.cohort == ProofCohortV1::LemmaHeldout
        && std::env::var_os("LAY_PRODUCTIVE_MATERIAL_FRAME_PROOF").is_some())
    .then(|| {
        evaluate_material_frame_case(
            case,
            runtime,
            canonical_l2,
            exact_l11,
            &groundings,
            &targets,
            hypothesis_covered,
            material_package_tuple,
        )
    });
    let mut live_cohort_compare = (case.cohort == ProofCohortV1::LemmaHeldout
        && std::env::var_os("LAY_PRODUCTIVE_LIVE_COHORT_COMPARE_PROOF").is_some())
    .then(|| evaluate_live_cohort_compare_case(case));
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
    let stage_telemetry_enabled = std::env::var_os("LAY_PRODUCTIVE_STAGE_TELEMETRY").is_some()
        || std::env::var_os("LAY_PRODUCTIVE_WORK_MEASUREMENT").is_some();
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
    if let Some(Ok(LiveCohortProofOutcomeV1::NoField(no_field))) = live_cohort_compare.as_mut() {
        let exact_l11 = exact_l11.as_ref().ok_or_else(|| {
            "live cohort NoField provenance requires the exact L1.1 surface index".to_string()
        })?;
        no_field.bind_target_provenance(
            exact_l11,
            canonical_l2,
            hypothesis_covered,
            target_exact_born,
        )?;
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
        enumeration_grounding_lookup_count: groundings.enumeration_grounding_lookup_count,
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
        material_frame,
        live_cohort_compare,
    })
}

fn evaluate_material_frame_case(
    case: &ProofCaseV1,
    runtime: &PackagedProductiveRuntimeV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    exact_l11: Option<&ExactL11SurfaceIndexV1>,
    groundings: &ProductiveGroundingsV1,
    targets: &[ValidTargetV1],
    hypothesis_covered: bool,
    package_tuple: ExactPackageTupleV1,
) -> Result<MaterialFrameCaseV1, String> {
    let diagnostics = groundings
        .target_cold_diagnostics
        .as_ref()
        .ok_or_else(|| "material-frame proof lacks target-blind diagnostics".to_string())?;
    let canonical_grounding = MaterialWorkCountersV1 {
        grounding_lookups: groundings.enumeration_grounding_lookup_count,
        ..MaterialWorkCountersV1::default()
    };
    let cold_binding = MaterialWorkCountersV1 {
        posting_visits: diagnostics.target_blind.posting_visit_count as u64,
        relation_replays: (diagnostics.target_blind.recovery_program_execution_count as u64)
            .checked_add(
                diagnostics
                    .target_blind
                    .exact_replay_program_execution_count as u64,
            )
            .ok_or_else(|| "material-frame cold replay count overflow".to_string())?,
        operator_steps: diagnostics.target_blind.operator_step_count,
        ..MaterialWorkCountersV1::default()
    };
    let preparatory_work = canonical_grounding
        .checked_add(cold_binding)
        .ok_or_else(|| "material-frame preparatory work overflow".to_string())?;
    let enumeration = runtime.enumerate_context_neutral_material(
        &case.damaged_surface,
        &groundings.grounded,
        &groundings.cold,
        preparatory_work,
        FROZEN_V90_ENUMERATION_WORK_BUDGET,
    );
    let productive_work = enumeration.productive_work;
    let aggregate_work = enumeration.aggregate_work;
    let work_budget_exceeded = enumeration.work_budget_exceeded;
    let contour_births = exact_l11.map_or_else(
        || enumerate_typed_contour_births(&case.damaged_surface, canonical_l2),
        |l11| enumerate_typed_contour_births_with_l11(&case.damaged_surface, canonical_l2, l11),
    );
    let contour_birth_count = contour_births.births.len();
    let contour_target_members = targets
        .iter()
        .filter(|target| {
            contour_births
                .births
                .iter()
                .any(|birth| birth.normalized_surface == target.surface)
        })
        .count();
    let contour_overflow = contour_births.overflow_reason.is_some();
    let contour_work = contour_births.work;
    let contour_report_work = EnumerationWorkCountersV1 {
        posting_visits: contour_work.posting_visits,
        relation_replays: contour_work.relation_replays,
        grounding_lookups: contour_work.grounding_lookups,
        generated_logical_targets: contour_work.generated_logical_targets,
        operator_steps: contour_work.operator_steps,
    };
    let contour_work_within_budget = contour_births.work_within_budget();
    let combined_work = aggregate_work
        .checked_add(contour_work)
        .ok_or_else(|| "material-frame aggregate contour work overflow".to_string())?;
    let boundary_births = enumerate_typed_boundary_births_from_packages(
        &case.damaged_surface,
        canonical_l2,
        exact_l11,
    );
    let boundary_birth_count = boundary_births.births.len();
    let boundary_overflow = boundary_births.overflow_reason.is_some();
    let boundary_work = boundary_births.work;
    let boundary_report_work = EnumerationWorkCountersV1 {
        posting_visits: boundary_work.posting_visits,
        relation_replays: boundary_work.relation_replays,
        grounding_lookups: boundary_work.grounding_lookups,
        generated_logical_targets: boundary_work.generated_logical_targets,
        operator_steps: boundary_work.operator_steps,
    };
    let boundary_work_within_budget = boundary_births.work_within_budget();
    let combined_work = combined_work
        .checked_add(boundary_work)
        .ok_or_else(|| "material-frame aggregate boundary work overflow".to_string())?;
    let material = prepare_context_neutral_productive_material_with_contours_and_boundaries(
        &case.damaged_surface,
        package_tuple,
        enumeration,
        contour_births,
        boundary_births,
    )?;
    let target_member = targets.iter().all(|target| {
        material
            .exact_target_surfaces()
            .any(|surface| surface == target.surface)
    });
    let completeness_state = material.completeness().state();
    let incompleteness_reason = material.completeness().reason();
    let complete_material = completeness_state == EnumerationStateV1::Complete;
    let explicit_incompleteness = completeness_state == EnumerationStateV1::Overflow
        && incompleteness_reason != IncompletenessReasonV1::None;
    let work_budget_respected = canonical_grounding
        .within(FROZEN_V90_ENUMERATION_WORK_BUDGET.canonical_grounding)
        && cold_binding.within(FROZEN_V90_ENUMERATION_WORK_BUDGET.cold_binding)
        && productive_work.within(FROZEN_V90_ENUMERATION_WORK_BUDGET.productive_traversal)
        && aggregate_work.within(FROZEN_V90_ENUMERATION_WORK_BUDGET.aggregate)
        && contour_work_within_budget
        && boundary_work_within_budget
        && material.work() == combined_work
        && !work_budget_exceeded;

    let mut contexts = Vec::with_capacity(3);
    for context in [
        String::new(),
        case.damaged_surface.clone(),
        case.event.observed_surface.clone(),
    ] {
        if !contexts.contains(&context) {
            contexts.push(context);
        }
    }
    if contexts.len() != 3 {
        return Err("material-frame proof case has fewer than three exact contexts".to_string());
    }
    let target_ref = targets
        .iter()
        .find_map(|target| {
            material
                .exact_target_surfaces()
                .position(|surface| surface == target.surface)
        })
        .or_else(|| material.exact_target_surfaces().next().map(|_| 0));
    let projected_target = target_ref
        .map(|target_ref| {
            material
                .exact_target_surfaces()
                .nth(target_ref)
                .map(str::to_string)
                .ok_or_else(|| "material-frame target reference cannot be dereferenced".to_string())
        })
        .transpose()?;
    let source_scalar_count = case.damaged_surface.chars().count();
    let package_generation = material.compact().key.package_generation;
    let digest_before = material.exact_digest();
    let mut arena = PreparedMaterialLeaseArenaV1::default();
    let mut frames = Vec::with_capacity(contexts.len());
    let mut leases = Vec::with_capacity(contexts.len());
    let mut bounds = Vec::with_capacity(contexts.len());
    let mut candidate_states = Vec::with_capacity(contexts.len());
    let mut original_preserve = 0_usize;
    let mut original_replace_permitted = 0_usize;
    let mut original_unresolved = 0_usize;
    let mut cohort_derivations = 0_usize;
    let mut cohort_context_mismatches = 0_usize;
    let mut cohort_winners = 0_usize;
    let mut cohort_ties = 0_usize;
    let mut cohort_abstains = 0_usize;
    let mut cohort_incomplete_winners = 0_usize;
    let mut cohort_false_singletons = 0_usize;
    let mut cohort_lost_grounded_targets = 0_usize;
    let mut cohort_multiple_component_authority = 0_usize;
    let mut cohort_preservation_bypass = 0_usize;
    let mut canonical_cohort = None;
    let witness_assessments_for = |target_ref: usize| {
        material.compact().targets.as_slice()[target_ref]
            .witnesses
            .witnesses()
            .iter()
            .enumerate()
            .map(|(index, witness)| WitnessFrameAssessmentV1 {
                material_witness_ref: index as u8,
                valid_geometry: matches!(
                    witness.verdict_membership,
                    VerdictMembershipV1::Grounded
                        | VerdictMembershipV1::L11Winner
                        | VerdictMembershipV1::L11Tied
                ),
                rejection: None,
            })
            .collect::<Vec<_>>()
    };
    let target_namespace_for = |target_ref: usize| {
        let witnesses = material.compact().targets.as_slice()[target_ref]
            .witnesses
            .witnesses();
        if witnesses.iter().any(|witness| {
            matches!(
                witness.verdict_membership,
                VerdictMembershipV1::Grounded
                    | VerdictMembershipV1::L11Winner
                    | VerdictMembershipV1::L11Tied
            )
        }) {
            TargetNamespaceSettlementV1::CompleteExactGrounding
        } else {
            TargetNamespaceSettlementV1::Incomplete(IncompletenessReasonV1::UpstreamIncomplete)
        }
    };
    let witness_assessments = target_ref.map(witness_assessments_for).unwrap_or_default();
    for (index, context) in contexts.into_iter().enumerate() {
        let identity = (index as u64).saturating_add(1);
        let field_generation = identity.saturating_add(100);
        let frame = ExactInputFrameV1::new(
            identity,
            identity.saturating_add(10),
            case.damaged_surface.clone(),
            context,
            source_scalar_count as u32,
            (source_scalar_count as u32, source_scalar_count as u32),
            String::new(),
            0,
            identity.saturating_add(20),
            identity.saturating_add(30),
            package_generation,
            field_generation,
        )
        .map_err(|reason| format!("material-frame exact frame rejected: {reason:?}"))?;
        let lease = arena
            .pin(
                &material,
                field_generation,
                1,
                [1, 1],
                10_000,
                LeaseConsumerStateV1::FrameSettlement,
            )
            .ok_or_else(|| "material-frame lease allocation failed".to_string())?;
        let original = derive_original_preservation_shadow(&material, lease, &frame, &frame, 1)
            .map_err(|error| format!("original-preservation shadow rejected: {error}"))?;
        match original.map(|preservation| preservation.verdict) {
            Some(FrameOriginalPreservationVerdictV1::Preserve) => original_preserve += 1,
            Some(FrameOriginalPreservationVerdictV1::ReplacePermitted) => {
                original_replace_permitted += 1
            }
            None => original_unresolved += 1,
        }
        if let (Some(target_ref), Some(projected_target)) = (target_ref, &projected_target) {
            let bound = bind_exact_frame_target(
                &material,
                lease,
                &frame,
                &frame,
                target_ref,
                0,
                source_scalar_count,
                0,
                0,
                1,
            )
            .map_err(|reason| format!("material-frame exact binding rejected: {reason:?}"))?;
            if bound.projected_target != *projected_target
                || bound.replayed_source_window != *projected_target
            {
                return Err("material-frame exact projection replay mismatch".to_string());
            }
            candidate_states.push(
                derive_candidate_validity_shadow(
                    &material,
                    lease,
                    &frame,
                    &frame,
                    &bound,
                    target_namespace_for(target_ref),
                    &witness_assessments,
                    1,
                )
                .map_err(|error| format!("candidate-state shadow rejected: {error}"))?,
            );
            bounds.push(bound);
        }

        let mut cohort_bounds = Vec::with_capacity(material.compact().targets.len());
        let mut cohort_states = Vec::with_capacity(material.compact().targets.len());
        for cohort_target_ref in 0..material.compact().targets.len() {
            let bound = bind_exact_frame_target(
                &material,
                lease,
                &frame,
                &frame,
                cohort_target_ref,
                0,
                source_scalar_count,
                0,
                0,
                1,
            )
            .map_err(|reason| format!("conflict-cohort exact binding rejected: {reason:?}"))?;
            let validity = derive_candidate_validity_shadow(
                &material,
                lease,
                &frame,
                &frame,
                &bound,
                target_namespace_for(cohort_target_ref),
                &witness_assessments_for(cohort_target_ref),
                1,
            )
            .map_err(|error| format!("conflict-cohort candidate state rejected: {error}"))?;
            cohort_bounds.push(bound);
            cohort_states.push(validity);
        }
        let bound_candidates = cohort_bounds.iter().zip(&cohort_states).collect::<Vec<_>>();
        let cohort =
            derive_conflict_cohort_shadow(&material, lease, &frame, &bound_candidates, original, 1)
                .map_err(|error| format!("conflict-cohort settlement rejected: {error}"))?;
        cohort_derivations += 1;
        match &cohort.verdict {
            CohortVerdictV1::Winner(winner_ref) => {
                cohort_winners += 1;
                let winner_is_grounded = cohort_states.iter().any(|candidate| {
                    candidate.material_target_ref == *winner_ref
                        && candidate.state == CandidateStateV1::Grounded
                });
                cohort_false_singletons += usize::from(
                    !cohort.complete_for_authority
                        || cohort.grounded_member_count != 1
                        || !winner_is_grounded,
                );
                cohort_incomplete_winners +=
                    usize::from(material.completeness().state() != EnumerationStateV1::Complete);
                cohort_multiple_component_authority += usize::from(cohort.component_count > 1);
                cohort_preservation_bypass += usize::from(!matches!(
                    original.map(|preservation| preservation.verdict),
                    Some(FrameOriginalPreservationVerdictV1::ReplacePermitted)
                ));
            }
            CohortVerdictV1::Tied { .. } => cohort_ties += 1,
            CohortVerdictV1::Abstain(_) => cohort_abstains += 1,
        }
        cohort_lost_grounded_targets += cohort_states
            .iter()
            .filter(|candidate| candidate.state == CandidateStateV1::Grounded)
            .filter(|candidate| {
                !cohort
                    .canonical_member_refs
                    .contains(&candidate.material_target_ref)
            })
            .count();
        let cohort_identity = (
            cohort.cohort_hash,
            cohort.canonical_member_refs.clone(),
            cohort.verdict.clone(),
        );
        if let Some(expected) = canonical_cohort.as_ref() {
            cohort_context_mismatches += usize::from(expected != &cohort_identity);
        } else {
            canonical_cohort = Some(cohort_identity);
        }
        leases.push(lease);
        frames.push(frame);
    }
    let context_bindings = bounds.len();
    let stale_reuse_attempts = bounds.len();
    let stale_reuse_accepts = target_ref.map_or(0, |target_ref| {
        (0..bounds.len())
            .filter(|index| {
                bind_exact_frame_target(
                    &material,
                    leases[*index],
                    &frames[*index],
                    &frames[(*index + 1) % frames.len()],
                    target_ref,
                    0,
                    source_scalar_count,
                    0,
                    0,
                    1,
                )
                .is_ok()
            })
            .count()
    });
    let stale_candidate_state_accepts = (0..bounds.len())
        .filter(|index| {
            derive_candidate_validity_shadow(
                &material,
                leases[*index],
                &frames[*index],
                &frames[(*index + 1) % frames.len()],
                &bounds[*index],
                target_namespace_for(target_ref.unwrap_or_default()),
                &witness_assessments,
                1,
            )
            .is_ok()
        })
        .count();
    let candidate_born = candidate_states
        .iter()
        .filter(|candidate| candidate.state == CandidateStateV1::Born)
        .count();
    let candidate_grounded = candidate_states
        .iter()
        .filter(|candidate| candidate.state == CandidateStateV1::Grounded)
        .count();
    let candidate_rejected = candidate_states
        .iter()
        .filter(|candidate| matches!(candidate.state, CandidateStateV1::Rejected(_)))
        .count();
    let candidate_false_grounding = candidate_states
        .iter()
        .filter(|candidate| {
            candidate.state == CandidateStateV1::Grounded && candidate.valid_grounded_witnesses == 0
        })
        .count();
    let candidate_cross_context_mismatches = candidate_states
        .first()
        .map(|first| {
            candidate_states
                .iter()
                .skip(1)
                .filter(|candidate| {
                    candidate.state != first.state
                        || candidate.authority_blockers != first.authority_blockers
                        || candidate.valid_grounded_witnesses != first.valid_grounded_witnesses
                        || candidate.rejected_witnesses != first.rejected_witnesses
                        || candidate.exact_projected_target_hash
                            != first.exact_projected_target_hash
                })
                .count()
        })
        .unwrap_or_default();
    Ok(MaterialFrameCaseV1 {
        proof_identity: case.event.proof_identity,
        damage_identity: damage_identity(
            &case.event.proof_identity,
            case.class,
            &case.damaged_surface,
        ),
        damage_class: case.class,
        hypothesis_covered,
        target_member,
        complete_material,
        explicit_incompleteness,
        incompleteness_reason,
        work_budget_respected,
        contour_births: contour_birth_count,
        contour_target_members,
        contour_overflow,
        contour_work: contour_report_work,
        boundary_births: boundary_birth_count,
        boundary_overflow,
        boundary_work: boundary_report_work,
        bindable_target: target_ref.is_some(),
        context_count: frames.len(),
        context_bindings,
        context_material_digest_exact: material.exact_digest() == digest_before,
        stale_reuse_attempts,
        stale_reuse_accepts,
        candidate_state_derivations: candidate_states.len(),
        candidate_born,
        candidate_grounded,
        candidate_rejected,
        candidate_false_grounding,
        candidate_cross_context_mismatches,
        stale_candidate_state_accepts,
        original_preserve,
        original_replace_permitted,
        original_unresolved,
        cohort_derivations,
        cohort_context_mismatches,
        cohort_winners,
        cohort_ties,
        cohort_abstains,
        cohort_incomplete_winners,
        cohort_false_singletons,
        cohort_lost_grounded_targets,
        cohort_multiple_component_authority,
        cohort_preservation_bypass,
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
        output.enumeration_grounding_lookup_count += 1;
        let descriptors = runtime.grounding_descriptors(identity.lemma_ref.0)?;
        if descriptors.is_empty() {
            if !cold_lemmas.insert(identity.lemma_ref.0) {
                continue;
            }
            let target = targets
                .iter()
                .find(|target| target.lemma_id == identity.lemma_ref.0);
            output.enumeration_grounding_lookup_count += 1;
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
            output.enumeration_grounding_lookup_count +=
                masked_observation.exact_source_forms.len() as u64;
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
            output.enumeration_grounding_lookup_count += 1;
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

#[cfg(test)]
mod frozen_proof_generation_tests {
    use super::*;

    #[test]
    fn generation_binding_accepts_only_exact_atomic_pairs() {
        assert_eq!(
            frozen_proof_generation_for_pair(
                FROZEN_V64_PACKAGE_SHA256,
                FROZEN_V64_L11_PACKAGE_SHA256,
            ),
            Some(FrozenProofGenerationV1::FrozenV64ManifestSource)
        );
        assert_eq!(
            frozen_proof_generation_for_pair(
                V9_BOUND_V90_PACKAGE_SHA256,
                ACTIVE_V9_L11_PACKAGE_SHA256,
            ),
            Some(FrozenProofGenerationV1::ActiveL11V9)
        );
        assert_eq!(
            frozen_proof_generation_for_pair(
                FROZEN_V64_PACKAGE_SHA256,
                ACTIVE_V9_L11_PACKAGE_SHA256,
            ),
            None
        );
        assert_eq!(
            frozen_proof_generation_for_pair(
                V9_BOUND_V90_PACKAGE_SHA256,
                FROZEN_V64_L11_PACKAGE_SHA256,
            ),
            None
        );
        assert_eq!(
            frozen_proof_generation_for_pair(&"0".repeat(64), ACTIVE_V9_L11_PACKAGE_SHA256),
            None
        );
    }

    #[test]
    fn active_v9_generation_cannot_create_a_replacement_manifest() {
        assert!(FrozenProofGenerationV1::FrozenV64ManifestSource.permits_manifest_creation());
        assert!(!FrozenProofGenerationV1::ActiveL11V9.permits_manifest_creation());
    }

    fn measured_case(id: u8, scale: u64) -> EnumerationWorkCaseV1 {
        let canonical_grounding = EnumerationWorkCountersV1 {
            grounding_lookups: scale,
            ..EnumerationWorkCountersV1::default()
        };
        let cold_binding = EnumerationWorkCountersV1 {
            posting_visits: scale * 2,
            relation_replays: scale * 3,
            operator_steps: scale * 4,
            ..EnumerationWorkCountersV1::default()
        };
        let productive_traversal = EnumerationWorkCountersV1 {
            relation_replays: scale * 5,
            generated_logical_targets: scale * 6,
            operator_steps: scale * 7,
            ..EnumerationWorkCountersV1::default()
        };
        let aggregate = canonical_grounding
            .checked_add(cold_binding)
            .and_then(|value| value.checked_add(productive_traversal))
            .expect("small work counters add");
        EnumerationWorkCaseV1 {
            proof_identity: [id; 32],
            damage_identity: [id.wrapping_add(1); 32],
            damage_class: "measurement_fixture",
            canonical_grounding,
            cold_binding,
            productive_traversal,
            aggregate,
        }
    }

    #[test]
    fn enumeration_work_summary_is_merge_order_invariant() {
        let forward = vec![measured_case(1, 3), measured_case(2, 5)];
        let mut reverse = forward.clone();
        reverse.reverse();

        assert_eq!(
            summarize_enumeration_work(forward, Vec::new(), 2, true),
            summarize_enumeration_work(reverse, Vec::new(), 2, true)
        );
    }

    #[test]
    fn enumeration_work_summary_rejects_an_inexact_aggregate() {
        let mut case = measured_case(1, 3);
        case.aggregate.operator_steps += 1;
        let summary = summarize_enumeration_work(vec![case], Vec::new(), 1, true);

        assert_eq!(summary["aggregate_exact"], false);
        assert_eq!(summary["complete"], false);
        assert_eq!(summary["verdict"], "INCOMPLETE_NOT_A_BUDGET");
    }

    fn produced_field_case(id: u8) -> LiveCohortCompareCaseV1 {
        LiveCohortCompareCaseV1 {
            class: "measurement_fixture",
            proof_identity: [id; 32],
            damaged_surface: "source".to_string(),
            target_surface: "target".to_string(),
            status: "READY",
            material_scope: "CONTEXT_SHAPED_OBSERVATION",
            legacy_kind: "ABSTAIN",
            cohort_kind: "ABSTAIN",
            first_divergence: None,
            field_candidate_count: 1,
            material_target_count: 1,
            retained_field_candidate_count: 1,
            grounded_l11_loss_count: 0,
            unretained_field_candidate_surfaces: Vec::new(),
            lost_grounded_l11_surfaces: Vec::new(),
            complete_for_authority: false,
            legacy_decision_parity_exact: true,
            field_producer_count: 1,
            cache_disposition: "produced",
            l11_us: 1,
            productive_v90_us: 1,
            bridge_total_us: 2,
            latency_us: 3,
        }
    }

    fn no_field_case(id: u8) -> LiveCohortNoFieldCaseV1 {
        LiveCohortNoFieldCaseV1 {
            class: "measurement_fixture",
            proof_identity: [id; 32],
            damaged_surface: "source".to_string(),
            target_surface: "target".to_string(),
            availability: "EMPTY_L11_LATTICE",
            field_producer_count: 0,
            cache_disposition: "not_requested",
            l11_us: 1,
            productive_v90_us: 0,
            bridge_total_us: 1,
            latency_us: 2,
            target_exact_l11: false,
            target_exact_v13: true,
            productive_hypothesis_covered: true,
            productive_exact_born: true,
            provenance_complete: true,
        }
    }

    #[test]
    fn live_cohort_summary_conserves_attempts_without_hiding_no_field_coverage() {
        let mut no_field = no_field_case(1);
        no_field.class = "second_measurement_fixture";
        let summary = summarize_live_cohort_compare_shadow(
            vec![produced_field_case(1)],
            vec![no_field],
            Vec::new(),
            2,
            true,
            7,
        );

        assert_eq!(summary["attempted_cases"], 2);
        assert_eq!(summary["unique_attempt_identities"], 2);
        assert_eq!(summary["attempt_denominator_conserved"], true);
        assert_eq!(summary["produced_field_cases"], 1);
        assert_eq!(summary["no_field_cases"], 1);
        assert_eq!(summary["status_counts"]["READY"], 1);
        assert_eq!(summary["no_field_provenance"]["records"], 1);
        assert_eq!(summary["no_field_provenance"]["incomplete_records"], 0);
        assert_eq!(summary["no_field_provenance"]["target_absent_l11"], 1);
        assert_eq!(summary["no_field_provenance"]["target_exact_v13"], 1);
        assert_eq!(summary["no_field_records"].as_array().unwrap().len(), 1);
        assert_eq!(summary["scoped_comparator_pass"], true);
        assert_eq!(summary["overall_pass"], false);
        assert_eq!(summary["promotion_eligible"], false);
    }

    #[test]
    fn live_cohort_summary_rejects_duplicate_or_error_outcomes() {
        let duplicate = summarize_live_cohort_compare_shadow(
            vec![produced_field_case(1)],
            vec![no_field_case(1)],
            Vec::new(),
            2,
            true,
            7,
        );
        assert_eq!(duplicate["attempt_denominator_conserved"], false);
        assert_eq!(duplicate["scoped_comparator_pass"], false);

        let error = summarize_live_cohort_compare_shadow(
            vec![produced_field_case(1)],
            Vec::new(),
            vec![LiveCohortCompareErrorV1 {
                class: "measurement_fixture",
                proof_identity: [2; 32],
                damaged_surface: "source".to_string(),
                target_surface: "target".to_string(),
                error: "transient source unavailable".to_string(),
            }],
            2,
            true,
            7,
        );
        assert_eq!(error["attempt_denominator_conserved"], true);
        assert_eq!(error["error_cases"], 1);
        assert_eq!(error["scoped_comparator_pass"], false);
    }

    #[test]
    fn live_cohort_summary_rejects_incomplete_no_field_provenance() {
        let mut no_field = no_field_case(2);
        no_field.provenance_complete = false;
        let summary = summarize_live_cohort_compare_shadow(
            vec![produced_field_case(1)],
            vec![no_field],
            Vec::new(),
            2,
            true,
            7,
        );

        assert_eq!(summary["attempt_denominator_conserved"], true);
        assert_eq!(summary["no_field_provenance"]["incomplete_records"], 1);
        assert_eq!(summary["scoped_comparator_pass"], false);
        assert_eq!(summary["promotion_eligible"], false);
    }
}
