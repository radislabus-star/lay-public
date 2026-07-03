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
fn generates_ka_oblique_forms_for_prefix_candidates() {
    let forms = ka_oblique_forms_for_prefix("дос", 7, 10, 4096);
    assert!(
        forms.iter().any(|form| form == "доставкой"),
        "expected доставка -> доставкой in ka oblique forms, got {forms:?}"
    );
}

#[test]
fn recognizes_common_live_noun_case_forms() {
    assert!(is_known_russian_word_or_form("авиапорту"));
}
