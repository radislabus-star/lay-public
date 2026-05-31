use crate::candidate_ranker::choose_best_with_gap;
use crate::phrase_lexicon::{is_known_russian_phrase_part, is_short_russian_function_word};
use crate::phrase_score::{
    contextual_glued_tail_split_score, is_contextual_glued_tail_split_shape,
};
use crate::text_case::apply_phrase_case;
use crate::word_reader::is_cyrillic_word;

use super::guards::read_plain_phrase_pair;

pub fn correct_contextual_glued_tail(core: &str) -> Option<String> {
    let pair = read_plain_phrase_pair(core)?;
    if !is_cyrillic_word(pair.left) || !is_cyrillic_word(pair.right) {
        return None;
    }

    let left_lower = pair.left.to_lowercase();
    let right_lower = pair.right.to_lowercase();
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

    let right_replacement = apply_phrase_case(pair.right, &right_replacement_lower);
    Some(format!(
        "{}{}{}{}",
        pair.left, pair.separator, right_replacement, pair.right_trailing
    ))
}
