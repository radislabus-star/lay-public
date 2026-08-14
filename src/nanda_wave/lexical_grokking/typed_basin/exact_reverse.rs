use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::super::atoms::{encode_wave_surface, AtomChannel};
use super::super::compiler;
use super::super::format;
use super::super::model::{LexicalGrokkingPackage, WaveCoupling, COUPLING_FLAG_CHARACTER_ANCHOR};
use super::support::ExactSupportField;

const MAX_REVERSE_LEXICAL_COUPLINGS: usize = 96;

#[derive(Clone, Debug, Default)]
pub(super) struct ReverseBank {
    relations: BTreeMap<u32, Arc<[WaveCoupling]>>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ReverseParityMetrics {
    pub(super) terminals_compared: usize,
    pub(super) left_relations: usize,
    pub(super) right_relations: usize,
    pub(super) terminals_missing_left: usize,
    pub(super) terminals_missing_right: usize,
    pub(super) relations_missing_left: usize,
    pub(super) relations_missing_right: usize,
    pub(super) relation_state_mismatches: usize,
    pub(super) terminal_vector_mismatches: usize,
}

#[derive(Clone, Copy, Debug)]
struct ResolvedOccurrence {
    atom_id: u32,
    position: u16,
    channel: AtomChannel,
}

impl ReverseBank {
    pub(super) fn exact(
        package: &LexicalGrokkingPackage,
        support: &ExactSupportField,
        terminal_ids: &[u32],
    ) -> Result<Self, String> {
        Self::build(terminal_ids, |terminal_id| {
            reconstruct_exact_reverse(package, support, terminal_id)
        })
    }

    pub(super) fn compiler_reference(
        package: &LexicalGrokkingPackage,
        support: &ExactSupportField,
        terminal_ids: &[u32],
    ) -> Result<Self, String> {
        Self::build(terminal_ids, |terminal_id| {
            compiler::reference_depth0_reverse(package, support.values(), terminal_id)
        })
    }

    pub(super) fn current_v8(
        package: &LexicalGrokkingPackage,
        terminal_ids: &[u32],
    ) -> Result<Self, String> {
        Self::build(terminal_ids, |terminal_id| {
            format::reconstruct_compact_center_reverse(package, terminal_id)
        })
    }

    fn build(
        terminal_ids: &[u32],
        mut reconstruct: impl FnMut(u32) -> Result<Vec<WaveCoupling>, String>,
    ) -> Result<Self, String> {
        let mut relations = BTreeMap::new();
        for terminal_id in terminal_ids.iter().copied() {
            let value: Arc<[WaveCoupling]> = reconstruct(terminal_id)?.into();
            if relations.insert(terminal_id, value).is_some() {
                return Err(format!(
                    "reverse bank contains duplicate terminal: {terminal_id}"
                ));
            }
        }
        if relations.len() != terminal_ids.len() {
            return Err("reverse bank terminal cardinality differs".to_string());
        }
        Ok(Self { relations })
    }

    pub(super) fn get(&self, terminal_id: u32) -> Option<&[WaveCoupling]> {
        self.relations.get(&terminal_id).map(AsRef::as_ref)
    }

    pub(super) fn len(&self) -> usize {
        self.relations.len()
    }

    pub(super) fn relation_count(&self) -> usize {
        self.relations
            .values()
            .map(|relations| relations.len())
            .sum()
    }

    pub(super) fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"lay.l11.phase8i.reverse-bank.v1");
        for (terminal_id, relations) in &self.relations {
            hasher.update(terminal_id.to_le_bytes());
            hasher.update((relations.len() as u64).to_le_bytes());
            for relation in relations.iter().copied() {
                hasher.update(relation.peer_id.to_le_bytes());
                hasher.update([
                    relation.strength,
                    relation.phase_relation as u8,
                    relation.position_mode,
                    relation.flags,
                ]);
            }
        }
        format!("{:x}", hasher.finalize())
    }

    pub(super) fn compare(&self, other: &Self) -> ReverseParityMetrics {
        let terminal_ids = self
            .relations
            .keys()
            .chain(other.relations.keys())
            .copied()
            .collect::<BTreeSet<_>>();
        let mut metrics = ReverseParityMetrics::default();
        for terminal_id in terminal_ids {
            match (
                self.relations.get(&terminal_id),
                other.relations.get(&terminal_id),
            ) {
                (Some(left), Some(right)) => {
                    metrics.terminals_compared += 1;
                    metrics.left_relations += left.len();
                    metrics.right_relations += right.len();
                    metrics.relations_missing_left += right.len().saturating_sub(left.len());
                    metrics.relations_missing_right += left.len().saturating_sub(right.len());
                    metrics.relation_state_mismatches += left
                        .iter()
                        .zip(right.iter())
                        .filter(|(left, right)| left != right)
                        .count();
                    metrics.terminal_vector_mismatches += usize::from(left != right);
                }
                (Some(left), None) => {
                    metrics.terminals_missing_right += 1;
                    metrics.left_relations += left.len();
                    metrics.relations_missing_right += left.len();
                }
                (None, Some(right)) => {
                    metrics.terminals_missing_left += 1;
                    metrics.right_relations += right.len();
                    metrics.relations_missing_left += right.len();
                }
                (None, None) => unreachable!(),
            }
        }
        metrics
    }
}

