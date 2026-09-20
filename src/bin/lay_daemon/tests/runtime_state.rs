use super::*;
use crate::boundary_runtime::{
    cancel_backspace_pending_assist_before_deferred_poll, handle_hard_boundary_if_needed,
    HardBoundaryContext,
};
use crate::buffer_filter_runtime::{should_skip_buffer_input, BufferFilterContext};
use crate::correction_memory_runtime::{
    remember_manual_text_correction, ManualTextCorrectionMemory,
};
use crate::pending_typing_assist::PendingTypingAssist;
use crate::trigger_dispatch::{
    apply_manual_correction_result, complete_manual_trigger, ManualTriggerCompletion,
};
use crate::typing_key_runtime::{handle_typing_key_press, TypingKeyContext};

fn test_text_context() -> DaemonTextContext {
    DaemonTextContext::new(Some("test-field".to_string()), 0)
}

fn default_shift_state() -> ShiftState {
    ShiftState::default()
}

#[test]
fn queued_completion_preserves_every_partial_successor_in_real_completion_path() {
    for successor in [
        DShiftState::FirstPress,
        DShiftState::WaitingSecond {
            first_release: Instant::now(),
        },
        DShiftState::SecondPress,
    ] {
        let mut current_layout_is_ru = false;
        let mut last_layout_poll = Instant::now();
        let mut suppress = false;
        let mut pending_assist = None;
        let mut shift_state = ShiftState::default();
        let mut dshift_state = successor;
        let mut pending_multi_tap = None;
        let mut last_double_at = None;
        let mut clear_on_next_typing = false;
        complete_manual_trigger(
            None,
            ManualTriggerCompletion {
                current_layout_is_ru: &mut current_layout_is_ru,
                last_layout_poll: &mut last_layout_poll,
                suppress_next_typing_assist_after_manual_replay: &mut suppress,
                pending_typing_assist_after_space: &mut pending_assist,
                shift_state: &mut shift_state,
                dshift_state: &mut dshift_state,
                pending_multi_tap: &mut pending_multi_tap,
                last_double_at: &mut last_double_at,
                clear_on_next_typing: &mut clear_on_next_typing,
                preserve_queued_dshift_state: true,
            },
        );
        assert_eq!(dshift_state, successor);
        assert!(clear_on_next_typing);
        assert!(last_double_at.is_some());
    }
}

fn pending_typing_assist() -> Option<PendingTypingAssist> {
    let prepared = typed_buffer_from_semicolon_fixture("rfr @us");
    let correction =
        find_typing_assist_correction(&prepared, true, 1).expect("pending correction exists");
    Some(PendingTypingAssist::new(correction, test_text_context()))
}

fn hard_context<'a>(
    buffer: &'a mut WordBuffer,
    pending: &'a mut Option<PendingTypingAssist>,
    events_since_word_start: &'a mut u32,
    shift_state: &'a ShiftState,
    clear_on_next_typing: &'a mut bool,
) -> HardBoundaryContext<'a> {
    HardBoundaryContext {
        buffer,
        pending_typing_assist_after_space: pending,
        events_since_word_start,
        shift_state,
        clear_on_next_typing,
        verbose: false,
    }
}

#[test]
fn idle_wait_uses_long_sleep_when_no_internal_deadlines() {
    let now = Instant::now();

    assert_eq!(
        idle_wait_timeout_at(now, None, now, Duration::from_millis(120)),
        Duration::from_millis(IDLE_EVENT_WAIT_MAX_MS)
    );
}

#[test]
fn idle_wait_keeps_multi_tap_deadline_precise() {
    let now = Instant::now();
    let pending = MultiTapPending {
        tap_count: 2,
        last_release: now - Duration::from_millis(80),
    };

    assert_eq!(
        idle_wait_timeout_at(now, Some(&pending), now, Duration::from_millis(120)),
        Duration::from_millis(40)
    );
}

