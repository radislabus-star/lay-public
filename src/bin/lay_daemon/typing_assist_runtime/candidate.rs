mod decoder;

use decoder::decode_completed_tail;
use lay::decoder::DecoderEditPlan;
use lay::keyboard::KeyEvent;
use lay::word_buffer::{WordBuffer, MAX_REPLACE_WORDS};
use std::time::Instant;

#[derive(Clone)]
pub(crate) struct TypingAssistCorrection {
    pub(crate) events: Vec<KeyEvent>,
    pub(crate) edit: DecoderEditPlan,
    pub(crate) rule_id: Option<String>,
    pub(crate) input_gate: Option<lay::action_log::RecentActionGateTrace>,
    pub(crate) decision_ms: u128,
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
            let decoded = decode_completed_tail(buf, word_count, &events, allow_layout_auto)?;
            let decision_ms = started.elapsed().as_millis();
            super::super::log(&format!(
                "  typing-assist decision: scope={} elapsed={}ms",
                word_count, decision_ms
            ));
            Some(TypingAssistCorrection {
                events,
                edit: decoded.edit,
                rule_id: decoded.rule_id,
                input_gate: decoded.input_gate,
                decision_ms,
            })
        })
}

fn completed_tail_scopes(buf: &WordBuffer, max_words: usize) -> Vec<usize> {
    completed_tail_scopes_from_len(buf.prev_words_len(), max_words)
}

fn completed_tail_scopes_from_len(prev_words_len: usize, max_words: usize) -> Vec<usize> {
    let max_scope = prev_words_len.min(max_words.max(1)).min(MAX_REPLACE_WORDS);
    if max_scope == 0 {
        Vec::new()
    } else if max_scope >= 2 {
        // The completed word is the normal correction unit. A wider context is
        // a fallback only when this local transition has no candidate.
        vec![1, 2]
    } else {
        vec![1]
    }
}

#[cfg(test)]
mod tests {
    use super::completed_tail_scopes_from_len;

    #[test]
    fn checks_current_word_before_optional_context() {
        assert_eq!(completed_tail_scopes_from_len(1, 2), vec![1]);
        assert_eq!(completed_tail_scopes_from_len(2, 2), vec![1, 2]);
        assert_eq!(completed_tail_scopes_from_len(3, 3), vec![1, 2]);
    }
}
