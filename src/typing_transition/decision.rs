use super::{action, verifier, TypingTransition};
use crate::candidate_contract::CorrectionSourceRole;
use crate::candidate_explanation::CandidateExplanation;
use crate::correction_bayes::BayesCandidateScore;
use crate::correction_core::{
    explanation_for_candidate, CandidateGateAction, CorrectionDecisionSource, TypingErrorClass,
    TypingErrorEvent, UnifiedCorrectionCandidate,
};
use crate::keyboard::is_cyrillic_letter;
use crate::nanda_wave::l3_phrase_gate::{evaluate_default_candidate, L3PhraseGateDecision};
use crate::nanda_wave::l4_goal_state::{derive_l4_scene_state, L4AllowedAction, L4SceneStateInput};
use crate::nanda_wave::l4_signed_memory::{l4_signed_memory_signal, L4SignedMemoryInput};
use crate::text_edit::{
    plan_decision_transition_edit, tail_chars, DecisionTransitionEditInput,
    LatentTextTransitionCandidate, TextReplacement, TextTransitionDecision,
    TextTransitionRejection, TransitionAudit, VisibleFieldState,
};
use crate::text_metrics::{damerau_levenshtein, score_to_milli};
use crate::transition_relation::{TransitionRelationAtoms, TransitionRelationInput};
use crate::word_reader::split_word_punctuation;
use std::cmp::Ordering;

pub(crate) struct TransitionDecisionCore;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TransitionDecisionPolicy {
    pub(crate) l2_phase_apply: bool,
}

impl TransitionDecisionCore {
    pub(crate) fn decide_visible_text_transition(
        state: &VisibleFieldState,
        candidate: LatentTextTransitionCandidate,
    ) -> TextTransitionDecision {
        if candidate.delete_chars == 0 && candidate.insert_text.is_empty() {
            return TextTransitionDecision::Reject {
                rejection: TextTransitionRejection::Noop,
                action: None,
            };
        }

        if !candidate.insert_text.is_empty() && state.visible_tail.ends_with(&candidate.insert_text)
        {
            return TextTransitionDecision::AlreadyApplied;
        }

        let original_text = tail_chars(&state.visible_tail, candidate.delete_chars as usize);
        if let Some(expected) = candidate.expected_tail.as_ref() {
            let focus_id = state.focus_id.as_deref();
            if expected.source != candidate.source || expected.focus_id.as_deref() != focus_id {
                return TextTransitionDecision::Reject {
                    rejection: TextTransitionRejection::StaleVisibleTail {
                        expected: expected.expected_suffix.clone(),
                        actual: original_text,
                    },
                    action: None,
                };
            }
            if expected.epoch != state.epoch {
                return TextTransitionDecision::Reject {
                    rejection: TextTransitionRejection::StaleVisibleRevision {
                        expected: expected.epoch,
                        actual: state.epoch,
                    },
                    action: None,
                };
            }
            if !expected
                .matches_current_suffix(&state.visible_tail, candidate.delete_chars as usize)
            {
                return TextTransitionDecision::Reject {
                    rejection: TextTransitionRejection::StaleVisibleTail {
                        expected: expected.expected_suffix.clone(),
                        actual: original_text,
                    },
                    action: None,
                };
            }
        }

        if state.external_selection_active {
            return TextTransitionDecision::Reject {
                rejection: TextTransitionRejection::StaleSurroundingText {
                    expected: original_text,
                    actual: state
                        .external_tail_before_cursor
                        .clone()
                        .unwrap_or_default(),
                },
                action: None,
            };
        }

        if state.external_state_present {
            let actual = state
                .external_tail_before_cursor
                .clone()
                .unwrap_or_default();
            if actual != original_text {
                return TextTransitionDecision::Reject {
                    rejection: TextTransitionRejection::StaleSurroundingText {
                        expected: original_text,
                        actual,
                    },
                    action: None,
                };
            }
        }

        let plan = TextReplacement {
            move_left: 0,
            backspaces: candidate.delete_chars,
            insert: candidate.insert_text.clone(),
            move_right: 0,
        };
        let transition = TransitionAudit::proven(
            candidate.intent.operator(),
            candidate.intent.proof(),
            true,
            false,
            1,
        );
        let receipt = DecisionTransitionReceipt::issue(
            original_text.clone(),
            candidate.insert_text.clone(),
            transition,
        );
        let action = plan_decision_transition_edit(
            DecisionTransitionEditInput {
                source: "ibus-committed-tail",
                confidence_milli: 1000,
                from_text: &original_text,
                to_text: &candidate.insert_text,
                plan: plan.clone(),
                selected_source_id: Some(candidate.source.source_id()),
                selected_error_class: None,
            },
            &receipt,
        );
        if !action.allow_apply() {
            return TextTransitionDecision::Reject {
                rejection: TextTransitionRejection::UnsafeEdit {
                    reason: action.safety_reason(),
                },
                action: Some(action),
            };
        }

        TextTransitionDecision::Apply { plan, action }
    }

