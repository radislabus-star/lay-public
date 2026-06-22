use lay::desktop::{is_ru_layout_id, LayoutBackend};
use std::time::Duration;

use super::{active_layout_backend, active_text_backend, layout_kde, layout_niri, layout_x11, log};

#[path = "layout_controller/backend_hint.rs"]
mod backend_hint;
#[path = "layout_controller/gnome_dbus.rs"]
mod gnome_dbus;
#[path = "layout_controller/ibus_bridge.rs"]
mod ibus_bridge;
#[path = "layout_controller/ime_bridge.rs"]
mod ime_bridge;

const LAYOUT_SWITCH_SETTLE_MS: u64 = 12;
const TRIGGER_RELEASE_SETTLE_MS: u64 = 80;
const LAYOUT_VERIFY_ATTEMPTS: usize = 5;
const LAYOUT_VERIFY_POLL_MS: u64 = 10;

pub(super) fn read_current_layout_is_ru() -> Result<bool, String> {
    match active_layout_backend() {
        LayoutBackend::Gnome => read_current_layout_gnome_is_ru(),
        LayoutBackend::Kde => layout_kde::read_current_layout_is_ru(),
        LayoutBackend::Niri => layout_niri::read_current_layout_is_ru(),
        LayoutBackend::X11 => layout_x11::read_current_layout_is_ru(),
    }
}

fn read_current_layout_gnome_is_ru() -> Result<bool, String> {
    if active_text_backend().should_try_ime() {
        read_current_ibus_layout_is_ru().or_else(|_| read_current_gnome_shell_layout_is_ru())
    } else {
        read_current_gnome_shell_layout_is_ru().or_else(|_| read_current_ibus_layout_is_ru())
    }
}

fn read_current_gnome_shell_layout_is_ru() -> Result<bool, String> {
    gnome_dbus::call_current_layout().map(|id| is_ru_layout_id(&id))
}

fn read_current_ibus_layout_is_ru() -> Result<bool, String> {
    ibus_bridge::read_current_layout_is_ru()
}

pub(super) fn call_ping() -> Result<String, String> {
    gnome_dbus::call_ping()
}

pub(super) fn call_focused_window_info() -> Result<String, String> {
    if active_layout_backend() != LayoutBackend::Gnome {
        return Err("FocusedWindowInfo is available only through the GNOME backend".to_string());
    }
    gnome_dbus::call_focused_window_info()
}

fn switch_to_layout(layout_id: &str, ibus_engine: &str, target_is_ru: bool) -> Result<(), String> {
    match active_layout_backend() {
        LayoutBackend::Gnome => switch_to_gnome_layout(layout_id, ibus_engine, target_is_ru),
        LayoutBackend::Kde => layout_kde::switch_to_layout(layout_id, target_is_ru),
        LayoutBackend::Niri => layout_niri::switch_to_layout(layout_id, target_is_ru),
        LayoutBackend::X11 => layout_x11::switch_to_layout(layout_id, target_is_ru),
    }
}

fn switch_to_gnome_layout(
    layout_id: &str,
    ibus_engine: &str,
    target_is_ru: bool,
) -> Result<(), String> {
    let activate_error = match gnome_dbus::call_activate_layout(layout_id) {
        Ok(true) => {
            if verify_gnome_shell_layout(target_is_ru) {
                None
            } else {
                Some("ActivateLayout returned true but layout verify failed".to_string())
            }
        }
        Ok(false) => Some("ActivateLayout returned false".to_string()),
        Err(error) => Some(error),
    };

    let ibus_error = ibus_bridge::ensure_engine(ibus_engine, target_is_ru).err();
    if verify_gnome_shell_layout(target_is_ru) {
        if let Some(error) = ibus_error {
            log(&format!(
                "⚠ SetGlobalEngine refresh failed, GNOME Shell layout verified: {error}"
            ));
        }
        return Ok(());
    }

    if verify_gnome_layout_stack(target_is_ru) {
        if let Some(error) = activate_error {
            log(&format!(
                "⚠ ActivateLayout failed, ibus layout verified: {error}"
            ));
        }
        return Ok(());
    }

    Err(match (activate_error, ibus_error) {
        (Some(activate), Some(ibus)) => {
            format!("ActivateLayout failed: {activate}; SetGlobalEngine failed: {ibus}; layout verify failed")
        }
        (Some(activate), None) => {
            format!("ActivateLayout failed: {activate}; layout verify failed")
        }
        (None, Some(ibus)) => format!("SetGlobalEngine failed: {ibus}; layout verify failed"),
        (None, None) => "layout verify failed".to_string(),
    })
}

pub(super) fn switch_to_target_layout(target_is_ru: bool) -> Result<&'static str, String> {
    let (layout_id, ibus_engine) = target_layout(target_is_ru);
    if active_layout_backend() != LayoutBackend::Gnome
        && read_current_layout_is_ru().is_ok_and(|current| current == target_is_ru)
    {
        return Ok(layout_id);
    }
    switch_to_layout(layout_id, ibus_engine, target_is_ru).map(|()| {
        settle_after_layout_switch();
        layout_id
    })
}

