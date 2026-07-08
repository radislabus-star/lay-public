use super::{changed_committed_tail_plan_from_gate, decode_input_gate_decision};
use crate::config::TypingAssistRuleConfig;
use crate::decoder::types::{CorrectionSource, CorrectionTrigger};
use crate::input_gate::InputGateTrigger;
use crate::keyboard::{map_original_events, KeyEvent};
use crate::text_edit::ensure_committed_tail_spacing;

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
    let decision = decode_input_gate_decision(
        InputGateTrigger::Space,
        &assist_input,
        allow_layout_auto,
        pipeline,
    );
    let crate::input_gate::InputGateAction::ApplyReplacement { replacement, .. } = &decision.action
    else {
        return None;
    };
    let replacement = replacement.trim_end().to_string();
    changed_committed_tail_plan_from_gate(
        &decision,
        CorrectionTrigger::AfterPunctuation,
        &original,
        &replacement,
        source,
    )
}

pub fn decode_enter_autocorrect_tail(
    events: &[KeyEvent],
    original_has_trailing_space: bool,
    _allow_layout_auto: bool,
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
    let decision =
        decode_input_gate_decision(InputGateTrigger::Enter, &assist_input, false, pipeline);
    let crate::input_gate::InputGateAction::ApplyReplacement { replacement, .. } = &decision.action
    else {
        return None;
    };
    let mut replacement = replacement.clone();
    if original_has_trailing_space {
        replacement = ensure_committed_tail_spacing(&original, replacement);
    } else {
        replacement = replacement.trim_end().to_string();
    }
    changed_committed_tail_plan_from_gate(
        &decision,
        CorrectionTrigger::Enter,
        &original,
        &replacement,
        CorrectionSource::EnterAutocorrect,
    )
}
