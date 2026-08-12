use std::cmp::Ordering;
use std::collections::BTreeMap;

use super::scene::SceneWaveV1;
use super::{
    L2_SCENE_PHASE_CELLS, MAX_AMBIGUITY_SUBCENTERS, MAX_ANTI_SUBCENTERS,
    MAX_HARD_NEGATIVE_SUBCENTERS, MAX_POSITIVE_SUBCENTERS, PRODUCTIVE_V1_INNER_FOLDS,
};

const PHASE_COSINE_SCALE: i64 = 1_000_000;
const PHASE_CENTER_SUPPORT_SATURATED: u8 = 1 << 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i8)]
pub(super) enum PhaseBankKindV1 {
    HardNegative = -2,
    Anti = -1,
    Ambiguity = 0,
    Positive = 1,
}

impl PhaseBankKindV1 {
    const fn limit(self) -> usize {
        match self {
            Self::Positive => MAX_POSITIVE_SUBCENTERS,
            Self::Anti => MAX_ANTI_SUBCENTERS,
            Self::HardNegative => MAX_HARD_NEGATIVE_SUBCENTERS,
            Self::Ambiguity => MAX_AMBIGUITY_SUBCENTERS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PhaseObservationV1 {
    pub(super) event_identity: [u8; 32],
    pub(super) inner_fold: u8,
    pub(super) wave: SceneWaveV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FittedPhaseCenterV1 {
    pub(super) cells: [i8; L2_SCENE_PHASE_CELLS],
    pub(super) feature_mask: u32,
    pub(super) context_mode_id: u32,
    pub(super) support: u16,
    pub(super) mass: u16,
    pub(super) polarity: i8,
    pub(super) flags: u8,
    pub(super) exact_support: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct FittedPhaseBankV1 {
    pub(super) selected_k: u8,
    pub(super) centers: Vec<FittedPhaseCenterV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PhaseRankingSelectionGroupV1 {
    pub(super) group_identity: [u8; 32],
    pub(super) inner_fold: u8,
    pub(super) members: Vec<SceneWaveV1>,
    pub(super) comparators: Vec<SceneWaveV1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AmbiguityPhaseMemberV1 {
    pub(super) slot_profile_id: u32,
    pub(super) wave: SceneWaveV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AmbiguityPhaseSelectionGroupV1 {
    pub(super) group_identity: [u8; 32],
    pub(super) inner_fold: u8,
    pub(super) valid_members: Vec<AmbiguityPhaseMemberV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RawPhaseClusterV1 {
    cells: [i8; L2_SCENE_PHASE_CELLS],
    exact_support: u32,
}

pub(super) fn integer_cosine(left: &SceneWaveV1, right: &SceneWaveV1) -> i64 {
    let mut dot = 0_i64;
    let mut left_norm = 0_u64;
    let mut right_norm = 0_u64;
    for (left, right) in left.0.into_iter().zip(right.0) {
        let left = i64::from(left);
        let right = i64::from(right);
        dot += left * right;
        left_norm += (left * left) as u64;
        right_norm += (right * right) as u64;
    }
    if left_norm == 0 || right_norm == 0 {
        return 0;
    }
    let denominator = integer_sqrt(u128::from(left_norm) * u128::from(right_norm)).max(1);
    let numerator = i128::from(dot) * i128::from(PHASE_COSINE_SCALE);
    (numerator / denominator as i128) as i64
}

pub(super) fn maximum_phase_coherence(wave: &SceneWaveV1, centers: &[FittedPhaseCenterV1]) -> i64 {
    centers
        .iter()
        .map(|center| integer_cosine(wave, &SceneWaveV1(center.cells)))
        .max()
        .unwrap_or_default()
}

pub(super) fn fit_ranking_phase_bank(
    kind: PhaseBankKindV1,
    observations: &[PhaseObservationV1],
    groups: &[PhaseRankingSelectionGroupV1],
    feature_mask: u32,
    context_mode_id: u32,
) -> Result<FittedPhaseBankV1, &'static str> {
    if kind == PhaseBankKindV1::Ambiguity {
        return Err("ambiguity phase banks require valid-set selection");
    }
    let observations = validated_observations(observations)?;
    let groups = validated_ranking_groups(groups)?;
    let baseline = ranking_fold_losses(&groups, |_| Ok(Vec::new()))?;
    let mut selected: Option<(usize, [f64; PRODUCTIVE_V1_INNER_FOLDS as usize])> = None;
    for k in 1..=kind.limit() {
        let losses = ranking_fold_losses(&groups, |fold| {
            let train = observations
                .iter()
                .copied()
                .filter(|observation| observation.inner_fold != fold)
                .collect::<Vec<_>>();
            cluster_phase_waves(&train, k)
        });
        let Ok(losses) = losses else {
            continue;
        };
        if losses
            .iter()
            .zip(baseline)
            .any(|(loss, baseline)| loss.total_cmp(&baseline) != Ordering::Less)
        {
            continue;
        }
        let replace = selected.as_ref().is_none_or(|(best_k, best_losses)| {
            mean_loss(&losses)
                .total_cmp(&mean_loss(best_losses))
                .then_with(|| k.cmp(best_k))
                .is_lt()
        });
        if replace {
            selected = Some((k, losses));
        }
    }
    let Some((selected_k, _)) = selected else {
        return Ok(FittedPhaseBankV1::default());
    };
    let raw = cluster_phase_waves(&observations, selected_k)?;
    Ok(package_centers(
        kind,
        selected_k,
        raw,
        feature_mask,
        context_mode_id,
    ))
}

pub(super) fn fit_ambiguity_phase_banks(
    observations_by_profile: &BTreeMap<u32, Vec<PhaseObservationV1>>,
    groups: &[AmbiguityPhaseSelectionGroupV1],
    profile_metadata: &BTreeMap<u32, (u32, u32)>,
) -> Result<BTreeMap<u32, FittedPhaseBankV1>, &'static str> {
    let groups = validated_ambiguity_groups(groups)?;
    let mut selected_k = BTreeMap::<u32, usize>::new();
    for (profile_id, source) in observations_by_profile {
        let observations = validated_observations(source)?;
        let mut selected: Option<(usize, u64, i128)> = None;
        for k in 1..=PhaseBankKindV1::Ambiguity.limit() {
            let mut retained_total = 0_u64;
            let mut coherence_total = 0_i128;
            let mut eligible = true;
            for fold in 0..PRODUCTIVE_V1_INNER_FOLDS as u8 {
                let train = observations
                    .iter()
                    .copied()
                    .filter(|observation| observation.inner_fold != fold)
                    .collect::<Vec<_>>();
                let Ok(centers) = cluster_phase_waves(&train, k) else {
                    eligible = false;
                    break;
                };
                let members = groups
                    .iter()
                    .filter(|group| group.inner_fold == fold)
                    .flat_map(|group| &group.valid_members)
                    .filter(|member| member.slot_profile_id == *profile_id)
                    .collect::<Vec<_>>();
                if members.is_empty() {
                    eligible = false;
                    break;
                }
                let retained = members
                    .iter()
                    .filter_map(|member| {
                        let coherence = maximum_raw_coherence(&member.wave, &centers);
                        (coherence > 0).then_some(coherence)
                    })
                    .collect::<Vec<_>>();
                if retained.is_empty() {
                    eligible = false;
                    break;
                }
                retained_total += retained.len() as u64;
                coherence_total += retained.into_iter().map(i128::from).sum::<i128>();
            }
            if !eligible {
                continue;
            }
            let replace =
                selected
                    .as_ref()
                    .is_none_or(|(best_k, best_retained, best_coherence)| {
                        retained_total
                            .cmp(best_retained)
                            .reverse()
                            .then_with(|| coherence_total.cmp(best_coherence).reverse())
                            .then_with(|| k.cmp(best_k))
                            .is_lt()
                    });
            if replace {
                selected = Some((k, retained_total, coherence_total));
            }
        }
        if let Some((k, _, _)) = selected {
            selected_k.insert(*profile_id, k);
        }
    }

    if selected_k.is_empty() {
        return Ok(BTreeMap::new());
    }
    for fold in 0..PRODUCTIVE_V1_INNER_FOLDS as u8 {
        let fold_banks = selected_k
            .iter()
            .map(|(profile_id, k)| {
                let train = observations_by_profile[profile_id]
                    .iter()
                    .copied()
                    .filter(|observation| observation.inner_fold != fold)
                    .collect::<Vec<_>>();
                Ok((*profile_id, cluster_phase_waves(&train, *k)?))
            })
            .collect::<Result<BTreeMap<_, _>, &'static str>>()?;
        for group in groups.iter().filter(|group| group.inner_fold == fold) {
            let retained = group
                .valid_members
                .iter()
                .filter(|member| {
                    fold_banks
                        .get(&member.slot_profile_id)
                        .is_some_and(|centers| maximum_raw_coherence(&member.wave, centers) > 0)
                })
                .count();
            if retained == 1 {
                return Ok(BTreeMap::new());
            }
        }
    }

    selected_k
        .into_iter()
        .map(|(profile_id, k)| {
            let metadata = profile_metadata
                .get(&profile_id)
                .copied()
                .ok_or("ambiguity profile lacks package metadata")?;
            let observations = validated_observations(&observations_by_profile[&profile_id])?;
            let raw = cluster_phase_waves(&observations, k)?;
            Ok((
                profile_id,
                package_centers(PhaseBankKindV1::Ambiguity, k, raw, metadata.0, metadata.1),
            ))
        })
        .collect()
}

fn validated_observations(
    observations: &[PhaseObservationV1],
) -> Result<Vec<PhaseObservationV1>, &'static str> {
    let mut ordered = observations.to_vec();
    if ordered
        .iter()
        .any(|observation| observation.inner_fold >= PRODUCTIVE_V1_INNER_FOLDS as u8)
    {
        return Err("phase observation has an invalid inner fold");
    }
    ordered.sort_by_key(|observation| observation.event_identity);
    if ordered
        .windows(2)
        .any(|pair| pair[0].event_identity == pair[1].event_identity)
    {
        return Err("phase observations repeat an event identity");
    }
    Ok(ordered)
}

fn validated_ranking_groups(
    groups: &[PhaseRankingSelectionGroupV1],
) -> Result<Vec<PhaseRankingSelectionGroupV1>, &'static str> {
    let mut ordered = groups.to_vec();
    if ordered.iter().any(|group| {
        group.inner_fold >= PRODUCTIVE_V1_INNER_FOLDS as u8
            || group.members.is_empty()
            || group.comparators.is_empty()
    }) {
        return Err("phase ranking group is empty or has an invalid fold");
    }
    ordered.sort_by_key(|group| group.group_identity);
    if ordered
        .windows(2)
        .any(|pair| pair[0].group_identity == pair[1].group_identity)
    {
        return Err("phase ranking groups repeat an identity");
    }
    Ok(ordered)
}

fn validated_ambiguity_groups(
    groups: &[AmbiguityPhaseSelectionGroupV1],
) -> Result<Vec<AmbiguityPhaseSelectionGroupV1>, &'static str> {
    let mut ordered = groups.to_vec();
    if ordered.iter().any(|group| {
        group.inner_fold >= PRODUCTIVE_V1_INNER_FOLDS as u8 || group.valid_members.len() < 2
    }) {
        return Err("ambiguity phase group is not multi-label or has an invalid fold");
    }
    ordered.sort_by_key(|group| group.group_identity);
    if ordered
        .windows(2)
        .any(|pair| pair[0].group_identity == pair[1].group_identity)
    {
        return Err("ambiguity phase groups repeat an identity");
    }
    Ok(ordered)
}

fn ranking_fold_losses(
    groups: &[PhaseRankingSelectionGroupV1],
    mut centers_for_fold: impl FnMut(u8) -> Result<Vec<RawPhaseClusterV1>, &'static str>,
) -> Result<[f64; PRODUCTIVE_V1_INNER_FOLDS as usize], &'static str> {
    let mut losses = [0.0; PRODUCTIVE_V1_INNER_FOLDS as usize];
    for fold in 0..PRODUCTIVE_V1_INNER_FOLDS as u8 {
        let centers = centers_for_fold(fold)?;
        let fold_groups = groups
            .iter()
            .filter(|group| group.inner_fold == fold)
            .collect::<Vec<_>>();
        if fold_groups.is_empty() {
            return Err("phase ranking fold has no independently licensed pair");
        }
        let mut fold_loss = 0.0;
        for group in &fold_groups {
            let mut pair_loss_twice = 0_u64;
            let pair_count = group.members.len().saturating_mul(group.comparators.len());
            for member in &group.members {
                let member_coherence = maximum_raw_coherence(member, &centers);
                for comparator in &group.comparators {
                    let comparator_coherence = maximum_raw_coherence(comparator, &centers);
                    pair_loss_twice += match member_coherence.cmp(&comparator_coherence) {
                        Ordering::Greater => 0,
                        Ordering::Equal => 1,
                        Ordering::Less => 2,
                    };
                }
            }
            fold_loss += pair_loss_twice as f64 / (2 * pair_count) as f64;
        }
        losses[fold as usize] = fold_loss / fold_groups.len() as f64;
    }
    Ok(losses)
}

fn cluster_phase_waves(
    observations: &[PhaseObservationV1],
    k: usize,
) -> Result<Vec<RawPhaseClusterV1>, &'static str> {
    if k == 0 {
        return Ok(Vec::new());
    }
    if observations.len() < k {
        return Err("phase center count exceeds available observations");
    }
    let mut seeds = vec![0_usize];
    while seeds.len() < k {
        let next = (0..observations.len())
            .filter(|candidate| !seeds.contains(candidate))
            .min_by(|left, right| {
                let left_coherence = seeds
                    .iter()
                    .map(|seed| {
                        integer_cosine(&observations[*left].wave, &observations[*seed].wave)
                    })
                    .max()
                    .unwrap_or_default();
                let right_coherence = seeds
                    .iter()
                    .map(|seed| {
                        integer_cosine(&observations[*right].wave, &observations[*seed].wave)
                    })
                    .max()
                    .unwrap_or_default();
                left_coherence.cmp(&right_coherence).then_with(|| {
                    observations[*left]
                        .event_identity
                        .cmp(&observations[*right].event_identity)
                })
            })
            .ok_or("phase seed selection exhausted observations")?;
        seeds.push(next);
    }
    let mut centers = seeds
        .into_iter()
        .map(|index| observations[index].wave)
        .collect::<Vec<_>>();
    let mut previous_assignments = None::<Vec<usize>>;
    let mut final_support = vec![0_u32; k];
    for _ in 0..32 {
        let assignments = observations
            .iter()
            .map(|observation| {
                centers
                    .iter()
                    .enumerate()
                    .max_by(|(left_index, left), (right_index, right)| {
                        integer_cosine(&observation.wave, left)
                            .cmp(&integer_cosine(&observation.wave, right))
                            .then_with(|| right_index.cmp(left_index))
                    })
                    .map(|(index, _)| index)
                    .expect("k is nonzero")
            })
            .collect::<Vec<_>>();
        if previous_assignments.as_ref() == Some(&assignments) {
            break;
        }
        let mut sums = vec![[0_i64; L2_SCENE_PHASE_CELLS]; k];
        let mut support = vec![0_u32; k];
        for (observation, owner) in observations.iter().zip(&assignments) {
            support[*owner] = support[*owner]
                .checked_add(1)
                .ok_or("phase cluster support exceeds u32")?;
            for (sum, value) in sums[*owner].iter_mut().zip(observation.wave.0) {
                *sum = sum
                    .checked_add(i64::from(value))
                    .ok_or("phase cluster component sum exceeds i64")?;
            }
        }
        if support.contains(&0) {
            return Err("phase center candidate produces an empty cluster");
        }
        for index in 0..k {
            let mut mean = [0_i64; L2_SCENE_PHASE_CELLS];
            for (output, sum) in mean.iter_mut().zip(sums[index]) {
                *output = sum / i64::from(support[index]);
            }
            centers[index] = SceneWaveV1(normalize_mean(mean)?);
        }
        final_support = support;
        previous_assignments = Some(assignments);
    }
    if final_support.contains(&0) {
        return Err("phase center fitting did not produce a complete assignment");
    }
    Ok(centers
        .into_iter()
        .zip(final_support)
        .map(|(center, exact_support)| RawPhaseClusterV1 {
            cells: center.0,
            exact_support,
        })
        .collect())
}

fn normalize_mean(
    mean: [i64; L2_SCENE_PHASE_CELLS],
) -> Result<[i8; L2_SCENE_PHASE_CELLS], &'static str> {
    let maximum = mean
        .iter()
        .map(|value| value.unsigned_abs())
        .max()
        .unwrap_or_default();
    if maximum == 0 {
        return Ok([0; L2_SCENE_PHASE_CELLS]);
    }
    let mut normalized = [0_i8; L2_SCENE_PHASE_CELLS];
    for (output, value) in normalized.iter_mut().zip(mean) {
        let numerator = u128::from(value.unsigned_abs())
            .checked_mul(120)
            .ok_or("phase center normalization overflow")?;
        let magnitude = numerator
            .checked_add(u128::from(maximum) / 2)
            .ok_or("phase center rounding overflow")?
            / u128::from(maximum);
        let magnitude = i8::try_from(magnitude).map_err(|_| "phase center exceeds i8")?;
        *output = if value < 0 { -magnitude } else { magnitude };
    }
    Ok(normalized)
}

fn package_centers(
    kind: PhaseBankKindV1,
    selected_k: usize,
    raw: Vec<RawPhaseClusterV1>,
    feature_mask: u32,
    context_mode_id: u32,
) -> FittedPhaseBankV1 {
    let total = raw
        .iter()
        .map(|center| u64::from(center.exact_support))
        .sum::<u64>();
    let centers = raw
        .into_iter()
        .map(|center| {
            let saturated = center.exact_support > u32::from(u16::MAX);
            let mass = if total == 0 {
                0
            } else {
                ((u64::from(center.exact_support) * u64::from(u16::MAX) + total / 2) / total) as u16
            };
            FittedPhaseCenterV1 {
                cells: center.cells,
                feature_mask,
                context_mode_id,
                support: center.exact_support.min(u32::from(u16::MAX)) as u16,
                mass,
                polarity: kind as i8,
                flags: u8::from(saturated) * PHASE_CENTER_SUPPORT_SATURATED,
                exact_support: center.exact_support,
            }
        })
        .collect();
    FittedPhaseBankV1 {
        selected_k: selected_k as u8,
        centers,
    }
}

fn maximum_raw_coherence(wave: &SceneWaveV1, centers: &[RawPhaseClusterV1]) -> i64 {
    centers
        .iter()
        .map(|center| integer_cosine(wave, &SceneWaveV1(center.cells)))
        .max()
        .unwrap_or_default()
}

fn mean_loss(losses: &[f64; PRODUCTIVE_V1_INNER_FOLDS as usize]) -> f64 {
    losses.iter().sum::<f64>() / PRODUCTIVE_V1_INNER_FOLDS as f64
}

fn integer_sqrt(value: u128) -> u128 {
    if value < 2 {
        return value;
    }
    let mut low = 1_u128;
    let mut high = 1_u128 << ((128 - value.leading_zeros() as usize).div_ceil(2));
    while low + 1 < high {
        let middle = (low + high) / 2;
        if middle <= value / middle {
            low = middle;
        } else {
            high = middle;
        }
    }
    low
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wave(first: i8, second: i8) -> SceneWaveV1 {
        let mut cells = [0_i8; L2_SCENE_PHASE_CELLS];
        cells[0] = first;
        cells[1] = second;
        SceneWaveV1(cells)
    }

    fn observation(index: u8, fold: u8, wave: SceneWaveV1) -> PhaseObservationV1 {
        let mut event_identity = [0_u8; 32];
        event_identity[0] = index;
        PhaseObservationV1 {
            event_identity,
            inner_fold: fold,
            wave,
        }
    }

    #[test]
    fn integer_cosine_is_exact_for_zero_equal_and_opposite_waves() {
        let positive = wave(120, 0);
        assert_eq!(integer_cosine(&SceneWaveV1::default(), &positive), 0);
        assert_eq!(integer_cosine(&positive, &positive), 1_000_000);
        assert_eq!(integer_cosine(&positive, &wave(-120, 0)), -1_000_000);
    }

    #[test]
    fn deterministic_clustering_rejects_duplicate_empty_modes() {
        let observations = (0..5_u8)
            .map(|index| observation(index, index, wave(120, 0)))
            .collect::<Vec<_>>();
        assert!(cluster_phase_waves(&observations, 1).is_ok());
        assert_eq!(
            cluster_phase_waves(&observations, 2),
            Err("phase center candidate produces an empty cluster")
        );
    }

    #[test]
    fn ranking_bank_selects_centers_only_from_independent_fold_improvement() {
        let observations = (0..10_u8)
            .map(|index| observation(index, index % 5, wave(120, 0)))
            .collect::<Vec<_>>();
        let groups = (0..5_u8)
            .map(|fold| {
                let mut group_identity = [0_u8; 32];
                group_identity[0] = fold;
                PhaseRankingSelectionGroupV1 {
                    group_identity,
                    inner_fold: fold,
                    members: vec![wave(120, 0)],
                    comparators: vec![wave(-120, 0)],
                }
            })
            .collect::<Vec<_>>();
        let fitted =
            fit_ranking_phase_bank(PhaseBankKindV1::Positive, &observations, &groups, 7, 9)
                .expect("fit");
        assert_eq!(fitted.selected_k, 1);
        assert_eq!(fitted.centers.len(), 1);
        assert_eq!(fitted.centers[0].feature_mask, 7);
        assert_eq!(fitted.centers[0].context_mode_id, 9);
    }

    #[test]
    fn ambiguity_banks_are_independent_and_reject_false_singletons() {
        let observations_by_profile = BTreeMap::from([
            (
                1,
                (0..10_u8)
                    .map(|index| observation(index, index % 5, wave(120, 0)))
                    .collect(),
            ),
            (
                2,
                (0..10_u8)
                    .map(|index| observation(index.wrapping_add(32), index % 5, wave(0, 120)))
                    .collect(),
            ),
        ]);
        let groups = (0..5_u8)
            .map(|fold| {
                let mut group_identity = [0_u8; 32];
                group_identity[0] = fold;
                AmbiguityPhaseSelectionGroupV1 {
                    group_identity,
                    inner_fold: fold,
                    valid_members: vec![
                        AmbiguityPhaseMemberV1 {
                            slot_profile_id: 1,
                            wave: wave(120, 0),
                        },
                        AmbiguityPhaseMemberV1 {
                            slot_profile_id: 2,
                            wave: wave(0, 120),
                        },
                    ],
                }
            })
            .collect::<Vec<_>>();
        let fitted = fit_ambiguity_phase_banks(
            &observations_by_profile,
            &groups,
            &BTreeMap::from([(1, (1, 11)), (2, (2, 22))]),
        )
        .expect("fit ambiguity");
        assert_eq!(fitted.len(), 2);
        assert!(fitted.values().all(|bank| bank.selected_k == 1));
    }
}
