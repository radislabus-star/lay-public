//! Shared text-correction decision facade.
//!
//! Runtime backends still own output and state. This module only answers one
//! question: should this completed text be replaced, and by which engine?

mod edit_transition;
mod l1_surface_signal;
mod l2_lattice;

use crate::candidate_explanation::{explain_candidate, CandidateExplanation};
use crate::config::{CorrectionSafety, TypingAssistRuleConfig};
use crate::correction_source_contract::{self, CorrectionSourceRole};
use crate::language_action::{operator_for_candidate, proof_for_candidate};
use crate::nanda_wave::l3_phrase_gate::{evaluate_default_candidate, L3PhraseGateDecision};
use crate::nanda_wave::{run_wave_trace, WaveDecision};
use crate::russian_typo_candidates::{
    inserted_char_position_for_missing_letter, repeated_run_deletion_candidates,
};
use crate::text_case::apply_word_case;
use crate::text_metrics::{damerau_levenshtein, has_cyrillic};
use crate::typing_assist::{explain_typing_assist_with_pipeline, split_ws_segments};
use crate::typing_context::{syntax_allows_candidate, typing_assist_pipeline_for_context};
use crate::typing_rule_graph::ids;
use crate::word_reader::{
    cyrillic_word_splits, is_cyrillic_letters_only, last_text_word, replace_last_text_word,
    split_edge_whitespace, split_word_punctuation,
};
use l1_surface_signal::L1SurfaceSignal;
use l2_lattice::L2CandidateLattice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

const COMPOSITE_TRANSPOSE_MIN_MARGIN: f64 = -8.0;

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
    Apply,
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
    pub source_id: String,
    pub error_class: TypingErrorClass,
    pub gate: CandidateGateDecision,
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
    pub(crate) risk_milli: i16,
    pub(crate) posterior_milli: i16,
    pub(crate) selected: bool,
}

pub fn decide_text_correction(req: CorrectionRequest<'_>) -> Option<CorrectionDecision> {
    resolve_text_correction(req).decision
}

