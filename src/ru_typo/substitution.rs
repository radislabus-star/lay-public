use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::text_case::apply_word_case;
use crate::word_reader::is_cyrillic_word;
use std::collections::HashMap;
use std::sync::OnceLock;

use super::keyboard::are_ru_keyboard_neighbors;

const SAFE_NEIGHBOR_SUBSTITUTION_FIXES: &str =
    include_str!("../../data/lexicon/russian_neighbor_substitution_fixes.tsv");

pub(crate) fn correct_single_letter_substitution(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }

    let candidate = safe_neighbor_substitution_fixes().get(lower.as_str())?;
    safe_neighbor_substitution_candidate(&lower, candidate)
        .then(|| apply_word_case(word, candidate))
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

fn safe_neighbor_substitution_fixes() -> &'static HashMap<String, String> {
    static FIXES: OnceLock<HashMap<String, String>> = OnceLock::new();
    FIXES.get_or_init(|| {
        SAFE_NEIGHBOR_SUBSTITUTION_FIXES
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .filter_map(|line| {
                let (wrong, correct) = line.split_once('\t')?;
                Some((wrong.to_string(), correct.to_string()))
            })
            .collect()
    })
}
