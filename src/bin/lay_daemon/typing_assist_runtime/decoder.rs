use lay::config::TypingAssistRuleConfig;
use lay::decoder::{
    decode_typing_assist_tail_with_context, decode_typing_assist_tail_with_context_and_nanda,
    CorrectionSource, DecoderEditPlan,
};
use lay::keyboard::KeyEvent;
use lay::typing_context::completed_tail_context;
use lay::word_buffer::WordBuffer;

#[cfg(not(test))]
use super::super::super::{
    active_nanda_autocorrect, active_typing_assist_pipeline_for_auto_replace,
};

pub(super) fn decode_completed_tail(
    buf: &WordBuffer,
    word_count: usize,
    events: &[KeyEvent],
    allow_layout_auto: bool,
) -> Option<DecoderEditPlan> {
    let context = completed_tail_context(buf, word_count, events);
    let pipeline = active_pipeline(&context);
    if nanda_enabled() {
        decode_typing_assist_tail_with_context_and_nanda(
            events,
            &context,
            allow_layout_auto,
            &pipeline,
            CorrectionSource::TypingAssist,
            true,
        )
    } else {
        decode_typing_assist_tail_with_context(
            events,
            &context,
            allow_layout_auto,
            &pipeline,
            CorrectionSource::TypingAssist,
        )
    }
}

pub(super) fn nanda_enabled() -> bool {
    #[cfg(test)]
    {
        false
    }
    #[cfg(not(test))]
    {
        active_nanda_autocorrect()
    }
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
