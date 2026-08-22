use super::{
    admit_evaluated_hidden_transition, producer_allows_authority_evaluation,
    unresolved_competitor_blocks,
};
use crate::candidate_contract::{CandidateOrigin, CorrectionSourceRole};
use crate::correction_core::{
    CandidateGateAction, CandidateGateDecision, CorrectionDecisionSource, MorphologySlotEvidence,
    TypingErrorClass, TypingErrorEvent, UnifiedCorrectionCandidate,
};

#[test]
fn closed_exact_taxonomy_is_exhaustive_and_fail_closed() {
    use crate::candidate_contract::CandidateReadoutRoute;
    use crate::text_edit::TransitionProof;
    use crate::transition_relation::TransitionOperatorKind;
    use crate::typing_candidate::TypingCandidateFamily;
    use crate::typing_transition::live_candidate::{LiveCandidateLane, ReplacementTargetEvidence};
    use crate::typing_transition::target_evidence::{
        CompletenessScopeKindV1, EnumerationStateV1, IncompletenessReasonV1,
    };

    assert!(super::closed_exact_readout_route_preserves_retained_target(
        CandidateReadoutRoute::CanonicalL2Field
    ));
    assert!(super::closed_exact_readout_route_preserves_retained_target(
        CandidateReadoutRoute::FullWave
    ));

    for lane in [
        LiveCandidateLane::ExactCompletion,
        LiveCandidateLane::LexicalRepairReplacement,
        LiveCandidateLane::CorrectedPrefixReplacement,
        LiveCandidateLane::GeneralReplacement,
        LiveCandidateLane::BoundaryReplacement,
    ] {
        assert!(!super::exact_live_candidate_lane(lane));
    }
    assert!(super::exact_live_candidate_lane(
        LiveCandidateLane::LayoutReplacement
    ));

    for family in [
        TypingCandidateFamily::Exact,
        TypingCandidateFamily::Visual,
        TypingCandidateFamily::Structural,
        TypingCandidateFamily::Typo,
        TypingCandidateFamily::Cleanup,
        TypingCandidateFamily::Unknown,
    ] {
        assert!(!super::exact_typing_candidate_family(family));
    }
    assert!(super::exact_typing_candidate_family(
        TypingCandidateFamily::Layout
    ));

    for evidence in [
        ReplacementTargetEvidence::None,
        ReplacementTargetEvidence::VerifiedLexicalEdit,
        ReplacementTargetEvidence::ContextBoundLexicalEdit,
        ReplacementTargetEvidence::VerifiedBoundary,
    ] {
        assert!(!super::exact_replacement_target_evidence(evidence));
    }
    assert!(super::exact_replacement_target_evidence(
        ReplacementTargetEvidence::ExactLayoutProjection
    ));

    assert!(super::exact_enumeration_state(EnumerationStateV1::Complete));
    assert!(!super::exact_enumeration_state(
        EnumerationStateV1::Overflow
    ));
    assert!(!super::exact_enumeration_state(EnumerationStateV1::Failed));
    assert!(super::exact_completeness_scope_kind(
        CompletenessScopeKindV1::WholePreparedField
    ));
    assert!(!super::exact_completeness_scope_kind(
        CompletenessScopeKindV1::EditFootprintPartition
    ));
    assert!(!super::exact_completeness_scope_kind(
        CompletenessScopeKindV1::RelationPartition
    ));
    assert!(super::exact_incompleteness_reason(
        IncompletenessReasonV1::None
    ));
    for reason in [
        IncompletenessReasonV1::StorageCapacity,
        IncompletenessReasonV1::WorkBudgetExceeded,
        IncompletenessReasonV1::UpstreamIncomplete,
        IncompletenessReasonV1::IntegrityFailure,
    ] {
        assert!(!super::exact_incompleteness_reason(reason));
    }

    assert!(super::exact_transition_proof(TransitionProof::Layout));
    for proof in [
        TransitionProof::Typo,
        TransitionProof::Boundary,
        TransitionProof::Completion,
        TransitionProof::Context,
        TransitionProof::Grammar,
        TransitionProof::VisibleState,
        TransitionProof::DecoderPlan,
        TransitionProof::ManualIntent,
        TransitionProof::UndoRecord,
        TransitionProof::NativeIntent,
        TransitionProof::Invariant,
    ] {
        assert!(!super::exact_transition_proof(proof));
    }

    assert!(super::exact_transition_operator_kind(
        TransitionOperatorKind::LayoutProjection
    ));
    for operator in [
        TransitionOperatorKind::AdjacentTransposition,
        TransitionOperatorKind::MissingLetterRepair,
        TransitionOperatorKind::RepeatedLetterRepair,
        TransitionOperatorKind::ExtraLetterRepair,
        TransitionOperatorKind::LetterSubstitution,
        TransitionOperatorKind::BoundarySplit,
        TransitionOperatorKind::BoundaryMerge,
        TransitionOperatorKind::AcceptCompletion,
        TransitionOperatorKind::CompositeTypo,
        TransitionOperatorKind::ContextChoice,
        TransitionOperatorKind::ManualToggle,
        TransitionOperatorKind::Other,
    ] {
        assert!(!super::exact_transition_operator_kind(operator));
    }

    assert!(!super::exact_candidate_authority(
        &crate::correction_core::CandidateAuthorityEvidence::None
    ));
    assert!(!super::exact_candidate_authority(
        &crate::correction_core::CandidateAuthorityEvidence::Conflict
    ));

    let no_certificate = super::TransitionDecisionCore::evaluate_candidates(
        &event("ghbdtn "),
        &[],
        super::TransitionDecisionPolicy::default(),
        super::DecisionEvidenceMode::ClosedExactAbsent,
    );
    assert!(no_certificate.selected_index.is_none());
    assert!(no_certificate.selected_transition.is_none());
}

