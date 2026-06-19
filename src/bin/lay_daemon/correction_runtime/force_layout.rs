use evdev::uinput::VirtualDevice;
use lay::word_buffer::WordBuffer;
use std::time::Instant;

use super::super::{
    log, release_possible_modifiers, settle_after_physical_trigger_release,
    switch_to_target_layout, ExecutingGuard,
};

pub(crate) fn handle_force_layout_hotkey(
    target_is_ru: bool,
    buf: &mut WordBuffer,
    virtual_kbd: Option<&mut VirtualDevice>,
    executing: &mut bool,
) -> Option<bool> {
    let started_at = Instant::now();
    settle_after_physical_trigger_release();
    *executing = true;
    let _executing_guard = ExecutingGuard(executing);
    if let Some(kbd) = virtual_kbd {
        if let Err(e) = release_possible_modifiers(kbd) {
            log(&format!("⚠ force-layout modifier cleanup failed: {e}"));
        }
    }
    match switch_to_target_layout(target_is_ru) {
        Ok(layout_id) => {
            buf.reset_all();
            log(&format!(
                "✓ force-layout → {layout_id} за {}ms",
                started_at.elapsed().as_millis()
            ));
            Some(target_is_ru)
        }
        Err(e) => {
            log(&format!("⚠ force-layout switch failed: {e}"));
            None
        }
    }
}
