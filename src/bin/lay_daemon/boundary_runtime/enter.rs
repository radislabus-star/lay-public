use evdev::{uinput::VirtualDevice, Device, KeyCode};
use lay::word_buffer::WordBuffer;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::super::pending_typing_assist::PendingTypingAssist;
use super::super::{
    active_enter_autocorrect, active_replace_words, grab_physical_device_for_correction,
    handle_enter_autocorrect, lock_virtual_keyboard, log,
};

pub(crate) struct EnterAutocorrectContext<'a> {
    pub(crate) buffer: &'a mut WordBuffer,
    pub(crate) device: &'a mut Device,
    pub(crate) virtual_kbd: &'a Arc<Mutex<Option<VirtualDevice>>>,
    pub(crate) executing: &'a mut bool,
    pub(crate) current_layout_is_ru: &'a mut bool,
    pub(crate) last_layout_poll: &'a mut Instant,
    pub(crate) pending_typing_assist_after_space: &'a mut Option<PendingTypingAssist>,
    pub(crate) ignore_current_token_until_space: &'a mut bool,
    pub(crate) events_since_word_start: &'a mut u32,
    pub(crate) clear_on_next_typing: &'a mut bool,
}

pub(crate) fn try_handle_enter_autocorrect(
    key: KeyCode,
    value: i32,
    ctx: EnterAutocorrectContext<'_>,
) -> bool {
    if key != KeyCode::KEY_ENTER
        || value != 1
        || !active_enter_autocorrect()
        || ctx.buffer.is_empty()
    {
        return false;
    }

    let _physical_grab = grab_physical_device_for_correction(ctx.device);
    let mut g = lock_virtual_keyboard(ctx.virtual_kbd);
    let correction_result = handle_enter_autocorrect(
        ctx.buffer,
        active_replace_words(),
        g.as_mut(),
        ctx.executing,
    );
    if let Some(is_ru) = correction_result {
        *ctx.current_layout_is_ru = is_ru;
        *ctx.last_layout_poll = Instant::now();
        ctx.buffer.reset_all();
        ctx.pending_typing_assist_after_space.take();
        *ctx.ignore_current_token_until_space = false;
        *ctx.events_since_word_start = 0;
        *ctx.clear_on_next_typing = true;
        log("· Enter autocorrect consumed boundary");
        return true;
    }
    false
}
