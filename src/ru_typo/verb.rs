use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_scoring::ngram_allows_ru_candidate;
use crate::text_case::apply_word_case;
use crate::word_reader::is_cyrillic_word;

use super::thresholds::NGRAM_VERB_ENDING_MARGIN;

pub(crate) fn correct_verb_ending_confusion(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }

    if let Some(stem) = lower.strip_suffix("тся") {
        if stem.chars().count() >= 3 {
            let candidate = format!("{stem}ться");
            if is_known_russian_word_or_form(&candidate)
                && ngram_allows_ru_candidate(&candidate, &lower, NGRAM_VERB_ENDING_MARGIN)
            {
                return Some(apply_word_case(word, &candidate));
            }
        }
    }

    for (from, to) in [("ешь", "ишь"), ("ет", "ит")] {
        let Some(stem) = lower.strip_suffix(from) else {
            continue;
        };
        if stem.chars().count() < 3 {
            continue;
        }
        let candidate = format!("{stem}{to}");
        if !is_known_russian_word_or_form(&candidate) {
            continue;
        }
        if !ngram_allows_ru_candidate(&candidate, &lower, NGRAM_VERB_ENDING_MARGIN) {
            continue;
        }
        return Some(apply_word_case(word, &candidate));
    }

    None
}
