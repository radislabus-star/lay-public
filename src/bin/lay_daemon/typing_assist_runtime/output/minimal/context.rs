use evdev::uinput::VirtualDevice;
use lay::decoder::DecoderEditPlan;
use lay::keyboard::KeyEvent;
use lay::word_buffer::WordBuffer;

use super::super::super::super::physical_input_grab::PhysicalInputGrab;
use super::super::memory::TypingAssistTiming;

pub(crate) struct MinimalTypingReplacementContext<'a, 'grab> {
    pub(crate) buf: &'a mut WordBuffer,
    pub(crate) events: &'a [KeyEvent],
    pub(crate) edit: &'a DecoderEditPlan,
    pub(crate) original: &'a str,
    pub(crate) replacement: &'a str,
    pub(crate) rule_id: Option<&'a str>,
    pub(crate) cursor_offset: u32,
    pub(crate) timing: TypingAssistTiming,
    pub(crate) physical_grab: &'a mut PhysicalInputGrab<'grab>,
    pub(crate) kbd: &'a mut VirtualDevice,
    pub(crate) original_layout: Option<bool>,
    pub(crate) prefer_full_token_plan: bool,
}
