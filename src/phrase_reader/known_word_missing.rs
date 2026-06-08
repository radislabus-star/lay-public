use crate::phrase_lexicon::{is_known_russian_phrase_part, is_short_russian_function_word};
use crate::russian_typo_candidates::{
    generate_missing_letter_candidates, inserted_char_position_for_missing_letter,
};
use crate::russian_typo_scoring::missing_letter_candidate_bonus;
use crate::word_reader::is_cyrillic_word;

pub(crate) fn correct_contextual_known_word_missing_letter(text: &str) -> Option<String> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }

    let left = parts[0].to_lowercase();
    let right = parts[1].to_lowercase();
    if !is_short_russian_function_word(&left) || !is_cyrillic_word(&right) {
        return None;
    }
    if !is_known_russian_phrase_part(&right) {
        return None;
    }

    let mut found: Option<String> = None;
    for candidate in generate_missing_letter_candidates(&right) {
        if !is_known_russian_phrase_part(&candidate) {
            continue;
        }
        let Some((inserted_at, inserted)) =
            inserted_char_position_for_missing_letter(&right, &candidate)
        else {
            continue;
        };
        if inserted != 'й' {
            continue;
        }
        if inserted_at >= right.chars().count() {
            continue;
        }
        if candidate.contains("йе") {
            continue;
        }

        let score = crate::ngram::ru_candidate_margin(&candidate, &right)
            + missing_letter_candidate_bonus(&right, &candidate);
        if score < -8.0 {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some(candidate);
    }

    found.map(|right| {
        format!(
            "{} {}",
            parts[0],
            crate::text_case::apply_word_case(parts[1], &right)
        )
    })
}