#[test]
fn idle_wait_returns_zero_when_a_deadline_is_due() {
    let now = Instant::now();
    let pending = MultiTapPending {
        tap_count: 2,
        last_release: now - Duration::from_millis(120),
    };

    assert_eq!(
        idle_wait_timeout_at(now, Some(&pending), now, Duration::from_millis(120),),
        Duration::ZERO
    );
}

#[test]
fn shift_state_cleanup_after_trigger_keeps_shortcuts_but_drops_caps() {
    let mut state = ShiftState::default();
    state.update(KeyCode::KEY_LEFTSHIFT, 1);
    state.update(KeyCode::KEY_RIGHTSHIFT, 1);
    state.update(KeyCode::KEY_LEFTCTRL, 1);

    assert!(state.any());
    assert!(state.shortcut_active());

    state.clear_shifts();

    assert!(!state.any());
    assert!(state.shortcut_active());
}

#[test]
fn double_shift_depends_on_key_sequence_not_hold_duration() {
    let start = Instant::now();
    let window = Duration::from_millis(800);
    let mut state = DShiftState::Idle;

    state.trigger_press(start, window);
    assert_eq!(
        state.trigger_release(start + Duration::from_secs(2)),
        DShiftRelease::None
    );
    state.trigger_press(start + Duration::from_millis(2100), window);
    assert_eq!(
        state.trigger_release(start + Duration::from_secs(4)),
        DShiftRelease::Double
    );
    assert!(state.is_idle());
}

#[test]
fn another_key_press_cancels_every_partial_double_shift_phase() {
    let start = Instant::now();
    let window = Duration::from_millis(800);

    for mut state in [
        DShiftState::FirstPress,
        DShiftState::WaitingSecond {
            first_release: start,
        },
        DShiftState::SecondPress,
        DShiftState::AdditionalPress,
    ] {
        state.cancel();
        assert!(state.is_idle());
    }

    let mut expired = DShiftState::WaitingSecond {
        first_release: start,
    };
    expired.trigger_press(start + Duration::from_millis(801), window);
    assert_eq!(expired, DShiftState::FirstPress);
}

#[test]
fn completed_double_shift_rearms_immediately_for_every_next_pair() {
    let start = Instant::now();
    let window = Duration::from_millis(800);
    let mut state = DShiftState::Idle;

    state.trigger_press(start, window);
    assert_eq!(
        state.trigger_release(start + Duration::from_millis(10)),
        DShiftRelease::None
    );
    state.trigger_press(start + Duration::from_millis(20), window);
    assert_eq!(
        state.trigger_release(start + Duration::from_millis(30)),
        DShiftRelease::Double
    );

    assert!(state.is_idle());
    state.trigger_press(start + Duration::from_millis(40), window);
    assert_eq!(
        state.trigger_release(start + Duration::from_millis(50)),
        DShiftRelease::None
    );
    state.trigger_press(start + Duration::from_millis(60), window);
    assert_eq!(
        state.trigger_release(start + Duration::from_millis(70)),
        DShiftRelease::Double
    );
    assert!(state.is_idle());
}

#[test]
fn double_left_shift_cannot_be_delayed_by_multi_tap_scope() {
    let mut config = LayConfig {
        multi_tap_scope: true,
        ..LayConfig::default()
    };
    config.trigger = "double-lshift".to_string();

    assert!(!crate::daemon_state::active_multi_tap_scope(
        &config, false, false
    ));
}

#[test]
fn marks_current_word_after_replay_for_next_toggle() {
    let mut buffer = WordBuffer::new();
    for key in [
        KeyCode::KEY_D,
        KeyCode::KEY_H,
        KeyCode::KEY_T,
        KeyCode::KEY_V,
        KeyCode::KEY_Z,
    ] {
        buffer.push(key_event(key, false));
    }

    buffer.mark_replayed_layout(1, true);
    let (events, _) = buffer.what_to_replay(1).expect("word is buffered");

    assert!(events.iter().all(|event| event.layout_is_ru));
    assert!(buffer.replay_toggle_ready());
}

