use evdev::KeyCode;

use super::{log, should_ignore_buffer_key, ShiftState};

pub(super) struct BufferFilterContext<'a> {
    pub(super) key: KeyCode,
    pub(super) code: u16,
    pub(super) shift_state: &'a ShiftState,
    pub(super) verbose: bool,
}

pub(super) fn should_skip_buffer_input(ctx: BufferFilterContext<'_>) -> bool {
    let BufferFilterContext {
        key,
        code,
        shift_state,
        verbose,
    } = ctx;
    if should_ignore_buffer_key(key, shift_state) {
        log_ignored_key(code, verbose);
        return true;
    }

    false
}

fn log_ignored_key(code: u16, verbose: bool) {
    if verbose {
        log(&format!("· key {code} ignored for buffer (shortcut/noise)"));
    }
}
