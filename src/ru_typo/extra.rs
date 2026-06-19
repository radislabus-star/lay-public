use crate::data_lines::data_lines;
use crate::phrase_lexicon::looks_like_short_function_word_glued_to_known_word;
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_candidates::generate_extra_letter_candidates;
use crate::russian_typo_scoring::best_unique_known_ngram_candidate;

use super::guards::{
    correct_invalid_adjective_tail, looks_like_plausible_russian_past_tense, unknown_cyrillic_lower,
};
use super::missing::missing_letter_candidate_exists;
use super::thresholds::NGRAM_EXTRA_LETTER_MARGIN;

const REFLEXIVE_CONFUSION_DATA: &str =
    include_str!("../../data/lexicon/russian_reflexive_confusion.tsv");

pub fn correct_extra_letters(word: &str) -> Option<String> {
    let lower = unknown_cyrillic_lower(word, 6)?;
    if reflexive_confusion_sources().any(|suffix| lower.ends_with(suffix))
        || looks_like_short_function_word_glued_to_known_word(&lower)
        || looks_like_plausible_russian_past_tense(&lower)
    {
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
        safe_extra_letter_candidates(&lower),
        NGRAM_EXTRA_LETTER_MARGIN,
    )
}

fn reflexive_confusion_sources() -> impl Iterator<Item = &'static str> {
    data_lines(REFLEXIVE_CONFUSION_DATA)
        .filter_map(|line| line.split_once('\t').map(|(from, _)| from))
}

pub(super) fn extra_letter_candidate_exists(lower: &str) -> bool {
    safe_extra_letter_candidates(lower)
        .into_iter()
        .any(|candidate| {
            candidate != lower
                && is_known_russian_word_or_form(&candidate)
                && crate::ngram::ru_candidate_margin(&candidate, lower) >= NGRAM_EXTRA_LETTER_MARGIN
        })
}

fn safe_extra_letter_candidates(lower: &str) -> Vec<String> {
    generate_extra_letter_candidates(lower)
        .into_iter()
        .filter(|candidate| {
            !looks_like_unsafe_first_letter_deletion(lower, candidate)
                && !looks_like_unsafe_present_tail_deletion(lower, candidate)
        })
        .collect()
}

fn looks_like_unsafe_first_letter_deletion(lower: &str, candidate: &str) -> bool {
    let chars = lower.chars().collect::<Vec<_>>();
    if chars.len() < 2 || candidate != chars[1..].iter().collect::<String>() {
        return false;
    }
    let first = chars[0];
    let second = chars[1];
    first != second && !matches!(first, 'ы' | 'ь' | 'ъ')
}

fn looks_like_unsafe_present_tail_deletion(lower: &str, candidate: &str) -> bool {
    const PRESENT_TAILS: &[&str] = &["ешь", "ишь", "еет", "ает", "яет", "ует"];
    lower.chars().count() > candidate.chars().count()
        && PRESENT_TAILS
            .iter()
            .any(|tail| lower.ends_with(tail) && candidate.ends_with(tail))
}
