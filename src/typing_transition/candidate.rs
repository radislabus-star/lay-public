use super::decision::{
    CandidateDecisionBatch, CandidateDecisionTiming, DecisionEvidenceMode, TransitionDecisionCore,
    TransitionDecisionPolicy,
};
use crate::correction_core::{
    CorrectionCandidateScoreTrace, CorrectionDecision, CorrectionResolution, CorrectionScoreboard,
    TypingErrorEvent, UnifiedCorrectionCandidate,
};
use crate::nanda_wave::WaveOptions;

pub(crate) struct L2CandidateLattice {
    event: TypingErrorEvent,
    // A proved exact target is retained outside the ordinary source frontier.
    // Source order and future top-k changes therefore cannot evict it.
    retained_exact: RetainedExactSlot,
    candidates: Vec<UnifiedCorrectionCandidate>,
    policy: TransitionDecisionPolicy,
    // None means that this request intentionally did not consult canonical L2.
    // Some(Unavailable) means that canonical L2 was requested but failed to
    // produce a field; that distinction is part of the apply-authority contract.
    l2_field_authority: Option<crate::nanda_wave::l2_field::L2FieldAuthority>,
}

#[expect(
    clippy::large_enum_variant,
    reason = "boxing would change the bounded inline candidate slot"
)]
enum RetainedExactSlot {
    Empty,
    Candidate(UnifiedCorrectionCandidate),
    Conflict,
}

impl L2CandidateLattice {
    #[cfg(test)]
    pub(crate) fn new(event: TypingErrorEvent) -> Self {
        Self {
            event,
            retained_exact: RetainedExactSlot::Empty,
            candidates: Vec::new(),
            policy: TransitionDecisionPolicy::default(),
            l2_field_authority: None,
        }
    }

    pub(crate) fn with_options(event: TypingErrorEvent, options: &WaveOptions) -> Self {
        Self {
            event,
            retained_exact: RetainedExactSlot::Empty,
            candidates: Vec::new(),
            policy: TransitionDecisionPolicy {
                l2_phase_apply: options.l2_phase_apply(),
            },
            l2_field_authority: None,
        }
    }

    pub(crate) fn push_source(&mut self, candidate: Option<UnifiedCorrectionCandidate>) {
        if let Some(candidate) = candidate {
            self.push(candidate);
        }
    }

    pub(crate) fn retain_exact(&mut self, mut candidate: UnifiedCorrectionCandidate) {
        if candidate.closed_exact_layout_certificate().is_none()
            || candidate.has_authority_conflict()
        {
            self.retained_exact = RetainedExactSlot::Conflict;
            return;
        }
        let mut index = 0;
        while index < self.candidates.len() {
            if self.candidates[index].replacement == candidate.replacement {
                let alias = self.candidates.remove(index);
                candidate.merge_evidence(alias);
            } else {
                index += 1;
            }
        }
        if candidate.has_authority_conflict() {
            self.retained_exact = RetainedExactSlot::Conflict;
            return;
        }
        match &mut self.retained_exact {
            RetainedExactSlot::Empty => {
                self.retained_exact = RetainedExactSlot::Candidate(candidate);
            }
            RetainedExactSlot::Candidate(existing)
                if existing.replacement == candidate.replacement =>
            {
                existing.merge_evidence(candidate);
                if existing.has_authority_conflict() {
                    self.retained_exact = RetainedExactSlot::Conflict;
                }
            }
            RetainedExactSlot::Candidate(_) | RetainedExactSlot::Conflict => {
                self.retained_exact = RetainedExactSlot::Conflict;
            }
        }
    }

    pub(crate) fn reject_exact_authority(&mut self) {
        self.retained_exact = RetainedExactSlot::Conflict;
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
        self.l2_field_authority = Some(authority);
    }

    fn push(&mut self, candidate: UnifiedCorrectionCandidate) {
        if let RetainedExactSlot::Candidate(retained) = &mut self.retained_exact {
            if retained.replacement == candidate.replacement {
                retained.merge_evidence(candidate);
                if retained.has_authority_conflict() {
                    self.retained_exact = RetainedExactSlot::Conflict;
                }
                return;
            }
        }
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
        resolve_l2_lattice(self, DecisionEvidenceMode::FullField(None)).0
    }

    pub(crate) fn into_observed_resolution_with_peak_context(
        self,
        peak_context: Option<&crate::nanda_wave::l2_wave_peak::L2CorrectionPeakContext>,
    ) -> (CorrectionResolution, CandidateDecisionTiming) {
        resolve_l2_lattice(self, DecisionEvidenceMode::FullField(peak_context))
    }

