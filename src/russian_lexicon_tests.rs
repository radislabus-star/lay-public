use super::*;

#[test]
fn attested_verb_forms_use_current_ime_lemma_authority_without_full_generated_forms() {
    let previous = crate::hot_field::process_policy();
    crate::hot_field::set_process_policy(crate::hot_field::HotFieldPolicy::ime());
    let generated_before = russian_generated_form_dictionary_is_warm();
    for (surface, lemma) in [
        ("несу", "нести"),
        ("везу", "везти"),
        ("ползу", "ползти"),
        ("трясу", "трясти"),
        ("привезу", "привезти"),
        ("отвезу", "отвезти"),
    ] {
        assert!(
            crate::nanda_wave::l2::l2_surface_foundation_has_authority(lemma),
            "missing live lemma {lemma}"
        );
        assert!(
            is_center_backed_russian_form(surface),
            "missing center-backed {surface}"
        );
        assert!(
            is_attested_center_backed_russian_form(surface),
            "missing attested center proof {surface}"
        );
        assert!(
            is_reference_backed_russian_form(surface),
            "missing reference-backed {surface}"
        );
        assert!(
            is_known_russian_word_or_form(surface),
            "missing public form {surface}"
        );
    }
    assert_eq!(
        russian_generated_form_dictionary_is_warm(),
        generated_before
    );
    crate::hot_field::set_process_policy(previous);
}

