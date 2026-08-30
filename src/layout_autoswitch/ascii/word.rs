use crate::lexicon::{is_common_ru_word, is_user_protected_ascii_word};
use crate::russian_lexicon::russian_tiny_dictionary;
use crate::word_reader::split_word_punctuation;

use super::super::english::is_known_english_layout_autoswitch_word;
use super::candidate::{
    ascii_to_russian_layout_candidate, exact_ascii_to_russian_layout_candidate,
};
use super::punctuation::correct_word_preserving_trailing_punctuation;
use super::symbols::{
    has_ascii_shift_letter_signal, is_blocked_ascii_layout_token, is_protected_ascii_layout_token,
};

pub(crate) fn correct_confident_wrong_layout_ascii_word(token: &str) -> Option<String> {
    if has_structured_ascii_identity(token) && is_protected_ascii_layout_token(token) {
        return None;
    }
    let (_, original_word, _) = split_word_punctuation(token);
    if is_user_protected_ascii_word(original_word) {
        return None;
    }

    let original_alpha_len = original_word
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .count();
    if original_alpha_len < 3 {
        return None;
    }

    if let Some(replacement) = correct_word_preserving_trailing_punctuation(token) {
        return Some(replacement);
    }

    let candidate = ascii_to_russian_layout_candidate(token, false)?;
    if !candidate.known || !candidate.raw_projection_stable {
        return None;
    }
    if is_protected_ascii_layout_token(token)
        && is_known_english_layout_autoswitch_word(&original_word.to_ascii_lowercase())
    {
        return None;
    }
    if candidate.clean_alpha {
        return None;
    }
    Some(candidate.replacement)
}

fn has_structured_ascii_identity(token: &str) -> bool {
    token.chars().any(|ch| matches!(ch, '.' | '/' | '\\' | '@'))
}

pub(crate) fn correct_wrong_layout_ascii_word(token: &str) -> Option<String> {
    if is_blocked_ascii_layout_token(token) {
        return None;
    }

    let (_, original_word, _) = split_word_punctuation(token);
    if original_word.is_empty() {
        return None;
    }
    if original_word
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .count()
        < 2
    {
        return None;
    }
    if is_user_protected_ascii_word(original_word) {
        return None;
    }

    if let Some(replacement) = correct_word_preserving_trailing_punctuation(token) {
        return Some(replacement);
    }

    let strong_shift_layout = is_standalone_all_caps_shift_layout_token(token);
    let candidate = ascii_to_russian_layout_candidate(token, strong_shift_layout)?;
    if !candidate.raw_projection_stable && !strong_shift_layout {
        return None;
    }
    let normalized = candidate.replacement;
    if strong_shift_layout {
        return Some(normalized);
    }
    if is_protected_ascii_layout_token(token)
        && is_known_english_layout_autoswitch_word(&original_word.to_ascii_lowercase())
    {
        return None;
    }

    if is_protected_ascii_layout_token(token) {
        return None;
    }
    if allow_short_layout_word(original_word, &candidate.word.to_lowercase())
        || is_common_ru_word(&candidate.word.to_lowercase())
    {
        Some(normalized)
    } else {
        None
    }
}

pub(crate) fn correct_exact_wrong_layout_ascii_word(token: &str) -> Option<String> {
    if is_blocked_ascii_layout_token(token) {
        return None;
    }

    let (_, original_word, _) = split_word_punctuation(token);
    if original_word.is_empty()
        || original_word
            .chars()
            .filter(|ch| ch.is_ascii_alphabetic())
            .count()
            < 2
        || is_user_protected_ascii_word(original_word)
    {
        return None;
    }

    let candidate = exact_ascii_to_russian_layout_candidate(token)?;
    let snapshot = crate::exact_layout_authority::exact_authority_snapshot_if_warm(
        crate::exact_layout_authority::FactoryEngineProfile::UsQwerty,
        crate::exact_layout_authority::ActiveDecoderLayout::Us,
    )?;
    if !crate::nanda_wave::exact_layout_terminal_contains_if_warm(
        &candidate.word.to_lowercase(),
        snapshot.russian_terminal_fingerprint(),
    )? {
        return None;
    }
    if is_protected_ascii_layout_token(token)
        && is_known_english_layout_autoswitch_word(&original_word.to_ascii_lowercase())
    {
        return None;
    }
    if is_protected_ascii_layout_token(token) {
        return None;
    }
    Some(candidate.replacement)
}

pub(crate) fn correct_wrong_layout_ascii_word_experimental(token: &str) -> Option<String> {
    correct_wrong_layout_ascii_word(token)
}

fn is_standalone_all_caps_shift_layout_token(token: &str) -> bool {
    if !has_ascii_shift_letter_signal(token) {
        return false;
    }

    let letters: Vec<char> = token
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .collect();
    letters.len() >= 4 && letters.iter().all(|ch| ch.is_ascii_uppercase())
}

fn allow_short_layout_word(original: &str, converted_lower: &str) -> bool {
    original
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .count()
        <= 3
        && (russian_tiny_dictionary().contains(converted_lower)
            || crate::hot_field::HotFieldSnapshot::current()
                .layout_projection_has_phase_authority(converted_lower))
}

#[cfg(test)]
mod tests {
    use super::{
        correct_confident_wrong_layout_ascii_word, correct_exact_wrong_layout_ascii_word,
        correct_wrong_layout_ascii_word,
    };

    #[test]
    fn protected_dotted_ascii_token_never_enters_layout_projection() {
        for token in ["archive.tar", "example.com", "src/main.rs"] {
            assert_eq!(correct_confident_wrong_layout_ascii_word(token), None);
            assert_eq!(correct_wrong_layout_ascii_word(token), None);
        }
    }

    #[test]
    fn exact_scope_accepts_only_raw_known_layout_projection() {
        crate::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
            .expect("warm exact-layout authority");
        assert_eq!(
            correct_exact_wrong_layout_ascii_word("ghbdtn").as_deref(),
            Some("привет")
        );
        assert_eq!(correct_exact_wrong_layout_ascii_word("pdf"), None);
        assert_eq!(correct_exact_wrong_layout_ascii_word("dnjpfvtyf"), None);
    }
}