#[test]
fn short_fragments_force_replay_without_llm() {
    assert!(should_force_replay_for_short_fragment("N"));
    assert!(should_force_replay_for_short_fragment("gh"));
    assert!(should_force_replay_for_short_fragment("т"));
    assert!(!should_force_replay_for_short_fragment("ghb"));
    assert!(!should_force_replay_for_short_fragment("a b"));
    assert!(!should_force_replay_for_short_fragment(""));
}

#[test]
fn typing_assist_after_space_is_suppressed_once_after_manual_replay() {
    let mut suppress_once = true;

    assert!(!should_schedule_typing_assist_after_space(
        true,
        &mut suppress_once
    ));
    assert!(!suppress_once);
    assert!(should_schedule_typing_assist_after_space(
        true,
        &mut suppress_once
    ));
    assert!(!should_schedule_typing_assist_after_space(
        false,
        &mut suppress_once
    ));
}

#[test]
fn successful_manual_replay_clears_already_pending_typing_assist() {
    let buffer = typed_buffer_from_semicolon_fixture("djn @us");
    let correction =
        find_typing_assist_correction(&buffer, true, 1).expect("pending correction exists");
    let mut pending = Some(PendingTypingAssist::new(correction, test_text_context()));
    let mut current_layout_is_ru = true;
    let mut last_layout_poll = Instant::now() - Duration::from_secs(10);
    let mut suppress_once = false;

    apply_manual_correction_result(
        Some(false),
        &mut current_layout_is_ru,
        &mut last_layout_poll,
        &mut suppress_once,
        &mut pending,
    );

    assert!(!current_layout_is_ru);
    assert!(suppress_once);
    assert!(pending.is_none());
}

#[test]
fn failed_manual_replay_keeps_already_pending_typing_assist() {
    let buffer = typed_buffer_from_semicolon_fixture("djn @us");
    let correction =
        find_typing_assist_correction(&buffer, true, 1).expect("pending correction exists");
    let mut pending = Some(PendingTypingAssist::new(correction, test_text_context()));
    let mut current_layout_is_ru = true;
    let mut last_layout_poll = Instant::now() - Duration::from_secs(10);
    let mut suppress_once = false;

    apply_manual_correction_result(
        None,
        &mut current_layout_is_ru,
        &mut last_layout_poll,
        &mut suppress_once,
        &mut pending,
    );

    assert!(current_layout_is_ru);
    assert!(!suppress_once);
    assert!(pending.is_some());
}

#[test]
fn typing_assist_runs_on_space_release_when_pending() {
    assert!(should_run_typing_assist_on_space_release(
        true, true, false, false
    ));
    assert!(!should_run_typing_assist_on_space_release(
        false, true, false, false
    ));
    assert!(!should_run_typing_assist_on_space_release(
        true, false, false, false
    ));
    assert!(!should_run_typing_assist_on_space_release(
        true, true, true, false
    ));
    assert!(!should_run_typing_assist_on_space_release(
        true, true, false, true
    ));
}

#[test]
fn edit_navigation_boundaries_reset_word_buffer_before_next_autocorrect() {
    for (key, shifted) in [
        (KeyCode::KEY_BACKSPACE, true),
        (KeyCode::KEY_DELETE, false),
        (KeyCode::KEY_LEFT, false),
    ] {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "свло", true);
        assert!(!buffer.is_empty(), "precondition key={key:?}");

        let mut pending_typing_assist_after_space = None;
        let mut events_since_word_start = 0;
        let mut shift_state = default_shift_state();
        shift_state.update(KeyCode::KEY_LEFTSHIFT, i32::from(shifted));
        let mut clear_on_next_typing = false;

        assert!(handle_hard_boundary_if_needed(
            key,
            1,
            hard_context(
                &mut buffer,
                &mut pending_typing_assist_after_space,
                &mut events_since_word_start,
                &shift_state,
                &mut clear_on_next_typing,
            ),
        ));

        assert!(buffer.is_empty(), "buffer survived key={key:?}");
        assert!(pending_typing_assist_after_space.is_none());
        assert_eq!(events_since_word_start, 0);
    }
}

