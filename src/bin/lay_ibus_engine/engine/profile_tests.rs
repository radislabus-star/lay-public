use super::{LayIbusEngine, ManualToggleAuthority, WordInputMode, IBUS_CAP_SURROUNDING_TEXT};
use lay::config::LayConfig;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
fn sensitive_content_types_disable_text_assistance() {
    let mut engine = engine();
    assert!(engine.content_allows_text_assistance());

    engine.set_content_type_state(8, 0);
    assert!(!engine.content_allows_text_assistance());

    engine.set_content_type_state(9, 0);
    assert!(!engine.content_allows_text_assistance());

    engine.set_content_type_state(10, 0);
    assert!(!engine.content_allows_text_assistance());

    engine.set_content_type_state(0, 1 << 11);
    assert!(!engine.content_allows_text_assistance());

    engine.set_content_type_state(0, 1 << 12);
    assert!(!engine.content_allows_text_assistance());

    engine.set_content_type_state(0, 0);
    assert!(engine.content_allows_text_assistance());
}

#[test]
fn sensitive_content_never_enters_committed_tail_memory() {
    let mut engine = engine();
    engine.push_tail_char('x');
    assert_eq!(engine.tail_buffer, "x");

    engine.set_content_type_state(8, 0);
    assert!(engine.tail_buffer.is_empty());

    engine.push_tail_char('p');
    assert!(engine.tail_buffer.is_empty());
    assert_eq!(engine.preedit_fast.token(), "");
}

#[test]
fn sensitive_content_discards_surrounding_text_snapshots() {
    let mut engine = engine();
    engine.observe_external_surrounding_text(Some(super::SurroundingTextSnapshot::new(
        "visible".to_string(),
        7,
        7,
    )));
    assert!(engine.surrounding_text_snapshot.is_some());

    engine.set_content_type_state(8, 0);
    engine.observe_external_surrounding_text(Some(super::SurroundingTextSnapshot::new(
        "secret".to_string(),
        6,
        6,
    )));

    assert!(engine.surrounding_text_snapshot.is_none());
}

#[test]
fn entering_sensitive_content_clears_visible_completion_state() {
    let mut engine = engine();
    engine.preedit_suffix = "ерить".to_string();
    engine.preedit_candidates = vec!["ерить".to_string()];
    engine.preedit_replacement_targets = vec![None];

    engine.set_content_type_state(8, 0);

    assert!(engine.preedit_suffix.is_empty());
    assert!(engine.preedit_candidates.is_empty());
    assert!(engine.preedit_replacement_targets.is_empty());
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
fn late_surrounding_text_capability_promotes_current_terminal_word() {
    let mut engine = engine();
    engine.cursor_cell_width = 2;
    engine.word_input_mode = Some(WordInputMode::TerminalPassthrough);
    engine.preedit_suffix = "suffix".to_string();

    engine.set_client_capabilities(1 << 5);

    assert_eq!(engine.word_input_mode, Some(WordInputMode::ManagedCommit));
    assert!(engine.pending_passthrough_preedit_clear);
    assert_eq!(engine.preedit_suffix, "suffix");
}

#[test]
fn capability_loss_does_not_demote_current_managed_word() {
    let mut engine = engine();
    engine.cursor_cell_width = 2;
    engine.word_input_mode = Some(WordInputMode::ManagedCommit);
    engine.set_client_capabilities(1 << 5);

    engine.set_client_capabilities(1 | 1 << 3);

    assert_eq!(engine.word_input_mode, Some(WordInputMode::ManagedCommit));
    assert!(!engine.pending_passthrough_preedit_clear);
}

#[test]
fn terminal_profile_stays_passthrough_without_surrounding_capability() {
    let mut engine = engine();
    engine.cursor_cell_width = 2;
    engine.word_input_mode = Some(WordInputMode::TerminalPassthrough);

    engine.set_client_capabilities(1 | 1 << 3);

    assert_eq!(
        engine.word_input_mode,
        Some(WordInputMode::TerminalPassthrough)
    );
    assert!(!engine.pending_passthrough_preedit_clear);
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
    assert!(engine
        .shared
        .lock()
        .expect("shared state")
        .handoff_tail_buffer
        .is_empty());
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
    first.remember_pending_ime_auto_undo(
        "старое ".to_string(),
        "старый ".to_string(),
        lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
    );
    first.publish_active_path_preserve_handoff(Instant::now() + Duration::from_millis(700));
    first.shift_active = true;
    first.shift_pressed_at = Some(Instant::now());
    first.publish_shift_gesture_handoff();
    first
        .shared
        .lock()
        .expect("shared state")
        .preserve_active_path_until = None;

    let mut second = LayIbusEngine::new(
        "/engine/b".to_string(),
        shared,
        true,
        true,
        LayConfig::default(),
    );
    assert!(second.bind_focus_path());
    assert!(second.tail_buffer.is_empty());
    let state = second.shared.lock().expect("shared state");
    assert!(state.handoff_tail_buffer.is_empty());
    assert!(state.pending_auto_undo.is_none());
    assert!(state.shift_gesture_handoff.is_none());
}

#[test]
fn shift_gesture_handoff_is_typed_and_one_shot() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut source = LayIbusEngine::new(
        "/engine/us".to_string(),
        Arc::clone(&shared),
        false,
        true,
        LayConfig::default(),
    );
    assert!(source.bind_focus_path());
    source.tail_buffer = "собака ".to_string();
    source.publish_tail_handoff();
    source.remember_pending_ime_auto_undo(
        "cj,frf ".to_string(),
        "собака ".to_string(),
        lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
    );
    source.publish_active_path_preserve_handoff(Instant::now() + Duration::from_millis(700));
    let pressed_at = Instant::now();
    let previous_release = pressed_at - Duration::from_millis(100);
    source.shift_active = true;
    source.shift_pressed_at = Some(pressed_at);
    source.shift_used_as_modifier = true;
    source.last_shift_release_at = Some(previous_release);
    source.publish_shift_gesture_handoff();

    let mut target = LayIbusEngine::new(
        "/engine/ru".to_string(),
        shared,
        true,
        true,
        LayConfig::default(),
    );
    assert!(target.bind_focus_path());
    target.consume_shift_gesture_handoff();

    assert!(target.shift_active);
    assert_eq!(target.shift_pressed_at, Some(pressed_at));
    assert!(target.shift_used_as_modifier);
    assert_eq!(target.last_shift_release_at, Some(previous_release));
    assert!(target
        .shared
        .lock()
        .expect("shared state")
        .shift_gesture_handoff
        .is_none());

    target.shift_active = false;
    target.shift_pressed_at = None;
    target.shift_used_as_modifier = false;
    target.last_shift_release_at = None;
    target.consume_shift_gesture_handoff();
    assert!(!target.shift_active);
    assert!(target.shift_pressed_at.is_none());
    assert!(target.last_shift_release_at.is_none());
}

