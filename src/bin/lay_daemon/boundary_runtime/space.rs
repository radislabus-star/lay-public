use evdev::{InputEvent, KeyCode};
use lay::word_buffer::WordBuffer;

use super::super::pending_typing_assist::PendingTypingAssist;
use super::super::{
    active_typing_assist, append_user_correction_learning_log, has_later_typing_press, log,
    prepare_typing_assist_after_space, should_run_typing_assist_on_space_release,
    should_schedule_typing_assist_after_space, ShiftState,
};

pub(crate) struct SpaceReleaseContext<'a> {
    pub(crate) events: &'a [InputEvent],
    pub(crate) event_idx: usize,
    pub(crate) buffer: &'a mut WordBuffer,
    pub(crate) pending_typing_assist_after_space: &'a mut Option<PendingTypingAssist>,
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
            ctx.pending_typing_assist_after_space.is_some(),
            active_typing_assist(),
            ctx.shift_state.any(),
            ctx.buffer.is_empty(),
        )
    {
        return false;
    }

    if let Some(pending) = ctx.pending_typing_assist_after_space.as_mut() {
        pending.note_separator_released();
    }

    if has_later_typing_press(ctx.events, ctx.event_idx) {
        if ctx.verbose {
            log("· typing-assist deferred: next key already queued");
        }
        return true;
    }

    if ctx.verbose {
        log("· typing-assist deferred: space release drained first");
    }
    true
}

pub(crate) struct SpacePressContext<'a> {
    pub(crate) buffer: &'a mut WordBuffer,
    pub(crate) pending_typing_assist_after_space: &'a mut Option<PendingTypingAssist>,
    pub(crate) events_since_word_start: &'a mut u32,
    pub(crate) suppress_next_typing_assist_after_manual_replay: &'a mut bool,
    pub(crate) verbose: bool,
}

pub(crate) fn handle_space_press(ctx: SpacePressContext<'_>) {
    if let Some(correction) = ctx.buffer.take_user_learning_correction(true) {
        append_user_correction_learning_log(&correction);
    }
    let already_pending = ctx.pending_typing_assist_after_space.is_some();
    ctx.buffer.handle_space();
    if let Some(pending) = ctx.pending_typing_assist_after_space.as_mut() {
        pending.note_visible_char();
    }
    *ctx.events_since_word_start = 0;
    if !already_pending
        && should_schedule_typing_assist_after_space(
            active_typing_assist(),
            ctx.suppress_next_typing_assist_after_manual_replay,
        )
    {
        *ctx.pending_typing_assist_after_space =
            prepare_typing_assist_after_space(ctx.buffer).map(PendingTypingAssist::new);
        if ctx.verbose {
            if ctx.pending_typing_assist_after_space.is_some() {
                log("· typing-assist scheduled after space");
            } else {
                log("· typing-assist checked after space: no correction");
            }
        }
    } else if already_pending && ctx.verbose {
        log("· typing-assist pending kept after extra space");
    }
    if ctx.verbose {
        log(&format!(
            "· space, history={:?}, current={:?}",
            ctx.buffer.prev_words_len(),
            ctx.buffer.current_len()
        ));
    }
}
