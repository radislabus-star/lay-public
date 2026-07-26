//! Deterministic n-gram graph used instead of hash identity.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use super::atoms::{AtomChannel, NGramKey};

const NO_ATOM: u32 = u32::MAX;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct NGramNode {
    pub(super) first_arc: u32,
    pub(super) arc_count: u16,
    pub(super) atom_id: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct NGramArc {
    pub(super) symbol: u32,
    pub(super) next_node: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct NGramGraph {
    pub(super) nodes: Vec<NGramNode>,
    pub(super) arcs: Vec<NGramArc>,
    pub(super) atom_count: u32,
}

impl NGramGraph {
    #[cfg(test)]
    pub(super) fn compile(keys: impl IntoIterator<Item = NGramKey>) -> Result<Self, String> {
        let keys = keys.into_iter().collect::<BTreeSet<_>>();
        Self::compile_sorted_unique(keys)
    }

    pub(super) fn compile_sorted_unique(keys: BTreeSet<NGramKey>) -> Result<Self, String> {
        let atom_count = u32::try_from(keys.len())
            .map_err(|_| "n-gram graph atom count exceeds u32".to_string())?;
        let mut entries = keys
            .into_iter()
            .enumerate()
            .map(|(atom_id, key)| (key, atom_id as u32))
            .collect::<Vec<_>>();
        entries.sort_unstable_by(|left, right| sequence_order(left.0, right.0));

        let mut nodes = Vec::new();
        let mut arcs = Vec::new();
        build_compact_node(&entries, 0, &mut nodes, &mut arcs)?;
        Ok(Self {
            nodes,
            arcs,
            atom_count,
        })
    }

    pub(super) fn atom_id(&self, key: NGramKey) -> Option<u32> {
        let mut node_index = 0usize;
        for symbol in key_symbols(key) {
            let node = self.nodes.get(node_index)?;
            let start = node.first_arc as usize;
            let end = start.checked_add(node.arc_count as usize)?;
            let arcs = self.arcs.get(start..end)?;
            let index = arcs.binary_search_by_key(&symbol, |arc| arc.symbol).ok()?;
            node_index = arcs[index].next_node as usize;
        }
        let atom_id = self.nodes.get(node_index)?.atom_id;
        (atom_id != NO_ATOM).then_some(atom_id)
    }
}

fn build_compact_node(
    entries: &[(NGramKey, u32)],
    depth: usize,
    nodes: &mut Vec<NGramNode>,
    arcs: &mut Vec<NGramArc>,
) -> Result<u32, String> {
    let node_id = u32::try_from(nodes.len())
        .map_err(|_| "n-gram graph node count exceeds u32".to_string())?;
    nodes.push(NGramNode::default());

    let terminal = entries
        .iter()
        .find_map(|(key, atom_id)| (sequence_len(*key) == depth).then_some(*atom_id))
        .unwrap_or(NO_ATOM);
    let mut first_child = 0;
    while first_child < entries.len() && sequence_len(entries[first_child].0) == depth {
        first_child += 1;
    }

    let mut group_count = 0_usize;
    let mut cursor = first_child;
    while cursor < entries.len() {
        let symbol = sequence_symbol(entries[cursor].0, depth);
        group_count += 1;
        cursor += 1;
        while cursor < entries.len() && sequence_symbol(entries[cursor].0, depth) == symbol {
            cursor += 1;
        }
    }
    let first_arc =
        u32::try_from(arcs.len()).map_err(|_| "n-gram graph arc count exceeds u32".to_string())?;
    let arc_count = u16::try_from(group_count)
        .map_err(|_| "n-gram graph node fanout exceeds u16".to_string())?;
    arcs.resize(arcs.len().saturating_add(group_count), NGramArc::default());
    nodes[node_id as usize] = NGramNode {
        first_arc,
        arc_count,
        atom_id: terminal,
    };

    let mut group = 0_usize;
    cursor = first_child;
    while cursor < entries.len() {
        let symbol = sequence_symbol(entries[cursor].0, depth);
        let start = cursor;
        cursor += 1;
        while cursor < entries.len() && sequence_symbol(entries[cursor].0, depth) == symbol {
            cursor += 1;
        }
        let next_node = build_compact_node(&entries[start..cursor], depth + 1, nodes, arcs)?;
        arcs[first_arc as usize + group] = NGramArc { symbol, next_node };
        group += 1;
    }
    Ok(node_id)
}

fn sequence_order(left: NGramKey, right: NGramKey) -> Ordering {
    key_symbols(left).cmp(key_symbols(right))
}

fn sequence_len(key: NGramKey) -> usize {
    1 + key.len as usize
}

fn sequence_symbol(key: NGramKey, depth: usize) -> u32 {
    if depth == 0 {
        0xff00_0000 | channel_id(key.channel)
    } else {
        key.units[depth - 1]
    }
}

fn key_symbols(key: NGramKey) -> impl Iterator<Item = u32> {
    let channel = 0xff00_0000 | channel_id(key.channel);
    std::iter::once(channel).chain(key.units.into_iter().take(key.len as usize))
}

fn channel_id(channel: AtomChannel) -> u32 {
    match channel {
        AtomChannel::ByteGram => 1,
        AtomChannel::CharacterGram => 2,
        AtomChannel::KeyboardGram => 3,
        AtomChannel::BoundaryPosition => 4,
        AtomChannel::CharacterBigram => 5,
        AtomChannel::KeyboardBigram => 6,
        AtomChannel::CharacterBagGram => 7,
        AtomChannel::KeyboardBagGram => 8,
        AtomChannel::CharacterSkipGram => 9,
        AtomChannel::KeyboardSkipGram => 10,
        AtomChannel::CharacterAnchor => 11,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::lexical_grokking::atoms::encode_wave_surface;

    #[test]
    fn graph_assigns_dense_deterministic_atom_ids() {
        let keys = encode_wave_surface("время")
            .into_iter()
            .map(|atom| atom.key)
            .collect::<Vec<_>>();
        let graph = NGramGraph::compile(keys.iter().copied()).expect("compile graph");
        let repeated = NGramGraph::compile(keys.iter().rev().copied()).expect("repeat graph");
        assert_eq!(graph, repeated);
        assert!(keys.iter().all(|key| graph.atom_id(*key).is_some()));
    }
}
