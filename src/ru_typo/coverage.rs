use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_candidates::{
    generate_extra_letter_candidates, generate_vowel_confusion_candidates,
};
use crate::russian_typo_scoring::missing_letter_candidate_bonus;
use crate::word_reader::is_cyrillic_word;

use super::extra::correct_extra_letters;
use super::hard_sign::correct_hard_sign_typo;
use super::memo::{memoized_bool, WordMaterialKind};
use super::missing::{
    correct_missing_letter, propose_missing_letter_candidate, safe_missing_letter_candidates,
};
use super::repeated::correct_repeated_letter;
use super::substitution::correct_single_letter_substitution;
use super::thresholds::{NGRAM_DICT_MISSING_LETTER_MARGIN, NGRAM_EXTRA_LETTER_MARGIN};
use super::transposition::correct_adjacent_transposition;
use super::verb::correct_verb_ending_confusion;
use super::vowel::correct_vowel_confusion;

pub(crate) fn has_plausible_russian_typo_candidate(lower: &str) -> bool {
    memoized_bool(WordMaterialKind::Plausible, lower, || {
        has_plausible_russian_typo_candidate_uncached(lower)
    })
}

fn has_plausible_russian_typo_candidate_uncached(lower: &str) -> bool {
    if lower.chars().count() < 5 || !is_cyrillic_word(lower) || is_known_russian_word_or_form(lower)
    {
        return false;
    }

    correct_verb_ending_confusion(lower).is_some()
        || correct_hard_sign_typo(lower).is_some()
        || correct_repeated_letter(lower).is_some()
        || correct_adjacent_transposition(lower).is_some()
        || correct_missing_letter(lower).is_some()
        || propose_missing_letter_candidate(lower).is_some()
        || correct_single_letter_substitution(lower).is_some()
        || correct_vowel_confusion(lower).is_some()
        || correct_extra_letters(lower).is_some()
        || has_generated_russian_typo_candidate(lower)
}

fn has_generated_russian_typo_candidate(lower: &str) -> bool {
    safe_missing_letter_candidates(lower).any(|candidate| {
        candidate != lower
            && is_known_russian_word_or_form(&candidate)
            && crate::ngram::ru_candidate_margin(&candidate, lower)
                + missing_letter_candidate_bonus(lower, &candidate)
                >= NGRAM_DICT_MISSING_LETTER_MARGIN
    }) || generate_vowel_confusion_candidates(lower)
        .into_iter()
        .any(|candidate| candidate != lower && is_known_russian_word_or_form(&candidate))
        || generate_extra_letter_candidates(lower)
            .into_iter()
            .any(|candidate| {
                candidate != lower
                    && is_known_russian_word_or_form(&candidate)
                    && crate::ngram::ru_candidate_margin(&candidate, lower)
                        >= NGRAM_EXTRA_LETTER_MARGIN
            })
}
