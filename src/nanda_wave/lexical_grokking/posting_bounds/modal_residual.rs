//! Proof-only modal residual projection over the complete forward field.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use rayon::prelude::*;
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::{
    activation_equal, exact_contribution, is_keyboard_channel, validate_forward_posting,
    ExactPostingResult, PostingClosure, QueryPosting, SearchMetrics,
};
use crate::nanda_wave::lexical_grokking::model::WaveCoupling;
use crate::nanda_wave::lexical_grokking::runtime::{
    ForwardActivation, LexicalGrokkingMemory, ObservedAtom,
};
use crate::nanda_wave::lexical_grokking::v8;

const ATOMS_PER_SHARD: usize = 32;
const OUTER_HEADER_BYTES: usize = 128;
const ATOM_INDEX_BYTES: usize = 16;
const SHARD_INDEX_BYTES: usize = 16;
const ZSTD_LEVEL: i32 = 19;
pub(super) const PROJECTION_EVENT_LIMIT: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ModalState {
    Absent,
    Relation { strength: u8, position: u8 },
}

impl ModalState {
    fn from_relation(relation: WaveCoupling) -> Self {
        Self::Relation {
            strength: relation.strength,
            position: relation.position_mode,
        }
    }

    fn state_id(self) -> u32 {
        match self {
            Self::Absent => 0,
            Self::Relation { strength, position } => {
                1 + (u32::from(strength) << 8) + u32::from(position)
            }
        }
    }

