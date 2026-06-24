use super::*;

#[test]
fn recognizes_adjective_plural_from_known_lemma() {
    assert!(is_known_russian_word_or_form("котовые"));
}

#[test]
fn recognizes_common_noun_forms_for_typo_candidates() {
    assert!(is_known_russian_word_or_form("кнопку"));
    assert!(is_known_russian_word_or_form("файлом"));
    assert!(is_known_russian_word_or_form("доставкой"));
}

#[test]
fn generates_ka_oblique_forms_for_prefix_candidates() {
    let forms = ka_oblique_forms_for_prefix("дос", 7, 10, 4096);
    assert!(
        forms.iter().any(|form| form == "доставкой"),
        "expected доставка -> доставкой in ka oblique forms, got {forms:?}"
    );
}
