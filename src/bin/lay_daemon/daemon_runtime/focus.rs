use evdev::{Device, EventType, InputEvent, KeyCode};
use std::os::fd::AsRawFd;
use std::sync::atomic::{AtomicU64, Ordering};

use super::super::daemon_state::DaemonLoopState;
use super::super::{
    log, poll_focused_window_state, poll_focused_window_state_for_key_event,
    wait_for_keyboard_event_or_timeout, DShiftState,
};

pub(in super::super) fn listen_pointer(
    device_path: std::path::PathBuf,
    field_context_epoch: std::sync::Arc<AtomicU64>,
    verbose: bool,
) -> std::io::Result<()> {
    let mut device = Device::open(&device_path)?;
    device.set_nonblocking(true)?;
    let device_fd = device.as_raw_fd();
    log(&format!(
        "► слушаю pointer: {device_path:?} имя={:?}",
        device.name().unwrap_or("?")
    ));

    loop {
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
                let epoch = field_context_epoch.fetch_add(1, Ordering::Relaxed) + 1;
                if verbose {
                    log(&format!("► pointer context changed: field epoch {epoch}"));
                }
            }
        }
    }
}

pub(super) fn update_focus_state(state: &mut DaemonLoopState) {
    let focus = poll_focused_window_state(&mut state.last_focus_ignore_poll);
    apply_focus_state(state, focus);
}

pub(super) fn update_focus_state_for_key_batch(events: &[InputEvent], state: &mut DaemonLoopState) {
    let has_key_event = events
        .iter()
        .any(|event| event.event_type() == EventType::KEY);
    let focus = if has_key_event {
        poll_focused_window_state_for_key_event(&mut state.last_focus_ignore_poll)
    } else {
        poll_focused_window_state(&mut state.last_focus_ignore_poll)
    };
    apply_focus_state(state, focus);
}

pub(super) fn sync_field_context_epoch(
    field_context_epoch: &AtomicU64,
    state: &mut DaemonLoopState,
) {
    let epoch = field_context_epoch.load(Ordering::Relaxed);
    if state.switch_field_context_epoch(epoch) {
        log("► text context changed: switched field buffer");
        state.dshift_state = DShiftState::Idle;
        state.pending_multi_tap = None;
    }
}

fn apply_focus_state(state: &mut DaemonLoopState, focus: Option<super::super::FocusedWindowState>) {
    let Some(focus) = focus else {
        return;
    };
    let identity_changed = state.switch_window_input_state(focus.identity);
    if identity_changed {
        log("► focused window changed: switched text tail buffer");
        state.dshift_state = DShiftState::Idle;
        state.pending_multi_tap = None;
    }

    if focus.ignored != state.focus_ignored {
        if focus.ignored {
            log("► focused window ignored: VM/remote viewer, host lay paused");
        } else {
            log("► focused window accepted: host lay resumed");
        }
    }

    if focus.ignored {
        state.buffer.reset_all();
        state.events_since_word_start = 0;
        state.pending_typing_assist_after_space.take();
        state.ignore_current_token_until_space = false;
    }
    state.focus_ignored = focus.ignored;
}
