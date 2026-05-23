use lay::desktop::{normalize_layout_id, parse_setxkbmap_layout};

use super::layout_controller::{command_exists, run_command_capture, verify_current_layout};
use super::log;

pub(super) fn read_current_layout_is_ru() -> Result<bool, String> {
    let layout = read_layout()?;
    Ok(lay::desktop::is_ru_layout_id(&layout))
}

pub(super) fn switch_to_layout(layout_id: &str, target_is_ru: bool) -> Result<(), String> {
    if let Err(native_error) = lay::x11_layout::lock_layout_id(layout_id) {
        log(&format!(
            "⚠ native X11 XKB layout switch failed: {native_error}; fallback shell tools"
        ));
    } else if verify_current_layout(target_is_ru) {
        return Ok(());
    } else {
        log("⚠ native X11 XKB layout verify failed; fallback shell tools");
    }

    if command_exists("xkb-switch") {
        run_command_capture("xkb-switch", &["-s", layout_id])?;
    } else {
        run_command_capture("setxkbmap", &[layout_id])?;
    }

    if verify_current_layout(target_is_ru) {
        Ok(())
    } else {
        Err("X11 layout verify failed".to_string())
    }
}

fn read_layout() -> Result<String, String> {
    if let Ok(layout) = lay::x11_layout::current_layout_id() {
        return Ok(layout);
    }

    if command_exists("xkb-switch") {
        return run_command_capture("xkb-switch", &[]).map(|layout| normalize_layout_id(&layout));
    }
    if command_exists("xkblayout-state") {
        return run_command_capture("xkblayout-state", &["print", "%s"])
            .map(|layout| normalize_layout_id(&layout));
    }

    let query = run_command_capture("setxkbmap", &["-query"])?;
    parse_setxkbmap_layout(&query).ok_or_else(|| format!("cannot parse setxkbmap output: {query}"))
}
