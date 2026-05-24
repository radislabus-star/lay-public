use super::*;

#[test]
fn typing_assist_fixes_common_missing_letter_typos() {
    assert_eq!(
        apply_typing_assist_exact("првильно "),
        Some("правильно ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("Првильно "),
        Some("Правильно ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("можн "),
        Some("можно ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("Можн "),
        Some("Можно ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("дльше "),
        Some("дальше ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("дальг "),
        Some("дальше ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("плозо "),
        Some("плохо ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("фактческим "),
        Some("фактическим ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("иблиотеку "),
        Some("библиотеку ".to_string())
    );
    assert_eq!(apply_typing_assist_exact("крипта "), None);
    assert_eq!(apply_typing_assist_exact("Крипта "), None);
}

#[test]
fn typing_assist_fixes_live_user_stream_typos() {
    assert!(is_known_russian_word_or_form("ориентироваться"));
    assert!(is_known_russian_word_or_form("переиспользоваться"));
    assert_eq!(
        apply_typing_assist_exact("занчит "),
        Some("значит ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("орентироваться "),
        Some("ориентироваться ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("переиспользоватся "),
        Some("переиспользоваться ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("работатет "),
        Some("работает ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("котром "),
        Some("котором ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("рабоТТА "),
        Some("работа ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("помагу "),
        Some("помогу ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("видешь "),
        Some("видишь ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("кнокопками "),
        Some("кнопками ".to_string())
    );
}

#[test]
fn typing_assist_normalizes_accidental_inner_uppercase() {
    assert_eq!(
        apply_typing_assist_exact("МОжно "),
        Some("Можно ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("моЖно "),
        Some("можно ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("рабоТА "),
        Some("работа ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("улУЧШИТЬ "),
        Some("улучшить ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("улУЧШИТЬ? "),
        Some("улучшить? ".to_string())
    );
    assert_eq!(apply_typing_assist_exact("МОЖНО "), None);
}

#[test]
fn typing_assist_single_letter_typos_only_use_neighbor_keys() {
    assert!(are_ru_keyboard_neighbors('з', 'х'));
    assert!(!are_ru_keyboard_neighbors('о', 'ь'));
    assert_eq!(apply_typing_assist_exact("покрыто "), None);
    assert_eq!(apply_typing_assist_exact("робило "), None);
    assert_eq!(
        apply_typing_assist_exact("плозо "),
        Some("плохо ".to_string())
    );
}
