//! Frame-bound candidate validity shadow.
//!
//! This module has no live rank, display, context, admission or mutation API.

use sha2::{Digest, Sha256};

use super::material_frame::{
    validate_lease, BoundFrameTargetV1, ExactInputFrameV1, PreparedTargetMaterialShadowV1,
};
use crate::typing_transition::target_evidence::{
    stable_bytes_ref, AbsoluteAuthorityBlockerV1, CandidateStateV1, EnumerationStateV1,
    FrameOriginalPreservationV1, FrameOriginalPreservationVerdictV1, GroundingNamespaceV1,
    IncompletenessReasonV1, InputFrameIdentityV1, PreparedMaterialLeaseV1,
    PreparedOriginalLexicalStatusV1, PreparedOriginalPunctuationStatusV1,
    PreparedOriginalScriptTokenStatusV1, TargetRejectionReasonV1, TargetRelationV1,
    VerdictMembershipV1, WitnessRejectionReasonV1, MAX_TARGET_WITNESSES_PER_TARGET,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TargetNamespaceSettlementV1 {
    Incomplete(IncompletenessReasonV1),
    CompleteExactGrounding,
    CompleteTargetAbsent,
    CompleteUnsupportedIdentity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WitnessFrameAssessmentV1 {
    pub(super) material_witness_ref: u8,
    pub(super) valid_geometry: bool,
    pub(super) rejection: Option<WitnessRejectionReasonV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CandidateValidityShadowV1 {
    pub(super) material_target_ref: u16,
    pub(super) state: CandidateStateV1,
    pub(super) authority_blockers: Vec<AbsoluteAuthorityBlockerV1>,
    pub(super) valid_grounded_witnesses: u8,
    pub(super) rejected_witnesses: u8,
    pub(super) exact_projected_target_hash: [u64; 2],
}

#[expect(
    clippy::too_many_arguments,
    reason = "sealed evidence inputs remain explicit"
)]
pub(super) fn derive_candidate_validity_shadow(
    material: &PreparedTargetMaterialShadowV1,
    lease: PreparedMaterialLeaseV1,
    expected: &ExactInputFrameV1,
    current: &ExactInputFrameV1,
    bound: &BoundFrameTargetV1,
    namespace: TargetNamespaceSettlementV1,
    witness_assessments: &[WitnessFrameAssessmentV1],
    now_monotonic_ns: u64,
) -> Result<CandidateValidityShadowV1, &'static str> {
    validate_lease(material, lease, expected, now_monotonic_ns)
        .map_err(|_| "candidate validity material lease is invalid")?;
    expected
        .compare_exact(current)
        .map_err(|_| "candidate validity frame is stale")?;
    validate_bound_target(current, bound)?;
    let target_ref = usize::from(bound.identity.material_target_ref);
    let target = material
        .compact()
        .targets
        .as_slice()
        .get(target_ref)
        .ok_or("bound target reference is outside prepared material")?;
    if stable_bytes_ref(bound.projected_target.as_bytes())
        != bound.identity.exact_projected_target_bytes_ref
    {
        return Err("bound projected target bytes disagree with frame identity");
    }

    let witnesses = target.witnesses.witnesses();
    let mut seen = [false; MAX_TARGET_WITNESSES_PER_TARGET];
    let mut valid_grounded_witnesses = 0_u8;
    let mut rejected_witnesses = 0_u8;
    let mut malformed_witness = false;
    for assessment in witness_assessments {
        let index = usize::from(assessment.material_witness_ref);
        let witness = witnesses
            .get(index)
            .ok_or("witness assessment reference is outside prepared target")?;
        if seen[index] {
            return Err("witness assessment is duplicated");
        }
        seen[index] = true;
        if assessment.valid_geometry && assessment.rejection.is_some() {
            return Err("witness cannot be valid and rejected simultaneously");
        }
        let root_is_grounded = witness.relation != TargetRelationV1::Unsupported
            && witness.grounding_namespace != GroundingNamespaceV1::None
            && matches!(
                witness.verdict_membership,
                VerdictMembershipV1::Grounded
                    | VerdictMembershipV1::L11Winner
                    | VerdictMembershipV1::L11Tied
            );
        if assessment.valid_geometry && root_is_grounded {
            valid_grounded_witnesses = valid_grounded_witnesses.saturating_add(1);
        } else if let Some(reason) = assessment.rejection {
            rejected_witnesses = rejected_witnesses.saturating_add(1);
            malformed_witness |= reason == WitnessRejectionReasonV1::MalformedEvidenceRoot;
        }
    }

    let material_complete = material.completeness().state() == EnumerationStateV1::Complete;
    let target_evidence_complete = target.witnesses.state() == EnumerationStateV1::Complete;
    let all_witnesses_assessed = witness_assessments.len() == witnesses.len();
    let mut authority_blockers = Vec::new();
    push_material_blocker(
        &mut authority_blockers,
        material.completeness().state(),
        material.completeness().reason(),
    );
    push_material_blocker(
        &mut authority_blockers,
        target.witnesses.state(),
        target.witnesses.reason(),
    );
    if malformed_witness || (!all_witnesses_assessed && !witnesses.is_empty()) {
        push_blocker(
            &mut authority_blockers,
            AbsoluteAuthorityBlockerV1::EvidenceIntegrityIncomplete,
        );
    }

    let state = if valid_grounded_witnesses > 0 {
        CandidateStateV1::Grounded
    } else {
        match namespace {
            TargetNamespaceSettlementV1::Incomplete(_) => CandidateStateV1::Born,
            TargetNamespaceSettlementV1::CompleteExactGrounding
                if material_complete && target_evidence_complete && all_witnesses_assessed =>
            {
                CandidateStateV1::Rejected(TargetRejectionReasonV1::CompleteNoValidGeometry)
            }
            TargetNamespaceSettlementV1::CompleteTargetAbsent
                if material_complete && target_evidence_complete =>
            {
                CandidateStateV1::Rejected(TargetRejectionReasonV1::TargetAbsentFromGrounding)
            }
            TargetNamespaceSettlementV1::CompleteUnsupportedIdentity
                if material_complete && target_evidence_complete =>
            {
                CandidateStateV1::Rejected(TargetRejectionReasonV1::UnsupportedTargetIdentity)
            }
            _ => CandidateStateV1::Born,
        }
    };
    if matches!(namespace, TargetNamespaceSettlementV1::Incomplete(_)) {
        push_blocker(
            &mut authority_blockers,
            AbsoluteAuthorityBlockerV1::UpstreamEnumerationIncomplete,
        );
    }

    Ok(CandidateValidityShadowV1 {
        material_target_ref: bound.identity.material_target_ref,
        state,
        authority_blockers,
        valid_grounded_witnesses,
        rejected_witnesses,
        exact_projected_target_hash: digest128(
            Sha256::digest(bound.projected_target.as_bytes()).into(),
        ),
    })
}