#[test]
fn layout_switch_path_preserves_fresh_committed_tail_handoff() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut first = LayIbusEngine::new(
        "/engine/us".to_string(),
        Arc::clone(&shared),
        false,
        true,
        LayConfig::default(),
    );
    assert!(first.bind_focus_path());
    first.tail_buffer = "вот ".to_string();
    first.publish_tail_handoff();
    first.remember_pending_ime_auto_undo(
        "djn ".to_string(),
        "вот ".to_string(),
        lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
    );
    first.publish_active_path_preserve_handoff(Instant::now() + Duration::from_millis(700));

    let mut second = LayIbusEngine::new(
        "/engine/ru".to_string(),
        Arc::clone(&shared),
        true,
        true,
        LayConfig::default(),
    );
    second.set_client_capabilities(IBUS_CAP_SURROUNDING_TEXT);

    assert!(second.bind_focus_path());
    assert_eq!(second.tail_buffer, "вот ");
    assert_eq!(
        second.manual_toggle_authority(),
        ManualToggleAuthority::ImeCommittedTail
    );
    let pending = second
        .take_pending_ime_auto_undo()
        .expect("autocorrect undo must cross the layout handoff");
    assert_eq!(pending.original, "djn ");
    assert_eq!(pending.replacement, "вот ");
}

#[test]
fn layout_switch_double_shift_waits_for_exact_surrounding_snapshot() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut first = LayIbusEngine::new(
        "/engine/us".to_string(),
        Arc::clone(&shared),
        false,
        true,
        LayConfig::default(),
    );
    assert!(first.bind_focus_path());
    first.tail_buffer = "собака ".to_string();
    first.publish_tail_handoff();
    first.remember_pending_ime_auto_undo(
        "cj,frf ".to_string(),
        "собака ".to_string(),
        lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
    );
    first.publish_active_path_preserve_handoff(Instant::now() + Duration::from_millis(700));

    let mut second = LayIbusEngine::new(
        "/engine/ru".to_string(),
        Arc::clone(&shared),
        true,
        true,
        LayConfig::default(),
    );
    assert!(second.bind_focus_path());
    second.surrounding_text_supported = true;
    second.surrounding_text_snapshot =
        Some(super::SurroundingTextSnapshot::new(String::new(), 0, 0));

    assert!(second.defer_pending_ime_auto_undo_until_visible());
    assert_eq!(
        second.pending_ime_auto_undo_retry_status(),
        "waiting_exact_snapshot"
    );
    let preserve_remaining = second
        .shared
        .lock()
        .expect("shared state")
        .preserve_active_path_until
        .expect("retry must preserve engine handoff")
        .saturating_duration_since(Instant::now());
    assert!(preserve_remaining >= Duration::from_secs(4));

    let mut refreshed = LayIbusEngine::new(
        "/engine/ru-refresh".to_string(),
        shared,
        true,
        true,
        LayConfig::default(),
    );
    assert!(refreshed.bind_focus_path());
    refreshed.surrounding_text_supported = true;
    refreshed.surrounding_text_snapshot = Some(super::SurroundingTextSnapshot::new(
        "собака ".to_string(),
        7,
        7,
    ));
    assert_eq!(refreshed.pending_ime_auto_undo_retry_status(), "ready");

    let pending = refreshed
        .take_pending_ime_auto_undo()
        .expect("exact snapshot releases the recorded undo");
    assert_eq!(pending.original, "cj,frf ");
    assert_eq!(pending.replacement, "собака ");
    assert_eq!(refreshed.pending_ime_auto_undo_retry_status(), "none");
}

