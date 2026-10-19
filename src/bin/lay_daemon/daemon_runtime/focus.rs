use evdev::{Device, EventType, InputEvent, KeyCode};
use std::os::fd::AsRawFd;
use std::sync::atomic::{AtomicU64, Ordering};

use super::super::daemon_state::DaemonLoopState;
use super::super::{
    log, poll_focused_window_state, poll_focused_window_state_for_key_event,
    wait_for_keyboard_event_or_timeout, DShiftState, SHUTDOWN_REQUESTED,
};

pub(in super::super) fn listen_pointer(
    device_path: std::path::PathBuf,
    mut device: Device,
    field_context_epoch: std::sync::Arc<AtomicU64>,
    verbose: bool,
) -> std::io::Result<()> {
    device.set_nonblocking(true)?;
    let device_fd = device.as_raw_fd();
    log(&format!(
        "► слушаю pointer: {device_path:?} имя={:?}",
        device.name().unwrap_or("?")
    ));

    while !SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
        let fetched_events = {
            device
                .fetch_events()
                .map(|events| events.collect::<Vec<_>>())
        };
        let events: Vec<InputEvent> = match fetched_events {
            Ok(events) => events,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                wait_for_keyboard_event_or_timeout(
                    device_fd,
                    std::time::Duration::from_millis(500),
                )?;
                continue;
            }
            Err(e) => return Err(e),
        };

        for event in events {
            if event.event_type() != EventType::KEY || event.value() != 1 {
                continue;
            }
            let key = KeyCode::new(event.code());
            if matches!(
                key,
                KeyCode::BTN_LEFT
                    | KeyCode::BTN_RIGHT
                    | KeyCode::BTN_MIDDLE
                    | KeyCode::BTN_SIDE
                    | KeyCode::BTN_EXTRA
                    | KeyCode::BTN_FORWARD
                    | KeyCode::BTN_BACK
                    | KeyCode::BTN_TASK
            ) {
                let epoch = field_context_epoch.fetch_add(1, Ordering::AcqRel) + 1;
                if verbose {
                    log(&format!("► pointer context changed: field epoch {epoch}"));
                }
            }
        }
    }
    Ok(())
}

pub(super) fn update_focus_state(state: &mut DaemonLoopState, field_context_epoch: &AtomicU64) {
    let focus = poll_focused_window_state(&mut state.last_focus_ignore_poll);
    apply_focus_state(state, focus, field_context_epoch);
}

pub(super) fn update_focus_state_for_key_batch(
    events: &[InputEvent],
    state: &mut DaemonLoopState,
    field_context_epoch: &AtomicU64,
) {
    let has_key_event = events
        .iter()
        .any(|event| event.event_type() == EventType::KEY);
    let focus = if has_key_event {
        poll_focused_window_state_for_key_event(&mut state.last_focus_ignore_poll)
    } else {
        poll_focused_window_state(&mut state.last_focus_ignore_poll)
    };
    apply_focus_state(state, focus, field_context_epoch);
}

pub(super) fn sync_field_context_epoch(
    field_context_epoch: &AtomicU64,
    state: &mut DaemonLoopState,
) {
    let epoch = field_context_epoch.load(Ordering::Acquire);
    if state.switch_field_context_epoch(epoch) {
        log("► text context changed: switched field buffer");
        state.dshift_state = DShiftState::Idle;
        state.pending_multi_tap = None;
        state.single_pressed_at = None;
        state.single_other_key = false;
        state.alt_shift_layout_before = None;
        state.force_layout_hotkeys.cancel_pending();
    }
}

fn apply_focus_state(
    state: &mut DaemonLoopState,
    focus: Option<super::super::FocusedWindowState>,
    field_context_epoch: &AtomicU64,
) {
    let Some(focus) = focus else {
        return;
    };
    if focus.identity != state.focused_window_identity {
        state.field_context_epoch = field_context_epoch.fetch_add(1, Ordering::AcqRel) + 1;
    }
    let identity_changed = state.switch_window_input_state(focus.identity);
    if identity_changed {
        log("► focused window changed: switched text tail buffer");
        state.dshift_state = DShiftState::Idle;
        state.pending_multi_tap = None;
    }

    if focus.ignored != state.focus_ignored {
        if focus.ignored {
            log("► focused window unavailable or ignored: host lay paused");
        } else {
            log("► focused window accepted: host lay resumed");
        }
    }

    if focus.ignored {
        state.buffer.reset_all();
        state.events_since_word_start = 0;
        state.pending_typing_assist_after_space.take();
    }
    state.focus_ignored = focus.ignored;
}

#[cfg(test)]
mod device_recovery_tests {
    use super::*;
    use crate::{
        pending_typing_assist::PendingTypingAssist, ForceLayoutHotkeyContext, MultiTapPending,
    };

