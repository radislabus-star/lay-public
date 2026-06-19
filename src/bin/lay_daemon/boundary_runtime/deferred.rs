use evdev::{uinput::VirtualDevice, Device};
use lay::word_buffer::WordBuffer;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::super::pending_typing_assist::PendingTypingAssist;
use super::super::{
    active_typing_assist, apply_prepared_typing_assist_after_space,
    focused_ime_engine_handles_typing, lock_virtual_keyboard, log,
    should_run_deferred_typing_assist_after_space, ShiftState, TypingAssistOutcome,
};

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

pub(crate) fn try_handle_deferred_typing_assist(ctx: DeferredTypingAssistContext<'_>) -> bool {
    if ctx.pending_typing_assist_after_space.is_some() && focused_ime_engine_handles_typing() {
        ctx.pending_typing_assist_after_space.take();
        log("· typing-assist deferred dropped: focused IME engine owns active text");
        return false;
    }
    if !should_run_deferred_typing_assist_after_space(
        ctx.pending_typing_assist_after_space.is_some(),
        active_typing_assist(),
        ctx.shift_state.any(),
    ) {
        return false;
    }

    let Some(pending) = ctx.pending_typing_assist_after_space.as_ref() else {
        return false;
    };
    if !pending.ready_to_apply() {
        return false;
    }
    let Some(pending) = ctx.pending_typing_assist_after_space.take() else {
        return false;
    };
    let (correction, cursor_offset) = pending.into_parts();
    let retry_correction = correction.clone();

    let mut g = lock_virtual_keyboard(ctx.virtual_kbd);
    let outcome = apply_prepared_typing_assist_after_space(
        ctx.buffer,
        g.as_mut(),
        Some(ctx.device),
        ctx.executing,
        cursor_offset,
        correction,
    );
    match outcome {
        TypingAssistOutcome::Applied { layout_is_ru } => {
            *ctx.current_layout_is_ru = layout_is_ru;
            *ctx.last_layout_poll = Instant::now();
        }
        TypingAssistOutcome::Deferred => {
            *ctx.pending_typing_assist_after_space = Some(PendingTypingAssist::with_cursor_offset(
                retry_correction,
                cursor_offset,
            ));
            log("· typing-assist deferred again: prepared edit still complex");
        }
        TypingAssistOutcome::NoCorrection => {}
    }
    true
}
