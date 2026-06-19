use crate::data_lines::data_lines;
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_candidates::generate_extra_letter_candidates;

use super::super::thresholds::NGRAM_EXTRA_LETTER_MARGIN;

const REFLEXIVE_CONFUSION_DATA: &str =
    include_str!("../../../data/lexicon/russian_reflexive_confusion.tsv");

pub(super) fn reflexive_confusion_sources() -> impl Iterator<Item = &'static str> {
    data_lines(REFLEXIVE_CONFUSION_DATA)
        .filter_map(|line| line.split_once('\t').map(|(from, _)| from))
}

pub(crate) fn extra_letter_candidate_exists(lower: &str) -> bool {
    safe_extra_letter_candidates(lower)
        .into_iter()
        .any(|candidate| {
            candidate != lower
                && is_known_russian_word_or_form(&candidate)
                && crate::ngram::ru_candidate_margin(&candidate, lower) >= NGRAM_EXTRA_LETTER_MARGIN
        })
}

pub(super) fn safe_extra_letter_candidates(lower: &str) -> Vec<String> {
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
