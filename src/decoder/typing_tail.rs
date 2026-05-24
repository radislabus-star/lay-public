use super::edit_plan::DecoderEditPlan;
use super::types::{CorrectionSource, CorrectionTrigger};
use crate::config::TypingAssistRuleConfig;
use crate::keyboard::{map_original_events, KeyEvent};
use crate::text_edit::ensure_committed_tail_spacing;
use crate::typing_assist::apply_typing_assist_with_pipeline;

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
    let original = map_original_events(events);
    if let Some(prefix) = context.strip_suffix(&original) {
        if !prefix.is_empty() {
            if let Some(replacement_context) =
                apply_typing_assist_with_pipeline(context, allow_layout_auto, pipeline)
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
    decode_typing_assist_tail(events, allow_layout_auto, pipeline, source)
}

pub fn decode_typing_assist_current_tail(
    events: &[KeyEvent],
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
    source: CorrectionSource,
) -> Option<DecoderEditPlan> {
    let original = map_original_events(events);
    if original.trim().is_empty()
        || original
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace)
    {
        return None;
    }
    let assist_input = format!("{original} ");
    let replacement =
        apply_typing_assist_with_pipeline(&assist_input, allow_layout_auto, pipeline)?
            .trim_end()
            .to_string();
    changed_committed_tail_plan(
        CorrectionTrigger::AfterPunctuation,
        &original,
        &replacement,
        source,
    )
}
pub fn decode_enter_autocorrect_tail(
    events: &[KeyEvent],
    original_has_trailing_space: bool,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<DecoderEditPlan> {
    let original = map_original_events(events);
    if original.trim().is_empty() {
        return None;
    }
    let assist_input = if original_has_trailing_space {
        original.clone()
    } else {
        format!("{original} ")
    };
    let mut replacement =
        apply_typing_assist_with_pipeline(&assist_input, allow_layout_auto, pipeline)?;
    if original_has_trailing_space {
        replacement = ensure_committed_tail_spacing(&original, replacement);
    } else {
        replacement = replacement.trim_end().to_string();
    }
    changed_committed_tail_plan(
        CorrectionTrigger::Enter,
        &original,
        &replacement,
        CorrectionSource::EnterAutocorrect,
    )
}

fn changed_committed_tail_plan(
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
