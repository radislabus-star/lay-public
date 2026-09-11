use evdev::KeyCode;
use lay::word_buffer::WordBuffer;

use super::super::pending_typing_assist::PendingTypingAssist;
use super::super::{append_user_correction_learning_log, is_hard_boundary, log, ShiftState};

pub(crate) struct HardBoundaryContext<'a> {
    pub(crate) buffer: &'a mut WordBuffer,
    pub(crate) pending_typing_assist_after_space: &'a mut Option<PendingTypingAssist>,
    pub(crate) events_since_word_start: &'a mut u32,
    pub(crate) shift_state: &'a ShiftState,
    pub(crate) clear_on_next_typing: &'a mut bool,
    pub(crate) verbose: bool,
}

pub(crate) fn handle_hard_boundary_if_needed(
    key: KeyCode,
    value: i32,
    ctx: HardBoundaryContext<'_>,
) -> bool {
    if !is_hard_boundary(key) {
        return false;
    }
    if value == 0 {
        return true;
    }

    ctx.pending_typing_assist_after_space.take();
    ctx.buffer.invalidate_replay_and_auto_undo();

    if key == KeyCode::KEY_BACKSPACE {
        if plain_backspace_no_modifiers(value, ctx.shift_state) {
            ctx.buffer.note_learning_backspace();
            if !*ctx.clear_on_next_typing
                && ctx.buffer.pop_known_current_event_preserving_nonempty()
            {
                *ctx.events_since_word_start = ctx.buffer.current_len() as u32;
                if ctx.verbose {
                    log(&format!(
                        "· backspace pop, current={} events={}",
                        ctx.buffer.current_len(),
                        *ctx.events_since_word_start
                    ));
                }
                return true;
            }
        } else {
            ctx.buffer.clear_pending_learning();
        }
    } else if key == KeyCode::KEY_DELETE {
        ctx.buffer.clear_pending_learning();
    } else if value == 1 {
        if let Some(correction) = ctx.buffer.take_user_learning_correction(false) {
            append_user_correction_learning_log(&correction);
        }
    }

    ctx.buffer.reset_all();
    *ctx.events_since_word_start = 0;
    if ctx.verbose {
        log(&format!("· reset (граница: {key:?})"));
    }
    true
}

pub(crate) fn cancel_backspace_pending_assist_before_deferred_poll(
    key: KeyCode,
    value: i32,
    pending_typing_assist_after_space: &mut Option<PendingTypingAssist>,
) -> bool {
    if key == KeyCode::KEY_BACKSPACE && matches!(value, 1 | 2) {
        pending_typing_assist_after_space.take().is_some()
    } else {
        false
    }
}

fn plain_backspace_no_modifiers(value: i32, shift_state: &ShiftState) -> bool {
    matches!(value, 1 | 2) && !shift_state.any_modifier_active()
}
