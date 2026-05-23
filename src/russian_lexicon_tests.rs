use super::*;

#[test]
fn recognizes_adjective_plural_from_known_lemma() {
    assert!(is_known_russian_word_or_form("котовые"));
}
