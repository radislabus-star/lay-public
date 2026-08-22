use crate::phrase_lexicon::is_short_russian_function_word;
use crate::russian_chars::is_russian_vowel;
use crate::russian_typo_candidates::repeated_run_deletion_candidates;
use crate::russian_typo_scoring::best_ranked_dictionary_candidate;
use crate::text_case::apply_word_case;
use crate::word_reader::is_cyrillic_word;

use super::memo::{memoized_text, WordMaterialKind};
use super::thresholds::NGRAM_TYPO_REJECT_MARGIN;

pub(crate) fn correct_repeated_letter(word: &str) -> Option<String> {
    memoized_text(WordMaterialKind::RepeatedLetter, word, || {
        correct_repeated_letter_uncached(word)
    })
}

fn correct_repeated_letter_uncached(word: &str) -> Option<String> {
    if !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if crate::russian_lexicon::is_exact_reference_russian_word(&lower)
        || crate::lexicon::is_ru_live_protected_word(&lower)
        || crate::lexicon::is_user_protected_word(&lower)
    {
        return None;
    }
    if let Some(candidate) = correct_short_repeated_function_word(word, &lower) {
        return Some(candidate);
    }
    if word.chars().count() < 5 {
        return None;
    }
    let chars = lower.chars().collect::<Vec<_>>();
    if chars.len() >= 2
        && chars[chars.len() - 1] == chars[chars.len() - 2]
        && is_russian_vowel(chars[chars.len() - 1])
    {
        return None;
    }

    best_ranked_dictionary_candidate(
        word,
        repeated_run_deletion_candidates(&lower),
        NGRAM_TYPO_REJECT_MARGIN,
        0.40,
    )
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
