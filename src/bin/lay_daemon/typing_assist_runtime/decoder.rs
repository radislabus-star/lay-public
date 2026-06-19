use lay::config::TypingAssistRuleConfig;
use lay::decoder::{CorrectionSource, DecoderEditPlan};
use lay::keyboard::KeyEvent;
use lay::typing_context::completed_tail_context;
use lay::word_buffer::WordBuffer;

#[path = "decoder/nanda.rs"]
mod nanda;
#[path = "decoder/pipeline.rs"]
mod pipeline;

#[cfg(not(test))]
use super::super::super::active_typing_assist_pipeline_for_auto_replace;

pub(super) fn decode_completed_tail(
    buf: &WordBuffer,
    word_count: usize,
    events: &[KeyEvent],
    allow_layout_auto: bool,
) -> Option<DecodedCompletedTail> {
    let context = completed_tail_context(buf, word_count, events);
    let pipeline = active_pipeline(&context);
    pipeline::decode_typing_assist_tail_with_context_and_rule(
        events,
        &context,
        allow_layout_auto,
        &pipeline,
        CorrectionSource::TypingAssist,
    )
    .or_else(|| nanda::decode_nanda_memory_tail(events, &context))
}

#[derive(Debug, Clone)]
pub(super) struct DecodedCompletedTail {
    pub(super) edit: DecoderEditPlan,
    pub(super) rule_id: Option<String>,
}

#[cfg(test)]
fn active_pipeline(context: &str) -> Vec<TypingAssistRuleConfig> {
    lay::typing_context::typing_assist_pipeline_for_context(
        true,
        lay::config::CorrectionSafety::Normal,
        &lay::config::default_typing_assist_pipeline(),
        context,
    )
}

#[cfg(not(test))]
fn active_pipeline(context: &str) -> Vec<TypingAssistRuleConfig> {
    active_typing_assist_pipeline_for_auto_replace(context)
}
