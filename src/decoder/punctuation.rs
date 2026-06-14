use super::changed_committed_tail_plan;
use crate::config::TypingAssistRuleConfig;
use crate::decoder::types::{CorrectionSource, CorrectionTrigger};
use crate::keyboard::{map_original_events, KeyEvent};
use crate::text_edit::ensure_committed_tail_spacing;
use crate::typing_assist::apply_typing_assist_with_pipeline;

pub fn decode_typing_assist_current_tail(
    events: &[KeyEvent],
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
    source: CorrectionSource,
) -> Option<super::DecoderEditPlan> {
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
) -> Option<super::DecoderEditPlan> {
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
