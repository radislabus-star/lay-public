use super::*;
use crate::keyboard::{
    map_events_to_layout, map_original_events, replay_layout_decision, text_to_key_events,
};
use crate::typing_assist_test_fixtures::{fixture_rows, parse_bool_fixture};
use evdev::KeyCode;
use std::time::Duration;

fn push_text_as_layout(buffer: &mut WordBuffer, text: &str, layout_is_ru: bool) {
    for event in text_events(text, layout_is_ru) {
        if event.keycode == KeyCode::KEY_SPACE.code() {
            buffer.handle_space();
        } else {
            buffer.push(event);
        }
    }
}

fn text_events(text: &str, layout_is_ru: bool) -> Vec<KeyEvent> {
    text_to_key_events(text, layout_is_ru)
        .unwrap_or_else(|| panic!("failed to create key events for {text:?}"))
}

#[test]
fn single_word_wrong_layout_replays_opposite_layout() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "ltkfq", false);

    let (events, backspaces) = buffer.what_to_replay(1).expect("word");
    let decision = replay_layout_decision(&events);

    assert_eq!(map_original_events(&events), "ltkfq");
    assert_eq!(backspaces, 5);
    assert!(decision.target_is_ru);
    assert_eq!(map_events_to_layout(&events, true), "делай");
}

#[test]
fn visible_tail_text_reads_completed_words_and_current_token() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "html djn api аш", false);

    assert_eq!(
        buffer.visible_tail_text(4).as_deref(),
        Some("html djn api аш")
    );
}

#[test]
fn replay_toggle_uses_only_remembered_word_even_with_wider_scope() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "abc l", false);

    buffer.mark_replayed_layout(1, true);
    let (events, backspaces) = buffer.what_to_replay(3).expect("toggle word");

    assert_eq!(backspaces, 1);
    assert_eq!(events.len(), 1);
    assert_eq!(map_original_events(&events), "д");
    assert!(buffer.replay_toggle_ready());
}

#[test]
fn replay_toggle_reuses_original_multiword_scope_after_replay() {
    let mut buffer = WordBuffer::new();
    assert!(buffer.remember_visible_text_for_correction("чем ещё луеен"));

    let (events, backspaces) = buffer.what_to_replay(3).expect("three-word tail");
    let decision = replay_layout_decision(&events);
    assert_eq!(map_original_events(&events), "чем ещё луеен");
    assert_eq!(backspaces, 13);
    assert!(!decision.target_is_ru);
    assert_eq!(map_events_to_layout(&events, false), "xtv to` ketty");

    buffer.mark_replayed_layout(3, false);

    let (undo_events, undo_backspaces) = buffer.what_to_replay(3).expect("full undo tail");
    let undo_decision = replay_layout_decision(&undo_events);
    assert_eq!(undo_backspaces, 13);
    assert_eq!(map_original_events(&undo_events), "xtv to` ketty");
    assert!(undo_decision.target_is_ru);
    assert_eq!(map_events_to_layout(&undo_events, true), "чем ещё луеен");
}

#[test]
fn replay_toggle_can_flip_same_word_four_times_with_wider_scope() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "good", false);

    for row in fixture_rows("word_buffer_toggle_sequence.tsv") {
        assert_eq!(row.len(), 3, "word buffer toggle fixture must be TSV");
        let target_is_ru = parse_bool_fixture(&row[2]);
        let (events, backspaces) = buffer.what_to_replay(3).expect("toggle word");
        let decision = replay_layout_decision(&events);

        assert_eq!(backspaces, 4);
        assert_eq!(map_original_events(&events), row[0]);
        assert_eq!(map_events_to_layout(&events, decision.target_is_ru), row[1]);
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
    push_text_as_layout(&mut buffer, "ab cd ", false);

    let (events, backspaces) = buffer.what_to_replay(2).expect("tail");

    assert_eq!(map_original_events(&events), "ab cd ");
    assert_eq!(backspaces, 6);
}

#[test]
fn completed_tail_remains_readable_while_next_word_is_being_typed() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "ab c", false);

    let events = buffer
        .last_completed_words_events(1)
        .expect("previous completed word");

    assert_eq!(map_original_events(&events), "ab ");
}

#[test]
fn learning_feedback_requires_user_delete_and_retype() {
    let mut buffer = WordBuffer::new();
    buffer.remember_pending_learning_correction("typing-assist", "смотри ", "смотрин ", 1, 1);
    buffer.note_learning_typed(text_events("п", true).remove(0));

    assert!(buffer.take_user_learning_correction(true).is_none());

    buffer.remember_pending_learning_correction("typing-assist", "смотри ", "смотрин ", 1, 1);
    for _ in 0.."смотрин ".chars().count() {
        buffer.note_learning_backspace();
    }
    for event in text_events("смотри", true) {
        buffer.note_learning_typed(event);
    }

    let correction = buffer
        .take_user_learning_correction(true)
        .expect("correction");

    assert_eq!(correction.from, "смотрин ");
    assert_eq!(correction.to, "смотри ");
}

#[test]
fn pending_auto_undo_readiness_does_not_consume_fresh_undo() {
    let mut buffer = WordBuffer::new();
    buffer.remember_pending_auto_undo("typing-assist", "посмотри", "посмотреть", 1, 1);

    assert!(buffer.pending_auto_undo_ready());
    assert!(buffer.take_pending_auto_undo().is_some());
}

#[test]
fn expired_pending_auto_undo_does_not_steal_manual_toggle() {
    let mut buffer = WordBuffer::new();
    buffer.remember_pending_auto_undo("typing-assist", "посмотри", "посмотреть", 1, 1);
    buffer
        .pending_auto_undo
        .as_mut()
        .expect("pending undo")
        .started_at = Instant::now()
        .checked_sub(Duration::from_secs(
            LEARNING_FEEDBACK_MAX_AGE_SECS.saturating_add(1),
        ))
        .expect("valid test instant");

    assert!(!buffer.pending_auto_undo_ready());
    assert!(buffer.take_pending_auto_undo().is_none());
}

#[test]
fn failed_backend_can_restore_unconsumed_pending_auto_undo() {
    let mut buffer = WordBuffer::new();
    buffer.remember_pending_auto_undo("typing-assist", "проверрка ", "проверка ", 1, 1);
    let undo = buffer.take_pending_auto_undo().expect("pending undo");

    buffer.restore_pending_auto_undo(undo);

    assert!(buffer.pending_auto_undo_ready());
    assert!(buffer.take_pending_auto_undo().is_some());
}
