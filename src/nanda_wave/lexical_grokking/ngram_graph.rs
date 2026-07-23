//! Deterministic n-gram graph used instead of hash identity.

use std::collections::{BTreeMap, BTreeSet};

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

#[derive(Default)]
struct BuildNode {
    children: BTreeMap<u32, usize>,
    atom_id: Option<u32>,
}

impl NGramGraph {
    pub(super) fn compile(keys: impl IntoIterator<Item = NGramKey>) -> Result<Self, String> {
        let keys = keys.into_iter().collect::<BTreeSet<_>>();
        let mut build = vec![BuildNode::default()];
        for (atom_id, key) in keys.iter().copied().enumerate() {
            let atom_id = u32::try_from(atom_id)
                .map_err(|_| "n-gram graph atom count exceeds u32".to_string())?;
            let mut node = 0usize;
            for symbol in key_symbols(key) {
                let next = if let Some(next) = build[node].children.get(&symbol) {
                    *next
                } else {
                    let next = build.len();
                    build.push(BuildNode::default());
                    build[node].children.insert(symbol, next);
                    next
                };
                node = next;
            }
            build[node].atom_id = Some(atom_id);
        }
        let mut nodes = Vec::with_capacity(build.len());
        let mut arcs = Vec::new();
        for node in &build {
            let first_arc = u32::try_from(arcs.len())
                .map_err(|_| "n-gram graph arc count exceeds u32".to_string())?;
            let arc_count = u16::try_from(node.children.len())
                .map_err(|_| "n-gram graph node fanout exceeds u16".to_string())?;
            arcs.extend(node.children.iter().map(|(symbol, next_node)| NGramArc {
                symbol: *symbol,
                next_node: *next_node as u32,
            }));
            nodes.push(NGramNode {
                first_arc,
                arc_count,
                atom_id: node.atom_id.unwrap_or(NO_ATOM),
            });
        }
        Ok(Self {
            nodes,
            arcs,
            atom_count: keys.len() as u32,
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