pub(super) fn sync_ime_engine_to_current_layout(current_is_ru: bool) {
    if !active_text_backend().should_try_ime() || active_layout_backend() != LayoutBackend::Gnome {
        return;
    }
    let (_, ibus_engine) = target_layout(current_is_ru);
    if let Err(error) = ibus_bridge::ensure_engine(ibus_engine, current_is_ru) {
        log(&format!(
            "⚠ IME engine sync failed for {ibus_engine}: {error}"
        ));
    }
}

pub(super) fn target_layout(target_is_ru: bool) -> (&'static str, &'static str) {
    if target_is_ru {
        (
            "ru",
            if active_text_backend().should_try_ime() {
                "lay-ime-ru"
            } else {
                "xkb:ru::rus"
            },
        )
    } else {
        (
            "us",
            if active_text_backend().should_try_ime() {
                "lay-ime-us"
            } else {
                "xkb:us::eng"
            },
        )
    }
}

pub(super) fn verify_current_layout(target_is_ru: bool) -> bool {
    verify_layout_with_retry(|| {
        read_current_layout_is_ru().is_ok_and(|current| current == target_is_ru)
    })
}

fn verify_gnome_shell_layout(target_is_ru: bool) -> bool {
    verify_layout_with_retry(|| {
        read_current_gnome_shell_layout_is_ru().is_ok_and(|current| current == target_is_ru)
    })
}

fn verify_gnome_layout_stack(target_is_ru: bool) -> bool {
    verify_layout_with_retry(|| verify_gnome_layout_stack_once(target_is_ru))
}

fn verify_gnome_layout_stack_once(target_is_ru: bool) -> bool {
    read_current_gnome_shell_layout_is_ru().is_ok_and(|current| current == target_is_ru)
        && read_current_ibus_layout_is_ru().is_ok_and(|current| current == target_is_ru)
}

fn verify_layout_with_retry(check: impl FnMut() -> bool) -> bool {
    verify_layout_with_retry_config(LAYOUT_VERIFY_ATTEMPTS, LAYOUT_VERIFY_POLL_MS, check)
}

fn verify_layout_with_retry_config(
    attempts: usize,
    poll_ms: u64,
    mut check: impl FnMut() -> bool,
) -> bool {
    for _ in 0..attempts {
        if check() {
            return true;
        }
        if poll_ms > 0 {
            std::thread::sleep(Duration::from_millis(poll_ms));
        }
    }
    false
}

fn settle_after_layout_switch() {
    std::thread::sleep(Duration::from_millis(LAYOUT_SWITCH_SETTLE_MS));
}

pub(super) fn settle_after_physical_trigger_release() {
    std::thread::sleep(Duration::from_millis(TRIGGER_RELEASE_SETTLE_MS));
}

pub(super) fn call_replace_text(
    move_left: u32,
    backspaces: u32,
    text: &str,
    move_right: u32,
    layout_id: &str,
) -> Result<bool, String> {
    if active_layout_backend() != LayoutBackend::Gnome {
        return Err("ReplaceText is available only through the GNOME backend".to_string());
    }
    gnome_dbus::call_replace_text(move_left, backspaces, text, move_right, layout_id)
}

pub(super) fn should_try_ime_text_backend() -> bool {
    ime_bridge::should_try_text_backend()
}

pub(super) fn try_ime_replace_tail(
    original: &str,
    replacement: &str,
    kind: &str,
) -> Result<bool, String> {
    ime_bridge::try_replace_tail(original, replacement, kind)
}

pub(super) fn call_ime_ping() -> Result<String, String> {
    ime_bridge::call_ping()
}

pub(super) fn focused_ime_engine_handles_typing() -> bool {
    if !active_text_backend().should_try_ime() {
        return false;
    }
    match ime_bridge::input_state() {
        Ok(state) => state.starts_with("active:"),
        Err(_) => ime_bridge::owns_active_text().unwrap_or(false),
    }
}

pub(super) fn suppress_next_ime_autocorrect() {
    if active_text_backend().should_try_ime() {
        let _ = ime_bridge::suppress_next_autocorrect();
    }
}

pub(super) fn try_ime_manual_toggle() -> Result<Option<bool>, String> {
    if !active_text_backend().should_try_ime() {
        return Ok(None);
    }
    let target_is_ru = match ime_bridge::manual_toggle_outcome() {
        Ok(outcome) => outcome.target_layout_is_ru(),
        Err(_) => {
            let handled = ime_bridge::manual_toggle()?;
            return Ok(handled.then_some(read_current_layout_is_ru().unwrap_or(false)));
        }
    };
    let Some(target_is_ru) = target_is_ru else {
        return Ok(None);
    };
    switch_to_target_layout(target_is_ru)?;
    Ok(Some(target_is_ru))
}

pub(super) fn detect_auto_layout_backend_hint() -> Option<LayoutBackend> {
    backend_hint::detect_auto_layout_backend_hint()
}

#[cfg(test)]
#[path = "layout_controller/tests.rs"]
mod tests;
