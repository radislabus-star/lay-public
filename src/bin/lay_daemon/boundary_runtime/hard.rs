use evdev::KeyCode;
use lay::word_buffer::WordBuffer;

use super::super::pending_typing_assist::PendingTypingAssist;
use super::super::{append_user_correction_learning_log, is_hard_boundary, log};

pub(crate) struct HardBoundaryContext<'a> {
    pub(crate) buffer: &'a mut WordBuffer,
    pub(crate) pending_typing_assist_after_space: &'a mut Option<PendingTypingAssist>,
    pub(crate) ignore_current_token_until_space: &'a mut bool,
    pub(crate) events_since_word_start: &'a mut u32,
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
    ctx: HardBoundaryContext<'_>,
) -> bool {
    if !is_hard_boundary(key) {
        return false;
    }
    if value == 1 && !ctx.buffer.is_empty() {
        if !matches!(key, KeyCode::KEY_BACKSPACE | KeyCode::KEY_DELETE) {
            if let Some(correction) = ctx.buffer.take_user_learning_correction(false) {
                append_user_correction_learning_log(&correction);
            }
        }
        ctx.buffer.reset_all();
        ctx.pending_typing_assist_after_space.take();
        *ctx.ignore_current_token_until_space = false;
        *ctx.events_since_word_start = 0;
        if ctx.verbose {
            log(&format!("· reset (граница: {key:?})"));
        }
    }
    true
}
