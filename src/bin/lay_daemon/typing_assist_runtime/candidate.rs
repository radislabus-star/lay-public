mod decoder;

use decoder::{decode_completed_tail, nanda_enabled};
use lay::decoder::DecoderEditPlan;
use lay::keyboard::KeyEvent;
use lay::word_buffer::{WordBuffer, MAX_REPLACE_WORDS};
use std::time::Instant;

#[derive(Clone)]
pub(crate) struct TypingAssistCorrection {
    pub(crate) events: Vec<KeyEvent>,
    pub(crate) edit: DecoderEditPlan,
}

pub(crate) fn find_typing_assist_correction(
    buf: &WordBuffer,
    allow_layout_auto: bool,
    max_words: usize,
) -> Option<TypingAssistCorrection> {
    completed_tail_scopes(buf, max_words)
        .into_iter()
        .find_map(|word_count| {
            let started = Instant::now();
            let events = buf.last_completed_words_events(word_count)?;
            let nanda_enabled = nanda_enabled();
            let edit = decode_completed_tail(buf, word_count, &events, allow_layout_auto)?;
            super::super::log(&format!(
                "  typing-assist decision: scope={} nanda={} elapsed={}ms",
                word_count,
                nanda_enabled,
                started.elapsed().as_millis()
            ));
            Some(TypingAssistCorrection { events, edit })
        })
}

fn completed_tail_scopes(buf: &WordBuffer, max_words: usize) -> Vec<usize> {
    let max_scope = buf
        .prev_words_len()
        .min(max_words.max(1))
        .min(MAX_REPLACE_WORDS);
    (1..=max_scope).rev().collect()
}
