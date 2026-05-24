//! Cyrillic hyphen-word recognition for layout autoswitch.

use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::is_ru_hyphen_particle;
use crate::phrase_lexicon::is_common_short_russian_preposition;
use crate::russian_chars::is_russian_vowel;
use crate::russian_lexicon::{is_known_cyrillic_hyphen_part, russian_short_dictionary};
use crate::word_reader::is_cyrillic_word;

pub(super) fn has_known_cyrillic_hyphen_fragment(word: &str) -> bool {
    if !word.contains('-') || !is_cyrillic_word(word) {
        return false;
    }

    word.split('-').any(|part| {
        let lower = part.to_lowercase();
        lower.chars().count() >= 3
            && is_known_cyrillic_hyphen_part(&lower, russian_short_dictionary())
    })
}

pub(crate) fn is_cyrillic_hyphenated_word_for_layout(word: &str) -> bool {
    is_known_cyrillic_hyphenated_word(word) || is_plausible_cyrillic_hyphenated_word(word)
}

fn is_known_cyrillic_hyphenated_word(word: &str) -> bool {
    if !is_cyrillic_word(word) {
        return false;
    }
    let dict = russian_short_dictionary();
    word.split('-')
        .all(|part| part.chars().count() >= 3 && is_known_cyrillic_hyphen_part(part, dict))
}

fn is_plausible_cyrillic_hyphenated_word(word: &str) -> bool {
    if !word.contains('-') || !is_cyrillic_word(word) {
        return false;
    }
    let parts: Vec<&str> = word.split('-').collect();
    if parts.len() < 2 || parts.iter().any(|part| part.is_empty()) {
        return false;
    }

    let mut strong_parts = 0usize;
    for (idx, part) in parts.iter().enumerate() {
        let lower = part.to_lowercase();
        let len = lower.chars().count();
        if len < 2 || !lower.chars().any(is_russian_vowel) {
            return false;
        }
        if len >= 3
            || is_known_cyrillic_hyphen_part(&lower, russian_short_dictionary())
            || (idx == 0 && is_common_short_russian_preposition(&lower))
            || (idx > 0 && is_russian_hyphen_particle(&lower))
        {
            strong_parts += 1;
        }
    }
    strong_parts >= 2
}

fn is_russian_hyphen_particle(part: &str) -> bool {
    is_ru_hyphen_particle(part)
}

pub(super) fn is_plain_cyrillic_technical_source(token: &str) -> bool {
    token.chars().any(is_cyrillic_letter)
        && !token.chars().any(|ch| ch.is_ascii_alphabetic())
        && token.chars().all(|ch| {
            is_cyrillic_letter(ch) || ch.is_ascii_digit() || matches!(ch, '-' | '_' | '.')
        })
}
