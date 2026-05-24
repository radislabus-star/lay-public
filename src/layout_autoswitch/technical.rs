//! Technical-token layout corrections.

use crate::keyboard::is_cyrillic_letter;
use crate::word_recognizer::is_ascii_technical_token;

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
