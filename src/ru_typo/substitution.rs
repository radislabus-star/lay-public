use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_candidates::RU_ALPHABET;
use crate::russian_typo_scoring::ngram_allows_ru_candidate;
use crate::text_case::apply_word_case;
use crate::word_reader::is_cyrillic_word;

use super::guards::looks_like_known_word_plus_one_letter_function_suffix;
use super::keyboard::are_ru_keyboard_neighbors;
use super::missing::missing_letter_candidate_exists;
use super::thresholds::NGRAM_TYPO_REJECT_MARGIN;

pub(crate) fn correct_single_letter_substitution(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }
    if missing_letter_candidate_exists(word, &lower) {
        return None;
    }

    let chars: Vec<char> = lower.chars().collect();
    let mut found: Option<String> = None;
    for idx in 0..chars.len() {
        // First-letter substitutions are too ambiguous for automatic correction:
        // slang, names and dialect forms often differ from dictionary words only there.
        if idx == 0 {
            continue;
        }
        for replacement in RU_ALPHABET {
            if replacement == chars[idx] {
                continue;
            }
            if !are_ru_keyboard_neighbors(chars[idx], replacement) {
                continue;
            }

            let mut candidate = chars.clone();
            candidate[idx] = replacement;
            let candidate: String = candidate.into_iter().collect();
            if !is_known_russian_word_or_form(&candidate) {
                continue;
            }
            if looks_like_known_word_plus_one_letter_function_suffix(&candidate) {
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

    found.map(|candidate| apply_word_case(word, &candidate))
}
