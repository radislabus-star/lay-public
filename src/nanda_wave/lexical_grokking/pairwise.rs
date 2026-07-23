use super::crystal::{ComplexBasisWave, WordCenter64, WAVE_DIMENSION};
use super::model::{PairKey, PairPhaseProfile};
use super::runtime::GrokkingCandidate;
use super::wave_basis::{complex_coherence_milli, expand_word};

const MAX_PAIRWISE_CANDIDATES: usize = 8;
const MIN_DIRECTION_SUPPORT: u16 = 2;
const MIN_SCENE_COHERENCE: u16 = 620;
const MIN_DIRECTION_MARGIN: u16 = 48;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EdgeOutcome {
    Unknown,
    Tie,
    LowWins { margin: u16, coherence: u16 },
    HighWins { margin: u16, coherence: u16 },
    Conflict,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct PairwiseCertificate {
    pub(super) known_edges: u16,
    pub(super) unknown_edges: u16,
    pub(super) ties: u16,
    pub(super) conflicts: u16,
    pub(super) cycles: u16,
    pub(super) suppressed_candidates: u16,
}

pub(super) fn apply_pairwise_field(
    profiles: &[PairPhaseProfile],
    centers: &[WordCenter64],
    basis: &[ComplexBasisWave],
    candidates: &mut [GrokkingCandidate],
    surface_re: &[i32; WAVE_DIMENSION],
    surface_im: &[i32; WAVE_DIMENSION],
) -> PairwiseCertificate {
    let mut certificate = PairwiseCertificate::default();
    if profiles.is_empty() || candidates.len() < 2 {
        return certificate;
    }

    let mut active = (0..candidates.len()).collect::<Vec<_>>();
    active.sort_unstable_by(|left, right| {
        super::runtime::candidate_order(&candidates[*left], &candidates[*right])
    });
    active.truncate(MAX_PAIRWISE_CANDIDATES);
    let mut rank = vec![usize::MAX; candidates.len()];
    for (position, candidate_index) in active.iter().copied().enumerate() {
        rank[candidate_index] = position;
    }

    let mut edges = Vec::new();
    for left in 0..active.len() {
        for right in left + 1..active.len() {
            let left_index = active[left];
            let right_index = active[right];
            let Some(key) = PairKey::new(
                candidates[left_index].terminal_id,
                candidates[right_index].terminal_id,
            ) else {
                continue;
            };
            let outcome = evaluate_edge(profiles, centers, basis, key, surface_re, surface_im);
            match outcome {
                EdgeOutcome::Unknown => {
                    certificate.unknown_edges = certificate.unknown_edges.saturating_add(1)
                }
                EdgeOutcome::Tie => certificate.ties = certificate.ties.saturating_add(1),
                EdgeOutcome::Conflict => {
                    certificate.conflicts = certificate.conflicts.saturating_add(1)
                }
                EdgeOutcome::LowWins { margin, coherence }
                | EdgeOutcome::HighWins { margin, coherence } => {
                    certificate.known_edges = certificate.known_edges.saturating_add(1);
                    let low_index = if candidates[left_index].terminal_id == key.low_terminal {
                        left_index
                    } else {
                        right_index
                    };
                    let high_index = if low_index == left_index {
                        right_index
                    } else {
                        left_index
                    };
                    let (winner, loser) = match outcome {
                        EdgeOutcome::LowWins { .. } => (low_index, high_index),
                        EdgeOutcome::HighWins { .. } => (high_index, low_index),
                        _ => unreachable!(),
                    };
                    // Pair memory is corrective evidence, not independent
                    // support. It acts only when the learned winner currently
                    // sits below the learned false competitor.
                    if rank[loser] < rank[winner]
                        && !candidates[loser].exact_reconstruction
                        && coherence > candidates[loser].positive_milli.saturating_add(24)
                    {
                        edges.push((winner, loser, margin));
                    }
                }
            }
        }
    }

    if contains_cycle(candidates.len(), &edges) {
        certificate.cycles = 1;
        return certificate;
    }

    for (_, loser, margin) in edges {
        // The accepted V32 exact ordered-subsequence certificate remains the
        // stronger structural proof. Pair memory cannot suppress it.
        if candidates[loser].legacy_sequence_milli == 1_000 {
            continue;
        }
        let pressure = margin.saturating_mul(6).min(1_000);
        if pressure > candidates[loser].pairwise_loss_milli {
            candidates[loser].pairwise_loss_milli = pressure;
            certificate.suppressed_candidates = certificate.suppressed_candidates.saturating_add(1);
        }
    }
    for candidate in candidates {
        let pressure = i32::from(candidate.pairwise_loss_milli).saturating_mul(4);
        candidate.settled_energy = candidate.settled_energy.saturating_sub(pressure);
        candidate.legacy_settled_energy = candidate.legacy_settled_energy.saturating_sub(pressure);
    }
    certificate
}

pub(super) fn evaluate_edge(
    profiles: &[PairPhaseProfile],
    centers: &[WordCenter64],
    basis: &[ComplexBasisWave],
    key: PairKey,
    surface_re: &[i32; WAVE_DIMENSION],
    surface_im: &[i32; WAVE_DIMENSION],
) -> EdgeOutcome {
    let Ok(index) = profiles.binary_search_by_key(&key, |profile| profile.key) else {
        return EdgeOutcome::Unknown;
    };
    let profile = profiles[index];
    let low = direction_coherence(
        centers,
        basis,
        profile.low_wins_start,
        profile.low_wins_count,
        surface_re,
        surface_im,
    );
    let high = direction_coherence(
        centers,
        basis,
        profile.high_wins_start,
        profile.high_wins_count,
        surface_re,
        surface_im,
    );
    match (low, high) {
        (None, None) => EdgeOutcome::Unknown,
        (Some(low), Some(high)) if low >= MIN_SCENE_COHERENCE && high >= MIN_SCENE_COHERENCE => {
            let margin = low.abs_diff(high);
            if margin < MIN_DIRECTION_MARGIN {
                EdgeOutcome::Conflict
            } else if low > high {
                EdgeOutcome::LowWins {
                    margin,
                    coherence: low,
                }
            } else {
                EdgeOutcome::HighWins {
                    margin,
                    coherence: high,
                }
            }
        }
        (Some(_), None) | (None, Some(_)) => EdgeOutcome::Unknown,
        (Some(_), Some(_)) => EdgeOutcome::Tie,
    }
}

fn direction_coherence(
    centers: &[WordCenter64],
    basis: &[ComplexBasisWave],
    start: u32,
    count: u16,
    surface_re: &[i32; WAVE_DIMENSION],
    surface_im: &[i32; WAVE_DIMENSION],
) -> Option<u16> {
    centers
        .get(start as usize..start as usize + count as usize)?
        .iter()
        .filter(|center| center.crystal_support >= MIN_DIRECTION_SUPPORT)
        .map(|center| {
            let (center_re, center_im) = expand_word(basis, *center);
            complex_coherence_milli(surface_re, surface_im, &center_re, &center_im)
        })
        .max()
}

fn contains_cycle(candidate_count: usize, edges: &[(usize, usize, u16)]) -> bool {
    let mut reach = vec![vec![false; candidate_count]; candidate_count];
    for (winner, loser, _) in edges {
        reach[*winner][*loser] = true;
    }
    for pivot in 0..candidate_count {
        for from in 0..candidate_count {
            for to in 0..candidate_count {
                reach[from][to] |= reach[from][pivot] && reach[pivot][to];
            }
        }
    }
    (0..candidate_count).any(|index| reach[index][index])
}
