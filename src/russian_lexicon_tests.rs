use super::*;

#[test]
fn short_passive_participle_is_backed_by_attested_long_form() {
    assert!(is_reference_backed_short_passive_participle("подключен"));
    assert!(is_reference_backed_short_passive_participle("подключена"));
    assert!(is_reference_backed_short_passive_participle("подлечен"));
    assert!(!is_reference_backed_short_passive_participle("подлюген"));
}

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
fn clean_surface_certificate_recognizes_short_noun_forms() {
    assert!(has_clean_russian_surface_certificate("коды"));
    assert!(has_clean_russian_surface_certificate("теорию"));
    assert!(has_clean_russian_surface_certificate("задачки"));
    assert!(has_clean_russian_surface_certificate("проверь"));
    assert!(is_reference_backed_russian_form("фактическим"));
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

#[test]
fn recognizes_imperative_i_forms_from_backed_et_lemmas() {
    for word in ["смотри", "посмотри", "досмотри", "подсмотри", "просмотри"]
    {
        assert!(
            is_known_russian_word_or_form(word),
            "missing imperative -и form from backed -еть lemma: {word:?}"
        );
    }
}

#[test]
fn clean_surface_certificate_recognizes_attested_consonant_alternations() {
    for word in ["могли", "скажу", "пиши", "китайцев"] {
        assert!(
            has_clean_russian_surface_certificate(word),
            "missing clean morphology certificate: {word:?}"
        );
    }
}

#[test]
fn generated_form_reference_does_not_promote_known_dirty_inputs() {
    let forms = full_russian_generated_form_dictionary();
    for dirty in ["пукнт", "звгрузи", "эсперемнт", "труссс"] {
        assert!(
            !forms.contains(dirty),
            "dirty input leaked into generated-form reference: {dirty:?}"
        );
    }
}
