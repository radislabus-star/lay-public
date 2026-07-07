use super::{
    bayes_score_for_candidate, edit_transition, explanation_for_candidate, CandidateGateAction,
    TypingErrorEvent, UnifiedCorrectionCandidate,
};
use crate::correction_source_contract::{self, CorrectionSourceRole};
use crate::nanda_wave::l3_phrase_gate::{evaluate_default_candidate, L3PhraseGateDecision};
use crate::nanda_wave::l4_goal_state::{derive_l4_scene_state, L4AllowedAction, L4SceneStateInput};
use crate::nanda_wave::l4_signed_memory::{l4_signed_memory_signal, L4SignedMemoryInput};

pub(super) struct CorrectionDecisionCore;

impl CorrectionDecisionCore {
    pub(super) fn select_apply_candidate(
        event: &TypingErrorEvent,
        candidates: &[UnifiedCorrectionCandidate],
    ) -> Option<UnifiedCorrectionCandidate> {
        candidates
            .iter()
            .filter(|candidate| candidate.gate.action == CandidateGateAction::Apply)
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
    let transition = edit_transition::prove_edit_transition(
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
        + transition_rank_bonus(transition, &candidate.source_id)
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
    if !super::l3_phrase_memory_applies_to(candidate.error_class) {
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
            signal: 0.0,
            rank_bonus: 0.0,
            decision: "neutral",
        },
    }
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

fn score_to_milli(value: f32) -> i16 {
    (value * 1000.0)
        .round()
        .clamp(i16::MIN as f32, i16::MAX as f32) as i16
}
