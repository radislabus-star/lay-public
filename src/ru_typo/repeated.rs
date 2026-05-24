use crate::phrase_lexicon::is_short_russian_function_word;
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_candidates::repeated_run_deletion_candidates;
use crate::russian_typo_scoring::ngram_allows_ru_candidate;
use crate::text_case::apply_word_case;
use crate::word_reader::is_cyrillic_word;

use super::thresholds::NGRAM_TYPO_REJECT_MARGIN;

pub(crate) fn correct_repeated_letter(word: &str) -> Option<String> {
    if !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }
    if let Some(candidate) = correct_short_repeated_function_word(word, &lower) {
        return Some(candidate);
    }
    if word.chars().count() < 5 {
        return None;
    }

    let chars: Vec<char> = lower.chars().collect();
    let mut found: Option<String> = None;
    let mut idx = 0;
    while idx < chars.len() {
        let mut end = idx + 1;
        while end < chars.len() && chars[end] == chars[idx] {
            end += 1;
        }

        if end - idx > 1 {
            for keep in 1..end - idx {
                let mut candidate = Vec::with_capacity(chars.len() - (end - idx - keep));
                candidate.extend_from_slice(&chars[..idx]);
                candidate.extend(std::iter::repeat(chars[idx]).take(keep));
                candidate.extend_from_slice(&chars[end..]);
                let candidate: String = candidate.into_iter().collect();
                if !is_known_russian_word_or_form(&candidate) {
                    continue;
                }
                if !ngram_allows_ru_candidate(&candidate, &lower, NGRAM_TYPO_REJECT_MARGIN) {
                    continue;
                }
                if found.is_some() {
                    return None;
                }
                found = Some(candidate);
            }
        }

        idx = end;
    }

    found.map(|candidate| apply_word_case(word, &candidate))
}

fn correct_short_repeated_function_word(original: &str, lower: &str) -> Option<String> {
    if !(3..=4).contains(&lower.chars().count()) {
        return None;
    }

    let mut found: Option<String> = None;
    for candidate in repeated_run_deletion_candidates(lower) {
        if candidate.chars().count() < 2 {
            continue;
        }
        if !is_short_russian_function_word(&candidate) {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some(candidate);
    }

    found.map(|candidate| apply_word_case(original, &candidate))
}