#[test]
fn recorded_undo_accepts_only_a_fresh_full_tail_boundary_elision() {
    let mut engine = engine();
    assert!(engine.bind_focus_path());
    engine.tail_buffer = "собака ".to_string();
    engine.publish_tail_handoff();
    engine.remember_pending_ime_auto_undo(
        "cj,frf ".to_string(),
        "собака ".to_string(),
        lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
    );
    engine.surrounding_text_supported = true;
    engine.surrounding_text_snapshot =
        Some(super::SurroundingTextSnapshot::new(String::new(), 0, 0));

    assert!(engine.defer_pending_ime_auto_undo_until_visible());
    engine.surrounding_text_snapshot = Some(super::SurroundingTextSnapshot::new(
        "собака".to_string(),
        6,
        6,
    ));

    assert_eq!(
        engine.pending_ime_auto_undo_retry_status(),
        "ready_boundary_elided"
    );
    assert!(engine.pending_ime_auto_undo_uses_boundary_elided_snapshot());

    engine.tail_buffer = "другая поверхность ".to_string();
    assert_eq!(engine.pending_ime_auto_undo_retry_status(), "invalidated");
    assert!(engine.take_pending_ime_auto_undo().is_none());
}

#[test]
fn recorded_undo_uses_an_already_visible_boundary_elision_without_waiting() {
    let mut engine = engine();
    assert!(engine.bind_focus_path());
    engine.tail_buffer = "собака ".to_string();
    engine.publish_tail_handoff();
    engine.remember_pending_ime_auto_undo(
        "cj,frf ".to_string(),
        "собака ".to_string(),
        lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
    );
    engine.surrounding_text_supported = true;
    engine.surrounding_text_snapshot = Some(super::SurroundingTextSnapshot::new(
        "собака".to_string(),
        6,
        6,
    ));

    assert!(!engine.defer_pending_ime_auto_undo_until_visible());
    assert_eq!(
        engine.pending_ime_auto_undo_retry_status(),
        "ready_boundary_elided"
    );
}

#[test]
fn recorded_undo_accepts_boundary_elision_inside_a_sentence_tail() {
    let mut engine = engine();
    assert!(engine.bind_focus_path());
    engine.tail_buffer = "контекст собака ".to_string();
    engine.publish_tail_handoff();
    engine.remember_pending_ime_auto_undo(
        "cj,frf ".to_string(),
        "собака ".to_string(),
        lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
    );
    engine.surrounding_text_supported = true;
    engine.surrounding_text_snapshot = Some(super::SurroundingTextSnapshot::new(
        "контекст собака".to_string(),
        15,
        15,
    ));

    assert!(!engine.defer_pending_ime_auto_undo_until_visible());
    assert_eq!(
        engine.pending_ime_auto_undo_retry_status(),
        "ready_boundary_elided"
    );
}

#[test]
fn expired_layout_switch_handoff_is_quarantined() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut first = LayIbusEngine::new(
        "/engine/us".to_string(),
        Arc::clone(&shared),
        false,
        true,
        LayConfig::default(),
    );
    assert!(first.bind_focus_path());
    first.tail_buffer = "старый ".to_string();
    first.publish_tail_handoff();
    first.remember_pending_ime_auto_undo(
        "старое ".to_string(),
        "старый ".to_string(),
        lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
    );
    first.publish_active_path_preserve_handoff(Instant::now() + Duration::from_millis(700));
    first.shift_active = true;
    first.shift_pressed_at = Some(Instant::now());
    first.publish_shift_gesture_handoff();
    first
        .shared
        .lock()
        .expect("shared state")
        .preserve_active_path_until = Some(Instant::now() - Duration::from_millis(1));

    let mut second = LayIbusEngine::new(
        "/engine/ru".to_string(),
        shared,
        true,
        true,
        LayConfig::default(),
    );

    assert!(second.bind_focus_path());
    assert!(second.tail_buffer.is_empty());
    assert_eq!(
        second.manual_toggle_authority(),
        ManualToggleAuthority::DaemonWordBuffer
    );
    assert!(second.take_pending_ime_auto_undo().is_none());
    assert!(second
        .shared
        .lock()
        .expect("shared state")
        .shift_gesture_handoff
        .is_none());
}
