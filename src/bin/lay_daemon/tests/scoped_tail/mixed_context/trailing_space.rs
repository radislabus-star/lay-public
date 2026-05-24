use super::*;

#[test]
fn scoped_tail_trailing_space_keeps_previous_good_word_and_flips_current_completed_latin_keys() {
    let mut buffer = WordBuffer::new();
    let left_events = [
        KeyEvent {
            keycode: KeyCode::KEY_D.code(),
            shift: true,
            layout_is_ru: false,
        },
        key_event(KeyCode::KEY_O, false),
        key_event(KeyCode::KEY_U, false),
        key_event(KeyCode::KEY_B, false),
        key_event(KeyCode::KEY_L, false),
        key_event(KeyCode::KEY_E, false),
    ];
    for event in left_events {
        buffer.push(event);
    }
    buffer.handle_space();
    let current_events = [
        key_event(KeyCode::KEY_N, false),
        key_event(KeyCode::KEY_J, false),
        key_event(KeyCode::KEY_SEMICOLON, false),
        key_event(KeyCode::KEY_T, false),
    ];
    for event in current_events {
        buffer.push(event);
    }
    buffer.handle_space();

    let scope = effective_replace_words(&buffer, 2, CorrectionEngine::Smart, true);
    let (events, backspaces) = buffer.what_to_replay(scope).expect("last word tail");
    let left = map_original_events(&left_events);
    let current_original = map_original_events(&current_events);
    let current_target = map_events_to_layout(&current_events, true);

    assert_eq!(scope, 2);
    assert_eq!(
        map_original_events(&events),
        format!("{left} {current_original} ")
    );
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some(format!("{left} {current_target} "))
    );
    assert_eq!(
        backspaces,
        (left.chars().count() + 1 + current_original.chars().count() + 1) as u32
    );
}

#[test]
fn scoped_tail_trailing_space_keeps_previous_russian_word_and_flips_completed_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "открывал", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "цзы", true);
    buffer.handle_space();

    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let original = map_original_events(&events);
    let replacement =
        decide_scoped_tail_correction_with_lem(&events, true).expect("smart replacement");

    assert_eq!(original, "открывал цзы ");
    assert_eq!(replacement, "открывал wps ");
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(TextReplacement {
            move_left: 1,
            backspaces: 3,
            insert: "wps".to_string(),
            move_right: 1,
        })
    );
}
