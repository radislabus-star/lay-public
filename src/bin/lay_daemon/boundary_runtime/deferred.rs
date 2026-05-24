use evdev::{uinput::VirtualDevice, Device};
use lay::word_buffer::WordBuffer;
use std::sync::{Arc, Mutex};

use super::super::{
    active_typing_assist, handle_typing_assist_after_space, lock_virtual_keyboard, log,
    should_run_deferred_typing_assist_after_space, typing_assist_cursor_offset_after_space,
    ShiftState, TypingAssistOutcome,
};

pub(crate) struct DeferredTypingAssistContext<'a> {
    pub(crate) buffer: &'a mut WordBuffer,
    pub(crate) device: &'a mut Device,
    pub(crate) virtual_kbd: &'a Arc<Mutex<Option<VirtualDevice>>>,
    pub(crate) executing: &'a mut bool,
    pub(crate) pending_typing_assist_after_space: &'a mut bool,
    pub(crate) shift_state: &'a ShiftState,
    pub(crate) verbose: bool,
}

pub(crate) fn try_handle_deferred_typing_assist(ctx: DeferredTypingAssistContext<'_>) -> bool {
    if !should_run_deferred_typing_assist_after_space(
        *ctx.pending_typing_assist_after_space,
        active_typing_assist(),
        ctx.shift_state.any(),
    ) {
        return false;
    }

    let cursor_offset = typing_assist_cursor_offset_after_space(ctx.buffer.current_len());
    if ctx.verbose && cursor_offset > 0 {
        log(&format!(
            "· typing-assist deferred idle run behind {cursor_offset} chars"
        ));
    }

    let mut g = lock_virtual_keyboard(ctx.virtual_kbd);
    let outcome = handle_typing_assist_after_space(
        ctx.buffer,
        g.as_mut(),
        Some(ctx.device),
        ctx.executing,
        cursor_offset,
    );
    *ctx.pending_typing_assist_after_space = matches!(outcome, TypingAssistOutcome::Deferred);
    true
}