impl ReverseParityMetrics {
    pub(super) fn mismatches(self) -> usize {
        self.terminals_missing_left
            .saturating_add(self.terminals_missing_right)
            .saturating_add(self.relations_missing_left)
            .saturating_add(self.relations_missing_right)
            .saturating_add(self.relation_state_mismatches)
            .saturating_add(self.terminal_vector_mismatches)
    }
}

fn reconstruct_exact_reverse(
    package: &LexicalGrokkingPackage,
    support: &ExactSupportField,
    terminal_id: u32,
) -> Result<Vec<WaveCoupling>, String> {
    let center = *package
        .centers
        .get(terminal_id as usize)
        .ok_or_else(|| format!("exact reverse terminal is invalid: {terminal_id}"))?;
    let surface = format::decode_center_surface(center, &package.decoder_nodes)?;
    let resolved = encode_wave_surface(&surface)
        .into_iter()
        .map(|atom| {
            let atom_id = package.graph.atom_id(atom.key).ok_or_else(|| {
                format!(
                    "exact reverse atom is absent from NGramGraph: terminal={terminal_id} channel={:?}",
                    atom.key.channel
                )
            })?;
            Ok(ResolvedOccurrence {
                atom_id,
                position: atom.position,
                channel: atom.key.channel,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut observations = BTreeMap::<u32, u32>::new();
    for occurrence in &resolved {
        let entry = observations.entry(occurrence.atom_id).or_default();
        *entry = entry
            .checked_add(1)
            .ok_or_else(|| "exact reverse observation count exceeds u32".to_string())?;
    }
    let mut relations = resolved
        .into_iter()
        .map(|occurrence| {
            let observation_count = observations
                .get(&occurrence.atom_id)
                .copied()
                .unwrap_or_default();
            let exact_support = support.get(occurrence.atom_id).ok_or_else(|| {
                format!("exact reverse atom lacks support: {}", occurrence.atom_id)
            })?;
            let position_mode = (occurrence.position / 257).min(255) as u8;
            Ok(WaveCoupling {
                peer_id: occurrence.atom_id,
                strength: coupling_strength(
                    observation_count,
                    exact_support,
                    package.centers.len(),
                ),
                phase_relation: position_phase(position_mode),
                position_mode,
                flags: if occurrence.channel == AtomChannel::CharacterAnchor {
                    COUPLING_FLAG_CHARACTER_ANCHOR
                } else {
                    0
                },
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    relations.sort_unstable_by(coupling_order);
    let anchor_count = relations
        .iter()
        .take_while(|relation| relation.flags != 0)
        .count();
    relations.truncate(anchor_count.saturating_add(MAX_REVERSE_LEXICAL_COUPLINGS));
    Ok(relations)
}

fn coupling_strength(observations: u32, atom_support: u32, word_count: usize) -> u8 {
    let reliability = observations.saturating_mul(255);
    let specificity =
        ((word_count as u32 + 1).saturating_mul(32) / atom_support.max(1)).clamp(32, 255);
    ((reliability.saturating_mul(specificity) / 255).clamp(1, 255)) as u8
}

fn position_phase(position: u8) -> i8 {
    (i16::from(position) - 128).clamp(-127, 127) as i8
}

fn coupling_order(left: &WaveCoupling, right: &WaveCoupling) -> std::cmp::Ordering {
    (right.flags != 0)
        .cmp(&(left.flags != 0))
        .then_with(|| {
            if left.flags != 0 && right.flags != 0 {
                left.position_mode.cmp(&right.position_mode)
            } else {
                right.strength.cmp(&left.strength)
            }
        })
        .then_with(|| left.peer_id.cmp(&right.peer_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_strength_does_not_saturate_support_to_u16() {
        assert_eq!(coupling_strength(1, 852_582, 852_582), 32);
        assert_eq!(coupling_strength(1, u16::MAX.into(), 852_582), 255);
    }
}
