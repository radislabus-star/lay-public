use super::*;

#[test]
fn field_only_authority_does_not_expose_reference_dictionaries() {
    let authority = crate::hot_field::HotAuthority::FieldSnapshotOnly;

    assert_eq!(russian_dictionary_for_authority(authority).len(), 0);
    assert_eq!(russian_short_dictionary_for_authority(authority).len(), 0);
    assert_eq!(russian_tiny_dictionary_for_authority(authority).len(), 0);
    assert_eq!(
        russian_generated_form_dictionary_for_authority(authority).len(),
        0
    );
}

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
fn recognizes_russian_technical_loanword_forms() {
    for word in [
        "грокать",
        "грокаем",
        "грокнулся",
        "грокалось",
        "грокингом",
        "гроканье",
        "пушить",
        "пушил",
        "запушил",
        "запушенный",
        "бейса",
        "скилы",
        "скилами",
        "тестить",
        "спектрал",
        "чате",
        "коммит",
        "едит",
        "лэем",
        "продакшене",
    ] {
        assert!(
            is_known_russian_word_or_form(word),
            "missing technical loanword form: {word:?}"
        );
    }
}

#[test]
fn recognizes_common_live_noun_case_forms() {
    assert!(is_known_russian_word_or_form("авиапорту"));
}

#[test]
fn recognizes_ch_verb_present_forms_from_l2_foundation_lemmas() {
    for word in ["можем", "может", "можешь", "поможем"] {
        assert!(
            is_known_russian_word_or_form(word),
            "missing -чь present form from L2 foundation lemma: {word:?}"
        );
    }
}
