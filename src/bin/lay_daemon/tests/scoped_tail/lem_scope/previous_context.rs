use super::*;

#[test]
fn scoped_tail_does_not_turn_valid_ascii_hyphen_tail_into_bad_russian() {
    let mut buffer = WordBuffer::new();
    push_keys(&mut buffer, &[KeyCode::KEY_D], true);
    buffer.handle_space();
    let mut current_events = vec![key_event(KeyCode::KEY_W, true)];
    current_events.extend([
        KeyEvent {
            keycode: KeyCode::KEY_W.code(),
            shift: true,
            layout_is_ru: false,
        },
        key_event(KeyCode::KEY_I, false),
        key_event(KeyCode::KEY_MINUS, false),
        key_event(KeyCode::KEY_F, false),
        key_event(KeyCode::KEY_I, false),
    ]);
    for event in &current_events {
        buffer.push(*event);
    }
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let left = map_events_to_layout(&[key_event(KeyCode::KEY_D, true)], true);
    let current_original = map_original_events(&current_events);
    let current_wrong_layout = map_events_to_layout(&current_events, true);

    assert_eq!(
        map_original_events(&events),
        format!("{left} {current_original}")
    );
    assert_ne!(
        decide_scoped_tail_correction(&events),
        Some(format!("{left} {current_wrong_layout}"))
    );
}

#[test]
fn scoped_tail_converts_confident_bad_previous_word() {
    let mut buffer = WordBuffer::new();
    push_keys(
        &mut buffer,
        &[
            KeyCode::KEY_G,
            KeyCode::KEY_H,
            KeyCode::KEY_B,
            KeyCode::KEY_D,
            KeyCode::KEY_T,
            KeyCode::KEY_N,
        ],
        false,
    );
    buffer.handle_space();
    push_keys(
        &mut buffer,
        &[KeyCode::KEY_V, KeyCode::KEY_B, KeyCode::KEY_H],
        false,
    );
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("привет мир".to_string())
    );
}

#[test]
fn scoped_tail_keeps_unknown_previous_word() {
    let mut buffer = WordBuffer::new();
    push_keys(
        &mut buffer,
        &[
            KeyCode::KEY_F,
            KeyCode::KEY_O,
            KeyCode::KEY_O,
            KeyCode::KEY_B,
            KeyCode::KEY_A,
            KeyCode::KEY_R,
        ],
        false,
    );
    buffer.handle_space();
    push_keys(
        &mut buffer,
        &[KeyCode::KEY_G, KeyCode::KEY_H, KeyCode::KEY_J],
        false,
    );
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("foobar про".to_string())
    );
}
