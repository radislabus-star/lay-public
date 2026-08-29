//! Candidate generators for Russian typo correction.
//!
//! This module only creates possible word forms. It does not rank candidates
//! and does not know whether a candidate is safe enough to apply.

use std::collections::HashSet;

use crate::russian_chars::is_russian_vowel;

pub(crate) const RU_ALPHABET: [char; 33] = [
    'а', 'б', 'в', 'г', 'д', 'е', 'ё', 'ж', 'з', 'и', 'й', 'к', 'л', 'м', 'н', 'о', 'п', 'р', 'с',
    'т', 'у', 'ф', 'х', 'ц', 'ч', 'ш', 'щ', 'ъ', 'ы', 'ь', 'э', 'ю', 'я',
];

pub(crate) fn repeated_run_deletion_candidates(lower: &str) -> Vec<String> {
    let chars: Vec<char> = lower.chars().collect();
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();
    let mut idx = 0usize;

    while idx < chars.len() {
        let mut end = idx + 1;
        while end < chars.len() && chars[end] == chars[idx] {
            end += 1;
        }

        let run_len = end - idx;
        if run_len > 1 {
            for keep in 1..run_len {
                let mut candidate = String::with_capacity(lower.len());
                candidate.extend(chars[..idx].iter());
                candidate.extend(std::iter::repeat_n(chars[idx], keep));
                candidate.extend(chars[end..].iter());
                if seen.insert(candidate.clone()) {
                    candidates.push(candidate);
                }
            }
        }

        idx = end;
    }

    candidates
}

pub(crate) fn generate_missing_letter_candidates(lower: &str) -> impl Iterator<Item = String> + '_ {
    let chars: Vec<char> = lower.chars().collect();
    (0..=chars.len()).flat_map(move |idx| {
        RU_ALPHABET.into_iter().map({
            let chars = chars.clone();
            move |inserted| {
                let mut candidate = String::with_capacity(lower.len() + inserted.len_utf8());
                candidate.extend(chars[..idx].iter());
                candidate.push(inserted);
                candidate.extend(chars[idx..].iter());
                candidate
            }
        })
    })
}

pub(crate) fn generate_extra_letter_candidates(lower: &str) -> Vec<String> {
    let chars: Vec<char> = lower.chars().collect();
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();

    for idx in 0..chars.len() {
        if idx > 1 && is_russian_vowel(chars[idx]) {
            continue;
        }
        let mut candidate = String::with_capacity(lower.len());
        candidate.extend(chars[..idx].iter());
        candidate.extend(chars[idx + 1..].iter());
        if seen.insert(candidate.clone()) {
            candidates.push(candidate);
        }
    }

    if chars.len() >= 10 {
        for idx in 0..=chars.len() - 2 {
            if idx + 2 == chars.len() {
                continue;
            }
            if chars[idx..idx + 2].iter().all(|ch| is_russian_vowel(*ch)) {
                continue;
            }
            let mut candidate = String::with_capacity(lower.len());
            candidate.extend(chars[..idx].iter());
            candidate.extend(chars[idx + 2..].iter());
            if candidate.chars().count() < 8 {
                continue;
            }
            if seen.insert(candidate.clone()) {
                candidates.push(candidate);
            }
        }
    }

    for candidate in repeated_run_deletion_candidates(lower) {
        if seen.insert(candidate.clone()) {
            candidates.push(candidate);
        }
    }

    candidates
}

pub(crate) fn generate_vowel_confusion_candidates(lower: &str) -> Vec<String> {
    let chars: Vec<char> = lower.chars().collect();
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();

    for idx in 1..chars.len() {
        for replacement in ru_vowel_confusion_replacements(chars[idx]).iter().copied() {
            let mut candidate = chars.clone();
            candidate[idx] = replacement;
            let candidate: String = candidate.into_iter().collect();
            if seen.insert(candidate.clone()) {
                candidates.push(candidate);
            }
        }
    }

    candidates
}

pub(crate) fn generate_hard_sign_candidates(lower: &str) -> impl Iterator<Item = String> + '_ {
    let chars: Vec<char> = lower.chars().collect();
    (0..chars.len().saturating_sub(1)).filter_map(move |idx| {
        if chars[idx] != 'ь' || !matches!(chars[idx + 1], 'е' | 'ё' | 'ю' | 'я') {
            return None;
        }
        let mut candidate = chars.clone();
        candidate[idx] = 'ъ';
        Some(candidate.into_iter().collect())
    })
}

pub(crate) fn inserted_char_position_for_missing_letter(
    lower: &str,
    candidate: &str,
) -> Option<(usize, char)> {
    let lower_chars: Vec<char> = lower.chars().collect();
    let candidate_chars: Vec<char> = candidate.chars().collect();
    if candidate_chars.len() != lower_chars.len() + 1 {
        return None;
    }

    let mut i = 0usize;
    let mut j = 0usize;
    let mut inserted = None;
    while i < lower_chars.len() && j < candidate_chars.len() {
        if lower_chars[i] == candidate_chars[j] {
            i += 1;
            j += 1;
        } else if inserted.is_none() {
            inserted = Some((i, candidate_chars[j]));
            j += 1;
        } else {
            return None;
        }
    }
    if inserted.is_none() && j < candidate_chars.len() {
        inserted = Some((i, candidate_chars[j]));
    }
    inserted
}

fn ru_vowel_confusion_replacements(ch: char) -> &'static [char] {
    match ch {
        'а' => &['о'],
        'о' => &['а'],
        'е' => &['и', 'ё'],
        'и' => &['е'],
        'у' => &['о'],
        'ё' => &['е'],
        _ => &[],
    }
}
