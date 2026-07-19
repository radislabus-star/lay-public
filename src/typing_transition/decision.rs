use super::{
    action,
    proposal_admission::{self, CandidateGateAction, CandidateGateDecision},
    verifier, TypingTransition,
};
use crate::candidate_contract::{CandidateOrigin, CorrectionSourceRole};
use crate::candidate_explanation::CandidateExplanation;
use crate::correction_bayes::BayesCandidateScore;
use crate::correction_core::{
    explanation_for_candidate, CorrectionDecisionSource, TypingErrorClass, TypingErrorEvent,
    UnifiedCorrectionCandidate,
};
use crate::keyboard::is_cyrillic_letter;
use crate::nanda_wave::l3_phrase_gate::L3PhraseGateDecision;
use crate::nanda_wave::l4_goal_state::L4AllowedAction;
use crate::nanda_wave::l4_hidden_state::{
    estimate_hidden_typing_state, predicted_state_id, L4HiddenCandidateInput, L4HiddenDisposition,
};
use crate::nanda_wave::l4_signed_memory::{l4_signed_memory_signal, L4SignedMemoryInput};
use crate::text_edit::TransitionAudit;
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
    pub(crate) fn admit_candidate_proposal(
        original: &str,
        replacement: &str,
        error_class: TypingErrorClass,
        origin: CandidateOrigin,
    ) -> CandidateGateDecision {
        proposal_admission::gate_candidate_with_origin(original, replacement, error_class, origin)
    }

    pub(crate) fn evaluate_candidates(
        event: &TypingErrorEvent,
        candidates: &[UnifiedCorrectionCandidate],
        policy: TransitionDecisionPolicy,
        prepared_l2_peak_context: Option<&crate::nanda_wave::l2_wave_peak::L2CorrectionPeakContext>,
    ) -> CandidateDecisionBatch {
        if candidates.is_empty() {
            return CandidateDecisionBatch {
                evaluations: Vec::new(),
                selected_index: None,
                selected_transition: None,
            };
        }
        let usage = crate::nanda_wave::cached_usage_prior_snapshot();
        let replacements = candidates
            .iter()
            .map(|candidate| candidate.replacement.as_str())
            .collect::<Vec<_>>();
        let l3_reports = crate::nanda_wave::l3_phrase_gate::evaluate_default_candidates(
            &event.original,
            &replacements,
        );
        let owned_l2_peak_context;
        let l2_peak_context = if let Some(context) = prepared_l2_peak_context {
            context
        } else {
            owned_l2_peak_context =
                crate::nanda_wave::l2_wave_peak::prepare_correction_peak_context(&event.original);
            &owned_l2_peak_context
        };
        let mut evaluations = candidates
            .iter()
            .zip(&l3_reports)
            .map(|(candidate, l3_report)| {
                CandidateDecisionEvaluation::build(
                    CandidateDecisionContext {
                        event,
                        candidate_count: candidates.len(),
                        usage: &usage,
                        l2_peak_context,
                        l3_report: l3_report.as_ref(),
                    },
                    candidate,
                )
            })
            .collect::<Vec<_>>();
        settle_transition_interference(candidates, &mut evaluations);
        settle_l4_hidden_state(event, candidates, &mut evaluations);
        if std::env::var_os("LAY_DEBUG_DECISION_CORE").is_some() {
            for (candidate, evaluation) in candidates.iter().zip(&evaluations) {
                eprintln!(
                    "decision-core-candidate origin={:?} source_id={} class={} gate={:?} rank={:.3} field={} attract={} repel={} uncertainty={} phase_competition={} lexical_ready={} operator_consensus={} usage={:.3} context={:.3} l3={} l4={} l4_state_specific={} l4_attract={} l4_repel={} hidden={} hidden_classes={} hidden_selected={} hidden_probe={} hidden_certificate={} replacement={:?}",
                    candidate.origin,
                    candidate.source_id,
                    candidate.error_class.as_str(),
                    candidate.gate.action,
                    evaluation.signals.rank_score,
                    evaluation.signals.transition_field_milli,
                    evaluation.signals.transition_field_attraction_milli,
                    evaluation.signals.transition_field_repulsion_milli,
                    evaluation.signals.transition_field_uncertainty_milli,
                    evaluation
                        .signals
                        .transition_field_phase_competition_milli,
                    evaluation.signals.l2_lexical_phase_competition_ready,
                    verified_operator_consensus_witness(candidate, evaluation),
                    evaluation.bayes.usage_prior,
                    evaluation.bayes.context_prior,
                    evaluation.signals.l3_phrase_milli,
                    evaluation.signals.l4_signed_milli,
                    evaluation.signals.l4_transition_state_specific,
                    evaluation.signals.l4_transition_attract_count,
                    evaluation.signals.l4_transition_repel_count,
                    evaluation.signals.l4_hidden_disposition.as_str(),
                    evaluation.signals.l4_hidden_semantic_classes,
                    evaluation.signals.l4_hidden_selected_class,
                    evaluation.signals.l4_hidden_probe,
                    evaluation.signals.l4_hidden_certificate_valid,
                    candidate.replacement
                );
            }
        }
        let selected_index = candidates
            .iter()
            .enumerate()
            .filter(|(index, candidate)| {
                producer_allows_authority_evaluation(
                    candidate.gate.action,
                    evaluations[*index].transition.l4_signed_signal,
                )
            })
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

fn producer_allows_authority_evaluation(
    action: CandidateGateAction,
    l4_signal: super::L4SignedTransitionSignal,
) -> bool {
    action == CandidateGateAction::Eligible
        || (action == CandidateGateAction::SuggestOnly && l4_signal.exact_positive())
}

fn unresolved_competitor_blocks(
    exact_positive_transition: bool,
    stronger_unresolved_exists: bool,
) -> bool {
    !exact_positive_transition && stronger_unresolved_exists
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
mod hard_structural_veto;
mod interference;
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
    l2_peak_context: &'a crate::nanda_wave::l2_wave_peak::L2CorrectionPeakContext,
    l3_report: Option<&'a crate::nanda_wave::l3_phrase_gate::L3PhraseGateReport>,
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
    non_field_rank_score: f32,
    l2_rank_energy: f32,
    l3_rank_energy: f32,
    l4_signed_rank_energy: f32,
    l2_transition_phase_margin_micro: i64,
    l2_transition_phase_threshold_micro: i64,
    l2_lexical_phase_margin_micro: i64,
    l2_lexical_phase_threshold_micro: i64,
    l2_lexical_phase_competition_ready: bool,
    pub(crate) rank_score: f32,
    pub(crate) rank_milli: i16,
    pub(crate) transition_field_milli: i16,
    pub(crate) transition_field_attraction_milli: i16,
    pub(crate) transition_field_repulsion_milli: i16,
    pub(crate) transition_field_uncertainty_milli: i16,
    pub(crate) transition_field_phase_competition_milli: i16,
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
    pub(crate) l3_relation_class: u64,
    pub(crate) l4_hidden_disposition: L4HiddenDisposition,
    pub(crate) l4_hidden_semantic_classes: u16,
    pub(crate) l4_hidden_unresolved_classes: u16,
    pub(crate) l4_hidden_selected_class: u64,
    pub(crate) l4_hidden_class_margin_milli: i16,
    pub(crate) l4_hidden_witness_count: u32,
    pub(crate) l4_hidden_ambiguity_authoritative: bool,
    pub(crate) l4_hidden_selected_witnessed: bool,
    pub(crate) l4_hidden_plan_commitment: u64,
    pub(crate) l4_hidden_receipts: u8,
    pub(crate) l4_hidden_probe: &'static str,
    pub(crate) l4_hidden_certificate_valid: bool,
    pub(crate) l4_scene_milli: i16,
    pub(crate) l4_scene_action: L4AllowedAction,
    pub(crate) l4_scene_reason: &'static str,
    pub(crate) l4_signed_milli: i16,
    pub(crate) l4_signed_reason: &'static str,
    pub(crate) l4_surface_status: crate::nanda_wave::l4_signed_memory::L4SurfaceStatus,
    pub(crate) l4_transition_state_specific: bool,
    pub(crate) l4_transition_attract_count: u32,
    pub(crate) l4_transition_repel_count: u32,
    pub(crate) l4_phase_witness_milli: i16,
    pub(crate) l4_phase_witness_supported: bool,
    pub(crate) l4_phase_positive_centers: u8,
    pub(crate) l4_phase_negative_centers: u8,
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
    let l3 = l3_phrase_signal(candidate.error_class, context.l3_report);
    let phase = crate::nanda_wave::l2_transition_phase_readout(
        action.operator.as_str(),
        relation.atoms(),
        &event.original,
        &candidate.replacement,
    );
    let l4_signed = l4_signed_signal_from_memory(l4_memory);
    let l2_wave_peak = l2_wave_peak_signal(
        candidate,
        context.candidate_count,
        phase,
        context.usage,
        context.l2_peak_context,
    );
    let non_field_rank_score = bayes.posterior
        + ((explanation.explanation_score_milli as f32 - 500.0) / 2_000.0)
        + transition_rank_bonus(&action, candidate)
        + ((candidate.evidence_count().saturating_sub(1).min(3) as f32) * 0.025);
    let transition_field =
        transition_interference_readout(l2_wave_peak, phase, l3, l4_signed, None);
    let rank_score = non_field_rank_score + transition_field.signal;

    CandidateDecisionSignals {
        non_field_rank_score,
        l2_rank_energy: l2_wave_peak.rank_energy,
        l3_rank_energy: l3.rank_energy,
        l4_signed_rank_energy: l4_signed.rank_energy,
        l2_transition_phase_margin_micro: phase.margin_micro,
        l2_transition_phase_threshold_micro: phase.threshold_micro,
        l2_lexical_phase_margin_micro: phase.lexical_margin_micro,
        l2_lexical_phase_threshold_micro: phase.lexical_threshold_micro,
        l2_lexical_phase_competition_ready: phase.lexical_competition_ready,
        rank_score,
        rank_milli: score_to_milli(rank_score),
        transition_field_milli: score_to_milli(transition_field.signal),
        transition_field_attraction_milli: score_to_milli(transition_field.attraction),
        transition_field_repulsion_milli: score_to_milli(transition_field.repulsion),
        transition_field_uncertainty_milli: score_to_milli(transition_field.uncertainty),
        transition_field_phase_competition_milli: score_to_milli(
            transition_field.phase_competition,
        ),
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
        l3_relation_class: l3.relation_class,
        l4_hidden_disposition: L4HiddenDisposition::Unobserved,
        l4_hidden_semantic_classes: 0,
        l4_hidden_unresolved_classes: 0,
        l4_hidden_selected_class: 0,
        l4_hidden_class_margin_milli: 0,
        l4_hidden_witness_count: 0,
        l4_hidden_ambiguity_authoritative: false,
        l4_hidden_selected_witnessed: false,
        l4_hidden_plan_commitment: 0,
        l4_hidden_receipts: 0,
        l4_hidden_probe: "none",
        l4_hidden_certificate_valid: false,
        l4_scene_milli: 0,
        l4_scene_action: L4AllowedAction::Wait,
        l4_scene_reason: "hidden_state_unobserved",
        l4_signed_milli: score_to_milli(l4_signed.signal),
        l4_signed_reason: l4_signed.reason,
        l4_surface_status: l4_signed.surface_status,
        l4_transition_state_specific: l4_signed.transition_state_specific,
        l4_transition_attract_count: l4_signed.transition_attract_count,
        l4_transition_repel_count: l4_signed.transition_repel_count,
        l4_phase_witness_milli: l4_signed.phase_witness_milli,
        l4_phase_witness_supported: l4_signed.phase_witness_supported,
        l4_phase_positive_centers: l4_signed.phase_positive_centers,
        l4_phase_negative_centers: l4_signed.phase_negative_centers,
    }
}

include!("decision_signals.rs");

#[cfg(test)]
mod tests;
