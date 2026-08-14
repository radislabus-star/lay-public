//! Proof-only forward view of the package decoder trie.
//!
//! The package stores parent links for compact terminal-to-root decoding. This
//! module derives the opposite direction without changing package bytes. It is
//! compiled only for tests or the lexical compiler proof tool.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::time::Instant;

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::format;
use super::model::{DecoderNode, LexicalGrokkingPackage};
use super::runtime::LexicalGrokkingMemory;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ForwardChild {
    symbol: u32,
    node_id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ForwardDecoderIndex {
    child_offsets: Vec<u32>,
    children: Vec<ForwardChild>,
    terminal_offsets: Vec<u32>,
    terminals: Vec<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
struct RoundtripStats {
    primary_terminals: usize,
    roundtripped_terminals: usize,
    decoded_utf8_bytes: usize,
    maximum_surface_chars: usize,
    terminal_collision_nodes: usize,
    maximum_terminals_per_node: usize,
}

impl ForwardDecoderIndex {
    fn build(package: &LexicalGrokkingPackage) -> Result<Self, String> {
        validate_parent_topology(&package.decoder_nodes)?;

        let node_count = package.decoder_nodes.len();
        let mut child_counts = vec![0_u32; node_count];
        for node in package.decoder_nodes.iter().skip(1) {
            let parent = node.parent as usize;
            child_counts[parent] = child_counts[parent]
                .checked_add(1)
                .ok_or_else(|| "decoder child count exceeds u32".to_string())?;
        }
        let child_offsets = prefix_offsets(&child_counts, "decoder child")?;
        let child_total = child_offsets.last().copied().unwrap_or_default() as usize;
        let mut children = vec![ForwardChild::default(); child_total];
        let mut child_cursors = child_offsets[..node_count].to_vec();
        for (node_id, node) in package.decoder_nodes.iter().copied().enumerate().skip(1) {
            let parent = node.parent as usize;
            let cursor = child_cursors[parent] as usize;
            children[cursor] = ForwardChild {
                symbol: node.symbol,
                node_id: u32::try_from(node_id)
                    .map_err(|_| "decoder node ID exceeds u32".to_string())?,
            };
            child_cursors[parent] = child_cursors[parent]
                .checked_add(1)
                .ok_or_else(|| "decoder child cursor exceeds u32".to_string())?;
        }
        for parent in 0..node_count {
            let start = child_offsets[parent] as usize;
            let end = child_offsets[parent + 1] as usize;
            let range = &mut children[start..end];
            range.sort_unstable_by_key(|child| (child.symbol, child.node_id));
            if range
                .windows(2)
                .any(|pair| pair[0].symbol == pair[1].symbol)
            {
                return Err(format!(
                    "decoder parent {parent} has duplicate transition symbols"
                ));
            }
        }

        let mut terminal_counts = vec![0_u32; node_count];
        for center in &package.centers {
            let node = center.decoder_terminal as usize;
            let count = terminal_counts
                .get_mut(node)
                .ok_or_else(|| "primary center references an invalid decoder node".to_string())?;
            *count = count
                .checked_add(1)
                .ok_or_else(|| "decoder terminal count exceeds u32".to_string())?;
        }
        let terminal_offsets = prefix_offsets(&terminal_counts, "decoder terminal")?;
        let terminal_total = terminal_offsets.last().copied().unwrap_or_default() as usize;
        let mut terminals = vec![0_u32; terminal_total];
        let mut terminal_cursors = terminal_offsets[..node_count].to_vec();
        for (terminal_id, center) in package.centers.iter().enumerate() {
            let node = center.decoder_terminal as usize;
            let cursor = terminal_cursors[node] as usize;
            terminals[cursor] = u32::try_from(terminal_id)
                .map_err(|_| "primary WordCenterId exceeds u32".to_string())?;
            terminal_cursors[node] = terminal_cursors[node]
                .checked_add(1)
                .ok_or_else(|| "decoder terminal cursor exceeds u32".to_string())?;
        }
        for node in 0..node_count {
            let start = terminal_offsets[node] as usize;
            let end = terminal_offsets[node + 1] as usize;
            terminals[start..end].sort_unstable();
        }

        Ok(Self {
            child_offsets,
            children,
            terminal_offsets,
            terminals,
        })
    }

    fn child(&self, parent: u32, symbol: u32) -> Option<u32> {
        let parent = parent as usize;
        let start = *self.child_offsets.get(parent)? as usize;
        let end = *self.child_offsets.get(parent + 1)? as usize;
        let children = self.children.get(start..end)?;
        children
            .binary_search_by_key(&symbol, |child| child.symbol)
            .ok()
            .map(|index| children[index].node_id)
    }

    fn traverse(&self, surface: &str) -> Option<u32> {
        let mut node = 0_u32;
        for symbol in surface.chars().map(|character| character as u32) {
            node = self.child(node, symbol)?;
        }
        Some(node)
    }

    fn terminals(&self, node: u32) -> &[u32] {
        let node = node as usize;
        let Some(&start) = self.terminal_offsets.get(node) else {
            return &[];
        };
        let Some(&end) = self.terminal_offsets.get(node + 1) else {
            return &[];
        };
        self.terminals
            .get(start as usize..end as usize)
            .unwrap_or_default()
    }

    fn validate_roundtrip(
        &self,
        package: &LexicalGrokkingPackage,
    ) -> Result<RoundtripStats, String> {
        let mut stats = RoundtripStats {
            primary_terminals: package.centers.len(),
            ..RoundtripStats::default()
        };
        for (terminal_id, center) in package.centers.iter().copied().enumerate() {
            let surface = format::decode_center_surface(center, &package.decoder_nodes)?;
            let traversed = self
                .traverse(&surface)
                .ok_or_else(|| format!("forward decoder cannot traverse terminal {terminal_id}"))?;
            if traversed != center.decoder_terminal {
                return Err(format!(
                    "terminal {terminal_id} round-tripped to node {traversed}, expected {}",
                    center.decoder_terminal
                ));
            }
            let terminal_id = u32::try_from(terminal_id)
                .map_err(|_| "primary WordCenterId exceeds u32".to_string())?;
            if self
                .terminals(traversed)
                .binary_search(&terminal_id)
                .is_err()
            {
                return Err(format!(
                    "terminal {terminal_id} is absent from decoder node {traversed}"
                ));
            }
            stats.roundtripped_terminals += 1;
            stats.decoded_utf8_bytes = stats.decoded_utf8_bytes.saturating_add(surface.len());
            stats.maximum_surface_chars = stats.maximum_surface_chars.max(surface.chars().count());
        }
        for node in 0..package.decoder_nodes.len() {
            let count = self.terminals(node as u32).len();
            stats.terminal_collision_nodes += usize::from(count > 1);
            stats.maximum_terminals_per_node = stats.maximum_terminals_per_node.max(count);
        }
        Ok(stats)
    }

    fn resident_bytes(&self) -> usize {
        self.child_offsets
            .capacity()
            .saturating_mul(std::mem::size_of::<u32>())
            .saturating_add(
                self.children
                    .capacity()
                    .saturating_mul(std::mem::size_of::<ForwardChild>()),
            )
            .saturating_add(
                self.terminal_offsets
                    .capacity()
                    .saturating_mul(std::mem::size_of::<u32>()),
            )
            .saturating_add(
                self.terminals
                    .capacity()
                    .saturating_mul(std::mem::size_of::<u32>()),
            )
    }

    fn maximum_children_per_node(&self) -> usize {
        self.child_offsets
            .windows(2)
            .map(|range| range[1].saturating_sub(range[0]) as usize)
            .max()
            .unwrap_or_default()
    }

    fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hash_u32_slice(&mut hasher, &self.child_offsets);
        for child in &self.children {
            hasher.update(child.symbol.to_le_bytes());
            hasher.update(child.node_id.to_le_bytes());
        }
        hash_u32_slice(&mut hasher, &self.terminal_offsets);
        hash_u32_slice(&mut hasher, &self.terminals);
        format!("{:x}", hasher.finalize())
    }
}

pub fn prove_l1_forward_decoder_index(package_path: &Path) -> io::Result<serde_json::Value> {
    let package_sha256_before = file_sha256(package_path)?;
    let package_bytes = std::fs::metadata(package_path)?.len();
    let rss_before_load = current_rss_bytes().unwrap_or_default();
    let load_started = Instant::now();
    let memory = LexicalGrokkingMemory::load(package_path).map_err(io::Error::other)?;
    let memory_load_ms = load_started.elapsed().as_millis();
    let rss_after_load = current_rss_bytes().unwrap_or_default();

    let build_started = Instant::now();
    let index = ForwardDecoderIndex::build(&memory.package).map_err(io::Error::other)?;
    let roundtrip = index
        .validate_roundtrip(&memory.package)
        .map_err(io::Error::other)?;
    let index_build_and_validation_ms = build_started.elapsed().as_millis();
    let rss_after_index = current_rss_bytes().unwrap_or_default();
    let package_sha256_after = file_sha256(package_path)?;
    if package_sha256_before != package_sha256_after {
        return Err(io::Error::other(
            "package bytes changed while building the proof-only decoder index",
        ));
    }

    Ok(serde_json::json!({
        "schema": "lay.l11.forward-decoder-index-proof.v1",
        "package": package_path.display().to_string(),
        "package_bytes": package_bytes,
        "package_sha256_before": package_sha256_before,
        "package_sha256_after": package_sha256_after,
        "package_bytes_unchanged": true,
        "decoder_nodes": memory.package.decoder_nodes.len(),
        "forward_edges": index.children.len(),
        "root_nodes": 1,
        "parent_bounds_valid": true,
        "acyclic": true,
        "transition_symbols_unique": true,
        "children_sorted_by_symbol_then_node_id": true,
        "primary_centers_only": true,
        "primary_terminals": roundtrip.primary_terminals,
        "roundtripped_terminals": roundtrip.roundtripped_terminals,
        "decoded_utf8_bytes": roundtrip.decoded_utf8_bytes,
        "maximum_surface_chars": roundtrip.maximum_surface_chars,
        "terminal_collision_nodes": roundtrip.terminal_collision_nodes,
        "maximum_terminals_per_node": roundtrip.maximum_terminals_per_node,
        "maximum_children_per_node": index.maximum_children_per_node(),
        "index_resident_bytes": index.resident_bytes(),
        "index_fingerprint_sha256": index.fingerprint(),
        "memory_load_ms": memory_load_ms,
        "index_build_and_validation_ms": index_build_and_validation_ms,
        "rss_before_load_bytes": rss_before_load,
        "rss_after_load_bytes": rss_after_load,
        "rss_after_index_bytes": rss_after_index,
        "index_incremental_rss_bytes": rss_after_index.saturating_sub(rss_after_load),
        "runtime_authority_changed": false,
        "package_format_changed": false
    }))
}

fn validate_parent_topology(nodes: &[DecoderNode]) -> Result<(), String> {
    let Some(root) = nodes.first() else {
        return Err("decoder trie requires a root node".to_string());
    };
    if root.parent != u32::MAX || root.symbol != 0 {
        return Err("decoder root record is invalid".to_string());
    }
    for (node_id, node) in nodes.iter().enumerate().skip(1) {
        if node.parent as usize >= nodes.len() {
            return Err(format!("decoder node {node_id} has an out-of-range parent"));
        }
        if node.parent as usize == node_id {
            return Err(format!("decoder node {node_id} is its own parent"));
        }
        if char::from_u32(node.symbol).is_none() {
            return Err(format!(
                "decoder node {node_id} has an invalid Unicode symbol"
            ));
        }
    }

    let mut states = vec![0_u8; nodes.len()];
    states[0] = 2;
    for start in 1..nodes.len() {
        if states[start] == 2 {
            continue;
        }
        let mut path = Vec::new();
        let mut node = start;
        loop {
            match states[node] {
                0 => {
                    states[node] = 1;
                    path.push(node);
                    node = nodes[node].parent as usize;
                }
                1 => return Err(format!("decoder parent cycle reaches node {node}")),
                2 => break,
                _ => unreachable!(),
            }
        }
        for node in path {
            states[node] = 2;
        }
    }
    Ok(())
}

fn prefix_offsets(counts: &[u32], name: &str) -> Result<Vec<u32>, String> {
    let mut offsets = Vec::with_capacity(counts.len().saturating_add(1));
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

fn hash_u32_slice(hasher: &mut Sha256, values: &[u32]) {
    hasher.update((values.len() as u64).to_le_bytes());
    for value in values {
        hasher.update(value.to_le_bytes());
    }
}

fn file_sha256(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn current_rss_bytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let kib = status
        .lines()
        .find_map(|line| line.strip_prefix("VmRSS:"))?
        .split_whitespace()
        .next()?
        .parse::<u64>()
        .ok()?;
    Some(kib.saturating_mul(1024))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::lexical_grokking::crystal::WordCenter64;

    fn node(parent: u32, symbol: char) -> DecoderNode {
        DecoderNode {
            parent,
            symbol: symbol as u32,
        }
    }

    fn package_with(nodes: Vec<DecoderNode>, terminals: &[u32]) -> LexicalGrokkingPackage {
        LexicalGrokkingPackage {
            decoder_nodes: nodes,
            centers: terminals
                .iter()
                .copied()
                .map(|decoder_terminal| WordCenter64 {
                    decoder_terminal,
                    ..WordCenter64::default()
                })
                .collect(),
            ..LexicalGrokkingPackage::default()
        }
    }

    fn valid_nodes() -> Vec<DecoderNode> {
        vec![
            DecoderNode {
                parent: u32::MAX,
                symbol: 0,
            },
            node(0, 'a'),
            node(1, 'b'),
            node(0, 'b'),
        ]
    }

    #[test]
    fn forward_index_roundtrips_primary_centers_and_collisions() {
        let package = package_with(valid_nodes(), &[2, 3, 2]);
        let index = ForwardDecoderIndex::build(&package).unwrap();
        let stats = index.validate_roundtrip(&package).unwrap();

        assert_eq!(index.traverse("ab"), Some(2));
        assert_eq!(index.traverse("b"), Some(3));
        assert_eq!(index.terminals(2), &[0, 2]);
        assert_eq!(stats.roundtripped_terminals, 3);
        assert_eq!(stats.terminal_collision_nodes, 1);
        assert_eq!(stats.maximum_terminals_per_node, 2);
    }

    #[test]
    fn invalid_parent_topologies_are_rejected() {
        let mut invalid_root = valid_nodes();
        invalid_root[0].parent = 0;
        assert!(ForwardDecoderIndex::build(&package_with(invalid_root, &[])).is_err());

        let mut out_of_range = valid_nodes();
        out_of_range[1].parent = 99;
        assert!(ForwardDecoderIndex::build(&package_with(out_of_range, &[])).is_err());

        let mut cycle = valid_nodes();
        cycle[1].parent = 2;
        cycle[2].parent = 1;
        assert!(ForwardDecoderIndex::build(&package_with(cycle, &[])).is_err());
    }

    #[test]
    fn duplicate_transition_symbols_are_rejected() {
        let mut nodes = valid_nodes();
        nodes.push(node(0, 'a'));
        let error = ForwardDecoderIndex::build(&package_with(nodes, &[])).unwrap_err();
        assert!(error.contains("duplicate transition symbols"));
    }

    #[test]
    fn relation_center_decoder_fields_cannot_enter_terminal_lists() {
        let mut package = package_with(valid_nodes(), &[2]);
        package.anti_centers.push(WordCenter64 {
            decoder_terminal: u32::MAX,
            ..WordCenter64::default()
        });
        package.ambiguity_subcenters.push(WordCenter64 {
            decoder_terminal: u32::MAX,
            ..WordCenter64::default()
        });

        let index = ForwardDecoderIndex::build(&package).unwrap();
        assert_eq!(index.terminals.len(), 1);
        assert_eq!(index.terminals(2), &[0]);
    }

    #[test]
    fn index_fingerprint_is_deterministic() {
        let package = package_with(valid_nodes(), &[2, 3, 2]);
        let first = ForwardDecoderIndex::build(&package).unwrap();
        let second = ForwardDecoderIndex::build(&package).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.fingerprint(), second.fingerprint());
    }

    #[test]
    fn full_external_package_roundtrips_when_requested() {
        let Ok(path) = std::env::var("LAY_L11_FORWARD_INDEX_PACKAGE") else {
            return;
        };
        let report = prove_l1_forward_decoder_index(Path::new(&path)).unwrap();
        assert_eq!(report["package_bytes_unchanged"], true);
        assert_eq!(
            report["primary_terminals"],
            report["roundtripped_terminals"]
        );
        assert_eq!(report["parent_bounds_valid"], true);
        assert_eq!(report["acyclic"], true);
        assert_eq!(report["transition_symbols_unique"], true);
    }

    #[test]
    fn production_owners_do_not_import_forward_decoder_index() {
        for source in [
            include_str!("runtime.rs"),
            include_str!("service.rs"),
            include_str!("peak_search/mod.rs"),
        ] {
            assert!(!source.contains("forward_decoder_index"));
            assert!(!source.contains("ForwardDecoderIndex"));
        }
    }
}
