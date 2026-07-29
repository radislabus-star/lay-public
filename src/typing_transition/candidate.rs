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
    l2_field_authority: crate::nanda_wave::l2_field::L2FieldAuthority,
}

impl L2CandidateLattice {
    #[cfg(test)]
    pub(crate) fn new(event: TypingErrorEvent) -> Self {
        Self {
            event,
            candidates: Vec::new(),
            policy: TransitionDecisionPolicy::default(),
            l2_field_authority: crate::nanda_wave::l2_field::L2FieldAuthority::Unavailable,
        }
    }

    pub(crate) fn with_options(event: TypingErrorEvent, options: &WaveOptions) -> Self {
        Self {
            event,
            candidates: Vec::new(),
            policy: TransitionDecisionPolicy {
                l2_phase_apply: options.l2_phase_apply(),
            },
            l2_field_authority: crate::nanda_wave::l2_field::L2FieldAuthority::Unavailable,
        }
    }

    pub(crate) fn push_source(&mut self, candidate: Option<UnifiedCorrectionCandidate>) {
        if let Some(candidate) = candidate {
            self.push(candidate);
        }
    }

    pub(crate) fn extend_source(
        &mut self,
        candidates: impl IntoIterator<Item = UnifiedCorrectionCandidate>,
    ) {
        for candidate in candidates {
            self.push(candidate);
        }
    }

    pub(crate) fn set_l2_field_authority(
        &mut self,
        authority: crate::nanda_wave::l2_field::L2FieldAuthority,
    ) {
        self.l2_field_authority = authority;
    }

    fn push(&mut self, candidate: UnifiedCorrectionCandidate) {
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

    #[cfg(test)]
    pub(crate) fn into_resolution(self) -> CorrectionResolution {
        resolve_l2_lattice(self, None)
    }

    pub(crate) fn into_resolution_with_peak_context(
        self,
        peak_context: Option<&crate::nanda_wave::l2_wave_peak::L2CorrectionPeakContext>,
    ) -> CorrectionResolution {
        resolve_l2_lattice(self, peak_context)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }
}

include!("candidate_resolution.rs");
