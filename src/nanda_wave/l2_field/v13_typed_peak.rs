//! Exact typed discovery over immutable canonical V13 identities.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};

use super::package_bytes::PackageBytes;
use super::productive_v1::{
    ExactPeakBirthEnumerationV1, ExactPeakCandidateInputV1, ExactSearchStructuralWorkV1,
    EXACT_RELATION_SEARCH_MAX_PRODUCT_STATES, EXACT_RELATION_SEARCH_MAX_SCRATCH_BYTES,
    EXACT_RELATION_SEARCH_MAX_TERMINALS,
};
use super::runtime::StandaloneL2Field;
use super::runtime_storage::RuntimeL2Package;
use crate::nanda_wave::lexical_grokking::{
    phase7d_semantics_digest, Phase7dCertificateOracle, Phase7dRetrievalLane,
};
use crate::typing_transition::target_evidence::IncompletenessReasonV1;

mod typed_exact;

const MAGIC: &[u8; 8] = b"LAYV13D3";
const VERSION: u32 = 3;
const HEADER_BYTES: usize = 256;
const STATE_BYTES: usize = 8;
const EDGE_BYTES: usize = 8;
const SYMBOL_BYTES: usize = 4;
const TERMINAL_FLAG: u16 = 1;
const PACKED_U24_MAX: u32 = (1 << 24) - 1;
const PACKED_U15_MAX: u16 = (1 << 15) - 1;
const NORMALIZATION_SEMANTICS_VERSION: u32 = 1;
const MAX_SIDECAR_BYTES: usize = 32 * 1024 * 1024;
const MAX_LOADER_METADATA_BYTES: usize = 4 * 1024 * 1024;
#[cfg(test)]
const MAX_QUERY_SCRATCH_BYTES: usize = 512 * 1024;
const MAX_QUERY_SYMBOLS: usize = 96;
const MAX_LEVENSHTEIN_RADIUS: u8 = 3;
const MAX_BAND_CELLS: usize = MAX_LEVENSHTEIN_RADIUS as usize * 2 + 1;
const DEAD_DLA_STATE: u16 = u16::MAX;
const DLA_HASH_BUCKET_BYTES: usize = 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct V13Identity {
    package_sha256: [u8; 32],
    package_bytes: u64,
    form_count: u32,
    binding_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BuilderEdge {
    symbol: u32,
    target: u32,
}

#[derive(Debug, Default)]
struct BuilderState {
    terminal: bool,
    edges: Vec<BuilderEdge>,
}

#[derive(Clone, Copy, Debug)]
struct UncheckedEdge {
    parent: u32,
    symbol: u32,
    child: u32,
}

struct MinimalDafsaBuilder {
    states: Vec<Option<BuilderState>>,
    free_states: Vec<u32>,
    register: HashMap<u64, Vec<u32>>,
    unchecked: Vec<UncheckedEdge>,
    previous: Vec<u32>,
    forms: u32,
}

impl MinimalDafsaBuilder {
    fn new() -> Self {
        Self {
            states: vec![Some(BuilderState::default())],
            free_states: Vec::new(),
            register: HashMap::new(),
            unchecked: Vec::new(),
            previous: Vec::new(),
            forms: 0,
        }
    }

    fn insert(&mut self, surface: &str) -> Result<(), String> {
        let symbols = surface.chars().map(|ch| ch as u32).collect::<Vec<_>>();
        if symbols.is_empty() {
            return Err("V13 DAFSA rejects an empty surface".to_string());
        }
        if !self.previous.is_empty() && self.previous.as_slice() >= symbols.as_slice() {
            return Err("V13 DAFSA input is not strictly increasing".to_string());
        }
        let common = self
            .previous
            .iter()
            .zip(&symbols)
            .take_while(|(left, right)| left == right)
            .count();
        self.minimize(common)?;

        let mut parent = if common == 0 {
            0
        } else {
            self.unchecked[common - 1].child
        };
        for symbol in symbols.iter().copied().skip(common) {
            let child = self.allocate_state();
            let state = self.state_mut(parent)?;
            if state.edges.last().is_some_and(|edge| edge.symbol >= symbol) {
                return Err("V13 DAFSA transition order is not deterministic".to_string());
            }
            state.edges.push(BuilderEdge {
                symbol,
                target: child,
            });
            self.unchecked.push(UncheckedEdge {
                parent,
                symbol,
                child,
            });
            parent = child;
        }
        self.state_mut(parent)?.terminal = true;
        self.previous = symbols;
        self.forms = self
            .forms
            .checked_add(1)
            .ok_or_else(|| "V13 DAFSA terminal count overflows u32".to_string())?;
        Ok(())
    }

    fn finish(mut self) -> Result<PackedDafsa, String> {
        self.minimize(0)?;
        let mut visited = vec![false; self.states.len()];
        let mut postorder = Vec::new();
        self.postorder(0, &mut visited, &mut postorder)?;
        let mut compact_ref = vec![u32::MAX; self.states.len()];
        for (compact, original) in postorder.iter().copied().enumerate() {
            compact_ref[original as usize] = u32::try_from(compact)
                .map_err(|_| "V13 DAFSA state count overflows u32".to_string())?;
        }

        let mut states = Vec::with_capacity(postorder.len());
        let mut edges = Vec::new();
        for original in postorder.iter().copied() {
            let source = self.state(original)?;
            let first_edge = u32::try_from(edges.len())
                .map_err(|_| "V13 DAFSA edge count overflows u32".to_string())?;
            for edge in &source.edges {
                let target = *compact_ref
                    .get(edge.target as usize)
                    .ok_or_else(|| "V13 DAFSA target is out of range".to_string())?;
                if target == u32::MAX {
                    return Err("V13 DAFSA contains an unreachable target".to_string());
                }
                edges.push(PackedEdge {
                    symbol: edge.symbol,
                    symbol_ref: 0,
                    target,
                    rank_delta: 0,
                });
            }
            states.push(PackedState {
                first_edge,
                suffix_count: 0,
                edge_count: u16::try_from(source.edges.len())
                    .map_err(|_| "V13 DAFSA state fanout exceeds u16".to_string())?,
                flags: if source.terminal { TERMINAL_FLAG } else { 0 },
            });
        }
        for state_id in 0..states.len() {
            let state = states[state_id];
            let mut suffix_count = u32::from(state.terminal());
            for edge in edge_slice(&edges, state)? {
                if edge.target as usize >= state_id {
                    return Err("V13 DAFSA compact order is not acyclic".to_string());
                }
                suffix_count = suffix_count
                    .checked_add(states[edge.target as usize].suffix_count)
                    .ok_or_else(|| "V13 DAFSA language count overflows u32".to_string())?;
            }
            states[state_id].suffix_count = suffix_count;
        }
        for state in &states {
            let mut rank_delta = u32::from(state.terminal());
            let start = state.first_edge as usize;
            let end = start
                .checked_add(state.edge_count as usize)
                .ok_or_else(|| "V13 DAFSA rank-delta range overflows usize".to_string())?;
            for edge in edges
                .get_mut(start..end)
                .ok_or_else(|| "V13 DAFSA rank-delta range is out of bounds".to_string())?
            {
                edge.rank_delta = rank_delta;
                rank_delta = rank_delta
                    .checked_add(states[edge.target as usize].suffix_count)
                    .ok_or_else(|| "V13 DAFSA rank delta overflows u32".to_string())?;
            }
            if rank_delta != state.suffix_count {
                return Err("V13 DAFSA rank delta does not cover the state language".to_string());
            }
        }
        let root_state = compact_ref[0];
        let root_count = states
            .get(root_state as usize)
            .ok_or_else(|| "V13 DAFSA root is missing".to_string())?
            .suffix_count;
        if root_count != self.forms {
            return Err(format!(
                "V13 DAFSA root language mismatch: {root_count} != {}",
                self.forms
            ));
        }
        Ok(PackedDafsa {
            states,
            edges,
            root_state,
            terminal_count: self.forms,
        })
    }

    fn allocate_state(&mut self) -> u32 {
        if let Some(state) = self.free_states.pop() {
            self.states[state as usize] = Some(BuilderState::default());
            state
        } else {
            let state = self.states.len() as u32;
            self.states.push(Some(BuilderState::default()));
            state
        }
    }

    fn minimize(&mut self, common_prefix: usize) -> Result<(), String> {
        while self.unchecked.len() > common_prefix {
            let edge = self.unchecked.pop().expect("checked length");
            let canonical = self.intern(edge.child)?;
            let parent = self.state_mut(edge.parent)?;
            let parent_edge = parent
                .edges
                .last_mut()
                .ok_or_else(|| "V13 DAFSA unchecked parent has no edge".to_string())?;
            if parent_edge.symbol != edge.symbol || parent_edge.target != edge.child {
                return Err("V13 DAFSA unchecked edge lost parent identity".to_string());
            }
            parent_edge.target = canonical;
        }
        Ok(())
    }

    fn intern(&mut self, state_id: u32) -> Result<u32, String> {
        let hash = self.state_hash(state_id)?;
        if let Some(existing) = self.register.get(&hash).and_then(|bucket| {
            bucket
                .iter()
                .copied()
                .find(|candidate| self.states_equal(*candidate, state_id))
        }) {
            self.states[state_id as usize] = None;
            self.free_states.push(state_id);
            return Ok(existing);
        }
        self.register.entry(hash).or_default().push(state_id);
        Ok(state_id)
    }

    fn state_hash(&self, state_id: u32) -> Result<u64, String> {
        let state = self.state(state_id)?;
        let mut hash = 0xcbf29ce484222325_u64 ^ u64::from(state.terminal);
        for edge in &state.edges {
            hash = (hash ^ u64::from(edge.symbol)).wrapping_mul(0x100000001b3);
            hash = (hash ^ u64::from(edge.target)).wrapping_mul(0x100000001b3);
        }
        Ok(hash)
    }

    fn states_equal(&self, left: u32, right: u32) -> bool {
        self.states
            .get(left as usize)
            .and_then(Option::as_ref)
            .zip(self.states.get(right as usize).and_then(Option::as_ref))
            .is_some_and(|(left, right)| {
                left.terminal == right.terminal && left.edges == right.edges
            })
    }

    fn postorder(
        &self,
        state_id: u32,
        visited: &mut [bool],
        output: &mut Vec<u32>,
    ) -> Result<(), String> {
        let marker = visited
            .get_mut(state_id as usize)
            .ok_or_else(|| "V13 DAFSA state is out of range".to_string())?;
        if *marker {
            return Ok(());
        }
        *marker = true;
        for edge in &self.state(state_id)?.edges {
            self.postorder(edge.target, visited, output)?;
        }
        output.push(state_id);
        Ok(())
    }

    fn state(&self, state_id: u32) -> Result<&BuilderState, String> {
        self.states
            .get(state_id as usize)
            .and_then(Option::as_ref)
            .ok_or_else(|| format!("V13 DAFSA state {state_id} is missing"))
    }

    fn state_mut(&mut self, state_id: u32) -> Result<&mut BuilderState, String> {
        self.states
            .get_mut(state_id as usize)
            .and_then(Option::as_mut)
            .ok_or_else(|| format!("V13 DAFSA state {state_id} is missing"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PackedState {
    first_edge: u32,
    suffix_count: u32,
    edge_count: u16,
    flags: u16,
}

impl PackedState {
    fn terminal(self) -> bool {
        self.flags & TERMINAL_FLAG != 0
    }

    fn encode(self) -> Result<u64, String> {
        if self.first_edge > PACKED_U24_MAX {
            return Err("V13 DAFSA first edge exceeds packed u24".to_string());
        }
        if self.suffix_count > PACKED_U24_MAX {
            return Err("V13 DAFSA suffix count exceeds packed u24".to_string());
        }
        if self.edge_count > PACKED_U15_MAX {
            return Err("V13 DAFSA edge count exceeds packed u15".to_string());
        }
        if self.flags & !TERMINAL_FLAG != 0 {
            return Err("V13 DAFSA state has unsupported packed flags".to_string());
        }
        Ok(u64::from(self.first_edge)
            | (u64::from(self.suffix_count) << 24)
            | (u64::from(self.edge_count) << 48)
            | ((self.terminal() as u64) << 63))
    }

    fn decode(word: u64) -> Self {
        Self {
            first_edge: (word & u64::from(PACKED_U24_MAX)) as u32,
            suffix_count: ((word >> 24) & u64::from(PACKED_U24_MAX)) as u32,
            edge_count: ((word >> 48) & u64::from(PACKED_U15_MAX)) as u16,
            flags: if word >> 63 == 0 { 0 } else { TERMINAL_FLAG },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PackedEdge {
    symbol: u32,
    symbol_ref: u16,
    target: u32,
    rank_delta: u32,
}

impl PackedEdge {
    fn encode(self, symbol_ref: u16) -> Result<u64, String> {
        if self.target > PACKED_U24_MAX {
            return Err("V13 DAFSA target exceeds packed u24".to_string());
        }
        if self.rank_delta > PACKED_U24_MAX {
            return Err("V13 DAFSA rank delta exceeds packed u24".to_string());
        }
        Ok(u64::from(symbol_ref)
            | (u64::from(self.target) << 16)
            | (u64::from(self.rank_delta) << 40))
    }

    fn decode(word: u64, symbol: u32) -> Self {
        Self {
            symbol,
            symbol_ref: (word & u64::from(u16::MAX)) as u16,
            target: ((word >> 16) & u64::from(PACKED_U24_MAX)) as u32,
            rank_delta: ((word >> 40) & u64::from(PACKED_U24_MAX)) as u32,
        }
    }
}

struct PackedDafsa {
    states: Vec<PackedState>,
    edges: Vec<PackedEdge>,
    root_state: u32,
    terminal_count: u32,
}

impl PackedDafsa {
    fn symbols(&self) -> Vec<u32> {
        self.edges
            .iter()
            .map(|edge| edge.symbol)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn symbol_digest(symbols: &[u32]) -> [u8; 32] {
        let mut digest = Sha256::new();
        for symbol in symbols {
            digest.update(symbol.to_le_bytes());
        }
        digest.finalize().into()
    }
}

fn edge_slice(edges: &[PackedEdge], state: PackedState) -> Result<&[PackedEdge], String> {
    let start = state.first_edge as usize;
    let end = start
        .checked_add(state.edge_count as usize)
        .ok_or_else(|| "V13 DAFSA edge range overflows usize".to_string())?;
    edges
        .get(start..end)
        .ok_or_else(|| "V13 DAFSA edge range is out of bounds".to_string())
}

fn compile_sidecar(package: &RuntimeL2Package, identity: V13Identity) -> Result<Vec<u8>, String> {
    if package.form_count() != identity.form_count as usize
        || package.binding_count() != identity.binding_count as usize
    {
        return Err("V13 identity counts do not match the canonical package".to_string());
    }
    let mut builder = MinimalDafsaBuilder::new();
    for form_ref in 0..package.form_count() {
        let surface = package
            .surface(form_ref)
            .ok_or_else(|| format!("V13 surface {form_ref} cannot be decoded"))?;
        builder.insert(surface.as_ref())?;
    }
    encode_sidecar(builder.finish()?, identity)
}

fn compile_test_sidecar(surfaces: &[String]) -> Result<(Vec<u8>, V13Identity), String> {
    let mut builder = MinimalDafsaBuilder::new();
    for surface in surfaces {
        builder.insert(surface)?;
    }
    let identity = V13Identity {
        package_sha256: Sha256::digest(
            surfaces
                .iter()
                .flat_map(|surface| surface.bytes().chain(std::iter::once(0)))
                .collect::<Vec<_>>(),
        )
        .into(),
        package_bytes: surfaces.iter().map(String::len).sum::<usize>() as u64,
        form_count: surfaces.len() as u32,
        binding_count: 0,
    };
    Ok((encode_sidecar(builder.finish()?, identity)?, identity))
}

fn encode_sidecar(dafsa: PackedDafsa, identity: V13Identity) -> Result<Vec<u8>, String> {
    let symbols = dafsa.symbols();
    if symbols.len() > u16::MAX as usize {
        return Err("V13 DAFSA symbol table exceeds packed u16".to_string());
    }
    let mut payload = Vec::with_capacity(
        dafsa.states.len() * STATE_BYTES
            + dafsa.edges.len() * EDGE_BYTES
            + symbols.len() * SYMBOL_BYTES,
    );
    for state in &dafsa.states {
        payload.extend_from_slice(&state.encode()?.to_le_bytes());
    }
    for edge in &dafsa.edges {
        let symbol_ref = symbols
            .binary_search(&edge.symbol)
            .map_err(|_| "V13 DAFSA edge symbol is missing from its table".to_string())?;
        payload.extend_from_slice(
            &edge
                .encode(
                    u16::try_from(symbol_ref)
                        .map_err(|_| "V13 DAFSA symbol reference exceeds u16".to_string())?,
                )?
                .to_le_bytes(),
        );
    }
    for symbol in &symbols {
        payload.extend_from_slice(&symbol.to_le_bytes());
    }
    let total_bytes = HEADER_BYTES
        .checked_add(payload.len())
        .ok_or_else(|| "V13 DAFSA sidecar size overflows usize".to_string())?;
    let mut bytes = vec![0_u8; HEADER_BYTES];
    bytes[..8].copy_from_slice(MAGIC);
    put_u32(&mut bytes, 8, VERSION);
    put_u32(&mut bytes, 12, HEADER_BYTES as u32);
    put_u64(&mut bytes, 16, total_bytes as u64);
    bytes[24..56].copy_from_slice(&<[u8; 32]>::from(Sha256::digest(&payload)));
    bytes[56..88].copy_from_slice(&identity.package_sha256);
    put_u64(&mut bytes, 88, identity.package_bytes);
    put_u32(&mut bytes, 96, identity.form_count);
    put_u32(&mut bytes, 100, identity.binding_count);
    put_u32(&mut bytes, 104, NORMALIZATION_SEMANTICS_VERSION);
    bytes[112..144].copy_from_slice(&phase7d_semantics_digest());
    bytes[144..176].copy_from_slice(&PackedDafsa::symbol_digest(&symbols));
    put_u32(
        &mut bytes,
        176,
        u32::try_from(dafsa.states.len())
            .map_err(|_| "V13 DAFSA state count overflows u32".to_string())?,
    );
    put_u32(
        &mut bytes,
        180,
        u32::try_from(dafsa.edges.len())
            .map_err(|_| "V13 DAFSA edge count overflows u32".to_string())?,
    );
    put_u32(&mut bytes, 184, dafsa.terminal_count);
    put_u32(&mut bytes, 188, dafsa.root_state);
    put_u32(&mut bytes, 192, STATE_BYTES as u32);
    put_u32(&mut bytes, 196, EDGE_BYTES as u32);
    put_u32(
        &mut bytes,
        200,
        u32::try_from(symbols.len())
            .map_err(|_| "V13 DAFSA symbol count overflows u32".to_string())?,
    );
    put_u32(&mut bytes, 204, SYMBOL_BYTES as u32);
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}

#[derive(Clone, Debug)]
struct V13DafsaView {
    bytes: PackageBytes,
    identity: V13Identity,
    state_count: u32,
    edge_count: u32,
    symbol_count: u32,
    root_state: u32,
    symbol_digest: [u8; 32],
}

impl V13DafsaView {
    fn load(path: &Path, identity: V13Identity) -> Result<Self, String> {
        Self::from_backing(PackageBytes::load(path)?, identity)
    }

    fn from_bytes(bytes: Vec<u8>, identity: V13Identity) -> Result<Self, String> {
        Self::from_backing(PackageBytes::from_vec(bytes), identity)
    }

    fn from_backing(bytes: PackageBytes, identity: V13Identity) -> Result<Self, String> {
        let data = bytes.as_slice();
        if data.len() < HEADER_BYTES || data.get(..8) != Some(MAGIC) {
            return Err("invalid V13 DAFSA magic or truncated header".to_string());
        }
        if read_u32(data, 8)? != VERSION
            || read_u32(data, 12)? as usize != HEADER_BYTES
            || read_u64(data, 16)? as usize != data.len()
        {
            return Err("invalid V13 DAFSA version or size".to_string());
        }
        if data[24..56] != <[u8; 32]>::from(Sha256::digest(&data[HEADER_BYTES..])) {
            return Err("V13 DAFSA payload checksum mismatch".to_string());
        }
        if data[56..88] != identity.package_sha256
            || read_u64(data, 88)? != identity.package_bytes
            || read_u32(data, 96)? != identity.form_count
            || read_u32(data, 100)? != identity.binding_count
        {
            return Err("V13 DAFSA canonical package identity mismatch".to_string());
        }
        if read_u32(data, 104)? != NORMALIZATION_SEMANTICS_VERSION
            || data[112..144] != phase7d_semantics_digest()
        {
            return Err("V13 DAFSA typed semantics mismatch".to_string());
        }
        if read_u32(data, 192)? as usize != STATE_BYTES
            || read_u32(data, 196)? as usize != EDGE_BYTES
            || read_u32(data, 204)? as usize != SYMBOL_BYTES
        {
            return Err("V13 DAFSA record widths mismatch".to_string());
        }
        let state_count = read_u32(data, 176)?;
        let edge_count = read_u32(data, 180)?;
        let terminal_count = read_u32(data, 184)?;
        let root_state = read_u32(data, 188)?;
        let symbol_count = read_u32(data, 200)?;
        if symbol_count == 0 || symbol_count > u16::MAX as u32 {
            return Err("V13 DAFSA symbol count is outside packed bounds".to_string());
        }
        let symbol_digest = data[144..176].try_into().expect("fixed digest width");
        let expected_bytes = HEADER_BYTES
            .checked_add(state_count as usize * STATE_BYTES)
            .and_then(|value| value.checked_add(edge_count as usize * EDGE_BYTES))
            .and_then(|value| value.checked_add(symbol_count as usize * SYMBOL_BYTES))
            .ok_or_else(|| "V13 DAFSA section size overflows usize".to_string())?;
        if expected_bytes != data.len() || root_state >= state_count {
            return Err("V13 DAFSA section bounds mismatch".to_string());
        }
        let mut view = Self {
            bytes,
            identity,
            state_count,
            edge_count,
            symbol_count,
            root_state,
            symbol_digest,
        };
        view.validate(terminal_count)?;
        Ok(view)
    }

    fn validate(&mut self, terminal_count: u32) -> Result<(), String> {
        let symbol_table = self.symbols()?;
        if symbol_table.windows(2).any(|pair| pair[0] >= pair[1])
            || symbol_table
                .iter()
                .any(|symbol| char::from_u32(*symbol).is_none())
        {
            return Err("V13 DAFSA Unicode symbol table is not strictly increasing".to_string());
        }
        if PackedDafsa::symbol_digest(&symbol_table) != self.symbol_digest {
            return Err("V13 DAFSA dense-symbol digest mismatch".to_string());
        }
        let mut symbols = BTreeSet::new();
        for state_id in 0..self.state_count {
            let state = self.state(state_id)?;
            let mut previous = None;
            let mut expected_count = u32::from(state.terminal());
            let mut expected_rank_delta = u32::from(state.terminal());
            for edge_id in self.edge_range(state)? {
                let edge = self.edge(edge_id)?;
                if previous.is_some_and(|symbol| symbol >= edge.symbol) {
                    return Err("V13 DAFSA has nondeterministic edge order".to_string());
                }
                if edge.target >= state_id {
                    return Err("V13 DAFSA edge violates acyclic compact order".to_string());
                }
                if edge.rank_delta != expected_rank_delta {
                    return Err(format!("V13 DAFSA edge {edge_id} rank delta mismatch"));
                }
                let child_count = self.state(edge.target)?.suffix_count;
                expected_count = expected_count
                    .checked_add(child_count)
                    .ok_or_else(|| "V13 DAFSA suffix count overflows u32".to_string())?;
                expected_rank_delta = expected_rank_delta
                    .checked_add(child_count)
                    .ok_or_else(|| "V13 DAFSA rank delta overflows u32".to_string())?;
                previous = Some(edge.symbol);
                symbols.insert(edge.symbol);
            }
            if expected_count != state.suffix_count {
                return Err(format!("V13 DAFSA state {state_id} suffix count mismatch"));
            }
        }
        if self.state(self.root_state)?.suffix_count != terminal_count
            || terminal_count != self.identity.form_count
        {
            return Err("V13 DAFSA root language count mismatch".to_string());
        }
        if symbols.len() != symbol_table.len()
            || !symbols.iter().copied().eq(symbol_table.iter().copied())
        {
            return Err("V13 DAFSA symbol table does not match edge symbols".to_string());
        }
        Ok(())
    }

    fn state(&self, state_id: u32) -> Result<PackedState, String> {
        if state_id >= self.state_count {
            return Err("V13 DAFSA state reference is out of range".to_string());
        }
        let start = HEADER_BYTES + state_id as usize * STATE_BYTES;
        Ok(PackedState::decode(read_u64(self.bytes.as_slice(), start)?))
    }

    fn edge_range(&self, state: PackedState) -> Result<std::ops::Range<usize>, String> {
        let first = state.first_edge as usize;
        let end = first
            .checked_add(state.edge_count as usize)
            .ok_or_else(|| "V13 DAFSA edge range overflows usize".to_string())?;
        if end > self.edge_count as usize {
            return Err("V13 DAFSA edge range is out of bounds".to_string());
        }
        Ok(first..end)
    }

    fn edge(&self, edge_id: usize) -> Result<PackedEdge, String> {
        let mut edge = self.packed_edge(edge_id)?;
        edge.symbol = self.symbol(edge.symbol_ref)?;
        Ok(edge)
    }

    fn packed_edge(&self, edge_id: usize) -> Result<PackedEdge, String> {
        if edge_id >= self.edge_count as usize {
            return Err("V13 DAFSA edge reference is out of range".to_string());
        }
        let start = HEADER_BYTES + self.state_count as usize * STATE_BYTES + edge_id * EDGE_BYTES;
        let word = read_u64(self.bytes.as_slice(), start)?;
        Ok(PackedEdge::decode(word, 0))
    }

    fn symbol(&self, symbol_ref: u16) -> Result<u32, String> {
        if u32::from(symbol_ref) >= self.symbol_count {
            return Err("V13 DAFSA edge symbol reference is out of range".to_string());
        }
        let start = HEADER_BYTES
            + self.state_count as usize * STATE_BYTES
            + self.edge_count as usize * EDGE_BYTES
            + symbol_ref as usize * SYMBOL_BYTES;
        read_u32(self.bytes.as_slice(), start)
    }

    fn symbols(&self) -> Result<Vec<u32>, String> {
        (0..self.symbol_count)
            .map(|symbol_ref| {
                self.symbol(
                    u16::try_from(symbol_ref)
                        .map_err(|_| "V13 DAFSA symbol reference exceeds u16".to_string())?,
                )
            })
            .collect()
    }

    fn exact_form_ref(&self, symbols: &[u32]) -> Result<Option<u32>, String> {
        let mut state_id = self.root_state;
        let mut rank = 0_u32;
        for symbol in symbols {
            let state = self.state(state_id)?;
            let mut selected = None;
            for edge_id in self.edge_range(state)? {
                let edge = self.edge(edge_id)?;
                if edge.symbol == *symbol {
                    selected = Some(edge);
                    break;
                }
                if edge.symbol > *symbol {
                    break;
                }
            }
            let Some(edge) = selected else {
                return Ok(None);
            };
            rank = rank
                .checked_add(edge.rank_delta)
                .ok_or_else(|| "V13 DAFSA rank overflows u32".to_string())?;
            state_id = edge.target;
        }
        Ok(self.state(state_id)?.terminal().then_some(rank))
    }

    fn sidecar_bytes(&self) -> usize {
        self.bytes.len()
    }

    fn mmap_backed(&self) -> bool {
        self.bytes.is_mapped()
    }

    fn owned_metadata_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

#[derive(Clone, Copy, Debug)]
struct SearchBudget {
    maximum_product_states: usize,
    maximum_terminals: usize,
    maximum_scratch_bytes: usize,
    maximum_elapsed: Option<Duration>,
}

impl SearchBudget {
    fn proof() -> Self {
        Self {
            maximum_product_states: EXACT_RELATION_SEARCH_MAX_PRODUCT_STATES,
            maximum_terminals: EXACT_RELATION_SEARCH_MAX_TERMINALS,
            maximum_scratch_bytes: EXACT_RELATION_SEARCH_MAX_SCRATCH_BYTES,
            maximum_elapsed: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SearchCompleteness {
    CertifiedExhaustive,
    Unresolved(&'static str),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct V13TypedPeak {
    form_ref: u32,
    certificate_keys: Vec<String>,
}

#[derive(Clone, Debug)]
struct SearchResult {
    retrieved_form_refs: Vec<u32>,
    peaks: Vec<V13TypedPeak>,
    completeness: SearchCompleteness,
    expanded_product_states: usize,
    maximum_scratch_bytes: usize,
    dla_states: usize,
    dla_transitions: usize,
    maximum_dla_classes: usize,
    dla_build_elapsed_us: u64,
    intersection_elapsed_us: u64,
    search_elapsed_us: u64,
    material_elapsed_us: u64,
    total_elapsed_us: u64,
}

pub(super) struct ExactV13Generation {
    package_sha256: [u8; 32],
    sidecar_sha256: [u8; 32],
    sidecar_bytes: usize,
    materialized: typed_exact::TypedMaterialization,
}

impl ExactV13Generation {
    pub(super) fn load(
        sidecar_path: &Path,
        package_sha256: [u8; 32],
        package_bytes: u64,
        form_count: usize,
        binding_count: usize,
    ) -> Result<Self, String> {
        let identity = V13Identity {
            package_sha256,
            package_bytes,
            form_count: u32::try_from(form_count)
                .map_err(|_| "V13 form count exceeds u32".to_string())?,
            binding_count: u32::try_from(binding_count)
                .map_err(|_| "V13 binding count exceeds u32".to_string())?,
        };
        let sidecar_bytes = usize::try_from(
            std::fs::metadata(sidecar_path)
                .map_err(|error| format!("{}: {error}", sidecar_path.display()))?
                .len(),
        )
        .map_err(|_| "V13 sidecar size exceeds usize".to_string())?;
        let sidecar_sha256 = super::sha256_file(sidecar_path)?;
        let index = V13DafsaView::load(sidecar_path, identity)?;
        if index.sidecar_bytes() != sidecar_bytes {
            return Err("V13 sidecar changed while loading".to_string());
        }
        let materialized = typed_exact::TypedMaterialization::from_validated(&index)?;
        Ok(Self {
            package_sha256,
            sidecar_sha256,
            sidecar_bytes,
            materialized,
        })
    }

    pub(super) const fn sidecar_sha256(&self) -> [u8; 32] {
        self.sidecar_sha256
    }

    pub(super) const fn sidecar_bytes(&self) -> usize {
        self.sidecar_bytes
    }

    pub(super) fn typed_payload_bytes(&self) -> usize {
        self.materialized.payload_bytes()
    }

    pub(super) fn exact_peaks(
        &self,
        canonical_index: &StandaloneL2Field,
        observed: &str,
    ) -> Result<ExactPeakBirthEnumerationV1, String> {
        let oracle = Phase7dCertificateOracle::new(observed)?;
        let query_lanes = oracle.retrieval_lanes();
        let exact = typed_exact::search(
            self.materialized.view(),
            &query_lanes,
            SearchBudget::proof(),
            false,
            false,
            || false,
        )?;
        if exact.unresolved.is_some() {
            return Ok(ExactPeakBirthEnumerationV1::incomplete(
                IncompletenessReasonV1::WorkBudgetExceeded,
            ));
        }

        let retrieved_form_refs = exact.retrieved_form_refs.clone();
        let mut candidates = Vec::new();
        for form_ref in &retrieved_form_refs {
            let surface = canonical_index
                .decode_form_ref(*form_ref)
                .ok_or_else(|| format!("V13 terminal rank {form_ref} cannot be decoded"))?;
            let certificates = oracle.certificate_evidence(surface.as_ref())?;
            if certificates.is_empty() {
                continue;
            }
            candidates.push(ExactPeakCandidateInputV1 {
                form_ref: *form_ref,
                normalized_surface: super::compositional::normalize_surface(surface.as_ref()),
                certificates,
            });
        }
        let to_u64 = |value: usize, label: &str| {
            u64::try_from(value).map_err(|_| format!("exact search {label} exceeds u64"))
        };
        let work = ExactSearchStructuralWorkV1 {
            lane_count: to_u64(exact.lane_count, "lane count")?,
            maximum_lane_product_states: to_u64(
                exact.maximum_lane_product_states,
                "maximum lane product states",
            )?,
            maximum_lane_terminals: to_u64(exact.maximum_lane_terminals, "maximum lane terminals")?,
            expanded_product_states: to_u64(
                exact.expanded_product_states,
                "expanded product states",
            )?,
            maximum_scratch_bytes: to_u64(exact.maximum_scratch_bytes, "scratch bytes")?,
            expanded_states: to_u64(exact.work.expanded_states, "expanded states")?,
            examined_edges: to_u64(exact.work.examined_edges, "examined edges")?,
            surviving_edges: to_u64(exact.work.surviving_edges, "surviving edges")?,
            pruned_edges: to_u64(exact.work.pruned_edges, "pruned edges")?,
            stack_pushes: to_u64(exact.work.stack_pushes, "stack pushes")?,
            stack_pops: to_u64(exact.work.stack_pops, "stack pops")?,
            terminal_hits: to_u64(exact.work.terminal_hits, "terminal hits")?,
            transition_checks: to_u64(exact.transition_checks, "transition checks")?,
            terminal_distance_checks: to_u64(
                exact.terminal_distance_checks,
                "terminal distance checks",
            )?,
            rank_prefix_count: to_u64(exact.rank_prefixes.len(), "rank prefixes")?,
            terminal_rank_count: to_u64(exact.terminal_ranks.len(), "terminal ranks")?,
        };
        ExactPeakBirthEnumerationV1::from_candidates(candidates)?.with_completed_search_proof(
            observed,
            self.package_sha256,
            self.sidecar_sha256,
            &query_lanes,
            retrieved_form_refs,
            work,
            &exact.rank_prefixes,
            &exact.terminal_ranks,
        )
    }
}

pub(super) fn compile_exact_sidecar_file(
    package_path: &Path,
    output_path: &Path,
) -> std::io::Result<serde_json::Value> {
    let started = Instant::now();
    let package_bytes = std::fs::metadata(package_path)?.len();
    let package_sha256 = super::sha256_file(package_path).map_err(std::io::Error::other)?;
    let package = RuntimeL2Package::load(package_path).map_err(std::io::Error::other)?;
    let identity = V13Identity {
        package_sha256,
        package_bytes,
        form_count: u32::try_from(package.form_count()).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "V13 form count exceeds u32",
            )
        })?,
        binding_count: u32::try_from(package.binding_count()).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "V13 binding count exceeds u32",
            )
        })?,
    };
    let sidecar = compile_sidecar(&package, identity).map_err(std::io::Error::other)?;
    let sidecar_sha256: [u8; 32] = Sha256::digest(&sidecar).into();
    let index =
        V13DafsaView::from_bytes(sidecar.clone(), identity).map_err(std::io::Error::other)?;
    let materialized =
        typed_exact::TypedMaterialization::from_validated(&index).map_err(std::io::Error::other)?;
    super::write_atomic(output_path, &sidecar)?;
    Ok(serde_json::json!({
        "kind": "canonical_l2_v13_exact_sidecar",
        "verdict": "PASS_exact_typed_roundtrip",
        "canonical_l2_package": package_path,
        "canonical_l2_package_bytes": package_bytes,
        "canonical_l2_package_sha256": hex_digest(package_sha256),
        "output": output_path,
        "output_bytes": sidecar.len(),
        "output_sha256": hex_digest(sidecar_sha256),
        "forms": identity.form_count,
        "bindings": identity.binding_count,
        "states": index.state_count,
        "edges": index.edge_count,
        "symbols": index.symbol_count,
        "typed_payload_bytes": materialized.payload_bytes(),
        "elapsed_us": elapsed_us(started),
        "runtime_authority_changed": false,
    }))
}

pub(super) fn query_exact_sidecar_file(
    package_path: &Path,
    sidecar_path: &Path,
    observed: &str,
) -> std::io::Result<serde_json::Value> {
    let started = Instant::now();
    let package_bytes = std::fs::metadata(package_path)?.len();
    let package_sha256 = super::sha256_file(package_path).map_err(std::io::Error::other)?;
    let canonical_index = StandaloneL2Field::load(package_path).map_err(std::io::Error::other)?;
    let (form_count, _, _, binding_count, _, _) = canonical_index.package_counts();
    let generation = ExactV13Generation::load(
        sidecar_path,
        package_sha256,
        package_bytes,
        form_count,
        binding_count,
    )
    .map_err(std::io::Error::other)?;
    let exact = generation
        .exact_peaks(&canonical_index, observed)
        .map_err(std::io::Error::other)?;

    Ok(serde_json::json!({
        "kind": "canonical_l2_v13_exact_sidecar_query",
        "verdict": "PASS_read_only_exact_query",
        "observed": observed,
        "canonical_l2_package": package_path,
        "canonical_l2_package_bytes": package_bytes,
        "canonical_l2_package_sha256": hex_digest(package_sha256),
        "sidecar": sidecar_path,
        "sidecar_bytes": generation.sidecar_bytes(),
        "sidecar_sha256": hex_digest(generation.sidecar_sha256()),
        "typed_payload_bytes": generation.typed_payload_bytes(),
        "exact": exact.diagnostic_json(),
        "elapsed_us": elapsed_us(started),
        "runtime_authority_changed": false,
    }))
}

fn hex_digest(value: [u8; 32]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Clone, Copy)]
enum SearchKernel {
    Dla,
    BandedOracle,
    FullRowOracle,
}

trait LevenshteinRow: Sized {
    fn initial(query_len: usize, radius: u8) -> Self;
    fn advance(&self, query: &[u32], symbol: u32, radius: u8) -> Self;
    fn minimum(&self, radius: u8) -> u8;
    fn terminal_distance(&self, query_len: usize, radius: u8) -> u8;
}

struct SearchNode<Row> {
    state_id: u32,
    rank_prefix: u32,
    row: Row,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct BandedLevenshteinRow {
    cells: [u8; MAX_BAND_CELLS],
    depth: u8,
    start: u8,
    len: u8,
}

impl BandedLevenshteinRow {
    fn identity_key(self) -> u64 {
        let mut key =
            u64::from(self.depth) | (u64::from(self.start) << 8) | (u64::from(self.len) << 16);
        for (index, cell) in self.cells.iter().copied().enumerate() {
            key |= u64::from(cell) << (24 + index * 3);
        }
        key
    }
}

impl BandedLevenshteinRow {
    fn value(&self, column: usize, outside: u8) -> u8 {
        let start = self.start as usize;
        let end = start + self.len as usize;
        if (start..end).contains(&column) {
            self.cells[column - start]
        } else {
            outside
        }
    }
}

impl LevenshteinRow for BandedLevenshteinRow {
    fn initial(query_len: usize, radius: u8) -> Self {
        let mut cells = [radius.saturating_add(1); MAX_BAND_CELLS];
        let len = query_len.min(radius as usize) + 1;
        for (column, cell) in cells.iter_mut().enumerate().take(len) {
            *cell = column as u8;
        }
        Self {
            cells,
            depth: 0,
            start: 0,
            len: len as u8,
        }
    }

    fn advance(&self, query: &[u32], symbol: u32, radius: u8) -> Self {
        let outside = radius.saturating_add(1);
        let depth = self.depth.checked_add(1).expect("bounded query depth");
        let start = depth.saturating_sub(radius) as usize;
        let end = (depth as usize + radius as usize).min(query.len());
        let len = end.checked_sub(start).map_or(0, |width| width + 1);
        let mut cells = [outside; MAX_BAND_CELLS];
        for column in start..start + len {
            let value = if column == 0 {
                depth.min(outside)
            } else {
                let left = if column > start {
                    cells[column - 1 - start]
                } else {
                    outside
                }
                .saturating_add(1)
                .min(outside);
                let above = self.value(column, outside).saturating_add(1).min(outside);
                let diagonal = self
                    .value(column - 1, outside)
                    .saturating_add(u8::from(query[column - 1] != symbol))
                    .min(outside);
                left.min(above).min(diagonal)
            };
            cells[column - start] = value;
        }
        Self {
            cells,
            depth,
            start: start as u8,
            len: len as u8,
        }
    }

    fn minimum(&self, radius: u8) -> u8 {
        self.cells[..self.len as usize]
            .iter()
            .copied()
            .min()
            .unwrap_or_else(|| radius.saturating_add(1))
    }

    fn terminal_distance(&self, query_len: usize, radius: u8) -> u8 {
        self.value(query_len, radius.saturating_add(1))
    }
}

struct DlaBuildFailure {
    reason: &'static str,
    maximum_scratch_bytes: usize,
    states: usize,
    transitions: usize,
    elapsed_us: u64,
}

struct QueryLocalDla {
    transitions: Vec<u16>,
    terminal_distances: Vec<u8>,
    class_by_symbol_ref: Vec<u8>,
    class_count: usize,
    maximum_scratch_bytes: usize,
    resident_scratch_bytes: usize,
    build_elapsed_us: u64,
}

impl QueryLocalDla {
    fn build(
        index: &V13DafsaView,
        query: &[u32],
        radius: u8,
        budget: SearchBudget,
        search_started: Instant,
    ) -> Result<Result<Self, DlaBuildFailure>, String> {
        let started = Instant::now();
        let sidecar_symbols = index.symbols()?;
        let query_symbols = query.iter().copied().collect::<BTreeSet<_>>();
        let mut class_symbols = vec![u32::MAX];
        class_symbols.extend(
            sidecar_symbols
                .iter()
                .copied()
                .filter(|symbol| query_symbols.contains(symbol)),
        );
        if class_symbols.len() > u8::MAX as usize {
            return Ok(Err(DlaBuildFailure {
                reason: "dla_class_budget",
                maximum_scratch_bytes: 0,
                states: 0,
                transitions: 0,
                elapsed_us: elapsed_us(started),
            }));
        }
        let class_by_symbol_ref = sidecar_symbols
            .iter()
            .map(|symbol| {
                class_symbols[1..]
                    .binary_search(symbol)
                    .map(|index| (index + 1) as u8)
                    .unwrap_or(0)
            })
            .collect::<Vec<_>>();

        let initial = BandedLevenshteinRow::initial(query.len(), radius);
        let mut rows = vec![initial];
        let mut state_ids = HashMap::new();
        state_ids.insert(initial.identity_key(), 0_u16);
        let mut transitions = Vec::new();
        let mut maximum_scratch_bytes = dla_builder_scratch_bytes(
            &rows,
            &state_ids,
            &transitions,
            &class_symbols,
            &class_by_symbol_ref,
            0,
        );
        if maximum_scratch_bytes > budget.maximum_scratch_bytes {
            return Ok(Err(DlaBuildFailure {
                reason: "scratch_budget",
                maximum_scratch_bytes,
                states: rows.len(),
                transitions: transitions.len(),
                elapsed_us: elapsed_us(started),
            }));
        }

        let mut cursor = 0_usize;
        while cursor < rows.len() {
            let row = rows[cursor];
            for symbol in class_symbols.iter().copied() {
                let next = row.advance(query, symbol, radius);
                let next_state = if next.minimum(radius) > radius {
                    DEAD_DLA_STATE
                } else if let Some(state_id) = state_ids.get(&next.identity_key()).copied() {
                    state_id
                } else {
                    if rows.len() >= DEAD_DLA_STATE as usize {
                        return Ok(Err(DlaBuildFailure {
                            reason: "dla_state_budget",
                            maximum_scratch_bytes,
                            states: rows.len(),
                            transitions: transitions.len(),
                            elapsed_us: elapsed_us(started),
                        }));
                    }
                    let state_id = rows.len() as u16;
                    rows.push(next);
                    state_ids.insert(next.identity_key(), state_id);
                    state_id
                };
                transitions.push(next_state);
            }
            cursor += 1;
            maximum_scratch_bytes = maximum_scratch_bytes.max(dla_builder_scratch_bytes(
                &rows,
                &state_ids,
                &transitions,
                &class_symbols,
                &class_by_symbol_ref,
                0,
            ));
            if maximum_scratch_bytes > budget.maximum_scratch_bytes {
                return Ok(Err(DlaBuildFailure {
                    reason: "scratch_budget",
                    maximum_scratch_bytes,
                    states: rows.len(),
                    transitions: transitions.len(),
                    elapsed_us: elapsed_us(started),
                }));
            }
            if budget
                .maximum_elapsed
                .is_some_and(|maximum| search_started.elapsed() > maximum)
            {
                return Ok(Err(DlaBuildFailure {
                    reason: "wall_deadline",
                    maximum_scratch_bytes,
                    states: rows.len(),
                    transitions: transitions.len(),
                    elapsed_us: elapsed_us(started),
                }));
            }
        }

        let terminal_distances = rows
            .iter()
            .map(|row| row.terminal_distance(query.len(), radius))
            .collect::<Vec<_>>();
        let resident_scratch_bytes = transitions.capacity() * std::mem::size_of::<u16>()
            + terminal_distances.capacity() * std::mem::size_of::<u8>()
            + class_by_symbol_ref.capacity() * std::mem::size_of::<u8>();
        maximum_scratch_bytes = maximum_scratch_bytes.max(dla_builder_scratch_bytes(
            &rows,
            &state_ids,
            &transitions,
            &class_symbols,
            &class_by_symbol_ref,
            terminal_distances.capacity(),
        ));
        if maximum_scratch_bytes > budget.maximum_scratch_bytes {
            return Ok(Err(DlaBuildFailure {
                reason: "scratch_budget",
                maximum_scratch_bytes,
                states: rows.len(),
                transitions: transitions.len(),
                elapsed_us: elapsed_us(started),
            }));
        }
        Ok(Ok(Self {
            transitions,
            terminal_distances,
            class_by_symbol_ref,
            class_count: class_symbols.len(),
            maximum_scratch_bytes,
            resident_scratch_bytes,
            build_elapsed_us: elapsed_us(started),
        }))
    }

    fn transition(&self, state_id: u16, symbol_ref: u16) -> Result<Option<u16>, String> {
        let class = *self
            .class_by_symbol_ref
            .get(symbol_ref as usize)
            .ok_or_else(|| "V11 DLA symbol reference is out of range".to_string())?
            as usize;
        let transition = *self
            .transitions
            .get(state_id as usize * self.class_count + class)
            .ok_or_else(|| "V11 DLA transition reference is out of range".to_string())?;
        Ok((transition != DEAD_DLA_STATE).then_some(transition))
    }

    fn terminal_distance(&self, state_id: u16) -> Result<u8, String> {
        self.terminal_distances
            .get(state_id as usize)
            .copied()
            .ok_or_else(|| "V11 DLA terminal reference is out of range".to_string())
    }

    fn state_count(&self) -> usize {
        self.terminal_distances.len()
    }
}

fn dla_builder_scratch_bytes(
    rows: &Vec<BandedLevenshteinRow>,
    state_ids: &HashMap<u64, u16>,
    transitions: &Vec<u16>,
    class_symbols: &Vec<u32>,
    class_by_symbol_ref: &Vec<u8>,
    terminal_capacity: usize,
) -> usize {
    rows.capacity() * std::mem::size_of::<BandedLevenshteinRow>()
        + state_ids.capacity() * DLA_HASH_BUCKET_BYTES
        + transitions.capacity() * std::mem::size_of::<u16>()
        + class_symbols.capacity() * std::mem::size_of::<u32>()
        + class_by_symbol_ref.capacity() * std::mem::size_of::<u8>()
        + terminal_capacity * std::mem::size_of::<u8>()
}

trait TraversalKernel {
    type State: Copy;

    fn initial(&self, query_len: usize, radius: u8) -> Self::State;
    fn transition(
        &self,
        state: Self::State,
        query: &[u32],
        edge: PackedEdge,
        radius: u8,
    ) -> Result<Option<Self::State>, String>;
    fn terminal_distance(
        &self,
        state: Self::State,
        query_len: usize,
        radius: u8,
    ) -> Result<u8, String>;
    fn edge(&self, index: &V13DafsaView, edge_id: usize) -> Result<PackedEdge, String>;
    fn resident_scratch_bytes(&self) -> usize;
}

struct RowTraversalKernel<Row>(std::marker::PhantomData<Row>);

impl<Row> Default for RowTraversalKernel<Row> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<Row: LevenshteinRow + Copy> TraversalKernel for RowTraversalKernel<Row> {
    type State = Row;

    fn initial(&self, query_len: usize, radius: u8) -> Self::State {
        Row::initial(query_len, radius)
    }

    fn transition(
        &self,
        state: Self::State,
        query: &[u32],
        edge: PackedEdge,
        radius: u8,
    ) -> Result<Option<Self::State>, String> {
        let next = state.advance(query, edge.symbol, radius);
        Ok((next.minimum(radius) <= radius).then_some(next))
    }

    fn terminal_distance(
        &self,
        state: Self::State,
        query_len: usize,
        radius: u8,
    ) -> Result<u8, String> {
        Ok(state.terminal_distance(query_len, radius))
    }

    fn edge(&self, index: &V13DafsaView, edge_id: usize) -> Result<PackedEdge, String> {
        index.edge(edge_id)
    }

    fn resident_scratch_bytes(&self) -> usize {
        0
    }
}

struct DlaTraversalKernel<'a>(&'a QueryLocalDla);

impl TraversalKernel for DlaTraversalKernel<'_> {
    type State = u16;

    fn initial(&self, _query_len: usize, _radius: u8) -> Self::State {
        0
    }

    fn transition(
        &self,
        state: Self::State,
        _query: &[u32],
        edge: PackedEdge,
        _radius: u8,
    ) -> Result<Option<Self::State>, String> {
        self.0.transition(state, edge.symbol_ref)
    }

    fn terminal_distance(
        &self,
        state: Self::State,
        _query_len: usize,
        _radius: u8,
    ) -> Result<u8, String> {
        self.0.terminal_distance(state)
    }

    fn edge(&self, index: &V13DafsaView, edge_id: usize) -> Result<PackedEdge, String> {
        index.packed_edge(edge_id)
    }

    fn resident_scratch_bytes(&self) -> usize {
        self.0.resident_scratch_bytes
    }
}

#[derive(Clone, Copy)]
struct FullLevenshteinRow {
    cells: [u8; MAX_QUERY_SYMBOLS + 1],
    len: u8,
}

impl LevenshteinRow for FullLevenshteinRow {
    fn initial(query_len: usize, _radius: u8) -> Self {
        let mut cells = [0_u8; MAX_QUERY_SYMBOLS + 1];
        for (value, cell) in cells.iter_mut().enumerate().take(query_len + 1) {
            *cell = value as u8;
        }
        Self {
            cells,
            len: (query_len + 1) as u8,
        }
    }

    fn advance(&self, query: &[u32], symbol: u32, _radius: u8) -> Self {
        Self {
            cells: levenshtein_row(&self.cells[..self.len as usize], query, symbol),
            len: self.len,
        }
    }

    fn minimum(&self, _radius: u8) -> u8 {
        self.cells[..self.len as usize]
            .iter()
            .copied()
            .min()
            .unwrap_or(u8::MAX)
    }

    fn terminal_distance(&self, _query_len: usize, _radius: u8) -> u8 {
        self.cells[self.len as usize - 1]
    }
}

fn search_typed_peaks(
    index: &V13DafsaView,
    package: &RuntimeL2Package,
    observed: &str,
    budget: SearchBudget,
) -> Result<SearchResult, String> {
    search_typed_peaks_with_kernel(index, package, observed, budget, false, SearchKernel::Dla)
}

fn search_typed_peaks_with_schedule(
    index: &V13DafsaView,
    package: &RuntimeL2Package,
    observed: &str,
    budget: SearchBudget,
    reverse_schedule: bool,
) -> Result<SearchResult, String> {
    search_typed_peaks_with_kernel(
        index,
        package,
        observed,
        budget,
        reverse_schedule,
        SearchKernel::Dla,
    )
}

fn search_typed_peaks_banded_oracle(
    index: &V13DafsaView,
    package: &RuntimeL2Package,
    observed: &str,
    budget: SearchBudget,
) -> Result<SearchResult, String> {
    search_typed_peaks_with_kernel(
        index,
        package,
        observed,
        budget,
        false,
        SearchKernel::BandedOracle,
    )
}

fn search_typed_peaks_full_row(
    index: &V13DafsaView,
    package: &RuntimeL2Package,
    observed: &str,
    budget: SearchBudget,
) -> Result<SearchResult, String> {
    search_typed_peaks_with_kernel(
        index,
        package,
        observed,
        budget,
        false,
        SearchKernel::FullRowOracle,
    )
}

fn search_typed_peaks_with_kernel(
    index: &V13DafsaView,
    package: &RuntimeL2Package,
    observed: &str,
    budget: SearchBudget,
    reverse_schedule: bool,
    kernel: SearchKernel,
) -> Result<SearchResult, String> {
    let started = Instant::now();
    let oracle = Phase7dCertificateOracle::new(observed)?;
    let lanes = oracle.retrieval_lanes();
    let mut terminal_refs = Vec::new();
    let mut expanded = 0_usize;
    let mut maximum_scratch = 0_usize;
    let mut dla_states = 0_usize;
    let mut dla_transitions = 0_usize;
    let mut maximum_dla_classes = 0_usize;
    let mut dla_build_elapsed_us = 0_u64;
    let mut intersection_elapsed_us = 0_u64;
    for lane in lanes {
        let retained_terminal_scratch = terminal_refs.capacity() * std::mem::size_of::<u32>();
        let lane_budget = SearchBudget {
            maximum_scratch_bytes: budget
                .maximum_scratch_bytes
                .saturating_sub(retained_terminal_scratch),
            ..budget
        };
        let outcome = match kernel {
            SearchKernel::Dla => {
                enumerate_dla_lane(index, &lane, lane_budget, started, reverse_schedule)?
            }
            SearchKernel::BandedOracle => enumerate_row_lane::<BandedLevenshteinRow>(
                index,
                &lane,
                lane_budget,
                started,
                reverse_schedule,
            )?,
            SearchKernel::FullRowOracle => enumerate_row_lane::<FullLevenshteinRow>(
                index,
                &lane,
                lane_budget,
                started,
                reverse_schedule,
            )?,
        };
        expanded = expanded.saturating_add(outcome.expanded);
        maximum_scratch = maximum_scratch.max(
            outcome
                .maximum_scratch
                .saturating_add(retained_terminal_scratch),
        );
        dla_states = dla_states.saturating_add(outcome.dla_states);
        dla_transitions = dla_transitions.saturating_add(outcome.dla_transitions);
        maximum_dla_classes = maximum_dla_classes.max(outcome.dla_classes);
        dla_build_elapsed_us = dla_build_elapsed_us.saturating_add(outcome.dla_build_elapsed_us);
        intersection_elapsed_us =
            intersection_elapsed_us.saturating_add(outcome.intersection_elapsed_us);
        if let Some(reason) = outcome.unresolved {
            return Ok(SearchResult {
                retrieved_form_refs: Vec::new(),
                peaks: Vec::new(),
                completeness: SearchCompleteness::Unresolved(reason),
                expanded_product_states: expanded,
                maximum_scratch_bytes: maximum_scratch,
                dla_states,
                dla_transitions,
                maximum_dla_classes,
                dla_build_elapsed_us,
                intersection_elapsed_us,
                search_elapsed_us: elapsed_us(started),
                material_elapsed_us: 0,
                total_elapsed_us: elapsed_us(started),
            });
        }
        terminal_refs.extend(outcome.form_refs);
        maximum_scratch =
            maximum_scratch.max(terminal_refs.capacity() * std::mem::size_of::<u32>());
    }
    terminal_refs.sort_unstable();
    terminal_refs.dedup();
    let search_elapsed_us = elapsed_us(started);
    let retrieved_form_refs = terminal_refs.clone();

    let mut peaks = Vec::new();
    for form_ref in terminal_refs {
        let surface = package
            .surface(form_ref as usize)
            .ok_or_else(|| format!("V13 terminal rank {form_ref} cannot be decoded"))?;
        let certificate_keys = oracle.certificate_keys(surface.as_ref())?;
        if !certificate_keys.is_empty() {
            peaks.push(V13TypedPeak {
                form_ref,
                certificate_keys,
            });
        }
    }
    let total_elapsed_us = elapsed_us(started);
    Ok(SearchResult {
        retrieved_form_refs,
        peaks,
        completeness: SearchCompleteness::CertifiedExhaustive,
        expanded_product_states: expanded,
        maximum_scratch_bytes: maximum_scratch,
        dla_states,
        dla_transitions,
        maximum_dla_classes,
        dla_build_elapsed_us,
        intersection_elapsed_us,
        search_elapsed_us,
        material_elapsed_us: total_elapsed_us.saturating_sub(search_elapsed_us),
        total_elapsed_us,
    })
}

struct LaneOutcome {
    form_refs: Vec<u32>,
    expanded: usize,
    maximum_scratch: usize,
    unresolved: Option<&'static str>,
    dla_states: usize,
    dla_transitions: usize,
    dla_classes: usize,
    dla_build_elapsed_us: u64,
    intersection_elapsed_us: u64,
}

impl LaneOutcome {
    fn unresolved(reason: &'static str, maximum_scratch: usize) -> Self {
        Self {
            form_refs: Vec::new(),
            expanded: 0,
            maximum_scratch,
            unresolved: Some(reason),
            dla_states: 0,
            dla_transitions: 0,
            dla_classes: 0,
            dla_build_elapsed_us: 0,
            intersection_elapsed_us: 0,
        }
    }
}

fn validate_lane(lane: &Phase7dRetrievalLane) -> Option<&'static str> {
    if lane.symbols.len() > MAX_QUERY_SYMBOLS {
        Some("query_symbol_budget")
    } else if lane.maximum_levenshtein_distance > MAX_LEVENSHTEIN_RADIUS {
        Some("lane_radius_budget")
    } else {
        None
    }
}

fn enumerate_dla_lane(
    index: &V13DafsaView,
    lane: &Phase7dRetrievalLane,
    budget: SearchBudget,
    started: Instant,
    reverse_schedule: bool,
) -> Result<LaneOutcome, String> {
    if let Some(reason) = validate_lane(lane) {
        return Ok(LaneOutcome::unresolved(reason, 0));
    }
    let query = lane.symbols.as_ref();
    let dla = match QueryLocalDla::build(
        index,
        query,
        lane.maximum_levenshtein_distance,
        budget,
        started,
    )? {
        Ok(dla) => dla,
        Err(failure) => {
            return Ok(LaneOutcome {
                form_refs: Vec::new(),
                expanded: 0,
                maximum_scratch: failure.maximum_scratch_bytes,
                unresolved: Some(failure.reason),
                dla_states: failure.states,
                dla_transitions: failure.transitions,
                dla_classes: 0,
                dla_build_elapsed_us: failure.elapsed_us,
                intersection_elapsed_us: 0,
            });
        }
    };
    let dla_states = dla.state_count();
    let dla_transitions = dla.transitions.len();
    let dla_classes = dla.class_count;
    let dla_build_elapsed_us = dla.build_elapsed_us;
    let maximum_build_scratch = dla.maximum_scratch_bytes;
    let intersection_started = Instant::now();
    let mut outcome = enumerate_lane_with_kernel(
        index,
        lane,
        budget,
        started,
        reverse_schedule,
        &DlaTraversalKernel(&dla),
    )?;
    outcome.maximum_scratch = outcome.maximum_scratch.max(maximum_build_scratch);
    outcome.dla_states = dla_states;
    outcome.dla_transitions = dla_transitions;
    outcome.dla_classes = dla_classes;
    outcome.dla_build_elapsed_us = dla_build_elapsed_us;
    outcome.intersection_elapsed_us = elapsed_us(intersection_started);
    Ok(outcome)
}

fn enumerate_row_lane<Row: LevenshteinRow + Copy>(
    index: &V13DafsaView,
    lane: &Phase7dRetrievalLane,
    budget: SearchBudget,
    started: Instant,
    reverse_schedule: bool,
) -> Result<LaneOutcome, String> {
    if let Some(reason) = validate_lane(lane) {
        return Ok(LaneOutcome::unresolved(reason, 0));
    }
    enumerate_lane_with_kernel(
        index,
        lane,
        budget,
        started,
        reverse_schedule,
        &RowTraversalKernel::<Row>::default(),
    )
}

fn enumerate_lane_with_kernel<Kernel: TraversalKernel>(
    index: &V13DafsaView,
    lane: &Phase7dRetrievalLane,
    budget: SearchBudget,
    started: Instant,
    reverse_schedule: bool,
    kernel: &Kernel,
) -> Result<LaneOutcome, String> {
    let query = lane.symbols.as_ref();
    let mut stack = vec![SearchNode {
        state_id: index.root_state,
        rank_prefix: 0,
        row: kernel.initial(query.len(), lane.maximum_levenshtein_distance),
    }];
    let mut form_refs = Vec::new();
    let mut expanded = 0_usize;
    let base_scratch = kernel.resident_scratch_bytes();
    let mut maximum_scratch = search_scratch_bytes(&stack, &form_refs, base_scratch);
    while let Some(node) = stack.pop() {
        expanded += 1;
        if expanded > budget.maximum_product_states {
            return Ok(LaneOutcome {
                form_refs,
                expanded,
                maximum_scratch,
                unresolved: Some("product_state_budget"),
                dla_states: 0,
                dla_transitions: 0,
                dla_classes: 0,
                dla_build_elapsed_us: 0,
                intersection_elapsed_us: 0,
            });
        }
        if budget
            .maximum_elapsed
            .is_some_and(|maximum| started.elapsed() > maximum)
        {
            return Ok(LaneOutcome {
                form_refs,
                expanded,
                maximum_scratch,
                unresolved: Some("wall_deadline"),
                dla_states: 0,
                dla_transitions: 0,
                dla_classes: 0,
                dla_build_elapsed_us: 0,
                intersection_elapsed_us: 0,
            });
        }
        let state = index.state(node.state_id)?;
        if state.terminal()
            && kernel.terminal_distance(node.row, query.len(), lane.maximum_levenshtein_distance)?
                <= lane.maximum_levenshtein_distance
        {
            form_refs.push(node.rank_prefix);
            if form_refs.len() > budget.maximum_terminals {
                return Ok(LaneOutcome {
                    form_refs,
                    expanded,
                    maximum_scratch,
                    unresolved: Some("terminal_budget"),
                    dla_states: 0,
                    dla_transitions: 0,
                    dla_classes: 0,
                    dla_build_elapsed_us: 0,
                    intersection_elapsed_us: 0,
                });
            }
        }
        if reverse_schedule {
            let mut children = Vec::new();
            for edge_id in index.edge_range(state)? {
                let edge = kernel.edge(index, edge_id)?;
                let rank_prefix = node
                    .rank_prefix
                    .checked_add(edge.rank_delta)
                    .ok_or_else(|| "V13 search rank overflows u32".to_string())?;
                if let Some(child) = search_child(&node, query, edge, rank_prefix, lane, kernel)? {
                    children.push(child);
                }
            }
            maximum_scratch = maximum_scratch.max(
                search_scratch_bytes(&stack, &form_refs, base_scratch)
                    + children.capacity() * std::mem::size_of::<SearchNode<Kernel::State>>(),
            );
            stack.extend(children.into_iter().rev());
        } else {
            for edge_id in index.edge_range(state)? {
                let edge = kernel.edge(index, edge_id)?;
                let rank_prefix = node
                    .rank_prefix
                    .checked_add(edge.rank_delta)
                    .ok_or_else(|| "V13 search rank overflows u32".to_string())?;
                if let Some(child) = search_child(&node, query, edge, rank_prefix, lane, kernel)? {
                    stack.push(child);
                }
            }
        }
        maximum_scratch =
            maximum_scratch.max(search_scratch_bytes(&stack, &form_refs, base_scratch));
        if maximum_scratch > budget.maximum_scratch_bytes {
            return Ok(LaneOutcome {
                form_refs,
                expanded,
                maximum_scratch,
                unresolved: Some("scratch_budget"),
                dla_states: 0,
                dla_transitions: 0,
                dla_classes: 0,
                dla_build_elapsed_us: 0,
                intersection_elapsed_us: 0,
            });
        }
    }
    Ok(LaneOutcome {
        form_refs,
        expanded,
        maximum_scratch,
        unresolved: None,
        dla_states: 0,
        dla_transitions: 0,
        dla_classes: 0,
        dla_build_elapsed_us: 0,
        intersection_elapsed_us: 0,
    })
}

fn search_child<Kernel: TraversalKernel>(
    parent: &SearchNode<Kernel::State>,
    query: &[u32],
    edge: PackedEdge,
    rank_prefix: u32,
    lane: &Phase7dRetrievalLane,
    kernel: &Kernel,
) -> Result<Option<SearchNode<Kernel::State>>, String> {
    Ok(kernel
        .transition(parent.row, query, edge, lane.maximum_levenshtein_distance)?
        .map(|next_row| SearchNode {
            state_id: edge.target,
            rank_prefix,
            row: next_row,
        }))
}

fn levenshtein_row(previous: &[u8], query: &[u32], symbol: u32) -> [u8; MAX_QUERY_SYMBOLS + 1] {
    let mut row = [u8::MAX; MAX_QUERY_SYMBOLS + 1];
    row[0] = previous[0].saturating_add(1);
    for column in 1..previous.len() {
        let insertion = row[column - 1].saturating_add(1);
        let deletion = previous[column].saturating_add(1);
        let substitution =
            previous[column - 1].saturating_add(u8::from(query[column - 1] != symbol));
        row[column] = insertion.min(deletion).min(substitution);
    }
    row
}

fn search_scratch_bytes<Row>(
    stack: &Vec<SearchNode<Row>>,
    terminals: &Vec<u32>,
    base: usize,
) -> usize {
    base + stack.capacity() * std::mem::size_of::<SearchNode<Row>>()
        + terminals.capacity() * std::mem::size_of::<u32>()
}

fn elapsed_us(started: Instant) -> u64 {
    started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    bytes
        .get(offset..offset + 4)
        .and_then(|value| value.try_into().ok())
        .map(u32::from_le_bytes)
        .ok_or_else(|| "truncated V13 DAFSA u32".to_string())
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, String> {
    bytes
        .get(offset..offset + 8)
        .and_then(|value| value.try_into().ok())
        .map(u64::from_le_bytes)
        .ok_or_else(|| "truncated V13 DAFSA u64".to_string())
}

#[cfg(test)]
mod tests;
