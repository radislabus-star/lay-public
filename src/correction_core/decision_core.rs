use super::{
    action_operator, bayes_score_for_candidate, edit_transition, explanation_for_candidate,
    CandidateGateAction, TypingErrorEvent, UnifiedCorrectionCandidate,
};
use crate::correction_source_contract::{self, CorrectionSourceRole};
use crate::nanda_wave::l3_phrase_gate::{evaluate_default_candidate, L3PhraseGateDecision};
use crate::nanda_wave::l4_goal_state::{derive_l4_scene_state, L4AllowedAction, L4SceneStateInput};
use crate::nanda_wave::l4_signed_memory::{l4_signed_memory_signal, L4SignedMemoryInput};
use crate::text_metrics::damerau_levenshtein;
use crate::word_reader::split_word_punctuation;

pub(super) struct CorrectionDecisionCore;

impl CorrectionDecisionCore {
    pub(super) fn select_apply_candidate(
        event: &TypingErrorEvent,
        candidates: &[UnifiedCorrectionCandidate],
    ) -> Option<UnifiedCorrectionCandidate> {
        candidates
            .iter()
            .filter(|candidate| candidate.gate.action == CandidateGateAction::Apply)
            .filter(|candidate| candidate_has_apply_authority(event, candidate, candidates))
            .cloned()
            .max_by(|left, right| {
                candidate_decision_signals(event, left, candidates.len())
                    .rank_score
                    .total_cmp(
                        &candidate_decision_signals(event, right, candidates.len()).rank_score,
                    )
            })
    }
}

