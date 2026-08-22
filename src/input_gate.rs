//! Unified input gate.
//!
//! This module is the stream-level owner for input decisions. Candidate
//! generators and correction engines may propose, but callers should route live
//! input events through this gate before applying or showing anything.

use crate::config::{CorrectionSafety, TypingAssistRuleConfig};
use crate::correction_core::{
    CandidateGateAction, CandidateReadoutRoute, CorrectionCandidateScoreTrace,
    CorrectionDecisionSource, CorrectionMode, CorrectionRequest, CorrectionResolution,
    CorrectionScoreboard, TypingErrorClass,
};
use crate::nanda_wave::WaveOptions;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputGateOutcome {
    Observe,
    KeepOriginal,
    Apply,
    SuggestOnly,
    Veto,
}

impl InputGateAction {
    fn outcome(&self) -> InputGateOutcome {
        match self {
            Self::Observe => InputGateOutcome::Observe,
            Self::KeepOriginal => InputGateOutcome::KeepOriginal,
            Self::ApplyReplacement { .. } => InputGateOutcome::Apply,
            Self::SuggestOnly { .. } => InputGateOutcome::SuggestOnly,
            Self::Veto { .. } => InputGateOutcome::Veto,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputGateDecision {
    pub trigger: InputGateTrigger,
    pub stage: InputGateStage,
    pub action: InputGateAction,
    pub correction: Option<CorrectionResolution>,
    pub trace: Option<InputGateDecisionTrace>,
}

pub(crate) struct ObservedInputGateDecision {
    pub(crate) decision: InputGateDecision,
    pub(crate) telemetry: crate::correction_core::CorrectionRouteTelemetry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputGateDecisionTrace {
    pub stage: InputGateStage,
    pub input_class: Option<TypingErrorClass>,
    pub candidate_count: usize,
    pub scoreboard: InputGateScoreboard,
    pub(crate) candidate_scores: Vec<InputGateCandidateScoreTrace>,
    pub selected_source: Option<CorrectionDecisionSource>,
    pub selected_source_id: Option<String>,
    pub selected_error_class: Option<TypingErrorClass>,
    pub(crate) selected_candidate_gate_action: Option<CandidateGateAction>,
    pub(crate) outcome: InputGateOutcome,
    pub reason: &'static str,
}

pub(crate) type InputGateCandidateScoreTrace = CorrectionCandidateScoreTrace;
pub type InputGateScoreboard = CorrectionScoreboard;

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
    pub nanda_candidate_route: CandidateReadoutRoute,
    pub nanda_wave_options: WaveOptions,
    pub correction_mode: CorrectionMode,
}

pub fn decide_input_gate(req: InputGateRequest<'_>) -> InputGateDecision {
    decide_input_gate_observed(req).decision
}

pub(crate) fn decide_input_gate_observed(req: InputGateRequest<'_>) -> ObservedInputGateDecision {
    if matches!(
        req.trigger,
        InputGateTrigger::Space | InputGateTrigger::Enter
    ) {
        return decide_space_autocorrect_observed(req);
    }
    let decision = match req.trigger {
        InputGateTrigger::Space | InputGateTrigger::Enter => unreachable!("handled above"),
        InputGateTrigger::DoubleShift => InputGateDecision {
            trigger: req.trigger,
            stage: InputGateStage::ManualToggle,
            action: InputGateAction::Observe,
            correction: None,
            trace: Some(observe_trace(
                InputGateStage::ManualToggle,
                "manual_toggle_route_observed",
            )),
        },
        InputGateTrigger::TabAccept => InputGateDecision {
            trigger: req.trigger,
            stage: InputGateStage::CompletionAccept,
            action: InputGateAction::Observe,
            correction: None,
            trace: Some(observe_trace(
                InputGateStage::CompletionAccept,
                "completion_accept_route_observed",
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
    };
    ObservedInputGateDecision {
        decision,
        telemetry: crate::correction_core::CorrectionRouteTelemetry::default(),
    }
}

pub(crate) fn decide_input_gate_observed_with_exact(
    req: InputGateRequest<'_>,
    certificate: &crate::exact_layout_authority::ExactLayoutContourCertificate,
) -> ObservedInputGateDecision {
    if !matches!(
        req.trigger,
        InputGateTrigger::Space | InputGateTrigger::Enter
    ) {
        return decide_input_gate_observed(req);
    }
    decide_space_autocorrect_observed_internal(
        req,
        SpaceCorrectionEvidence::FullField(Some(certificate)),
    )
}

pub(crate) fn decide_closed_exact_input_gate_observed(
    req: InputGateRequest<'_>,
    certificate: Option<&crate::exact_layout_authority::ExactLayoutContourCertificate>,
) -> ObservedInputGateDecision {
    if !matches!(
        req.trigger,
        InputGateTrigger::Space | InputGateTrigger::Enter
    ) {
        return decide_input_gate_observed(req);
    }
    decide_space_autocorrect_observed_internal(
        req,
        SpaceCorrectionEvidence::ClosedExact(certificate),
    )
}

fn decide_space_autocorrect_observed(req: InputGateRequest<'_>) -> ObservedInputGateDecision {
    decide_space_autocorrect_observed_internal(req, SpaceCorrectionEvidence::FullField(None))
}

#[derive(Clone, Copy)]
enum SpaceCorrectionEvidence<'a> {
    FullField(Option<&'a crate::exact_layout_authority::ExactLayoutContourCertificate>),
    ClosedExact(Option<&'a crate::exact_layout_authority::ExactLayoutContourCertificate>),
}

fn decide_space_autocorrect_observed_internal(
    req: InputGateRequest<'_>,
    evidence: SpaceCorrectionEvidence<'_>,
) -> ObservedInputGateDecision {
    let correction_request = CorrectionRequest {
        text: req.text_tail,
        auto_replace: req.auto_replace,
        typing_assist: req.typing_assist,
        auto_switch_layout: req.auto_switch_layout,
        correction_safety: req.correction_safety,
        typing_assist_pipeline: req.typing_assist_pipeline,
        nanda_autocorrect: req.nanda_autocorrect,
        nanda_candidate_route: req.nanda_candidate_route,
        nanda_wave_options: req.nanda_wave_options,
        mode: req.correction_mode,
    };
    let observed = match evidence {
        SpaceCorrectionEvidence::FullField(None) => {
            crate::correction_core::resolve_text_correction_observed(correction_request)
        }
        SpaceCorrectionEvidence::FullField(Some(certificate)) => {
            crate::correction_core::resolve_text_correction_observed_with_exact(
                correction_request,
                certificate,
            )
        }
        SpaceCorrectionEvidence::ClosedExact(certificate) => {
            crate::correction_core::resolve_closed_exact_text_correction_observed(
                correction_request,
                certificate,
            )
        }
    };
    let resolution = observed.resolution;
    let action = word_boundary_action(&resolution);

    ObservedInputGateDecision {
        decision: InputGateDecision {
            trigger: req.trigger,
            stage: InputGateStage::WordBoundary,
            trace: Some(word_boundary_trace(&resolution, action.outcome())),
            action,
            correction: Some(resolution),
        },
        telemetry: observed.telemetry,
    }
}

/// Builds lazy candidate state through the same boundary route used at runtime.
/// The results are discarded; this grants no candidate apply authority.
pub(crate) fn warm_up_word_boundary() {
    let pipeline = crate::config::default_typing_assist_pipeline();
    for text_tail in ["руддщ ", "проврека "] {
        let _ = decide_input_gate(InputGateRequest {
            trigger: InputGateTrigger::Space,
            text_tail,
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: &pipeline,
            nanda_autocorrect: false,
            nanda_candidate_route: CandidateReadoutRoute::live_default(),
            nanda_wave_options: WaveOptions::default(),
            correction_mode: CorrectionMode::DeterministicOnly,
        });
    }
}

fn observe_trace(stage: InputGateStage, reason: &'static str) -> InputGateDecisionTrace {
    InputGateDecisionTrace {
        stage,
        input_class: None,
        candidate_count: 0,
        scoreboard: InputGateScoreboard::default(),
        candidate_scores: Vec::new(),
        selected_source: None,
        selected_source_id: None,
        selected_error_class: None,
        selected_candidate_gate_action: None,
        outcome: InputGateOutcome::Observe,
        reason,
    }
}

fn word_boundary_trace(
    resolution: &CorrectionResolution,
    outcome: InputGateOutcome,
) -> InputGateDecisionTrace {
    let selected = resolution.selected.as_ref();
    InputGateDecisionTrace {
        stage: InputGateStage::WordBoundary,
        input_class: Some(resolution.event.input_class),
        candidate_count: resolution.candidates.len(),
        scoreboard: resolution.scoreboard,
        candidate_scores: resolution.candidate_scores.to_vec(),
        selected_source: selected.map(|candidate| candidate.source),
        selected_source_id: selected.map(|candidate| candidate.source_id.clone()),
        selected_error_class: selected.map(|candidate| candidate.error_class),
        selected_candidate_gate_action: selected.map(|candidate| candidate.gate.action),
        outcome,
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
        .any(|candidate| candidate.gate.action == CandidateGateAction::KeepOriginal)
    {
        return "keep_original_candidate";
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
    let mut keep_original = false;
    let mut veto_reason = None;
    for candidate in &resolution.candidates {
        match candidate.gate.action {
            CandidateGateAction::KeepOriginal => {
                keep_original = true;
            }
            CandidateGateAction::Eligible | CandidateGateAction::SuggestOnly => {
                best_suggestion.get_or_insert_with(|| candidate.replacement.clone());
            }
            CandidateGateAction::Veto => {
                veto_reason.get_or_insert(candidate.gate.reason);
            }
        }
    }

    if keep_original {
        InputGateAction::KeepOriginal
    } else if best_suggestion.is_some() {
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
            nanda_candidate_route: CandidateReadoutRoute::FullWave,
            nanda_wave_options: WaveOptions::default(),
            correction_mode: CorrectionMode::NandaOnly,
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
    fn double_shift_is_visible_as_manual_toggle_operator() {
        let decision = decide_input_gate(request(InputGateTrigger::DoubleShift, "ghbdtn"));
        assert_eq!(decision.stage, InputGateStage::ManualToggle);
        assert_eq!(decision.action, InputGateAction::Observe);
        assert!(decision.correction.is_none());
        let trace = decision.trace.as_ref().expect("manual toggle trace");
        assert_eq!(trace.reason, "manual_toggle_route_observed");
    }

    #[test]
    fn tab_accept_is_visible_as_completion_operator() {
        let decision = decide_input_gate(request(InputGateTrigger::TabAccept, "пров"));
        assert_eq!(decision.stage, InputGateStage::CompletionAccept);
        assert_eq!(decision.action, InputGateAction::Observe);
        let trace = decision.trace.as_ref().expect("completion trace");
        assert_eq!(trace.reason, "completion_accept_route_observed");
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
            nanda_candidate_route: CandidateReadoutRoute::FullWave,
            nanda_wave_options: WaveOptions::default(),
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
        assert!(trace.candidate_count >= 1);
        assert_eq!(trace.scoreboard.apply_candidates, 1);
        assert!(trace.scoreboard.deterministic_candidates >= 1);
        assert_eq!(trace.scoreboard.nanda_candidates, 0);
        assert!(
            trace.scoreboard.selected_bayes_posterior_milli.is_some(),
            "selected candidate should expose Bayes posterior in the input gate scoreboard"
        );
        assert_eq!(trace.candidate_scores.len(), trace.candidate_count);
        let score = trace
            .candidate_scores
            .iter()
            .find(|score| score.selected)
            .expect("selected candidate score");
        assert_eq!(score.replacement, "давай ");
        assert_eq!(score.source, CorrectionDecisionSource::Deterministic);
        assert_eq!(score.error_class, TypingErrorClass::WrongLayout);
        assert_eq!(score.gate_action, CandidateGateAction::Eligible);
        assert!(score.selected);
        assert!(score.posterior_milli > 0);
        assert!(score.decision_rank_milli > 0);
        assert_eq!(score.l4_scene_action, "suggest");
        assert!(score.l4_scene_milli > 0);
        assert_eq!(
            trace.selected_source,
            Some(CorrectionDecisionSource::Deterministic)
        );
        assert_eq!(
            trace.selected_error_class,
            Some(TypingErrorClass::WrongLayout)
        );
        assert_eq!(
            trace.selected_candidate_gate_action,
            Some(CandidateGateAction::Eligible)
        );
        assert_eq!(trace.outcome, InputGateOutcome::Apply);
        assert_eq!(trace.reason, "apply_selected_candidate");
    }

    #[test]
    fn word_boundary_does_not_apply_after_safety_gate() {
        let pipeline = default_typing_assist_pipeline();
        let decision = decide_input_gate(request_with_pipeline(
            InputGateTrigger::Space,
            "патерна ",
            &pipeline,
        ));
        assert!(matches!(
            decision.action,
            InputGateAction::KeepOriginal | InputGateAction::SuggestOnly { .. }
        ));
        let trace = decision.trace.as_ref().expect("input gate trace");
        assert_ne!(trace.reason, "apply_selected_candidate");
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
            nanda_candidate_route: CandidateReadoutRoute::FullWave,
            nanda_wave_options: WaveOptions::default(),
            correction_mode: CorrectionMode::DeterministicOnly,
        });
        assert_eq!(decision.action, InputGateAction::KeepOriginal);
        assert_eq!(
            decision.trace.as_ref().map(|trace| trace.reason),
            Some("no_candidate")
        );
        assert_eq!(
            decision
                .trace
                .as_ref()
                .map(|trace| trace.scoreboard.apply_candidates),
            Some(0)
        );
    }
}
