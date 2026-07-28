use super::action::DecisionTransitionEditInput;
use super::gate::{plan_decision_transition_edit, plan_recorded_undo_edit};
use super::mutation::TransitionAudit;
use super::transition::{
    LatentTextTransitionCandidate, TextTransitionDecision, TextTransitionRejection,
    VisibleFieldState,
};
use super::types::TextReplacement;
use crate::text_edit::tail_chars;
use crate::typing_transition::decision::DecisionTransitionReceipt;

/// Independent verifier for an already-selected text transition.
///
/// This module never generates or ranks candidates. It verifies that the
/// winner is still applicable to the exact visible snapshot and produces the
/// minimal physical edit plan. Candidate competition stays in
/// `TransitionDecisionCore`.
pub(crate) fn verify_visible_text_transition(
    state: &VisibleFieldState,
    candidate: LatentTextTransitionCandidate,
) -> TextTransitionDecision {
    if candidate.delete_chars == 0 && candidate.insert_text.is_empty() {
        return TextTransitionDecision::Reject {
            rejection: TextTransitionRejection::Noop,
            action: None,
        };
    }

    if !candidate.insert_text.is_empty() && state.visible_tail.ends_with(&candidate.insert_text) {
        return TextTransitionDecision::AlreadyApplied;
    }

    let original_text = tail_chars(&state.visible_tail, candidate.delete_chars as usize);
    if let Some(expected) = candidate.expected_tail.as_ref() {
        let focus_id = state.focus_id.as_deref();
        if expected.source != candidate.source || expected.focus_id.as_deref() != focus_id {
            return stale_tail(expected.expected_suffix.clone(), original_text);
        }
        if expected.epoch != state.epoch {
            return TextTransitionDecision::Reject {
                rejection: TextTransitionRejection::StaleVisibleRevision {
                    expected: expected.epoch,
                    actual: state.epoch,
                },
                action: None,
            };
        }
        if !expected.matches_current_suffix(&state.visible_tail, candidate.delete_chars as usize) {
            return stale_tail(expected.expected_suffix.clone(), original_text);
        }
    }

    if state.external_selection_active {
        return stale_surrounding(
            original_text,
            state
                .external_tail_before_cursor
                .clone()
                .unwrap_or_default(),
        );
    }

    if state.external_state_present {
        let actual = state
            .external_tail_before_cursor
            .clone()
            .unwrap_or_default();
        if actual != original_text {
            return stale_surrounding(original_text, actual);
        }
    }

    let plan = TextReplacement {
        move_left: 0,
        backspaces: candidate.delete_chars,
        insert: candidate.insert_text.clone(),
        move_right: 0,
    };
    let receipt = DecisionTransitionReceipt::for_visible_tail(
        original_text.clone(),
        candidate.insert_text.clone(),
        TransitionAudit::proven(
            candidate.intent.operator(),
            candidate.intent.proof(),
            true,
            false,
            1,
        ),
    );
    // PROTECTED USER CONTRACT: exact autocorrect rollback is recorded user
    // intent, never an automatic correction decision.
    let action = if candidate.intent == super::transition::TextTransitionIntent::ImeAutoUndo {
        plan_recorded_undo_edit(&original_text, &candidate.insert_text, plan.clone(), 1)
    } else {
        plan_decision_transition_edit(
            DecisionTransitionEditInput {
                source: "ibus-committed-tail",
                confidence_milli: 1000,
                from_text: &original_text,
                to_text: &candidate.insert_text,
                plan: plan.clone(),
                selected_source_id: Some(candidate.source.source_id()),
                selected_error_class: None,
            },
            &receipt,
        )
    };
    if !action.allow_apply() {
        let reason = action
            .execution_rejection_reason()
            .unwrap_or_else(|| action.safety_reason());
        return TextTransitionDecision::Reject {
            rejection: TextTransitionRejection::UnsafeEdit { reason },
            action: Some(action),
        };
    }

    TextTransitionDecision::Apply { plan, action }
}

fn stale_tail(expected: String, actual: String) -> TextTransitionDecision {
    TextTransitionDecision::Reject {
        rejection: TextTransitionRejection::StaleVisibleTail { expected, actual },
        action: None,
    }
}

fn stale_surrounding(expected: String, actual: String) -> TextTransitionDecision {
    TextTransitionDecision::Reject {
        rejection: TextTransitionRejection::StaleSurroundingText { expected, actual },
        action: None,
    }
}