fn morphology_evidence(
    lemma_id: u32,
    target_feature_mask: u32,
    context_positive_support: u32,
    context_alternative_support: u32,
    context_posterior_milli: u16,
    slot_evidence_milli: i32,
    joint_evidence_milli: u16,
) -> MorphologySlotEvidence {
    MorphologySlotEvidence {
        lemma_id,
        source_feature_mask: 1,
        target_feature_mask,
        context_positive_support,
        context_alternative_support,
        context_posterior_milli,
        slot_evidence_milli,
        joint_evidence_milli,
        generated: false,
    }
}

#[test]
fn morphology_same_lemma_slot_evidence_reranks_the_supported_ending() {
    let mut supported = l2_candidate(
        "вы принуждаете ",
        "CanonicalL2FieldSurface",
        TypingErrorClass::GrammarAgreement,
    );
    supported.extend_morphology_slot_evidence([morphology_evidence(17, 10, 4, 0, 820, 700, 910)]);
    let mut alternative = l2_candidate(
        "вы принуждали ",
        "CanonicalL2FieldSurface",
        TypingErrorClass::GrammarAgreement,
    );
    alternative.extend_morphology_slot_evidence([morphology_evidence(17, 11, 0, 4, 420, 300, 760)]);

    let signals = super::l2_morphology_slot_signals(&[supported, alternative]);

    assert_eq!(signals[0].disposition, "same_lemma_support");
    assert!(signals[0].rank_energy > 0.0);
    assert_eq!(signals[0].lemma_id, 17);
    assert_eq!(signals[0].target_feature_mask, 10);
    assert_eq!(signals[0].competitors, 1);
    assert_eq!(signals[1].disposition, "not_applicable");
    assert_eq!(signals[1].rank_energy, 0.0);
}

