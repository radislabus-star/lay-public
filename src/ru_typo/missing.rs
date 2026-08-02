use crate::russian_chars::is_russian_vowel;
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_candidates::{
    generate_missing_letter_candidates, inserted_char_position_for_missing_letter,
};
use crate::russian_typo_scoring::{
    best_ranked_dictionary_candidate, missing_letter_candidate_bonus,
};

use super::guards::{
    looks_like_plausible_russian_past_tense, looks_like_prefix_plus_known_russian_word,
    looks_like_present_or_reflexive_verb,
};
use super::thresholds::NGRAM_DICT_MISSING_LETTER_MARGIN;

pub fn correct_missing_letter(word: &str) -> Option<String> {
    if word.contains('-') || word.chars().count() < 4 || !crate::word_reader::is_cyrillic_word(word)
    {
        return None;
    }
    let lower = word.to_lowercase();
    let field_knows_original = crate::nanda_wave::l2::l2_surface_foundation_has_authority(&lower)
        || crate::russian_lexicon::is_center_backed_russian_form(&lower);
    if field_knows_original || crate::russian_lexicon::has_clean_russian_surface_certificate(&lower)
    {
        return None;
    }
    if looks_like_plausible_russian_past_tense(&lower)
        && !missing_letter_candidate_exists(word, &lower)
    {
        return None;
    }
    if looks_like_prefix_plus_known_russian_word(&lower)
        && !vowel_nonverb_missing_letter_candidate_exists(word, &lower)
    {
        return None;
    }

    if std::env::var_os("LAY_TRACE_RU_TYPO").is_some() {
        let ranked = safe_missing_letter_candidates(&lower)
            .filter(|candidate| {
                is_known_russian_word_or_form(candidate)
                    || crate::russian_lexicon::is_reference_backed_russian_form(candidate)
            })
            .map(|candidate| {
                let score = crate::ngram::ru_candidate_margin(&candidate, &lower)
                    + missing_letter_candidate_bonus(&lower, &candidate);
                (candidate, score)
            })
            .collect::<Vec<_>>();
        eprintln!("missing_letter word={word:?} field_knows_original={field_knows_original} ranked={ranked:?}");
    }

    best_ranked_dictionary_candidate(
        word,
        safe_missing_letter_candidates(&lower),
        NGRAM_DICT_MISSING_LETTER_MARGIN,
        0.40,
    )
}

#[cfg(test)]
fn has_common_missing_letter_candidate(lower: &str) -> bool {
    safe_missing_letter_candidates(lower).any(|candidate| {
        candidate != lower
            && crate::lexicon::is_common_ru_word(&candidate)
            && is_known_russian_word_or_form(&candidate)
    })
}

pub(super) fn missing_letter_candidate_exists(word: &str, lower: &str) -> bool {
    let original_lower = word.to_lowercase();
    safe_missing_letter_candidates(lower).any(|candidate| {
        candidate != original_lower
            && is_known_russian_word_or_form(&candidate)
            && crate::ngram::ru_candidate_margin(&candidate, &original_lower)
                + missing_letter_candidate_bonus(&original_lower, &candidate)
                >= NGRAM_DICT_MISSING_LETTER_MARGIN
    })
}

fn vowel_nonverb_missing_letter_candidate_exists(word: &str, lower: &str) -> bool {
    let original_lower = word.to_lowercase();
    safe_missing_letter_candidates(lower).any(|candidate| {
        let Some((_, inserted)) = inserted_char_position_for_missing_letter(lower, &candidate)
        else {
            return false;
        };
        is_russian_vowel(inserted)
            && !looks_like_present_or_reflexive_verb(&candidate)
            && candidate != original_lower
            && is_known_russian_word_or_form(&candidate)
            && crate::ngram::ru_candidate_margin(&candidate, &original_lower)
                + missing_letter_candidate_bonus(&original_lower, &candidate)
                >= NGRAM_DICT_MISSING_LETTER_MARGIN
    })
}

pub(crate) fn safe_missing_letter_candidates(lower: &str) -> impl Iterator<Item = String> + '_ {
    safe_missing_letter_candidates_impl(lower)
}

include!("missing/safety.rs");

#[cfg(test)]
#[path = "missing/tests.rs"]
mod tests;
