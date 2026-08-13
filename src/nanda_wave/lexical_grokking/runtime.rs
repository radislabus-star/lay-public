use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use rayon::prelude::*;

mod config;
mod contract;
mod diagnostics;
mod geometry;
mod host;
mod legacy;
mod relations;
mod settlement;

use legacy::select_birth_atoms;
#[cfg(test)]
use legacy::should_expand_operator_lattice;
#[cfg(test)]
use settlement::truncate_with_reconstruction_tail;
#[allow(unused_imports)]
pub(super) use settlement::{
    apply_geometry_certificate_interference, apply_position_certificate_interference,
    apply_sequence_certificate_interference, candidate_order, legacy_sequence_coherence_milli,
    sequence_coherence_milli,
};
use settlement::{
    is_anchor_channel, is_keyboard_channel, observed_sequence, position_coherence,
    reconstruction_mode_rank,
};

use config::{
    birth_atoms_per_channel, birth_posting_budget, first_touch_profile_word_count,
    readout_trace_enabled, readout_trace_terminal, FIRST_TOUCH_TRANSIENT_RESERVE_MIB,
};
#[cfg(test)]
use config::{DEFAULT_BIRTH_ATOMS_PER_CHANNEL, DEFAULT_BIRTH_POSTING_BUDGET};
pub(super) use contract::{AmbiguityObservation, GrokkingCandidate, ReadoutMode};
use contract::{
    AnchorSequence, BirthAtom, CachePlanOrder, FirstTouchWarmProfile, ForwardActivation,
    ObservedAtom, PreparedReadout,
};
pub(super) use diagnostics::candidate_json;
use diagnostics::percent_usize;
pub use diagnostics::{
    benchmark_diverse_restoration, benchmark_package, inspect_package_header, query_package,
    restore_surface,
};
use geometry::MAX_ANCHOR_SEQUENCE;
pub(super) use geometry::{
    ambiguity_geometry_link, damerau_distance, reconstruction_modes,
    surface_operator_reconstruction_modes, RECONSTRUCTION_MODE_DELETION,
    RECONSTRUCTION_MODE_DELETION_TRANSPOSITION, RECONSTRUCTION_MODE_DOUBLE_SUBSTITUTION,
    RECONSTRUCTION_MODE_NON_ADJACENT_TRANSPOSITION, RECONSTRUCTION_MODE_PREFIX_TRUNCATION,
    RECONSTRUCTION_MODE_SINGLE_DELETION, RECONSTRUCTION_MODE_SINGLE_SUBSTITUTION,
    RECONSTRUCTION_MODE_SUFFIX_TRUNCATION,
};
#[cfg(test)]
use geometry::{is_ordered_subsequence, is_subsequence_after_one_adjacent_swap};
pub use host::{L1RestorationHost, L1RestorationHostStats};
use relations::{RelationStore, ReverseCache};

use crate::stable_hash::mix64_golden;

use super::atoms::{
    encode_wave_surface, normalize_lexical_surface, physical_key_sequence, AtomChannel, NGramKey,
};
use super::corruption::split_scale_damages;
use super::crystal::{AmbiguityPhaseCenter64, WAVE_DIMENSION};
use super::format;
use super::model::{
    LexicalGrokkingPackage, WaveCoupling, CENTER_PHASE_FLAG_PHYSICAL_KEY_GEOMETRY,
    COUPLING_FLAG_CHARACTER_ANCHOR,
};
use super::v8::{self, V8Artifact};
use super::wave_basis::{
    complex_coherence_milli, expand_atom, expand_word, pair_residual_atoms, positioned_atom_code,
};

const MAX_PHASE_FRONTIER: usize = 128;
const MAX_GEOMETRY_RESERVE: usize = 32;
const MAX_OPERATOR_RESERVE: usize = 64;
const MAX_RECONSTRUCTION_RESERVE: usize = 64;
const MAX_RECONSTRUCTION_SCAN: usize = 8_192;
const MAX_RECONSTRUCTION_TAIL: usize = 32;
const MAX_GEOMETRY_SCAN: usize = 1_024;
const SETTLING_ITERATIONS: u8 = 3;
const MAX_EXACT_COLLISION_OPERATOR_CHARS: usize = 16;

pub(super) struct LexicalGrokkingMemory {
    pub(super) package: LexicalGrokkingPackage,
    exact_surface_index: HashMap<u64, u32>,
    exact_surface_collisions: HashMap<u64, Vec<u32>>,
    character_anchor_by_char: HashMap<char, u32>,
    character_anchor_offsets: Vec<u32>,
    character_anchor_atoms: Vec<u32>,
    relations: RelationStore,
    reverse_cache: Mutex<ReverseCache>,
}

