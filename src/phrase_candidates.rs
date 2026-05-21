//! Candidate builders for phrase-level corrections.

use crate::phrase_lexicon::{is_known_russian_phrase_part, is_short_russian_function_word};
use crate::ru_typo::{
    correct_adjacent_transposition, correct_hard_sign_typo, correct_missing_letter,
    correct_repeated_letter, safe_missing_letter_candidates,
};
use crate::word_reader::MAX_RU_FUNCTION_GLUE_LEFT_LEN;

pub(crate) fn glued_phrase_part_candidates(part: &str) -> Vec<(String, f64)> {
    let mut out = Vec::new();
    push_glued_phrase_part_candidate(&mut out, part.to_string(), 0.0);

    let part_len = part.chars().count();
    if (3..=MAX_RU_FUNCTION_GLUE_LEFT_LEN).contains(&part_len)
        && !is_known_russian_phrase_part(part)
    {
        for candidate in safe_missing_letter_candidates(part) {
            if candidate.chars().count() <= MAX_RU_FUNCTION_GLUE_LEFT_LEN
                && is_short_russian_function_word(&candidate)
            {
                push_glued_phrase_part_candidate(&mut out, candidate, 0.85);
            }
        }
    }

    if part_len >= 5 && !is_known_russian_phrase_part(part) {
        for candidate in [
            correct_missing_letter(part),
            correct_adjacent_transposition(part),
            correct_repeated_letter(part),
            correct_hard_sign_typo(part),
        ]
        .into_iter()
        .flatten()
        {
            push_glued_phrase_part_candidate(&mut out, candidate.to_lowercase(), 1.0);
        }
    }

    out.retain(|(candidate, _)| is_known_russian_phrase_part(candidate));
    out
}

fn push_glued_phrase_part_candidate(out: &mut Vec<(String, f64)>, candidate: String, cost: f64) {
    if out.iter().any(|(existing, _)| existing == &candidate) {
        return;
    }
    out.push((candidate, cost));
}
