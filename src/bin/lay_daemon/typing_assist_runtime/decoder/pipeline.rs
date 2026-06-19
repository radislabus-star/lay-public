use lay::config::TypingAssistRuleConfig;
use lay::decoder::{CorrectionSource, CorrectionTrigger, DecoderEditPlan};
use lay::keyboard::{map_original_events, KeyEvent};
use lay::typing_assist::explain_typing_assist_with_pipeline;

use super::DecodedCompletedTail;

pub(super) fn decode_typing_assist_tail_with_context_and_rule(
    events: &[KeyEvent],
    context: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
    source: CorrectionSource,
) -> Option<DecodedCompletedTail> {
    let original = map_original_events(events);
    if let Some(decoded) =
        decode_context_tail(&original, context, allow_layout_auto, pipeline, source)
    {
        return Some(decoded);
    }
    decode_plain_tail(&original, allow_layout_auto, pipeline, source)
}

fn decode_context_tail(
    original: &str,
    context: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
    source: CorrectionSource,
) -> Option<DecodedCompletedTail> {
    let prefix = context.strip_suffix(original)?;
    if prefix.is_empty() {
        return None;
    }
    let explanation = explain_typing_assist_with_pipeline(context, allow_layout_auto, pipeline);
    let replacement_context = explanation.output.as_deref()?;
    if replacement_context == context || !replacement_context.starts_with(prefix) {
        return None;
    }
    let replacement = &replacement_context[prefix.len()..];
    let edit = DecoderEditPlan::committed_tail(
        CorrectionTrigger::AfterSpace,
        original,
        replacement,
        source,
    )?;
    Some(DecodedCompletedTail {
        edit,
        rule_id: explanation.chosen.map(|candidate| candidate.rule_id),
    })
}

fn decode_plain_tail(
    original: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
    source: CorrectionSource,
) -> Option<DecodedCompletedTail> {
    let explanation = explain_typing_assist_with_pipeline(original, allow_layout_auto, pipeline);
    let replacement = explanation.output.as_deref()?;
    let edit = DecoderEditPlan::committed_tail(
        CorrectionTrigger::AfterSpace,
        original,
        replacement,
        source,
    )?;
    Some(DecodedCompletedTail {
        edit,
        rule_id: explanation.chosen.map(|candidate| candidate.rule_id),
    })
}
