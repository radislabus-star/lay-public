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
    if word.chars().count() < 6 || !crate::word_reader::is_cyrillic_word(word) {
        return None;
    }
    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) && !has_common_missing_letter_candidate(&lower) {
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

    best_ranked_dictionary_candidate(
        word,
        safe_missing_letter_candidates(&lower),
        NGRAM_DICT_MISSING_LETTER_MARGIN,
        0.40,
    )
}

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
    generate_missing_letter_candidates(lower)
        .filter(move |candidate| is_safe_missing_letter_candidate(lower, candidate))
}

fn is_safe_missing_letter_candidate(lower: &str, candidate: &str) -> bool {
    if let Some((idx, inserted)) = inserted_char_position_for_missing_letter(lower, candidate) {
        if is_risky_vowel_insert_into_verb_tail(lower, inserted) {
            return false;
        }
        if idx == lower.chars().count() {
            if lower.ends_with("ств") {
                return false;
            }
            return is_russian_vowel(inserted)
                && lower
                    .chars()
                    .last()
                    .is_some_and(|last| !is_russian_vowel(last));
        }
    }
    if let Some(inserted) = candidate.strip_suffix(lower) {
        if inserted == "о" && is_known_russian_word_or_form(candidate) {
            return true;
        }
        return inserted.chars().count() != 1 || lower.chars().next().is_some_and(is_russian_vowel);
    }

    true
}

fn is_risky_vowel_insert_into_verb_tail(lower: &str, inserted: char) -> bool {
    is_russian_vowel(inserted)
        && [
            "аешь", "яешь", "еешь", "оешь", "уешь", "ешь", "ишь", "еет", "ает", "яет", "ует",
        ]
        .iter()
        .any(|tail| lower.ends_with(tail))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_letter_repairs_known_weak_form_to_common_word() {
        let candidates = safe_missing_letter_candidates("недоказно").collect::<Vec<_>>();
        assert!(
            candidates.iter().any(|candidate| candidate == "недоказано"),
            "safe missing-letter candidates: {candidates:?}"
        );
        assert!(
            has_common_missing_letter_candidate("недоказно"),
            "expected common missing-letter candidate"
        );
        assert_eq!(
            correct_missing_letter("недоказно").as_deref(),
            Some("недоказано")
        );
    }
}
