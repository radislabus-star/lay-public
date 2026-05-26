use super::*;

#[test]
fn recognizes_plain_words_and_technical_tokens() {
    let ru = recognize_token("филе");
    assert_eq!(ru.script, WordScript::Cyrillic);
    assert_eq!(ru.kind, WordKind::PlainWord);
    assert!(ru.known_ru);

    let en = recognize_token("file");
    assert_eq!(en.script, WordScript::Ascii);
    assert_eq!(en.kind, WordKind::PlainWord);
    assert!(en.known_en);

    let technical = recognize_token("wi-fi");
    assert_eq!(technical.kind, WordKind::TechnicalToken);
    assert!(technical.technical);
    assert!(is_ascii_technical_token("wi-fi"));
    assert!(is_ascii_technical_or_brand_token("wi-fi"));
    assert!(is_ascii_technical_or_brand_token("file.txt"));
    assert!(!is_ascii_technical_or_brand_token("vty.ghfdbkmyj"));

    let acronym = recognize_token("API");
    assert!(acronym.protected);
    assert!(acronym.technical);
    assert!(is_protected_ascii_token("AmoCRM"));
    assert!(is_ascii_technical_or_brand_token("AmoCRM"));
    assert!(is_ascii_titlecase_token("Wechat"));
    assert!(!is_ascii_titlecase_token("wechat"));
    assert!(is_mixed_cyrillic_ascii_alpha_token("ВщгиDo"));
}

#[test]
fn plain_layout_word_autocorrect_is_risky() {
    assert!(!is_plain_layout_autocorrect_risky("ашду", "file"));
    assert!(!is_plain_layout_autocorrect_risky("ашдуы", "files"));
    assert!(is_plain_layout_autocorrect_risky("abkt", "филе"));
    assert!(is_plain_layout_autocorrect_risky("njkmrj", "только"));
    assert!(is_plain_layout_autocorrect_risky("Lfdfq", "Давай"));
}

#[test]
fn technical_and_mixed_tokens_are_not_plain_layout_risk() {
    assert!(!is_plain_layout_autocorrect_risky("цwi-fi", "wi-fi"));
    assert!(!is_plain_layout_autocorrect_risky("цш-аш", "wi-fi"));
    assert!(!is_plain_layout_autocorrect_risky("API", "API"));
}
