use std::collections::{BTreeMap, BTreeSet};

use crate::nanda_wave::lexical_grokking::restoration::{
    AbstainReason, RestorationCandidate, RestorationReadout,
};

use super::calibrate::ProductiveCalibratedVerdictV1;
use super::packaged_runtime::{PackagedProductiveCandidateV1, PackagedProductiveReadoutV1};
use super::types::{ContradictionCertificateV1, ProductiveCandidateIdentityV1};

const MAX_GROUNDED_LANE: usize = 32;
const MAX_PRODUCTIVE_LANE: usize = 32;

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
}
