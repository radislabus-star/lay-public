use crate::data_lines::data_lines;
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_candidates::generate_extra_letter_candidates;
use crate::russian_typo_scoring::ngram_allows_ru_candidate;

use super::super::guards::rewrites_protected_pattern_term_stem;
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
    if has_safe_adjacent_transposition_candidate(lower) {
        return Vec::new();
    }
    generate_extra_letter_candidates(lower)
        .into_iter()
        .filter(|candidate| {
            !looks_like_unsafe_first_letter_deletion(lower, candidate)
                && !looks_like_unsafe_leading_pair_deletion(lower, candidate)
                && !looks_like_unsafe_internal_y_deletion(lower, candidate)
                && !rewrites_protected_pattern_term_stem(lower, candidate)
                && !looks_like_unsafe_vowel_join_deletion(lower, candidate)
                && !looks_like_unsafe_chsh_deletion(lower, candidate)
        })
        .collect()
}

fn has_safe_adjacent_transposition_candidate(lower: &str) -> bool {
    let chars: Vec<char> = lower.chars().collect();
    if chars.len() < 5 {
        return false;
    }
    (0..chars.len().saturating_sub(1)).any(|idx| {
        if chars[idx] == chars[idx + 1] {
            return false;
        }
        let mut candidate = chars.clone();
        candidate.swap(idx, idx + 1);
        let candidate: String = candidate.into_iter().collect();
        candidate != lower
            && is_known_russian_word_or_form(&candidate)
            && ngram_allows_ru_candidate(
                &candidate,
                lower,
                super::super::thresholds::NGRAM_TRANSPOSE_MARGIN,
            )
    })
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

fn looks_like_unsafe_leading_pair_deletion(lower: &str, candidate: &str) -> bool {
    let chars = lower.chars().collect::<Vec<_>>();
    if chars.len() < 8 || candidate != chars[2..].iter().collect::<String>() {
        return false;
    }
    chars[0] != chars[1]
}

fn looks_like_unsafe_internal_y_deletion(lower: &str, candidate: &str) -> bool {
    let lower_chars = lower.chars().collect::<Vec<_>>();
    let candidate_chars = candidate.chars().collect::<Vec<_>>();
    if lower_chars.len() != candidate_chars.len() + 1 {
        return false;
    }
    for idx in 1..lower_chars.len().saturating_sub(1) {
        if lower_chars[idx] != 'й' {
            continue;
        }
        let without: String = lower_chars[..idx]
            .iter()
            .chain(lower_chars[idx + 1..].iter())
            .collect();
        if without == candidate {
            return true;
        }
    }
    false
}

fn looks_like_unsafe_vowel_join_deletion(lower: &str, candidate: &str) -> bool {
    if looks_like_present_tail(candidate) {
        return false;
    }
    let lower_chars = lower.chars().collect::<Vec<_>>();
    let candidate_chars = candidate.chars().collect::<Vec<_>>();
    if lower_chars.len() != candidate_chars.len() + 1 {
        return false;
    }
    for idx in 1..lower_chars.len().saturating_sub(1) {
        let without: String = lower_chars[..idx]
            .iter()
            .chain(lower_chars[idx + 1..].iter())
            .collect();
        if without == candidate
            && crate::russian_chars::is_russian_vowel(lower_chars[idx - 1])
            && crate::russian_chars::is_russian_vowel(lower_chars[idx + 1])
        {
            return true;
        }
    }
    false
}

fn looks_like_present_tail(word: &str) -> bool {
    const PRESENT_TAILS: &[&str] = &[
        "ется", "ётся", "атся", "ятся", "ешь", "ишь", "ете", "ите", "ают", "яют", "уют", "ют",
        "ут", "ат", "ят", "ает", "яет", "ует", "ет", "ит",
    ];
    PRESENT_TAILS.iter().any(|tail| word.ends_with(tail))
}

fn looks_like_unsafe_chsh_deletion(lower: &str, candidate: &str) -> bool {
    let lower_chars = lower.chars().collect::<Vec<_>>();
    let candidate_chars = candidate.chars().collect::<Vec<_>>();
    if lower_chars.len() != candidate_chars.len() + 1 {
        return false;
    }
    for idx in 1..lower_chars.len() {
        if lower_chars[idx - 1] != 'ч' || lower_chars[idx] != 'ш' {
            continue;
        }
        let without: String = lower_chars[..idx]
            .iter()
            .chain(lower_chars[idx + 1..].iter())
            .collect();
        if without == candidate {
            return true;
        }
    }
    false
}
