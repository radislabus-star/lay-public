//! Shared text-correction decision facade.
//!
//! Runtime backends still own output and state. This module only answers one
//! question: should this completed text be replaced, and by which engine?

use crate::config::{CorrectionSafety, TypingAssistRuleConfig};
use crate::nanda_wave::{run_wave_trace, WaveDecision};
use crate::typing_assist::apply_typing_assist_with_pipeline;
use crate::typing_context::typing_assist_pipeline_for_context;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionMode {
    DeterministicOnly,
    NandaOnly,
    DeterministicThenNanda,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionDecisionSource {
    Deterministic,
    Nanda,
}

#[derive(Debug, Clone)]
pub struct CorrectionRequest<'a> {
    pub text: &'a str,
    pub auto_replace: bool,
    pub typing_assist: bool,
    pub auto_switch_layout: bool,
    pub correction_safety: CorrectionSafety,
    pub typing_assist_pipeline: &'a [TypingAssistRuleConfig],
    pub nanda_autocorrect: bool,
    pub mode: CorrectionMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrectionDecision {
    pub replacement: String,
    pub source: CorrectionDecisionSource,
}

pub fn decide_text_correction(req: CorrectionRequest<'_>) -> Option<CorrectionDecision> {
    match req.mode {
        CorrectionMode::DeterministicOnly => deterministic_text_correction(&req),
        CorrectionMode::NandaOnly => nanda_text_correction(&req),
        CorrectionMode::DeterministicThenNanda => {
            deterministic_text_correction(&req).or_else(|| nanda_text_correction(&req))
        }
    }
}

fn deterministic_text_correction(req: &CorrectionRequest<'_>) -> Option<CorrectionDecision> {
    if !(req.auto_replace || req.typing_assist || req.auto_switch_layout) {
        return None;
    }

    let pipeline = typing_assist_pipeline_for_context(
        req.auto_replace,
        req.correction_safety,
        req.typing_assist_pipeline,
        req.text,
    );
    apply_typing_assist_with_pipeline(req.text, req.auto_switch_layout, &pipeline).map(
        |replacement| CorrectionDecision {
            replacement,
            source: CorrectionDecisionSource::Deterministic,
        },
    )
}

fn nanda_text_correction(req: &CorrectionRequest<'_>) -> Option<CorrectionDecision> {
    if !req.nanda_autocorrect {
        return None;
    }

    match run_wave_trace(req.text).decision {
        WaveDecision::Apply { text, .. } if text != req.text => Some(CorrectionDecision {
            replacement: text,
            source: CorrectionDecisionSource::Nanda,
        }),
        WaveDecision::Apply { .. } | WaveDecision::Keep { .. } | WaveDecision::Veto { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_typing_assist_pipeline;

    fn request<'a>(
        text: &'a str,
        pipeline: &'a [TypingAssistRuleConfig],
        mode: CorrectionMode,
    ) -> CorrectionRequest<'a> {
        CorrectionRequest {
            text,
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: pipeline,
            nanda_autocorrect: true,
            mode,
        }
    }

    #[test]
    fn deterministic_mode_corrects_wrong_layout_text() {
        let pipeline = default_typing_assist_pipeline();
        let decision = decide_text_correction(request(
            "lfdfq ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ))
        .unwrap();
        assert_eq!(decision.replacement, "давай ");
        assert_eq!(decision.source, CorrectionDecisionSource::Deterministic);
    }

    #[test]
    fn nanda_mode_corrects_wave_writer_text() {
        let pipeline = default_typing_assist_pipeline();
        let decision =
            decide_text_correction(request("тфтвф ", &pipeline, CorrectionMode::NandaOnly))
                .expect("nanda should produce a layout candidate");
        assert_eq!(decision.replacement, "nanda ");
        assert_eq!(decision.source, CorrectionDecisionSource::Nanda);
    }

    #[test]
    fn disabled_runtime_flags_keep_original() {
        let pipeline = default_typing_assist_pipeline();
        let decision = decide_text_correction(CorrectionRequest {
            text: "lfdfq ",
            auto_replace: false,
            typing_assist: false,
            auto_switch_layout: false,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: &pipeline,
            nanda_autocorrect: false,
            mode: CorrectionMode::DeterministicThenNanda,
        });
        assert_eq!(decision, None);
    }
}
