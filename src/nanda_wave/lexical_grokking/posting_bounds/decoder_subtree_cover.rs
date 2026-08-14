//! Proof-only exact posting cover over homogeneous decoder subtrees.

use std::collections::{BTreeMap, BTreeSet};
use std::mem::size_of;
use std::path::Path;
use std::time::Instant;

use rayon::prelude::*;
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::{
    activation_equal, exact_contribution, is_keyboard_channel, validate_forward_posting,
    ExactPostingResult, PostingClosure, QueryPosting, SearchMetrics,
};
use crate::nanda_wave::lexical_grokking::atoms::AtomChannel;
use crate::nanda_wave::lexical_grokking::model::{DecoderNode, WaveCoupling};
use crate::nanda_wave::lexical_grokking::ngram_graph::NGramGraph;
use crate::nanda_wave::lexical_grokking::runtime::{
    ForwardActivation, LexicalGrokkingMemory, ObservedAtom,
};
use crate::nanda_wave::lexical_grokking::v8;

const ATOMS_PER_SHARD: usize = 32;
const OUTER_HEADER_BYTES: usize = 128;
const ATOM_INDEX_BYTES: usize = 16;
const SHARD_INDEX_BYTES: usize = 16;
const ZSTD_LEVEL: i32 = 19;
pub(super) const SUBTREE_PROJECTION_WORK_LIMIT: usize = 100_000;

const TOKEN_SUBTREE: u8 = 0;
const TOKEN_TERMINAL: u8 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct RelationState {
    strength: u8,
    position: u8,
}

