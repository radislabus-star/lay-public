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
    let selected = select_native_output_stage_with(
        common,
        input_gate,
        try_ime_replace_output,
        try_gnome_native_replace_output,
    );
    let (output, reason) = selected?;
    forward_queued_after_native_output(virtual_kbd, physical_grab, common, &output, reason);
    Some(output.result)
}

pub(super) fn select_native_output_stage_with(
    common: &mut ManualOutputCommon<'_>,
    input_gate: Option<RecentActionGateTrace>,
    try_ime: impl FnOnce(
        &mut ManualOutputCommon<'_>,
        Option<RecentActionGateTrace>,
    ) -> NativeReplaceAttempt,
    try_gnome: impl FnOnce(
        &mut ManualOutputCommon<'_>,
        Option<RecentActionGateTrace>,
    ) -> NativeReplaceAttempt,
) -> Option<(NativeReplaceOutput, &'static str)> {
    if common.output_route.allows_ime_stage() {
        match try_ime(common, input_gate.clone()) {
            NativeReplaceAttempt::NotSelected => {}
            NativeReplaceAttempt::Finished(output) => return Some((output, "manual-ime")),
        }
    }
    match try_gnome(common, input_gate) {
        NativeReplaceAttempt::NotSelected => None,
        NativeReplaceAttempt::Finished(output) => Some((output, "manual-gnome")),
    }
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
