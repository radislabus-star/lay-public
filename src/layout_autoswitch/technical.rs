//! Technical-token layout corrections.

use crate::keyboard::is_cyrillic_letter;
use crate::token_language::is_known_ru_token;
use crate::word_recognizer::{
    is_ascii_technical_or_brand_token, is_ascii_technical_token, is_upper_ascii_acronym,
};

use super::hyphen::{
    has_known_cyrillic_hyphen_fragment, is_cyrillic_hyphenated_word_for_layout,
    is_plain_cyrillic_technical_source,
};

pub fn correct_duplicate_layout_prefix_on_ascii_token(token: &str) -> Option<String> {
    let mut chars = token.chars();
    let first = chars.next()?;
    if !is_cyrillic_letter(first) {
        return None;
    }

    let rest: String = chars.collect();
    if !is_ascii_technical_token(&rest) {
        return None;
    }

    let mapped = crate::dict::convert(&first.to_string(), crate::dict::Direction::Ru2Us);
    let mut mapped_chars = mapped.chars();
    let mapped = mapped_chars.next()?;
    if mapped_chars.next().is_some() {
        return None;
    }

    let rest_first = rest.chars().next()?;
    if rest_first.is_ascii_alphabetic() && mapped.eq_ignore_ascii_case(&rest_first) {
        Some(rest)
    } else {
        None
    }
}

pub fn correct_wrong_layout_ascii_technical_token(token: &str) -> Option<String> {
    if let Some(repaired) = correct_ascii_prefix_with_ru_layout_tail(token) {
        return Some(repaired);
    }

    if !token.contains('-') || !is_plain_cyrillic_technical_source(token) {
        return None;
    }

    let converted = crate::dict::convert(token, crate::dict::Direction::Ru2Us);
    if converted == token || !is_ascii_technical_token(&converted) {
        return None;
    }
    if !has_clear_ascii_technical_layout_signal(&converted) {
        return None;
    }

    let has_clear_separator = converted.contains('-');
    let has_short_ascii_segment = converted
        .split(['-', '_', '.'])
        .any(|part| (2..=4).contains(&part.chars().count()));
    let original_known_hyphen_word = token.contains('-')
        && (is_cyrillic_hyphenated_word_for_layout(token)
            || has_known_cyrillic_hyphen_fragment(token));

    if has_clear_separator && has_short_ascii_segment && !original_known_hyphen_word {
        Some(converted)
    } else {
        None
    }
}

fn correct_ascii_prefix_with_ru_layout_tail(token: &str) -> Option<String> {
    let (prefix, tail) = token.split_once('-')?;
    if prefix.is_empty()
        || tail.is_empty()
        || tail.contains('-')
        || !tail.chars().all(|ch| ch.is_ascii_alphabetic())
    {
        return None;
    }
    if !is_ascii_layout_anchor(prefix) {
        return None;
    }

    let converted_tail = crate::dict::convert(tail, crate::dict::Direction::Us2Ru);
    if converted_tail == tail || !is_known_ru_token(&converted_tail) {
        return None;
    }

    Some(format!("{prefix}-{converted_tail}"))
}

fn is_ascii_layout_anchor(prefix: &str) -> bool {
    prefix.is_ascii()
        && prefix.chars().any(|ch| ch.is_ascii_alphabetic())
        && (is_upper_ascii_acronym(prefix) || is_ascii_technical_or_brand_token(prefix))
}

pub fn should_keep_plain_cyrillic_before_ascii_technical(original: &str, converted: &str) -> bool {
    original.chars().count() >= 4
        && original.chars().all(is_cyrillic_letter)
        && converted != original
        && is_ascii_technical_token(converted)
}

fn has_clear_ascii_technical_layout_signal(token: &str) -> bool {
    let alpha_total = token.chars().filter(|ch| ch.is_ascii_alphabetic()).count();
    let alpha_segment = token
        .split(['-', '_', '.', '@', '/', '\\', ':', '+', '#'])
        .any(|part| part.chars().filter(|ch| ch.is_ascii_alphabetic()).count() >= 2);

    alpha_total >= 4 && alpha_segment
}
