use crate::dict::{convert, detect_direction};
use crate::keyboard::preferred_layout_for_text;
pub use crate::text_edit::{VisibleTail, VisibleTailSource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualToggleRoute {
    Daemon,
    ImeActiveComposition,
    ImeCommittedTail,
}

impl VisibleTailSource {
    pub fn route(self) -> ManualToggleRoute {
        match self {
            Self::DaemonWordBuffer => ManualToggleRoute::Daemon,
            Self::ImeActiveComposition => ManualToggleRoute::ImeActiveComposition,
            Self::ImeCommittedTail => ManualToggleRoute::ImeCommittedTail,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualToggleRequest<'a> {
    pub visible_tail: VisibleTail<'a>,
    pub current_layout_is_ru: bool,
    pub preserve_trailing_whitespace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualToggleEditPlan {
    pub source: VisibleTailSource,
    pub original_tail: String,
    pub original_token: String,
    pub delete_chars: u32,
    pub insert_text: String,
    pub target_layout_is_ru: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualTogglePlan {
    pub route: ManualToggleRoute,
    pub backspaces: u32,
    pub replacement: String,
    pub target_layout_is_ru: bool,
    pub suppress_next_autocorrect: bool,
    pub edit: ManualToggleEditPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImeManualToggleOutcome {
    NotHandled,
    DelegateDaemon,
    Handled { target_layout_is_ru: bool },
}

impl ImeManualToggleOutcome {
    pub fn handled(target_layout_is_ru: bool) -> Self {
        Self::Handled {
            target_layout_is_ru,
        }
    }

    pub fn target_layout_is_ru(self) -> Option<bool> {
        match self {
            Self::NotHandled | Self::DelegateDaemon => None,
            Self::Handled {
                target_layout_is_ru,
            } => Some(target_layout_is_ru),
        }
    }

    pub fn as_legacy_v2(self) -> (bool, bool) {
        match self {
            Self::NotHandled | Self::DelegateDaemon => (false, false),
            Self::Handled {
                target_layout_is_ru,
            } => (true, target_layout_is_ru),
        }
    }

    pub fn as_v3(self) -> (u8, bool) {
        match self {
            Self::NotHandled => (0, false),
            Self::Handled {
                target_layout_is_ru,
            } => (1, target_layout_is_ru),
            Self::DelegateDaemon => (2, false),
        }
    }

    pub fn from_v3(status: u8, target_layout_is_ru: bool) -> Result<Self, &'static str> {
        match (status, target_layout_is_ru) {
            (0, false) => Ok(Self::NotHandled),
            (1, target_layout_is_ru) => Ok(Self::handled(target_layout_is_ru)),
            (2, false) => Ok(Self::DelegateDaemon),
            _ => Err("invalid ManualToggleV3 outcome"),
        }
    }
}

pub fn plan_manual_toggle(request: ManualToggleRequest<'_>) -> Option<ManualTogglePlan> {
    let tail = request.visible_tail.text;
    let token = last_tail_token(tail)?;
    let trailing_ws = if request.preserve_trailing_whitespace {
        crate::word_reader::trailing_whitespace_char_count(tail)
    } else {
        0
    };

    let mut backspaces = token.chars().count() as u32;
    let mut replacement = double_shift_replacement(token);

    if replacement == token {
        return None;
    }
    let original_token = token.to_string();
    for _ in 0..trailing_ws {
        replacement.push(' ');
    }
    backspaces = backspaces.saturating_add(trailing_ws as u32);
    let target_layout_is_ru = preferred_layout_for_text(&replacement, request.current_layout_is_ru);

    Some(ManualTogglePlan {
        route: request.visible_tail.source.route(),
        backspaces,
        replacement: replacement.clone(),
        target_layout_is_ru,
        suppress_next_autocorrect: true,
        edit: ManualToggleEditPlan {
            source: request.visible_tail.source,
            original_tail: tail.to_string(),
            original_token,
            delete_chars: backspaces,
            insert_text: replacement,
            target_layout_is_ru,
        },
    })
}

pub fn double_shift_replacement(text: &str) -> String {
    crate::mixed_script_repair::repair_mixed_script(text)
        .unwrap_or_else(|| convert(text, detect_direction(text)))
}

fn last_tail_token(tail: &str) -> Option<&str> {
    let end = tail
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| (!ch.is_whitespace()).then_some(idx + ch.len_utf8()))?;
    let start = tail[..end]
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| ch.is_whitespace().then_some(idx + ch.len_utf8()))
        .unwrap_or(0);
    Some(&tail[start..end])
}

#[cfg(test)]
mod tests {
    use super::{
        plan_manual_toggle, ImeManualToggleOutcome, ManualToggleRequest, ManualToggleRoute,
        VisibleTail, VisibleTailSource,
    };

    fn request(tail: &str) -> ManualToggleRequest<'_> {
        ManualToggleRequest {
            visible_tail: VisibleTail::ime_committed_tail(tail),
            current_layout_is_ru: false,
            preserve_trailing_whitespace: true,
        }
    }

    #[test]
    fn committed_tail_plan_preserves_separator() {
        let plan = plan_manual_toggle(request("работает ")).expect("toggle");

        assert_eq!(plan.backspaces, 9);
        assert_eq!(plan.replacement, "hf,jnftn ");
        assert_eq!(plan.route, ManualToggleRoute::ImeCommittedTail);
        assert_eq!(plan.edit.source, VisibleTailSource::ImeCommittedTail);
        assert_eq!(plan.edit.original_tail, "работает ");
        assert_eq!(plan.edit.original_token, "работает");
        assert_eq!(plan.edit.delete_chars, 9);
        assert_eq!(plan.edit.insert_text, "hf,jnftn ");
        assert!(!plan.target_layout_is_ru);
        assert!(plan.suppress_next_autocorrect);
    }

    #[test]
    fn committed_tail_plan_is_exact_layout_projection() {
        let plan = plan_manual_toggle(request("hbdtn")).expect("toggle");

        assert_eq!(plan.backspaces, 5);
        assert_eq!(plan.replacement, "ривет");
        assert!(plan.target_layout_is_ru);
    }

    #[test]
    fn committed_tail_plan_keeps_internal_layout_symbol_in_the_token() {
        let plan = plan_manual_toggle(request("ye;ty")).expect("toggle");

        assert_eq!(plan.edit.original_token, "ye;ty");
        assert_eq!(plan.backspaces, 5);
        assert_eq!(plan.replacement, "нужен");
        assert!(plan.target_layout_is_ru);
    }

    #[test]
    fn committed_tail_plan_does_not_invent_missing_initial_letter() {
        let plan = plan_manual_toggle(request("flyj ")).expect("toggle");

        assert_eq!(plan.backspaces, 5);
        assert_eq!(plan.replacement, "адно ");
        assert!(plan.target_layout_is_ru);
    }

    #[test]
    fn exact_projection_does_not_delete_separator_before_current_token() {
        let plan = plan_manual_toggle(request("push ltkfq")).expect("toggle");

        assert_eq!(plan.edit.original_token, "ltkfq");
        assert_eq!(plan.backspaces, 5);
        assert_eq!(plan.replacement, "делай");
        assert!(plan.target_layout_is_ru);
    }

    #[test]
    fn active_composition_plan_does_not_append_separator() {
        let plan = plan_manual_toggle(ManualToggleRequest {
            visible_tail: VisibleTail::ime_active_composition("ghbdtn"),
            current_layout_is_ru: false,
            preserve_trailing_whitespace: false,
        })
        .expect("toggle");

        assert_eq!(plan.backspaces, 6);
        assert_eq!(plan.replacement, "привет");
        assert_eq!(plan.route, ManualToggleRoute::ImeActiveComposition);
        assert_eq!(plan.edit.source, VisibleTailSource::ImeActiveComposition);
        assert_eq!(plan.edit.delete_chars, 6);
        assert_eq!(plan.edit.insert_text, "привет");
    }

    #[test]
    fn daemon_word_buffer_plan_marks_daemon_source() {
        let plan = plan_manual_toggle(ManualToggleRequest {
            visible_tail: VisibleTail::daemon_word_buffer("ghbdtn"),
            current_layout_is_ru: false,
            preserve_trailing_whitespace: false,
        })
        .expect("toggle");

        assert_eq!(plan.edit.source, VisibleTailSource::DaemonWordBuffer);
        assert_eq!(plan.route, ManualToggleRoute::Daemon);
        assert_eq!(plan.edit.original_token, "ghbdtn");
        assert_eq!(plan.edit.delete_chars, 6);
        assert_eq!(plan.edit.insert_text, "привет");
    }

    #[test]
    fn manual_toggle_keeps_l2_surface_layout_word_bidirectional() {
        let to_ru = plan_manual_toggle(ManualToggleRequest {
            visible_tail: VisibleTail::daemon_word_buffer("ljgecnbv"),
            current_layout_is_ru: false,
            preserve_trailing_whitespace: false,
        })
        .expect("toggle to ru");

        assert_eq!(to_ru.edit.insert_text, "допустим");
        assert!(to_ru.target_layout_is_ru);

        let to_en = plan_manual_toggle(ManualToggleRequest {
            visible_tail: VisibleTail::daemon_word_buffer("допустим"),
            current_layout_is_ru: true,
            preserve_trailing_whitespace: false,
        })
        .expect("toggle to en");

        assert_eq!(to_en.edit.insert_text, "ljgecnbv");
        assert!(!to_en.target_layout_is_ru);
    }

    #[test]
    fn ime_manual_toggle_outcome_keeps_legacy_wire_format_at_the_boundary() {
        assert_eq!(
            ImeManualToggleOutcome::NotHandled.target_layout_is_ru(),
            None
        );
        assert_eq!(
            ImeManualToggleOutcome::NotHandled.as_legacy_v2(),
            (false, false)
        );
        assert_eq!(
            ImeManualToggleOutcome::DelegateDaemon.as_legacy_v2(),
            (false, false)
        );

        let handled = ImeManualToggleOutcome::handled(true);
        assert_eq!(handled.target_layout_is_ru(), Some(true));
        assert_eq!(handled.as_legacy_v2(), (true, true));
    }

    #[test]
    fn ime_manual_toggle_v3_keeps_delegation_distinct_and_rejects_malformed_status() {
        for outcome in [
            ImeManualToggleOutcome::NotHandled,
            ImeManualToggleOutcome::DelegateDaemon,
            ImeManualToggleOutcome::handled(false),
            ImeManualToggleOutcome::handled(true),
        ] {
            let wire = outcome.as_v3();
            assert_eq!(ImeManualToggleOutcome::from_v3(wire.0, wire.1), Ok(outcome));
        }

        assert!(ImeManualToggleOutcome::from_v3(0, true).is_err());
        assert!(ImeManualToggleOutcome::from_v3(2, true).is_err());
        assert!(ImeManualToggleOutcome::from_v3(3, false).is_err());
    }
}
