use crate::lexicon::is_user_protected_ascii_word;
use crate::russian_lexicon::russian_tiny_dictionary;
use crate::word_reader::split_word_punctuation;

use super::super::english::is_known_english_layout_autoswitch_word;
use super::super::score::lem_prefers_layout_candidate;
use super::candidate::ascii_to_russian_layout_candidate;
use super::symbols::{
    has_ascii_shift_letter_signal, is_ascii_layout_token_symbol, is_blocked_ascii_layout_token,
    is_protected_ascii_layout_token,
};

pub(crate) fn correct_confident_wrong_layout_ascii_word(token: &str) -> Option<String> {
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

    let candidate = ascii_to_russian_layout_candidate(token, false)?;
    if !candidate.known {
        return None;
    }
    if is_protected_ascii_layout_token(token)
        && is_known_english_layout_autoswitch_word(&original_word.to_ascii_lowercase())
    {
        return None;
    }
    if candidate.clean_alpha && !lem_prefers_layout_candidate(original_word, &candidate.word) {
        return None;
    }
    Some(candidate.replacement)
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

    if let Some(replacement) =
        correct_wrong_layout_ascii_word_preserving_trailing_punctuation(token)
    {
        return Some(replacement);
    }

    let strong_shift_layout = is_standalone_all_caps_shift_layout_token(token);
    let candidate = ascii_to_russian_layout_candidate(token, strong_shift_layout)?;
    let normalized = candidate.replacement;
    let normalized_word = candidate.word;
    let normalized_lower = normalized_word.to_lowercase();
    if strong_shift_layout {
        return Some(normalized);
    }
    if is_protected_ascii_layout_token(token)
        && is_known_english_layout_autoswitch_word(&original_word.to_ascii_lowercase())
    {
        return None;
    }

    if is_protected_ascii_layout_token(token) {
        return lem_prefers_layout_candidate(original_word, &normalized_word).then_some(normalized);
    }
    match crate::llm::choose_token_hybrid(original_word, &normalized_word) {
        Ok(Some(choice)) if choice == normalized_word => Some(normalized),
        Ok(Some(choice)) if choice == original_word => {
            allow_short_layout_word(original_word, &normalized_lower).then_some(normalized)
        }
        _ => Some(normalized),
    }
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

fn correct_wrong_layout_ascii_word_preserving_trailing_punctuation(token: &str) -> Option<String> {
    let trailing_start = token
        .char_indices()
        .rev()
        .find(|(_, ch)| ch.is_ascii_alphanumeric())
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(0);
    if trailing_start == 0 || trailing_start == token.len() {
        return None;
    }

    let core = &token[..trailing_start];
    let trailing = &token[trailing_start..];
    if !core.chars().any(|ch| ch.is_ascii_alphabetic())
        || !core
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || is_ascii_layout_token_symbol(ch))
        || !trailing.chars().all(is_ascii_layout_token_symbol)
    {
        return None;
    }

    let (_, original_word, _) = split_word_punctuation(core);
    if original_word.is_empty() {
        return None;
    }
    if is_user_protected_ascii_word(original_word) {
        return None;
    }
    let normalized = ascii_to_russian_layout_candidate(core, false)?.replacement;
    Some(format!("{normalized}{trailing}"))
}

fn allow_short_layout_word(original: &str, converted_lower: &str) -> bool {
    original
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .count()
        <= 3
        && russian_tiny_dictionary().contains(converted_lower)
}
