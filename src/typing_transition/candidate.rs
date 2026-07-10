use super::decision::TransitionDecisionCore;
use crate::correction_core::{
    CorrectionCandidateScoreTrace, CorrectionDecision, CorrectionResolution, CorrectionScoreboard,
    TypingErrorEvent, UnifiedCorrectionCandidate,
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
                existing.merge_evidence(candidate);
                return;
            }
            self.candidates.push(candidate);
        }
    }

    pub(crate) fn into_resolution(self) -> CorrectionResolution {
        resolve_l2_lattice(self)
    }
}

include!("candidate_resolution.rs");
