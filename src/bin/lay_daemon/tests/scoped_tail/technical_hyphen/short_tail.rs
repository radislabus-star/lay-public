use super::*;

#[test]
fn scoped_tail_keeps_ascii_hyphen_word_and_flips_current_short_tail() {
    let mut buffer = WordBuffer::new();
    let completed_events = key_events(&ascii_hyphen_token_keycodes(), false);
    for event in &completed_events {
        buffer.push(*event);
    }
    buffer.handle_space();
    let current_events = key_events(&[KeyCode::KEY_Y, KeyCode::KEY_E], false);
    for event in &current_events {
        buffer.push(*event);
    }
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let completed_original = map_original_events(&completed_events);
    let current_original = map_original_events(&current_events);
    let current_target = map_events_to_layout(&current_events, true);

    assert_eq!(
        map_original_events(&events),
        format!("{completed_original} {current_original}")
    );
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some(format!("{completed_original} {current_target}"))
    );
    assert_eq!(
        plan_text_replacement(
            &format!("{completed_original} {current_original}"),
            &format!("{completed_original} {current_target}")
        ),
        Some(text_replacement(
            0,
            current_original.chars().count() as u32,
            current_target,
            0,
        ))
    );
}

#[test]
fn trailing_space_scope_keeps_ascii_hyphen_word_and_flips_last_short_word() {
    let mut buffer = WordBuffer::new();
    let completed_events = key_events(&ascii_hyphen_token_keycodes(), false);
    for event in &completed_events {
        buffer.push(*event);
    }
    buffer.handle_space();
    let current_events = key_events(&[KeyCode::KEY_Y, KeyCode::KEY_E], false);
    for event in &current_events {
        buffer.push(*event);
    }
    buffer.handle_space();

    let scope = effective_replace_words(&buffer, 2, CorrectionEngine::Smart, true);
    let (events, backspaces) = buffer.what_to_replay(scope).expect("last word tail");
    let left = map_original_events(&completed_events);
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
