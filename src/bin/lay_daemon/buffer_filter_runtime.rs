use evdev::KeyCode;

use super::pending_typing_assist::PendingTypingAssist;
use super::{
    is_hard_boundary, log, should_ignore_buffer_key, should_start_ignored_buffer_token, ShiftState,
};
use lay::keyboard::is_typing_key;

pub(super) struct BufferFilterContext<'a> {
    pub(super) key: KeyCode,
    pub(super) code: u16,
    pub(super) shift_state: &'a ShiftState,
    pub(super) current_empty: bool,
    pub(super) ignore_current_token_until_space: &'a mut bool,
    pub(super) events_since_word_start: &'a mut u32,
    pub(super) pending_typing_assist_after_space: &'a mut Option<PendingTypingAssist>,
    pub(super) verbose: bool,
}

pub(super) fn should_skip_buffer_input(ctx: BufferFilterContext<'_>) -> bool {
    let BufferFilterContext {
        key,
        code,
        shift_state,
        current_empty,
        ignore_current_token_until_space,
        events_since_word_start,
        pending_typing_assist_after_space,
        verbose,
    } = ctx;
    if shift_state.shortcut_active() && should_ignore_buffer_key(key, shift_state, current_empty) {
        log_ignored_key(code, verbose);
        return true;
    }

    if *ignore_current_token_until_space {
        if key == KeyCode::KEY_SPACE {
            *ignore_current_token_until_space = false;
            *events_since_word_start = 0;
            pending_typing_assist_after_space.take();
            return true;
        }
        if is_hard_boundary(key) {
            *ignore_current_token_until_space = false;
        } else if is_typing_key(key) {
            if verbose {
                log(&format!("· key {code} ignored inside non-word token"));
            }
            return true;
        }
    }

    if should_start_ignored_buffer_token(key, shift_state, current_empty) {
        *ignore_current_token_until_space = true;
        if verbose {
            log(&format!("· key {code} starts ignored non-word token"));
        }
        return true;
    }

    if should_ignore_buffer_key(key, shift_state, current_empty) {
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
