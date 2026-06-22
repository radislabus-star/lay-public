use evdev::uinput::VirtualDevice;
use lay::word_buffer::WordBuffer;

use super::super::super::super::physical_input_grab::PhysicalInputGrab;

pub(super) fn forward_after_ime_replace<'kbd, 'grab>(
    virtual_kbd: &mut Option<&'kbd mut VirtualDevice>,
    physical_grab: &mut PhysicalInputGrab<'grab>,
    buf: &mut WordBuffer,
    target_layout: bool,
    skip_spaces: usize,
) -> usize {
    let Some(kbd) = virtual_kbd.as_deref_mut() else {
        return 0;
    };
    physical_grab
        .forward_queued_typing(kbd, buf, target_layout, "typing-assist", skip_spaces)
        .spaces
}

pub(super) fn trailing_space_count(text: &str) -> usize {
    text.chars().rev().take_while(|ch| *ch == ' ').count()
}