impl RelationState {
    fn from_relation(relation: WaveCoupling) -> Self {
        Self {
            strength: relation.strength,
            position: relation.position_mode,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CoverToken {
    address: u32,
    kind: u8,
    strength: u8,
    position: u8,
    _reserved: u8,
}

impl CoverToken {
    fn subtree(node_id: u32, state: RelationState) -> Self {
        Self {
            address: node_id,
            kind: TOKEN_SUBTREE,
            strength: state.strength,
            position: state.position,
            _reserved: 0,
        }
    }

    fn terminal(terminal_rank: u32, state: RelationState) -> Self {
        Self {
            address: terminal_rank,
            kind: TOKEN_TERMINAL,
            strength: state.strength,
            position: state.position,
            _reserved: 0,
        }
    }

    fn state(self) -> RelationState {
        RelationState {
            strength: self.strength,
            position: self.position,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub(super) struct ChannelCoverMetrics {
    original_relations: usize,
    cover_tokens: usize,
    represented_relations: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct DecoderSubtreePackageMetrics {
    pub(super) current_file_bytes: u64,
    pub(super) compact_base_bytes: u64,
    pub(super) atoms: usize,
    pub(super) terminals: u32,
    pub(super) decoder_nodes: usize,
    pub(super) decoder_edges: usize,
    pub(super) maximum_decoder_depth: u8,
    pub(super) terminal_collisions: usize,
    pub(super) original_relation_events: usize,
    pub(super) represented_relation_events: usize,
    pub(super) cover_token_events: usize,
    pub(super) subtree_tokens: usize,
    pub(super) terminal_tokens: usize,
    pub(super) atoms_reduced: usize,
    pub(super) atoms_unchanged: usize,
    pub(super) largest_original_lane: usize,
    pub(super) largest_cover_lane: usize,
    pub(super) largest_token_cardinality: usize,
    pub(super) token_cardinality_p50: usize,
    pub(super) token_cardinality_p95: usize,
    pub(super) token_cardinality_p99: usize,
    pub(super) token_cardinality_histogram: BTreeMap<String, usize>,
    pub(super) channel_work: BTreeMap<String, ChannelCoverMetrics>,
    pub(super) atom_event_pairs_sha256: String,
    pub(super) raw_token_bytes: usize,
    pub(super) compressed_token_bytes: usize,
    pub(super) projected_atom_index_bytes: usize,
    pub(super) projected_shard_index_bytes: usize,
    pub(super) projected_alignment_bytes: usize,
    pub(super) projected_tree_metadata_bytes: usize,
    pub(super) projected_package_bytes: u64,
    pub(super) token_lanes_resident_bytes: usize,
    pub(super) topology_resident_bytes: usize,
    pub(super) atoms_per_shard: usize,
    pub(super) shard_count: usize,
    pub(super) zstd_level: i32,
    pub(super) token_payload_sha256: String,
    pub(super) state_partition_omissions: usize,
    pub(super) state_partition_duplicates: usize,
    pub(super) cover_overlap_violations: usize,
    pub(super) event_bound_violations: usize,
    pub(super) topology_build_ms: u128,
    pub(super) cover_build_ms: u128,
}

#[derive(Clone, Debug, Default, Serialize)]
pub(super) struct DecoderSubtreeQueryMetrics {
    pub(super) query_postings: usize,
    pub(super) original_relation_events: usize,
    pub(super) cover_token_events: usize,
    pub(super) subtree_tokens: usize,
    pub(super) terminal_tokens: usize,
    pub(super) unique_updated_nodes: usize,
    pub(super) naive_ancestor_insertions: usize,
    pub(super) activated_ancestor_closure_nodes: usize,
    pub(super) virtual_tree_edges: usize,
    pub(super) symbolic_cohort_records: usize,
    pub(super) merged_symbolic_activation_cohorts: usize,
    pub(super) maximum_symbolic_cohort_multiplicity: usize,
    pub(super) symbolic_terminal_count: usize,
    pub(super) symbolic_beta: u64,
    pub(super) dense_beta: u64,
    pub(super) symbolic_retained_count: usize,
    pub(super) retained_terminal_id_expansions: usize,
    pub(super) dense_retained_count: usize,
    pub(super) retained_id_symmetric_difference: usize,
    pub(super) activation_histogram_mismatches: usize,
    pub(super) reconstruction_field_mismatches: usize,
    pub(super) kth_or_equality_mismatches: usize,
    pub(super) projected_sparse_work_units: usize,
    pub(super) projected_hot_full_center_scans: usize,
    pub(super) projected_hot_full_decoder_scans: usize,
    pub(super) proof_only_dense_validation_scans: usize,
    pub(super) projection_us: u64,
}

pub(super) struct DecoderSubtreeQueryProjection {
    pub(super) exact: ExactPostingResult,
    pub(super) metrics: DecoderSubtreeQueryMetrics,
}

pub(super) struct DecoderSubtreeCoverProjection {
    topology: DecoderTopology,
    token_lanes: Vec<Vec<CoverToken>>,
    pub(super) package: DecoderSubtreePackageMetrics,
}

#[derive(Clone, Copy)]
struct PackageLayout {
    current_file_bytes: u64,
    compact_base_bytes: u64,
}

#[derive(Clone, Copy, Debug)]
struct RankedRelation {
    rank: u32,
    state: RelationState,
}

struct AtomCover {
    tokens: Vec<CoverToken>,
    original_relations: usize,
    represented_relations: usize,
    subtree_tokens: usize,
    terminal_tokens: usize,
    cardinality_counts: BTreeMap<u32, usize>,
}

struct BuiltShard {
    atom_covers: Vec<AtomCover>,
    raw_len: usize,
    compressed: Vec<u8>,
}

struct DecoderTopology {
    parents: Vec<u32>,
    child_offsets: Vec<u32>,
    children: Vec<u32>,
    terminal_offsets: Vec<u32>,
    terminal_ranks_by_node: Vec<u32>,
    terminal_id_to_rank: Vec<u32>,
    terminal_rank_to_id: Vec<u32>,
    terminal_rank_to_node: Vec<u32>,
    first_terminal_rank: Vec<u32>,
    subtree_terminal_count: Vec<u32>,
    depths: Vec<u8>,
    start_offsets: Vec<u32>,
    start_nodes: Vec<u32>,
    terminal_collisions: usize,
    maximum_depth: u8,
}

impl DecoderTopology {
    fn build(memory: &LexicalGrokkingMemory) -> Result<Self, String> {
        let nodes = &memory.package.decoder_nodes;
        let centers = &memory.package.centers;
        validate_decoder_parents(nodes)?;
        let node_count = nodes.len();
        let terminal_count = centers.len();

        let mut child_counts = vec![0_u32; node_count];
        for node in nodes.iter().skip(1) {
            let parent = node.parent as usize;
            child_counts[parent] = child_counts[parent]
                .checked_add(1)
                .ok_or_else(|| "decoder child count exceeds u32".to_string())?;
        }
        let child_offsets = prefix_offsets(&child_counts, "decoder child")?;
        let mut children_with_symbols = vec![(0_u32, 0_u32); node_count.saturating_sub(1)];
        let mut child_cursors = child_offsets[..node_count].to_vec();
        for (node_id, node) in nodes.iter().copied().enumerate().skip(1) {
            let parent = node.parent as usize;
            let cursor = child_cursors[parent] as usize;
            children_with_symbols[cursor] = (node.symbol, node_id as u32);
            child_cursors[parent] += 1;
        }
        for node_id in 0..node_count {
            let start = child_offsets[node_id] as usize;
            let end = child_offsets[node_id + 1] as usize;
            children_with_symbols[start..end].sort_unstable();
            if children_with_symbols[start..end]
                .windows(2)
                .any(|pair| pair[0].0 == pair[1].0)
            {
                return Err(format!(
                    "decoder node {node_id} has duplicate child symbols"
                ));
            }
        }
        let children = children_with_symbols
            .into_iter()
            .map(|(_, node_id)| node_id)
            .collect::<Vec<_>>();

        let mut terminal_counts = vec![0_u32; node_count];
        for center in centers {
            let node = center.decoder_terminal as usize;
            let count = terminal_counts
                .get_mut(node)
                .ok_or_else(|| "center references invalid decoder node".to_string())?;
            *count = count
                .checked_add(1)
                .ok_or_else(|| "decoder terminal collision count exceeds u32".to_string())?;
        }
        let terminal_collisions = terminal_counts.iter().filter(|count| **count > 1).count();
        let terminal_offsets = prefix_offsets(&terminal_counts, "decoder terminal")?;
        let mut terminal_ids_by_node = vec![0_u32; terminal_count];
        let mut terminal_cursors = terminal_offsets[..node_count].to_vec();
        for (terminal_id, center) in centers.iter().enumerate() {
            let node = center.decoder_terminal as usize;
            let cursor = terminal_cursors[node] as usize;
            terminal_ids_by_node[cursor] = terminal_id as u32;
            terminal_cursors[node] += 1;
        }
        for node_id in 0..node_count {
            let start = terminal_offsets[node_id] as usize;
            let end = terminal_offsets[node_id + 1] as usize;
            terminal_ids_by_node[start..end].sort_unstable();
        }

        let mut topology = Self {
            parents: nodes.iter().map(|node| node.parent).collect(),
            child_offsets,
            children,
            terminal_offsets,
            terminal_ranks_by_node: vec![0_u32; terminal_count],
            terminal_id_to_rank: vec![u32::MAX; terminal_count],
            terminal_rank_to_id: vec![u32::MAX; terminal_count],
            terminal_rank_to_node: vec![u32::MAX; terminal_count],
            first_terminal_rank: vec![u32::MAX; node_count],
            subtree_terminal_count: vec![0_u32; node_count],
            depths: vec![0_u8; node_count],
            start_offsets: Vec::new(),
            start_nodes: Vec::new(),
            terminal_collisions,
            maximum_depth: 0,
        };
        let mut next_rank = 0_u32;
        let mut visited = vec![false; node_count];
        topology.assign_dfs(0, 0, &terminal_ids_by_node, &mut next_rank, &mut visited)?;
        if next_rank as usize != terminal_count || visited.iter().any(|value| !*value) {
            return Err("decoder DFS does not cover every node and terminal".to_string());
        }
        if topology
            .terminal_id_to_rank
            .iter()
            .any(|rank| *rank == u32::MAX)
            || topology
                .terminal_rank_to_id
                .iter()
                .any(|terminal_id| *terminal_id == u32::MAX)
        {
            return Err("decoder DFS terminal permutation is incomplete".to_string());
        }
        topology.build_start_lists()?;
        Ok(topology)
    }

    fn assign_dfs(
        &mut self,
        node_id: u32,
        depth: u8,
        terminal_ids_by_node: &[u32],
        next_rank: &mut u32,
        visited: &mut [bool],
    ) -> Result<(), String> {
        let node = node_id as usize;
        if visited[node] {
            return Err(format!("decoder DFS revisits node {node_id}"));
        }
        visited[node] = true;
        self.depths[node] = depth;
        self.maximum_depth = self.maximum_depth.max(depth);
        self.first_terminal_rank[node] = *next_rank;

        let terminal_start = self.terminal_offsets[node] as usize;
        let terminal_end = self.terminal_offsets[node + 1] as usize;
        for slot in terminal_start..terminal_end {
            let terminal_id = terminal_ids_by_node[slot];
            let rank = *next_rank;
            self.terminal_ranks_by_node[slot] = rank;
            self.terminal_id_to_rank[terminal_id as usize] = rank;
            self.terminal_rank_to_id[rank as usize] = terminal_id;
            self.terminal_rank_to_node[rank as usize] = node_id;
            *next_rank = next_rank
                .checked_add(1)
                .ok_or_else(|| "decoder DFS rank exceeds u32".to_string())?;
        }

        let child_start = self.child_offsets[node] as usize;
        let child_end = self.child_offsets[node + 1] as usize;
        for cursor in child_start..child_end {
            let child = self.children[cursor];
            self.assign_dfs(
                child,
                depth
                    .checked_add(1)
                    .ok_or_else(|| "decoder depth exceeds u8".to_string())?,
                terminal_ids_by_node,
                next_rank,
                visited,
            )?;
        }
        self.subtree_terminal_count[node] =
            next_rank
                .checked_sub(self.first_terminal_rank[node])
                .ok_or_else(|| "decoder subtree rank underflow".to_string())?;
        Ok(())
    }

    fn build_start_lists(&mut self) -> Result<(), String> {
        let terminal_count = self.terminal_rank_to_id.len();
        let mut counts = vec![0_u32; terminal_count];
        for (node, count) in self.subtree_terminal_count.iter().copied().enumerate() {
            if count == 0 {
                continue;
            }
            let rank = self.first_terminal_rank[node] as usize;
            counts[rank] = counts[rank]
                .checked_add(1)
                .ok_or_else(|| "decoder start-list count exceeds u32".to_string())?;
        }
        self.start_offsets = prefix_offsets(&counts, "decoder start-list")?;
        self.start_nodes =
            vec![0_u32; self.start_offsets.last().copied().unwrap_or_default() as usize];
        let mut cursors = self.start_offsets[..terminal_count].to_vec();
        for (node, count) in self.subtree_terminal_count.iter().copied().enumerate() {
            if count == 0 {
                continue;
            }
            let rank = self.first_terminal_rank[node] as usize;
            let cursor = cursors[rank] as usize;
            self.start_nodes[cursor] = node as u32;
            cursors[rank] += 1;
        }
        for rank in 0..terminal_count {
            let start = self.start_offsets[rank] as usize;
            let end = self.start_offsets[rank + 1] as usize;
            self.start_nodes[start..end].sort_unstable_by(|left, right| {
                let left_index = *left as usize;
                let right_index = *right as usize;
                self.subtree_terminal_count[right_index]
                    .cmp(&self.subtree_terminal_count[left_index])
                    .then_with(|| self.depths[left_index].cmp(&self.depths[right_index]))
                    .then_with(|| left.cmp(right))
            });
        }
        Ok(())
    }

    fn children(&self, node_id: u32) -> &[u32] {
        let node = node_id as usize;
        let start = self.child_offsets[node] as usize;
        let end = self.child_offsets[node + 1] as usize;
        &self.children[start..end]
    }

    fn direct_terminal_ranks(&self, node_id: u32) -> &[u32] {
        let node = node_id as usize;
        let start = self.terminal_offsets[node] as usize;
        let end = self.terminal_offsets[node + 1] as usize;
        &self.terminal_ranks_by_node[start..end]
    }

    fn nodes_starting_at(&self, rank: u32) -> &[u32] {
        let rank = rank as usize;
        let start = self.start_offsets[rank] as usize;
        let end = self.start_offsets[rank + 1] as usize;
        &self.start_nodes[start..end]
    }

    fn token_cardinality(&self, token: CoverToken) -> Result<usize, String> {
        match token.kind {
            TOKEN_SUBTREE => self
                .subtree_terminal_count
                .get(token.address as usize)
                .copied()
                .map(|count| count as usize)
                .ok_or_else(|| "subtree token references invalid decoder node".to_string()),
            TOKEN_TERMINAL => self
                .terminal_rank_to_id
                .get(token.address as usize)
                .map(|_| 1)
                .ok_or_else(|| "terminal token references invalid DFS rank".to_string()),
            _ => Err("cover token has invalid kind".to_string()),
        }
    }

    fn resident_bytes(&self) -> usize {
        self.parents.capacity() * size_of::<u32>()
            + self.child_offsets.capacity() * size_of::<u32>()
            + self.children.capacity() * size_of::<u32>()
            + self.terminal_offsets.capacity() * size_of::<u32>()
            + self.terminal_ranks_by_node.capacity() * size_of::<u32>()
            + self.terminal_id_to_rank.capacity() * size_of::<u32>()
            + self.terminal_rank_to_id.capacity() * size_of::<u32>()
            + self.terminal_rank_to_node.capacity() * size_of::<u32>()
            + self.first_terminal_rank.capacity() * size_of::<u32>()
            + self.subtree_terminal_count.capacity() * size_of::<u32>()
            + self.depths.capacity() * size_of::<u8>()
            + self.start_offsets.capacity() * size_of::<u32>()
            + self.start_nodes.capacity() * size_of::<u32>()
    }
}

impl DecoderSubtreeCoverProjection {
    pub(super) fn build(
        memory: &LexicalGrokkingMemory,
        package_path: &Path,
    ) -> Result<Self, String> {
        let bytes = std::fs::read(package_path)
            .map_err(|error| format!("read subtree projection package: {error}"))?;
        let header = v8::read_header(&bytes)?;
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
        let topology_started = Instant::now();
        let topology = DecoderTopology::build(memory)?;
        let topology_build_ms = topology_started.elapsed().as_millis();
        let cover_started = Instant::now();
        let atom_count = memory.package.atoms.len();
        let channels = atom_channels(&memory.package.graph, atom_count)?;
        let shard_count = atom_count.div_ceil(ATOMS_PER_SHARD);
        let shards = (0..shard_count)
            .into_par_iter()
            .map(|shard_id| {
                build_shard(
                    memory,
                    &topology,
                    shard_id.saturating_mul(ATOMS_PER_SHARD),
                    atom_count.min((shard_id + 1).saturating_mul(ATOMS_PER_SHARD)),
                )
            })
            .collect::<Result<Vec<_>, String>>()?;

        let mut token_lanes = Vec::with_capacity(atom_count);
        let mut original_relation_events = 0_usize;
        let mut represented_relation_events = 0_usize;
        let mut cover_token_events = 0_usize;
        let mut subtree_tokens = 0_usize;
        let mut terminal_tokens = 0_usize;
        let mut atoms_reduced = 0_usize;
        let mut atoms_unchanged = 0_usize;
        let mut largest_original_lane = 0_usize;
        let mut largest_cover_lane = 0_usize;
        let mut raw_token_bytes = 0_usize;
        let mut compressed_token_bytes = 0_usize;
        let mut cardinality_counts = BTreeMap::<u32, usize>::new();
        let mut channel_work = (0..11)
            .map(|_| ChannelCoverMetrics::default())
            .collect::<Vec<_>>();
        let mut payload_hasher = Sha256::new();
        payload_hasher.update(b"lay.l11.decoder-subtree-cover-projection.v1");
        let mut atom_hasher = Sha256::new();
        atom_hasher.update(b"lay.l11.decoder-subtree-cover-atom-events.v1");

        for (shard_id, shard) in shards.into_iter().enumerate() {
            raw_token_bytes = raw_token_bytes.saturating_add(shard.raw_len);
            compressed_token_bytes = compressed_token_bytes.saturating_add(shard.compressed.len());
            payload_hasher.update((shard_id as u32).to_le_bytes());
            payload_hasher.update((shard.raw_len as u32).to_le_bytes());
            payload_hasher.update((shard.compressed.len() as u32).to_le_bytes());
            payload_hasher.update(&shard.compressed);
            for (local_index, cover) in shard.atom_covers.into_iter().enumerate() {
                let atom_id = shard_id * ATOMS_PER_SHARD + local_index;
                let channel = channels[atom_id];
                let channel_index = channel_index(channel);
                original_relation_events =
                    original_relation_events.saturating_add(cover.original_relations);
                represented_relation_events =
                    represented_relation_events.saturating_add(cover.represented_relations);
                cover_token_events = cover_token_events.saturating_add(cover.tokens.len());
                subtree_tokens = subtree_tokens.saturating_add(cover.subtree_tokens);
                terminal_tokens = terminal_tokens.saturating_add(cover.terminal_tokens);
                atoms_reduced += usize::from(cover.tokens.len() < cover.original_relations);
                atoms_unchanged += usize::from(cover.tokens.len() == cover.original_relations);
                largest_original_lane = largest_original_lane.max(cover.original_relations);
                largest_cover_lane = largest_cover_lane.max(cover.tokens.len());
                channel_work[channel_index].original_relations = channel_work[channel_index]
                    .original_relations
                    .saturating_add(cover.original_relations);
                channel_work[channel_index].cover_tokens = channel_work[channel_index]
                    .cover_tokens
                    .saturating_add(cover.tokens.len());
                channel_work[channel_index].represented_relations = channel_work[channel_index]
                    .represented_relations
                    .saturating_add(cover.represented_relations);
                for (cardinality, count) in cover.cardinality_counts {
                    *cardinality_counts.entry(cardinality).or_default() += count;
                }
                atom_hasher.update((atom_id as u32).to_le_bytes());
                atom_hasher.update((cover.original_relations as u32).to_le_bytes());
                atom_hasher.update((cover.tokens.len() as u32).to_le_bytes());
                token_lanes.push(cover.tokens);
            }
        }

        if token_lanes.len() != atom_count
            || original_relation_events != memory.forward_relation_count()
            || represented_relation_events != original_relation_events
            || cover_token_events > original_relation_events
        {
            return Err("decoder subtree full-field accounting differs".to_string());
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
        let token_offset = align8(shard_index_offset.saturating_add(projected_shard_index_bytes));
        let projected_package_bytes = token_offset.saturating_add(compressed_token_bytes);
        let projected_alignment_bytes = index_offset
            .saturating_sub(OUTER_HEADER_BYTES + layout.compact_base_bytes as usize)
            .saturating_add(
                shard_index_offset.saturating_sub(index_offset + projected_atom_index_bytes),
            )
            .saturating_add(
                token_offset.saturating_sub(shard_index_offset + projected_shard_index_bytes),
            );
        let token_lanes_resident_bytes = token_lanes
            .iter()
            .map(|lane| lane.capacity().saturating_mul(size_of::<CoverToken>()))
            .sum::<usize>()
            .saturating_add(
                token_lanes
                    .capacity()
                    .saturating_mul(size_of::<Vec<CoverToken>>()),
            );
        let channel_work = channels_to_map(channel_work);
        let token_cardinality_histogram = cardinality_histogram(&cardinality_counts);
        let package = DecoderSubtreePackageMetrics {
            current_file_bytes: layout.current_file_bytes,
            compact_base_bytes: layout.compact_base_bytes,
            atoms: atom_count,
            terminals: memory.package.terminal_count(),
            decoder_nodes: topology.parents.len(),
            decoder_edges: topology.children.len(),
            maximum_decoder_depth: topology.maximum_depth,
            terminal_collisions: topology.terminal_collisions,
            original_relation_events,
            represented_relation_events,
            cover_token_events,
            subtree_tokens,
            terminal_tokens,
            atoms_reduced,
            atoms_unchanged,
            largest_original_lane,
            largest_cover_lane,
            largest_token_cardinality: cardinality_counts
                .keys()
                .next_back()
                .copied()
                .unwrap_or_default() as usize,
            token_cardinality_p50: histogram_percentile(&cardinality_counts, 50),
            token_cardinality_p95: histogram_percentile(&cardinality_counts, 95),
            token_cardinality_p99: histogram_percentile(&cardinality_counts, 99),
            token_cardinality_histogram,
            channel_work,
            atom_event_pairs_sha256: format!("{:x}", atom_hasher.finalize()),
            raw_token_bytes,
            compressed_token_bytes,
            projected_atom_index_bytes,
            projected_shard_index_bytes,
            projected_alignment_bytes,
            projected_tree_metadata_bytes: 0,
            projected_package_bytes: projected_package_bytes as u64,
            token_lanes_resident_bytes,
            topology_resident_bytes: topology.resident_bytes(),
            atoms_per_shard: ATOMS_PER_SHARD,
            shard_count,
            zstd_level: ZSTD_LEVEL,
            token_payload_sha256: format!("{:x}", payload_hasher.finalize()),
            state_partition_omissions: 0,
            state_partition_duplicates: 0,
            cover_overlap_violations: 0,
            event_bound_violations: 0,
            topology_build_ms,
            cover_build_ms: cover_started.elapsed().as_millis(),
        };
        Ok(Self {
            topology,
            token_lanes,
            package,
        })
    }

    pub(super) fn project_query(
        &self,
        postings: &[QueryPosting<'_>],
        requested_k: usize,
        dense_beta: u64,
        dense_all: &[ForwardActivation],
    ) -> Result<DecoderSubtreeQueryProjection, String> {
        let started = Instant::now();
        let node_count = self.topology.parents.len();
        let terminal_count = self.topology.terminal_rank_to_id.len();
        if dense_all.len() != terminal_count {
            return Err("subtree projection dense field has wrong terminal count".to_string());
        }
        let mut subtree_lazy = vec![ForwardActivation::default(); node_count];
        let mut terminal_lazy = vec![ForwardActivation::default(); terminal_count];
        let mut updated_nodes = vec![false; node_count];
        let mut updated_terminals = vec![false; terminal_count];
        let mut metrics = DecoderSubtreeQueryMetrics {
            query_postings: postings.len(),
            dense_beta,
            projected_hot_full_center_scans: 0,
            projected_hot_full_decoder_scans: 0,
            proof_only_dense_validation_scans: 1,
            ..DecoderSubtreeQueryMetrics::default()
        };

        for posting in postings {
            metrics.original_relation_events = metrics
                .original_relation_events
                .saturating_add(posting.relations.len());
            let tokens = self
                .token_lanes
                .get(posting.atom_id as usize)
                .ok_or_else(|| format!("missing subtree lane for atom {}", posting.atom_id))?;
            metrics.cover_token_events = metrics.cover_token_events.saturating_add(tokens.len());
            for token in tokens.iter().copied() {
                let activation = activation_for_state(posting.atom, token.state());
                match token.kind {
                    TOKEN_SUBTREE => {
                        let node = token.address as usize;
                        checked_add_activation(&mut subtree_lazy[node], activation)?;
                        updated_nodes[node] = true;
                        metrics.subtree_tokens += 1;
                    }
                    TOKEN_TERMINAL => {
                        let rank = token.address as usize;
                        checked_add_activation(&mut terminal_lazy[rank], activation)?;
                        updated_terminals[rank] = true;
                        updated_nodes[self.topology.terminal_rank_to_node[rank] as usize] = true;
                        metrics.terminal_tokens += 1;
                    }
                    _ => return Err("subtree query token kind is invalid".to_string()),
                }
            }
        }

        metrics.unique_updated_nodes = updated_nodes.iter().filter(|value| **value).count();
        let mut closure = updated_nodes;
        closure[0] = true;
        let initially_updated = closure
            .iter()
            .enumerate()
            .filter_map(|(node, active)| (*active).then_some(node as u32))
            .collect::<Vec<_>>();
        for mut node in initially_updated {
            while node != 0 {
                metrics.naive_ancestor_insertions += 1;
                node = self.topology.parents[node as usize];
                if closure[node as usize] {
                    break;
                }
                closure[node as usize] = true;
            }
        }
        let mut closure_nodes = closure
            .iter()
            .enumerate()
            .filter_map(|(node, active)| (*active).then_some(node as u32))
            .collect::<Vec<_>>();
        metrics.activated_ancestor_closure_nodes = closure_nodes.len();
        metrics.virtual_tree_edges = closure_nodes.len().saturating_sub(1);
        closure_nodes.sort_unstable_by_key(|node| (self.topology.depths[*node as usize], *node));

        let mut active_child_terminals = vec![0_u32; node_count];
        for node in closure_nodes.iter().copied().filter(|node| *node != 0) {
            let parent = self.topology.parents[node as usize] as usize;
            active_child_terminals[parent] = active_child_terminals[parent]
                .checked_add(self.topology.subtree_terminal_count[node as usize])
                .ok_or_else(|| "active child terminal count exceeds u32".to_string())?;
        }
        let mut path_activation = vec![ForwardActivation::default(); node_count];
        let mut cohorts = Vec::<(ForwardActivation, usize)>::new();
        for node_id in closure_nodes.iter().copied() {
            let node = node_id as usize;
            let inherited = if node_id == 0 {
                ForwardActivation::default()
            } else {
                path_activation[self.topology.parents[node] as usize]
            };
            let mut path = inherited;
            checked_add_activation(&mut path, subtree_lazy[node])?;
            path_activation[node] = path;
            let direct = self.topology.direct_terminal_ranks(node_id);
            let active_direct = direct
                .iter()
                .filter(|rank| updated_terminals[**rank as usize])
                .count();
            let complement = self.topology.subtree_terminal_count[node] as usize
                - active_child_terminals[node] as usize
                - active_direct;
            if complement != 0 {
                cohorts.push((path, complement));
            }
            for rank in direct.iter().copied() {
                if !updated_terminals[rank as usize] {
                    continue;
                }
                let mut activation = path;
                checked_add_activation(&mut activation, terminal_lazy[rank as usize])?;
                cohorts.push((activation, 1));
            }
        }
        metrics.symbolic_cohort_records = cohorts.len();
        metrics.maximum_symbolic_cohort_multiplicity = cohorts
            .iter()
            .map(|(_, count)| *count)
            .max()
            .unwrap_or_default();
        metrics.symbolic_terminal_count = cohorts.iter().map(|(_, count)| *count).sum();
        if metrics.symbolic_terminal_count != terminal_count {
            return Err(format!(
                "symbolic subtree cohorts cover {} terminals, expected {terminal_count}",
                metrics.symbolic_terminal_count
            ));
        }
        let symbolic_histogram = activation_histogram(&cohorts);
        metrics.merged_symbolic_activation_cohorts = symbolic_histogram.len();
        metrics.symbolic_beta = symbolic_kth_mass(&cohorts, requested_k)?;
        metrics.symbolic_retained_count = cohorts
            .iter()
            .filter(|(activation, _)| activation.mass >= metrics.symbolic_beta)
            .map(|(_, count)| *count)
            .sum();
        // The next owner consumes concrete terminal IDs, so equality cohorts
        // above beta must pay their complete materialization cost.
        metrics.retained_terminal_id_expansions = metrics.symbolic_retained_count;

        let mut all = vec![ForwardActivation::default(); terminal_count];
        self.expand_activations(
            0,
            ForwardActivation::default(),
            &subtree_lazy,
            &terminal_lazy,
            &mut all,
        )?;
        metrics.reconstruction_field_mismatches = all
            .iter()
            .copied()
            .zip(dense_all.iter().copied())
            .filter(|(left, right)| !activation_equal(*left, *right))
            .count();
        let expanded_histogram = activation_histogram(
            &all.iter()
                .copied()
                .map(|activation| (activation, 1_usize))
                .collect::<Vec<_>>(),
        );
        metrics.activation_histogram_mismatches =
            histogram_mismatches(&symbolic_histogram, &expanded_histogram);
        let exact = exact_result_from_activations(all, postings, requested_k);
        metrics.dense_retained_count = dense_all
            .iter()
            .filter(|activation| activation.mass >= dense_beta)
            .count();
        let exact_ids = exact
            .closure
            .retained
            .iter()
            .map(|(terminal_id, _)| *terminal_id)
            .collect::<BTreeSet<_>>();
        let dense_ids = dense_all
            .iter()
            .enumerate()
            .filter_map(|(terminal_id, activation)| {
                (activation.mass >= dense_beta).then_some(terminal_id as u32)
            })
            .collect::<BTreeSet<_>>();
        metrics.retained_id_symmetric_difference =
            exact_ids.symmetric_difference(&dense_ids).count();
        metrics.kth_or_equality_mismatches = usize::from(
            exact.closure.beta_k != dense_beta
                || metrics.symbolic_beta != dense_beta
                || metrics.symbolic_retained_count != metrics.dense_retained_count
                || metrics.retained_id_symmetric_difference != 0,
        );
        metrics.projected_sparse_work_units = metrics
            .cover_token_events
            .saturating_add(metrics.activated_ancestor_closure_nodes)
            .saturating_add(metrics.symbolic_cohort_records)
            .saturating_add(metrics.retained_terminal_id_expansions);
        metrics.projection_us = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
        Ok(DecoderSubtreeQueryProjection { exact, metrics })
    }

    fn expand_activations(
        &self,
        node_id: u32,
        inherited: ForwardActivation,
        subtree_lazy: &[ForwardActivation],
        terminal_lazy: &[ForwardActivation],
        all: &mut [ForwardActivation],
    ) -> Result<(), String> {
        let mut path = inherited;
        checked_add_activation(&mut path, subtree_lazy[node_id as usize])?;
        for rank in self.topology.direct_terminal_ranks(node_id).iter().copied() {
            let mut activation = path;
            checked_add_activation(&mut activation, terminal_lazy[rank as usize])?;
            let terminal_id = self.topology.terminal_rank_to_id[rank as usize] as usize;
            all[terminal_id] = activation;
        }
        for child in self.topology.children(node_id).iter().copied() {
            self.expand_activations(child, path, subtree_lazy, terminal_lazy, all)?;
        }
        Ok(())
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
    topology: &DecoderTopology,
    atom_start: usize,
    atom_end: usize,
) -> Result<BuiltShard, String> {
    let atom_ids = (atom_start..atom_end)
        .map(|atom_id| u32::try_from(atom_id).map_err(|_| "atom id exceeds u32".to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let postings = memory.complete_forward_couplings_batch(&atom_ids)?;
    if postings.len() != atom_ids.len() {
        return Err("subtree shard posting count differs".to_string());
    }
    let mut raw = Vec::new();
    let mut atom_covers = Vec::with_capacity(postings.len());
    for relations in postings {
        validate_forward_posting(&relations, memory.package.terminal_count())?;
        let cover = build_atom_cover(&relations, topology)?;
        put_varint(&mut raw, cover.tokens.len() as u32);
        for token in &cover.tokens {
            raw.push(token.kind);
            put_varint(&mut raw, token.address);
            raw.push(token.strength);
            raw.push(token.position);
        }
        atom_covers.push(cover);
    }
    let raw_len = raw.len();
    let compressed = zstd::bulk::compress(&raw, ZSTD_LEVEL)
        .map_err(|error| format!("subtree token compression failed: {error}"))?;
    Ok(BuiltShard {
        atom_covers,
        raw_len,
        compressed,
    })
}

fn build_atom_cover(
    relations: &[WaveCoupling],
    topology: &DecoderTopology,
) -> Result<AtomCover, String> {
    let mut ranked = relations
        .iter()
        .copied()
        .map(|relation| {
            let rank = *topology
                .terminal_id_to_rank
                .get(relation.peer_id as usize)
                .ok_or_else(|| "posting terminal exceeds decoder terminal map".to_string())?;
            Ok(RankedRelation {
                rank,
                state: RelationState::from_relation(relation),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    ranked.sort_unstable_by_key(|relation| relation.rank);
    if ranked.windows(2).any(|pair| pair[0].rank == pair[1].rank) {
        return Err("decoder-ranked posting contains duplicate terminal".to_string());
    }
    let mut run_lengths = vec![1_usize; ranked.len()];
    for index in (0..ranked.len().saturating_sub(1)).rev() {
        if ranked[index + 1].rank == ranked[index].rank + 1
            && ranked[index + 1].state == ranked[index].state
        {
            run_lengths[index] = run_lengths[index + 1].saturating_add(1);
        }
    }
    let mut tokens = Vec::new();
    let mut cardinality_counts = BTreeMap::new();
    let mut represented_relations = 0_usize;
    let mut subtree_tokens = 0_usize;
    let mut terminal_tokens = 0_usize;
    let mut cursor = 0_usize;
    while cursor < ranked.len() {
        let relation = ranked[cursor];
        let selected = topology
            .nodes_starting_at(relation.rank)
            .iter()
            .copied()
            .find(|node| {
                topology.subtree_terminal_count[*node as usize] as usize <= run_lengths[cursor]
            });
        let (token, cardinality) = if let Some(node) = selected {
            let cardinality = topology.subtree_terminal_count[node as usize] as usize;
            subtree_tokens += 1;
            (CoverToken::subtree(node, relation.state), cardinality)
        } else {
            terminal_tokens += 1;
            (CoverToken::terminal(relation.rank, relation.state), 1)
        };
        if cardinality == 0
            || cursor.saturating_add(cardinality) > ranked.len()
            || ranked[cursor + cardinality - 1].rank
                != relation.rank.saturating_add(cardinality as u32 - 1)
            || ranked[cursor..cursor + cardinality]
                .iter()
                .any(|item| item.state != relation.state)
        {
            return Err("subtree token does not cover one exact contiguous state run".to_string());
        }
        represented_relations = represented_relations.saturating_add(cardinality);
        *cardinality_counts.entry(cardinality as u32).or_default() += 1;
        tokens.push(token);
        cursor += cardinality;
    }
    if represented_relations != relations.len() || tokens.len() > relations.len() {
        return Err("subtree atom cover accounting differs".to_string());
    }
    Ok(AtomCover {
        tokens,
        original_relations: relations.len(),
        represented_relations,
        subtree_tokens,
        terminal_tokens,
        cardinality_counts,
    })
}

fn validate_decoder_parents(nodes: &[DecoderNode]) -> Result<(), String> {
    let Some(root) = nodes.first() else {
        return Err("decoder topology has no root".to_string());
    };
    if root.parent != u32::MAX || root.symbol != 0 {
        return Err("decoder root is invalid".to_string());
    }
    for (node_id, node) in nodes.iter().enumerate().skip(1) {
        if node.parent as usize >= nodes.len() || node.parent as usize == node_id {
            return Err(format!("decoder node {node_id} has invalid parent"));
        }
    }
    Ok(())
}

fn prefix_offsets(counts: &[u32], name: &str) -> Result<Vec<u32>, String> {
    let mut offsets = Vec::with_capacity(counts.len() + 1);
    offsets.push(0_u32);
    for count in counts {
        let next = offsets
            .last()
            .copied()
            .unwrap_or_default()
            .checked_add(*count)
            .ok_or_else(|| format!("{name} offsets exceed u32"))?;
        offsets.push(next);
    }
    Ok(offsets)
}

fn activation_for_state(atom: ObservedAtom, state: RelationState) -> ForwardActivation {
    let mut activation = ForwardActivation {
        mass: exact_contribution(
            atom,
            WaveCoupling {
                strength: state.strength,
                position_mode: state.position,
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

fn checked_add_activation(
    target: &mut ForwardActivation,
    value: ForwardActivation,
) -> Result<(), String> {
    target.mass = target
        .mass
        .checked_add(value.mass)
        .ok_or_else(|| "subtree activation mass overflow".to_string())?;
    target.hits = target
        .hits
        .checked_add(value.hits)
        .ok_or_else(|| "subtree activation hits overflow".to_string())?;
    target.surface_hits = target
        .surface_hits
        .checked_add(value.surface_hits)
        .ok_or_else(|| "subtree activation surface hits overflow".to_string())?;
    target.keyboard_hits = target
        .keyboard_hits
        .checked_add(value.keyboard_hits)
        .ok_or_else(|| "subtree activation keyboard hits overflow".to_string())?;
    Ok(())
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
                posting_iterators: postings.len(),
                ..SearchMetrics::default()
            },
        },
        touched,
        all,
    }
}

type ActivationKey = (u64, u16, u16, u16);

fn activation_key(activation: ForwardActivation) -> ActivationKey {
    (
        activation.mass,
        activation.hits,
        activation.surface_hits,
        activation.keyboard_hits,
    )
}

fn activation_histogram(cohorts: &[(ForwardActivation, usize)]) -> BTreeMap<ActivationKey, usize> {
    let mut histogram = BTreeMap::new();
    for (activation, count) in cohorts {
        *histogram.entry(activation_key(*activation)).or_default() += *count;
    }
    histogram
}

fn histogram_mismatches(
    left: &BTreeMap<ActivationKey, usize>,
    right: &BTreeMap<ActivationKey, usize>,
) -> usize {
    left.keys()
        .chain(right.keys())
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|key| {
            left.get(key).copied().unwrap_or_default()
                != right.get(key).copied().unwrap_or_default()
        })
        .count()
}

fn symbolic_kth_mass(
    cohorts: &[(ForwardActivation, usize)],
    requested_k: usize,
) -> Result<u64, String> {
    let total = cohorts.iter().map(|(_, count)| *count).sum::<usize>();
    if total == 0 {
        return Err("symbolic subtree readout has no terminals".to_string());
    }
    let effective_k = requested_k.max(1).min(total);
    let mut masses = cohorts
        .iter()
        .map(|(activation, count)| (activation.mass, *count))
        .collect::<Vec<_>>();
    masses.sort_unstable_by(|left, right| right.0.cmp(&left.0));
    let mut cumulative = 0_usize;
    for (mass, count) in masses {
        cumulative = cumulative.saturating_add(count);
        if cumulative >= effective_k {
            return Ok(mass);
        }
    }
    Err("symbolic subtree K-th mass is unresolved".to_string())
}

fn atom_channels(graph: &NGramGraph, atom_count: usize) -> Result<Vec<AtomChannel>, String> {
    let mut channels = vec![None; atom_count];
    let root = graph
        .nodes
        .first()
        .ok_or_else(|| "n-gram graph has no root".to_string())?;
    let start = root.first_arc as usize;
    let end = start.saturating_add(root.arc_count as usize);
    let mut stack = graph.arcs[start..end]
        .iter()
        .map(|arc| {
            Ok((
                arc.next_node,
                atom_channel_from_id((arc.symbol & 0xff) as u8)?,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    while let Some((node_id, channel)) = stack.pop() {
        let node = graph
            .nodes
            .get(node_id as usize)
            .ok_or_else(|| "n-gram graph arc references invalid node".to_string())?;
        if node.atom_id != u32::MAX {
            let slot = channels
                .get_mut(node.atom_id as usize)
                .ok_or_else(|| "n-gram graph atom id exceeds package atoms".to_string())?;
            if slot.replace(channel).is_some() {
                return Err("n-gram graph assigns atom channel twice".to_string());
            }
        }
        let start = node.first_arc as usize;
        let end = start.saturating_add(node.arc_count as usize);
        stack.extend(
            graph.arcs[start..end]
                .iter()
                .map(|arc| (arc.next_node, channel)),
        );
    }
    channels
        .into_iter()
        .map(|channel| channel.ok_or_else(|| "package atom has no n-gram channel".to_string()))
        .collect()
}

fn atom_channel_from_id(id: u8) -> Result<AtomChannel, String> {
    match id {
        1 => Ok(AtomChannel::ByteGram),
        2 => Ok(AtomChannel::CharacterGram),
        3 => Ok(AtomChannel::KeyboardGram),
        4 => Ok(AtomChannel::BoundaryPosition),
        5 => Ok(AtomChannel::CharacterBigram),
        6 => Ok(AtomChannel::KeyboardBigram),
        7 => Ok(AtomChannel::CharacterBagGram),
        8 => Ok(AtomChannel::KeyboardBagGram),
        9 => Ok(AtomChannel::CharacterSkipGram),
        10 => Ok(AtomChannel::KeyboardSkipGram),
        11 => Ok(AtomChannel::CharacterAnchor),
        _ => Err(format!("unknown atom channel id {id}")),
    }
}

fn channel_index(channel: AtomChannel) -> usize {
    channel as usize - 1
}

fn channels_to_map(values: Vec<ChannelCoverMetrics>) -> BTreeMap<String, ChannelCoverMetrics> {
    let channels = [
        AtomChannel::ByteGram,
        AtomChannel::CharacterGram,
        AtomChannel::KeyboardGram,
        AtomChannel::BoundaryPosition,
        AtomChannel::CharacterBigram,
        AtomChannel::KeyboardBigram,
        AtomChannel::CharacterBagGram,
        AtomChannel::KeyboardBagGram,
        AtomChannel::CharacterSkipGram,
        AtomChannel::KeyboardSkipGram,
        AtomChannel::CharacterAnchor,
    ];
    channels
        .into_iter()
        .zip(values)
        .map(|(channel, metrics)| (channel_name(channel).to_string(), metrics))
        .collect()
}

fn channel_name(channel: AtomChannel) -> &'static str {
    match channel {
        AtomChannel::ByteGram => "byte_gram",
        AtomChannel::CharacterGram => "character_gram",
        AtomChannel::KeyboardGram => "keyboard_gram",
        AtomChannel::BoundaryPosition => "boundary_position",
        AtomChannel::CharacterBigram => "character_bigram",
        AtomChannel::KeyboardBigram => "keyboard_bigram",
        AtomChannel::CharacterBagGram => "character_bag_gram",
        AtomChannel::KeyboardBagGram => "keyboard_bag_gram",
        AtomChannel::CharacterSkipGram => "character_skip_gram",
        AtomChannel::KeyboardSkipGram => "keyboard_skip_gram",
        AtomChannel::CharacterAnchor => "character_anchor",
    }
}

fn cardinality_histogram(counts: &BTreeMap<u32, usize>) -> BTreeMap<String, usize> {
    let mut buckets = BTreeMap::from([
        ("1".to_string(), 0_usize),
        ("2-3".to_string(), 0),
        ("4-7".to_string(), 0),
        ("8-15".to_string(), 0),
        ("16-31".to_string(), 0),
        ("32-63".to_string(), 0),
        ("64-127".to_string(), 0),
        ("128-255".to_string(), 0),
        ("256-1023".to_string(), 0),
        ("1024+".to_string(), 0),
    ]);
    for (cardinality, count) in counts {
        let label = match *cardinality {
            1 => "1",
            2..=3 => "2-3",
            4..=7 => "4-7",
            8..=15 => "8-15",
            16..=31 => "16-31",
            32..=63 => "32-63",
            64..=127 => "64-127",
            128..=255 => "128-255",
            256..=1023 => "256-1023",
            _ => "1024+",
        };
        *buckets.get_mut(label).expect("closed cardinality bucket") += *count;
    }
    buckets
}

fn histogram_percentile(counts: &BTreeMap<u32, usize>, percentile: usize) -> usize {
    let total = counts.values().sum::<usize>();
    if total == 0 {
        return 0;
    }
    let target = total.saturating_sub(1).saturating_mul(percentile) / 100;
    let mut cumulative = 0_usize;
    for (value, count) in counts {
        cumulative = cumulative.saturating_add(*count);
        if cumulative > target {
            return *value as usize;
        }
    }
    counts.keys().next_back().copied().unwrap_or_default() as usize
}

pub(super) fn summarize_queries(metrics: &[DecoderSubtreeQueryMetrics]) -> serde_json::Value {
    let sum = |field: fn(&DecoderSubtreeQueryMetrics) -> usize| {
        metrics
            .iter()
            .map(field)
            .fold(0_usize, usize::saturating_add)
    };
    let max = |field: fn(&DecoderSubtreeQueryMetrics) -> usize| {
        metrics.iter().map(field).max().unwrap_or_default()
    };
    serde_json::json!({
        "cases": metrics.len(),
        "query_postings": sum(|item| item.query_postings),
        "original_relation_events": sum(|item| item.original_relation_events),
        "cover_token_events": sum(|item| item.cover_token_events),
        "cover_token_events_max": max(|item| item.cover_token_events),
        "subtree_tokens": sum(|item| item.subtree_tokens),
        "terminal_tokens": sum(|item| item.terminal_tokens),
        "unique_updated_nodes_max": max(|item| item.unique_updated_nodes),
        "naive_ancestor_insertions_max": max(|item| item.naive_ancestor_insertions),
        "activated_ancestor_closure_nodes_max": max(|item| item.activated_ancestor_closure_nodes),
        "virtual_tree_edges_max": max(|item| item.virtual_tree_edges),
        "symbolic_cohort_records_max": max(|item| item.symbolic_cohort_records),
        "merged_symbolic_activation_cohorts_max": max(|item| item.merged_symbolic_activation_cohorts),
        "maximum_symbolic_cohort_multiplicity": max(|item| item.maximum_symbolic_cohort_multiplicity),
        "symbolic_terminal_count_min": metrics.iter().map(|item| item.symbolic_terminal_count).min().unwrap_or_default(),
        "symbolic_retained_count_max": max(|item| item.symbolic_retained_count),
        "retained_terminal_id_expansions_max": max(|item| item.retained_terminal_id_expansions),
        "retained_id_symmetric_difference": sum(|item| item.retained_id_symmetric_difference),
        "activation_histogram_mismatches": sum(|item| item.activation_histogram_mismatches),
        "reconstruction_field_mismatches": sum(|item| item.reconstruction_field_mismatches),
        "kth_or_equality_mismatches": sum(|item| item.kth_or_equality_mismatches),
        "projected_sparse_work_units_max": max(|item| item.projected_sparse_work_units),
        "projected_hot_full_center_scans": sum(|item| item.projected_hot_full_center_scans),
        "projected_hot_full_decoder_scans": sum(|item| item.projected_hot_full_decoder_scans),
        "proof_only_dense_validation_scans": sum(|item| item.proof_only_dense_validation_scans),
        "projection_us_max": metrics.iter().map(|item| item.projection_us).max().unwrap_or_default(),
        "cover_token_gate": metrics.iter().all(|item| item.cover_token_events <= SUBTREE_PROJECTION_WORK_LIMIT),
        "closure_gate": metrics.iter().all(|item| item.activated_ancestor_closure_nodes <= SUBTREE_PROJECTION_WORK_LIMIT),
        "cohort_gate": metrics.iter().all(|item| item.symbolic_cohort_records <= SUBTREE_PROJECTION_WORK_LIMIT),
        "retained_id_expansion_gate": metrics.iter().all(|item| item.retained_terminal_id_expansions <= SUBTREE_PROJECTION_WORK_LIMIT),
        "combined_work_gate": metrics.iter().all(|item| item.projected_sparse_work_units <= SUBTREE_PROJECTION_WORK_LIMIT),
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
    use crate::nanda_wave::lexical_grokking::compiler::compile;
    use crate::nanda_wave::lexical_grokking::training_corpus::TrainingWord;

    fn memory(words: &[&str]) -> LexicalGrokkingMemory {
        let words = words
            .iter()
            .enumerate()
            .map(|(terminal_id, surface)| TrainingWord {
                terminal_id: terminal_id as u32,
                surface: (*surface).to_string(),
                training_surfaces: Vec::new(),
            })
            .collect::<Vec<_>>();
        LexicalGrokkingMemory::from_package(compile(&words).expect("compile subtree fixture"))
    }

    #[test]
    fn decoder_dfs_is_a_complete_terminal_permutation() {
        let memory = memory(&["a", "ab", "abc", "abd", "b", "ба", "бар"]);
        let topology = DecoderTopology::build(&memory).unwrap();
        assert_eq!(topology.subtree_terminal_count[0], 7);
        assert_eq!(topology.terminal_rank_to_id.len(), 7);
        for terminal_id in 0..7_u32 {
            let rank = topology.terminal_id_to_rank[terminal_id as usize];
            assert_eq!(topology.terminal_rank_to_id[rank as usize], terminal_id);
        }
        for node in 1..topology.parents.len() {
            let parent = topology.parents[node] as usize;
            assert!(topology.first_terminal_rank[parent] <= topology.first_terminal_rank[node]);
            assert!(
                topology.first_terminal_rank[node] + topology.subtree_terminal_count[node]
                    <= topology.first_terminal_rank[parent]
                        + topology.subtree_terminal_count[parent]
            );
        }
    }

    #[test]
    fn every_atom_cover_expands_to_complete_relation_states() {
        let memory = memory(&["a", "ab", "abc", "abd", "b", "ба", "бар"]);
        let projection = DecoderSubtreeCoverProjection::build_for_test(&memory).unwrap();
        assert_eq!(
            projection.package.represented_relation_events,
            projection.package.original_relation_events
        );
        assert!(
            projection.package.cover_token_events <= projection.package.original_relation_events
        );
        for (atom_id, tokens) in projection.token_lanes.iter().enumerate() {
            let relations = memory.complete_forward_couplings(atom_id as u32).unwrap();
            let mut expected = vec![None; memory.package.terminal_count() as usize];
            for relation in relations.iter().copied() {
                expected[relation.peer_id as usize] = Some(RelationState::from_relation(relation));
            }
            let mut represented = vec![None; expected.len()];
            for token in tokens.iter().copied() {
                let (start, count) = match token.kind {
                    TOKEN_SUBTREE => (
                        projection.topology.first_terminal_rank[token.address as usize],
                        projection.topology.token_cardinality(token).unwrap(),
                    ),
                    TOKEN_TERMINAL => (token.address, 1),
                    _ => panic!("invalid token kind"),
                };
                for rank in start..start + count as u32 {
                    let terminal_id = projection.topology.terminal_rank_to_id[rank as usize];
                    assert!(
                        represented[terminal_id as usize]
                            .replace(token.state())
                            .is_none(),
                        "overlap for atom {atom_id}, terminal {terminal_id}"
                    );
                }
            }
            assert_eq!(represented, expected, "atom {atom_id}");
        }
    }

    #[test]
    fn word_that_is_a_prefix_can_fall_back_to_terminal_token() {
        let memory = memory(&["a", "ab", "abc", "abd"]);
        let projection = DecoderSubtreeCoverProjection::build_for_test(&memory).unwrap();
        assert!(projection.package.terminal_tokens > 0);
    }

    #[test]
    fn decoder_terminal_collisions_preserve_every_center() {
        let memory = memory(&["same", "same", "sameness"]);
        let projection = DecoderSubtreeCoverProjection::build_for_test(&memory).unwrap();
        assert_eq!(projection.package.terminal_collisions, 1);
        let postings = super::super::query_postings(&memory, None, "same").unwrap();
        let dense =
            super::super::exact_posting_closure(&postings, memory.package.terminal_count(), 2);
        let actual = projection
            .project_query(&postings, 2, dense.closure.beta_k, &dense.all)
            .unwrap();
        assert_eq!(actual.metrics.reconstruction_field_mismatches, 0);
        assert_eq!(actual.metrics.kth_or_equality_mismatches, 0);
        assert_eq!(actual.metrics.retained_id_symmetric_difference, 0);
    }

    #[test]
    fn subtree_projection_matches_dense_activation_and_equality() {
        let memory = memory(&["form", "formal", "format", "farm", "foam", "from", "форма"]);
        let projection = DecoderSubtreeCoverProjection::build_for_test(&memory).unwrap();
        for surface in ["form", "frmo", "forma", "фрма"] {
            let postings = super::super::query_postings(&memory, None, surface).unwrap();
            let dense =
                super::super::exact_posting_closure(&postings, memory.package.terminal_count(), 3);
            let actual = projection
                .project_query(&postings, 3, dense.closure.beta_k, &dense.all)
                .unwrap();
            assert_eq!(
                actual.metrics.reconstruction_field_mismatches, 0,
                "{surface}"
            );
            assert_eq!(
                actual.metrics.activation_histogram_mismatches, 0,
                "{surface}"
            );
            assert_eq!(actual.metrics.kth_or_equality_mismatches, 0, "{surface}");
            assert_eq!(
                actual.metrics.retained_id_symmetric_difference, 0,
                "{surface}"
            );
            assert_eq!(
                actual.exact.closure.beta_k, dense.closure.beta_k,
                "{surface}"
            );
        }
    }

    #[test]
    fn empty_query_remains_one_exact_zero_cohort() {
        let memory = memory(&["form", "formal", "format", "farm"]);
        let projection = DecoderSubtreeCoverProjection::build_for_test(&memory).unwrap();
        let dense = vec![ForwardActivation::default(); 4];
        let actual = projection.project_query(&[], 3, 0, &dense).unwrap();
        assert_eq!(actual.metrics.symbolic_cohort_records, 1);
        assert_eq!(actual.metrics.symbolic_terminal_count, 4);
        assert_eq!(actual.metrics.symbolic_beta, 0);
        assert_eq!(actual.metrics.symbolic_retained_count, 4);
        assert_eq!(actual.metrics.retained_terminal_id_expansions, 4);
        assert_eq!(actual.metrics.projected_sparse_work_units, 6);
        assert_eq!(actual.metrics.kth_or_equality_mismatches, 0);
    }

    #[test]
    fn retained_equality_materialization_is_part_of_the_projection_gate() {
        let summary = summarize_queries(&[DecoderSubtreeQueryMetrics {
            retained_terminal_id_expansions: SUBTREE_PROJECTION_WORK_LIMIT + 1,
            projected_sparse_work_units: SUBTREE_PROJECTION_WORK_LIMIT + 1,
            ..DecoderSubtreeQueryMetrics::default()
        }]);
        assert_eq!(summary["retained_id_expansion_gate"], false);
        assert_eq!(summary["combined_work_gate"], false);
    }
}
