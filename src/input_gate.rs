//! Unified input gate.
//!
//! This module is the stream-level owner for input decisions. Candidate
//! generators and correction engines may propose, but callers should route live
//! input events through this gate before applying or showing anything.

use crate::config::{CorrectionSafety, TypingAssistRuleConfig};
use crate::correction_core::{
    CandidateGateAction, CorrectionCandidateScoreTrace, CorrectionDecisionSource, CorrectionMode,
    CorrectionRequest, CorrectionResolution, CorrectionScoreboard, TypingErrorClass,
};
use crate::nanda_wave::WaveOptions;

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
    pub scoreboard: InputGateScoreboard,
    pub(crate) candidate_scores: Vec<InputGateCandidateScoreTrace>,
    pub selected_source: Option<CorrectionDecisionSource>,
    pub selected_source_id: Option<String>,
    pub selected_error_class: Option<TypingErrorClass>,
    pub selected_gate_action: Option<CandidateGateAction>,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InputGateCandidateScoreTrace {
    pub(crate) replacement: String,
    pub(crate) source: CorrectionDecisionSource,
    pub(crate) source_id: String,
    pub(crate) error_class: TypingErrorClass,
    pub(crate) action_operator: &'static str,
    pub(crate) action_proof: &'static str,
    pub(crate) edit_transition_operator: &'static str,
    pub(crate) edit_transition_proof: &'static str,
    pub(crate) edit_transition_operator_kind: crate::text_edit::TransitionOperator,
    pub(crate) edit_transition_proof_kind: crate::text_edit::TransitionProof,
    pub(crate) edit_transition_verified: bool,
    pub(crate) edit_transition_left_context_changed: bool,
    pub(crate) edit_transition_changed_tokens: usize,
    pub(crate) edit_shape: &'static str,
    pub(crate) preservation_milli: i16,
    pub(crate) lost_mass_milli: i16,
    pub(crate) added_mass_milli: i16,
    pub(crate) operator_fit_milli: i16,
    pub(crate) shortcut_risk_milli: i16,
    pub(crate) anti_wave_milli: i16,
    pub(crate) explanation_score_milli: i16,
    pub(crate) gate_action: CandidateGateAction,
    pub(crate) gate_reason: &'static str,
    pub(crate) likelihood_milli: i16,
    pub(crate) usage_prior_milli: i16,
    pub(crate) context_prior_milli: i16,
    pub(crate) l2_wave_peak_milli: i16,
    pub(crate) l2_wave_peak_positive_milli: i16,
    pub(crate) l2_wave_peak_negative_milli: i16,
    pub(crate) l2_wave_peak_uncertainty_milli: i16,
    pub(crate) l2_wave_peak_reason: &'static str,
    pub(crate) l2_transition_phase_milli: i16,
    pub(crate) l2_transition_phase_threshold_milli: i16,
    pub(crate) l2_transition_phase_verdict: &'static str,
    pub(crate) l2_transition_phase_package_loaded: bool,
    pub(crate) l2_transition_phase_operator_present: bool,
    pub(crate) l2_transition_phase_operator_promoted: bool,
    pub(crate) l2_transition_phase_positive_centers: u8,
    pub(crate) l2_transition_phase_anti_centers: u8,
    pub(crate) l2_transition_phase_surfaces: u32,
    pub(crate) l3_phrase_milli: i16,
    pub(crate) l3_phrase_decision: &'static str,
    pub(crate) l4_scene_milli: i16,
    pub(crate) l4_scene_action: &'static str,
    pub(crate) l4_scene_reason: &'static str,
    pub(crate) l4_signed_milli: i16,
    pub(crate) l4_signed_reason: &'static str,
    pub(crate) l4_surface_status: &'static str,
    pub(crate) l4_transition_state_specific: bool,
    pub(crate) l4_transition_attract_count: u32,
    pub(crate) l4_transition_repel_count: u32,
    pub(crate) risk_milli: i16,
    pub(crate) posterior_milli: i16,
    pub(crate) decision_rank_milli: i16,
    pub(crate) selected: bool,
}