    pub(crate) fn evaluate_candidates(
        event: &TypingErrorEvent,
        candidates: &[UnifiedCorrectionCandidate],
        policy: TransitionDecisionPolicy,
    ) -> CandidateDecisionBatch {
        let usage = crate::nanda_wave::cached_usage_prior_snapshot();
        let l4_scene = l4_scene_signal(event, candidates.len());
        let context = CandidateDecisionContext {
            event,
            candidate_count: candidates.len(),
            usage: &usage,
            l4_scene,
        };
        let evaluations = candidates
            .iter()
            .map(|candidate| CandidateDecisionEvaluation::build(context, candidate))
            .collect::<Vec<_>>();
        if std::env::var_os("LAY_DEBUG_DECISION_CORE").is_some() {
            for (candidate, evaluation) in candidates.iter().zip(&evaluations) {
                eprintln!(
                    "decision-core-candidate origin={:?} source_id={} class={} gate={:?} rank={:.3} usage={:.3} context={:.3} l3={} l4={} replacement={:?}",
                    candidate.origin,
                    candidate.source_id,
                    candidate.error_class.as_str(),
                    candidate.gate.action,
                    evaluation.signals.rank_score,
                    evaluation.bayes.usage_prior,
                    evaluation.bayes.context_prior,
                    evaluation.signals.l3_phrase_milli,
                    evaluation.signals.l4_signed_milli,
                    candidate.replacement
                );
            }
        }
        let selected_index = candidates
            .iter()
            .enumerate()
            .filter(|(_, candidate)| candidate.gate.action == CandidateGateAction::Eligible)
            .filter(|(index, _)| {
                candidate_has_apply_authority(event, *index, candidates, &evaluations, policy)
            })
            .max_by(|(left, _), (right, _)| {
                compare_candidate_decision_order(*left, *right, candidates, &evaluations)
            })
            .map(|(index, _)| index);

        let selected_transition = selected_index.map(|index| {
            DecisionTransitionReceipt::from_selected_candidate(
                event,
                &candidates[index],
                &evaluations[index],
            )
        });

        CandidateDecisionBatch {
            evaluations,
            selected_index,
            selected_transition,
        }
    }
}

fn compare_candidate_decision_order(
    left: usize,
    right: usize,
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &[CandidateDecisionEvaluation],
) -> Ordering {
    let left_eval = &evaluations[left];
    let right_eval = &evaluations[right];
    left_eval
        .signals
        .rank_score
        .total_cmp(&right_eval.signals.rank_score)
        .then_with(|| right_eval.bayes.risk.total_cmp(&left_eval.bayes.risk))
        .then_with(|| {
            left_eval
                .action
                .verifier_passed
                .cmp(&right_eval.action.verifier_passed)
        })
        .then_with(|| {
            right_eval
                .action
                .changed_tokens
                .cmp(&left_eval.action.changed_tokens)
        })
        .then_with(|| {
            candidates[right]
                .replacement
                .cmp(&candidates[left].replacement)
        })
}

mod calibration;
mod receipt;
pub(crate) use receipt::DecisionTransitionReceipt;

#[derive(Debug, Clone)]
pub(crate) struct CandidateDecisionEvaluation {
    pub(crate) bayes: BayesCandidateScore,
    pub(crate) explanation: CandidateExplanation,
    pub(crate) action: action::CorrectionActionOperatorReport,
    pub(crate) signals: CandidateDecisionSignals,
    pub(crate) transition: TypingTransition,
}

