use lay::word_buffer::WordBuffer;

use super::log;

pub(super) fn log_manual_trigger_cross_check(buffer: &WordBuffer, events_since_word_start: u32) {
    let buf_count = buffer.current_len() as u32;
    log(&format!(
        "═ CROSS-CHECK: buffer.current={} events_since_word_start={}{}",
        buf_count,
        events_since_word_start,
        if buf_count != events_since_word_start {
            " ⚠ MISMATCH"
        } else {
            " ✓"
        }
    ));
}