    #[test]
    fn overlapping_kde_poll_keeps_word_gesture_and_shared_epoch_but_service_loss_invalidates() {
        let mut state = DaemonLoopState::new(&lay::config::LayConfig::default(), false, true);
        state.switch_window_input_state(Some("contention-fixture".into()));
        state.field_context_epoch = 7;
        state.buffer.push(lay::keyboard::KeyEvent {
            keycode: KeyCode::KEY_A.code(),
            shift: false,
            layout_is_ru: false,
        });
        state.dshift_state = DShiftState::FirstPress;
        state.pending_multi_tap = Some(MultiTapPending {
            tap_count: 2,
            last_release: std::time::Instant::now(),
        });
        let epoch = AtomicU64::new(7);
        let (entered_tx, entered_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel::<()>();
        let holder = std::thread::spawn(move || {
            crate::kde_focus::with_occupied_provider_for_test(|| {
                entered_tx.send(()).unwrap();
                // Bound the fixture even if a regression assertion panics.
                let _ = release_rx.recv_timeout(std::time::Duration::from_secs(2));
            });
        });
        entered_rx
            .recv_timeout(std::time::Duration::from_secs(1))
            .unwrap();

        let poll = crate::focus_guard::kde_focus_poll_state(crate::kde_focus::read());
        apply_focus_state(&mut state, poll, &epoch);
        assert!(!state.focus_ignored);
        assert_eq!(
            state.focused_window_identity.as_deref(),
            Some("contention-fixture")
        );
        assert_eq!(epoch.load(Ordering::Acquire), 7);
        assert_eq!(state.field_context_epoch, 7);
        assert_eq!(
            state.buffer.what_to_replay(1).unwrap().0[0].keycode,
            KeyCode::KEY_A.code()
        );
        assert!(matches!(state.dshift_state, DShiftState::FirstPress));
        assert_eq!(state.pending_multi_tap.as_ref().unwrap().tap_count, 2);
        let exact = crate::focus_guard::exact_kde_window_identity(crate::kde_focus::read());
        assert!(exact.unwrap_err().contains("provider busy"));
        drop(release_tx);
        holder.join().unwrap();

        let loss = crate::focus_guard::kde_focus_poll_state(Err(
            crate::kde_focus::ReadError::Unavailable("controlled service loss".into()),
        ));
        apply_focus_state(&mut state, loss, &epoch);
        assert!(state.focus_ignored);
        assert!(state.focused_window_identity.is_none());
        assert_eq!(epoch.load(Ordering::Acquire), 8);
        assert_eq!(state.field_context_epoch, 8);
        assert!(state.buffer.current_is_empty());
        assert!(matches!(state.dshift_state, DShiftState::Idle));
        assert!(state.pending_multi_tap.is_none());
    }

    #[test]
    fn late_permissions_epoch_cancels_both_force_layout_keys_before_release() {
        let cfg = lay::config::LayConfig {
            force_layout_hotkeys: true,
            ..Default::default()
        };
        let output = std::sync::Arc::new(std::sync::Mutex::new(None));
        for key in [KeyCode::KEY_RIGHTCTRL, KeyCode::KEY_RIGHTALT] {
            let mut state = DaemonLoopState::new(&cfg, false, true);
            state.shift_tap_max = std::time::Duration::MAX;
            state.shift_state.update(KeyCode::KEY_LEFTSHIFT, 1);
            let previous_layout = state.current_layout_is_ru;
            let epoch = AtomicU64::new(0);
            for value in [1, 0] {
                if value == 0 {
                    epoch.store(1, Ordering::Release);
                    sync_field_context_epoch(&epoch, &mut state);
                }
                assert!(state.force_layout_hotkeys.handle_event(
                    key,
                    value,
                    ForceLayoutHotkeyContext {
                        buffer: &mut state.buffer,
                        virtual_kbd: &output,
                        executing: &mut state.executing,
                        current_layout_is_ru: &mut state.current_layout_is_ru,
                        last_layout_poll: &mut state.last_layout_poll,
                        shift_state: &mut state.shift_state,
                        dshift_state: &mut state.dshift_state,
                        pending_multi_tap: &mut state.pending_multi_tap,
                        single_pressed_at: &mut state.single_pressed_at,
                        last_double_at: &mut state.last_double_at,
                        clear_on_next_typing: &mut state.clear_on_next_typing,
                        shift_tap_max: state.shift_tap_max,
                        debounce_window: state.debounce_window,
                    },
                ));
            }
            assert!(
                state.last_double_at.is_none(),
                "stale force key must not fire"
            );
            assert!(!state.clear_on_next_typing);
            assert!(!state.executing);
            assert_eq!(state.current_layout_is_ru, previous_layout);
            assert!(state.shift_state.any(), "physical Shift remains held");
        }
    }

    #[test]
    fn late_permissions_epoch_cancels_idle_word_and_gesture_intent_without_losing_held_keys() {
        let mut state = DaemonLoopState::new(&lay::config::LayConfig::default(), false, true);
        state.switch_window_input_state(Some("device-recovery-fixture".into()));
        state.buffer.push(lay::keyboard::KeyEvent {
            keycode: KeyCode::KEY_A.code(),
            shift: false,
            layout_is_ru: false,
        });
        state.single_pressed_at = Some(std::time::Instant::now());
        state.single_other_key = true;
        state.alt_shift_layout_before = Some(true);
        state.dshift_state = DShiftState::FirstPress;
        state.pending_multi_tap = Some(MultiTapPending {
            tap_count: 2,
            last_release: std::time::Instant::now(),
        });
        state.pending_typing_assist_after_space =
            Some(PendingTypingAssist::waiting(7, state.daemon_text_context()));
        state.shift_state.update(KeyCode::KEY_LEFTSHIFT, 1);
        let epoch = AtomicU64::new(1);
        sync_field_context_epoch(&epoch, &mut state);
        assert!(state.buffer.current_is_empty());
        assert!(state.pending_typing_assist_after_space.is_none());
        assert!(state.pending_multi_tap.is_none());
        assert!(matches!(state.dshift_state, DShiftState::Idle));
        assert!(state.single_pressed_at.is_none());
        assert!(!state.single_other_key);
        assert!(state.alt_shift_layout_before.is_none());
        assert!(state.shift_state.any());
        state.single_pressed_at = Some(std::time::Instant::now());
        sync_field_context_epoch(&epoch, &mut state);
        assert!(
            state.single_pressed_at.is_some(),
            "unchanged epoch must not invalidate again"
        );
    }
}
