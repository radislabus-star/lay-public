use super::*;
use crate::boundary_runtime::{handle_hard_boundary_if_needed, HardBoundaryContext};

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

        let virtual_kbd = std::sync::Arc::new(std::sync::Mutex::new(None));
        let mut executing = false;
        let mut pending_typing_assist_after_space = None;
        let mut current_layout_is_ru = true;
        let mut last_layout_poll = Instant::now();
        let mut ignore_current_token_until_space = false;
        let mut events_since_word_start = 0;
        let shift_state = ShiftState::default();

        assert!(handle_hard_boundary_if_needed(
            key,
            1,
            HardBoundaryContext {
                buffer: &mut buffer,
                virtual_kbd: &virtual_kbd,
                executing: &mut executing,
                pending_typing_assist_after_space: &mut pending_typing_assist_after_space,
                current_layout_is_ru: &mut current_layout_is_ru,
                last_layout_poll: &mut last_layout_poll,
                ignore_current_token_until_space: &mut ignore_current_token_until_space,
                events_since_word_start: &mut events_since_word_start,
                shift_state: &shift_state,
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