#[test]
fn morphology_slot_evidence_cannot_settle_cross_lemma_competition() {
    let mut left = l2_candidate(
        "вы принуждаете ",
        "CanonicalL2FieldSurface",
        TypingErrorClass::GrammarAgreement,
    );
    left.extend_morphology_slot_evidence([morphology_evidence(17, 10, 4, 0, 900, 900, 950)]);
    let mut right = l2_candidate(
        "вы приближаете ",
        "CanonicalL2FieldSurface",
        TypingErrorClass::GrammarAgreement,
    );
    right.extend_morphology_slot_evidence([morphology_evidence(23, 10, 4, 0, 300, 200, 700)]);

    let signals = super::l2_morphology_slot_signals(&[left, right]);

    assert!(signals
        .iter()
        .all(|signal| signal.disposition == "not_applicable" && signal.rank_energy == 0.0));
}

#[test]
fn morphology_two_context_supported_slots_remain_tied_despite_frequency_difference() {
    let mut frequent = l2_candidate(
        "ты принуждай ",
        "CanonicalL2FieldSurface",
        TypingErrorClass::GrammarAgreement,
    );
    frequent.extend_morphology_slot_evidence([morphology_evidence(17, 20, 12, 2, 900, 900, 950)]);
    let mut less_frequent = l2_candidate(
        "ты принуждаешь ",
        "CanonicalL2FieldSurface",
        TypingErrorClass::GrammarAgreement,
    );
    less_frequent
        .extend_morphology_slot_evidence([morphology_evidence(17, 21, 2, 12, 300, 200, 700)]);

    let signals = super::l2_morphology_slot_signals(&[frequent, less_frequent]);

    assert!(signals
        .iter()
        .all(|signal| signal.disposition == "not_applicable" && signal.rank_energy == 0.0));
}

#[test]
fn morphology_conflicting_same_lemma_axes_remain_tied() {
    let mut context_favored = l2_candidate(
        "они принуждают ",
        "CanonicalL2FieldSurface",
        TypingErrorClass::GrammarAgreement,
    );
    context_favored
        .extend_morphology_slot_evidence([morphology_evidence(17, 20, 4, 0, 850, 200, 840)]);
    let mut slot_favored = l2_candidate(
        "они принуждали ",
        "CanonicalL2FieldSurface",
        TypingErrorClass::GrammarAgreement,
    );
    slot_favored
        .extend_morphology_slot_evidence([morphology_evidence(17, 21, 4, 0, 650, 800, 820)]);

    let signals = super::l2_morphology_slot_signals(&[context_favored, slot_favored]);

    assert!(signals
        .iter()
        .all(|signal| signal.disposition == "not_applicable" && signal.rank_energy == 0.0));
}

#[test]
fn morphology_budget_lift_preserves_number_geometry_inside_one_case_basin() {
    let singular = super::lift_preserving_relative_geometry(0.42, 0.61, 0.90);
    let plural = super::lift_preserving_relative_geometry(0.61, 0.61, 0.90);

    assert!((plural - singular - 0.19).abs() < f32::EPSILON);
    assert!((plural - 0.90).abs() < f32::EPSILON);
}

#[test]
fn productive_v90_tie_requires_common_l3_and_verified_transition() {
    let event = event("форма нужна форм ");
    let mut recurrent = l2_candidate(
        "форма нужна форма ",
        "ProductiveL2V90Surface",
        TypingErrorClass::GrammarAgreement,
    );
    recurrent.gate = CandidateGateDecision {
        action: CandidateGateAction::SuggestOnly,
        reason: "productive_v90_lattice_requires_common_l3",
    };
    recurrent.extend_morphology_slot_evidence([morphology_evidence(17, 10, 0, 0, 0, 0, 0)]);
    let mut alternative = l2_candidate(
        "форма нужна формы ",
        "ProductiveL2V90Surface",
        TypingErrorClass::GrammarAgreement,
    );
    alternative.gate = CandidateGateDecision {
        action: CandidateGateAction::SuggestOnly,
        reason: "productive_v90_lattice_requires_common_l3",
    };
    alternative.extend_morphology_slot_evidence([morphology_evidence(17, 11, 0, 0, 0, 0, 0)]);

    let candidates = [recurrent, alternative];
    let batch = super::TransitionDecisionCore::evaluate_candidates(
        &event,
        &candidates,
        super::TransitionDecisionPolicy::default(),
        super::DecisionEvidenceMode::FullField(None),
    );

    assert!(batch.evaluations[0].signals.l3_pairwise_certified);
    assert!(!batch.evaluations[1].signals.l3_pairwise_certified);
    assert!(batch.evaluations[0].action.verifier_passed);
    assert_eq!(batch.selected_index, Some(0), "{batch:#?}");
    assert!(batch.selected_transition.is_some());
}