pub fn resolve_text_correction(req: CorrectionRequest<'_>) -> CorrectionResolution {
    let started = Instant::now();
    let mut lattice = L2CandidateLattice::new(TypingErrorEvent::from_text(req.text));

    for source in L2CandidateSource::for_mode(req.mode) {
        lattice.push_source(source.propose(&req));
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
    fn from_candidates(
        original: &str,
        candidates: &[UnifiedCorrectionCandidate],
        selected: Option<&UnifiedCorrectionCandidate>,
    ) -> Self {
        let mut scoreboard = Self {
            total_candidates: candidates.len(),
            ..Self::default()
        };

        for candidate in candidates {
            match candidate.gate.action {
                CandidateGateAction::Apply => scoreboard.apply_candidates += 1,
                CandidateGateAction::SuggestOnly => scoreboard.suggest_only_candidates += 1,
                CandidateGateAction::KeepOriginal => scoreboard.keep_original_candidates += 1,
                CandidateGateAction::Veto => scoreboard.veto_candidates += 1,
            }
            match candidate.source {
                CorrectionDecisionSource::Deterministic => {
                    scoreboard.deterministic_candidates += 1;
                }
                CorrectionDecisionSource::Nanda => {
                    scoreboard.nanda_candidates += 1;
                }
            }
        }

        scoreboard.selected_bayes_posterior_milli = selected.map(|candidate| {
            let posterior = bayes_score_for_candidate(original, candidate).posterior;
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
    fn from_candidates(
        original: &str,
        candidates: &[UnifiedCorrectionCandidate],
        selected: Option<&UnifiedCorrectionCandidate>,
    ) -> Vec<Self> {
        candidates
            .iter()
            .map(|candidate| {
                let score = bayes_score_for_candidate(original, candidate);
                let explanation = explanation_for_candidate(original, candidate);
                let transition = edit_transition::prove_edit_transition(
                    original,
                    &candidate.replacement,
                    candidate.error_class,
                    &candidate.source_id,
                );
                Self {
                    replacement: candidate.replacement.clone(),
                    source: candidate.source,
                    source_id: candidate.source_id.clone(),
                    error_class: candidate.error_class,
                    action_operator: operator_for_candidate(
                        candidate.error_class,
                        &candidate.source_id,
                    )
                    .as_str(),
                    action_proof: proof_for_candidate(candidate.error_class, &candidate.source_id)
                        .as_str(),
                    edit_transition_operator: transition.operator.as_str(),
                    edit_transition_proof: transition.language_proof.as_str(),
                    edit_transition_verified: transition.verified,
                    edit_transition_left_context_changed: transition.left_context_changed,
                    edit_transition_changed_tokens: transition.changed_tokens,
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
                    risk_milli: score_to_milli(score.risk),
                    posterior_milli: score_to_milli(score.posterior),
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

fn explanation_for_candidate(
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

fn candidate_rank_score(original: &str, candidate: &UnifiedCorrectionCandidate) -> f32 {
    let bayes = bayes_score_for_candidate(original, candidate).posterior;
    let explanation = explanation_for_candidate(original, candidate);
    let transition = edit_transition::prove_edit_transition(
        original,
        &candidate.replacement,
        candidate.error_class,
        &candidate.source_id,
    );
    bayes
        + ((explanation.explanation_score_milli as f32 - 500.0) / 10_000.0)
        + transition_rank_bonus(transition, &candidate.source_id)
}

fn transition_rank_bonus(transition: edit_transition::EditTransitionProof, source_id: &str) -> f32 {
    if !transition.verified {
        return -0.20;
    }
    match transition.operator {
        edit_transition::EditTransitionOperator::BoundaryShift
        | edit_transition::EditTransitionOperator::SplitPreviousGluedAndRepairTail => 0.34,
        edit_transition::EditTransitionOperator::LayoutProjection => 0.28,
        edit_transition::EditTransitionOperator::PhraseTokenRepair => 0.16,
        edit_transition::EditTransitionOperator::ReplaceCurrentWord => {
            match correction_source_contract::source_role(source_id) {
                CorrectionSourceRole::DeterministicTypo => 0.08,
                CorrectionSourceRole::L2Surface => -0.08,
                _ => 0.0,
            }
        }
        edit_transition::EditTransitionOperator::Completion
        | edit_transition::EditTransitionOperator::Protected
        | edit_transition::EditTransitionOperator::Unknown => 0.0,
    }
}

impl TypingErrorEvent {
    fn from_text(text: &str) -> Self {
        L1SurfaceSignal::from_text(text).into_event()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum L2CandidateSource {
    Deterministic,
    Nanda,
}

impl L2CandidateSource {
    const DETERMINISTIC_ONLY: [Self; 1] = [Self::Deterministic];
    const NANDA_ONLY: [Self; 1] = [Self::Nanda];
    const DETERMINISTIC_THEN_NANDA: [Self; 2] = [Self::Deterministic, Self::Nanda];

    fn for_mode(mode: CorrectionMode) -> &'static [Self] {
        match mode {
            CorrectionMode::DeterministicOnly => &Self::DETERMINISTIC_ONLY,
            CorrectionMode::NandaOnly => &Self::NANDA_ONLY,
            CorrectionMode::DeterministicThenNanda => &Self::DETERMINISTIC_THEN_NANDA,
        }
    }

    fn propose(self, req: &CorrectionRequest<'_>) -> Option<UnifiedCorrectionCandidate> {
        match self {
            Self::Deterministic => deterministic_text_correction(req),
            Self::Nanda => nanda_text_correction(req),
        }
    }
}

fn deterministic_text_correction(
    req: &CorrectionRequest<'_>,
) -> Option<UnifiedCorrectionCandidate> {
    if !(req.auto_replace || req.typing_assist || req.auto_switch_layout) {
        return None;
    }

    let pipeline = typing_assist_pipeline_for_context(
        req.auto_replace,
        req.correction_safety,
        req.typing_assist_pipeline,
        req.text,
    );
    let explanation =
        explain_typing_assist_with_pipeline(req.text, req.auto_switch_layout, &pipeline);
    let Some(replacement) = explanation.output else {
        return deterministic_composite_text_correction(req, &pipeline);
    };
    let rule_id = explanation
        .chosen
        .as_ref()
        .map(|candidate| candidate.rule_id.as_str())
        .unwrap_or("deterministic");
    let error_class = rule_error_class(rule_id);
    let gate = gate_candidate_with_source(req.text, &replacement, error_class, rule_id);
    if matches!(
        error_class,
        TypingErrorClass::RepeatedLetter | TypingErrorClass::ExtraLetter
    ) {
        if let Some(composite) = deterministic_composite_text_correction(req, &pipeline) {
            if should_prefer_composite_after_repeated_repair(
                req.text,
                &replacement,
                &composite.replacement,
            ) {
                return Some(composite);
            }
        }
    }
    if gate.action != CandidateGateAction::Apply {
        return deterministic_composite_text_correction(req, &pipeline).or(Some(
            UnifiedCorrectionCandidate {
                replacement,
                source: CorrectionDecisionSource::Deterministic,
                source_id: rule_id.to_string(),
                error_class,
                gate,
            },
        ));
    }

    Some(UnifiedCorrectionCandidate {
        replacement,
        source: CorrectionDecisionSource::Deterministic,
        source_id: rule_id.to_string(),
        error_class,
        gate,
    })
}

fn deterministic_composite_text_correction(
    req: &CorrectionRequest<'_>,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<UnifiedCorrectionCandidate> {
    layout_then_typo_candidate(req, pipeline)
        .or_else(|| repeated_letter_fallback_candidate(req))
        .or_else(|| composite_russian_typo_candidate(req, pipeline))
}

fn repeated_letter_fallback_candidate(
    req: &CorrectionRequest<'_>,
) -> Option<UnifiedCorrectionCandidate> {
    if !req.typing_assist && !req.auto_replace {
        return None;
    }
    let (_, core, _) = split_edge_whitespace(req.text);
    let current_word = last_text_word(core)?;
    let replacement_word = crate::ru_typo::correct_repeated_letter(&current_word)
        .or_else(|| unique_known_repeated_deletion_word(&current_word))?;
    let replacement = replace_last_text_word(req.text, &replacement_word)?;
    if replacement == req.text || !syntax_allows_candidate(req.text, &replacement) {
        return None;
    }

    let source_id = ids::REPEATED_LETTER;
    let gate = gate_candidate_with_source(
        req.text,
        &replacement,
        TypingErrorClass::RepeatedLetter,
        source_id,
    );
    Some(UnifiedCorrectionCandidate {
        replacement,
        source: CorrectionDecisionSource::Deterministic,
        source_id: source_id.to_string(),
        error_class: TypingErrorClass::RepeatedLetter,
        gate,
    })
}

fn unique_known_repeated_deletion_word(word: &str) -> Option<String> {
    let lower = word.to_lowercase();
    let mut candidates = repeated_run_deletion_candidates(&lower)
        .into_iter()
        .filter(|candidate| {
            crate::russian_lexicon::is_known_russian_word_or_form(candidate)
                || crate::lexicon::is_common_ru_word(candidate)
        })
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.dedup();
    let [candidate] = candidates.as_slice() else {
        return None;
    };
    Some(apply_word_case(word, candidate))
}

fn layout_then_typo_candidate(
    req: &CorrectionRequest<'_>,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<UnifiedCorrectionCandidate> {
    if !req.auto_switch_layout {
        return None;
    }

    let (_, core, _) = split_edge_whitespace(req.text);
    let current_word = last_text_word(core)?;
    if !looks_like_ascii_layout_word(&current_word) {
        return None;
    }

    let converted_word = crate::dict::convert(&current_word, crate::dict::Direction::Us2Ru);
    if converted_word == current_word || !is_cyrillic_letters_only(&converted_word) {
        return None;
    }

    let converted_text = replace_last_text_word(req.text, &converted_word)?;
    let explanation = explain_typing_assist_with_pipeline(&converted_text, false, pipeline);
    let final_replacement = explanation.output.unwrap_or_else(|| {
        if crate::russian_lexicon::is_known_russian_word_or_form(&converted_word) {
            converted_text.clone()
        } else {
            String::new()
        }
    });
    if final_replacement.is_empty() || final_replacement == req.text {
        return None;
    }
    let source_id = explanation
        .chosen
        .as_ref()
        .map(|candidate| format!("layout_then_{}", candidate.rule_id))
        .unwrap_or_else(|| "layout_then_known_word".to_string());
    let gate = gate_candidate_with_source(
        req.text,
        &final_replacement,
        TypingErrorClass::CompositeTypo,
        &source_id,
    );
    Some(UnifiedCorrectionCandidate {
        replacement: final_replacement,
        source: CorrectionDecisionSource::Deterministic,
        source_id,
        error_class: TypingErrorClass::CompositeTypo,
        gate,
    })
}

fn composite_russian_typo_candidate(
    req: &CorrectionRequest<'_>,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<UnifiedCorrectionCandidate> {
    if !req.typing_assist && !req.auto_replace {
        return None;
    }

    let (_, core, _) = split_edge_whitespace(req.text);
    let current_word = last_text_word(core)?;
    let lower = current_word.to_lowercase();
    if !is_cyrillic_letters_only(&current_word)
        || crate::russian_lexicon::is_known_russian_word_or_form(&lower)
    {
        return None;
    }

    if let Some(replacement_word) = unique_adjacent_transposition_word(&current_word) {
        let replacement = replace_last_word_and_split_previous_glued(req.text, &replacement_word)
            .or_else(|| replace_last_text_word(req.text, &replacement_word))?;
        if replacement != req.text && syntax_allows_candidate(req.text, &replacement) {
            let source_id = ids::ADJACENT_TRANSPOSITION;
            let gate = gate_candidate_with_source(
                req.text,
                &replacement,
                TypingErrorClass::AdjacentTransposition,
                source_id,
            );
            return Some(UnifiedCorrectionCandidate {
                replacement,
                source: CorrectionDecisionSource::Deterministic,
                source_id: source_id.to_string(),
                error_class: TypingErrorClass::AdjacentTransposition,
                gate,
            });
        }
    }

    if lower.chars().count() < 4 {
        return None;
    }

    if let Some(candidate) = repeated_prefix_composite_word(&lower) {
        let replacement_word = apply_word_case(&current_word, &candidate);
        let replacement = replace_last_word_and_split_previous_glued(req.text, &replacement_word)
            .or_else(|| replace_last_text_word(req.text, &replacement_word))?;
        if replacement != req.text && syntax_allows_candidate(req.text, &replacement) {
            let source_id = "composite_ru_typo";
            let gate = gate_candidate_with_source(
                req.text,
                &replacement,
                TypingErrorClass::CompositeTypo,
                source_id,
            );
            return Some(UnifiedCorrectionCandidate {
                replacement,
                source: CorrectionDecisionSource::Deterministic,
                source_id: source_id.to_string(),
                error_class: TypingErrorClass::CompositeTypo,
                gate,
            });
        }
    }

    let single_step = current_word_rule_candidate(req, pipeline, &current_word);
    let Some((candidate, _)) = crate::candidate_ranker::choose_best_with_gap(
        crate::ru_typo::fuzzy_known_word_candidates(&lower),
        0.85,
        |candidate| {
            if candidate == &lower
                || repeated_run_deletion_candidates(&lower)
                    .iter()
                    .any(|repaired| repaired == candidate)
                || !crate::russian_lexicon::is_known_russian_word_or_form(candidate)
            {
                return None;
            }
            let distance = damerau_levenshtein(&lower, candidate);
            if distance == 0 || distance > 3 {
                return None;
            }
            let inserted = candidate
                .chars()
                .count()
                .saturating_sub(lower.chars().count());
            if risky_short_initial_insertion(&lower, candidate) {
                return None;
            }
            let common_word_recovery = lower.chars().count() >= 7
                && inserted <= 2
                && distance <= 3
                && repeated_run_deletion_candidates(&lower).is_empty()
                && crate::lexicon::is_common_ru_word(candidate);
            if !common_word_recovery
                && !compatible_composite_typo_shape(&lower, candidate, distance)
            {
                return None;
            }
            let margin = crate::ngram::ru_candidate_margin(candidate, &lower);
            if inserted > 1 && !common_word_recovery {
                return None;
            }
            let shape_bonus = inserted as f64 * 8.0;
            let close_insert_bonus = if distance == 1 && inserted == 1 {
                12.0
            } else {
                0.0
            };
            let common_word_bonus = if common_word_recovery { 12.0 } else { 0.0 };
            let repeated_repair_bonus =
                if repeated_prefix_typo_shape_is_preserved(&lower, candidate) {
                    8.0
                } else {
                    0.0
                };
            let initial_vowel_bonus =
                missing_initial_vowel_before_double_consonant_bonus(&lower, candidate);
            let score = margin
                + shape_bonus
                + close_insert_bonus
                + common_word_bonus
                + repeated_repair_bonus
                + initial_vowel_bonus
                - distance as f64 * 0.35;
            (score >= 0.0).then_some(score)
        },
    ) else {
        return single_step;
    };
    let replacement_word = apply_word_case(&current_word, &candidate);
    let replacement = replace_last_word_and_split_previous_glued(req.text, &replacement_word)
        .or_else(|| replace_last_text_word(req.text, &replacement_word))?;
    if replacement == req.text {
        return None;
    }
    if !syntax_allows_candidate(req.text, &replacement) {
        return None;
    }

    let source_id = "composite_ru_typo";
    let gate = gate_candidate_with_source(
        req.text,
        &replacement,
        TypingErrorClass::CompositeTypo,
        source_id,
    );
    let composite = UnifiedCorrectionCandidate {
        replacement,
        source: CorrectionDecisionSource::Deterministic,
        source_id: source_id.to_string(),
        error_class: TypingErrorClass::CompositeTypo,
        gate,
    };
    if let Some(single_step) = single_step {
        if !should_prefer_composite_after_repeated_repair(
            req.text,
            &single_step.replacement,
            &composite.replacement,
        ) {
            return Some(single_step);
        }
    }
    Some(composite)
}

fn current_word_rule_candidate(
    req: &CorrectionRequest<'_>,
    pipeline: &[TypingAssistRuleConfig],
    current_word: &str,
) -> Option<UnifiedCorrectionCandidate> {
    let current_tail = format!("{current_word} ");
    let explanation = explain_typing_assist_with_pipeline(&current_tail, false, pipeline);
    let replacement_tail = explanation.output?;
    let replacement_word = last_text_word(&replacement_tail)?;
    if replacement_word == current_word || !is_cyrillic_letters_only(&replacement_word) {
        return None;
    }
    let replacement = replace_last_text_word(req.text, &replacement_word)?;
    if replacement == req.text || !syntax_allows_candidate(req.text, &replacement) {
        return None;
    }
    let rule_id = explanation
        .chosen
        .as_ref()
        .map(|candidate| candidate.rule_id.as_str())
        .unwrap_or("current_word_rule");
    let error_class = rule_error_class(rule_id);
    let gate = gate_candidate_with_source(req.text, &replacement, error_class, rule_id);
    Some(UnifiedCorrectionCandidate {
        replacement,
        source: CorrectionDecisionSource::Deterministic,
        source_id: rule_id.to_string(),
        error_class,
        gate,
    })
}

fn repeated_prefix_composite_word(lower: &str) -> Option<String> {
    let repaired_forms = repeated_run_deletion_candidates(lower);
    if repaired_forms.is_empty() {
        return None;
    }
    crate::candidate_ranker::choose_best_with_gap(
        crate::ru_typo::fuzzy_known_word_candidates(lower)
            .into_iter()
            .filter(|candidate| {
                candidate != lower
                    && !repaired_forms.iter().any(|repaired| repaired == candidate)
                    && crate::russian_lexicon::is_known_russian_word_or_form(candidate)
                    && repaired_forms
                        .iter()
                        .any(|repaired| damerau_levenshtein(repaired, candidate) <= 1)
            }),
        0.25,
        |candidate| {
            let best_repaired_distance = repaired_forms
                .iter()
                .map(|repaired| damerau_levenshtein(repaired, candidate))
                .min()?;
            if best_repaired_distance > 1 {
                return None;
            }
            let margin = crate::ngram::ru_candidate_margin(candidate, lower);
            Some(margin + 8.0 - best_repaired_distance as f64 * 0.35)
        },
    )
    .map(|(candidate, _)| candidate)
}

fn nanda_text_correction(req: &CorrectionRequest<'_>) -> Option<UnifiedCorrectionCandidate> {
    if !req.nanda_autocorrect {
        return None;
    }

    let trace = run_wave_trace(req.text);
    match &trace.decision {
        WaveDecision::Apply { text, .. } if text != req.text => {
            let source_id = accepted_wave_source(&trace, text).unwrap_or("NANDA");
            let error_class = nanda_source_error_class(source_id);
            let gate = gate_candidate_with_source(req.text, text, error_class, source_id);
            Some(UnifiedCorrectionCandidate {
                replacement: text.clone(),
                source: CorrectionDecisionSource::Nanda,
                source_id: source_id.to_string(),
                error_class,
                gate,
            })
        }
        WaveDecision::Apply { .. } | WaveDecision::Keep { .. } | WaveDecision::Veto { .. } => None,
    }
}

fn unique_adjacent_transposition_word(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_letters_only(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if crate::russian_lexicon::is_known_russian_word_or_form(&lower) {
        return None;
    }
    let chars: Vec<char> = lower.chars().collect();
    let mut found: Option<String> = None;

    for idx in 0..chars.len().saturating_sub(1) {
        if chars[idx] == chars[idx + 1] {
            continue;
        }

        let mut candidate = chars.clone();
        candidate.swap(idx, idx + 1);
        let candidate: String = candidate.into_iter().collect();
        if candidate == lower || !crate::russian_lexicon::is_known_russian_word_or_form(&candidate)
        {
            continue;
        }
        if crate::ngram::ru_candidate_margin(&candidate, &lower) < COMPOSITE_TRANSPOSE_MIN_MARGIN {
            continue;
        }

        if found.is_some() {
            return None;
        }
        found = Some(candidate);
    }

    found.map(|candidate| apply_word_case(word, &candidate))
}

fn replace_last_word_and_split_previous_glued(
    text: &str,
    replacement_word: &str,
) -> Option<String> {
    let (leading_ws, core, trailing_ws) = split_edge_whitespace(text);
    let segments = split_ws_segments(core);
    let word_indices: Vec<usize> = segments
        .iter()
        .enumerate()
        .filter_map(|(idx, (_, is_ws))| (!*is_ws).then_some(idx))
        .collect();
    let [.., prev_idx, last_idx] = word_indices.as_slice() else {
        return None;
    };

    let (prev_leading, prev_word, prev_trailing) = split_word_punctuation(segments[*prev_idx].0);
    let (last_leading, last_word, last_trailing) = split_word_punctuation(segments[*last_idx].0);
    if !prev_leading.is_empty()
        || !prev_trailing.is_empty()
        || !last_leading.is_empty()
        || prev_word.is_empty()
        || last_word.is_empty()
        || !is_cyrillic_letters_only(prev_word)
        || !is_cyrillic_letters_only(last_word)
    {
        return None;
    }

    let split_previous = split_previous_glued_before_typo(prev_word)?;
    let mut output = String::with_capacity(text.len() + replacement_word.len() + 1);
    output.push_str(leading_ws);
    for (idx, (segment, _is_ws)) in segments.iter().enumerate() {
        if idx == *prev_idx {
            output.push_str(&split_previous);
        } else if idx == *last_idx {
            output.push_str(replacement_word);
            output.push_str(last_trailing);
        } else {
            output.push_str(segment);
        }
    }
    output.push_str(trailing_ws);

    (output != text).then_some(output)
}

fn split_previous_glued_before_typo(word: &str) -> Option<String> {
    let lower = word.to_lowercase();
    if lower.chars().count() < 8 || crate::russian_lexicon::is_known_russian_word_or_form(&lower) {
        return None;
    }

    let mut candidates = Vec::new();
    for split in cyrillic_word_splits(&lower) {
        if split.left_len < 4 || split.left_len > 8 || split.right_len < 5 {
            continue;
        }
        if !crate::phrase_lexicon::is_known_russian_phrase_part(split.left) {
            continue;
        }
        if !crate::phrase_lexicon::is_known_russian_phrase_part(split.right)
            && !looks_like_contextual_russian_verb_form(split.right)
        {
            continue;
        }

        let candidate = format!("{} {}", split.left, split.right);
        let margin = crate::ngram::ru_candidate_margin(&candidate, &lower);
        if margin < -30.0 {
            continue;
        }
        let verb_bonus = if looks_like_contextual_russian_verb_form(split.right) {
            2.0
        } else {
            0.0
        };
        candidates.push((candidate, margin + verb_bonus));
    }

    let ((candidate, _), _) =
        crate::candidate_ranker::choose_best_with_gap(candidates, 0.75, |(_, score)| Some(*score))?;
    Some(crate::text_case::apply_phrase_case(word, &candidate))
}

fn looks_like_contextual_russian_verb_form(word: &str) -> bool {
    word.chars().count() >= 5
        && [
            "ается",
            "яется",
            "уется",
            "ется",
            "етесь",
            "итесь",
            "аешь",
            "яешь",
            "уешь",
            "ешь",
            "ишь",
            "аете",
            "яете",
            "уете",
            "аете",
            "яете",
            "ует",
            "ает",
            "яет",
            "ете",
            "ите",
            "ают",
            "яют",
            "уют",
            "ет",
            "ит",
            "ут",
            "ют",
            "ат",
            "ят",
        ]
        .iter()
        .any(|ending| word.ends_with(ending))
}

fn looks_like_ascii_layout_word(word: &str) -> bool {
    word.is_ascii()
        && word.chars().filter(|ch| ch.is_ascii_alphabetic()).count() >= 3
        && !word.chars().any(|ch| ch.is_ascii_digit())
        && word.chars().all(|ch| {
            ch.is_ascii_alphabetic()
                || matches!(
                    ch,
                    '\'' | ';'
                        | '['
                        | ']'
                        | '`'
                        | ','
                        | '.'
                        | '-'
                        | '{'
                        | '}'
                        | ':'
                        | '"'
                        | '<'
                        | '>'
                        | '~'
                )
        })
}

fn loose_original_shape_is_preserved(original: &str, candidate: &str) -> bool {
    if candidate.chars().count() < original.chars().count() {
        return false;
    }

    let mut original_chars = original.chars().map(loose_shape_char);
    let mut needed = original_chars.next();
    for ch in candidate.chars().map(loose_shape_char) {
        if needed == Some(ch) {
            needed = original_chars.next();
            if needed.is_none() {
                return true;
            }
        }
    }
    needed.is_none()
}

fn compatible_composite_typo_shape(original: &str, candidate: &str, distance: usize) -> bool {
    if loose_original_shape_is_preserved(original, candidate) {
        return true;
    }
    if repeated_prefix_typo_shape_is_preserved(original, candidate) {
        return true;
    }

    distance == 1 && original.chars().count() == candidate.chars().count()
}

fn missing_initial_vowel_before_double_consonant_bonus(original: &str, candidate: &str) -> f64 {
    let Some((idx, inserted)) = inserted_char_position_for_missing_letter(original, candidate)
    else {
        return 0.0;
    };
    if idx != 0 {
        return 0.0;
    }
    let mut chars = original.chars();
    let Some(first) = chars.next() else {
        return 0.0;
    };
    if chars.next() != Some(first) {
        return 0.0;
    }
    match inserted {
        'э' => 10.0,
        'а' | 'о' | 'и' | 'у' | 'е' | 'ё' | 'ю' | 'я' => 2.0,
        _ => 0.0,
    }
}

fn risky_short_initial_insertion(original: &str, candidate: &str) -> bool {
    if original.chars().count() > 6 {
        return false;
    }
    let Some((idx, inserted)) = inserted_char_position_for_missing_letter(original, candidate)
    else {
        return false;
    };
    idx == 0 && (!crate::russian_chars::is_russian_vowel(inserted) || original.chars().count() <= 6)
}

fn repeated_prefix_typo_shape_is_preserved(original: &str, candidate: &str) -> bool {
    repeated_run_deletion_candidates(original)
        .into_iter()
        .any(|repaired| {
            if loose_original_shape_is_preserved(&repaired, candidate) {
                return true;
            }
            repaired.chars().count() == candidate.chars().count()
                && damerau_levenshtein(&repaired, candidate) <= 1
        })
}

fn loose_shape_char(ch: char) -> char {
    match ch {
        'ё' => 'е',
        'Ё' => 'Е',
        'щ' => 'ш',
        'Щ' => 'Ш',
        other => other,
    }
}

fn rule_error_class(rule_id: &str) -> TypingErrorClass {
    match rule_id {
        ids::MIXED_SCRIPT_LAYOUT | ids::DUPLICATE_LAYOUT_PREFIX => TypingErrorClass::MixedScript,
        ids::LAYOUT_TECHNICAL => TypingErrorClass::TechnicalToken,
        ids::FAST_LAYOUT_EN_TO_RU
        | ids::CONTEXTUAL_RU_CONJUNCTION_I
        | ids::CONTEXTUAL_RU_PREPOSITION_V
        | ids::LAYOUT_RU_TO_EN
        | ids::LAYOUT_EN_TO_RU
        | ids::CONTEXTUAL_LAYOUT_EN_TO_RU
        | ids::EXPERIMENTAL_LAYOUT_EN_TO_RU
        | ids::EXPERIMENTAL_LAYOUT_RU_TO_EN
        | ids::VISUAL_B => TypingErrorClass::WrongLayout,
        ids::MOVED_PREFIX_PAIR => TypingErrorClass::PartialLayout,
        ids::SPLIT_WORD_PAIR => TypingErrorClass::SplitWord,
        ids::CYRILLIC_CASE => TypingErrorClass::CaseNoise,
        ids::HARD_SIGN | ids::SINGLE_LETTER_SUBSTITUTION | ids::VOWEL_CONFUSION => {
            TypingErrorClass::LetterSubstitution
        }
        ids::ADJACENT_TRANSPOSITION => TypingErrorClass::AdjacentTransposition,
        ids::REPEATED_LETTER => TypingErrorClass::RepeatedLetter,
        ids::EXTRA_LETTERS => TypingErrorClass::ExtraLetter,
        ids::MISSING_LETTER => TypingErrorClass::MissingLetter,
        ids::VERB_ENDING => TypingErrorClass::GrammarAgreement,
        ids::GLUED_PHRASE => TypingErrorClass::GluedWords,
        ids::PERSONAL_PHRASE | ids::PERSONAL_TOKEN => TypingErrorClass::CompositeTypo,
        _ => TypingErrorClass::Unknown,
    }
}

fn nanda_source_error_class(source: &str) -> TypingErrorClass {
    match correction_source_contract::source_role(source) {
        CorrectionSourceRole::Layout => TypingErrorClass::WrongLayout,
        CorrectionSourceRole::Boundary => TypingErrorClass::GluedWords,
        CorrectionSourceRole::Completion => TypingErrorClass::CompletionOnly,
        CorrectionSourceRole::L2Surface | CorrectionSourceRole::L3Context => {
            TypingErrorClass::CompositeTypo
        }
        CorrectionSourceRole::Technical => TypingErrorClass::TechnicalToken,
        CorrectionSourceRole::DeterministicTypo | CorrectionSourceRole::Unknown => match source {
            "ShortTokenCell32" => TypingErrorClass::PartialLayout,
            "GrammarCell32" => TypingErrorClass::GrammarAgreement,
            "CommonRuFixCell32" | "LearnedMemoryCell32" | "PhraseMemoryCell32" => {
                TypingErrorClass::CompositeTypo
            }
            _ => TypingErrorClass::Unknown,
        },
    }
}

#[cfg(test)]
fn gate_candidate(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> CandidateGateDecision {
    gate_candidate_with_source(original, replacement, error_class, "candidate_gate")
}

fn gate_candidate_with_source(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    source_id: &str,
) -> CandidateGateDecision {
    if original == replacement {
        return CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "unchanged",
        };
    }
    let explanation = explain_candidate(original, replacement, error_class, source_id);
    if explanation.blocks_apply() {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unexplained_signal_loss",
        };
    }
    if replacement_glues_separate_words_without_boundary_class(original, replacement, error_class) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "word_count_shrink_requires_boundary_class",
        };
    }
    if boundary_candidate_glues_short_function_tail(original, replacement, error_class) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unsafe_boundary_glue_short_function_tail",
        };
    }
    if moved_prefix_candidate_eats_known_current_word(original, replacement, source_id) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "moved_prefix_eats_known_current_word",
        };
    }
    if multi_word_candidate_only_completes_last_vowel(original, replacement, error_class) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unsafe_multi_word_vowel_completion",
        };
    }
    if adjacent_transposition_competes_with_single_letter_boundary(
        original,
        replacement,
        error_class,
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "single_letter_boundary_beats_transposition",
        };
    }
    if boundary_candidate_splits_known_russian_word(original, replacement, error_class) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_single_word_boundary_split",
        };
    }
    if boundary_candidate_splits_to_short_function_and_weak_tail(original, replacement, error_class)
    {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "weak_boundary_split_tail",
        };
    }
    if let Some(reason) =
        edit_transition::prove_edit_transition(original, replacement, error_class, source_id)
            .reject_apply_reason()
    {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason,
        };
    }
    if surface_or_context_candidate_changes_left_context(original, replacement, source_id) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "surface_left_context_apply_blocked",
        };
    }
    if l2_surface_candidate_truncates_to_stem_without_deletion_proof(
        original,
        replacement,
        source_id,
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "l2_surface_stem_truncation_low",
        };
    }
    if let Some(decision) = l3_context_gate(original, replacement, error_class, source_id) {
        return decision;
    }
    if semantic_wave_candidate_lacks_surface_authority(original, replacement, source_id) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "semantic_wave_surface_authority_low",
        };
    }
    if l2_surface_candidate_lacks_local_typo_proof(original, replacement, error_class, source_id) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "l2_surface_local_typo_proof_low",
        };
    }
    if error_class == TypingErrorClass::CompletionOnly {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "completion_is_not_autocorrect",
        };
    }
    if let Some(reason) = crate::correction_bayes::bayes_suggest_only_reason(
        original,
        replacement,
        error_class.as_str(),
        source_id,
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason,
        };
    }

    match error_class {
        TypingErrorClass::TechnicalToken | TypingErrorClass::ProtectedToken => {
            CandidateGateDecision {
                action: CandidateGateAction::Veto,
                reason: "protected_or_technical",
            }
        }
        TypingErrorClass::RepeatedLetter | TypingErrorClass::ExtraLetter
            if replacement_last_word_is_unknown_cyrillic(original, replacement) =>
        {
            CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: "single_step_typo_still_unknown",
            }
        }
        TypingErrorClass::RepeatedLetter | TypingErrorClass::ExtraLetter
            if repeated_single_step_has_competing_composite(original, replacement) =>
        {
            CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: "single_step_typo_has_competing_composite",
            }
        }
        TypingErrorClass::Unknown => CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unknown_error_class",
        },
        _ => CandidateGateDecision {
            action: CandidateGateAction::Apply,
            reason: "class_allows_apply",
        },
    }
}

