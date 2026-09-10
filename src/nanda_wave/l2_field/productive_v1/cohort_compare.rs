//! Frame-bound cohort settlement and typed lexical-capability projection.
//!
//! Comparison remains observational; only a complete exact-partition winner
//! can issue the candidate-bound capability consumed by DecisionCore.

use std::collections::BTreeSet;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use sha2::{Digest, Sha256};

use super::candidate_state::{
    derive_candidate_validity_shadow, derive_original_preservation_shadow,
    CandidateValidityShadowV1, TargetNamespaceSettlementV1, WitnessFrameAssessmentV1,
};
use super::conflict_cohort::derive_conflict_cohort_shadow;
use super::live::{PreparedCanonicalTokenField, PreparedFieldMaterialScopeV1};
use super::material_frame::{
    bind_exact_frame_target, supported_phase7d_relation, validate_lease, BoundFrameTargetV1,
    ExactInputFrameV1, PreparedMaterialLeaseArenaV1,
};
use crate::correction_core::{CandidateAuthorityLaneIdentityV1, UnifiedCorrectionCandidate};
use crate::lexical_authority_frame::LexicalAuthorityFrameV1;
use crate::nanda_wave::l2_field::runtime::{L2FieldAuthority, StandaloneL2Field};
use crate::nanda_wave::lexical_grokking::{Phase7dCertificateClass, Phase7dCertificateOracle};
use crate::typing_transition::target_evidence::{
    stable_bytes_ref, AuthorityCertificateCoreV1, AuthorityCertificateV1, CandidateStateV1,
    CanonicalL1AnchorKindV1, CanonicalL1AnchorProofV1, CohortVerdictV1, EnumerationCompletenessV1,
    FrameOriginalPreservationV1, FrameOriginalPreservationVerdictV1, GroundingNamespaceV1,
    IncompletenessReasonV1, InputFrameIdentityV1, LeaseConsumerStateV1, PreparedMaterialLeaseV1,
    PreparedOriginalLexicalStatusV1, TargetRelationV1, TargetWitnessV1, VerdictMembershipV1,
    WitnessRejectionReasonV1,
};
use crate::word_reader::replace_last_text_word;

// One lease spans a single full-field correction event, including the ordinary
// L2/L3/L4 evaluations that run after settlement. Frame, owner, epoch and
// generation equality still invalidate it immediately; the wall-clock bound
// only prevents an otherwise unchanged event from being replayed indefinitely.
const FRAME_BOUND_LEXICAL_LEASE_NS: u64 = 30_000_000_000;
const SHARED_WITNESS_OPERATOR_BASE: u32 = 0x5348_0000;
const EXACT_PEAK_WITNESS_OPERATOR_BASE: u32 = 0x5631_0000;

/// Borrowed immutable owners used to replay a materialized canonical L1
/// anchor. This context neither loads packages nor owns a cache/resolver.
#[derive(Clone, Copy)]
pub(in crate::nanda_wave::l2_field) struct CanonicalL1AnchorReplayContextV1<'a> {
    canonical: &'a StandaloneL2Field,
    l11_sha256: [u8; 32],
    canonical_l2_sha256: [u8; 32],
}

impl<'a> CanonicalL1AnchorReplayContextV1<'a> {
    pub(in crate::nanda_wave::l2_field) const fn new(
        canonical: &'a StandaloneL2Field,
        l11_sha256: [u8; 32],
        canonical_l2_sha256: [u8; 32],
    ) -> Self {
        Self {
            canonical,
            l11_sha256,
            canonical_l2_sha256,
        }
    }

