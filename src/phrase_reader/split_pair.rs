use crate::phrase_lexicon::{is_known_russian_phrase_part, is_one_letter_russian_function_word};
use crate::phrase_score::NGRAM_SPLIT_REJECT_MARGIN;
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_scoring::ngram_allows_ru_candidate;
use crate::text_case::apply_word_case;
use crate::word_reader::is_cyrillic_word;

use super::guards::{
    can_merge_split_without_dictionary, is_shouty_cyrillic_word, read_plain_phrase_pair,
    should_keep_standalone_known_pair, should_keep_standalone_pair_with_function_left,
    should_keep_standalone_pair_with_function_right, should_keep_standalone_pair_with_short_right,
};

pub fn correct_split_word_pair(text: &str) -> Option<String> {
    let pair = read_plain_phrase_pair(text)?;

    let left_lower = pair.left.to_lowercase();
    let right_lower = pair.right.to_lowercase();
    if is_shouty_cyrillic_word(pair.right) {
        return None;
    }
    if crate::layout_autoswitch::correct_wrong_layout_cyrillic_word(pair.left).is_some()
        || crate::layout_autoswitch::correct_wrong_layout_cyrillic_word(pair.right).is_some()
    {
        return None;
    }
    if should_keep_standalone_pair_with_short_right(&left_lower, &right_lower) {
        return None;
    }
    if should_keep_standalone_known_pair(&left_lower, &right_lower) {
        return None;
    }
    if should_keep_standalone_pair_with_function_right(&left_lower, &right_lower) {
        return None;
    }
    if should_keep_standalone_pair_with_function_left(&left_lower, &right_lower) {
        return None;
    }
    if is_known_russian_phrase_part(&left_lower)
        && is_one_letter_russian_function_word(&right_lower)
    {
        return None;
    }

    let glued = format!("{}{}", pair.left, pair.right);
    if glued.chars().count() < 4 || !is_cyrillic_word(&glued) {
        return None;
    }

    let lower = glued.to_lowercase();
    if !is_known_russian_word_or_form(&lower)
        && !can_merge_split_without_dictionary(pair.left, pair.right, &lower, text)
    {
        return None;
    }
    if !ngram_allows_ru_candidate(&lower, text, NGRAM_SPLIT_REJECT_MARGIN) {
        return None;
    }

    Some(format!(
        "{}{}",
        apply_word_case(&glued, &lower),
        pair.right_trailing
    ))
}
