//! Unified input gate.
//!
//! This module is the stream-level owner for input decisions. Candidate
//! generators and correction engines may propose, but callers should route live
//! input events through this gate before applying or showing anything.

use crate::config::{CorrectionSafety, TypingAssistRuleConfig};
use crate::correction_core::{
    resolve_text_correction, CandidateGateAction, CorrectionDecisionSource, CorrectionMode,
    CorrectionRequest, CorrectionResolution, TypingErrorClass,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputGateTrigger {
    KeyChar,
    Backspace,
    Space,
    Enter,
    DoubleShift,
    TabAccept,
    FocusChanged,
    LayoutChanged,
    ImeCompositionChanged,
}

impl InputGateTrigger {
    pub fn closes_word(self) -> bool {
        matches!(self, Self::Space | Self::Enter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputGateStage {
    LiveInput,
    WordBoundary,
    ManualToggle,
    CompletionAccept,
    FocusOrLayout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputGateAction {
    Observe,
    KeepOriginal,
    ApplyReplacement {
        replacement: String,
        source: CorrectionDecisionSource,
    },
    SuggestOnly {
        best: Option<String>,
    },
    Veto {
        reason: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputGateDecision {
    pub trigger: InputGateTrigger,
    pub stage: InputGateStage,
    pub action: InputGateAction,
    pub correction: Option<CorrectionResolution>,
}

#[derive(Debug, Clone)]
pub struct InputGateRequest<'a> {
    pub trigger: InputGateTrigger,
    pub text_tail: &'a str,
    pub auto_replace: bool,
    pub typing_assist: bool,
    pub auto_switch_layout: bool,
    pub correction_safety: CorrectionSafety,
    pub typing_assist_pipeline: &'a [TypingAssistRuleConfig],
    pub nanda_autocorrect: bool,
    pub correction_mode: CorrectionMode,
}

pub fn decide_input_gate(req: InputGateRequest<'_>) -> InputGateDecision {
    match req.trigger {
        InputGateTrigger::Space | InputGateTrigger::Enter => decide_word_boundary(req),
        InputGateTrigger::DoubleShift => InputGateDecision {
            trigger: req.trigger,
            stage: InputGateStage::ManualToggle,
            action: InputGateAction::Observe,
            correction: None,
        },
        InputGateTrigger::TabAccept => InputGateDecision {
            trigger: req.trigger,
            stage: InputGateStage::CompletionAccept,
            action: InputGateAction::Observe,
            correction: None,
        },
        InputGateTrigger::FocusChanged | InputGateTrigger::LayoutChanged => InputGateDecision {
            trigger: req.trigger,
            stage: InputGateStage::FocusOrLayout,
            action: InputGateAction::Observe,
            correction: None,
        },
        InputGateTrigger::KeyChar
        | InputGateTrigger::Backspace
        | InputGateTrigger::ImeCompositionChanged => InputGateDecision {
            trigger: req.trigger,
            stage: InputGateStage::LiveInput,
            action: InputGateAction::Observe,
            correction: None,
        },
    }
}

fn decide_word_boundary(req: InputGateRequest<'_>) -> InputGateDecision {
    let resolution = resolve_text_correction(CorrectionRequest {
        text: req.text_tail,
        auto_replace: req.auto_replace,
        typing_assist: req.typing_assist,
        auto_switch_layout: req.auto_switch_layout,
        correction_safety: req.correction_safety,
        typing_assist_pipeline: req.typing_assist_pipeline,
        nanda_autocorrect: req.nanda_autocorrect,
        mode: req.correction_mode,
    });
    let action = word_boundary_action(&resolution);

    InputGateDecision {
        trigger: req.trigger,
        stage: InputGateStage::WordBoundary,
        action,
        correction: Some(resolution),
    }
}

fn word_boundary_action(resolution: &CorrectionResolution) -> InputGateAction {
    if let Some(decision) = &resolution.decision {
        return InputGateAction::ApplyReplacement {
            replacement: decision.replacement.clone(),
            source: decision.source,
        };
    }

    let mut best_suggestion = None;
    let mut veto_reason = None;
    for candidate in &resolution.candidates {
        match candidate.gate.action {
            CandidateGateAction::SuggestOnly => {
                best_suggestion.get_or_insert_with(|| candidate.replacement.clone());
            }
            CandidateGateAction::Veto => {
                veto_reason.get_or_insert(candidate.gate.reason);
            }
            CandidateGateAction::KeepOriginal | CandidateGateAction::Apply => {}
        }
    }

    if best_suggestion.is_some() {
        InputGateAction::SuggestOnly {
            best: best_suggestion,
        }
    } else if let Some(reason) = veto_reason {
        InputGateAction::Veto { reason }
    } else {
        InputGateAction::KeepOriginal
    }
}

pub fn selected_error_class(decision: &InputGateDecision) -> Option<TypingErrorClass> {
    decision
        .correction
        .as_ref()
        .and_then(|resolution| resolution.selected.as_ref())
        .map(|candidate| candidate.error_class)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_typing_assist_pipeline;

    fn request<'a>(trigger: InputGateTrigger, text_tail: &'a str) -> InputGateRequest<'a> {
        InputGateRequest {
            trigger,
            text_tail,
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: &[],
            nanda_autocorrect: true,
            correction_mode: CorrectionMode::DeterministicThenNanda,
        }
    }

    fn request_with_pipeline<'a>(
        trigger: InputGateTrigger,
        text_tail: &'a str,
        pipeline: &'a [TypingAssistRuleConfig],
    ) -> InputGateRequest<'a> {
        InputGateRequest {
            typing_assist_pipeline: pipeline,
            ..request(trigger, text_tail)
        }
    }

    #[test]
    fn live_key_only_observes_without_correction() {
        let decision = decide_input_gate(request(InputGateTrigger::KeyChar, "пров"));
        assert_eq!(decision.stage, InputGateStage::LiveInput);
        assert_eq!(decision.action, InputGateAction::Observe);
        assert!(decision.correction.is_none());
    }

    #[test]
    fn space_boundary_applies_existing_correction_core_decision() {
        let pipeline = default_typing_assist_pipeline();
        let decision = decide_input_gate(request_with_pipeline(
            InputGateTrigger::Space,
            "lfdfq ",
            &pipeline,
        ));
        assert_eq!(decision.stage, InputGateStage::WordBoundary);
        assert_eq!(
            decision.action,
            InputGateAction::ApplyReplacement {
                replacement: "давай ".to_string(),
                source: CorrectionDecisionSource::Deterministic,
            }
        );
        assert_eq!(
            selected_error_class(&decision),
            Some(TypingErrorClass::WrongLayout)
        );
    }

    #[test]
    fn word_boundary_can_keep_original_after_safety_gate() {
        let pipeline = default_typing_assist_pipeline();
        let decision = decide_input_gate(request_with_pipeline(
            InputGateTrigger::Space,
            "патерна ",
            &pipeline,
        ));
        assert_eq!(decision.action, InputGateAction::KeepOriginal);
    }

    #[test]
    fn disabled_sources_keep_original_on_boundary() {
        let pipeline = default_typing_assist_pipeline();
        let decision = decide_input_gate(InputGateRequest {
            trigger: InputGateTrigger::Space,
            text_tail: "lfdfq ",
            auto_replace: false,
            typing_assist: false,
            auto_switch_layout: false,
            correction_safety: CorrectionSafety::Normal,
            typing_assist_pipeline: &pipeline,
            nanda_autocorrect: false,
            correction_mode: CorrectionMode::DeterministicThenNanda,
        });
        assert_eq!(decision.action, InputGateAction::KeepOriginal);
    }
}
