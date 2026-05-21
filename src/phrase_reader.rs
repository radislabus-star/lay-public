//! Phrase-level reading and scoring for typing assist.
//!
//! This module handles corrections that need more than one token of context:
//! glued words, accidentally split words, and a letter moved into the next word.
//! It does not talk to the daemon or emit text; it only returns deterministic
//! candidate text for the higher-level typing assist arbiter.

use crate::candidate_ranker::choose_best_with_gap;
use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::{is_common_ru_word, is_ru_single_letter_pronoun};
use crate::phrase_candidates::glued_phrase_part_candidates;
use crate::phrase_lexicon::{
    is_common_short_russian_preposition, is_known_russian_phrase_part,
    is_one_letter_russian_function_word, is_short_russian_function_word,
};
use crate::phrase_score::{
    contextual_glued_tail_split_score, is_confident_multiword_glued_phrase,
    is_contextual_glued_tail_split_shape, multiword_glued_phrase_score, MAX_RU_GLUED_PHRASE_PARTS,
    NGRAM_GLUED_SPLIT_MARGIN, NGRAM_MOVED_PREFIX_MARGIN, NGRAM_MOVED_PREFIX_RIGHT_MARGIN,
    NGRAM_NODICT_SPLIT_REJECT_MARGIN, NGRAM_SPLIT_REJECT_MARGIN,
};
use crate::russian_chars::same_letter_ignore_case;
use crate::russian_lexicon::{
    is_known_russian_adverb_o_form, is_known_russian_ka_oblique_form,
    is_known_russian_word_or_form, russian_dictionary, russian_generated_form_dictionary,
    russian_short_dictionary, russian_tiny_dictionary,
};
use crate::russian_prefixes::is_derivational_prefix_fragment;
use crate::russian_typo_scoring::ngram_allows_ru_candidate;
use crate::text_case::{apply_phrase_case, apply_word_case};
use crate::word_reader::{
    cyrillic_word_segmentations, cyrillic_word_splits, is_cyrillic_word, split_word_punctuation,
    split_ws_segments, MAX_RU_FUNCTION_GLUE_LEFT_LEN,
};

