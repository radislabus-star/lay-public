// Private resolution authority for the L2 candidate lattice. This file is
// included by `candidate.rs`, so it can use lattice state without widening the
// typing-transition API.

fn resolve_l2_lattice(lattice: L2CandidateLattice) -> CorrectionResolution {
    let selected = TransitionDecisionCore::select_apply_candidate(
        &lattice.event,
        &lattice.candidates,
        lattice.policy,
    );
    let decision = selected.as_ref().map(|candidate| CorrectionDecision {
        replacement: candidate.replacement.clone(),
        source: candidate.source,
    });
    let scoreboard =
        CorrectionScoreboard::from_candidates(&lattice.event, &lattice.candidates, selected.as_ref());
    let candidate_scores = CorrectionCandidateScoreTrace::from_candidates(
        &lattice.event,
        &lattice.candidates,
        selected.as_ref(),
    );

    CorrectionResolution {
        event: lattice.event,
        candidates: lattice.candidates,
        selected,
        decision,
        scoreboard,
        candidate_scores,
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

        let resolution = resolve_l2_lattice(lattice);
        assert!(resolution.selected.is_some());
    }
}
