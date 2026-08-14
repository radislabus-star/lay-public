use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::super::atoms::{encode_wave_surface, AtomChannel};
use super::super::format;
use super::super::model::WaveCoupling;
use super::super::runtime::{ForwardActivation, LexicalGrokkingMemory, ObservedAtom};
use super::implicit_forward::ImplicitCandidate;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct OracleParityMetrics {
    pub(super) candidates_compared: usize,
    pub(super) canonical_relations_expected: usize,
    pub(super) implicit_relations_compared: usize,
    pub(super) implicit_relations_missing_from_v8: usize,
    pub(super) v8_relations_missing_implicitly: usize,
    pub(super) canonical_relations_missing_from_v8: usize,
    pub(super) implicit_relations_outside_canonical_surface: usize,
    pub(super) relation_state_mismatches: usize,
    pub(super) activation_mass_mismatches: usize,
    pub(super) activation_hits_mismatches: usize,
    pub(super) activation_surface_hits_mismatches: usize,
    pub(super) activation_keyboard_hits_mismatches: usize,
}

pub(super) struct V8QueryOracle {
    postings: BTreeMap<u32, Arc<[WaveCoupling]>>,
    expected_atom_ids: BTreeMap<u32, Vec<u32>>,
}

impl V8QueryOracle {
    pub(super) fn build(
        memory: &LexicalGrokkingMemory,
        observed: &BTreeMap<u32, ObservedAtom>,
        candidates: &[ImplicitCandidate],
    ) -> Result<Self, String> {
        let mut expected_atom_ids = BTreeMap::new();
        for candidate in candidates {
            let atom_ids = canonical_lexical_atom_ids(memory, candidate.terminal_id)?;
            if expected_atom_ids
                .insert(candidate.terminal_id, atom_ids)
                .is_some()
            {
                return Err(format!(
                    "V8 query oracle received duplicate terminal {}",
                    candidate.terminal_id
                ));
            }
        }
        let atom_ids = observed
            .keys()
            .copied()
            .chain(
                candidates
                    .iter()
                    .flat_map(|candidate| candidate.relations.iter().map(|item| item.atom_id)),
            )
            .chain(
                expected_atom_ids
                    .values()
                    .flat_map(|atom_ids| atom_ids.iter().copied()),
            )
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let decoded = memory.complete_forward_couplings_batch(&atom_ids)?;
        if atom_ids.len() != decoded.len() {
            return Err("V8 query oracle posting batch cardinality differs".to_string());
        }
        let mut postings = BTreeMap::new();
        for (atom_id, relations) in atom_ids.into_iter().zip(decoded) {
            validate_posting(atom_id, &relations, memory.package.terminal_count())?;
            postings.insert(atom_id, relations);
        }
        Ok(Self {
            postings,
            expected_atom_ids,
        })
    }

    pub(super) fn compare(
        &self,
        observed: &BTreeMap<u32, ObservedAtom>,
        candidates: &[ImplicitCandidate],
    ) -> OracleParityMetrics {
        let mut metrics = OracleParityMetrics::default();
        for candidate in candidates {
            metrics.candidates_compared += 1;
            let expected_atom_ids = self
                .expected_atom_ids
                .get(&candidate.terminal_id)
                .map(Vec::as_slice)
                .unwrap_or_default();
            metrics.canonical_relations_expected = metrics
                .canonical_relations_expected
                .saturating_add(expected_atom_ids.len());
            for atom_id in expected_atom_ids {
                let implicit = candidate
                    .relations
                    .binary_search_by_key(atom_id, |relation| relation.atom_id)
                    .ok()
                    .and_then(|index| candidate.relations.get(index));
                metrics.implicit_relations_compared += 1;
                let v8 = self
                    .postings
                    .get(atom_id)
                    .and_then(|relations| relation_for_terminal(relations, candidate.terminal_id));
                match (v8, implicit) {
                    (Some(v8), Some(implicit)) if v8 == implicit.coupling => {}
                    (Some(_), Some(_)) => metrics.relation_state_mismatches += 1,
                    (Some(_), None) => metrics.v8_relations_missing_implicitly += 1,
                    (None, Some(_)) => metrics.implicit_relations_missing_from_v8 += 1,
                    (None, None) => metrics.canonical_relations_missing_from_v8 += 1,
                }
            }
            metrics.implicit_relations_outside_canonical_surface = metrics
                .implicit_relations_outside_canonical_surface
                .saturating_add(
                    candidate
                        .relations
                        .iter()
                        .filter(|relation| {
                            expected_atom_ids.binary_search(&relation.atom_id).is_err()
                        })
                        .count(),
                );

            let expected = self.activation_for_terminal(candidate.terminal_id, observed);
            metrics.activation_mass_mismatches +=
                usize::from(expected.mass != candidate.activation.mass);
            metrics.activation_hits_mismatches +=
                usize::from(expected.hits != candidate.activation.hits);
            metrics.activation_surface_hits_mismatches +=
                usize::from(expected.surface_hits != candidate.activation.surface_hits);
            metrics.activation_keyboard_hits_mismatches +=
                usize::from(expected.keyboard_hits != candidate.activation.keyboard_hits);
        }
        metrics
    }

