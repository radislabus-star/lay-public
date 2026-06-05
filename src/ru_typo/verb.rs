use crate::data_lines::data_lines;
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_scoring::ngram_allows_ru_candidate;
use crate::text_case::apply_word_case;
use crate::word_reader::is_cyrillic_word;

use super::thresholds::NGRAM_VERB_ENDING_MARGIN;

const REFLEXIVE_CONFUSION_DATA: &str =
    include_str!("../../data/lexicon/russian_reflexive_confusion.tsv");
const VERB_ENDING_CONFUSION_DATA: &str =
    include_str!("../../data/lexicon/russian_verb_ending_confusion.tsv");

pub(crate) fn correct_verb_ending_confusion(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }

    for (from, to) in suffix_pairs(REFLEXIVE_CONFUSION_DATA) {
        let Some(stem) = lower.strip_suffix(from) else {
            continue;
        };
        if stem.chars().count() >= 3 {
            let candidate = format!("{stem}{to}");
            if is_known_russian_word_or_form(&candidate)
                && ngram_allows_ru_candidate(&candidate, &lower, NGRAM_VERB_ENDING_MARGIN)
            {
                return Some(apply_word_case(word, &candidate));
            }
        }
    }

    for (from, to) in suffix_pairs(VERB_ENDING_CONFUSION_DATA) {
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

fn suffix_pairs(data: &'static str) -> impl Iterator<Item = (&'static str, &'static str)> {
    data_lines(data).filter_map(|line| line.split_once('\t'))
}
