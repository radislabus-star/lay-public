use evdev::KeyCode;

use super::{
    is_hard_boundary, log, should_ignore_buffer_key, should_start_ignored_buffer_token, ShiftState,
};
use lay::keyboard::is_typing_key;

#[allow(clippy::too_many_arguments)]
pub(super) fn should_skip_buffer_input(
    key: KeyCode,
    code: u16,
    shift_state: &ShiftState,
    current_empty: bool,
    ignore_current_token_until_space: &mut bool,
    events_since_word_start: &mut u32,
    pending_typing_assist_after_space: &mut bool,
    verbose: bool,
) -> bool {
    if shift_state.shortcut_active() && should_ignore_buffer_key(key, shift_state, current_empty) {
        log_ignored_key(code, verbose);
        return true;
    }

    if *ignore_current_token_until_space {
        if key == KeyCode::KEY_SPACE {
            *ignore_current_token_until_space = false;
            *events_since_word_start = 0;
            *pending_typing_assist_after_space = false;
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
