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
