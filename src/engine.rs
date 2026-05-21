//! Platform-neutral correction engine.
//!
//! Desktop adapters provide buffered key events and current policy. The engine
//! decides what should happen, but never performs DBus calls, uinput replay,
//! layout switching, sleeps, logging, or file I/O.

use crate::config::CorrectionEngine;
use crate::decoder::{
    decode_manual_tail, DecoderAction, DecoderEditPlan, ManualDecodeRequest, RankedDecoderDecision,
};
use crate::keyboard::{preferred_layout_for_text, replay_layout_decision, KeyEvent};
use crate::typing_assist::ScopedTailOptions;

#[derive(Debug, Clone, Copy)]
pub struct ManualCorrectionPolicy {
    pub engine: CorrectionEngine,
    pub force_replay: bool,
    pub auto_replace: bool,
    pub scoped_options: ScopedTailOptions,
}

#[derive(Debug, Clone, Copy)]
pub struct ManualCorrectionInput<'a> {
    pub events: &'a [KeyEvent],
    pub original: &'a str,
    pub converted: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualCorrectionDecision {
    pub action: DecoderAction,
    pub edit: Option<DecoderEditPlan>,
    pub ranked: Option<RankedDecoderDecision>,
    pub replay_target_is_ru: bool,
    pub replay_mixed_layouts: bool,
    pub output_text: String,
    pub output_target_is_ru: bool,
}

pub fn decide_manual_correction(
    input: ManualCorrectionInput<'_>,
    policy: ManualCorrectionPolicy,
) -> ManualCorrectionDecision {
    let replay = replay_layout_decision(input.events);
    let decoded = decode_manual_tail(ManualDecodeRequest {
        events: input.events,
        original: input.original,
        converted: input.converted,
        engine: policy.engine,
        force_replay: policy.force_replay,
        auto_replace: policy.auto_replace,
        scoped_options: policy.scoped_options,
    });

    let output_text = decoded
        .action
        .replacement_text()
        .unwrap_or(input.converted)
        .to_string();
    let output_target_is_ru = preferred_layout_for_text(&output_text, replay.target_is_ru);

    ManualCorrectionDecision {
        action: decoded.action,
        edit: decoded.edit,
        ranked: decoded.ranked,
        replay_target_is_ru: replay.target_is_ru,
        replay_mixed_layouts: replay.mixed_layouts,
        output_text,
        output_target_is_ru,
    }
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