#[test]
fn transition_admission_blocks_unverified_left_context() {
    let event = event("содержкой ");
    let candidate = UnifiedCorrectionCandidate::new(
        "что получилось вроде хороший ввод и даже фикс был шикарный но с содержать ",
        CorrectionDecisionSource::Nanda,
        CandidateOrigin::L2Surface,
        "L2SurfaceMotifCell32",
        TypingErrorClass::CompositeTypo,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "surface_candidate",
        },
    );
    let admission = admit(
        &event,
        &candidate,
        1,
        CorrectionSourceRole::L2Surface,
        false,
    );

    assert!(!admission.allow_apply);
    assert_eq!(admission.reason, "latent_context_unverified");
}

#[test]
fn transition_admission_allows_verified_current_word() {
    let event = event("провека ");
    let candidate = candidate("проверка ", "composite_ru_typo");
    let admission = admit(
        &event,
        &candidate,
        1,
        CorrectionSourceRole::DeterministicTypo,
        true,
    );

    assert!(admission.allow_apply, "reason={}", admission.reason);
}

#[test]
fn suggest_only_verified_tail_boundary_can_enter_authority_evaluation() {
    let event = event("я думаю допусти мнабираю ");
    let candidate = UnifiedCorrectionCandidate::new(
        "я думаю допустим набираю ",
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::Boundary,
        "moved_prefix_pair",
        TypingErrorClass::BoundaryShift,
        CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "edit_transition_not_verified",
        },
    );

    let batch = super::TransitionDecisionCore::evaluate_candidates(
        &event,
        &[candidate],
        super::TransitionDecisionPolicy::default(),
        super::DecisionEvidenceMode::FullField(None),
    );

    assert_eq!(batch.selected_index, Some(0), "{batch:#?}");
}

fn event(text: &str) -> TypingErrorEvent {
    TypingErrorEvent {
        original: text.to_string(),
        core: text.trim().to_string(),
        current_word: text
            .split_whitespace()
            .last()
            .unwrap_or_default()
            .to_string(),
        input_class: TypingErrorClass::CompositeTypo,
    }
}

fn candidate(replacement: &str, source_id: &str) -> UnifiedCorrectionCandidate {
    UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::DeterministicTypo,
        source_id,
        TypingErrorClass::CompositeTypo,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "class_allows_apply",
        },
    )
}

fn layout_candidate(replacement: &str) -> UnifiedCorrectionCandidate {
    UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::Layout,
        "contextual_layout_en_to_ru",
        TypingErrorClass::WrongLayout,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "layout_projection_verified",
        },
    )
}

fn layout_then_typo_candidate(replacement: &str) -> UnifiedCorrectionCandidate {
    UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::LayoutThenTypo,
        "layout_then_known_word",
        TypingErrorClass::CompositeTypo,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "layout_then_typo_verified",
        },
    )
}

fn l2_candidate(
    replacement: &str,
    source_id: &str,
    error_class: TypingErrorClass,
) -> UnifiedCorrectionCandidate {
    UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Nanda,
        CandidateOrigin::L2Surface,
        source_id,
        error_class,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "class_allows_apply",
        },
    )
}

fn admit(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    candidate_count: usize,
    source_role: CorrectionSourceRole,
    context_state_support: bool,
) -> super::TransitionAdmission {
    admit_with_l4_signal(
        event,
        candidate,
        candidate_count,
        source_role,
        context_state_support,
        false,
        crate::typing_transition::L4SignedTransitionSignal {
            negative: false,
            state_specific: false,
            attract_count: 0,
            repel_count: 0,
        },
    )
}

