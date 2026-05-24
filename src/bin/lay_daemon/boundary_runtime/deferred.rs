use evdev::{uinput::VirtualDevice, Device};
use lay::word_buffer::WordBuffer;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::super::pending_typing_assist::PendingTypingAssist;
use super::super::{
    active_typing_assist, apply_prepared_typing_assist_after_space, lock_virtual_keyboard, log,
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
    pub(crate) verbose: bool,
}

pub(crate) fn try_handle_deferred_typing_assist(ctx: DeferredTypingAssistContext<'_>) -> bool {
    if !should_run_deferred_typing_assist_after_space(
        ctx.pending_typing_assist_after_space.is_some(),
        active_typing_assist(),
        ctx.shift_state.any(),
    ) {
        return false;
    }

    if ctx
        .pending_typing_assist_after_space
        .as_ref()
        .is_some_and(|pending| !pending.ready_to_apply())
    {
        return false;
    }

    let Some(pending) = ctx.pending_typing_assist_after_space.take() else {
        return false;
    };
    let (correction, cursor_offset) = pending.into_parts();
    let retry_correction = correction.clone();
    if ctx.verbose && cursor_offset > 0 {
        log(&format!(
            "· typing-assist deferred idle run behind {cursor_offset} chars"
        ));
    }

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
