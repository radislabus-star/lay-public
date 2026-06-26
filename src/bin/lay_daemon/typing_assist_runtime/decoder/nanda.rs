use lay::correction_core::{decide_text_correction, CorrectionMode, CorrectionRequest};
use lay::decoder::{CorrectionSource, CorrectionTrigger, DecoderEditPlan};
use lay::keyboard::{map_original_events, KeyEvent};
use lay::typing_assist::NANDA_WAVE_RULE_ID;

use super::DecodedCompletedTail;

#[cfg(not(test))]
use super::super::super::super::active_nanda_autocorrect;

pub(super) fn decode_nanda_memory_tail(
    events: &[KeyEvent],
    context: &str,
) -> Option<DecodedCompletedTail> {
    if !active_nanda_memory() {
        return None;
    }
    let original = map_original_events(events);
    let replacement = nanda_context_replacement(context, &original)
        .or_else(|| nanda_output(&original).filter(|output| output != &original))?;
    let edit = DecoderEditPlan::committed_tail(
        CorrectionTrigger::AfterSpace,
        &original,
        &replacement,
        CorrectionSource::TypingAssist,
    )?;
    Some(DecodedCompletedTail::new(
        edit,
        Some(NANDA_WAVE_RULE_ID.to_string()),
    ))
}

fn nanda_context_replacement(context: &str, original: &str) -> Option<String> {
    let prefix = context.strip_suffix(original)?;
    if prefix.is_empty() {
        return None;
    }
    let output = nanda_output(context)?;
    (output != context && output.starts_with(prefix)).then(|| output[prefix.len()..].to_string())
}

fn nanda_output(text: &str) -> Option<String> {
    decide_text_correction(CorrectionRequest {
        text,
        auto_replace: false,
        typing_assist: false,
        auto_switch_layout: false,
        correction_safety: lay::config::CorrectionSafety::Normal,
        typing_assist_pipeline: &[],
        nanda_autocorrect: true,
        mode: CorrectionMode::NandaOnly,
    })
    .map(|decision| decision.replacement)
}

#[cfg(test)]
fn active_nanda_memory() -> bool {
    false
}

#[cfg(not(test))]
fn active_nanda_memory() -> bool {
    active_nanda_autocorrect()
}
