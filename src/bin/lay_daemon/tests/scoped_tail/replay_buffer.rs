use super::*;

fn replay_buffer_case(id: &str) -> Vec<String> {
    fixture_row_by_id("daemon_scoped_tail_replay_buffer.tsv", id)
}

#[test]
fn two_word_replay_keeps_space_and_backspace_count() {
    let row = replay_buffer_case("two_word_replay");
    assert_eq!(row.len(), 8, "bad fixture row: {row:?}");
    let scope: usize = row[3].parse().expect("scope");
    let (_buffer, events, backspaces) = typed_tail(
        &[(&row[1], layout_from_fixture(&row[2]))],
        scope,
        "two words are buffered",
    );

    assert_eq!(map_original_events(&events), row[4]);
    assert_eq!(backspaces, row[5].parse::<u32>().expect("backspaces"));
    assert_eq!(
        events[row[7].parse::<usize>().expect("space index")].keycode,
        KeyCode::KEY_SPACE.code()
    );
    let decision = replay_layout_decision(&events);
    assert_eq!(map_target_events(&events, decision.target_is_ru), row[6]);
}

#[test]
fn two_word_trailing_space_replay_deletes_expected_tail() {
    let row = replay_buffer_case("two_word_trailing_space");
    assert_eq!(row.len(), 8, "bad fixture row: {row:?}");
    let scope: usize = row[3].parse().expect("scope");
    let (_buffer, events, backspaces) = typed_tail(
        &[(&row[1], layout_from_fixture(&row[2]))],
        scope,
        "two completed words",
    );

    assert_eq!(map_original_events(&events), row[4]);
    assert_eq!(backspaces, row[5].parse::<u32>().expect("backspaces"));
}

#[test]
fn smart_scope_after_trailing_space_keeps_configured_scope() {
    let row = replay_buffer_case("smart_scope_trailing_space");
    assert_eq!(row.len(), 8, "bad fixture row: {row:?}");
    let configured_scope: usize = row[3].parse().expect("scope");
    let buffer = typed_buffer(&[(&row[1], layout_from_fixture(&row[2]))]);

    let scope = effective_replace_words(&buffer, configured_scope, CorrectionEngine::Smart, true);
    let (events, backspaces) = buffer.what_to_replay(scope).expect("last word is buffered");

    assert_eq!(scope, configured_scope);
    assert_eq!(map_original_events(&events), row[4]);
    assert_eq!(backspaces, row[5].parse::<u32>().expect("backspaces"));
}

#[test]
fn layout_decision_ignores_inserted_space() {
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
