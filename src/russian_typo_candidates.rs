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

/// Visits every lowercase-Russian surface at exact Damerau distance one.
///
/// This is a complete candidate frontier, not a ranker: there is no top-k
/// cutoff and the caller decides what evidence, if any, a generated surface
/// carries. The order favors cheap deletion/transposition matches before the
/// wider substitution and insertion frontiers.
pub(crate) fn any_single_damerau_edit_candidate(
    lower: &str,
    mut predicate: impl FnMut(&str) -> bool,
) -> bool {
    let chars = lower.chars().collect::<Vec<_>>();
    let mut candidate = String::with_capacity(lower.len() + 2);

    for removed in 0..chars.len() {
        candidate.clear();
        candidate.extend(chars[..removed].iter());
        candidate.extend(chars[removed + 1..].iter());
        if predicate(&candidate) {
            return true;
        }
    }

    let mut transposed = chars.clone();
    for left in 0..transposed.len().saturating_sub(1) {
        if transposed[left] == transposed[left + 1] {
            continue;
        }
        transposed.swap(left, left + 1);
        candidate.clear();
        candidate.extend(transposed.iter());
        transposed.swap(left, left + 1);
        if predicate(&candidate) {
            return true;
        }
    }

    for replaced in 0..chars.len() {
        for replacement in RU_ALPHABET {
            if replacement == chars[replaced] {
                continue;
            }
            candidate.clear();
            candidate.extend(chars[..replaced].iter());
            candidate.push(replacement);
            candidate.extend(chars[replaced + 1..].iter());
            if predicate(&candidate) {
                return true;
            }
        }
    }

    for inserted_at in 0..=chars.len() {
        for inserted in RU_ALPHABET {
            candidate.clear();
            candidate.extend(chars[..inserted_at].iter());
            candidate.push(inserted);
            candidate.extend(chars[inserted_at..].iter());
            if predicate(&candidate) {
                return true;
            }
        }
    }

    false
}

pub(crate) fn has_clean_single_damerau_edit_candidate(lower: &str) -> bool {
    any_single_damerau_edit_candidate(lower, |candidate| {
        // The shared clean certificate accepts attested and lexicon-backed
        // morphology surfaces, but excludes unconstrained generated material.
        // That makes it suitable as contradictory one-word evidence without
        // coupling the veto to a bounded candidate ranking.
        crate::russian_lexicon::has_clean_russian_surface_certificate(candidate)
    })
}

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

/// Returns true when a one-letter insertion creates an adjacent run of the
/// same Russian consonant.
///
/// This is transition geometry only. It can lower automatic authority when no
/// independent evidence proves that the input omitted a repeated consonant;
/// it must not generate, rank, or positively authorize a candidate.
pub(crate) fn missing_letter_inserts_adjacent_duplicate_consonant(
    lower: &str,
    candidate: &str,
) -> bool {
    let Some((idx, inserted)) = inserted_char_position_for_missing_letter(lower, candidate) else {
        return false;
    };
    if !crate::keyboard::is_cyrillic_letter(inserted)
        || crate::russian_chars::is_russian_vowel(inserted)
        || matches!(inserted, 'ь' | 'Ь' | 'ъ' | 'Ъ')
    {
        return false;
    }

    let candidate_chars = candidate.chars().collect::<Vec<_>>();
    idx.checked_sub(1)
        .and_then(|left| candidate_chars.get(left))
        .is_some_and(|left| *left == inserted)
        || candidate_chars
            .get(idx + 1)
            .is_some_and(|right| *right == inserted)
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

#[cfg(test)]
mod tests {
    use super::{
        any_single_damerau_edit_candidate, has_clean_single_damerau_edit_candidate,
        missing_letter_inserts_adjacent_duplicate_consonant,
    };

    #[test]
    fn complete_one_edit_frontier_covers_every_geometry_without_top_k() {
        for (original, expected) in [
            ("коот", "кот"),
            ("кто", "кот"),
            ("кит", "кот"),
            ("кт", "кот"),
        ] {
            assert!(
                any_single_damerau_edit_candidate(original, |candidate| candidate == expected),
                "missing exact one-edit surface: {original:?} -> {expected:?}"
            );
        }

        assert!(!any_single_damerau_edit_candidate("кот", |candidate| {
            candidate == "кот"
        }));
        assert!(!any_single_damerau_edit_candidate("кот", |candidate| {
            crate::text_metrics::damerau_levenshtein("кот", candidate) != 1
        }));
    }

    #[test]
    fn boundary_conflict_requires_an_attested_one_word_surface() {
        assert!(crate::russian_lexicon::has_clean_russian_surface_certificate("воротами"));
        assert!(has_clean_single_damerau_edit_candidate("воротаим"));

        assert!(!has_clean_single_damerau_edit_candidate("документыим"));
        assert!(!crate::russian_lexicon::has_clean_russian_surface_certificate("документним"));
        assert!(!crate::russian_lexicon::is_exact_reference_russian_word(
            "документним"
        ));
    }

    #[test]
    fn duplicate_consonant_insertion_geometry_is_source_neutral() {
        for (original, candidate) in [
            ("руских", "русских"),
            ("отточеная", "отточенная"),
            ("тон", "тонн"),
            ("тона", "тонна"),
        ] {
            assert!(missing_letter_inserts_adjacent_duplicate_consonant(
                original, candidate
            ));
        }

        for (original, candidate) in [
            ("протколах", "протоколах"),
            ("дейстия", "действия"),
            ("лушее", "лучшее"),
            ("тона", "торна"),
            ("тон", "тонны"),
        ] {
            assert!(!missing_letter_inserts_adjacent_duplicate_consonant(
                original, candidate
            ));
        }
    }
}
