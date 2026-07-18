use super::{
    admit_evaluated_hidden_transition, phase_policy_rejection,
    producer_allows_authority_evaluation, unresolved_competitor_blocks, TransitionDecisionPolicy,
};
use crate::candidate_contract::{CandidateOrigin, CorrectionSourceRole};
use crate::correction_core::{
    CandidateGateAction, CandidateGateDecision, CorrectionDecisionSource, TypingErrorClass,
    TypingErrorEvent, UnifiedCorrectionCandidate,
};

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
fn phase_cutover_blocks_learned_and_deterministic_typo_sources() {
    let disabled = TransitionDecisionPolicy {
        l2_phase_apply: false,
    };
    let enabled = TransitionDecisionPolicy {
        l2_phase_apply: true,
    };

    assert_eq!(
        phase_policy_rejection(
            disabled,
            CorrectionSourceRole::L2Surface,
            true,
            true,
            true,
            crate::nanda_wave::PhaseVerdict::Repel,
        ),
        None
    );
    assert_eq!(
        phase_policy_rejection(
            enabled,
            CorrectionSourceRole::L2Surface,
            true,
            true,
            true,
            crate::nanda_wave::PhaseVerdict::Repel,
        ),
        Some("l2_transition_phase_repel")
    );
    assert_eq!(
        phase_policy_rejection(
            enabled,
            CorrectionSourceRole::L2Surface,
            true,
            true,
            true,
            crate::nanda_wave::PhaseVerdict::Unknown,
        ),
        Some("l2_transition_phase_unknown")
    );
    assert_eq!(
        phase_policy_rejection(
            enabled,
            CorrectionSourceRole::DeterministicTypo,
            true,
            true,
            true,
            crate::nanda_wave::PhaseVerdict::Repel
        ),
        Some("l2_transition_phase_repel")
    );
    assert_eq!(
        phase_policy_rejection(
            enabled,
            CorrectionSourceRole::DeterministicTypo,
            true,
            true,
            true,
            crate::nanda_wave::PhaseVerdict::Unknown
        ),
        Some("l2_transition_phase_unknown")
    );
    assert_eq!(
        phase_policy_rejection(
            enabled,
            CorrectionSourceRole::DeterministicTypo,
            true,
            true,
            true,
            crate::nanda_wave::PhaseVerdict::Support
        ),
        None
    );
    assert_eq!(
        phase_policy_rejection(
            enabled,
            CorrectionSourceRole::Layout,
            true,
            true,
            true,
            crate::nanda_wave::PhaseVerdict::Repel
        ),
        None
    );
    assert_eq!(
        phase_policy_rejection(
            enabled,
            CorrectionSourceRole::L2Surface,
            false,
            false,
            false,
            crate::nanda_wave::PhaseVerdict::Unknown,
        ),
        Some("l2_transition_phase_package_missing")
    );
    assert_eq!(
        phase_policy_rejection(
            enabled,
            CorrectionSourceRole::L2Surface,
            true,
            true,
            false,
            crate::nanda_wave::PhaseVerdict::Support,
        ),
        Some("l2_transition_phase_shadow_only")
    );
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
    assert!(super::known_word_drift_has_authority(
        CorrectionSourceRole::DeterministicTypo,
        1,
        false,
        true,
    ));
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
        neutral,
    ));
    assert!(!producer_allows_authority_evaluation(
        CandidateGateAction::SuggestOnly,
        neutral,
    ));
    assert!(producer_allows_authority_evaluation(
        CandidateGateAction::SuggestOnly,
        exact_positive,
    ));
    assert!(!producer_allows_authority_evaluation(
        CandidateGateAction::SuggestOnly,
        exact_negative,
    ));
}

#[test]
fn exact_positive_l4_memory_outvotes_only_unresolved_competitors() {
    assert!(unresolved_competitor_blocks(false, true));
    assert!(!unresolved_competitor_blocks(true, true));
    assert!(!unresolved_competitor_blocks(false, false));
}
