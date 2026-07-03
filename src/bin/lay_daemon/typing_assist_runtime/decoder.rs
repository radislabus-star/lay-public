include!("decoder/runtime.rs");
include!("decoder/gate.rs");
#[cfg(test)]
mod tests;

use lay::config::TypingAssistRuleConfig;
use lay::decoder::{CorrectionSource, DecoderEditPlan};
use lay::keyboard::KeyEvent;
use lay::typing_context::completed_tail_context;
use lay::word_buffer::WordBuffer;

pub(super) fn decode_completed_tail(
    buf: &WordBuffer,
    word_count: usize,
    events: &[KeyEvent],
    allow_layout_auto: bool,
) -> Option<DecodedCompletedTail> {
    let context = completed_tail_context(buf, word_count, events);
    let pipeline = active_pipeline(&context);
    if let Some(decoded) = decode_input_gate_tail(events, &context, allow_layout_auto, &pipeline) {
        return Some(decoded);
    }
    decode_input_gate_tail(events, &context, allow_layout_auto, &pipeline)
}

#[derive(Debug, Clone)]
pub(super) struct DecodedCompletedTail {
    pub(super) edit: DecoderEditPlan,
    pub(super) rule_id: Option<String>,
    pub(super) input_gate: Option<lay::action_log::RecentActionGateTrace>,
}

impl DecodedCompletedTail {
    fn with_input_gate(
        edit: DecoderEditPlan,
        rule_id: Option<String>,
        input_gate: Option<lay::action_log::RecentActionGateTrace>,
    ) -> Self {
        Self {
            edit,
            rule_id,
            input_gate,
        }
    }
}