impl From<&CorrectionCandidateScoreTrace> for InputGateCandidateScoreTrace {
    fn from(score: &CorrectionCandidateScoreTrace) -> Self {
        Self {
            replacement: score.replacement.clone(),
            source: score.source,
            source_id: score.source_id.clone(),
            error_class: score.error_class,
            action_operator: score.action_operator,
            action_proof: score.action_proof,
            edit_transition_operator: score.edit_transition_operator,
            edit_transition_proof: score.edit_transition_proof,
            edit_transition_operator_kind: score.edit_transition_operator_kind,
            edit_transition_proof_kind: score.edit_transition_proof_kind,
            edit_transition_verified: score.edit_transition_verified,
            edit_transition_left_context_changed: score.edit_transition_left_context_changed,
            edit_transition_changed_tokens: score.edit_transition_changed_tokens,
            edit_shape: score.edit_shape,
            preservation_milli: score.preservation_milli,
            lost_mass_milli: score.lost_mass_milli,
            added_mass_milli: score.added_mass_milli,
            operator_fit_milli: score.operator_fit_milli,
            shortcut_risk_milli: score.shortcut_risk_milli,
            anti_wave_milli: score.anti_wave_milli,
            explanation_score_milli: score.explanation_score_milli,
            gate_action: score.gate_action,
            gate_reason: score.gate_reason,
            likelihood_milli: score.likelihood_milli,
            usage_prior_milli: score.usage_prior_milli,
            context_prior_milli: score.context_prior_milli,
            l2_wave_peak_milli: score.l2_wave_peak_milli,
            l2_wave_peak_positive_milli: score.l2_wave_peak_positive_milli,
            l2_wave_peak_negative_milli: score.l2_wave_peak_negative_milli,
            l2_wave_peak_uncertainty_milli: score.l2_wave_peak_uncertainty_milli,
            l2_wave_peak_reason: score.l2_wave_peak_reason,
            l2_transition_phase_milli: score.l2_transition_phase_milli,
            l2_transition_phase_threshold_milli: score.l2_transition_phase_threshold_milli,
            l2_transition_phase_verdict: score.l2_transition_phase_verdict,
            l2_transition_phase_package_loaded: score.l2_transition_phase_package_loaded,
            l2_transition_phase_operator_present: score.l2_transition_phase_operator_present,
            l2_transition_phase_operator_promoted: score.l2_transition_phase_operator_promoted,
            l2_transition_phase_positive_centers: score.l2_transition_phase_positive_centers,
            l2_transition_phase_anti_centers: score.l2_transition_phase_anti_centers,
            l2_transition_phase_surfaces: score.l2_transition_phase_surfaces,
            l3_phrase_milli: score.l3_phrase_milli,
            l3_phrase_decision: score.l3_phrase_decision,
            l4_scene_milli: score.l4_scene_milli,
            l4_scene_action: score.l4_scene_action,
            l4_scene_reason: score.l4_scene_reason,
            l4_signed_milli: score.l4_signed_milli,
            l4_signed_reason: score.l4_signed_reason,
            l4_surface_status: score.l4_surface_status,
            l4_transition_state_specific: score.l4_transition_state_specific,
            l4_transition_attract_count: score.l4_transition_attract_count,
            l4_transition_repel_count: score.l4_transition_repel_count,
            risk_milli: score.risk_milli,
            posterior_milli: score.posterior_milli,
            decision_rank_milli: score.decision_rank_milli,
            selected: score.selected,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InputGateScoreboard {
    pub total_candidates: usize,
    pub apply_candidates: usize,
    pub suggest_only_candidates: usize,
    pub keep_original_candidates: usize,
    pub veto_candidates: usize,
    pub deterministic_candidates: usize,
    pub nanda_candidates: usize,
    pub selected_bayes_posterior_milli: Option<i16>,
}

impl From<CorrectionScoreboard> for InputGateScoreboard {
    fn from(scoreboard: CorrectionScoreboard) -> Self {
        Self {
            total_candidates: scoreboard.total_candidates,
            apply_candidates: scoreboard.apply_candidates,
            suggest_only_candidates: scoreboard.suggest_only_candidates,
            keep_original_candidates: scoreboard.keep_original_candidates,
            veto_candidates: scoreboard.veto_candidates,
            deterministic_candidates: scoreboard.deterministic_candidates,
            nanda_candidates: scoreboard.nanda_candidates,
            selected_bayes_posterior_milli: scoreboard.selected_bayes_posterior_milli,
        }
    }
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
    pub nanda_wave_options: WaveOptions,
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
        scoreboard: resolution.scoreboard.into(),
        candidate_scores: resolution
            .candidate_scores
            .iter()
            .map(InputGateCandidateScoreTrace::from)
            .collect(),
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
            nanda_wave_options: WaveOptions::default(),
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
        assert_eq!(trace.candidate_count, 1);
        assert_eq!(trace.scoreboard.apply_candidates, 1);
        assert_eq!(trace.scoreboard.deterministic_candidates, 1);
        assert_eq!(trace.scoreboard.nanda_candidates, 0);
        assert!(
            trace.scoreboard.selected_bayes_posterior_milli.is_some(),
            "selected candidate should expose Bayes posterior in the input gate scoreboard"
        );
        assert_eq!(trace.candidate_scores.len(), 1);
        let score = &trace.candidate_scores[0];
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
            trace.selected_gate_action,
            Some(CandidateGateAction::Eligible)
        );
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
            nanda_wave_options: WaveOptions::default(),
            correction_mode: CorrectionMode::DeterministicThenNanda,
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
