use super::{LayIbusEngine, ManualToggleAuthority, WordInputMode, IBUS_CAP_SURROUNDING_TEXT};
use crate::protocol::ShiftGestureHandoffAuthority;
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

fn arm_exact_cycle(
    engine: &mut LayIbusEngine,
    tail: &str,
    source_layout_is_ru: bool,
    target_layout_is_ru: bool,
) {
    engine.tail_buffer = tail.to_string();
    engine.layout_is_ru = target_layout_is_ru;
    engine.publish_tail_handoff();
    assert!(engine.arm_cyclic_layout_handoff(source_layout_is_ru, target_layout_is_ru));
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
fn ordinary_layout_cycle_handoff_is_exact_and_one_shot() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut source = LayIbusEngine::new(
        "/engine/us-cycle".to_string(),
        Arc::clone(&shared),
        false,
        true,
        LayConfig::default(),
    );
    assert!(source.bind_focus_path());
    arm_exact_cycle(&mut source, "Пере ", false, true);
    let pressed_at = Instant::now();
    let previous_release = pressed_at - Duration::from_millis(100);
    source.shift_active = true;
    source.shift_pressed_at = Some(pressed_at);
    source.last_shift_release_at = Some(previous_release);
    source.publish_shift_gesture_handoff();
    assert_eq!(
        shared
            .lock()
            .expect("shared state")
            .shift_gesture_handoff
            .as_ref()
            .map(|gesture| gesture.authority),
        Some(ShiftGestureHandoffAuthority::CyclicLayout)
    );

    let mut target = LayIbusEngine::new(
        "/engine/ru-cycle".to_string(),
        Arc::clone(&shared),
        true,
        true,
        LayConfig::default(),
    );
    assert!(target.bind_focus_path());
    target.consume_shift_gesture_handoff();

    assert!(target.shift_active);
    assert_eq!(target.shift_pressed_at, Some(pressed_at));
    assert_eq!(target.last_shift_release_at, Some(previous_release));
    {
        let state = shared.lock().expect("shared state");
        assert!(state.cyclic_layout_handoff.is_none());
        assert!(state.shift_gesture_handoff.is_none());
    }

    target.shift_active = false;
    target.shift_pressed_at = None;
    target.last_shift_release_at = None;
    target.consume_shift_gesture_handoff();
    assert!(!target.shift_active);
    assert!(target.shift_pressed_at.is_none());
    assert!(target.last_shift_release_at.is_none());
}

#[test]
fn cyclic_layout_handoff_can_be_rearmed_for_every_completed_cycle() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut current = LayIbusEngine::new(
        "/engine/cycle-0".to_string(),
        Arc::clone(&shared),
        false,
        true,
        LayConfig::default(),
    );
    assert!(current.bind_focus_path());
    let mut source_layout_is_ru = false;

    for cycle in 1..=6 {
        let target_layout_is_ru = !source_layout_is_ru;
        let tail = if target_layout_is_ru {
            "Пере "
        } else {
            "Gtht "
        };
        arm_exact_cycle(&mut current, tail, source_layout_is_ru, target_layout_is_ru);
        let previous_release = Instant::now() - Duration::from_millis(100);
        current.shift_active = false;
        current.shift_pressed_at = None;
        current.shift_used_as_modifier = false;
        current.last_shift_release_at = Some(previous_release);
        current.publish_shift_gesture_handoff();

        let mut next = LayIbusEngine::new(
            format!("/engine/cycle-{cycle}"),
            Arc::clone(&shared),
            target_layout_is_ru,
            true,
            LayConfig::default(),
        );
        assert!(next.bind_focus_path());
        next.consume_shift_gesture_handoff();
        assert_eq!(next.tail_buffer, tail);
        assert_eq!(next.last_shift_release_at, Some(previous_release));
        assert!(shared
            .lock()
            .expect("shared state")
            .cyclic_layout_handoff
            .is_none());

        current = next;
        source_layout_is_ru = target_layout_is_ru;
    }
}

#[test]
fn pending_auto_undo_keeps_priority_before_the_next_ordinary_cycle() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut source = LayIbusEngine::new(
        "/engine/undo-priority".to_string(),
        Arc::clone(&shared),
        true,
        true,
        LayConfig::default(),
    );
    assert!(source.bind_focus_path());
    source.tail_buffer = "вот ".to_string();
    source.publish_tail_handoff();
    source.remember_pending_ime_auto_undo(
        "djn ".to_string(),
        "вот ".to_string(),
        lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
    );
    source.publish_active_path_preserve_handoff(Instant::now() + Duration::from_millis(700));
    source.last_shift_release_at = Some(Instant::now() - Duration::from_millis(100));
    source.publish_shift_gesture_handoff();
    assert_eq!(
        shared
            .lock()
            .expect("shared state")
            .shift_gesture_handoff
            .as_ref()
            .map(|gesture| gesture.authority),
        Some(ShiftGestureHandoffAuthority::PendingAutoUndo)
    );

    assert!(source.take_pending_ime_auto_undo().is_some());
    arm_exact_cycle(&mut source, "djn ", true, false);
    source.last_shift_release_at = Some(Instant::now() - Duration::from_millis(100));
    source.publish_shift_gesture_handoff();
    assert_eq!(
        shared
            .lock()
            .expect("shared state")
            .shift_gesture_handoff
            .as_ref()
            .map(|gesture| gesture.authority),
        Some(ShiftGestureHandoffAuthority::CyclicLayout)
    );
}

