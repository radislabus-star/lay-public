// Private resolution authority for the L2 candidate lattice. This file is
// included by `candidate.rs`, so it can use lattice state without widening the
// typing-transition API.

fn resolve_l2_lattice(
    mut lattice: L2CandidateLattice,
    peak_context: Option<&crate::nanda_wave::l2_wave_peak::L2CorrectionPeakContext>,
) -> CorrectionResolution {
    crate::nanda_wave::l2_field::bridge::apply_authority_to_candidate_lattice(
        &mut lattice.candidates,
        &lattice.l2_field_authority,
    );
    let decision_batch = TransitionDecisionCore::evaluate_candidates(
        &lattice.event,
        &lattice.candidates,
        lattice.policy,
        peak_context,
    );
    let selected = decision_batch
        .selected_index
        .and_then(|index| lattice.candidates.get(index))
        .cloned();
    let decision = selected.as_ref().map(|candidate| CorrectionDecision {
        replacement: candidate.replacement.clone(),
        source: candidate.source,
    });
    let scoreboard = CorrectionScoreboard::from_candidates(&lattice.candidates, &decision_batch);
    let candidate_scores =
        CorrectionCandidateScoreTrace::from_decision_batch(&lattice.candidates, &decision_batch);
    let selected_transition = decision_batch.selected_transition.clone();

    CorrectionResolution {
        event: lattice.event,
        candidates: lattice.candidates,
        selected,
        decision,
        scoreboard,
        candidate_scores,
        selected_transition,
    }
}

#[cfg(test)]
mod resolution_tests {
    use super::*;
    use crate::correction_core::{
        CandidateGateAction, CandidateGateDecision, CorrectionDecisionSource, TypingErrorClass,
        TypingErrorEvent, UnifiedCorrectionCandidate,
    };
    use crate::candidate_contract::CandidateOrigin;

    #[test]
    fn private_authority_is_the_only_lattice_to_resolution_path() {
        let mut lattice = L2CandidateLattice::new(TypingErrorEvent {
            original: "провека ".to_string(),
            core: "провека".to_string(),
            current_word: "провека".to_string(),
            input_class: TypingErrorClass::MissingLetter,
        });
        lattice.push_source(Some(UnifiedCorrectionCandidate::new(
            "проверка ",
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::L2Surface,
            "L2SurfaceMotifCell32",
            TypingErrorClass::MissingLetter,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "test",
            },
        )));

        let resolution = lattice.into_resolution();
        assert!(resolution.selected.is_some());
    }
}
