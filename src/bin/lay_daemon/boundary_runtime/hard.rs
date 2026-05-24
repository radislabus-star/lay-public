use evdev::{uinput::VirtualDevice, KeyCode};
use lay::word_buffer::WordBuffer;
use std::sync::{Arc, Mutex};

use super::super::{
    active_typing_assist, append_user_correction_learning_log, handle_typing_assist_after_space,
    is_hard_boundary, lock_virtual_keyboard, log, ShiftState,
};

pub(crate) struct HardBoundaryContext<'a> {
    pub(crate) buffer: &'a mut WordBuffer,
    pub(crate) virtual_kbd: &'a Arc<Mutex<Option<VirtualDevice>>>,
    pub(crate) executing: &'a mut bool,
    pub(crate) pending_typing_assist_after_space: &'a mut bool,
    pub(crate) ignore_current_token_until_space: &'a mut bool,
    pub(crate) events_since_word_start: &'a mut u32,
    pub(crate) shift_state: &'a ShiftState,
    pub(crate) verbose: bool,
}

pub(crate) fn note_learning_backspace_if_needed(key: KeyCode, value: i32, buffer: &mut WordBuffer) {
    if matches!(key, KeyCode::KEY_BACKSPACE | KeyCode::KEY_DELETE) && value == 1 {
        buffer.note_learning_backspace();
    }
}

pub(crate) fn handle_hard_boundary_if_needed(
    key: KeyCode,
    value: i32,
    mut ctx: HardBoundaryContext<'_>,
) -> bool {
    if !is_hard_boundary(key) {
        return false;
    }
    if value == 1 && !ctx.buffer.is_empty() {
        run_pending_typing_assist_before_boundary(&mut ctx);
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

fn run_pending_typing_assist_before_boundary(ctx: &mut HardBoundaryContext<'_>) {
    if *ctx.pending_typing_assist_after_space && active_typing_assist() && !ctx.shift_state.any() {
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
}
