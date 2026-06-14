use crate::phrase_lexicon::is_known_russian_phrase_part;
use crate::phrase_score::{NGRAM_MOVED_PREFIX_MARGIN, NGRAM_MOVED_PREFIX_RIGHT_MARGIN};
use crate::russian_chars::same_letter_ignore_case;
use crate::russian_lexicon::{
    is_known_russian_word_or_form, russian_short_dictionary, russian_tiny_dictionary,
};
use crate::russian_typo_scoring::ngram_allows_ru_candidate;
use crate::word_reader::is_cyrillic_word;

use super::guards::{is_safe_short_moved_prefix_right, read_plain_phrase_pair};

pub fn correct_moved_prefix_letter_pair(text: &str) -> Option<String> {
    let pair = read_plain_phrase_pair(text)?;
    if pair.right.chars().count() < 2 {
        return None;
    }

    let mut right_chars = pair.right.chars();
    let moved = right_chars.next()?;
    if is_known_russian_word_or_form(&pair.right.to_lowercase()) {
        return None;
    }
    let right_rest: String = right_chars.collect();
    let left_candidate = format!("{}{}", pair.left, moved);
    let candidate = format!("{left_candidate} {right_rest}");

    if !is_cyrillic_word(&left_candidate) || !is_cyrillic_word(&right_rest) {
        return None;
    }

    let left_candidate_lower = left_candidate.to_lowercase();
    let right_rest_lower = right_rest.to_lowercase();
    let right_lower = pair.right.to_lowercase();
    let short_right_is_safe = is_safe_short_moved_prefix_right(&right_rest_lower)
        && !is_known_russian_word_or_form(&right_lower);
    let left_original_lower = pair.left.to_lowercase();

    if same_letter_ignore_case(moved, 'й')
        && left_candidate.chars().count() >= 5
        && right_rest.chars().count() >= 5
        && !is_known_russian_word_or_form(&left_original_lower)
        && !is_known_russian_word_or_form(&right_lower)
        && is_known_russian_word_or_form(&left_candidate_lower)
        && is_known_russian_phrase_part(&right_rest_lower)
    {
        return Some(format!("{}{}", candidate, pair.right_trailing));
    }

    if left_candidate.chars().count() >= 5
        && short_right_is_safe
        && is_known_russian_word_or_form(&left_candidate_lower)
        && ngram_allows_ru_candidate(&candidate.to_lowercase(), text, NGRAM_MOVED_PREFIX_MARGIN)
    {
        return Some(format!("{}{}", candidate, pair.right_trailing));
    }

    if let Some(left_last) = pair.left.chars().last() {
        if short_right_is_safe
            && same_letter_ignore_case(left_last, moved)
            && pair.left.chars().count() > 1
            && is_known_russian_word_or_form(&left_original_lower)
            && crate::ngram::ru_candidate_margin(&right_rest_lower, &right_lower)
                >= NGRAM_MOVED_PREFIX_RIGHT_MARGIN
        {
            let candidate = format!("{} {}", pair.left, right_rest);
            if ngram_allows_ru_candidate(&candidate.to_lowercase(), text, NGRAM_MOVED_PREFIX_MARGIN)
            {
                return Some(format!("{}{}", candidate, pair.right_trailing));
            }
        }
    }

    if left_candidate.chars().count() <= 3
        && !is_known_russian_word_or_form(&left_original_lower)
        && (russian_tiny_dictionary().contains(&left_candidate_lower)
            || russian_short_dictionary().contains(&left_candidate_lower))
        && right_rest.chars().count() >= 5
        && is_known_russian_phrase_part(&right_rest_lower)
    {
        return Some(format!("{}{}", candidate, pair.right_trailing));
    }

    if left_candidate.chars().count() < 5 || right_rest.chars().count() < 5 {
        return None;
    }
    if !is_known_russian_word_or_form(&left_candidate_lower)
        || !is_known_russian_word_or_form(&right_rest_lower)
    {
        return None;
    }
    if crate::ngram::ru_candidate_margin(&right_rest_lower, &right_lower)
        < NGRAM_MOVED_PREFIX_RIGHT_MARGIN
    {
        return None;
    }
    if !ngram_allows_ru_candidate(&candidate.to_lowercase(), text, NGRAM_MOVED_PREFIX_MARGIN) {
        return None;
    }

    Some(format!("{}{}", candidate, pair.right_trailing))
}