impl LexicalGrokkingMemory {
    pub(super) fn from_package(package: LexicalGrokkingPackage) -> Self {
        let (
            exact_surface_index,
            exact_surface_collisions,
            character_anchor_by_char,
            character_anchor_offsets,
            character_anchor_atoms,
        ) = compile_surface_indices(&package);
        Self {
            package,
            exact_surface_index,
            exact_surface_collisions,
            character_anchor_by_char,
            character_anchor_offsets,
            character_anchor_atoms,
            relations: RelationStore::Eager,
            reverse_cache: Mutex::new(ReverseCache::default()),
        }
    }

    pub(super) fn into_package(self) -> LexicalGrokkingPackage {
        self.package
    }

    pub(super) fn ambiguity_center_count(&self) -> usize {
        self.package.ambiguity_subcenters.len()
    }

    pub(super) fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if v8::is_v8(bytes) {
            return Err("L1.1 V8 must be loaded from a path for mmap access".to_string());
        }
        Ok(Self::from_package(format::decode(bytes)?))
    }

    pub(super) fn load(path: &Path) -> Result<Self, String> {
        let mut prefix = [0_u8; 8];
        {
            use std::io::Read;
            let mut file = std::fs::File::open(path)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            file.read_exact(&mut prefix)
                .map_err(|error| format!("{}: {error}", path.display()))?;
        }
        if !v8::is_v8(&prefix) {
            let bytes =
                std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
            return Self::from_bytes(&bytes);
        }
        let artifact = V8Artifact::load(path)?;
        let package = artifact.decode_base()?;
        let (
            exact_surface_index,
            exact_surface_collisions,
            character_anchor_by_char,
            character_anchor_offsets,
            character_anchor_atoms,
        ) = compile_surface_indices(&package);
        Ok(Self {
            package,
            exact_surface_index,
            exact_surface_collisions,
            character_anchor_by_char,
            character_anchor_offsets,
            character_anchor_atoms,
            relations: RelationStore::LazyV8(artifact),
            reverse_cache: Mutex::new(ReverseCache::default()),
        })
    }

    pub(super) fn readout(
        &self,
        surface: &str,
        limit: usize,
        mode: ReadoutMode,
    ) -> Vec<GrokkingCandidate> {
        if limit == 0 {
            return Vec::new();
        }
        if limit == 1 && mode == ReadoutMode::Full {
            if let Some(candidate) = self.exact_singleton_readout(surface) {
                return vec![candidate];
            }
        }
        let Some(prepared) = self.prepare_readout(surface, limit) else {
            return Vec::new();
        };
        self.finish_readout(surface, limit, mode, &prepared)
    }

    pub(super) fn warm_first_touch(&self) -> io::Result<serde_json::Value> {
        let RelationStore::LazyV8(artifact) = &self.relations else {
            return Ok(serde_json::json!({
                "format": "eager",
                "eligible_atoms": 0,
                "eligible_relations": 0,
            }));
        };
        let profile = self.first_touch_warm_profile(artifact.posting_cache_budget_bytes());
        let stats = artifact
            .warm_first_touch(&profile.atom_ids)
            .map_err(io::Error::other)?;
        Ok(serde_json::json!({
            "format": "v8",
            "sampled_words": profile.sampled_words,
            "damage_surfaces": profile.damage_surfaces,
            "observed_atoms": profile.observed_atoms,
            "protected_budget_bytes": profile.protected_budget_bytes,
            "eligible_atoms": stats.eligible_atoms,
            "eligible_relations": stats.eligible_relations,
            "posting_cache_bytes": stats.posting_cache_bytes,
            "posting_cache_entries": stats.posting_cache_entries,
            "protected_cache_bytes": stats.protected_cache_bytes,
            "protected_cache_entries": stats.protected_cache_entries,
            "shard_cache_bytes": stats.shard_cache_bytes,
            "shard_cache_entries": stats.shard_cache_entries,
        }))
    }

    fn first_touch_warm_profile(&self, cache_budget_bytes: usize) -> FirstTouchWarmProfile {
        let terminal_count = self.package.terminal_count() as usize;
        let sampled_words = first_touch_profile_word_count().min(terminal_count);
        let mut atom_uses = BTreeMap::<u32, usize>::new();
        let mut damage_surfaces = 0_usize;
        for sample in 0..sampled_words {
            let terminal_id = sample
                .saturating_mul(terminal_count)
                .saturating_add(sampled_words / 2)
                / sampled_words.max(1);
            let Some(word) = self.decode_terminal(terminal_id as u32) else {
                continue;
            };
            let (training, heldout) = split_scale_damages(&word, false);
            let mut by_class = BTreeMap::<&'static str, String>::new();
            for example in training.into_iter().chain(heldout) {
                by_class
                    .entry(example.class)
                    .and_modify(|surface| {
                        if example.surface < *surface {
                            *surface = example.surface.clone();
                        }
                    })
                    .or_insert(example.surface);
            }
            for surface in by_class.into_values() {
                damage_surfaces += 1;
                let observed = self.resolve_surface(&surface);
                let mut by_channel: [Vec<BirthAtom>; 12] = std::array::from_fn(|_| Vec::new());
                for (atom_id, atom) in observed
                    .iter()
                    .filter(|(_, atom)| !is_anchor_channel(atom.channel))
                {
                    by_channel[atom.channel as usize].push((
                        self.forward_degree(*atom_id),
                        *atom_id,
                        *atom,
                    ));
                }
                for (_, atom_id, _) in select_birth_atoms(
                    &mut by_channel,
                    birth_atoms_per_channel(),
                    birth_posting_budget(),
                ) {
                    *atom_uses.entry(atom_id).or_default() += 1;
                }
            }
        }
        let observed_atoms = atom_uses.len();
        let mut ranked = atom_uses
            .into_iter()
            .map(|(atom_id, uses)| (uses, self.forward_degree(atom_id), atom_id))
            .filter(|(_, degree, _)| *degree != 0)
            .collect::<Vec<_>>();
        ranked.sort_unstable_by(|left, right| {
            right
                .0
                .cmp(&left.0)
                .then_with(|| right.1.cmp(&left.1))
                .then_with(|| left.2.cmp(&right.2))
        });
        let reserve = FIRST_TOUCH_TRANSIENT_RESERVE_MIB.saturating_mul(1024 * 1024);
        let protected_budget_bytes =
            cache_budget_bytes.saturating_sub(reserve.min(cache_budget_bytes));
        let mut protected_bytes = 0_usize;
        let mut atom_ids = Vec::new();
        for (_, degree, atom_id) in ranked {
            let posting_bytes = degree.saturating_mul(std::mem::size_of::<WaveCoupling>());
            if posting_bytes > protected_budget_bytes.saturating_sub(protected_bytes) {
                continue;
            }
            protected_bytes = protected_bytes.saturating_add(posting_bytes);
            atom_ids.push(atom_id);
        }
        FirstTouchWarmProfile {
            atom_ids,
            sampled_words,
            damage_surfaces,
            observed_atoms,
            protected_budget_bytes,
        }
    }

    fn birth_profile(&self, surfaces: &[String]) -> serde_json::Value {
        let mut uses = BTreeMap::<u32, (usize, AtomChannel)>::new();
        let mut selected_references = 0_usize;
        let mut selected_relations = 0_usize;
        for surface in surfaces {
            let observed = self.resolve_surface(surface);
            let mut by_channel: [Vec<BirthAtom>; 12] = std::array::from_fn(|_| Vec::new());
            for (atom_id, atom) in observed
                .iter()
                .filter(|(_, atom)| !is_anchor_channel(atom.channel))
            {
                by_channel[atom.channel as usize].push((
                    self.forward_degree(*atom_id),
                    *atom_id,
                    *atom,
                ));
            }
            for (degree, atom_id, atom) in select_birth_atoms(
                &mut by_channel,
                birth_atoms_per_channel(),
                birth_posting_budget(),
            ) {
                selected_references += 1;
                selected_relations = selected_relations.saturating_add(degree);
                uses.entry(atom_id)
                    .and_modify(|entry| entry.0 += 1)
                    .or_insert((1, atom.channel));
            }
        }
        let unique_relations = uses
            .keys()
            .map(|atom_id| self.forward_degree(*atom_id))
            .sum::<usize>();
        let mut hottest = uses
            .iter()
            .map(|(atom_id, (uses, channel))| {
                let atom_id = *atom_id;
                let uses = *uses;
                let channel = *channel;
                let degree = self.forward_degree(atom_id);
                (uses.saturating_mul(degree), atom_id, uses, degree, channel)
            })
            .collect::<Vec<_>>();
        hottest.sort_unstable_by(|left, right| {
            right
                .0
                .cmp(&left.0)
                .then_with(|| right.3.cmp(&left.3))
                .then_with(|| left.1.cmp(&right.1))
        });
        let cache_plans = [96_usize, 128]
            .into_iter()
            .map(|budget_mib| {
                let budget_bytes = budget_mib.saturating_mul(1024 * 1024);
                serde_json::json!({
                    "budget_mib": budget_mib,
                    "support": self.simulate_posting_cache_plan(
                        &uses,
                        selected_references,
                        selected_relations,
                        budget_bytes,
                        CachePlanOrder::Support,
                    ),
                    "degree": self.simulate_posting_cache_plan(
                        &uses,
                        selected_references,
                        selected_relations,
                        budget_bytes,
                        CachePlanOrder::Degree,
                    ),
                    "oracle": self.simulate_posting_cache_plan(
                        &uses,
                        selected_references,
                        selected_relations,
                        budget_bytes,
                        CachePlanOrder::ObservedUses,
                    ),
                })
            })
            .collect::<Vec<_>>();
        serde_json::json!({
            "selected_references": selected_references,
            "selected_relations": selected_relations,
            "unique_atoms": hottest.len(),
            "unique_relations": unique_relations,
            "unique_decoded_bytes": unique_relations.saturating_mul(std::mem::size_of::<WaveCoupling>()),
            "cache_plans": cache_plans,
            "top_expected_decode_work": hottest
                .into_iter()
                .take(64)
                .map(|(work, atom_id, uses, degree, channel)| serde_json::json!({
                    "atom_id": atom_id,
                    "channel": format!("{channel:?}"),
                    "uses": uses,
                    "degree": degree,
                    "expected_decode_work": work,
                    "decoded_bytes": degree.saturating_mul(std::mem::size_of::<WaveCoupling>()),
                }))
                .collect::<Vec<_>>(),
        })
    }

    fn simulate_posting_cache_plan(
        &self,
        observed_uses: &BTreeMap<u32, (usize, AtomChannel)>,
        selected_references: usize,
        selected_relations: usize,
        budget_bytes: usize,
        order: CachePlanOrder,
    ) -> serde_json::Value {
        let mut atoms = self
            .package
            .atoms
            .iter()
            .enumerate()
            .map(|(atom_id, record)| {
                let atom_id = atom_id as u32;
                let degree = self.forward_degree(atom_id);
                let uses = observed_uses
                    .get(&atom_id)
                    .map(|entry| entry.0)
                    .unwrap_or_default();
                (atom_id, usize::from(record.support), degree, uses)
            })
            .collect::<Vec<_>>();
        atoms.sort_unstable_by(|left, right| {
            let left_key = match order {
                CachePlanOrder::Support => left.1,
                CachePlanOrder::Degree => left.2,
                CachePlanOrder::ObservedUses => left.3,
            };
            let right_key = match order {
                CachePlanOrder::Support => right.1,
                CachePlanOrder::Degree => right.2,
                CachePlanOrder::ObservedUses => right.3,
            };
            right_key
                .cmp(&left_key)
                .then_with(|| right.2.cmp(&left.2))
                .then_with(|| left.0.cmp(&right.0))
        });
        let mut bytes = 0_usize;
        let mut atom_count = 0_usize;
        let mut relation_count = 0_usize;
        let mut observed_reference_hits = 0_usize;
        let mut observed_relation_hits = 0_usize;
        for (_, _, degree, uses) in atoms {
            let posting_bytes = degree.saturating_mul(std::mem::size_of::<WaveCoupling>());
            if posting_bytes > budget_bytes.saturating_sub(bytes) {
                continue;
            }
            bytes = bytes.saturating_add(posting_bytes);
            atom_count += 1;
            relation_count = relation_count.saturating_add(degree);
            observed_reference_hits = observed_reference_hits.saturating_add(uses);
            observed_relation_hits =
                observed_relation_hits.saturating_add(uses.saturating_mul(degree));
        }
        serde_json::json!({
            "atom_count": atom_count,
            "relation_count": relation_count,
            "bytes": bytes,
            "observed_reference_coverage_percent": percent_usize(
                observed_reference_hits,
                selected_references,
            ),
            "observed_decode_work_coverage_percent": percent_usize(
                observed_relation_hits,
                selected_relations,
            ),
        })
    }

    pub(super) fn decode_terminal(&self, terminal_id: u32) -> Option<String> {
        let center = *self.package.centers.get(terminal_id as usize)?;
        let mut node = center.decoder_terminal;
        let mut symbols = Vec::new();
        while node != 0 {
            let item = *self.package.decoder_nodes.get(node as usize)?;
            symbols.push(char::from_u32(item.symbol)?);
            node = item.parent;
        }
        symbols.reverse();
        Some(symbols.into_iter().collect())
    }

    pub(super) fn character_anchors(&self, terminal_id: u32) -> &[u32] {
        let index = terminal_id as usize;
        let Some(&start) = self.character_anchor_offsets.get(index) else {
            return &[];
        };
        let Some(&end) = self.character_anchor_offsets.get(index.saturating_add(1)) else {
            return &[];
        };
        self.character_anchor_atoms
            .get(start as usize..end as usize)
            .unwrap_or_default()
    }
}