pub(super) fn derive_original_preservation_shadow(
    material: &PreparedTargetMaterialShadowV1,
    lease: PreparedMaterialLeaseV1,
    expected: &ExactInputFrameV1,
    current: &ExactInputFrameV1,
    now_monotonic_ns: u64,
) -> Result<Option<FrameOriginalPreservationV1>, &'static str> {
    validate_lease(material, lease, expected, now_monotonic_ns)
        .map_err(|_| "original preservation material lease is invalid")?;
    expected
        .compare_exact(current)
        .map_err(|_| "original preservation frame is stale")?;
    if current.source_window().as_bytes() != material.exact_observed().as_bytes()
        || stable_bytes_ref(current.source_window().as_bytes())
            != material.compact().original.exact_observed_bytes_ref
    {
        return Err("original preservation source does not match prepared material");
    }
    let original = material.compact().original;
    let verdict = match (
        original.lexical_status,
        original.script_token_status,
        original.punctuation_status,
    ) {
        (PreparedOriginalLexicalStatusV1::Protected, _, _)
        | (PreparedOriginalLexicalStatusV1::Clean, _, _)
        | (_, PreparedOriginalScriptTokenStatusV1::Unsupported, _)
        | (_, _, PreparedOriginalPunctuationStatusV1::Protected) => {
            FrameOriginalPreservationVerdictV1::Preserve
        }
        (
            PreparedOriginalLexicalStatusV1::Damaged,
            PreparedOriginalScriptTokenStatusV1::Supported,
            PreparedOriginalPunctuationStatusV1::Stable,
        ) => FrameOriginalPreservationVerdictV1::ReplacePermitted,
        _ => return Ok(None),
    };
    let mut original_hasher = Sha256::new();
    original_hasher.update(b"lay-prepared-original-material-v1\0");
    original_hasher.update(original.exact_observed_scalars_ref.to_le_bytes());
    original_hasher.update(original.exact_observed_bytes_ref.to_le_bytes());
    original_hasher.update(original.preservation_schema.to_le_bytes());
    original_hasher.update([
        original.lexical_status as u8,
        original.script_token_status as u8,
        original.punctuation_status as u8,
    ]);
    Ok(Some(FrameOriginalPreservationV1 {
        prepared_material_lease_id: lease.lease_identity,
        exact_frame_identity_ref: stable_bytes_ref(&frame_identity_bytes(current.identity())),
        prepared_original_material_hash: digest128(original_hasher.finalize().into()),
        verdict,
        reserved: [0; 3],
    }))
}