fn surface_or_context_candidate_changes_left_context(
    original: &str,
    replacement: &str,
    source_id: &str,
) -> bool {
    let source_may_only_fix_current_word = match correction_source_contract::source_role(source_id)
    {
        CorrectionSourceRole::L2Surface => true,
        _ => false,
    };
    source_may_only_fix_current_word && candidate_changes_non_last_word(original, replacement)
}

fn candidate_changes_non_last_word(original: &str, replacement: &str) -> bool {
    let original_words = normalized_correction_words(original);
    let replacement_words = normalized_correction_words(replacement);
    if original_words.len() != replacement_words.len() {
        return original_words.len() > 1 || replacement_words.len() > 1;
    }
    if original_words.len() <= 1 {
        return false;
    }
    original_words[..original_words.len() - 1] != replacement_words[..replacement_words.len() - 1]
}

fn normalized_correction_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|token| {
            let (_, word, _) = split_word_punctuation(token);
            (!word.is_empty()).then(|| word.to_lowercase())
        })
        .collect()
}

fn bayes_score_for_candidate(
    original: &str,
    candidate: &UnifiedCorrectionCandidate,
) -> crate::correction_bayes::BayesCandidateScore {
    crate::correction_bayes::bayes_score_candidate(
        original,
        &candidate.replacement,
        candidate.error_class.as_str(),
        &candidate.source_id,
    )
}

