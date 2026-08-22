use lay::desktop::{is_ru_layout_id, LayoutBackend};
use lay::text_backend::ImeReplaceRequest;
use lay::text_edit::{AuthorizedEdit, BackendDispatchReceipt, TextEditBackend, VisibleTailSource};
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
#[path = "layout_controller/ime_manual_toggle.rs"]
mod ime_manual_toggle;
#[path = "layout_controller/reconcile.rs"]
mod reconcile;
#[path = "layout_controller/verify.rs"]
mod verify;

const LAYOUT_SWITCH_SETTLE_MS: u64 = 12;
const TRIGGER_RELEASE_SETTLE_MS: u64 = 80;

pub(super) use verify::verify_current_layout;

pub(super) fn read_current_layout_is_ru() -> Result<bool, String> {
    match active_layout_backend() {
        LayoutBackend::Gnome => read_current_layout_gnome_is_ru(),
        LayoutBackend::Kde => layout_kde::read_current_layout_is_ru(),
        LayoutBackend::Niri => layout_niri::read_current_layout_is_ru(),
        LayoutBackend::X11 => layout_x11::read_current_layout_is_ru(),
    }
}

fn read_current_layout_gnome_is_ru() -> Result<bool, String> {
    read_current_gnome_shell_layout_is_ru().or_else(|_| read_current_ibus_layout_is_ru())
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
        Ok(true) => None,
        Ok(false) => Some("ActivateLayout returned false".to_string()),
        Err(error) => Some(error),
    };

    if activate_error.is_none() {
        reconcile_ime_engine_after_gnome_switch(target_is_ru, ibus_engine);
        return Ok(());
    }

    let ibus_error = ibus_bridge::ensure_engine(ibus_engine, target_is_ru).err();
    if verify::verify_gnome_shell_layout(target_is_ru) {
        if let Some(error) = ibus_error {
            log(&format!(
                "⚠ SetGlobalEngine refresh failed, GNOME Shell layout verified: {error}"
            ));
        }
        return Ok(());
    }

    if verify::verify_gnome_layout_stack(target_is_ru) {
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

fn reconcile_ime_engine_after_gnome_switch(target_is_ru: bool, ibus_engine: &str) {
    reconcile::submit(target_is_ru, ibus_engine);
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

fn settle_after_layout_switch() {
    std::thread::sleep(Duration::from_millis(LAYOUT_SWITCH_SETTLE_MS));
}

pub(super) fn settle_after_physical_trigger_release() {
    std::thread::sleep(Duration::from_millis(TRIGGER_RELEASE_SETTLE_MS));
}

pub(super) fn call_replace_text(
    authorized: AuthorizedEdit,
    layout_id: &str,
) -> Result<bool, String> {
    if authorized.backend() != TextEditBackend::Daemon {
        return Err(format!(
            "GNOME ReplaceText requires a daemon AuthorizedEdit, got {}",
            authorized.backend().as_str()
        ));
    }
    if active_layout_backend() != LayoutBackend::Gnome {
        return Err("ReplaceText is available only through the GNOME backend".to_string());
    }
    let action = authorized.action();
    let plan = action
        .plan()
        .ok_or_else(|| "authorized edit has no replacement plan".to_string())?;
    gnome_dbus::call_replace_text(
        plan.move_left,
        plan.backspaces,
        &plan.insert,
        plan.move_right,
        layout_id,
    )
}

pub(super) fn should_try_ime_text_backend() -> bool {
    ime_bridge::should_try_text_backend()
}

pub(super) fn try_ime_replace_tail(
    authorized: AuthorizedEdit,
    kind: &str,
) -> BackendDispatchReceipt {
    let backend = TextEditBackend::Ime;
    if authorized.backend() != TextEditBackend::Ime {
        return BackendDispatchReceipt::rejected(backend, "authorized_backend_mismatch");
    }
    match ime_bridge::input_state() {
        Ok(state) if VisibleTailSource::from_bridge_state(&state).is_some() => {}
        Ok(_) => {
            return BackendDispatchReceipt::not_dispatched(backend, "no_focused_ime");
        }
        Err(error) => {
            log(&format!(
                "⚠ IME preflight unavailable before dispatch: {error}; daemon backend may be selected"
            ));
            return BackendDispatchReceipt::not_dispatched(backend, "ime_preflight_unavailable");
        }
    }
    let action = authorized.action();
    let request = ImeReplaceRequest::committed_tail(action.from_text(), action.to_text());
    match ime_bridge::can_replace_committed_tail(request.backspaces) {
        Ok(true) => {}
        Ok(false) => {
            log("  IME committed-tail capability unavailable; edit was not dispatched");
            return BackendDispatchReceipt::not_dispatched(
                backend,
                "ime_replace_capability_unavailable",
            );
        }
        Err(error) => {
            log(&format!(
                "⚠ IME capability preflight unavailable before dispatch: {error}; daemon backend may be selected"
            ));
            return BackendDispatchReceipt::not_dispatched(
                backend,
                "ime_capability_preflight_unavailable",
            );
        }
    }
    match ime_bridge::try_replace_tail(action.from_text(), action.to_text(), kind) {
        Ok(true) => BackendDispatchReceipt::dispatched(backend),
        Ok(false) => BackendDispatchReceipt::rejected(backend, "ime_visible_state_rejected"),
        Err(error) => BackendDispatchReceipt::indeterminate(backend, error),
    }
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

pub(super) fn try_ime_manual_toggle() -> Result<lay::manual_toggle::ImeManualToggleOutcome, String>
{
    ime_manual_toggle::try_manual_toggle(active_text_backend().should_try_ime())
}

pub(super) fn detect_auto_layout_backend_hint() -> Option<LayoutBackend> {
    backend_hint::detect_auto_layout_backend_hint()
}

#[cfg(test)]
#[path = "layout_controller/tests.rs"]
mod tests;
