use crate::trigger_fsm::MultiTapPending;
use lay::desktop::LayoutBackend;
use std::os::fd::RawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use super::{active_layout_backend, call_focused_window_info, call_focused_window_info_once, log};

pub(super) const FOCUS_IGNORE_POLL_INTERVAL_MS: u64 = 500;
pub(super) const FOCUS_KEY_EVENT_POLL_INTERVAL_MS: u64 = 50;
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

pub(super) struct FocusedWindowState {
    pub(super) ignored: bool,
    pub(super) identity: Option<String>,
}

pub(super) fn poll_focused_window_state(last_poll: &mut Instant) -> Option<FocusedWindowState> {
    poll_focused_window_state_after(
        last_poll,
        Duration::from_millis(FOCUS_IGNORE_POLL_INTERVAL_MS),
    )
}

pub(super) fn poll_focused_window_state_for_key_event(
    last_poll: &mut Instant,
) -> Option<FocusedWindowState> {
    poll_focused_window_state_after(
        last_poll,
        Duration::from_millis(FOCUS_KEY_EVENT_POLL_INTERVAL_MS),
    )
}

fn poll_focused_window_state_after(
    last_poll: &mut Instant,
    min_interval: Duration,
) -> Option<FocusedWindowState> {
    if last_poll.elapsed() < min_interval {
        return None;
    }
    *last_poll = Instant::now();
    if active_layout_backend() != LayoutBackend::Gnome {
        return None;
    }
    focused_window_state()
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

fn focused_window_state() -> Option<FocusedWindowState> {
    if FOCUS_INFO_UNAVAILABLE.load(Ordering::Relaxed) {
        return None;
    }
    match call_focused_window_info() {
        Ok(json) => Some(FocusedWindowState {
            ignored: focused_window_json_is_ignored(&json),
            identity: focused_window_identity_from_json(&json),
        }),
        Err(error) => {
            FOCUS_INFO_UNAVAILABLE.store(true, Ordering::Relaxed);
            log(&format!(
                "⚠ FocusedWindowInfo unavailable, host focus guard disabled: {error}"
            ));
            None
        }
    }
}

pub(super) fn focused_window_identity_from_json(json: &str) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(json).ok()?;
    let object = value.as_object()?;
    let title = json_text_field(object, "title");

    for key in ["stableSequence", "windowId"] {
        if let Some(text) = object.get(key).and_then(|item| item.as_str()) {
            let text = text.trim();
            if !text.is_empty() {
                let base = format!("gnome-window:{key}:{text}");
                return Some(match (is_tabbable_app(object), title.as_deref()) {
                    (true, Some(title)) => {
                        format!("{base}:tab-title:{}", title.to_ascii_lowercase())
                    }
                    _ => base,
                });
            }
        }
    }

    let mut parts = Vec::new();
    for key in [
        "kind",
        "value",
        "appId",
        "wmClass",
        "wmClassInstance",
        "title",
    ] {
        if let Some(text) = object.get(key).and_then(|item| item.as_str()) {
            let text = text.trim();
            if !text.is_empty() {
                parts.push(text.to_ascii_lowercase());
            }
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(format!("gnome-window:fallback:{}", parts.join("\u{1f}")))
    }
}

pub(super) fn capture_exact_focused_window_identity() -> Result<String, String> {
    let json = call_focused_window_info_once()
        .map_err(|error| format!("exact focused-window read failed: {error}"))?;
    focused_window_identity_from_json(&json)
        .ok_or_else(|| "exact focused-window identity is unavailable".to_string())
}

pub(super) fn verify_exact_focused_window_identity(expected: &str) -> Result<(), String> {
    let observed = capture_exact_focused_window_identity()?;
    if observed == expected {
        Ok(())
    } else {
        Err(format!(
            "focused window changed: expected={expected:?} observed={observed:?}"
        ))
    }
}

fn json_text_field(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    object
        .get(key)
        .and_then(|item| item.as_str())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

fn is_tabbable_app(object: &serde_json::Map<String, serde_json::Value>) -> bool {
    let mut haystack = String::new();
    for key in ["appId", "wmClass", "wmClassInstance"] {
        if let Some(text) = object.get(key).and_then(|item| item.as_str()) {
            haystack.push_str(&text.to_ascii_lowercase());
            haystack.push(' ');
        }
    }
    [
        "chrome",
        "chromium",
        "firefox",
        "brave",
        "vivaldi",
        "opera",
        "microsoft-edge",
        "msedge",
    ]
    .iter()
    .any(|needle| haystack.contains(needle))
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
