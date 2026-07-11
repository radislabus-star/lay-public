use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::text_case::apply_word_case;
use crate::word_reader::is_cyrillic_word;

use super::keyboard::are_ru_keyboard_neighbors;

pub(crate) fn correct_single_letter_substitution(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }

    let (candidate, _) = crate::candidate_ranker::choose_best_with_gap(
        crate::nanda_wave::l2::l2_center_near_surfaces(&lower, 64),
        0.40,
        |candidate| {
            if !safe_neighbor_substitution_candidate(&lower, candidate) {
                return None;
            }
            let center_prior = crate::nanda_wave::l2::l2_surface_foundation_rank(candidate)
                .map(|rank| 12.0 / (1.0 + rank as f64 / 2_000.0))
                .unwrap_or(0.0);
            Some(crate::ngram::ru_candidate_margin(candidate, &lower) + center_prior)
        },
    )?;
    Some(apply_word_case(word, &candidate))
}

fn safe_neighbor_substitution_candidate(original: &str, candidate: &str) -> bool {
    if !is_known_russian_word_or_form(candidate) {
        return false;
    }
    let original_chars: Vec<char> = original.chars().collect();
    let candidate_chars: Vec<char> = candidate.chars().collect();
    if original_chars.len() != candidate_chars.len() {
        return false;
    }
    let diffs: Vec<(char, char)> = original_chars
        .into_iter()
        .zip(candidate_chars)
        .filter(|(left, right)| left != right)
        .collect();
    matches!(diffs.as_slice(), [(left, right)] if are_ru_keyboard_neighbors(*left, *right))
}