#[test]
fn cyclic_layout_handoff_rejects_tail_epoch_mismatch() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut source = LayIbusEngine::new(
        "/engine/mismatch-us".to_string(),
        Arc::clone(&shared),
        false,
        true,
        LayConfig::default(),
    );
    assert!(source.bind_focus_path());
    arm_exact_cycle(&mut source, "Пере ", false, true);
    source.shift_active = true;
    source.shift_pressed_at = Some(Instant::now());
    source.publish_shift_gesture_handoff();

    let mut target = LayIbusEngine::new(
        "/engine/mismatch-ru".to_string(),
        shared,
        true,
        true,
        LayConfig::default(),
    );
    assert!(target.bind_focus_path());
    target.tail_epoch = target.tail_epoch.wrapping_add(1);
    target.consume_shift_gesture_handoff();

    assert!(!target.shift_active);
    let state = target.shared.lock().expect("shared state");
    assert!(state.cyclic_layout_handoff.is_none());
    assert!(state.shift_gesture_handoff.is_none());
}

#[test]
fn expired_or_modifier_used_cycle_never_becomes_a_tap() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut source = LayIbusEngine::new(
        "/engine/expiry-us".to_string(),
        Arc::clone(&shared),
        false,
        true,
        LayConfig::default(),
    );
    assert!(source.bind_focus_path());
    arm_exact_cycle(&mut source, "Пере ", false, true);
    source.shift_active = true;
    source.shift_pressed_at = Some(Instant::now());
    source.shift_used_as_modifier = true;
    source.last_shift_release_at = Some(Instant::now() - Duration::from_millis(100));
    source.publish_shift_gesture_handoff();

    let mut modifier_target = LayIbusEngine::new(
        "/engine/modifier-ru".to_string(),
        Arc::clone(&shared),
        true,
        true,
        LayConfig::default(),
    );
    assert!(modifier_target.bind_focus_path());
    modifier_target.consume_shift_gesture_handoff();
    assert!(modifier_target.shift_used_as_modifier);

    arm_exact_cycle(&mut modifier_target, "Gtht ", true, false);
    modifier_target.shift_active = true;
    modifier_target.shift_pressed_at = Some(Instant::now());
    modifier_target.publish_shift_gesture_handoff();
    {
        let mut state = shared.lock().expect("shared state");
        state.preserve_active_path_until = Some(Instant::now() - Duration::from_millis(1));
        if let Some(handoff) = state.cyclic_layout_handoff.as_mut() {
            handoff.expires_at = Instant::now() - Duration::from_millis(1);
        }
        if let Some(gesture) = state.shift_gesture_handoff.as_mut() {
            gesture.expires_at = Instant::now() - Duration::from_millis(1);
        }
    }

    let mut expired_target = LayIbusEngine::new(
        "/engine/expired-us".to_string(),
        shared,
        false,
        true,
        LayConfig::default(),
    );
    assert!(expired_target.bind_focus_path());
    expired_target.consume_shift_gesture_handoff();
    assert!(!expired_target.shift_active);
    assert!(expired_target.shift_pressed_at.is_none());
}

#[test]
fn generic_focus_preservation_cannot_publish_shift_authority() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut source = LayIbusEngine::new(
        "/engine/generic-a".to_string(),
        Arc::clone(&shared),
        false,
        true,
        LayConfig::default(),
    );
    assert!(source.bind_focus_path());
    source.tail_buffer = "tail ".to_string();
    source.publish_tail_handoff();
    source.publish_active_path_preserve_handoff(Instant::now() + Duration::from_millis(700));
    source.shift_active = true;
    source.shift_pressed_at = Some(Instant::now());
    source.publish_shift_gesture_handoff();
    assert!(shared
        .lock()
        .expect("shared state")
        .shift_gesture_handoff
        .is_none());

    let mut target = LayIbusEngine::new(
        "/engine/generic-b".to_string(),
        shared,
        true,
        true,
        LayConfig::default(),
    );
    assert!(target.bind_focus_path());
    target.consume_shift_gesture_handoff();
    assert!(!target.shift_active);
    assert!(target.shift_pressed_at.is_none());
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
