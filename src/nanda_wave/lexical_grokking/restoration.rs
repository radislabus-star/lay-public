use std::cmp::Ordering;

use serde::Serialize;

use super::runtime::GrokkingCandidate;

pub(super) const MAX_TIED_CANDIDATES: usize = 32;
const MAX_RECONSTRUCTION_FRONTIER: usize = 8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RestorationCalibration {
    pub(super) max_geometry_distance: u8,
    pub(super) min_positive_milli: u16,
    pub(super) min_backward_milli: u16,
    pub(super) min_tied_energy_margin: u16,
}

impl RestorationCalibration {
    pub(super) const LEGACY_PERMISSIVE: Self = Self {
        max_geometry_distance: u8::MAX,
        min_positive_milli: 0,
        min_backward_milli: 0,
        min_tied_energy_margin: 0,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum AbstainReason {
    NoCandidates,
    OutsideCalibratedBasin,
    WeakPositivePhase,
    WeakBackwardReconstruction,
    ConflictingEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(super) struct RestorationEvidence {
    pub(super) geometry_distance: u8,
    pub(super) reconstruction_modes: u8,
    pub(super) positive_milli: u16,
    pub(super) backward_milli: u16,
    pub(super) anti_milli: u16,
    pub(super) hard_negative_milli: u16,
    pub(super) ambiguity_milli: u16,
    pub(super) ambiguity_threshold_milli: u16,
    pub(super) ambiguity_linked: bool,
    pub(super) ambiguity_shell: bool,
    pub(super) crystallization_wins: u8,
    pub(super) crystallization_required: u8,
    pub(super) crystallization_margin_milli: u16,
    pub(super) crystallization_known_edges: u16,
    pub(super) crystallization_unknown_edges: u16,
    pub(super) crystallization_tied_edges: u16,
    pub(super) crystallization_conflicts: u16,
    pub(super) crystallization_cycles: u16,
}

impl From<&GrokkingCandidate> for RestorationEvidence {
    fn from(candidate: &GrokkingCandidate) -> Self {
        Self {
            geometry_distance: candidate.geometry_distance,
            reconstruction_modes: candidate.reconstruction_modes,
            positive_milli: candidate
                .positive_subcenter_milli
                .max(candidate.positive_milli),
            backward_milli: candidate.backward_milli,
            anti_milli: candidate.anti_subcenter_milli.max(candidate.anti_milli),
            hard_negative_milli: candidate.hard_negative_milli,
            ambiguity_milli: candidate.ambiguity_milli,
            ambiguity_threshold_milli: candidate.ambiguity_threshold_milli,
            ambiguity_linked: candidate.ambiguity_linked,
            ambiguity_shell: candidate.ambiguity_shell,
            crystallization_wins: candidate.crystallization_wins,
            crystallization_required: candidate.crystallization_required,
            crystallization_margin_milli: candidate.crystallization_margin_milli,
            crystallization_known_edges: candidate.crystallization_known_edges,
            crystallization_unknown_edges: candidate.crystallization_unknown_edges,
            crystallization_tied_edges: candidate.crystallization_tied_edges,
            crystallization_conflicts: candidate.crystallization_conflicts,
            crystallization_cycles: candidate.crystallization_cycles,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(super) struct RestorationCandidate {
    pub(super) terminal_id: u32,
    pub(super) evidence: RestorationEvidence,
}

impl From<&GrokkingCandidate> for RestorationCandidate {
    fn from(candidate: &GrokkingCandidate) -> Self {
        Self {
            terminal_id: candidate.terminal_id,
            evidence: candidate.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub(super) enum RestorationReadout {
    Winner {
        candidate: RestorationCandidate,
    },
    Tied {
        geometry_distance: u8,
        candidates: Vec<RestorationCandidate>,
    },
    TiedOverflow {
        geometry_distance: u8,
        total_candidates: usize,
        candidates: Vec<RestorationCandidate>,
    },
    Abstain {
        reason: AbstainReason,
        geometry_distance: Option<u8>,
        candidates: Vec<RestorationCandidate>,
    },
}

pub(super) fn classify(
    candidates: &[GrokkingCandidate],
    calibration: RestorationCalibration,
) -> RestorationReadout {
    let Some(minimum_distance) = candidates
        .iter()
        .map(|candidate| candidate.geometry_distance)
        .min()
    else {
        return RestorationReadout::Abstain {
            reason: AbstainReason::NoCandidates,
            geometry_distance: None,
            candidates: Vec::new(),
        };
    };
    let mut nearest = geometric_basin(candidates);
    nearest.sort_unstable_by(evidence_order);
    if minimum_distance > calibration.max_geometry_distance {
        return RestorationReadout::Abstain {
            reason: AbstainReason::OutsideCalibratedBasin,
            geometry_distance: Some(minimum_distance),
            candidates: bounded_evidence(&nearest),
        };
    }

    if nearest.len() > 1 {
        let certified = nearest
            .iter()
            .copied()
            .filter(|candidate| candidate.crystallization_complete)
            .collect::<Vec<_>>();
        if certified.len() == 1 && authority_evidence_is_consistent(certified[0], calibration) {
            return RestorationReadout::Winner {
                candidate: certified[0].into(),
            };
        }
        if nearest.len() > MAX_TIED_CANDIDATES {
            return RestorationReadout::TiedOverflow {
                geometry_distance: minimum_distance,
                total_candidates: nearest.len(),
                candidates: bounded_evidence(&nearest),
            };
        }
        return RestorationReadout::Tied {
            geometry_distance: minimum_distance,
            candidates: bounded_evidence(&nearest),
        };
    }

    let winner = nearest[0];
    let evidence = RestorationEvidence::from(winner);
    if evidence.hard_negative_milli >= evidence.positive_milli
        || evidence.anti_milli > evidence.positive_milli
        || ambiguity_veto(winner)
    {
        return RestorationReadout::Abstain {
            reason: AbstainReason::ConflictingEvidence,
            geometry_distance: Some(minimum_distance),
            candidates: vec![winner.into()],
        };
    }
    if evidence.positive_milli < calibration.min_positive_milli {
        return RestorationReadout::Abstain {
            reason: AbstainReason::WeakPositivePhase,
            geometry_distance: Some(minimum_distance),
            candidates: vec![winner.into()],
        };
    }
    if winner.backward_milli < calibration.min_backward_milli {
        return RestorationReadout::Abstain {
            reason: AbstainReason::WeakBackwardReconstruction,
            geometry_distance: Some(minimum_distance),
            candidates: vec![winner.into()],
        };
    }
    RestorationReadout::Winner {
        candidate: winner.into(),
    }
}

pub(super) fn tied_energy_winner(candidates: &[GrokkingCandidate]) -> Option<(u32, u16)> {
    let nearest = geometric_basin(candidates);
    if !(2..=MAX_TIED_CANDIDATES).contains(&nearest.len()) {
        return None;
    }
    let mut ranked = nearest
        .into_iter()
        .map(|candidate| (candidate.terminal_id, candidate.settled_energy))
        .collect::<Vec<_>>();
    ranked.sort_unstable_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    let margin = ranked[0].1.saturating_sub(ranked[1].1);
    if margin <= 0 {
        return None;
    }
    Some((ranked[0].0, margin.min(i32::from(u16::MAX)) as u16))
}

fn authority_evidence_is_consistent(
    candidate: &GrokkingCandidate,
    calibration: RestorationCalibration,
) -> bool {
    let evidence = RestorationEvidence::from(candidate);
    evidence.hard_negative_milli < evidence.positive_milli
        && evidence.anti_milli <= evidence.positive_milli
        && evidence.positive_milli >= calibration.min_positive_milli
        && candidate.backward_milli >= calibration.min_backward_milli
}

fn ambiguity_veto(candidate: &GrokkingCandidate) -> bool {
    !candidate.exact_reconstruction && candidate.ambiguity_linked
}

pub(super) fn geometric_basin(candidates: &[GrokkingCandidate]) -> Vec<&GrokkingCandidate> {
    let Some(minimum_distance) = candidates
        .iter()
        .map(|candidate| candidate.geometry_distance)
        .min()
    else {
        return Vec::new();
    };
    candidates
        .iter()
        .enumerate()
        .filter(|(rank, candidate)| {
            candidate.geometry_distance == minimum_distance
                || candidate.ambiguity_shell
                || (minimum_distance > 0
                    && *rank < MAX_RECONSTRUCTION_FRONTIER
                    && candidate.reconstruction_modes != 0)
        })
        .map(|(_, candidate)| candidate)
        .collect()
}

fn bounded_evidence(candidates: &[&GrokkingCandidate]) -> Vec<RestorationCandidate> {
    candidates
        .iter()
        .take(MAX_TIED_CANDIDATES)
        .map(|candidate| RestorationCandidate::from(*candidate))
        .collect()
}

fn evidence_order(left: &&GrokkingCandidate, right: &&GrokkingCandidate) -> Ordering {
    let left_evidence = RestorationEvidence::from(*left);
    let right_evidence = RestorationEvidence::from(*right);
    right_evidence
        .positive_milli
        .cmp(&left_evidence.positive_milli)
        .then_with(|| {
            right_evidence
                .backward_milli
                .cmp(&left_evidence.backward_milli)
        })
        .then_with(|| left_evidence.anti_milli.cmp(&right_evidence.anti_milli))
        .then_with(|| {
            left_evidence
                .hard_negative_milli
                .cmp(&right_evidence.hard_negative_milli)
        })
        .then_with(|| left.terminal_id.cmp(&right.terminal_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(terminal_id: u32, geometry_distance: u8) -> GrokkingCandidate {
        GrokkingCandidate {
            terminal_id,
            geometry_distance,
            positive_milli: 900,
            backward_milli: 800,
            ..GrokkingCandidate::default()
        }
    }

    #[test]
    fn unique_nearest_center_owns_the_winner() {
        let candidates = [candidate(9, 2), candidate(3, 1), candidate(1, 3)];
        let readout = classify(&candidates, RestorationCalibration::LEGACY_PERMISSIVE);
        assert!(matches!(
            readout,
            RestorationReadout::Winner {
                candidate: RestorationCandidate { terminal_id: 3, .. }
            }
        ));
    }

    #[test]
    fn equal_geometry_is_preserved_as_a_position_independent_tie() {
        let candidates = [candidate(9, 1), candidate(3, 1), candidate(1, 2)];
        let readout = classify(&candidates, RestorationCalibration::LEGACY_PERMISSIVE);
        let RestorationReadout::Tied { candidates, .. } = readout else {
            panic!("expected tied restoration lattice");
        };
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.terminal_id)
                .collect::<Vec<_>>(),
            vec![3, 9]
        );
    }

    #[test]
    fn ambiguity_shell_extends_the_tied_basin_without_granting_authority() {
        let nearest = candidate(9, 1);
        let mut shell = candidate(3, 2);
        shell.ambiguity_shell = true;
        let readout = classify(
            &[nearest, shell, candidate(1, 3)],
            RestorationCalibration::LEGACY_PERMISSIVE,
        );
        let RestorationReadout::Tied { candidates, .. } = readout else {
            panic!("expected tied restoration lattice");
        };
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.terminal_id)
                .collect::<Vec<_>>(),
            vec![3, 9]
        );
    }

    #[test]
    fn calibration_can_abstain_without_inventing_a_winner() {
        let candidates = [candidate(3, 4)];
        let readout = classify(
            &candidates,
            RestorationCalibration {
                max_geometry_distance: 3,
                ..RestorationCalibration::LEGACY_PERMISSIVE
            },
        );
        assert_eq!(
            readout,
            RestorationReadout::Abstain {
                reason: AbstainReason::OutsideCalibratedBasin,
                geometry_distance: Some(4),
                candidates: vec![(&candidate(3, 4)).into()],
            }
        );
    }

    #[test]
    fn oversized_tie_preserves_a_bounded_evidence_lattice() {
        let candidates = (0..40)
            .map(|terminal_id| candidate(terminal_id, 1))
            .collect::<Vec<_>>();
        let readout = classify(&candidates, RestorationCalibration::LEGACY_PERMISSIVE);
        let RestorationReadout::TiedOverflow {
            total_candidates,
            candidates,
            ..
        } = readout
        else {
            panic!("expected tied overflow lattice");
        };
        assert_eq!(total_candidates, 40);
        assert_eq!(candidates.len(), MAX_TIED_CANDIDATES);
        assert_eq!(candidates[0].terminal_id, 0);
    }

    #[test]
    fn anti_evidence_cannot_collapse_an_objective_tied_basin() {
        let keep = candidate(1, 1);
        let mut reject = candidate(2, 1);
        reject.hard_negative_milli = 900;
        let readout = classify(&[keep, reject], RestorationCalibration::LEGACY_PERMISSIVE);
        let RestorationReadout::Tied { candidates, .. } = readout else {
            panic!("anti evidence collapsed a tied basin");
        };
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.terminal_id)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn complete_pairwise_certificate_resolves_a_geometry_tie() {
        let mut winner = candidate(1, 1);
        winner.crystallization_wins = 1;
        winner.crystallization_required = 1;
        winner.crystallization_margin_milli = 120;
        winner.crystallization_complete = true;
        let readout = classify(
            &[winner, candidate(2, 1)],
            RestorationCalibration::LEGACY_PERMISSIVE,
        );
        assert!(matches!(
            readout,
            RestorationReadout::Winner {
                candidate: RestorationCandidate { terminal_id: 1, .. }
            }
        ));
    }

    #[test]
    fn anti_veto_preserves_the_full_certified_basin() {
        let mut attempted_winner = candidate(1, 1);
        attempted_winner.crystallization_wins = 1;
        attempted_winner.crystallization_required = 1;
        attempted_winner.crystallization_margin_milli = 120;
        attempted_winner.crystallization_complete = true;
        attempted_winner.hard_negative_milli = 900;
        let readout = classify(
            &[attempted_winner, candidate(2, 1)],
            RestorationCalibration::LEGACY_PERMISSIVE,
        );
        assert!(matches!(readout, RestorationReadout::Tied { .. }));
    }

    #[test]
    fn learned_ambiguity_wave_vetoes_mutation_but_not_exact_noop() {
        let mut damaged = candidate(1, 1);
        damaged.ambiguity_milli = 920;
        damaged.ambiguity_threshold_milli = 900;
        damaged.ambiguity_linked = true;
        let readout = classify(&[damaged], RestorationCalibration::LEGACY_PERMISSIVE);
        assert!(matches!(
            readout,
            RestorationReadout::Abstain {
                reason: AbstainReason::ConflictingEvidence,
                ..
            }
        ));

        damaged.exact_reconstruction = true;
        let readout = classify(&[damaged], RestorationCalibration::LEGACY_PERMISSIVE);
        assert!(matches!(readout, RestorationReadout::Winner { .. }));
    }

    #[test]
    fn deletion_reconstruction_joins_the_scalar_nearest_basin() {
        let scalar_nearest = candidate(1, 1);
        let mut reconstructed = candidate(2, 2);
        reconstructed.reconstruction_modes = super::super::runtime::RECONSTRUCTION_MODE_DELETION;
        let unrelated = candidate(3, 3);
        let readout = classify(
            &[scalar_nearest, reconstructed, unrelated],
            RestorationCalibration {
                max_geometry_distance: 2,
                ..RestorationCalibration::LEGACY_PERMISSIVE
            },
        );
        let RestorationReadout::Tied { candidates, .. } = readout else {
            panic!("expected multimodal restoration lattice");
        };
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.terminal_id)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn exact_lexical_center_closes_deletion_hypotheses() {
        let exact_surface = candidate(1, 0);
        let mut reconstructed = candidate(2, 1);
        reconstructed.reconstruction_modes = super::super::runtime::RECONSTRUCTION_MODE_DELETION;
        let readout = classify(
            &[exact_surface, reconstructed],
            RestorationCalibration::LEGACY_PERMISSIVE,
        );
        assert!(matches!(
            readout,
            RestorationReadout::Winner {
                candidate: RestorationCandidate { terminal_id: 1, .. }
            }
        ));
    }
}
