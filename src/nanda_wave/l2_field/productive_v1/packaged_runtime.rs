use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use super::anchor_recovery_package::{recovery_sidecar_path, AnchorRecoveryPackageViewV1};
use super::calibrate::{
    calibrated_readout_packaged, CandidateProvenanceClassV1, CandidateRankOriginV1,
    ObservableCalibrationStratumV1, ProductiveCalibratedVerdictV1, ReadoutCandidateV1,
    AMBIGUITY_CROSS_LEMMA_BASIN, AMBIGUITY_GENERATED_OVERFLOW, AMBIGUITY_SAME_LEMMA_MULTI_LABEL,
    AMBIGUITY_SYNCRHETIC_SLOT,
};
use super::format::{ProductiveAlgorithmModeV1, ProductivePackageViewV1, ProductiveSectionKindV1};
use super::geometry::{
    BatchGeometryEvaluatorV1, GeometryPathIdentityV1, GeometryTerminalEvidenceV1,
    GeometryTraversalStateV1, ObservedGeometryV1,
};
use super::induce::{SourceAnchorV1, COPY_TO_RETAINED_EDGE};
use super::phase::integer_cosine;
use super::records::{
    CalibrationCellRecordV1, DirectionalResidualRecordV1, EvidencePriorRecordV1, FixedRecordV1,
    ModelCoefficientRecordV1, MorphOpRecordV1, MorphOpcodeV1, MorphProgramHeaderRecordV1,
    ParadigmCenterRecordV1, ParadigmCompatibilityIndexRecordV1, ParadigmPostingRecordV1,
    PhaseCenterRecordV1, ProductiveTerminalRecordV1, ProductiveTrieArcOpcodeV1,
    ProductiveTrieArcRecordV1, ProductiveTrieNodeRecordV1, SlotPhaseProfileRecordV1,
};
use super::runtime::{
    emit_scalar, resolve_source_offset, ScalarTraceArenaV1, TraversalFrameV1,
    PRODUCTIVE_PHYSICAL_TOP_K,
};
use super::scene::{directional_scene_key, encode_scene_wave, L2LocalSceneV1, SceneWaveV1};
use super::score::{
    extract_feature_vector, fixed_point_score_q16, productive_feature_schema_hash_low,
    CountEvidenceV1, QuantizedFeatureVectorV1, TerminalFeatureInputV1, PRODUCTIVE_FEATURE_COUNT,
};
use super::semantic_estimator::SemanticExecutionIndexV1;
use super::types::MorphologySlotKeyV1;
use super::types::{LemmaParadigmBindingV1, ProductiveCandidateIdentityV1};
use crate::typing_transition::target_evidence::{
    EnumerationWorkBudgetV1, EnumerationWorkCountersV1,
};

