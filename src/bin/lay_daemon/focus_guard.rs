use crate::trigger_fsm::MultiTapPending;
use lay::desktop::LayoutBackend;
use lay::word_buffer::WordBuffer;
use std::os::fd::RawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use super::{active_layout_backend, call_focused_window_info, log};

pub(super) const FOCUS_IGNORE_POLL_INTERVAL_MS: u64 = 500;
pub(super) const IDLE_EVENT_WAIT_MAX_MS: u64 = 500;
const HOST_FOCUS_IGNORE_HINTS: &[&str] = &[
    "org.virt-manager.virt-manager",
    "virt-manager",
    "remote-viewer",
    "virt-viewer",
    "spicy",
    "org.gnome.boxes",
    "gnome-boxes",
    "virtualbox machine",
    "virtualboxvm",
    "qemu-system",
    "lay-kde-test spice",
    "lay-kde-spice-viewer-clipboard",
    "spice clipboard",
];
static FOCUS_INFO_UNAVAILABLE: AtomicBool = AtomicBool::new(false);

pub(super) fn update_focus_ignore_state(
    focus_ignored: &mut bool,
    last_poll: &mut Instant,
    buffer: &mut WordBuffer,
    events_since_word_start: &mut u32,
) {
    if active_layout_backend() != LayoutBackend::Gnome
        || last_poll.elapsed() < Duration::from_millis(FOCUS_IGNORE_POLL_INTERVAL_MS)
    {
        return;
    }
    *last_poll = Instant::now();

    let ignored = focused_window_should_be_ignored();
    if ignored != *focus_ignored {
        if ignored {
            log("► focused window ignored: VM/remote viewer, host lay paused");
        } else {
            log("► focused window accepted: host lay resumed");
        }
    }
    if ignored {
        buffer.reset_all();
        *events_since_word_start = 0;
    }
    *focus_ignored = ignored;
}

pub(super) fn wait_for_keyboard_event_or_timeout(
    fd: RawFd,
    timeout: Duration,
) -> std::io::Result<()> {
    let mut poll_fd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    let timeout_ms = timeout
        .max(Duration::from_millis(1))
        .as_millis()
        .min(i32::MAX as u128) as i32;

    let rc = unsafe { libc::poll(&mut poll_fd, 1, timeout_ms) };
    if rc < 0 {
        let err = std::io::Error::last_os_error();
        if err.kind() == std::io::ErrorKind::Interrupted {
            return Ok(());
        }
        return Err(err);
    }
    Ok(())
}

pub(super) fn idle_wait_timeout(
    pending_multi_tap: Option<&MultiTapPending>,
    last_focus_ignore_poll: Instant,
    shift_window: Duration,
) -> Duration {
    idle_wait_timeout_at(
        Instant::now(),
        pending_multi_tap,
        last_focus_ignore_poll,
        shift_window,
    )
}

pub(super) fn idle_wait_timeout_at(
    now: Instant,
    pending_multi_tap: Option<&MultiTapPending>,
    last_focus_ignore_poll: Instant,
    shift_window: Duration,
) -> Duration {
    let mut wait = Duration::from_millis(IDLE_EVENT_WAIT_MAX_MS);

    if let Some(pending) = pending_multi_tap {
        wait = wait.min(deadline_remaining(now, pending.last_release, shift_window));
    }
    wait.min(deadline_remaining(
        now,
        last_focus_ignore_poll,
        Duration::from_millis(FOCUS_IGNORE_POLL_INTERVAL_MS),
    ))
}

fn deadline_remaining(now: Instant, started_at: Instant, delay: Duration) -> Duration {
    started_at
        .checked_add(delay)
        .and_then(|deadline| deadline.checked_duration_since(now))
        .unwrap_or(Duration::ZERO)
}

pub(super) fn focused_window_should_be_ignored() -> bool {
    if FOCUS_INFO_UNAVAILABLE.load(Ordering::Relaxed) {
        return false;
    }
    match call_focused_window_info() {
        Ok(json) => focused_window_json_is_ignored(&json),
        Err(error) => {
            FOCUS_INFO_UNAVAILABLE.store(true, Ordering::Relaxed);
            log(&format!(
                "⚠ FocusedWindowInfo unavailable, host focus guard disabled: {error}"
            ));
            false
        }
    }
}

pub(super) fn focused_window_json_is_ignored(json: &str) -> bool {
    let haystack = focused_window_haystack(json);
    HOST_FOCUS_IGNORE_HINTS
        .iter()
        .any(|hint| haystack.contains(&hint.to_ascii_lowercase()))
}

pub(super) fn focused_window_haystack(json: &str) -> String {
    let value = match serde_json::from_str::<serde_json::Value>(json) {
        Ok(value) => value,
        Err(_) => return json.to_ascii_lowercase(),
    };
    let mut parts = Vec::new();
    if let Some(object) = value.as_object() {
        for key in [
            "appId",
            "wmClass",
            "wmClassInstance",
            "label",
            "title",
            "value",
        ] {
            if let Some(text) = object.get(key).and_then(|item| item.as_str()) {
                parts.push(text);
            }
        }
    }
    parts.join(" ").to_ascii_lowercase()
}
