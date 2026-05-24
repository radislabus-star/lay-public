use super::*;

#[test]
fn typing_assist_removes_duplicate_layout_prefix_from_ascii_technical_token() {
    let prefix_lower = map_events_to_layout(&[key_event(KeyCode::KEY_W, true)], true);
    let prefix_upper = map_events_to_layout(
        &[KeyEvent {
            keycode: KeyCode::KEY_W.code(),
            shift: true,
            layout_is_ru: true,
        }],
        true,
    );
    let technical_lower =
        map_events_to_layout(&key_events(&ascii_hyphen_token_keycodes(), false), false);
    let technical_upper = map_events_to_layout(
        &[
            KeyEvent {
                keycode: KeyCode::KEY_W.code(),
                shift: true,
                layout_is_ru: false,
            },
            key_event(KeyCode::KEY_I, false),
            key_event(KeyCode::KEY_MINUS, false),
            KeyEvent {
                keycode: KeyCode::KEY_F.code(),
                shift: true,
                layout_is_ru: false,
            },
            key_event(KeyCode::KEY_I, false),
        ],
        false,
    );
    let no_separator = map_events_to_layout(
        &key_events(
            &[
                KeyCode::KEY_W,
                KeyCode::KEY_I,
                KeyCode::KEY_F,
                KeyCode::KEY_I,
            ],
            false,
        ),
        false,
    );

    assert_eq!(
        apply_typing_assist_exact(&format!("{prefix_lower}{technical_lower} ")),
        Some(format!("{technical_lower} "))
    );
    assert_eq!(
        apply_typing_assist_exact(&format!("{prefix_upper}{technical_upper}, ")),
        Some(format!("{technical_upper}, "))
    );
    assert_eq!(
        apply_typing_assist_exact(&format!("{prefix_lower}{no_separator} ")),
        None
    );
}

#[test]
fn typing_assist_does_not_move_normal_word_prefixes() {
    for input in fixture_lines("daemon_typing_assist_prefix_keep.txt") {
        assert_eq!(apply_typing_assist_exact(&input), None, "input={input:?}");
    }
}