#[derive(Clone, Copy)]
struct CandidateDecisionContext<'a> {
    event: &'a TypingErrorEvent,
    candidate_count: usize,
    usage: &'a crate::nanda_wave::UsagePriorSnapshot,
    l4_scene: L4SceneSignal,
}

struct CandidateSignalReadouts<'a> {
    context: CandidateDecisionContext<'a>,
    candidate: &'a UnifiedCorrectionCandidate,
    bayes: &'a BayesCandidateScore,
    explanation: CandidateExplanation,
    action: action::CorrectionActionOperatorReport,
    relation: &'a TransitionRelationAtoms,
    l4_memory: &'a crate::nanda_wave::l4_signed_memory::L4SignedMemorySignal,
}

impl CandidateDecisionEvaluation {
    fn build(
        context: CandidateDecisionContext<'_>,
        candidate: &UnifiedCorrectionCandidate,
    ) -> Self {
        let event = context.event;
        let usage = context.usage;
        let explanation = explanation_for_candidate(&event.original, candidate);
        let action = action::verify_action_operator(
            &event.original,
            &candidate.replacement,
            candidate.error_class,
            candidate.origin,
        );
        let relation = TransitionRelationAtoms::encode(
            &event.original,
            &candidate.replacement,
            TransitionRelationInput {
                action_operator: action.operator.as_str(),
                edit_operator: action.edit_operator.as_str(),
                proof: action.edit_proof.as_str(),
                verifier_passed: action.verifier_passed,
                left_context_changed: action.left_context_changed,
                changed_tokens: action.changed_tokens,
            },
        );
        let l4_memory = l4_signed_memory_readout(event, candidate, relation.surface_key(), usage);
        let bayes = crate::correction_bayes::bayes_score_candidate_with_readout(
            &event.original,
            &candidate.replacement,
            candidate.error_class.as_str(),
            candidate.origin,
            usage,
            &l4_memory,
        );
        let signals = candidate_decision_signals_from_readouts(CandidateSignalReadouts {
            context,
            candidate,
            bayes: &bayes,
            explanation,
            action,
            relation: &relation,
            l4_memory: &l4_memory,
        });
        let transition =
            TypingTransition::from_evaluated_candidate(super::EvaluatedTransitionInput {
                original: &event.original,
                replacement: &candidate.replacement,
                error_class: candidate.error_class,
                origin: candidate.origin,
                source_id: &candidate.source_id,
                candidate_count: context.candidate_count,
                action,
                l4_signed_signal: signals.l4_transition_signal(),
            });
        Self {
            bayes,
            explanation,
            action,
            signals,
            transition,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CandidateDecisionBatch {
    pub(crate) evaluations: Vec<CandidateDecisionEvaluation>,
    pub(crate) selected_index: Option<usize>,
    pub(crate) selected_transition: Option<DecisionTransitionReceipt>,
}

mod admission;
use admission::candidate_has_apply_authority;
#[cfg(test)]
use admission::{
    admit_evaluated_hidden_transition, known_word_drift_has_authority, phase_policy_rejection,
    TransitionAdmission,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CandidateDecisionSignals {
    pub(crate) rank_score: f32,
    pub(crate) rank_milli: i16,
    pub(crate) l2_wave_peak_milli: i16,
    pub(crate) l2_wave_peak_positive_milli: i16,
    pub(crate) l2_wave_peak_negative_milli: i16,
    pub(crate) l2_wave_peak_uncertainty_milli: i16,
    pub(crate) l2_wave_peak_reason: &'static str,
    pub(crate) l2_transition_phase_milli: i16,
    pub(crate) l2_transition_phase_threshold_milli: i16,
    pub(crate) l2_transition_phase_verdict: crate::nanda_wave::PhaseVerdict,
    pub(crate) l2_transition_phase_package_loaded: bool,
    pub(crate) l2_transition_phase_operator_present: bool,
    pub(crate) l2_transition_phase_operator_promoted: bool,
    pub(crate) l2_transition_phase_positive_centers: u8,
    pub(crate) l2_transition_phase_anti_centers: u8,
    pub(crate) l2_transition_phase_surfaces: u32,
    pub(crate) l3_phrase_milli: i16,
    pub(crate) l3_phrase_decision: L3ContextDisposition,
    pub(crate) l4_scene_milli: i16,
    pub(crate) l4_scene_action: L4AllowedAction,
    pub(crate) l4_scene_reason: &'static str,
    pub(crate) l4_signed_milli: i16,
    pub(crate) l4_signed_reason: &'static str,
    pub(crate) l4_surface_status: crate::nanda_wave::l4_signed_memory::L4SurfaceStatus,
    pub(crate) l4_transition_state_specific: bool,
    pub(crate) l4_transition_attract_count: u32,
    pub(crate) l4_transition_repel_count: u32,
}

impl CandidateDecisionSignals {
    fn l4_transition_signal(&self) -> super::L4SignedTransitionSignal {
        let exact_positive = self.l4_transition_state_specific
            && self.l4_transition_attract_count > self.l4_transition_repel_count;
        let exact_negative = self.l4_transition_state_specific
            && self.l4_transition_repel_count > self.l4_transition_attract_count;
        super::L4SignedTransitionSignal {
            negative: exact_negative || (!exact_positive && self.l4_signed_milli <= -450),
            state_specific: self.l4_transition_state_specific,
            attract_count: self.l4_transition_attract_count,
            repel_count: self.l4_transition_repel_count,
        }
    }
}

fn candidate_decision_signals_from_readouts(
    readouts: CandidateSignalReadouts<'_>,
) -> CandidateDecisionSignals {
    let CandidateSignalReadouts {
        context,
        candidate,
        bayes,
        explanation,
        action,
        relation,
        l4_memory,
    } = readouts;
    let event = context.event;
    let l4_scene = context.l4_scene;
    let l3 = l3_phrase_signal(event, candidate);
    let phase =
        crate::nanda_wave::l2_transition_phase_readout(action.operator.as_str(), relation.atoms());
    let l4_signed = l4_signed_signal_from_memory(l4_memory);
    let l2_wave_peak = l2_wave_peak_signal(
        event,
        candidate,
        context.candidate_count,
        phase,
        context.usage,
    );
    let rank_score = bayes.posterior
        + ((explanation.explanation_score_milli as f32 - 500.0) / 2_000.0)
        + transition_rank_bonus(&action, candidate)
        + ((candidate.evidence_count().saturating_sub(1).min(3) as f32) * 0.025)
        + l2_wave_peak.rank_bonus
        + l3.rank_bonus
        + l4_scene.rank_bonus
        + l4_signed.rank_bonus;

    CandidateDecisionSignals {
        rank_score,
        rank_milli: score_to_milli(rank_score),
        l2_wave_peak_milli: score_to_milli(l2_wave_peak.signal),
        l2_wave_peak_positive_milli: l2_wave_peak.positive_milli,
        l2_wave_peak_negative_milli: l2_wave_peak.negative_milli,
        l2_wave_peak_uncertainty_milli: l2_wave_peak.uncertainty_milli,
        l2_wave_peak_reason: l2_wave_peak.reason,
        l2_transition_phase_milli: l2_wave_peak.transition_phase_milli,
        l2_transition_phase_threshold_milli: l2_wave_peak.transition_phase_threshold_milli,
        l2_transition_phase_verdict: l2_wave_peak.transition_phase_verdict,
        l2_transition_phase_package_loaded: l2_wave_peak.transition_phase_package_loaded,
        l2_transition_phase_operator_present: l2_wave_peak.transition_phase_operator_present,
        l2_transition_phase_operator_promoted: l2_wave_peak.transition_phase_operator_promoted,
        l2_transition_phase_positive_centers: l2_wave_peak.transition_phase_positive_centers,
        l2_transition_phase_anti_centers: l2_wave_peak.transition_phase_anti_centers,
        l2_transition_phase_surfaces: l2_wave_peak.transition_phase_surfaces,
        l3_phrase_milli: score_to_milli(l3.signal),
        l3_phrase_decision: l3.decision,
        l4_scene_milli: score_to_milli(l4_scene.signal),
        l4_scene_action: l4_scene.action,
        l4_scene_reason: l4_scene.reason,
        l4_signed_milli: score_to_milli(l4_signed.signal),
        l4_signed_reason: l4_signed.reason,
        l4_surface_status: l4_signed.surface_status,
        l4_transition_state_specific: l4_signed.transition_state_specific,
        l4_transition_attract_count: l4_signed.transition_attract_count,
        l4_transition_repel_count: l4_signed.transition_repel_count,
    }
}

include!("decision_signals.rs");

#[cfg(test)]
mod tests;
