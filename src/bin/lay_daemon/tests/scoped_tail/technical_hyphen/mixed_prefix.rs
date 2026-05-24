use super::*;

#[test]
fn scoped_tail_collapses_cyrillic_prefix_before_ascii_hyphen_tail() {
    let mut buffer = WordBuffer::new();
    push_keys(
        &mut buffer,
        &[
            KeyCode::KEY_C,
            KeyCode::KEY_K,
            KeyCode::KEY_J,
            KeyCode::KEY_D,
            KeyCode::KEY_J,
        ],
        true,
    );
    buffer.handle_space();
    let mut current_events = vec![key_event(KeyCode::KEY_G, true)];
    current_events.extend(key_events(
        &[
            KeyCode::KEY_G,
            KeyCode::KEY_F,
            KeyCode::KEY_H,
            KeyCode::KEY_F,
            KeyCode::KEY_MINUS,
            KeyCode::KEY_G,
            KeyCode::KEY_F,
            KeyCode::KEY_H,
            KeyCode::KEY_F,
        ],
        false,
    ));
    for event in &current_events {
        buffer.push(*event);
    }
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let left = map_events_to_layout(
        &[
            key_event(KeyCode::KEY_C, true),
            key_event(KeyCode::KEY_K, true),
            key_event(KeyCode::KEY_J, true),
            key_event(KeyCode::KEY_D, true),
            key_event(KeyCode::KEY_J, true),
        ],
        true,
    );
    let current_original = map_original_events(&current_events);
    let current_target =
        repair_cyrillic_prefix_before_ascii_tail(&current_events).expect("prefix collapse repair");

    assert_eq!(
        map_original_events(&events),
        format!("{left} {current_original}")
    );
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some(format!("{left} {current_target}"))
    );
}

#[test]
fn scoped_tail_repairs_mixed_cyrillic_prefix_ascii_hyphen_word_and_keeps_undo() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "Иракскую", true);
    buffer.handle_space();
    buffer.push(text_key_event('к', true));
    for ch in "jrf-rjke".chars() {
        buffer.push(text_key_event(ch, false));
    }

    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let original = map_original_events(&events);
    let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");
    let plan = plan_text_replacement(&original, &replacement).expect("minimal plan");

    assert_eq!(original, "Иракскую кjrf-rjke");
    assert_eq!(replacement, "Иракскую кока-колу");
    assert_eq!(
        plan,
        TextReplacement {
            move_left: 0,
            backspaces: 8,
            insert: "ока-колу".to_string(),
            move_right: 0,
        }
    );
    assert!(buffer.remember_replacement_last_word_for_replay(&events, &plan, &replacement));

    let (undo_events, undo_backspaces) = buffer.what_to_replay(2).expect("undo tail");
    let undo_decision = replay_layout_decision(&undo_events);
    assert_eq!(map_original_events(&undo_events), "кока-колу");
    assert_eq!(undo_backspaces, 9);
    assert!(!undo_decision.target_is_ru);
    assert_eq!(map_events_to_layout(&undo_events, false), "rjrf-rjke");
    assert!(buffer.replay_toggle_ready());
}

#[test]
fn scoped_tail_repairs_mixed_cyrillic_prefix_ascii_hyphen_dative_word() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "Иракскую", true);
    buffer.handle_space();
    buffer.push(text_key_event('к', true));
    for ch in "jrf-rjkt".chars() {
        buffer.push(text_key_event(ch, false));
    }

    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let original = map_original_events(&events);
    let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");
    let plan = plan_text_replacement(&original, &replacement).expect("minimal plan");

    assert_eq!(original, "Иракскую кjrf-rjkt");
    assert_eq!(replacement, "Иракскую кока-коле");
    assert_eq!(
        plan,
        TextReplacement {
            move_left: 0,
            backspaces: 8,
            insert: "ока-коле".to_string(),
            move_right: 0,
        }
    );
    assert!(buffer.remember_replacement_last_word_for_replay(&events, &plan, &replacement));

    let (undo_events, undo_backspaces) = buffer.what_to_replay(2).expect("undo tail");
    let undo_decision = replay_layout_decision(&undo_events);
    assert_eq!(map_original_events(&undo_events), "кока-коле");
    assert_eq!(undo_backspaces, 9);
    assert!(!undo_decision.target_is_ru);
    assert_eq!(map_events_to_layout(&undo_events, false), "rjrf-rjkt");
    assert!(buffer.replay_toggle_ready());
}
