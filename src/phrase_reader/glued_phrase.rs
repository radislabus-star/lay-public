use crate::candidate_ranker::choose_best_with_gap;
use crate::keyboard::is_cyrillic_letter;
use crate::phrase_candidates::glued_phrase_part_candidates;
use crate::phrase_lexicon::{
    is_common_be_verb_form, is_common_short_russian_preposition, is_known_russian_phrase_part,
    is_short_russian_function_word, looks_like_short_function_word_glued_to_known_word,
};
use crate::phrase_score::{
    contains_preferable_merged_russian_part, is_confident_multiword_glued_phrase,
    is_known_multiword_glue_part, multiword_glued_phrase_score, MAX_RU_GLUED_PHRASE_PARTS,
    NGRAM_GLUED_SPLIT_MARGIN,
};
use crate::russian_lexicon::russian_dictionary;
use crate::russian_prefixes::is_derivational_prefix_fragment;
use crate::text_case::apply_phrase_case;
use crate::word_reader::cyrillic_word_splits;

use super::guards::{
    can_split_glued_trailing_ya, is_confident_glued_phrase_split, is_single_letter_russian_pronoun,
    is_standalone_russian_phrase_part, looks_like_word_glued_to_trailing_ya,
};
use super::preposition_guard::{
    starts_with_multi_letter_preposition, starts_with_multi_letter_preposition_text,
};
use super::verb_guard::looks_like_single_prefixed_verb;

pub fn correct_glued_russian_phrase(word: &str) -> Option<String> {
    let char_len = word.chars().count();
    if !(4..=24).contains(&char_len) || !word.chars().all(is_cyrillic_letter) {
        return None;
    }

    let lower = word.to_lowercase();
    if lower.starts_with("не") && (lower.ends_with("ти") || lower.ends_with("ть")) {
        return None;
    }
    if looks_like_single_prefixed_verb(&lower) {
        return None;
    }
    if russian_dictionary().contains(&lower)
        || crate::nanda_wave::l2::l2_surface_foundation_has_authority(&lower)
        || (crate::russian_lexicon::is_exact_reference_russian_word(&lower)
            && !looks_like_word_glued_to_trailing_ya(&lower)
            && !looks_like_short_function_word_glued_to_known_word(&lower))
    {
        return None;
    }
    let protected_form = crate::russian_lexicon::has_clean_russian_surface_certificate(&lower);
    if protected_form
        && !looks_like_word_glued_to_trailing_ya(&lower)
        && !looks_like_short_function_word_glued_to_known_word(&lower)
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

        let left_cheap = glued_phrase_part_candidates(left, false);
        let right_cheap = glued_phrase_part_candidates(right, false);
        if left_cheap.is_empty() && right_cheap.is_empty() {
            continue;
        }
        let left_candidates = if right_cheap.is_empty() {
            left_cheap
        } else {
            glued_phrase_part_candidates(left, true)
        };
        let right_candidates = if left_candidates.is_empty() {
            right_cheap
        } else {
            glued_phrase_part_candidates(right, true)
        };
        if left_candidates.is_empty() || right_candidates.is_empty() {
            continue;
        }

        let left_has_standalone_candidate = left_candidates
            .iter()
            .any(|(candidate, _)| is_standalone_russian_phrase_part(candidate));
        let right_has_standalone_candidate = right_candidates.iter().any(|(candidate, _)| {
            is_standalone_russian_phrase_part(candidate) || is_common_be_verb_form(candidate)
        });
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
                if starts_with_multi_letter_preposition_text(&candidate) {
                    continue;
                }
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

    let trace = std::env::var_os("LAY_L2_FIELD_TRACE").is_some();
    let started = std::time::Instant::now();
    let mut complete_paths = 0usize;
    let mut merged_rejects = 0usize;
    let mut confidence_rejects = 0usize;
    let mut guard_us = 0u128;
    let mut score_us = 0u128;
    let mut scored_candidates = Vec::new();
    for_each_viable_multiword_segmentation(lower, MAX_RU_GLUED_PHRASE_PARTS, |parts| {
        complete_paths += 1;
        let guard_started = std::time::Instant::now();
        if starts_with_multi_letter_preposition(parts) {
            guard_us += guard_started.elapsed().as_micros();
            return;
        }
        if contains_preferable_merged_russian_part(parts) {
            merged_rejects += 1;
            guard_us += guard_started.elapsed().as_micros();
            return;
        }
        if !is_confident_multiword_glued_phrase(parts) && !is_function_chain_glued_phrase(parts) {
            confidence_rejects += 1;
            guard_us += guard_started.elapsed().as_micros();
            return;
        }
        guard_us += guard_started.elapsed().as_micros();

        let score_started = std::time::Instant::now();
        let candidate = parts.join(" ");
        let margin = crate::ngram::ru_candidate_margin(&candidate, lower);
        let score = multiword_glued_phrase_score(parts, margin);
        score_us += score_started.elapsed().as_micros();
        if score < 7.0 {
            return;
        }

        scored_candidates.push((candidate, score));
    });
    if trace {
        eprintln!(
            "glued_multiword_trace total_us={} complete_paths={} merged_rejects={} confidence_rejects={} scored={} guard_us={} score_us={}",
            started.elapsed().as_micros(),
            complete_paths,
            merged_rejects,
            confidence_rejects,
            scored_candidates.len(),
            guard_us,
            score_us,
        );
    }

    let ((candidate, _), _) =
        choose_best_with_gap(scored_candidates, 0.75, |(_, score)| Some(*score))?;
    Some(candidate)
}

