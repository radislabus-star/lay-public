use super::{
    candidate_rank_score, CandidateGateAction, CorrectionCandidateScoreTrace, CorrectionDecision,
    CorrectionResolution, CorrectionScoreboard, TypingErrorEvent, UnifiedCorrectionCandidate,
};

pub(super) struct L2CandidateLattice {
    event: TypingErrorEvent,
    candidates: Vec<UnifiedCorrectionCandidate>,
}

impl L2CandidateLattice {
    pub(super) fn new(event: TypingErrorEvent) -> Self {
        Self {
            event,
            candidates: Vec::new(),
        }
    }

    pub(super) fn push_source(&mut self, candidate: Option<UnifiedCorrectionCandidate>) {
        if let Some(candidate) = candidate {
            self.candidates.push(candidate);
        }
    }

    fn selected_apply_candidate(&self) -> Option<UnifiedCorrectionCandidate> {
        self.candidates
            .iter()
            .filter(|candidate| candidate.gate.action == CandidateGateAction::Apply)
            .cloned()
            .max_by(|left, right| {
                candidate_rank_score(&self.event.original, left)
                    .total_cmp(&candidate_rank_score(&self.event.original, right))
            })
    }

    pub(super) fn into_resolution(self) -> CorrectionResolution {
        let selected = self.selected_apply_candidate();
        let decision = selected.as_ref().map(|candidate| CorrectionDecision {
            replacement: candidate.replacement.clone(),
            source: candidate.source,
        });
        let scoreboard = CorrectionScoreboard::from_candidates(
            &self.event.original,
            &self.candidates,
            selected.as_ref(),
        );
        let candidate_scores = CorrectionCandidateScoreTrace::from_candidates(
            &self.event.original,
            &self.candidates,
            selected.as_ref(),
        );

        CorrectionResolution {
            event: self.event,
            candidates: self.candidates,
            selected,
            decision,
            scoreboard,
            candidate_scores,
        }
    }
}