fn candidate_has_apply_authority(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    candidates: &[UnifiedCorrectionCandidate],
) -> bool {
    let bayes = bayes_score_for_candidate(&event.original, candidate);
    let source_role = if matches!(
        candidate.error_class,
        super::TypingErrorClass::WrongLayout
            | super::TypingErrorClass::PartialLayout
            | super::TypingErrorClass::MixedScript
    ) {
        CorrectionSourceRole::Layout
    } else {
        correction_source_contract::source_role(&candidate.source_id)
    };
    let action = action_operator::verify_action_operator(
        &event.original,
        &candidate.replacement,
        candidate.error_class,
        &candidate.source_id,
    );
    if !action.verifier_passed
        && !matches!(
            source_role,
            CorrectionSourceRole::Layout
                | CorrectionSourceRole::Boundary
                | CorrectionSourceRole::DeterministicTypo
        )
    {
        debug_decision_reject(
            candidate,
            "unverified_transition",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    let signals = candidate_decision_signals(event, candidate, candidates.len());
    let strong_learned_support = bayes.usage_prior >= 0.080
        || bayes.context_prior >= 0.080
        || signals.l3_phrase_milli >= 420
        || signals.l4_signed_milli >= 120;
    if bayes.risk >= 0.62
        && !matches!(
            source_role,
            CorrectionSourceRole::Layout
                | CorrectionSourceRole::Boundary
                | CorrectionSourceRole::DeterministicTypo
        )
    {
        debug_decision_reject(candidate, "high_risk", bayes.posterior, bayes.risk);
        return false;
    }
    let posterior_floor = match source_role {
        CorrectionSourceRole::Layout => 0.0,
        CorrectionSourceRole::Boundary | CorrectionSourceRole::DeterministicTypo => 0.20,
        CorrectionSourceRole::L3Context => 0.28,
        CorrectionSourceRole::L2Surface | CorrectionSourceRole::Unknown => 0.34,
        CorrectionSourceRole::Completion | CorrectionSourceRole::Technical => 0.40,
    };
    if bayes.posterior < posterior_floor && !strong_learned_support {
        debug_decision_reject(candidate, "low_posterior", bayes.posterior, bayes.risk);
        return false;
    }
    if source_role == CorrectionSourceRole::L2Surface
        && !strong_learned_support
        && short_same_length_surface_drift(&event.current_word, &candidate.replacement)
    {
        debug_decision_reject(
            candidate,
            "short_same_length_surface_drift",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    let allowed = !matches!(
        source_role,
        CorrectionSourceRole::L2Surface
            | CorrectionSourceRole::L3Context
            | CorrectionSourceRole::Unknown
    ) || !better_non_apply_candidate_exists(event, candidate, candidates);
    if !allowed {
        debug_decision_reject(candidate, "better_non_apply", bayes.posterior, bayes.risk);
    }
    allowed
}

fn short_same_length_surface_drift(original_word: &str, replacement: &str) -> bool {
    let Some(replacement_word) = last_replacement_word(replacement) else {
        return false;
    };
    let original = original_word.to_lowercase();
    let replacement = replacement_word.to_lowercase();
    let original_len = original.chars().count();
    original_len <= 6
        && original_len == replacement.chars().count()
        && original.chars().all(is_russian_letter)
        && replacement.chars().all(is_russian_letter)
        && damerau_levenshtein(&original, &replacement) <= 2
}

fn last_replacement_word(text: &str) -> Option<String> {
    text.split_whitespace().rev().find_map(|token| {
        let (_, word, _) = split_word_punctuation(token);
        (!word.is_empty()).then(|| word.to_string())
    })
}

fn is_russian_letter(ch: char) -> bool {
    matches!(ch, 'а'..='я' | 'ё')
}

fn debug_decision_reject(
    candidate: &UnifiedCorrectionCandidate,
    reason: &'static str,
    posterior: f32,
    risk: f32,
) {
    if std::env::var_os("LAY_DEBUG_DECISION_CORE").is_some() {
        eprintln!(
            "decision-core-reject reason={reason} source_id={} class={} replacement={:?} posterior={:.3} risk={:.3}",
            candidate.source_id,
            candidate.error_class.as_str(),
            candidate.replacement,
            posterior,
            risk
        );
    }
}

fn better_non_apply_candidate_exists(
    event: &TypingErrorEvent,
    selected: &UnifiedCorrectionCandidate,
    candidates: &[UnifiedCorrectionCandidate],
) -> bool {
    let selected_bayes = bayes_score_for_candidate(&event.original, selected);
    candidates.iter().any(|candidate| {
        if candidate == selected || candidate.gate.action == CandidateGateAction::Veto {
            return false;
        }
        let candidate_bayes = bayes_score_for_candidate(&event.original, candidate);
        candidate_bayes.risk <= selected_bayes.risk
            && candidate_bayes.posterior >= selected_bayes.posterior + 0.12
    })
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct CandidateDecisionSignals {
    pub(super) rank_score: f32,
    pub(super) rank_milli: i16,
    pub(super) l3_phrase_milli: i16,
    pub(super) l3_phrase_decision: &'static str,
    pub(super) l4_scene_milli: i16,
    pub(super) l4_scene_action: &'static str,
    pub(super) l4_scene_reason: &'static str,
    pub(super) l4_signed_milli: i16,
    pub(super) l4_signed_reason: &'static str,
}

pub(super) fn candidate_decision_signals(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    candidate_count: usize,
) -> CandidateDecisionSignals {
    let bayes = bayes_score_for_candidate(&event.original, candidate).posterior;
    let explanation = explanation_for_candidate(&event.original, candidate);
    let action = action_operator::verify_action_operator(
        &event.original,
        &candidate.replacement,
        candidate.error_class,
        &candidate.source_id,
    );
    let l3 = l3_phrase_signal(event, candidate);
    let l4_scene = l4_scene_signal(event, candidate_count);
    let l4_signed = l4_signed_signal(event, candidate);
    let rank_score = bayes
        + ((explanation.explanation_score_milli as f32 - 500.0) / 10_000.0)
        + transition_rank_bonus(&action, &candidate.source_id)
        + l3.rank_bonus
        + l4_scene.rank_bonus
        + l4_signed.rank_bonus;

    CandidateDecisionSignals {
        rank_score,
        rank_milli: score_to_milli(rank_score),
        l3_phrase_milli: score_to_milli(l3.signal),
        l3_phrase_decision: l3.decision,
        l4_scene_milli: score_to_milli(l4_scene.signal),
        l4_scene_action: l4_scene.action,
        l4_scene_reason: l4_scene.reason,
        l4_signed_milli: score_to_milli(l4_signed.signal),
        l4_signed_reason: l4_signed.reason,
    }
}

#[derive(Debug, Clone, Copy)]
struct L3Signal {
    signal: f32,
    rank_bonus: f32,
    decision: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct L4SceneSignal {
    signal: f32,
    rank_bonus: f32,
    action: &'static str,
    reason: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct L4SignedSignal {
    signal: f32,
    rank_bonus: f32,
    reason: &'static str,
}

fn l3_phrase_signal(event: &TypingErrorEvent, candidate: &UnifiedCorrectionCandidate) -> L3Signal {
    if !l3_phrase_signal_observes(candidate.error_class) {
        return L3Signal {
            signal: 0.0,
            rank_bonus: 0.0,
            decision: "not_applicable",
        };
    }
    let Some(report) = evaluate_default_candidate(&event.original, &candidate.replacement) else {
        return L3Signal {
            signal: 0.0,
            rank_bonus: 0.0,
            decision: "no_memory",
        };
    };
    match report.decision {
        L3PhraseGateDecision::Support => {
            let signal = report.score.clamp(0.0, 1.0);
            L3Signal {
                signal,
                rank_bonus: signal * 0.16,
                decision: "support",
            }
        }
        L3PhraseGateDecision::Suppress => L3Signal {
            signal: -0.56,
            rank_bonus: -0.14,
            decision: "suppress",
        },
        L3PhraseGateDecision::Neutral => L3Signal {
            signal: (report.score * 0.20).clamp(0.0, 0.20),
            rank_bonus: 0.0,
            decision: "neutral",
        },
    }
}

fn l3_phrase_signal_observes(error_class: super::TypingErrorClass) -> bool {
    !matches!(
        error_class,
        super::TypingErrorClass::CompletionOnly
            | super::TypingErrorClass::TechnicalToken
            | super::TypingErrorClass::ProtectedToken
            | super::TypingErrorClass::Unknown
    )
}

fn l4_scene_signal(event: &TypingErrorEvent, candidate_count: usize) -> L4SceneSignal {
    let scene = derive_l4_scene_state(L4SceneStateInput {
        context_prefix: &event.core,
        current_word: &event.current_word,
        candidate_count,
    });
    let signal = match scene.allowed_action {
        L4AllowedAction::Suggest => scene.confidence,
        L4AllowedAction::Wait => -scene.confidence * 0.50,
        L4AllowedAction::Block => -scene.confidence,
    };
    L4SceneSignal {
        signal,
        rank_bonus: signal * 0.06,
        action: scene.allowed_action.as_str(),
        reason: scene.reason,
    }
}

fn l4_signed_signal(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
) -> L4SignedSignal {
    let mut context = super::normalized_correction_words(&event.original);
    context.pop();
    let word = super::normalized_correction_words(&candidate.replacement)
        .pop()
        .unwrap_or_default();
    if word.is_empty() {
        return L4SignedSignal {
            signal: 0.0,
            rank_bonus: 0.0,
            reason: "learned_state_empty",
        };
    }
    let usage = crate::nanda_wave::cached_usage_prior_snapshot();
    let signed = l4_signed_memory_signal(L4SignedMemoryInput {
        context: &context,
        source: &candidate.source_id,
        operation: "replacement",
        word: &word,
        usage: &usage,
    });
    L4SignedSignal {
        signal: signed.signed_weight,
        rank_bonus: signed.signed_weight * 0.12,
        reason: signed.reason,
    }
}

fn transition_rank_bonus(
    action: &action_operator::CorrectionActionOperatorReport,
    source_id: &str,
) -> f32 {
    if !action.verifier_passed {
        return -0.20;
    }
    match action.edit_operator {
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

fn score_to_milli(value: f32) -> i16 {
    (value * 1000.0)
        .round()
        .clamp(i16::MIN as f32, i16::MAX as f32) as i16
}