#[test]
fn plain_backspace_inside_known_current_word_preserves_shortened_raw_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "может", true);
    buffer.remember_pending_auto_undo("typing-assist", "можешь", "может", 1, 1);
    let mut pending_typing_assist_after_space = None;
    let mut events_since_word_start = buffer.current_len() as u32;
    let shift_state = default_shift_state();
    let mut clear_on_next_typing = false;

    assert!(handle_hard_boundary_if_needed(
        KeyCode::KEY_BACKSPACE,
        1,
        hard_context(
            &mut buffer,
            &mut pending_typing_assist_after_space,
            &mut events_since_word_start,
            &shift_state,
            &mut clear_on_next_typing,
        ),
    ));

    assert_eq!(buffer.current_len(), 4);
    assert_eq!(events_since_word_start, 4);
    assert!(!buffer.pending_auto_undo_ready());
    let (events, backspaces) = buffer
        .what_to_replay(1)
        .expect("shortened current word remains buffered");
    assert_eq!(map_original_events(&events), "може");
    assert_eq!(backspaces, 4);
}

#[test]
fn modified_backspace_resets_buffer_and_clears_learning() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "может", true);
    buffer.remember_pending_learning_correction("typing-assist", "смотри", "смотрин", 1, 1);
    let mut pending_typing_assist_after_space = pending_typing_assist();
    let mut events_since_word_start = buffer.current_len() as u32;
    let mut shift_state = default_shift_state();
    shift_state.update(KeyCode::KEY_LEFTCTRL, 1);
    let mut clear_on_next_typing = false;

    assert!(handle_hard_boundary_if_needed(
        KeyCode::KEY_BACKSPACE,
        1,
        hard_context(
            &mut buffer,
            &mut pending_typing_assist_after_space,
            &mut events_since_word_start,
            &shift_state,
            &mut clear_on_next_typing,
        ),
    ));

    assert!(buffer.is_empty());
    assert!(pending_typing_assist_after_space.is_none());
    assert_eq!(events_since_word_start, 0);
    buffer.note_learning_backspace();
    buffer.note_learning_typed(key_event(KeyCode::KEY_C, true));
    assert!(buffer.take_user_learning_correction(false).is_none());
}

#[test]
fn stale_clear_on_next_typing_blocks_backspace_pop_and_resets() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "может", true);
    let mut pending_typing_assist_after_space = None;
    let mut events_since_word_start = buffer.current_len() as u32;
    let shift_state = default_shift_state();
    let mut clear_on_next_typing = true;

    assert!(handle_hard_boundary_if_needed(
        KeyCode::KEY_BACKSPACE,
        1,
        hard_context(
            &mut buffer,
            &mut pending_typing_assist_after_space,
            &mut events_since_word_start,
            &shift_state,
            &mut clear_on_next_typing,
        ),
    ));

    assert!(buffer.is_empty());
    assert_eq!(events_since_word_start, 0);
    assert!(clear_on_next_typing);
}

