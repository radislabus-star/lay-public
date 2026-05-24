use crate::russian_typo_candidates::generate_hard_sign_candidates;
use crate::russian_typo_scoring::best_unique_known_ngram_candidate;
use crate::word_reader::is_cyrillic_word;

use super::thresholds::NGRAM_HARD_SIGN_MARGIN;

pub(crate) fn correct_hard_sign_typo(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    best_unique_known_ngram_candidate(
        word,
        generate_hard_sign_candidates(&lower),
        NGRAM_HARD_SIGN_MARGIN,
    )
}
