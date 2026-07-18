use lay::word_buffer::WordBuffer;

use super::{active_nanda_precognition, active_nanda_trace};

const PRECOGNITION_TAIL_WORDS: usize = 8;

pub(super) fn record_precognition_tick_if_enabled(stage: &str, buffer: &WordBuffer) {
    if !active_nanda_precognition() {
        return;
    }
    let Some(text) = buffer.visible_tail_text(PRECOGNITION_TAIL_WORDS) else {
        return;
    };
    lay::typing_cpu::TypingCpu::record_precognition_tick(stage, &text, active_nanda_trace());
}