pub(super) fn validate_bound_target(
    frame: &ExactInputFrameV1,
    bound: &BoundFrameTargetV1,
) -> Result<(), &'static str> {
    let span = bound.identity.replacement_span;
    let source = frame.source_window().chars().collect::<Vec<_>>();
    if span.source_scalar_len as usize != source.len() {
        return Err("bound target source length is stale");
    }
    let start = span.scalar_start as usize;
    let end = start
        .checked_add(span.scalar_len as usize)
        .filter(|end| *end <= source.len())
        .ok_or("bound target span is outside exact frame")?;
    let source_slice = source[start..end].iter().collect::<String>();
    if digest128(Sha256::digest(source_slice.as_bytes()).into()) != span.exact_source_slice_hash {
        return Err("bound target source slice hash disagrees with exact frame");
    }
    let replayed = source[..start]
        .iter()
        .chain(bound.projected_target.chars().collect::<Vec<_>>().iter())
        .chain(source[end..].iter())
        .collect::<String>();
    if replayed.as_bytes() != bound.replayed_source_window.as_bytes() {
        return Err("bound target replay is not exact");
    }
    Ok(())
}

fn push_material_blocker(
    blockers: &mut Vec<AbsoluteAuthorityBlockerV1>,
    state: EnumerationStateV1,
    reason: IncompletenessReasonV1,
) {
    if state == EnumerationStateV1::Complete {
        return;
    }
    let blocker = match reason {
        IncompletenessReasonV1::StorageCapacity => AbsoluteAuthorityBlockerV1::TargetSetOverflow,
        IncompletenessReasonV1::IntegrityFailure => {
            AbsoluteAuthorityBlockerV1::EvidenceIntegrityIncomplete
        }
        IncompletenessReasonV1::WorkBudgetExceeded
        | IncompletenessReasonV1::UpstreamIncomplete
        | IncompletenessReasonV1::None => AbsoluteAuthorityBlockerV1::UpstreamEnumerationIncomplete,
    };
    push_blocker(blockers, blocker);
}

fn push_blocker(
    blockers: &mut Vec<AbsoluteAuthorityBlockerV1>,
    blocker: AbsoluteAuthorityBlockerV1,
) {
    if !blockers.contains(&blocker) {
        blockers.push(blocker);
    }
}