#[test]
fn unknown_current_event_mapping_resets_raw_buffer_but_preserves_verified_learning() {
    for events in [
        [
            key_event(KeyCode::KEY_A, false),
            lay::keyboard::KeyEvent {
                keycode: KeyCode::KEY_F13.code(),
                shift: false,
                layout_is_ru: true,
            },
        ],
        [
            lay::keyboard::KeyEvent {
                keycode: KeyCode::KEY_F13.code(),
                shift: false,
                layout_is_ru: true,
            },
            key_event(KeyCode::KEY_A, false),
        ],
    ] {
        let mut buffer = WordBuffer::new();
        for event in events {
            buffer.push(event);
        }
        buffer.remember_pending_learning_correction("typing-assist", "abc", "abd", 1, 1);
        let mut pending_typing_assist_after_space = None;
        let mut events_since_word_start = buffer.current_len() as u32;
        let shift_state = default_shift_state();
        let mut clear_on_next_typing = false;

        assert!(handle_hard_boundary_if_needed(
            KeyCode::KEY_BACKSPACE,
            1,
            hard_context(
                &mut buffer,
                &mut pending_typing_assist_after_space,
                &mut events_since_word_start,
                &shift_state,
                &mut clear_on_next_typing,
            ),
        ));

        assert!(buffer.is_empty());
        assert_eq!(events_since_word_start, 0);
        buffer.note_learning_typed(key_event(KeyCode::KEY_C, false));
        let correction = buffer
            .take_user_learning_correction(false)
            .expect("verified target learning survives raw reset");
        assert_eq!(correction.from, "d");
        assert_eq!(correction.to, "c");
    }
}

#[test]
fn clear_on_next_typing_vetoes_raw_pop_but_keeps_manual_target_learning() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "можешь", true);
    let (events, _) = buffer.what_to_replay(1).expect("manual source word");
    let original = map_original_events(&events);
    let replacement = "может";
    let plan = plan_text_replacement(&original, replacement).expect("manual replacement plan");
    remember_manual_text_correction(
        &mut buffer,
        ManualTextCorrectionMemory {
            events: &events,
            plan: &plan,
            original: &original,
            replacement,
            kind: "manual-toggle",
            replace_words: 1,
            words: 1,
            inserted_layout_is_ru: Some(true),
        },
    );
    let mut pending_typing_assist_after_space = None;
    let mut events_since_word_start = buffer.current_len() as u32;
    let shift_state = default_shift_state();
    let mut clear_on_next_typing = true;

    assert!(handle_hard_boundary_if_needed(
        KeyCode::KEY_BACKSPACE,
        1,
        hard_context(
            &mut buffer,
            &mut pending_typing_assist_after_space,
            &mut events_since_word_start,
            &shift_state,
            &mut clear_on_next_typing,
        ),
    ));
    for event in lay::keyboard::text_to_key_events("шь", true).expect("typed correction events") {
        buffer.note_learning_typed(event);
    }

    assert!(buffer.is_empty());
    assert_eq!(events_since_word_start, 0);
    assert!(clear_on_next_typing);
    let correction = buffer
        .take_user_learning_correction(false)
        .expect("manual target learning survives stale raw buffer");
    assert_eq!(correction.from, "т");
    assert_eq!(correction.to, "шь");
    assert_eq!(correction.user_target().as_deref(), Some("можешь"));
}

#[test]
fn plain_backspace_at_word_boundary_resets_without_reviving_previous_word() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "до м", true);
    let mut pending_typing_assist_after_space = None;
    let mut events_since_word_start = 1;
    let shift_state = default_shift_state();
    let mut clear_on_next_typing = false;

    assert!(handle_hard_boundary_if_needed(
        KeyCode::KEY_BACKSPACE,
        1,
        hard_context(
            &mut buffer,
            &mut pending_typing_assist_after_space,
            &mut events_since_word_start,
            &shift_state,
            &mut clear_on_next_typing,
        ),
    ));

    assert!(buffer.is_empty());
    assert_eq!(events_since_word_start, 0);
    assert!(buffer.what_to_replay(1).is_none());
}