pub fn correct_moved_prefix_letter_pair(text: &str) -> Option<String> {
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
        || right.chars().count() < 2
    {
        return None;
    }

    let mut right_chars = right.chars();
    let moved = right_chars.next()?;
    if is_known_russian_word_or_form(&right.to_lowercase()) {
        return None;
    }
    let right_rest: String = right_chars.collect();
    let left_candidate = format!("{left}{moved}");
    let candidate = format!("{left_candidate} {right_rest}");

    if !is_cyrillic_word(&left_candidate) || !is_cyrillic_word(&right_rest) {
        return None;
    }

    let left_candidate_lower = left_candidate.to_lowercase();
    let right_rest_lower = right_rest.to_lowercase();
    let right_lower = right.to_lowercase();
    let short_right_is_safe = is_safe_short_moved_prefix_right(&right_rest_lower)
        && !is_known_russian_word_or_form(&right_lower);

    if left_candidate.chars().count() >= 5
        && short_right_is_safe
        && is_known_russian_word_or_form(&left_candidate_lower)
        && ngram_allows_ru_candidate(&candidate.to_lowercase(), text, NGRAM_MOVED_PREFIX_MARGIN)
    {
        return Some(format!("{candidate}{right_trailing}"));
    }

    if let Some(left_last) = left.chars().last() {
        if short_right_is_safe
            && same_letter_ignore_case(left_last, moved)
            && left.chars().count() > 1
            && is_known_russian_word_or_form(&left.to_lowercase())
            && crate::ngram::ru_candidate_margin(&right_rest_lower, &right_lower)
                >= NGRAM_MOVED_PREFIX_RIGHT_MARGIN
        {
            let candidate = format!("{left} {right_rest}");
            if ngram_allows_ru_candidate(&candidate.to_lowercase(), text, NGRAM_MOVED_PREFIX_MARGIN)
            {
                return Some(format!("{candidate}{right_trailing}"));
            }
        }
    }

    if left_candidate.chars().count() <= 3
        && !is_known_russian_word_or_form(&left.to_lowercase())
        && (russian_tiny_dictionary().contains(&left_candidate_lower)
            || russian_short_dictionary().contains(&left_candidate_lower))
        && right_rest.chars().count() >= 5
        && is_known_russian_phrase_part(&right_rest_lower)
    {
        return Some(format!("{candidate}{right_trailing}"));
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

    Some(format!("{candidate}{right_trailing}"))
}

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

pub fn correct_contextual_glued_tail(core: &str) -> Option<String> {
    let segments = split_ws_segments(core);
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
        || !is_cyrillic_word(left)
        || !is_cyrillic_word(right)
    {
        return None;
    }

    let left_lower = left.to_lowercase();
    let right_lower = right.to_lowercase();
    if !is_short_russian_function_word(&left_lower) || !is_known_russian_phrase_part(&left_lower) {
        return None;
    }

    let right_len = right_lower.chars().count();
    if !(6..=18).contains(&right_len) {
        return None;
    }

    let split_candidates = right_lower
        .char_indices()
        .skip(1)
        .map(|(idx, _)| idx)
        .take(right_len.saturating_sub(1))
        .filter_map(|split_at| {
            let (right_left, right_right) = right_lower.split_at(split_at);
            let right_left_len = right_left.chars().count();
            let right_right_len = right_right.chars().count();

            if !(2..=4).contains(&right_left_len) || right_right_len < 4 {
                return None;
            }
            if !is_known_russian_phrase_part(right_left)
                || !is_known_russian_phrase_part(right_right)
                || !is_contextual_glued_tail_split_shape(&left_lower, right_left, right_right)
            {
                return None;
            }

            let candidate_lower = format!("{left_lower} {right_left} {right_right}");
            let baseline_lower = format!("{left_lower} {right_lower}");
            let margin = crate::ngram::ru_candidate_margin(&candidate_lower, &baseline_lower);
            let score =
                contextual_glued_tail_split_score(&left_lower, right_left, right_right, margin);
            if score < 5.0 {
                return None;
            }

            Some((format!("{right_left} {right_right}"), score))
        });

    let ((right_replacement_lower, _), _) =
        choose_best_with_gap(split_candidates, 0.75, |(_, score)| Some(*score))?;

    let right_replacement = apply_phrase_case(right, &right_replacement_lower);
    Some(format!(
        "{}{}{}{}",
        left, segments[1].0, right_replacement, right_trailing
    ))
}

pub fn correct_glued_russian_phrase(word: &str) -> Option<String> {
    let char_len = word.chars().count();
    if !(4..=24).contains(&char_len) || !word.chars().all(is_cyrillic_letter) {
        return None;
    }

    let lower = word.to_lowercase();
    if russian_dictionary().contains(&lower)
        || russian_generated_form_dictionary().contains(&lower)
        || (is_known_russian_word_or_form(&lower) && !looks_like_word_glued_to_trailing_ya(&lower))
    {
        return None;
    }
    if let Some(candidate) = correct_multiword_glued_russian_phrase(&lower) {
        return Some(apply_phrase_case(word, &candidate));
    }

    let mut scored_candidates = Vec::new();
    for split in cyrillic_word_splits(&lower) {
        let left = split.left;
        let right = split.right;
        let left_len = split.left_len;
        let right_len = split.right_len;
        if is_derivational_prefix_fragment(left, right) {
            continue;
        }
        let short_left_pronoun = left_len == 1 && is_single_letter_russian_pronoun(left);
        let short_right_function = right_len == 1
            && can_split_glued_trailing_ya(left)
            && is_single_letter_russian_pronoun(right);
        if left_len == 1 && !short_left_pronoun {
            continue;
        }
        if (right_len < 3 && !short_right_function) || left_len > 8 {
            continue;
        }

        let left_candidates = glued_phrase_part_candidates(left);
        let right_candidates = glued_phrase_part_candidates(right);
        if left_candidates.is_empty() || right_candidates.is_empty() {
            continue;
        }

        let left_has_standalone_candidate = left_candidates
            .iter()
            .any(|(candidate, _)| is_standalone_russian_phrase_part(candidate));
        let right_has_standalone_candidate = right_candidates
            .iter()
            .any(|(candidate, _)| is_standalone_russian_phrase_part(candidate));
        let right_has_known_candidate = right_candidates
            .iter()
            .any(|(candidate, _)| is_known_russian_phrase_part(candidate));

        if short_left_pronoun && !right_has_standalone_candidate {
            continue;
        }
        if !short_left_pronoun && left_len <= 3 && !left_has_standalone_candidate {
            continue;
        }
        if !short_left_pronoun
            && left_len <= 3
            && is_short_russian_function_word(left)
            && !right_has_known_candidate
        {
            continue;
        }
        if left_len <= 3 && is_common_short_russian_preposition(left) {
            continue;
        }
        for (left_candidate, left_cost) in left_candidates {
            for (right_candidate, right_cost) in &right_candidates {
                let repair_cost = left_cost + *right_cost;
                if repair_cost > 1.0 {
                    continue;
                }

                let candidate = format!("{left_candidate} {right_candidate}");
                let margin = crate::ngram::ru_candidate_margin(&candidate, &lower);
                if repair_cost == 0.0
                    && !is_confident_glued_phrase_split(left, right)
                    && margin < NGRAM_GLUED_SPLIT_MARGIN
                {
                    continue;
                }

                let score = margin - repair_cost * 0.85;
                scored_candidates.push((candidate, score));
            }
        }
    }

    let ((candidate, _), _) =
        choose_best_with_gap(scored_candidates, 0.40, |(_, score)| Some(*score))?;
    Some(apply_phrase_case(word, &candidate))
}

fn correct_multiword_glued_russian_phrase(lower: &str) -> Option<String> {
    if lower.chars().count() < 7 {
        return None;
    }

    let mut scored_candidates = Vec::new();
    for parts in cyrillic_word_segmentations(lower, MAX_RU_GLUED_PHRASE_PARTS) {
        if !is_confident_multiword_glued_phrase(&parts) {
            continue;
        }

        let candidate = parts.join(" ");
        let margin = crate::ngram::ru_candidate_margin(&candidate, lower);
        let score = multiword_glued_phrase_score(&parts, margin);
        if score < 7.0 {
            continue;
        }

        scored_candidates.push((candidate, score));
    }

    let ((candidate, _), _) =
        choose_best_with_gap(scored_candidates, 0.75, |(_, score)| Some(*score))?;
    Some(candidate)
}

fn looks_like_word_glued_to_trailing_ya(word: &str) -> bool {
    let Some(left) = word.strip_suffix('я') else {
        return false;
    };
    can_split_glued_trailing_ya(left) && is_known_russian_phrase_part(left)
}

fn is_standalone_russian_phrase_part(word: &str) -> bool {
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

fn is_single_letter_russian_pronoun(word: &str) -> bool {
    is_ru_single_letter_pronoun(word)
}

fn is_confident_glued_phrase_split(left: &str, right: &str) -> bool {
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

fn can_split_glued_trailing_ya(left: &str) -> bool {
    let len = left.chars().count();
    (4..=5).contains(&len)
        && (is_common_ru_word(left)
            || is_known_russian_adverb_o_form(left)
            || russian_short_dictionary().contains(left))
}

fn is_shouty_cyrillic_word(word: &str) -> bool {
    let letters: Vec<char> = word.chars().filter(|ch| ch.is_alphabetic()).collect();
    letters.len() >= 3
        && letters.iter().all(|ch| is_cyrillic_letter(*ch))
        && letters.iter().all(|ch| ch.is_uppercase())
}

fn should_keep_standalone_pair_with_short_right(left: &str, right: &str) -> bool {
    let right_len = right.chars().count();
    right_len <= 3 && is_known_russian_phrase_part(left) && is_known_russian_phrase_part(right)
}

fn should_keep_standalone_pair_with_function_left(left: &str, right: &str) -> bool {
    if is_single_letter_russian_pronoun(left) {
        return false;
    }
    is_short_russian_function_word(left) && right.chars().count() >= 2 && is_cyrillic_word(right)
}

fn can_merge_split_without_dictionary(
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

fn is_safe_short_moved_prefix_right(word: &str) -> bool {
    (3..=4).contains(&word.chars().count()) && russian_short_dictionary().contains(word)
}

#[cfg(test)]
#[path = "phrase_reader_tests.rs"]
mod tests;
