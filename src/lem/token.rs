use crate::dict;
use crate::keyboard::is_cyrillic_letter;
use crate::token_language::{is_known_en_token, is_known_ru_token};
use crate::word_recognizer::is_ascii_technical_or_brand_token;

pub(super) fn trim_token(token: &str) -> &str {
    token.trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '-')
}

pub fn is_known_text(text: &str) -> bool {
    let mut saw_token = false;
    for token in text.split_whitespace().map(trim_token) {
        if token.is_empty() {
            continue;
        }
        saw_token = true;
        if !is_plausible_token(token) {
            return false;
        }
    }
    saw_token
}

fn is_plausible_token(token: &str) -> bool {
    !is_layout_garbage_token(token) && is_plausible_non_layout_token(token)
}

fn is_plausible_non_layout_token(token: &str) -> bool {
    is_known_word(token)
        || is_ascii_technical_or_brand_token(token)
        || is_natural_hyphenated_token(token)
}

pub(super) fn is_natural_hyphenated_token(token: &str) -> bool {
    let mut saw_part = false;
    let mut alpha_parts = 0usize;
    let mut known_parts = 0usize;
    let mut long_parts = 0usize;
    for part in token.split('-') {
        if part.is_empty() {
            return false;
        }
        saw_part = true;
        let alpha_count = part.chars().filter(|ch| ch.is_alphabetic()).count();
        if alpha_count == 0 || alpha_count != part.chars().count() {
            return false;
        }
        if alpha_count >= 2 {
            alpha_parts += 1;
        }
        if alpha_count >= 3 {
            long_parts += 1;
        }
        if is_known_word(part) {
            known_parts += 1;
        }
    }
    saw_part && alpha_parts >= 2 && (known_parts > 0 || long_parts > 0)
}

pub(super) fn is_layout_garbage_token(token: &str) -> bool {
    if is_known_word(token)
        || (token.chars().any(is_cyrillic_letter) && is_natural_hyphenated_token(token))
    {
        return false;
    }
    if token.chars().filter(|ch| ch.is_alphabetic()).count() < 3 {
        return false;
    }
    let converted = dict::convert(token, dict::detect_direction(token));
    converted != token && is_plausible_non_layout_token(&converted)
}

pub(super) fn has_ascii_layout_letter_punctuation(token: &str) -> bool {
    token
        .chars()
        .any(|ch| matches!(ch, '\'' | ';' | '[' | ']' | '`' | ',' | '.'))
}

pub(super) fn is_known_word(token: &str) -> bool {
    let lower = token.to_lowercase();
    if lower.chars().all(is_cyrillic_letter) {
        return is_known_ru_token(&lower);
    }
    if lower.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return is_known_en_token(&lower);
    }
    false
}
