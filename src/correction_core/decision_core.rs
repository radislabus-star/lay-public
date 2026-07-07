use super::{
    bayes_score_for_candidate, edit_transition, explanation_for_candidate, CandidateGateAction,
    TypingErrorEvent, UnifiedCorrectionCandidate,
};
use crate::correction_source_contract::{self, CorrectionSourceRole};

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
                candidate_rank_score(&event.original, left)
                    .total_cmp(&candidate_rank_score(&event.original, right))
            })
    }
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
