use super::*;
use crate::nanda_wave::l2_field::productive_v1::calibrate::ProductiveCalibratedVerdictV1;
use crate::nanda_wave::l2_field::productive_v1::contour_birth::{
    TypedContourBirthEnumerationV1, TypedContourBirthV1,
};
use crate::nanda_wave::l2_field::productive_v1::material_frame::{
    prepare_context_neutral_productive_material_with_contours,
    prepare_context_neutral_productive_material_with_contours_and_exact_peaks, ExactPackageTupleV1,
    ExactPeakBirthEnumerationV1, ExactPeakCandidateInputV1, PreparedTargetMaterialShadowV1,
};
use crate::nanda_wave::l2_field::productive_v1::packaged_runtime::{
    ContextNeutralProductiveEnumerationV1, PackagedProductiveReadoutV1,
};
use crate::typing_transition::target_evidence::EnumerationWorkCountersV1;

const SOURCE: &str = "aa";
const TARGET: &str = "aba";
const GROUNDING_REF: u32 = 41;

fn package_tuple() -> ExactPackageTupleV1 {
    ExactPackageTupleV1 {
        l11_sha256: [1; 32],
        canonical_l2_sha256: [2; 32],
        productive_sha256: [3; 32],
    }
}

fn canonical_index() -> &'static StandaloneL2Field {
    static CANONICAL: OnceLock<StandaloneL2Field> = OnceLock::new();
    CANONICAL.get_or_init(|| {
        let corpus = crate::nanda_wave::l2_field::teacher::L2TeacherCorpus::parse_tsv(
            "F\tlemma-a\taba\tnoun:nom:sg\n\
             T\tlemma-a\taba\tnoun:nom:sg\t_ now\n\
             H\tlemma-a\taba\tnoun:nom:sg\tbefore _\n",
        )
        .expect("canonical witness corpus");
        let (package, _) =
            crate::nanda_wave::l2_field::compiler::compile_l2_package(&corpus, 7, |surface| {
                (surface == TARGET).then_some(GROUNDING_REF)
            })
            .expect("canonical witness package");
        StandaloneL2Field::from_package(package).expect("canonical witness index")
    })
}

fn anchor_replay() -> CanonicalL1AnchorReplayContextV1<'static> {
    CanonicalL1AnchorReplayContextV1::new(
        canonical_index(),
        package_tuple().l11_sha256,
        package_tuple().canonical_l2_sha256,
    )
}

fn empty_enumeration() -> ContextNeutralProductiveEnumerationV1 {
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
        productive_work: EnumerationWorkCountersV1::default(),
        aggregate_work: EnumerationWorkCountersV1::default(),
        work_budget_exceeded: false,
    }
}

fn material_with_root(
    relation: TargetRelationV1,
    namespace: GroundingNamespaceV1,
    membership: VerdictMembershipV1,
    operator_ref: u32,
    grounding_ref: u32,
    derivation_ref: u32,
) -> PreparedTargetMaterialShadowV1 {
    prepare_context_neutral_productive_material_with_contours(
        SOURCE,
        package_tuple(),
        empty_enumeration(),
        TypedContourBirthEnumerationV1 {
            births: vec![TypedContourBirthV1 {
                normalized_surface: TARGET.to_string(),
                grounding_namespace: namespace,
                grounding_ref,
                relation,
                operator_ref,
                derivation_ref,
                verdict_membership: membership,
                support_milli: 1_000,
            }],
            canonical_l1_anchor_proofs: Vec::new(),
            work: EnumerationWorkCountersV1::default(),
            logical_match_count: 1,
            all_seen_digest: [11, 13],
            overflow_reason: None,
        },
    )
    .expect("one exact witness root")
}

fn retained_witness(material: &PreparedTargetMaterialShadowV1) -> TargetWitnessV1 {
    material.compact().targets.as_slice()[0]
        .witnesses
        .witnesses()[0]
}

fn assess(
    material: &PreparedTargetMaterialShadowV1,
    witness: TargetWitnessV1,
) -> WitnessFrameAssessmentV1 {
    assess_exact_witness(material, anchor_replay(), 0, 0, SOURCE, TARGET, witness)
}

fn shared_derivation() -> u32 {
    shared_witness_derivation_ref(SOURCE, SOURCE, TARGET)
}

fn raw_terminal_material(operator_ref: u32, derivation_ref: u32) -> PreparedTargetMaterialShadowV1 {
    material_with_root(
        TargetRelationV1::L11Restoration,
        GroundingNamespaceV1::L11Terminal,
        VerdictMembershipV1::L11Winner,
        operator_ref,
        GROUNDING_REF,
        derivation_ref,
    )
}

fn canonical_anchor_material(
    operator_ref: u32,
    derivation_ref: u32,
) -> PreparedTargetMaterialShadowV1 {
    let canonical = canonical_index();
    let target_form_ref = canonical
        .form_ref_for_surface(TARGET)
        .expect("canonical target form");
    let anchor = canonical
        .l1_lexical_anchor_for_form_ref(target_form_ref)
        .expect("canonical target anchor");
    let proof = CanonicalL1AnchorProofV1::new(
        TARGET.to_string(),
        target_form_ref,
        anchor.anchor_form_ref(),
        anchor.lemma_id(),
        anchor.terminal_id(),
        if anchor.is_direct() {
            CanonicalL1AnchorKindV1::Direct
        } else {
            CanonicalL1AnchorKindV1::SameLemma
        },
        package_tuple().l11_sha256,
        package_tuple().canonical_l2_sha256,
    )
    .expect("canonical anchor proof");
    prepare_context_neutral_productive_material_with_contours(
        SOURCE,
        package_tuple(),
        empty_enumeration(),
        TypedContourBirthEnumerationV1 {
            births: vec![TypedContourBirthV1 {
                normalized_surface: TARGET.to_string(),
                grounding_namespace: GroundingNamespaceV1::CanonicalL1Anchor,
                grounding_ref: proof.proof_ref,
                relation: TargetRelationV1::L11Restoration,
                operator_ref,
                derivation_ref,
                verdict_membership: VerdictMembershipV1::L11Winner,
                support_milli: 1_000,
            }],
            canonical_l1_anchor_proofs: vec![proof],
            work: EnumerationWorkCountersV1::default(),
            logical_match_count: 1,
            all_seen_digest: [17, 23],
            overflow_reason: None,
        },
    )
    .expect("one canonical anchor witness root")
}