const PRODUCTIVE_HEAP_WITH_OVERFLOW_SENTINEL: usize = super::runtime::PRODUCTIVE_PHYSICAL_TOP_K + 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PackagedGroundedLemmaV1 {
    pub(super) lemma_id: u32,
    pub(super) pos_domain: u16,
    pub(super) canonical_source_form_ref: u32,
    pub(super) source_slot_id: u32,
    pub(super) normalized_source: String,
    pub(super) grounded_support: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ColdLemmaSourceV1 {
    pub(super) pos_domain: u16,
    pub(super) canonical_source_form_ref: u32,
    pub(super) source_slot_id: u32,
    pub(super) normalized_source: String,
    pub(super) grounded_support: u32,
    pub(super) canonical_preference: u8,
    pub(super) canonical_source: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ColdLemmaBindingV1 {
    lemma: PackagedGroundedLemmaV1,
    paradigm_id: u32,
    observed_slots: Vec<u32>,
    recovered_anchor: bool,
    cross_lane_certified: bool,
}

impl ColdLemmaBindingV1 {
    pub(super) const fn lemma_id(&self) -> u32 {
        self.lemma.lemma_id
    }

    pub(super) const fn paradigm_id(&self) -> u32 {
        self.paradigm_id
    }

    pub(super) const fn pos_domain(&self) -> u16 {
        self.lemma.pos_domain
    }

    pub(super) const fn source_slot_id(&self) -> u32 {
        self.lemma.source_slot_id
    }

    pub(super) fn normalized_source(&self) -> &str {
        &self.lemma.normalized_source
    }

    fn validate(&self, maximum_source_scalars: u16) -> Result<(), &'static str> {
        self.lemma.validate(maximum_source_scalars)?;
        if self.paradigm_id == 0
            || self.observed_slots.is_empty()
            || !self.observed_slots.windows(2).all(|pair| pair[0] < pair[1])
            || !self.recovered_anchor
                && self
                    .observed_slots
                    .binary_search(&self.lemma.source_slot_id)
                    .is_err()
            || !self.recovered_anchor && self.cross_lane_certified
        {
            return Err("productive cold lemma binding is malformed");
        }
        Ok(())
    }

    fn into_active(self) -> ActiveBindingV1 {
        ActiveBindingV1 {
            lemma: self.lemma,
            binding: None,
            paradigm_id: self.paradigm_id,
            observed_slots: self.observed_slots,
            provenance: CandidateProvenanceClassV1::ColdLemmaBinding,
            rank_origin: if self.recovered_anchor {
                CandidateRankOriginV1::RecoveredV66
            } else {
                CandidateRankOriginV1::BaseV64
            },
            cross_lane_certified: self.cross_lane_certified,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct RecoveryIdentityAnchorRefV1 {
    pub(super) paradigm_id: u32,
    pub(super) source_slot_id: u32,
    pub(super) canonical_source_form_ref: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ColdBindingDerivationDiagnosticsV1 {
    pub(super) source_count: usize,
    pub(super) source_pos_domain_count: usize,
    pub(super) observed_slot_count: usize,
    pub(super) posting_lookup_count: usize,
    pub(super) posting_visit_count: usize,
    pub(super) posting_miss_count: usize,
    pub(super) posting_paradigm_count: usize,
    pub(super) slot_compatible_paradigm_count: usize,
    pub(super) exact_reconstructing_paradigm_count: usize,
    pub(super) retained_binding_count: usize,
    pub(super) recovery_lookup_count: usize,
    pub(super) recovery_path_count: usize,
    pub(super) recovered_anchor_count: usize,
    pub(super) recovery_exact_reconstructing_count: usize,
    pub(super) structural_eligible_paradigm_count: usize,
    pub(super) recovery_post_intersection_count: usize,
    pub(super) recovery_program_execution_count: usize,
    pub(super) recovery_unique_anchor_count: usize,
    pub(super) recovery_post_frontier_anchor_count: usize,
    pub(super) recovery_frontier_dropped_count: usize,
    pub(super) recovery_max_independent_source_count: usize,
    pub(super) identity_bridge_candidate_count: usize,
    pub(super) exact_replay_program_execution_count: usize,
    pub(super) operator_step_count: u64,
    pub(super) shared_hypothesis_observation_count: usize,
    pub(super) shared_hypothesis_unique_count: usize,
    pub(super) shared_hypothesis_join_attempt_count: usize,
    pub(super) shared_hypothesis_exact_count: usize,
    pub(super) shared_hypothesis_replay_execution_count: usize,
    pub(super) transition_equivalence_class_count: usize,
    pub(super) transition_equivalence_owner_count: usize,
    pub(super) transition_equivalence_max_class_size: usize,
    pub(super) transition_equivalence_representative_replay_count: usize,
    pub(super) transition_equivalence_exact_class_count: usize,
    pub(super) transition_equivalence_exact_owner_fanout_count: usize,
    pub(super) source_pos_domains: BTreeSet<u16>,
    pub(super) posting_paradigm_ids: BTreeSet<u32>,
    pub(super) slot_compatible_paradigm_ids: BTreeSet<u32>,
    pub(super) exact_reconstructing_paradigm_ids: BTreeSet<u32>,
    pub(super) retained_paradigm_ids: BTreeSet<u32>,
    pub(super) recovery_posting_paradigm_ids: BTreeSet<u32>,
    pub(super) recovered_anchor_paradigm_ids: BTreeSet<u32>,
    pub(super) recovery_post_frontier_paradigm_ids: BTreeSet<u32>,
    pub(super) recovery_exact_paradigm_ids: BTreeSet<u32>,
    pub(super) identity_anchors_pre_frontier: BTreeSet<RecoveryIdentityAnchorRefV1>,
    pub(super) identity_anchors_post_frontier: BTreeSet<RecoveryIdentityAnchorRefV1>,
    pub(super) identity_anchors_exact: BTreeSet<RecoveryIdentityAnchorRefV1>,
}

#[derive(Clone, Debug)]
struct RecoveredAnchorCandidateV1 {
    paradigm_id: u32,
    paradigm: ParadigmCenterRecordV1,
    source: ColdLemmaSourceV1,
    independent_sources: BTreeSet<usize>,
    learned_recovery: bool,
    maximum_train_lemma_support: u32,
    maximum_stability: u16,
    exact_certified: bool,
    shared_hypothesis: bool,
}

#[derive(Clone, Debug)]
struct SharedAnchorHypothesisV1 {
    source: ColdLemmaSourceV1,
    independent_sources: BTreeSet<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SharedReplayConstraintV1 {
    pub(super) slot_id: u32,
    pub(super) normalized_surface: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SharedHypothesisReplayAuditV1 {
    pub(super) pos_domain: u16,
    pub(super) anchor_slot_id: u32,
    pub(super) normalized_source: String,
    pub(super) constraints: Vec<SharedReplayConstraintV1>,
    pub(super) eligible_paradigm_ids: Vec<u32>,
    pub(super) direct_exact_paradigm_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TransitionOperationSignatureV1 {
    opcode: u8,
    anchor: u8,
    flags: u16,
    arg0: i32,
    arg1: u32,
    arg2: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TransitionProgramSignatureV1 {
    source_slot_id: u32,
    target_slot_id: u32,
    flags: u16,
    operations: Vec<TransitionOperationSignatureV1>,
}

#[derive(Clone, Debug)]
struct TransitionEquivalenceClassV1 {
    representative: ParadigmCenterRecordV1,
    owners: Vec<(u32, ParadigmCenterRecordV1)>,
}

#[derive(Clone, Debug)]
struct ExposedSlotConstraintsV1 {
    slot_id: u32,
    match_start: usize,
    surfaces: Vec<String>,
}

#[derive(Clone, Debug, Default)]
struct ExposedFormConstraintsV1 {
    slots: Vec<ExposedSlotConstraintsV1>,
    surface_count: usize,
}

impl ExposedFormConstraintsV1 {
    fn insert(&mut self, slot_id: u32, surface: &str) {
        let slot_index = match self
            .slots
            .binary_search_by_key(&slot_id, |constraints| constraints.slot_id)
        {
            Ok(index) => index,
            Err(index) => {
                let match_start = index
                    .checked_sub(1)
                    .and_then(|previous| self.slots.get(previous))
                    .map_or(0, |previous| previous.match_start + previous.surfaces.len());
                self.slots.insert(
                    index,
                    ExposedSlotConstraintsV1 {
                        slot_id,
                        match_start,
                        surfaces: Vec::new(),
                    },
                );
                index
            }
        };
        let surfaces = &mut self.slots[slot_index].surfaces;
        let surface_index =
            match surfaces.binary_search_by(|expected| expected.as_str().cmp(surface)) {
                Ok(_) => return,
                Err(index) => index,
            };
        surfaces.insert(surface_index, surface.to_string());
        self.surface_count += 1;
        for constraints in &mut self.slots[slot_index + 1..] {
            constraints.match_start += 1;
        }
    }

    fn slot(&self, slot_id: u32) -> Option<&ExposedSlotConstraintsV1> {
        self.slots
            .binary_search_by_key(&slot_id, |constraints| constraints.slot_id)
            .ok()
            .map(|index| &self.slots[index])
    }
}

impl RecoveredAnchorCandidateV1 {
    fn observe_learned_path(
        &mut self,
        source_ordinal: usize,
        train_lemma_support: u32,
        stability: u16,
    ) {
        self.independent_sources.insert(source_ordinal);
        self.learned_recovery = true;
        self.maximum_train_lemma_support =
            self.maximum_train_lemma_support.max(train_lemma_support);
        self.maximum_stability = self.maximum_stability.max(stability);
    }

    fn evidence_order(left: &Self, right: &Self) -> Ordering {
        right
            .exact_certified
            .cmp(&left.exact_certified)
            .then_with(|| right.shared_hypothesis.cmp(&left.shared_hypothesis))
            .then_with(|| {
                right
                    .independent_sources
                    .len()
                    .cmp(&left.independent_sources.len())
            })
            .then_with(|| right.learned_recovery.cmp(&left.learned_recovery))
            .then_with(|| {
                right
                    .maximum_train_lemma_support
                    .cmp(&left.maximum_train_lemma_support)
            })
            .then_with(|| right.maximum_stability.cmp(&left.maximum_stability))
            .then_with(|| right.paradigm.support.cmp(&left.paradigm.support))
            .then_with(|| right.paradigm.stability.cmp(&left.paradigm.stability))
            .then_with(|| {
                left.source
                    .canonical_preference
                    .cmp(&right.source.canonical_preference)
            })
            .then_with(|| {
                right
                    .source
                    .grounded_support
                    .cmp(&left.source.grounded_support)
            })
            .then_with(|| left.paradigm_id.cmp(&right.paradigm_id))
            .then_with(|| left.source.source_slot_id.cmp(&right.source.source_slot_id))
            .then_with(|| {
                left.source
                    .normalized_source
                    .cmp(&right.source.normalized_source)
            })
    }
}

impl From<&RecoveredAnchorCandidateV1> for RecoveryIdentityAnchorRefV1 {
    fn from(candidate: &RecoveredAnchorCandidateV1) -> Self {
        Self {
            paradigm_id: candidate.paradigm_id,
            source_slot_id: candidate.source.source_slot_id,
            canonical_source_form_ref: candidate.source.canonical_source_form_ref,
        }
    }
}

impl PackagedGroundedLemmaV1 {
    fn validate(&self, maximum_source_scalars: u16) -> Result<(), &'static str> {
        if self.pos_domain == 0
            || self.source_slot_id == 0
            || self.normalized_source.is_empty()
            || self.grounded_support == 0
        {
            return Err("productive grounded lemma input has a zero identity or support");
        }
        if self.normalized_source.chars().count() > usize::from(maximum_source_scalars) {
            return Err("productive grounded source exceeds the package scalar bound");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PackagedProductiveCandidateV1 {
    pub(super) identity: ProductiveCandidateIdentityV1,
    pub(super) equivalent_identities: Vec<ProductiveCandidateIdentityV1>,
    pub(super) normalized_surface: Arc<str>,
    pub(super) score_q16: i64,
    pub(super) geometry: GeometryTerminalEvidenceV1,
    pub(super) provenance: CandidateProvenanceClassV1,
    pub(super) minimum_independent_support: u32,
    pub(super) grounded_support: u32,
    pub(super) ambiguity_center_cosine: i64,
    pub(super) equivalent_identity_count: u32,
    pub(super) equivalent_paradigm_count: u32,
    pub(super) minimum_equivalent_support: u32,
    pub(super) maximum_equivalent_support: u32,
    pub(super) rank_origin: CandidateRankOriginV1,
    pub(super) cross_lane_certified: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PackagedProductiveReadoutV1 {
    pub(super) verdict: ProductiveCalibratedVerdictV1,
    pub(super) candidates: Vec<PackagedProductiveCandidateV1>,
    pub(super) logical_terminal_count: u64,
    pub(super) logical_surface_basin_count: u64,
    pub(super) integrity_error: Option<String>,
}

fn empty_productive_readout() -> PackagedProductiveReadoutV1 {
    PackagedProductiveReadoutV1 {
        verdict: ProductiveCalibratedVerdictV1::Abstain {
            suggestions: Vec::new(),
            productive_overflow: false,
        },
        candidates: Vec::new(),
        logical_terminal_count: 0,
        logical_surface_basin_count: 0,
        integrity_error: None,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub(super) struct ProductiveEvaluationTelemetryV1 {
    pub(super) setup_us: u64,
    pub(super) binding_preparation_us: u64,
    pub(super) traversal_us: u64,
    pub(super) surface_reduce_us: u64,
    pub(super) final_readout_us: u64,
    pub(super) active_binding_count: u64,
    pub(super) logical_terminal_count: u64,
    pub(super) logical_surface_basin_count: u64,
    pub(super) selected_candidate_count: u64,
    pub(super) relation_replay_count: u64,
    pub(super) operator_step_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProductiveEvaluationModeV1 {
    ContextShaped,
    ContextNeutralMaterial,
}

#[derive(Clone, Debug)]
pub(super) struct ContextNeutralProductiveEnumerationV1 {
    pub(super) readout: PackagedProductiveReadoutV1,
    pub(super) productive_work: EnumerationWorkCountersV1,
    pub(super) aggregate_work: EnumerationWorkCountersV1,
    pub(super) work_budget_exceeded: bool,
}

struct ProductiveWorkLimiterV1 {
    productive_ceiling: EnumerationWorkCountersV1,
    aggregate_ceiling: EnumerationWorkCountersV1,
    preparatory_work: EnumerationWorkCountersV1,
    productive_work: EnumerationWorkCountersV1,
}

impl ProductiveWorkLimiterV1 {
    fn new(
        budget: EnumerationWorkBudgetV1,
        preparatory_work: EnumerationWorkCountersV1,
    ) -> Result<Self, String> {
        if !preparatory_work.within(budget.aggregate) {
            return Err(WORK_BUDGET_EXCEEDED.to_string());
        }
        Ok(Self {
            productive_ceiling: budget.productive_traversal,
            aggregate_ceiling: budget.aggregate,
            preparatory_work,
            productive_work: EnumerationWorkCountersV1::default(),
        })
    }

    fn consume(&mut self, delta: EnumerationWorkCountersV1) -> Result<(), String> {
        let next_productive = self
            .productive_work
            .checked_add(delta)
            .ok_or_else(|| WORK_BUDGET_EXCEEDED.to_string())?;
        let next_aggregate = self
            .preparatory_work
            .checked_add(next_productive)
            .ok_or_else(|| WORK_BUDGET_EXCEEDED.to_string())?;
        if !next_productive.within(self.productive_ceiling)
            || !next_aggregate.within(self.aggregate_ceiling)
        {
            return Err(WORK_BUDGET_EXCEEDED.to_string());
        }
        self.productive_work = next_productive;
        Ok(())
    }

    fn aggregate_work(&self) -> EnumerationWorkCountersV1 {
        self.preparatory_work
            .checked_add(self.productive_work)
            .unwrap_or(EnumerationWorkCountersV1 {
                posting_visits: u64::MAX,
                relation_replays: u64::MAX,
                grounding_lookups: u64::MAX,
                generated_logical_targets: u64::MAX,
                operator_steps: u64::MAX,
            })
    }
}

const WORK_BUDGET_EXCEEDED: &str = "productive context-neutral work budget exceeded";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProductiveTargetProbeIdentityV1 {
    pub(super) lemma_id: u32,
    pub(super) target_slot_id: u32,
    pub(super) normalized_surface: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ProductiveTargetProbeV1 {
    targets: Vec<ProductiveTargetProbeIdentityV1>,
    pub(super) exact_pre_slot_bound: bool,
    pub(super) exact_post_slot_bound: bool,
    pub(super) exact_post_surface_basin_bound: bool,
}

impl ProductiveTargetProbeV1 {
    pub(super) fn new(mut targets: Vec<ProductiveTargetProbeIdentityV1>) -> Self {
        targets.sort_by(|left, right| {
            (left.lemma_id, left.target_slot_id, &left.normalized_surface).cmp(&(
                right.lemma_id,
                right.target_slot_id,
                &right.normalized_surface,
            ))
        });
        targets.dedup();
        Self {
            targets,
            ..Self::default()
        }
    }

    fn matches(&self, candidate: &PackagedProductiveCandidateV1) -> bool {
        self.targets.iter().any(|target| {
            (candidate.identity.lemma_id == target.lemma_id
                && candidate.identity.target_slot_id == target.target_slot_id
                || candidate.equivalent_identities.iter().any(|identity| {
                    identity.lemma_id == target.lemma_id
                        && identity.target_slot_id == target.target_slot_id
                }))
                && candidate.normalized_surface.as_ref() == target.normalized_surface
        })
    }

    fn observe_pre_slot_bound(&mut self, candidate: &PackagedProductiveCandidateV1) {
        self.exact_pre_slot_bound |= self.matches(candidate);
    }

    fn observe_post_slot_bound(&mut self, candidate: &PackagedProductiveCandidateV1) {
        self.exact_post_slot_bound |= self.matches(candidate);
    }

    fn observe_post_surface_basin_bound(&mut self, candidate: &PackagedProductiveCandidateV1) {
        self.exact_post_surface_basin_bound |= self.matches(candidate);
    }
}

#[derive(Clone, Copy, Debug)]
struct RuntimePriorV1 {
    positive: f64,
    contradiction: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SharedReplayModeV1 {
    Legacy,
    ShadowCompare,
    SemanticProofAuthority,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PackagedGroundingDescriptorV1 {
    pub(super) lemma_id: u32,
    pub(super) pos_domain: u16,
    pub(super) canonical_source_form_ref: u32,
    pub(super) source_slot_id: u32,
    pub(super) grounded_support: u32,
}

#[derive(Clone, Debug)]
pub(in crate::nanda_wave::l2_field) struct PackagedProductiveRuntimeV1 {
    package: ProductivePackageViewV1,
    anchor_recovery: Option<AnchorRecoveryPackageViewV1>,
    semantic_transducer: Option<SemanticExecutionIndexV1>,
    shared_replay_mode: SharedReplayModeV1,
    coefficients_q16: [i32; PRODUCTIVE_FEATURE_COUNT],
    priors: [RuntimePriorV1; 4],
    paradigms: Box<[ParadigmCenterRecordV1]>,
    programs: Box<[PreparedMorphProgramV1]>,
    operations: Box<[MorphOpRecordV1]>,
    terminals: Box<[ProductiveTerminalRecordV1]>,
    slot_profiles: Box<[SlotPhaseProfileRecordV1]>,
    terminal_index_by_program: Box<[u32]>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PreparedMorphProgramV1 {
    record: MorphProgramHeaderRecordV1,
    suffix_drop: u16,
}

#[derive(Clone, Debug)]
struct StructurallyEligibleParadigmsV1 {
    ordered: Vec<(u32, ParadigmCenterRecordV1)>,
    membership: Vec<u8>,
}

impl StructurallyEligibleParadigmsV1 {
    fn len(&self) -> usize {
        self.ordered.len()
    }

    fn iter(&self) -> impl Iterator<Item = &(u32, ParadigmCenterRecordV1)> {
        self.ordered.iter()
    }

    fn contains(&self, paradigm_id: u32) -> bool {
        self.membership
            .get(paradigm_id as usize)
            .is_some_and(|present| *present != 0)
    }
}

const PARADIGM_FLAG_POSTING: u8 = 1 << 0;
const PARADIGM_FLAG_SLOT_COMPATIBLE: u8 = 1 << 1;
const PARADIGM_FLAG_EXACT_RECONSTRUCTING: u8 = 1 << 2;
const PARADIGM_FLAG_RECOVERY_POSTING: u8 = 1 << 3;
const PARADIGM_FLAG_RECOVERED_ANCHOR: u8 = 1 << 4;
const PARADIGM_FLAG_RECOVERY_EXACT: u8 = 1 << 5;
const PARADIGM_FLAG_DIRECT_SELECTED: u8 = 1 << 6;

#[derive(Clone, Debug)]
struct DenseParadigmFlagsV1 {
    flags: Vec<u8>,
}

impl DenseParadigmFlagsV1 {
    fn new(paradigm_count: usize) -> Result<Self, String> {
        let lane_len = paradigm_count
            .checked_add(1)
            .ok_or_else(|| "productive paradigm flag length overflows".to_string())?;
        Ok(Self {
            flags: vec![0_u8; lane_len],
        })
    }

    fn mark(&mut self, paradigm_id: u32, flag: u8) -> Result<(), String> {
        let value = self
            .flags
            .get_mut(paradigm_id as usize)
            .ok_or_else(|| "productive paradigm flag identity exceeds package".to_string())?;
        if paradigm_id == 0 {
            return Err("productive paradigm flag identity is zero".to_string());
        }
        *value |= flag;
        Ok(())
    }

    fn contains(&self, paradigm_id: u32, flag: u8) -> bool {
        self.flags
            .get(paradigm_id as usize)
            .is_some_and(|value| value & flag != 0)
    }

    fn ids_with(&self, flag: u8) -> Result<BTreeSet<u32>, String> {
        self.flags
            .iter()
            .copied()
            .enumerate()
            .skip(1)
            .filter(|(_, value)| value & flag != 0)
            .map(|(index, _)| {
                u32::try_from(index)
                    .map_err(|_| "productive paradigm flag identity exceeds u32".to_string())
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
struct ActiveBindingV1 {
    lemma: PackagedGroundedLemmaV1,
    binding: Option<LemmaParadigmBindingV1>,
    paradigm_id: u32,
    observed_slots: Vec<u32>,
    provenance: CandidateProvenanceClassV1,
    rank_origin: CandidateRankOriginV1,
    cross_lane_certified: bool,
}

impl ActiveBindingV1 {
    fn positive_support(&self) -> u32 {
        self.binding.map_or(0, |binding| binding.positive_support)
    }

    fn explicit_anti_support(&self) -> u32 {
        self.binding
            .map_or(0, |binding| binding.explicit_anti_support)
    }

    fn stability(&self) -> u16 {
        self.binding.map_or(0, |binding| binding.stability)
    }

    fn candidate_provenance(&self, target_slot_id: u32) -> CandidateProvenanceClassV1 {
        match self.provenance {
            CandidateProvenanceClassV1::ColdLemmaBinding => {
                CandidateProvenanceClassV1::ColdLemmaBinding
            }
            _ if self.observed_slots.binary_search(&target_slot_id).is_ok() => {
                CandidateProvenanceClassV1::TrainingSeenGenerated
            }
            _ => CandidateProvenanceClassV1::UnobservedLemmaSlot,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PreparedSlotEvaluationV1 {
    profile: SlotPhaseProfileRecordV1,
    invariant_features: QuantizedFeatureVectorV1,
    ambiguity_center_cosine: i64,
    minimum_independent_support: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RankedCandidateV1 {
    output: PackagedProductiveCandidateV1,
    exact_osa_distance: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SurfaceBasinKeyV1 {
    lemma_id: u32,
    target_slot_id: Option<u32>,
    normalized_surface: Arc<str>,
}

impl SurfaceBasinKeyV1 {
    fn from_candidate(candidate: &RankedCandidateV1, coalesce_slots: bool) -> Self {
        Self {
            lemma_id: candidate.output.identity.lemma_id,
            target_slot_id: (!coalesce_slots).then_some(candidate.output.identity.target_slot_id),
            normalized_surface: Arc::clone(&candidate.output.normalized_surface),
        }
    }
}

#[derive(Clone, Debug)]
struct SurfaceBasinV1 {
    representative: RankedCandidateV1,
    additional_identities: Vec<ProductiveCandidateIdentityV1>,
    minimum_support: u32,
    maximum_support: u32,
    maximum_ambiguity_center_cosine: i64,
    cross_lane_certified: bool,
}

impl SurfaceBasinV1 {
    fn new(mut candidate: RankedCandidateV1) -> Self {
        let identity = candidate.output.identity;
        let mut additional_identities = std::mem::take(&mut candidate.output.equivalent_identities);
        additional_identities.retain(|equivalent| *equivalent != identity);
        let support = candidate.output.minimum_independent_support;
        let ambiguity = candidate.output.ambiguity_center_cosine;
        let cross_lane_certified = candidate.output.cross_lane_certified;
        Self {
            representative: candidate,
            additional_identities,
            minimum_support: support,
            maximum_support: support,
            maximum_ambiguity_center_cosine: ambiguity,
            cross_lane_certified,
        }
    }

    fn merge(&mut self, mut candidate: RankedCandidateV1) {
        let representative_identity = self.representative.output.identity;
        let identity = candidate.output.identity;
        self.additional_identities.push(identity);
        self.additional_identities
            .append(&mut candidate.output.equivalent_identities);
        self.minimum_support = self
            .minimum_support
            .min(candidate.output.minimum_independent_support);
        self.maximum_support = self
            .maximum_support
            .max(candidate.output.minimum_independent_support);
        self.maximum_ambiguity_center_cosine = self
            .maximum_ambiguity_center_cosine
            .max(candidate.output.ambiguity_center_cosine);
        self.cross_lane_certified |= candidate.output.cross_lane_certified;
        if ranked_candidate_order(&candidate, &self.representative).is_lt() {
            self.additional_identities.push(representative_identity);
            self.representative = candidate;
        }
    }

    fn absorb_identities_only(&mut self, mut other: Self) {
        self.additional_identities
            .push(other.representative.output.identity);
        self.additional_identities
            .append(&mut other.additional_identities);
    }

    fn into_representative(mut self) -> RankedCandidateV1 {
        self.additional_identities
            .push(self.representative.output.identity);
        self.additional_identities.sort_unstable();
        self.additional_identities.dedup();
        let mut paradigms = self
            .additional_identities
            .iter()
            .map(|identity| identity.paradigm_id)
            .collect::<Vec<_>>();
        paradigms.sort_unstable();
        paradigms.dedup();
        self.representative.output.equivalent_identities = self.additional_identities;
        self.representative.output.equivalent_identity_count =
            u32::try_from(self.representative.output.equivalent_identities.len())
                .unwrap_or(u32::MAX);
        self.representative.output.equivalent_paradigm_count =
            u32::try_from(paradigms.len()).unwrap_or(u32::MAX);
        self.representative.output.minimum_equivalent_support = self.minimum_support;
        self.representative.output.maximum_equivalent_support = self.maximum_support;
        self.representative.output.ambiguity_center_cosine = self.maximum_ambiguity_center_cosine;
        self.representative.output.cross_lane_certified = self.cross_lane_certified;
        self.representative
    }
}

struct BindingCandidateFrontierV1 {
    slots: Vec<BindingSlotFrontierV1>,
}

struct BindingSlotFrontierV1 {
    slot_id: u32,
    candidates: Vec<RankedCandidateV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DenseProgramExecutionV1 {
    Rejected,
    Accepted {
        normalized_surface: Arc<str>,
        geometry: GeometryTerminalEvidenceV1,
    },
}

struct DenseProgramExecutionLaneV1 {
    value_index_by_class: Vec<u32>,
    touched_classes: Vec<u32>,
    values: Vec<DenseProgramExecutionV1>,
}

impl DenseProgramExecutionLaneV1 {
    fn new(execution_class_count: usize) -> Result<Self, String> {
        let lane_len = execution_class_count
            .checked_add(1)
            .ok_or_else(|| "productive semantic execution lane overflows".to_string())?;
        Ok(Self {
            value_index_by_class: vec![0; lane_len],
            touched_classes: Vec::new(),
            values: Vec::new(),
        })
    }

    fn begin_source(&mut self) {
        for class_id in self.touched_classes.drain(..) {
            self.value_index_by_class[class_id as usize] = 0;
        }
        self.values.clear();
    }

    fn get(&self, class_id: u32) -> Result<Option<&DenseProgramExecutionV1>, String> {
        let value_index = *self
            .value_index_by_class
            .get(class_id as usize)
            .filter(|_| class_id != 0)
            .ok_or_else(|| "productive semantic execution class exceeds lane".to_string())?;
        if value_index == 0 {
            return Ok(None);
        }
        self.values
            .get(value_index as usize - 1)
            .map(Some)
            .ok_or_else(|| "productive semantic execution value index is invalid".to_string())
    }

    fn insert(&mut self, class_id: u32, execution: DenseProgramExecutionV1) -> Result<(), String> {
        let value_index = self
            .value_index_by_class
            .get_mut(class_id as usize)
            .filter(|_| class_id != 0)
            .ok_or_else(|| "productive semantic execution class exceeds lane".to_string())?;
        if *value_index != 0 {
            return Err("productive semantic execution class inserted twice".to_string());
        }
        let next_index = u32::try_from(self.values.len() + 1)
            .map_err(|_| "productive semantic execution values exceed u32".to_string())?;
        *value_index = next_index;
        self.touched_classes.push(class_id);
        self.values.push(execution);
        Ok(())
    }
}

impl BindingCandidateFrontierV1 {
    fn new(slot_ids: impl IntoIterator<Item = u32>) -> Result<Self, String> {
        let slots = slot_ids
            .into_iter()
            .map(|slot_id| BindingSlotFrontierV1 {
                slot_id,
                candidates: Vec::new(),
            })
            .collect::<Vec<_>>();
        if slots.is_empty()
            || slots.iter().any(|slot| slot.slot_id == 0)
            || !slots
                .windows(2)
                .all(|pair| pair[0].slot_id < pair[1].slot_id)
        {
            return Err("productive prepared slot frontier is not strictly ordered".to_string());
        }
        Ok(Self { slots })
    }

    fn retain(&mut self, profile_index: usize, candidate: RankedCandidateV1) -> Result<(), String> {
        let slot = self.slots.get_mut(profile_index).ok_or_else(|| {
            "productive candidate profile index exceeds slot frontier".to_string()
        })?;
        if slot.slot_id != candidate.output.identity.target_slot_id {
            return Err("productive candidate profile and target slot disagree".to_string());
        }
        retain_ranked_candidate(&mut slot.candidates, candidate);
        Ok(())
    }

    fn into_geometry_selected(mut self, slot_limit: usize) -> Vec<RankedCandidateV1> {
        let slot_count = self.slots.len();
        let mut slot_leaders = self
            .slots
            .iter_mut()
            .enumerate()
            .filter_map(|(profile_index, slot)| {
                slot.candidates.sort_by(ranked_candidate_order);
                slot.candidates
                    .first()
                    .map(|leader| (profile_index, slot.slot_id, leader))
            })
            .collect::<Vec<_>>();
        slot_leaders.sort_by(|(_, left_slot, left), (_, right_slot, right)| {
            ranked_candidate_order(left, right).then_with(|| left_slot.cmp(right_slot))
        });
        let mut selected_slots = vec![false; slot_count];
        for (profile_index, _, _) in slot_leaders.into_iter().take(slot_limit) {
            selected_slots[profile_index] = true;
        }
        let mut selected = self
            .slots
            .into_iter()
            .zip(selected_slots)
            .filter(|(_, selected)| *selected)
            .flat_map(|(slot, _)| slot.candidates)
            .collect::<Vec<_>>();
        selected.sort_by(ranked_candidate_order);
        selected
    }
}

impl PackagedProductiveRuntimeV1 {
    pub(in crate::nanda_wave::l2_field) fn package_sha256(&self) -> [u8; 32] {
        self.package.package_sha256()
    }

    pub(in crate::nanda_wave::l2_field) fn l11_package_sha256(&self) -> [u8; 32] {
        self.package.header.l11_package_sha256
    }

    pub(in crate::nanda_wave::l2_field) fn canonical_l2_package_sha256(&self) -> [u8; 32] {
        self.package.header.canonical_l2_package_sha256
    }

    pub(super) fn load(
        path: &Path,
        expected_l11_sha256: [u8; 32],
        expected_canonical_l2_sha256: [u8; 32],
    ) -> Result<Self, String> {
        let package = ProductivePackageViewV1::load(path)?;
        let package_sha256 = package.package_sha256();
        let sidecar_path = recovery_sidecar_path(path);
        let anchor_recovery = sidecar_path
            .is_file()
            .then(|| AnchorRecoveryPackageViewV1::load(&sidecar_path, package_sha256))
            .transpose()?;
        Self::from_package(
            package,
            anchor_recovery,
            expected_l11_sha256,
            expected_canonical_l2_sha256,
        )
    }

    pub(in crate::nanda_wave::l2_field) fn load_with_semantic_transducer(
        path: &Path,
        expected_l11_sha256: [u8; 32],
        expected_canonical_l2_sha256: [u8; 32],
    ) -> Result<Self, String> {
        let mut runtime = Self::load(path, expected_l11_sha256, expected_canonical_l2_sha256)?;
        runtime.semantic_transducer =
            Some(SemanticExecutionIndexV1::from_package(&runtime.package)?);
        runtime.shared_replay_mode = SharedReplayModeV1::ShadowCompare;
        Ok(runtime)
    }

    pub(super) fn load_with_semantic_proof_authority(
        path: &Path,
        expected_l11_sha256: [u8; 32],
        expected_canonical_l2_sha256: [u8; 32],
    ) -> Result<Self, String> {
        let mut runtime = Self::load_with_semantic_transducer(
            path,
            expected_l11_sha256,
            expected_canonical_l2_sha256,
        )?;
        runtime.shared_replay_mode = SharedReplayModeV1::SemanticProofAuthority;
        Ok(runtime)
    }

    pub(super) fn load_without_anchor_recovery(
        path: &Path,
        expected_l11_sha256: [u8; 32],
        expected_canonical_l2_sha256: [u8; 32],
    ) -> Result<Self, String> {
        Self::from_package(
            ProductivePackageViewV1::load(path)?,
            None,
            expected_l11_sha256,
            expected_canonical_l2_sha256,
        )
    }

    pub(super) fn from_bytes(
        bytes: Vec<u8>,
        expected_l11_sha256: [u8; 32],
        expected_canonical_l2_sha256: [u8; 32],
    ) -> Result<Self, String> {
        Self::from_package(
            ProductivePackageViewV1::from_bytes(bytes)?,
            None,
            expected_l11_sha256,
            expected_canonical_l2_sha256,
        )
    }

    fn from_package(
        package: ProductivePackageViewV1,
        anchor_recovery: Option<AnchorRecoveryPackageViewV1>,
        expected_l11_sha256: [u8; 32],
        expected_canonical_l2_sha256: [u8; 32],
    ) -> Result<Self, String> {
        if package.header.mode != ProductiveAlgorithmModeV1::ProductiveV1Model {
            return Err("trained productive runtime received a speed-parity package".to_string());
        }
        if package.header.l11_package_sha256 != expected_l11_sha256
            || package.header.canonical_l2_package_sha256 != expected_canonical_l2_sha256
        {
            return Err(
                "productive package fingerprint does not match L1.1/canonical L2".to_string(),
            );
        }
        let expected_feature_schema =
            productive_feature_schema_hash_low().map_err(str::to_string)?;
        let mut coefficients_q16 = [0_i32; PRODUCTIVE_FEATURE_COUNT];
        for (index, output) in coefficients_q16.iter_mut().enumerate() {
            let record = package.record::<ModelCoefficientRecordV1>(
                ProductiveSectionKindV1::ModelCoefficients,
                index,
            )?;
            if record.feature_id as usize != index + 1
                || record.feature_schema_hash_low != expected_feature_schema
            {
                return Err(
                    "productive coefficient cache disagrees with feature schema".to_string()
                );
            }
            *output = record.coefficient_q16;
        }
        let mut priors = [RuntimePriorV1 {
            positive: 0.0,
            contradiction: 0.0,
        }; 4];
        for (index, output) in priors.iter_mut().enumerate() {
            let record = package
                .record::<EvidencePriorRecordV1>(ProductiveSectionKindV1::EvidencePriors, index)?;
            if record.channel_id as usize != index + 1 {
                return Err("productive prior cache has a noncanonical channel order".to_string());
            }
            *output = RuntimePriorV1 {
                positive: record.positive_prior_twice as f64,
                contradiction: record.contradiction_prior_twice as f64,
            };
        }
        let paradigms = decode_prepared_section::<ParadigmCenterRecordV1>(
            &package,
            ProductiveSectionKindV1::ParadigmCenters,
        )?;
        let operations = decode_prepared_section::<MorphOpRecordV1>(
            &package,
            ProductiveSectionKindV1::MorphOperations,
        )?;
        let program_records = decode_prepared_section::<MorphProgramHeaderRecordV1>(
            &package,
            ProductiveSectionKindV1::MorphProgramHeaders,
        )?;
        let mut programs = Vec::with_capacity(program_records.len());
        for record in program_records {
            let start = record.op_start as usize;
            let end = start
                .checked_add(record.op_count as usize)
                .filter(|end| *end <= operations.len())
                .ok_or_else(|| {
                    "productive prepared program operation range is invalid".to_string()
                })?;
            let mut suffix_drop = 0_u32;
            for operation in &operations[start..end] {
                if operation.decoded_opcode().map_err(str::to_string)?
                    == MorphOpcodeV1::DropSourceSuffix
                {
                    suffix_drop = suffix_drop.checked_add(operation.arg1).ok_or_else(|| {
                        "productive prepared suffix drop overflows u32".to_string()
                    })?;
                }
            }
            programs.push(PreparedMorphProgramV1 {
                record,
                suffix_drop: u16::try_from(suffix_drop)
                    .map_err(|_| "productive prepared suffix drop exceeds u16".to_string())?,
            });
        }
        let programs = programs.into_boxed_slice();
        let terminals = decode_prepared_section::<ProductiveTerminalRecordV1>(
            &package,
            ProductiveSectionKindV1::Terminals,
        )?;
        let slot_profiles = decode_prepared_section::<SlotPhaseProfileRecordV1>(
            &package,
            ProductiveSectionKindV1::SlotPhaseProfiles,
        )?;
        for paradigm in paradigms.iter().copied() {
            let start = paradigm.slot_profile_start as usize;
            let end = start
                .checked_add(paradigm.slot_profile_count as usize)
                .ok_or_else(|| "productive prepared slot-profile range overflows".to_string())?;
            let profiles = slot_profiles
                .get(start..end)
                .ok_or_else(|| "productive prepared slot-profile range is invalid".to_string())?;
            if profiles.is_empty()
                || !profiles
                    .windows(2)
                    .all(|pair| pair[0].slot_id < pair[1].slot_id)
            {
                return Err(
                    "productive prepared slot profiles are not in strict canonical order"
                        .to_string(),
                );
            }
        }
        let program_count = programs.len();
        let mut terminal_index_by_program = vec![0_u32; program_count];
        for (terminal_index, terminal) in terminals.iter().copied().enumerate() {
            let slot = terminal
                .program_id
                .checked_sub(1)
                .and_then(|index| usize::try_from(index).ok())
                .filter(|index| *index < program_count)
                .ok_or_else(|| {
                    "productive terminal program exceeds the program index".to_string()
                })?;
            if terminal_index_by_program[slot] != 0 {
                return Err("productive program maps to multiple terminals".to_string());
            }
            terminal_index_by_program[slot] = u32::try_from(terminal_index + 1)
                .map_err(|_| "productive terminal index exceeds u32".to_string())?;
        }
        Ok(Self {
            package,
            anchor_recovery,
            semantic_transducer: None,
            shared_replay_mode: SharedReplayModeV1::Legacy,
            coefficients_q16,
            priors,
            paradigms,
            programs,
            operations,
            terminals,
            slot_profiles,
            terminal_index_by_program: terminal_index_by_program.into_boxed_slice(),
        })
    }

    pub(in crate::nanda_wave::l2_field) fn mmap_backed(&self) -> bool {
        self.package.mmap_backed()
            && self
                .anchor_recovery
                .as_ref()
                .is_none_or(AnchorRecoveryPackageViewV1::mmap_backed)
    }

    pub(in crate::nanda_wave::l2_field) fn package_bytes(&self) -> usize {
        self.package.backing_bytes()
    }

    pub(in crate::nanda_wave::l2_field) fn anchor_recovery_package_bytes(&self) -> usize {
        self.anchor_recovery
            .as_ref()
            .map_or(0, AnchorRecoveryPackageViewV1::backing_bytes)
    }

    pub(in crate::nanda_wave::l2_field) fn anchor_recovery_path_count(&self) -> usize {
        self.anchor_recovery
            .as_ref()
            .map_or(0, AnchorRecoveryPackageViewV1::path_count)
    }

    pub(in crate::nanda_wave::l2_field) fn resident_cache_bytes(&self) -> usize {
        std::mem::size_of::<[i32; PRODUCTIVE_FEATURE_COUNT]>()
            + std::mem::size_of::<[RuntimePriorV1; 4]>()
            + self.paradigms.len() * std::mem::size_of::<ParadigmCenterRecordV1>()
            + self.programs.len() * std::mem::size_of::<PreparedMorphProgramV1>()
            + self.operations.len() * std::mem::size_of::<MorphOpRecordV1>()
            + self.terminals.len() * std::mem::size_of::<ProductiveTerminalRecordV1>()
            + self.slot_profiles.len() * std::mem::size_of::<SlotPhaseProfileRecordV1>()
            + self.terminal_index_by_program.len() * std::mem::size_of::<u32>()
            + self
                .anchor_recovery
                .as_ref()
                .map_or(0, AnchorRecoveryPackageViewV1::resident_cache_bytes)
    }

    pub(super) const fn split_seed(&self) -> u64 {
        self.package.header.split_seed
    }

    pub(super) fn slot_id(&self, slot: MorphologySlotKeyV1) -> Result<Option<u32>, String> {
        let count = self.package.record_count(ProductiveSectionKindV1::SlotKeys);
        let mut low = 0_usize;
        let mut high = count;
        while low < high {
            let middle = low + (high - low) / 2;
            let row = self
                .package
                .record::<MorphologySlotKeyV1>(ProductiveSectionKindV1::SlotKeys, middle)?;
            if row < slot {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        if low == count {
            return Ok(None);
        }
        let row = self
            .package
            .record::<MorphologySlotKeyV1>(ProductiveSectionKindV1::SlotKeys, low)?;
        Ok((row == slot).then_some(low as u32 + 1))
    }

    pub(super) fn has_packaged_lemma(&self, lemma_id: u32) -> Result<bool, String> {
        let start = self.lower_bound_binding(lemma_id)?;
        if start
            == self
                .package
                .record_count(ProductiveSectionKindV1::LemmaBindings)
        {
            return Ok(false);
        }
        Ok(self
            .package
            .record::<LemmaParadigmBindingV1>(ProductiveSectionKindV1::LemmaBindings, start)?
            .lemma_id
            == lemma_id)
    }

    pub(super) fn grounding_descriptors(
        &self,
        lemma_id: u32,
    ) -> Result<Vec<PackagedGroundingDescriptorV1>, String> {
        let count = self
            .package
            .record_count(ProductiveSectionKindV1::LemmaBindings);
        let start = self.lower_bound_binding(lemma_id)?;
        let mut descriptors = Vec::new();
        for index in start..count {
            let binding = self
                .package
                .record::<LemmaParadigmBindingV1>(ProductiveSectionKindV1::LemmaBindings, index)?;
            if binding.lemma_id != lemma_id {
                break;
            }
            let paradigm = self.paradigm(binding.paradigm_id)?;
            if paradigm.program_count == 0 {
                return Err("productive proof binding has no paradigm program".to_string());
            }
            let first = self.program(paradigm.program_start as usize)?.record;
            for offset in 1..paradigm.program_count as usize {
                let program = self
                    .program(paradigm.program_start as usize + offset)?
                    .record;
                if program.source_slot_id != first.source_slot_id {
                    return Err(
                        "productive proof paradigm has multiple canonical source slots".to_string(),
                    );
                }
            }
            descriptors.push(PackagedGroundingDescriptorV1 {
                lemma_id,
                pos_domain: paradigm.pos_domain,
                canonical_source_form_ref: binding.canonical_source_form_ref,
                source_slot_id: first.source_slot_id,
                grounded_support: binding.positive_support.max(1),
            });
        }
        descriptors.sort_unstable();
        descriptors.dedup();
        Ok(descriptors)
    }

    pub(super) fn derive_cold_lemma_bindings(
        &self,
        lemma_id: u32,
        sources: &[ColdLemmaSourceV1],
    ) -> Result<Vec<ColdLemmaBindingV1>, String> {
        self.derive_cold_lemma_bindings_inner(lemma_id, sources, false, None)
            .map(|(bindings, _)| bindings)
    }

    pub(super) fn derive_cold_lemma_bindings_with_diagnostics(
        &self,
        lemma_id: u32,
        sources: &[ColdLemmaSourceV1],
    ) -> Result<(Vec<ColdLemmaBindingV1>, ColdBindingDerivationDiagnosticsV1), String> {
        self.derive_cold_lemma_bindings_inner(lemma_id, sources, false, None)
    }

    pub(super) fn derive_cold_lemma_bindings_with_anchor_trace(
        &self,
        lemma_id: u32,
        sources: &[ColdLemmaSourceV1],
    ) -> Result<(Vec<ColdLemmaBindingV1>, ColdBindingDerivationDiagnosticsV1), String> {
        self.derive_cold_lemma_bindings_inner(lemma_id, sources, true, None)
    }

    pub(super) fn derive_cold_lemma_bindings_with_shared_replay_audit(
        &self,
        lemma_id: u32,
        sources: &[ColdLemmaSourceV1],
    ) -> Result<
        (
            Vec<ColdLemmaBindingV1>,
            ColdBindingDerivationDiagnosticsV1,
            Vec<SharedHypothesisReplayAuditV1>,
        ),
        String,
    > {
        let mut audits = Vec::new();
        let (bindings, diagnostics) =
            self.derive_cold_lemma_bindings_inner(lemma_id, sources, true, Some(&mut audits))?;
        Ok((bindings, diagnostics, audits))
    }

    pub(super) fn derive_cold_lemma_bindings_with_semantic_shadow(
        &self,
        lemma_id: u32,
        sources: &[ColdLemmaSourceV1],
    ) -> Result<
        (
            Vec<ColdLemmaBindingV1>,
            ColdBindingDerivationDiagnosticsV1,
            Vec<SharedHypothesisReplayAuditV1>,
        ),
        String,
    > {
        if self.semantic_transducer.is_none() {
            return Err("productive semantic shadow index is not loaded".to_string());
        }
        let mut audits = Vec::new();
        let (bindings, diagnostics) =
            self.derive_cold_lemma_bindings_inner(lemma_id, sources, true, Some(&mut audits))?;
        Ok((bindings, diagnostics, audits))
    }

    fn derive_cold_lemma_bindings_inner(
        &self,
        lemma_id: u32,
        sources: &[ColdLemmaSourceV1],
        trace_identity_anchors: bool,
        mut shared_replay_audits: Option<&mut Vec<SharedHypothesisReplayAuditV1>>,
    ) -> Result<(Vec<ColdLemmaBindingV1>, ColdBindingDerivationDiagnosticsV1), String> {
        let mut diagnostics = ColdBindingDerivationDiagnosticsV1::default();
        if sources.is_empty() {
            return Ok((Vec::new(), diagnostics));
        }
        let mut sources = sources.to_vec();
        for source in &sources {
            PackagedGroundedLemmaV1 {
                lemma_id,
                pos_domain: source.pos_domain,
                canonical_source_form_ref: source.canonical_source_form_ref,
                source_slot_id: source.source_slot_id,
                normalized_source: source.normalized_source.clone(),
                grounded_support: source.grounded_support,
            }
            .validate(self.package.header.maximum_observed_scalars)
            .map_err(str::to_string)?;
        }
        sources.sort_by(|left, right| {
            right
                .canonical_source
                .cmp(&left.canonical_source)
                .then_with(|| left.canonical_preference.cmp(&right.canonical_preference))
                .then_with(|| {
                    left.normalized_source
                        .chars()
                        .count()
                        .cmp(&right.normalized_source.chars().count())
                })
                .then_with(|| left.source_slot_id.cmp(&right.source_slot_id))
                .then_with(|| {
                    left.canonical_source_form_ref
                        .cmp(&right.canonical_source_form_ref)
                })
                .then_with(|| left.normalized_source.cmp(&right.normalized_source))
        });
        sources.dedup_by(|left, right| {
            (
                left.pos_domain,
                left.canonical_source_form_ref,
                left.source_slot_id,
            ) == (
                right.pos_domain,
                right.canonical_source_form_ref,
                right.source_slot_id,
            )
        });

        diagnostics.source_count = sources.len();
        let source_pos_domains = sources
            .iter()
            .map(|source| source.pos_domain)
            .collect::<BTreeSet<_>>();
        diagnostics.source_pos_domain_count = source_pos_domains.len();
        diagnostics.observed_slot_count = sources
            .iter()
            .map(|source| (source.pos_domain, source.source_slot_id))
            .collect::<BTreeSet<_>>()
            .len();
        let mut exposed_by_pos = BTreeMap::<u16, ExposedFormConstraintsV1>::new();
        for source in &sources {
            exposed_by_pos
                .entry(source.pos_domain)
                .or_default()
                .insert(source.source_slot_id, &source.normalized_source);
        }

        let mut selected = BTreeMap::<(u32, Option<String>), ColdLemmaBindingV1>::new();
        let mut paradigm_flags = DenseParadigmFlagsV1::new(self.paradigms.len())?;
        for pos_domain in source_pos_domains.iter().copied() {
            let pos_sources = sources
                .iter()
                .filter(|source| source.pos_domain == pos_domain)
                .collect::<Vec<_>>();
            let mut observed_slots = pos_sources
                .iter()
                .map(|source| source.source_slot_id)
                .collect::<Vec<_>>();
            observed_slots.sort_unstable();
            observed_slots.dedup();

            let eligible = self.structurally_eligible_paradigms(pos_domain, &observed_slots)?;
            let mut eligible_by_anchor_slot =
                BTreeMap::<u32, Vec<(u32, ParadigmCenterRecordV1)>>::new();
            for (paradigm_id, paradigm) in eligible.iter().copied() {
                for profile in self.prepared_slot_profiles(paradigm)? {
                    eligible_by_anchor_slot
                        .entry(profile.slot_id)
                        .or_default()
                        .push((paradigm_id, paradigm));
                }
            }
            let exposed = exposed_by_pos
                .get(&pos_domain)
                .ok_or_else(|| "productive POS basin has no exposed constraints".to_string())?;
            diagnostics.structural_eligible_paradigm_count += eligible.len();
            for (paradigm_id, _) in eligible.iter() {
                paradigm_flags.mark(*paradigm_id, PARADIGM_FLAG_SLOT_COMPATIBLE)?;
            }

            let mut direct_sources = BTreeMap::<u32, Vec<&ColdLemmaSourceV1>>::new();
            let mut recovered = BTreeMap::<(u32, u32, String), RecoveredAnchorCandidateV1>::new();
            let mut shared_hypotheses = BTreeMap::<(u32, String), SharedAnchorHypothesisV1>::new();
            for (source_ordinal, source) in pos_sources.iter().enumerate() {
                diagnostics.posting_lookup_count += 1;
                let compatible =
                    self.compatible_paradigms(source.pos_domain, source.source_slot_id)?;
                diagnostics.posting_visit_count += compatible.len();
                diagnostics.posting_miss_count += usize::from(compatible.is_empty());
                for paradigm_id in compatible {
                    paradigm_flags.mark(paradigm_id, PARADIGM_FLAG_POSTING)?;
                    if eligible.contains(paradigm_id) {
                        direct_sources.entry(paradigm_id).or_default().push(source);
                    }
                }

                let Some(anchor_recovery) = self.anchor_recovery.as_ref() else {
                    continue;
                };
                diagnostics.recovery_lookup_count += 1;
                let paths =
                    anchor_recovery.recovery_paths(source.pos_domain, source.source_slot_id)?;
                diagnostics.recovery_path_count += paths.len();
                diagnostics.posting_visit_count += paths.len();
                let source_scalars = source.normalized_source.chars().collect::<Vec<_>>();
                let mut recovered_anchor = String::new();
                let mut group_start = 0_usize;
                while group_start < paths.len() {
                    let program_id = paths[group_start].posting.program_id;
                    let group_end = group_start
                        + paths[group_start..]
                            .partition_point(|path| path.posting.program_id == program_id);
                    let group = &paths[group_start..group_end];
                    let mut first_eligible = None;
                    for path in group.iter().copied() {
                        let paradigm_id = path.posting.paradigm_id;
                        paradigm_flags.mark(paradigm_id, PARADIGM_FLAG_POSTING)?;
                        paradigm_flags.mark(paradigm_id, PARADIGM_FLAG_RECOVERY_POSTING)?;
                        if eligible.contains(paradigm_id) {
                            diagnostics.recovery_post_intersection_count += 1;
                            first_eligible.get_or_insert(path);
                        }
                    }
                    let shared_anchor_slot = group[0].program.target_slot_id;
                    let shared_join_possible = eligible_by_anchor_slot
                        .get(&shared_anchor_slot)
                        .is_some_and(|paradigms| !paradigms.is_empty());
                    let Some(execution_path) =
                        first_eligible.or_else(|| shared_join_possible.then_some(group[0]))
                    else {
                        group_start = group_end;
                        continue;
                    };
                    diagnostics.recovery_program_execution_count += 1;
                    diagnostics.operator_step_count += u64::from(execution_path.program.op_count);
                    if anchor_recovery
                        .recover_path_into(execution_path, &source_scalars, &mut recovered_anchor)
                        .is_err()
                    {
                        group_start = group_end;
                        continue;
                    }
                    if shared_join_possible {
                        diagnostics.shared_hypothesis_observation_count += 1;
                        let hypothesis = shared_hypotheses
                            .entry((shared_anchor_slot, recovered_anchor.clone()))
                            .or_insert_with(|| SharedAnchorHypothesisV1 {
                                source: ColdLemmaSourceV1 {
                                    pos_domain: source.pos_domain,
                                    canonical_source_form_ref: source.canonical_source_form_ref,
                                    source_slot_id: shared_anchor_slot,
                                    normalized_source: recovered_anchor.clone(),
                                    grounded_support: source.grounded_support,
                                    canonical_preference: source.canonical_preference,
                                    canonical_source: true,
                                },
                                independent_sources: BTreeSet::new(),
                            });
                        hypothesis.independent_sources.insert(source_ordinal);
                    }
                    for path in group
                        .iter()
                        .copied()
                        .filter(|path| eligible.contains(path.posting.paradigm_id))
                    {
                        let paradigm_id = path.posting.paradigm_id;
                        diagnostics.recovered_anchor_count += 1;
                        paradigm_flags.mark(paradigm_id, PARADIGM_FLAG_RECOVERED_ANCHOR)?;
                        let paradigm = self.paradigm(paradigm_id)?;
                        let candidate = recovered
                            .entry((
                                paradigm_id,
                                path.program.target_slot_id,
                                recovered_anchor.clone(),
                            ))
                            .or_insert_with(|| RecoveredAnchorCandidateV1 {
                                paradigm_id,
                                paradigm,
                                source: ColdLemmaSourceV1 {
                                    pos_domain: source.pos_domain,
                                    canonical_source_form_ref: source.canonical_source_form_ref,
                                    source_slot_id: path.program.target_slot_id,
                                    normalized_source: recovered_anchor.clone(),
                                    grounded_support: source.grounded_support,
                                    canonical_preference: source.canonical_preference,
                                    canonical_source: true,
                                },
                                independent_sources: BTreeSet::new(),
                                learned_recovery: true,
                                maximum_train_lemma_support: 0,
                                maximum_stability: 0,
                                exact_certified: false,
                                shared_hypothesis: false,
                            });
                        candidate.observe_learned_path(
                            source_ordinal,
                            path.posting.train_lemma_support,
                            path.posting.stability,
                        );
                    }
                    group_start = group_end;
                }
            }

            diagnostics.shared_hypothesis_unique_count += shared_hypotheses.len();
            let mut equivalence_classes_by_anchor =
                BTreeMap::<u32, Vec<TransitionEquivalenceClassV1>>::new();
            if self.shared_replay_mode != SharedReplayModeV1::SemanticProofAuthority {
                for anchor_slot_id in shared_hypotheses
                    .keys()
                    .map(|(anchor_slot_id, _)| *anchor_slot_id)
                    .collect::<BTreeSet<_>>()
                {
                    let Some(join_paradigms) = eligible_by_anchor_slot.get(&anchor_slot_id) else {
                        continue;
                    };
                    let classes = self.transition_equivalence_classes(
                        anchor_slot_id,
                        join_paradigms,
                        exposed,
                    )?;
                    diagnostics.transition_equivalence_class_count += classes.len();
                    diagnostics.transition_equivalence_owner_count += classes
                        .iter()
                        .map(|class| class.owners.len())
                        .sum::<usize>();
                    diagnostics.transition_equivalence_max_class_size =
                        diagnostics.transition_equivalence_max_class_size.max(
                            classes
                                .iter()
                                .map(|class| class.owners.len())
                                .max()
                                .unwrap_or_default(),
                        );
                    equivalence_classes_by_anchor.insert(anchor_slot_id, classes);
                }
            }
            for ((anchor_slot_id, recovered_surface), hypothesis) in shared_hypotheses {
                let Some(join_paradigms) = eligible_by_anchor_slot.get(&anchor_slot_id) else {
                    continue;
                };
                let mut eligible_paradigm_ids = join_paradigms
                    .iter()
                    .map(|(paradigm_id, _)| *paradigm_id)
                    .collect::<Vec<_>>();
                eligible_paradigm_ids.sort_unstable();
                eligible_paradigm_ids.dedup();
                let constraints = exposed
                    .slots
                    .iter()
                    .flat_map(|slot| {
                        slot.surfaces
                            .iter()
                            .map(move |surface| SharedReplayConstraintV1 {
                                slot_id: slot.slot_id,
                                normalized_surface: surface.clone(),
                            })
                    })
                    .collect::<Vec<_>>();
                let semantic_exact_paradigm_ids = match self.shared_replay_mode {
                    SharedReplayModeV1::Legacy => None,
                    SharedReplayModeV1::ShadowCompare
                    | SharedReplayModeV1::SemanticProofAuthority => Some(
                        self.semantic_transducer
                            .as_ref()
                            .ok_or("productive semantic replay mode has no semantic index")?
                            .exact_owners(
                                pos_domain,
                                anchor_slot_id,
                                &recovered_surface,
                                &constraints,
                                &eligible_paradigm_ids,
                            )?
                            .0,
                    ),
                };
                let mut direct_exact_paradigm_ids = Vec::new();
                if self.shared_replay_mode != SharedReplayModeV1::SemanticProofAuthority {
                    let classes = equivalence_classes_by_anchor
                        .get(&anchor_slot_id)
                        .ok_or("productive legacy replay has no equivalence classes")?;
                    diagnostics.shared_hypothesis_join_attempt_count += classes
                        .iter()
                        .map(|class| class.owners.len())
                        .sum::<usize>();
                    for class in classes {
                        let mut shared_source = hypothesis.source.clone();
                        shared_source.normalized_source = recovered_surface.clone();
                        let (reconstructs, executed, operator_steps) = self
                            .paradigm_reconstructs_exposed_forms(
                                class.representative,
                                &shared_source,
                                exposed,
                            )?;
                        diagnostics.transition_equivalence_representative_replay_count += 1;
                        diagnostics.exact_replay_program_execution_count += executed;
                        diagnostics.operator_step_count += operator_steps;
                        diagnostics.shared_hypothesis_replay_execution_count += executed;
                        if !reconstructs {
                            continue;
                        }
                        diagnostics.transition_equivalence_exact_class_count += 1;
                        direct_exact_paradigm_ids
                            .extend(class.owners.iter().map(|(paradigm_id, _)| *paradigm_id));
                    }
                } else if let Some(semantic) = &semantic_exact_paradigm_ids {
                    direct_exact_paradigm_ids.clone_from(semantic);
                }
                direct_exact_paradigm_ids.sort_unstable();
                direct_exact_paradigm_ids.dedup();
                if let Some(semantic) = &semantic_exact_paradigm_ids {
                    if *semantic != direct_exact_paradigm_ids {
                        return Err(
                            "productive semantic shadow owner set disagrees with legacy replay"
                                .to_string(),
                        );
                    }
                }
                let mut shared_source = hypothesis.source.clone();
                shared_source.normalized_source = recovered_surface.clone();
                for paradigm_id in direct_exact_paradigm_ids.iter().copied() {
                    let paradigm = self.paradigm(paradigm_id)?;
                    let key = (paradigm_id, anchor_slot_id, recovered_surface.clone());
                    if recovered
                        .get(&key)
                        .is_some_and(|candidate| candidate.exact_certified)
                    {
                        continue;
                    }
                    diagnostics.shared_hypothesis_exact_count += 1;
                    diagnostics.transition_equivalence_exact_owner_fanout_count += 1;
                    paradigm_flags.mark(paradigm_id, PARADIGM_FLAG_RECOVERED_ANCHOR)?;
                    let candidate =
                        recovered
                            .entry(key)
                            .or_insert_with(|| RecoveredAnchorCandidateV1 {
                                paradigm_id,
                                paradigm,
                                source: shared_source.clone(),
                                independent_sources: BTreeSet::new(),
                                learned_recovery: true,
                                maximum_train_lemma_support: 0,
                                maximum_stability: 0,
                                exact_certified: true,
                                shared_hypothesis: true,
                            });
                    candidate
                        .independent_sources
                        .extend(hypothesis.independent_sources.iter().copied());
                    candidate.learned_recovery = true;
                    candidate.exact_certified = true;
                    candidate.shared_hypothesis = true;
                }
                if let Some(audits) = shared_replay_audits.as_deref_mut() {
                    audits.push(SharedHypothesisReplayAuditV1 {
                        pos_domain,
                        anchor_slot_id,
                        normalized_source: recovered_surface,
                        constraints,
                        eligible_paradigm_ids,
                        direct_exact_paradigm_ids,
                    });
                }
            }

            for (paradigm_id, candidate_sources) in direct_sources {
                let paradigm = self.paradigm(paradigm_id)?;
                for source in candidate_sources {
                    let (reconstructs, executed, operator_steps) =
                        self.paradigm_reconstructs_exposed_forms(paradigm, source, exposed)?;
                    diagnostics.exact_replay_program_execution_count += executed;
                    diagnostics.operator_step_count += operator_steps;
                    if !reconstructs {
                        continue;
                    }
                    paradigm_flags.mark(paradigm_id, PARADIGM_FLAG_EXACT_RECONSTRUCTING)?;
                    paradigm_flags.mark(paradigm_id, PARADIGM_FLAG_DIRECT_SELECTED)?;
                    selected
                        .entry((paradigm_id, None))
                        .or_insert_with(|| ColdLemmaBindingV1 {
                            lemma: PackagedGroundedLemmaV1 {
                                lemma_id,
                                pos_domain: source.pos_domain,
                                canonical_source_form_ref: source.canonical_source_form_ref,
                                source_slot_id: source.source_slot_id,
                                normalized_source: source.normalized_source.clone(),
                                grounded_support: source.grounded_support,
                            },
                            paradigm_id,
                            observed_slots: observed_slots.clone(),
                            recovered_anchor: false,
                            cross_lane_certified: false,
                        });
                    break;
                }
            }

            if self.anchor_recovery.is_some() {
                for (paradigm_id, paradigm) in eligible.iter() {
                    if paradigm.support < 2 {
                        continue;
                    }
                    for (source_ordinal, source) in pos_sources.iter().enumerate() {
                        let Some(anchor_slot_id) =
                            self.identity_anchor_slot(*paradigm, source.source_slot_id)?
                        else {
                            continue;
                        };
                        let key = (
                            *paradigm_id,
                            anchor_slot_id,
                            source.normalized_source.clone(),
                        );
                        if recovered.contains_key(&key) {
                            continue;
                        }
                        diagnostics.identity_bridge_candidate_count += 1;
                        paradigm_flags.mark(*paradigm_id, PARADIGM_FLAG_RECOVERED_ANCHOR)?;
                        recovered.insert(
                            key,
                            RecoveredAnchorCandidateV1 {
                                paradigm_id: *paradigm_id,
                                paradigm: *paradigm,
                                source: ColdLemmaSourceV1 {
                                    pos_domain: source.pos_domain,
                                    canonical_source_form_ref: source.canonical_source_form_ref,
                                    source_slot_id: anchor_slot_id,
                                    normalized_source: source.normalized_source.clone(),
                                    grounded_support: source.grounded_support,
                                    canonical_preference: source.canonical_preference,
                                    canonical_source: true,
                                },
                                independent_sources: BTreeSet::from([source_ordinal]),
                                learned_recovery: false,
                                maximum_train_lemma_support: paradigm.support,
                                maximum_stability: paradigm.stability,
                                exact_certified: false,
                                shared_hypothesis: false,
                            },
                        );
                    }
                }
            }

            diagnostics.recovery_unique_anchor_count += recovered.len();
            diagnostics.recovery_max_independent_source_count =
                diagnostics.recovery_max_independent_source_count.max(
                    recovered
                        .values()
                        .map(|candidate| candidate.independent_sources.len())
                        .max()
                        .unwrap_or_default(),
                );
            if trace_identity_anchors {
                diagnostics.identity_anchors_pre_frontier.extend(
                    recovered
                        .values()
                        .filter(|candidate| !candidate.learned_recovery)
                        .map(RecoveryIdentityAnchorRefV1::from),
                );
            }
            let pre_frontier_count = recovered.len();
            let mut recovered = recovered.into_values().collect::<Vec<_>>();
            recovered.sort_by(RecoveredAnchorCandidateV1::evidence_order);
            recovered.truncate(PRODUCTIVE_PHYSICAL_TOP_K);
            diagnostics.recovery_post_frontier_anchor_count += recovered.len();
            diagnostics.recovery_frontier_dropped_count +=
                pre_frontier_count.saturating_sub(recovered.len());
            diagnostics
                .recovery_post_frontier_paradigm_ids
                .extend(recovered.iter().map(|candidate| candidate.paradigm_id));
            if trace_identity_anchors {
                diagnostics.identity_anchors_post_frontier.extend(
                    recovered
                        .iter()
                        .filter(|candidate| !candidate.learned_recovery)
                        .map(RecoveryIdentityAnchorRefV1::from),
                );
            }
            for candidate in recovered {
                if paradigm_flags.contains(candidate.paradigm_id, PARADIGM_FLAG_DIRECT_SELECTED) {
                    continue;
                }
                let (reconstructs, executed, operator_steps) = if candidate.exact_certified {
                    (true, 0, 0)
                } else {
                    self.paradigm_reconstructs_exposed_forms(
                        candidate.paradigm,
                        &candidate.source,
                        exposed,
                    )?
                };
                diagnostics.exact_replay_program_execution_count += executed;
                diagnostics.operator_step_count += operator_steps;
                if !reconstructs {
                    continue;
                }
                diagnostics.recovery_exact_reconstructing_count += 1;
                if trace_identity_anchors && !candidate.learned_recovery {
                    diagnostics
                        .identity_anchors_exact
                        .insert(RecoveryIdentityAnchorRefV1::from(&candidate));
                }
                paradigm_flags.mark(candidate.paradigm_id, PARADIGM_FLAG_EXACT_RECONSTRUCTING)?;
                paradigm_flags.mark(candidate.paradigm_id, PARADIGM_FLAG_RECOVERY_EXACT)?;
                let recovered_anchor = candidate.source.normalized_source.clone();
                selected
                    .entry((candidate.paradigm_id, Some(recovered_anchor.clone())))
                    .or_insert_with(|| ColdLemmaBindingV1 {
                        lemma: PackagedGroundedLemmaV1 {
                            lemma_id,
                            pos_domain: candidate.source.pos_domain,
                            canonical_source_form_ref: candidate.source.canonical_source_form_ref,
                            source_slot_id: candidate.source.source_slot_id,
                            normalized_source: recovered_anchor,
                            grounded_support: candidate.source.grounded_support,
                        },
                        paradigm_id: candidate.paradigm_id,
                        observed_slots: observed_slots.clone(),
                        recovered_anchor: true,
                        cross_lane_certified: has_cross_lane_certificate(
                            candidate.exact_certified,
                            candidate.shared_hypothesis,
                            candidate.independent_sources.len(),
                        ),
                    });
            }
        }
        selected.retain(|(paradigm_id, recovered_anchor), _| {
            recovered_anchor.is_none()
                || !paradigm_flags.contains(*paradigm_id, PARADIGM_FLAG_DIRECT_SELECTED)
        });
        let posting_paradigms = paradigm_flags.ids_with(PARADIGM_FLAG_POSTING)?;
        let slot_compatible_paradigms = paradigm_flags.ids_with(PARADIGM_FLAG_SLOT_COMPATIBLE)?;
        let exact_reconstructing_paradigms =
            paradigm_flags.ids_with(PARADIGM_FLAG_EXACT_RECONSTRUCTING)?;
        let recovery_posting_paradigms = paradigm_flags.ids_with(PARADIGM_FLAG_RECOVERY_POSTING)?;
        let recovered_anchor_paradigms = paradigm_flags.ids_with(PARADIGM_FLAG_RECOVERED_ANCHOR)?;
        let recovery_exact_paradigms = paradigm_flags.ids_with(PARADIGM_FLAG_RECOVERY_EXACT)?;
        diagnostics.posting_paradigm_count = posting_paradigms.len();
        diagnostics.slot_compatible_paradigm_count = slot_compatible_paradigms.len();
        diagnostics.exact_reconstructing_paradigm_count = exact_reconstructing_paradigms.len();
        diagnostics.retained_binding_count = selected.len();
        diagnostics.source_pos_domains = source_pos_domains;
        diagnostics.posting_paradigm_ids = posting_paradigms;
        diagnostics.slot_compatible_paradigm_ids = slot_compatible_paradigms;
        diagnostics.exact_reconstructing_paradigm_ids = exact_reconstructing_paradigms;
        diagnostics.retained_paradigm_ids = selected.keys().map(|(id, _)| *id).collect();
        diagnostics.recovery_posting_paradigm_ids = recovery_posting_paradigms;
        diagnostics.recovered_anchor_paradigm_ids = recovered_anchor_paradigms;
        diagnostics.recovery_exact_paradigm_ids = recovery_exact_paradigms;
        let bindings = selected.into_values().collect::<Vec<_>>();
        for binding in &bindings {
            binding
                .validate(self.package.header.maximum_observed_scalars)
                .map_err(str::to_string)?;
        }
        Ok((bindings, diagnostics))
    }

    fn structurally_eligible_paradigms(
        &self,
        pos_domain: u16,
        observed_slots: &[u32],
    ) -> Result<StructurallyEligibleParadigmsV1, String> {
        let membership_len = self
            .paradigms
            .len()
            .checked_add(1)
            .ok_or_else(|| "productive paradigm membership length overflows".to_string())?;
        let mut ordered = Vec::new();
        let mut membership = vec![0_u8; membership_len];
        for (index, paradigm) in self.paradigms.iter().copied().enumerate() {
            if paradigm.pos_domain != pos_domain {
                continue;
            }
            let profiles = self.prepared_slot_profiles(paradigm)?;
            if observed_slots.iter().all(|slot| {
                profiles
                    .binary_search_by_key(slot, |profile| profile.slot_id)
                    .is_ok()
            }) {
                let paradigm_id = u32::try_from(index + 1)
                    .map_err(|_| "productive paradigm identity exceeds u32".to_string())?;
                ordered.push((paradigm_id, paradigm));
                membership[index + 1] = 1;
            }
        }
        Ok(StructurallyEligibleParadigmsV1 {
            ordered,
            membership,
        })
    }

    fn transition_equivalence_classes(
        &self,
        anchor_slot_id: u32,
        paradigms: &[(u32, ParadigmCenterRecordV1)],
        exposed: &ExposedFormConstraintsV1,
    ) -> Result<Vec<TransitionEquivalenceClassV1>, String> {
        let mut grouped =
            BTreeMap::<Vec<TransitionProgramSignatureV1>, Vec<(u32, ParadigmCenterRecordV1)>>::new(
            );
        for (paradigm_id, paradigm) in paradigms.iter().copied() {
            let mut signature = Vec::new();
            for offset in 0..paradigm.program_count as usize {
                let program = self.program(paradigm.program_start as usize + offset)?;
                if program.record.source_slot_id != anchor_slot_id
                    || exposed.slot(program.record.target_slot_id).is_none()
                {
                    continue;
                }
                let operations = self
                    .program_operations(program)?
                    .iter()
                    .map(|operation| TransitionOperationSignatureV1 {
                        opcode: operation.opcode,
                        anchor: operation.anchor,
                        flags: operation.flags,
                        arg0: operation.arg0,
                        arg1: operation.arg1,
                        arg2: operation.arg2,
                    })
                    .collect::<Vec<_>>();
                signature.push(TransitionProgramSignatureV1 {
                    source_slot_id: program.record.source_slot_id,
                    target_slot_id: program.record.target_slot_id,
                    flags: program.record.flags,
                    operations,
                });
            }
            signature.sort_unstable();
            signature.dedup();
            grouped
                .entry(signature)
                .or_default()
                .push((paradigm_id, paradigm));
        }
        Ok(grouped
            .into_values()
            .map(|owners| TransitionEquivalenceClassV1 {
                representative: owners[0].1,
                owners,
            })
            .collect())
    }

    fn identity_anchor_slot(
        &self,
        paradigm: ParadigmCenterRecordV1,
        observed_slot_id: u32,
    ) -> Result<Option<u32>, String> {
        let mut anchor_slot = None;
        for offset in 0..paradigm.program_count as usize {
            let program = self.program(paradigm.program_start as usize + offset)?;
            if program.record.target_slot_id != observed_slot_id
                || !self.program_is_identity(program)?
            {
                continue;
            }
            if anchor_slot.is_some_and(|current| current != program.record.source_slot_id) {
                return Err("identity bridge has multiple canonical anchor slots".to_string());
            }
            anchor_slot = Some(program.record.source_slot_id);
        }
        Ok(anchor_slot)
    }

    fn program_is_identity(&self, program: &PreparedMorphProgramV1) -> Result<bool, String> {
        if program.record.op_count != 2 {
            return Ok(false);
        }
        let operations = self.program_operations(program)?;
        let [copy, terminate] = operations else {
            return Ok(false);
        };
        Ok(
            copy.decoded_opcode().map_err(str::to_string)? == MorphOpcodeV1::CopySourceRange
                && copy.anchor == SourceAnchorV1::Start as u8
                && copy.arg0 == 0
                && copy.arg1 == u32::from(COPY_TO_RETAINED_EDGE)
                && copy.arg2 == 0
                && terminate.decoded_opcode().map_err(str::to_string)? == MorphOpcodeV1::Terminate
                && terminate.arg1 == program.record.target_slot_id
                && terminate.arg2 != 0,
        )
    }

    fn paradigm_reconstructs_exposed_forms(
        &self,
        paradigm: ParadigmCenterRecordV1,
        source: &ColdLemmaSourceV1,
        exposed: &ExposedFormConstraintsV1,
    ) -> Result<(bool, usize, u64), String> {
        let mut matched = vec![0_u8; exposed.surface_count];
        let mut matched_count = 0_usize;
        let mut executed = 0_usize;
        let mut operator_steps = 0_u64;
        let source_scalars = source.normalized_source.chars().collect::<Vec<_>>();
        let mut surface = String::new();
        for offset in 0..paradigm.program_count as usize {
            let program = self.program(paradigm.program_start as usize + offset)?;
            if program.record.source_slot_id != source.source_slot_id {
                continue;
            }
            let Some(expected) = exposed.slot(program.record.target_slot_id) else {
                continue;
            };
            executed += 1;
            operator_steps = operator_steps
                .checked_add(u64::from(program.record.op_count))
                .ok_or_else(|| "productive replay operator-step count overflow".to_string())?;
            if !self.execute_packaged_program_into(program, &source_scalars, &mut surface)? {
                continue;
            }
            let Ok(surface_index) = expected
                .surfaces
                .binary_search_by(|candidate| candidate.as_str().cmp(surface.as_str()))
            else {
                continue;
            };
            let match_index = expected.match_start + surface_index;
            if matched[match_index] == 0 {
                matched[match_index] = 1;
                matched_count += 1;
            }
        }
        Ok((
            matched_count == exposed.surface_count,
            executed,
            operator_steps,
        ))
    }

    pub(super) fn identity_anchor_reconstructs_exposed_forms(
        &self,
        anchor: RecoveryIdentityAnchorRefV1,
        normalized_source: &str,
        exposed: &[ColdLemmaSourceV1],
    ) -> Result<bool, String> {
        let paradigm = self.paradigm(anchor.paradigm_id)?;
        let source = ColdLemmaSourceV1 {
            pos_domain: paradigm.pos_domain,
            canonical_source_form_ref: anchor.canonical_source_form_ref,
            source_slot_id: anchor.source_slot_id,
            normalized_source: normalized_source.to_string(),
            grounded_support: 1,
            canonical_preference: 0,
            canonical_source: true,
        };
        let mut constraints = ExposedFormConstraintsV1::default();
        for target in exposed
            .iter()
            .filter(|target| target.pos_domain == source.pos_domain)
        {
            constraints.insert(target.source_slot_id, &target.normalized_source);
        }
        self.paradigm_reconstructs_exposed_forms(paradigm, &source, &constraints)
            .map(|(reconstructs, _, _)| reconstructs)
    }

    fn execute_packaged_program_into(
        &self,
        program: &PreparedMorphProgramV1,
        source: &[char],
        output: &mut String,
    ) -> Result<bool, String> {
        if source.len() > usize::from(self.package.header.maximum_observed_scalars) {
            return Ok(false);
        }
        let operations = self.program_operations(program)?;
        let Some(retained_end) = source.len().checked_sub(usize::from(program.suffix_drop)) else {
            return Ok(false);
        };
        output.clear();
        let mut cursor = 0_usize;
        let mut terminated = false;
        for (operation_index, operation) in operations.iter().copied().enumerate() {
            if terminated {
                return Err("productive direct replay operation follows terminate".to_string());
            }
            match operation.decoded_opcode().map_err(str::to_string)? {
                MorphOpcodeV1::CopySourceRange => {
                    let source_delta = i16::try_from(operation.arg0).map_err(|_| {
                        "productive direct replay source delta exceeds i16".to_string()
                    })?;
                    let Some(start) = resolve_source_offset(
                        source.len(),
                        decode_anchor(operation.anchor)?,
                        source_delta,
                    ) else {
                        return Ok(false);
                    };
                    let end = if operation.arg1 == u32::from(COPY_TO_RETAINED_EDGE) {
                        if let Some(next) = operations.get(operation_index + 1).copied() {
                            if next.decoded_opcode().map_err(str::to_string)?
                                == MorphOpcodeV1::ReplaceSourceRange
                            {
                                let delta = i16::try_from(next.arg0).map_err(|_| {
                                    "productive direct replay replacement delta exceeds i16"
                                        .to_string()
                                })?;
                                let Some(end) =
                                    resolve_source_offset(source.len(), SourceAnchorV1::End, delta)
                                else {
                                    return Ok(false);
                                };
                                end
                            } else {
                                retained_end
                            }
                        } else {
                            retained_end
                        }
                    } else {
                        let Some(end) = start.checked_add(operation.arg1 as usize) else {
                            return Ok(false);
                        };
                        end
                    };
                    if start != cursor || end <= start || end > retained_end {
                        return Ok(false);
                    }
                    output.extend(source[start..end].iter());
                    cursor = end;
                }
                MorphOpcodeV1::DropSourcePrefix => {
                    let count = operation.arg1 as usize;
                    if cursor != 0 || count == 0 || count > source.len() {
                        return Ok(false);
                    }
                    cursor = count;
                }
                MorphOpcodeV1::DropSourceSuffix => {
                    let count = operation.arg1 as usize;
                    if count == 0 || cursor.checked_add(count) != Some(source.len()) {
                        return Ok(false);
                    }
                    cursor = source.len();
                }
                MorphOpcodeV1::EmitSegment => {
                    output.push_str(self.package.segment(operation.arg1)?);
                }
                MorphOpcodeV1::ReplaceSourceRange => {
                    let Some(start) = source.len().checked_add_signed(operation.arg0 as isize)
                    else {
                        return Ok(false);
                    };
                    let Some(end) = start.checked_add(operation.arg1 as usize) else {
                        return Ok(false);
                    };
                    if start != cursor || end > source.len() {
                        return Ok(false);
                    }
                    if operation.arg2 != 0 {
                        output.push_str(self.package.segment(operation.arg2)?);
                    }
                    cursor = end;
                }
                MorphOpcodeV1::EmitExactAllomorph => {
                    return Err(
                        "lemma-local exact allomorph leaked into transferable direct replay"
                            .to_string(),
                    );
                }
                MorphOpcodeV1::Terminate => {
                    if operation.arg1 != program.record.target_slot_id
                        || operation.arg2 == 0
                        || cursor != source.len()
                    {
                        return Err("productive direct replay terminate is invalid".to_string());
                    }
                    terminated = true;
                }
            }
        }
        if !terminated
            || output.is_empty()
            || output.chars().count() > usize::from(self.package.header.maximum_generated_scalars)
        {
            return Ok(false);
        }
        Ok(true)
    }

    fn instantiate_paradigm_surfaces(
        &self,
        paradigm_id: u32,
        paradigm: ParadigmCenterRecordV1,
        source_slot_id: u32,
        normalized_source: &str,
    ) -> Result<BTreeMap<u32, BTreeSet<String>>, String> {
        let source = normalized_source.chars().collect::<Vec<_>>();
        let observed = ObservedGeometryV1::new(normalized_source).map_err(str::to_string)?;
        let mut trace_arena = ScalarTraceArenaV1::default();
        let mut generated = BTreeMap::<u32, BTreeSet<String>>::new();
        let mut stack = vec![TraversalFrameV1 {
            node_id: paradigm.root_node,
            source_cursor: 0,
            trace_ref: None,
            geometry: GeometryTraversalStateV1::new(
                &observed,
                GeometryPathIdentityV1 {
                    paradigm_id,
                    ..GeometryPathIdentityV1::default()
                },
            )
            .map_err(str::to_string)?,
            exact_allomorph: false,
        }];
        while let Some(frame) = stack.pop() {
            let node = self.package.record::<ProductiveTrieNodeRecordV1>(
                ProductiveSectionKindV1::TrieNodes,
                frame.node_id as usize,
            )?;
            if frame.source_cursor == source.len() {
                for offset in 0..usize::from(node.terminal_count) {
                    let terminal = self.terminal(node.terminal_start as usize + offset)?;
                    let program = self.program(terminal.program_id as usize - 1)?.record;
                    if program.source_slot_id != source_slot_id {
                        continue;
                    }
                    let surface = trace_arena
                        .scalars(frame.trace_ref)
                        .map_err(str::to_string)?
                        .into_iter()
                        .map(|scalar| {
                            char::from_u32(scalar).ok_or_else(|| {
                                "productive cold compatibility emitted an invalid scalar"
                                    .to_string()
                            })
                        })
                        .collect::<Result<String, _>>()?;
                    if surface.is_empty()
                        || surface.chars().count()
                            > usize::from(self.package.header.maximum_generated_scalars)
                    {
                        return Err(
                            "productive cold compatibility violates generated scalar bounds"
                                .to_string(),
                        );
                    }
                    generated
                        .entry(terminal.target_slot_id)
                        .or_default()
                        .insert(surface);
                }
            }
            for offset in (0..usize::from(node.arc_count)).rev() {
                let arc = self.package.record::<ProductiveTrieArcRecordV1>(
                    ProductiveSectionKindV1::TrieArcs,
                    node.arc_start as usize + offset,
                )?;
                let mut child = frame.clone();
                child.node_id = arc.child_node;
                if self.advance_packaged_arc(arc, &source, &mut child, &mut trace_arena)? {
                    stack.push(child);
                }
            }
        }
        Ok(generated)
    }

    pub(super) fn evaluate_shadow(
        &self,
        observed_surface: &str,
        scene: &L2LocalSceneV1,
        grounded_lemmas: &[PackagedGroundedLemmaV1],
        grounded_winner_conflict: bool,
    ) -> PackagedProductiveReadoutV1 {
        self.evaluate_shadow_with_cold_bindings(
            observed_surface,
            scene,
            grounded_lemmas,
            &[],
            grounded_winner_conflict,
        )
    }

    pub(super) fn evaluate_shadow_with_cold_bindings(
        &self,
        observed_surface: &str,
        scene: &L2LocalSceneV1,
        grounded_lemmas: &[PackagedGroundedLemmaV1],
        cold_bindings: &[ColdLemmaBindingV1],
        grounded_winner_conflict: bool,
    ) -> PackagedProductiveReadoutV1 {
        match self.evaluate_checked(
            observed_surface,
            scene,
            grounded_lemmas,
            cold_bindings,
            grounded_winner_conflict,
            None,
            None,
            ProductiveEvaluationModeV1::ContextShaped,
            None,
        ) {
            Ok(readout) => readout,
            Err(error) => PackagedProductiveReadoutV1 {
                verdict: ProductiveCalibratedVerdictV1::Abstain {
                    suggestions: Vec::new(),
                    productive_overflow: false,
                },
                candidates: Vec::new(),
                logical_terminal_count: 0,
                logical_surface_basin_count: 0,
                integrity_error: Some(error),
            },
        }
    }

    pub(super) fn evaluate_shadow_with_cold_bindings_profiled(
        &self,
        observed_surface: &str,
        scene: &L2LocalSceneV1,
        grounded_lemmas: &[PackagedGroundedLemmaV1],
        cold_bindings: &[ColdLemmaBindingV1],
        grounded_winner_conflict: bool,
    ) -> (PackagedProductiveReadoutV1, ProductiveEvaluationTelemetryV1) {
        let mut telemetry = ProductiveEvaluationTelemetryV1::default();
        let readout = match self.evaluate_checked(
            observed_surface,
            scene,
            grounded_lemmas,
            cold_bindings,
            grounded_winner_conflict,
            None,
            Some(&mut telemetry),
            ProductiveEvaluationModeV1::ContextShaped,
            None,
        ) {
            Ok(readout) => readout,
            Err(error) => PackagedProductiveReadoutV1 {
                verdict: ProductiveCalibratedVerdictV1::Abstain {
                    suggestions: Vec::new(),
                    productive_overflow: false,
                },
                candidates: Vec::new(),
                logical_terminal_count: 0,
                logical_surface_basin_count: 0,
                integrity_error: Some(error),
            },
        };
        (readout, telemetry)
    }

    pub(super) fn evaluate_shadow_with_cold_bindings_probed(
        &self,
        observed_surface: &str,
        scene: &L2LocalSceneV1,
        grounded_lemmas: &[PackagedGroundedLemmaV1],
        cold_bindings: &[ColdLemmaBindingV1],
        grounded_winner_conflict: bool,
        target_probe: &mut ProductiveTargetProbeV1,
    ) -> PackagedProductiveReadoutV1 {
        match self.evaluate_checked(
            observed_surface,
            scene,
            grounded_lemmas,
            cold_bindings,
            grounded_winner_conflict,
            Some(target_probe),
            None,
            ProductiveEvaluationModeV1::ContextShaped,
            None,
        ) {
            Ok(readout) => readout,
            Err(error) => PackagedProductiveReadoutV1 {
                verdict: ProductiveCalibratedVerdictV1::Abstain {
                    suggestions: Vec::new(),
                    productive_overflow: false,
                },
                candidates: Vec::new(),
                logical_terminal_count: 0,
                logical_surface_basin_count: 0,
                integrity_error: Some(error),
            },
        }
    }

    pub(super) fn enumerate_context_neutral_material(
        &self,
        observed_surface: &str,
        grounded_lemmas: &[PackagedGroundedLemmaV1],
        cold_bindings: &[ColdLemmaBindingV1],
        preparatory_work: EnumerationWorkCountersV1,
        budget: EnumerationWorkBudgetV1,
    ) -> ContextNeutralProductiveEnumerationV1 {
        let scene = L2LocalSceneV1 {
            current_token: observed_surface.to_string(),
            current_normalized_scalars: observed_surface.chars().map(u32::from).collect(),
            ..L2LocalSceneV1::default()
        };
        let mut limiter = match ProductiveWorkLimiterV1::new(budget, preparatory_work) {
            Ok(limiter) => limiter,
            Err(_) => {
                return ContextNeutralProductiveEnumerationV1 {
                    readout: empty_productive_readout(),
                    productive_work: EnumerationWorkCountersV1::default(),
                    aggregate_work: preparatory_work,
                    work_budget_exceeded: true,
                };
            }
        };
        let result = self.evaluate_checked(
            observed_surface,
            &scene,
            grounded_lemmas,
            cold_bindings,
            false,
            None,
            None,
            ProductiveEvaluationModeV1::ContextNeutralMaterial,
            Some(&mut limiter),
        );
        let work_budget_exceeded = result
            .as_ref()
            .is_err_and(|error| error == WORK_BUDGET_EXCEEDED);
        let readout = match result {
            Ok(readout) => readout,
            Err(error) if error == WORK_BUDGET_EXCEEDED => empty_productive_readout(),
            Err(error) => PackagedProductiveReadoutV1 {
                integrity_error: Some(error),
                ..empty_productive_readout()
            },
        };
        ContextNeutralProductiveEnumerationV1 {
            readout,
            productive_work: limiter.productive_work,
            aggregate_work: limiter.aggregate_work(),
            work_budget_exceeded,
        }
    }

    fn evaluate_checked(
        &self,
        observed_surface: &str,
        scene: &L2LocalSceneV1,
        grounded_lemmas: &[PackagedGroundedLemmaV1],
        cold_bindings: &[ColdLemmaBindingV1],
        grounded_winner_conflict: bool,
        mut target_probe: Option<&mut ProductiveTargetProbeV1>,
        mut telemetry: Option<&mut ProductiveEvaluationTelemetryV1>,
        mode: ProductiveEvaluationModeV1,
        mut work_limiter: Option<&mut ProductiveWorkLimiterV1>,
    ) -> Result<PackagedProductiveReadoutV1, String> {
        let stage_started = telemetry.as_ref().map(|_| Instant::now());
        scene.validate().map_err(str::to_string)?;
        let observed = ObservedGeometryV1::new(observed_surface).map_err(str::to_string)?;
        if observed.characters.len() > usize::from(self.package.header.maximum_observed_scalars) {
            return Err("productive observation exceeds the package scalar bound".to_string());
        }
        let (wave, scene_key) = match mode {
            ProductiveEvaluationModeV1::ContextShaped => (
                encode_scene_wave(scene),
                directional_scene_key(scene).map_err(str::to_string)?,
            ),
            ProductiveEvaluationModeV1::ContextNeutralMaterial => (SceneWaveV1::default(), 0),
        };
        let lemmas = self.select_grounded_lemmas(grounded_lemmas)?;
        let mut base_surface_basins = BTreeMap::<SurfaceBasinKeyV1, SurfaceBasinV1>::new();
        let mut recovered_surface_basins = BTreeMap::<SurfaceBasinKeyV1, SurfaceBasinV1>::new();
        let coalesce_slots = self.shared_replay_mode == SharedReplayModeV1::SemanticProofAuthority;
        let mut logical_terminal_count = 0_u64;
        let mut relation_replay_count = 0_u64;
        let mut operator_step_count = 0_u64;
        let mut batch_geometry =
            BatchGeometryEvaluatorV1::new(&observed).map_err(str::to_string)?;
        if let (Some(telemetry), Some(stage_started)) = (telemetry.as_deref_mut(), stage_started) {
            telemetry.setup_us = stage_started.elapsed().as_micros() as u64;
        }

        let stage_started = telemetry.as_ref().map(|_| Instant::now());
        let mut active_bindings = Vec::new();
        for lemma in lemmas {
            active_bindings.extend(self.active_bindings(lemma)?);
        }
        for binding in cold_bindings {
            binding
                .validate(self.package.header.maximum_observed_scalars)
                .map_err(str::to_string)?;
            active_bindings.push(binding.clone().into_active());
        }
        active_bindings.sort_by(|left, right| {
            left.rank_origin
                .cmp(&right.rank_origin)
                .then_with(|| {
                    right
                        .lemma
                        .grounded_support
                        .cmp(&left.lemma.grounded_support)
                })
                .then_with(|| left.lemma.lemma_id.cmp(&right.lemma.lemma_id))
                .then_with(|| left.paradigm_id.cmp(&right.paradigm_id))
                .then_with(|| left.lemma.source_slot_id.cmp(&right.lemma.source_slot_id))
                .then_with(|| {
                    left.lemma
                        .canonical_source_form_ref
                        .cmp(&right.lemma.canonical_source_form_ref)
                })
                .then_with(|| {
                    left.lemma
                        .normalized_source
                        .cmp(&right.lemma.normalized_source)
                })
        });
        active_bindings.dedup_by(|left, right| {
            (
                left.lemma.lemma_id,
                left.paradigm_id,
                left.lemma.source_slot_id,
                left.lemma.canonical_source_form_ref,
                &left.lemma.normalized_source,
            ) == (
                right.lemma.lemma_id,
                right.paradigm_id,
                right.lemma.source_slot_id,
                right.lemma.canonical_source_form_ref,
                &right.lemma.normalized_source,
            ) && left.rank_origin == right.rank_origin
        });
        let mut admitted_lemmas = BTreeSet::new();
        active_bindings.retain(|binding| {
            admitted_lemmas.contains(&binding.lemma.lemma_id)
                || admitted_lemmas.len() < super::super::CANONICAL_L2_ACTIVE_LEMMA_LIMIT
                    && admitted_lemmas.insert(binding.lemma.lemma_id)
        });
        if self.semantic_transducer.is_some() {
            active_bindings.sort_unstable_by(|left, right| {
                left.lemma
                    .normalized_source
                    .cmp(&right.lemma.normalized_source)
                    .then_with(|| left.rank_origin.cmp(&right.rank_origin))
                    .then_with(|| left.lemma.lemma_id.cmp(&right.lemma.lemma_id))
                    .then_with(|| left.paradigm_id.cmp(&right.paradigm_id))
                    .then_with(|| left.lemma.source_slot_id.cmp(&right.lemma.source_slot_id))
                    .then_with(|| {
                        left.lemma
                            .canonical_source_form_ref
                            .cmp(&right.lemma.canonical_source_form_ref)
                    })
            });
        }
        if let (Some(telemetry), Some(stage_started)) = (telemetry.as_deref_mut(), stage_started) {
            telemetry.binding_preparation_us = stage_started.elapsed().as_micros() as u64;
            telemetry.active_binding_count = active_bindings.len() as u64;
        }

        let stage_started = telemetry.as_ref().map(|_| Instant::now());
        let mut execution_lane = self
            .semantic_transducer
            .as_ref()
            .map(|index| DenseProgramExecutionLaneV1::new(index.execution_class_count()))
            .transpose()?;
        let mut execution_source = String::new();
        let mut execution_source_scalars = Vec::new();
        let mut execution_output = String::new();
        let mut has_execution_source = false;
        for binding in active_bindings {
            if !has_execution_source || execution_source != binding.lemma.normalized_source {
                if let Some(lane) = execution_lane.as_mut() {
                    lane.begin_source();
                }
                execution_source.clear();
                execution_source.push_str(&binding.lemma.normalized_source);
                execution_source_scalars.clear();
                execution_source_scalars.extend(binding.lemma.normalized_source.chars());
                execution_output.clear();
                has_execution_source = true;
            }
            let paradigm = self.paradigm(binding.paradigm_id)?;
            let slot_profiles = self.prepared_slot_profiles(paradigm)?;
            if slot_profiles.is_empty() {
                continue;
            }
            let binding_candidates = self.traverse_binding(
                &observed,
                &mut batch_geometry,
                &wave,
                scene_key,
                &binding,
                paradigm,
                &slot_profiles,
                execution_lane.as_mut(),
                &execution_source_scalars,
                &mut execution_output,
                &mut logical_terminal_count,
                &mut relation_replay_count,
                &mut operator_step_count,
                target_probe.as_deref_mut(),
                mode,
                work_limiter.as_deref_mut(),
            )?;
            for candidate in binding_candidates {
                match candidate.output.rank_origin {
                    CandidateRankOriginV1::BaseV64 => {
                        retain_surface_basin(&mut base_surface_basins, candidate, coalesce_slots)
                    }
                    CandidateRankOriginV1::RecoveredV66 => retain_surface_basin(
                        &mut recovered_surface_basins,
                        candidate,
                        coalesce_slots,
                    ),
                }
            }
        }
        if let (Some(telemetry), Some(stage_started)) = (telemetry.as_deref_mut(), stage_started) {
            telemetry.traversal_us = stage_started.elapsed().as_micros() as u64;
            telemetry.logical_terminal_count = logical_terminal_count;
            telemetry.relation_replay_count = relation_replay_count;
            telemetry.operator_step_count = operator_step_count;
        }

        let stage_started = telemetry.as_ref().map(|_| Instant::now());
        let logical_surface_basin_count = u64::try_from(
            base_surface_basins
                .keys()
                .chain(recovered_surface_basins.keys())
                .collect::<BTreeSet<_>>()
                .len(),
        )
        .map_err(|_| "productive surface basin count exceeds u64".to_string())?;
        if coalesce_slots {
            absorb_shared_recovered_identities(
                &mut base_surface_basins,
                &mut recovered_surface_basins,
            )?;
        }

        let mut base_basins = base_surface_basins.into_values().collect::<Vec<_>>();
        base_basins.sort_by(|left, right| {
            ranked_candidate_order(&left.representative, &right.representative)
        });
        let dropped_best = base_basins
            .get(super::runtime::PRODUCTIVE_PHYSICAL_TOP_K)
            .map(|basin| basin.representative.clone());
        base_basins.truncate(super::runtime::PRODUCTIVE_PHYSICAL_TOP_K);
        let mut base_heap = base_basins
            .into_iter()
            .map(SurfaceBasinV1::into_representative)
            .collect::<Vec<_>>();

        let mut recovered_basins = recovered_surface_basins.into_values().collect::<Vec<_>>();
        recovered_basins.sort_by(|left, right| {
            ranked_candidate_order(&left.representative, &right.representative)
        });
        let recovered_overflow = recovered_basins.len() > super::runtime::PRODUCTIVE_PHYSICAL_TOP_K;
        recovered_basins.truncate(super::runtime::PRODUCTIVE_PHYSICAL_TOP_K);
        let mut recovered_heap = recovered_basins
            .into_iter()
            .map(SurfaceBasinV1::into_representative)
            .collect::<Vec<_>>();
        let base_keys = base_heap
            .iter()
            .map(|candidate| SurfaceBasinKeyV1::from_candidate(candidate, coalesce_slots))
            .collect::<BTreeSet<_>>();
        recovered_heap.retain(|candidate| {
            !base_keys.contains(&SurfaceBasinKeyV1::from_candidate(
                candidate,
                coalesce_slots,
            ))
        });
        base_heap.extend(recovered_heap);
        base_heap.sort_by(rank_preserving_candidate_order);
        let mut candidates = base_heap
            .into_iter()
            .map(|candidate| candidate.output)
            .collect::<Vec<_>>();
        if let (Some(telemetry), Some(stage_started)) = (telemetry.as_deref_mut(), stage_started) {
            telemetry.surface_reduce_us = stage_started.elapsed().as_micros() as u64;
            telemetry.logical_surface_basin_count = logical_surface_basin_count;
            telemetry.selected_candidate_count = candidates.len() as u64;
        }

        let stage_started = telemetry.as_ref().map(|_| Instant::now());
        for candidate in &mut candidates {
            candidate.geometry = batch_geometry
                .evaluate(&candidate.normalized_surface)
                .map_err(str::to_string)?;
        }
        if let Some(probe) = target_probe.as_deref_mut() {
            for candidate in &candidates {
                probe.observe_post_surface_basin_bound(candidate);
            }
        }
        let cross_lemma_ownership_satisfied = candidates
            .iter()
            .map(|candidate| candidate.identity.lemma_id)
            .collect::<BTreeSet<_>>()
            .len()
            <= 1;
        let ambiguity_kind = packaged_ambiguity_kind(&candidates, logical_surface_basin_count);
        let selected_calibration = if let Some(leader) = candidates.first() {
            let stratum = ObservableCalibrationStratumV1::new(
                observed_surface,
                &leader.normalized_surface,
                leader.provenance,
                leader.minimum_independent_support,
                ambiguity_kind,
            )
            .map_err(str::to_string)?;
            self.calibration_cell(&stratum, None)?
        } else {
            None
        };
        let productive_overflow = match (
            selected_calibration,
            dropped_best.as_ref(),
            candidates.first(),
        ) {
            (Some((_, cell)), Some(dropped), Some(leader)) => {
                recovered_overflow
                    || leader
                        .score_q16
                        .checked_sub(dropped.output.score_q16)
                        .is_none_or(|difference| difference <= i64::from(cell.tie_radius_q16))
            }
            (None, Some(_), _) => true,
            _ => recovered_overflow,
        };
        let readout_candidates = candidates
            .iter()
            .map(|candidate| ReadoutCandidateV1 {
                identity: candidate.identity,
                equivalent_identities: candidate.equivalent_identities.clone(),
                normalized_surface: candidate.normalized_surface.to_string(),
                score_q16: candidate.score_q16,
                grounded_lemma_evidence: candidate.grounded_support,
                exact_osa_distance: candidate
                    .geometry
                    .character_distance
                    .min(candidate.geometry.keyboard_distance),
                exact_form: false,
                cross_lemma_ownership_satisfied,
                rank_origin: candidate.rank_origin,
                cross_lane_certified: candidate.cross_lane_certified,
            })
            .collect();
        let verdict = calibrated_readout_packaged(
            selected_calibration,
            readout_candidates,
            productive_overflow,
            grounded_winner_conflict,
            None,
        );
        candidates.sort_by(packaged_rank_preserving_order);
        if let (Some(telemetry), Some(stage_started)) = (telemetry, stage_started) {
            telemetry.final_readout_us = stage_started.elapsed().as_micros() as u64;
        }
        Ok(PackagedProductiveReadoutV1 {
            verdict,
            candidates,
            logical_terminal_count,
            logical_surface_basin_count,
            integrity_error: None,
        })
    }

    #[cfg(test)]
    pub(super) fn verify_exposed_replay_parity(
        &self,
        binding: &ColdLemmaBindingV1,
        exposed: &[ColdLemmaSourceV1],
    ) -> Result<(), String> {
        let paradigm = self.paradigm(binding.paradigm_id)?;
        let source = ColdLemmaSourceV1 {
            pos_domain: binding.lemma.pos_domain,
            canonical_source_form_ref: binding.lemma.canonical_source_form_ref,
            source_slot_id: binding.lemma.source_slot_id,
            normalized_source: binding.lemma.normalized_source.clone(),
            grounded_support: binding.lemma.grounded_support,
            canonical_preference: 0,
            canonical_source: true,
        };
        let mut constraints = ExposedFormConstraintsV1::default();
        for target in exposed
            .iter()
            .filter(|target| target.pos_domain == source.pos_domain)
        {
            constraints.insert(target.source_slot_id, &target.normalized_source);
        }
        let direct = self
            .paradigm_reconstructs_exposed_forms(paradigm, &source, &constraints)?
            .0;
        let generated = self.instantiate_paradigm_surfaces(
            binding.paradigm_id,
            paradigm,
            source.source_slot_id,
            &source.normalized_source,
        )?;
        let complete = exposed
            .iter()
            .filter(|target| target.pos_domain == source.pos_domain)
            .all(|target| {
                generated
                    .get(&target.source_slot_id)
                    .is_some_and(|surfaces| surfaces.contains(&target.normalized_source))
            });
        if direct != complete {
            return Err("exposed-slot replay differs from complete-trie oracle".to_string());
        }
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn verify_cold_execution_parity(
        &self,
        observed_surface: &str,
        scene: &L2LocalSceneV1,
        binding: &ColdLemmaBindingV1,
    ) -> Result<(), String> {
        let observed = ObservedGeometryV1::new(observed_surface).map_err(str::to_string)?;
        let wave = encode_scene_wave(scene);
        let scene_key = directional_scene_key(scene).map_err(str::to_string)?;
        let active = binding.clone().into_active();
        let paradigm = self.paradigm(active.paradigm_id)?;
        let selected_slots = self.prepared_slot_profiles(paradigm)?;
        let mut batch_geometry =
            BatchGeometryEvaluatorV1::new(&observed).map_err(str::to_string)?;
        let mut direct_count = 0;
        let mut direct_relation_replay_count = 0;
        let mut direct_operator_step_count = 0;
        let mut execution_lane = self
            .semantic_transducer
            .as_ref()
            .map(|index| DenseProgramExecutionLaneV1::new(index.execution_class_count()))
            .transpose()?;
        let source_scalars = active.lemma.normalized_source.chars().collect::<Vec<_>>();
        let mut execution_output = String::new();
        let mut direct = self.traverse_binding(
            &observed,
            &mut batch_geometry,
            &wave,
            scene_key,
            &active,
            paradigm,
            &selected_slots,
            execution_lane.as_mut(),
            &source_scalars,
            &mut execution_output,
            &mut direct_count,
            &mut direct_relation_replay_count,
            &mut direct_operator_step_count,
            None,
            ProductiveEvaluationModeV1::ContextShaped,
            None,
        )?;
        for candidate in &mut direct {
            candidate.output.geometry = batch_geometry
                .evaluate(&candidate.output.normalized_surface)
                .map_err(str::to_string)?;
        }
        let mut complete_count = 0;
        let mut complete_relation_replay_count = 0;
        let mut complete_operator_step_count = 0;
        let complete = self.traverse_binding_complete_trie(
            &observed,
            &wave,
            scene_key,
            &active,
            paradigm,
            &selected_slots,
            &mut complete_count,
            &mut complete_relation_replay_count,
            &mut complete_operator_step_count,
            None,
            ProductiveEvaluationModeV1::ContextShaped,
            None,
        )?;
        if direct_count != complete_count || direct != complete {
            return Err("direct cold execution differs from complete-trie oracle".to_string());
        }
        Ok(())
    }

    pub(super) fn cold_binding_has_slot(
        &self,
        binding: &ColdLemmaBindingV1,
        target_slot_id: u32,
    ) -> Result<bool, String> {
        binding
            .validate(self.package.header.maximum_observed_scalars)
            .map_err(str::to_string)?;
        let paradigm = self.paradigm(binding.paradigm_id)?;
        Ok(self
            .prepared_slot_profiles(paradigm)?
            .binary_search_by_key(&target_slot_id, |profile| profile.slot_id)
            .is_ok())
    }

    fn select_grounded_lemmas(
        &self,
        grounded_lemmas: &[PackagedGroundedLemmaV1],
    ) -> Result<Vec<PackagedGroundedLemmaV1>, String> {
        let mut lemmas = grounded_lemmas.to_vec();
        for lemma in &lemmas {
            lemma
                .validate(self.package.header.maximum_observed_scalars)
                .map_err(str::to_string)?;
        }
        lemmas.sort_by(|left, right| {
            right
                .grounded_support
                .cmp(&left.grounded_support)
                .then_with(|| left.lemma_id.cmp(&right.lemma_id))
                .then_with(|| left.source_slot_id.cmp(&right.source_slot_id))
                .then_with(|| {
                    left.canonical_source_form_ref
                        .cmp(&right.canonical_source_form_ref)
                })
        });
        lemmas.dedup_by(|left, right| {
            (left.lemma_id, left.pos_domain) == (right.lemma_id, right.pos_domain)
        });
        lemmas.truncate(super::super::CANONICAL_L2_ACTIVE_LEMMA_LIMIT);
        Ok(lemmas)
    }

    fn active_bindings(
        &self,
        lemma: PackagedGroundedLemmaV1,
    ) -> Result<Vec<ActiveBindingV1>, String> {
        let mut bindings = Vec::new();
        let count = self
            .package
            .record_count(ProductiveSectionKindV1::LemmaBindings);
        let start = self.lower_bound_binding(lemma.lemma_id)?;
        let mut packaged_lemma_seen = false;
        for index in start..count {
            let binding = self
                .package
                .record::<LemmaParadigmBindingV1>(ProductiveSectionKindV1::LemmaBindings, index)?;
            if binding.lemma_id != lemma.lemma_id {
                break;
            }
            packaged_lemma_seen = true;
            if binding.canonical_source_form_ref != lemma.canonical_source_form_ref {
                continue;
            }
            let mut observed_slots = self
                .package
                .observed_slot_ids(binding.observed_slot_set_ref)?;
            observed_slots.sort_unstable();
            observed_slots.dedup();
            bindings.push(ActiveBindingV1 {
                lemma: lemma.clone(),
                binding: Some(binding),
                paradigm_id: binding.paradigm_id,
                observed_slots,
                provenance: CandidateProvenanceClassV1::TrainingSeenGenerated,
                rank_origin: CandidateRankOriginV1::BaseV64,
                cross_lane_certified: false,
            });
        }
        if !bindings.is_empty() {
            return Ok(bindings);
        }
        if packaged_lemma_seen {
            return Ok(Vec::new());
        }
        for paradigm_id in self.compatible_paradigms(lemma.pos_domain, lemma.source_slot_id)? {
            bindings.push(ActiveBindingV1 {
                lemma: lemma.clone(),
                binding: None,
                paradigm_id,
                observed_slots: vec![lemma.source_slot_id],
                provenance: CandidateProvenanceClassV1::ColdLemmaBinding,
                rank_origin: CandidateRankOriginV1::BaseV64,
                cross_lane_certified: false,
            });
        }
        Ok(bindings)
    }

    fn lower_bound_binding(&self, lemma_id: u32) -> Result<usize, String> {
        let mut low = 0_usize;
        let mut high = self
            .package
            .record_count(ProductiveSectionKindV1::LemmaBindings);
        while low < high {
            let middle = low + (high - low) / 2;
            let binding = self
                .package
                .record::<LemmaParadigmBindingV1>(ProductiveSectionKindV1::LemmaBindings, middle)?;
            if binding.lemma_id < lemma_id {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        Ok(low)
    }

    fn compatible_paradigms(
        &self,
        pos_domain: u16,
        source_slot_id: u32,
    ) -> Result<Vec<u32>, String> {
        let count = self
            .package
            .record_count(ProductiveSectionKindV1::ParadigmCompatibilityIndex);
        let mut low = 0_usize;
        let mut high = count;
        while low < high {
            let middle = low + (high - low) / 2;
            let row = self.package.record::<ParadigmCompatibilityIndexRecordV1>(
                ProductiveSectionKindV1::ParadigmCompatibilityIndex,
                middle,
            )?;
            if (row.pos_domain, row.source_slot_id) < (pos_domain, source_slot_id) {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        if low == count {
            return Ok(Vec::new());
        }
        let row = self.package.record::<ParadigmCompatibilityIndexRecordV1>(
            ProductiveSectionKindV1::ParadigmCompatibilityIndex,
            low,
        )?;
        if (row.pos_domain, row.source_slot_id) != (pos_domain, source_slot_id) {
            return Ok(Vec::new());
        }
        (0..row.posting_count as usize)
            .map(|offset| {
                self.package
                    .record::<ParadigmPostingRecordV1>(
                        ProductiveSectionKindV1::ParadigmPostings,
                        row.posting_start as usize + offset,
                    )
                    .map(|posting| posting.paradigm_id)
            })
            .collect()
    }

    fn paradigm(&self, paradigm_id: u32) -> Result<ParadigmCenterRecordV1, String> {
        let index = paradigm_id
            .checked_sub(1)
            .ok_or_else(|| "productive paradigm identity is zero".to_string())?
            as usize;
        self.paradigms
            .get(index)
            .copied()
            .ok_or_else(|| "productive paradigm identity exceeds prepared records".to_string())
    }

    fn program(&self, index: usize) -> Result<&PreparedMorphProgramV1, String> {
        self.programs
            .get(index)
            .ok_or_else(|| "productive program identity exceeds prepared records".to_string())
    }

    fn program_operations(
        &self,
        program: &PreparedMorphProgramV1,
    ) -> Result<&[MorphOpRecordV1], String> {
        let start = program.record.op_start as usize;
        let end = start
            .checked_add(program.record.op_count as usize)
            .ok_or_else(|| "productive prepared operation range overflows".to_string())?;
        self.operations
            .get(start..end)
            .ok_or_else(|| "productive prepared operation range is invalid".to_string())
    }

    fn terminal(&self, index: usize) -> Result<ProductiveTerminalRecordV1, String> {
        self.terminals
            .get(index)
            .copied()
            .ok_or_else(|| "productive terminal identity exceeds prepared records".to_string())
    }

    fn prepared_slot_profiles(
        &self,
        paradigm: ParadigmCenterRecordV1,
    ) -> Result<&[SlotPhaseProfileRecordV1], String> {
        let start = paradigm.slot_profile_start as usize;
        let end = start
            .checked_add(paradigm.slot_profile_count as usize)
            .ok_or_else(|| "productive prepared slot-profile range overflows".to_string())?;
        self.slot_profiles
            .get(start..end)
            .ok_or_else(|| "productive prepared slot-profile range is invalid".to_string())
    }

    #[allow(clippy::too_many_arguments)]
    fn traverse_binding(
        &self,
        observed: &ObservedGeometryV1,
        batch_geometry: &mut BatchGeometryEvaluatorV1,
        wave: &SceneWaveV1,
        scene_key: u32,
        binding: &ActiveBindingV1,
        paradigm: ParadigmCenterRecordV1,
        selected_slots: &[SlotPhaseProfileRecordV1],
        mut execution_lane: Option<&mut DenseProgramExecutionLaneV1>,
        source_scalars: &[char],
        normalized_surface: &mut String,
        logical_terminal_count: &mut u64,
        relation_replay_count: &mut u64,
        operator_step_count: &mut u64,
        mut target_probe: Option<&mut ProductiveTargetProbeV1>,
        mode: ProductiveEvaluationModeV1,
        mut work_limiter: Option<&mut ProductiveWorkLimiterV1>,
    ) -> Result<Vec<RankedCandidateV1>, String> {
        // Keep the sidecar-disabled runtime as the independent V64 trie oracle.
        // The validated V66 runtime executes the same packaged programs directly.
        if binding.binding.is_some() && self.anchor_recovery.is_none() {
            return self.traverse_binding_complete_trie(
                observed,
                wave,
                scene_key,
                binding,
                paradigm,
                selected_slots,
                logical_terminal_count,
                relation_replay_count,
                operator_step_count,
                target_probe,
                mode,
                work_limiter,
            );
        }

        let prepared_slots = self.prepare_slot_evaluations(
            binding,
            paradigm,
            selected_slots,
            wave,
            scene_key,
            mode,
        )?;
        let mut frontier = BindingCandidateFrontierV1::new(
            prepared_slots.iter().map(|slot| slot.profile.slot_id),
        )?;
        for offset in 0..paradigm.program_count as usize {
            let program_index = paradigm.program_start as usize + offset;
            let program_id = u32::try_from(program_index + 1)
                .map_err(|_| "productive program identity exceeds u32".to_string())?;
            let program = self.program(program_index)?;
            if program.record.source_slot_id != binding.lemma.source_slot_id {
                continue;
            }
            let Ok(profile_index) = prepared_slots
                .binary_search_by_key(&program.record.target_slot_id, |selected| {
                    selected.profile.slot_id
                })
            else {
                continue;
            };
            if let Some(limiter) = work_limiter.as_deref_mut() {
                limiter.consume(EnumerationWorkCountersV1 {
                    relation_replays: 1,
                    ..EnumerationWorkCountersV1::default()
                })?;
            }
            *relation_replay_count = relation_replay_count
                .checked_add(1)
                .ok_or_else(|| "productive relation-replay count overflow".to_string())?;
            let execution_class_id = execution_lane
                .as_ref()
                .map(|_| {
                    self.semantic_transducer
                        .as_ref()
                        .ok_or_else(|| {
                            "productive semantic execution lane has no index".to_string()
                        })?
                        .execution_class_for_program(program_index)
                })
                .transpose()?;
            let cached_execution = match (execution_class_id, execution_lane.as_deref_mut()) {
                (Some(class_id), Some(lane)) => lane.get(class_id)?.cloned(),
                _ => None,
            };
            let execution = if let Some(execution) = cached_execution {
                execution
            } else {
                if let Some(limiter) = work_limiter.as_deref_mut() {
                    limiter.consume(EnumerationWorkCountersV1 {
                        operator_steps: u64::from(program.record.op_count),
                        ..EnumerationWorkCountersV1::default()
                    })?;
                }
                *operator_step_count = operator_step_count
                    .checked_add(u64::from(program.record.op_count))
                    .ok_or_else(|| "productive operator-step count overflow".to_string())?;
                let execution = if self.execute_packaged_program_into(
                    program,
                    source_scalars,
                    normalized_surface,
                )? {
                    let (geometry, normalized_surface) = batch_geometry
                        .evaluate_for_ranking_interned(normalized_surface)
                        .map_err(str::to_string)?;
                    DenseProgramExecutionV1::Accepted {
                        normalized_surface,
                        geometry,
                    }
                } else {
                    DenseProgramExecutionV1::Rejected
                };
                if let (Some(class_id), Some(lane)) =
                    (execution_class_id, execution_lane.as_deref_mut())
                {
                    lane.insert(class_id, execution.clone())?;
                }
                execution
            };
            let DenseProgramExecutionV1::Accepted {
                normalized_surface: interned_surface,
                geometry,
            } = execution
            else {
                continue;
            };
            let terminal = self
                .terminal_for_program_id(program_id)?
                .ok_or_else(|| "productive program has no terminal attribution".to_string())?;
            if terminal.target_slot_id != program.record.target_slot_id {
                return Err("productive direct replay terminal slot mismatch".to_string());
            }
            let selected_profile = prepared_slots[profile_index];
            let score_q16 = fixed_point_score_q16(
                &self.coefficients_q16,
                selected_profile
                    .invariant_features
                    .with_geometry(geometry)
                    .map_err(str::to_string)?,
            )
            .map_err(str::to_string)?;
            let provenance = binding.candidate_provenance(terminal.target_slot_id);
            let identity = ProductiveCandidateIdentityV1 {
                lemma_id: binding.lemma.lemma_id,
                paradigm_id: binding.paradigm_id,
                program_id: terminal.program_id,
                target_slot_id: terminal.target_slot_id,
                normalized_surface_id: terminal.stable_identity_hash,
                variant_id: terminal.variant_id,
            };
            if let Some(limiter) = work_limiter.as_deref_mut() {
                limiter.consume(EnumerationWorkCountersV1 {
                    generated_logical_targets: 1,
                    ..EnumerationWorkCountersV1::default()
                })?;
            }
            *logical_terminal_count = logical_terminal_count
                .checked_add(1)
                .ok_or_else(|| "productive logical terminal count overflow".to_string())?;
            let candidate = RankedCandidateV1 {
                exact_osa_distance: geometry.character_distance.min(geometry.keyboard_distance),
                output: PackagedProductiveCandidateV1 {
                    identity,
                    equivalent_identities: Vec::new(),
                    normalized_surface: interned_surface,
                    score_q16,
                    geometry,
                    provenance,
                    minimum_independent_support: selected_profile.minimum_independent_support,
                    grounded_support: binding.lemma.grounded_support,
                    ambiguity_center_cosine: selected_profile.ambiguity_center_cosine,
                    equivalent_identity_count: 1,
                    equivalent_paradigm_count: 1,
                    minimum_equivalent_support: selected_profile.minimum_independent_support,
                    maximum_equivalent_support: selected_profile.minimum_independent_support,
                    rank_origin: binding.rank_origin,
                    cross_lane_certified: binding.cross_lane_certified,
                },
            };
            if let Some(probe) = target_probe.as_deref_mut() {
                probe.observe_pre_slot_bound(&candidate.output);
            }
            frontier.retain(profile_index, candidate)?;
        }
        let selected = frontier.into_geometry_selected(super::super::CANONICAL_L2_FEATURE_LIMIT);
        if let Some(probe) = target_probe {
            for candidate in &selected {
                probe.observe_post_slot_bound(&candidate.output);
            }
        }
        Ok(selected)
    }

    fn terminal_for_program_id(
        &self,
        program_id: u32,
    ) -> Result<Option<ProductiveTerminalRecordV1>, String> {
        let Some(index) = program_id
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| self.terminal_index_by_program.get(index))
            .copied()
            .filter(|index| *index != 0)
        else {
            return Ok(None);
        };
        self.terminal(index as usize - 1).map(Some)
    }

    #[allow(clippy::too_many_arguments)]
    fn traverse_binding_complete_trie(
        &self,
        observed: &ObservedGeometryV1,
        wave: &SceneWaveV1,
        scene_key: u32,
        binding: &ActiveBindingV1,
        paradigm: ParadigmCenterRecordV1,
        selected_slots: &[SlotPhaseProfileRecordV1],
        logical_terminal_count: &mut u64,
        relation_replay_count: &mut u64,
        operator_step_count: &mut u64,
        mut target_probe: Option<&mut ProductiveTargetProbeV1>,
        mode: ProductiveEvaluationModeV1,
        mut work_limiter: Option<&mut ProductiveWorkLimiterV1>,
    ) -> Result<Vec<RankedCandidateV1>, String> {
        let prepared_slots = self.prepare_slot_evaluations(
            binding,
            paradigm,
            selected_slots,
            wave,
            scene_key,
            mode,
        )?;
        let source = binding.lemma.normalized_source.chars().collect::<Vec<_>>();
        let mut trace_arena = ScalarTraceArenaV1::default();
        let mut frontier = BindingCandidateFrontierV1::new(
            prepared_slots.iter().map(|slot| slot.profile.slot_id),
        )?;
        let mut stack = vec![TraversalFrameV1 {
            node_id: paradigm.root_node,
            source_cursor: 0,
            trace_ref: None,
            geometry: GeometryTraversalStateV1::new(
                observed,
                GeometryPathIdentityV1 {
                    lemma_id: binding.lemma.lemma_id,
                    paradigm_id: binding.paradigm_id,
                    ..GeometryPathIdentityV1::default()
                },
            )
            .map_err(str::to_string)?,
            exact_allomorph: false,
        }];

        while let Some(frame) = stack.pop() {
            let node = self.package.record::<ProductiveTrieNodeRecordV1>(
                ProductiveSectionKindV1::TrieNodes,
                frame.node_id as usize,
            )?;
            if frame.source_cursor == source.len() {
                for offset in 0..usize::from(node.terminal_count) {
                    let terminal = self.terminal(node.terminal_start as usize + offset)?;
                    let program = self.program(terminal.program_id as usize - 1)?.record;
                    if program.source_slot_id != binding.lemma.source_slot_id {
                        continue;
                    }
                    let Ok(profile_index) = prepared_slots
                        .binary_search_by_key(&terminal.target_slot_id, |selected| {
                            selected.profile.slot_id
                        })
                    else {
                        continue;
                    };
                    let selected_profile = prepared_slots[profile_index];
                    let mut geometry_state = frame.geometry.clone();
                    geometry_state.identity.slot_id = terminal.target_slot_id;
                    geometry_state.identity.program_id = terminal.program_id;
                    geometry_state.identity.variant_id = terminal.variant_id;
                    geometry_state.identity.decoder_trace_ref = frame.trace_ref.unwrap_or(u32::MAX);
                    let geometry = geometry_state.terminal_evidence();
                    let normalized_surface = trace_arena
                        .scalars(frame.trace_ref)
                        .map_err(str::to_string)?
                        .into_iter()
                        .map(|scalar| {
                            char::from_u32(scalar).ok_or_else(|| {
                                "productive decoder emitted an invalid scalar".to_string()
                            })
                        })
                        .collect::<Result<String, _>>()?;
                    if normalized_surface.is_empty()
                        || normalized_surface.chars().count()
                            > usize::from(self.package.header.maximum_generated_scalars)
                    {
                        return Err(
                            "productive terminal violates generated scalar bounds".to_string()
                        );
                    }
                    let score_q16 = fixed_point_score_q16(
                        &self.coefficients_q16,
                        selected_profile
                            .invariant_features
                            .with_geometry(geometry)
                            .map_err(str::to_string)?,
                    )
                    .map_err(str::to_string)?;
                    let provenance = binding.candidate_provenance(terminal.target_slot_id);
                    let identity = ProductiveCandidateIdentityV1 {
                        lemma_id: binding.lemma.lemma_id,
                        paradigm_id: binding.paradigm_id,
                        program_id: terminal.program_id,
                        target_slot_id: terminal.target_slot_id,
                        normalized_surface_id: terminal.stable_identity_hash,
                        variant_id: terminal.variant_id,
                    };
                    if let Some(limiter) = work_limiter.as_deref_mut() {
                        limiter.consume(EnumerationWorkCountersV1 {
                            generated_logical_targets: 1,
                            ..EnumerationWorkCountersV1::default()
                        })?;
                    }
                    *logical_terminal_count = logical_terminal_count
                        .checked_add(1)
                        .ok_or_else(|| "productive logical terminal count overflow".to_string())?;
                    let candidate = RankedCandidateV1 {
                        exact_osa_distance: geometry
                            .character_distance
                            .min(geometry.keyboard_distance),
                        output: PackagedProductiveCandidateV1 {
                            identity,
                            equivalent_identities: Vec::new(),
                            normalized_surface: Arc::from(normalized_surface),
                            score_q16,
                            geometry,
                            provenance,
                            minimum_independent_support: selected_profile
                                .minimum_independent_support,
                            grounded_support: binding.lemma.grounded_support,
                            ambiguity_center_cosine: selected_profile.ambiguity_center_cosine,
                            equivalent_identity_count: 1,
                            equivalent_paradigm_count: 1,
                            minimum_equivalent_support: selected_profile
                                .minimum_independent_support,
                            maximum_equivalent_support: selected_profile
                                .minimum_independent_support,
                            rank_origin: binding.rank_origin,
                            cross_lane_certified: binding.cross_lane_certified,
                        },
                    };
                    if let Some(probe) = target_probe.as_deref_mut() {
                        probe.observe_pre_slot_bound(&candidate.output);
                    }
                    frontier.retain(profile_index, candidate)?;
                }
            }
            for offset in (0..usize::from(node.arc_count)).rev() {
                if let Some(limiter) = work_limiter.as_deref_mut() {
                    limiter.consume(EnumerationWorkCountersV1 {
                        relation_replays: 1,
                        operator_steps: 1,
                        ..EnumerationWorkCountersV1::default()
                    })?;
                }
                *relation_replay_count = relation_replay_count
                    .checked_add(1)
                    .ok_or_else(|| "productive trie relation count overflow".to_string())?;
                *operator_step_count = operator_step_count
                    .checked_add(1)
                    .ok_or_else(|| "productive trie operator-step count overflow".to_string())?;
                let arc = self.package.record::<ProductiveTrieArcRecordV1>(
                    ProductiveSectionKindV1::TrieArcs,
                    node.arc_start as usize + offset,
                )?;
                let mut child = frame.clone();
                child.node_id = arc.child_node;
                if self.advance_packaged_arc(arc, &source, &mut child, &mut trace_arena)? {
                    stack.push(child);
                }
            }
        }
        let selected = frontier.into_geometry_selected(super::super::CANONICAL_L2_FEATURE_LIMIT);
        if let Some(probe) = target_probe {
            for candidate in &selected {
                probe.observe_post_slot_bound(&candidate.output);
            }
        }
        Ok(selected)
    }

    fn advance_packaged_arc(
        &self,
        arc: ProductiveTrieArcRecordV1,
        source: &[char],
        frame: &mut TraversalFrameV1,
        trace_arena: &mut ScalarTraceArenaV1,
    ) -> Result<bool, String> {
        let source_delta = i16::try_from(arc.arg0)
            .map_err(|_| "productive trie source delta exceeds i16".to_string())?;
        match arc.decoded_opcode().map_err(str::to_string)? {
            ProductiveTrieArcOpcodeV1::CopySourceRange => {
                let Some(start) =
                    resolve_source_offset(source.len(), decode_anchor(arc.anchor)?, source_delta)
                else {
                    return Ok(false);
                };
                let end = start
                    .checked_add(arc.arg1 as usize)
                    .ok_or_else(|| "productive copy range overflow".to_string())?;
                if start != frame.source_cursor || end > source.len() {
                    return Ok(false);
                }
                for scalar in &source[start..end] {
                    emit_scalar(frame, trace_arena, *scalar).map_err(str::to_string)?;
                }
                frame.source_cursor = end;
            }
            ProductiveTrieArcOpcodeV1::CopyToRetainedEdge => {
                let retained_end_delta = i16::from_le_bytes((arc.arg1 as u16).to_le_bytes());
                let Some(start) =
                    resolve_source_offset(source.len(), decode_anchor(arc.anchor)?, source_delta)
                else {
                    return Ok(false);
                };
                let Some(end) = source
                    .len()
                    .checked_add_signed(isize::from(retained_end_delta))
                else {
                    return Ok(false);
                };
                if start != frame.source_cursor || end <= start || end > source.len() {
                    return Ok(false);
                }
                for scalar in &source[start..end] {
                    emit_scalar(frame, trace_arena, *scalar).map_err(str::to_string)?;
                }
                frame.source_cursor = end;
            }
            ProductiveTrieArcOpcodeV1::DropSourcePrefix => {
                if frame.source_cursor != 0 || arc.arg1 as usize > source.len() {
                    return Ok(false);
                }
                frame.source_cursor = arc.arg1 as usize;
            }
            ProductiveTrieArcOpcodeV1::DropSourceSuffix => {
                if frame.source_cursor.checked_add(arc.arg1 as usize) != Some(source.len()) {
                    return Ok(false);
                }
                frame.source_cursor = source.len();
            }
            ProductiveTrieArcOpcodeV1::EmitSegment => {
                for scalar in self.package.segment(arc.arg1)?.chars() {
                    emit_scalar(frame, trace_arena, scalar).map_err(str::to_string)?;
                }
            }
            ProductiveTrieArcOpcodeV1::ReplaceSourceStart => {
                let Some(start) = source.len().checked_add_signed(isize::from(source_delta)) else {
                    return Ok(false);
                };
                let end = start
                    .checked_add(arc.arg1 as usize)
                    .ok_or_else(|| "productive replacement range overflow".to_string())?;
                if start != frame.source_cursor || end > source.len() {
                    return Ok(false);
                }
                frame.source_cursor = end;
            }
            ProductiveTrieArcOpcodeV1::EmitExactAllomorph => {
                return Err("lemma-local exact allomorph leaked into packaged trie".to_string());
            }
        }
        Ok(true)
    }

    fn prepare_slot_evaluations(
        &self,
        binding: &ActiveBindingV1,
        paradigm: ParadigmCenterRecordV1,
        selected_slots: &[SlotPhaseProfileRecordV1],
        wave: &SceneWaveV1,
        scene_key: u32,
        mode: ProductiveEvaluationModeV1,
    ) -> Result<Vec<PreparedSlotEvaluationV1>, String> {
        selected_slots
            .iter()
            .map(|profile| {
                let profile = *profile;
                let invariant_features = extract_feature_vector(self.feature_input(
                    binding,
                    paradigm,
                    profile,
                    GeometryTerminalEvidenceV1::default(),
                    wave,
                    scene_key,
                    mode,
                )?)
                .map_err(str::to_string)?
                .quantize()
                .map_err(str::to_string)?;
                Ok(PreparedSlotEvaluationV1 {
                    profile,
                    invariant_features,
                    ambiguity_center_cosine: match mode {
                        ProductiveEvaluationModeV1::ContextShaped => self
                            .maximum_phase_coherence(
                                ProductiveSectionKindV1::AmbiguityPhaseCenters,
                                profile.ambiguity_start,
                                profile.ambiguity_count,
                                wave,
                            )?
                            .unwrap_or_default(),
                        ProductiveEvaluationModeV1::ContextNeutralMaterial => 0,
                    },
                    minimum_independent_support: minimum_nonzero_support([
                        binding.positive_support(),
                        paradigm.support,
                        profile.support,
                    ]),
                })
            })
            .collect()
    }

    fn feature_input(
        &self,
        binding: &ActiveBindingV1,
        paradigm: ParadigmCenterRecordV1,
        profile: SlotPhaseProfileRecordV1,
        geometry: GeometryTerminalEvidenceV1,
        wave: &SceneWaveV1,
        scene_key: u32,
        mode: ProductiveEvaluationModeV1,
    ) -> Result<TerminalFeatureInputV1, String> {
        let directional = match mode {
            ProductiveEvaluationModeV1::ContextShaped => {
                self.directional_residual(scene_key, binding.lemma.source_slot_id, profile.slot_id)?
            }
            ProductiveEvaluationModeV1::ContextNeutralMaterial => None,
        };
        let stability = match (binding.stability(), paradigm.stability) {
            (binding, paradigm) if binding != 0 && paradigm != 0 => Some(binding.min(paradigm)),
            _ => None,
        };
        Ok(TerminalFeatureInputV1 {
            lemma: self.count_evidence(
                0,
                binding.positive_support(),
                binding.explicit_anti_support(),
            ),
            paradigm: self.count_evidence(1, paradigm.support, 0),
            slot: self.count_evidence(2, profile.support, profile.explicit_anti_support),
            directional: directional.and_then(|record| {
                self.count_evidence(3, record.positive_support, record.explicit_anti_support)
            }),
            positive_center_cosine: match mode {
                ProductiveEvaluationModeV1::ContextShaped => self.maximum_phase_coherence(
                    ProductiveSectionKindV1::PositivePhaseCenters,
                    profile.positive_start,
                    profile.positive_count,
                    wave,
                )?,
                ProductiveEvaluationModeV1::ContextNeutralMaterial => None,
            },
            anti_center_cosine: match mode {
                ProductiveEvaluationModeV1::ContextShaped => self.maximum_phase_coherence(
                    ProductiveSectionKindV1::AntiPhaseCenters,
                    profile.anti_start,
                    profile.anti_count,
                    wave,
                )?,
                ProductiveEvaluationModeV1::ContextNeutralMaterial => None,
            },
            hard_negative_center_cosine: match mode {
                ProductiveEvaluationModeV1::ContextShaped => self.maximum_phase_coherence(
                    ProductiveSectionKindV1::HardNegativePhaseCenters,
                    profile.hard_negative_start,
                    profile.hard_negative_count,
                    wave,
                )?,
                ProductiveEvaluationModeV1::ContextNeutralMaterial => None,
            },
            geometry,
            support: (profile.support != 0).then_some(profile.support),
            stability,
        })
    }

    fn count_evidence(
        &self,
        channel: usize,
        positive: u32,
        contradiction: u32,
    ) -> Option<CountEvidenceV1> {
        if positive == 0 && contradiction == 0 {
            return None;
        }
        let prior = self.priors[channel];
        Some(CountEvidenceV1 {
            positive,
            contradiction,
            train_positive_prior: prior.positive,
            train_contradiction_prior: prior.contradiction,
        })
    }

    fn maximum_phase_coherence(
        &self,
        section: ProductiveSectionKindV1,
        start: u32,
        count: u16,
        wave: &SceneWaveV1,
    ) -> Result<Option<i64>, String> {
        if count == 0 {
            return Ok(None);
        }
        let mut maximum = i64::MIN;
        for offset in 0..usize::from(count) {
            let center = self
                .package
                .record::<PhaseCenterRecordV1>(section, start as usize + offset)?;
            maximum = maximum.max(integer_cosine(wave, &SceneWaveV1(center.cells)));
        }
        Ok(Some(maximum))
    }

    fn directional_residual(
        &self,
        scene_key: u32,
        from_slot_id: u32,
        to_slot_id: u32,
    ) -> Result<Option<DirectionalResidualRecordV1>, String> {
        let count = self
            .package
            .record_count(ProductiveSectionKindV1::DirectionalResiduals);
        let target = (scene_key, from_slot_id, to_slot_id);
        let mut low = 0_usize;
        let mut high = count;
        while low < high {
            let middle = low + (high - low) / 2;
            let row = self.package.record::<DirectionalResidualRecordV1>(
                ProductiveSectionKindV1::DirectionalResiduals,
                middle,
            )?;
            if (row.source_scene_key, row.from_slot_id, row.to_slot_id) < target {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        if low == count {
            return Ok(None);
        }
        let row = self.package.record::<DirectionalResidualRecordV1>(
            ProductiveSectionKindV1::DirectionalResiduals,
            low,
        )?;
        Ok(((row.source_scene_key, row.from_slot_id, row.to_slot_id) == target).then_some(row))
    }

    fn calibration_cell(
        &self,
        stratum: &ObservableCalibrationStratumV1,
        fallback_row: Option<u16>,
    ) -> Result<Option<(u32, CalibrationCellRecordV1)>, String> {
        for key in stratum.packaged_backoff_key_ids().map_err(str::to_string)? {
            if let Some((row_id, cell)) = self.calibration_cell_by_key(key)? {
                if cell.support >= 200 {
                    return Ok(Some((row_id, cell)));
                }
            }
        }
        fallback_row
            .filter(|row| *row != 0)
            .map(|row| {
                self.package
                    .record::<CalibrationCellRecordV1>(
                        ProductiveSectionKindV1::CalibrationCells,
                        row as usize - 1,
                    )
                    .map(|cell| (u32::from(row), cell))
            })
            .transpose()
    }

    fn calibration_cell_by_key(
        &self,
        key: u32,
    ) -> Result<Option<(u32, CalibrationCellRecordV1)>, String> {
        let count = self
            .package
            .record_count(ProductiveSectionKindV1::CalibrationCells);
        let mut low = 0_usize;
        let mut high = count;
        while low < high {
            let middle = low + (high - low) / 2;
            let row = self.package.record::<CalibrationCellRecordV1>(
                ProductiveSectionKindV1::CalibrationCells,
                middle,
            )?;
            if row.stratum_key_id < key {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        if low == count {
            return Ok(None);
        }
        let row = self
            .package
            .record::<CalibrationCellRecordV1>(ProductiveSectionKindV1::CalibrationCells, low)?;
        Ok((row.stratum_key_id == key).then_some((low as u32 + 1, row)))
    }
}

fn decode_prepared_section<T: FixedRecordV1>(
    package: &ProductivePackageViewV1,
    section: ProductiveSectionKindV1,
) -> Result<Box<[T]>, String> {
    (0..package.record_count(section))
        .map(|index| package.record::<T>(section, index))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn decode_anchor(value: u8) -> Result<SourceAnchorV1, String> {
    match value {
        1 => Ok(SourceAnchorV1::Start),
        2 => Ok(SourceAnchorV1::End),
        _ => Err("productive trie arc has an invalid source anchor".to_string()),
    }
}

fn minimum_nonzero_support(values: [u32; 3]) -> u32 {
    values
        .into_iter()
        .filter(|value| *value != 0)
        .min()
        .unwrap_or(1)
}

fn retain_ranked_candidate(heap: &mut Vec<RankedCandidateV1>, candidate: RankedCandidateV1) {
    if let Some(existing) = heap
        .iter_mut()
        .find(|existing| existing.output.identity == candidate.output.identity)
    {
        if ranked_candidate_order(&candidate, existing).is_lt() {
            *existing = candidate;
        }
        return;
    }
    if heap.len() < PRODUCTIVE_HEAP_WITH_OVERFLOW_SENTINEL {
        heap.push(candidate);
        return;
    }
    let worst_index = heap
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| ranked_candidate_order(left, right))
        .map(|(index, _)| index)
        .expect("non-empty bounded productive frontier");
    if ranked_candidate_order(&candidate, &heap[worst_index]).is_lt() {
        heap[worst_index] = candidate;
    }
}

fn retain_surface_basin(
    basins: &mut BTreeMap<SurfaceBasinKeyV1, SurfaceBasinV1>,
    candidate: RankedCandidateV1,
    coalesce_slots: bool,
) {
    let key = SurfaceBasinKeyV1::from_candidate(&candidate, coalesce_slots);
    match basins.entry(key) {
        std::collections::btree_map::Entry::Occupied(mut basin) => {
            basin.get_mut().merge(candidate);
        }
        std::collections::btree_map::Entry::Vacant(basin) => {
            basin.insert(SurfaceBasinV1::new(candidate));
        }
    }
}

fn absorb_shared_recovered_identities(
    base_basins: &mut BTreeMap<SurfaceBasinKeyV1, SurfaceBasinV1>,
    recovered_basins: &mut BTreeMap<SurfaceBasinKeyV1, SurfaceBasinV1>,
) -> Result<(), String> {
    let shared_surface_keys = recovered_basins
        .keys()
        .filter(|key| base_basins.contains_key(*key))
        .cloned()
        .collect::<Vec<_>>();
    for key in shared_surface_keys {
        let recovered = recovered_basins
            .remove(&key)
            .ok_or_else(|| "productive shared recovered basin disappeared".to_string())?;
        base_basins
            .get_mut(&key)
            .ok_or_else(|| "productive shared base basin disappeared".to_string())?
            .absorb_identities_only(recovered);
    }
    Ok(())
}

pub(super) fn coalesce_surface_projection_v1(
    candidates: &[PackagedProductiveCandidateV1],
) -> Vec<PackagedProductiveCandidateV1> {
    let mut basins = BTreeMap::<SurfaceBasinKeyV1, SurfaceBasinV1>::new();
    for output in candidates.iter().cloned() {
        let exact_osa_distance = output
            .geometry
            .character_distance
            .min(output.geometry.keyboard_distance);
        retain_surface_basin(
            &mut basins,
            RankedCandidateV1 {
                output,
                exact_osa_distance,
            },
            true,
        );
    }
    let mut projection = basins
        .into_values()
        .map(|basin| basin.into_representative().output)
        .collect::<Vec<_>>();
    projection.sort_by(packaged_rank_preserving_order);
    projection
}

pub(super) fn base_surface_projection_preserved_v1(
    legacy: &[PackagedProductiveCandidateV1],
    semantic: &[PackagedProductiveCandidateV1],
) -> bool {
    let legacy_projection = coalesce_surface_projection_v1(legacy);
    let legacy_keys = legacy_projection
        .iter()
        .map(surface_node_key)
        .collect::<Vec<_>>();
    let legacy_key_set = legacy_keys.iter().cloned().collect::<BTreeSet<_>>();
    let semantic_old_keys = semantic
        .iter()
        .map(surface_node_key)
        .filter(|key| legacy_key_set.contains(key))
        .collect::<Vec<_>>();
    if semantic_old_keys != legacy_keys {
        return false;
    }

    legacy_projection.iter().all(|legacy_node| {
        let key = surface_node_key(legacy_node);
        let Some(semantic_node) = semantic
            .iter()
            .find(|candidate| surface_node_key(candidate) == key)
        else {
            return false;
        };
        let legacy_identities = legacy_node
            .equivalent_identities
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let semantic_identities = semantic_node
            .equivalent_identities
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        legacy_identities.is_subset(&semantic_identities)
            && semantic_node.identity == legacy_node.identity
            && semantic_node.normalized_surface == legacy_node.normalized_surface
            && semantic_node.score_q16 == legacy_node.score_q16
            && semantic_node.geometry == legacy_node.geometry
            && semantic_node.provenance == legacy_node.provenance
            && semantic_node.minimum_independent_support == legacy_node.minimum_independent_support
            && semantic_node.grounded_support == legacy_node.grounded_support
            && semantic_node.rank_origin == legacy_node.rank_origin
            && semantic_node.cross_lane_certified == legacy_node.cross_lane_certified
    })
}

fn surface_node_key(candidate: &PackagedProductiveCandidateV1) -> (u32, Arc<str>) {
    (
        candidate.identity.lemma_id,
        Arc::clone(&candidate.normalized_surface),
    )
}

fn ranked_candidate_order(left: &RankedCandidateV1, right: &RankedCandidateV1) -> Ordering {
    right
        .output
        .score_q16
        .cmp(&left.output.score_q16)
        .then_with(|| {
            right
                .output
                .grounded_support
                .cmp(&left.output.grounded_support)
        })
        .then_with(|| left.exact_osa_distance.cmp(&right.exact_osa_distance))
        .then_with(|| {
            left.output
                .identity
                .lemma_id
                .cmp(&right.output.identity.lemma_id)
        })
        .then_with(|| {
            left.output
                .identity
                .paradigm_id
                .cmp(&right.output.identity.paradigm_id)
        })
        .then_with(|| {
            left.output
                .identity
                .target_slot_id
                .cmp(&right.output.identity.target_slot_id)
        })
        .then_with(|| {
            left.output
                .identity
                .variant_id
                .cmp(&right.output.identity.variant_id)
        })
        .then_with(|| {
            left.output
                .normalized_surface
                .cmp(&right.output.normalized_surface)
        })
}

fn rank_preserving_candidate_order(
    left: &RankedCandidateV1,
    right: &RankedCandidateV1,
) -> Ordering {
    cross_lane_order(&left.output)
        .cmp(&cross_lane_order(&right.output))
        .then_with(|| ranked_candidate_order(left, right))
}

fn packaged_rank_preserving_order(
    left: &PackagedProductiveCandidateV1,
    right: &PackagedProductiveCandidateV1,
) -> Ordering {
    cross_lane_order(left)
        .cmp(&cross_lane_order(right))
        .then_with(|| right.score_q16.cmp(&left.score_q16))
        .then_with(|| right.grounded_support.cmp(&left.grounded_support))
        .then_with(|| {
            left.geometry
                .character_distance
                .min(left.geometry.keyboard_distance)
                .cmp(
                    &right
                        .geometry
                        .character_distance
                        .min(right.geometry.keyboard_distance),
                )
        })
        .then_with(|| left.identity.cmp(&right.identity))
        .then_with(|| left.normalized_surface.cmp(&right.normalized_surface))
}

fn cross_lane_order(candidate: &PackagedProductiveCandidateV1) -> u8 {
    match (candidate.rank_origin, candidate.cross_lane_certified) {
        (CandidateRankOriginV1::BaseV64, _) | (CandidateRankOriginV1::RecoveredV66, true) => 0,
        (CandidateRankOriginV1::RecoveredV66, false) => 1,
    }
}

fn has_cross_lane_certificate(
    exact_certified: bool,
    shared_hypothesis: bool,
    independent_source_count: usize,
) -> bool {
    exact_certified && shared_hypothesis && independent_source_count != 0
}

pub(super) fn packaged_ambiguity_kind(
    candidates: &[PackagedProductiveCandidateV1],
    logical_count: u64,
) -> u8 {
    let mut kind = 0_u8;
    if logical_count > super::runtime::PRODUCTIVE_PHYSICAL_TOP_K as u64 {
        kind |= AMBIGUITY_GENERATED_OVERFLOW;
    }
    if candidates
        .iter()
        .any(|candidate| candidate.ambiguity_center_cosine > 0)
    {
        kind |= AMBIGUITY_SAME_LEMMA_MULTI_LABEL;
    }
    if candidates.iter().any(|candidate| {
        candidate
            .equivalent_identities
            .iter()
            .map(|identity| identity.target_slot_id)
            .collect::<BTreeSet<_>>()
            .len()
            > 1
    }) {
        kind |= AMBIGUITY_SAME_LEMMA_MULTI_LABEL;
    }
    for (index, left) in candidates.iter().enumerate() {
        for right in candidates.iter().skip(index + 1) {
            if left.identity.lemma_id == right.identity.lemma_id {
                kind |= AMBIGUITY_SAME_LEMMA_MULTI_LABEL;
                if left.normalized_surface == right.normalized_surface
                    && left.identity.target_slot_id != right.identity.target_slot_id
                {
                    kind |= AMBIGUITY_SYNCRHETIC_SLOT;
                }
            } else {
                kind |= AMBIGUITY_CROSS_LEMMA_BASIN;
            }
        }
    }
    kind
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ranked(slot_id: u32, variant_id: u16, score_q16: i64) -> RankedCandidateV1 {
        RankedCandidateV1 {
            exact_osa_distance: u16::MAX.saturating_sub(slot_id as u16),
            output: PackagedProductiveCandidateV1 {
                identity: ProductiveCandidateIdentityV1 {
                    lemma_id: 1,
                    paradigm_id: 1,
                    program_id: slot_id,
                    target_slot_id: slot_id,
                    normalized_surface_id: slot_id,
                    variant_id,
                },
                equivalent_identities: Vec::new(),
                normalized_surface: format!("fixture-{slot_id}-{variant_id}").into(),
                score_q16,
                geometry: GeometryTerminalEvidenceV1::default(),
                provenance: CandidateProvenanceClassV1::ColdLemmaBinding,
                minimum_independent_support: 1,
                grounded_support: 1,
                ambiguity_center_cosine: 0,
                equivalent_identity_count: 1,
                equivalent_paradigm_count: 1,
                minimum_equivalent_support: 1,
                maximum_equivalent_support: 1,
                rank_origin: CandidateRankOriginV1::BaseV64,
                cross_lane_certified: false,
            },
        }
    }

    #[test]
    fn exposed_constraints_keep_dense_offsets_after_ordered_dedup() {
        let mut constraints = ExposedFormConstraintsV1::default();
        constraints.insert(5, "beta");
        constraints.insert(2, "alpha");
        constraints.insert(5, "alpha");
        constraints.insert(2, "alpha");

        assert_eq!(constraints.surface_count, 3);
        assert_eq!(
            constraints
                .slots
                .iter()
                .map(|slot| (slot.slot_id, slot.match_start))
                .collect::<Vec<_>>(),
            vec![(2, 0), (5, 1)]
        );
        assert_eq!(constraints.slots[0].surfaces, vec!["alpha".to_string()]);
        assert_eq!(
            constraints.slots[1].surfaces,
            vec!["alpha".to_string(), "beta".to_string()]
        );
    }

    #[test]
    fn dense_paradigm_flags_preserve_independent_sorted_memberships() {
        let mut flags = DenseParadigmFlagsV1::new(4).expect("dense flags");
        flags.mark(4, PARADIGM_FLAG_POSTING).expect("posting");
        flags.mark(2, PARADIGM_FLAG_POSTING).expect("posting");
        flags
            .mark(4, PARADIGM_FLAG_RECOVERED_ANCHOR)
            .expect("recovered anchor");
        flags.mark(2, PARADIGM_FLAG_POSTING).expect("duplicate");

        assert!(flags.contains(2, PARADIGM_FLAG_POSTING));
        assert!(!flags.contains(2, PARADIGM_FLAG_RECOVERED_ANCHOR));
        assert_eq!(
            flags.ids_with(PARADIGM_FLAG_POSTING).expect("posting ids"),
            BTreeSet::from([2, 4])
        );
        assert_eq!(
            flags
                .ids_with(PARADIGM_FLAG_RECOVERED_ANCHOR)
                .expect("recovered ids"),
            BTreeSet::from([4])
        );
        assert!(flags.mark(0, PARADIGM_FLAG_POSTING).is_err());
        assert!(flags.mark(5, PARADIGM_FLAG_POSTING).is_err());
    }

    #[test]
    fn geometry_aware_slot_birth_selects_after_terminal_scoring() {
        let mut frontier = BindingCandidateFrontierV1::new(1..=17).expect("slot frontier");
        for slot_id in 1..=17 {
            frontier
                .retain(
                    slot_id as usize - 1,
                    ranked(slot_id, 1, 2_000 - i64::from(slot_id)),
                )
                .expect("matching slot");
        }
        frontier
            .retain(16, ranked(17, 2, 4_000))
            .expect("matching slot");

        let selected = frontier.into_geometry_selected(16);
        let selected_slots = selected
            .iter()
            .map(|candidate| candidate.output.identity.target_slot_id)
            .collect::<BTreeSet<_>>();

        assert_eq!(selected_slots.len(), 16);
        assert!(selected_slots.contains(&17));
        assert!(!selected_slots.contains(&16));
        assert_eq!(
            selected
                .iter()
                .filter(|candidate| candidate.output.identity.target_slot_id == 17)
                .count(),
            2
        );
    }

    #[test]
    fn dense_slot_frontier_rejects_profile_slot_mismatch() {
        let mut frontier = BindingCandidateFrontierV1::new([3, 7]).expect("slot frontier");

        assert!(frontier.retain(0, ranked(7, 1, 1_000)).is_err());
    }

    #[test]
    fn dense_execution_lane_clears_only_classes_touched_by_the_previous_source() {
        let mut lane = DenseProgramExecutionLaneV1::new(8).expect("execution lane");
        let accepted = DenseProgramExecutionV1::Accepted {
            normalized_surface: Arc::from("форма"),
            geometry: GeometryTerminalEvidenceV1::default(),
        };
        lane.insert(2, accepted.clone()).expect("class 2");
        lane.insert(7, DenseProgramExecutionV1::Rejected)
            .expect("class 7");

        assert_eq!(lane.get(2).unwrap(), Some(&accepted));
        assert_eq!(
            lane.get(7).unwrap(),
            Some(&DenseProgramExecutionV1::Rejected)
        );
        assert_eq!(lane.get(3).unwrap(), None);

        lane.begin_source();

        assert_eq!(lane.get(2).unwrap(), None);
        assert_eq!(lane.get(7).unwrap(), None);
        assert_eq!(
            lane.value_index_by_class
                .iter()
                .filter(|index| **index != 0)
                .count(),
            0
        );
    }

    #[test]
    fn bounded_frontier_matches_eager_sort_for_every_input_permutation() {
        fn retain_reference(selected: &mut Vec<RankedCandidateV1>, candidate: RankedCandidateV1) {
            if let Some(existing) = selected
                .iter_mut()
                .find(|existing| existing.output.identity == candidate.output.identity)
            {
                if ranked_candidate_order(&candidate, existing).is_lt() {
                    *existing = candidate;
                }
                return;
            }
            selected.push(candidate);
            selected.sort_by(ranked_candidate_order);
            selected.truncate(PRODUCTIVE_HEAP_WITH_OVERFLOW_SENTINEL);
        }

        let candidates = (1..=48)
            .flat_map(|slot_id| {
                let mut better_duplicate = ranked(slot_id, 1, 10_000 - i64::from(slot_id));
                better_duplicate.output.normalized_surface = format!("surface-{slot_id}").into();
                let mut worse_duplicate = better_duplicate.clone();
                worse_duplicate.output.score_q16 -= 1_000;
                [worse_duplicate, better_duplicate]
            })
            .collect::<Vec<_>>();
        let permutations = [
            candidates.clone(),
            candidates.iter().cloned().rev().collect(),
            candidates
                .iter()
                .step_by(2)
                .chain(candidates.iter().skip(1).step_by(2))
                .cloned()
                .collect(),
        ];

        for stream in permutations {
            let mut expected = Vec::new();
            let mut actual = Vec::new();
            for candidate in stream {
                retain_reference(&mut expected, candidate.clone());
                retain_ranked_candidate(&mut actual, candidate);
            }
            actual.sort_by(ranked_candidate_order);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn surface_node_coalesces_slots_and_preserves_all_identities() {
        let mut first = ranked(3, 1, 1_000);
        first.output.normalized_surface = "общая".into();
        let mut second = ranked(3, 2, 2_000);
        second.output.identity.paradigm_id = 2;
        second.output.identity.program_id = 99;
        second.output.normalized_surface = "общая".into();
        second.output.minimum_independent_support = 4;
        second.output.ambiguity_center_cosine = 17;
        let mut syncretic = ranked(4, 1, 1_500);
        syncretic.output.identity.paradigm_id = 3;
        syncretic.output.normalized_surface = "общая".into();

        let mut basins = BTreeMap::new();
        retain_surface_basin(&mut basins, first, true);
        retain_surface_basin(&mut basins, second, true);
        retain_surface_basin(&mut basins, syncretic, true);

        assert_eq!(basins.len(), 1);
        let merged = basins
            .remove(&SurfaceBasinKeyV1 {
                lemma_id: 1,
                target_slot_id: None,
                normalized_surface: "общая".into(),
            })
            .expect("merged surface basin")
            .into_representative()
            .output;
        assert_eq!(merged.identity.paradigm_id, 2);
        assert_eq!(merged.equivalent_identity_count, 3);
        assert_eq!(merged.equivalent_identities.len(), 3);
        assert_eq!(merged.equivalent_paradigm_count, 3);
        assert_eq!(merged.minimum_equivalent_support, 1);
        assert_eq!(merged.maximum_equivalent_support, 4);
        assert_eq!(merged.ambiguity_center_cosine, 17);
    }

    #[test]
    fn shared_recovered_surface_adds_identity_without_changing_base_evidence() {
        let mut base = ranked(3, 1, 1_000);
        base.output.normalized_surface = "shared".into();
        base.output.geometry.character_distance = 2;
        let expected_base = base.output.clone();

        let mut recovered = ranked(4, 2, 100_000);
        recovered.output.normalized_surface = "shared".into();
        recovered.output.identity.paradigm_id = 9;
        recovered.output.rank_origin = CandidateRankOriginV1::RecoveredV66;
        recovered.output.cross_lane_certified = true;
        recovered.output.minimum_independent_support = 99;
        let recovered_identity = recovered.output.identity;

        let mut base_basins = BTreeMap::new();
        let mut recovered_basins = BTreeMap::new();
        retain_surface_basin(&mut base_basins, base, true);
        retain_surface_basin(&mut recovered_basins, recovered, true);
        absorb_shared_recovered_identities(&mut base_basins, &mut recovered_basins)
            .expect("shared identity merge");

        assert!(recovered_basins.is_empty());
        let merged = base_basins
            .into_values()
            .next()
            .expect("base basin")
            .into_representative()
            .output;
        assert_eq!(merged.identity, expected_base.identity);
        assert_eq!(merged.score_q16, expected_base.score_q16);
        assert_eq!(merged.geometry, expected_base.geometry);
        assert_eq!(merged.provenance, expected_base.provenance);
        assert_eq!(
            merged.minimum_independent_support,
            expected_base.minimum_independent_support
        );
        assert_eq!(merged.grounded_support, expected_base.grounded_support);
        assert_eq!(merged.rank_origin, CandidateRankOriginV1::BaseV64);
        assert!(!merged.cross_lane_certified);
        assert_eq!(
            merged.equivalent_identities,
            vec![expected_base.identity, recovered_identity]
        );
    }

    #[test]
    fn base_surface_projection_allows_new_nodes_but_rejects_old_identity_loss() {
        let mut first = ranked(1, 1, 3_000).output;
        first.normalized_surface = "общая".into();
        let mut syncretic = ranked(2, 1, 2_000).output;
        syncretic.normalized_surface = "общая".into();
        let legacy = vec![first.clone(), syncretic.clone()];

        let mut new_surface = ranked(3, 1, 1_000).output;
        new_surface.normalized_surface = "новая".into();
        let semantic = coalesce_surface_projection_v1(&[first, syncretic.clone(), new_surface]);
        assert!(base_surface_projection_preserved_v1(&legacy, &semantic));

        let mut missing = semantic;
        let shared = missing
            .iter_mut()
            .find(|candidate| candidate.normalized_surface.as_ref() == "общая")
            .expect("shared surface node");
        shared
            .equivalent_identities
            .retain(|identity| *identity != syncretic.identity);
        assert!(!base_surface_projection_preserved_v1(&legacy, &missing));
    }

    #[test]
    fn recovered_lane_cannot_demote_or_consume_a_base_lane_candidate() {
        let base = (1..=32)
            .map(|slot| ranked(slot, 1, 10_000 - i64::from(slot)))
            .collect::<Vec<_>>();
        let mut recovered = (33..=72)
            .map(|slot| {
                let mut candidate = ranked(slot, 1, 100_000 - i64::from(slot));
                candidate.output.rank_origin = CandidateRankOriginV1::RecoveredV66;
                candidate
            })
            .collect::<Vec<_>>();
        recovered.sort_by(ranked_candidate_order);
        recovered.truncate(super::super::runtime::PRODUCTIVE_PHYSICAL_TOP_K);

        let mut combined = base.clone();
        combined.extend(recovered);
        combined.sort_by(rank_preserving_candidate_order);

        assert_eq!(combined.len(), 64);
        assert_eq!(
            combined
                .iter()
                .take(32)
                .map(|candidate| candidate.output.identity)
                .collect::<Vec<_>>(),
            base.iter()
                .map(|candidate| candidate.output.identity)
                .collect::<Vec<_>>()
        );
        assert!(combined
            .iter()
            .skip(32)
            .all(|candidate| candidate.output.rank_origin == CandidateRankOriginV1::RecoveredV66));
    }

    #[test]
    fn cross_lane_certificate_requires_all_three_independent_facts() {
        assert!(has_cross_lane_certificate(true, true, 1));
        assert!(!has_cross_lane_certificate(false, true, 1));
        assert!(!has_cross_lane_certificate(true, false, 1));
        assert!(!has_cross_lane_certificate(true, true, 0));
    }

    #[test]
    fn only_certified_recovered_candidates_share_the_base_score_order() {
        let base = ranked(1, 1, 100);
        let mut recovered = ranked(2, 1, 10_000);
        recovered.output.rank_origin = CandidateRankOriginV1::RecoveredV66;

        let mut uncertified = vec![recovered.clone(), base.clone()];
        uncertified.sort_by(rank_preserving_candidate_order);
        assert_eq!(uncertified[0].output.identity, base.output.identity);

        recovered.output.cross_lane_certified = true;
        let mut certified = vec![recovered.clone(), base.clone()];
        certified.sort_by(rank_preserving_candidate_order);
        assert_eq!(certified[0].output.identity, recovered.output.identity);
        assert_eq!(certified[1].output.identity, base.output.identity);
    }
}