fn compile_surface_indices(
    package: &LexicalGrokkingPackage,
) -> (
    HashMap<u64, u32>,
    HashMap<u64, Vec<u32>>,
    HashMap<char, u32>,
    Vec<u32>,
    Vec<u32>,
) {
    let mut index = HashMap::with_capacity(package.centers.len());
    let mut collisions = HashMap::<u64, Vec<u32>>::new();
    let mut anchor_by_char = HashMap::new();
    let mut offsets = Vec::with_capacity(package.centers.len().saturating_add(1));
    let mut atoms = Vec::new();
    offsets.push(0);
    for (terminal, center) in package.centers.iter().enumerate() {
        let mut anchors = AnchorSequence::default();
        let mut complete = false;
        if let Ok(surface) = format::decode_center_surface(*center, &package.decoder_nodes) {
            complete = true;
            for (position, ch) in surface.chars().take(MAX_ANCHOR_SEQUENCE).enumerate() {
                let Some(atom_id) = package.graph.atom_id(NGramKey {
                    channel: AtomChannel::CharacterAnchor,
                    len: 1,
                    units: [ch as u32, 0, 0, 0],
                }) else {
                    complete = false;
                    break;
                };
                anchor_by_char.entry(ch).or_insert(atom_id);
                anchors.atoms[position] = atom_id;
                anchors.len = anchors.len.saturating_add(1);
            }
        }
        if complete && !anchors.as_slice().is_empty() {
            let hash = anchor_sequence_hash(anchors.as_slice());
            match index.entry(hash) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(terminal as u32);
                }
                std::collections::hash_map::Entry::Occupied(_) => {
                    collisions.entry(hash).or_default().push(terminal as u32);
                }
            }
            atoms.extend_from_slice(anchors.as_slice());
        }
        offsets.push(atoms.len() as u32);
    }
    (index, collisions, anchor_by_char, offsets, atoms)
}

