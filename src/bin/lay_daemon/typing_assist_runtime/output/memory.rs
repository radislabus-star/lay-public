use lay::keyboard::KeyEvent;
use lay::text_edit::TextReplacement;
use lay::word_buffer::WordBuffer;
use std::time::Instant;

use super::super::super::correction_memory_runtime::{
    remember_assisted_text_correction, AssistedCorrectionMemory,
};
use super::super::super::record_recent_action;

pub(crate) fn remember_typing_assist_correction(
    buf: &mut WordBuffer,
    events: &[KeyEvent],
    plan: &TextReplacement,
    original: &str,
    replacement: &str,
    cursor_offset: u32,
    started_at: Instant,
) {
    let words = original.split_whitespace().count();
    remember_assisted_text_correction(
        buf,
        AssistedCorrectionMemory {
            events,
            plan,
            original,
            replacement,
            kind: "typing-assist",
            replace_words: words.max(1),
            words,
            cursor_offset,
        },
    );
    record_recent_action(
        "typing-assist",
        original,
        replacement,
        words.max(1),
        words,
        started_at,
        true,
    );
}
