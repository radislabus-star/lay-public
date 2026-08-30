use crate::phrase_lexicon::{
    is_known_russian_phrase_part, is_one_letter_russian_function_word,
    looks_like_short_function_chain_glued, looks_like_short_function_word_glued_to_known_word,
};
use crate::phrase_score::NGRAM_SPLIT_REJECT_MARGIN;
use crate::ru_typo::propose_repeated_letter_candidate;
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_scoring::ngram_allows_ru_candidate;
use crate::text_case::apply_word_case;
use crate::word_reader::{is_cyrillic_letters_only, is_cyrillic_word};

use super::guards::{
    can_merge_split_without_dictionary, is_shouty_cyrillic_word, read_plain_phrase_pair,
    should_keep_standalone_known_pair, should_keep_standalone_pair_with_function_left,
    should_keep_standalone_pair_with_function_right, should_keep_standalone_pair_with_short_right,
};

pub fn correct_split_word_pair(text: &str) -> Option<String> {
    let pair = read_plain_phrase_pair(text)?;

    let left_lower = pair.left.to_lowercase();
    let right_lower = pair.right.to_lowercase();
    (!is_shouty_cyrillic_word(pair.right)).then_some(())?;
    (crate::layout_autoswitch::correct_wrong_layout_cyrillic_word(pair.left).is_none()
        && crate::layout_autoswitch::correct_wrong_layout_cyrillic_word(pair.right).is_none())
    .then_some(())?;
    let glued = format!("{}{}", pair.left, pair.right);
    (glued.chars().count() >= 4 && is_cyrillic_word(&glued)).then_some(())?;

    let lower = glued.to_lowercase();
    (left_lower.chars().count() != 1 || crate::lexicon::is_common_ru_word(&lower)).then_some(())?;
    let direct_glued_is_known = is_known_russian_word_or_form(&lower);
    let repeats_boundary = left_lower.chars().next_back() == right_lower.chars().next();
    let exact_glued_has_authority = crate::russian_lexicon::is_exact_reference_russian_word(&lower)
        || crate::nanda_wave::l2::l2_surface_foundation_has_authority(&lower);
    if repeats_boundary && !exact_glued_has_authority {
        return None;
    }
    let glued_candidate = split_word_merge_candidate(&glued, &lower);
    let glued_is_preferable = glued_candidate.as_ref().is_some_and(|(_, lower)| {
        ngram_allows_ru_candidate(lower, text, NGRAM_SPLIT_REJECT_MARGIN)
    });
    (!should_keep_standalone_pair_with_short_right(&left_lower, &right_lower)).then_some(())?;
    if left_lower.chars().count() >= 4
        && right_lower.chars().count() <= 3
        && is_known_russian_phrase_part(&left_lower)
        && !is_known_russian_phrase_part(&right_lower)
        && !glued_is_preferable
    {
        return None;
    }
    if should_keep_standalone_known_pair(&left_lower, &right_lower)
        && (!glued_is_preferable || right_lower.chars().count() <= 4)
    {
        return None;
    }
    if crate::lexicon::is_ru_short_pronoun(&left_lower)
        && right_lower.chars().count() >= 4
        && is_cyrillic_letters_only(&right_lower)
        && is_known_russian_word_or_form(&right_lower)
        && (!direct_glued_is_known || !glued_is_preferable)
    {
        return None;
    }
    (!should_keep_standalone_pair_with_function_right(&left_lower, &right_lower)).then_some(())?;
    (!should_keep_standalone_pair_with_function_left(&left_lower, &right_lower)).then_some(())?;
    if is_known_russian_phrase_part(&left_lower)
        && is_one_letter_russian_function_word(&right_lower)
    {
        return None;
    }
    if looks_like_short_function_word_glued_to_known_word(&left_lower)
        && is_known_russian_phrase_part(&right_lower)
    {
        return None;
    }
    if looks_like_short_function_chain_glued(&left_lower)
        && is_known_russian_phrase_part(&right_lower)
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

#[rustfmt::skip]
fn split_word_merge_candidate(original_glued: &str, lower: &str) -> Option<(String, String)> {
    if is_known_russian_word_or_form(lower) { return Some((apply_word_case(original_glued, lower), lower.to_string())); }
    let repaired = propose_repeated_letter_candidate(original_glued)?;
    let repaired_lower = repaired.to_lowercase();
    is_known_russian_word_or_form(&repaired_lower).then_some((repaired, repaired_lower))
}