fn frame_identity_bytes(identity: InputFrameIdentityV1) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(96);
    bytes.extend_from_slice(&identity.focus_serial.to_le_bytes());
    bytes.extend_from_slice(&identity.tail_epoch.to_le_bytes());
    bytes.extend_from_slice(&identity.exact_source_window_ref.to_le_bytes());
    bytes.extend_from_slice(&identity.exact_left_context_ref.to_le_bytes());
    bytes.extend_from_slice(&identity.source_scalar_count.to_le_bytes());
    bytes.extend_from_slice(&identity.caret_scalar.to_le_bytes());
    bytes.extend_from_slice(&identity.selection_start_scalar.to_le_bytes());
    bytes.extend_from_slice(&identity.selection_end_scalar.to_le_bytes());
    bytes.extend_from_slice(&identity.exact_preedit_bytes_ref.to_le_bytes());
    bytes.extend_from_slice(&identity.preedit_cursor_scalar.to_le_bytes());
    bytes.extend_from_slice(&identity.layout_generation.to_le_bytes());
    bytes.extend_from_slice(&identity.config_generation.to_le_bytes());
    bytes.extend_from_slice(&identity.package_generation.to_le_bytes());
    bytes.extend_from_slice(&identity.field_generation.to_le_bytes());
    bytes
}

