use super::*;
use crate::daemon_state::DaemonLoopState;

#[test]
fn window_input_state_keeps_separate_word_buffers() {
    let mut state = DaemonLoopState::new(&LayConfig::default(), false, false);

    assert!(state.switch_window_input_state(Some("window-a".to_string())));
    push_text_as_layout(&mut state.buffer, "ghb", false);
    assert_eq!(
        map_original_events(&state.buffer.what_to_replay(1).unwrap().0),
        "ghb"
    );

    assert!(state.switch_window_input_state(Some("window-b".to_string())));
    assert!(state.buffer.current_is_empty());
    push_text_as_layout(&mut state.buffer, "djn", false);
    assert_eq!(
        map_original_events(&state.buffer.what_to_replay(1).unwrap().0),
        "djn"
    );

    assert!(state.switch_window_input_state(Some("window-a".to_string())));
    assert_eq!(
        map_original_events(&state.buffer.what_to_replay(1).unwrap().0),
        "ghb"
    );

    assert!(state.switch_window_input_state(Some("window-b".to_string())));
    assert_eq!(
        map_original_events(&state.buffer.what_to_replay(1).unwrap().0),
        "djn"
    );
}

#[test]
fn text_field_context_keeps_unfinished_words_separate_inside_same_window() {
    let mut state = DaemonLoopState::new(&LayConfig::default(), false, false);

    assert!(state.switch_window_input_state(Some("browser-window".to_string())));
    push_text_as_layout(&mut state.buffer, "file", false);
    assert_eq!(
        map_original_events(&state.buffer.what_to_replay(1).unwrap().0),
        "file"
    );

    assert!(state.switch_field_context_epoch(1));
    assert!(state.buffer.current_is_empty());
    push_text_as_layout(&mut state.buffer, "djn", false);
    assert_eq!(
        map_original_events(&state.buffer.what_to_replay(1).unwrap().0),
        "djn"
    );
}

#[test]
fn text_field_context_separates_fields_without_window_identity() {
    let mut state = DaemonLoopState::new(&LayConfig::default(), false, false);

    push_text_as_layout(&mut state.buffer, "qwe", false);
    assert_eq!(
        map_original_events(&state.buffer.what_to_replay(1).unwrap().0),
        "qwe"
    );

    assert!(state.switch_field_context_epoch(1));
    assert!(state.buffer.current_is_empty());
    push_text_as_layout(&mut state.buffer, "qwe", false);
    assert_eq!(
        map_original_events(&state.buffer.what_to_replay(1).unwrap().0),
        "qwe"
    );
}

#[test]
fn text_field_context_does_not_accumulate_empty_slots() {
    let mut state = DaemonLoopState::new(&LayConfig::default(), false, false);

    assert!(state.switch_window_input_state(Some("browser-window".to_string())));
    for epoch in 1..80 {
        state.switch_field_context_epoch(epoch);
    }

    assert!(state.window_states.is_empty());
}

#[test]
fn text_field_context_prunes_old_saved_slots() {
    let mut state = DaemonLoopState::new(&LayConfig::default(), false, false);

    assert!(state.switch_window_input_state(Some("browser-window".to_string())));
    for epoch in 1..80 {
        push_text_as_layout(&mut state.buffer, "x", false);
        state.switch_field_context_epoch(epoch);
    }

    assert!(state.window_states.len() <= 50);
}