    fn replays(&self, exact_target: &str, proof: &CanonicalL1AnchorProofV1) -> bool {
        if !proof.has_canonical_ref()
            || proof.exact_target_bytes.as_bytes() != exact_target.as_bytes()
            || proof.l11_sha256 != self.l11_sha256
            || proof.canonical_l2_sha256 != self.canonical_l2_sha256
            || self.canonical.form_ref_for_surface(exact_target) != Some(proof.target_form_ref)
            || self
                .canonical
                .decode_form_ref(proof.target_form_ref)
                .is_none_or(|surface| surface.as_bytes() != exact_target.as_bytes())
        {
            return false;
        }
        let Some(anchor) = self
            .canonical
            .l1_lexical_anchor_for_form_ref(proof.target_form_ref)
        else {
            return false;
        };
        let replayed_kind = if anchor.is_direct() {
            CanonicalL1AnchorKindV1::Direct
        } else {
            CanonicalL1AnchorKindV1::SameLemma
        };
        proof.anchor_form_ref == anchor.anchor_form_ref()
            && proof.lemma_id == anchor.lemma_id()
            && proof.terminal_id == anchor.terminal_id()
            && proof.kind == replayed_kind
            && self
                .canonical
                .l1_terminal_for_form_ref(proof.anchor_form_ref)
                == Some(proof.terminal_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LexicalAuthorityFailureV1 {
    MissingFrame,
    MissingCoordinates,
    FrameMismatch,
    LeaseUnavailable,
    SettlementFailed,
    ProjectionCardinality,
    AuthorityConflict,
}

#[derive(Clone, Debug)]
pub(crate) struct BoundSettlementContextV1 {
    settlement: Arc<FrameBoundLexicalSettlementV1>,
    current_frame: LexicalAuthorityFrameV1,
}

impl PartialEq for BoundSettlementContextV1 {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.settlement, &other.settlement)
            && self.current_frame == other.current_frame
    }
}

impl Eq for BoundSettlementContextV1 {}

#[derive(Clone, Debug, Default)]
pub(crate) enum LexicalAuthorityEvaluationContextV1 {
    #[default]
    NotConsulted,
    SettledNoCertificate(BoundSettlementContextV1),
    Invalid(LexicalAuthorityFailureV1),
    InvalidAfterCertification(BoundSettlementContextV1, LexicalAuthorityFailureV1),
    Certified(BoundSettlementContextV1),
}

impl PartialEq for LexicalAuthorityEvaluationContextV1 {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NotConsulted, Self::NotConsulted) => true,
            (Self::Invalid(left), Self::Invalid(right)) => left == right,
            (
                Self::InvalidAfterCertification(left_context, left_failure),
                Self::InvalidAfterCertification(right_context, right_failure),
            ) => left_context == right_context && left_failure == right_failure,
            (Self::SettledNoCertificate(left), Self::SettledNoCertificate(right))
            | (Self::Certified(left), Self::Certified(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for LexicalAuthorityEvaluationContextV1 {}

impl LexicalAuthorityEvaluationContextV1 {
    #[cfg(test)]
    pub(crate) fn is_certified(&self) -> bool {
        matches!(self, Self::Certified(_))
    }

    pub(crate) fn owns_canonical_l2_field(&self) -> bool {
        matches!(
            self,
            Self::Certified(_) | Self::InvalidAfterCertification(_, _)
        )
    }

    pub(crate) fn admissions_for_event(
        &self,
        event: &crate::correction_core::TypingErrorEvent,
        candidates: &[UnifiedCorrectionCandidate],
    ) -> Vec<bool> {
        let now_monotonic_ns = monotonic_now_ns();
        candidates
            .iter()
            .map(|candidate| self.admits_candidate_at(event, candidate, now_monotonic_ns))
            .collect()
    }

    pub(crate) fn admits_candidate_at(
        &self,
        event: &crate::correction_core::TypingErrorEvent,
        candidate: &UnifiedCorrectionCandidate,
        now_monotonic_ns: u64,
    ) -> bool {
        let Self::Certified(context) = self else {
            return false;
        };
        let Some(capability) = candidate.frame_bound_lexical_capability() else {
            return false;
        };
        Arc::ptr_eq(&context.settlement, capability.settlement())
            && context.settlement.validates_candidate(
                &context.current_frame,
                event,
                candidate,
                capability,
                now_monotonic_ns,
            )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct FrameBoundLexicalCapabilityV1 {
    settlement: Arc<FrameBoundLexicalSettlementV1>,
    material_target_ref: u16,
    exact_replacement: String,
    canonical_owner: CandidateAuthorityLaneIdentityV1,
}

impl PartialEq for FrameBoundLexicalCapabilityV1 {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.settlement, &other.settlement)
            && self.material_target_ref == other.material_target_ref
            && self.exact_replacement.as_bytes() == other.exact_replacement.as_bytes()
            && self.canonical_owner == other.canonical_owner
    }
}

impl Eq for FrameBoundLexicalCapabilityV1 {}

impl FrameBoundLexicalCapabilityV1 {
    fn new(
        settlement: Arc<FrameBoundLexicalSettlementV1>,
        canonical_owner: CandidateAuthorityLaneIdentityV1,
    ) -> Option<Self> {
        Some(Self {
            material_target_ref: settlement.winner_target_ref?,
            exact_replacement: settlement.winner_replacement.clone()?,
            settlement,
            canonical_owner,
        })
    }

    pub(crate) fn same_event_binding(&self, other: &Self) -> bool {
        self == other
    }

    fn settlement(&self) -> &Arc<FrameBoundLexicalSettlementV1> {
        &self.settlement
    }

    pub(crate) fn canonical_owner_identity(&self) -> &CandidateAuthorityLaneIdentityV1 {
        &self.canonical_owner
    }
}

#[derive(Debug)]
pub(crate) struct FrameBoundLexicalSettlementV1 {
    field: Arc<PreparedCanonicalTokenField>,
    expected_frame: ExactInputFrameV1,
    expected_lexical_frame: LexicalAuthorityFrameV1,
    lease_arena: PreparedMaterialLeaseArenaV1,
    lease: PreparedMaterialLeaseV1,
    bounds: Vec<BoundFrameTargetV1>,
    states: Vec<CandidateValidityShadowV1>,
    original_preservation: Option<FrameOriginalPreservationV1>,
    cohort: super::conflict_cohort::ConflictCohortShadowV1,
    certificate: Option<AuthorityCertificateV1>,
    winner_target_ref: Option<u16>,
    winner_replacement: Option<String>,
    original_text: String,
    field_generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) struct LexicalCohortSettlementReadoutV1 {
    pub(in crate::nanda_wave::l2_field) compare: LexicalCohortCompareV1,
    pub(crate) authority_context: LexicalAuthorityEvaluationContextV1,
}

impl LexicalCohortSettlementReadoutV1 {
    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn capability_count(&self) -> usize {
        usize::from(self.authority_context.is_certified())
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn winner_replacement(&self) -> Option<&str> {
        match &self.authority_context {
            LexicalAuthorityEvaluationContextV1::Certified(context) => {
                context.settlement.winner_replacement.as_deref()
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) enum CohortCompareStatusV1 {
    MissingFrame,
    MissingCoordinates,
    FrameMismatch,
    LeaseUnavailable,
    SettlementFailed,
    Ready,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) enum LexicalVerdictObservationV1 {
    Winner(String),
    Tied(Vec<String>),
    Abstain,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) enum CohortFirstDivergenceV1 {
    CandidateRetention,
    VerdictKind,
    WinnerSurface,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) struct LexicalCohortCompareV1 {
    pub(in crate::nanda_wave::l2_field) status: CohortCompareStatusV1,
    pub(in crate::nanda_wave::l2_field) legacy: LexicalVerdictObservationV1,
    pub(in crate::nanda_wave::l2_field) cohort: LexicalVerdictObservationV1,
    pub(in crate::nanda_wave::l2_field) field_candidate_count: usize,
    pub(in crate::nanda_wave::l2_field) material_target_count: usize,
    pub(in crate::nanda_wave::l2_field) retained_field_candidate_count: usize,
    pub(in crate::nanda_wave::l2_field) grounded_l11_loss_count: usize,
    pub(in crate::nanda_wave::l2_field) unretained_field_candidate_surfaces: Vec<String>,
    pub(in crate::nanda_wave::l2_field) lost_grounded_l11_surfaces: Vec<String>,
    pub(in crate::nanda_wave::l2_field) material_scope: PreparedFieldMaterialScopeV1,
    pub(in crate::nanda_wave::l2_field) complete_for_authority: bool,
    pub(in crate::nanda_wave::l2_field) first_divergence: Option<CohortFirstDivergenceV1>,
}

impl LexicalCohortCompareV1 {
    fn unavailable(field: &PreparedCanonicalTokenField, status: CohortCompareStatusV1) -> Self {
        let membership = compare_material_membership(field);
        Self {
            status,
            legacy: observe_legacy(field.legacy_authority()),
            cohort: LexicalVerdictObservationV1::Unavailable,
            field_candidate_count: field.replacement_lattice_surfaces().len(),
            material_target_count: field.authority_material().compact().targets.len(),
            retained_field_candidate_count: membership.retained_field_candidate_count,
            grounded_l11_loss_count: membership.lost_grounded_l11_surfaces.len(),
            unretained_field_candidate_surfaces: membership.unretained_field_candidate_surfaces,
            lost_grounded_l11_surfaces: membership.lost_grounded_l11_surfaces,
            material_scope: field.material_scope(),
            complete_for_authority: false,
            first_divergence: None,
        }
    }
}

#[cfg(test)]
pub(in crate::nanda_wave::l2_field) fn compare_shared_canonical_cohort(
    field: &PreparedCanonicalTokenField,
    lexical_frame: Option<&LexicalAuthorityFrameV1>,
    field_generation: u64,
    anchor_replay: CanonicalL1AnchorReplayContextV1<'_>,
) -> LexicalCohortCompareV1 {
    let original = lexical_frame
        .map(LexicalAuthorityFrameV1::committed_tail)
        .unwrap_or(field.observed());
    settle_shared_canonical_cohort(
        Arc::new(field.clone()),
        lexical_frame,
        field_generation,
        original,
        anchor_replay,
    )
    .compare
}

pub(in crate::nanda_wave::l2_field) fn settle_shared_canonical_cohort(
    field: Arc<PreparedCanonicalTokenField>,
    lexical_frame: Option<&LexicalAuthorityFrameV1>,
    field_generation: u64,
    original_text: &str,
    anchor_replay: CanonicalL1AnchorReplayContextV1<'_>,
) -> LexicalCohortSettlementReadoutV1 {
    match try_settle_shared_canonical_cohort(
        Arc::clone(&field),
        lexical_frame,
        field_generation,
        original_text,
        anchor_replay,
    ) {
        Ok(settlement) => {
            let compare = compare_settlement(&settlement);
            let Some(current_frame) = lexical_frame.cloned() else {
                unreachable!("successful settlement always owns an exact lexical frame")
            };
            let context = BoundSettlementContextV1 {
                settlement: Arc::clone(&settlement),
                current_frame,
            };
            let authority_context = if settlement.certificate.is_some() {
                LexicalAuthorityEvaluationContextV1::Certified(context)
            } else {
                LexicalAuthorityEvaluationContextV1::SettledNoCertificate(context)
            };
            LexicalCohortSettlementReadoutV1 {
                compare,
                authority_context,
            }
        }
        Err((status, failure)) => LexicalCohortSettlementReadoutV1 {
            compare: LexicalCohortCompareV1::unavailable(&field, status),
            authority_context: LexicalAuthorityEvaluationContextV1::Invalid(failure),
        },
    }
}

pub(in crate::nanda_wave::l2_field) fn project_frame_bound_lexical_capability(
    candidates: &mut [UnifiedCorrectionCandidate],
    authority_context: &mut LexicalAuthorityEvaluationContextV1,
) {
    let current = std::mem::take(authority_context);
    let LexicalAuthorityEvaluationContextV1::Certified(context) = current else {
        *authority_context = current;
        return;
    };
    let matches = candidates
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| {
            let owner = candidate
                .canonical_l2_exact_partition_evidence()?
                .authority_lane_identity();
            context
                .settlement
                .matches_projection_candidate(candidate)
                .then_some((index, owner))
        })
        .collect::<Vec<_>>();
    if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
        eprintln!(
            "l2_frame_bound_projection_trace replacement={:?} matches={}",
            context.settlement.winner_replacement,
            matches.len(),
        );
    }
    let [(index, owner)] = matches.as_slice() else {
        *authority_context = LexicalAuthorityEvaluationContextV1::InvalidAfterCertification(
            context,
            LexicalAuthorityFailureV1::ProjectionCardinality,
        );
        return;
    };
    let Some(capability) =
        FrameBoundLexicalCapabilityV1::new(Arc::clone(&context.settlement), owner.clone())
    else {
        *authority_context = LexicalAuthorityEvaluationContextV1::InvalidAfterCertification(
            context,
            LexicalAuthorityFailureV1::SettlementFailed,
        );
        return;
    };
    candidates[*index].attach_frame_bound_lexical_capability(capability);
    if candidates[*index].has_authority_conflict() {
        *authority_context = LexicalAuthorityEvaluationContextV1::InvalidAfterCertification(
            context,
            LexicalAuthorityFailureV1::AuthorityConflict,
        );
    } else {
        *authority_context = LexicalAuthorityEvaluationContextV1::Certified(context);
    }
}

fn try_settle_shared_canonical_cohort(
    field: Arc<PreparedCanonicalTokenField>,
    lexical_frame: Option<&LexicalAuthorityFrameV1>,
    field_generation: u64,
    original_text: &str,
    anchor_replay: CanonicalL1AnchorReplayContextV1<'_>,
) -> Result<Arc<FrameBoundLexicalSettlementV1>, (CohortCompareStatusV1, LexicalAuthorityFailureV1)>
{
    let Some(lexical_frame) = lexical_frame else {
        return Err((
            CohortCompareStatusV1::MissingFrame,
            LexicalAuthorityFailureV1::MissingFrame,
        ));
    };
    let Some(coordinates) = lexical_frame.coordinates() else {
        return Err((
            CohortCompareStatusV1::MissingCoordinates,
            LexicalAuthorityFailureV1::MissingCoordinates,
        ));
    };
    if coordinates.source_window().as_bytes() != field.observed().as_bytes()
        || coordinates.source_window().as_bytes() != lexical_frame.observed_token().as_bytes()
        || coordinates.left_context().as_bytes() != lexical_frame.context_prefix().as_bytes()
        || field_generation == 0
    {
        return Err((
            CohortCompareStatusV1::FrameMismatch,
            LexicalAuthorityFailureV1::FrameMismatch,
        ));
    }
    let material = field.authority_material();
    let frame = exact_input_frame(lexical_frame, material, field_generation).map_err(|_| {
        (
            CohortCompareStatusV1::FrameMismatch,
            LexicalAuthorityFailureV1::FrameMismatch,
        )
    })?;
    let now = monotonic_now_ns();
    let mut arena = PreparedMaterialLeaseArenaV1::default();
    let Some(lease) = arena.pin(
        material,
        field_generation,
        coordinates.runtime_owner_lease_identity(),
        coordinates.monotonic_epoch_identity(),
        now.saturating_add(FRAME_BOUND_LEXICAL_LEASE_NS),
        LeaseConsumerStateV1::FrameSettlement,
    ) else {
        return Err((
            CohortCompareStatusV1::LeaseUnavailable,
            LexicalAuthorityFailureV1::LeaseUnavailable,
        ));
    };
    let relation_partition_complete = material.validates_relation_partition_proof();
    if relation_partition_complete
        && material.compact().original.lexical_status == PreparedOriginalLexicalStatusV1::Clean
        && material
            .original_canonical_l1_anchor_proof()
            .is_none_or(|proof| !anchor_replay.replays(material.exact_observed(), proof))
    {
        return Err((
            CohortCompareStatusV1::SettlementFailed,
            LexicalAuthorityFailureV1::SettlementFailed,
        ));
    }
    let original_preservation = derive_original_preservation_shadow(
        material, lease, &frame, &frame, now,
    )
    .map_err(|_| {
        (
            CohortCompareStatusV1::SettlementFailed,
            LexicalAuthorityFailureV1::SettlementFailed,
        )
    })?;
    let source_scalars = coordinates.source_window().chars().count();
    let mut bounds = Vec::<BoundFrameTargetV1>::with_capacity(material.compact().targets.len());
    let mut states =
        Vec::<CandidateValidityShadowV1>::with_capacity(material.compact().targets.len());
    for target_ref in 0..material.compact().targets.len() {
        let bound = match bind_exact_frame_target(
            material,
            lease,
            &frame,
            &frame,
            target_ref,
            0,
            source_scalars,
            1,
            0,
            now,
        ) {
            Ok(bound) => bound,
            Err(_) => {
                return Err((
                    CohortCompareStatusV1::SettlementFailed,
                    LexicalAuthorityFailureV1::SettlementFailed,
                ))
            }
        };
        let span = bound.identity.replacement_span;
        if span.scalar_start != 0
            || usize::try_from(span.scalar_len).ok() != Some(source_scalars)
            || usize::try_from(span.source_scalar_len).ok() != Some(source_scalars)
            || bound.replayed_source_window.as_bytes() != bound.projected_target.as_bytes()
        {
            return Err((
                CohortCompareStatusV1::SettlementFailed,
                LexicalAuthorityFailureV1::SettlementFailed,
            ));
        }
        let target = &material.compact().targets.as_slice()[target_ref];
        let assessments = target
            .witnesses
            .witnesses()
            .iter()
            .enumerate()
            .map(|(index, witness)| {
                assess_exact_witness(
                    material,
                    anchor_replay,
                    target_ref,
                    index,
                    coordinates.source_window(),
                    &bound.projected_target,
                    *witness,
                )
            })
            .collect::<Vec<_>>();
        let namespace = if material.completeness().state()
            == crate::typing_transition::target_evidence::EnumerationStateV1::Complete
            && target.witnesses.state()
                == crate::typing_transition::target_evidence::EnumerationStateV1::Complete
            && assessments.len() == target.witnesses.witnesses().len()
        {
            TargetNamespaceSettlementV1::CompleteExactGrounding
        } else {
            let reason = if material.completeness().state()
                != crate::typing_transition::target_evidence::EnumerationStateV1::Complete
            {
                material.completeness().reason()
            } else if target.witnesses.state()
                != crate::typing_transition::target_evidence::EnumerationStateV1::Complete
            {
                target.witnesses.reason()
            } else {
                IncompletenessReasonV1::IntegrityFailure
            };
            TargetNamespaceSettlementV1::Incomplete(reason)
        };
        let state = match derive_candidate_validity_shadow(
            material,
            lease,
            &frame,
            &frame,
            &bound,
            namespace,
            &assessments,
            now,
        ) {
            Ok(state) => state,
            Err(_) => {
                return Err((
                    CohortCompareStatusV1::SettlementFailed,
                    LexicalAuthorityFailureV1::SettlementFailed,
                ))
            }
        };
        bounds.push(bound);
        states.push(state);
    }
    let members = bounds.iter().zip(states.iter()).collect::<Vec<_>>();
    let cohort = match derive_conflict_cohort_shadow(
        material,
        lease,
        &frame,
        &members,
        original_preservation,
        now,
    ) {
        Ok(cohort) => cohort,
        Err(_) => {
            return Err((
                CohortCompareStatusV1::SettlementFailed,
                LexicalAuthorityFailureV1::SettlementFailed,
            ))
        }
    };
    if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
        let mut grounded = Vec::new();
        let mut born_count = 0_usize;
        let mut rejected_count = 0_usize;
        for (target_ref, state) in states.iter().enumerate() {
            let surface = material
                .exact_target_surface(target_ref)
                .unwrap_or("<missing>");
            match state.state {
                CandidateStateV1::Grounded => grounded.push(surface),
                CandidateStateV1::Born => born_count += 1,
                CandidateStateV1::Rejected(_) => rejected_count += 1,
            }
        }
        eprintln!(
            "l2_frame_bound_settlement_trace completeness={:?} original={:?} grounded={grounded:?} born={} rejected={} blockers={} cohort={:?}",
            material.completeness(),
            original_preservation.map(|value| value.verdict),
            born_count,
            rejected_count,
            states
                .iter()
                .filter(|state| !state.authority_blockers.is_empty())
                .count(),
            cohort.verdict,
        );
    }
    let winner_target_ref = match &cohort.verdict {
        CohortVerdictV1::Winner(target_ref)
            if cohort.complete_for_authority
                && relation_partition_complete
                && field.material_scope() == PreparedFieldMaterialScopeV1::ContextNeutral =>
        {
            Some(*target_ref)
        }
        _ => None,
    };
    let winner_replacement = winner_target_ref.and_then(|target_ref| {
        let bound = bounds.get(usize::from(target_ref))?;
        replace_last_text_word(original_text, &bound.projected_target)
            .filter(|replacement| replacement.as_bytes() != original_text.as_bytes())
    });
    let certificate =
        winner_target_ref
            .zip(winner_replacement.as_ref())
            .and_then(|(target_ref, _)| {
                issue_l2_certificate(
                    material,
                    lease,
                    &frame,
                    &bounds,
                    &states,
                    original_preservation,
                    &cohort,
                    target_ref,
                )
            });
    Ok(Arc::new(FrameBoundLexicalSettlementV1 {
        field,
        expected_frame: frame,
        expected_lexical_frame: lexical_frame.clone(),
        lease_arena: arena,
        lease,
        bounds,
        states,
        original_preservation,
        cohort,
        certificate,
        winner_target_ref,
        winner_replacement,
        original_text: original_text.to_string(),
        field_generation,
    }))
}

fn compare_settlement(settlement: &FrameBoundLexicalSettlementV1) -> LexicalCohortCompareV1 {
    let field = &settlement.field;
    let material = field.authority_material();
    let membership = compare_material_membership(field);
    let field_surfaces = field.replacement_lattice_surfaces();
    let legacy = observe_legacy(field.legacy_authority());
    let observed_cohort = observe_cohort(material, settlement.cohort.verdict.clone());
    let first_divergence = if membership.retained_field_candidate_count != field_surfaces.len() {
        Some(CohortFirstDivergenceV1::CandidateRetention)
    } else if verdict_kind(&legacy) != verdict_kind(&observed_cohort) {
        Some(CohortFirstDivergenceV1::VerdictKind)
    } else if matches!(
        (&legacy, &observed_cohort),
        (LexicalVerdictObservationV1::Winner(left), LexicalVerdictObservationV1::Winner(right))
            if !left.eq_ignore_ascii_case(right)
    ) {
        Some(CohortFirstDivergenceV1::WinnerSurface)
    } else {
        None
    };
    LexicalCohortCompareV1 {
        status: CohortCompareStatusV1::Ready,
        legacy,
        cohort: observed_cohort,
        field_candidate_count: field_surfaces.len(),
        material_target_count: material.compact().targets.len(),
        retained_field_candidate_count: membership.retained_field_candidate_count,
        grounded_l11_loss_count: membership.lost_grounded_l11_surfaces.len(),
        unretained_field_candidate_surfaces: membership.unretained_field_candidate_surfaces,
        lost_grounded_l11_surfaces: membership.lost_grounded_l11_surfaces,
        material_scope: field.material_scope(),
        complete_for_authority: settlement.cohort.complete_for_authority
            && material.validates_relation_partition_proof()
            && field.material_scope() == PreparedFieldMaterialScopeV1::ContextNeutral,
        first_divergence,
    }
}

impl FrameBoundLexicalSettlementV1 {
    fn matches_projection_candidate(&self, candidate: &UnifiedCorrectionCandidate) -> bool {
        let Some(target_ref) = self.winner_target_ref.map(usize::from) else {
            return false;
        };
        let Some(target) = self
            .field
            .authority_material()
            .exact_target_surface(target_ref)
        else {
            return false;
        };
        let Some(projected) = replace_last_text_word(&self.original_text, target) else {
            return false;
        };
        candidate.belongs_to_canonical_l2_exact_partition()
            && candidate.replacement.as_bytes() == projected.as_bytes()
            && self
                .winner_replacement
                .as_deref()
                .is_some_and(|winner| winner.as_bytes() == projected.as_bytes())
    }

    fn validates_candidate(
        &self,
        current_lexical_frame: &LexicalAuthorityFrameV1,
        event: &crate::correction_core::TypingErrorEvent,
        candidate: &UnifiedCorrectionCandidate,
        capability: &FrameBoundLexicalCapabilityV1,
        now_monotonic_ns: u64,
    ) -> bool {
        if current_lexical_frame != &self.expected_lexical_frame
            || event.original.as_bytes() != self.original_text.as_bytes()
            || !self.matches_projection_candidate(candidate)
            || capability.material_target_ref != self.winner_target_ref.unwrap_or(u16::MAX)
            || capability.exact_replacement.as_bytes() != candidate.replacement.as_bytes()
            || !candidate.contains_authority_lane(&capability.canonical_owner)
            || candidate.has_authority_conflict()
            || self.field.material_scope() != PreparedFieldMaterialScopeV1::ContextNeutral
            || !super::super::cache::generation_is_current(self.field_generation)
            || !self.lease_arena.contains(self.lease)
        {
            return false;
        }
        let Some(coordinates) = current_lexical_frame.coordinates() else {
            return false;
        };
        if coordinates.runtime_owner_lease_identity() != self.lease.runtime_owner_lease_identity
            || coordinates.monotonic_epoch_identity() != self.lease.monotonic_epoch_identity
        {
            return false;
        }
        let material = self.field.authority_material();
        let Ok(current_frame) =
            exact_input_frame(current_lexical_frame, material, self.field_generation)
        else {
            return false;
        };
        if self.expected_frame.compare_exact(&current_frame).is_err()
            || validate_lease(material, self.lease, &current_frame, now_monotonic_ns).is_err()
        {
            return false;
        }
        self.certificate
            == self.winner_target_ref.and_then(|target_ref| {
                issue_l2_certificate(
                    material,
                    self.lease,
                    &current_frame,
                    &self.bounds,
                    &self.states,
                    self.original_preservation,
                    &self.cohort,
                    target_ref,
                )
            })
    }
}

fn exact_input_frame(
    lexical_frame: &LexicalAuthorityFrameV1,
    material: &super::material_frame::PreparedTargetMaterialShadowV1,
    field_generation: u64,
) -> Result<ExactInputFrameV1, crate::typing_transition::target_evidence::FrameInvalidationReasonV1>
{
    let coordinates = lexical_frame.coordinates().ok_or(
        crate::typing_transition::target_evidence::FrameInvalidationReasonV1::SourceWindow,
    )?;
    ExactInputFrameV1::new(
        coordinates.focus_serial(),
        lexical_frame.tail_epoch(),
        coordinates.source_window().to_string(),
        coordinates.left_context().to_string(),
        coordinates.caret_scalar(),
        coordinates.selection(),
        coordinates.preedit().to_string(),
        coordinates.preedit_cursor_scalar(),
        coordinates.layout_generation(),
        coordinates.config_generation(),
        material.compact().key.package_generation,
        field_generation,
    )
}

#[derive(Clone, Debug)]
struct ExactTypedGeometryV1 {
    relation: TargetRelationV1,
    class: Phase7dCertificateClass,
    canonical_key: String,
    authority_supported: bool,
}

fn exact_typed_geometries(source: &str, target: &str) -> Result<Vec<ExactTypedGeometryV1>, String> {
    let evidence = Phase7dCertificateOracle::new(source)?.certificate_evidence(target)?;
    let mut geometries = evidence
        .into_iter()
        .map(|evidence| ExactTypedGeometryV1 {
            relation: phase7d_material_relation(evidence.class),
            authority_supported: supported_phase7d_relation(&evidence).is_some(),
            class: evidence.class,
            canonical_key: evidence.canonical_key,
        })
        .collect::<Vec<_>>();
    geometries.sort_by(|left, right| {
        (left.relation, left.canonical_key.as_str())
            .cmp(&(right.relation, right.canonical_key.as_str()))
    });
    geometries.dedup_by(|left, right| {
        left.relation == right.relation && left.canonical_key == right.canonical_key
    });
    Ok(geometries)
}

const fn phase7d_material_relation(class: Phase7dCertificateClass) -> TargetRelationV1 {
    match class {
        Phase7dCertificateClass::Identity | Phase7dCertificateClass::PunctuationSuffix => {
            TargetRelationV1::L11Restoration
        }
        Phase7dCertificateClass::PrefixTruncation
        | Phase7dCertificateClass::SuffixTruncation
        | Phase7dCertificateClass::MissingLetter => TargetRelationV1::MissingLetter,
        Phase7dCertificateClass::ExtraLetter => TargetRelationV1::ExtraLetter,
        Phase7dCertificateClass::Substitution => TargetRelationV1::Substitution,
        Phase7dCertificateClass::KeyboardLayout => TargetRelationV1::ExactLayout,
        Phase7dCertificateClass::AdjacentTransposition => TargetRelationV1::AdjacentTransposition,
        Phase7dCertificateClass::NonAdjacentTransposition => {
            TargetRelationV1::NonAdjacentTransposition
        }
        Phase7dCertificateClass::RepeatedFragment => TargetRelationV1::RepeatedFragment,
        Phase7dCertificateClass::SparseMultiOmission
        | Phase7dCertificateClass::OmissionTransposition => TargetRelationV1::SparseOmission,
    }
}

fn assess_exact_witness(
    material: &super::material_frame::PreparedTargetMaterialShadowV1,
    anchor_replay: CanonicalL1AnchorReplayContextV1<'_>,
    target_ref: usize,
    index: usize,
    source: &str,
    target: &str,
    witness: TargetWitnessV1,
) -> WitnessFrameAssessmentV1 {
    let mut assessment = WitnessFrameAssessmentV1 {
        material_witness_ref: index.min(usize::from(u8::MAX)) as u8,
        valid_geometry: false,
        rejection: Some(WitnessRejectionReasonV1::GeometryReplayMismatch),
    };
    if !material.exact_witness_root_matches(target_ref, index, witness) {
        assessment.rejection = Some(WitnessRejectionReasonV1::MalformedEvidenceRoot);
        return assessment;
    }
    if witness.relation == TargetRelationV1::Unsupported
        || witness.grounding_namespace == GroundingNamespaceV1::None
    {
        return assessment;
    }
    let Ok(geometries) = exact_typed_geometries(source, target) else {
        assessment.rejection = Some(WitnessRejectionReasonV1::MalformedEvidenceRoot);
        return assessment;
    };
    let shared_grounded_root = matches!(
        (witness.grounding_namespace, witness.verdict_membership),
        (
            GroundingNamespaceV1::CanonicalL1Anchor,
            VerdictMembershipV1::Grounded
                | VerdictMembershipV1::L11Winner
                | VerdictMembershipV1::L11Tied
        )
    ) && matches!(
        witness.relation,
        TargetRelationV1::ExactLayout
            | TargetRelationV1::LayoutThenTypo
            | TargetRelationV1::L11Restoration
    );
    if shared_grounded_root {
        let Some(anchor_proof) =
            material.canonical_l1_anchor_proof_for_target_witness(target_ref, index, witness)
        else {
            assessment.rejection = Some(WitnessRejectionReasonV1::MalformedEvidenceRoot);
            return assessment;
        };
        if witness.operator_ref == shared_witness_operator_ref(witness.relation)
            && witness.derivation_ref == shared_witness_derivation_ref(source, source, target)
            && geometries
                .iter()
                .any(|geometry| geometry.authority_supported)
            && anchor_replay.replays(target, anchor_proof)
        {
            assessment.valid_geometry = true;
            assessment.rejection = None;
        } else {
            assessment.rejection = Some(WitnessRejectionReasonV1::MalformedEvidenceRoot);
        }
        return assessment;
    }
    let exact_peak_root = witness.grounding_namespace == GroundingNamespaceV1::CanonicalForm
        && witness.verdict_membership == VerdictMembershipV1::Born;
    if exact_peak_root {
        let matching = geometries
            .iter()
            .filter(|geometry| {
                geometry.relation == witness.relation
                    && witness.operator_ref
                        == (EXACT_PEAK_WITNESS_OPERATOR_BASE | u32::from(geometry.class as u8))
                    && witness.derivation_ref == stable_bytes_ref(geometry.canonical_key.as_bytes())
            })
            .collect::<Vec<_>>();
        let [geometry] = matching.as_slice() else {
            assessment.rejection = Some(WitnessRejectionReasonV1::MalformedEvidenceRoot);
            return assessment;
        };
        if geometry.authority_supported {
            assessment.valid_geometry = true;
            assessment.rejection = None;
        }
        return assessment;
    }
    assessment
}

pub(super) const fn shared_witness_operator_ref(relation: TargetRelationV1) -> u32 {
    SHARED_WITNESS_OPERATOR_BASE | relation as u32
}

pub(super) fn shared_witness_derivation_ref(
    source: &str,
    query_surface: &str,
    target: &str,
) -> u32 {
    let mut derivation_bytes = b"lay-shared-canonical-field-derivation-v1\0".to_vec();
    for value in [source, query_surface, target] {
        derivation_bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
        derivation_bytes.extend_from_slice(value.as_bytes());
    }
    stable_bytes_ref(&derivation_bytes)
}

#[expect(
    clippy::too_many_arguments,
    reason = "certificate issuance keeps every bound proof input explicit"
)]
fn issue_l2_certificate(
    material: &super::material_frame::PreparedTargetMaterialShadowV1,
    lease: PreparedMaterialLeaseV1,
    frame: &ExactInputFrameV1,
    bounds: &[BoundFrameTargetV1],
    states: &[CandidateValidityShadowV1],
    original: Option<FrameOriginalPreservationV1>,
    cohort: &super::conflict_cohort::ConflictCohortShadowV1,
    winner_target_ref: u16,
) -> Option<AuthorityCertificateV1> {
    if bounds.len() != material.compact().targets.len()
        || states.len() != material.compact().targets.len()
        || !material.validates_relation_partition_proof()
        || !cohort.complete_for_authority
        || cohort.verdict != CohortVerdictV1::Winner(winner_target_ref)
        || original.map(|value| value.verdict)
            != Some(FrameOriginalPreservationVerdictV1::ReplacePermitted)
    {
        return None;
    }
    let target_ref = usize::from(winner_target_ref);
    let bound = bounds.get(target_ref)?;
    let state = states.get(target_ref)?;
    let target = material.compact().targets.as_slice().get(target_ref)?;
    let span = bound.identity.replacement_span;
    if state.state != CandidateStateV1::Grounded
        || state.material_target_ref != winner_target_ref
        || !state.authority_blockers.is_empty()
        || span.scalar_start != 0
        || span.scalar_len != frame.identity().source_scalar_count
        || span.source_scalar_len != frame.identity().source_scalar_count
        || bound.replayed_source_window.as_bytes() != bound.projected_target.as_bytes()
        || stable_bytes_ref(bound.projected_target.as_bytes())
            != bound.identity.exact_projected_target_bytes_ref
    {
        return None;
    }
    let original = original?;
    let frame_bytes = frame_identity_bytes(frame.identity());
    let mut cohort_and_preservation = Vec::with_capacity(48);
    cohort_and_preservation.extend_from_slice(&cohort.cohort_hash[0].to_le_bytes());
    cohort_and_preservation.extend_from_slice(&cohort.cohort_hash[1].to_le_bytes());
    cohort_and_preservation
        .extend_from_slice(&original.prepared_original_material_hash[0].to_le_bytes());
    cohort_and_preservation
        .extend_from_slice(&original.prepared_original_material_hash[1].to_le_bytes());
    cohort_and_preservation.extend_from_slice(&winner_target_ref.to_le_bytes());
    let core = AuthorityCertificateCoreV1 {
        prepared_material_lease_id: lease.lease_identity,
        exact_frame_identity_ref: stable_bytes_ref(&frame_bytes),
        exact_framed_target_ref: framed_target_ref(bound),
        exact_cohort_and_preservation_ref: stable_bytes_ref(&cohort_and_preservation),
        schema_versions: 0x0001_0001,
        frame_identity_hash: bound.identity.frame_identity_hash,
        exact_projected_target_hash: state.exact_projected_target_hash,
        evidence_hash: target_evidence_hash(target.witnesses.witnesses()),
        cohort_hash: cohort.cohort_hash,
        completeness_hash: completeness_hash(material.completeness()),
        material_generation: material.compact().key.package_generation,
        frame_generation: frame.identity().field_generation,
        monotonic_epoch_identity: lease.monotonic_epoch_identity,
        expires_at_monotonic_ns: lease.expires_at_monotonic_ns,
    };
    Some(AuthorityCertificateV1::L2Certified(core))
}

fn framed_target_ref(bound: &BoundFrameTargetV1) -> u32 {
    let mut bytes = Vec::with_capacity(bound.projected_target.len() + 64);
    let span = bound.identity.replacement_span;
    bytes.extend_from_slice(&bound.identity.material_target_ref.to_le_bytes());
    bytes.extend_from_slice(&span.scalar_start.to_le_bytes());
    bytes.extend_from_slice(&span.scalar_len.to_le_bytes());
    bytes.extend_from_slice(&span.source_scalar_len.to_le_bytes());
    bytes.extend_from_slice(
        &bound
            .identity
            .exact_projected_target_bytes_ref
            .to_le_bytes(),
    );
    bytes.extend_from_slice(bound.projected_target.as_bytes());
    stable_bytes_ref(&bytes)
}

fn target_evidence_hash(witnesses: &[TargetWitnessV1]) -> [u64; 2] {
    let mut hasher = Sha256::new();
    hasher.update(b"lay-frame-bound-lexical-evidence-v1\0");
    for witness in witnesses {
        hasher.update([
            witness.relation as u8,
            witness.grounding_namespace as u8,
            witness.verdict_membership as u8,
            witness.flags,
        ]);
        for value in [
            witness.operator_ref,
            witness.grounding_ref,
            witness.derivation_ref,
            u32::from(witness.support_milli),
            u32::from(witness.provenance_annotations),
        ] {
            hasher.update(value.to_le_bytes());
        }
    }
    digest128(hasher.finalize().into())
}

fn completeness_hash(completeness: EnumerationCompletenessV1) -> [u64; 2] {
    let mut hasher = Sha256::new();
    hasher.update(b"lay-frame-bound-lexical-completeness-v1\0");
    hasher.update([completeness.state() as u8, completeness.reason() as u8]);
    hasher.update([completeness.scope().kind() as u8]);
    hasher.update(completeness.logical_count_lower_bound().to_le_bytes());
    hasher.update(completeness.retained_count().to_le_bytes());
    for value in completeness.all_seen_digest() {
        hasher.update(value.to_le_bytes());
    }
    digest128(hasher.finalize().into())
}

fn frame_identity_bytes(identity: InputFrameIdentityV1) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(96);
    for value in [identity.focus_serial, identity.tail_epoch] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in [
        identity.exact_source_window_ref,
        identity.exact_left_context_ref,
        identity.source_scalar_count,
        identity.caret_scalar,
        identity.selection_start_scalar,
        identity.selection_end_scalar,
        identity.exact_preedit_bytes_ref,
        identity.preedit_cursor_scalar,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in [
        identity.layout_generation,
        identity.config_generation,
        identity.package_generation,
        identity.field_generation,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn digest128(digest: [u8; 32]) -> [u64; 2] {
    [
        u64::from_le_bytes(digest[..8].try_into().expect("SHA-256 prefix")),
        u64::from_le_bytes(digest[8..16].try_into().expect("SHA-256 prefix")),
    ]
}

struct MaterialMembershipComparisonV1 {
    retained_field_candidate_count: usize,
    unretained_field_candidate_surfaces: Vec<String>,
    lost_grounded_l11_surfaces: Vec<String>,
}

fn compare_material_membership(
    field: &PreparedCanonicalTokenField,
) -> MaterialMembershipComparisonV1 {
    let material_surfaces = field
        .authority_material()
        .exact_target_surfaces()
        .collect::<BTreeSet<_>>();
    let field_surfaces = field.replacement_lattice_surfaces();
    let unretained_field_candidate_surfaces = field_surfaces
        .iter()
        .filter(|surface| !material_surfaces.contains(**surface))
        .map(|surface| (*surface).to_string())
        .collect::<Vec<_>>();
    let mut lost_grounded_l11_surfaces = field
        .replacement_grounded_l11_surfaces()
        .iter()
        .filter(|surface| !material_surfaces.contains(**surface))
        .map(|surface| (*surface).to_string())
        .collect::<Vec<_>>();
    if field.original_has_grounded_l11_evidence()
        && !field
            .authority_material()
            .original_has_grounded_l11_evidence()
    {
        lost_grounded_l11_surfaces.push(field.observed().to_string());
    }
    MaterialMembershipComparisonV1 {
        retained_field_candidate_count: field_surfaces
            .len()
            .saturating_sub(unretained_field_candidate_surfaces.len()),
        unretained_field_candidate_surfaces,
        lost_grounded_l11_surfaces,
    }
}

fn observe_legacy(authority: &L2FieldAuthority) -> LexicalVerdictObservationV1 {
    match authority {
        L2FieldAuthority::Winner { surface } => {
            LexicalVerdictObservationV1::Winner(surface.clone())
        }
        L2FieldAuthority::Tied { surfaces } => LexicalVerdictObservationV1::Tied(surfaces.clone()),
        L2FieldAuthority::Abstain => LexicalVerdictObservationV1::Abstain,
        L2FieldAuthority::Unavailable => LexicalVerdictObservationV1::Unavailable,
    }
}

fn observe_cohort(
    material: &super::material_frame::PreparedTargetMaterialShadowV1,
    verdict: CohortVerdictV1,
) -> LexicalVerdictObservationV1 {
    match verdict {
        CohortVerdictV1::Winner(target_ref) => material
            .exact_target_surface(usize::from(target_ref))
            .map(|surface| LexicalVerdictObservationV1::Winner(surface.to_string()))
            .unwrap_or(LexicalVerdictObservationV1::Unavailable),
        CohortVerdictV1::Tied {
            members,
            member_count,
            ..
        } => LexicalVerdictObservationV1::Tied(
            members[..usize::from(member_count)]
                .iter()
                .filter_map(|target_ref| material.exact_target_surface(usize::from(*target_ref)))
                .map(str::to_string)
                .collect(),
        ),
        CohortVerdictV1::Abstain(_) => LexicalVerdictObservationV1::Abstain,
    }
}

fn verdict_kind(verdict: &LexicalVerdictObservationV1) -> u8 {
    match verdict {
        LexicalVerdictObservationV1::Winner(_) => 0,
        LexicalVerdictObservationV1::Tied(_) => 1,
        LexicalVerdictObservationV1::Abstain => 2,
        LexicalVerdictObservationV1::Unavailable => 3,
    }
}

fn monotonic_now_ns() -> u64 {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    EPOCH
        .get_or_init(Instant::now)
        .elapsed()
        .as_nanos()
        .min(u128::from(u64::MAX)) as u64
        + 1
}

#[cfg(test)]
mod boundary_tests;

#[cfg(test)]
pub(crate) use boundary_tests::valid_pipeline as test_bound_lexical_pipeline;

#[cfg(test)]
pub(super) use boundary_tests::raw_peaks_with_unsupported_overflow as test_raw_peaks_with_unsupported_overflow;

#[cfg(test)]
mod witness_replay_tests;
