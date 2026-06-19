use evdev::{uinput::VirtualDevice, Device};
use lay::word_buffer::WordBuffer;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::super::super::{pending_typing_assist::PendingTypingAssist, ShiftState};

pub(crate) struct DeferredTypingAssistContext<'a> {
    pub(crate) buffer: &'a mut WordBuffer,
    pub(crate) device: &'a mut Device,
    pub(crate) virtual_kbd: &'a Arc<Mutex<Option<VirtualDevice>>>,
    pub(crate) executing: &'a mut bool,
    pub(crate) current_layout_is_ru: &'a mut bool,
    pub(crate) last_layout_poll: &'a mut Instant,
    pub(crate) pending_typing_assist_after_space: &'a mut Option<PendingTypingAssist>,
    pub(crate) shift_state: &'a ShiftState,
}
