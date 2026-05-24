use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_candidates::generate_vowel_confusion_candidates;
use crate::russian_typo_scoring::best_unique_known_ngram_candidate;
use crate::word_reader::is_cyrillic_word;

use super::guards::looks_like_plausible_russian_past_tense;
use super::thresholds::NGRAM_VOWEL_CONFUSION_MARGIN;

pub(crate) fn correct_vowel_confusion(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }
    if looks_like_plausible_russian_past_tense(&lower) {
        return None;
    }

    best_unique_known_ngram_candidate(
        word,
        generate_vowel_confusion_candidates(&lower),
        NGRAM_VOWEL_CONFUSION_MARGIN,
    )
}
