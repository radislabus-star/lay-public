//! Deterministic repair for tokens that contain both Cyrillic and Latin letters.
//!
//! This layer is not an LLM. It fixes narrow layout islands inside one token,
//! for example a Latin tail accidentally typed inside a Russian word or a
//! Cyrillic lookalike prefix glued to an ASCII technical token.

use crate::text_metrics::{has_cyrillic, has_latin, is_cyrillic_char};
use crate::token_language::is_known_ru_token;
use crate::word_recognizer::{is_protected_ascii_token, is_upper_ascii_acronym};

const RU_VOWELS: &str = "аеёиоуыэюяАЕЁИОУЫЭЮЯ";

pub fn repair_mixed_script(text: &str) -> Option<String> {
    if !has_cyrillic(text) || !has_latin(text) {
        return None;
    }

    let mut out = String::with_capacity(text.len());
    let mut token = String::new();
    for ch in text.chars() {
        if ch.is_alphabetic() {
            token.push(ch);
        } else {
            push_repaired_token(&mut out, &token);
            token.clear();
            out.push(ch);
        }
    }
    push_repaired_token(&mut out, &token);

    if out != text {
        Some(out)
    } else {
        None
    }
}

fn push_repaired_token(out: &mut String, token: &str) {
    if token.is_empty() {
        return;
    }

    let token_has_cyr = has_cyrillic(token);
    let token_has_lat = has_latin(token);
    if token_has_cyr && token_has_lat {
        if let Some(ascii) = repair_mixed_ascii_token(token) {
            out.push_str(&ascii);
        } else if let Some(russian) = repair_mixed_russian_token(token) {
            out.push_str(&russian);
        } else {
            out.push_str(token);
        }
    } else if token_has_lat && should_convert_latin_island(token) {
        out.push_str(&crate::dict::convert(token, crate::dict::Direction::Us2Ru));
    } else {
        out.push_str(token);
    }
}

fn repair_mixed_ascii_token(token: &str) -> Option<String> {
    if !starts_with_latin_letter(token) {
        return None;
    }

    let candidate: String = token
        .chars()
        .map(|ch| {
            if is_cyrillic_char(ch) {
                crate::dict::convert(&ch.to_string(), crate::dict::Direction::Ru2Us)
            } else {
                ch.to_string()
            }
        })
        .collect();

    (candidate != token && is_protected_ascii_token(&candidate)).then_some(candidate)
}

fn starts_with_latin_letter(token: &str) -> bool {
    token
        .chars()
        .find(|ch| ch.is_alphabetic())
        .is_some_and(|ch| ch.is_ascii_alphabetic())
}

fn repair_mixed_russian_token(token: &str) -> Option<String> {
    let candidate = latin_chars_to_ru(token);
    if candidate == token {
        return None;
    }
    if is_known_ru_token(&candidate) {
        return Some(candidate);
    }

    let latin_count = token.chars().filter(|ch| ch.is_ascii_alphabetic()).count();
    if latin_count == 1 && crate::quality::score(&candidate, "ru") >= 0.99 {
        return Some(candidate);
    }

    should_repair_trailing_latin_as_ru(token, &candidate).then_some(candidate)
}

fn should_repair_trailing_latin_as_ru(token: &str, candidate: &str) -> bool {
    let mut prefix = String::new();
    let mut latin_tail = String::new();
    let mut seen_latin = false;

    for ch in token.chars() {
        if ch.is_ascii_alphabetic() {
            seen_latin = true;
            latin_tail.push(ch);
        } else if is_cyrillic_char(ch) {
            if seen_latin {
                return false;
            }
            prefix.push(ch);
        } else {
            return false;
        }
    }

    let prefix_len = prefix.chars().count();
    let tail_len = latin_tail.chars().count();
    let short_upper_prefix = prefix_len >= 2
        && tail_len >= 4
        && token
            .chars()
            .filter(|ch| ch.is_alphabetic())
            .all(char::is_uppercase);
    if (prefix_len < 3 && !short_upper_prefix) || !(2..=4).contains(&tail_len) {
        return false;
    }

    let converted_tail = latin_chars_to_ru(&latin_tail);
    if converted_tail == latin_tail {
        return false;
    }

    let tail_has_vowel = converted_tail.chars().any(|ch| RU_VOWELS.contains(ch));
    if !tail_has_vowel {
        return false;
    }

    let prefix_lower = prefix.to_lowercase();
    let tail_lower = converted_tail.to_lowercase();
    if prefix_lower.ends_with(&tail_lower) {
        return false;
    }

    crate::quality::score(candidate, "ru") >= 0.99
}

fn should_convert_latin_island(token: &str) -> bool {
    let len = token.chars().count();
    len == 1 && !is_upper_ascii_acronym(token)
}

fn latin_chars_to_ru(token: &str) -> String {
    token
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphabetic() {
                crate::dict::convert(&ch.to_string(), crate::dict::Direction::Us2Ru)
            } else {
                ch.to_string()
            }
        })
        .collect()
}
