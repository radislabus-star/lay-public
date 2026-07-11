//! Typing Transition CPU.
//!
//! Candidate producers may propose text, but this module owns the transition
//! shape, verifier contract, and final apply authority.

pub(crate) mod action;
pub(crate) mod candidate;
pub(crate) mod decision;
pub(crate) mod executor_contract;
pub(crate) mod l4_state_estimator;
pub(crate) mod memory;
pub(crate) mod state;
pub(crate) mod verifier;

use crate::correction_core::TypingErrorClass;
use crate::correction_source_contract::CandidateOrigin;
use crate::language_action::LanguageActionOperator;
use crate::transition_relation::{TransitionRelationAtoms, TransitionRelationInput};
use l4_state_estimator::{
    L4ObservationKind, L4StateEstimate, L4StateEstimator, L4StateObservation,
};
use state::LatentTypingState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypingTransition {
    pub(crate) state_before: LatentTypingState,
    pub(crate) action_operator: LanguageActionOperator,
    pub(crate) candidate_text: String,
    pub(crate) state_after_predicted: LatentTypingState,
    pub(crate) evidence: TransitionEvidence,
    pub(crate) l1_signal: L1TransitionSignal,
    pub(crate) l2_signal: L2TransitionSignal,
    pub(crate) l3_signal: L3TransitionSignal,
    pub(crate) l4_signed_signal: L4SignedTransitionSignal,
    pub(crate) l4_state_estimate: L4StateEstimate,
    relation_atoms: TransitionRelationAtoms,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransitionEvidence {
    pub(crate) origin: CandidateOrigin,
    /// Diagnostic provenance only. Decision and verifier authority use `origin`.
    pub(crate) source_id: String,
    pub(crate) error_class: TypingErrorClass,
    pub(crate) verifier_passed: bool,
    pub(crate) left_context_changed: bool,
    pub(crate) changed_tokens: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct L1TransitionSignal {
    pub(crate) boundary_changed: bool,
    pub(crate) word_count_changed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct L2TransitionSignal {
    pub(crate) candidate_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct L3TransitionSignal {
    pub(crate) observes_context: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct L4SignedTransitionSignal {
    pub(crate) negative: bool,
}

impl TypingTransition {
    pub(crate) fn from_candidate(
        original: &str,
        replacement: &str,
        error_class: TypingErrorClass,
        origin: CandidateOrigin,
        source_id: &str,
        candidate_count: usize,
    ) -> Self {
        let action = action::verify_action_operator(original, replacement, error_class, origin);
        let state_before = LatentTypingState::from_text(original);
        let state_after_predicted = LatentTypingState::from_text(replacement);
        let boundary_changed = state_before.word_count_changed(&state_after_predicted);
        let l4_negative = !memory::TransitionMemory::allows_apply(original, replacement, origin);
        let l4_state_estimate = L4StateEstimator::estimate(L4StateObservation {
            kind: if boundary_changed {
                L4ObservationKind::SpaceBoundary
            } else {
                L4ObservationKind::CandidateApply
            },
            has_active_composition: false,
            boundary_seen: boundary_changed,
            left_context_changed: action.left_context_changed,
            word_count_changed: boundary_changed,
            verifier_passed: action.verifier_passed,
            l4_negative,
        });
        let relation_atoms = TransitionRelationAtoms::encode(
            original,
            replacement,
            TransitionRelationInput {
                action_operator: action.operator.as_str(),
                edit_operator: action.edit_operator.as_str(),
                proof: action.edit_proof.as_str(),
                verifier_passed: action.verifier_passed,
                left_context_changed: action.left_context_changed,
                changed_tokens: action.changed_tokens,
            },
        );

        Self {
            state_before,
            action_operator: action.operator,
            candidate_text: replacement.to_string(),
            state_after_predicted,
            evidence: TransitionEvidence {
                origin,
                source_id: source_id.to_string(),
                error_class,
                verifier_passed: action.verifier_passed,
                left_context_changed: action.left_context_changed,
                changed_tokens: action.changed_tokens,
            },
            l1_signal: L1TransitionSignal {
                boundary_changed,
                word_count_changed: boundary_changed,
            },
            l2_signal: L2TransitionSignal { candidate_count },
            l3_signal: L3TransitionSignal {
                observes_context: matches!(origin, CandidateOrigin::L3Context),
            },
            l4_signed_signal: L4SignedTransitionSignal {
                negative: l4_negative,
            },
            l4_state_estimate,
            relation_atoms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_names_state_and_verifier_evidence() {
        let transition = TypingTransition::from_candidate(
            "провека ",
            "проверка ",
            TypingErrorClass::CompositeTypo,
            CandidateOrigin::DeterministicTypo,
            "composite_ru_typo",
            1,
        );

        assert_eq!(transition.state_before.current_word, "провека");
        assert_eq!(transition.state_after_predicted.current_word, "проверка");
        assert!(transition.evidence.verifier_passed);
        assert!(!transition.evidence.left_context_changed);
        assert!(transition.l4_state_estimate.apply_allowed);
    }
}