    fn activation_for_terminal(
        &self,
        terminal_id: u32,
        observed: &BTreeMap<u32, ObservedAtom>,
    ) -> ForwardActivation {
        let mut activation = ForwardActivation::default();
        for (atom_id, atom) in observed {
            let Some(relation) = self
                .postings
                .get(atom_id)
                .and_then(|relations| relation_for_terminal(relations, terminal_id))
            else {
                continue;
            };
            activation.mass = activation.mass.saturating_add(
                u64::from(relation.strength)
                    .saturating_mul(u64::from(atom.weight))
                    .saturating_mul(u64::from(position_coherence(
                        atom.position,
                        relation.position_mode,
                    ))),
            );
            activation.hits = activation.hits.saturating_add(1);
            if is_keyboard_channel(atom.channel) {
                activation.keyboard_hits = activation.keyboard_hits.saturating_add(1);
            } else {
                activation.surface_hits = activation.surface_hits.saturating_add(1);
            }
        }
        activation
    }
}

impl OracleParityMetrics {
    pub(super) fn relation_mismatches(self) -> usize {
        self.implicit_relations_missing_from_v8
            .saturating_add(self.v8_relations_missing_implicitly)
            .saturating_add(self.canonical_relations_missing_from_v8)
            .saturating_add(self.implicit_relations_outside_canonical_surface)
            .saturating_add(self.relation_state_mismatches)
    }

    pub(super) fn activation_mismatches(self) -> usize {
        self.activation_mass_mismatches
            .saturating_add(self.activation_hits_mismatches)
            .saturating_add(self.activation_surface_hits_mismatches)
            .saturating_add(self.activation_keyboard_hits_mismatches)
    }
}

fn canonical_lexical_atom_ids(
    memory: &LexicalGrokkingMemory,
    terminal_id: u32,
) -> Result<Vec<u32>, String> {
    let center = *memory
        .package
        .centers
        .get(terminal_id as usize)
        .ok_or_else(|| format!("V8 query oracle terminal is invalid: {terminal_id}"))?;
    let surface = format::decode_center_surface(center, &memory.package.decoder_nodes)?;
    let mut atom_ids = encode_wave_surface(&surface)
        .into_iter()
        .filter(|atom| atom.key.channel != AtomChannel::CharacterAnchor)
        .map(|atom| {
            memory.package.graph.atom_id(atom.key).ok_or_else(|| {
                format!(
                    "V8 query oracle canonical atom is unresolved: terminal={terminal_id} channel={:?}",
                    atom.key.channel
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    atom_ids.sort_unstable();
    atom_ids.dedup();
    Ok(atom_ids)
}

fn validate_posting(
    atom_id: u32,
    relations: &[WaveCoupling],
    terminal_count: u32,
) -> Result<(), String> {
    let mut previous = None;
    for relation in relations {
        if relation.peer_id >= terminal_count || relation.flags != 0 {
            return Err(format!(
                "V8 query oracle posting is invalid: atom={atom_id}"
            ));
        }
        if previous.is_some_and(|peer| peer >= relation.peer_id) {
            return Err(format!(
                "V8 query oracle posting is not strictly terminal ordered: atom={atom_id}"
            ));
        }
        previous = Some(relation.peer_id);
    }
    Ok(())
}

fn relation_for_terminal(relations: &[WaveCoupling], terminal_id: u32) -> Option<WaveCoupling> {
    relations
        .binary_search_by_key(&terminal_id, |relation| relation.peer_id)
        .ok()
        .map(|index| relations[index])
}

fn position_coherence(observed: u8, expected: u8) -> u16 {
    256_u16.saturating_sub(u16::from(observed.abs_diff(expected)))
}

fn is_keyboard_channel(channel: AtomChannel) -> bool {
    matches!(
        channel,
        AtomChannel::KeyboardGram
            | AtomChannel::KeyboardBigram
            | AtomChannel::KeyboardBagGram
            | AtomChannel::KeyboardSkipGram
    )
}
