use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_candidates::generate_vowel_confusion_candidates;
use crate::russian_typo_scoring::best_unique_known_ngram_candidate;
use crate::word_reader::is_cyrillic_word;

use super::guards::looks_like_plausible_russian_past_tense;
use super::thresholds::NGRAM_VOWEL_CONFUSION_MARGIN;

pub(crate) fn correct_vowel_confusion(word: &str) -> Option<String> {
    correct_vowel_confusion_impl(word, false)
}

pub(crate) fn correct_vowel_confusion_contextual_past_tense(word: &str) -> Option<String> {
    correct_vowel_confusion_impl(word, true)
}

fn correct_vowel_confusion_impl(word: &str, allow_safe_past_tense: bool) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }
    let candidates = generate_vowel_confusion_candidates(&lower);
    if looks_like_plausible_russian_past_tense(&lower)
        && (!allow_safe_past_tense
            || !vowel_confusion_past_tense_candidate_exists(&lower, &candidates))
    {
        return None;
    }

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

fn has_same_simple_past_tense_tail(left: &str, right: &str) -> bool {
    [
        "ился",
        "илась",
        "ились",
        "илось",
        "ался",
        "алась",
        "ались",
        "алось",
        "ила",
        "или",
        "ило",
        "ил",
        "ала",
        "али",
        "ало",
        "ал",
    ]
    .iter()
    .any(|tail| left.ends_with(tail) && right.ends_with(tail))
}