fn admit_with_l4_signal(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    candidate_count: usize,
    source_role: CorrectionSourceRole,
    context_state_support: bool,
    operator_consensus_witness: bool,
    l4_signed_signal: crate::typing_transition::L4SignedTransitionSignal,
) -> super::TransitionAdmission {
    let action = crate::typing_transition::action::verify_action_operator(
        &event.original,
        &candidate.replacement,
        candidate.error_class,
        candidate.origin,
    );
    let transition = crate::typing_transition::TypingTransition::from_evaluated_candidate(
        crate::typing_transition::EvaluatedTransitionInput {
            original: &event.original,
            replacement: &candidate.replacement,
            error_class: candidate.error_class,
            origin: candidate.origin,
            source_id: &candidate.source_id,
            candidate_count,
            action,
            l4_signed_signal,
        },
    );
    admit_evaluated_hidden_transition(
        candidate_count,
        source_role,
        context_state_support,
        operator_consensus_witness,
        &transition,
    )
}

#[test]
fn hidden_state_blocks_single_weak_known_word_drift() {
    let admission = admit(
        &event("мы можем "),
        &candidate("мы модем ", "composite_ru_typo"),
        1,
        CorrectionSourceRole::DeterministicTypo,
        false,
    );

    assert!(!admission.allow_apply);
    assert_eq!(
        admission.reason,
        "latent_known_word_drift_needs_state_proof"
    );
}

#[test]
fn hidden_state_blocks_live_known_form_drifts_from_logs() {
    for (input, replacement, error_class) in [
        ("новости ", "новость ", TypingErrorClass::LetterSubstitution),
        ("модели ", "модель ", TypingErrorClass::LetterSubstitution),
        ("коде ", "код ", TypingErrorClass::ExtraLetter),
        ("вышли ", "вышил ", TypingErrorClass::AdjacentTransposition),
        (
            "окнах ",
            "локонах ",
            TypingErrorClass::SparseInternalMultiOmission,
        ),
    ] {
        let admission = admit(
            &event(input),
            &l2_candidate(replacement, "CanonicalL2FieldSurface", error_class),
            2,
            CorrectionSourceRole::L2Surface,
            false,
        );

        assert!(!admission.allow_apply, "{input:?} -> {replacement:?}");
        assert!(
            matches!(
                admission.reason,
                "known_form_drift_needs_state_proof" | "latent_known_word_drift_needs_state_proof"
            ),
            "{input:?} -> {replacement:?}"
        );
    }
}

#[test]
fn hidden_state_blocks_short_transposition_fragments_from_logs() {
    for (input, replacement) in [
        ("ая ", "яа "),
        ("ту ", "ут "),
        ("вно ", "вон "),
        ("ям ", "мя "),
    ] {
        let admission = admit(
            &event(input),
            &l2_candidate(
                replacement,
                "CanonicalL2FieldSurface",
                TypingErrorClass::AdjacentTransposition,
            ),
            2,
            CorrectionSourceRole::L2Surface,
            false,
        );

        assert!(!admission.allow_apply, "{input:?} -> {replacement:?}");
        assert_eq!(
            admission.reason, "short_transposition_needs_state_proof",
            "{input:?} -> {replacement:?}"
        );
    }
}

#[test]
fn exact_l4_state_proof_allows_known_form_drift() {
    let admission = admit_with_l4_signal(
        &event("новости "),
        &l2_candidate(
            "новость ",
            "CanonicalL2FieldSurface",
            TypingErrorClass::LetterSubstitution,
        ),
        2,
        CorrectionSourceRole::L2Surface,
        false,
        false,
        crate::typing_transition::L4SignedTransitionSignal {
            negative: false,
            state_specific: true,
            attract_count: 2,
            repel_count: 0,
        },
    );

    assert!(admission.allow_apply, "{admission:?}");
}

