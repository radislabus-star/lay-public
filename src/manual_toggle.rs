use crate::dict::{convert, detect_direction};
use crate::keyboard::preferred_layout_for_text;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualToggleRoute {
    Daemon,
    ImeActiveComposition,
    ImeCommittedTail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualToggleRequest<'a> {
    pub tail: &'a str,
    pub current_layout_is_ru: bool,
    pub route: ManualToggleRoute,
    pub recover_missing_initial: bool,
    pub preserve_trailing_whitespace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualTogglePlan {
    pub backspaces: u32,
    pub replacement: String,
    pub target_layout_is_ru: bool,
    pub suppress_next_autocorrect: bool,
}

pub fn plan_manual_toggle(request: ManualToggleRequest<'_>) -> Option<ManualTogglePlan> {
    let token = last_tail_token(request.tail)?;
    let trailing_ws = if request.preserve_trailing_whitespace {
        trailing_whitespace_chars(request.tail)
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
    for _ in 0..trailing_ws {
        replacement.push(' ');
    }
    backspaces = backspaces.saturating_add(trailing_ws as u32);

    Some(ManualTogglePlan {
        backspaces,
        target_layout_is_ru: preferred_layout_for_text(&replacement, request.current_layout_is_ru),
        replacement,
        suppress_next_autocorrect: true,
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

fn trailing_whitespace_chars(text: &str) -> usize {
    text.chars()
        .rev()
        .take_while(|ch| ch.is_whitespace())
        .count()
}

fn recover_missing_initial_layout_toggle(token: &str) -> Option<(u32, String)> {
    if token.chars().count() < 5 || !token.chars().all(char::is_alphabetic) {
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

    let (_, candidate, replacement) = best?;
    Some((candidate.chars().count() as u32, replacement))
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
        && (crate::lexicon::is_common_ru_word(&word)
            || crate::russian_lexicon::is_known_russian_word_or_form(&word))
}

#[cfg(test)]
mod tests {
    use super::{plan_manual_toggle, ManualToggleRequest, ManualToggleRoute};

    fn request(tail: &str) -> ManualToggleRequest<'_> {
        ManualToggleRequest {
            tail,
            current_layout_is_ru: false,
            route: ManualToggleRoute::ImeCommittedTail,
            recover_missing_initial: true,
            preserve_trailing_whitespace: true,
        }
    }

    #[test]
    fn committed_tail_plan_preserves_separator() {
        let plan = plan_manual_toggle(request("работает ")).expect("toggle");

        assert_eq!(plan.backspaces, 9);
        assert_eq!(plan.replacement, "hf,jnftn ");
        assert!(!plan.target_layout_is_ru);
        assert!(plan.suppress_next_autocorrect);
    }

    #[test]
    fn committed_tail_plan_recovers_missing_initial_ascii_layout_letter() {
        let plan = plan_manual_toggle(request("hbdtn")).expect("toggle");

        assert_eq!(plan.backspaces, 6);
        assert_eq!(plan.replacement, "привет");
        assert!(plan.target_layout_is_ru);
    }

    #[test]
    fn active_composition_plan_does_not_append_separator() {
        let plan = plan_manual_toggle(ManualToggleRequest {
            tail: "ghbdtn",
            current_layout_is_ru: false,
            route: ManualToggleRoute::ImeActiveComposition,
            recover_missing_initial: false,
            preserve_trailing_whitespace: false,
        })
        .expect("toggle");

        assert_eq!(plan.backspaces, 6);
        assert_eq!(plan.replacement, "привет");
    }
}
