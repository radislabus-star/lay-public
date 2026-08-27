use std::collections::{BTreeMap, BTreeSet};

use crate::nanda_wave::lexical_grokking::restoration::{
    AbstainReason, RestorationCandidate, RestorationReadout,
};
use crate::typing_transition::target_evidence::{
    stable_bytes_ref, EnumerationCompletenessV1, IncompletenessReasonV1, MAX_TARGETS_PER_FIELD,
};

use super::calibrate::ProductiveCalibratedVerdictV1;
use super::packaged_runtime::{PackagedProductiveCandidateV1, PackagedProductiveReadoutV1};
use super::types::{ContradictionCertificateV1, ProductiveCandidateIdentityV1};

const MAX_GROUNDED_LANE: usize = 32;
const MAX_PRODUCTIVE_LANE: usize = 32;
const MAX_CONTOUR_LANE: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CompositeGroundedCandidateV1 {
    pub(super) candidate: RestorationCandidate,
    pub(super) decoded_surface: String,
    pub(super) normalized_surface: String,
    pub(super) protected_winner: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CompositeGroundedVerdictV1 {
    Winner {
        terminal_id: u32,
    },
    Tied {
        geometry_distance: u8,
    },
    TiedOverflow {
        geometry_distance: u8,
        total_candidates: usize,
    },
    Abstain {
        reason: AbstainReason,
        geometry_distance: Option<u8>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CompositeSurfaceGroupV1 {
    pub(super) normalized_surface: String,
    pub(super) grounded_terminal_ids: Vec<u32>,
    pub(super) productive_identities: Vec<ProductiveCandidateIdentityV1>,
    pub(super) grounded_protection: bool,
    pub(super) contour_grounding: bool,
    pub(super) exact_peak_birth: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CompositeL2LatticeV1 {
    pub(super) grounded_candidates: Vec<CompositeGroundedCandidateV1>,
    pub(super) productive_candidates: Vec<PackagedProductiveCandidateV1>,
    pub(super) surface_groups: Vec<CompositeSurfaceGroupV1>,
    pub(super) original_l11_verdict: CompositeGroundedVerdictV1,
    pub(super) productive_verdict: ProductiveCalibratedVerdictV1,
    pub(super) contradiction_certificate: Option<ContradictionCertificateV1>,
    pub(super) productive_overflow: bool,
    pub(super) productive_integrity_error: Option<String>,
}

impl CompositeL2LatticeV1 {
    pub(super) fn assemble(
        l11_readout: &RestorationReadout,
        mut decode_terminal: impl FnMut(u32) -> Option<String>,
        productive: PackagedProductiveReadoutV1,
        contradiction_certificate: Option<ContradictionCertificateV1>,
    ) -> Result<Self, String> {
        if productive.candidates.len() > MAX_PRODUCTIVE_LANE {
            return Err("productive composite lane exceeds 32 candidates".to_string());
        }
        if let Some(certificate) = contradiction_certificate {
            certificate.validate().map_err(str::to_string)?;
        }
        let (original_l11_verdict, candidates) = grounded_snapshot(l11_readout);
        if candidates.len() > MAX_GROUNDED_LANE {
            return Err("grounded composite lane exceeds 32 candidates".to_string());
        }
        let protected_winner = match original_l11_verdict {
            CompositeGroundedVerdictV1::Winner { terminal_id } => Some(terminal_id),
            _ => None,
        };
        let mut grounded_ids = BTreeSet::new();
        let mut grounded_candidates = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            if !grounded_ids.insert(candidate.terminal_id) {
                return Err("grounded composite lane repeats a terminal identity".to_string());
            }
            let decoded_surface = decode_terminal(candidate.terminal_id)
                .ok_or_else(|| "grounded composite terminal cannot be decoded".to_string())?;
            let normalized_surface =
                super::super::compositional::normalize_surface(&decoded_surface);
            if normalized_surface.is_empty() {
                return Err("grounded composite terminal decodes to an empty surface".to_string());
            }
            grounded_candidates.push(CompositeGroundedCandidateV1 {
                protected_winner: protected_winner == Some(candidate.terminal_id),
                candidate,
                decoded_surface,
                normalized_surface,
            });
        }

        let mut productive_ids = BTreeSet::new();
        for candidate in &productive.candidates {
            let identities = candidate
                .equivalent_identities
                .iter()
                .copied()
                .collect::<BTreeSet<_>>();
            if candidate.normalized_surface.is_empty()
                || identities.is_empty()
                || identities.len() != candidate.equivalent_identities.len()
                || !identities.contains(&candidate.identity)
                || identities
                    .iter()
                    .any(|identity| identity.lemma_id != candidate.identity.lemma_id)
                || !identities
                    .into_iter()
                    .all(|identity| productive_ids.insert(identity))
            {
                return Err(
                    "productive composite lane has an invalid surface identity node".to_string(),
                );
            }
        }
        let surface_groups = surface_groups(&grounded_candidates, &productive.candidates);
        let productive_overflow = match &productive.verdict {
            ProductiveCalibratedVerdictV1::Abstain {
                productive_overflow,
                ..
            } => *productive_overflow,
            _ => false,
        };
        Ok(Self {
            grounded_candidates,
            productive_candidates: productive.candidates,
            surface_groups,
            original_l11_verdict,
            productive_verdict: productive.verdict,
            contradiction_certificate,
            productive_overflow,
            productive_integrity_error: productive.integrity_error,
        })
    }

    pub(super) fn grounded_winner_is_preserved(&self) -> bool {
        let CompositeGroundedVerdictV1::Winner { terminal_id } = self.original_l11_verdict else {
            return true;
        };
        self.grounded_candidates.iter().any(|candidate| {
            candidate.candidate.terminal_id == terminal_id && candidate.protected_winner
        })
    }

    pub(super) fn common_completeness(&self) -> EnumerationCompletenessV1 {
        if self.productive_integrity_error.is_some() {
            return EnumerationCompletenessV1::failed(IncompletenessReasonV1::IntegrityFailure);
        }

        let mut logical_count_lower_bound = self.surface_groups.len();
        let mut overflow = self.productive_overflow;
        if let CompositeGroundedVerdictV1::TiedOverflow {
            total_candidates, ..
        } = &self.original_l11_verdict
        {
            logical_count_lower_bound = logical_count_lower_bound.max(*total_candidates);
            overflow = true;
        }
        if self.productive_overflow {
            logical_count_lower_bound =
                logical_count_lower_bound.max(self.surface_groups.len().saturating_add(1));
        }

        let digest = self
            .surface_groups
            .iter()
            .fold([0_u64; 2], |mut digest, group| {
                let surface = u64::from(stable_bytes_ref(group.normalized_surface.as_bytes()));
                digest[0] ^= surface.rotate_left((group.normalized_surface.len() % 63) as u32);
                digest[1] = digest[1].wrapping_add(surface);
                digest
            });
        if overflow {
            EnumerationCompletenessV1::overflow(
                self.surface_groups.len(),
                logical_count_lower_bound,
                IncompletenessReasonV1::UpstreamIncomplete,
                digest,
            )
        } else {
            EnumerationCompletenessV1::complete(self.surface_groups.len(), digest)
        }
    }

    pub(super) fn merge_contour_surfaces(
        &mut self,
        surfaces: impl IntoIterator<Item = String>,
    ) -> Result<(), String> {
        let normalized = surfaces
            .into_iter()
            .map(|surface| super::super::compositional::normalize_surface(&surface))
            .filter(|surface| !surface.is_empty())
            .collect::<BTreeSet<_>>();
        if normalized.len() > MAX_CONTOUR_LANE {
            return Err("typed contour lane exceeds 8 candidates".to_string());
        }

        for surface in normalized.iter() {
            if let Some(group) = self
                .surface_groups
                .iter_mut()
                .find(|group| group.normalized_surface == *surface)
            {
                group.contour_grounding = true;
                continue;
            }
            self.surface_groups.push(CompositeSurfaceGroupV1 {
                normalized_surface: surface.clone(),
                grounded_terminal_ids: Vec::new(),
                productive_identities: Vec::new(),
                grounded_protection: false,
                contour_grounding: true,
                exact_peak_birth: false,
            });
        }
        self.surface_groups
            .sort_by(|left, right| left.normalized_surface.cmp(&right.normalized_surface));
        Ok(())
    }

    pub(super) fn merge_exact_peak_surfaces(
        &mut self,
        surfaces: impl IntoIterator<Item = String>,
    ) -> Result<(), String> {
        let normalized = surfaces
            .into_iter()
            .map(|surface| super::super::compositional::normalize_surface(&surface))
            .filter(|surface| !surface.is_empty())
            .collect::<BTreeSet<_>>();
        for surface in normalized {
            if let Some(group) = self
                .surface_groups
                .iter_mut()
                .find(|group| group.normalized_surface == surface)
            {
                group.exact_peak_birth = true;
                continue;
            }
            self.surface_groups.push(CompositeSurfaceGroupV1 {
                normalized_surface: surface,
                grounded_terminal_ids: Vec::new(),
                productive_identities: Vec::new(),
                grounded_protection: false,
                contour_grounding: false,
                exact_peak_birth: true,
            });
        }
        self.surface_groups
            .sort_by(|left, right| left.normalized_surface.cmp(&right.normalized_surface));
        self.retain_ranked_productive_capacity()?;
        Ok(())
    }

    fn retain_ranked_productive_capacity(&mut self) -> Result<(), String> {
        let excess = self
            .surface_groups
            .len()
            .saturating_sub(MAX_TARGETS_PER_FIELD);
        if excess == 0 {
            return Ok(());
        }

        let mut productive_rank = BTreeMap::<String, usize>::new();
        for (rank, candidate) in self.productive_candidates.iter().enumerate() {
            let surface =
                super::super::compositional::normalize_surface(&candidate.normalized_surface);
            productive_rank.entry(surface).or_insert(rank);
        }
        let mut removable = self
            .surface_groups
            .iter()
            .filter(|group| {
                group.grounded_terminal_ids.is_empty()
                    && !group.grounded_protection
                    && !group.contour_grounding
                    && !group.exact_peak_birth
                    && !group.productive_identities.is_empty()
            })
            .map(|group| {
                (
                    group.normalized_surface.clone(),
                    productive_rank
                        .get(&group.normalized_surface)
                        .copied()
                        .unwrap_or(usize::MAX),
                )
            })
            .collect::<Vec<_>>();
        if removable.len() < excess {
            return Err(
                "exact, grounded, and contour surfaces exceed common 74-target capacity"
                    .to_string(),
            );
        }
        removable.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| right.0.cmp(&left.0)));
        let removed = removable
            .into_iter()
            .take(excess)
            .map(|(surface, _)| surface)
            .collect::<BTreeSet<_>>();
        self.surface_groups
            .retain(|group| !removed.contains(&group.normalized_surface));
        self.productive_overflow = true;
        Ok(())
    }
}

fn grounded_snapshot(
    readout: &RestorationReadout,
) -> (CompositeGroundedVerdictV1, Vec<RestorationCandidate>) {
    match readout {
        RestorationReadout::Winner { candidate } => (
            CompositeGroundedVerdictV1::Winner {
                terminal_id: candidate.terminal_id,
            },
            vec![*candidate],
        ),
        RestorationReadout::Tied {
            geometry_distance,
            candidates,
        } => (
            CompositeGroundedVerdictV1::Tied {
                geometry_distance: *geometry_distance,
            },
            candidates.clone(),
        ),
        RestorationReadout::TiedOverflow {
            geometry_distance,
            total_candidates,
            candidates,
        } => (
            CompositeGroundedVerdictV1::TiedOverflow {
                geometry_distance: *geometry_distance,
                total_candidates: *total_candidates,
            },
            candidates.clone(),
        ),
        RestorationReadout::Abstain {
            reason,
            geometry_distance,
            candidates,
        } => (
            CompositeGroundedVerdictV1::Abstain {
                reason: *reason,
                geometry_distance: *geometry_distance,
            },
            candidates.clone(),
        ),
    }
}

fn surface_groups(
    grounded: &[CompositeGroundedCandidateV1],
    productive: &[PackagedProductiveCandidateV1],
) -> Vec<CompositeSurfaceGroupV1> {
    let mut groups = BTreeMap::<String, CompositeSurfaceGroupV1>::new();
    for candidate in grounded {
        let group = groups
            .entry(candidate.normalized_surface.clone())
            .or_insert_with(|| CompositeSurfaceGroupV1 {
                normalized_surface: candidate.normalized_surface.to_string(),
                grounded_terminal_ids: Vec::new(),
                productive_identities: Vec::new(),
                grounded_protection: false,
                contour_grounding: false,
                exact_peak_birth: false,
            });
        group
            .grounded_terminal_ids
            .push(candidate.candidate.terminal_id);
        group.grounded_protection |= candidate.protected_winner;
    }
    for candidate in productive {
        let normalized =
            super::super::compositional::normalize_surface(&candidate.normalized_surface);
        let group = groups
            .entry(normalized.clone())
            .or_insert_with(|| CompositeSurfaceGroupV1 {
                normalized_surface: normalized,
                grounded_terminal_ids: Vec::new(),
                productive_identities: Vec::new(),
                grounded_protection: false,
                contour_grounding: false,
                exact_peak_birth: false,
            });
        group
            .productive_identities
            .extend(candidate.equivalent_identities.iter().copied());
    }
    for group in groups.values_mut() {
        group.grounded_terminal_ids.sort_unstable();
        group.productive_identities.sort_unstable();
    }
    groups.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::super::calibrate::{ProductiveCalibratedVerdictV1, ReadoutCandidateV1};
    use super::super::geometry::GeometryTerminalEvidenceV1;
    use super::super::packaged_runtime::PackagedProductiveReadoutV1;
    use super::super::types::ProductiveCandidateIdentityV1;
    use super::*;
    use crate::nanda_wave::lexical_grokking::restoration::RestorationEvidence;

    fn grounded(terminal_id: u32) -> RestorationCandidate {
        RestorationCandidate {
            terminal_id,
            evidence: RestorationEvidence::default(),
        }
    }

    fn productive(index: u32, surface: String) -> PackagedProductiveCandidateV1 {
        PackagedProductiveCandidateV1 {
            identity: ProductiveCandidateIdentityV1 {
                lemma_id: index,
                paradigm_id: 1,
                program_id: index,
                target_slot_id: 1,
                normalized_surface_id: index,
                variant_id: 1,
            },
            equivalent_identities: vec![ProductiveCandidateIdentityV1 {
                lemma_id: index,
                paradigm_id: 1,
                program_id: index,
                target_slot_id: 1,
                normalized_surface_id: index,
                variant_id: 1,
            }],
            normalized_surface: surface.into(),
            score_q16: i64::from(index),
            geometry: GeometryTerminalEvidenceV1::default(),
            provenance: super::super::calibrate::CandidateProvenanceClassV1::TrainingSeenGenerated,
            minimum_independent_support: 1,
            grounded_support: 1,
            ambiguity_center_cosine: 0,
            equivalent_identity_count: 1,
            equivalent_paradigm_count: 1,
            minimum_equivalent_support: 1,
            maximum_equivalent_support: 1,
            rank_origin: super::super::calibrate::CandidateRankOriginV1::BaseV64,
            cross_lane_certified: false,
        }
    }

    fn productive_readout(
        candidates: Vec<PackagedProductiveCandidateV1>,
    ) -> PackagedProductiveReadoutV1 {
        let suggestions = candidates
            .iter()
            .map(|candidate| ReadoutCandidateV1 {
                identity: candidate.identity,
                equivalent_identities: candidate.equivalent_identities.clone(),
                normalized_surface: candidate.normalized_surface.to_string(),
                score_q16: candidate.score_q16,
                grounded_lemma_evidence: candidate.grounded_support,
                exact_osa_distance: 0,
                exact_form: false,
                cross_lemma_ownership_satisfied: false,
                rank_origin: candidate.rank_origin,
                cross_lane_certified: candidate.cross_lane_certified,
            })
            .collect();
        PackagedProductiveReadoutV1 {
            verdict: ProductiveCalibratedVerdictV1::Abstain {
                suggestions,
                productive_overflow: false,
            },
            candidates,
            logical_terminal_count: 32,
            logical_surface_basin_count: 32,
            integrity_error: None,
        }
    }

    #[test]
    fn composite_preserves_all_32_plus_32_identities_before_display_dedup() {
        let grounded = (1..=32).map(grounded).collect::<Vec<_>>();
        let l11 = RestorationReadout::Tied {
            geometry_distance: 1,
            candidates: grounded,
        };
        let productive = (1..=32)
            .map(|index| {
                let surface = if index == 1 {
                    "grounded-1".to_string()
                } else {
                    format!("productive-{index}")
                };
                productive(index, surface)
            })
            .collect::<Vec<_>>();
        let lattice = CompositeL2LatticeV1::assemble(
            &l11,
            |terminal_id| Some(format!("grounded-{terminal_id}")),
            productive_readout(productive),
            None,
        )
        .expect("composite lattice");

        assert_eq!(lattice.grounded_candidates.len(), 32);
        assert_eq!(lattice.productive_candidates.len(), 32);
        assert_eq!(
            lattice
                .surface_groups
                .iter()
                .map(|group| {
                    group.grounded_terminal_ids.len() + group.productive_identities.len()
                })
                .sum::<usize>(),
            64
        );
        let shared = lattice
            .surface_groups
            .iter()
            .find(|group| group.normalized_surface == "grounded-1")
            .expect("shared display group");
        assert_eq!(shared.grounded_terminal_ids.len(), 1);
        assert_eq!(shared.productive_identities.len(), 1);
        assert_eq!(lattice.grounded_candidates.len(), 32);
        assert_eq!(lattice.productive_candidates.len(), 32);
        assert_eq!(lattice.surface_groups.len(), 63);
    }

    #[test]
    fn productive_verdict_cannot_remove_a_grounded_winner() {
        let winner = grounded(7);
        let l11 = RestorationReadout::Winner { candidate: winner };
        let productive_candidate = productive(1, "other".to_string());
        let readout_candidate = ReadoutCandidateV1 {
            identity: productive_candidate.identity,
            equivalent_identities: productive_candidate.equivalent_identities.clone(),
            normalized_surface: productive_candidate.normalized_surface.to_string(),
            score_q16: productive_candidate.score_q16,
            grounded_lemma_evidence: 1,
            exact_osa_distance: 0,
            exact_form: false,
            cross_lemma_ownership_satisfied: true,
            rank_origin: productive_candidate.rank_origin,
            cross_lane_certified: productive_candidate.cross_lane_certified,
        };
        let productive = PackagedProductiveReadoutV1 {
            verdict: ProductiveCalibratedVerdictV1::Winner {
                candidate: readout_candidate,
                calibration_stratum_id: 1,
            },
            candidates: vec![productive_candidate],
            logical_terminal_count: 1,
            logical_surface_basin_count: 1,
            integrity_error: None,
        };
        let lattice = CompositeL2LatticeV1::assemble(
            &l11,
            |terminal_id| Some(format!("winner-{terminal_id}")),
            productive,
            None,
        )
        .expect("protected composite");
        assert!(lattice.grounded_winner_is_preserved());
        assert_eq!(lattice.grounded_candidates[0].candidate.terminal_id, 7);
        assert!(lattice.grounded_candidates[0].protected_winner);
    }

    #[test]
    fn common_completeness_preserves_complete_overflow_and_failed_states() {
        let l11 = RestorationReadout::Abstain {
            reason: AbstainReason::NoCandidates,
            geometry_distance: None,
            candidates: Vec::new(),
        };
        let mut lattice = CompositeL2LatticeV1::assemble(
            &l11,
            |_| None,
            productive_readout(vec![
                productive(1, "first".to_string()),
                productive(2, "second".to_string()),
            ]),
            None,
        )
        .expect("composite lattice");

        let complete = lattice.common_completeness();
        assert_eq!(
            complete.state(),
            crate::typing_transition::target_evidence::EnumerationStateV1::Complete
        );
        assert_eq!(complete.retained_count(), 2);
        assert_eq!(complete.logical_count_lower_bound(), 2);

        lattice.productive_overflow = true;
        let overflow = lattice.common_completeness();
        assert_eq!(
            overflow.state(),
            crate::typing_transition::target_evidence::EnumerationStateV1::Overflow
        );
        assert_eq!(
            overflow.reason(),
            IncompletenessReasonV1::UpstreamIncomplete
        );
        assert_eq!(overflow.retained_count(), 2);
        assert!(overflow.logical_count_lower_bound() >= 3);

        lattice.productive_integrity_error = Some("invalid package".to_string());
        let failed = lattice.common_completeness();
        assert_eq!(
            failed.state(),
            crate::typing_transition::target_evidence::EnumerationStateV1::Failed
        );
        assert_eq!(failed.reason(), IncompletenessReasonV1::IntegrityFailure);
    }

    #[test]
    fn exact_capacity_preserves_mandatory_surfaces_and_drops_productive_tail() {
        let grounded = (1..=13).map(grounded).collect::<Vec<_>>();
        let l11 = RestorationReadout::Tied {
            geometry_distance: 1,
            candidates: grounded,
        };
        let productive = (1..=32)
            .map(|index| productive(index, format!("productive-{index:02}")))
            .collect::<Vec<_>>();
        let mut lattice = CompositeL2LatticeV1::assemble(
            &l11,
            |terminal_id| Some(format!("grounded-{terminal_id:02}")),
            productive_readout(productive),
            None,
        )
        .expect("composite lattice");
        lattice
            .merge_contour_surfaces((1..=3).map(|index| format!("contour-{index:02}")))
            .expect("contours");
        lattice
            .merge_exact_peak_surfaces((1..=56).map(|index| format!("exact-{index:02}")))
            .expect("bounded exact merge");

        assert_eq!(lattice.surface_groups.len(), MAX_TARGETS_PER_FIELD);
        assert!(lattice.productive_overflow);
        for index in 1..=13 {
            assert!(lattice
                .surface_groups
                .iter()
                .any(|group| group.normalized_surface == format!("grounded-{index:02}")));
        }
        for index in 1..=56 {
            assert!(lattice.surface_groups.iter().any(|group| {
                group.normalized_surface == format!("exact-{index:02}") && group.exact_peak_birth
            }));
        }
        for index in 1..=3 {
            assert!(lattice.surface_groups.iter().any(|group| {
                group.normalized_surface == format!("contour-{index:02}") && group.contour_grounding
            }));
        }
        assert!(lattice
            .surface_groups
            .iter()
            .any(|group| group.normalized_surface == "productive-01"));
        assert!(lattice
            .surface_groups
            .iter()
            .any(|group| group.normalized_surface == "productive-02"));
        assert!(!lattice
            .surface_groups
            .iter()
            .any(|group| group.normalized_surface == "productive-03"));
    }
}
