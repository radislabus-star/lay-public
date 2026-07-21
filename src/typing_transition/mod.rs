//! Typing Transition CPU.
//!
//! Candidate producers may propose text, but this module owns the transition
//! shape, verifier contract, and final apply authority.

pub(crate) mod action;
pub(crate) mod candidate;
pub(crate) mod decision;
pub(crate) mod executor_contract;
pub(crate) mod live_candidate;
pub(crate) mod proposal_admission;
pub(crate) mod state;
pub(crate) mod verifier;

use crate::candidate_contract::CandidateOrigin;
use crate::correction_core::TypingErrorClass;
use crate::language_action::{LanguageActionOperator, LanguageActionProof};
use crate::text_edit::TransitionOperator;
use crate::transition_relation::{TransitionRelationAtoms, TransitionRelationInput};
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
    relation_atoms: TransitionRelationAtoms,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransitionEvidence {
    pub(crate) origin: CandidateOrigin,
    /// Diagnostic provenance only. Decision and verifier authority use `origin`.
    pub(crate) source_id: String,
    pub(crate) error_class: TypingErrorClass,
    pub(crate) edit_operator: TransitionOperator,
    pub(crate) edit_proof: LanguageActionProof,
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
    pub(crate) state_specific: bool,
    pub(crate) attract_count: u32,
    pub(crate) repel_count: u32,
}

impl L4SignedTransitionSignal {
    pub(crate) const fn exact_positive(self) -> bool {
        self.state_specific && self.attract_count > self.repel_count
    }
}

pub(crate) struct EvaluatedTransitionInput<'a> {
    pub(crate) original: &'a str,
    pub(crate) replacement: &'a str,
    pub(crate) error_class: TypingErrorClass,
    pub(crate) origin: CandidateOrigin,
    pub(crate) source_id: &'a str,
    pub(crate) candidate_count: usize,
    pub(crate) action: action::CorrectionActionOperatorReport,
    pub(crate) l4_signed_signal: L4SignedTransitionSignal,
}

impl TypingTransition {
    pub(crate) fn from_evaluated_candidate(input: EvaluatedTransitionInput<'_>) -> Self {
        let EvaluatedTransitionInput {
            original,
            replacement,
            error_class,
            origin,
            source_id,
            candidate_count,
            action,
            l4_signed_signal,
        } = input;
        let state_before = LatentTypingState::from_text(original);
        let state_after_predicted = LatentTypingState::from_text(replacement);
        let boundary_changed = state_before.word_count_changed(&state_after_predicted);
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
                edit_operator: action.edit_operator,
                edit_proof: action.edit_proof,
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
            l4_signed_signal,
            relation_atoms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_names_state_and_verifier_evidence() {
        let action = action::verify_action_operator(
            "провека ",
            "проверка ",
            TypingErrorClass::CompositeTypo,
            CandidateOrigin::DeterministicTypo,
        );
        let transition = TypingTransition::from_evaluated_candidate(EvaluatedTransitionInput {
            original: "провека ",
            replacement: "проверка ",
            error_class: TypingErrorClass::CompositeTypo,
            origin: CandidateOrigin::DeterministicTypo,
            source_id: "composite_ru_typo",
            candidate_count: 1,
            action,
            l4_signed_signal: L4SignedTransitionSignal {
                negative: false,
                state_specific: false,
                attract_count: 0,
                repel_count: 0,
            },
        });

        assert_eq!(transition.state_before.current_word, "провека");
        assert_eq!(transition.state_after_predicted.current_word, "проверка");
        assert!(transition.evidence.verifier_passed);
        assert!(!transition.evidence.left_context_changed);
        assert!(!transition.l4_signed_signal.negative);
    }
}
