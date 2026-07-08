//! Typing Transition CPU.
//!
//! Candidate producers may propose text, but this module owns the transition
//! shape, verifier contract, and final apply authority.

pub(crate) mod action;
pub(crate) mod candidate;
pub(crate) mod decision;
pub(crate) mod executor_contract;
pub(crate) mod memory;
pub(crate) mod state;
pub(crate) mod verifier;

use crate::correction_core::TypingErrorClass;
use crate::language_action::LanguageActionOperator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypingTransition {
    pub(crate) state_before: TypingState,
    pub(crate) action_operator: LanguageActionOperator,
    pub(crate) candidate_text: String,
    pub(crate) state_after_predicted: TypingState,
    pub(crate) evidence: TransitionEvidence,
    pub(crate) l1_signal: L1TransitionSignal,
    pub(crate) l2_signal: L2TransitionSignal,
    pub(crate) l3_signal: L3TransitionSignal,
    pub(crate) l4_signed_signal: L4SignedTransitionSignal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypingState {
    pub(crate) text: String,
    pub(crate) current_word: String,
    pub(crate) word_count: usize,
}

impl TypingState {
    pub(crate) fn from_text(text: &str) -> Self {
        let current_word = crate::word_reader::last_text_word(text).unwrap_or_default();
        Self {
            text: text.to_string(),
            current_word,
            word_count: text.split_whitespace().count(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransitionEvidence {
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
        source_id: &str,
        candidate_count: usize,
    ) -> Self {
        let action = action::verify_action_operator(original, replacement, error_class, source_id);
        let state_before = TypingState::from_text(original);
        let state_after_predicted = TypingState::from_text(replacement);

        Self {
            state_before,
            action_operator: action.operator,
            candidate_text: replacement.to_string(),
            state_after_predicted,
            evidence: TransitionEvidence {
                source_id: source_id.to_string(),
                error_class,
                verifier_passed: action.verifier_passed,
                left_context_changed: action.left_context_changed,
                changed_tokens: action.changed_tokens,
            },
            l1_signal: L1TransitionSignal {
                boundary_changed: original.split_whitespace().count()
                    != replacement.split_whitespace().count(),
                word_count_changed: original.split_whitespace().count()
                    != replacement.split_whitespace().count(),
            },
            l2_signal: L2TransitionSignal { candidate_count },
            l3_signal: L3TransitionSignal {
                observes_context: crate::correction_source_contract::is_l3_context_source(
                    source_id,
                ),
            },
            l4_signed_signal: L4SignedTransitionSignal {
                negative: !memory::TransitionMemory::allows_apply(original, replacement, source_id),
            },
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
            "composite_ru_typo",
            1,
        );

        assert_eq!(transition.state_before.current_word, "провека");
        assert_eq!(transition.state_after_predicted.current_word, "проверка");
        assert!(transition.evidence.verifier_passed);
        assert!(!transition.evidence.left_context_changed);
    }
}
