use super::*;

#[test]
fn typing_assist_converts_wrong_layout_ascii_hyphen_token() {
    let technical_events = [
        key_event(KeyCode::KEY_W, true),
        key_event(KeyCode::KEY_I, true),
        key_event(KeyCode::KEY_MINUS, true),
        key_event(KeyCode::KEY_F, true),
        key_event(KeyCode::KEY_I, true),
    ];
    let typed_technical = map_events_to_layout(&technical_events, true);
    let target_technical = map_events_to_layout(&technical_events, false);
    assert_eq!(
        apply_typing_assist_exact(&format!("{typed_technical} ")),
        Some(format!("{target_technical} "))
    );
}

#[test]
fn typing_assist_keeps_natural_cyrillic_hyphen_words() {
    assert_eq!(apply_typing_assist("что-то ", true), None);
    assert_eq!(apply_typing_assist("кто-то ", true), None);
    assert_eq!(apply_typing_assist("где-то ", true), None);
    assert_eq!(apply_typing_assist("как-то ", true), None);
    assert_eq!(apply_typing_assist("из-за ", true), None);
    assert_eq!(apply_typing_assist("кока-коле ", true), None);
    assert_eq!(apply_typing_assist("код-дэ-вуар ", true), None);
    assert_eq!(apply_typing_assist("чек-лист! ", true), None);
    assert_eq!(apply_typing_assist("к-лист! ", true), None);
    assert_eq!(correct_wrong_layout_ascii_technical_token("из-за"), None);
    assert_eq!(
        correct_wrong_layout_ascii_technical_token("цш-аш"),
        Some("wi-fi".to_string())
    );
    assert_eq!(correct_wrong_layout_ascii_technical_token("15р-16р"), None);
    assert_eq!(apply_typing_assist("15р-16р ", true), None);
}
