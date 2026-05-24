use crate::text_case::apply_word_case;
use crate::word_reader::{is_cyrillic_word, split_word_punctuation};

use super::super::{
    is_known_russian_layout_autoswitch_word, polish_converted_russian_layout_token,
};
use super::symbols::{has_ascii_shift_letter_signal, is_blocked_ascii_layout_token};

#[derive(Debug, Clone)]
pub(super) struct AsciiToRussianLayoutCandidate {
    pub replacement: String,
    pub word: String,
    pub known: bool,
    pub clean_alpha: bool,
    pub shift_letter_signal: bool,
}

pub(super) fn ascii_to_russian_layout_candidate(
    token: &str,
    allow_shift_fallback: bool,
) -> Option<AsciiToRussianLayoutCandidate> {
    if is_blocked_ascii_layout_token(token) {
        return None;
    }

    let (_, original_word, _) = split_word_punctuation(token);
    if original_word.is_empty() {
        return None;
    }

    let converted = crate::dict::convert(token, crate::dict::Direction::Us2Ru);
    if converted == token {
        return None;
    }

    let (_, converted_word, _) = split_word_punctuation(&converted);
    if converted_word.is_empty() || !is_cyrillic_word(converted_word) {
        return None;
    }

    let converted_lower = converted_word.to_lowercase();
    let known = is_known_russian_layout_autoswitch_word(&converted_lower);
    let shift_letter_signal = has_ascii_shift_letter_signal(token);
    if !(known || allow_shift_fallback && shift_letter_signal) {
        return None;
    }

    let replacement = if known {
        let normalized_word = apply_word_case(original_word, &converted_lower);
        let (converted_leading, _, converted_trailing) = split_word_punctuation(&converted);
        format!("{converted_leading}{normalized_word}{converted_trailing}")
    } else {
        converted
    };

    let replacement = polish_converted_russian_layout_token(&replacement).unwrap_or(replacement);
    let (_, replacement_word, _) = split_word_punctuation(&replacement);
    let word = replacement_word.to_string();
    let replacement_known = known || is_known_russian_layout_autoswitch_word(&word.to_lowercase());

    Some(AsciiToRussianLayoutCandidate {
        replacement,
        word,
        known: replacement_known,
        clean_alpha: token.chars().all(|ch| ch.is_ascii_alphabetic()),
        shift_letter_signal,
    })
}
