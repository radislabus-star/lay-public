use crate::keyboard::{map_original_events, KeyEvent};
use crate::word_buffer::WordBuffer;

const COMPLETED_TAIL_CONTEXT_WORDS: usize = 24;

pub fn completed_tail_context(
    buf: &WordBuffer,
    word_count: usize,
    fallback_events: &[KeyEvent],
) -> String {
    if word_count == 0 {
        return map_original_events(fallback_events);
    }

    let max_context_words = COMPLETED_TAIL_CONTEXT_WORDS.max(word_count);
    if word_count < max_context_words {
        for context_words in ((word_count + 1)..=max_context_words).rev() {
            if let Some(context_events) = buf.last_completed_words_events(context_words) {
                return map_original_events(&context_events);
            }
        }
    }

    map_original_events(fallback_events)
}