fn l3_context_gate(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    source_id: &str,
) -> Option<CandidateGateDecision> {
    if candidate_over_compresses_word(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "candidate_over_compresses_word",
        });
    }
    if candidate_drops_letter_after_one_letter_function_prefix(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "function_prefix_letter_drop",
        });
    }
    if known_phrase_part_only_grows_by_one_letter(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_phrase_part_one_letter_growth",
        });
    }
    if short_word_only_grows_initial_letter(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_initial_letter_growth",
        });
    }
    if short_word_gets_case_vowel_drift(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_case_vowel_drift",
        });
    }
    if soft_sign_word_gets_vowel_drift(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "soft_sign_vowel_drift",
        });
    }
    if short_word_gets_internal_consonant_drift(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_internal_consonant_drift",
        });
    }
    if short_word_same_length_multi_edit_drift(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_same_length_multi_edit_drift",
        });
    }
    if same_tail_single_consonant_drift(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "same_tail_single_consonant_drift",
        });
    }
    if known_russian_word_rewritten_to_different_known_word(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_word_to_different_known_word",
        });
    }
    if short_layout_candidate_lacks_phrase_context(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "short_layout_without_phrase_context",
        });
    }
    if short_cyrillic_word_switches_to_ascii_layout(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_cyrillic_to_ascii_layout",
        });
    }
    if short_nanda_composite_candidate_shrinks_word(original, replacement, error_class, source_id) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_nanda_word_shrink",
        });
    }
    if short_nanda_candidate_inserts_internal_vowel(original, replacement, error_class, source_id) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_nanda_internal_vowel_growth",
        });
    }
    if nanda_surface_candidate_outputs_unknown_word(original, replacement, error_class, source_id) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "nanda_surface_unknown_word",
        });
    }
    if let Some(decision) = l3_phrase_memory_gate(original, replacement, error_class) {
        return Some(decision);
    }
    None
}

fn l3_phrase_memory_gate(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> Option<CandidateGateDecision> {
    if !l3_phrase_memory_applies_to(error_class) {
        return None;
    }
    let report = evaluate_default_candidate(original, replacement)?;
    match report.decision {
        L3PhraseGateDecision::Support => Some(CandidateGateDecision {
            action: CandidateGateAction::Apply,
            reason: "l3_phrase_memory_support",
        }),
        L3PhraseGateDecision::Suppress => Some(CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "l3_phrase_memory_conflict",
        }),
        L3PhraseGateDecision::Neutral => None,
    }
}

fn l3_phrase_memory_applies_to(error_class: TypingErrorClass) -> bool {
    matches!(
        error_class,
        TypingErrorClass::CompositeTypo
            | TypingErrorClass::MissingLetter
            | TypingErrorClass::ExtraLetter
            | TypingErrorClass::AdjacentTransposition
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    )
}

fn replacement_glues_separate_words_without_boundary_class(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if matches!(
        error_class,
        TypingErrorClass::SplitWord | TypingErrorClass::GluedWords
    ) {
        return false;
    }
    let original_words = core_word_count(original);
    let replacement_words = core_word_count(replacement);
    original_words >= 2 && replacement_words < original_words
}

fn boundary_candidate_glues_short_function_tail(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::SplitWord | TypingErrorClass::GluedWords
    ) {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.len() != 2 || replacement_words.len() != 1 {
        return false;
    }
    let left = original_words[0];
    let right = original_words[1];
    let merged = replacement_words[0];
    if !edit_transition::same_cyrillic_token(&format!("{left}{right}"), merged) {
        return false;
    }
    let (_, right_word, _) = split_word_punctuation(right);
    let right_lower = right_word.to_lowercase();
    if matches!(right_lower.as_str(), "ся" | "сь") {
        return false;
    }
    right_word.chars().count() <= 3
        && (crate::phrase_lexicon::is_known_russian_phrase_part(&right_lower)
            || crate::russian_lexicon::is_known_russian_word_or_form(&right_lower)
            || crate::lexicon::is_common_ru_word(&right_lower))
}

