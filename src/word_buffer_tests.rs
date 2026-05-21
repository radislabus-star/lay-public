use super::*;
use crate::keyboard::{map_events_to_layout, map_original_events, replay_layout_decision};

fn key_event(key: KeyCode, layout_is_ru: bool) -> KeyEvent {
    KeyEvent {
        keycode: key.code(),
        shift: false,
        layout_is_ru,
    }
}

fn push_text_as_layout(buffer: &mut WordBuffer, keys: &[KeyCode], layout_is_ru: bool) {
    for key in keys {
        buffer.push(key_event(*key, layout_is_ru));
    }
}

#[test]
fn single_word_wrong_layout_replays_opposite_layout() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(
        &mut buffer,
        &[
            KeyCode::KEY_L,
            KeyCode::KEY_T,
            KeyCode::KEY_K,
            KeyCode::KEY_F,
            KeyCode::KEY_Q,
        ],
        false,
    );

    let (events, backspaces) = buffer.what_to_replay(1).expect("word");
    let decision = replay_layout_decision(&events);

    assert_eq!(map_original_events(&events), "ltkfq");
    assert_eq!(backspaces, 5);
    assert!(decision.target_is_ru);
    assert_eq!(map_events_to_layout(&events, true), "делай");
}

#[test]
fn replay_toggle_uses_only_remembered_word_even_with_wider_scope() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(
        &mut buffer,
        &[KeyCode::KEY_A, KeyCode::KEY_B, KeyCode::KEY_C],
        false,
    );
    buffer.handle_space();
    push_text_as_layout(&mut buffer, &[KeyCode::KEY_L], false);

    buffer.mark_replayed_layout(1, true);
    let (events, backspaces) = buffer.what_to_replay(3).expect("toggle word");

    assert_eq!(backspaces, 1);
    assert_eq!(events.len(), 1);
    assert_eq!(map_original_events(&events), "д");
    assert!(buffer.replay_toggle_ready());
}

#[test]
fn replay_toggle_can_flip_same_word_four_times_with_wider_scope() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(
        &mut buffer,
        &[
            KeyCode::KEY_G,
            KeyCode::KEY_O,
            KeyCode::KEY_O,
            KeyCode::KEY_D,
        ],
        false,
    );

    for (original, target, target_is_ru) in [
        ("good", "пщщв", true),
        ("пщщв", "good", false),
        ("good", "пщщв", true),
        ("пщщв", "good", false),
    ] {
        let (events, backspaces) = buffer.what_to_replay(3).expect("toggle word");
        let decision = replay_layout_decision(&events);

        assert_eq!(backspaces, 4);
        assert_eq!(map_original_events(&events), original);
        assert_eq!(map_events_to_layout(&events, decision.target_is_ru), target);
        assert_eq!(decision.target_is_ru, target_is_ru);

        buffer.mark_replayed_layout(3, decision.target_is_ru);
    }
}

#[test]
fn visible_text_after_auto_undo_can_be_corrected_again() {
    let mut buffer = WordBuffer::new();

    assert!(buffer.remember_visible_text_for_correction("слово кjrf-rjke"));
    assert!(!buffer.replay_toggle_ready());

    let (events, backspaces) = buffer.what_to_replay(3).expect("visible tail");

    assert_eq!(map_original_events(&events), "слово кjrf-rjke");
    assert_eq!(backspaces, 15);
}

#[test]
fn completed_two_word_tail_includes_one_space_and_trailing_space() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, &[KeyCode::KEY_A, KeyCode::KEY_B], false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, &[KeyCode::KEY_C, KeyCode::KEY_D], false);
    buffer.handle_space();

    let (events, backspaces) = buffer.what_to_replay(2).expect("tail");

    assert_eq!(map_original_events(&events), "ab cd ");
    assert_eq!(backspaces, 6);
}

#[test]
fn completed_tail_remains_readable_while_next_word_is_being_typed() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, &[KeyCode::KEY_A, KeyCode::KEY_B], false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, &[KeyCode::KEY_C], false);

    let events = buffer
        .last_completed_words_events(1)
        .expect("previous completed word");

    assert_eq!(map_original_events(&events), "ab ");
}

#[test]
fn learning_feedback_requires_user_delete_and_retype() {
    let mut buffer = WordBuffer::new();
    buffer.remember_pending_learning_correction("typing-assist", "смотри ", "смотрин ", 1, 1);
    buffer.note_learning_typed(key_event(KeyCode::KEY_G, true));

    assert!(buffer.take_user_learning_correction(true).is_none());

    buffer.remember_pending_learning_correction("typing-assist", "смотри ", "смотрин ", 1, 1);
    for _ in 0.."смотрин ".chars().count() {
        buffer.note_learning_backspace();
    }
    for key in [
        KeyCode::KEY_C,
        KeyCode::KEY_V,
        KeyCode::KEY_J,
        KeyCode::KEY_N,
        KeyCode::KEY_H,
        KeyCode::KEY_B,
    ] {
        buffer.note_learning_typed(key_event(key, true));
    }

    let correction = buffer
        .take_user_learning_correction(true)
        .expect("correction");

    assert_eq!(correction.from, "смотрин ");
    assert_eq!(correction.to, "смотри ");
}