#[test]
fn inflection_metadata_initialization_is_shared_and_reports_its_bounded_cost() {
    let started = std::time::Instant::now();
    let gate = std::sync::Arc::new(std::sync::Barrier::new(9));
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let gate = gate.clone();
            std::thread::spawn(move || {
                gate.wait();
                russian_inflection_metadata().unwrap() as *const RussianInflectionMetadata as usize
            })
        })
        .collect();
    gate.wait();
    let pointers: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    assert!(pointers.iter().all(|pointer| *pointer == pointers[0]));
    let metadata = russian_inflection_metadata().unwrap();
    let pairs: usize = metadata
        .attested_forms
        .values()
        .map(std::collections::HashSet::len)
        .sum();
    let payload_bytes: usize = metadata
        .attested_forms
        .iter()
        .map(|(lemma, surfaces)| lemma.len() + surfaces.iter().map(String::len).sum::<usize>())
        .sum();
    assert!((1..=4096).contains(&pairs));
    eprintln!("attested_inflection_cost pairs={pairs} utf8_payload_bytes={payload_bytes} cold_shared_init_us={} os_cache=uncontrolled", started.elapsed().as_micros());
}

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
fn recognizes_regular_a_ya_noun_inflections_from_backed_lemmas() {
    for word in [
        "воде",
        "воды",
        "воду",
        "водой",
        "команде",
        "команду",
        "земле",
        "неделе",
        "истории",
        "окне",
        "дела",
        "слова",
    ] {
        assert!(
            has_clean_russian_surface_certificate(word),
            "missing clean regular noun inflection certificate: {word:?}"
        );
    }

    for invalid in ["историе", "армие", "станцие", "рукы", "ногы", "дачы"]
    {
        assert!(
            !has_clean_russian_surface_certificate(invalid),
            "invalid -ия noun form received a clean certificate: {invalid:?}"
        );
    }
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
fn clean_surface_certificate_respects_adjective_inflection_spelling() {
    assert!(has_clean_russian_surface_certificate("фактическим"));
    assert!(has_clean_russian_surface_certificate("точнее"));
    assert!(has_clean_russian_surface_certificate("воротами"));
    assert!(!has_clean_russian_surface_certificate("документним"));
    assert!(!has_clean_russian_surface_certificate("точние"));
}

#[test]
fn adjective_surface_certificate_respects_encoded_suffix_and_lemma_classes() {
    let valid = [
        // Hard -ый paradigm and its regular comparative.
        "точного",
        "точным",
        "точные",
        "точнее",
        // Soft -ий paradigm.
        "синего",
        "синим",
        "синяя",
        "синее",
        // Velar -ий stems use hard singular endings and и in the remaining forms.
        "русского",
        "русскому",
        "русском",
        "русской",
        "русская",
        "русское",
        "русским",
        "русскими",
        "русские",
        "русских",
        "тихого",
        "легкого",
        "фактического",
        // Sibilant -ий stems keep -его/-ему but use -ая.
        "хорошего",
        "хорошая",
        "хорошее",
        "свежего",
        "свежая",
        "свежее",
        // Restricted-stem -ой lemmas mix hard singular and и-spelled forms.
        "плохого",
        "плохим",
        "чужого",
        "чужим",
        // Hunspell class O marks the soft-sign possessive -ий paradigm.
        "птичьего",
        "птичья",
        "птичье",
        "птичьи",
        "птичьим",
        "птичьими",
        "птичьих",
        "волчьего",
        "медвежьего",
    ];
    for word in valid {
        assert!(
            is_reference_backed_russian_form(word),
            "valid adjective form is not reference-backed: {word:?}"
        );
        assert!(
            has_clean_russian_surface_certificate(word),
            "valid adjective form lacks a clean certificate: {word:?}"
        );
    }

    let invalid = [
        // A velar -ий stem cannot take soft singular endings.
        "тихего",
        "русскего",
        "русскему",
        "русскем",
        "русскей",
        "русскяя",
        "русскее",
        "тихее",
        "легкее",
        "фактическее",
        // A regular -ый lemma needs Hunspell class E before -ее may be
        // treated as its synthetic comparative.
        "почтовее",
        "атомнее",
        "даннее",
        "школьнее",
        // A restricted-stem -ой lemma cannot take ы-spelled plural endings.
        "плохым",
        "плохыми",
        "плохые",
        "плохых",
        // Nor can it take the soft singular endings of an -ий lemma.
        "чужего",
        "чужему",
        "чужем",
        "чужей",
        "чужяя",
        // Class O lemmas cannot use the no-soft-sign class A surfaces.
        "птичего",
        "птичая",
        "птичее",
        "птичие",
        "волчего",
        "медвежего",
        "медвежая",
        // Existing mixed-paradigm regressions remain rejected.
        "фактическыми",
        "русскыми",
        "хорошяя",
        "точние",
        "документним",
    ];
    for word in invalid {
        assert!(
            !is_reference_backed_russian_form(word),
            "invalid adjective form became reference-backed: {word:?}"
        );
        assert!(
            !has_clean_russian_surface_certificate(word),
            "invalid adjective form received a clean certificate: {word:?}"
        );
    }
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
    for word in [
        "можем",
        "может",
        "можешь",
        "поможем",
        "наполняю",
        "наполняешь",
        "наполняет",
        "наполняем",
        "наполняете",
        "наполняют",
    ] {
        assert!(
            is_known_russian_word_or_form(word),
            "missing backed present form from L2 foundation lemma: {word:?}"
        );
        if word.starts_with("наполня") {
            assert!(
                full_russian_generated_form_dictionary().contains(word),
                "valid -ять present form missing from generated Hunspell authority: {word:?}"
            );
        }
    }

    for invalid in [
        "стояю",
        "стояешь",
        "стояет",
        "стояем",
        "стояете",
        "стояют",
        "состояет",
        "выстояет",
        "засеяет",
        "залаяет",
    ] {
        assert!(
            !full_russian_dictionary().contains(invalid),
            "wrong -ять form entered exact dictionary authority: {invalid:?}"
        );
        assert!(
            !full_russian_generated_form_dictionary().contains(invalid),
            "wrong -ять form entered generated Hunspell authority: {invalid:?}"
        );
        assert!(
            !forms::is_known_russian_form(invalid),
            "wrong -ять form entered morphology authority: {invalid:?}"
        );
        assert!(
            !crate::lexicon::is_ru_technical_loanword(invalid),
            "wrong -ять form entered technical-word authority: {invalid:?}"
        );
        assert!(
            !is_known_russian_word_or_form(invalid),
            "wrong -ять conjugation was promoted to known: {invalid:?}"
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
