use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::{is_common_ru_word, is_ru_single_letter_pronoun};
use crate::phrase_lexicon::{
    is_common_short_russian_preposition, is_known_russian_phrase_part,
    is_one_letter_russian_function_word, is_short_russian_function_word,
};
use crate::phrase_score::NGRAM_NODICT_SPLIT_REJECT_MARGIN;
use crate::russian_lexicon::{
    is_known_russian_adverb_o_form, is_known_russian_ka_oblique_form, russian_dictionary,
    russian_short_dictionary,
};
use crate::word_reader::{is_cyrillic_word, MAX_RU_FUNCTION_GLUE_LEFT_LEN};

pub(super) fn looks_like_word_glued_to_trailing_ya(word: &str) -> bool {
    let Some(left) = word.strip_suffix('я') else {
        return false;
    };
    can_split_glued_trailing_ya(left) && is_known_russian_phrase_part(left)
}

pub(super) fn is_standalone_russian_phrase_part(word: &str) -> bool {
    let len = word.chars().count();
    if len == 1 {
        return is_one_letter_russian_function_word(word);
    }
    if len <= MAX_RU_FUNCTION_GLUE_LEFT_LEN && is_short_russian_function_word(word) {
        return true;
    }
    if len <= 3 {
        return is_common_ru_word(word);
    }
    russian_dictionary().contains(word)
        || is_known_russian_adverb_o_form(word)
        || is_known_russian_ka_oblique_form(word)
}

pub(super) fn is_single_letter_russian_pronoun(word: &str) -> bool {
    is_ru_single_letter_pronoun(word)
}

pub(super) fn is_confident_glued_phrase_split(left: &str, right: &str) -> bool {
    (left.chars().count() == 1 && is_single_letter_russian_pronoun(left))
        || (right.chars().count() == 1
            && can_split_glued_trailing_ya(left)
            && is_single_letter_russian_pronoun(right))
        || (left.chars().count() <= MAX_RU_FUNCTION_GLUE_LEFT_LEN
            && right.chars().count() >= 4
            && is_short_russian_function_word(left)
            && !is_common_short_russian_preposition(left)
            && is_known_russian_phrase_part(right))
        || (left.chars().count() <= MAX_RU_FUNCTION_GLUE_LEFT_LEN
            && right.chars().count() >= 2
            && is_short_russian_function_word(left)
            && !is_common_short_russian_preposition(left)
            && is_common_ru_word(right))
        || (left.chars().count() >= 4
            && right.chars().count() >= 4
            && is_known_russian_adverb_o_form(right))
        || (left.chars().count() >= 4
            && right.chars().count() >= 4
            && is_standalone_russian_phrase_part(left)
            && is_standalone_russian_phrase_part(right)
            && (is_short_russian_function_word(left) || is_short_russian_function_word(right)))
}

pub(super) fn can_split_glued_trailing_ya(left: &str) -> bool {
    let len = left.chars().count();
    (4..=5).contains(&len)
        && (is_common_ru_word(left)
            || is_known_russian_adverb_o_form(left)
            || russian_short_dictionary().contains(left))
}

pub(super) fn is_shouty_cyrillic_word(word: &str) -> bool {
    let letters: Vec<char> = word.chars().filter(|ch| ch.is_alphabetic()).collect();
    letters.len() >= 3
        && letters.iter().all(|ch| is_cyrillic_letter(*ch))
        && letters.iter().all(|ch| ch.is_uppercase())
}

pub(super) fn should_keep_standalone_pair_with_short_right(left: &str, right: &str) -> bool {
    let right_len = right.chars().count();
    right_len <= 3 && is_known_russian_phrase_part(left) && is_known_russian_phrase_part(right)
}

pub(super) fn should_keep_standalone_pair_with_function_left(left: &str, right: &str) -> bool {
    if is_single_letter_russian_pronoun(left) {
        return false;
    }
    is_short_russian_function_word(left) && right.chars().count() >= 2 && is_cyrillic_word(right)
}

pub(super) fn can_merge_split_without_dictionary(
    left: &str,
    right: &str,
    glued_lower: &str,
    text: &str,
) -> bool {
    let left_len = left.chars().count();
    let right_len = right.chars().count();
    let glued_len = glued_lower.chars().count();
    if russian_short_dictionary().contains(&right.to_lowercase()) {
        return false;
    }

    (2..=3).contains(&right_len)
        && left_len == 1
        && is_single_letter_russian_pronoun(&left.to_lowercase())
        && glued_len >= 4
        && crate::ngram::ru_candidate_margin(glued_lower, text) >= NGRAM_NODICT_SPLIT_REJECT_MARGIN
}

pub(super) fn is_safe_short_moved_prefix_right(word: &str) -> bool {
    (3..=4).contains(&word.chars().count()) && russian_short_dictionary().contains(word)
}
