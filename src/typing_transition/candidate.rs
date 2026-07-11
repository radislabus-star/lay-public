use super::decision::TransitionDecisionCore;
use super::decision::TransitionDecisionPolicy;
use crate::correction_core::{
    CorrectionCandidateScoreTrace, CorrectionDecision, CorrectionResolution, CorrectionScoreboard,
    TypingErrorEvent, UnifiedCorrectionCandidate,
};
use crate::nanda_wave::WaveOptions;

pub(crate) struct L2CandidateLattice {
    event: TypingErrorEvent,
    candidates: Vec<UnifiedCorrectionCandidate>,
    policy: TransitionDecisionPolicy,
}

impl L2CandidateLattice {
    #[cfg(test)]
    pub(crate) fn new(event: TypingErrorEvent) -> Self {
        Self {
            event,
            candidates: Vec::new(),
            policy: TransitionDecisionPolicy::default(),
        }
    }

    pub(crate) fn with_options(event: TypingErrorEvent, options: &WaveOptions) -> Self {
        Self {
            event,
            candidates: Vec::new(),
            policy: TransitionDecisionPolicy {
                l2_phase_apply: options.l2_phase_apply(),
            },
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
