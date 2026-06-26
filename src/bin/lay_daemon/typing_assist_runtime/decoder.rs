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
use super::super::super::{
    active_auto_replace, active_auto_switch_layout, active_correction_safety,
    active_nanda_autocorrect, active_typing_assist, active_typing_assist_pipeline_for_auto_replace,
};

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

fn decode_input_gate_tail(
    events: &[KeyEvent],
    context: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<DecodedCompletedTail> {
    let original = lay::keyboard::map_original_events(events);
    let text_tail = if context.ends_with(&original) {
        context
    } else {
        &original
    };
    let decision = lay::input_gate::decide_input_gate(lay::input_gate::InputGateRequest {
        trigger: lay::input_gate::InputGateTrigger::Space,
        text_tail,
        auto_replace: active_auto_replace_for_gate(),
        typing_assist: active_typing_assist_for_gate(),
        auto_switch_layout: allow_layout_auto && active_auto_switch_layout_for_gate(),
        correction_safety: active_correction_safety_for_gate(),
        typing_assist_pipeline: pipeline,
        nanda_autocorrect: active_nanda_autocorrect_for_gate(),
        correction_mode: lay::correction_core::CorrectionMode::DeterministicThenNanda,
    });
    let lay::input_gate::InputGateAction::ApplyReplacement { replacement, .. } = decision.action
    else {
        return None;
    };

    let replacement_tail = if text_tail == context {
        let prefix = context.strip_suffix(&original)?;
        if prefix.is_empty() {
            replacement.as_str()
        } else {
            replacement.strip_prefix(prefix)?
        }
    } else {
        replacement.as_str()
    };
    let edit = DecoderEditPlan::committed_tail(
        lay::decoder::CorrectionTrigger::AfterSpace,
        &original,
        replacement_tail,
        CorrectionSource::TypingAssist,
    )?;
    let rule_id = decision
        .correction
        .as_ref()
        .and_then(|resolution| resolution.selected.as_ref())
        .map(|candidate| {
            if candidate.source == lay::correction_core::CorrectionDecisionSource::Nanda {
                lay::typing_assist::NANDA_WAVE_RULE_ID.to_string()
            } else {
                candidate.source_id.clone()
            }
        });
    Some(DecodedCompletedTail { edit, rule_id })
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

#[cfg(test)]
fn active_auto_replace_for_gate() -> bool {
    true
}

#[cfg(not(test))]
fn active_auto_replace_for_gate() -> bool {
    active_auto_replace()
}

#[cfg(test)]
fn active_typing_assist_for_gate() -> bool {
    true
}

#[cfg(not(test))]
fn active_typing_assist_for_gate() -> bool {
    active_typing_assist()
}

#[cfg(test)]
fn active_auto_switch_layout_for_gate() -> bool {
    true
}

#[cfg(not(test))]
fn active_auto_switch_layout_for_gate() -> bool {
    active_auto_switch_layout()
}

#[cfg(test)]
fn active_nanda_autocorrect_for_gate() -> bool {
    false
}

#[cfg(not(test))]
fn active_nanda_autocorrect_for_gate() -> bool {
    active_nanda_autocorrect()
}

#[cfg(test)]
fn active_correction_safety_for_gate() -> lay::config::CorrectionSafety {
    lay::config::CorrectionSafety::Normal
}

#[cfg(not(test))]
fn active_correction_safety_for_gate() -> lay::config::CorrectionSafety {
    active_correction_safety()
}
