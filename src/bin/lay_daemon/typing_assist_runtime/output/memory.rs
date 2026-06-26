use lay::keyboard::KeyEvent;
use lay::text_edit::TextReplacement;
use lay::word_buffer::WordBuffer;
use std::time::Instant;

use super::super::super::correction_memory_runtime::{
    remember_assisted_text_correction, AssistedCorrectionMemory,
};
use super::nanda_trace::record_nanda_trace_if_enabled;

#[derive(Clone, Copy)]
pub(crate) struct TypingAssistTiming {
    pub(crate) decision_ms: u128,
    pub(crate) started_at: Instant,
}

pub(crate) struct TypingAssistMemoryContext<'a> {
    pub(crate) buf: &'a mut WordBuffer,
    pub(crate) events: &'a [KeyEvent],
    pub(crate) plan: &'a TextReplacement,
    pub(crate) original: &'a str,
    pub(crate) replacement: &'a str,
    pub(crate) rule_id: Option<&'a str>,
    pub(crate) input_gate: Option<lay::action_log::RecentActionGateTrace>,
    pub(crate) cursor_offset: u32,
    pub(crate) timing: TypingAssistTiming,
}

pub(crate) fn remember_typing_assist_correction(ctx: TypingAssistMemoryContext<'_>) {
    let TypingAssistMemoryContext {
        buf,
        events,
        plan,
        original,
        replacement,
        rule_id,
        input_gate,
        cursor_offset,
        timing,
    } = ctx;
    let words = original.split_whitespace().count();
    remember_assisted_text_correction(
        buf,
        AssistedCorrectionMemory {
            events,
            plan,
            original,
            replacement,
            kind: "typing-assist",
            rule_id,
            replace_words: words.max(1),
            words,
            cursor_offset,
        },
    );
    let output_ms = timing.started_at.elapsed().as_millis();
    lay::action_log::record_action_with_stages_and_gate(
        "typing-assist",
        original,
        replacement,
        words.max(1),
        words,
        timing.decision_ms + output_ms,
        Some(timing.decision_ms),
        Some(output_ms),
        input_gate,
        true,
    );
    record_nanda_trace_if_enabled(original, replacement);
}
