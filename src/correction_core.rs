//! Shared text-correction decision facade.
//!
//! Runtime backends still own output and state. This module only answers one
//! question: should this completed text be replaced, and by which engine?

use crate::candidate_explanation::{explain_candidate, CandidateExplanation};
use crate::config::{CorrectionSafety, TypingAssistRuleConfig};
use crate::correction_source_contract::{self, CandidateOrigin, CorrectionSourceRole};
use crate::nanda_wave::l3_phrase_gate::{evaluate_default_candidate, L3PhraseGateDecision};
use crate::nanda_wave::{run_wave_trace_with_options, WaveOptions, WordCandidate};
use crate::russian_typo_candidates::{
    inserted_char_position_for_missing_letter, repeated_run_deletion_candidates,
};
use crate::text_case::apply_word_case;
use crate::text_metrics::{damerau_levenshtein, has_cyrillic};
use crate::typing_assist::split_ws_segments;
use crate::typing_context::{syntax_allows_candidate, typing_assist_pipeline_for_context};
use crate::typing_pipeline::{
    collect_typing_assist_candidates_with_pipeline, explain_typing_assist_with_pipeline,
};
use crate::typing_rule_graph::ids;
use crate::typing_transition::{
    action as action_operator, candidate::L2CandidateLattice, decision::candidate_decision_signals,
    state::L1SurfaceSignal, verifier as edit_transition,
};
use crate::word_reader::{
    cyrillic_word_splits, is_cyrillic_letters_only, last_text_word, replace_last_text_word,
    split_edge_whitespace, split_word_punctuation,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

const COMPOSITE_TRANSPOSE_MIN_MARGIN: f64 = -8.0;
const REPEATED_DELETE_SURFACE_MARGIN: f64 = 0.25;

#[path = "correction_core/gate.rs"]
mod gate;
#[cfg(test)]
use gate::gate_candidate;
pub(crate) use gate::{bayes_score_for_candidate, normalized_correction_words};
use gate::{
    gate_candidate_with_source, repeated_deletion_has_surface_support,
    should_prefer_composite_after_repeated_repair,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypingErrorClass {
    WrongLayout,
    PartialLayout,
    MixedScript,
    MissingLetter,
    ExtraLetter,
    RepeatedLetter,
    AdjacentTransposition,
    LetterSubstitution,
    CompositeTypo,
    BoundaryShift,
    SplitWord,
    GluedWords,
    CaseNoise,
    GrammarAgreement,
    CompletionOnly,
    TechnicalToken,
    ProtectedToken,
    Unknown,
}

impl TypingErrorClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WrongLayout => "wrong_layout",
            Self::PartialLayout => "partial-layout",
            Self::MixedScript => "mixed-script",
            Self::MissingLetter => "missing-letter",
            Self::ExtraLetter => "extra-letter",
            Self::RepeatedLetter => "repeated-letter",
            Self::AdjacentTransposition => "adjacent-transposition",
            Self::LetterSubstitution => "letter-substitution",
            Self::CompositeTypo => "composite-typo",
            Self::BoundaryShift => "boundary-shift",
            Self::SplitWord => "split-word",
            Self::GluedWords => "glued-words",
            Self::CaseNoise => "case-noise",
            Self::GrammarAgreement => "grammar-agreement",
            Self::CompletionOnly => "completion-only",
            Self::TechnicalToken => "technical-token",
            Self::ProtectedToken => "protected-token",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateGateAction {
    /// Producer supplied evidence and no local constraint blocked it. Only the
    /// TransitionDecisionCore may turn this into a physical Apply.
    Eligible,
    SuggestOnly,
    KeepOriginal,
    Veto,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateGateDecision {
    pub action: CandidateGateAction,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypingErrorEvent {
    pub original: String,
    pub core: String,
    pub current_word: String,
    pub input_class: TypingErrorClass,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnifiedCorrectionCandidate {
    pub replacement: String,
    pub source: CorrectionDecisionSource,
    pub(crate) origin: CandidateOrigin,
    pub source_id: String,
    pub error_class: TypingErrorClass,
    pub gate: CandidateGateDecision,
    pub(crate) evidence: Vec<CandidateEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CandidateEvidence {
    pub(crate) source: CorrectionDecisionSource,
    pub(crate) origin: CandidateOrigin,
    pub(crate) source_id: String,
    pub(crate) error_class: TypingErrorClass,
    pub(crate) gate: CandidateGateDecision,
}

impl UnifiedCorrectionCandidate {
    pub fn new(
        replacement: impl Into<String>,
        source: CorrectionDecisionSource,
        source_id: impl Into<String>,
        error_class: TypingErrorClass,
        gate: CandidateGateDecision,
    ) -> Self {
        let source_id = source_id.into();
        let origin = correction_source_contract::candidate_origin(&source_id);
        Self {
            replacement: replacement.into(),
            source,
            origin,
            source_id: source_id.clone(),
            error_class,
            gate: gate.clone(),
            evidence: vec![CandidateEvidence {
                source,
                origin,
                source_id,
                error_class,
                gate,
            }],
        }
    }

    pub(crate) fn merge_evidence(&mut self, candidate: Self) {
        let promote_eligible = candidate.origin.source_role() == CorrectionSourceRole::Layout
            && candidate.gate.action == CandidateGateAction::Eligible
            && self.gate.action == CandidateGateAction::SuggestOnly;
        if promote_eligible {
            self.source = candidate.source;
            self.origin = candidate.origin;
            self.source_id.clone_from(&candidate.source_id);
            self.error_class = candidate.error_class;
            self.gate = candidate.gate.clone();
        }
        for evidence in candidate.evidence {
            let already_present = self.evidence.iter().any(|existing| {
                existing.source == evidence.source
                    && existing.origin == evidence.origin
                    && existing.source_id == evidence.source_id
                    && existing.error_class == evidence.error_class
                    && existing.gate == evidence.gate
            });
            if !already_present {
                self.evidence.push(evidence);
            }
        }
    }

    pub(crate) fn has_origin(&self, origin: CandidateOrigin) -> bool {
        self.evidence
            .iter()
            .any(|evidence| evidence.origin == origin)
    }

    pub(crate) fn evidence_count(&self) -> usize {
        self.evidence.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CorrectionResolution {
    pub event: TypingErrorEvent,
    pub candidates: Vec<UnifiedCorrectionCandidate>,
    pub selected: Option<UnifiedCorrectionCandidate>,
    pub decision: Option<CorrectionDecision>,
    pub scoreboard: CorrectionScoreboard,
    pub(crate) candidate_scores: Vec<CorrectionCandidateScoreTrace>,
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
    pub nanda_wave_options: WaveOptions,
    pub mode: CorrectionMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrectionDecision {
    pub replacement: String,
    pub source: CorrectionDecisionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CorrectionScoreboard {
    pub total_candidates: usize,
    pub apply_candidates: usize,
    pub suggest_only_candidates: usize,
    pub keep_original_candidates: usize,
    pub veto_candidates: usize,
    pub deterministic_candidates: usize,
    pub nanda_candidates: usize,
    pub selected_bayes_posterior_milli: Option<i16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct CorrectionGateRuntimeStats {
    requests: u64,
    total_candidates: u64,
    apply_candidates: u64,
    suggest_only_candidates: u64,
    keep_original_candidates: u64,
    veto_candidates: u64,
    deterministic_candidates: u64,
    nanda_candidates: u64,
    selected_apply: u64,
    total_us: u64,
    max_us: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CorrectionCandidateScoreTrace {
    pub(crate) replacement: String,
    pub(crate) source: CorrectionDecisionSource,
    pub(crate) source_id: String,
    pub(crate) error_class: TypingErrorClass,
    pub(crate) action_operator: &'static str,
    pub(crate) action_proof: &'static str,
    pub(crate) edit_transition_operator: &'static str,
    pub(crate) edit_transition_proof: &'static str,
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

pub fn decide_text_correction(req: CorrectionRequest<'_>) -> Option<CorrectionDecision> {
    resolve_text_correction(req).decision
}

pub fn resolve_text_correction(req: CorrectionRequest<'_>) -> CorrectionResolution {
    let started = Instant::now();
    let mut lattice = L2CandidateLattice::with_options(
        TypingErrorEvent::from_text(req.text),
        &req.nanda_wave_options,
    );

    for source in L2CandidateSource::for_mode(req.mode) {
        source.push_candidates(&req, &mut lattice);
    }
    if L2CandidateSource::for_mode(req.mode).contains(&L2CandidateSource::Deterministic) {
        lattice.push_source(short_cyrillic_layout_shadow_candidate(&req));
    }

    let resolution = lattice.into_resolution();
    record_correction_gate_stats(started, &resolution);
    resolution
}

pub fn correction_gate_stats_json() -> serde_json::Value {
    let stats = correction_gate_runtime_stats();
    let avg_us = stats.total_us.checked_div(stats.requests).unwrap_or(0);
    serde_json::json!({
        "requests": stats.requests,
        "total_candidates": stats.total_candidates,
        "apply_candidates": stats.apply_candidates,
        "suggest_only_candidates": stats.suggest_only_candidates,
        "keep_original_candidates": stats.keep_original_candidates,
        "veto_candidates": stats.veto_candidates,
        "deterministic_candidates": stats.deterministic_candidates,
        "nanda_candidates": stats.nanda_candidates,
        "selected_apply": stats.selected_apply,
        "avg_us": avg_us,
        "max_us": stats.max_us,
    })
}

impl CorrectionScoreboard {
    pub(crate) fn from_candidates(
        event: &TypingErrorEvent,
        candidates: &[UnifiedCorrectionCandidate],
        selected: Option<&UnifiedCorrectionCandidate>,
    ) -> Self {
        let mut scoreboard = Self {
            total_candidates: candidates.len(),
            ..Self::default()
        };

        for candidate in candidates {
            match candidate.gate.action {
                CandidateGateAction::Eligible if selected == Some(candidate) => {
                    scoreboard.apply_candidates += 1;
                }
                CandidateGateAction::Eligible => scoreboard.suggest_only_candidates += 1,
                CandidateGateAction::SuggestOnly => scoreboard.suggest_only_candidates += 1,
                CandidateGateAction::KeepOriginal => scoreboard.keep_original_candidates += 1,
                CandidateGateAction::Veto => scoreboard.veto_candidates += 1,
            }
            for evidence in &candidate.evidence {
                match evidence.source {
                    CorrectionDecisionSource::Deterministic => {
                        scoreboard.deterministic_candidates += 1;
                    }
                    CorrectionDecisionSource::Nanda => {
                        scoreboard.nanda_candidates += 1;
                    }
                }
            }
        }

        scoreboard.selected_bayes_posterior_milli = selected.map(|candidate| {
            let posterior = bayes_score_for_candidate(&event.original, candidate).posterior;
            (posterior * 1000.0).round() as i16
        });
        scoreboard
    }
}

fn record_correction_gate_stats(started: Instant, resolution: &CorrectionResolution) {
    let elapsed_us = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    let stats = correction_gate_stats();
    stats.requests.fetch_add(1, Ordering::Relaxed);
    stats.total_candidates.fetch_add(
        resolution.scoreboard.total_candidates as u64,
        Ordering::Relaxed,
    );
    stats.apply_candidates.fetch_add(
        resolution.scoreboard.apply_candidates as u64,
        Ordering::Relaxed,
    );
    stats.suggest_only_candidates.fetch_add(
        resolution.scoreboard.suggest_only_candidates as u64,
        Ordering::Relaxed,
    );
    stats.keep_original_candidates.fetch_add(
        resolution.scoreboard.keep_original_candidates as u64,
        Ordering::Relaxed,
    );
    stats.veto_candidates.fetch_add(
        resolution.scoreboard.veto_candidates as u64,
        Ordering::Relaxed,
    );
    stats.deterministic_candidates.fetch_add(
        resolution.scoreboard.deterministic_candidates as u64,
        Ordering::Relaxed,
    );
    stats.nanda_candidates.fetch_add(
        resolution.scoreboard.nanda_candidates as u64,
        Ordering::Relaxed,
    );
    if resolution.decision.is_some() {
        stats.selected_apply.fetch_add(1, Ordering::Relaxed);
    }
    stats.total_us.fetch_add(elapsed_us, Ordering::Relaxed);
    update_max_atomic(&stats.max_us, elapsed_us);
}

fn correction_gate_runtime_stats() -> CorrectionGateRuntimeStats {
    let stats = correction_gate_stats();
    CorrectionGateRuntimeStats {
        requests: stats.requests.load(Ordering::Relaxed),
        total_candidates: stats.total_candidates.load(Ordering::Relaxed),
        apply_candidates: stats.apply_candidates.load(Ordering::Relaxed),
        suggest_only_candidates: stats.suggest_only_candidates.load(Ordering::Relaxed),
        keep_original_candidates: stats.keep_original_candidates.load(Ordering::Relaxed),
        veto_candidates: stats.veto_candidates.load(Ordering::Relaxed),
        deterministic_candidates: stats.deterministic_candidates.load(Ordering::Relaxed),
        nanda_candidates: stats.nanda_candidates.load(Ordering::Relaxed),
        selected_apply: stats.selected_apply.load(Ordering::Relaxed),
        total_us: stats.total_us.load(Ordering::Relaxed),
        max_us: stats.max_us.load(Ordering::Relaxed),
    }
}

fn correction_gate_stats() -> &'static CorrectionGateAtomicStats {
    static STATS: OnceLock<CorrectionGateAtomicStats> = OnceLock::new();
    STATS.get_or_init(CorrectionGateAtomicStats::default)
}

#[derive(Default)]
struct CorrectionGateAtomicStats {
    requests: AtomicU64,
    total_candidates: AtomicU64,
    apply_candidates: AtomicU64,
    suggest_only_candidates: AtomicU64,
    keep_original_candidates: AtomicU64,
    veto_candidates: AtomicU64,
    deterministic_candidates: AtomicU64,
    nanda_candidates: AtomicU64,
    selected_apply: AtomicU64,
    total_us: AtomicU64,
    max_us: AtomicU64,
}

fn update_max_atomic(target: &AtomicU64, value: u64) {
    let mut current = target.load(Ordering::Relaxed);
    while value > current {
        match target.compare_exchange_weak(current, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(next) => current = next,
        }
    }
}

impl CorrectionCandidateScoreTrace {
    pub(crate) fn from_candidates(
        event: &TypingErrorEvent,
        candidates: &[UnifiedCorrectionCandidate],
        selected: Option<&UnifiedCorrectionCandidate>,
    ) -> Vec<Self> {
        candidates
            .iter()
            .map(|candidate| {
                let score = bayes_score_for_candidate(&event.original, candidate);
                let explanation = explanation_for_candidate(&event.original, candidate);
                let action = action_operator::verify_action_operator(
                    &event.original,
                    &candidate.replacement,
                    candidate.error_class,
                    candidate.origin,
                );
                let decision_signals =
                    candidate_decision_signals(event, candidate, candidates.len());
                Self {
                    replacement: candidate.replacement.clone(),
                    source: candidate.source,
                    source_id: candidate.source_id.clone(),
                    error_class: candidate.error_class,
                    action_operator: action.operator.as_str(),
                    action_proof: action.proof.as_str(),
                    edit_transition_operator: action.edit_operator.as_str(),
                    edit_transition_proof: action.edit_proof.as_str(),
                    edit_transition_verified: action.verifier_passed,
                    edit_transition_left_context_changed: action.left_context_changed,
                    edit_transition_changed_tokens: action.changed_tokens,
                    edit_shape: explanation.edit_shape,
                    preservation_milli: explanation.preservation_milli,
                    lost_mass_milli: explanation.lost_mass_milli,
                    added_mass_milli: explanation.added_mass_milli,
                    operator_fit_milli: explanation.operator_fit_milli,
                    shortcut_risk_milli: explanation.shortcut_risk_milli,
                    anti_wave_milli: explanation.anti_wave_milli,
                    explanation_score_milli: explanation.explanation_score_milli,
                    gate_action: candidate.gate.action,
                    gate_reason: candidate.gate.reason,
                    likelihood_milli: score_to_milli(score.likelihood),
                    usage_prior_milli: score_to_milli(score.usage_prior),
                    context_prior_milli: score_to_milli(score.context_prior),
                    l2_wave_peak_milli: decision_signals.l2_wave_peak_milli,
                    l2_wave_peak_positive_milli: decision_signals.l2_wave_peak_positive_milli,
                    l2_wave_peak_negative_milli: decision_signals.l2_wave_peak_negative_milli,
                    l2_wave_peak_uncertainty_milli: decision_signals.l2_wave_peak_uncertainty_milli,
                    l2_wave_peak_reason: decision_signals.l2_wave_peak_reason,
                    l2_transition_phase_milli: decision_signals.l2_transition_phase_milli,
                    l2_transition_phase_threshold_milli: decision_signals
                        .l2_transition_phase_threshold_milli,
                    l2_transition_phase_verdict: decision_signals.l2_transition_phase_verdict,
                    l2_transition_phase_package_loaded: decision_signals
                        .l2_transition_phase_package_loaded,
                    l2_transition_phase_operator_present: decision_signals
                        .l2_transition_phase_operator_present,
                    l2_transition_phase_operator_promoted: decision_signals
                        .l2_transition_phase_operator_promoted,
                    l2_transition_phase_positive_centers: decision_signals
                        .l2_transition_phase_positive_centers,
                    l2_transition_phase_anti_centers: decision_signals
                        .l2_transition_phase_anti_centers,
                    l2_transition_phase_surfaces: decision_signals.l2_transition_phase_surfaces,
                    l3_phrase_milli: decision_signals.l3_phrase_milli,
                    l3_phrase_decision: decision_signals.l3_phrase_decision,
                    l4_scene_milli: decision_signals.l4_scene_milli,
                    l4_scene_action: decision_signals.l4_scene_action,
                    l4_scene_reason: decision_signals.l4_scene_reason,
                    l4_signed_milli: decision_signals.l4_signed_milli,
                    l4_signed_reason: decision_signals.l4_signed_reason,
                    l4_surface_status: decision_signals.l4_surface_status,
                    l4_transition_state_specific: decision_signals.l4_transition_state_specific,
                    l4_transition_attract_count: decision_signals.l4_transition_attract_count,
                    l4_transition_repel_count: decision_signals.l4_transition_repel_count,
                    risk_milli: score_to_milli(score.risk),
                    posterior_milli: score_to_milli(score.posterior),
                    decision_rank_milli: decision_signals.rank_milli,
                    selected: selected.is_some_and(|selected| selected == candidate),
                }
            })
            .collect()
    }
}

fn score_to_milli(value: f32) -> i16 {
    (value * 1000.0)
        .round()
        .clamp(i16::MIN as f32, i16::MAX as f32) as i16
}

pub(crate) fn explanation_for_candidate(
    original: &str,
    candidate: &UnifiedCorrectionCandidate,
) -> CandidateExplanation {
    explain_candidate(
        original,
        &candidate.replacement,
        candidate.error_class,
        &candidate.source_id,
    )
}

impl TypingErrorEvent {
    fn from_text(text: &str) -> Self {
        L1SurfaceSignal::from_text(text).into_event()
    }
}
include!("correction_core/candidate_sources.rs");
include!("correction_core/tests.rs");
