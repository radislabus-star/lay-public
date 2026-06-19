use lay::keyboard::KeyEvent;
use lay::word_buffer::WordBuffer;
use std::time::{Duration, Instant};

use super::{
    log, read_current_layout_is_ru, record_precognition_tick_if_enabled,
    sync_ime_engine_to_current_layout, ShiftState, LAYOUT_POLL_INTERVAL_MS,
};

pub(super) struct TypingKeyContext<'a> {
    pub(super) buffer: &'a mut WordBuffer,
    pub(super) shift_state: &'a ShiftState,
    pub(super) current_layout_is_ru: &'a mut bool,
    pub(super) last_layout_poll: &'a mut Instant,
    pub(super) events_since_word_start: &'a mut u32,
    pub(super) clear_on_next_typing: &'a mut bool,
    pub(super) ignore_current_token_until_space: &'a mut bool,
    pub(super) suppress_next_typing_assist_after_manual_replay: &'a mut bool,
    pub(super) verbose: bool,
}

pub(super) fn handle_typing_key_press(code: u16, value: i32, ctx: TypingKeyContext<'_>) {
    if *ctx.clear_on_next_typing {
        ctx.buffer.reset_all();
        *ctx.events_since_word_start = 0;
        *ctx.clear_on_next_typing = false;
        *ctx.ignore_current_token_until_space = false;
        *ctx.suppress_next_typing_assist_after_manual_replay = false;
    }
    let starts_new_word = ctx.buffer.current_is_empty();
    *ctx.events_since_word_start += 1;

    let accept = if value == 2 {
        ctx.buffer.current_last_keycode() == Some(code)
    } else {
        true
    };
    if !accept {
        if ctx.verbose {
            log(&format!(
                "· key {code} v=2 SKIP (autorepeat другой) events={}",
                *ctx.events_since_word_start
            ));
        }
        return;
    }
    if starts_new_word
        || ctx.last_layout_poll.elapsed() >= Duration::from_millis(LAYOUT_POLL_INTERVAL_MS)
    {
        if let Ok(is_ru) = read_current_layout_is_ru() {
            *ctx.current_layout_is_ru = is_ru;
            sync_ime_engine_to_current_layout(is_ru);
        }
        *ctx.last_layout_poll = Instant::now();
    }
    let typed_event = KeyEvent {
        keycode: code,
        shift: ctx.shift_state.any(),
        layout_is_ru: *ctx.current_layout_is_ru,
    };
    ctx.buffer.push(typed_event);
    ctx.buffer.note_learning_typed(typed_event);
    record_precognition_tick_if_enabled("key", ctx.buffer);
    if ctx.verbose {
        log(&format!(
            "· key {code} v={value} shift={} → current={} events={}",
            ctx.shift_state.any(),
            ctx.buffer.current_len(),
            *ctx.events_since_word_start
        ));
    }
}