    fn preferred_to(self, other: Self) -> bool {
        match (self, other) {
            (Self::Absent, Self::Absent) => false,
            (Self::Absent, _) => true,
            (_, Self::Absent) => false,
            (
                Self::Relation {
                    strength: left_strength,
                    position: left_position,
                },
                Self::Relation {
                    strength: right_strength,
                    position: right_position,
                },
            ) => {
                left_strength > right_strength
                    || (left_strength == right_strength && left_position < right_position)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct SignedActivation {
    mass: i128,
    hits: i32,
    surface_hits: i32,
    keyboard_hits: i32,
}

impl SignedActivation {
    fn from_forward(value: ForwardActivation) -> Self {
        Self {
            mass: i128::from(value.mass),
            hits: i32::from(value.hits),
            surface_hits: i32::from(value.surface_hits),
            keyboard_hits: i32::from(value.keyboard_hits),
        }
    }

    fn checked_add(self, other: Self) -> Result<Self, String> {
        Ok(Self {
            mass: self
                .mass
                .checked_add(other.mass)
                .ok_or_else(|| "modal residual mass overflow".to_string())?,
            hits: self
                .hits
                .checked_add(other.hits)
                .ok_or_else(|| "modal residual hit overflow".to_string())?,
            surface_hits: self
                .surface_hits
                .checked_add(other.surface_hits)
                .ok_or_else(|| "modal residual surface-hit overflow".to_string())?,
            keyboard_hits: self
                .keyboard_hits
                .checked_add(other.keyboard_hits)
                .ok_or_else(|| "modal residual keyboard-hit overflow".to_string())?,
        })
    }

    fn checked_sub(self, other: Self) -> Result<Self, String> {
        Ok(Self {
            mass: self
                .mass
                .checked_sub(other.mass)
                .ok_or_else(|| "modal residual mass underflow".to_string())?,
            hits: self
                .hits
                .checked_sub(other.hits)
                .ok_or_else(|| "modal residual hit underflow".to_string())?,
            surface_hits: self
                .surface_hits
                .checked_sub(other.surface_hits)
                .ok_or_else(|| "modal residual surface-hit underflow".to_string())?,
            keyboard_hits: self
                .keyboard_hits
                .checked_sub(other.keyboard_hits)
                .ok_or_else(|| "modal residual keyboard-hit underflow".to_string())?,
        })
    }

    fn into_forward(self) -> Result<ForwardActivation, String> {
        Ok(ForwardActivation {
            mass: u64::try_from(self.mass)
                .map_err(|_| "modal reconstruction mass is outside u64".to_string())?,
            hits: u16::try_from(self.hits)
                .map_err(|_| "modal reconstruction hits are outside u16".to_string())?,
            surface_hits: u16::try_from(self.surface_hits)
                .map_err(|_| "modal reconstruction surface hits are outside u16".to_string())?,
            keyboard_hits: u16::try_from(self.keyboard_hits)
                .map_err(|_| "modal reconstruction keyboard hits are outside u16".to_string())?,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ModalPackageProjectionMetrics {
    pub(super) current_file_bytes: u64,
    pub(super) compact_base_bytes: u64,
    pub(super) atoms: usize,
    pub(super) terminals: u32,
    pub(super) original_relation_events: usize,
    pub(super) residual_events: usize,
    pub(super) modal_absent_atoms: usize,
    pub(super) modal_relation_atoms: usize,
    pub(super) largest_modal_state: usize,
    pub(super) largest_residual_state: usize,
    pub(super) largest_residual_lane: usize,
    pub(super) raw_residual_bytes: usize,
    pub(super) compressed_residual_bytes: usize,
    pub(super) projected_atom_index_bytes: usize,
    pub(super) projected_shard_index_bytes: usize,
    pub(super) projected_alignment_bytes: usize,
    pub(super) projected_package_bytes: u64,
    pub(super) atoms_per_shard: usize,
    pub(super) shard_count: usize,
    pub(super) zstd_level: i32,
    pub(super) residual_payload_sha256: String,
    pub(super) state_partition_omissions: usize,
    pub(super) state_partition_duplicates: usize,
    pub(super) residual_bound_violations: usize,
    pub(super) build_ms: u128,
}

#[derive(Clone, Debug, Default, Serialize)]
pub(super) struct ModalQueryMetrics {
    pub(super) query_postings: usize,
    pub(super) original_relation_events: usize,
    pub(super) residual_events: usize,
    pub(super) positive_residual_events: usize,
    pub(super) negative_residual_events: usize,
    pub(super) zero_mass_residual_events: usize,
    pub(super) positive_cells: usize,
    pub(super) largest_residual_state: usize,
    pub(super) largest_positive_state_cell: usize,
    pub(super) modal_baseline_mass: u64,
    pub(super) dense_beta: u64,
    pub(super) default_cohort_resolved: bool,
    pub(super) oracle_threshold_certified: bool,
    pub(super) oracle_greedy_cells: usize,
    pub(super) oracle_greedy_events: usize,
    pub(super) oracle_greedy_unique_centers: usize,
    pub(super) oracle_greedy_equality_layers: usize,
    pub(super) largest_consumed_equality_layer: usize,
    pub(super) oracle_final_untouched_upper: u64,
    pub(super) untouched_upper_bound_violations: usize,
    pub(super) fractional_event_lower_bound: usize,
    pub(super) exact_signed_lookup_probes: usize,
    pub(super) exact_signed_lookup_hits: usize,
    pub(super) reconstruction_field_mismatches: usize,
    pub(super) kth_or_equality_mismatches: usize,
    pub(super) projection_us: u64,
}

pub(super) struct ModalQueryProjection {
    pub(super) exact: ExactPostingResult,
    pub(super) metrics: ModalQueryMetrics,
}

pub(super) struct ModalResidualProjection {
    terminal_count: u32,
    defaults: Vec<ModalState>,
    pub(super) package: ModalPackageProjectionMetrics,
}

#[derive(Clone, Copy)]
struct PackageLayout {
    current_file_bytes: u64,
    compact_base_bytes: u64,
}

#[derive(Clone, Copy)]
struct AtomSummary {
    modal: ModalState,
    modal_count: usize,
    residual_count: usize,
    largest_residual_state: usize,
    raw_offset: u32,
    raw_len: u32,
}

struct BuiltShard {
    atoms: Vec<AtomSummary>,
    raw_len: u32,
    compressed: Vec<u8>,
    original_relations: usize,
}

#[derive(Debug)]
struct PositiveCell {
    delta: u64,
    terminal_ids: Vec<u32>,
}

#[derive(Debug)]
struct ModalQueryLane {
    positive_cells: Vec<PositiveCell>,
    residual_terminal_ids: Vec<u32>,
}

impl ModalResidualProjection {
    pub(super) fn build(
        memory: &LexicalGrokkingMemory,
        package_path: &Path,
    ) -> Result<Self, String> {
        let bytes = std::fs::read(package_path)
            .map_err(|error| format!("read modal projection package: {error}"))?;
        let header = v8::read_header(&bytes)?;
        let atom_count = memory.package.atoms.len();
        let expected_shards = atom_count.div_ceil(ATOMS_PER_SHARD);
        if header.base_offset as usize != OUTER_HEADER_BYTES
            || header.index_count as usize != atom_count
            || header.shard_count as usize != expected_shards
            || header.forward_relations as usize != memory.forward_relation_count()
        {
            return Err("V8 header differs from fixed modal projection topology".to_string());
        }
        Self::build_with_layout(
            memory,
            PackageLayout {
                current_file_bytes: header.file_bytes,
                compact_base_bytes: header.base_bytes,
            },
        )
    }

    fn build_with_layout(
        memory: &LexicalGrokkingMemory,
        layout: PackageLayout,
    ) -> Result<Self, String> {
        let started = Instant::now();
        let atom_count = memory.package.atoms.len();
        let terminal_count = memory.package.terminal_count();
        let shard_count = atom_count.div_ceil(ATOMS_PER_SHARD);
        let shards = (0..shard_count)
            .into_par_iter()
            .map(|shard_id| {
                build_shard(
                    memory,
                    terminal_count,
                    shard_id.saturating_mul(ATOMS_PER_SHARD),
                    atom_count.min((shard_id + 1).saturating_mul(ATOMS_PER_SHARD)),
                )
            })
            .collect::<Result<Vec<_>, String>>()?;

        let mut defaults = Vec::with_capacity(atom_count);
        let mut original_relations = 0_usize;
        let mut residual_events = 0_usize;
        let mut modal_absent_atoms = 0_usize;
        let mut modal_relation_atoms = 0_usize;
        let mut largest_modal_state = 0_usize;
        let mut largest_residual_state = 0_usize;
        let mut largest_residual_lane = 0_usize;
        let mut raw_residual_bytes = 0_usize;
        let mut compressed_residual_bytes = 0_usize;
        let mut hasher = Sha256::new();
        hasher.update(b"lay.l11.modal-residual-projection.v1");

        for (shard_id, shard) in shards.iter().enumerate() {
            original_relations = original_relations.saturating_add(shard.original_relations);
            raw_residual_bytes = raw_residual_bytes.saturating_add(shard.raw_len as usize);
            compressed_residual_bytes =
                compressed_residual_bytes.saturating_add(shard.compressed.len());
            hasher.update((shard_id as u32).to_le_bytes());
            hasher.update(shard.raw_len.to_le_bytes());
            hasher.update((shard.compressed.len() as u32).to_le_bytes());
            for atom in &shard.atoms {
                defaults.push(atom.modal);
                residual_events = residual_events.saturating_add(atom.residual_count);
                modal_absent_atoms += usize::from(atom.modal == ModalState::Absent);
                modal_relation_atoms += usize::from(atom.modal != ModalState::Absent);
                largest_modal_state = largest_modal_state.max(atom.modal_count);
                largest_residual_state = largest_residual_state.max(atom.largest_residual_state);
                largest_residual_lane = largest_residual_lane.max(atom.residual_count);
                hasher.update(atom.modal.state_id().to_le_bytes());
                hasher.update((atom.residual_count as u32).to_le_bytes());
                hasher.update(atom.raw_offset.to_le_bytes());
                hasher.update(atom.raw_len.to_le_bytes());
            }
            hasher.update(&shard.compressed);
        }

        if defaults.len() != atom_count
            || original_relations != memory.forward_relation_count()
            || residual_events > original_relations
        {
            return Err("modal full-field accounting differs from complete postings".to_string());
        }

        let projected_atom_index_bytes = atom_count.saturating_mul(ATOM_INDEX_BYTES);
        let projected_shard_index_bytes = shard_count.saturating_mul(SHARD_INDEX_BYTES);
        let index_offset = align8(
            OUTER_HEADER_BYTES.saturating_add(
                usize::try_from(layout.compact_base_bytes)
                    .map_err(|_| "compact base exceeds usize".to_string())?,
            ),
        );
        let shard_index_offset = align8(index_offset.saturating_add(projected_atom_index_bytes));
        let postings_offset =
            align8(shard_index_offset.saturating_add(projected_shard_index_bytes));
        let projected_package_bytes = postings_offset.saturating_add(compressed_residual_bytes);
        let projected_alignment_bytes = index_offset
            .saturating_sub(OUTER_HEADER_BYTES + layout.compact_base_bytes as usize)
            .saturating_add(
                shard_index_offset.saturating_sub(index_offset + projected_atom_index_bytes),
            )
            .saturating_add(
                postings_offset.saturating_sub(shard_index_offset + projected_shard_index_bytes),
            );

        Ok(Self {
            terminal_count,
            defaults,
            package: ModalPackageProjectionMetrics {
                current_file_bytes: layout.current_file_bytes,
                compact_base_bytes: layout.compact_base_bytes,
                atoms: atom_count,
                terminals: terminal_count,
                original_relation_events: original_relations,
                residual_events,
                modal_absent_atoms,
                modal_relation_atoms,
                largest_modal_state,
                largest_residual_state,
                largest_residual_lane,
                raw_residual_bytes,
                compressed_residual_bytes,
                projected_atom_index_bytes,
                projected_shard_index_bytes,
                projected_alignment_bytes,
                projected_package_bytes: projected_package_bytes as u64,
                atoms_per_shard: ATOMS_PER_SHARD,
                shard_count,
                zstd_level: ZSTD_LEVEL,
                residual_payload_sha256: format!("{:x}", hasher.finalize()),
                state_partition_omissions: 0,
                state_partition_duplicates: 0,
                residual_bound_violations: 0,
                build_ms: started.elapsed().as_millis(),
            },
        })
    }

    pub(super) fn project_query(
        &self,
        postings: &[QueryPosting<'_>],
        requested_k: usize,
        dense_beta: u64,
        dense_all: &[ForwardActivation],
    ) -> Result<ModalQueryProjection, String> {
        let started = Instant::now();
        if dense_all.len() != self.terminal_count as usize {
            return Err("modal projection dense field has the wrong terminal count".to_string());
        }

        let mut baseline = SignedActivation::default();
        let mut residual_field = vec![SignedActivation::default(); self.terminal_count as usize];
        let mut lanes = Vec::with_capacity(postings.len());
        let mut metrics = ModalQueryMetrics {
            query_postings: postings.len(),
            dense_beta,
            ..ModalQueryMetrics::default()
        };

        for posting in postings {
            let default = *self
                .defaults
                .get(posting.atom_id as usize)
                .ok_or_else(|| format!("missing modal state for atom {}", posting.atom_id))?;
            let default_activation = activation_for_state(posting.atom, default);
            baseline = baseline.checked_add(SignedActivation::from_forward(default_activation))?;
            let mut residual_terminal_ids = Vec::new();
            let mut positive = BTreeMap::<u64, Vec<u32>>::new();
            let mut residual_states = BTreeMap::<ModalState, usize>::new();
            metrics.original_relation_events = metrics
                .original_relation_events
                .saturating_add(posting.relations.len());

            visit_residual_states(
                &posting.relations,
                self.terminal_count,
                default,
                |terminal_id, state| {
                    let residual =
                        SignedActivation::from_forward(activation_for_state(posting.atom, state))
                            .checked_sub(SignedActivation::from_forward(default_activation))?;
                    residual_field[terminal_id as usize] =
                        residual_field[terminal_id as usize].checked_add(residual)?;
                    residual_terminal_ids.push(terminal_id);
                    *residual_states.entry(state).or_default() += 1;
                    metrics.residual_events += 1;
                    match residual.mass.cmp(&0) {
                        Ordering::Greater => {
                            metrics.positive_residual_events += 1;
                            positive
                                .entry(u64::try_from(residual.mass).map_err(|_| {
                                    "positive modal residual exceeds u64".to_string()
                                })?)
                                .or_default()
                                .push(terminal_id);
                        }
                        Ordering::Less => metrics.negative_residual_events += 1,
                        Ordering::Equal => metrics.zero_mass_residual_events += 1,
                    }
                    Ok(())
                },
            )?;

            metrics.largest_residual_state = metrics
                .largest_residual_state
                .max(residual_states.values().copied().max().unwrap_or_default());
            let positive_cells = positive
                .into_iter()
                .rev()
                .map(|(delta, terminal_ids)| PositiveCell {
                    delta,
                    terminal_ids,
                })
                .collect::<Vec<_>>();
            metrics.positive_cells = metrics.positive_cells.saturating_add(positive_cells.len());
            metrics.largest_positive_state_cell = metrics.largest_positive_state_cell.max(
                positive_cells
                    .iter()
                    .map(|cell| cell.terminal_ids.len())
                    .max()
                    .unwrap_or_default(),
            );
            lanes.push(ModalQueryLane {
                positive_cells,
                residual_terminal_ids,
            });
        }

        let baseline_forward = baseline.into_forward()?;
        metrics.modal_baseline_mass = baseline_forward.mass;
        metrics.default_cohort_resolved = dense_beta > baseline_forward.mass;
        let mut all = Vec::with_capacity(self.terminal_count as usize);
        for residual in residual_field {
            all.push(baseline.checked_add(residual)?.into_forward()?);
        }
        metrics.reconstruction_field_mismatches = all
            .iter()
            .copied()
            .zip(dense_all.iter().copied())
            .filter(|(left, right)| !activation_equal(*left, *right))
            .count();

        let exact = exact_result_from_activations(all, postings, requested_k);
        metrics.kth_or_equality_mismatches = usize::from(exact.closure.beta_k != dense_beta);
        let oracle = oracle_screen(&lanes, baseline_forward.mass, dense_beta, dense_all);
        metrics.oracle_threshold_certified = oracle.certified;
        metrics.oracle_greedy_cells = oracle.cells;
        metrics.oracle_greedy_events = oracle.events;
        metrics.oracle_greedy_unique_centers = oracle.unique_centers;
        metrics.oracle_greedy_equality_layers = oracle.equality_layers;
        metrics.largest_consumed_equality_layer = oracle.largest_equality_layer;
        metrics.oracle_final_untouched_upper = oracle.final_upper;
        metrics.untouched_upper_bound_violations = oracle.bound_violations;
        metrics.fractional_event_lower_bound =
            fractional_event_lower_bound(&lanes, baseline_forward.mass, dense_beta);
        metrics.exact_signed_lookup_probes = oracle.unique_centers.saturating_mul(postings.len());
        metrics.exact_signed_lookup_hits = lanes
            .iter()
            .map(|lane| {
                lane.residual_terminal_ids
                    .iter()
                    .filter(|terminal_id| oracle.touched[**terminal_id as usize])
                    .count()
            })
            .sum();
        metrics.projection_us = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;

        Ok(ModalQueryProjection { exact, metrics })
    }

    #[cfg(test)]
    fn build_for_test(memory: &LexicalGrokkingMemory) -> Result<Self, String> {
        Self::build_with_layout(
            memory,
            PackageLayout {
                current_file_bytes: 0,
                compact_base_bytes: 0,
            },
        )
    }
}

fn build_shard(
    memory: &LexicalGrokkingMemory,
    terminal_count: u32,
    atom_start: usize,
    atom_end: usize,
) -> Result<BuiltShard, String> {
    let atom_ids = (atom_start..atom_end)
        .map(|atom_id| u32::try_from(atom_id).map_err(|_| "atom id exceeds u32".to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let postings = memory.complete_forward_couplings_batch(&atom_ids)?;
    if postings.len() != atom_ids.len() {
        return Err("modal shard batch posting count differs".to_string());
    }
    let mut raw = Vec::new();
    let mut atoms = Vec::with_capacity(atom_ids.len());
    let mut original_relations = 0_usize;
    for relations in postings {
        validate_forward_posting(&relations, terminal_count)?;
        original_relations = original_relations.saturating_add(relations.len());
        let raw_offset = u32::try_from(raw.len())
            .map_err(|_| "modal shard raw offset exceeds u32".to_string())?;
        let (modal, modal_count, residual_count, largest_residual_state) =
            encode_atom_lane(&mut raw, &relations, terminal_count)?;
        let raw_len = u32::try_from(raw.len().saturating_sub(raw_offset as usize))
            .map_err(|_| "modal atom lane exceeds u32".to_string())?;
        atoms.push(AtomSummary {
            modal,
            modal_count,
            residual_count,
            largest_residual_state,
            raw_offset,
            raw_len,
        });
    }
    let raw_len =
        u32::try_from(raw.len()).map_err(|_| "modal residual shard exceeds u32".to_string())?;
    let compressed = zstd::bulk::compress(&raw, ZSTD_LEVEL)
        .map_err(|error| format!("modal residual shard compression failed: {error}"))?;
    Ok(BuiltShard {
        atoms,
        raw_len,
        compressed,
        original_relations,
    })
}

fn encode_atom_lane(
    raw: &mut Vec<u8>,
    relations: &[WaveCoupling],
    terminal_count: u32,
) -> Result<(ModalState, usize, usize, usize), String> {
    let counts = state_counts(relations, terminal_count)?;
    let (modal, modal_count) = select_modal_state(&counts)?;
    put_varint(raw, modal.state_id());
    let mut previous = None;
    let mut residual_count = 0_usize;
    visit_residual_states(relations, terminal_count, modal, |terminal_id, state| {
        let delta = previous.map_or(terminal_id, |prior| terminal_id - prior);
        put_varint(raw, delta);
        put_varint(raw, state.state_id());
        previous = Some(terminal_id);
        residual_count += 1;
        Ok(())
    })?;
    let largest_residual_state = counts
        .iter()
        .filter_map(|(state, count)| (*state != modal).then_some(*count))
        .max()
        .unwrap_or_default();
    if residual_count != terminal_count as usize - modal_count || residual_count > relations.len() {
        return Err("modal residual representation bound failed".to_string());
    }
    Ok((modal, modal_count, residual_count, largest_residual_state))
}

fn state_counts(
    relations: &[WaveCoupling],
    terminal_count: u32,
) -> Result<BTreeMap<ModalState, usize>, String> {
    validate_forward_posting(relations, terminal_count)?;
    let mut counts = BTreeMap::new();
    let absent = terminal_count as usize - relations.len();
    if absent != 0 {
        counts.insert(ModalState::Absent, absent);
    }
    for relation in relations.iter().copied() {
        *counts
            .entry(ModalState::from_relation(relation))
            .or_default() += 1;
    }
    if counts.values().sum::<usize>() != terminal_count as usize {
        return Err("modal state partition does not cover terminal domain".to_string());
    }
    Ok(counts)
}

fn select_modal_state(counts: &BTreeMap<ModalState, usize>) -> Result<(ModalState, usize), String> {
    counts
        .iter()
        .map(|(state, count)| (*state, *count))
        .reduce(|best, candidate| {
            if candidate.1 > best.1 || (candidate.1 == best.1 && candidate.0.preferred_to(best.0)) {
                candidate
            } else {
                best
            }
        })
        .ok_or_else(|| "modal state partition is empty".to_string())
}

fn visit_residual_states(
    relations: &[WaveCoupling],
    terminal_count: u32,
    modal: ModalState,
    mut visitor: impl FnMut(u32, ModalState) -> Result<(), String>,
) -> Result<(), String> {
    match modal {
        ModalState::Absent => {
            for relation in relations.iter().copied() {
                visitor(relation.peer_id, ModalState::from_relation(relation))?;
            }
        }
        ModalState::Relation { .. } => {
            let mut relation_index = 0_usize;
            for terminal_id in 0..terminal_count {
                let state = if relations
                    .get(relation_index)
                    .is_some_and(|relation| relation.peer_id == terminal_id)
                {
                    let state = ModalState::from_relation(relations[relation_index]);
                    relation_index += 1;
                    state
                } else {
                    ModalState::Absent
                };
                if state != modal {
                    visitor(terminal_id, state)?;
                }
            }
            if relation_index != relations.len() {
                return Err("modal relation merge did not consume complete posting".to_string());
            }
        }
    }
    Ok(())
}

fn activation_for_state(atom: ObservedAtom, state: ModalState) -> ForwardActivation {
    let ModalState::Relation { strength, position } = state else {
        return ForwardActivation::default();
    };
    let mut activation = ForwardActivation {
        mass: exact_contribution(
            atom,
            WaveCoupling {
                strength,
                position_mode: position,
                ..WaveCoupling::default()
            },
        ),
        hits: 1,
        ..ForwardActivation::default()
    };
    if is_keyboard_channel(atom.channel) {
        activation.keyboard_hits = 1;
    } else {
        activation.surface_hits = 1;
    }
    activation
}

fn exact_result_from_activations(
    all: Vec<ForwardActivation>,
    postings: &[QueryPosting<'_>],
    requested_k: usize,
) -> ExactPostingResult {
    if all.is_empty() {
        return ExactPostingResult {
            closure: PostingClosure {
                beta_k: 0,
                retained: Vec::new(),
                metrics: SearchMetrics::default(),
            },
            touched: Vec::new(),
            all,
        };
    }
    let effective_k = requested_k.max(1).min(all.len());
    let mut masses = all
        .iter()
        .map(|activation| activation.mass)
        .collect::<Vec<_>>();
    masses.select_nth_unstable_by(effective_k - 1, |left, right| right.cmp(left));
    let beta_k = masses[effective_k - 1];
    let retained = all
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(terminal_id, activation)| {
            (activation.mass >= beta_k).then_some((terminal_id as u32, activation))
        })
        .collect();
    let touched = all
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(terminal_id, activation)| {
            (activation.hits != 0).then_some((terminal_id as u32, activation))
        })
        .collect();
    ExactPostingResult {
        closure: PostingClosure {
            beta_k,
            retained,
            metrics: SearchMetrics {
                posting_relations_total: postings
                    .iter()
                    .map(|posting| posting.relations.len())
                    .sum(),
                posting_relations_decoded: 0,
                posting_iterators: postings.len(),
                centers_evaluated: all.len(),
                ..SearchMetrics::default()
            },
        },
        touched,
        all,
    }
}

struct OracleScreen {
    certified: bool,
    cells: usize,
    events: usize,
    unique_centers: usize,
    equality_layers: usize,
    largest_equality_layer: usize,
    final_upper: u64,
    bound_violations: usize,
    touched: Vec<bool>,
}

fn oracle_screen(
    lanes: &[ModalQueryLane],
    baseline: u64,
    beta: u64,
    dense_all: &[ForwardActivation],
) -> OracleScreen {
    let mut cursors = vec![0_usize; lanes.len()];
    let mut head_sum = lanes
        .iter()
        .map(|lane| {
            lane.positive_cells
                .first()
                .map_or(0_u128, |cell| u128::from(cell.delta))
        })
        .sum::<u128>();
    let mut touched = vec![false; dense_all.len()];
    let mut cells = 0_usize;
    let mut events = 0_usize;
    let mut equality_layers = 0_usize;
    let mut largest_equality_layer = 0_usize;

    loop {
        let upper = u128::from(baseline).saturating_add(head_sum);
        if upper < u128::from(beta) {
            break;
        }
        let maximum = lanes
            .iter()
            .zip(&cursors)
            .filter_map(|(lane, cursor)| lane.positive_cells.get(*cursor).map(|cell| cell.delta))
            .max()
            .unwrap_or_default();
        if maximum == 0 {
            break;
        }
        equality_layers += 1;
        let mut layer_events = 0_usize;
        for (lane, cursor) in lanes.iter().zip(&mut cursors) {
            let Some(cell) = lane.positive_cells.get(*cursor) else {
                continue;
            };
            if cell.delta != maximum {
                continue;
            }
            cells += 1;
            layer_events = layer_events.saturating_add(cell.terminal_ids.len());
            for terminal_id in &cell.terminal_ids {
                touched[*terminal_id as usize] = true;
            }
            head_sum = head_sum.saturating_sub(u128::from(cell.delta));
            *cursor += 1;
            head_sum = head_sum.saturating_add(u128::from(
                lane.positive_cells
                    .get(*cursor)
                    .map_or(0, |next| next.delta),
            ));
        }
        events = events.saturating_add(layer_events);
        largest_equality_layer = largest_equality_layer.max(layer_events);
    }

    let final_upper_wide = u128::from(baseline).saturating_add(head_sum);
    let final_upper = final_upper_wide.min(u128::from(u64::MAX)) as u64;
    let bound_violations = dense_all
        .iter()
        .zip(&touched)
        .filter(|(activation, touched)| !**touched && activation.mass > final_upper)
        .count();
    OracleScreen {
        certified: final_upper_wide < u128::from(beta),
        cells,
        events,
        unique_centers: touched.iter().filter(|value| **value).count(),
        equality_layers,
        largest_equality_layer,
        final_upper,
        bound_violations,
        touched,
    }
}

#[derive(Clone, Copy)]
struct HeadDrop {
    benefit: u64,
    cost: usize,
}

fn fractional_event_lower_bound(lanes: &[ModalQueryLane], baseline: u64, beta: u64) -> usize {
    if beta <= baseline {
        return 0;
    }
    let initial_head_sum = lanes
        .iter()
        .map(|lane| {
            lane.positive_cells
                .first()
                .map_or(0_u128, |cell| u128::from(cell.delta))
        })
        .sum::<u128>();
    let allowed = u128::from(beta - baseline - 1);
    if initial_head_sum <= allowed {
        return 0;
    }
    let mut required = initial_head_sum - allowed;
    let mut drops = Vec::new();
    for lane in lanes {
        for (index, cell) in lane.positive_cells.iter().enumerate() {
            let next = lane
                .positive_cells
                .get(index + 1)
                .map_or(0, |next| next.delta);
            drops.push(HeadDrop {
                benefit: cell.delta - next,
                cost: cell.terminal_ids.len(),
            });
        }
    }
    drops.sort_unstable_by(|left, right| {
        let left_ratio = u128::from(left.benefit).saturating_mul(right.cost as u128);
        let right_ratio = u128::from(right.benefit).saturating_mul(left.cost as u128);
        right_ratio
            .cmp(&left_ratio)
            .then_with(|| right.benefit.cmp(&left.benefit))
            .then_with(|| left.cost.cmp(&right.cost))
    });
    let mut cost = 0_u128;
    for drop in drops {
        let benefit = u128::from(drop.benefit);
        if benefit >= required {
            let partial = (drop.cost as u128)
                .saturating_mul(required)
                .div_ceil(benefit);
            cost = cost.saturating_add(partial);
            required = 0;
            break;
        }
        required -= benefit;
        cost = cost.saturating_add(drop.cost as u128);
    }
    if required != 0 {
        usize::MAX
    } else {
        cost.min(usize::MAX as u128) as usize
    }
}

pub(super) fn summarize_queries(metrics: &[ModalQueryMetrics]) -> serde_json::Value {
    let sum = |field: fn(&ModalQueryMetrics) -> usize| {
        metrics
            .iter()
            .map(field)
            .fold(0_usize, usize::saturating_add)
    };
    let max = |field: fn(&ModalQueryMetrics) -> usize| {
        metrics.iter().map(field).max().unwrap_or_default()
    };
    serde_json::json!({
        "cases": metrics.len(),
        "query_postings": sum(|item| item.query_postings),
        "original_relation_events": sum(|item| item.original_relation_events),
        "residual_events": sum(|item| item.residual_events),
        "residual_events_max": max(|item| item.residual_events),
        "positive_residual_events": sum(|item| item.positive_residual_events),
        "negative_residual_events": sum(|item| item.negative_residual_events),
        "zero_mass_residual_events": sum(|item| item.zero_mass_residual_events),
        "positive_cells": sum(|item| item.positive_cells),
        "largest_residual_state": max(|item| item.largest_residual_state),
        "largest_positive_state_cell": max(|item| item.largest_positive_state_cell),
        "modal_baseline_mass_max": metrics.iter().map(|item| item.modal_baseline_mass).max().unwrap_or_default(),
        "dense_beta_min": metrics.iter().map(|item| item.dense_beta).min().unwrap_or_default(),
        "default_cohort_resolved": metrics.iter().filter(|item| item.default_cohort_resolved).count(),
        "oracle_threshold_certified": metrics.iter().filter(|item| item.oracle_threshold_certified).count(),
        "oracle_greedy_cells": sum(|item| item.oracle_greedy_cells),
        "oracle_greedy_events": sum(|item| item.oracle_greedy_events),
        "oracle_greedy_events_max": max(|item| item.oracle_greedy_events),
        "oracle_greedy_unique_centers_max": max(|item| item.oracle_greedy_unique_centers),
        "oracle_greedy_equality_layers": sum(|item| item.oracle_greedy_equality_layers),
        "largest_consumed_equality_layer": max(|item| item.largest_consumed_equality_layer),
        "oracle_final_untouched_upper_max": metrics.iter().map(|item| item.oracle_final_untouched_upper).max().unwrap_or_default(),
        "untouched_upper_bound_violations": sum(|item| item.untouched_upper_bound_violations),
        "fractional_event_lower_bound_max": max(|item| item.fractional_event_lower_bound),
        "exact_signed_lookup_probes": sum(|item| item.exact_signed_lookup_probes),
        "exact_signed_lookup_probes_max": max(|item| item.exact_signed_lookup_probes),
        "exact_signed_lookup_hits": sum(|item| item.exact_signed_lookup_hits),
        "reconstruction_field_mismatches": sum(|item| item.reconstruction_field_mismatches),
        "kth_or_equality_mismatches": sum(|item| item.kth_or_equality_mismatches),
        "projection_us_max": metrics.iter().map(|item| item.projection_us).max().unwrap_or_default(),
        "scan_event_gate": metrics.iter().all(|item| item.residual_events <= PROJECTION_EVENT_LIMIT),
    })
}

fn put_varint(bytes: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn align8(value: usize) -> usize {
    value.saturating_add(7) & !7
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::lexical_grokking::atoms::AtomChannel;
    use crate::nanda_wave::lexical_grokking::compiler::compile;
    use crate::nanda_wave::lexical_grokking::training_corpus::TrainingWord;

    fn memory() -> LexicalGrokkingMemory {
        let words = ["form", "farm", "foam", "from", "frame", "формат", "ферма"]
            .into_iter()
            .enumerate()
            .map(|(terminal_id, surface)| TrainingWord {
                terminal_id: terminal_id as u32,
                surface: surface.to_string(),
                training_surfaces: Vec::new(),
            })
            .collect::<Vec<_>>();
        LexicalGrokkingMemory::from_package(compile(&words).expect("compile modal test package"))
    }

    #[test]
    fn modal_ties_follow_query_independent_structural_order() {
        let counts = BTreeMap::from([
            (ModalState::Absent, 4),
            (
                ModalState::Relation {
                    strength: 200,
                    position: 90,
                },
                4,
            ),
        ]);
        assert_eq!(
            select_modal_state(&counts).unwrap(),
            (ModalState::Absent, 4)
        );
        let counts = BTreeMap::from([
            (
                ModalState::Relation {
                    strength: 100,
                    position: 90,
                },
                4,
            ),
            (
                ModalState::Relation {
                    strength: 200,
                    position: 120,
                },
                4,
            ),
            (
                ModalState::Relation {
                    strength: 200,
                    position: 80,
                },
                4,
            ),
        ]);
        assert_eq!(
            select_modal_state(&counts).unwrap().0,
            ModalState::Relation {
                strength: 200,
                position: 80
            }
        );
    }

    #[test]
    fn modal_residual_count_never_exceeds_complete_posting() {
        let relations = (0..7_u32)
            .map(|peer_id| WaveCoupling {
                peer_id,
                strength: if peer_id < 5 { 200 } else { 100 },
                position_mode: if peer_id < 5 { 64 } else { 128 },
                ..WaveCoupling::default()
            })
            .collect::<Vec<_>>();
        let mut bytes = Vec::new();
        let (modal, _, residuals, _) = encode_atom_lane(&mut bytes, &relations, 9).unwrap();
        assert_eq!(
            modal,
            ModalState::Relation {
                strength: 200,
                position: 64
            }
        );
        assert_eq!(residuals, 4);
        assert!(residuals <= relations.len());
    }

    #[test]
    fn modal_projection_reconstructs_complete_field_across_k() {
        let memory = memory();
        let projection = ModalResidualProjection::build_for_test(&memory).unwrap();
        for surface in ["form", "frmo", "форма"] {
            let postings = super::super::query_postings(&memory, None, surface).unwrap();
            for k in [1, 3, 7] {
                let dense = super::super::exact_posting_closure(
                    &postings,
                    memory.package.terminal_count(),
                    k,
                );
                let projected = projection
                    .project_query(&postings, k, dense.closure.beta_k, &dense.all)
                    .unwrap();
                assert_eq!(projected.metrics.reconstruction_field_mismatches, 0);
                assert_eq!(projected.exact.closure.beta_k, dense.closure.beta_k);
                assert!(super::super::retained_equal(
                    &projected.exact.closure.retained,
                    &dense.closure.retained
                ));
                assert_eq!(projected.metrics.untouched_upper_bound_violations, 0);
            }
        }
    }

    #[test]
    fn full_field_projection_is_deterministic() {
        let memory = memory();
        let first = ModalResidualProjection::build_for_test(&memory).unwrap();
        let second = ModalResidualProjection::build_for_test(&memory).unwrap();
        assert_eq!(first.defaults, second.defaults);
        assert_eq!(
            first.package.residual_payload_sha256,
            second.package.residual_payload_sha256
        );
        assert_eq!(
            first.package.projected_package_bytes,
            second.package.projected_package_bytes
        );
    }

    #[test]
    fn fractional_relaxation_never_exceeds_integral_schedule_fixture() {
        let lanes = vec![
            ModalQueryLane {
                positive_cells: vec![
                    PositiveCell {
                        delta: 10,
                        terminal_ids: vec![0, 1, 2],
                    },
                    PositiveCell {
                        delta: 4,
                        terminal_ids: vec![3],
                    },
                ],
                residual_terminal_ids: vec![],
            },
            ModalQueryLane {
                positive_cells: vec![PositiveCell {
                    delta: 8,
                    terminal_ids: vec![4, 5],
                }],
                residual_terminal_ids: vec![],
            },
        ];
        let relaxed = fractional_event_lower_bound(&lanes, 0, 7);
        let mut integral = usize::MAX;
        for first_cursor in 0..=2 {
            for second_cursor in 0..=1 {
                let head = lanes[0]
                    .positive_cells
                    .get(first_cursor)
                    .map_or(0, |cell| cell.delta)
                    + lanes[1]
                        .positive_cells
                        .get(second_cursor)
                        .map_or(0, |cell| cell.delta);
                if head < 7 {
                    let cost = lanes[0].positive_cells[..first_cursor]
                        .iter()
                        .map(|cell| cell.terminal_ids.len())
                        .sum::<usize>()
                        + lanes[1].positive_cells[..second_cursor]
                            .iter()
                            .map(|cell| cell.terminal_ids.len())
                            .sum::<usize>();
                    integral = integral.min(cost);
                }
            }
        }
        assert!(relaxed <= integral);
    }

    #[test]
    fn activation_vector_keeps_negative_and_zero_mass_coordinates() {
        let atom = ObservedAtom {
            position: 100,
            weight: 3,
            channel: AtomChannel::CharacterGram,
        };
        let default = ModalState::Relation {
            strength: 200,
            position: 90,
        };
        let absent = SignedActivation::from_forward(activation_for_state(atom, ModalState::Absent))
            .checked_sub(SignedActivation::from_forward(activation_for_state(
                atom, default,
            )))
            .unwrap();
        assert!(absent.mass < 0);
        assert_eq!(absent.hits, -1);
        assert_eq!(absent.surface_hits, -1);
    }
}
