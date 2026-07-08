use super::decision::TransitionDecisionCore;
use crate::correction_core::{
    CorrectionCandidateScoreTrace, CorrectionDecision, CorrectionDecisionSource,
    CorrectionResolution, CorrectionScoreboard, TypingErrorEvent, UnifiedCorrectionCandidate,
};

pub(crate) struct L2CandidateLattice {
    event: TypingErrorEvent,
    candidates: Vec<UnifiedCorrectionCandidate>,
}

impl L2CandidateLattice {
    pub(crate) fn new(event: TypingErrorEvent) -> Self {
        Self {
            event,
            candidates: Vec::new(),
        }
    }

    pub(crate) fn push_source(&mut self, candidate: Option<UnifiedCorrectionCandidate>) {
        if let Some(candidate) = candidate {
            if let Some(existing) = self
                .candidates
                .iter_mut()
                .find(|existing| existing.replacement == candidate.replacement)
            {
                if source_owner_priority(candidate.source) > source_owner_priority(existing.source)
                {
                    *existing = candidate;
                }
                return;
            }
            self.candidates.push(candidate);
        }
    }

    pub(crate) fn into_resolution(self) -> CorrectionResolution {
        let selected =
            TransitionDecisionCore::select_apply_candidate(&self.event, &self.candidates);
        let decision = selected.as_ref().map(|candidate| CorrectionDecision {
            replacement: candidate.replacement.clone(),
            source: candidate.source,
        });
        let scoreboard =
            CorrectionScoreboard::from_candidates(&self.event, &self.candidates, selected.as_ref());
        let candidate_scores = CorrectionCandidateScoreTrace::from_candidates(
            &self.event,
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

fn source_owner_priority(source: CorrectionDecisionSource) -> u8 {
    match source {
        CorrectionDecisionSource::Deterministic => 2,
        CorrectionDecisionSource::Nanda => 1,
    }
}