fn moved_prefix_candidate_eats_known_current_word(
    original: &str,
    replacement: &str,
    source_id: &str,
) -> bool {
    if source_id != ids::MOVED_PREFIX_PAIR {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.len() < 2 || original_words.len() != replacement_words.len() {
        return false;
    }
    let Some((original_last, original_prefix)) = original_words.split_last() else {
        return false;
    };
    let Some((replacement_last, replacement_prefix)) = replacement_words.split_last() else {
        return false;
    };
    if original_prefix != replacement_prefix {
        return false;
    }
    let (_, original_word, _) = split_word_punctuation(original_last);
    let (_, replacement_word, _) = split_word_punctuation(replacement_last);
    if !is_cyrillic_letters_only(original_word)
        || !is_cyrillic_letters_only(replacement_word)
        || original_word.chars().count() < 4
    {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let stripped = original_lower.chars().skip(1).collect::<String>();
    stripped == replacement_lower
}

fn multi_word_candidate_only_completes_last_vowel(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter | TypingErrorClass::CompositeTypo
    ) {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.len() < 2 || original_words.len() != replacement_words.len() {
        return false;
    }
    let Some((original_last, original_prefix)) = original_words.split_last() else {
        return false;
    };
    let Some((replacement_last, replacement_prefix)) = replacement_words.split_last() else {
        return false;
    };
    if original_prefix != replacement_prefix {
        return false;
    }
    let (_, original_word, _) = split_word_punctuation(original_last);
    let (_, replacement_word, _) = split_word_punctuation(replacement_last);
    if !is_cyrillic_letters_only(original_word)
        || !is_cyrillic_letters_only(replacement_word)
        || original_word.chars().count() < 5
    {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let Some(suffix) = replacement_lower.strip_prefix(&original_lower) else {
        return false;
    };
    suffix.chars().count() == 1
        && suffix.chars().next().is_some_and(|ch| {
            matches!(
                ch,
                'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ы' | 'э' | 'ю' | 'я'
            )
        })
}

fn adjacent_transposition_competes_with_single_letter_boundary(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if error_class != TypingErrorClass::AdjacentTransposition {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.len() < 2 || original_words.len() != replacement_words.len() {
        return false;
    }
    let Some((original_last, original_prefix)) = original_words.split_last() else {
        return false;
    };
    let Some((replacement_last, replacement_prefix)) = replacement_words.split_last() else {
        return false;
    };
    if original_prefix != replacement_prefix {
        return false;
    }
    let (_, original_word, _) = split_word_punctuation(original_last);
    let (_, replacement_word, _) = split_word_punctuation(replacement_last);
    if !is_cyrillic_letters_only(original_word)
        || !is_cyrillic_letters_only(replacement_word)
        || original_word.chars().count() < 4
    {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower == replacement_lower {
        return false;
    }
    let mut chars = original_lower.chars();
    let Some(preposition) = chars.next() else {
        return false;
    };
    if !matches!(preposition, 'в' | 'к' | 'с') {
        return false;
    }
    let tail = chars.collect::<String>();
    tail.chars().count() >= 3
        && (crate::phrase_lexicon::is_known_russian_phrase_part(&tail)
            || crate::russian_lexicon::is_known_russian_word_or_form(&tail)
            || crate::lexicon::is_common_ru_word(&tail))
}

fn boundary_candidate_splits_known_russian_word(
    original: &str,
    replacement: &str,
    _error_class: TypingErrorClass,
) -> bool {
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.is_empty() || replacement_words.len() != original_words.len() + 1 {
        return false;
    }

    for original_idx in 0..original_words.len() {
        let first = replacement_words
            .get(original_idx)
            .copied()
            .unwrap_or_default();
        let second = replacement_words
            .get(original_idx + 1)
            .copied()
            .unwrap_or_default();
        let merged = format!("{first}{second}");
        if !same_known_russian_token(original_words[original_idx], &merged) {
            continue;
        }

        let before_matches = original_words[..original_idx] == replacement_words[..original_idx];
        let after_matches =
            original_words[original_idx + 1..] == replacement_words[original_idx + 2..];
        if before_matches && after_matches {
            return true;
        }
    }
    false
}

fn boundary_candidate_splits_to_short_function_and_weak_tail(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::SplitWord | TypingErrorClass::GluedWords
    ) {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.is_empty() || replacement_words.len() != original_words.len() + 1 {
        return false;
    }

    for (original_idx, original_word) in original_words.iter().enumerate() {
        let first = replacement_words
            .get(original_idx)
            .copied()
            .unwrap_or_default();
        let second = replacement_words
            .get(original_idx + 1)
            .copied()
            .unwrap_or_default();
        let merged = format!("{first}{second}");
        if !edit_transition::same_cyrillic_token(original_word, &merged) {
            continue;
        }

        let (_, first_word, _) = split_word_punctuation(first);
        let (_, second_word, _) = split_word_punctuation(second);
        let first_lower = first_word.to_lowercase();
        let second_lower = second_word.to_lowercase();
        if first_word.chars().count() == 1
            && crate::phrase_lexicon::is_one_letter_russian_function_word(&first_lower)
            && !strong_standalone_split_tail(&second_lower)
        {
            return true;
        }
    }
    false
}

fn semantic_wave_candidate_lacks_surface_authority(
    original: &str,
    replacement: &str,
    source_id: &str,
) -> bool {
    if source_id != crate::nanda_wave::context_wave::SEMANTIC_WORD_SOURCE {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return true;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return true;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }

    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let distance = damerau_levenshtein(&original_lower, &replacement_lower);
    if distance <= 1 {
        return false;
    }

    let max_len = original_lower
        .chars()
        .count()
        .max(replacement_lower.chars().count());
    let original_len = original_lower.chars().count();
    let replacement_len = replacement_lower.chars().count();
    if distance >= 2 && original_len == replacement_len {
        return true;
    }
    let prefix = common_prefix_len(&original_lower, &replacement_lower);
    let known_replacement =
        crate::russian_lexicon::is_known_russian_word_or_form(&replacement_lower)
            || crate::lexicon::is_common_ru_word(&replacement_lower);
    let known_original = crate::russian_lexicon::is_known_russian_word_or_form(&original_lower);

    if distance == 2 && original_len <= 8 && prefix >= 4 && replacement_len <= original_len + 1 {
        return true;
    }
    if distance == 2 && max_len >= 7 && prefix >= 2 && known_replacement {
        return false;
    }
    if distance == 3
        && original_len >= 9
        && max_len >= 10
        && prefix >= 3
        && known_replacement
        && !known_original
    {
        return false;
    }
    true
}

fn l2_surface_candidate_lacks_local_typo_proof(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    source_id: &str,
) -> bool {
    if !correction_source_contract::is_surface_or_context_source(source_id)
        || !matches!(
            error_class,
            TypingErrorClass::CompositeTypo
                | TypingErrorClass::LetterSubstitution
                | TypingErrorClass::GrammarAgreement
        )
    {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return true;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return true;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }

    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower == replacement_lower {
        return false;
    }
    let distance = damerau_levenshtein(&original_lower, &replacement_lower);
    if distance <= 1 {
        return false;
    }

    let original_len = original_lower.chars().count();
    let replacement_len = replacement_lower.chars().count();
    let prefix = common_prefix_len(&original_lower, &replacement_lower);
    if original_len >= 6
        && replacement_len >= original_len
        && distance >= 2
        && prefix >= 2
        && prefix + 3 < original_len.max(replacement_len)
    {
        return true;
    }
    false
}

fn l2_surface_candidate_truncates_to_stem_without_deletion_proof(
    original: &str,
    replacement: &str,
    source_id: &str,
) -> bool {
    if correction_source_contract::source_role(source_id) != CorrectionSourceRole::L2Surface {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let original_len = original_lower.chars().count();
    let replacement_len = replacement_lower.chars().count();
    if replacement_len >= original_len || replacement_len < 4 {
        return false;
    }
    if one_deletion_reduces_to(&original_lower, &replacement_lower) {
        return false;
    }
    let prefix = common_prefix_len(&original_lower, &replacement_lower);
    prefix + 1 >= replacement_len
}

fn one_deletion_reduces_to(original: &str, replacement: &str) -> bool {
    let original_chars = original.chars().collect::<Vec<_>>();
    let replacement_chars = replacement.chars().collect::<Vec<_>>();
    if original_chars.len() != replacement_chars.len() + 1 {
        return false;
    }
    for skip in 0..original_chars.len() {
        if original_chars
            .iter()
            .enumerate()
            .filter_map(|(idx, ch)| (idx != skip).then_some(*ch))
            .eq(replacement_chars.iter().copied())
        {
            return true;
        }
    }
    false
}

fn common_prefix_len(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .count()
}

fn candidate_over_compresses_word(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
            | TypingErrorClass::RepeatedLetter
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_len = original_word.chars().count();
    let replacement_len = replacement_word.chars().count();
    original_len >= 6 && replacement_len + 3 <= original_len
}

fn candidate_drops_letter_after_one_letter_function_prefix(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
    ) {
        return false;
    }

    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.len() < 2 || original_words.len() != replacement_words.len() {
        return false;
    }
    let Some((original_last, original_prefix)) = original_words.split_last() else {
        return false;
    };
    let Some((replacement_last, replacement_prefix)) = replacement_words.split_last() else {
        return false;
    };
    if original_prefix != replacement_prefix {
        return false;
    }

    let (_, original_word, _) = split_word_punctuation(original_last);
    let (_, replacement_word, _) = split_word_punctuation(replacement_last);
    if !is_cyrillic_letters_only(original_word) || !is_cyrillic_letters_only(replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let original_chars = original_lower.chars().collect::<Vec<_>>();
    if original_chars.len() < 4 || replacement_lower.chars().count() + 1 != original_chars.len() {
        return false;
    }
    let prefix = original_chars[0].to_string();
    if !crate::phrase_lexicon::is_one_letter_russian_function_word(&prefix) {
        return false;
    }

    let compressed = std::iter::once(original_chars[0])
        .chain(original_chars.iter().skip(2).copied())
        .collect::<String>();
    compressed == replacement_lower
}

fn known_russian_word_rewritten_to_different_known_word(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::CompositeTypo
            | TypingErrorClass::MissingLetter
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }

    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower == replacement_lower {
        return false;
    }

    known_russian_autocorrect_token(&original_lower)
        && known_russian_autocorrect_token(&replacement_lower)
}

fn known_russian_autocorrect_token(lower: &str) -> bool {
    crate::lexicon::is_common_ru_word(lower)
        || crate::lexicon::is_ru_live_protected_word(lower)
        || crate::lexicon::is_user_protected_word(lower)
        || crate::russian_lexicon::is_known_russian_word_or_form(lower)
        || crate::russian_lexicon::is_known_russian_adverb_o_form(lower)
        || crate::russian_lexicon::is_known_russian_ka_oblique_form(lower)
        || protected_pattern_term_stem(lower)
}

fn protected_pattern_term_stem(lower: &str) -> bool {
    lower.starts_with("патерн") || lower.starts_with("паттерн")
}

fn known_phrase_part_only_grows_by_one_letter(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
            | TypingErrorClass::AdjacentTransposition
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if !is_cyrillic_letters_only(&original_word)
        || !is_cyrillic_letters_only(&replacement_word)
        || original_lower == replacement_lower
        || !crate::phrase_lexicon::is_known_russian_phrase_part(&original_lower)
    {
        return false;
    }

    inserted_char_position_for_missing_letter(&original_lower, &replacement_lower).is_some()
}

fn short_word_only_grows_initial_letter(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
            | TypingErrorClass::AdjacentTransposition
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if !is_cyrillic_letters_only(&original_word)
        || !is_cyrillic_letters_only(&replacement_word)
        || original_lower == replacement_lower
    {
        return false;
    }
    let Some((idx, _inserted)) =
        inserted_char_position_for_missing_letter(&original_lower, &replacement_lower)
    else {
        return false;
    };
    idx == 0 && original_lower.chars().count() <= 6
}

fn short_word_gets_case_vowel_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower.chars().count() > 5 || !replacement_lower.starts_with(&original_lower) {
        return false;
    }
    let suffix = replacement_lower
        .strip_prefix(&original_lower)
        .unwrap_or_default();
    suffix.chars().count() == 1 && matches!(suffix, "а" | "я" | "у" | "ю" | "ы" | "и")
}

fn soft_sign_word_gets_vowel_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if !original_lower.ends_with('ь') || original_lower.chars().count() > 6 {
        return false;
    }
    let original_stem = original_lower.trim_end_matches('ь');
    replacement_lower.starts_with(original_stem)
        && replacement_lower
            .chars()
            .last()
            .is_some_and(crate::russian_chars::is_russian_vowel)
}

fn short_word_gets_internal_consonant_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower.chars().count() > 6 {
        return false;
    }
    let Some((idx, inserted)) =
        inserted_char_position_for_missing_letter(&original_lower, &replacement_lower)
    else {
        return false;
    };
    if crate::russian_chars::is_russian_vowel(inserted) {
        return false;
    }
    let previous_original = idx
        .checked_sub(1)
        .and_then(|previous_idx| original_lower.chars().nth(previous_idx));
    let next_original = original_lower.chars().nth(idx);
    if Some(inserted) == previous_original || Some(inserted) == next_original {
        return false;
    }
    !(inserted == 'ч' && matches!(next_original, Some('ш' | 'щ')))
}

fn short_word_same_length_multi_edit_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let original_len = original_lower.chars().count();
    let replacement_len = replacement_lower.chars().count();
    original_len <= 6
        && original_len == replacement_len
        && damerau_levenshtein(&original_lower, &replacement_lower) >= 2
}

fn same_tail_single_consonant_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::CompositeTypo | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let original_chars = original_lower.chars().collect::<Vec<_>>();
    let replacement_chars = replacement_lower.chars().collect::<Vec<_>>();
    if original_chars.len() < 6
        || original_chars.len() != replacement_chars.len()
        || damerau_levenshtein(&original_lower, &replacement_lower) != 1
    {
        return false;
    }
    let diffs = original_chars
        .iter()
        .zip(&replacement_chars)
        .enumerate()
        .filter(|(_, (left, right))| left != right)
        .collect::<Vec<_>>();
    let [(idx, (left, right))] = diffs.as_slice() else {
        return false;
    };
    *idx > 1
        && *idx + 2 < original_chars.len()
        && is_russian_consonant(**left)
        && is_russian_consonant(**right)
        && original_chars[original_chars.len() - 2..]
            == replacement_chars[replacement_chars.len() - 2..]
}

