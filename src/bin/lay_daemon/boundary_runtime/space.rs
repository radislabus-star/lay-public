use evdev::{uinput::VirtualDevice, Device, InputEvent, KeyCode};
use lay::word_buffer::WordBuffer;
use std::sync::{Arc, Mutex};

use super::super::{
    active_typing_assist, append_user_correction_learning_log, handle_typing_assist_after_space,
    has_later_typing_press, lock_virtual_keyboard, log, should_run_typing_assist_on_space_release,
    should_schedule_typing_assist_after_space, ShiftState, TypingAssistOutcome,
};

pub(crate) struct SpaceReleaseContext<'a> {
    pub(crate) events: &'a [InputEvent],
    pub(crate) event_idx: usize,
    pub(crate) buffer: &'a mut WordBuffer,
    pub(crate) device: &'a mut Device,
    pub(crate) virtual_kbd: &'a Arc<Mutex<Option<VirtualDevice>>>,
    pub(crate) executing: &'a mut bool,
    pub(crate) pending_typing_assist_after_space: &'a mut bool,
    pub(crate) shift_state: &'a ShiftState,
    pub(crate) verbose: bool,
}

pub(crate) fn try_handle_space_release(
    key: KeyCode,
    value: i32,
    ctx: SpaceReleaseContext<'_>,
) -> bool {
    if key != KeyCode::KEY_SPACE
        || value != 0
        || !should_run_typing_assist_on_space_release(
            *ctx.pending_typing_assist_after_space,
            active_typing_assist(),
            ctx.shift_state.any(),
            ctx.buffer.is_empty(),
        )
    {
        return false;
    }

    if has_later_typing_press(ctx.events, ctx.event_idx) {
        if ctx.verbose {
            log("· typing-assist deferred: next key already queued");
        }
        return true;
    }

    let mut g = lock_virtual_keyboard(ctx.virtual_kbd);
    let outcome = handle_typing_assist_after_space(
        ctx.buffer,
        g.as_mut(),
        Some(ctx.device),
        ctx.executing,
        0,
    );
    *ctx.pending_typing_assist_after_space = matches!(outcome, TypingAssistOutcome::Deferred);
    true
}

pub(crate) struct SpacePressContext<'a> {
    pub(crate) buffer: &'a mut WordBuffer,
    pub(crate) pending_typing_assist_after_space: &'a mut bool,
    pub(crate) events_since_word_start: &'a mut u32,
    pub(crate) suppress_next_typing_assist_after_manual_replay: &'a mut bool,
    pub(crate) verbose: bool,
}

pub(crate) fn handle_space_press(ctx: SpacePressContext<'_>) {
    if let Some(correction) = ctx.buffer.take_user_learning_correction(true) {
        append_user_correction_learning_log(&correction);
    }
    ctx.buffer.handle_space();
    *ctx.events_since_word_start = 0;
    if should_schedule_typing_assist_after_space(
        active_typing_assist(),
        ctx.suppress_next_typing_assist_after_manual_replay,
    ) {
        *ctx.pending_typing_assist_after_space = true;
        if ctx.verbose {
            log("· typing-assist scheduled after space");
        }
    }
    if ctx.verbose {
        log(&format!(
            "· space, history={:?}, current={:?}",
            ctx.buffer.prev_words_len(),
            ctx.buffer.current_len()
        ));
    }
}
