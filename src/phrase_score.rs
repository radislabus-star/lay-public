//! Phrase-level scoring helpers for typing assist.

use crate::lexicon::{is_common_ru_word, is_ru_short_pronoun, is_ru_single_letter_pronoun};
use crate::phrase_lexicon::{
    is_common_short_russian_preposition, is_known_russian_phrase_part,
    is_one_letter_russian_function_word, is_short_russian_function_word,
};
use crate::word_reader::MAX_RU_FUNCTION_GLUE_LEFT_LEN;

pub(crate) const NGRAM_SPLIT_REJECT_MARGIN: f64 = 0.25;
pub(crate) const NGRAM_NODICT_SPLIT_REJECT_MARGIN: f64 = 1.0;
pub(crate) const NGRAM_MOVED_PREFIX_MARGIN: f64 = 0.5;
pub(crate) const NGRAM_MOVED_PREFIX_RIGHT_MARGIN: f64 = 5.0;
pub(crate) const NGRAM_GLUED_SPLIT_MARGIN: f64 = -0.25;
pub(crate) const MAX_RU_GLUED_PHRASE_PARTS: usize = 7;

const CONTEXTUAL_NGRAM_SCALE: f64 = 10.0;
const CONTEXTUAL_NGRAM_FLOOR: f64 = -3.0;
const CONTEXTUAL_LEFT_FUNCTION_BONUS: f64 = 1.0;
const CONTEXTUAL_RIGHT_PRONOUN_BONUS: f64 = 8.0;
const CONTEXTUAL_RIGHT_COMMON_WORD_BONUS: f64 = 2.0;

const MULTIWORD_NGRAM_SCALE: f64 = 15.0;
const MULTIWORD_NGRAM_FLOOR: f64 = -2.0;
const MULTIWORD_LONG_PHRASE_BONUS: f64 = 1.0;
const MULTIWORD_FUNCTION_WITH_STRONG_PARTS_BONUS: f64 = 1.5;
const MULTIWORD_ONE_LETTER_FUNCTION_BONUS: f64 = 2.0;
const MULTIWORD_SHORT_PRONOUN_BONUS: f64 = 1.5;
const MULTIWORD_SHORT_PREPOSITION_BONUS: f64 = 1.5;
const MULTIWORD_SHORT_FUNCTION_BONUS: f64 = 1.2;
const MULTIWORD_COMMON_WORD_BONUS: f64 = 1.5;
const MULTIWORD_STRONG_PART_BONUS: f64 = 2.0;

pub(crate) fn is_contextual_glued_tail_split_shape(
    left: &str,
    right_left: &str,
    right_right: &str,
) -> bool {
    char_len(left) <= 3
        && char_len(right_left) <= 4
        && char_len(right_right) >= 4
        && (is_common_ru_word(right_left)
            || is_ru_short_pronoun(right_left)
            || crate::russian_lexicon::russian_tiny_dictionary().contains(right_left)
            || crate::russian_lexicon::russian_short_dictionary().contains(right_left))
        && is_known_russian_phrase_part(right_right)
}

pub(crate) fn contextual_glued_tail_split_score(
    left: &str,
    right_left: &str,
    right_right: &str,
    ngram_margin: f64,
) -> f64 {
    let mut score = (ngram_margin / CONTEXTUAL_NGRAM_SCALE).max(CONTEXTUAL_NGRAM_FLOOR);
    if is_common_short_russian_preposition(left) || is_one_letter_russian_function_word(left) {
        score += CONTEXTUAL_LEFT_FUNCTION_BONUS;
    }
    if is_ru_short_pronoun(right_left) {
        score += CONTEXTUAL_RIGHT_PRONOUN_BONUS;
    }
    if is_common_ru_word(right_right) {
        score += CONTEXTUAL_RIGHT_COMMON_WORD_BONUS;
    }
    score
}

