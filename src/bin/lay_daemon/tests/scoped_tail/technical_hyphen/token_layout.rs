use super::*;

#[test]
fn scoped_tail_flips_cyrillic_hyphen_technical_token_to_ascii() {
    let mut buffer = WordBuffer::new();
    let left_events = [
        key_event(KeyCode::KEY_C, true),
        key_event(KeyCode::KEY_K, true),
        key_event(KeyCode::KEY_J, true),
        key_event(KeyCode::KEY_D, true),
        key_event(KeyCode::KEY_J, true),
    ];
    for event in left_events {
        buffer.push(event);
    }
    buffer.handle_space();
    let technical_events = [
        key_event(KeyCode::KEY_W, true),
        key_event(KeyCode::KEY_I, true),
        key_event(KeyCode::KEY_MINUS, true),
        key_event(KeyCode::KEY_F, true),
        key_event(KeyCode::KEY_I, true),
    ];
    for event in technical_events {
        buffer.push(event);
    }
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let left = map_events_to_layout(&left_events, true);
    let typed_technical = map_events_to_layout(&technical_events, true);
    let target_technical = map_events_to_layout(&technical_events, false);

    assert_eq!(
        map_original_events(&events),
        format!("{left} {typed_technical}")
    );
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some(format!("{left} {target_technical}"))
    );
}

#[test]
fn scoped_tail_keeps_unknown_previous_word_and_flips_cyrillic_hyphen_technical_token() {
    let mut buffer = WordBuffer::new();
    let left_events = [
        key_event_with_shift(KeyCode::KEY_SEMICOLON, true, true),
        key_event(KeyCode::KEY_SEMICOLON, true),
        key_event(KeyCode::KEY_SEMICOLON, true),
        key_event(KeyCode::KEY_SEMICOLON, true),
    ];
    for event in left_events {
        buffer.push(event);
    }
    buffer.handle_space();
    let technical_events = [
        key_event(KeyCode::KEY_W, true),
        key_event(KeyCode::KEY_I, true),
        key_event(KeyCode::KEY_MINUS, true),
        key_event(KeyCode::KEY_F, true),
        key_event(KeyCode::KEY_I, true),
    ];
    for event in technical_events {
        buffer.push(event);
    }
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let left = map_events_to_layout(&left_events, true);
    let typed_technical = map_events_to_layout(&technical_events, true);
    let target_technical = map_events_to_layout(&technical_events, false);

    assert_eq!(
        map_original_events(&events),
        format!("{left} {typed_technical}")
    );
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some(format!("{left} {target_technical}"))
    );
}