#[test]
fn exact_witness_replay_rejects_raw_terminal_substitution() {
    let material = raw_terminal_material(
        SHARED_WITNESS_OPERATOR_BASE | u32::from(TargetRelationV1::L11Restoration as u8),
        shared_derivation(),
    );
    let assessment = assess(&material, retained_witness(&material));

    assert!(!assessment.valid_geometry);
    assert_eq!(
        assessment.rejection,
        Some(WitnessRejectionReasonV1::GeometryReplayMismatch)
    );
}

#[test]
fn exact_witness_replay_accepts_owned_exact_peak_root() {
    let certificate = Phase7dCertificateOracle::new(SOURCE)
        .unwrap()
        .certificate_evidence(TARGET)
        .unwrap()
        .into_iter()
        .find(|evidence| supported_phase7d_relation(evidence).is_some())
        .expect("one authority-supported typed geometry");
    let exact_peaks =
        ExactPeakBirthEnumerationV1::from_candidates(vec![ExactPeakCandidateInputV1 {
            form_ref: GROUNDING_REF,
            normalized_surface: TARGET.to_string(),
            certificates: vec![certificate],
        }])
        .unwrap();
    let material = prepare_context_neutral_productive_material_with_contours_and_exact_peaks(
        SOURCE,
        package_tuple(),
        empty_enumeration(),
        TypedContourBirthEnumerationV1::complete_empty(),
        exact_peaks,
    )
    .unwrap();
    let assessment = assess(&material, retained_witness(&material));

    assert!(assessment.valid_geometry);
    assert_eq!(assessment.rejection, None);
}

#[test]
fn exact_witness_replay_rejects_wrong_operator() {
    let material = canonical_anchor_material(0x5348_ffff, shared_derivation());
    let assessment = assess(&material, retained_witness(&material));

    assert!(!assessment.valid_geometry);
    assert_eq!(
        assessment.rejection,
        Some(WitnessRejectionReasonV1::MalformedEvidenceRoot)
    );
}

#[test]
fn exact_witness_replay_rejects_wrong_grounding_namespace_or_ref() {
    let material = material_with_root(
        TargetRelationV1::L11Restoration,
        GroundingNamespaceV1::ProductiveSurface,
        VerdictMembershipV1::Grounded,
        SHARED_WITNESS_OPERATOR_BASE | u32::from(TargetRelationV1::L11Restoration as u8),
        GROUNDING_REF,
        shared_derivation(),
    );
    assert!(!assess(&material, retained_witness(&material)).valid_geometry);

    let material = raw_terminal_material(
        SHARED_WITNESS_OPERATOR_BASE | u32::from(TargetRelationV1::L11Restoration as u8),
        shared_derivation(),
    );
    let retained = retained_witness(&material);
    let wrong_ref = TargetWitnessV1::new(
        retained.relation,
        retained.grounding_namespace,
        retained.verdict_membership,
        retained.flags,
        retained.operator_ref,
        retained.grounding_ref.wrapping_add(1),
        retained.derivation_ref,
        retained.support_milli,
        retained.provenance_annotations,
    );
    let assessment = assess(&material, wrong_ref);
    assert!(!assessment.valid_geometry);
    assert_eq!(
        assessment.rejection,
        Some(WitnessRejectionReasonV1::MalformedEvidenceRoot)
    );
}

#[test]
fn exact_witness_replay_rejects_wrong_derivation() {
    let material = canonical_anchor_material(
        SHARED_WITNESS_OPERATOR_BASE | u32::from(TargetRelationV1::L11Restoration as u8),
        shared_derivation().wrapping_add(1),
    );
    let assessment = assess(&material, retained_witness(&material));

    assert!(!assessment.valid_geometry);
    assert_eq!(
        assessment.rejection,
        Some(WitnessRejectionReasonV1::MalformedEvidenceRoot)
    );
}

#[test]
fn exact_witness_replay_rejects_stale_root_accelerator() {
    let material = raw_terminal_material(
        SHARED_WITNESS_OPERATOR_BASE | u32::from(TargetRelationV1::L11Restoration as u8),
        shared_derivation(),
    );
    let mut stale = retained_witness(&material);
    stale.semantic_root_accelerator ^= u32::MAX;
    let assessment = assess(&material, stale);

    assert!(!assessment.valid_geometry);
    assert_eq!(
        assessment.rejection,
        Some(WitnessRejectionReasonV1::MalformedEvidenceRoot)
    );
}

#[test]
fn exact_witness_replay_rejects_relation_only_unique_geometry() {
    let material = material_with_root(
        TargetRelationV1::MissingLetter,
        GroundingNamespaceV1::ReferenceSurface,
        VerdictMembershipV1::Born,
        0x5335_0002,
        stable_bytes_ref(TARGET.as_bytes()),
        0xdead_beef,
    );
    let assessment = assess(&material, retained_witness(&material));

    assert!(!assessment.valid_geometry);
    assert_eq!(
        assessment.rejection,
        Some(WitnessRejectionReasonV1::GeometryReplayMismatch)
    );
}
