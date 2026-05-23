use super::*;

#[test]
fn lexical_data_loads_without_code_word_lists() {
    assert!(is_common_ru_word("и"));
    assert!(is_common_ru_word("она"));
    assert!(is_common_en_technical_word("api"));
    assert!(is_ru_one_letter_function_word("в"));
    assert!(is_ru_single_letter_pronoun("я"));
    assert!(is_ru_short_pronoun("мне"));
    assert!(is_ru_short_preposition("при"));
    assert!(is_ru_short_function_word("для"));
    assert!(is_ru_hyphen_particle("таки"));
    assert_eq!(visual_b_default_replacement(), "в");
    assert_eq!(visual_b_after_ascii_replacement(), "и");
    assert!(!is_common_en_technical_word("hello"));
}

#[test]
fn protected_ascii_words_parser_keeps_short_user_tokens() {
    let words = parse_ascii_word_data(
        r#"
        # comments are ignored
        vs
        WPS
        AmoCRM
        привет
        wi-fi
        "#,
        1,
    );

    assert!(words.contains("vs"));
    assert!(words.contains("wps"));
    assert!(words.contains("amocrm"));
    assert!(words.contains("wi-fi"));
    assert!(!words.contains("привет"));
}