#[test]
fn backspace_release_does_not_delete_or_reset_buffer_state() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "может", true);
    let mut pending_typing_assist_after_space = None;
    let mut events_since_word_start = buffer.current_len() as u32;
    let shift_state = default_shift_state();
    let mut clear_on_next_typing = false;

    assert!(handle_hard_boundary_if_needed(
        KeyCode::KEY_BACKSPACE,
        0,
        hard_context(
            &mut buffer,
            &mut pending_typing_assist_after_space,
            &mut events_since_word_start,
            &shift_state,
            &mut clear_on_next_typing,
        ),
    ));

    assert_eq!(buffer.current_len(), 5);
    assert_eq!(events_since_word_start, 5);
    let (events, backspaces) = buffer.what_to_replay(1).expect("word remains buffered");
    assert_eq!(map_original_events(&events), "может");
    assert_eq!(backspaces, 5);
}

#[test]
fn backspace_autorepeat_deletes_one_known_event_without_release_effects() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "может", true);
    let mut pending_typing_assist_after_space = None;
    let mut events_since_word_start = buffer.current_len() as u32;
    let shift_state = default_shift_state();
    let mut clear_on_next_typing = false;

    assert!(handle_hard_boundary_if_needed(
        KeyCode::KEY_BACKSPACE,
        2,
        hard_context(
            &mut buffer,
            &mut pending_typing_assist_after_space,
            &mut events_since_word_start,
            &shift_state,
            &mut clear_on_next_typing,
        ),
    ));

    assert_eq!(buffer.current_len(), 4);
    assert_eq!(events_since_word_start, 4);
    let (events, backspaces) = buffer
        .what_to_replay(1)
        .expect("autorepeat removes one terminal key event");
    assert_eq!(map_original_events(&events), "може");
    assert_eq!(backspaces, 4);
}

#[test]
fn pending_ready_typing_assist_is_cancelled_by_backspace_even_when_buffer_empty() {
    let mut pending = pending_typing_assist();
    pending
        .as_mut()
        .expect("pending correction")
        .note_separator_released();
    let mut buffer = WordBuffer::new();
    let mut events_since_word_start = 0;
    let shift_state = default_shift_state();
    let mut clear_on_next_typing = false;

    assert!(handle_hard_boundary_if_needed(
        KeyCode::KEY_BACKSPACE,
        1,
        hard_context(
            &mut buffer,
            &mut pending,
            &mut events_since_word_start,
            &shift_state,
            &mut clear_on_next_typing,
        ),
    ));

    assert!(pending.is_none());
    assert_eq!(events_since_word_start, 0);
}

#[test]
fn pre_poll_backspace_cancels_pending_assist_without_mutating_buffer() {
    let mut pending = pending_typing_assist();
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "может", true);

    assert!(cancel_backspace_pending_assist_before_deferred_poll(
        KeyCode::KEY_BACKSPACE,
        1,
        &mut pending,
    ));

    assert!(pending.is_none());
    assert_eq!(buffer.current_len(), 5);
    let (events, backspaces) = buffer.what_to_replay(1).expect("buffer unchanged");
    assert_eq!(map_original_events(&events), "может");
    assert_eq!(backspaces, 5);
}

#[test]
fn pre_poll_backspace_release_does_not_cancel_pending_assist() {
    let mut pending = pending_typing_assist();

    assert!(!cancel_backspace_pending_assist_before_deferred_poll(
        KeyCode::KEY_BACKSPACE,
        0,
        &mut pending,
    ));

    assert!(pending.is_some());
}

