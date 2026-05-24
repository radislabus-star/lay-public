use super::*;

#[test]
fn applies_builtin_auto_replace_with_trailing_space() {
    assert_eq!(
        apply_auto_replace("gjlk.xbcm ", "подлючись "),
        Some("подключись ".to_string())
    );
    assert_eq!(apply_auto_replace("Tcnm ", "Есть "), None);
}

#[test]
fn typing_assist_uses_exact_rules_only() {
    assert_eq!(
        apply_typing_assist_exact("подлючись "),
        Some("подключись ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("Надйи "),
        Some("Найди ".to_string())
    );
    assert_eq!(apply_typing_assist_exact("нормально "), None);
    assert_eq!(apply_typing_assist_exact("Есть "), None);
}

#[test]
fn russian_suffix_forms_are_known_candidates() {
    assert!(is_known_russian_word_or_form("препаратов"));
    assert!(is_known_russian_word_or_form("кнопками"));
    assert!(is_known_russian_word_or_form("могу"));
    assert!(is_known_russian_word_or_form("помогу"));
    assert!(is_known_russian_word_or_form("видишь"));
    assert!(is_known_russian_word_or_form("значит"));
    assert!(is_known_russian_word_or_form("страдает"));
    assert!(is_known_russian_word_or_form("установки"));
}

#[test]
fn typing_assist_auto_switch_blocks_plain_layout_words_and_keeps_explicit_cases() {
    for input in [
        "njkmrj ",
        "vjue ",
        "yt ",
        "hf,jnftn ",
        "'nj ",
        "Lfdfq ",
        "lfkmit ",
    ] {
        assert_eq!(
            apply_typing_assist(input, true),
            None,
            "plain layout word must not be auto-switched: {input:?}"
        );
    }

    assert_eq!(
        apply_typing_assist("double b ", true),
        Some("double и ".to_string())
    );
    for row in fixture_rows("daemon_typing_assist_tail_cases.tsv") {
        assert_eq!(row.len(), 2, "tail cases fixture must be TSV");
        assert_eq!(
            apply_typing_assist_to_text_tail(&row[0]),
            Some(row[1].clone())
        );
    }
    assert_eq!(
        apply_typing_assist("ашду ", true),
        Some("file ".to_string())
    );
    assert_eq!(
        apply_typing_assist("ашдуы ", true),
        Some("files ".to_string())
    );
    assert_eq!(
        apply_typing_assist("еукьштфд ", true),
        Some("terminal ".to_string())
    );
    assert_eq!(
        apply_typing_assist("сфкпщ ", true),
        Some("cargo ".to_string())
    );
    assert_eq!(
        apply_typing_assist("ОБYJDB ", true),
        Some("ОБНОВИ ".to_string())
    );
    assert_eq!(
        apply_typing_assist("CRBK ", true),
        Some("СКИЛ ".to_string())
    );
    for row in fixture_rows("daemon_typing_assist_layout_explicit.tsv") {
        assert_eq!(row.len(), 2, "layout explicit fixture must be TSV");
        assert_eq!(apply_typing_assist(&row[0], true), Some(row[1].clone()));
    }
}

#[test]
fn typing_assist_auto_replace_off_keeps_layout_only_rules() {
    let pipeline =
        typing_assist_pipeline_for_auto_replace(false, &default_typing_assist_pipeline());

    assert_eq!(
        apply_typing_assist_with_pipeline("кгы ", true, &pipeline),
        Some("rus ".to_string())
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("утп ", true, &pipeline),
        Some("eng ".to_string())
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("njkmrj ", true, &pipeline),
        None
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("прорватся ", false, &pipeline),
        None
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("фактческим ", false, &pipeline),
        None
    );
}

#[test]
fn typing_assist_auto_replace_pipeline_avoids_risky_deletions() {
    let pipeline = typing_assist_pipeline_for_auto_replace(true, &default_typing_assist_pipeline());

    assert_eq!(
        apply_typing_assist_with_pipeline("исправленнно ", false, &pipeline),
        Some("исправлено ".to_string())
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("кнокопками ", false, &pipeline),
        None
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("бешанный ", false, &pipeline),
        None
    );
}

#[test]
fn typing_assist_prefers_reflexive_verb_fix_over_extra_letter_guess() {
    assert_eq!(correct_extra_letters("прорватся"), None);
    assert_eq!(
        apply_typing_assist("прорватся ", false),
        Some("прорваться ".to_string())
    );
    assert_eq!(
        apply_typing_assist("ошибатся ", false),
        Some("ошибаться ".to_string())
    );
}

#[test]
fn typing_assist_auto_switch_keeps_english_and_protected_ascii() {
    assert_eq!(apply_typing_assist("hello ", true), None);
    assert_eq!(apply_typing_assist("test ", true), None);
    assert_eq!(apply_typing_assist("good ", true), None);
    assert_eq!(apply_typing_assist("три ", true), None);
    assert_eq!(apply_typing_assist("раскладок ", true), None);
    assert_eq!(apply_typing_assist("API ", true), None);
    assert_eq!(apply_typing_assist("BTC ", true), None);
    assert_eq!(apply_typing_assist("ETH ", true), None);
    assert_eq!(apply_typing_assist("TRX ", true), None);
    assert_eq!(apply_typing_assist("AmoCRM ", true), None);
    assert_eq!(apply_typing_assist("wi-fi ", true), None);
    assert_eq!(apply_typing_assist("command -f ", true), None);
    assert_eq!(apply_typing_assist("command -r ", true), None);
    assert_eq!(apply_typing_assist("command -c ", true), None);
    assert_eq!(apply_typing_assist("grep --color=auto ", true), None);
}

#[test]
fn typing_assist_keeps_user_protected_ascii_words_when_configured() {
    if std::env::var_os("LAY_TEST_USER_PROTECTED_ASCII").is_none() {
        return;
    }

    assert_eq!(apply_typing_assist("vs ", true), None);
}
