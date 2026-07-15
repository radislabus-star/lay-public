use evdev::uinput::VirtualDevice;

use super::super::super::{
    log, release_possible_modifiers, release_possible_modifiers_fast,
    settle_after_physical_trigger_release,
};

pub(super) fn prepare_uinput_output(kbd: &mut VirtualDevice, input_isolated: bool) {
    if input_isolated {
        log("  input isolated: skip trigger settle");
        if let Err(e) = release_possible_modifiers_fast(kbd) {
            log(&format!(
                "⚠ fast modifier cleanup before backspace failed: {e}"
            ));
        }
    } else {
        settle_after_physical_trigger_release();
        if let Err(e) = release_possible_modifiers(kbd) {
            log(&format!("⚠ modifier cleanup before backspace failed: {e}"));
        }
    }
}
