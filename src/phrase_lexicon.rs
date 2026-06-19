//! Phrase-level Russian lexical predicates.
//!
//! This is a neutral dependency for both word typo rules and phrase readers.
//! It does not generate corrections.

use crate::lexicon::{
    is_common_ru_word, is_ru_one_letter_function_word, is_ru_short_function_word,
    is_ru_short_preposition, is_ru_short_pronoun,
};
use crate::russian_lexicon::{
    is_known_russian_adverb_o_form, is_known_russian_ka_oblique_form,
    is_known_russian_word_or_form, russian_short_dictionary, russian_tiny_dictionary,
};
use crate::word_reader::{cyrillic_word_splits, MAX_RU_FUNCTION_GLUE_LEFT_LEN};

pub(crate) fn is_known_russian_phrase_part(word: &str) -> bool {
    let len = word.chars().count();
    if len == 1 {
        return is_one_letter_russian_function_word(word);
    }
    if is_ru_short_function_word(word) {
        return true;
    }
    if len <= MAX_RU_FUNCTION_GLUE_LEFT_LEN && is_short_russian_function_word(word) {
        return true;
    }
    if len <= 3 {
        return is_common_ru_word(word)
            || is_common_short_russian_pronoun(word)
            || russian_tiny_dictionary().contains(word)
            || russian_short_dictionary().contains(word);
    }
    is_known_russian_word_or_form(word)
        || is_known_russian_adverb_o_form(word)
        || is_known_russian_ka_oblique_form(word)
        || russian_short_dictionary().contains(word)
}

pub(crate) fn is_one_letter_russian_function_word(word: &str) -> bool {
    is_ru_one_letter_function_word(word)
}

pub(crate) fn is_short_russian_function_word(word: &str) -> bool {
    is_ru_short_function_word(word)
        || (word.chars().count() <= MAX_RU_FUNCTION_GLUE_LEFT_LEN
            && (is_one_letter_russian_function_word(word)
                || is_common_ru_word(word)
                || is_common_short_russian_preposition(word)))
}

pub(crate) fn is_common_short_russian_preposition(word: &str) -> bool {
    is_ru_short_preposition(word)
}

pub(crate) fn looks_like_short_function_word_glued_to_known_word(word: &str) -> bool {
    let char_len = word.chars().count();
    if char_len < 5 {
        return false;
    }

    for split in cyrillic_word_splits(word) {
        let left = split.left;
        let right = split.right;
        let left_len = split.left_len;
        let right_len = split.right_len;
        if left_len > MAX_RU_FUNCTION_GLUE_LEFT_LEN {
            break;
        }
        if right_len < 4 {
            continue;
        }
        if is_short_russian_function_word(left) && is_known_russian_phrase_part(right) {
            return true;
        }
    }
    false
}

fn is_common_short_russian_pronoun(word: &str) -> bool {
    is_ru_short_pronoun(word)
}
