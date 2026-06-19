use crate::phrase_lexicon::looks_like_short_function_word_glued_to_known_word;

#[path = "extra/candidates.rs"]
mod candidates;
#[path = "extra/scoring.rs"]
mod scoring;

pub(super) use candidates::extra_letter_candidate_exists;
use candidates::{reflexive_confusion_sources, safe_extra_letter_candidates};
use scoring::{best_common_extra_letter_candidate, best_extra_letter_candidate};

use super::guards::{
    correct_invalid_adjective_tail, looks_like_plausible_russian_past_tense, unknown_cyrillic_lower,
};
use super::missing::missing_letter_candidate_exists;

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
    best_extra_letter_candidate(word, safe_extra_letter_candidates(&lower))
}

pub fn repair_extra_letters_after_layout(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !crate::word_reader::is_cyrillic_word(word) {
        return None;
    }
    let lower = word.to_lowercase();
    best_common_extra_letter_candidate(word, safe_extra_letter_candidates(&lower))
}

#[cfg(test)]
mod tests {
    use super::repair_extra_letters_after_layout;

    #[test]
    fn layout_cleanup_can_repair_common_ru_word_outside_morphology_dictionary() {
        assert_eq!(
            repair_extra_letters_after_layout("стразу"),
            Some("сразу".to_string())
        );
    }
}
