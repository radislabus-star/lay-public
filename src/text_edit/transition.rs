use super::action::EditAction;
use super::diff_plan::tail_chars;
use super::gate::authorize_replacement_with_transition;
use super::mutation::TransitionAudit;
use super::types::TextReplacement;
use super::visible_tail::{VisibleTailSnapshot, VisibleTailSource};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleFieldState {
    visible_tail: String,
    focus_id: Option<String>,
    external_state_present: bool,
    external_tail_before_cursor: Option<String>,
    external_selection_active: bool,
}

impl VisibleFieldState {
    pub fn committed_tail(visible_tail: impl Into<String>, focus_id: Option<String>) -> Self {
        Self {
            visible_tail: visible_tail.into(),
            focus_id,
            external_state_present: false,
            external_tail_before_cursor: None,
            external_selection_active: false,
        }
    }

    pub fn with_external_tail_before_cursor(
        mut self,
        external_tail_before_cursor: Option<String>,
        external_selection_active: bool,
    ) -> Self {
        self.external_state_present = true;
        self.external_tail_before_cursor = external_tail_before_cursor;
        self.external_selection_active = external_selection_active;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextTransitionIntent {
    ImeAutocorrect,
    ImeManualToggle,
    DaemonBridge,
}

impl TextTransitionIntent {
    fn operator(self) -> &'static str {
        match self {
            Self::ImeAutocorrect => "ime_committed_tail_autocorrect",
            Self::ImeManualToggle => "ime_committed_tail_manual_toggle",
            Self::DaemonBridge => "daemon_visible_tail_bridge",
        }
    }

    fn proof(self) -> &'static str {
        "visible_field_state_and_edit_plan_checked"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatentTextTransitionCandidate {
    source: VisibleTailSource,
    delete_chars: u32,
    insert_text: String,
    intent: TextTransitionIntent,
    expected_tail: Option<VisibleTailSnapshot>,
}

impl LatentTextTransitionCandidate {
    pub fn new(
        source: VisibleTailSource,
        delete_chars: u32,
        insert_text: impl Into<String>,
        intent: TextTransitionIntent,
        expected_tail: Option<VisibleTailSnapshot>,
    ) -> Self {
        Self {
            source,
            delete_chars,
            insert_text: insert_text.into(),
            intent,
            expected_tail,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextTransitionDecision {
    Apply {
        plan: TextReplacement,
        action: EditAction,
    },
    Reject {
        rejection: TextTransitionRejection,
        action: Option<EditAction>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextTransitionRejection {
    Noop,
    StaleVisibleTail { expected: String, actual: String },
    StaleSurroundingText { expected: String, actual: String },
    UnsafeEdit { reason: &'static str },
}

impl TextTransitionRejection {
    pub fn reason(&self) -> &'static str {
        match self {
            Self::Noop => "noop_transition",
            Self::StaleVisibleTail { .. } => "stale_visible_tail",
            Self::StaleSurroundingText { .. } => "stale_surrounding_text",
            Self::UnsafeEdit { reason } => reason,
        }
    }

    pub fn expected(&self) -> &str {
        match self {
            Self::Noop | Self::UnsafeEdit { .. } => "",
            Self::StaleVisibleTail { expected, .. }
            | Self::StaleSurroundingText { expected, .. } => expected,
        }
    }

    pub fn actual(&self) -> &str {
        match self {
            Self::Noop | Self::UnsafeEdit { .. } => "",
            Self::StaleVisibleTail { actual, .. } | Self::StaleSurroundingText { actual, .. } => {
                actual
            }
        }
    }
}

pub fn decide_text_transition(
    state: &VisibleFieldState,
    candidate: LatentTextTransitionCandidate,
) -> TextTransitionDecision {
    if candidate.delete_chars == 0 && candidate.insert_text.is_empty() {
        return TextTransitionDecision::Reject {
            rejection: TextTransitionRejection::Noop,
            action: None,
        };
    }

    let original_text = tail_chars(&state.visible_tail, candidate.delete_chars as usize);
    if let Some(expected) = candidate.expected_tail.as_ref() {
        let focus_id = state.focus_id.as_deref();
        if !expected.matches_source_and_focus(candidate.source, focus_id)
            || !expected
                .matches_current_suffix(&state.visible_tail, candidate.delete_chars as usize)
        {
            return TextTransitionDecision::Reject {
                rejection: TextTransitionRejection::StaleVisibleTail {
                    expected: expected.expected_suffix.clone(),
                    actual: original_text,
                },
                action: None,
            };
        }
    }

    if state.external_selection_active {
        return TextTransitionDecision::Reject {
            rejection: TextTransitionRejection::StaleSurroundingText {
                expected: original_text,
                actual: state
                    .external_tail_before_cursor
                    .clone()
                    .unwrap_or_default(),
            },
            action: None,
        };
    }

    if state.external_state_present {
        let actual = state
            .external_tail_before_cursor
            .clone()
            .unwrap_or_default();
        if actual != original_text {
            return TextTransitionDecision::Reject {
                rejection: TextTransitionRejection::StaleSurroundingText {
                    expected: original_text,
                    actual,
                },
                action: None,
            };
        }
    }

    let plan = TextReplacement {
        move_left: 0,
        backspaces: candidate.delete_chars,
        insert: candidate.insert_text.clone(),
        move_right: 0,
    };
    let transition = TransitionAudit::proven(
        candidate.intent.operator(),
        candidate.intent.proof(),
        true,
        false,
        1,
    );
    let action = authorize_replacement_with_transition(
        "ibus-committed-tail",
        1000,
        &original_text,
        &candidate.insert_text,
        plan.clone(),
        Some(candidate.source.source_id()),
        None,
        transition,
    );
    if !action.allow_apply() {
        return TextTransitionDecision::Reject {
            rejection: TextTransitionRejection::UnsafeEdit {
                reason: action.safety_reason(),
            },
            action: Some(action),
        };
    }

    TextTransitionDecision::Apply { plan, action }
}

#[cfg(test)]
mod tests {
    use super::{
        decide_text_transition, LatentTextTransitionCandidate, TextTransitionDecision,
        TextTransitionIntent, TextTransitionRejection, VisibleFieldState,
    };
    use crate::text_edit::{EditActionKind, VisibleTailSnapshot, VisibleTailSource};

    fn candidate(delete_chars: u32, insert_text: &str) -> LatentTextTransitionCandidate {
        LatentTextTransitionCandidate::new(
            VisibleTailSource::ImeCommittedTail,
            delete_chars,
            insert_text,
            TextTransitionIntent::ImeAutocorrect,
            None,
        )
    }

    #[test]
    fn text_transition_applies_matching_visible_and_surrounding_tail() {
        let state = VisibleFieldState::committed_tail("abc ghbdtn", Some("/test".to_string()))
            .with_external_tail_before_cursor(Some("bdtn".to_string()), false);

        let decision = decide_text_transition(&state, candidate(4, "вет"));

        match decision {
            TextTransitionDecision::Apply { plan, action } => {
                assert_eq!(plan.backspaces, 4);
                assert_eq!(plan.insert, "вет");
                assert_eq!(action.kind, EditActionKind::ReplaceLastToken);
                assert_eq!(
                    action.transition.operator.as_deref(),
                    Some("ime_committed_tail_autocorrect")
                );
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[test]
    fn text_transition_rejects_stale_visible_tail_snapshot() {
        let state = VisibleFieldState::committed_tail("abc ghjdt", Some("/test".to_string()));
        let mut request = candidate(4, "вет");
        request.expected_tail = Some(VisibleTailSnapshot::new(
            VisibleTailSource::ImeCommittedTail,
            "bdtn",
            Some("/test".to_string()),
            0,
        ));

        let decision = decide_text_transition(&state, request);

        match decision {
            TextTransitionDecision::Reject { rejection, action } => {
                assert_eq!(action, None);
                assert_eq!(rejection.reason(), "stale_visible_tail");
                assert_eq!(rejection.expected(), "bdtn");
                assert_eq!(rejection.actual(), "hjdt");
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[test]
    fn text_transition_rejects_stale_surrounding_text() {
        let state = VisibleFieldState::committed_tail("abc ghbdtn", Some("/test".to_string()))
            .with_external_tail_before_cursor(Some("hjdt".to_string()), false);

        let decision = decide_text_transition(&state, candidate(4, "вет"));

        match decision {
            TextTransitionDecision::Reject { rejection, action } => {
                assert_eq!(action, None);
                assert_eq!(rejection.reason(), "stale_surrounding_text");
                assert_eq!(rejection.expected(), "bdtn");
                assert_eq!(rejection.actual(), "hjdt");
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[test]
    fn text_transition_rejects_active_external_selection() {
        let state = VisibleFieldState::committed_tail("abc ghbdtn", Some("/test".to_string()))
            .with_external_tail_before_cursor(Some("bdtn".to_string()), true);

        let decision = decide_text_transition(&state, candidate(4, "вет"));

        match decision {
            TextTransitionDecision::Reject { rejection, action } => {
                assert_eq!(action, None);
                assert_eq!(rejection.reason(), "stale_surrounding_text");
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[test]
    fn text_transition_rejects_present_but_unreadable_surrounding_text() {
        let state = VisibleFieldState::committed_tail("abc ghbdtn", Some("/test".to_string()))
            .with_external_tail_before_cursor(None, false);

        let decision = decide_text_transition(&state, candidate(4, "вет"));

        match decision {
            TextTransitionDecision::Reject { rejection, action } => {
                assert_eq!(action, None);
                assert_eq!(rejection.reason(), "stale_surrounding_text");
                assert_eq!(rejection.expected(), "bdtn");
                assert_eq!(rejection.actual(), "");
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[test]
    fn text_transition_blocks_unsafe_multiword_edit() {
        let state = VisibleFieldState::committed_tail("одно два", Some("/test".to_string()));

        let decision = decide_text_transition(&state, candidate(8, "однотри"));

        match decision {
            TextTransitionDecision::Reject { rejection, action } => {
                assert_eq!(
                    rejection,
                    TextTransitionRejection::UnsafeEdit {
                        reason: "unsafe_multiword_autocorrect_scope"
                    }
                );
                assert_eq!(
                    action.expect("edit action").kind,
                    EditActionKind::BlockUnsafe
                );
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }
}
