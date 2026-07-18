use super::{LayIbusEngine, WordInputMode};
use lay::config::LayConfig;
use std::sync::{Arc, Mutex};

fn engine() -> LayIbusEngine {
    LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            ..LayConfig::default()
        },
    )
}

#[test]
fn narrow_cursor_uses_terminal_passthrough_profile() {
    let mut engine = engine();
    engine.cursor_cell_width = 2;

    assert_eq!(
        engine.initial_word_input_mode(),
        WordInputMode::TerminalPassthrough
    );
}

#[test]
fn wide_text_cursor_uses_managed_commit_profile() {
    let mut engine = engine();
    engine.cursor_cell_width = 11;

    assert_eq!(
        engine.initial_word_input_mode(),
        WordInputMode::ManagedCommit
    );
}

#[test]
fn surrounding_text_client_uses_managed_commit_even_with_cursor_width() {
    let mut engine = engine();
    engine.cursor_cell_width = 2;
    engine.surrounding_text_supported = true;

    assert_eq!(
        engine.initial_word_input_mode(),
        WordInputMode::ManagedCommit
    );
}

#[test]
fn cursor_driven_client_defers_preedit_until_cursor_ack() {
    let mut engine = engine();
    engine.cursor_cell_width = 11;

    assert!(engine.preedit_waits_for_cursor_ack());
}

#[test]
fn surrounding_text_client_publishes_preedit_immediately() {
    let mut engine = engine();
    engine.cursor_cell_width = 11;
    engine.surrounding_text_supported = true;

    assert!(!engine.preedit_waits_for_cursor_ack());
}

#[test]
fn identified_focus_publishes_preedit_without_cursor_ack() {
    let mut engine = engine();
    engine.cursor_cell_width = 11;

    assert!(engine.preedit_waits_for_cursor_ack());
    assert!(engine.bind_focus_receipt("/field/a".to_string(), "client-a".to_string()));
    assert!(!engine.preedit_waits_for_cursor_ack());
}

#[test]
fn changed_focus_receipt_quarantines_committed_tail() {
    let mut engine = engine();
    engine.bind_focus_receipt("/field/a".to_string(), "client-a".to_string());
    engine.tail_buffer = "старый ".to_string();
    engine.publish_tail_handoff();

    assert!(engine.bind_focus_receipt("/field/b".to_string(), "client-b".to_string()));
    assert!(engine.tail_buffer.is_empty());
    assert!(engine.shared.lock().expect("shared state").handoff_tail_buffer.is_empty());
}

#[test]
fn changed_engine_path_quarantines_handoff_without_focus_in_id() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut first = LayIbusEngine::new(
        "/engine/a".to_string(),
        Arc::clone(&shared),
        true,
        true,
        LayConfig::default(),
    );
    assert!(first.bind_focus_path());
    first.tail_buffer = "старый ".to_string();
    first.publish_tail_handoff();

    let mut second = LayIbusEngine::new(
        "/engine/b".to_string(),
        shared,
        true,
        true,
        LayConfig::default(),
    );
    assert!(second.bind_focus_path());
    assert!(second.tail_buffer.is_empty());
    assert!(second.shared.lock().expect("shared state").handoff_tail_buffer.is_empty());
}