#[test]
fn hidden_state_allows_unknown_to_known_typo_repair() {
    let admission = admit(
        &event("звгрузи "),
        &candidate("загрузи ", "composite_ru_typo"),
        1,
        CorrectionSourceRole::DeterministicTypo,
        false,
    );

    assert!(admission.allow_apply, "{admission:?}");
}

#[test]
fn exact_state_proof_allows_single_learned_drift() {
    assert!(super::known_word_drift_has_authority(false, true));
}

#[test]
fn l2_operator_phase_is_not_context_state_proof() {
    let admission = admit(
        &event("мы можем "),
        &candidate("мы модем ", "composite_ru_typo"),
        1,
        CorrectionSourceRole::DeterministicTypo,
        false,
    );
    assert!(!admission.allow_apply);
    assert_eq!(
        admission.reason,
        "latent_known_word_drift_needs_state_proof"
    );
}

#[test]
fn hidden_state_blocks_context_imported_candidate_text() {
    let admission = admit(
        &event("можем "),
        &candidate("мы модем ", "composite_ru_typo"),
        2,
        CorrectionSourceRole::DeterministicTypo,
        true,
    );

    assert!(!admission.allow_apply);
    assert_eq!(admission.reason, "latent_context_unverified");
}

#[test]
fn admission_truth_table_uses_verifier_latent_invariants_and_signed_l4_memory() {
    struct Case {
        name: &'static str,
        event: TypingErrorEvent,
        candidate: UnifiedCorrectionCandidate,
        source_role: CorrectionSourceRole,
        strong_transition_support: bool,
        operator_consensus_witness: bool,
        l4_signed_signal: crate::typing_transition::L4SignedTransitionSignal,
        expected_reason: Option<&'static str>,
    }

    let neutral_l4 = crate::typing_transition::L4SignedTransitionSignal {
        negative: false,
        state_specific: false,
        attract_count: 0,
        repel_count: 0,
    };
    let negative_l4 = crate::typing_transition::L4SignedTransitionSignal {
        negative: true,
        state_specific: true,
        attract_count: 0,
        repel_count: 1,
    };
    let generic_negative_l4 = crate::typing_transition::L4SignedTransitionSignal {
        negative: true,
        state_specific: false,
        attract_count: 0,
        repel_count: 16,
    };
    let cases = [
        Case {
            name: "verified_current_word",
            event: event("провека "),
            candidate: candidate("проверка ", "composite_ru_typo"),
            source_role: CorrectionSourceRole::DeterministicTypo,
            strong_transition_support: true,
            operator_consensus_witness: false,
            l4_signed_signal: neutral_l4,
            expected_reason: None,
        },
        Case {
            name: "unverified_context_change",
            event: event("можем "),
            candidate: candidate("мы модем ", "composite_ru_typo"),
            source_role: CorrectionSourceRole::DeterministicTypo,
            strong_transition_support: true,
            operator_consensus_witness: false,
            l4_signed_signal: neutral_l4,
            expected_reason: Some("latent_context_unverified"),
        },
        Case {
            name: "known_word_drift_without_state_proof",
            event: event("мы можем "),
            candidate: candidate("мы модем ", "composite_ru_typo"),
            source_role: CorrectionSourceRole::DeterministicTypo,
            strong_transition_support: false,
            operator_consensus_witness: false,
            l4_signed_signal: neutral_l4,
            expected_reason: Some("latent_known_word_drift_needs_state_proof"),
        },
        Case {
            name: "learned_l4_signed_negative",
            event: event("провека "),
            candidate: candidate("проверка ", "composite_ru_typo"),
            source_role: CorrectionSourceRole::DeterministicTypo,
            strong_transition_support: true,
            operator_consensus_witness: true,
            l4_signed_signal: negative_l4,
            expected_reason: Some("latent_l4_negative_transition_memory"),
        },
        Case {
            name: "generic_l4_negative_without_consensus",
            event: event("преоверка "),
            candidate: candidate("проверка ", "composite_ru_typo"),
            source_role: CorrectionSourceRole::DeterministicTypo,
            strong_transition_support: true,
            operator_consensus_witness: false,
            l4_signed_signal: generic_negative_l4,
            expected_reason: Some("latent_l4_negative_transition_memory"),
        },
        Case {
            name: "operator_consensus_survives_generic_l4_negative",
            event: event("преоверка "),
            candidate: candidate("проверка ", "composite_ru_typo"),
            source_role: CorrectionSourceRole::DeterministicTypo,
            strong_transition_support: true,
            operator_consensus_witness: true,
            l4_signed_signal: generic_negative_l4,
            expected_reason: None,
        },
        Case {
            name: "verified_layout_survives_generic_l4_negative",
            event: event("gjhn "),
            candidate: layout_candidate("порт "),
            source_role: CorrectionSourceRole::Layout,
            strong_transition_support: false,
            operator_consensus_witness: false,
            l4_signed_signal: generic_negative_l4,
            expected_reason: None,
        },
        Case {
            name: "verified_layout_then_typo_survives_generic_l4_negative",
            event: event("lfkmit "),
            candidate: layout_then_typo_candidate("дальше "),
            source_role: CorrectionSourceRole::Layout,
            strong_transition_support: false,
            operator_consensus_witness: false,
            l4_signed_signal: generic_negative_l4,
            expected_reason: None,
        },
        Case {
            name: "verified_layout_blocked_by_state_specific_l4_negative",
            event: event("gjhn "),
            candidate: layout_candidate("порт "),
            source_role: CorrectionSourceRole::Layout,
            strong_transition_support: false,
            operator_consensus_witness: false,
            l4_signed_signal: negative_l4,
            expected_reason: Some("latent_l4_negative_transition_memory"),
        },
    ];

    for case in cases {
        let admission = admit_with_l4_signal(
            &case.event,
            &case.candidate,
            1,
            case.source_role,
            case.strong_transition_support,
            case.operator_consensus_witness,
            case.l4_signed_signal,
        );

        match case.expected_reason {
            Some(reason) => {
                assert!(
                    !admission.allow_apply,
                    "{} unexpectedly admitted: {admission:?}",
                    case.name
                );
                assert_eq!(admission.reason, reason, "{}", case.name);
            }
            None => assert!(
                admission.allow_apply,
                "{} unexpectedly rejected: {admission:?}",
                case.name
            ),
        }
    }
}