fn for_each_viable_multiword_segmentation<'a>(
    word: &'a str,
    max_parts: usize,
    mut visit: impl FnMut(&[&'a str]),
) {
    if max_parts < 3 {
        return;
    }
    let mut boundaries = word
        .char_indices()
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    boundaries.push(word.len());
    let char_len = boundaries.len().saturating_sub(1);
    if char_len < 3 {
        return;
    }

    let stride = char_len + 1;
    let mut known_parts = vec![false; stride * stride];
    for start in 1..char_len {
        for end in start + 1..=char_len {
            if end - start < 2 {
                continue;
            }
            let part = &word[boundaries[start]..boundaries[end]];
            known_parts[start * stride + end] = is_known_multiword_glue_part(part);
        }
    }

    let mut parts = Vec::with_capacity(max_parts);
    for first_end in 1..char_len {
        let first = &word[..boundaries[first_end]];
        if !crate::lexicon::is_ru_single_letter_pronoun(first)
            && !is_short_russian_function_word(first)
        {
            continue;
        }
        parts.push(first);
        collect_viable_multiword_segmentations(
            word,
            &boundaries,
            &known_parts,
            stride,
            first_end,
            max_parts,
            &mut parts,
            &mut visit,
        );
        parts.pop();
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "bounded search state remains explicit"
)]
fn collect_viable_multiword_segmentations<'a>(
    word: &'a str,
    boundaries: &[usize],
    known_parts: &[bool],
    stride: usize,
    start: usize,
    max_parts: usize,
    parts: &mut Vec<&'a str>,
    visit: &mut impl FnMut(&[&'a str]),
) {
    let char_len = boundaries.len().saturating_sub(1);
    if start == char_len {
        if parts.len() >= 3 {
            visit(parts);
        }
        return;
    }
    if parts.len() >= max_parts {
        return;
    }

    let function_chain = parts.get(1).is_some_and(|part| *part == "и");
    for end in start + 1..=char_len {
        if function_chain && end != char_len {
            continue;
        }
        let part = &word[boundaries[start]..boundaries[end]];
        let part_len = end - start;
        let middle_function_chain = parts.len() == 1
            && part == "и"
            && parts[0].chars().count() >= 2
            && is_short_russian_function_word(parts[0]);
        let known_part = part_len >= 2 && known_parts[start * stride + end];
        if !middle_function_chain && !known_part {
            continue;
        }
        if function_chain
            && (part_len < 3 || !crate::phrase_lexicon::is_known_russian_phrase_part(part))
        {
            continue;
        }
        parts.push(part);
        collect_viable_multiword_segmentations(
            word,
            boundaries,
            known_parts,
            stride,
            end,
            max_parts,
            parts,
            visit,
        );
        parts.pop();
    }
}

fn is_function_chain_glued_phrase(parts: &[&str]) -> bool {
    if parts.len() != 3 {
        return false;
    }
    let [left, middle, right] = parts else {
        return false;
    };
    left.chars().count() >= 2
        && middle == &"и"
        && is_short_russian_function_word(left)
        && right.chars().count() >= 3
        && is_known_russian_phrase_part(right)
}
