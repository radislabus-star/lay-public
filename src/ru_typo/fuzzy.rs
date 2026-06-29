use std::collections::HashSet;

use crate::russian_lexicon::{
    is_known_russian_word_or_form, ka_oblique_forms_for_prefix, russian_dictionary,
    russian_generated_form_dictionary,
};
use crate::russian_typo_candidates::{
    generate_missing_letter_candidates, generate_vowel_confusion_candidates,
    repeated_run_deletion_candidates, RU_ALPHABET,
};
use crate::text_metrics::damerau_levenshtein;

const MAX_FUZZY_EDIT_DISTANCE: usize = 3;
const MAX_PREFIX_CANDIDATES_PER_DICT: usize = 4096;

pub(crate) fn fuzzy_known_word_candidates(lower: &str) -> Vec<String> {
    let len = lower.chars().count();
    if len > 12 {
        return Vec::new();
    }
    let prefixes = fuzzy_prefixes(lower);
    let min_len = len.saturating_sub(1).max(7);
    let max_len = len + 2;

    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for prefix in prefixes {
        for candidate in russian_dictionary()
            .prefix_words(&prefix, min_len, max_len, MAX_PREFIX_CANDIDATES_PER_DICT)
            .into_iter()
            .chain(russian_generated_form_dictionary().prefix_words(
                &prefix,
                min_len,
                max_len,
                MAX_PREFIX_CANDIDATES_PER_DICT,
            ))
            .chain(ka_oblique_forms_for_prefix(
                &prefix,
                min_len,
                max_len,
                MAX_PREFIX_CANDIDATES_PER_DICT,
            ))
        {
            if !seen.insert(candidate.clone()) || !is_known_russian_word_or_form(&candidate) {
                continue;
            }
            if len.abs_diff(candidate.chars().count()) > MAX_FUZZY_EDIT_DISTANCE {
                continue;
            }
            let distance = damerau_levenshtein(lower, &candidate);
            if distance == 0 || distance > MAX_FUZZY_EDIT_DISTANCE {
                continue;
            }
            out.push(candidate);
        }
    }
    for candidate in local_known_typo_candidates(lower) {
        if seen.insert(candidate.clone()) {
            out.push(candidate);
        }
    }
    if len >= 7 {
        for candidate in crate::lexicon::common_ru_words_iter() {
            if !seen.insert(candidate.to_string()) || !is_known_russian_word_or_form(candidate) {
                continue;
            }
            if len.abs_diff(candidate.chars().count()) > MAX_FUZZY_EDIT_DISTANCE {
                continue;
            }
            let distance = damerau_levenshtein(lower, candidate);
            if distance == 0 || distance > MAX_FUZZY_EDIT_DISTANCE {
                continue;
            }
            out.push(candidate.to_string());
        }
    }
    out
}

fn local_known_typo_candidates(lower: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for candidate in generate_missing_letter_candidates(lower)
        .chain(generate_vowel_confusion_candidates(lower))
        .chain(generate_adjacent_transposition_candidates(lower))
        .chain(generate_keyboard_neighbor_substitution_candidates(lower))
    {
        if candidate == lower
            || !seen.insert(candidate.clone())
            || !is_known_russian_word_or_form(&candidate)
        {
            continue;
        }
        let distance = damerau_levenshtein(lower, &candidate);
        if distance == 0 || distance > MAX_FUZZY_EDIT_DISTANCE {
            continue;
        }
        out.push(candidate);
    }
    out
}

fn generate_adjacent_transposition_candidates(lower: &str) -> Vec<String> {
    let chars: Vec<char> = lower.chars().collect();
    let mut out = Vec::new();
    for idx in 0..chars.len().saturating_sub(1) {
        if chars[idx] == chars[idx + 1] {
            continue;
        }
        let mut candidate = chars.clone();
        candidate.swap(idx, idx + 1);
        out.push(candidate.into_iter().collect());
    }
    out
}

fn generate_keyboard_neighbor_substitution_candidates(lower: &str) -> Vec<String> {
    let chars: Vec<char> = lower.chars().collect();
    let mut out = Vec::new();
    for idx in 0..chars.len() {
        for replacement in RU_ALPHABET {
            if replacement == chars[idx]
                || !super::keyboard::are_ru_keyboard_neighbors(chars[idx], replacement)
            {
                continue;
            }
            let mut candidate = chars.clone();
            candidate[idx] = replacement;
            out.push(candidate.into_iter().collect());
        }
    }
    out
}

fn fuzzy_prefixes(lower: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut prefixes = Vec::with_capacity(4);
    for source in std::iter::once(lower.to_string()).chain(repeated_run_deletion_candidates(lower))
    {
        let prefix = source.chars().take(3).collect::<String>();
        if prefix.chars().count() == 3 && seen.insert(prefix.clone()) {
            prefixes.push(prefix);
        }
    }
    prefixes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_known_word_candidates_include_delivery_form() {
        let candidates = fuzzy_known_word_candidates("досвкйо");
        assert!(candidates.iter().any(|candidate| candidate == "доставкой"));
    }

    #[test]
    fn fuzzy_known_word_candidates_use_repeated_prefix_repair() {
        let candidates = fuzzy_known_word_candidates("ппоникаешь");
        assert!(candidates.iter().any(|candidate| candidate == "понимаешь"));
    }

    #[test]
    fn fuzzy_known_word_candidates_can_recover_common_word_with_broken_prefix() {
        let candidates = fuzzy_known_word_candidates("эсперемнт");
        assert!(candidates
            .iter()
            .any(|candidate| candidate == "эксперимент"));
    }

    #[test]
    fn fuzzy_known_word_candidates_can_recover_generated_forms() {
        let candidates = fuzzy_known_word_candidates("руских");
        assert!(candidates.iter().any(|candidate| candidate == "русских"));

        let candidates = fuzzy_known_word_candidates("звгрузи");
        assert!(candidates.iter().any(|candidate| candidate == "загрузи"));
    }

    #[test]
    fn damerau_counts_adjacent_swap_as_one_edit() {
        assert_eq!(damerau_levenshtein("йо", "ой"), 1);
    }
}
