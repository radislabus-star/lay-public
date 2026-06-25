use crate::phrase_lexicon::is_common_short_russian_preposition;

pub(super) fn starts_with_multi_letter_preposition(parts: &[&str]) -> bool {
    parts
        .first()
        .is_some_and(|part| part.chars().count() >= 2 && is_common_short_russian_preposition(part))
}

pub(super) fn starts_with_multi_letter_preposition_text(text: &str) -> bool {
    text.split_whitespace()
        .next()
        .is_some_and(|part| part.chars().count() >= 2 && is_common_short_russian_preposition(part))
}
