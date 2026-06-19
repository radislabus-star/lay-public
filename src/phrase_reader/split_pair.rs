use crate::phrase_lexicon::{is_known_russian_phrase_part, is_one_letter_russian_function_word};
use crate::phrase_score::NGRAM_SPLIT_REJECT_MARGIN;
use crate::ru_typo::correct_repeated_letter;
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
    let glued = format!("{}{}", pair.left, pair.right);
    if glued.chars().count() < 4 || !is_cyrillic_word(&glued) {
        return None;
    }

    let lower = glued.to_lowercase();
    let glued_candidate = split_word_merge_candidate(&glued, &lower);
    let glued_is_preferable = glued_candidate.as_ref().is_some_and(|(_, lower)| {
        ngram_allows_ru_candidate(lower, text, NGRAM_SPLIT_REJECT_MARGIN)
    });
    if should_keep_standalone_pair_with_short_right(&left_lower, &right_lower) {
        return None;
    }
    if should_keep_standalone_known_pair(&left_lower, &right_lower)
        && (!glued_is_preferable || right_lower.chars().count() <= 4)
    {
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

    if glued_candidate.is_none()
        && !can_merge_split_without_dictionary(pair.left, pair.right, &lower, text)
    {
        return None;
    }
    let (replacement, replacement_lower) =
        glued_candidate.unwrap_or_else(|| (apply_word_case(&glued, &lower), lower.clone()));
    if !ngram_allows_ru_candidate(&replacement_lower, text, NGRAM_SPLIT_REJECT_MARGIN) {
        return None;
    }

    Some(format!("{replacement}{}", pair.right_trailing))
}

fn split_word_merge_candidate(original_glued: &str, lower: &str) -> Option<(String, String)> {
    if is_known_russian_word_or_form(lower) {
        return Some((apply_word_case(original_glued, lower), lower.to_string()));
    }

    let repaired = correct_repeated_letter(original_glued)?;
    let repaired_lower = repaired.to_lowercase();
    is_known_russian_word_or_form(&repaired_lower).then_some((repaired, repaired_lower))
}
