use crate::candidate_ranker::choose_best_with_gap;
use crate::keyboard::is_cyrillic_letter;
use crate::phrase_candidates::glued_phrase_part_candidates;
use crate::phrase_lexicon::{
    is_common_short_russian_preposition, is_known_russian_phrase_part,
    is_short_russian_function_word,
};
use crate::phrase_score::{
    contains_preferable_merged_russian_part, is_confident_multiword_glued_phrase,
    multiword_glued_phrase_score, MAX_RU_GLUED_PHRASE_PARTS, NGRAM_GLUED_SPLIT_MARGIN,
};
use crate::russian_lexicon::{
    is_known_russian_word_or_form, russian_dictionary, russian_generated_form_dictionary,
};
use crate::russian_prefixes::is_derivational_prefix_fragment;
use crate::text_case::apply_phrase_case;
use crate::word_reader::{cyrillic_word_segmentations, cyrillic_word_splits};

use super::guards::{
    can_split_glued_trailing_ya, is_confident_glued_phrase_split, is_single_letter_russian_pronoun,
    is_standalone_russian_phrase_part, looks_like_word_glued_to_trailing_ya,
};

pub fn correct_glued_russian_phrase(word: &str) -> Option<String> {
    let char_len = word.chars().count();
    if !(4..=24).contains(&char_len) || !word.chars().all(is_cyrillic_letter) {
        return None;
    }

    let lower = word.to_lowercase();
    if looks_like_single_prefixed_verb(&lower) {
        return None;
    }
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

    let mut scored_candidates = Vec::new();
    for parts in cyrillic_word_segmentations(lower, MAX_RU_GLUED_PHRASE_PARTS) {
        if starts_with_multi_letter_preposition(&parts) {
            continue;
        }
        if contains_preferable_merged_russian_part(&parts) {
            continue;
        }
        if !is_confident_multiword_glued_phrase(&parts) && !is_function_chain_glued_phrase(&parts) {
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

fn starts_with_multi_letter_preposition(parts: &[&str]) -> bool {
    parts
        .first()
        .is_some_and(|part| part.chars().count() >= 2 && is_common_short_russian_preposition(part))
}

fn starts_with_multi_letter_preposition_text(text: &str) -> bool {
    text.split_whitespace()
        .next()
        .is_some_and(|part| part.chars().count() >= 2 && is_common_short_russian_preposition(part))
}

fn looks_like_single_prefixed_verb(lower: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "пере", "недо", "пред", "про", "при", "под", "над", "без", "раз", "рас", "воз", "вос",
        "до", "за", "на", "от", "по", "вы", "об",
    ];
    const VERB_TAILS: &[&str] = &[
        "ется",
        "ётся",
        "атся",
        "ятся",
        "уется",
        "ается",
        "яется",
        "ывает",
        "ивает",
        "ешь",
        "ишь",
        "ает",
        "яет",
        "ует",
        "ит",
        "ет",
    ];
    lower.chars().count() >= 8
        && PREFIXES.iter().any(|prefix| lower.starts_with(prefix))
        && VERB_TAILS.iter().any(|tail| lower.ends_with(tail))
}
