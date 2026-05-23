use evdev::{uinput::VirtualDevice, Device, InputEvent, KeyCode};
use lay::word_buffer::WordBuffer;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::{
    active_enter_autocorrect, active_replace_words, active_typing_assist,
    append_user_correction_learning_log, grab_physical_device_for_correction,
    handle_enter_autocorrect, handle_typing_assist_after_space, has_later_typing_press,
    is_hard_boundary, lock_virtual_keyboard, log, should_drop_stale_typing_assist_after_space,
    should_run_typing_assist_on_space_release, should_schedule_typing_assist_after_space,
    ShiftState, TypingAssistOutcome,
};

pub(super) struct SpaceReleaseContext<'a> {
    pub(super) events: &'a [InputEvent],
    pub(super) event_idx: usize,
    pub(super) buffer: &'a mut WordBuffer,
    pub(super) device: &'a mut Device,
    pub(super) virtual_kbd: &'a Arc<Mutex<Option<VirtualDevice>>>,
    pub(super) executing: &'a mut bool,
    pub(super) pending_typing_assist_after_space: &'a mut bool,
    pub(super) shift_state: &'a ShiftState,
    pub(super) verbose: bool,
}

pub(super) fn try_handle_space_release(
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

pub(super) struct SpacePressContext<'a> {
    pub(super) buffer: &'a mut WordBuffer,
    pub(super) pending_typing_assist_after_space: &'a mut bool,
    pub(super) events_since_word_start: &'a mut u32,
    pub(super) suppress_next_typing_assist_after_manual_replay: &'a mut bool,
    pub(super) verbose: bool,
}

pub(super) fn handle_space_press(ctx: SpacePressContext<'_>) {
    if should_drop_stale_typing_assist_after_space(
        *ctx.pending_typing_assist_after_space,
        ctx.buffer.current_len(),
    ) {
        *ctx.pending_typing_assist_after_space = false;
        if ctx.verbose {
            log("· typing-assist stale previous word skipped behind current word");
        }
    }
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

pub(super) struct EnterAutocorrectContext<'a> {
    pub(super) buffer: &'a mut WordBuffer,
    pub(super) device: &'a mut Device,
    pub(super) virtual_kbd: &'a Arc<Mutex<Option<VirtualDevice>>>,
    pub(super) executing: &'a mut bool,
    pub(super) current_layout_is_ru: &'a mut bool,
    pub(super) last_layout_poll: &'a mut Instant,
    pub(super) pending_typing_assist_after_space: &'a mut bool,
    pub(super) ignore_current_token_until_space: &'a mut bool,
    pub(super) events_since_word_start: &'a mut u32,
    pub(super) clear_on_next_typing: &'a mut bool,
}

pub(super) fn try_handle_enter_autocorrect(
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
        *ctx.pending_typing_assist_after_space = false;
        *ctx.ignore_current_token_until_space = false;
        *ctx.events_since_word_start = 0;
        *ctx.clear_on_next_typing = true;
        log("· Enter autocorrect consumed boundary");
        return true;
    }
    false
}

pub(super) struct HardBoundaryContext<'a> {
    pub(super) buffer: &'a mut WordBuffer,
    pub(super) virtual_kbd: &'a Arc<Mutex<Option<VirtualDevice>>>,
    pub(super) executing: &'a mut bool,
    pub(super) pending_typing_assist_after_space: &'a mut bool,
    pub(super) ignore_current_token_until_space: &'a mut bool,
    pub(super) events_since_word_start: &'a mut u32,
    pub(super) shift_state: &'a ShiftState,
    pub(super) verbose: bool,
}

pub(super) fn note_learning_backspace_if_needed(key: KeyCode, value: i32, buffer: &mut WordBuffer) {
    if matches!(key, KeyCode::KEY_BACKSPACE | KeyCode::KEY_DELETE) && value == 1 {
        buffer.note_learning_backspace();
    }
}

pub(super) fn handle_hard_boundary_if_needed(
    key: KeyCode,
    value: i32,
    ctx: HardBoundaryContext<'_>,
) -> bool {
    if !is_hard_boundary(key) {
        return false;
    }
    if value == 1 && !ctx.buffer.is_empty() {
        if *ctx.pending_typing_assist_after_space
            && active_typing_assist()
            && !ctx.shift_state.any()
        {
            let cursor_offset = ctx.buffer.current_len() as u32;
            let mut g = lock_virtual_keyboard(ctx.virtual_kbd);
            let _ = handle_typing_assist_after_space(
                ctx.buffer,
                g.as_mut(),
                None,
                ctx.executing,
                cursor_offset,
            );
        }
        if !matches!(key, KeyCode::KEY_BACKSPACE | KeyCode::KEY_DELETE) {
            if let Some(correction) = ctx.buffer.take_user_learning_correction(false) {
                append_user_correction_learning_log(&correction);
            }
        }
        ctx.buffer.reset_all();
        *ctx.pending_typing_assist_after_space = false;
        *ctx.ignore_current_token_until_space = false;
        *ctx.events_since_word_start = 0;
        if ctx.verbose {
            log(&format!("· reset (граница: {key:?})"));
        }
    }
    true
}
