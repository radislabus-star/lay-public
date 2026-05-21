use super::*;

#[test]
fn two_word_replay_keeps_space_and_backspace_count() {
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

    let (events, backspaces) = buffer.what_to_replay(2).expect("two words are buffered");

    assert_eq!(map_original_events(&events), "ghbdtn vbh");
    assert_eq!(backspaces, 10);
    assert_eq!(events[6].keycode, KeyCode::KEY_SPACE.code());
    let decision = replay_layout_decision(&events);
    assert_eq!(
        map_target_events(&events, decision.target_is_ru),
        "привет мир"
    );
}

#[test]
fn two_word_trailing_space_replay_deletes_expected_tail() {
    let mut buffer = WordBuffer::new();
    push_keys(&mut buffer, &[KeyCode::KEY_G, KeyCode::KEY_H], false);
    buffer.handle_space();
    push_keys(&mut buffer, &[KeyCode::KEY_V, KeyCode::KEY_B], false);
    buffer.handle_space();

    let (events, backspaces) = buffer.what_to_replay(2).expect("two completed words");

    assert_eq!(map_original_events(&events), "gh vb ");
    assert_eq!(backspaces, 6);
}

#[test]
fn smart_scope_after_trailing_space_keeps_configured_scope() {
    let mut buffer = WordBuffer::new();
    push_keys(
        &mut buffer,
        &[
            KeyCode::KEY_R,
            KeyCode::KEY_J,
            KeyCode::KEY_H,
            KeyCode::KEY_J,
            KeyCode::KEY_X,
            KeyCode::KEY_T,
        ],
        true,
    );
    buffer.handle_space();
    push_keys(
        &mut buffer,
        &[KeyCode::KEY_N, KeyCode::KEY_F, KeyCode::KEY_V],
        true,
    );
    buffer.handle_space();

    let scope = effective_replace_words(&buffer, 2, CorrectionEngine::Smart, true);
    let (events, backspaces) = buffer.what_to_replay(scope).expect("last word is buffered");

    assert_eq!(scope, 2);
    assert_eq!(map_original_events(&events), "короче там ");
    assert_eq!(backspaces, 11);
}

#[test]
fn replay_layout_decision_ignores_inserted_space() {
    let events = [
        key_event(KeyCode::KEY_G, true),
        key_event(KeyCode::KEY_H, true),
        key_event(KeyCode::KEY_SPACE, false),
        key_event(KeyCode::KEY_V, true),
        key_event(KeyCode::KEY_B, true),
    ];

    assert!(!is_layout_decision_key(KeyCode::KEY_SPACE));
    assert_eq!(
        replay_layout_decision(&events),
        ReplayLayoutDecision {
            target_is_ru: false,
            mixed_layouts: false,
        }
    );
}

#[test]
fn shortcut_modified_text_keys_do_not_enter_word_buffer() {
    let mut modifiers = ShiftState::default();

    modifiers.update(KeyCode::KEY_LEFTCTRL, 1);
    assert!(should_ignore_buffer_key(
        KeyCode::KEY_EQUAL,
        &modifiers,
        true
    ));
    assert!(should_ignore_buffer_key(
        KeyCode::KEY_MINUS,
        &modifiers,
        true
    ));
    assert!(should_ignore_buffer_key(
        KeyCode::KEY_SPACE,
        &modifiers,
        true
    ));
    assert!(should_ignore_buffer_key(KeyCode::KEY_A, &modifiers, true));

    modifiers.update(KeyCode::KEY_LEFTCTRL, 0);
    assert!(!should_ignore_buffer_key(KeyCode::KEY_A, &modifiers, true));
}

#[test]
fn leading_plus_minus_symbols_do_not_attach_to_next_word() {
    let mut buffer = WordBuffer::new();

    for (key, shift) in [
        (KeyCode::KEY_EQUAL, true),
        (KeyCode::KEY_EQUAL, false),
        (KeyCode::KEY_MINUS, true),
        (KeyCode::KEY_MINUS, false),
    ] {
        let mut modifiers = ShiftState::default();
        modifiers.update(KeyCode::KEY_LEFTSHIFT, i32::from(shift));
        if !should_ignore_buffer_key(key, &modifiers, buffer.current_is_empty()) {
            buffer.push(key_event(key, shift));
        }
    }
    push_text_as_layout(&mut buffer, "есть", true);

    let (events, backspaces) = buffer.what_to_replay(1).expect("word tail");

    assert_eq!(map_original_events(&events), "есть");
    assert_eq!(backspaces, 4);
}

#[test]
fn visual_latin_word_with_cyrillic_c_homoglyph_replays_to_ru() {
    let events = [
        key_event(KeyCode::KEY_C, true),
        key_event(KeyCode::KEY_H, false),
        key_event(KeyCode::KEY_E, false),
        key_event(KeyCode::KEY_C, false),
    ];

    assert_eq!(map_original_events(&events), "сhec");
    assert_eq!(
        replay_layout_decision(&events),
        ReplayLayoutDecision {
            target_is_ru: true,
            mixed_layouts: true,
        }
    );
    assert_eq!(map_target_events(&events, true), "срус");
}