#[test]
fn ignored_shortcut_and_shift_insert_invalidate_buffer_learning_and_pending_assist() {
    for (key, modifier_key) in [
        (KeyCode::KEY_A, KeyCode::KEY_LEFTCTRL),
        (KeyCode::KEY_INSERT, KeyCode::KEY_LEFTSHIFT),
    ] {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "может", true);
        buffer.remember_pending_learning_correction("typing-assist", "abc", "abd", 1, 1);
        let mut pending = pending_typing_assist();
        let mut events_since_word_start = buffer.current_len() as u32;
        let mut clear_on_next_typing = true;
        let mut shift_state = default_shift_state();
        shift_state.update(modifier_key, 1);

        assert!(should_skip_buffer_input(BufferFilterContext {
            key,
            code: key.code(),
            shift_state: &shift_state,
            buffer: &mut buffer,
            pending_typing_assist_after_space: &mut pending,
            events_since_word_start: &mut events_since_word_start,
            clear_on_next_typing: &mut clear_on_next_typing,
            verbose: false,
        }));

        assert!(buffer.is_empty(), "buffer survived key={key:?}");
        assert!(pending.is_none(), "pending assist survived key={key:?}");
        assert_eq!(events_since_word_start, 0, "events survived key={key:?}");
        assert!(
            clear_on_next_typing,
            "stale clear flag was cleared before next typing key={key:?}"
        );
        buffer.note_learning_backspace();
        buffer.note_learning_typed(key_event(KeyCode::KEY_C, false));
        assert!(buffer.take_user_learning_correction(false).is_none());
    }
}

#[test]
fn raw_reset_keeps_manual_suppression_until_next_typing_consumes_pair() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "может", true);
    let mut pending_typing_assist_after_space = None;
    let mut events_since_word_start = buffer.current_len() as u32;
    let shift_state = default_shift_state();
    let mut clear_on_next_typing = true;
    let mut suppress_next_typing_assist_after_manual_replay = true;

    assert!(handle_hard_boundary_if_needed(
        KeyCode::KEY_BACKSPACE,
        1,
        hard_context(
            &mut buffer,
            &mut pending_typing_assist_after_space,
            &mut events_since_word_start,
            &shift_state,
            &mut clear_on_next_typing,
        ),
    ));
    assert!(clear_on_next_typing);
    assert!(suppress_next_typing_assist_after_manual_replay);

    let mut current_layout_is_ru = true;
    let mut last_layout_poll = Instant::now();
    handle_typing_key_press(
        KeyCode::KEY_C.code(),
        1,
        TypingKeyContext {
            buffer: &mut buffer,
            shift_state: &shift_state,
            current_layout_is_ru: &mut current_layout_is_ru,
            last_layout_poll: &mut last_layout_poll,
            events_since_word_start: &mut events_since_word_start,
            clear_on_next_typing: &mut clear_on_next_typing,
            suppress_next_typing_assist_after_manual_replay:
                &mut suppress_next_typing_assist_after_manual_replay,
            pending_typing_assist_after_space: &mut pending_typing_assist_after_space,
            verbose: false,
        },
    );

    assert!(!clear_on_next_typing);
    assert!(!suppress_next_typing_assist_after_manual_replay);
    assert!(should_schedule_typing_assist_after_space(
        true,
        &mut suppress_next_typing_assist_after_manual_replay
    ));
}

#[test]
fn ignored_shortcut_keeps_manual_suppression_until_next_typing_consumes_pair() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "может", true);
    let mut pending = pending_typing_assist();
    let mut events_since_word_start = buffer.current_len() as u32;
    let mut clear_on_next_typing = true;
    let mut suppress_next_typing_assist_after_manual_replay = true;
    let mut shift_state = default_shift_state();
    shift_state.update(KeyCode::KEY_LEFTCTRL, 1);

    assert!(should_skip_buffer_input(BufferFilterContext {
        key: KeyCode::KEY_A,
        code: KeyCode::KEY_A.code(),
        shift_state: &shift_state,
        buffer: &mut buffer,
        pending_typing_assist_after_space: &mut pending,
        events_since_word_start: &mut events_since_word_start,
        clear_on_next_typing: &mut clear_on_next_typing,
        verbose: false,
    }));
    assert!(clear_on_next_typing);
    assert!(suppress_next_typing_assist_after_manual_replay);

    shift_state.update(KeyCode::KEY_LEFTCTRL, 0);
    let mut current_layout_is_ru = true;
    let mut last_layout_poll = Instant::now();
    handle_typing_key_press(
        KeyCode::KEY_C.code(),
        1,
        TypingKeyContext {
            buffer: &mut buffer,
            shift_state: &shift_state,
            current_layout_is_ru: &mut current_layout_is_ru,
            last_layout_poll: &mut last_layout_poll,
            events_since_word_start: &mut events_since_word_start,
            clear_on_next_typing: &mut clear_on_next_typing,
            suppress_next_typing_assist_after_manual_replay:
                &mut suppress_next_typing_assist_after_manual_replay,
            pending_typing_assist_after_space: &mut pending,
            verbose: false,
        },
    );

    assert!(!clear_on_next_typing);
    assert!(!suppress_next_typing_assist_after_manual_replay);
    assert!(should_schedule_typing_assist_after_space(
        true,
        &mut suppress_next_typing_assist_after_manual_replay
    ));
}

