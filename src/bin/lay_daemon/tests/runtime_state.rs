use super::*;
use crate::boundary_runtime::{handle_hard_boundary_if_needed, HardBoundaryContext};
use crate::pending_typing_assist::PendingTypingAssist;
use crate::trigger_dispatch::apply_manual_correction_result;

fn test_text_context() -> DaemonTextContext {
    DaemonTextContext::new(Some("test-field".to_string()), 0)
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
    for key in [
        KeyCode::KEY_BACKSPACE,
        KeyCode::KEY_DELETE,
        KeyCode::KEY_LEFT,
    ] {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "свло", true);
        assert!(!buffer.is_empty(), "precondition key={key:?}");

        let mut pending_typing_assist_after_space = None;
        let mut ignore_current_token_until_space = false;
        let mut events_since_word_start = 0;

        assert!(handle_hard_boundary_if_needed(
            key,
            1,
            HardBoundaryContext {
                buffer: &mut buffer,
                pending_typing_assist_after_space: &mut pending_typing_assist_after_space,
                ignore_current_token_until_space: &mut ignore_current_token_until_space,
                events_since_word_start: &mut events_since_word_start,
                verbose: false,
            },
        ));

        assert!(buffer.is_empty(), "buffer survived key={key:?}");
        assert!(pending_typing_assist_after_space.is_none());
        assert!(!ignore_current_token_until_space);
        assert_eq!(events_since_word_start, 0);
    }
}

#[test]
fn leading_cli_option_token_is_ignored_until_space() {
    for (leader, leader_shift, token_key, next_word) in [
        (KeyCode::KEY_MINUS, false, KeyCode::KEY_B, "feature"),
        (KeyCode::KEY_EQUAL, true, KeyCode::KEY_X, "script"),
    ] {
        let mut modifiers = ShiftState::default();
        modifiers.update(KeyCode::KEY_LEFTSHIFT, i32::from(leader_shift));
        let mut buffer = WordBuffer::new();
        let mut ignore_token =
            should_start_ignored_buffer_token(leader, &modifiers, buffer.current_is_empty());
        assert!(ignore_token);

        if !ignore_token {
            buffer.push(key_event(token_key, false));
        }
        assert!(buffer.current_is_empty());

        if ignore_token {
            ignore_token = false;
        } else {
            buffer.handle_space();
        }
        assert!(!ignore_token);
        assert!(!buffer.prev_had_trailing_space());

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
