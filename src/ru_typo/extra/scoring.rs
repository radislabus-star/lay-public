use crate::lexicon::is_common_ru_word;
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::text_case::apply_word_case;
use crate::word_reader::is_cyrillic_word;

use super::super::thresholds::NGRAM_EXTRA_LETTER_MARGIN;

pub(super) fn best_extra_letter_candidate(
    original: &str,
    candidates: Vec<String>,
) -> Option<String> {
    let lower = original.to_lowercase();
    let mut found = None;
    for candidate in candidates {
        if candidate == lower
            || !is_cyrillic_word(&candidate)
            || !(is_known_russian_word_or_form(&candidate) || is_common_ru_word(&candidate))
        {
            continue;
        }
        if crate::ngram::ru_candidate_margin(&candidate, &lower) < NGRAM_EXTRA_LETTER_MARGIN {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some(candidate);
    }
    found.map(|candidate| apply_word_case(original, &candidate))
}

pub(super) fn best_common_extra_letter_candidate(
    original: &str,
    candidates: Vec<String>,
) -> Option<String> {
    let lower = original.to_lowercase();
    let mut found = None;
    for candidate in candidates {
        if candidate == lower || !is_cyrillic_word(&candidate) || !is_common_ru_word(&candidate) {
            continue;
        }
        if crate::ngram::ru_candidate_margin(&candidate, &lower) < NGRAM_EXTRA_LETTER_MARGIN {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some(candidate);
    }
    found.map(|candidate| apply_word_case(original, &candidate))
}
