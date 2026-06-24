use std::collections::HashSet;

use crate::russian_lexicon::{
    is_known_russian_word_or_form, ka_oblique_forms_for_prefix, russian_dictionary,
    russian_generated_form_dictionary,
};
use crate::text_metrics::damerau_levenshtein;

const MAX_FUZZY_EDIT_DISTANCE: usize = 3;
const MAX_PREFIX_CANDIDATES_PER_DICT: usize = 4096;

pub(crate) fn fuzzy_known_word_candidates(lower: &str) -> Vec<String> {
    let len = lower.chars().count();
    if len > 12 {
        return Vec::new();
    }
    let prefix = lower.chars().take(3).collect::<String>();
    let min_len = len.saturating_sub(1).max(7);
    let max_len = len + 2;

    let mut seen = HashSet::new();
    let mut out = Vec::new();
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
    out
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
    fn damerau_counts_adjacent_swap_as_one_edit() {
        assert_eq!(damerau_levenshtein("йо", "ой"), 1);
    }
}
