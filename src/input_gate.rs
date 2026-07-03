//! Unified input gate.
//!
//! This module is the stream-level owner for input decisions. Candidate
//! generators and correction engines may propose, but callers should route live
//! input events through this gate before applying or showing anything.

use crate::config::{CorrectionSafety, TypingAssistRuleConfig};
use crate::correction_core::{
    CandidateGateAction, CorrectionDecisionSource, CorrectionMode, CorrectionRequest,
    CorrectionResolution, TypingErrorClass,
};

include!("correction_pipeline.rs");

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
    pub trace: Option<InputGateDecisionTrace>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputGateDecisionTrace {
    pub stage: InputGateStage,
    pub input_class: Option<TypingErrorClass>,
    pub candidate_count: usize,
    pub selected_source: Option<CorrectionDecisionSource>,
    pub selected_source_id: Option<String>,
    pub selected_error_class: Option<TypingErrorClass>,
    pub selected_gate_action: Option<CandidateGateAction>,
    pub reason: &'static str,
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
        InputGateTrigger::Space | InputGateTrigger::Enter => decide_space_autocorrect(req),
        InputGateTrigger::DoubleShift => InputGateDecision {
            trigger: req.trigger,
            stage: InputGateStage::ManualToggle,
            action: InputGateAction::Observe,
            correction: None,
            trace: Some(observe_trace(
                InputGateStage::ManualToggle,
                "manual_toggle_owned_elsewhere",
            )),
        },
        InputGateTrigger::TabAccept => InputGateDecision {
            trigger: req.trigger,
            stage: InputGateStage::CompletionAccept,
            action: InputGateAction::Observe,
            correction: None,
            trace: Some(observe_trace(
                InputGateStage::CompletionAccept,
                "completion_accept_owned_elsewhere",
            )),
        },
        InputGateTrigger::FocusChanged | InputGateTrigger::LayoutChanged => InputGateDecision {
            trigger: req.trigger,
            stage: InputGateStage::FocusOrLayout,
            action: InputGateAction::Observe,
            correction: None,
            trace: Some(observe_trace(
                InputGateStage::FocusOrLayout,
                "state_change_only",
            )),
        },
        InputGateTrigger::KeyChar
        | InputGateTrigger::Backspace
        | InputGateTrigger::ImeCompositionChanged => InputGateDecision {
            trigger: req.trigger,
            stage: InputGateStage::LiveInput,
            action: InputGateAction::Observe,
            correction: None,
            trace: Some(observe_trace(
                InputGateStage::LiveInput,
                "live_input_observe_only",
            )),
        },
    }
}

fn observe_trace(stage: InputGateStage, reason: &'static str) -> InputGateDecisionTrace {
    InputGateDecisionTrace {
        stage,
        input_class: None,
        candidate_count: 0,
        selected_source: None,
        selected_source_id: None,
        selected_error_class: None,
        selected_gate_action: None,
        reason,
    }
}

fn word_boundary_trace(resolution: &CorrectionResolution) -> InputGateDecisionTrace {
    let selected = resolution.selected.as_ref();
    InputGateDecisionTrace {
        stage: InputGateStage::WordBoundary,
        input_class: Some(resolution.event.input_class),
        candidate_count: resolution.candidates.len(),
        selected_source: selected.map(|candidate| candidate.source),
        selected_source_id: selected.map(|candidate| candidate.source_id.clone()),
        selected_error_class: selected.map(|candidate| candidate.error_class),
        selected_gate_action: selected.map(|candidate| candidate.gate.action),
        reason: word_boundary_trace_reason(resolution),
    }
}

fn word_boundary_trace_reason(resolution: &CorrectionResolution) -> &'static str {
    if resolution.decision.is_some() {
        return "apply_selected_candidate";
    }
    if resolution
        .candidates
        .iter()
        .any(|candidate| candidate.gate.action == CandidateGateAction::SuggestOnly)
    {
        return "suggest_only_candidate";
    }
    if resolution
        .candidates
        .iter()
        .any(|candidate| candidate.gate.action == CandidateGateAction::Veto)
    {
        return "candidate_veto";
    }
    "no_candidate"
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
        assert_eq!(
            decision.trace.as_ref().map(|trace| trace.reason),
            Some("live_input_observe_only")
        );
    }

    #[test]
    fn space_boundary_applies_existing_correction_core_decision() {
        let pipeline = default_typing_assist_pipeline();
        let decision = decide_input_gate(InputGateRequest {
            trigger: InputGateTrigger::Space,
            text_tail: "lfdfq ",
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: &pipeline,
            nanda_autocorrect: false,
            correction_mode: CorrectionMode::DeterministicOnly,
        });
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
        let trace = decision.trace.as_ref().expect("input gate trace");
        assert_eq!(trace.candidate_count, 1);
        assert_eq!(
            trace.selected_source,
            Some(CorrectionDecisionSource::Deterministic)
        );
        assert_eq!(
            trace.selected_error_class,
            Some(TypingErrorClass::WrongLayout)
        );
        assert_eq!(trace.selected_gate_action, Some(CandidateGateAction::Apply));
        assert_eq!(trace.reason, "apply_selected_candidate");
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
        let trace = decision.trace.as_ref().expect("input gate trace");
        assert_eq!(trace.reason, "no_candidate");
        assert_eq!(trace.selected_source, None);
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
        assert_eq!(
            decision.trace.as_ref().map(|trace| trace.reason),
            Some("no_candidate")
        );
    }
}
