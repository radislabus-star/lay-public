use crate::phrase_lexicon::{is_known_russian_phrase_part, is_one_letter_russian_function_word};
use crate::phrase_score::NGRAM_SPLIT_REJECT_MARGIN;
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_scoring::ngram_allows_ru_candidate;
use crate::text_case::apply_word_case;
use crate::word_reader::{is_cyrillic_word, split_word_punctuation, split_ws_segments};

use super::guards::{
    can_merge_split_without_dictionary, is_shouty_cyrillic_word,
    should_keep_standalone_pair_with_function_left,
    should_keep_standalone_pair_with_function_right, should_keep_standalone_pair_with_short_right,
};

pub fn correct_split_word_pair(text: &str) -> Option<String> {
    let segments = split_ws_segments(text);
    if segments.len() != 3 || segments[0].1 || !segments[1].1 || segments[2].1 {
        return None;
    }

    let (left_leading, left, left_trailing) = split_word_punctuation(segments[0].0);
    let (right_leading, right, right_trailing) = split_word_punctuation(segments[2].0);
    if !left_leading.is_empty()
        || !left_trailing.is_empty()
        || !right_leading.is_empty()
        || left.is_empty()
        || right.is_empty()
    {
        return None;
    }

    let left_lower = left.to_lowercase();
    let right_lower = right.to_lowercase();
    if is_shouty_cyrillic_word(right) {
        return None;
    }
    if should_keep_standalone_pair_with_short_right(&left_lower, &right_lower) {
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

    let glued = format!("{left}{right}");
    if glued.chars().count() < 4 || !is_cyrillic_word(&glued) {
        return None;
    }

    let lower = glued.to_lowercase();
    if !is_known_russian_word_or_form(&lower)
        && !can_merge_split_without_dictionary(left, right, &lower, text)
    {
        return None;
    }
    if !ngram_allows_ru_candidate(&lower, text, NGRAM_SPLIT_REJECT_MARGIN) {
        return None;
    }

    Some(format!(
        "{}{}",
        apply_word_case(&glued, &lower),
        right_trailing
    ))
}
