use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_scoring::ngram_allows_ru_candidate;
use crate::text_case::apply_word_case;

use super::extra::extra_letter_candidate_exists;
use super::guards::{
    looks_like_known_word_plus_one_letter_function_suffix, unknown_cyrillic_lower,
};
use super::thresholds::NGRAM_TRANSPOSE_MARGIN;

pub(crate) fn correct_adjacent_transposition(word: &str) -> Option<String> {
    let lower = unknown_cyrillic_lower(word, 5)?;
    if extra_letter_candidate_exists(&lower) {
        return None;
    }

    let chars: Vec<char> = lower.chars().collect();
    let mut found: Option<String> = None;
    for idx in 0..chars.len().saturating_sub(1) {
        if chars[idx] == chars[idx + 1] {
            continue;
        }

        let mut candidate = chars.clone();
        candidate.swap(idx, idx + 1);
        let candidate: String = candidate.into_iter().collect();
        if !is_known_russian_word_or_form(&candidate) {
            continue;
        }
        if looks_like_known_word_plus_one_letter_function_suffix(&candidate) {
            continue;
        }
        if !ngram_allows_ru_candidate(&candidate, &lower, NGRAM_TRANSPOSE_MARGIN) {
            continue;
        }

        if found.is_some() {
            return None;
        }
        found = Some(candidate);
    }

    found.map(|candidate| apply_word_case(word, &candidate))
}