fn digest128(digest: [u8; 32]) -> [u64; 2] {
    [
        u64::from_le_bytes(digest[..8].try_into().expect("SHA-256 prefix")),
        u64::from_le_bytes(digest[8..16].try_into().expect("SHA-256 prefix")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::l2_field::productive_v1::calibrate::{
        CandidateProvenanceClassV1, CandidateRankOriginV1, ProductiveCalibratedVerdictV1,
    };
    use crate::nanda_wave::l2_field::productive_v1::geometry::GeometryTerminalEvidenceV1;
    use crate::nanda_wave::l2_field::productive_v1::material_frame::{
        bind_exact_frame_target, prepare_context_neutral_productive_material,
        set_original_status_for_test, ExactPackageTupleV1, PreparedMaterialLeaseArenaV1,
    };
    use crate::nanda_wave::l2_field::productive_v1::packaged_runtime::{
        ContextNeutralProductiveEnumerationV1, PackagedProductiveCandidateV1,
        PackagedProductiveReadoutV1,
    };
    use crate::nanda_wave::l2_field::productive_v1::types::ProductiveCandidateIdentityV1;
    use crate::typing_transition::target_evidence::{
        EnumerationWorkCountersV1, LeaseConsumerStateV1,
    };

    fn package_tuple() -> ExactPackageTupleV1 {
        ExactPackageTupleV1 {
            l11_sha256: [1; 32],
            canonical_l2_sha256: [2; 32],
            productive_sha256: [3; 32],
        }
    }

    fn candidate(
        surface: &str,
        grounded: bool,
        identities: usize,
    ) -> PackagedProductiveCandidateV1 {
        let identity = ProductiveCandidateIdentityV1 {
            lemma_id: 1,
            paradigm_id: 2,
            program_id: 3,
            target_slot_id: 4,
            normalized_surface_id: stable_bytes_ref(surface.as_bytes()),
            variant_id: 1,
        };
        let equivalent_identities = (0..identities)
            .map(|index| ProductiveCandidateIdentityV1 {
                lemma_id: identity.lemma_id + index as u32,
                paradigm_id: identity.paradigm_id + index as u32,
                program_id: identity.program_id + index as u32,
                target_slot_id: identity.target_slot_id,
                normalized_surface_id: identity.normalized_surface_id,
                variant_id: identity.variant_id,
            })
            .collect::<Vec<_>>();
        PackagedProductiveCandidateV1 {
            identity,
            equivalent_identities,
            normalized_surface: surface.into(),
            score_q16: 0,
            geometry: GeometryTerminalEvidenceV1::default(),
            provenance: CandidateProvenanceClassV1::ColdLemmaBinding,
            minimum_independent_support: 2,
            grounded_support: u32::from(grounded) * 2,
            ambiguity_center_cosine: 0,
            equivalent_identity_count: identities as u32,
            equivalent_paradigm_count: identities as u32,
            minimum_equivalent_support: 2,
            maximum_equivalent_support: 2,
            rank_origin: CandidateRankOriginV1::BaseV64,
            cross_lane_certified: false,
        }
    }

    fn material(
        grounded: bool,
        identities: usize,
        work_budget_exceeded: bool,
    ) -> PreparedTargetMaterialShadowV1 {
        let candidates = vec![candidate("target", grounded, identities)];
        let count = candidates.len() as u64;
        prepare_context_neutral_productive_material(
            "source",
            package_tuple(),
            ContextNeutralProductiveEnumerationV1 {
                readout: PackagedProductiveReadoutV1 {
                    verdict: ProductiveCalibratedVerdictV1::Abstain {
                        suggestions: Vec::new(),
                        productive_overflow: false,
                    },
                    candidates,
                    logical_terminal_count: count,
                    logical_surface_basin_count: count,
                    integrity_error: None,
                },
                productive_work: EnumerationWorkCountersV1::default(),
                aggregate_work: EnumerationWorkCountersV1::default(),
                work_budget_exceeded,
            },
        )
        .expect("material")
    }

    fn frame(material: &PreparedTargetMaterialShadowV1) -> ExactInputFrameV1 {
        ExactInputFrameV1::new(
            11,
            12,
            "source".to_string(),
            "context".to_string(),
            6,
            (6, 6),
            String::new(),
            0,
            13,
            14,
            material.compact().key.package_generation,
            15,
        )
        .expect("frame")
    }

    fn lease_and_bound(
        material: &PreparedTargetMaterialShadowV1,
        frame: &ExactInputFrameV1,
    ) -> (PreparedMaterialLeaseV1, BoundFrameTargetV1) {
        let mut arena = PreparedMaterialLeaseArenaV1::default();
        let lease = arena
            .pin(
                material,
                15,
                16,
                [17, 18],
                1_000,
                LeaseConsumerStateV1::FrameSettlement,
            )
            .expect("lease");
        let bound = bind_exact_frame_target(material, lease, frame, frame, 0, 0, 6, 0, 0, 500)
            .expect("bound");
        (lease, bound)
    }

    #[test]
    fn independent_grounded_witness_survives_local_rejection() {
        let material = material(true, 2, false);
        let frame = frame(&material);
        let (lease, bound) = lease_and_bound(&material, &frame);
        let result = derive_candidate_validity_shadow(
            &material,
            lease,
            &frame,
            &frame,
            &bound,
            TargetNamespaceSettlementV1::CompleteExactGrounding,
            &[
                WitnessFrameAssessmentV1 {
                    material_witness_ref: 0,
                    valid_geometry: false,
                    rejection: Some(WitnessRejectionReasonV1::GeometryReplayMismatch),
                },
                WitnessFrameAssessmentV1 {
                    material_witness_ref: 1,
                    valid_geometry: true,
                    rejection: None,
                },
            ],
            500,
        )
        .unwrap();
        assert_eq!(result.state, CandidateStateV1::Grounded);
        assert_eq!(result.valid_grounded_witnesses, 1);
        assert_eq!(result.rejected_witnesses, 1);
    }

    #[test]
    fn incomplete_namespace_remains_born_and_blocks_authority() {
        let material = material(false, 1, false);
        let frame = frame(&material);
        let (lease, bound) = lease_and_bound(&material, &frame);
        let result = derive_candidate_validity_shadow(
            &material,
            lease,
            &frame,
            &frame,
            &bound,
            TargetNamespaceSettlementV1::Incomplete(IncompletenessReasonV1::WorkBudgetExceeded),
            &[WitnessFrameAssessmentV1 {
                material_witness_ref: 0,
                valid_geometry: false,
                rejection: None,
            }],
            500,
        )
        .unwrap();
        assert_eq!(result.state, CandidateStateV1::Born);
        assert!(result
            .authority_blockers
            .contains(&AbsoluteAuthorityBlockerV1::UpstreamEnumerationIncomplete));
    }

    #[test]
    fn complete_namespace_can_reject_only_after_all_geometry_is_accounted() {
        let material = material(true, 1, false);
        let frame = frame(&material);
        let (lease, bound) = lease_and_bound(&material, &frame);
        let result = derive_candidate_validity_shadow(
            &material,
            lease,
            &frame,
            &frame,
            &bound,
            TargetNamespaceSettlementV1::CompleteExactGrounding,
            &[WitnessFrameAssessmentV1 {
                material_witness_ref: 0,
                valid_geometry: false,
                rejection: Some(WitnessRejectionReasonV1::GeometryReplayMismatch),
            }],
            500,
        )
        .unwrap();
        assert_eq!(
            result.state,
            CandidateStateV1::Rejected(TargetRejectionReasonV1::CompleteNoValidGeometry)
        );
    }

    #[test]
    fn unassessed_witness_prevents_complete_geometry_rejection() {
        let material = material(true, 2, false);
        let frame = frame(&material);
        let (lease, bound) = lease_and_bound(&material, &frame);
        let result = derive_candidate_validity_shadow(
            &material,
            lease,
            &frame,
            &frame,
            &bound,
            TargetNamespaceSettlementV1::CompleteExactGrounding,
            &[WitnessFrameAssessmentV1 {
                material_witness_ref: 0,
                valid_geometry: false,
                rejection: Some(WitnessRejectionReasonV1::GeometryReplayMismatch),
            }],
            500,
        )
        .unwrap();
        assert_eq!(result.state, CandidateStateV1::Born);
        assert!(result
            .authority_blockers
            .contains(&AbsoluteAuthorityBlockerV1::EvidenceIntegrityIncomplete));
    }

    #[test]
    fn valid_grounding_survives_field_incompleteness_but_cannot_gain_authority() {
        let material = material(true, 1, true);
        let frame = frame(&material);
        let (lease, bound) = lease_and_bound(&material, &frame);
        let result = derive_candidate_validity_shadow(
            &material,
            lease,
            &frame,
            &frame,
            &bound,
            TargetNamespaceSettlementV1::CompleteExactGrounding,
            &[WitnessFrameAssessmentV1 {
                material_witness_ref: 0,
                valid_geometry: true,
                rejection: None,
            }],
            500,
        )
        .unwrap();
        assert_eq!(result.state, CandidateStateV1::Grounded);
        assert!(result
            .authority_blockers
            .contains(&AbsoluteAuthorityBlockerV1::UpstreamEnumerationIncomplete));
    }

    #[test]
    fn stale_frame_aborts_globally_instead_of_rejecting_target() {
        let material = material(true, 1, false);
        let expected = frame(&material);
        let (lease, bound) = lease_and_bound(&material, &expected);
        let current = ExactInputFrameV1::new(
            99,
            12,
            "source".to_string(),
            "context".to_string(),
            6,
            (6, 6),
            String::new(),
            0,
            13,
            14,
            material.compact().key.package_generation,
            15,
        )
        .unwrap();
        assert!(derive_candidate_validity_shadow(
            &material,
            lease,
            &expected,
            &current,
            &bound,
            TargetNamespaceSettlementV1::CompleteExactGrounding,
            &[],
            500,
        )
        .is_err());
    }

    #[test]
    fn original_preservation_is_separate_and_typed() {
        let mut material = material(true, 1, false);
        let frame = frame(&material);
        set_original_status_for_test(
            &mut material,
            PreparedOriginalLexicalStatusV1::Clean,
            PreparedOriginalScriptTokenStatusV1::Supported,
            PreparedOriginalPunctuationStatusV1::Stable,
        );
        let (lease, _) = lease_and_bound(&material, &frame);
        let preservation =
            derive_original_preservation_shadow(&material, lease, &frame, &frame, 500)
                .unwrap()
                .unwrap();
        assert_eq!(
            preservation.verdict,
            FrameOriginalPreservationVerdictV1::Preserve
        );
        assert_eq!(material.compact().targets.len(), 1);
    }
}