fn anchor_sequence_hash(sequence: &[u32]) -> u64 {
    let mut state = mix64_golden(0x4c31_4558_4143_5431 ^ sequence.len() as u64);
    for atom in sequence {
        state = mix64_golden(state ^ u64::from(*atom));
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_birth_keeps_a_rare_budgeted_channel_frontier() {
        let mut channels: [Vec<BirthAtom>; 12] = std::array::from_fn(|_| Vec::new());
        channels[AtomChannel::CharacterGram as usize] = (0_u32..40)
            .map(|atom_id| {
                (
                    (40 - atom_id) as usize,
                    atom_id,
                    ObservedAtom {
                        position: atom_id as u8,
                        weight: 1,
                        channel: AtomChannel::CharacterGram,
                    },
                )
            })
            .collect();

        let selected = select_birth_atoms(
            &mut channels,
            DEFAULT_BIRTH_ATOMS_PER_CHANNEL,
            DEFAULT_BIRTH_POSTING_BUDGET,
        );

        assert_eq!(DEFAULT_BIRTH_ATOMS_PER_CHANNEL, 4);
        assert_eq!(selected.len(), 4);
        assert_eq!(selected.first().map(|atom| atom.1), Some(39));
        assert_eq!(selected.last().map(|atom| atom.1), Some(36));
    }

    #[test]
    fn candidate_birth_stays_within_the_global_posting_budget() {
        let mut channels: [Vec<BirthAtom>; 12] = std::array::from_fn(|_| Vec::new());
        for (channel, atoms) in channels.iter_mut().take(3).enumerate() {
            *atoms = (0_u32..4)
                .map(|atom_id| {
                    (
                        50_000,
                        atom_id + channel as u32 * 10,
                        ObservedAtom {
                            position: atom_id as u8,
                            weight: 1,
                            channel: AtomChannel::CharacterGram,
                        },
                    )
                })
                .collect();
        }

        let selected = select_birth_atoms(&mut channels, 4, DEFAULT_BIRTH_POSTING_BUDGET);

        assert_eq!(selected.len(), 2);
        assert!(selected.iter().map(|atom| atom.0).sum::<usize>() <= DEFAULT_BIRTH_POSTING_BUDGET);
    }

    #[test]
    fn geometry_reserve_keeps_the_nearest_basin_and_ambiguity_shell() {
        let anchor_sequences = [[1_u32, 9, 9], [1, 2, 4], [1, 2, 5]];
        let reverse_couplings = anchor_sequences
            .iter()
            .flatten()
            .map(|atom_id| WaveCoupling {
                peer_id: *atom_id,
                flags: COUPLING_FLAG_CHARACTER_ANCHOR,
                ..WaveCoupling::default()
            })
            .collect::<Vec<_>>();
        let centers = (0..anchor_sequences.len())
            .map(|terminal_id| super::super::crystal::WordCenter64 {
                coupling_start: (terminal_id * 3) as u32,
                coupling_count: 3,
                ..Default::default()
            })
            .collect();
        let memory = LexicalGrokkingMemory {
            package: LexicalGrokkingPackage {
                centers,
                reverse_couplings,
                restoration_calibration: super::super::restoration::RestorationCalibration {
                    max_geometry_distance: 2,
                    ..Default::default()
                },
                ..Default::default()
            },
            exact_surface_index: HashMap::new(),
            exact_surface_collisions: HashMap::new(),
            character_anchor_by_char: HashMap::new(),
            character_anchor_offsets: vec![0, 3, 6, 9],
            character_anchor_atoms: anchor_sequences.into_iter().flatten().collect(),
            relations: RelationStore::Eager,
            reverse_cache: Mutex::new(ReverseCache::default()),
        };
        let frontier = vec![
            (
                0,
                ForwardActivation {
                    mass: 10_000,
                    hits: 10,
                    ..Default::default()
                },
            ),
            (
                1,
                ForwardActivation {
                    mass: 100,
                    hits: 2,
                    ..Default::default()
                },
            ),
            (
                2,
                ForwardActivation {
                    mass: 90,
                    hits: 2,
                    ..Default::default()
                },
            ),
        ];

        let reserve = memory.geometry_reserve(&frontier, &[1, 2, 3]);

        assert_eq!(
            reserve
                .into_iter()
                .map(|(terminal_id, _)| terminal_id)
                .collect::<Vec<_>>(),
            vec![1, 2, 0]
        );
    }

    #[test]
    fn reconstruction_origin_does_not_override_geometry_evidence() {
        let primary = GrokkingCandidate {
            terminal_id: 1,
            geometry_distance: 2,
            settled_energy: 900,
            ..Default::default()
        };
        let reconstructed_tail = GrokkingCandidate {
            terminal_id: 2,
            reconstruction_only: true,
            reconstruction_modes: RECONSTRUCTION_MODE_DELETION,
            geometry_distance: 1,
            settled_energy: 1_000,
            ..Default::default()
        };
        let mut candidates = [primary, reconstructed_tail];

        apply_geometry_certificate_interference(&mut candidates);

        assert_eq!(candidates[0].terminal_id, reconstructed_tail.terminal_id);
        assert_eq!(candidates[1].terminal_id, primary.terminal_id);
    }

    #[test]
    fn exact_two_omission_inverse_operator_can_cross_raw_edit_distance() {
        let nearer_incumbent = GrokkingCandidate {
            terminal_id: 1,
            geometry_distance: 1,
            settled_energy: 7_200,
            ..Default::default()
        };
        let two_omission_reconstruction = GrokkingCandidate {
            terminal_id: 2,
            reconstruction_modes: RECONSTRUCTION_MODE_DELETION,
            sequence_milli: 1_000,
            geometry_distance: 2,
            settled_energy: 6_000,
            ..Default::default()
        };
        let mut candidates = [nearer_incumbent, two_omission_reconstruction];

        apply_geometry_certificate_interference(&mut candidates);

        assert_eq!(
            candidates[0].terminal_id,
            two_omission_reconstruction.terminal_id
        );
    }

    #[test]
    fn inverse_operator_cannot_spend_unbounded_energy_to_cross_distance() {
        let nearer_incumbent = GrokkingCandidate {
            terminal_id: 1,
            geometry_distance: 1,
            settled_energy: 8_000,
            ..Default::default()
        };
        let weak_two_omission_reconstruction = GrokkingCandidate {
            terminal_id: 2,
            reconstruction_modes: RECONSTRUCTION_MODE_DELETION,
            sequence_milli: 1_000,
            geometry_distance: 2,
            settled_energy: 3_500,
            ..Default::default()
        };
        let mut candidates = [nearer_incumbent, weak_two_omission_reconstruction];

        apply_geometry_certificate_interference(&mut candidates);

        assert_eq!(candidates[0].terminal_id, nearer_incumbent.terminal_id);
    }

    #[test]
    fn two_omission_operator_does_not_displace_a_stronger_one_omission_inverse() {
        let one_omission_inverse = GrokkingCandidate {
            terminal_id: 1,
            reconstruction_modes: RECONSTRUCTION_MODE_SINGLE_DELETION,
            sequence_milli: 1_000,
            geometry_distance: 1,
            settled_energy: 8_200,
            ..Default::default()
        };
        let weaker_two_omission_inverse = GrokkingCandidate {
            terminal_id: 2,
            reconstruction_modes: RECONSTRUCTION_MODE_DELETION,
            sequence_milli: 1_000,
            geometry_distance: 2,
            settled_energy: 7_800,
            ..Default::default()
        };
        let mut candidates = [one_omission_inverse, weaker_two_omission_inverse];

        apply_geometry_certificate_interference(&mut candidates);

        assert_eq!(candidates[0].terminal_id, one_omission_inverse.terminal_id);
    }

    #[test]
    fn exact_boundary_truncation_outranks_a_two_omission_completion() {
        let two_omission_completion = GrokkingCandidate {
            terminal_id: 1,
            reconstruction_modes: RECONSTRUCTION_MODE_DELETION,
            sequence_milli: 1_000,
            geometry_distance: 2,
            settled_energy: 8_000,
            ..Default::default()
        };
        let suffix_completion = GrokkingCandidate {
            terminal_id: 2,
            reconstruction_modes: RECONSTRUCTION_MODE_SUFFIX_TRUNCATION,
            sequence_milli: 1_000,
            geometry_distance: 1,
            settled_energy: 7_500,
            ..Default::default()
        };
        let mut candidates = [two_omission_completion, suffix_completion];

        apply_geometry_certificate_interference(&mut candidates);

        assert_eq!(candidates[0].terminal_id, suffix_completion.terminal_id);
    }

    #[test]
    fn direct_surface_operator_outranks_cross_script_keyboard_projection() {
        let keyboard_projection = GrokkingCandidate {
            terminal_id: 1,
            keyboard_hits: 20,
            surface_hits: 2,
            geometry_distance: 2,
            settled_energy: 1_000,
            ..Default::default()
        };
        let double_substitution = GrokkingCandidate {
            terminal_id: 2,
            reconstruction_modes: RECONSTRUCTION_MODE_DOUBLE_SUBSTITUTION,
            keyboard_hits: 10,
            surface_hits: 20,
            geometry_distance: 2,
            settled_energy: 900,
            ..Default::default()
        };
        let mut candidates = [keyboard_projection, double_substitution];

        apply_geometry_certificate_interference(&mut candidates);

        assert_eq!(candidates[0].terminal_id, double_substitution.terminal_id);
    }

    #[test]
    fn deletion_transposition_operator_outranks_cross_script_keyboard_projection() {
        let keyboard_projection = GrokkingCandidate {
            terminal_id: 1,
            keyboard_hits: 20,
            surface_hits: 2,
            geometry_distance: 1,
            settled_energy: 1_000,
            ..Default::default()
        };
        let deletion_transposition = GrokkingCandidate {
            terminal_id: 2,
            reconstruction_modes: RECONSTRUCTION_MODE_DELETION_TRANSPOSITION,
            keyboard_hits: 10,
            surface_hits: 20,
            geometry_distance: 2,
            settled_energy: 900,
            ..Default::default()
        };
        let mut candidates = [keyboard_projection, deletion_transposition];

        apply_geometry_certificate_interference(&mut candidates);

        assert_eq!(
            candidates[0].terminal_id,
            deletion_transposition.terminal_id
        );
    }

    #[test]
    fn reconstruction_evidence_survives_bounded_lattice_without_reordering_primary() {
        let mut candidates = (0..6)
            .map(|terminal_id| GrokkingCandidate {
                terminal_id,
                reconstruction_modes: if terminal_id >= 4 {
                    RECONSTRUCTION_MODE_DELETION
                } else {
                    0
                },
                ..Default::default()
            })
            .collect::<Vec<_>>();

        truncate_with_reconstruction_tail(&mut candidates, 4);

        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.terminal_id)
                .collect::<Vec<_>>(),
            vec![0, 1, 4, 5]
        );
    }

    #[test]
    fn geometry_shell_evidence_survives_bounded_lattice() {
        let mut candidates = (0..6)
            .map(|terminal_id| GrokkingCandidate {
                terminal_id,
                ambiguity_shell: terminal_id == 5,
                geometry_distance: if terminal_id == 5 { 0 } else { 1 },
                settled_energy: 1_000 - terminal_id as i32,
                ..Default::default()
            })
            .collect::<Vec<_>>();

        truncate_with_reconstruction_tail(&mut candidates, 4);

        assert_eq!(candidates.len(), 4);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.terminal_id == 5));
    }

    #[test]
    fn adjacent_swap_plus_omission_matcher_is_exact_without_heap_storage() {
        assert!(is_subsequence_after_one_adjacent_swap(
            &[1, 3, 2, 4],
            &[1, 2, 3, 4, 5]
        ));
        assert!(is_subsequence_after_one_adjacent_swap(
            &[2, 1, 4],
            &[1, 2, 3, 4]
        ));
        assert!(is_subsequence_after_one_adjacent_swap(
            &[1, 4, 2],
            &[1, 2, 3, 4]
        ));
        assert!(!is_subsequence_after_one_adjacent_swap(
            &[1, 2, 3],
            &[1, 2, 3, 4]
        ));
    }

    #[test]
    fn adjacent_swap_plus_omission_matcher_preserves_reference_semantics() {
        fn reference(observed: &[u32], expected: &[u32]) -> bool {
            if observed.len() < 2 {
                return false;
            }
            let mut repaired = observed.to_vec();
            for index in 0..observed.len() - 1 {
                repaired.swap(index, index + 1);
                if is_ordered_subsequence(&repaired, expected) {
                    return true;
                }
                repaired.swap(index, index + 1);
            }
            false
        }

        fn surfaces(length: usize) -> Vec<Vec<u32>> {
            let count = 3_usize.pow(length as u32);
            (0..count)
                .map(|mut encoded| {
                    let mut surface = vec![0; length];
                    for symbol in &mut surface {
                        *symbol = (encoded % 3) as u32;
                        encoded /= 3;
                    }
                    surface
                })
                .collect()
        }

        for observed_length in 2..=4 {
            for observed in surfaces(observed_length) {
                for expected in surfaces(observed_length + 1) {
                    assert_eq!(
                        is_subsequence_after_one_adjacent_swap(&observed, &expected),
                        reference(&observed, &expected),
                        "observed={observed:?} expected={expected:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn adjacent_transposition_is_an_exact_surface_operator() {
        assert_eq!(
            surface_operator_reconstruction_modes("ba", "ab"),
            RECONSTRUCTION_MODE_NON_ADJACENT_TRANSPOSITION
        );
    }

    #[test]
    fn bounded_tail_keeps_the_strongest_operator_evidence() {
        let mut candidates = (0..100)
            .map(|terminal_id| GrokkingCandidate {
                terminal_id,
                reconstruction_modes: if terminal_id >= 64 {
                    RECONSTRUCTION_MODE_SINGLE_DELETION
                } else {
                    0
                },
                geometry_distance: 1,
                settled_energy: 2_000 - terminal_id as i32,
                ..Default::default()
            })
            .collect::<Vec<_>>();
        candidates[99].reconstruction_modes = RECONSTRUCTION_MODE_DOUBLE_SUBSTITUTION;

        truncate_with_reconstruction_tail(&mut candidates, 64);

        assert_eq!(candidates.len(), 64);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.terminal_id == 99));
    }

    #[test]
    fn exact_surface_keeps_clean_fast_path_but_expands_lattice_readout() {
        assert!(!should_expand_operator_lattice(1, 1));
        assert!(should_expand_operator_lattice(1, 64));
        assert!(should_expand_operator_lattice(0, 1));
    }

    #[test]
    fn bounded_tail_does_not_evict_operator_evidence_already_inside_limit() {
        let mut candidates = (0..100)
            .map(|terminal_id| GrokkingCandidate {
                terminal_id,
                reconstruction_modes: if terminal_id == 42 || terminal_id >= 64 {
                    RECONSTRUCTION_MODE_SINGLE_DELETION
                } else {
                    0
                },
                geometry_distance: 1,
                settled_energy: 2_000 - terminal_id as i32,
                ..Default::default()
            })
            .collect::<Vec<_>>();

        truncate_with_reconstruction_tail(&mut candidates, 64);

        assert_eq!(candidates.len(), 64);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.terminal_id == 42));
        assert!(candidates
            .iter()
            .any(|candidate| candidate.terminal_id >= 64));
    }
}
