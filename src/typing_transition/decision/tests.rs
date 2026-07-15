use super::{admit_evaluated_hidden_transition, phase_policy_rejection, TransitionDecisionPolicy};
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
    strong_transition_support: bool,
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
            l4_signed_signal: crate::typing_transition::L4SignedTransitionSignal {
                negative: false,
                state_specific: false,
                attract_count: 0,
                repel_count: 0,
            },
        },
    );
    admit_evaluated_hidden_transition(
        candidate_count,
        source_role,
        strong_transition_support,
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
