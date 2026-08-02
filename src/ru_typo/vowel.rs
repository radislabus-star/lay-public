use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_candidates::generate_vowel_confusion_candidates;
use crate::russian_typo_scoring::best_unique_known_ngram_candidate;
use crate::word_reader::is_cyrillic_word;

use super::guards::{
    looks_like_plausible_russian_past_tense, rewrites_protected_pattern_term_stem,
};
use super::thresholds::NGRAM_VOWEL_CONFUSION_MARGIN;

#[path = "vowel/past_tense.rs"]
mod past_tense;
use past_tense::has_same_simple_past_tense_tail;

pub(crate) fn correct_vowel_confusion(word: &str) -> Option<String> {
    correct_vowel_confusion_impl(word, false)
}

pub(crate) fn correct_contextual_past_tense_vowel_confusion(word: &str) -> Option<String> {
    correct_vowel_confusion_impl(word, true)
}

fn correct_vowel_confusion_impl(word: &str, allow_safe_past_tense: bool) -> Option<String> {
    if word.contains('-') {
        return None;
    }
    (word.chars().count() >= 5 && is_cyrillic_word(word)).then_some(())?;

    let lower = word.to_lowercase();
    (!is_known_russian_word_or_form(&lower)).then_some(())?;
    (!crate::russian_lexicon::has_clean_russian_surface_certificate(&lower)).then_some(())?;
    (!super::missing::missing_letter_candidate_exists(word, &lower)).then_some(())?;
    let candidates = generate_vowel_confusion_candidates(&lower)
        .into_iter()
        .filter(|candidate| !rewrites_protected_pattern_term_stem(&lower, candidate))
        .collect::<Vec<_>>();
    (!looks_like_plausible_russian_past_tense(&lower)
        || (allow_safe_past_tense
            && vowel_confusion_past_tense_candidate_exists(&lower, &candidates)))
    .then_some(())?;

    best_unique_known_ngram_candidate(word, candidates, NGRAM_VOWEL_CONFUSION_MARGIN)
}

fn vowel_confusion_past_tense_candidate_exists(lower: &str, candidates: &[String]) -> bool {
    candidates.iter().any(|candidate| {
        candidate != lower
            && is_known_russian_word_or_form(candidate)
            && has_same_simple_past_tense_tail(lower, candidate)
            && crate::ngram::ru_candidate_margin(candidate, lower) >= NGRAM_VOWEL_CONFUSION_MARGIN
    })
}
