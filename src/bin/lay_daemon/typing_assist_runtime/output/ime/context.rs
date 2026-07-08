use evdev::uinput::VirtualDevice;
use lay::decoder::DecoderEditPlan;
use lay::keyboard::KeyEvent;
use lay::word_buffer::WordBuffer;

use super::super::super::super::physical_input_grab::PhysicalInputGrab;
use super::super::memory::TypingAssistTiming;

pub(crate) struct ImeTypingReplacementContext<'a, 'kbd, 'grab> {
    pub(crate) buf: &'a mut WordBuffer,
    pub(crate) virtual_kbd: &'a mut Option<&'kbd mut VirtualDevice>,
    pub(crate) physical_grab: &'a mut PhysicalInputGrab<'grab>,
    pub(crate) events: &'a [KeyEvent],
    pub(crate) edit: &'a DecoderEditPlan,
    pub(crate) original: &'a str,
    pub(crate) replacement: &'a str,
    pub(crate) rule_id: Option<&'a str>,
    pub(crate) input_gate: Option<lay::action_log::RecentActionGateTrace>,
    pub(crate) timing: TypingAssistTiming,
}
