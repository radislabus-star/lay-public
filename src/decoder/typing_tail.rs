use super::edit_plan::DecoderEditPlan;
use super::types::{CorrectionSource, CorrectionTrigger};
use crate::config::{CorrectionSafety, TypingAssistRuleConfig};
use crate::correction_core::CorrectionMode;
use crate::input_gate::{decide_input_gate, InputGateAction, InputGateRequest, InputGateTrigger};
use crate::keyboard::{map_original_events, KeyEvent};

mod punctuation;

pub use punctuation::{decode_enter_autocorrect_tail, decode_typing_assist_current_tail};

pub fn decode_typing_assist_tail(
    events: &[KeyEvent],
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
    source: CorrectionSource,
) -> Option<DecoderEditPlan> {
    let original = map_original_events(events);
    let replacement = decode_typing_assist_text(&original, allow_layout_auto, pipeline)?;
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
                decode_typing_assist_text(context, allow_layout_auto, pipeline)
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

fn decode_typing_assist_text(
    text_tail: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<String> {
    let decision = decide_input_gate(InputGateRequest {
        trigger: InputGateTrigger::Space,
        text_tail,
        auto_replace: true,
        typing_assist: true,
        auto_switch_layout: allow_layout_auto,
        correction_safety: CorrectionSafety::Normal,
        typing_assist_pipeline: pipeline,
        nanda_autocorrect: false,
        correction_mode: CorrectionMode::DeterministicOnly,
    });
    let InputGateAction::ApplyReplacement { replacement, .. } = decision.action else {
        return None;
    };
    Some(replacement)
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