#[test]
fn exact_positive_l4_memory_can_promote_a_suggestion_to_authority_evaluation() {
    let neutral = crate::typing_transition::L4SignedTransitionSignal {
        negative: false,
        state_specific: false,
        attract_count: 0,
        repel_count: 0,
    };
    let exact_positive = crate::typing_transition::L4SignedTransitionSignal {
        negative: false,
        state_specific: true,
        attract_count: 6,
        repel_count: 0,
    };
    let exact_negative = crate::typing_transition::L4SignedTransitionSignal {
        negative: true,
        state_specific: true,
        attract_count: 0,
        repel_count: 8,
    };

    assert!(producer_allows_authority_evaluation(
        CandidateGateAction::Eligible,
        false,
        neutral,
    ));
    assert!(!producer_allows_authority_evaluation(
        CandidateGateAction::SuggestOnly,
        false,
        neutral,
    ));
    assert!(producer_allows_authority_evaluation(
        CandidateGateAction::SuggestOnly,
        false,
        exact_positive,
    ));
    assert!(!producer_allows_authority_evaluation(
        CandidateGateAction::SuggestOnly,
        false,
        exact_negative,
    ));
    assert!(producer_allows_authority_evaluation(
        CandidateGateAction::SuggestOnly,
        true,
        neutral,
    ));
}

#[test]
fn exact_positive_l4_memory_outvotes_only_unresolved_competitors() {
    assert!(unresolved_competitor_blocks(false, true));
    assert!(!unresolved_competitor_blocks(true, true));
    assert!(!unresolved_competitor_blocks(false, false));
}
