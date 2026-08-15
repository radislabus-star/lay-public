//! Cold bounded merge for incremental L4 packages.

use std::collections::{BTreeMap, BTreeSet};
use std::io;

use super::model::{L4CrossScenePackage, L4CrossScenePairProfile, L4CrossSceneProfile};
use super::{
    ENCODER_HASH, ENCODER_VERSION, MAX_AMBIGUITY_CENTERS_PER_BANK, MAX_CENTERS_PER_BANK,
    MAX_HARD_CENTERS_PER_BANK, MAX_PAIR_PROFILES, MAX_PROFILES, MAX_SYMBOLS, SPLIT_COHERENCE,
};
use crate::nanda_wave::phase_field::{
    add_phase_vector, phase_center_from_sum, vector_phase_coherence, PhaseCenter,
};

pub(super) fn merge_package_delta(
    mut base: L4CrossScenePackage,
    delta: L4CrossScenePackage,
    applied_segment: u64,
) -> io::Result<L4CrossScenePackage> {
    require_v2(&base)?;
    require_v2(&delta)?;
    if delta.applied_segment != 0 || applied_segment <= base.applied_segment {
        return Err(invalid_data("invalid incremental L4 checkpoint"));
    }

    let mut symbols = std::mem::take(&mut base.symbols)
        .into_iter()
        .chain(delta.symbols)
        .collect::<BTreeSet<_>>();
    if symbols.len() > MAX_SYMBOLS {
        return Err(invalid_data("incremental L4 symbol budget exceeded"));
    }
    base.symbols = std::mem::take(&mut symbols).into_iter().collect();

    let mut profiles = std::mem::take(&mut base.profiles)
        .into_iter()
        .map(|profile| (profile.key, profile))
        .collect::<BTreeMap<_, _>>();
    for incoming in delta.profiles {
        if let Some(existing) = profiles.get_mut(&incoming.key) {
            merge_profile(existing, incoming);
        } else {
            profiles.insert(incoming.key, incoming);
        }
    }
    if profiles.len() > MAX_PROFILES {
        return Err(invalid_data("incremental L4 profile budget exceeded"));
    }
    base.profiles = profiles.into_values().collect();

    let mut pairs = std::mem::take(&mut base.pair_profiles)
        .into_iter()
        .map(|pair| ((pair.key, pair.low_relation, pair.high_relation), pair))
        .collect::<BTreeMap<_, _>>();
    for incoming in delta.pair_profiles {
        let key = (incoming.key, incoming.low_relation, incoming.high_relation);
        if let Some(existing) = pairs.get_mut(&key) {
            merge_pair(existing, incoming);
        } else {
            pairs.insert(key, incoming);
        }
    }
    if pairs.len() > MAX_PAIR_PROFILES {
        return Err(invalid_data("incremental L4 pair-profile budget exceeded"));
    }
    base.pair_profiles = pairs.into_values().collect();

    base.applied_segment = applied_segment;
    base.source_observations = base
        .source_observations
        .saturating_add(delta.source_observations);
    base.joined_observations = base
        .joined_observations
        .saturating_add(delta.joined_observations);
    base.positive_observations = base
        .positive_observations
        .saturating_add(delta.positive_observations);
    base.negative_observations = base
        .negative_observations
        .saturating_add(delta.negative_observations);
    base.reverted_observations = base
        .reverted_observations
        .saturating_add(delta.reverted_observations);
    base.ambiguity_observations = base
        .ambiguity_observations
        .saturating_add(delta.ambiguity_observations);
    base.censored_observations = base
        .censored_observations
        .saturating_add(delta.censored_observations);
    Ok(base)
}

pub(super) fn require_v2(package: &L4CrossScenePackage) -> io::Result<()> {
    if (package.encoder_version, package.encoder_hash) != (ENCODER_VERSION, ENCODER_HASH) {
        return Err(invalid_data("incremental L4 updater requires a V2 package"));
    }
    Ok(())
}

fn merge_profile(existing: &mut L4CrossSceneProfile, incoming: L4CrossSceneProfile) {
    existing.threshold_micro = existing.threshold_micro.max(incoming.threshold_micro);
    existing.positive_examples = existing
        .positive_examples
        .saturating_add(incoming.positive_examples);
    existing.negative_examples = existing
        .negative_examples
        .saturating_add(incoming.negative_examples);
    existing.reverted_examples = existing
        .reverted_examples
        .saturating_add(incoming.reverted_examples);
    existing.ambiguity_examples = existing
        .ambiguity_examples
        .saturating_add(incoming.ambiguity_examples);
    existing.censored_examples = existing
        .censored_examples
        .saturating_add(incoming.censored_examples);
    merge_bank(
        &mut existing.positive,
        incoming.positive,
        MAX_CENTERS_PER_BANK,
    );
    merge_bank(
        &mut existing.negative,
        incoming.negative,
        MAX_CENTERS_PER_BANK,
    );
    merge_bank(
        &mut existing.hard_negative,
        incoming.hard_negative,
        MAX_HARD_CENTERS_PER_BANK,
    );
    merge_bank(
        &mut existing.ambiguity,
        incoming.ambiguity,
        MAX_AMBIGUITY_CENTERS_PER_BANK,
    );
}