    pub(crate) fn into_closed_exact_resolution(
        self,
        certificate: Option<&crate::exact_layout_authority::ExactLayoutContourCertificate>,
    ) -> (CorrectionResolution, CandidateDecisionTiming) {
        let mode = match certificate {
            Some(certificate) => DecisionEvidenceMode::ClosedExact(certificate),
            None => DecisionEvidenceMode::ClosedExactAbsent,
        };
        resolve_l2_lattice(self, mode)
    }

    pub(crate) fn is_empty(&self) -> bool {
        matches!(self.retained_exact, RetainedExactSlot::Empty) && self.candidates.is_empty()
    }
}

#[cfg(test)]
mod retained_exact_tests {
    use super::*;
    use crate::candidate_contract::CandidateOrigin;
    use crate::correction_core::{
        CandidateGateAction, CandidateGateDecision, CorrectionDecisionSource, TypingErrorClass,
    };
    use crate::exact_layout_authority::{
        exact_authority_snapshot_if_warm, warm_up_exact_layout_authority_for_ibus,
        ActiveDecoderLayout, ExactLayoutContourCertificate, ExactLayoutFrame, FactoryEngineProfile,
    };

    fn certificate(frame_fingerprint: u64) -> ExactLayoutContourCertificate {
        warm_up_exact_layout_authority_for_ibus().expect("warm exact authority");
        let frame = ExactLayoutFrame {
            frame_revision: 17,
            frame_fingerprint,
            observed_token: "ghbdtn".to_string(),
            active_composition: true,
            factory_engine_profile: FactoryEngineProfile::UsQwerty,
            active_decoder_layout: ActiveDecoderLayout::Us,
            authority_snapshot: exact_authority_snapshot_if_warm(
                FactoryEngineProfile::UsQwerty,
                ActiveDecoderLayout::Us,
            ),
        };
        crate::exact_layout_authority::certify_closed_exact_layout("ghbdtn ", &frame, true, true)
            .expect("closed exact certificate")
    }

    fn exact_candidate(certificate: ExactLayoutContourCertificate) -> UnifiedCorrectionCandidate {
        UnifiedCorrectionCandidate::new(
            certificate.replacement_text(),
            CorrectionDecisionSource::Deterministic,
            CandidateOrigin::Layout,
            "exact-layout-test",
            TypingErrorClass::WrongLayout,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "exact-layout-test",
            },
        )
        .with_closed_exact_layout_authority(certificate)
    }

    fn alias(replacement: &str, source_id: &str) -> UnifiedCorrectionCandidate {
        UnifiedCorrectionCandidate::new(
            replacement,
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::L2Surface,
            source_id,
            TypingErrorClass::WrongLayout,
            CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: "same-surface-alias",
            },
        )
    }

    fn lattice() -> L2CandidateLattice {
        L2CandidateLattice::new(TypingErrorEvent {
            original: "ghbdtn ".to_string(),
            core: "ghbdtn".to_string(),
            current_word: "ghbdtn".to_string(),
            input_class: TypingErrorClass::WrongLayout,
        })
    }

    fn retained(lattice: &L2CandidateLattice) -> &UnifiedCorrectionCandidate {
        let RetainedExactSlot::Candidate(candidate) = &lattice.retained_exact else {
            panic!("expected retained exact candidate")
        };
        candidate
    }

    #[test]
    fn same_surface_alias_merge_is_producer_order_invariant() {
        let certificate = certificate(0x2710);
        let replacement = certificate.replacement_text().to_string();
        let exact = exact_candidate(certificate);
        let alias = alias(&replacement, "same-surface-alias");

        let mut exact_first = lattice();
        exact_first.retain_exact(exact.clone());
        exact_first.push_source(Some(alias.clone()));

        let mut alias_first = lattice();
        alias_first.push_source(Some(alias));
        alias_first.retain_exact(exact);

        assert!(exact_first.candidates.is_empty());
        assert!(alias_first.candidates.is_empty());
        assert_eq!(retained(&exact_first), retained(&alias_first));
        assert_eq!(retained(&exact_first).evidence_count(), 2);
    }

    #[test]
    fn conflicting_same_surface_certificates_fail_closed() {
        let mut lattice = lattice();
        lattice.retain_exact(exact_candidate(certificate(0x2710)));
        lattice.retain_exact(exact_candidate(certificate(0x2711)));

        assert!(matches!(
            lattice.retained_exact,
            RetainedExactSlot::Conflict
        ));
    }

    #[test]
    fn ordinary_competitor_count_cannot_evict_retained_exact_candidate() {
        let certificate = certificate(0x2710);
        let replacement = certificate.replacement_text().to_string();
        let mut lattice = lattice();
        lattice.retain_exact(exact_candidate(certificate));
        for index in 0..128 {
            lattice.push_source(Some(alias(
                &format!("ordinary-competitor-{index}"),
                "ordinary-competitor",
            )));
        }

        assert_eq!(retained(&lattice).replacement, replacement);
        assert_eq!(lattice.candidates.len(), 128);
    }
}

include!("candidate_resolution.rs");