fn is_russian_consonant(ch: char) -> bool {
    matches!(ch, 'а'..='я' | 'ё')
        && !crate::russian_chars::is_russian_vowel(ch)
        && !matches!(ch, 'ь' | 'ъ')
}

fn short_layout_candidate_lacks_phrase_context(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(error_class, TypingErrorClass::PartialLayout) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if original_word.chars().count() != 1 || replacement_word.chars().count() != 1 {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let previous_words = original_core
        .split_whitespace()
        .take(original_core.split_whitespace().count().saturating_sub(1))
        .collect::<Vec<_>>();
    let has_cyrillic_context = previous_words.iter().any(|word| has_cyrillic(word));
    let has_ascii_context = previous_words
        .iter()
        .any(|word| word.chars().any(|ch| ch.is_ascii_alphabetic()));

    has_ascii_context && !has_cyrillic_context
}

fn short_cyrillic_word_switches_to_ascii_layout(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if error_class != TypingErrorClass::WrongLayout {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    original_word.chars().count() <= 2
        && original_word
            .chars()
            .any(|ch| matches!(ch, 'а'..='я' | 'ё' | 'А'..='Я' | 'Ё'))
        && replacement_word
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || matches!(ch, '`'))
}

fn short_nanda_composite_candidate_shrinks_word(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    source_id: &str,
) -> bool {
    if !matches!(error_class, TypingErrorClass::CompositeTypo)
        || !correction_source_contract::is_surface_or_context_source(source_id)
    {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_len = original_word.chars().count();
    let replacement_len = replacement_word.chars().count();
    original_len <= 4 && replacement_len < original_len
}

fn nanda_surface_candidate_outputs_unknown_word(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    source_id: &str,
) -> bool {
    if !matches!(error_class, TypingErrorClass::CompositeTypo)
        || !correction_source_contract::is_surface_or_context_source(source_id)
    {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if original_word == replacement_word || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let replacement_lower = replacement_word.to_lowercase();
    !crate::russian_lexicon::is_known_russian_word_or_form(&replacement_lower)
        && !crate::lexicon::is_common_ru_word(&replacement_lower)
        && !crate::phrase_lexicon::is_known_russian_phrase_part(&replacement_lower)
}

fn short_nanda_candidate_inserts_internal_vowel(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    source_id: &str,
) -> bool {
    if !matches!(error_class, TypingErrorClass::CompositeTypo)
        || !correction_source_contract::is_surface_or_context_source(source_id)
    {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower.chars().count() > 6
        || damerau_levenshtein(&original_lower, &replacement_lower) != 1
    {
        return false;
    }
    let Some((idx, inserted)) =
        inserted_char_position_for_missing_letter(&original_lower, &replacement_lower)
    else {
        return false;
    };
    idx > 0 && crate::russian_chars::is_russian_vowel(inserted)
}

fn same_known_russian_token(original: &str, candidate: &str) -> bool {
    let (_, original_word, _) = split_word_punctuation(original);
    let (_, candidate_word, _) = split_word_punctuation(candidate);
    if original_word.is_empty()
        || candidate_word.is_empty()
        || !is_cyrillic_letters_only(original_word)
        || !is_cyrillic_letters_only(candidate_word)
    {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    original_lower == candidate_word.to_lowercase()
        && crate::russian_lexicon::is_known_russian_word_or_form(&original_lower)
}

fn strong_standalone_split_tail(lower: &str) -> bool {
    lower.chars().count() >= 4
        && (crate::lexicon::is_common_ru_word(lower)
            || crate::russian_lexicon::russian_dictionary().contains(lower)
            || crate::russian_lexicon::is_known_russian_adverb_o_form(lower)
            || crate::russian_lexicon::is_known_russian_ka_oblique_form(lower))
}

fn core_word_count(text: &str) -> usize {
    let (_, core, _) = split_edge_whitespace(text);
    core.split_whitespace().count()
}

fn replacement_last_word_is_unknown_cyrillic(original: &str, replacement: &str) -> bool {
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if original_word == replacement_word || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let replacement_lower = replacement_word.to_lowercase();
    !crate::russian_lexicon::is_known_russian_word_or_form(&replacement_lower)
        && !crate::lexicon::is_common_ru_word(&replacement_lower)
}

fn repeated_single_step_has_competing_composite(original: &str, replacement: &str) -> bool {
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if !repeated_run_deletion_candidates(&original_lower)
        .into_iter()
        .any(|candidate| candidate == replacement_lower)
    {
        return false;
    }
    crate::ru_typo::fuzzy_known_word_candidates(&original_lower)
        .into_iter()
        .any(|candidate| {
            candidate != replacement_lower
                && crate::russian_lexicon::is_known_russian_word_or_form(&candidate)
                && damerau_levenshtein(&replacement_lower, &candidate) <= 1
        })
}

fn should_prefer_composite_after_repeated_repair(
    original: &str,
    single_step: &str,
    composite: &str,
) -> bool {
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(single_word) = last_text_word(single_step) else {
        return false;
    };
    let Some(composite_word) = last_text_word(composite) else {
        return false;
    };
    let original_lower = original_word.to_lowercase();
    let single_lower = single_word.to_lowercase();
    let composite_lower = composite_word.to_lowercase();
    if single_lower == composite_lower || !is_cyrillic_letters_only(&composite_word) {
        return false;
    }
    if single_word.chars().count() < original_word.chars().count()
        && composite_word.chars().count() > original_word.chars().count()
        && damerau_levenshtein(&original_lower, &composite_lower) <= 1
        && crate::russian_lexicon::is_known_russian_word_or_form(&composite_lower)
    {
        return true;
    }
    repeated_run_deletion_candidates(&original_lower)
        .into_iter()
        .any(|candidate| candidate == single_lower)
        && composite_word.chars().count() > original_word.chars().count()
        && damerau_levenshtein(&single_lower, &composite_lower) <= 1
        && crate::russian_lexicon::is_known_russian_word_or_form(&composite_lower)
}

fn accepted_wave_source<'a>(
    trace: &'a crate::nanda_wave::WaveTrace,
    replacement: &str,
) -> Option<&'a str> {
    let trimmed = replacement.trim();
    trace
        .l2_candidates
        .iter()
        .find(|candidate| candidate.text.trim() == trimmed)
        .map(|candidate| candidate.source)
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
    fn l2_candidate_sources_follow_correction_mode_order() {
        assert_eq!(
            L2CandidateSource::for_mode(CorrectionMode::DeterministicOnly),
            &[L2CandidateSource::Deterministic]
        );
        assert_eq!(
            L2CandidateSource::for_mode(CorrectionMode::NandaOnly),
            &[L2CandidateSource::Nanda]
        );
        assert_eq!(
            L2CandidateSource::for_mode(CorrectionMode::DeterministicThenNanda),
            &[L2CandidateSource::Deterministic, L2CandidateSource::Nanda]
        );
    }

    #[test]
    fn l2_candidate_lattice_keeps_sources_and_selects_only_apply_candidate() {
        let mut lattice = L2CandidateLattice::new(TypingErrorEvent::from_text("автозаена "));
        lattice.push_source(Some(UnifiedCorrectionCandidate {
            replacement: "автозамена ".to_string(),
            source: CorrectionDecisionSource::Nanda,
            source_id: "L2SurfaceMotifCell32".to_string(),
            error_class: TypingErrorClass::MissingLetter,
            gate: CandidateGateDecision {
                action: CandidateGateAction::Apply,
                reason: "class_allows_apply",
            },
        }));
        lattice.push_source(Some(UnifiedCorrectionCandidate {
            replacement: "автозамена ".to_string(),
            source: CorrectionDecisionSource::Deterministic,
            source_id: ids::MISSING_LETTER.to_string(),
            error_class: TypingErrorClass::MissingLetter,
            gate: CandidateGateDecision {
                action: CandidateGateAction::Apply,
                reason: "class_allows_apply",
            },
        }));
        lattice.push_source(Some(UnifiedCorrectionCandidate {
            replacement: "авто замена ".to_string(),
            source: CorrectionDecisionSource::Nanda,
            source_id: "BoundaryCell32".to_string(),
            error_class: TypingErrorClass::GluedWords,
            gate: CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: "requires_boundary_proof",
            },
        }));

        let resolution = lattice.into_resolution();

        assert_eq!(resolution.candidates.len(), 2);
        assert_eq!(
            resolution
                .candidates
                .iter()
                .filter(|candidate| candidate.replacement == "автозамена ")
                .count(),
            1,
            "duplicate same-replacement candidates must collapse to the owner"
        );
        assert_eq!(resolution.scoreboard.total_candidates, 2);
        assert_eq!(resolution.scoreboard.deterministic_candidates, 1);
        assert_eq!(resolution.scoreboard.nanda_candidates, 1);
        assert_eq!(resolution.scoreboard.apply_candidates, 1);
        assert_eq!(resolution.scoreboard.suggest_only_candidates, 1);
        assert_eq!(
            resolution.selected.as_ref().map(|candidate| {
                (
                    candidate.replacement.as_str(),
                    candidate.source,
                    candidate.gate.action,
                )
            }),
            Some((
                "автозамена ",
                CorrectionDecisionSource::Deterministic,
                CandidateGateAction::Apply,
            ))
        );
        assert_eq!(
            resolution.decision,
            Some(CorrectionDecision {
                replacement: "автозамена ".to_string(),
                source: CorrectionDecisionSource::Deterministic,
            })
        );
    }

    #[test]
    fn l2_surface_candidate_cannot_apply_left_context_rewrite() {
        let gate = gate_candidate_with_source(
            "коретка улитела ",
            "етка улитка ",
            TypingErrorClass::CompositeTypo,
            "L2SurfaceMotifCell32",
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_ne!(gate.reason, "class_allows_apply");
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
    fn deterministic_mode_corrects_multiword_wrong_layout_tail() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "HF<JNF NTCN CFV ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert_eq!(
            resolution
                .decision
                .as_ref()
                .map(|decision| decision.replacement.as_str()),
            Some("РАБОТА ТЕСТ САМ ")
        );
        assert_eq!(
            resolution
                .selected
                .as_ref()
                .map(|candidate| candidate.gate.action),
            Some(CandidateGateAction::Apply)
        );
    }

    #[test]
    fn deterministic_mode_corrects_multiword_wrong_layout_tail_with_context_pipeline() {
        let default_pipeline = default_typing_assist_pipeline();
        let pipeline = crate::typing_context::typing_assist_pipeline_for_context(
            true,
            CorrectionSafety::Normal,
            &default_pipeline,
            "HF<JNF NTCN CFV ",
        );
        let resolution = resolve_text_correction(CorrectionRequest {
            text: "HF<JNF NTCN CFV ",
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: CorrectionSafety::Normal,
            typing_assist_pipeline: &pipeline,
            nanda_autocorrect: false,
            mode: CorrectionMode::DeterministicOnly,
        });

        assert_eq!(
            resolution
                .decision
                .as_ref()
                .map(|decision| decision.replacement.as_str()),
            Some("РАБОТА ТЕСТ САМ ")
        );
        assert_eq!(
            resolution
                .selected
                .as_ref()
                .map(|candidate| candidate.gate.action),
            Some(CandidateGateAction::Apply)
        );
    }

    #[test]
    fn resolution_routes_missing_letter_through_unified_gate() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "автозаена ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected candidate: {resolution:?}"));
        assert_eq!(selected.replacement, "автозамена ");
        assert_eq!(selected.error_class, TypingErrorClass::MissingLetter);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
        assert_eq!(resolution.scoreboard.total_candidates, 1);
        assert_eq!(resolution.scoreboard.apply_candidates, 1);
        assert_eq!(resolution.scoreboard.deterministic_candidates, 1);
        assert_eq!(resolution.scoreboard.nanda_candidates, 0);
        assert!(
            resolution
                .scoreboard
                .selected_bayes_posterior_milli
                .is_some(),
            "selected candidate must expose Bayes posterior"
        );
        assert_eq!(resolution.candidate_scores.len(), 1);
        let score = &resolution.candidate_scores[0];
        assert_eq!(score.replacement, "автозамена ");
        assert_eq!(score.error_class, TypingErrorClass::MissingLetter);
        assert_eq!(score.action_operator, "restore_missing_letter");
        assert_eq!(score.action_proof, "typo");
        assert_eq!(score.gate_action, CandidateGateAction::Apply);
        assert!(score.selected);
        assert!(score.likelihood_milli > 0);
        assert!(score.posterior_milli > 0);
    }

    #[test]
    fn unexplained_signal_loss_blocks_l2_shortcut_candidate() {
        let gate = gate_candidate_with_source(
            "тоесть ",
            "есть ",
            TypingErrorClass::CompositeTypo,
            "L2SurfaceMotifCell32",
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "unexplained_signal_loss");
    }

    #[test]
    fn surface_candidate_cannot_apply_extra_left_context() {
        let gate = gate_candidate_with_source(
            "содержкой ",
            "что получилось вроде хороший ввод и даже фикс был шикарный но с содержать ",
            TypingErrorClass::CompositeTypo,
            "L2SurfaceMotifCell32",
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "edit_transition_not_verified");
    }

    #[test]
    fn surface_candidate_may_replace_only_current_word_with_same_prefix() {
        let gate = gate_candidate_with_source(
            "что получилось содержкой ",
            "что получилось содержать ",
            TypingErrorClass::CompositeTypo,
            "L2SurfaceMotifCell32",
        );

        assert_ne!(gate.reason, "edit_transition_not_verified");
    }

    #[test]
    fn layout_candidate_cannot_add_context_to_single_word() {
        let gate = gate_candidate_with_source(
            "uрафике ",
            "на графике ",
            TypingErrorClass::WrongLayout,
            "LayoutWordCell32",
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "edit_transition_not_verified");
    }

    #[test]
    fn layout_candidate_may_rewrite_multiword_layout_tail() {
        let gate = gate_candidate_with_source(
            "HF<JNF NTCN CFV ",
            "РАБОТА ТЕСТ САМ ",
            TypingErrorClass::WrongLayout,
            ids::LAYOUT_EN_TO_RU,
        );

        assert_eq!(gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn boundary_preserving_candidate_survives_explanation_gate() {
        let gate = gate_candidate_with_source(
            "тоесть ",
            "то есть ",
            TypingErrorClass::CompositeTypo,
            ids::PERSONAL_PHRASE,
        );

        assert_eq!(gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn split_phrase_candidate_wins_over_l2_shortcut() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "тоесть ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "то есть ");
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn unknown_russian_shape_is_classified_before_candidate_generation() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "приудишна ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        assert_eq!(resolution.event.current_word, "приудишна");
        assert_eq!(
            resolution.event.input_class,
            TypingErrorClass::CompositeTypo
        );
        assert_eq!(resolution.decision, None);
    }

    #[test]
    fn l3_anti_shortcut_blocks_overcompressed_word_candidate() {
        let gate = gate_candidate_with_source(
            "патерна ",
            "пара ",
            TypingErrorClass::CompositeTypo,
            crate::nanda_wave::context_wave::SEMANTIC_WORD_SOURCE,
        );

        assert_eq!(gate.action, CandidateGateAction::KeepOriginal);
        assert_eq!(gate.reason, "candidate_over_compresses_word");
    }

    #[test]
    fn l2_surface_cannot_apply_context_stem_truncation() {
        let gate = gate_candidate_with_source(
            "я прохоил ",
            "я проход ",
            TypingErrorClass::CompositeTypo,
            "L2SurfaceMotifCell32",
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "l2_surface_stem_truncation_low");
    }

    #[test]
    fn l3_anti_shortcut_blocks_function_prefix_letter_drop_from_logs() {
        let gate = gate_candidate_with_source(
            "ответили вчате ",
            "ответили вате ",
            TypingErrorClass::CompositeTypo,
            crate::nanda_wave::context_wave::SEMANTIC_WORD_SOURCE,
        );

        assert_eq!(gate.action, CandidateGateAction::KeepOriginal);
        assert_eq!(gate.reason, "function_prefix_letter_drop");
    }

    #[test]
    fn l3_anti_shortcut_blocks_short_layout_without_phrase_context() {
        let pipeline = default_typing_assist_pipeline();
        let resolution =
            resolve_text_correction(request("wave b ", &pipeline, CorrectionMode::NandaOnly));

        assert_eq!(resolution.decision, None);
        assert!(
            resolution.candidates.is_empty(),
            "short layout candidate must be stopped inside NANDA L3 before correction_core: {resolution:?}"
        );
    }

    #[test]
    fn known_russian_word_with_yo_is_not_layout_switched_to_ascii() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "ещё ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert_eq!(resolution.decision, None);
        assert!(
            resolution.candidates.iter().all(|candidate| {
                candidate.replacement != "to` "
                    && candidate.gate.action != CandidateGateAction::Apply
            }),
            "known Russian word must not autoswitch to ASCII layout: {resolution:?}"
        );
    }

    #[test]
    fn short_russian_word_does_not_autoswitch_to_ascii_from_logs() {
        let pipeline = default_typing_assist_pipeline();
        let resolution =
            resolve_text_correction(request("ой ", &pipeline, CorrectionMode::DeterministicOnly));

        assert_eq!(resolution.decision, None);
        assert!(resolution.candidates.iter().any(|candidate| {
            candidate.replacement == "jq "
                && candidate.gate.action == CandidateGateAction::SuggestOnly
                && candidate.gate.reason == "short_cyrillic_to_ascii_layout"
        }));
    }

    #[test]
    fn russian_phrase_context_still_allows_short_preposition_repair() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "читай cola d wechat ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected candidate: {resolution:?}"));
        assert_eq!(selected.replacement, "читай cola в wechat ");
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn layout_then_typo_repairs_dirty_wrong_layout_word() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "hf,jfntn ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "работает ");
        assert_eq!(selected.source_id, "layout_then_adjacent_transposition");
        assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn composite_typo_repairs_known_russian_word() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "помшник ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected candidate: {resolution:?}"));
        assert_eq!(selected.replacement, "помощник ");
        assert_eq!(selected.source_id, "composite_ru_typo");
        assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn composite_typo_does_not_jump_over_known_single_step_repair() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "мы отвравим ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "мы отравим ");
        assert_ne!(selected.replacement, "мы отвратим ");
    }

    #[test]
    fn nanda_semantic_drift_does_not_beat_local_single_step_repair() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "мы отвравим ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        let selected = resolution.selected.as_ref().expect("selected candidate");
        assert_eq!(selected.replacement, "мы отравим ");
        assert_eq!(selected.source, CorrectionDecisionSource::Deterministic);
        assert!(
            resolution.candidates.iter().all(|candidate| {
                candidate.source != CorrectionDecisionSource::Nanda
                    || candidate.replacement != selected.replacement
            }),
            "NANDA must not steal deterministic ownership for the same replacement: {resolution:?}"
        );
        assert!(
            resolution.candidates.iter().all(|candidate| {
                candidate.replacement != "мы отвратим "
                    || candidate.gate.action != CandidateGateAction::Apply
            }),
            "semantic drift candidate must not be apply: {resolution:?}"
        );
    }

    #[test]
    fn composite_gate_blocks_same_tail_consonant_semantic_drift() {
        let decision = gate_candidate_with_source(
            "будет примать ",
            "будет придать ",
            TypingErrorClass::CompositeTypo,
            crate::nanda_wave::context_wave::SEMANTIC_WORD_SOURCE,
        );

        assert_eq!(decision.action, CandidateGateAction::SuggestOnly);
        assert_eq!(decision.reason, "same_tail_single_consonant_drift");
    }

    #[test]
    fn composite_typo_rejects_short_initial_consonant_growth() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "давай лушее ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.expect("safe candidate should remain");
        assert_eq!(selected.replacement, "давай лучшее ");
        assert!(!resolution
            .candidates
            .iter()
            .any(|candidate| candidate.replacement == "давай глушее "
                && candidate.gate.action == CandidateGateAction::Apply));
    }

    #[test]
    fn composite_typo_rejects_short_initial_vowel_growth_from_logs() {
        let pipeline = default_typing_assist_pipeline();
        for (input, forbidden) in [("рина ", "арина "), ("решение задачь ", "решение озадачь ")]
        {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            ));

            assert!(
                resolution
                    .decision
                    .as_ref()
                    .map(|decision| &decision.replacement)
                    != Some(&forbidden.to_string()),
                "forbidden candidate auto-applied: {resolution:?}"
            );
        }
    }

    #[test]
    fn known_russian_words_do_not_autorewrite_to_other_known_words() {
        let pipeline = default_typing_assist_pipeline();
        for (input, forbidden) in [
            ("искать хрень! ", "искать хрену "),
            ("будет плох ", "будет плоха "),
            ("Блин ", "Блина "),
            ("не мение ", "не мерние "),
            ("не мение ", "не менте "),
            ("теорию бейса ", "теорию бейсяа "),
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            ));

            assert!(
                resolution
                    .decision
                    .as_ref()
                    .map(|decision| &decision.replacement)
                    != Some(&forbidden.to_string()),
                "forbidden known-word rewrite auto-applied: {resolution:?}"
            );
        }
    }

    #[test]
    fn composite_typo_recovers_common_word_with_broken_prefix() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "где эсперемнт ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "где эксперимент ");
        assert_eq!(selected.source_id, "composite_ru_typo");
        assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn composite_typo_prefers_effective_over_affective_for_missing_initial_vowel() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "на сколько ффективная ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "на сколько эффективная ");
        assert_eq!(selected.source_id, "composite_ru_typo");
        assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn boundary_gate_does_not_split_known_single_word() {
        let gate = gate_candidate("уровне ", "у ровне ", TypingErrorClass::GluedWords);

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "known_single_word_boundary_split");
    }

    #[test]
    fn boundary_gate_does_not_split_known_word_inside_phrase() {
        let gate = gate_candidate("на уровне ", "на у ровне ", TypingErrorClass::GluedWords);

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "known_single_word_boundary_split");
    }

    #[test]
    fn boundary_gate_rejects_known_word_split_from_non_boundary_candidate() {
        let gate = gate_candidate_with_source(
            "за настройки ",
            "за нас тройки ",
            TypingErrorClass::CompositeTypo,
            crate::nanda_wave::context_wave::SEMANTIC_WORD_SOURCE,
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "known_single_word_boundary_split");
    }

    #[test]
    fn boundary_gate_rejects_short_function_split_with_unknown_tail() {
        let gate = gate_candidate("со скрина ", "со с крина ", TypingErrorClass::GluedWords);

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "weak_boundary_split_tail");
    }

    #[test]
    fn composite_typo_repairs_generated_russian_forms() {
        let pipeline = default_typing_assist_pipeline();
        for (input, expected) in [("руских ", "русских "), ("звгрузи ", "загрузи ")]
        {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            ));

            let selected = resolution.selected.expect("selected candidate");
            assert_eq!(selected.replacement, expected, "input={input:?}");
            assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
            assert_eq!(selected.gate.action, CandidateGateAction::Apply);
        }
    }

    #[test]
    fn known_phrase_parts_do_not_autogrow_by_one_letter() {
        let pipeline = default_typing_assist_pipeline();
        for (input, forbidden) in [
            ("у меня ", "у меняю "),
            ("твой ", "тывой "),
            ("к тебе ", "к требе "),
            ("Тебе ", "Требе "),
            ("в план! ", "в плана! "),
            ("но пока ", "но прока "),
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            ));

            assert_eq!(resolution.decision, None, "input={input:?}");
            assert!(
                resolution.candidates.iter().all(|candidate| {
                    candidate.replacement != forbidden
                        || candidate.gate.action != CandidateGateAction::Apply
                }),
                "forbidden candidate auto-applied: {resolution:?}"
            );
        }
    }

    #[test]
    fn nanda_candidate_cannot_autogrow_known_phrase_part_either() {
        let gate = gate_candidate("твой ", "тывой ", TypingErrorClass::CompositeTypo);

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "known_phrase_part_one_letter_growth");
    }

    #[test]
    fn nanda_semantic_candidate_cannot_rewrite_known_word_to_neighbor_word() {
        let gate = gate_candidate(
            "искать хрень! ",
            "искать хрену ",
            TypingErrorClass::CompositeTypo,
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "soft_sign_vowel_drift");
    }

    #[test]
    fn semantic_word_cell_far_surface_jumps_are_suggest_only() {
        for (input, replacement) in [
            ("реально помагаешь ", "реально понимаешь "),
            ("она спраивтя ", "она спрашивая "),
        ] {
            assert!(
                semantic_wave_candidate_lacks_surface_authority(
                    input,
                    replacement,
                    crate::nanda_wave::context_wave::SEMANTIC_WORD_SOURCE,
                ),
                "semantic helper must reject {input:?} -> {replacement:?}"
            );
            let gate = gate_candidate_with_source(
                input,
                replacement,
                TypingErrorClass::CompositeTypo,
                crate::nanda_wave::context_wave::SEMANTIC_WORD_SOURCE,
            );

            assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
            assert_eq!(gate.reason, "semantic_wave_surface_authority_low");
        }
    }

    #[test]
    fn nanda_l3_support_cannot_override_live_protected_terms() {
        let pipeline = default_typing_assist_pipeline();
        for input in [
            "это патерн ",
            "в гугле ",
            "блять ",
            "слово грокать ",
            "тоже грокнулся. ",
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicThenNanda,
            ));

            assert_eq!(resolution.decision, None, "input={input:?}: {resolution:?}");
            assert!(
                resolution.candidates.iter().all(|candidate| {
                    candidate.source != CorrectionDecisionSource::Nanda
                        || candidate.gate.action != CandidateGateAction::Apply
                }),
                "NANDA candidate bypassed hard safety: {resolution:?}"
            );
        }
    }

    #[test]
    fn nanda_surface_candidates_from_logs_are_suggest_only_when_surface_is_weak() {
        for (input, replacement, reason) in [
            ("тели ", "тел ", "short_nanda_word_shrink"),
            (
                "нас моного ",
                "нас мюоного ",
                "short_nanda_internal_vowel_growth",
            ),
        ] {
            let gate = gate_candidate_with_source(
                input,
                replacement,
                TypingErrorClass::CompositeTypo,
                "L2SurfaceMotifCell32",
            );

            assert_eq!(
                gate.action,
                CandidateGateAction::SuggestOnly,
                "input={input:?}"
            );
            assert_eq!(gate.reason, reason, "input={input:?}");
        }
    }

    #[test]
    fn composite_typo_splits_previous_glued_word_when_fixing_current_typo() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "ее простозальет свтеом ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "ее просто зальет светом ");
        assert_eq!(selected.source_id, ids::ADJACENT_TRANSPOSITION);
        assert_eq!(
            selected.error_class,
            TypingErrorClass::AdjacentTransposition
        );
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn composite_typo_does_not_glue_two_committed_words() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "реально ое ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert_eq!(resolution.decision, None);
        assert!(!resolution
            .candidates
            .iter()
            .any(|candidate| candidate.replacement == "реальное "));
    }

    #[test]
    fn live_log_multi_word_drifts_do_not_autoreplace_neighbors() {
        let pipeline = default_typing_assist_pipeline();
        for input in ["мете ты ", "тут тоже ", "я позвол ", "мы токенов "]
        {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicThenNanda,
            ));

            assert_eq!(
                resolution.decision, None,
                "multi-word dirty log case must not auto-apply: {input:?}: {resolution:?}"
            );
        }
    }

    #[test]
    fn single_letter_boundary_beats_wrong_transposition_candidate() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "посмотреть влогах ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        let selected = resolution.selected.expect("selected boundary candidate");
        assert_eq!(selected.replacement, "посмотреть в логах ");
        assert_eq!(selected.source_id, "BoundaryCell32");
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
        assert!(resolution.candidates.iter().all(|candidate| {
            candidate.replacement != "посмотреть волгах "
                || candidate.gate.action != CandidateGateAction::Apply
        }));
    }

    #[test]
    fn repeated_letter_repairs_short_all_caps_word() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "ТРУССС ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected candidate, resolution={resolution:?}"));
        assert_eq!(selected.replacement, "ТРУС ");
        assert_eq!(selected.source_id, ids::REPEATED_LETTER);
        assert_eq!(selected.error_class, TypingErrorClass::RepeatedLetter);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn repeated_prefix_plus_letter_substitution_does_not_apply_intermediate_word() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "ППОНИКАЕШЬ? ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert_eq!(resolution.decision, None);
        assert!(resolution.candidates.iter().any(|candidate| {
            candidate.replacement == "ПОНИКАЕШЬ? "
                && candidate.gate.action == CandidateGateAction::SuggestOnly
                && candidate.gate.reason == "single_step_typo_has_competing_composite"
        }));
    }

    #[test]
    fn composite_typo_repairs_short_adjacent_transposition_in_phrase() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "имеет смылс ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "имеет смысл ");
        assert_eq!(selected.source_id, ids::ADJACENT_TRANSPOSITION);
        assert_eq!(
            selected.error_class,
            TypingErrorClass::AdjacentTransposition
        );
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn adjacent_transposition_keeps_already_known_word() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "Ладно ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert!(resolution.selected.is_none());
        assert!(resolution.decision.is_none());
    }

    #[test]
    fn future_auxiliary_blocks_non_infinitive_typo_candidate() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "будет несити ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert!(resolution.selected.is_none());
        assert!(resolution.decision.is_none());
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
    fn nanda_candidate_also_passes_unified_gate() {
        let pipeline = default_typing_assist_pipeline();
        let resolution =
            resolve_text_correction(request("тфтвф ", &pipeline, CorrectionMode::NandaOnly));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "nanda ");
        assert_eq!(selected.error_class, TypingErrorClass::WrongLayout);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn nanda_surface_motif_can_apply_known_typo() {
        let pipeline = default_typing_assist_pipeline();
        let resolution =
            resolve_text_correction(request("звгрузи ", &pipeline, CorrectionMode::NandaOnly));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "загрузи ");
        assert_eq!(selected.source, CorrectionDecisionSource::Nanda);
        assert_eq!(selected.source_id, "L2SurfaceMotifCell32");
        assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn nanda_surface_completion_is_suggest_only_not_autocorrect() {
        let pipeline = default_typing_assist_pipeline();
        let resolution =
            resolve_text_correction(request("делай пров ", &pipeline, CorrectionMode::NandaOnly));

        assert!(resolution.decision.is_none());
        let completion = resolution
            .candidates
            .iter()
            .find(|candidate| candidate.source_id == "L2SurfaceCompletionCell32")
            .expect("completion candidate");
        assert_eq!(completion.error_class, TypingErrorClass::CompletionOnly);
        assert_eq!(completion.gate.action, CandidateGateAction::SuggestOnly);
    }

    #[test]
    fn nanda_corrects_customs_actor_phrase_with_right_anchor() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "Поставщик говорит что цена до склада нашего покупателя но таможен мы! ",
            &pipeline,
            CorrectionMode::NandaOnly,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(
            selected.replacement,
            "Поставщик говорит что цена до склада нашего покупателя но таможим мы! "
        );
        assert_eq!(selected.source, CorrectionDecisionSource::Nanda);
        assert_eq!(selected.source_id, "PhraseCell32");
        assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn nanda_does_not_correct_customs_actor_phrase_without_right_anchor() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "Поставщик говорит что цена до склада нашего покупателя но таможен ",
            &pipeline,
            CorrectionMode::NandaOnly,
        ));

        assert!(resolution.selected.is_none());
        assert!(resolution.decision.is_none());
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
