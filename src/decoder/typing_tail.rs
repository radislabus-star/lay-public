use super::edit_plan::DecoderEditPlan;
use super::types::{CorrectionSource, CorrectionTrigger};
use crate::config::TypingAssistRuleConfig;
use crate::keyboard::{map_original_events, KeyEvent};
use crate::typing_assist::{apply_typing_assist_with_nanda, apply_typing_assist_with_pipeline};

mod punctuation;

pub use punctuation::{decode_enter_autocorrect_tail, decode_typing_assist_current_tail};

pub fn decode_typing_assist_tail(
    events: &[KeyEvent],
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
    source: CorrectionSource,
) -> Option<DecoderEditPlan> {
    let original = map_original_events(events);
    let replacement = apply_typing_assist_with_pipeline(&original, allow_layout_auto, pipeline)?;
    changed_committed_tail_plan(
        CorrectionTrigger::AfterSpace,
        &original,
        &replacement,
        source,
    )
}
pub fn decode_typing_assist_tail_with_context(
    events: &[KeyEvent],
    context: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
    source: CorrectionSource,
) -> Option<DecoderEditPlan> {
    decode_typing_assist_tail_with_context_and_nanda(
        events,
        context,
        allow_layout_auto,
        pipeline,
        source,
        false,
    )
}

pub fn decode_typing_assist_tail_with_context_and_nanda(
    events: &[KeyEvent],
    context: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
    source: CorrectionSource,
    nanda_enabled: bool,
) -> Option<DecoderEditPlan> {
    let original = map_original_events(events);
    if let Some(prefix) = context.strip_suffix(&original) {
        if !prefix.is_empty() {
            if let Some(replacement_context) =
                apply_typing_assist_runtime(context, allow_layout_auto, pipeline, nanda_enabled)
            {
                if replacement_context != context && replacement_context.starts_with(prefix) {
                    let replacement = &replacement_context[prefix.len()..];
                    return changed_committed_tail_plan(
                        CorrectionTrigger::AfterSpace,
                        &original,
                        replacement,
                        source,
                    );
                }
            }
        }
    }
    decode_typing_assist_tail_runtime(events, allow_layout_auto, pipeline, source, nanda_enabled)
}

fn decode_typing_assist_tail_runtime(
    events: &[KeyEvent],
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
    source: CorrectionSource,
    nanda_enabled: bool,
) -> Option<DecoderEditPlan> {
    let original = map_original_events(events);
    let replacement =
        apply_typing_assist_runtime(&original, allow_layout_auto, pipeline, nanda_enabled)?;
    changed_committed_tail_plan(
        CorrectionTrigger::AfterSpace,
        &original,
        &replacement,
        source,
    )
}

fn apply_typing_assist_runtime(
    text: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
    nanda_enabled: bool,
) -> Option<String> {
    if nanda_enabled {
        apply_typing_assist_with_nanda(text, allow_layout_auto, pipeline)
    } else {
        apply_typing_assist_with_pipeline(text, allow_layout_auto, pipeline)
    }
}

pub(super) fn changed_committed_tail_plan(
    trigger: CorrectionTrigger,
    original: &str,
    replacement: &str,
    source: CorrectionSource,
) -> Option<DecoderEditPlan> {
    if replacement == original || replacement.trim().is_empty() {
        return None;
    }
    DecoderEditPlan::committed_tail(trigger, original, replacement, source)
}