#[test]
fn within_word_backspace_then_suffix_space_keeps_complete_raw_usage_word() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "может", true);
    let mut pending_typing_assist_after_space = None;
    let mut events_since_word_start = buffer.current_len() as u32;
    let shift_state = default_shift_state();
    let mut clear_on_next_typing = false;

    assert!(handle_hard_boundary_if_needed(
        KeyCode::KEY_BACKSPACE,
        1,
        hard_context(
            &mut buffer,
            &mut pending_typing_assist_after_space,
            &mut events_since_word_start,
            &shift_state,
            &mut clear_on_next_typing,
        ),
    ));
    push_text_as_layout(&mut buffer, "шь ", true);

    let word = buffer.prev_word_events(0).expect("completed raw word");
    assert_eq!(map_original_events(word), "можешь");
    assert_eq!(buffer.current_len(), 0);
    assert!(buffer.prev_had_trailing_space());
}

#[test]
fn leading_cli_option_token_is_retained_through_space() {
    for (leader, leader_shift, token_key, option, next_word) in [
        (KeyCode::KEY_MINUS, false, KeyCode::KEY_B, "-b", "feature"),
        (KeyCode::KEY_EQUAL, true, KeyCode::KEY_X, "+x", "script"),
    ] {
        let mut modifiers = ShiftState::default();
        modifiers.update(KeyCode::KEY_LEFTSHIFT, i32::from(leader_shift));
        let mut buffer = WordBuffer::new();
        assert!(!should_ignore_buffer_key(leader, &modifiers));
        buffer.push(key_event_with_shift(leader, leader_shift, false));
        buffer.push(key_event(token_key, false));
        let (events, erase) = buffer.what_to_replay(1).expect("option");
        assert_eq!(map_original_events(&events), option);
        assert_eq!(erase, 2);

        buffer.handle_space();
        assert!(buffer.prev_had_trailing_space());
        let (events, erase) = buffer.what_to_replay(1).expect("completed option");
        assert_eq!(map_original_events(&events), format!("{option} "));
        assert_eq!(erase, 3);

        push_text_as_layout(&mut buffer, next_word, false);
        let (events, _) = buffer.what_to_replay(1).expect("word");
        assert_eq!(map_original_events(&events), next_word);
    }
}

#[test]
fn multi_tap_scope_design_contract_maps_taps_to_scope() {
    assert_eq!(multi_tap_scope_for_taps(0), None);
    assert_eq!(multi_tap_scope_for_taps(1), None);
    assert_eq!(multi_tap_scope_for_taps(2), Some(1));
    assert_eq!(multi_tap_scope_for_taps(3), Some(2));
    assert_eq!(multi_tap_scope_for_taps(4), Some(3));
    assert_eq!(multi_tap_scope_for_taps(5), Some(3));
}

#[test]
fn typing_after_replay_clears_toggle_shortcut() {
    let mut buffer = WordBuffer::new();
    buffer.push(key_event(KeyCode::KEY_D, false));
    buffer.mark_replayed_layout(1, true);

    buffer.push(key_event(KeyCode::KEY_H, true));

    assert!(!buffer.replay_toggle_ready());
}