fn merge_pair(existing: &mut L4CrossScenePairProfile, incoming: L4CrossScenePairProfile) {
    existing.threshold_micro = existing.threshold_micro.max(incoming.threshold_micro);
    existing.observations = existing.observations.saturating_add(incoming.observations);
    merge_bank(
        &mut existing.low_wins,
        incoming.low_wins,
        MAX_CENTERS_PER_BANK,
    );
    merge_bank(
        &mut existing.high_wins,
        incoming.high_wins,
        MAX_CENTERS_PER_BANK,
    );
    merge_bank(
        &mut existing.hard_low_wins,
        incoming.hard_low_wins,
        MAX_HARD_CENTERS_PER_BANK,
    );
    merge_bank(
        &mut existing.hard_high_wins,
        incoming.hard_high_wins,
        MAX_HARD_CENTERS_PER_BANK,
    );
    merge_bank(
        &mut existing.ambiguity,
        incoming.ambiguity,
        MAX_AMBIGUITY_CENTERS_PER_BANK,
    );
}

fn merge_bank(target: &mut Vec<PhaseCenter>, incoming: Vec<PhaseCenter>, maximum: usize) {
    for mut center in incoming {
        center.materialize_sum();
        let best = target
            .iter_mut()
            .enumerate()
            .map(|(index, current)| {
                current.materialize_sum();
                (
                    index,
                    vector_phase_coherence(&center.center, &current.center),
                )
            })
            .max_by(|left, right| {
                left.1
                    .total_cmp(&right.1)
                    .then_with(|| right.0.cmp(&left.0))
            });
        if let Some((index, coherence)) = best {
            if coherence >= SPLIT_COHERENCE || target.len() >= maximum {
                let current = &mut target[index];
                add_phase_vector(&mut current.sum, &center.sum);
                current.center = phase_center_from_sum(&current.sum);
                current.support = current.support.saturating_add(center.support);
                continue;
            }
        }
        target.push(center);
    }
}

fn invalid_data(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::l4_cross_scene::model::L4CrossSceneProfileKey;
    use crate::nanda_wave::phase_field::{add_hashed_atom, empty_vector, phase_center_from_sum};
    use crate::transition_relation::TransitionOperatorKind;

    fn center(identity: u64, support: u32) -> PhaseCenter {
        let mut vector = empty_vector(super::super::CELLS);
        add_hashed_atom(&mut vector, identity, identity.rotate_left(7), 1.0);
        PhaseCenter::from_center(phase_center_from_sum(&vector), support)
    }

    fn profile(positive: bool) -> L4CrossSceneProfile {
        let key = L4CrossSceneProfileKey::new(TransitionOperatorKind::Other, None, None);
        L4CrossSceneProfile {
            key,
            threshold_micro: 40_000,
            positive: positive.then(|| center(11, 3)).into_iter().collect(),
            negative: (!positive).then(|| center(11, 2)).into_iter().collect(),
            hard_negative: Vec::new(),
            ambiguity: Vec::new(),
            positive_examples: if positive { 3 } else { 0 },
            negative_examples: if positive { 0 } else { 2 },
            reverted_examples: 0,
            ambiguity_examples: 0,
            censored_examples: 0,
        }
    }

    #[test]
    fn incremental_merge_keeps_old_and_new_signed_evidence() {
        let mut base = L4CrossScenePackage::default();
        base.profiles.push(profile(true));
        base.source_observations = 3;
        base.joined_observations = 3;
        base.positive_observations = 3;
        let base = super::super::format::canonical_runtime_package(&base).unwrap();
        let mut delta = L4CrossScenePackage::default();
        delta.profiles.push(profile(false));
        delta.source_observations = 2;
        delta.joined_observations = 2;
        delta.negative_observations = 2;

        let merged = merge_package_delta(base, delta, 9).unwrap();

        assert_eq!(merged.applied_segment, 9);
        assert_eq!(merged.source_observations, 5);
        assert_eq!(merged.profiles[0].positive_examples, 3);
        assert_eq!(merged.profiles[0].negative_examples, 2);
        assert!(!merged.profiles[0].positive.is_empty());
        assert!(!merged.profiles[0].negative.is_empty());
    }
}
