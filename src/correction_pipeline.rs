// Internal canonical text correction pipeline contract.
//
// This file is included inside `input_gate.rs`, so its entrypoint stays private
// to the existing public input gate. It does not change runtime/IME output
// behavior: it only makes Space/Enter flow through the staged pipeline contract.

use crate::correction_core::UnifiedCorrectionCandidate;
use crate::nanda_wave::{run_wave_trace, WaveDecision, WaveTrace};

/// Canonical after-space autocorrection entrypoint for runtime callers.
///
/// This intentionally exposes only the existing `InputGateRequest` /
/// `InputGateDecision` contract. Pipeline internals stay private until each
/// runtime route is migrated and proven separately.
fn decide_space_autocorrect(req: InputGateRequest<'_>) -> InputGateDecision {
    CanonicalTextPipeline::decide(PipelineRequest {
        snapshot: TailSnapshot::new(req.text_tail, req.trigger, TailSnapshotSource::Daemon),
        auto_replace: req.auto_replace,
        typing_assist: req.typing_assist,
        auto_switch_layout: req.auto_switch_layout,
        correction_safety: req.correction_safety,
        typing_assist_pipeline: req.typing_assist_pipeline,
        nanda_autocorrect: req.nanda_autocorrect,
        correction_mode: req.correction_mode,
        include_l3_report: false,
    })
    .input_gate
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum TailSnapshotSource {
    Daemon,
    Ime,
    Cli,
    Test,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TailSnapshot {
    text: String,
    trigger: InputGateTrigger,
    source: TailSnapshotSource,
    cursor: Option<usize>,
}

impl TailSnapshot {
    fn new(text: impl Into<String>, trigger: InputGateTrigger, source: TailSnapshotSource) -> Self {
        Self {
            text: text.into(),
            trigger,
            source,
            cursor: None,
        }
    }

    #[allow(dead_code)]
    fn with_cursor(mut self, cursor: usize) -> Self {
        self.cursor = Some(cursor);
        self
    }

    #[allow(dead_code)]
    fn active_word(&self) -> Option<&str> {
        self.text.split_whitespace().next_back()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct CandidateSet {
    original_tail: String,
    candidates: Vec<PipelineCandidate>,
}

impl CandidateSet {
    fn from_resolution(resolution: &CorrectionResolution) -> Self {
        Self {
            original_tail: resolution.event.original.clone(),
            candidates: resolution
                .candidates
                .iter()
                .map(PipelineCandidate::from_unified)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct PipelineCandidate {
    replacement: String,
    source: CorrectionDecisionSource,
    source_id: String,
    error_class: TypingErrorClass,
    gate_action: CandidateGateAction,
    gate_reason: &'static str,
}

impl PipelineCandidate {
    fn from_unified(candidate: &UnifiedCorrectionCandidate) -> Self {
        Self {
            replacement: candidate.replacement.clone(),
            source: candidate.source,
            source_id: candidate.source_id.clone(),
            error_class: candidate.error_class,
            gate_action: candidate.gate.action,
            gate_reason: candidate.gate.reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct GateDecision {
    action: InputGateAction,
    selected_source: Option<CorrectionDecisionSource>,
    selected_source_id: Option<String>,
    selected_error_class: Option<TypingErrorClass>,
}

impl GateDecision {
    fn from_input_gate(decision: &InputGateDecision) -> Self {
        let selected = decision
            .correction
            .as_ref()
            .and_then(|resolution| resolution.selected.as_ref());
        Self {
            action: decision.action.clone(),
            selected_source: selected.map(|candidate| candidate.source),
            selected_source_id: selected.map(|candidate| candidate.source_id.clone()),
            selected_error_class: selected.map(|candidate| candidate.error_class),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EditPlan {
    original: String,
    replacement: String,
    apply_space: bool,
}

impl EditPlan {
    fn from_gate(snapshot: &TailSnapshot, gate: &GateDecision) -> Option<Self> {
        let InputGateAction::ApplyReplacement { replacement, .. } = &gate.action else {
            return None;
        };
        Some(Self {
            original: snapshot.text.clone(),
            replacement: replacement.clone(),
            apply_space: snapshot.trigger.closes_word(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ArbitrationReport {
    snapshot: TailSnapshot,
    candidate_set: CandidateSet,
    l3_report: Option<L3ArbitrationReport>,
    gate: GateDecision,
    edit_plan: Option<EditPlan>,
    input_gate: InputGateDecision,
}

#[derive(Debug, Clone)]
struct PipelineRequest<'a> {
    snapshot: TailSnapshot,
    auto_replace: bool,
    typing_assist: bool,
    auto_switch_layout: bool,
    correction_safety: CorrectionSafety,
    typing_assist_pipeline: &'a [TypingAssistRuleConfig],
    nanda_autocorrect: bool,
    correction_mode: CorrectionMode,
    include_l3_report: bool,
}

#[allow(dead_code)]
trait OutputBackend {
    type Error;

    fn apply_edit_plan(&mut self, plan: &EditPlan) -> Result<(), Self::Error>;
}

struct ErrorGate;

impl ErrorGate {
    fn decide(req: &PipelineRequest<'_>) -> InputGateDecision {
        decide_space_autocorrect_gate(InputGateRequest {
            trigger: req.snapshot.trigger,
            text_tail: &req.snapshot.text,
            auto_replace: req.auto_replace,
            typing_assist: req.typing_assist,
            auto_switch_layout: req.auto_switch_layout,
            correction_safety: req.correction_safety,
            typing_assist_pipeline: req.typing_assist_pipeline,
            nanda_autocorrect: req.nanda_autocorrect,
            correction_mode: req.correction_mode,
        })
    }
}

fn decide_space_autocorrect_gate(req: InputGateRequest<'_>) -> InputGateDecision {
    let resolution = crate::correction_core::resolve_text_correction(CorrectionRequest {
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
        trace: Some(word_boundary_trace(&resolution)),
        correction: Some(resolution),
    }
}

#[derive(Debug, Clone, PartialEq)]
struct L3ArbitrationReport {
    decision: L3DecisionKind,
    output: Option<String>,
    trace_len: usize,
}

impl L3ArbitrationReport {
    fn from_wave_trace(trace: &WaveTrace) -> Self {
        let (decision, output) = match &trace.decision {
            WaveDecision::Apply { text, .. } => {
                (L3DecisionKind::ApplyCandidate, Some(text.clone()))
            }
            WaveDecision::Keep { .. } => (L3DecisionKind::Keep, None),
            WaveDecision::Veto { .. } => (L3DecisionKind::Veto, None),
        };
        Self {
            decision,
            output,
            trace_len: trace.l3.len(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum L3DecisionKind {
    ApplyCandidate,
    Keep,
    Veto,
}

struct CanonicalTextPipeline;

impl CanonicalTextPipeline {
    fn decide(req: PipelineRequest<'_>) -> ArbitrationReport {
        let input_gate = ErrorGate::decide(&req);
        let candidate_set = input_gate
            .correction
            .as_ref()
            .map(CandidateSet::from_resolution)
            .unwrap_or_else(|| CandidateSet {
                original_tail: req.snapshot.text.clone(),
                candidates: Vec::new(),
            });
        let gate = GateDecision::from_input_gate(&input_gate);
        let edit_plan = EditPlan::from_gate(&req.snapshot, &gate);
        let l3_report = req
            .include_l3_report
            .then(|| run_wave_trace(&req.snapshot.text))
            .map(|trace| L3ArbitrationReport::from_wave_trace(&trace));

        ArbitrationReport {
            snapshot: req.snapshot,
            candidate_set,
            l3_report,
            gate,
            edit_plan,
            input_gate,
        }
    }
}

#[cfg(test)]
mod correction_pipeline_tests {
    use super::*;
    use crate::config::{default_typing_assist_pipeline, CorrectionSafety, TypingAssistRuleConfig};
    use crate::correction_core::CorrectionMode;
    use crate::input_gate::{InputGateRequest, InputGateTrigger};

    fn request<'a>(text: &str, pipeline: &'a [TypingAssistRuleConfig]) -> PipelineRequest<'a> {
        PipelineRequest {
            snapshot: TailSnapshot::new(text, InputGateTrigger::Space, TailSnapshotSource::Test),
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: pipeline,
            nanda_autocorrect: true,
            correction_mode: CorrectionMode::DeterministicThenNanda,
            include_l3_report: false,
        }
    }

    fn public_input_gate<'a>(
        text: &'a str,
        pipeline: &'a [TypingAssistRuleConfig],
    ) -> InputGateDecision {
        crate::input_gate::decide_input_gate(InputGateRequest {
            trigger: InputGateTrigger::Space,
            text_tail: text,
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: pipeline,
            nanda_autocorrect: true,
            correction_mode: CorrectionMode::DeterministicThenNanda,
        })
    }

    fn public_space_gate<'a>(
        text: &'a str,
        pipeline: &'a [TypingAssistRuleConfig],
    ) -> InputGateDecision {
        decide_space_autocorrect(InputGateRequest {
            trigger: InputGateTrigger::Space,
            text_tail: text,
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: pipeline,
            nanda_autocorrect: true,
            correction_mode: CorrectionMode::DeterministicThenNanda,
        })
    }

    #[test]
    fn pipeline_builds_candidate_set_and_edit_plan() {
        let pipeline = default_typing_assist_pipeline();
        let report = CanonicalTextPipeline::decide(request("читай логии ", &pipeline));

        assert!(!report.candidate_set.candidates.is_empty());
        assert_eq!(
            report
                .edit_plan
                .as_ref()
                .map(|plan| plan.replacement.as_str()),
            Some("читай логи ")
        );
    }

    #[test]
    fn pipeline_keeps_l3_as_report_not_output_backend() {
        let pipeline = default_typing_assist_pipeline();
        let report = CanonicalTextPipeline::decide(PipelineRequest {
            include_l3_report: true,
            ..request("она спраивтя ", &pipeline)
        });

        assert!(report.l3_report.is_some());
        assert!(report.edit_plan.is_none());
    }

    #[test]
    fn pipeline_matches_public_input_gate_on_dirty_tails() {
        let pipeline = default_typing_assist_pipeline();
        let cases = [
            "читай логии ",
            "пукнут ",
            "звгрузи ",
            "не посчетал ",
            "ябыл ",
            "ghbdtn ",
            "gkfvz ",
            "fавтозамена ",
        ];

        for text in cases {
            let report = CanonicalTextPipeline::decide(request(text, &pipeline));
            let public = public_space_gate(text, &pipeline);
            let public_input_gate = public_input_gate(text, &pipeline);

            assert_eq!(report.input_gate.action, public.action, "{text:?}");
            assert_eq!(report.input_gate.trace, public.trace, "{text:?}");
            assert_eq!(public_input_gate.action, public.action, "{text:?}");
            assert_eq!(public_input_gate.trace, public.trace, "{text:?}");
            assert_eq!(
                report.input_gate.correction.as_ref().map(|resolution| {
                    (
                        resolution.selected.as_ref().map(|candidate| {
                            (
                                candidate.replacement.as_str(),
                                candidate.source,
                                candidate.error_class,
                            )
                        }),
                        resolution.candidates.len(),
                    )
                }),
                public.correction.as_ref().map(|resolution| {
                    (
                        resolution.selected.as_ref().map(|candidate| {
                            (
                                candidate.replacement.as_str(),
                                candidate.source,
                                candidate.error_class,
                            )
                        }),
                        resolution.candidates.len(),
                    )
                }),
                "{text:?}"
            );
        }
    }
}
