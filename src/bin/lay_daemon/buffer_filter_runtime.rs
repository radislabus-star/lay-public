use evdev::KeyCode;
use lay::word_buffer::WordBuffer;

use super::{
    log, pending_typing_assist::PendingTypingAssist, should_ignore_buffer_key, ShiftState,
};

pub(super) struct BufferFilterContext<'a> {
    pub(super) key: KeyCode,
    pub(super) code: u16,
    pub(super) shift_state: &'a ShiftState,
    pub(super) buffer: &'a mut WordBuffer,
    pub(super) pending_typing_assist_after_space: &'a mut Option<PendingTypingAssist>,
    pub(super) events_since_word_start: &'a mut u32,
    pub(super) clear_on_next_typing: &'a mut bool,
    pub(super) verbose: bool,
}

pub(super) fn should_skip_buffer_input(ctx: BufferFilterContext<'_>) -> bool {
    let BufferFilterContext {
        key,
        code,
        shift_state,
        buffer,
        pending_typing_assist_after_space,
        events_since_word_start,
        clear_on_next_typing,
        verbose,
    } = ctx;
    if should_ignore_buffer_key(key, shift_state) || shift_insert_active(key, shift_state) {
        buffer.clear_pending_learning();
        buffer.reset_all();
        pending_typing_assist_after_space.take();
        *events_since_word_start = 0;
        let _ = clear_on_next_typing;
        log_ignored_key(code, verbose);
        return true;
    }

    false
}

fn shift_insert_active(key: KeyCode, shift_state: &ShiftState) -> bool {
    key == KeyCode::KEY_INSERT && shift_state.any()
}

fn log_ignored_key(code: u16, verbose: bool) {
    if verbose {
        log(&format!("· key {code} ignored for buffer (shortcut/noise)"));
    }
}
