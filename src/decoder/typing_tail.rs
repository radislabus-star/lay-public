use super::edit_plan::DecoderEditPlan;
use super::types::{CorrectionSource, CorrectionTrigger};
use crate::config::{CorrectionSafety, TypingAssistRuleConfig};
use crate::correction_core::CorrectionMode;
use crate::input_gate::{
    decide_input_gate, InputGateAction, InputGateDecision, InputGateRequest, InputGateTrigger,
};
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
    let decision = decode_input_gate_decision(
        InputGateTrigger::Space,
        &original,
        allow_layout_auto,
        pipeline,
    );
    let InputGateAction::ApplyReplacement { replacement, .. } = &decision.action else {
        return None;
    };
    changed_committed_tail_plan_from_gate(
        &decision,
        CorrectionTrigger::AfterSpace,
        &original,
        replacement,
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
            let decision = decode_input_gate_decision(
                InputGateTrigger::Space,
                context,
                allow_layout_auto,
                pipeline,
            );
            if let InputGateAction::ApplyReplacement {
                replacement: replacement_context,
                ..
            } = &decision.action
            {
                if replacement_context != context && replacement_context.starts_with(prefix) {
                    let replacement = &replacement_context[prefix.len()..];
                    return changed_committed_tail_plan_from_gate(
                        &decision,
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

pub(super) fn decode_input_gate_decision(
    trigger: InputGateTrigger,
    text_tail: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> InputGateDecision {
    decide_input_gate(InputGateRequest {
        trigger,
        text_tail,
        lexical_authority_frame: None,
        auto_replace: true,
        typing_assist: true,
        auto_switch_layout: allow_layout_auto,
        correction_safety: CorrectionSafety::Normal,
        typing_assist_pipeline: pipeline,
        nanda_autocorrect: false,
        nanda_candidate_route: crate::correction_core::CandidateReadoutRoute::FullWave,
        nanda_wave_options: crate::nanda_wave::WaveOptions::default(),
        correction_mode: CorrectionMode::DeterministicOnly,
    })
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

pub(super) fn changed_committed_tail_plan_from_gate(
    decision: &InputGateDecision,
    trigger: CorrectionTrigger,
    original: &str,
    replacement: &str,
    source: CorrectionSource,
) -> Option<DecoderEditPlan> {
    let edit = changed_committed_tail_plan(trigger, original, replacement, source)?;
    Some(edit.with_text_edit_input_gate_decision(decision))
}
