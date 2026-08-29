use evdev::uinput::VirtualDevice;
use lay::action_log::RecentActionGateTrace;

use super::super::super::physical_input_grab::PhysicalInputGrab;
use super::context::ManualOutputCommon;
use super::native::{
    try_gnome_native_replace_output, try_ime_replace_output, NativeReplaceAttempt,
    NativeReplaceOutput,
};

pub(super) fn try_native_output_stage<'a, 'grab>(
    common: &mut ManualOutputCommon<'_>,
    virtual_kbd: &mut Option<&'a mut VirtualDevice>,
    physical_grab: &mut Option<&'a mut PhysicalInputGrab<'grab>>,
    input_gate: Option<RecentActionGateTrace>,
) -> Option<Option<bool>> {
    if common.output_route.allows_ime_stage() {
        match try_ime_replace_output(common, input_gate.clone()) {
            NativeReplaceAttempt::NotSelected => {}
            NativeReplaceAttempt::Finished(output) => {
                forward_queued_after_native_output(
                    virtual_kbd,
                    physical_grab,
                    common,
                    &output,
                    "manual-ime",
                );
                return Some(output.result);
            }
        }
    }
    match try_gnome_native_replace_output(common, input_gate) {
        NativeReplaceAttempt::NotSelected => {}
        NativeReplaceAttempt::Finished(output) => {
            forward_queued_after_native_output(
                virtual_kbd,
                physical_grab,
                common,
                &output,
                "manual-gnome",
            );
            return Some(output.result);
        }
    }
    None
}

fn forward_queued_after_native_output<'a, 'grab>(
    virtual_kbd: &mut Option<&'a mut VirtualDevice>,
    physical_grab: &mut Option<&'a mut PhysicalInputGrab<'grab>>,
    common: &mut ManualOutputCommon<'_>,
    output: &NativeReplaceOutput,
    reason: &'static str,
) {
    if let (Some(kbd), Some(grab)) = (virtual_kbd.as_deref_mut(), physical_grab.as_deref_mut()) {
        grab.forward_queued_typing(
            kbd,
            common.buf,
            output.layout_is_ru,
            reason,
            output.trailing_spaces,
            false,
        );
    }
}