pub(crate) fn is_confident_multiword_glued_phrase(parts: &[&str]) -> bool {
    if !(3..=MAX_RU_GLUED_PHRASE_PARTS).contains(&parts.len()) {
        return false;
    }
    if !parts.first().is_some_and(|part| {
        is_ru_single_letter_pronoun(part) || is_short_russian_function_word(part)
    }) {
        return false;
    }

    let mut function_parts = 0usize;
    let mut strong_parts = 0usize;
    let mut one_letter_parts = 0usize;
    for (idx, part) in parts.iter().enumerate() {
        let len = char_len(part);
        if looks_like_incomplete_russian_reflexive_part(part) {
            return false;
        }
        if len == 1 {
            if idx != 0 || !is_ru_single_letter_pronoun(part) {
                return false;
            }
            one_letter_parts += 1;
            if one_letter_parts > 1 {
                return false;
            }
            function_parts += 1;
            continue;
        }

        if !is_known_russian_phrase_part(part) {
            return false;
        }
        if is_short_russian_function_word(part)
            || is_common_short_russian_preposition(part)
            || is_ru_short_pronoun(part)
        {
            function_parts += 1;
        }
        if is_strong_phrase_part(part) {
            strong_parts += 1;
        }
    }

    if contains_preferable_merged_russian_part(parts) {
        return false;
    }

    (function_parts >= 2 && strong_parts >= 1)
        || (parts.len() >= 3 && function_parts >= 1 && strong_parts >= 2)
}

pub(crate) fn multiword_glued_phrase_score(parts: &[&str], ngram_margin: f64) -> f64 {
    let mut score = (ngram_margin / MULTIWORD_NGRAM_SCALE).max(MULTIWORD_NGRAM_FLOOR);
    if parts.len() >= 4 {
        score += MULTIWORD_LONG_PHRASE_BONUS;
    }
    let starts_with_function = parts
        .first()
        .is_some_and(|part| is_short_russian_function_word(part));
    let strong_parts = parts
        .iter()
        .filter(|part| is_strong_phrase_part(part))
        .count();
    if starts_with_function && strong_parts >= 2 {
        score += MULTIWORD_FUNCTION_WITH_STRONG_PARTS_BONUS;
    }
    for part in parts {
        let len = char_len(part);
        if len == 1 && is_one_letter_russian_function_word(part) {
            score += MULTIWORD_ONE_LETTER_FUNCTION_BONUS;
        }
        if is_ru_short_pronoun(part) {
            score += MULTIWORD_SHORT_PRONOUN_BONUS;
        }
        if is_common_short_russian_preposition(part) {
            score += MULTIWORD_SHORT_PREPOSITION_BONUS;
        }
        if is_short_russian_function_word(part) {
            score += MULTIWORD_SHORT_FUNCTION_BONUS;
        }
        if is_common_ru_word(part) {
            score += MULTIWORD_COMMON_WORD_BONUS;
        }
        if is_strong_phrase_part(part) {
            score += MULTIWORD_STRONG_PART_BONUS;
        }
    }
    score
}

fn contains_preferable_merged_russian_part(parts: &[&str]) -> bool {
    parts.windows(2).any(|window| {
        let left = window[0];
        let right = window[1];
        let left_len = char_len(left);
        let right_len = char_len(right);
        if left_len > MAX_RU_FUNCTION_GLUE_LEFT_LEN || right_len < 4 {
            return false;
        }
        if !is_short_russian_function_word(left) {
            return false;
        }

        let merged = format!("{left}{right}");
        char_len(&merged) >= 5 && is_known_russian_phrase_part(&merged)
    })
}

fn looks_like_incomplete_russian_reflexive_part(part: &str) -> bool {
    let len = char_len(part);
    len >= 6 && (part.ends_with("тьс") || part.ends_with("тс"))
}

fn is_strong_phrase_part(part: &str) -> bool {
    char_len(part) >= 4 && is_known_russian_phrase_part(part)
}

fn char_len(text: &str) -> usize {
    text.chars().count()
}
