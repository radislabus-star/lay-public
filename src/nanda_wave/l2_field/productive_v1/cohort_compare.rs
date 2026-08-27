//! Observation-only comparison between the legacy field-wide readout and the
//! complete frame-bound cohort derived from the same prepared canonical field.

use std::collections::BTreeSet;
use std::sync::OnceLock;
use std::time::Instant;

use super::candidate_state::{
    derive_candidate_validity_shadow, derive_original_preservation_shadow,
    CandidateValidityShadowV1, TargetNamespaceSettlementV1, WitnessFrameAssessmentV1,
};
use super::conflict_cohort::derive_conflict_cohort_shadow;
use super::live::{PreparedCanonicalTokenField, PreparedFieldMaterialScopeV1};
use super::material_frame::{
    bind_exact_frame_target, BoundFrameTargetV1, ExactInputFrameV1, PreparedMaterialLeaseArenaV1,
};
use crate::lexical_authority_frame::LexicalAuthorityFrameV1;
use crate::nanda_wave::l2_field::runtime::L2FieldAuthority;
use crate::typing_transition::target_evidence::{
    CohortVerdictV1, GroundingNamespaceV1, IncompletenessReasonV1, LeaseConsumerStateV1,
    TargetRelationV1, VerdictMembershipV1,
};

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
            material_target_count: field.prepared_material().compact().targets.len(),
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

pub(in crate::nanda_wave::l2_field) fn compare_shared_canonical_cohort(
    field: &PreparedCanonicalTokenField,
    lexical_frame: Option<&LexicalAuthorityFrameV1>,
    field_generation: u64,
) -> LexicalCohortCompareV1 {
    let Some(lexical_frame) = lexical_frame else {
        return LexicalCohortCompareV1::unavailable(field, CohortCompareStatusV1::MissingFrame);
    };
    let Some(coordinates) = lexical_frame.coordinates() else {
        return LexicalCohortCompareV1::unavailable(
            field,
            CohortCompareStatusV1::MissingCoordinates,
        );
    };
    if coordinates.source_window().as_bytes() != field.observed().as_bytes()
        || coordinates.source_window().as_bytes() != lexical_frame.observed_token().as_bytes()
        || coordinates.left_context().as_bytes() != lexical_frame.context_prefix().as_bytes()
        || field_generation == 0
    {
        return LexicalCohortCompareV1::unavailable(field, CohortCompareStatusV1::FrameMismatch);
    }
    let material = field.prepared_material();
    let frame = match ExactInputFrameV1::new(
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
    ) {
        Ok(frame) => frame,
        Err(_) => {
            return LexicalCohortCompareV1::unavailable(field, CohortCompareStatusV1::FrameMismatch)
        }
    };
    let now = monotonic_now_ns();
    let mut arena = PreparedMaterialLeaseArenaV1::default();
    let Some(lease) = arena.pin(
        material,
        field_generation,
        coordinates.runtime_owner_lease_identity(),
        coordinates.monotonic_epoch_identity(),
        now.saturating_add(5_000_000),
        LeaseConsumerStateV1::FrameSettlement,
    ) else {
        return LexicalCohortCompareV1::unavailable(field, CohortCompareStatusV1::LeaseUnavailable);
    };
    let original = match derive_original_preservation_shadow(material, lease, &frame, &frame, now) {
        Ok(original) => original,
        Err(_) => {
            return LexicalCohortCompareV1::unavailable(
                field,
                CohortCompareStatusV1::SettlementFailed,
            )
        }
    };
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
                return LexicalCohortCompareV1::unavailable(
                    field,
                    CohortCompareStatusV1::SettlementFailed,
                )
            }
        };
        let target = &material.compact().targets.as_slice()[target_ref];
        let assessments = target
            .witnesses
            .witnesses()
            .iter()
            .enumerate()
            .map(|(index, witness)| WitnessFrameAssessmentV1 {
                material_witness_ref: index as u8,
                valid_geometry: witness.relation != TargetRelationV1::Unsupported
                    && witness.grounding_namespace != GroundingNamespaceV1::None
                    && matches!(
                        witness.verdict_membership,
                        VerdictMembershipV1::Grounded
                            | VerdictMembershipV1::L11Winner
                            | VerdictMembershipV1::L11Tied
                    ),
                rejection: None,
            })
            .collect::<Vec<_>>();
        let namespace = if assessments
            .iter()
            .any(|assessment| assessment.valid_geometry)
        {
            TargetNamespaceSettlementV1::CompleteExactGrounding
        } else {
            TargetNamespaceSettlementV1::Incomplete(IncompletenessReasonV1::UpstreamIncomplete)
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
                return LexicalCohortCompareV1::unavailable(
                    field,
                    CohortCompareStatusV1::SettlementFailed,
                )
            }
        };
        bounds.push(bound);
        states.push(state);
    }
    let members = bounds.iter().zip(states.iter()).collect::<Vec<_>>();
    let cohort =
        match derive_conflict_cohort_shadow(material, lease, &frame, &members, original, now) {
            Ok(cohort) => cohort,
            Err(_) => {
                return LexicalCohortCompareV1::unavailable(
                    field,
                    CohortCompareStatusV1::SettlementFailed,
                )
            }
        };
    let membership = compare_material_membership(field);
    let field_surfaces = field.replacement_lattice_surfaces();
    let legacy = observe_legacy(field.legacy_authority());
    let observed_cohort = observe_cohort(material, cohort.verdict);
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
        complete_for_authority: cohort.complete_for_authority
            && field.material_scope() == PreparedFieldMaterialScopeV1::ContextNeutral,
        first_divergence,
    }
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
        .prepared_material()
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
            .prepared_material()
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
