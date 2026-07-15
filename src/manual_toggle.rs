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
    pub recover_missing_initial: bool,
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
            Self::NotHandled => None,
            Self::Handled {
                target_layout_is_ru,
            } => Some(target_layout_is_ru),
        }
    }

    pub fn as_legacy_v2(self) -> (bool, bool) {
        match self {
            Self::NotHandled => (false, false),
            Self::Handled {
                target_layout_is_ru,
            } => (true, target_layout_is_ru),
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

    let (mut backspaces, mut replacement) = if request.recover_missing_initial {
        recover_missing_initial_layout_toggle(token).unwrap_or_else(|| {
            let converted = double_shift_replacement(token);
            (token.chars().count() as u32, converted)
        })
    } else {
        let converted = double_shift_replacement(token);
        (token.chars().count() as u32, converted)
    };

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

pub fn recovered_initial_double_shift_replacement(token: &str) -> Option<String> {
    recover_missing_initial_layout_toggle(token).map(|(_, replacement)| replacement)
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

fn recover_missing_initial_layout_toggle(token: &str) -> Option<(u32, String)> {
    if token.chars().count() < 4 || !token.chars().all(char::is_alphabetic) {
        return None;
    }

    let normal = double_shift_replacement(token);
    let mut best: Option<(f32, String, String)> = None;
    for prefix in missing_initial_prefixes(token) {
        let candidate = format!("{prefix}{token}");
        let replacement = double_shift_replacement(&candidate);
        if replacement.is_empty() || replacement == candidate || replacement == normal {
            continue;
        }
        let score = replacement_quality_score(&replacement);
        if score < 0.98 || !known_layout_recovery_replacement(&replacement) {
            continue;
        }
        if best
            .as_ref()
            .map_or(true, |(best_score, _, _)| score > *best_score)
        {
            best = Some((score, candidate, replacement));
        }
    }

    let (_, _, replacement) = best?;
    Some((token.chars().count() as u32, replacement))
}

fn missing_initial_prefixes(token: &str) -> impl Iterator<Item = char> {
    let ascii = token.chars().all(|ch| ch.is_ascii_alphabetic());
    let prefixes: &'static str = if ascii && token.chars().all(|ch| ch.is_ascii_lowercase()) {
        "abcdefghijklmnopqrstuvwxyz"
    } else if ascii && token.chars().all(|ch| ch.is_ascii_uppercase()) {
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    } else if ascii {
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
    } else {
        "абвгдеёжзийклмнопрстуфхцчшщъыьэюяАБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ"
    };
    prefixes.chars()
}

fn replacement_quality_score(replacement: &str) -> f32 {
    let has_cyrillic = replacement
        .chars()
        .any(|ch| ('а'..='я').contains(&ch) || ('А'..='Я').contains(&ch));
    let has_ascii = replacement.chars().any(|ch| ch.is_ascii_alphabetic());
    if has_cyrillic && !has_ascii {
        crate::quality::score(replacement, "ru")
    } else if has_ascii && !has_cyrillic {
        crate::quality::score(replacement, "en")
    } else {
        0.0
    }
}

fn known_layout_recovery_replacement(replacement: &str) -> bool {
    let word = replacement.trim().to_lowercase();
    !word.is_empty()
        && word
            .chars()
            .all(|ch| ('а'..='я').contains(&ch) || ch == 'ё')
        && crate::layout_autoswitch::is_russian_layout_surface_authority_word(&word)
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
            recover_missing_initial: true,
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
    fn committed_tail_plan_recovers_missing_initial_ascii_layout_letter() {
        let plan = plan_manual_toggle(request("hbdtn")).expect("toggle");

        assert_eq!(plan.backspaces, 5);
        assert_eq!(plan.replacement, "привет");
        assert!(plan.target_layout_is_ru);
    }

    #[test]
    fn committed_tail_plan_recovers_four_letter_tail_missing_initial_layout_letter() {
        let plan = plan_manual_toggle(request("flyj ")).expect("toggle");

        assert_eq!(plan.backspaces, 5);
        assert_eq!(plan.replacement, "ладно ");
        assert!(plan.target_layout_is_ru);
    }

    #[test]
    fn recovered_initial_does_not_delete_separator_before_current_token() {
        let plan = plan_manual_toggle(request("push ltkfq")).expect("toggle");

        assert_eq!(plan.edit.original_token, "ltkfq");
        assert_eq!(plan.backspaces, 5);
        assert_eq!(plan.replacement, "сделай");
        assert!(plan.target_layout_is_ru);
    }

    #[test]
    fn active_composition_plan_does_not_append_separator() {
        let plan = plan_manual_toggle(ManualToggleRequest {
            visible_tail: VisibleTail::ime_active_composition("ghbdtn"),
            current_layout_is_ru: false,
            recover_missing_initial: false,
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
            recover_missing_initial: false,
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
            recover_missing_initial: false,
            preserve_trailing_whitespace: false,
        })
        .expect("toggle to ru");

        assert_eq!(to_ru.edit.insert_text, "допустим");
        assert!(to_ru.target_layout_is_ru);

        let to_en = plan_manual_toggle(ManualToggleRequest {
            visible_tail: VisibleTail::daemon_word_buffer("допустим"),
            current_layout_is_ru: true,
            recover_missing_initial: false,
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

        let handled = ImeManualToggleOutcome::handled(true);
        assert_eq!(handled.target_layout_is_ru(), Some(true));
        assert_eq!(handled.as_legacy_v2(), (true, true));
    }
}
