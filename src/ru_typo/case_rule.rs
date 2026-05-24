use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::text_case::apply_word_case;
use crate::word_reader::is_cyrillic_word;

pub(crate) fn correct_cyrillic_word_case(word: &str) -> Option<String> {
    if word.chars().count() < 2 || !is_cyrillic_word(word) {
        return None;
    }
    if word
        .chars()
        .all(|ch| !ch.is_alphabetic() || !ch.is_uppercase())
        || word
            .chars()
            .all(|ch| !ch.is_alphabetic() || ch.is_uppercase())
    {
        return None;
    }

    let lower = word.to_lowercase();
    if !is_known_russian_word_or_form(&lower) {
        return None;
    }

    let normalized = apply_word_case(word, &lower);
    (normalized != word).then_some(normalized)
}
