use crate::phrase_lexicon::looks_like_short_function_word_glued_to_known_word;
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_candidates::generate_extra_letter_candidates;
use crate::russian_typo_scoring::best_unique_known_ngram_candidate;

use super::guards::{correct_invalid_adjective_tail, unknown_cyrillic_lower};
use super::missing::missing_letter_candidate_exists;
use super::thresholds::NGRAM_EXTRA_LETTER_MARGIN;

pub fn correct_extra_letters(word: &str) -> Option<String> {
    let lower = unknown_cyrillic_lower(word, 6)?;
    if lower.ends_with("тся") {
        return None;
    }
    if looks_like_short_function_word_glued_to_known_word(&lower) {
        return None;
    }
    if let Some(candidate) = correct_invalid_adjective_tail(word, &lower) {
        return Some(candidate);
    }
    if missing_letter_candidate_exists(word, &lower) {
        return None;
    }

    best_unique_known_ngram_candidate(
        word,
        generate_extra_letter_candidates(&lower),
        NGRAM_EXTRA_LETTER_MARGIN,
    )
}

pub(super) fn extra_letter_candidate_exists(lower: &str) -> bool {
    generate_extra_letter_candidates(lower)
        .into_iter()
        .any(|candidate| {
            candidate != lower
                && is_known_russian_word_or_form(&candidate)
                && crate::ngram::ru_candidate_margin(&candidate, lower) >= NGRAM_EXTRA_LETTER_MARGIN
        })
}
