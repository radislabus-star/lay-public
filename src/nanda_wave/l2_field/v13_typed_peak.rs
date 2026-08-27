//! Exact typed discovery over immutable canonical V13 identities.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};

use super::package_bytes::PackageBytes;
use super::productive_v1::{ExactPeakBirthEnumerationV1, ExactPeakCandidateInputV1};
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
            maximum_product_states: 100_000,
            maximum_terminals: 16_384,
            maximum_scratch_bytes: MAX_QUERY_SCRATCH_BYTES,
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
        let exact = typed_exact::search(
            self.materialized.view(),
            &oracle.retrieval_lanes(),
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

        let mut candidates = Vec::new();
        for form_ref in exact.retrieved_form_refs {
            let surface = canonical_index
                .decode_form_ref(form_ref)
                .ok_or_else(|| format!("V13 terminal rank {form_ref} cannot be decoded"))?;
            let certificates = oracle.certificate_evidence(surface.as_ref())?;
            if certificates.is_empty() {
                continue;
            }
            candidates.push(ExactPeakCandidateInputV1 {
                form_ref,
                normalized_surface: super::compositional::normalize_surface(surface.as_ref()),
                certificates,
            });
        }
        ExactPeakBirthEnumerationV1::from_candidates(candidates)
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
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::process::{Child, Command, Stdio};

    use crate::nanda_wave::l2_field::model::{FormCenterRef, L2FieldPackage, MorphBinding};
    use crate::nanda_wave::l2_field::productive_v1::{
        materialize_live_productive_v1_field, prepare_live_productive_v1_field,
        prepare_live_productive_v1_field_with_exact_peaks, ExactPeakBirthEnumerationV1,
        ExactPeakCandidateInputV1, PreparedFieldMaterialScopeV1,
        PRODUCTIVE_V90_TYPED_EXACT_SOURCE_ID,
    };
    use crate::typing_transition::proposal_admission::CandidateGateAction;

    const EXPECTED_V13_SHA256: &str =
        "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b";
    const EXPECTED_V13_BYTES: u64 = 140_556_462;
    const EXPECTED_V13_FORMS: usize = 1_875_032;
    const EXPECTED_V7_SHA256: &str =
        "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4";
    const EXPECTED_V10_SIDECAR_SHA256: &str =
        "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd";
    const EXPECTED_V10_SIDECAR_BYTES: u64 = 3_689_884;
    const EXPECTED_V11_SIDECAR_SHA256: &str =
        "5ebffb813ba0ca1e0080ec01756a2dafc51346297558d37cdd135abfde6acfaa";
    const EXPECTED_V11_SIDECAR_BYTES: usize = 2_460_144;
    const EXPECTED_HISTORICAL_PHASE7D_SOURCE_SHA256: &str =
        "d983d16169c7526d56e9f78299524ab80d3d4f3e67ff19770f07a7ae6e61045d";
    const EXPECTED_CURRENT_PHASE7D_SOURCE_SHA256: &str =
        "16b6cc0128099e99c5a77037feac5cf49efd2ed088f6ddb2ade433da92241e5b";
    const EXPECTED_FIXED_CASES: usize = 382;
    const EXPECTED_FIXED_EXAMINED_EDGES: usize = 25_145_756;
    const EXPECTED_FIXED_EXPANDED_STATES: usize = 8_059_788;
    const PSS_HELPER_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v11_mmap_pss_helper";

    #[derive(Clone, Debug)]
    struct ProofCase {
        availability: String,
        class: String,
        damaged_surface: String,
        proof_identity: String,
        target_surface: String,
    }

    fn unique_attempt_identities(cases: &[ProofCase]) -> usize {
        cases
            .iter()
            .map(|case| (case.class.as_str(), case.proof_identity.as_str()))
            .collect::<BTreeSet<_>>()
            .len()
    }

    #[derive(Default)]
    struct ClassRetention {
        records: usize,
        target_form_retained: usize,
        target_lemma_retained: usize,
        unresolved: usize,
        false_certificates: usize,
    }

    struct ProofBindingIndex {
        offsets: Vec<u32>,
        indices: Vec<u32>,
    }

    impl ProofBindingIndex {
        fn build(package: &RuntimeL2Package) -> Result<Self, String> {
            let mut offsets = vec![0_u32; package.form_count() + 1];
            for index in 0..package.binding_count() {
                let binding = package
                    .binding(index)
                    .ok_or_else(|| format!("missing V13 binding {index}"))?;
                let offset = offsets
                    .get_mut(binding.form_center_ref as usize + 1)
                    .ok_or_else(|| format!("binding {index} has an invalid form_ref"))?;
                *offset = offset
                    .checked_add(1)
                    .ok_or_else(|| "V13 binding offset overflows u32".to_string())?;
            }
            for index in 1..offsets.len() {
                offsets[index] = offsets[index]
                    .checked_add(offsets[index - 1])
                    .ok_or_else(|| "V13 binding prefix sum overflows u32".to_string())?;
            }
            let mut indices = vec![0_u32; package.binding_count()];
            let mut next = offsets[..package.form_count()].to_vec();
            for index in 0..package.binding_count() {
                let binding = package.binding(index).expect("validated binding");
                let form_ref = binding.form_center_ref as usize;
                let output = next[form_ref] as usize;
                indices[output] = index as u32;
                next[form_ref] += 1;
            }
            Ok(Self { offsets, indices })
        }

        fn bindings_for_form<'a>(
            &'a self,
            package: &'a RuntimeL2Package,
            form_ref: u32,
        ) -> impl Iterator<Item = MorphBinding> + 'a {
            let start = self.offsets[form_ref as usize] as usize;
            let end = self.offsets[form_ref as usize + 1] as usize;
            self.indices[start..end]
                .iter()
                .filter_map(|index| package.binding(*index as usize))
        }

        fn lemma_ids(&self, package: &RuntimeL2Package, form_ref: u32) -> BTreeSet<u32> {
            self.bindings_for_form(package, form_ref)
                .map(|binding| binding.lemma_center_id)
                .collect()
        }

        fn proof_auxiliary_bytes(&self) -> usize {
            (self.offsets.capacity() + self.indices.capacity()) * std::mem::size_of::<u32>()
        }
    }

    fn package_for(surfaces: &[String]) -> RuntimeL2Package {
        let mut decoder_bytes = Vec::new();
        let mut form_refs = Vec::new();
        for surface in surfaces {
            let decoder_ref = decoder_bytes.len() as u32;
            decoder_bytes.extend_from_slice(surface.as_bytes());
            decoder_bytes.push(0);
            form_refs.push(FormCenterRef {
                decoder_ref,
                ..FormCenterRef::default()
            });
        }
        RuntimeL2Package::from_reference(L2FieldPackage {
            form_refs,
            decoder_bytes,
            ..L2FieldPackage::default()
        })
    }

    fn sorted_surfaces(values: &[&str]) -> Vec<String> {
        let mut values = values
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>();
        values.sort();
        values.dedup();
        values
    }

    #[test]
    fn minimal_dafsa_preserves_exact_lexicographic_form_refs() {
        let surfaces = sorted_surfaces(&[
            "а",
            "аб",
            "ав",
            "ба",
            "бб",
            "ва",
            "вб",
            "проверка",
            "проверки",
        ]);
        let (bytes, identity) = compile_test_sidecar(&surfaces).expect("compile sidecar");
        let view = V13DafsaView::from_bytes(bytes, identity).expect("load sidecar");
        assert_eq!(
            view.state(view.root_state).unwrap().suffix_count,
            surfaces.len() as u32
        );
        for (form_ref, surface) in surfaces.iter().enumerate() {
            let symbols = surface.chars().map(|ch| ch as u32).collect::<Vec<_>>();
            assert_eq!(
                view.exact_form_ref(&symbols).unwrap(),
                Some(form_ref as u32)
            );
        }
        assert_eq!(view.exact_form_ref(&['я' as u32]).unwrap(), None);
        assert!(view.sidecar_bytes() <= MAX_SIDECAR_BYTES);
        assert!(view.owned_metadata_bytes() <= MAX_LOADER_METADATA_BYTES);
    }

    #[test]
    fn radius_three_retrieval_is_only_a_superset_of_phase7d_certificates() {
        let surfaces = sorted_surfaces(&[
            "молоко",
            "мороз",
            "проверка",
            "проверки",
            "проверить",
            "работает",
            "работают",
            "разверставшие",
            "собака",
            "слово",
        ]);
        let package = package_for(&surfaces);
        let (bytes, identity) = compile_test_sidecar(&surfaces).expect("compile sidecar");
        let view = V13DafsaView::from_bytes(bytes, identity).expect("load sidecar");
        for observed in [
            "проврека",
            "провека",
            "проверрка",
            "проверк",
            "рбаотают",
            "развёрстбвшие",
            "cj,frf",
            "слово!",
        ] {
            let result = search_typed_peaks(&view, &package, observed, SearchBudget::proof())
                .expect("typed search");
            assert_eq!(result.completeness, SearchCompleteness::CertifiedExhaustive);
            let found = result
                .peaks
                .iter()
                .map(|peak| peak.form_ref)
                .collect::<BTreeSet<_>>();
            let expected = surfaces
                .iter()
                .enumerate()
                .filter_map(|(form_ref, surface)| {
                    (!crate::nanda_wave::lexical_grokking::phase7d_certificate_keys(
                        observed, surface,
                    )
                    .expect("oracle")
                    .is_empty())
                    .then_some(form_ref as u32)
                })
                .collect::<BTreeSet<_>>();
            assert_eq!(found, expected, "observed={observed}");
            assert!(result.maximum_scratch_bytes <= MAX_QUERY_SCRATCH_BYTES);
        }
    }

    #[test]
    fn generated_phase7d_matrix_matches_tiny_exhaustive_oracle() {
        let alphabet = "абвгдежзийклмнопрстуфхцчшщыэюя".chars().collect::<Vec<_>>();
        let mut surfaces = (0..96)
            .map(|index| {
                format!(
                    "провер{}{}ние",
                    alphabet[index % alphabet.len()],
                    alphabet[index / alphabet.len()]
                )
            })
            .collect::<Vec<_>>();
        surfaces.sort();
        surfaces.dedup();
        let cases =
            crate::nanda_wave::lexical_grokking::prepare_fixed_heldout_cases(&surfaces, 2, 0)
                .expect("generated Phase 7D cases");
        let classes = cases.iter().map(|case| case.class).collect::<BTreeSet<_>>();
        assert_eq!(classes.len(), 13, "every Phase 7D family must be generated");

        let package = package_for(&surfaces);
        let (bytes, identity) = compile_test_sidecar(&surfaces).expect("compile sidecar");
        let view = V13DafsaView::from_bytes(bytes, identity).expect("load sidecar");
        for case in cases {
            let result = search_typed_peaks(&view, &package, &case.surface, SearchBudget::proof())
                .expect("typed search");
            let banded = search_typed_peaks_banded_oracle(
                &view,
                &package,
                &case.surface,
                SearchBudget::proof(),
            )
            .expect("banded oracle");
            let full_row =
                search_typed_peaks_full_row(&view, &package, &case.surface, SearchBudget::proof())
                    .expect("full-row oracle");
            assert_eq!(result.retrieved_form_refs, banded.retrieved_form_refs);
            assert_eq!(result.peaks, banded.peaks);
            assert_eq!(result.completeness, banded.completeness);
            assert_eq!(
                result.expanded_product_states,
                banded.expanded_product_states
            );
            assert_eq!(
                result.retrieved_form_refs, full_row.retrieved_form_refs,
                "retrieval class={} observed={}",
                case.class, case.surface
            );
            assert_eq!(
                result.peaks, full_row.peaks,
                "certificates class={} observed={}",
                case.class, case.surface
            );
            assert_eq!(result.completeness, full_row.completeness);
            assert_eq!(
                result.expanded_product_states,
                full_row.expanded_product_states
            );
            assert_eq!(
                result.completeness,
                SearchCompleteness::CertifiedExhaustive,
                "class={} observed={}",
                case.class,
                case.surface
            );
            let found = result
                .peaks
                .iter()
                .map(|peak| peak.form_ref)
                .collect::<BTreeSet<_>>();
            let expected = surfaces
                .iter()
                .enumerate()
                .filter_map(|(form_ref, surface)| {
                    (!crate::nanda_wave::lexical_grokking::phase7d_certificate_keys(
                        &case.surface,
                        surface,
                    )
                    .expect("oracle")
                    .is_empty())
                    .then_some(form_ref as u32)
                })
                .collect::<BTreeSet<_>>();
            assert_eq!(
                found, expected,
                "class={} observed={}",
                case.class, case.surface
            );
            assert!(found.contains(&case.terminal_id));
        }
    }

    #[test]
    fn query_local_dla_transitions_match_the_banded_oracle() {
        let surfaces = sorted_surfaces(&[
            "молоко",
            "проверка",
            "проверить",
            "работает",
            "работают",
            "собака",
        ]);
        let (bytes, identity) = compile_test_sidecar(&surfaces).expect("compile sidecar");
        let view = V13DafsaView::from_bytes(bytes, identity).expect("load sidecar");
        for observed in ["проврека", "cj,frf", "слово!"] {
            let query = observed.chars().map(|ch| ch as u32).collect::<Vec<_>>();
            for radius in 0..=MAX_LEVENSHTEIN_RADIUS {
                assert_dla_transition_parity(&view, &query, radius);
            }
        }
    }

    fn assert_dla_transition_parity(index: &V13DafsaView, query: &[u32], radius: u8) {
        let dla = QueryLocalDla::build(index, query, radius, SearchBudget::proof(), Instant::now())
            .expect("DLA build source")
            .map_err(|failure| failure.reason)
            .expect("DLA build budget");
        let sidecar_symbols = index.symbols().expect("sidecar symbols");
        let query_symbols = query.iter().copied().collect::<BTreeSet<_>>();
        let mut class_symbols = vec![u32::MAX];
        class_symbols.extend(
            sidecar_symbols
                .iter()
                .copied()
                .filter(|symbol| query_symbols.contains(symbol)),
        );
        assert_eq!(dla.class_count, class_symbols.len());
        for (symbol_ref, symbol) in sidecar_symbols.iter().enumerate() {
            let expected = class_symbols[1..]
                .binary_search(symbol)
                .map(|index| (index + 1) as u8)
                .unwrap_or(0);
            assert_eq!(dla.class_by_symbol_ref[symbol_ref], expected);
        }

        let initial = BandedLevenshteinRow::initial(query.len(), radius);
        let mut rows = vec![initial];
        let mut ids = HashMap::from([(initial.identity_key(), 0_u16)]);
        let mut cursor = 0_usize;
        while cursor < rows.len() {
            let row = rows[cursor];
            assert_eq!(
                dla.terminal_distance(cursor as u16).expect("DLA terminal"),
                row.terminal_distance(query.len(), radius)
            );
            for (class, symbol) in class_symbols.iter().copied().enumerate() {
                let next = row.advance(query, symbol, radius);
                let expected = if next.minimum(radius) > radius {
                    DEAD_DLA_STATE
                } else if let Some(state_id) = ids.get(&next.identity_key()).copied() {
                    state_id
                } else {
                    let state_id = rows.len() as u16;
                    rows.push(next);
                    ids.insert(next.identity_key(), state_id);
                    state_id
                };
                assert_eq!(
                    dla.transitions[cursor * class_symbols.len() + class],
                    expected
                );
            }
            cursor += 1;
        }
        assert_eq!(dla.state_count(), rows.len());
        assert!(dla.maximum_scratch_bytes <= MAX_QUERY_SCRATCH_BYTES);
    }

    #[test]
    fn budget_exhaustion_is_unresolved_and_cannot_emit_certified_peaks() {
        let surfaces = sorted_surfaces(&["проверка", "проверки", "проверить", "проверяет"]);
        let package = package_for(&surfaces);
        let (bytes, identity) = compile_test_sidecar(&surfaces).expect("compile sidecar");
        let view = V13DafsaView::from_bytes(bytes, identity).expect("load sidecar");
        let result = search_typed_peaks(
            &view,
            &package,
            "проврека",
            SearchBudget {
                maximum_product_states: 1,
                ..SearchBudget::proof()
            },
        )
        .expect("typed search");
        assert!(matches!(
            result.completeness,
            SearchCompleteness::Unresolved("product_state_budget")
        ));
        assert!(result.peaks.is_empty());
    }

    #[test]
    fn corruption_and_identity_mismatch_fail_closed() {
        let surfaces = sorted_surfaces(&["код", "кот", "коты"]);
        let (bytes, identity) = compile_test_sidecar(&surfaces).expect("compile sidecar");
        let mut corrupted = bytes.clone();
        *corrupted.last_mut().expect("payload") ^= 1;
        assert!(V13DafsaView::from_bytes(corrupted, identity)
            .unwrap_err()
            .contains("checksum"));
        let wrong = V13Identity {
            package_sha256: [9; 32],
            ..identity
        };
        assert!(V13DafsaView::from_bytes(bytes, wrong)
            .unwrap_err()
            .contains("identity"));
    }

    #[test]
    fn rank_delta_corruption_with_valid_checksum_fails_closed() {
        let surfaces = sorted_surfaces(&["а", "аб", "ав", "ба", "бб"]);
        let (mut bytes, identity) = compile_test_sidecar(&surfaces).expect("compile sidecar");
        let state_count = read_u32(&bytes, 176).expect("state count") as usize;
        let edge_offset = HEADER_BYTES + state_count * STATE_BYTES;
        let edge = read_u64(&bytes, edge_offset).expect("packed edge");
        put_u64(&mut bytes, edge_offset, edge ^ (1_u64 << 40));
        let checksum: [u8; 32] = Sha256::digest(&bytes[HEADER_BYTES..]).into();
        bytes[24..56].copy_from_slice(&checksum);
        assert!(V13DafsaView::from_bytes(bytes, identity)
            .unwrap_err()
            .contains("rank delta"));
    }

    #[test]
    fn symbol_reference_corruption_with_valid_checksum_fails_closed() {
        let surfaces = sorted_surfaces(&["а", "аб", "ав", "ба", "бб"]);
        let (mut bytes, identity) = compile_test_sidecar(&surfaces).expect("compile sidecar");
        let state_count = read_u32(&bytes, 176).expect("state count") as usize;
        let symbol_count = read_u32(&bytes, 200).expect("symbol count");
        let edge_offset = HEADER_BYTES + state_count * STATE_BYTES;
        let edge = read_u64(&bytes, edge_offset).expect("packed edge");
        let corrupted = (edge & !u64::from(u16::MAX)) | u64::from(symbol_count);
        put_u64(&mut bytes, edge_offset, corrupted);
        let checksum: [u8; 32] = Sha256::digest(&bytes[HEADER_BYTES..]).into();
        bytes[24..56].copy_from_slice(&checksum);
        assert!(V13DafsaView::from_bytes(bytes, identity)
            .unwrap_err()
            .contains("symbol reference"));
    }

    #[test]
    fn packed_field_overflow_is_rejected_before_encoding() {
        assert!(PackedState {
            first_edge: PACKED_U24_MAX + 1,
            suffix_count: 1,
            edge_count: 1,
            flags: 0,
        }
        .encode()
        .unwrap_err()
        .contains("first edge"));
        assert!(PackedEdge {
            symbol: 'а' as u32,
            symbol_ref: 0,
            target: 1,
            rank_delta: PACKED_U24_MAX + 1,
        }
        .encode(0)
        .unwrap_err()
        .contains("rank delta"));
    }

    #[test]
    fn attempt_identity_includes_damage_class() {
        let case = |class: &str| ProofCase {
            availability: "available".to_string(),
            class: class.to_string(),
            damaged_surface: "observed".to_string(),
            proof_identity: "shared-source-identity".to_string(),
            target_surface: "target".to_string(),
        };
        assert_eq!(
            unique_attempt_identities(&[case("class-a"), case("class-b")]),
            2
        );
    }

    #[test]
    #[ignore = "one bounded remote V11 A/B/C proof"]
    fn v11_full_v13_abc_proof() {
        let receipt_path = std::env::var_os("LAY_V11_RECEIPT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(
                    "docs/structural_gates/receipts/\
                     LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/\
                     slice8b-v11-packed-dafsa-dla-abc.json",
                )
            });
        let receipt = match run_v11_full_proof() {
            Ok(receipt) => receipt,
            Err(error) => serde_json::json!({
                "schema": "lay.slice8b-v11-packed-dafsa-dla-abc.v1",
                "verdict": "ERROR",
                "error": error,
                "runtime_authority_changed": false,
                "installed_lay_changed": false,
                "promotion_eligible": false,
            }),
        };
        if let Some(parent) = receipt_path.parent() {
            std::fs::create_dir_all(parent).expect("create V11 receipt directory");
        }
        std::fs::write(
            &receipt_path,
            serde_json::to_vec_pretty(&receipt).expect("serialize V11 receipt"),
        )
        .expect("write V11 receipt");
        eprintln!("v11_receipt={}", receipt_path.display());
        assert_eq!(
            receipt.get("verdict").and_then(serde_json::Value::as_str),
            Some("PASS_V11_A_B_C"),
            "{}",
            serde_json::to_string_pretty(&receipt).expect("render failed receipt")
        );
    }

    fn run_v11_full_proof() -> Result<serde_json::Value, String> {
        let package_path = std::env::var_os("LAY_V11_V13_PACKAGE")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(
                    "/home/e/.local/share/lay/nanda_wave/l2/LAY-L2-RU-FULL-v13.bin",
                )
            });
        let v7_path = std::env::var_os("LAY_V11_V7_RECEIPT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(
                    "docs/structural_gates/receipts/\
                     LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/\
                     slice8b-v7-fixed-13x100.json",
                )
            });
        let sidecar_path = std::env::var_os("LAY_V11_SIDECAR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("target/v11/LAY-L2-RU-FULL-v13.dafsa"));

        let package_bytes = std::fs::metadata(&package_path)
            .map_err(|error| format!("{}: {error}", package_path.display()))?
            .len();
        let package_sha256 = sha256_file(&package_path)?;
        let package = RuntimeL2Package::load(&package_path)?;
        let identity = V13Identity {
            package_sha256,
            package_bytes,
            form_count: u32::try_from(package.form_count())
                .map_err(|_| "V13 form count exceeds u32".to_string())?,
            binding_count: u32::try_from(package.binding_count())
                .map_err(|_| "V13 binding count exceeds u32".to_string())?,
        };

        let compile_started = Instant::now();
        let sidecar_bytes = compile_sidecar(&package, identity)?;
        let compile_elapsed_ms = compile_started.elapsed().as_millis() as u64;
        if let Some(parent) = sidecar_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("{}: {error}", parent.display()))?;
        }
        let temporary_sidecar = sidecar_path.with_extension("dafsa.tmp");
        std::fs::write(&temporary_sidecar, &sidecar_bytes)
            .map_err(|error| format!("{}: {error}", temporary_sidecar.display()))?;
        std::fs::rename(&temporary_sidecar, &sidecar_path)
            .map_err(|error| format!("{}: {error}", sidecar_path.display()))?;
        let sidecar_sha256: [u8; 32] = Sha256::digest(&sidecar_bytes).into();
        drop(sidecar_bytes);

        let index = V13DafsaView::load(&sidecar_path, identity)?;
        let roundtrip = validate_full_roundtrip(&index, &package)?;
        let binding_index = ProofBindingIndex::build(&package)?;
        let binding_parity = validate_binding_parity(&package, &binding_index)?;
        let pss = measure_two_process_pss(&sidecar_path, identity)?;
        let gate_b = run_generated_gate_b()?;

        let v7_sha256 = sha256_file(&v7_path)?;
        let source: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&v7_path).map_err(|error| format!("{}: {error}", v7_path.display()))?,
        )
        .map_err(|error| format!("{}: {error}", v7_path.display()))?;
        let cases = parse_v7_cases(&source)?;
        let gate_c = run_gate_c(&index, &package, &binding_index, &cases)?;

        let source_identity_pass = hex(package_sha256) == EXPECTED_V13_SHA256
            && package_bytes == EXPECTED_V13_BYTES
            && package.form_count() == EXPECTED_V13_FORMS
            && hex(v7_sha256) == EXPECTED_V7_SHA256;
        let gate_a_pass = source_identity_pass
            && roundtrip
                .get("mismatches")
                .and_then(serde_json::Value::as_u64)
                == Some(0)
            && roundtrip
                .get("strictly_increasing")
                .and_then(serde_json::Value::as_bool)
                == Some(true)
            && binding_parity
                .get("exact")
                .and_then(serde_json::Value::as_bool)
                == Some(true)
            && index.state(index.root_state)?.suffix_count == identity.form_count
            && index.sidecar_bytes() <= MAX_SIDECAR_BYTES
            && index.owned_metadata_bytes() <= MAX_LOADER_METADATA_BYTES
            && index.mmap_backed()
            && pss
                .get("aggregate_delta_kib")
                .and_then(serde_json::Value::as_u64)
                <= Some(40 * 1024);
        let gate_b_pass = gate_b.get("pass").and_then(serde_json::Value::as_bool) == Some(true);
        let gate_c_pass = gate_c.get("pass").and_then(serde_json::Value::as_bool) == Some(true);
        let verdict = if gate_a_pass && gate_b_pass && gate_c_pass {
            "PASS_V11_A_B_C"
        } else {
            "FAIL_V11_A_B_C"
        };

        Ok(serde_json::json!({
            "schema": "lay.slice8b-v11-packed-dafsa-dla-abc.v1",
            "source": {
                "v13_package": package_path,
                "v13_bytes": package_bytes,
                "v13_sha256": hex(package_sha256),
                "v13_forms": package.form_count(),
                "v13_bindings": package.binding_count(),
                "v7_receipt": v7_path,
                "v7_receipt_sha256": hex(v7_sha256),
            },
            "gate_v11_a": {
                "pass": gate_a_pass,
                "sidecar_path": sidecar_path,
                "sidecar_sha256": hex(sidecar_sha256),
                "sidecar_bytes": index.sidecar_bytes(),
                "state_count": index.state_count,
                "edge_count": index.edge_count,
                "symbol_count": index.symbol_count,
                "state_bytes": STATE_BYTES,
                "edge_bytes": EDGE_BYTES,
                "symbol_bytes": SYMBOL_BYTES,
                "terminal_count": index.state(index.root_state)?.suffix_count,
                "root_state": index.root_state,
                "compile_elapsed_ms": compile_elapsed_ms,
                "loader_owned_metadata_bytes": index.owned_metadata_bytes(),
                "mmap_backed": index.mmap_backed(),
                "roundtrip": roundtrip,
                "binding_parity": binding_parity,
                "proof_only_binding_index_bytes": binding_index.proof_auxiliary_bytes(),
                "two_process_pss": pss,
            },
            "gate_v11_b": gate_b,
            "gate_v11_c": gate_c,
            "claim_boundary": {
                "lexical_discovery_and_typed_evidence_only": true,
                "productive_v90_selection_proven": false,
                "l3_selection_proven": false,
                "physical_input_proven": false,
            },
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
            "promotion_eligible": gate_a_pass && gate_b_pass && gate_c_pass,
            "verdict": verdict,
        }))
    }

    fn validate_full_roundtrip(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
    ) -> Result<serde_json::Value, String> {
        let started = Instant::now();
        let mut mismatches = 0_usize;
        let mut mismatch_samples = Vec::new();
        let mut previous = None::<String>;
        let mut strictly_increasing = true;
        for form_ref in 0..package.form_count() {
            let surface = package
                .surface(form_ref)
                .ok_or_else(|| format!("missing V13 surface {form_ref}"))?
                .into_owned();
            if previous.as_ref().is_some_and(|prior| prior >= &surface) {
                strictly_increasing = false;
            }
            let symbols = surface.chars().map(|ch| ch as u32).collect::<Vec<_>>();
            let resolved = index.exact_form_ref(&symbols)?;
            if resolved != Some(form_ref as u32) {
                mismatches += 1;
                if mismatch_samples.len() < 8 {
                    mismatch_samples.push(serde_json::json!({
                        "form_ref": form_ref,
                        "surface": surface,
                        "resolved": resolved,
                    }));
                }
            }
            previous = Some(surface);
        }
        Ok(serde_json::json!({
            "decoded": package.form_count(),
            "rank_equal_form_ref": package.form_count() - mismatches,
            "mismatches": mismatches,
            "mismatch_samples": mismatch_samples,
            "strictly_increasing": strictly_increasing,
            "elapsed_ms": started.elapsed().as_millis() as u64,
        }))
    }

    fn validate_binding_parity(
        package: &RuntimeL2Package,
        index: &ProofBindingIndex,
    ) -> Result<serde_json::Value, String> {
        let tuple = |binding: MorphBinding| {
            (
                binding.form_center_ref,
                binding.lemma_center_id,
                binding.feature_mask,
                binding.support,
                binding.phase,
                binding.flags,
            )
        };
        let mut expected = (0..package.binding_count())
            .map(|binding| {
                package
                    .binding(binding)
                    .map(&tuple)
                    .ok_or_else(|| format!("missing V13 binding {binding}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut actual = (0..package.form_count())
            .flat_map(|form_ref| index.bindings_for_form(package, form_ref as u32))
            .map(tuple)
            .collect::<Vec<_>>();
        expected.sort_unstable();
        actual.sort_unstable();
        let multi_lemma_forms = (0..package.form_count())
            .filter(|form_ref| index.lemma_ids(package, *form_ref as u32).len() > 1)
            .count();
        Ok(serde_json::json!({
            "expected_bindings": expected.len(),
            "materialized_bindings": actual.len(),
            "multi_lemma_forms": multi_lemma_forms,
            "exact": expected == actual,
        }))
    }

    fn run_generated_gate_b() -> Result<serde_json::Value, String> {
        let alphabet = "абвгдежзийклмнопрстуфхцчшщыэюя".chars().collect::<Vec<_>>();
        let mut surfaces = (0..96)
            .map(|index| {
                format!(
                    "провер{}{}ние",
                    alphabet[index % alphabet.len()],
                    alphabet[index / alphabet.len()]
                )
            })
            .collect::<Vec<_>>();
        surfaces.sort();
        surfaces.dedup();
        let cases =
            crate::nanda_wave::lexical_grokking::prepare_fixed_heldout_cases(&surfaces, 4, 0)
                .map_err(|error| error.to_string())?;
        let classes = cases.iter().map(|case| case.class).collect::<BTreeSet<_>>();
        let package = package_for(&surfaces);
        let (bytes, identity) = compile_test_sidecar(&surfaces)?;
        let view = V13DafsaView::from_bytes(bytes.clone(), identity)?;
        let mut terminal_set_mismatches = 0_usize;
        let mut certificate_mismatches = 0_usize;
        let mut schedule_mismatches = 0_usize;
        let mut v10_banded_terminal_mismatches = 0_usize;
        let mut v10_banded_certificate_mismatches = 0_usize;
        let mut v10_banded_completeness_mismatches = 0_usize;
        let mut v10_banded_work_mismatches = 0_usize;
        let mut full_row_terminal_mismatches = 0_usize;
        let mut full_row_certificate_mismatches = 0_usize;
        let mut full_row_completeness_mismatches = 0_usize;
        let mut full_row_work_mismatches = 0_usize;
        let mut target_losses = 0_usize;
        let mut maximum_scratch = 0_usize;
        let mut maximum_dla_states = 0_usize;
        let mut maximum_dla_transitions = 0_usize;
        let mut maximum_dla_classes = 0_usize;
        for case in &cases {
            let forward = search_typed_peaks_with_schedule(
                &view,
                &package,
                &case.surface,
                SearchBudget::proof(),
                false,
            )?;
            let reverse = search_typed_peaks_with_schedule(
                &view,
                &package,
                &case.surface,
                SearchBudget::proof(),
                true,
            )?;
            let banded = search_typed_peaks_banded_oracle(
                &view,
                &package,
                &case.surface,
                SearchBudget::proof(),
            )?;
            let full_row =
                search_typed_peaks_full_row(&view, &package, &case.surface, SearchBudget::proof())?;
            maximum_scratch = maximum_scratch
                .max(forward.maximum_scratch_bytes)
                .max(reverse.maximum_scratch_bytes);
            maximum_dla_states = maximum_dla_states
                .max(forward.dla_states)
                .max(reverse.dla_states);
            maximum_dla_transitions = maximum_dla_transitions
                .max(forward.dla_transitions)
                .max(reverse.dla_transitions);
            maximum_dla_classes = maximum_dla_classes
                .max(forward.maximum_dla_classes)
                .max(reverse.maximum_dla_classes);
            v10_banded_completeness_mismatches +=
                usize::from(forward.completeness != banded.completeness);
            v10_banded_terminal_mismatches +=
                usize::from(forward.retrieved_form_refs != banded.retrieved_form_refs);
            v10_banded_certificate_mismatches += usize::from(forward.peaks != banded.peaks);
            v10_banded_work_mismatches +=
                usize::from(forward.expanded_product_states != banded.expanded_product_states);
            if forward.completeness != full_row.completeness {
                full_row_completeness_mismatches += 1;
            }
            if forward.retrieved_form_refs != full_row.retrieved_form_refs {
                full_row_terminal_mismatches += 1;
            }
            if forward.peaks != full_row.peaks {
                full_row_certificate_mismatches += 1;
            }
            if forward.expanded_product_states != full_row.expanded_product_states {
                full_row_work_mismatches += 1;
            }
            if forward.completeness != SearchCompleteness::CertifiedExhaustive
                || reverse.completeness != SearchCompleteness::CertifiedExhaustive
                || banded.completeness != SearchCompleteness::CertifiedExhaustive
                || full_row.completeness != SearchCompleteness::CertifiedExhaustive
            {
                terminal_set_mismatches += 1;
                continue;
            }
            if forward.retrieved_form_refs != reverse.retrieved_form_refs
                || forward.peaks != reverse.peaks
            {
                schedule_mismatches += 1;
            }
            let expected = surfaces
                .iter()
                .enumerate()
                .filter_map(|(form_ref, surface)| {
                    let certificates =
                        crate::nanda_wave::lexical_grokking::phase7d_certificate_keys(
                            &case.surface,
                            surface,
                        )
                        .ok()?;
                    (!certificates.is_empty()).then_some((form_ref as u32, certificates))
                })
                .collect::<BTreeMap<_, _>>();
            let actual = forward
                .peaks
                .iter()
                .map(|peak| (peak.form_ref, peak.certificate_keys.clone()))
                .collect::<BTreeMap<_, _>>();
            if actual.keys().collect::<Vec<_>>() != expected.keys().collect::<Vec<_>>() {
                terminal_set_mismatches += 1;
            }
            if actual != expected {
                certificate_mismatches += 1;
            }
            if !forward
                .peaks
                .iter()
                .any(|peak| peak.form_ref == case.terminal_id)
            {
                target_losses += 1;
            }
        }

        let mut rank_corrupted = bytes.clone();
        let state_count = read_u32(&rank_corrupted, 176)? as usize;
        let edge_offset = HEADER_BYTES + state_count * STATE_BYTES;
        let edge = read_u64(&rank_corrupted, edge_offset)?;
        put_u64(&mut rank_corrupted, edge_offset, edge ^ (1_u64 << 40));
        let checksum: [u8; 32] = Sha256::digest(&rank_corrupted[HEADER_BYTES..]).into();
        rank_corrupted[24..56].copy_from_slice(&checksum);
        let rank_delta_corruption_false_accepts =
            usize::from(V13DafsaView::from_bytes(rank_corrupted, identity).is_ok());

        let mut symbol_ref_corrupted = bytes.clone();
        let symbol_count = read_u32(&symbol_ref_corrupted, 200)?;
        let edge = read_u64(&symbol_ref_corrupted, edge_offset)?;
        put_u64(
            &mut symbol_ref_corrupted,
            edge_offset,
            (edge & !u64::from(u16::MAX)) | u64::from(symbol_count),
        );
        let checksum: [u8; 32] = Sha256::digest(&symbol_ref_corrupted[HEADER_BYTES..]).into();
        symbol_ref_corrupted[24..56].copy_from_slice(&checksum);
        let symbol_reference_corruption_false_accepts =
            usize::from(V13DafsaView::from_bytes(symbol_ref_corrupted, identity).is_ok());

        let mut corrupted = bytes;
        *corrupted
            .last_mut()
            .ok_or_else(|| "empty tiny sidecar".to_string())? ^= 1;
        let corruption_false_accepts =
            usize::from(V13DafsaView::from_bytes(corrupted, identity).is_ok());
        let budget_result = search_typed_peaks(
            &view,
            &package,
            &cases
                .first()
                .ok_or_else(|| "generated Phase 7D matrix is empty".to_string())?
                .surface,
            SearchBudget {
                maximum_product_states: 1,
                ..SearchBudget::proof()
            },
        )?;
        let false_completeness_certificates = usize::from(!matches!(
            budget_result.completeness,
            SearchCompleteness::Unresolved(_)
        ));
        let pass = classes.len() == 13
            && terminal_set_mismatches == 0
            && certificate_mismatches == 0
            && schedule_mismatches == 0
            && v10_banded_terminal_mismatches == 0
            && v10_banded_certificate_mismatches == 0
            && v10_banded_completeness_mismatches == 0
            && v10_banded_work_mismatches == 0
            && full_row_terminal_mismatches == 0
            && full_row_certificate_mismatches == 0
            && full_row_completeness_mismatches == 0
            && full_row_work_mismatches == 0
            && target_losses == 0
            && corruption_false_accepts == 0
            && rank_delta_corruption_false_accepts == 0
            && symbol_reference_corruption_false_accepts == 0
            && false_completeness_certificates == 0
            && maximum_scratch <= MAX_QUERY_SCRATCH_BYTES;
        Ok(serde_json::json!({
            "pass": pass,
            "generated_cases": cases.len(),
            "operator_classes": classes,
            "operator_class_count": classes.len(),
            "terminal_set_mismatches": terminal_set_mismatches,
            "certificate_mismatches": certificate_mismatches,
            "schedule_permutation_mismatches": schedule_mismatches,
            "v10_banded_dla_terminal_mismatches": v10_banded_terminal_mismatches,
            "v10_banded_dla_certificate_mismatches": v10_banded_certificate_mismatches,
            "v10_banded_dla_completeness_mismatches": v10_banded_completeness_mismatches,
            "v10_banded_dla_work_mismatches": v10_banded_work_mismatches,
            "full_row_dla_terminal_mismatches": full_row_terminal_mismatches,
            "full_row_dla_certificate_mismatches": full_row_certificate_mismatches,
            "full_row_dla_completeness_mismatches": full_row_completeness_mismatches,
            "full_row_dla_work_mismatches": full_row_work_mismatches,
            "target_losses": target_losses,
            "corruption_false_accepts": corruption_false_accepts,
            "rank_delta_corruption_false_accepts": rank_delta_corruption_false_accepts,
            "symbol_reference_corruption_false_accepts": symbol_reference_corruption_false_accepts,
            "false_completeness_certificates": false_completeness_certificates,
            "maximum_query_scratch_bytes": maximum_scratch,
            "maximum_dla_states": maximum_dla_states,
            "maximum_dla_transitions": maximum_dla_transitions,
            "maximum_dla_classes": maximum_dla_classes,
            "runtime_literal_or_case_branches": 0,
            "scope": "tiny exhaustive DAFSA retrieval against independent Phase 7D dense oracle",
        }))
    }

    fn parse_v7_cases(source: &serde_json::Value) -> Result<Vec<ProofCase>, String> {
        let records = source
            .pointer("/live_cohort_compare_shadow/no_field_records")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| "V7 receipt has no conserved no_field_records".to_string())?;
        records
            .iter()
            .map(|record| {
                let string = |name: &str| {
                    record
                        .get(name)
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string)
                        .ok_or_else(|| format!("V7 record lacks {name}"))
                };
                Ok(ProofCase {
                    availability: string("availability")?,
                    class: string("class")?,
                    damaged_surface: string("damaged_surface")?,
                    proof_identity: serde_json::to_string(
                        record
                            .get("proof_identity")
                            .ok_or_else(|| "V7 record lacks proof_identity".to_string())?,
                    )
                    .map_err(|error| error.to_string())?,
                    target_surface: string("target_surface")?,
                })
            })
            .collect()
    }

    fn run_gate_c(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        bindings: &ProofBindingIndex,
        cases: &[ProofCase],
    ) -> Result<serde_json::Value, String> {
        let unique_attempts = unique_attempt_identities(cases);
        let mut by_class = BTreeMap::<String, ClassRetention>::new();
        let mut by_availability = BTreeMap::<String, usize>::new();
        let mut target_form_retained = 0_usize;
        let mut target_lemma_retained = 0_usize;
        let mut false_certificates = 0_usize;
        let mut unresolved = 0_usize;
        let mut v10_banded_terminal_mismatches = 0_usize;
        let mut v10_banded_certificate_mismatches = 0_usize;
        let mut v10_banded_completeness_mismatches = 0_usize;
        let mut v10_banded_work_mismatches = 0_usize;
        let mut full_row_terminal_mismatches = 0_usize;
        let mut full_row_certificate_mismatches = 0_usize;
        let mut full_row_completeness_mismatches = 0_usize;
        let mut full_row_work_mismatches = 0_usize;
        let mut maximum_scratch = 0_usize;
        let mut maximum_product_states = 0_usize;
        let mut maximum_dla_states = 0_usize;
        let mut maximum_dla_transitions = 0_usize;
        let mut maximum_dla_classes = 0_usize;
        let mut dla_build_latency_us = Vec::with_capacity(cases.len());
        let mut intersection_latency_us = Vec::with_capacity(cases.len());
        let mut search_latency_us = Vec::with_capacity(cases.len());
        let mut material_latency_us = Vec::with_capacity(cases.len());
        let mut total_latency_us = Vec::with_capacity(cases.len());

        for case in cases {
            *by_availability
                .entry(case.availability.clone())
                .or_default() += 1;
            let class = by_class.entry(case.class.clone()).or_default();
            class.records += 1;
            let result =
                search_typed_peaks(index, package, &case.damaged_surface, SearchBudget::proof())?;
            let banded = search_typed_peaks_banded_oracle(
                index,
                package,
                &case.damaged_surface,
                SearchBudget::proof(),
            )?;
            let full_row = search_typed_peaks_full_row(
                index,
                package,
                &case.damaged_surface,
                SearchBudget::proof(),
            )?;
            v10_banded_terminal_mismatches +=
                usize::from(result.retrieved_form_refs != banded.retrieved_form_refs);
            v10_banded_certificate_mismatches += usize::from(result.peaks != banded.peaks);
            v10_banded_completeness_mismatches +=
                usize::from(result.completeness != banded.completeness);
            v10_banded_work_mismatches +=
                usize::from(result.expanded_product_states != banded.expanded_product_states);
            full_row_terminal_mismatches +=
                usize::from(result.retrieved_form_refs != full_row.retrieved_form_refs);
            full_row_certificate_mismatches += usize::from(result.peaks != full_row.peaks);
            full_row_completeness_mismatches +=
                usize::from(result.completeness != full_row.completeness);
            full_row_work_mismatches +=
                usize::from(result.expanded_product_states != full_row.expanded_product_states);
            dla_build_latency_us.push(result.dla_build_elapsed_us);
            intersection_latency_us.push(result.intersection_elapsed_us);
            search_latency_us.push(result.search_elapsed_us);
            material_latency_us.push(result.material_elapsed_us);
            total_latency_us.push(result.total_elapsed_us);
            maximum_scratch = maximum_scratch.max(result.maximum_scratch_bytes);
            maximum_product_states = maximum_product_states.max(result.expanded_product_states);
            maximum_dla_states = maximum_dla_states.max(result.dla_states);
            maximum_dla_transitions = maximum_dla_transitions.max(result.dla_transitions);
            maximum_dla_classes = maximum_dla_classes.max(result.maximum_dla_classes);
            if result.completeness != SearchCompleteness::CertifiedExhaustive {
                unresolved += 1;
                class.unresolved += 1;
                continue;
            }

            // The target enters only after the target-blind result is frozen.
            let target_ref = package
                .form_ref_for_surface(&case.target_surface)
                .ok_or_else(|| format!("V13 lacks target {}", case.target_surface))?;
            let found_refs = result
                .peaks
                .iter()
                .map(|peak| peak.form_ref)
                .collect::<BTreeSet<_>>();
            let form_retained = found_refs.contains(&target_ref);
            target_form_retained += usize::from(form_retained);
            class.target_form_retained += usize::from(form_retained);

            let target_lemmas = bindings.lemma_ids(package, target_ref);
            let found_lemmas = found_refs
                .iter()
                .flat_map(|form_ref| bindings.lemma_ids(package, *form_ref))
                .collect::<BTreeSet<_>>();
            let lemma_retained = !target_lemmas.is_empty()
                && target_lemmas
                    .iter()
                    .any(|lemma| found_lemmas.contains(lemma));
            target_lemma_retained += usize::from(lemma_retained);
            class.target_lemma_retained += usize::from(lemma_retained);

            for peak in &result.peaks {
                let surface = package
                    .surface(peak.form_ref as usize)
                    .ok_or_else(|| format!("missing emitted V13 form {}", peak.form_ref))?;
                let independently_verified =
                    crate::nanda_wave::lexical_grokking::phase7d_certificate_keys(
                        &case.damaged_surface,
                        surface.as_ref(),
                    )?;
                if independently_verified.is_empty()
                    || independently_verified != peak.certificate_keys
                {
                    false_certificates += 1;
                    class.false_certificates += 1;
                }
            }
        }

        let concurrent = run_twenty_client_gate(index, package, cases)?;
        let class_report = by_class
            .into_iter()
            .map(|(name, metrics)| {
                (
                    name,
                    serde_json::json!({
                        "records": metrics.records,
                        "target_form_retained": metrics.target_form_retained,
                        "target_lemma_retained": metrics.target_lemma_retained,
                        "unresolved": metrics.unresolved,
                        "false_certificates": metrics.false_certificates,
                    }),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        let search_p99 = percentile(&mut search_latency_us, 99);
        let total_p99 = percentile(&mut total_latency_us, 99);
        let pass = cases.len() == 382
            && unique_attempts == 382
            && target_form_retained == cases.len()
            && target_lemma_retained == cases.len()
            && false_certificates == 0
            && unresolved == 0
            && v10_banded_terminal_mismatches == 0
            && v10_banded_certificate_mismatches == 0
            && v10_banded_completeness_mismatches == 0
            && v10_banded_work_mismatches == 0
            && full_row_terminal_mismatches == 0
            && full_row_certificate_mismatches == 0
            && full_row_completeness_mismatches == 0
            && full_row_work_mismatches == 0
            && maximum_scratch <= MAX_QUERY_SCRATCH_BYTES
            && search_p99 <= 3_000
            && total_p99 <= 5_000
            && concurrent.get("pass").and_then(serde_json::Value::as_bool) == Some(true);
        Ok(serde_json::json!({
            "pass": pass,
            "records": cases.len(),
            "unique_attempt_identities": unique_attempts,
            "target_form_retained": target_form_retained,
            "target_lemma_retained": target_lemma_retained,
            "false_certificates": false_certificates,
            "unresolved": unresolved,
            "v10_banded_dla_terminal_mismatches": v10_banded_terminal_mismatches,
            "v10_banded_dla_certificate_mismatches": v10_banded_certificate_mismatches,
            "v10_banded_dla_completeness_mismatches": v10_banded_completeness_mismatches,
            "v10_banded_dla_work_mismatches": v10_banded_work_mismatches,
            "full_row_dla_terminal_mismatches": full_row_terminal_mismatches,
            "full_row_dla_certificate_mismatches": full_row_certificate_mismatches,
            "full_row_dla_completeness_mismatches": full_row_completeness_mismatches,
            "full_row_dla_work_mismatches": full_row_work_mismatches,
            "maximum_product_states": maximum_product_states,
            "maximum_query_scratch_bytes": maximum_scratch,
            "maximum_dla_states": maximum_dla_states,
            "maximum_dla_transitions": maximum_dla_transitions,
            "maximum_dla_classes": maximum_dla_classes,
            "by_availability": by_availability,
            "by_damage_class": class_report,
            "single_client_latency_us": {
                "dla_build_p50": percentile(&mut dla_build_latency_us, 50),
                "dla_build_p99": percentile(&mut dla_build_latency_us, 99),
                "dla_build_max": maximum(&dla_build_latency_us),
                "intersection_p50": percentile(&mut intersection_latency_us, 50),
                "intersection_p99": percentile(&mut intersection_latency_us, 99),
                "intersection_max": maximum(&intersection_latency_us),
                "search_p50": percentile(&mut search_latency_us, 50),
                "search_p99": search_p99,
                "search_max": maximum(&search_latency_us),
                "material_p99": percentile(&mut material_latency_us, 99),
                "total_p50": percentile(&mut total_latency_us, 50),
                "total_p99": total_p99,
                "total_max": maximum(&total_latency_us),
            },
            "twenty_client": concurrent,
        }))
    }

    fn run_twenty_client_gate(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        cases: &[ProofCase],
    ) -> Result<serde_json::Value, String> {
        let workers = 20_usize.min(cases.len().max(1));
        let chunk_size = cases.len().div_ceil(workers);
        let wall_started = Instant::now();
        let outcomes = std::thread::scope(|scope| {
            cases
                .chunks(chunk_size)
                .map(|chunk| {
                    scope.spawn(move || {
                        chunk
                            .iter()
                            .map(|case| {
                                search_typed_peaks(
                                    index,
                                    package,
                                    &case.damaged_surface,
                                    SearchBudget::proof(),
                                )
                                .map(|result| {
                                    (
                                        result.completeness,
                                        result.dla_build_elapsed_us,
                                        result.intersection_elapsed_us,
                                        result.search_elapsed_us,
                                        result.total_elapsed_us,
                                    )
                                })
                            })
                            .collect::<Vec<_>>()
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .flat_map(|handle| handle.join().expect("V11 20-client worker panicked"))
                .collect::<Vec<_>>()
        });
        let wall_elapsed_us = elapsed_us(wall_started);
        let mut dla_build = Vec::new();
        let mut intersection = Vec::new();
        let mut search = Vec::new();
        let mut total = Vec::new();
        let mut errors = 0_usize;
        let mut unresolved = 0_usize;
        for outcome in outcomes {
            match outcome {
                Ok((completeness, dla_build_us, intersection_us, search_us, total_us)) => {
                    unresolved +=
                        usize::from(completeness != SearchCompleteness::CertifiedExhaustive);
                    dla_build.push(dla_build_us);
                    intersection.push(intersection_us);
                    search.push(search_us);
                    total.push(total_us);
                }
                Err(_) => errors += 1,
            }
        }
        let total_p99 = percentile(&mut total, 99);
        Ok(serde_json::json!({
            "pass": errors == 0 && unresolved == 0 && total_p99 <= 5_000,
            "workers": workers,
            "requests": cases.len(),
            "errors": errors,
            "unresolved": unresolved,
            "dla_build_p99_us": percentile(&mut dla_build, 99),
            "intersection_p99_us": percentile(&mut intersection, 99),
            "search_p99_us": percentile(&mut search, 99),
            "total_p99_us": total_p99,
            "total_max_us": maximum(&total),
            "wall_elapsed_us": wall_elapsed_us,
        }))
    }

    fn measure_two_process_pss(
        sidecar_path: &Path,
        identity: V13Identity,
    ) -> Result<serde_json::Value, String> {
        let sync_dir = std::env::temp_dir().join(format!(
            "lay-v11-pss-{}-{}",
            std::process::id(),
            Instant::now().elapsed().as_nanos()
        ));
        std::fs::create_dir_all(&sync_dir)
            .map_err(|error| format!("{}: {error}", sync_dir.display()))?;
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        let mut children = Vec::<Child>::new();
        for child_index in 0..2 {
            let child = Command::new(&executable)
                .args(["--ignored", "--exact", PSS_HELPER_TEST, "--nocapture"])
                .env("LAY_V11_PSS_HELPER", "1")
                .env("LAY_V11_PSS_CHILD", child_index.to_string())
                .env("LAY_V11_PSS_SYNC", &sync_dir)
                .env("LAY_V11_SIDECAR", sidecar_path)
                .env("LAY_V11_ID_SHA256", hex(identity.package_sha256))
                .env("LAY_V11_ID_BYTES", identity.package_bytes.to_string())
                .env("LAY_V11_ID_FORMS", identity.form_count.to_string())
                .env("LAY_V11_ID_BINDINGS", identity.binding_count.to_string())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|error| format!("spawn V11 PSS helper: {error}"))?;
            children.push(child);
        }
        wait_for_markers(&sync_dir, "ready", 2, Duration::from_secs(30))?;
        let before = children
            .iter()
            .map(|child| process_pss_kib(child.id()))
            .collect::<Result<Vec<_>, _>>()?;
        std::fs::write(sync_dir.join("start"), b"start").map_err(|error| error.to_string())?;
        wait_for_markers(&sync_dir, "loaded", 2, Duration::from_secs(60))?;
        let after = children
            .iter()
            .map(|child| process_pss_kib(child.id()))
            .collect::<Result<Vec<_>, _>>()?;
        std::fs::write(sync_dir.join("stop"), b"stop").map_err(|error| error.to_string())?;
        for child in &mut children {
            let status = child.wait().map_err(|error| error.to_string())?;
            if !status.success() {
                return Err(format!("V11 PSS helper exited with {status}"));
            }
        }
        let _ = std::fs::remove_dir_all(&sync_dir);
        let before_total = before.iter().sum::<u64>();
        let after_total = after.iter().sum::<u64>();
        Ok(serde_json::json!({
            "processes": 2,
            "before_kib": before,
            "after_kib": after,
            "aggregate_before_kib": before_total,
            "aggregate_after_kib": after_total,
            "aggregate_delta_kib": after_total.saturating_sub(before_total),
        }))
    }

    #[test]
    #[ignore = "child process for the V11 mmap PSS gate"]
    fn v11_mmap_pss_helper() {
        if std::env::var("LAY_V11_PSS_HELPER").as_deref() != Ok("1") {
            return;
        }
        let child = std::env::var("LAY_V11_PSS_CHILD").expect("PSS child index");
        let sync =
            std::path::PathBuf::from(std::env::var_os("LAY_V11_PSS_SYNC").expect("PSS sync"));
        let identity = V13Identity {
            package_sha256: parse_hex_32(&std::env::var("LAY_V11_ID_SHA256").expect("PSS SHA"))
                .expect("parse PSS SHA"),
            package_bytes: std::env::var("LAY_V11_ID_BYTES")
                .expect("PSS bytes")
                .parse()
                .expect("parse PSS bytes"),
            form_count: std::env::var("LAY_V11_ID_FORMS")
                .expect("PSS forms")
                .parse()
                .expect("parse PSS forms"),
            binding_count: std::env::var("LAY_V11_ID_BINDINGS")
                .expect("PSS bindings")
                .parse()
                .expect("parse PSS bindings"),
        };
        std::fs::write(sync.join(format!("ready-{child}")), b"ready").expect("PSS ready");
        wait_for_path(&sync.join("start"), Duration::from_secs(60)).expect("PSS start");
        let sidecar =
            std::path::PathBuf::from(std::env::var_os("LAY_V11_SIDECAR").expect("sidecar"));
        let view = V13DafsaView::load(&sidecar, identity).expect("PSS mmap load");
        std::hint::black_box(view.state(view.root_state).expect("PSS root"));
        std::fs::write(sync.join(format!("loaded-{child}")), b"loaded").expect("PSS loaded");
        wait_for_path(&sync.join("stop"), Duration::from_secs(60)).expect("PSS stop");
        std::hint::black_box(view);
    }

    fn wait_for_markers(
        directory: &Path,
        prefix: &str,
        count: usize,
        timeout: Duration,
    ) -> Result<(), String> {
        let deadline = Instant::now() + timeout;
        loop {
            let present = (0..count)
                .filter(|index| directory.join(format!("{prefix}-{index}")).is_file())
                .count();
            if present == count {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(format!("timed out waiting for {prefix} PSS markers"));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn wait_for_path(path: &Path, timeout: Duration) -> Result<(), String> {
        let deadline = Instant::now() + timeout;
        while !path.is_file() {
            if Instant::now() >= deadline {
                return Err(format!("timed out waiting for {}", path.display()));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }

    fn process_pss_kib(pid: u32) -> Result<u64, String> {
        let path = format!("/proc/{pid}/smaps_rollup");
        let status = std::fs::read_to_string(&path).map_err(|error| format!("{path}: {error}"))?;
        status
            .lines()
            .find_map(|line| {
                line.strip_prefix("Pss:")
                    .and_then(|value| value.split_whitespace().next())
                    .and_then(|value| value.parse().ok())
            })
            .ok_or_else(|| format!("{path} has no Pss field"))
    }

    fn sha256_file(path: &Path) -> Result<[u8; 32], String> {
        let mut file =
            std::fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let mut digest = Sha256::new();
        std::io::copy(&mut file, &mut digest).map_err(|error| error.to_string())?;
        Ok(digest.finalize().into())
    }

    fn hex(value: [u8; 32]) -> String {
        value.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn parse_hex_32(value: &str) -> Result<[u8; 32], String> {
        if value.len() != 64 {
            return Err("SHA-256 must contain 64 hexadecimal characters".to_string());
        }
        let mut output = [0_u8; 32];
        for (index, byte) in output.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
                .map_err(|error| error.to_string())?;
        }
        Ok(output)
    }

    fn percentile(values: &mut [u64], percentile: usize) -> u64 {
        if values.is_empty() {
            return 0;
        }
        values.sort_unstable();
        let index = (values.len() * percentile).div_ceil(100).saturating_sub(1);
        values[index.min(values.len() - 1)]
    }

    fn maximum(values: &[u64]) -> u64 {
        values.iter().copied().max().unwrap_or_default()
    }

    struct ByteLaneObservation {
        form_refs: Vec<u32>,
        expanded: usize,
        maximum_scratch: usize,
        unresolved: Option<&'static str>,
        work: typed_exact::StructuralWork,
    }

    fn byte_exact_search(
        index: &V13DafsaView,
        lanes: &[Phase7dRetrievalLane],
        budget: SearchBudget,
        reverse_schedule: bool,
    ) -> Result<typed_exact::ExactObservation, String> {
        let started = Instant::now();
        let mut terminal_refs = Vec::new();
        let mut expanded = 0_usize;
        let mut maximum_scratch = 0_usize;
        let mut work = typed_exact::StructuralWork::default();
        let mut rank_prefixes = Vec::new();
        let mut terminal_ranks = Vec::new();

        for lane in lanes {
            let masks = typed_exact::equality_masks(lane.symbols.as_ref());
            let outcome = byte_exact_lane(
                index,
                lane,
                &masks,
                budget,
                started,
                reverse_schedule,
                &mut rank_prefixes,
                &mut terminal_ranks,
            )?;
            work.add(outcome.work);
            expanded = expanded.saturating_add(outcome.expanded);
            maximum_scratch = maximum_scratch.max(outcome.maximum_scratch);
            if let Some(reason) = outcome.unresolved {
                return Ok(typed_exact::ExactObservation {
                    retrieved_form_refs: Vec::new(),
                    unresolved: Some(reason),
                    expanded_product_states: expanded,
                    maximum_scratch_bytes: maximum_scratch,
                    work,
                    rank_prefixes,
                    terminal_ranks,
                    transition_checks: 0,
                    terminal_distance_checks: 0,
                });
            }
            terminal_refs.extend(outcome.form_refs);
        }

        terminal_refs.sort_unstable();
        terminal_refs.dedup();
        Ok(typed_exact::ExactObservation {
            retrieved_form_refs: terminal_refs,
            unresolved: None,
            expanded_product_states: expanded,
            maximum_scratch_bytes: maximum_scratch,
            work,
            rank_prefixes,
            terminal_ranks,
            transition_checks: 0,
            terminal_distance_checks: 0,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn byte_exact_lane(
        index: &V13DafsaView,
        lane: &Phase7dRetrievalLane,
        masks: &[[u64; 2]; typed_exact::ALPHABET_SYMBOLS],
        budget: SearchBudget,
        started: Instant,
        reverse_schedule: bool,
        rank_prefixes: &mut Vec<u32>,
        terminal_ranks: &mut Vec<u32>,
    ) -> Result<ByteLaneObservation, String> {
        if let Some(reason) = validate_lane(lane) {
            return Ok(ByteLaneObservation {
                form_refs: Vec::new(),
                expanded: 0,
                maximum_scratch: 0,
                unresolved: Some(reason),
                work: typed_exact::StructuralWork::default(),
            });
        }

        let query = lane.symbols.as_ref();
        let radius = lane.maximum_levenshtein_distance;
        let mut stack = vec![typed_exact::PackedNode {
            state_id: index.root_state,
            rank_prefix: 0,
            row: typed_exact::initial_row(query.len(), radius),
        }];
        let mut form_refs = Vec::new();
        let mut expanded = 0_usize;
        let mut work = typed_exact::StructuralWork {
            stack_pushes: 1,
            ..typed_exact::StructuralWork::default()
        };
        let mut maximum_scratch = packed_scratch_bytes(&stack, &form_refs);

        while let Some(node) = stack.pop() {
            expanded = expanded.saturating_add(1);
            work.expanded_states = work.expanded_states.saturating_add(1);
            work.stack_pops = work.stack_pops.saturating_add(1);
            if expanded > budget.maximum_product_states {
                return Ok(byte_lane_unresolved(
                    form_refs,
                    expanded,
                    maximum_scratch,
                    "product_state_budget",
                    work,
                ));
            }
            if budget
                .maximum_elapsed
                .is_some_and(|maximum| started.elapsed() > maximum)
            {
                return Ok(byte_lane_unresolved(
                    form_refs,
                    expanded,
                    maximum_scratch,
                    "wall_deadline",
                    work,
                ));
            }

            let state = index.state(node.state_id)?;
            if state.terminal()
                && typed_exact::terminal_distance(node.row, query.len(), radius) <= radius
            {
                work.terminal_hits = work.terminal_hits.saturating_add(1);
                terminal_ranks.push(node.rank_prefix);
                form_refs.push(node.rank_prefix);
                if form_refs.len() > budget.maximum_terminals {
                    return Ok(byte_lane_unresolved(
                        form_refs,
                        expanded,
                        maximum_scratch,
                        "terminal_budget",
                        work,
                    ));
                }
            }

            if reverse_schedule {
                let mut children = Vec::new();
                for edge_id in index.edge_range(state)? {
                    let edge = index.edge(edge_id)?;
                    let rank_prefix = node
                        .rank_prefix
                        .checked_add(edge.rank_delta)
                        .ok_or_else(|| "byte exact rank overflows u32".to_string())?;
                    work.examined_edges = work.examined_edges.saturating_add(1);
                    rank_prefixes.push(rank_prefix);
                    let next = typed_exact::advance_for_symbol(
                        node.row,
                        masks,
                        edge.symbol,
                        radius,
                        query.len(),
                    )?;
                    if next.minimum <= radius {
                        work.surviving_edges = work.surviving_edges.saturating_add(1);
                        children.push(typed_exact::PackedNode {
                            state_id: edge.target,
                            rank_prefix,
                            row: next.state,
                        });
                    } else {
                        work.pruned_edges = work.pruned_edges.saturating_add(1);
                    }
                }
                work.stack_pushes = work.stack_pushes.saturating_add(children.len());
                stack.extend(children.into_iter().rev());
            } else {
                for edge_id in index.edge_range(state)? {
                    let edge = index.edge(edge_id)?;
                    let rank_prefix = node
                        .rank_prefix
                        .checked_add(edge.rank_delta)
                        .ok_or_else(|| "byte exact rank overflows u32".to_string())?;
                    work.examined_edges = work.examined_edges.saturating_add(1);
                    rank_prefixes.push(rank_prefix);
                    let next = typed_exact::advance_for_symbol(
                        node.row,
                        masks,
                        edge.symbol,
                        radius,
                        query.len(),
                    )?;
                    if next.minimum <= radius {
                        work.surviving_edges = work.surviving_edges.saturating_add(1);
                        work.stack_pushes = work.stack_pushes.saturating_add(1);
                        stack.push(typed_exact::PackedNode {
                            state_id: edge.target,
                            rank_prefix,
                            row: next.state,
                        });
                    } else {
                        work.pruned_edges = work.pruned_edges.saturating_add(1);
                    }
                }
            }

            maximum_scratch = maximum_scratch.max(packed_scratch_bytes(&stack, &form_refs));
            if maximum_scratch > budget.maximum_scratch_bytes {
                return Ok(byte_lane_unresolved(
                    form_refs,
                    expanded,
                    maximum_scratch,
                    "scratch_budget",
                    work,
                ));
            }
        }

        Ok(ByteLaneObservation {
            form_refs,
            expanded,
            maximum_scratch,
            unresolved: None,
            work,
        })
    }

    fn byte_lane_unresolved(
        form_refs: Vec<u32>,
        expanded: usize,
        maximum_scratch: usize,
        reason: &'static str,
        work: typed_exact::StructuralWork,
    ) -> ByteLaneObservation {
        ByteLaneObservation {
            form_refs,
            expanded,
            maximum_scratch,
            unresolved: Some(reason),
            work,
        }
    }

    fn packed_scratch_bytes(stack: &Vec<typed_exact::PackedNode>, terminals: &Vec<u32>) -> usize {
        stack
            .capacity()
            .saturating_mul(std::mem::size_of::<typed_exact::PackedNode>())
            .saturating_add(
                terminals
                    .capacity()
                    .saturating_mul(std::mem::size_of::<u32>()),
            )
    }

    fn exact_result(
        observed: &str,
        package: &RuntimeL2Package,
        exact: &typed_exact::ExactObservation,
    ) -> Result<SearchResult, String> {
        let oracle = Phase7dCertificateOracle::new(observed)?;
        if let Some(reason) = exact.unresolved {
            return Ok(SearchResult {
                retrieved_form_refs: Vec::new(),
                peaks: Vec::new(),
                completeness: SearchCompleteness::Unresolved(reason),
                expanded_product_states: exact.expanded_product_states,
                maximum_scratch_bytes: exact.maximum_scratch_bytes,
                dla_states: 0,
                dla_transitions: 0,
                maximum_dla_classes: 0,
                dla_build_elapsed_us: 0,
                intersection_elapsed_us: 0,
                search_elapsed_us: 0,
                material_elapsed_us: 0,
                total_elapsed_us: 0,
            });
        }

        let mut peaks = Vec::new();
        for form_ref in exact.retrieved_form_refs.iter().copied() {
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
        Ok(SearchResult {
            retrieved_form_refs: exact.retrieved_form_refs.clone(),
            peaks,
            completeness: SearchCompleteness::CertifiedExhaustive,
            expanded_product_states: exact.expanded_product_states,
            maximum_scratch_bytes: exact.maximum_scratch_bytes,
            dla_states: 0,
            dla_transitions: 0,
            maximum_dla_classes: 0,
            dla_build_elapsed_us: 0,
            intersection_elapsed_us: 0,
            search_elapsed_us: 0,
            material_elapsed_us: 0,
            total_elapsed_us: 0,
        })
    }

    fn exact_result_equal(left: &SearchResult, right: &SearchResult) -> bool {
        left.retrieved_form_refs == right.retrieved_form_refs
            && left.peaks == right.peaks
            && left.completeness == right.completeness
            && left.expanded_product_states == right.expanded_product_states
            && left.maximum_scratch_bytes == right.maximum_scratch_bytes
    }

    fn generic_result_equal(left: &SearchResult, right: &SearchResult) -> bool {
        left.retrieved_form_refs == right.retrieved_form_refs
            && left.peaks == right.peaks
            && left.completeness == right.completeness
            && left.expanded_product_states == right.expanded_product_states
    }

    #[test]
    #[ignore = "one fixed local M3 test-source integration proof"]
    fn m3_test_source_integration_fixed_proof() {
        let receipt_path = std::env::var_os("LAY_M3_TEST_SOURCE_INTEGRATION_RECEIPT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(
                    "docs/structural_gates/receipts/\
                     LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_\
                     M3_TEST_SOURCE_INTEGRATION_V1_2026-08-27/INTEGRATION_RECEIPT.json",
                )
            });
        let receipt = match run_m3_test_source_integration() {
            Ok(receipt) => receipt,
            Err(error) => serde_json::json!({
                "schema": "lay.m3-test-source-integration.v1",
                "verdict": "BLOCKED_PROVENANCE",
                "error": error,
                "runtime_authority_changed": false,
                "installed_lay_changed": false,
                "production_source_changed": false,
            }),
        };
        write_integration_receipt(&receipt_path, &receipt).expect("write integration receipt");
        eprintln!(
            "m3_test_source_integration_receipt={}",
            receipt_path.display()
        );
        assert_eq!(
            receipt.get("verdict").and_then(serde_json::Value::as_str),
            Some("M3_TEST_SOURCE_INTEGRATION_PASS"),
            "{}",
            serde_json::to_string_pretty(&receipt).expect("render failed receipt")
        );
    }

    fn run_m3_test_source_integration() -> Result<serde_json::Value, String> {
        let package_path = std::env::var_os("LAY_M3_TEST_SOURCE_PACKAGE")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(
                    "/home/ubu/.local/share/lay/nanda_wave/l2/LAY-L2-RU-FULL-v13.bin",
                )
            });
        let historical_sidecar_path = std::env::var_os("LAY_M3_TEST_SOURCE_HISTORICAL_SIDECAR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(
                    "/home/ubu/.local/share/lay/provenance/slice8b-v10-f6178f/artifacts/\
                         LAY-L2-RU-FULL-v13.dafsa",
                )
            });
        let v7_path = std::env::var_os("LAY_M3_TEST_SOURCE_V7")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(
                    "/home/ubu/.local/share/lay/provenance/slice8b-v10-f6178f/artifacts/\
                     slice8b-v7-fixed-13x100.json",
                )
            });

        let package_bytes = std::fs::metadata(&package_path)
            .map_err(|error| format!("{}: {error}", package_path.display()))?
            .len();
        let package_sha256 = sha256_file(&package_path)?;
        if package_bytes != EXPECTED_V13_BYTES || hex(package_sha256) != EXPECTED_V13_SHA256 {
            return Err("fixed V13 package identity drift".to_string());
        }
        let historical_bytes = std::fs::metadata(&historical_sidecar_path)
            .map_err(|error| format!("{}: {error}", historical_sidecar_path.display()))?
            .len();
        let historical_sha256 = sha256_file(&historical_sidecar_path)?;
        if historical_bytes != EXPECTED_V10_SIDECAR_BYTES
            || hex(historical_sha256) != EXPECTED_V10_SIDECAR_SHA256
        {
            return Err("historical V10 sidecar identity drift".to_string());
        }
        let v7_bytes = std::fs::metadata(&v7_path)
            .map_err(|error| format!("{}: {error}", v7_path.display()))?
            .len();
        let v7_sha256 = sha256_file(&v7_path)?;
        if v7_bytes != 1_606_189 || hex(v7_sha256) != EXPECTED_V7_SHA256 {
            return Err("fixed V7 evidence identity drift".to_string());
        }

        let package = RuntimeL2Package::load(&package_path)?;
        let identity = V13Identity {
            package_sha256,
            package_bytes,
            form_count: u32::try_from(package.form_count())
                .map_err(|_| "V13 form count exceeds u32".to_string())?,
            binding_count: u32::try_from(package.binding_count())
                .map_err(|_| "V13 binding count exceeds u32".to_string())?,
        };
        let reconstructed_sidecar = compile_sidecar(&package, identity)?;
        let reconstructed_sha256: [u8; 32] = Sha256::digest(&reconstructed_sidecar).into();
        if reconstructed_sidecar.len() != EXPECTED_V11_SIDECAR_BYTES
            || hex(reconstructed_sha256) != EXPECTED_V11_SIDECAR_SHA256
        {
            return Err(format!(
                "reconstructed V11 sidecar identity drift: {} / {}",
                reconstructed_sidecar.len(),
                hex(reconstructed_sha256)
            ));
        }
        let index = V13DafsaView::from_bytes(reconstructed_sidecar, identity)?;
        let materialized = typed_exact::TypedMaterialization::from_validated(&index)?;
        if materialized.view().identity() != identity
            || materialized.view().symbol_digest() != index.symbol_digest
            || materialized.view().root_state() != index.root_state
        {
            return Err("typed materialization identity drift".to_string());
        }

        let source: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&v7_path).map_err(|error| format!("{}: {error}", v7_path.display()))?,
        )
        .map_err(|error| format!("{}: {error}", v7_path.display()))?;
        let cases = parse_v7_cases(&source)?;
        if cases.len() != EXPECTED_FIXED_CASES {
            return Err(format!("fixed V7 case count drift: {}", cases.len()));
        }

        let mut exact_result_mismatches = 0_usize;
        let mut generic_result_mismatches = 0_usize;
        let mut work_mismatches = 0_usize;
        let mut rank_prefix_mismatches = 0_usize;
        let mut terminal_rank_mismatches = 0_usize;
        let mut transition_check_mismatches = 0_usize;
        let mut schedule_rows = Vec::new();

        for reverse_schedule in [false, true] {
            let mut byte_examined_edges = 0_usize;
            let mut typed_examined_edges = 0_usize;
            let mut byte_expanded_states = 0_usize;
            let mut typed_expanded_states = 0_usize;
            let mut transition_checks = 0_usize;
            let mut terminal_distance_checks = 0_usize;

            for case in &cases {
                let oracle = Phase7dCertificateOracle::new(&case.damaged_surface)?;
                let lanes = oracle.retrieval_lanes();
                let byte =
                    byte_exact_search(&index, &lanes, SearchBudget::proof(), reverse_schedule)?;
                let typed = typed_exact::search(
                    materialized.view(),
                    &lanes,
                    SearchBudget::proof(),
                    reverse_schedule,
                    true,
                    || false,
                )?;
                let generic = search_typed_peaks_with_kernel(
                    &index,
                    &package,
                    &case.damaged_surface,
                    SearchBudget::proof(),
                    reverse_schedule,
                    SearchKernel::BandedOracle,
                )?;
                let byte_result = exact_result(&case.damaged_surface, &package, &byte)?;
                let typed_result = exact_result(&case.damaged_surface, &package, &typed)?;

                exact_result_mismatches +=
                    usize::from(!exact_result_equal(&byte_result, &typed_result));
                generic_result_mismatches +=
                    usize::from(!generic_result_equal(&generic, &typed_result));
                work_mismatches += usize::from(byte.work != typed.work);
                rank_prefix_mismatches += usize::from(byte.rank_prefixes != typed.rank_prefixes);
                terminal_rank_mismatches +=
                    usize::from(byte.terminal_ranks != typed.terminal_ranks);
                transition_check_mismatches += usize::from(
                    typed.transition_checks != typed.work.examined_edges
                        || typed.terminal_distance_checks != typed.work.expanded_states,
                );

                byte_examined_edges = byte_examined_edges.saturating_add(byte.work.examined_edges);
                typed_examined_edges =
                    typed_examined_edges.saturating_add(typed.work.examined_edges);
                byte_expanded_states =
                    byte_expanded_states.saturating_add(byte.work.expanded_states);
                typed_expanded_states =
                    typed_expanded_states.saturating_add(typed.work.expanded_states);
                transition_checks = transition_checks.saturating_add(typed.transition_checks);
                terminal_distance_checks =
                    terminal_distance_checks.saturating_add(typed.terminal_distance_checks);
            }

            if byte_examined_edges != EXPECTED_FIXED_EXAMINED_EDGES
                || typed_examined_edges != EXPECTED_FIXED_EXAMINED_EDGES
                || byte_expanded_states != EXPECTED_FIXED_EXPANDED_STATES
                || typed_expanded_states != EXPECTED_FIXED_EXPANDED_STATES
                || transition_checks != EXPECTED_FIXED_EXAMINED_EDGES
                || terminal_distance_checks != EXPECTED_FIXED_EXPANDED_STATES
            {
                return Err(format!(
                    "fixed structural denominator drift: edges={byte_examined_edges}/{typed_examined_edges}/{transition_checks} expanded={byte_expanded_states}/{typed_expanded_states}/{terminal_distance_checks}"
                ));
            }
            schedule_rows.push(serde_json::json!({
                "schedule": if reverse_schedule { "REVERSED" } else { "FORWARD" },
                "queries": cases.len(),
                "byte_examined_edges": byte_examined_edges,
                "typed_examined_edges": typed_examined_edges,
                "byte_expanded_states": byte_expanded_states,
                "typed_expanded_states": typed_expanded_states,
                "transition_checks": transition_checks,
                "terminal_distance_checks": terminal_distance_checks,
            }));
        }

        let mismatch_total = exact_result_mismatches
            + generic_result_mismatches
            + work_mismatches
            + rank_prefix_mismatches
            + terminal_rank_mismatches
            + transition_check_mismatches;
        let pass = mismatch_total == 0
            && materialized.states_checked() == typed_exact::EXPECTED_STATE_COUNT
            && materialized.edges_checked() == typed_exact::EXPECTED_EDGE_COUNT
            && materialized.payload_bytes() == typed_exact::EXPECTED_TYPED_PAYLOAD_BYTES
            && materialized.view().state_count() == typed_exact::EXPECTED_STATE_COUNT
            && materialized.view().edge_count() == typed_exact::EXPECTED_EDGE_COUNT;

        Ok(serde_json::json!({
            "schema": "lay.m3-test-source-integration.v1",
            "verdict": if pass { "M3_TEST_SOURCE_INTEGRATION_PASS" } else { "BLOCKED_PARITY" },
            "source": {
                "package_path": package_path,
                "package_bytes": package_bytes,
                "package_sha256": hex(package_sha256),
                "historical_v10_sidecar_path": historical_sidecar_path,
                "historical_v10_sidecar_bytes": historical_bytes,
                "historical_v10_sidecar_sha256": hex(historical_sha256),
                "v7_path": v7_path,
                "v7_bytes": v7_bytes,
                "v7_sha256": hex(v7_sha256),
            },
            "v11_reconstruction": {
                "count": 1,
                "in_memory_only": true,
                "sidecar_file_written": false,
                "bytes": EXPECTED_V11_SIDECAR_BYTES,
                "sha256": hex(reconstructed_sha256),
                "magic": "LAYV13D3",
                "encoded_record_bytes": {"state": STATE_BYTES, "edge": EDGE_BYTES, "symbol": SYMBOL_BYTES},
            },
            "typed_materialization": {
                "count": 1,
                "states_checked": materialized.states_checked(),
                "edges_checked": materialized.edges_checked(),
                "state_record_bytes": 12,
                "edge_record_bytes": 12,
                "payload_bytes": materialized.payload_bytes(),
                "root_state": materialized.view().root_state(),
                "identity_match": true,
                "symbol_digest_match": true,
            },
            "proof": {
                "fixed_cases": cases.len(),
                "schedules": schedule_rows,
                "exact_result_mismatches": exact_result_mismatches,
                "generic_result_mismatches": generic_result_mismatches,
                "work_mismatches": work_mismatches,
                "rank_prefix_mismatches": rank_prefix_mismatches,
                "terminal_rank_mismatches": terminal_rank_mismatches,
                "transition_check_mismatches": transition_check_mismatches,
                "total_mismatches": mismatch_total,
            },
            "claim_boundary": {
                "m3_machine_gain_transferred": false,
                "end_to_end_latency_proven": false,
                "production_generation_owner_proven": false,
                "actual_owner_paper_only_if_pass": pass,
            },
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
            "production_source_changed": false,
            "network_or_remote_used": false,
            "perf_or_pmu_used": false,
        }))
    }

    #[test]
    #[ignore = "one fixed local M3 actual-owner parity proof"]
    fn m3_actual_owner_fixed_proof() {
        let receipt_path = std::env::var_os("LAY_M3_ACTUAL_OWNER_RECEIPT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(
                    "docs/structural_gates/receipts/\
                     LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_\
                     M3_ACTUAL_OWNER_V1_2026-08-27/OWNER_RECEIPT.json",
                )
            });
        let receipt = match run_m3_actual_owner_proof() {
            Ok(receipt) => receipt,
            Err(error) => serde_json::json!({
                "schema": "lay.m3-actual-owner-parity.v2",
                "verdict": "BLOCKED_PROVENANCE",
                "error": error,
                "runtime_authority_changed": false,
                "installed_lay_changed": false,
                "production_activation_admitted": false,
            }),
        };
        write_integration_receipt(&receipt_path, &receipt).expect("write actual-owner receipt");
        eprintln!("m3_actual_owner_receipt={}", receipt_path.display());
        assert_eq!(
            receipt.get("verdict").and_then(serde_json::Value::as_str),
            Some("M3_ACTUAL_OWNER_PARITY_PASS"),
            "{}",
            serde_json::to_string_pretty(&receipt).expect("render failed owner receipt")
        );
    }

    #[test]
    #[ignore = "one fixed local M3 actual-owner semantic diagnosis"]
    fn m3_actual_owner_semantic_diagnosis() {
        let receipt_path = std::env::var_os("LAY_M3_ACTUAL_OWNER_DIAGNOSIS_RECEIPT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(
                    "docs/structural_gates/receipts/\
                     LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_\
                     M3_ACTUAL_OWNER_SEMANTIC_DIAGNOSIS_V1_2026-08-27/DIAGNOSIS_RECEIPT.json",
                )
            });
        let mut receipt = match run_m3_actual_owner_proof() {
            Ok(receipt) => receipt,
            Err(error) => serde_json::json!({
                "schema": "lay.m3-actual-owner-semantic-diagnosis.v1",
                "verdict": "BLOCKED_PROVENANCE",
                "diagnostic_verdict": "BLOCKED_PROVENANCE",
                "error": error,
                "runtime_authority_changed": false,
                "installed_lay_changed": false,
                "production_activation_admitted": false,
            }),
        };
        let fixed = receipt.get("fixed_proof");
        let diagnosis_complete = receipt.get("verdict").and_then(serde_json::Value::as_str)
            == Some("BLOCKED_SEMANTIC")
            && fixed
                .and_then(|value| value.get("owner_requests_completed"))
                .and_then(serde_json::Value::as_u64)
                == Some((EXPECTED_FIXED_CASES * 2) as u64)
            && fixed
                .and_then(|value| value.get("candidate_mismatches"))
                .and_then(serde_json::Value::as_u64)
                == Some(0)
            && fixed
                .and_then(|value| value.get("certificate_mismatches"))
                .and_then(serde_json::Value::as_u64)
                == Some(0)
            && receipt
                .pointer("/semantic_diagnosis/mismatch_samples")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|samples| !samples.is_empty());
        receipt
            .as_object_mut()
            .expect("actual-owner diagnosis receipt must be an object")
            .insert(
                "diagnostic_verdict".to_string(),
                serde_json::Value::String(
                    if diagnosis_complete {
                        "M3_ACTUAL_OWNER_SEMANTIC_DIAGNOSIS_COMPLETE"
                    } else {
                        "BLOCKED_DIAGNOSIS"
                    }
                    .to_string(),
                ),
            );
        write_integration_receipt(&receipt_path, &receipt)
            .expect("write actual-owner diagnosis receipt");
        eprintln!(
            "m3_actual_owner_diagnosis_receipt={}",
            receipt_path.display()
        );
        assert_eq!(
            receipt
                .get("diagnostic_verdict")
                .and_then(serde_json::Value::as_str),
            Some("M3_ACTUAL_OWNER_SEMANTIC_DIAGNOSIS_COMPLETE"),
            "{}",
            serde_json::to_string_pretty(&receipt).expect("render failed diagnosis receipt")
        );
    }

    fn run_m3_actual_owner_proof() -> Result<serde_json::Value, String> {
        const EXPECTED_PRODUCTIVE_V90_SHA256: &str =
            "40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44";
        const EXPECTED_L11_SHA256: &str =
            "bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7";
        const PROTECTED_FILES: [(&str, &str); 7] = [
            (
                "src/nanda_wave/l2_field/mod.rs",
                "c76c28debdac8e43d360a5a82811ea86e3e6f03b98e4f81cb6678e07953e0953",
            ),
            (
                "src/nanda_wave/l2_field/bridge.rs",
                "5f3cd350c59b0b84a6f2250077f4d3e6f061c93a548451c9aafe4cf0f5f820ad",
            ),
            (
                "src/nanda_wave/l2_field/cache.rs",
                "9da969bfff12dba0217954647b1ba8e21302365770abaa82178953fcf63fec07",
            ),
            (
                "src/nanda_wave/l2_field/v13_typed_peak/typed_exact.rs",
                "325bdd386b13de77bd030ef1af0fecb12be8bdd2bdd3c841d10257647a4ceaf4",
            ),
            (
                "Cargo.toml",
                "90a3ae6b16677ff49b70aab3457b794cd1e114a547f9e113a8b49595f976207b",
            ),
            (
                "Cargo.lock",
                "e6399be48c393e1557b451fcb04d886f9fbdaa9812109410ac3d9ecdc98b93f1",
            ),
            (
                "scripts/cargo-guard.sh",
                "a5b01f9a12b32ee2bcfc957a5e9f65efb52fe6e7644850ae3f00e412648243fe",
            ),
        ];

        let package_path = std::env::var_os("LAY_M3_ACTUAL_OWNER_PACKAGE")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(
                    "/home/ubu/.local/share/lay/nanda_wave/l2/LAY-L2-RU-FULL-v13.bin",
                )
            });
        let v7_path = std::env::var_os("LAY_M3_ACTUAL_OWNER_V7")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(
                    "/home/ubu/.local/share/lay/provenance/slice8b-v10-f6178f/artifacts/\
                     slice8b-v7-fixed-13x100.json",
                )
            });
        let package_bytes = std::fs::metadata(&package_path)
            .map_err(|error| format!("{}: {error}", package_path.display()))?
            .len();
        let package_sha256 = sha256_file(&package_path)?;
        if package_bytes != EXPECTED_V13_BYTES || hex(package_sha256) != EXPECTED_V13_SHA256 {
            return Err("fixed V13 package identity drift".to_string());
        }
        let v7_bytes = std::fs::metadata(&v7_path)
            .map_err(|error| format!("{}: {error}", v7_path.display()))?
            .len();
        let v7_sha256 = sha256_file(&v7_path)?;
        if v7_bytes != 1_606_189 || hex(v7_sha256) != EXPECTED_V7_SHA256 {
            return Err("fixed V7 evidence identity drift".to_string());
        }
        let mut protected_rows = Vec::new();
        for (path, expected_sha256) in PROTECTED_FILES {
            let actual_sha256 = hex(sha256_file(Path::new(path))?);
            if actual_sha256 != expected_sha256 {
                return Err(format!("protected source identity drift: {path}"));
            }
            protected_rows.push(serde_json::json!({
                "path": path,
                "sha256": actual_sha256,
            }));
        }

        let package = RuntimeL2Package::load(&package_path)?;
        let identity = V13Identity {
            package_sha256,
            package_bytes,
            form_count: u32::try_from(package.form_count())
                .map_err(|_| "V13 form count exceeds u32".to_string())?,
            binding_count: u32::try_from(package.binding_count())
                .map_err(|_| "V13 binding count exceeds u32".to_string())?,
        };
        let reconstructed_sidecar = compile_sidecar(&package, identity)?;
        let reconstructed_sha256: [u8; 32] = Sha256::digest(&reconstructed_sidecar).into();
        if reconstructed_sidecar.len() != EXPECTED_V11_SIDECAR_BYTES {
            return Err(format!(
                "reconstructed V11 sidecar size drift: {}",
                reconstructed_sidecar.len()
            ));
        }
        let current_semantics_sha256 = phase7d_semantics_digest();
        if hex(current_semantics_sha256) != EXPECTED_CURRENT_PHASE7D_SOURCE_SHA256
            || reconstructed_sidecar[112..144] != current_semantics_sha256
        {
            return Err("current Phase 7D source-bound header identity drift".to_string());
        }
        let payload_sha256_from_header: [u8; 32] = reconstructed_sidecar[24..56]
            .try_into()
            .expect("fixed V11 payload digest width");
        let payload_sha256_recomputed: [u8; 32] =
            Sha256::digest(&reconstructed_sidecar[HEADER_BYTES..]).into();
        if payload_sha256_from_header != payload_sha256_recomputed {
            return Err("reconstructed V11 payload checksum mismatch".to_string());
        }
        let mut historical_projection = reconstructed_sidecar.clone();
        historical_projection[112..144]
            .copy_from_slice(&parse_hex_32(EXPECTED_HISTORICAL_PHASE7D_SOURCE_SHA256)?);
        let historical_projection_sha256: [u8; 32] = Sha256::digest(&historical_projection).into();
        if hex(historical_projection_sha256) != EXPECTED_V11_SIDECAR_SHA256 {
            return Err(format!(
                "historical-header V11 projection identity drift: {}",
                hex(historical_projection_sha256)
            ));
        }
        let index = V13DafsaView::from_bytes(reconstructed_sidecar, identity)?;
        let materialized = typed_exact::TypedMaterialization::from_validated(&index)?;
        if materialized.view().identity() != identity
            || materialized.view().symbol_digest() != index.symbol_digest
            || materialized.view().root_state() != index.root_state
        {
            return Err("typed materialization identity drift".to_string());
        }

        let source: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&v7_path).map_err(|error| format!("{}: {error}", v7_path.display()))?,
        )
        .map_err(|error| format!("{}: {error}", v7_path.display()))?;
        let cases = parse_v7_cases(&source)?;
        if cases.len() != EXPECTED_FIXED_CASES {
            return Err(format!("fixed V7 case count drift: {}", cases.len()));
        }

        let canonical_index = super::super::installed_l2_field().map_err(str::to_string)?;
        let productive = super::super::installed_productive_l2_v1()?;
        if canonical_index.form_count() != EXPECTED_V13_FORMS
            || hex(productive.package_sha256()) != EXPECTED_PRODUCTIVE_V90_SHA256
            || hex(productive.l11_package_sha256()) != EXPECTED_L11_SHA256
            || productive.canonical_l2_package_sha256() != package_sha256
        {
            return Err("actual owner package tuple drift".to_string());
        }

        let empty_case = cases
            .first()
            .ok_or_else(|| "fixed V7 case set is empty".to_string())?;
        let live_empty = prepare_live_productive_v1_field(
            "",
            &empty_case.damaged_surface,
            canonical_index,
            productive.as_ref(),
            &[],
            &[],
            &[],
        )?;
        let explicit_empty = prepare_live_productive_v1_field_with_exact_peaks(
            "",
            &empty_case.damaged_surface,
            canonical_index,
            productive.as_ref(),
            &[],
            &[],
            &[],
            ExactPeakBirthEnumerationV1::complete_empty(),
        )?;
        let empty_lane_mismatches = usize::from(live_empty != explicit_empty);

        type CandidateRows = Vec<(u32, String)>;
        type CertificateRows = Vec<(u32, String, u8, String)>;
        let mut forward_rows = vec![None::<(CandidateRows, CertificateRows)>; cases.len()];
        let mut owner_elapsed_us = Vec::<u64>::with_capacity(cases.len() * 2);
        let mut candidate_mismatches = 0_usize;
        let mut certificate_mismatches = 0_usize;
        let mut structured_certificate_mismatches = 0_usize;
        let mut schedule_mismatches = 0_usize;
        let mut completeness_mismatches = 0_usize;
        let mut lattice_marker_mismatches = 0_usize;
        let mut emitted_surface_mismatches = 0_usize;
        let mut gate_mismatches = 0_usize;
        let mut owner_identity_mismatches = 0_usize;
        let mut adapter_errors = 0_usize;
        let mut certificate_ref_collisions = 0_usize;
        let mut capacity_failures = 0_usize;
        let mut owner_requests_completed = 0_usize;
        let mut maximum_candidates = 0_usize;
        let mut maximum_certificates_per_target = 0_usize;
        let mut maximum_certificates_per_request = 0_usize;
        let mut certificate_table_bytes = 0_u64;
        let mut failure_samples = Vec::<String>::new();
        let mut lattice_marker_mismatches_by_class = BTreeMap::<String, usize>::new();
        let mut emitted_surface_mismatches_by_class = BTreeMap::<String, usize>::new();
        let mut gate_mismatches_by_class = BTreeMap::<String, usize>::new();
        let mut mismatch_samples = Vec::<serde_json::Value>::new();
        let normalized_raw_difference_cases = cases
            .iter()
            .filter(|case| {
                super::super::compositional::normalize_surface(&case.damaged_surface)
                    != case.damaged_surface
            })
            .count();

        for reverse_schedule in [false, true] {
            for (case_index, case) in cases.iter().enumerate() {
                let oracle = Phase7dCertificateOracle::new(&case.damaged_surface)?;
                let typed = typed_exact::search(
                    materialized.view(),
                    &oracle.retrieval_lanes(),
                    SearchBudget::proof(),
                    reverse_schedule,
                    true,
                    || false,
                )?;
                let result = exact_result(&case.damaged_surface, &package, &typed)?;
                if result.completeness != SearchCompleteness::CertifiedExhaustive {
                    completeness_mismatches = completeness_mismatches.saturating_add(1);
                    continue;
                }

                let mut inputs = Vec::<ExactPeakCandidateInputV1>::new();
                let mut expected_candidates = BTreeSet::<(u32, String)>::new();
                let mut expected_certificates = BTreeSet::<(u32, String, u8, String)>::new();
                let mut roots_by_surface = BTreeMap::<String, usize>::new();
                for peak in &result.peaks {
                    let surface = package
                        .surface(peak.form_ref as usize)
                        .ok_or_else(|| format!("missing V13 form {}", peak.form_ref))?;
                    let normalized =
                        super::super::compositional::normalize_surface(surface.as_ref());
                    let evidence = oracle.certificate_evidence(surface.as_ref())?;
                    let evidence_keys = evidence
                        .iter()
                        .map(|certificate| certificate.canonical_key.clone())
                        .collect::<Vec<_>>();
                    structured_certificate_mismatches = structured_certificate_mismatches
                        .saturating_add(usize::from(evidence_keys != peak.certificate_keys));
                    expected_candidates.insert((peak.form_ref, normalized.clone()));
                    for certificate in &evidence {
                        expected_certificates.insert((
                            peak.form_ref,
                            normalized.clone(),
                            certificate.class as u8,
                            certificate.canonical_key.clone(),
                        ));
                        certificate_table_bytes = certificate_table_bytes
                            .saturating_add(certificate.canonical_key.len() as u64)
                            .saturating_add(9);
                        *roots_by_surface.entry(normalized.clone()).or_default() += 1;
                    }
                    inputs.push(ExactPeakCandidateInputV1 {
                        form_ref: peak.form_ref,
                        normalized_surface: normalized,
                        certificates: evidence,
                    });
                }
                let expected_candidates = expected_candidates.into_iter().collect::<Vec<_>>();
                let expected_certificates = expected_certificates.into_iter().collect::<Vec<_>>();
                maximum_candidates = maximum_candidates.max(expected_candidates.len());
                maximum_certificates_per_request =
                    maximum_certificates_per_request.max(expected_certificates.len());
                maximum_certificates_per_target = maximum_certificates_per_target
                    .max(roots_by_surface.values().copied().max().unwrap_or_default());
                if reverse_schedule {
                    schedule_mismatches = schedule_mismatches.saturating_add(usize::from(
                        forward_rows[case_index].as_ref()
                            != Some(&(expected_candidates.clone(), expected_certificates.clone())),
                    ));
                } else {
                    forward_rows[case_index] =
                        Some((expected_candidates.clone(), expected_certificates.clone()));
                }

                let exact_peaks = match ExactPeakBirthEnumerationV1::from_candidates(inputs) {
                    Ok(exact_peaks) => exact_peaks,
                    Err(error) => {
                        adapter_errors = adapter_errors.saturating_add(1);
                        certificate_ref_collisions = certificate_ref_collisions
                            .saturating_add(usize::from(error.contains("collision")));
                        if failure_samples.len() < 8 {
                            failure_samples.push(error);
                        }
                        continue;
                    }
                };
                if exact_peaks.capacity_exceeded() {
                    capacity_failures = capacity_failures.saturating_add(1);
                    continue;
                }

                let owner_started = Instant::now();
                let field = match prepare_live_productive_v1_field_with_exact_peaks(
                    "",
                    &case.damaged_surface,
                    canonical_index,
                    productive.as_ref(),
                    &[],
                    &[],
                    &[],
                    exact_peaks,
                ) {
                    Ok(field) => field,
                    Err(error) => {
                        if error.contains("capacity") || error.contains("StorageCapacity") {
                            capacity_failures = capacity_failures.saturating_add(1);
                        } else {
                            owner_identity_mismatches = owner_identity_mismatches.saturating_add(1);
                        }
                        if failure_samples.len() < 8 {
                            failure_samples.push(error);
                        }
                        continue;
                    }
                };
                owner_elapsed_us.push(
                    owner_started
                        .elapsed()
                        .as_micros()
                        .min(u128::from(u64::MAX)) as u64,
                );
                owner_requests_completed = owner_requests_completed.saturating_add(1);
                candidate_mismatches = candidate_mismatches.saturating_add(usize::from(
                    field.exact_peak_candidate_rows() != expected_candidates,
                ));
                certificate_mismatches = certificate_mismatches.saturating_add(usize::from(
                    field.exact_peak_certificate_rows() != expected_certificates,
                ));
                let material_completeness = field.exact_peak_material_completeness();
                completeness_mismatches = completeness_mismatches.saturating_add(usize::from(
                    material_completeness.state()
                        != crate::typing_transition::target_evidence::EnumerationStateV1::Complete,
                ));
                owner_identity_mismatches = owner_identity_mismatches.saturating_add(usize::from(
                    field.productive_package_sha256() != productive.package_sha256()
                        || field.material_scope()
                            != PreparedFieldMaterialScopeV1::ContextShapedObservation,
                ));

                let observed_normalized =
                    super::super::compositional::normalize_surface(&case.damaged_surface);
                let expected_lattice = expected_candidates
                    .iter()
                    .map(|(_, surface)| surface.clone())
                    .filter(|surface| !surface.eq_ignore_ascii_case(&observed_normalized))
                    .collect::<BTreeSet<_>>();
                let actual_lattice = field
                    .exact_peak_lattice_surfaces()
                    .into_iter()
                    .map(str::to_string)
                    .collect::<BTreeSet<_>>();
                if actual_lattice != expected_lattice {
                    lattice_marker_mismatches = lattice_marker_mismatches.saturating_add(1);
                    *lattice_marker_mismatches_by_class
                        .entry(case.class.clone())
                        .or_default() += 1;
                    if mismatch_samples.len() < 256 {
                        mismatch_samples.push(serde_json::json!({
                            "stage": "lattice_marker",
                            "schedule": if reverse_schedule { "REVERSED" } else { "FORWARD" },
                            "case_index": case_index,
                            "class": case.class,
                            "damaged_surface": case.damaged_surface,
                            "normalized_damaged_surface": observed_normalized,
                            "expected_surfaces": expected_lattice,
                            "actual_surfaces": actual_lattice,
                        }));
                    }
                }

                let readout = materialize_live_productive_v1_field(
                    &case.damaged_surface,
                    &case.damaged_surface,
                    &field,
                )?;
                let exact_candidates = readout
                    .candidates
                    .iter()
                    .filter(|candidate| candidate.source_id == PRODUCTIVE_V90_TYPED_EXACT_SOURCE_ID)
                    .collect::<Vec<_>>();
                let actual_emitted = exact_candidates
                    .iter()
                    .map(|candidate| {
                        super::super::compositional::normalize_surface(&candidate.replacement)
                    })
                    .collect::<BTreeSet<_>>();
                if actual_emitted != expected_lattice {
                    emitted_surface_mismatches = emitted_surface_mismatches.saturating_add(1);
                    *emitted_surface_mismatches_by_class
                        .entry(case.class.clone())
                        .or_default() += 1;
                    if mismatch_samples.len() < 256 {
                        let missing = expected_lattice
                            .difference(&actual_emitted)
                            .cloned()
                            .collect::<BTreeSet<_>>();
                        let unexpected = actual_emitted
                            .difference(&expected_lattice)
                            .cloned()
                            .collect::<BTreeSet<_>>();
                        let relevant_candidates = readout
                            .candidates
                            .iter()
                            .filter_map(|candidate| {
                                let normalized = super::super::compositional::normalize_surface(
                                    &candidate.replacement,
                                );
                                (missing.contains(&normalized) || unexpected.contains(&normalized))
                                    .then(|| {
                                        serde_json::json!({
                                            "normalized_replacement": normalized,
                                            "replacement": candidate.replacement,
                                            "source_id": candidate.source_id,
                                            "origin": format!("{:?}", candidate.origin),
                                            "gate_action": format!("{:?}", candidate.gate.action),
                                            "gate_reason": candidate.gate.reason,
                                        })
                                    })
                            })
                            .collect::<Vec<_>>();
                        mismatch_samples.push(serde_json::json!({
                            "stage": "emitted_surface",
                            "schedule": if reverse_schedule { "REVERSED" } else { "FORWARD" },
                            "case_index": case_index,
                            "class": case.class,
                            "damaged_surface": case.damaged_surface,
                            "normalized_damaged_surface": observed_normalized,
                            "expected_surfaces": expected_lattice,
                            "actual_surfaces": actual_emitted,
                            "missing_surfaces": missing,
                            "unexpected_surfaces": unexpected,
                            "relevant_candidates": relevant_candidates,
                        }));
                    }
                }
                for candidate in exact_candidates {
                    let normalized =
                        super::super::compositional::normalize_surface(&candidate.replacement);
                    let independently_authorized =
                        field.exact_peak_surface_has_independent_authority(&normalized);
                    if candidate.gate.action != CandidateGateAction::SuggestOnly
                        && !(candidate.gate.action == CandidateGateAction::Eligible
                            && independently_authorized)
                    {
                        gate_mismatches = gate_mismatches.saturating_add(1);
                        *gate_mismatches_by_class
                            .entry(case.class.clone())
                            .or_default() += 1;
                        if mismatch_samples.len() < 256 {
                            mismatch_samples.push(serde_json::json!({
                                "stage": "gate",
                                "schedule": if reverse_schedule { "REVERSED" } else { "FORWARD" },
                                "case_index": case_index,
                                "class": case.class,
                                "damaged_surface": case.damaged_surface,
                                "normalized_damaged_surface": observed_normalized,
                                "normalized_replacement": normalized,
                                "replacement": candidate.replacement,
                                "source_id": candidate.source_id,
                                "origin": format!("{:?}", candidate.origin),
                                "gate_action": format!("{:?}", candidate.gate.action),
                                "gate_reason": candidate.gate.reason,
                                "independently_authorized": independently_authorized,
                            }));
                        }
                    }
                }
            }
        }

        let fixed_owner_denominator = cases.len().saturating_mul(2);
        let lattice_mismatches =
            lattice_marker_mismatches.saturating_add(emitted_surface_mismatches);
        let semantic_mismatches = candidate_mismatches
            .saturating_add(certificate_mismatches)
            .saturating_add(structured_certificate_mismatches)
            .saturating_add(schedule_mismatches)
            .saturating_add(completeness_mismatches)
            .saturating_add(lattice_mismatches)
            .saturating_add(gate_mismatches)
            .saturating_add(empty_lane_mismatches);
        let verdict = if capacity_failures > 0 {
            "BLOCKED_CAPACITY"
        } else if adapter_errors > 0
            || certificate_ref_collisions > 0
            || certificate_mismatches > 0
            || structured_certificate_mismatches > 0
        {
            "BLOCKED_CERTIFICATE"
        } else if owner_identity_mismatches > 0 {
            "BLOCKED_OWNER_IDENTITY"
        } else if semantic_mismatches > 0 || owner_requests_completed != fixed_owner_denominator {
            "BLOCKED_SEMANTIC"
        } else {
            "M3_ACTUAL_OWNER_PARITY_PASS"
        };

        Ok(serde_json::json!({
            "schema": "lay.m3-actual-owner-parity.v3",
            "verdict": verdict,
            "source": {
                "v13_package": package_path,
                "v13_bytes": package_bytes,
                "v13_sha256": hex(package_sha256),
                "v7_fixed_proof": v7_path,
                "v7_bytes": v7_bytes,
                "v7_sha256": hex(v7_sha256),
                "productive_v90_sha256": hex(productive.package_sha256()),
                "l11_sha256": hex(productive.l11_package_sha256()),
                "canonical_l2_sha256": hex(productive.canonical_l2_package_sha256()),
                "protected_files": protected_rows,
            },
            "generation_owner": {
                "typed_materializations": 1,
                "sidecar_reconstructions": 1,
                "sidecar_file_written": false,
                "reconstructed_sidecar_bytes": EXPECTED_V11_SIDECAR_BYTES,
                "reconstructed_sidecar_sha256": hex(reconstructed_sha256),
                "sidecar_identity": {
                    "historical_full_sha256": EXPECTED_V11_SIDECAR_SHA256,
                    "historical_semantics_source_sha256": EXPECTED_HISTORICAL_PHASE7D_SOURCE_SHA256,
                    "current_full_sha256": hex(reconstructed_sha256),
                    "current_semantics_source_sha256": hex(current_semantics_sha256),
                    "payload_sha256_from_header": hex(payload_sha256_from_header),
                    "payload_sha256_recomputed": hex(payload_sha256_recomputed),
                    "historical_projection_full_sha256": hex(historical_projection_sha256),
                    "historical_projection_match": true,
                    "projection_changed_byte_range": "112..144",
                    "projected_historical_clone_consumed": false,
                    "current_state_count": read_u32(index.bytes.as_slice(), 176)?,
                    "current_edge_count": read_u32(index.bytes.as_slice(), 180)?,
                    "current_root_state": read_u32(index.bytes.as_slice(), 188)?,
                    "current_symbol_count": read_u32(index.bytes.as_slice(), 200)?,
                    "current_symbol_digest_sha256": hex(index.symbol_digest),
                },
                "typed_states": materialized.states_checked(),
                "typed_edges": materialized.edges_checked(),
                "typed_payload_bytes": materialized.payload_bytes(),
                "per_request_typed_materializations": 0,
                "global_owner_created": false,
            },
            "fixed_proof": {
                "cases": cases.len(),
                "schedules": 2,
                "owner_request_denominator": fixed_owner_denominator,
                "owner_requests_completed": owner_requests_completed,
                "candidate_mismatches": candidate_mismatches,
                "certificate_mismatches": certificate_mismatches,
                "structured_certificate_mismatches": structured_certificate_mismatches,
                "schedule_mismatches": schedule_mismatches,
                "completeness_mismatches": completeness_mismatches,
                "lattice_mismatches": lattice_mismatches,
                "gate_mismatches": gate_mismatches,
                "empty_lane_mismatches": empty_lane_mismatches,
                "owner_identity_mismatches": owner_identity_mismatches,
                "adapter_errors": adapter_errors,
                "certificate_ref_collisions": certificate_ref_collisions,
                "capacity_failures": capacity_failures,
                "maximum_candidates_per_request": maximum_candidates,
                "maximum_certificates_per_target": maximum_certificates_per_target,
                "maximum_certificates_per_request": maximum_certificates_per_request,
                "certificate_table_bytes_observed": certificate_table_bytes,
                "failure_samples": failure_samples,
            },
            "semantic_diagnosis": {
                "normalized_raw_difference_cases": normalized_raw_difference_cases,
                "lattice_marker_mismatches": lattice_marker_mismatches,
                "emitted_surface_mismatches": emitted_surface_mismatches,
                "gate_mismatches": gate_mismatches,
                "lattice_marker_mismatches_by_class": lattice_marker_mismatches_by_class,
                "emitted_surface_mismatches_by_class": emitted_surface_mismatches_by_class,
                "gate_mismatches_by_class": gate_mismatches_by_class,
                "mismatch_sample_limit": 256,
                "mismatch_samples_recorded": mismatch_samples.len(),
                "mismatch_samples": mismatch_samples,
                "owner_behavior_changed_for_diagnosis": false,
            },
            "diagnostic_only": {
                "owner_prepare_p50_us": percentile(&mut owner_elapsed_us.clone(), 50),
                "owner_prepare_p99_us": percentile(&mut owner_elapsed_us, 99),
                "not_end_to_end_latency": true,
            },
            "claim_boundary": {
                "candidate_and_certificate_parity_only": true,
                "machine_gain_transferred": false,
                "end_to_end_latency_proven": false,
                "rss_admitted": false,
                "reload_generation_identity_admitted": false,
                "production_authority_admitted": false,
                "next_if_pass": "separate end-to-end latency RSS reload preflight",
            },
            "network_or_remote_used": false,
            "perf_or_pmu_used": false,
            "installed_lay_changed": false,
            "runtime_authority_changed": false,
            "production_activation_admitted": false,
        }))
    }

    mod m3_v8 {
        use super::*;
        use crate::nanda_wave::l2_field::productive_v1::PackagedProductiveRuntimeV1;
        use crate::nanda_wave::l2_field::runtime::StandaloneL2Field;
        use std::sync::{Arc, RwLock};

        const PSS_HELPER_TEST: &str =
            "nanda_wave::l2_field::v13_typed_peak::tests::m3_v8::m3_end_to_end_pss_helper";
        const EXPECTED_PRODUCTIVE_V90_SHA256: &str =
            "40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44";
        const EXPECTED_L11_SHA256: &str =
            "bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7";
        const EXPECTED_TYPED_PAYLOAD_BYTES: usize = 3_689_628;
        const SEARCH_P99_GATE_US: u64 = 3_000;
        const TOTAL_P99_GATE_US: u64 = 5_000;
        const PSS_DELTA_GATE_KIB: u64 = 40 * 1024;

        struct PublishedGeneration<T> {
            ordinal: u64,
            value: T,
        }

        struct GenerationLease<T>(Arc<PublishedGeneration<T>>);

        impl<T> Clone for GenerationLease<T> {
            fn clone(&self) -> Self {
                Self(Arc::clone(&self.0))
            }
        }

        impl<T> GenerationLease<T> {
            fn ordinal(&self) -> u64 {
                self.0.ordinal
            }

            fn value(&self) -> &T {
                &self.0.value
            }

            fn same_publication(&self, other: &Self) -> bool {
                Arc::ptr_eq(&self.0, &other.0)
            }
        }

        struct GenerationOwner<T> {
            current: RwLock<Arc<PublishedGeneration<T>>>,
        }

        impl<T> GenerationOwner<T> {
            fn new(initial: T) -> Self {
                Self {
                    current: RwLock::new(Arc::new(PublishedGeneration {
                        ordinal: 1,
                        value: initial,
                    })),
                }
            }

            fn borrow(&self) -> Result<GenerationLease<T>, String> {
                self.current
                    .read()
                    .map(|current| GenerationLease(Arc::clone(&current)))
                    .map_err(|_| "M3 V8 generation read lock poisoned".to_string())
            }

            fn publish(&self, next: T) -> Result<GenerationLease<T>, String> {
                let mut current = self
                    .current
                    .write()
                    .map_err(|_| "M3 V8 generation write lock poisoned".to_string())?;
                let ordinal = current
                    .ordinal
                    .checked_add(1)
                    .ok_or_else(|| "M3 V8 generation ordinal overflow".to_string())?;
                let published = Arc::new(PublishedGeneration {
                    ordinal,
                    value: next,
                });
                *current = Arc::clone(&published);
                Ok(GenerationLease(published))
            }

            fn try_publish<F>(&self, build: F) -> Result<GenerationLease<T>, String>
            where
                F: FnOnce() -> Result<T, String>,
            {
                self.publish(build()?)
            }

            fn commit_if_current<R, F>(
                &self,
                lease: &GenerationLease<T>,
                commit: F,
            ) -> Result<Option<R>, String>
            where
                F: FnOnce(&T) -> R,
            {
                let current = self
                    .current
                    .read()
                    .map_err(|_| "M3 V8 generation read lock poisoned".to_string())?;
                if current.ordinal != lease.ordinal() || !Arc::ptr_eq(&current, &lease.0) {
                    return Ok(None);
                }
                Ok(Some(commit(&current.value)))
            }
        }

        #[derive(Clone, Debug, PartialEq, Eq)]
        struct ExactGenerationIdentity {
            package_sha256: [u8; 32],
            package_bytes: u64,
            sidecar_sha256: [u8; 32],
            payload_sha256: [u8; 32],
            semantics_sha256: [u8; 32],
            state_count: u32,
            edge_count: u32,
            root_state: u32,
            symbol_count: u32,
            symbol_digest: [u8; 32],
            typed_payload_bytes: usize,
        }

        impl ExactGenerationIdentity {
            fn as_json(&self, ordinal: u64) -> serde_json::Value {
                serde_json::json!({
                    "owner_generation": ordinal,
                    "package_sha256": hex(self.package_sha256),
                    "package_bytes": self.package_bytes,
                    "sidecar_sha256": hex(self.sidecar_sha256),
                    "payload_sha256": hex(self.payload_sha256),
                    "semantics_sha256": hex(self.semantics_sha256),
                    "state_count": self.state_count,
                    "edge_count": self.edge_count,
                    "root_state": self.root_state,
                    "symbol_count": self.symbol_count,
                    "symbol_digest_sha256": hex(self.symbol_digest),
                    "typed_payload_bytes": self.typed_payload_bytes,
                })
            }
        }

        struct ExactGeneration {
            identity: ExactGenerationIdentity,
            _validated_sidecar: V13DafsaView,
            materialized: typed_exact::TypedMaterialization,
        }

        impl ExactGeneration {
            fn from_validated(
                sidecar: V13DafsaView,
                sidecar_sha256: [u8; 32],
            ) -> Result<Self, String> {
                let bytes = sidecar.bytes.as_slice();
                let payload_sha256 = bytes[24..56]
                    .try_into()
                    .map_err(|_| "M3 V8 payload digest width drift".to_string())?;
                let semantics_sha256 = bytes[112..144]
                    .try_into()
                    .map_err(|_| "M3 V8 semantics digest width drift".to_string())?;
                let materialized = typed_exact::TypedMaterialization::from_validated(&sidecar)?;
                let identity = ExactGenerationIdentity {
                    package_sha256: sidecar.identity.package_sha256,
                    package_bytes: sidecar.identity.package_bytes,
                    sidecar_sha256,
                    payload_sha256,
                    semantics_sha256,
                    state_count: sidecar.state_count,
                    edge_count: sidecar.edge_count,
                    root_state: sidecar.root_state,
                    symbol_count: sidecar.symbol_count,
                    symbol_digest: sidecar.symbol_digest,
                    typed_payload_bytes: materialized.payload_bytes(),
                };
                if identity.typed_payload_bytes != EXPECTED_TYPED_PAYLOAD_BYTES
                    || materialized.view().identity() != sidecar.identity
                    || materialized.view().root_state() != sidecar.root_state
                    || materialized.view().symbol_digest() != sidecar.symbol_digest
                {
                    return Err("M3 V8 typed generation identity drift".to_string());
                }
                Ok(Self {
                    identity,
                    _validated_sidecar: sidecar,
                    materialized,
                })
            }
        }

        fn write_sync_marker(path: &Path, value: &[u8]) -> Result<(), String> {
            use std::io::Write;

            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            file.write_all(value)
                .and_then(|()| file.sync_all())
                .map_err(|error| format!("{}: {error}", path.display()))
        }

        fn process_status_kib(pid: u32, field: &str) -> Result<u64, String> {
            let path = format!("/proc/{pid}/status");
            let status =
                std::fs::read_to_string(&path).map_err(|error| format!("{path}: {error}"))?;
            status
                .lines()
                .find_map(|line| {
                    line.strip_prefix(field)
                        .and_then(|value| value.split_whitespace().next())
                        .and_then(|value| value.parse().ok())
                })
                .ok_or_else(|| format!("{path} has no {field} field"))
        }

        fn process_memory_row(pid: u32) -> Result<serde_json::Value, String> {
            Ok(serde_json::json!({
                "pid": pid,
                "pss_kib": process_pss_kib(pid)?,
                "rss_kib": process_status_kib(pid, "VmRSS:")?,
                "hwm_kib": process_status_kib(pid, "VmHWM:")?,
            }))
        }

        fn memory_value(row: &serde_json::Value, field: &str) -> Result<u64, String> {
            row.get(field)
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(|| format!("M3 V8 memory row lacks {field}"))
        }

        fn measure_two_process_typed_pss(
            sidecar_path: &Path,
            identity: V13Identity,
            sidecar_sha256: [u8; 32],
        ) -> Result<serde_json::Value, String> {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_nanos();
            let sync_dir =
                std::env::temp_dir().join(format!("lay-m3-v8-pss-{}-{nonce}", std::process::id()));
            std::fs::create_dir(&sync_dir)
                .map_err(|error| format!("{}: {error}", sync_dir.display()))?;
            let executable = std::env::current_exe().map_err(|error| error.to_string())?;
            let mut children = Vec::<std::process::Child>::new();
            for child_index in 0..2 {
                let child = match std::process::Command::new(&executable)
                    .args([
                        "--ignored",
                        "--exact",
                        PSS_HELPER_TEST,
                        "--nocapture",
                        "--test-threads=1",
                    ])
                    .env("LAY_M3_V8_PSS_HELPER", "1")
                    .env("LAY_M3_V8_PSS_CHILD", child_index.to_string())
                    .env("LAY_M3_V8_PSS_SYNC", &sync_dir)
                    .env("LAY_M3_V8_SIDECAR", sidecar_path)
                    .env("LAY_M3_V8_ID_SHA256", hex(identity.package_sha256))
                    .env("LAY_M3_V8_ID_BYTES", identity.package_bytes.to_string())
                    .env("LAY_M3_V8_ID_FORMS", identity.form_count.to_string())
                    .env("LAY_M3_V8_ID_BINDINGS", identity.binding_count.to_string())
                    .env("LAY_M3_V8_SIDECAR_SHA256", hex(sidecar_sha256))
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                {
                    Ok(child) => child,
                    Err(error) => {
                        for child in &mut children {
                            let _ = child.kill();
                            let _ = child.wait();
                        }
                        let _ = std::fs::remove_dir_all(&sync_dir);
                        return Err(format!("spawn M3 V8 PSS helper: {error}"));
                    }
                };
                children.push(child);
            }

            let observation = (|| {
                wait_for_markers(&sync_dir, "ready", 2, Duration::from_secs(30))?;
                let before = children
                    .iter()
                    .map(|child| process_memory_row(child.id()))
                    .collect::<Result<Vec<_>, _>>()?;
                write_sync_marker(&sync_dir.join("start"), b"start\n")?;
                wait_for_markers(&sync_dir, "loaded", 2, Duration::from_secs(90))?;
                let after = children
                    .iter()
                    .map(|child| process_memory_row(child.id()))
                    .collect::<Result<Vec<_>, _>>()?;
                let identities = (0..2)
                    .map(|child| {
                        let path = sync_dir.join(format!("identity-{child}.json"));
                        serde_json::from_slice::<serde_json::Value>(
                            &std::fs::read(&path)
                                .map_err(|error| format!("{}: {error}", path.display()))?,
                        )
                        .map_err(|error| format!("{}: {error}", path.display()))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let expected_identity = identities
                    .first()
                    .cloned()
                    .ok_or_else(|| "M3 V8 PSS helper identity set is empty".to_string())?;
                let expected_sidecar_sha256 = hex(sidecar_sha256);
                if identities.iter().any(|row| row != &expected_identity)
                    || expected_identity
                        .get("sidecar_sha256")
                        .and_then(serde_json::Value::as_str)
                        != Some(expected_sidecar_sha256.as_str())
                    || expected_identity
                        .get("typed_payload_bytes")
                        .and_then(serde_json::Value::as_u64)
                        != Some(EXPECTED_TYPED_PAYLOAD_BYTES as u64)
                {
                    return Err("M3 V8 PSS helper identity mismatch".to_string());
                }
                let before_total = before
                    .iter()
                    .map(|row| memory_value(row, "pss_kib"))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .sum::<u64>();
                let after_total = after
                    .iter()
                    .map(|row| memory_value(row, "pss_kib"))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .sum::<u64>();
                Ok(serde_json::json!({
                    "processes": 2,
                    "before": before,
                    "after": after,
                    "aggregate_before_pss_kib": before_total,
                    "aggregate_after_pss_kib": after_total,
                    "aggregate_delta_pss_kib": after_total.saturating_sub(before_total),
                    "sidecar_bytes": std::fs::metadata(sidecar_path)
                        .map_err(|error| format!("{}: {error}", sidecar_path.display()))?
                        .len(),
                    "typed_owned_bytes_per_process": EXPECTED_TYPED_PAYLOAD_BYTES,
                    "helper_failures": 0,
                    "identity": expected_identity,
                }))
            })();

            let stop_result = write_sync_marker(&sync_dir.join("stop"), b"stop\n");
            let mut exit_failure = stop_result.err();
            for child in &mut children {
                if observation.is_err() || exit_failure.is_some() {
                    let _ = child.kill();
                }
                match child.wait() {
                    Ok(status) if status.success() => {}
                    Ok(status) => exit_failure = Some(format!("PSS helper exited with {status}")),
                    Err(error) => exit_failure = Some(format!("wait for PSS helper: {error}")),
                }
            }
            let _ = std::fs::remove_dir_all(&sync_dir);
            if let Some(error) = exit_failure {
                return Err(error);
            }
            observation
        }

        #[test]
        #[ignore = "child process for the M3 V8 typed-view PSS gate"]
        fn m3_end_to_end_pss_helper() {
            if std::env::var("LAY_M3_V8_PSS_HELPER").as_deref() != Ok("1") {
                return;
            }
            let child = std::env::var("LAY_M3_V8_PSS_CHILD").expect("M3 V8 PSS child index");
            let sync = std::path::PathBuf::from(
                std::env::var_os("LAY_M3_V8_PSS_SYNC").expect("M3 V8 PSS sync path"),
            );
            let sidecar = std::path::PathBuf::from(
                std::env::var_os("LAY_M3_V8_SIDECAR").expect("M3 V8 PSS sidecar path"),
            );
            let identity = V13Identity {
                package_sha256: parse_hex_32(
                    &std::env::var("LAY_M3_V8_ID_SHA256").expect("M3 V8 package SHA"),
                )
                .expect("parse M3 V8 package SHA"),
                package_bytes: std::env::var("LAY_M3_V8_ID_BYTES")
                    .expect("M3 V8 package bytes")
                    .parse()
                    .expect("parse M3 V8 package bytes"),
                form_count: std::env::var("LAY_M3_V8_ID_FORMS")
                    .expect("M3 V8 form count")
                    .parse()
                    .expect("parse M3 V8 form count"),
                binding_count: std::env::var("LAY_M3_V8_ID_BINDINGS")
                    .expect("M3 V8 binding count")
                    .parse()
                    .expect("parse M3 V8 binding count"),
            };
            let sidecar_sha256 = parse_hex_32(
                &std::env::var("LAY_M3_V8_SIDECAR_SHA256").expect("M3 V8 sidecar SHA"),
            )
            .expect("parse M3 V8 sidecar SHA");
            write_sync_marker(&sync.join(format!("ready-{child}")), b"ready\n")
                .expect("publish M3 V8 PSS ready");
            wait_for_path(&sync.join("start"), Duration::from_secs(60))
                .expect("wait for M3 V8 PSS start");
            let view = V13DafsaView::load(&sidecar, identity).expect("load M3 V8 sidecar");
            let generation = ExactGeneration::from_validated(view, sidecar_sha256)
                .expect("materialize M3 V8 typed generation");
            let identity_row = generation.identity.as_json(1);
            write_integration_receipt(&sync.join(format!("identity-{child}.json")), &identity_row)
                .expect("publish M3 V8 PSS identity");
            std::hint::black_box(generation.materialized.view().state_count());
            write_sync_marker(&sync.join(format!("loaded-{child}")), b"loaded\n")
                .expect("publish M3 V8 PSS loaded");
            wait_for_path(&sync.join("stop"), Duration::from_secs(90))
                .expect("wait for M3 V8 PSS stop");
            std::hint::black_box(generation);
        }

        #[derive(Clone, Debug, PartialEq, Eq)]
        struct RequestFingerprint {
            candidate_rows: Vec<(u32, String)>,
            certificate_rows: Vec<(u32, String, u8, String)>,
            lattice_surfaces: Vec<String>,
            emitted_surfaces: Vec<String>,
            gate_rows: Vec<(String, String, bool)>,
        }

        #[derive(Clone, Debug, Default)]
        struct SemanticCounters {
            candidate_mismatches: usize,
            certificate_mismatches: usize,
            structured_certificate_mismatches: usize,
            schedule_mismatches: usize,
            completeness_mismatches: usize,
            lattice_marker_mismatches: usize,
            emitted_surface_mismatches: usize,
            gate_mismatches: usize,
            certificate_collisions: usize,
            capacity_failures: usize,
            unresolved: usize,
        }

        impl SemanticCounters {
            fn add(&mut self, other: &Self) {
                self.candidate_mismatches = self
                    .candidate_mismatches
                    .saturating_add(other.candidate_mismatches);
                self.certificate_mismatches = self
                    .certificate_mismatches
                    .saturating_add(other.certificate_mismatches);
                self.structured_certificate_mismatches = self
                    .structured_certificate_mismatches
                    .saturating_add(other.structured_certificate_mismatches);
                self.schedule_mismatches = self
                    .schedule_mismatches
                    .saturating_add(other.schedule_mismatches);
                self.completeness_mismatches = self
                    .completeness_mismatches
                    .saturating_add(other.completeness_mismatches);
                self.lattice_marker_mismatches = self
                    .lattice_marker_mismatches
                    .saturating_add(other.lattice_marker_mismatches);
                self.emitted_surface_mismatches = self
                    .emitted_surface_mismatches
                    .saturating_add(other.emitted_surface_mismatches);
                self.gate_mismatches = self.gate_mismatches.saturating_add(other.gate_mismatches);
                self.certificate_collisions = self
                    .certificate_collisions
                    .saturating_add(other.certificate_collisions);
                self.capacity_failures = self
                    .capacity_failures
                    .saturating_add(other.capacity_failures);
                self.unresolved = self.unresolved.saturating_add(other.unresolved);
            }

            fn semantic_total(&self) -> usize {
                self.candidate_mismatches
                    .saturating_add(self.certificate_mismatches)
                    .saturating_add(self.structured_certificate_mismatches)
                    .saturating_add(self.schedule_mismatches)
                    .saturating_add(self.completeness_mismatches)
                    .saturating_add(self.lattice_marker_mismatches)
                    .saturating_add(self.emitted_surface_mismatches)
                    .saturating_add(self.gate_mismatches)
                    .saturating_add(self.certificate_collisions)
            }

            fn as_json(&self) -> serde_json::Value {
                serde_json::json!({
                    "candidate_mismatches": self.candidate_mismatches,
                    "certificate_mismatches": self.certificate_mismatches,
                    "structured_certificate_mismatches": self.structured_certificate_mismatches,
                    "schedule_mismatches": self.schedule_mismatches,
                    "completeness_mismatches": self.completeness_mismatches,
                    "lattice_marker_mismatches": self.lattice_marker_mismatches,
                    "emitted_surface_mismatches": self.emitted_surface_mismatches,
                    "gate_mismatches": self.gate_mismatches,
                    "certificate_collisions": self.certificate_collisions,
                    "capacity_failures": self.capacity_failures,
                    "unresolved": self.unresolved,
                    "semantic_total": self.semantic_total(),
                })
            }
        }

        struct RequestObservation {
            fingerprint: RequestFingerprint,
            counters: SemanticCounters,
            search_us: u64,
            total_us: u64,
            owner_prepare_us: u64,
            final_materialize_us: u64,
            maximum_scratch_bytes: usize,
            cpu_before: i32,
            cpu_after: i32,
        }

        struct V8Failure {
            verdict: &'static str,
            message: String,
        }

        impl V8Failure {
            fn provenance(error: impl ToString) -> Self {
                Self {
                    verdict: "BLOCKED_PROVENANCE",
                    message: error.to_string(),
                }
            }

            fn semantic(error: impl ToString) -> Self {
                Self {
                    verdict: "BLOCKED_SEMANTIC",
                    message: error.to_string(),
                }
            }

            fn capacity(error: impl ToString) -> Self {
                Self {
                    verdict: "BLOCKED_CAPACITY",
                    message: error.to_string(),
                }
            }
        }

        fn elapsed_micros(started: Instant) -> u64 {
            started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
        }

        #[cfg(target_os = "linux")]
        fn pin_current_thread(cpu: usize) -> Result<(), String> {
            unsafe {
                let mut set: libc::cpu_set_t = std::mem::zeroed();
                libc::CPU_ZERO(&mut set);
                libc::CPU_SET(cpu, &mut set);
                if libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set) != 0 {
                    return Err(format!(
                        "sched_setaffinity({cpu}) failed: {}",
                        std::io::Error::last_os_error()
                    ));
                }
            }
            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        fn pin_current_thread(_cpu: usize) -> Result<(), String> {
            Err("M3 V8 requires Linux CPU affinity".to_string())
        }

        #[cfg(target_os = "linux")]
        fn current_cpu() -> i32 {
            unsafe { libc::sched_getcpu() }
        }

        #[cfg(not(target_os = "linux"))]
        fn current_cpu() -> i32 {
            -1
        }

        fn exhaustive_result(
            oracle: &Phase7dCertificateOracle,
            package: &RuntimeL2Package,
            exact: &typed_exact::ExactObservation,
        ) -> Result<SearchResult, V8Failure> {
            if let Some(reason) = exact.unresolved {
                return Err(V8Failure::capacity(format!(
                    "typed exact search unresolved: {reason}"
                )));
            }
            let mut peaks = Vec::new();
            for form_ref in exact.retrieved_form_refs.iter().copied() {
                let surface = package.surface(form_ref as usize).ok_or_else(|| {
                    V8Failure::provenance(format!("V13 terminal rank {form_ref} cannot be decoded"))
                })?;
                let certificate_keys = oracle
                    .certificate_keys(surface.as_ref())
                    .map_err(V8Failure::semantic)?;
                if !certificate_keys.is_empty() {
                    peaks.push(V13TypedPeak {
                        form_ref,
                        certificate_keys,
                    });
                }
            }
            Ok(SearchResult {
                retrieved_form_refs: exact.retrieved_form_refs.clone(),
                peaks,
                completeness: SearchCompleteness::CertifiedExhaustive,
                expanded_product_states: exact.expanded_product_states,
                maximum_scratch_bytes: exact.maximum_scratch_bytes,
                dla_states: 0,
                dla_transitions: 0,
                maximum_dla_classes: 0,
                dla_build_elapsed_us: 0,
                intersection_elapsed_us: 0,
                search_elapsed_us: 0,
                material_elapsed_us: 0,
                total_elapsed_us: 0,
            })
        }

        fn run_closed_request(
            generation: &ExactGeneration,
            case: &ProofCase,
            reverse_schedule: bool,
            package: &RuntimeL2Package,
            canonical_index: &StandaloneL2Field,
            productive: &PackagedProductiveRuntimeV1,
        ) -> Result<RequestObservation, V8Failure> {
            let cpu_before = current_cpu();
            let total_started = Instant::now();
            let search_started = Instant::now();
            let oracle = Phase7dCertificateOracle::new(&case.damaged_surface)
                .map_err(V8Failure::semantic)?;
            let exact = typed_exact::search(
                generation.materialized.view(),
                &oracle.retrieval_lanes(),
                SearchBudget::proof(),
                reverse_schedule,
                false,
                || false,
            )
            .map_err(V8Failure::semantic)?;
            let search_us = elapsed_micros(search_started);
            let result = exhaustive_result(&oracle, package, &exact)?;

            let mut counters = SemanticCounters::default();
            counters.completeness_mismatches =
                usize::from(result.completeness != SearchCompleteness::CertifiedExhaustive);
            counters.unresolved = usize::from(exact.unresolved.is_some());
            let mut inputs = Vec::<ExactPeakCandidateInputV1>::new();
            let mut expected_candidates = BTreeSet::<(u32, String)>::new();
            let mut expected_certificates = BTreeSet::<(u32, String, u8, String)>::new();
            for peak in &result.peaks {
                let surface = package.surface(peak.form_ref as usize).ok_or_else(|| {
                    V8Failure::provenance(format!("missing V13 form {}", peak.form_ref))
                })?;
                let normalized =
                    super::super::super::compositional::normalize_surface(surface.as_ref());
                let evidence = oracle
                    .certificate_evidence(surface.as_ref())
                    .map_err(V8Failure::semantic)?;
                let evidence_keys = evidence
                    .iter()
                    .map(|certificate| certificate.canonical_key.clone())
                    .collect::<Vec<_>>();
                counters.structured_certificate_mismatches = counters
                    .structured_certificate_mismatches
                    .saturating_add(usize::from(evidence_keys != peak.certificate_keys));
                expected_candidates.insert((peak.form_ref, normalized.clone()));
                for certificate in &evidence {
                    expected_certificates.insert((
                        peak.form_ref,
                        normalized.clone(),
                        certificate.class as u8,
                        certificate.canonical_key.clone(),
                    ));
                }
                inputs.push(ExactPeakCandidateInputV1 {
                    form_ref: peak.form_ref,
                    normalized_surface: normalized,
                    certificates: evidence,
                });
            }
            let expected_candidates = expected_candidates.into_iter().collect::<Vec<_>>();
            let expected_certificates = expected_certificates.into_iter().collect::<Vec<_>>();
            let exact_peaks =
                ExactPeakBirthEnumerationV1::from_candidates(inputs).map_err(|error| {
                    if error.contains("collision") {
                        counters.certificate_collisions =
                            counters.certificate_collisions.saturating_add(1);
                    }
                    V8Failure::semantic(error)
                })?;
            if exact_peaks.capacity_exceeded() {
                return Err(V8Failure::capacity(
                    "exact peak birth enumeration exceeded capacity",
                ));
            }

            let owner_started = Instant::now();
            let field = prepare_live_productive_v1_field_with_exact_peaks(
                "",
                &case.damaged_surface,
                canonical_index,
                productive,
                &[],
                &[],
                &[],
                exact_peaks,
            )
            .map_err(|error| {
                if error.contains("capacity") || error.contains("StorageCapacity") {
                    V8Failure::capacity(error)
                } else {
                    V8Failure::semantic(error)
                }
            })?;
            let owner_prepare_us = elapsed_micros(owner_started);
            let actual_candidates = field.exact_peak_candidate_rows();
            let actual_certificates = field.exact_peak_certificate_rows();
            let material_completeness = field.exact_peak_material_completeness();
            let observed_normalized =
                super::super::super::compositional::normalize_surface(&case.damaged_surface);
            let expected_lattice = expected_candidates
                .iter()
                .map(|(_, surface)| surface.clone())
                .filter(|surface| !surface.eq_ignore_ascii_case(&observed_normalized))
                .collect::<BTreeSet<_>>();
            let actual_lattice = field
                .exact_peak_lattice_surfaces()
                .into_iter()
                .map(str::to_string)
                .collect::<BTreeSet<_>>();

            let materialize_started = Instant::now();
            let readout = materialize_live_productive_v1_field(
                &case.damaged_surface,
                &case.damaged_surface,
                &field,
            )
            .map_err(V8Failure::semantic)?;
            let final_materialize_us = elapsed_micros(materialize_started);
            let exact_candidates = readout
                .candidates
                .iter()
                .filter(|candidate| candidate.source_id == PRODUCTIVE_V90_TYPED_EXACT_SOURCE_ID)
                .collect::<Vec<_>>();
            let actual_emitted = exact_candidates
                .iter()
                .map(|candidate| {
                    super::super::super::compositional::normalize_surface(&candidate.replacement)
                })
                .collect::<BTreeSet<_>>();
            let gate_rows = exact_candidates
                .iter()
                .map(|candidate| {
                    let normalized = super::super::super::compositional::normalize_surface(
                        &candidate.replacement,
                    );
                    let independently_authorized =
                        field.exact_peak_surface_has_independent_authority(&normalized);
                    (
                        normalized,
                        format!("{:?}", candidate.gate.action),
                        independently_authorized,
                    )
                })
                .collect::<Vec<_>>();
            let total_us = elapsed_micros(total_started);
            let cpu_after = current_cpu();

            counters.candidate_mismatches = usize::from(actual_candidates != expected_candidates);
            counters.certificate_mismatches =
                usize::from(actual_certificates != expected_certificates);
            counters.completeness_mismatches =
                counters.completeness_mismatches.saturating_add(usize::from(
                    material_completeness.state()
                        != crate::typing_transition::target_evidence::EnumerationStateV1::Complete,
                ));
            counters.lattice_marker_mismatches = usize::from(actual_lattice != expected_lattice);
            counters.emitted_surface_mismatches = usize::from(actual_emitted != expected_lattice);
            counters.gate_mismatches = exact_candidates
                .iter()
                .filter(|candidate| {
                    let normalized = super::super::super::compositional::normalize_surface(
                        &candidate.replacement,
                    );
                    let independently_authorized =
                        field.exact_peak_surface_has_independent_authority(&normalized);
                    candidate.gate.action != CandidateGateAction::SuggestOnly
                        && !(candidate.gate.action == CandidateGateAction::Eligible
                            && independently_authorized)
                })
                .count();
            if field.productive_package_sha256() != productive.package_sha256()
                || field.material_scope() != PreparedFieldMaterialScopeV1::ContextShapedObservation
            {
                return Err(V8Failure::semantic(
                    "M3 V8 prepared owner identity or scope drift",
                ));
            }

            Ok(RequestObservation {
                fingerprint: RequestFingerprint {
                    candidate_rows: expected_candidates,
                    certificate_rows: expected_certificates,
                    lattice_surfaces: actual_lattice.into_iter().collect(),
                    emitted_surfaces: actual_emitted.into_iter().collect(),
                    gate_rows,
                },
                counters,
                search_us,
                total_us,
                owner_prepare_us,
                final_materialize_us,
                maximum_scratch_bytes: result.maximum_scratch_bytes,
                cpu_before,
                cpu_after,
            })
        }

        fn write_sidecar_once(path: &Path, bytes: &[u8]) -> Result<(), String> {
            use std::io::Write;

            if path.exists() {
                return Err(format!(
                    "M3 V8 evidence sidecar already exists: {}",
                    path.display()
                ));
            }
            let parent = path
                .parent()
                .ok_or_else(|| "M3 V8 evidence sidecar has no parent".to_string())?;
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("{}: {error}", parent.display()))?;
            let temporary = path.with_extension(format!("dafsa.tmp-{}", std::process::id()));
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
                .map_err(|error| format!("{}: {error}", temporary.display()))?;
            file.write_all(bytes)
                .and_then(|()| file.sync_all())
                .map_err(|error| format!("{}: {error}", temporary.display()))?;
            std::fs::rename(&temporary, path).map_err(|error| {
                format!("{} -> {}: {error}", temporary.display(), path.display())
            })?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o444))
                    .map_err(|error| format!("{}: {error}", path.display()))?;
            }
            std::fs::File::open(parent)
                .and_then(|directory| directory.sync_all())
                .map_err(|error| format!("{}: {error}", parent.display()))
        }

        fn timing_distribution(values: &[u64]) -> serde_json::Value {
            let mut p50 = values.to_vec();
            let mut p99 = values.to_vec();
            serde_json::json!({
                "samples": values.len(),
                "p50_us": percentile(&mut p50, 50),
                "p99_us": percentile(&mut p99, 99),
                "max_us": maximum(values),
            })
        }

        fn timing_p99(row: &serde_json::Value) -> u64 {
            row.get("p99_us")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(u64::MAX)
        }

        fn exercise_generation_reload(
            owner: &Arc<GenerationOwner<ExactGeneration>>,
            sidecar_path: &Path,
            identity: V13Identity,
            sidecar_sha256: [u8; 32],
        ) -> Result<serde_json::Value, V8Failure> {
            let generation_a = owner.borrow().map_err(V8Failure::provenance)?;
            let readers = (0..8)
                .map(|_| {
                    let owner = Arc::clone(owner);
                    std::thread::spawn(move || owner.borrow())
                })
                .map(|reader| {
                    reader
                        .join()
                        .map_err(|_| V8Failure::provenance("M3 V8 generation reader panicked"))?
                        .map_err(V8Failure::provenance)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let reader_identity_mismatches = readers
                .iter()
                .filter(|reader| {
                    !reader.same_publication(&generation_a)
                        || reader.value().identity != generation_a.value().identity
                })
                .count();

            let view_b =
                V13DafsaView::load(sidecar_path, identity).map_err(V8Failure::provenance)?;
            let next_b = ExactGeneration::from_validated(view_b, sidecar_sha256)
                .map_err(V8Failure::provenance)?;
            let generation_b = owner.publish(next_b).map_err(V8Failure::provenance)?;
            let current_b = owner.borrow().map_err(V8Failure::provenance)?;
            let mixed_generation_observations = usize::from(
                generation_a.ordinal() != 1
                    || generation_b.ordinal() != 2
                    || !generation_b.same_publication(&current_b)
                    || generation_a.value().identity != generation_b.value().identity,
            );
            let stale_commit = owner
                .commit_if_current(&generation_a, |generation| {
                    generation.materialized.view().state_count()
                })
                .map_err(V8Failure::provenance)?;
            let current_commit = owner
                .commit_if_current(&generation_b, |generation| {
                    generation.materialized.view().state_count()
                })
                .map_err(V8Failure::provenance)?;
            let held_a_state_count = generation_a.value().materialized.view().state_count();
            let failed_c = owner.try_publish(|| Err("injected generation-C failure".to_string()));
            let after_failed_c = owner.borrow().map_err(V8Failure::provenance)?;
            let failed_build_publications = usize::from(failed_c.is_ok());
            let rollback_identity_mismatches = usize::from(
                !after_failed_c.same_publication(&generation_b)
                    || after_failed_c.ordinal() != 2
                    || after_failed_c.value().identity != generation_b.value().identity,
            );

            Ok(serde_json::json!({
                "reader_count": readers.len(),
                "reader_identity_mismatches": reader_identity_mismatches,
                "mixed_generation_observations": mixed_generation_observations,
                "generation_a": generation_a.value().identity.as_json(generation_a.ordinal()),
                "generation_b": generation_b.value().identity.as_json(generation_b.ordinal()),
                "held_a_survived_publication": held_a_state_count
                    == typed_exact::EXPECTED_STATE_COUNT,
                "stale_a_commits": usize::from(stale_commit.is_some()),
                "stale_a_cancellations": usize::from(stale_commit.is_none()),
                "current_b_commits": usize::from(current_commit.is_some()),
                "failed_build_publications": failed_build_publications,
                "rollback_identity_mismatches": rollback_identity_mismatches,
                "typed_materializations": 2,
                "published_generations": 2,
                "per_request_typed_materializations": 0,
            }))
        }

        fn reload_count(reload: &serde_json::Value, field: &str) -> usize {
            reload
                .get(field)
                .and_then(serde_json::Value::as_u64)
                .and_then(|value| usize::try_from(value).ok())
                .unwrap_or(usize::MAX)
        }

        fn run_m3_end_to_end_physical_proof() -> Result<serde_json::Value, V8Failure> {
            let package_path = std::env::var_os("LAY_M3_ACTUAL_OWNER_PACKAGE")
                .map(std::path::PathBuf::from)
                .ok_or_else(|| V8Failure::provenance("LAY_M3_ACTUAL_OWNER_PACKAGE is required"))?;
            let v7_path = std::env::var_os("LAY_M3_ACTUAL_OWNER_V7")
                .map(std::path::PathBuf::from)
                .ok_or_else(|| V8Failure::provenance("LAY_M3_ACTUAL_OWNER_V7 is required"))?;
            let evidence_dir = std::env::var_os("LAY_M3_V8_EVIDENCE_DIR")
                .map(std::path::PathBuf::from)
                .ok_or_else(|| V8Failure::provenance("LAY_M3_V8_EVIDENCE_DIR is required"))?;
            let sidecar_path = evidence_dir.join("CURRENT_SIDECAR.dafsa");

            let package_bytes = std::fs::metadata(&package_path)
                .map_err(V8Failure::provenance)?
                .len();
            let package_sha256 = sha256_file(&package_path).map_err(V8Failure::provenance)?;
            if package_bytes != EXPECTED_V13_BYTES || hex(package_sha256) != EXPECTED_V13_SHA256 {
                return Err(V8Failure::provenance("fixed V13 package identity drift"));
            }
            let v7_bytes = std::fs::metadata(&v7_path)
                .map_err(V8Failure::provenance)?
                .len();
            let v7_sha256 = sha256_file(&v7_path).map_err(V8Failure::provenance)?;
            if v7_bytes != 1_606_189 || hex(v7_sha256) != EXPECTED_V7_SHA256 {
                return Err(V8Failure::provenance("fixed V7 evidence identity drift"));
            }

            let package = RuntimeL2Package::load(&package_path).map_err(V8Failure::provenance)?;
            let identity = V13Identity {
                package_sha256,
                package_bytes,
                form_count: u32::try_from(package.form_count()).map_err(V8Failure::provenance)?,
                binding_count: u32::try_from(package.binding_count())
                    .map_err(V8Failure::provenance)?,
            };
            let sidecar_bytes =
                compile_sidecar(&package, identity).map_err(V8Failure::provenance)?;
            if sidecar_bytes.len() != EXPECTED_V11_SIDECAR_BYTES {
                return Err(V8Failure::provenance(format!(
                    "current sidecar byte length drift: {}",
                    sidecar_bytes.len()
                )));
            }
            let sidecar_sha256: [u8; 32] = Sha256::digest(&sidecar_bytes).into();
            let payload_sha256: [u8; 32] = sidecar_bytes[24..56]
                .try_into()
                .map_err(V8Failure::provenance)?;
            if payload_sha256 != <[u8; 32]>::from(Sha256::digest(&sidecar_bytes[HEADER_BYTES..]))
                || sidecar_bytes[112..144] != phase7d_semantics_digest()
            {
                return Err(V8Failure::provenance(
                    "current sidecar payload or semantics identity drift",
                ));
            }
            let mut historical_projection = sidecar_bytes.clone();
            historical_projection[112..144].copy_from_slice(
                &parse_hex_32(EXPECTED_HISTORICAL_PHASE7D_SOURCE_SHA256)
                    .map_err(V8Failure::provenance)?,
            );
            if hex(<[u8; 32]>::from(Sha256::digest(&historical_projection)))
                != EXPECTED_V11_SIDECAR_SHA256
            {
                return Err(V8Failure::provenance(
                    "historical sidecar projection identity drift",
                ));
            }
            write_sidecar_once(&sidecar_path, &sidecar_bytes).map_err(V8Failure::provenance)?;
            drop(sidecar_bytes);

            let sidecar =
                V13DafsaView::load(&sidecar_path, identity).map_err(V8Failure::provenance)?;
            if !sidecar.mmap_backed() {
                return Err(V8Failure::provenance(
                    "M3 V8 current sidecar is not mmap backed",
                ));
            }
            let generation_a = ExactGeneration::from_validated(sidecar, sidecar_sha256)
                .map_err(V8Failure::provenance)?;
            let owner = Arc::new(GenerationOwner::new(generation_a));
            let request_generation = owner.borrow().map_err(V8Failure::provenance)?;

            let source: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&v7_path).map_err(V8Failure::provenance)?)
                    .map_err(V8Failure::provenance)?;
            let cases = parse_v7_cases(&source).map_err(V8Failure::provenance)?;
            if cases.len() != EXPECTED_FIXED_CASES {
                return Err(V8Failure::provenance(format!(
                    "fixed V7 case count drift: {}",
                    cases.len()
                )));
            }
            let canonical_index = super::super::super::installed_l2_field()
                .map_err(|error| V8Failure::provenance(error.to_string()))?;
            let productive =
                super::super::super::installed_productive_l2_v1().map_err(V8Failure::provenance)?;
            if canonical_index.form_count() != EXPECTED_V13_FORMS
                || hex(productive.package_sha256()) != EXPECTED_PRODUCTIVE_V90_SHA256
                || hex(productive.l11_package_sha256()) != EXPECTED_L11_SHA256
                || productive.canonical_l2_package_sha256() != package_sha256
            {
                return Err(V8Failure::provenance(
                    "M3 V8 actual owner package tuple drift",
                ));
            }
            let first_case = cases
                .first()
                .ok_or_else(|| V8Failure::provenance("fixed V7 case set is empty"))?;
            let live_empty = prepare_live_productive_v1_field(
                "",
                &first_case.damaged_surface,
                canonical_index,
                productive.as_ref(),
                &[],
                &[],
                &[],
            )
            .map_err(V8Failure::semantic)?;
            let explicit_empty = prepare_live_productive_v1_field_with_exact_peaks(
                "",
                &first_case.damaged_surface,
                canonical_index,
                productive.as_ref(),
                &[],
                &[],
                &[],
                ExactPeakBirthEnumerationV1::complete_empty(),
            )
            .map_err(V8Failure::semantic)?;
            let empty_lane_mismatches = usize::from(live_empty != explicit_empty);

            pin_current_thread(0).map_err(V8Failure::provenance)?;
            let mut warmup_counters = SemanticCounters::default();
            let mut warmup_cpu_mismatches = 0_usize;
            for case in &cases {
                let observation = run_closed_request(
                    request_generation.value(),
                    case,
                    false,
                    &package,
                    canonical_index,
                    productive.as_ref(),
                )?;
                warmup_cpu_mismatches = warmup_cpu_mismatches.saturating_add(usize::from(
                    observation.cpu_before != 0 || observation.cpu_after != 0,
                ));
                warmup_counters.add(&observation.counters);
            }
            if warmup_counters.semantic_total() != 0
                || warmup_counters.capacity_failures != 0
                || warmup_counters.unresolved != 0
            {
                return Err(V8Failure::semantic("M3 V8 warmup semantic mismatch"));
            }

            let schedules = [false, true, false, true];
            let mut reference_fingerprints = vec![None::<RequestFingerprint>; cases.len()];
            let mut counters = SemanticCounters::default();
            let mut round_rows = Vec::new();
            let mut pooled_search = Vec::with_capacity(cases.len() * schedules.len());
            let mut pooled_total = Vec::with_capacity(cases.len() * schedules.len());
            let mut pooled_owner = Vec::with_capacity(cases.len() * schedules.len());
            let mut pooled_materialize = Vec::with_capacity(cases.len() * schedules.len());
            let mut maximum_scratch_bytes = 0_usize;
            let mut cpu_mismatches = 0_usize;
            for (round_index, reverse_schedule) in schedules.into_iter().enumerate() {
                let mut round_search = Vec::with_capacity(cases.len());
                let mut round_total = Vec::with_capacity(cases.len());
                let mut round_owner = Vec::with_capacity(cases.len());
                let mut round_materialize = Vec::with_capacity(cases.len());
                for (case_index, case) in cases.iter().enumerate() {
                    let mut observation = run_closed_request(
                        request_generation.value(),
                        case,
                        reverse_schedule,
                        &package,
                        canonical_index,
                        productive.as_ref(),
                    )?;
                    if round_index == 0 {
                        reference_fingerprints[case_index] = Some(observation.fingerprint.clone());
                    } else if reference_fingerprints[case_index].as_ref()
                        != Some(&observation.fingerprint)
                    {
                        observation.counters.schedule_mismatches =
                            observation.counters.schedule_mismatches.saturating_add(1);
                    }
                    cpu_mismatches = cpu_mismatches.saturating_add(usize::from(
                        observation.cpu_before != 0 || observation.cpu_after != 0,
                    ));
                    maximum_scratch_bytes =
                        maximum_scratch_bytes.max(observation.maximum_scratch_bytes);
                    round_search.push(observation.search_us);
                    round_total.push(observation.total_us);
                    round_owner.push(observation.owner_prepare_us);
                    round_materialize.push(observation.final_materialize_us);
                    pooled_search.push(observation.search_us);
                    pooled_total.push(observation.total_us);
                    pooled_owner.push(observation.owner_prepare_us);
                    pooled_materialize.push(observation.final_materialize_us);
                    counters.add(&observation.counters);
                }
                round_rows.push(serde_json::json!({
                    "round": round_index + 1,
                    "schedule": if reverse_schedule { "REVERSED" } else { "FORWARD" },
                    "search": timing_distribution(&round_search),
                    "total_material": timing_distribution(&round_total),
                    "owner_prepare": timing_distribution(&round_owner),
                    "final_materialize": timing_distribution(&round_materialize),
                }));
            }
            counters.lattice_marker_mismatches = counters
                .lattice_marker_mismatches
                .saturating_add(empty_lane_mismatches);

            let maximum_round_search_p99 = round_rows
                .iter()
                .map(|round| timing_p99(&round["search"]))
                .max()
                .unwrap_or(u64::MAX);
            let maximum_round_total_p99 = round_rows
                .iter()
                .map(|round| timing_p99(&round["total_material"]))
                .max()
                .unwrap_or(u64::MAX);
            let reload =
                exercise_generation_reload(&owner, &sidecar_path, identity, sidecar_sha256)?;
            let pss = measure_two_process_typed_pss(&sidecar_path, identity, sidecar_sha256)
                .map_err(V8Failure::provenance)?;

            let reload_pass = reload_count(&reload, "reader_identity_mismatches") == 0
                && reload_count(&reload, "mixed_generation_observations") == 0
                && reload_count(&reload, "stale_a_commits") == 0
                && reload_count(&reload, "stale_a_cancellations") == 1
                && reload_count(&reload, "current_b_commits") == 1
                && reload_count(&reload, "failed_build_publications") == 0
                && reload_count(&reload, "rollback_identity_mismatches") == 0
                && reload_count(&reload, "typed_materializations") == 2
                && reload_count(&reload, "per_request_typed_materializations") == 0
                && reload
                    .get("held_a_survived_publication")
                    .and_then(serde_json::Value::as_bool)
                    == Some(true);
            let pss_delta = pss
                .get("aggregate_delta_pss_kib")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(u64::MAX);
            let pss_pass = pss_delta <= PSS_DELTA_GATE_KIB
                && pss.get("sidecar_bytes").and_then(serde_json::Value::as_u64)
                    == Some(EXPECTED_V11_SIDECAR_BYTES as u64)
                && pss
                    .get("typed_owned_bytes_per_process")
                    .and_then(serde_json::Value::as_u64)
                    == Some(EXPECTED_TYPED_PAYLOAD_BYTES as u64)
                && pss
                    .get("helper_failures")
                    .and_then(serde_json::Value::as_u64)
                    == Some(0);
            let semantic_pass = counters.semantic_total() == 0;
            let capacity_pass = counters.capacity_failures == 0
                && counters.unresolved == 0
                && maximum_scratch_bytes <= MAX_QUERY_SCRATCH_BYTES;
            let latency_pass = maximum_round_search_p99 <= SEARCH_P99_GATE_US
                && maximum_round_total_p99 <= TOTAL_P99_GATE_US;
            let environment_pass = cpu_mismatches == 0 && warmup_cpu_mismatches == 0;
            let verdict = if !semantic_pass {
                "BLOCKED_SEMANTIC"
            } else if !capacity_pass {
                "BLOCKED_CAPACITY"
            } else if !reload_pass {
                "BLOCKED_RELOAD_IDENTITY"
            } else if !pss_pass {
                "BLOCKED_RSS"
            } else if !latency_pass {
                "BLOCKED_LATENCY"
            } else if !environment_pass {
                "BLOCKED_ENVIRONMENT"
            } else {
                "M3_END_TO_END_TEST_OWNER_PASS"
            };
            let executable = std::env::current_exe().map_err(V8Failure::provenance)?;

            Ok(serde_json::json!({
                "schema": "lay.m3-end-to-end-test-owner.v1",
                "verdict": verdict,
                "source": {
                    "v13_package": package_path,
                    "v13_bytes": package_bytes,
                    "v13_sha256": hex(package_sha256),
                    "v7_fixed_proof": v7_path,
                    "v7_bytes": v7_bytes,
                    "v7_sha256": hex(v7_sha256),
                    "productive_v90_sha256": hex(productive.package_sha256()),
                    "l11_sha256": hex(productive.l11_package_sha256()),
                    "test_elf": executable,
                    "test_elf_sha256": hex(sha256_file(&executable).map_err(V8Failure::provenance)?),
                },
                "generation_owner": {
                    "current_sidecar_path": sidecar_path,
                    "current_sidecar_sha256": hex(sidecar_sha256),
                    "current_payload_sha256": hex(payload_sha256),
                    "initial_generation": request_generation
                        .value()
                        .identity
                        .as_json(request_generation.ordinal()),
                    "one_materialization_per_generation": true,
                    "per_request_typed_materializations": 0,
                },
                "fixed_proof": {
                    "cases": cases.len(),
                    "warmup_rounds": 1,
                    "measured_rounds": 4,
                    "measured_samples": pooled_total.len(),
                    "schedule": ["FORWARD", "REVERSED", "FORWARD", "REVERSED"],
                    "rounds": round_rows,
                    "pooled": {
                        "search": timing_distribution(&pooled_search),
                        "total_material": timing_distribution(&pooled_total),
                        "owner_prepare": timing_distribution(&pooled_owner),
                        "final_materialize": timing_distribution(&pooled_materialize),
                    },
                    "maximum_round_search_p99_us": maximum_round_search_p99,
                    "maximum_round_total_material_p99_us": maximum_round_total_p99,
                    "search_p99_gate_us": SEARCH_P99_GATE_US,
                    "total_material_p99_gate_us": TOTAL_P99_GATE_US,
                    "maximum_query_scratch_bytes": maximum_scratch_bytes,
                    "maximum_query_scratch_gate_bytes": MAX_QUERY_SCRATCH_BYTES,
                    "semantic": counters.as_json(),
                    "empty_lane_mismatches": empty_lane_mismatches,
                    "cpu": 0,
                    "cpu_mismatches": cpu_mismatches,
                    "warmup_cpu_mismatches": warmup_cpu_mismatches,
                },
                "reload": reload,
                "pss": pss,
                "gates": {
                    "semantic": semantic_pass,
                    "capacity": capacity_pass,
                    "reload_identity": reload_pass,
                    "rss": pss_pass,
                    "latency": latency_pass,
                    "environment": environment_pass,
                },
                "claim_boundary": {
                    "test_only_generation_owner": true,
                    "end_to_end_fixed_owner_proven_if_pass": verdict
                        == "M3_END_TO_END_TEST_OWNER_PASS",
                    "production_authority_admitted": false,
                    "runtime_reload_edit_admitted": false,
                    "daemon_or_ibus_tested": false,
                    "queue_inclusive_p99_tested": false,
                    "next_if_pass": "separate production authority decision paper",
                },
                "network_used_by_subject": false,
                "perf_or_pmu_used": false,
                "installed_package_changed": false,
                "runtime_authority_changed": false,
                "production_activation_admitted": false,
            }))
        }

        #[test]
        #[ignore = "one sealed target-host M3 V8 end-to-end physical proof"]
        fn m3_end_to_end_physical_proof() {
            let receipt_path = std::env::var_os("LAY_M3_V8_RECEIPT")
                .map(std::path::PathBuf::from)
                .expect("LAY_M3_V8_RECEIPT is required");
            let receipt = match run_m3_end_to_end_physical_proof() {
                Ok(receipt) => receipt,
                Err(failure) => serde_json::json!({
                    "schema": "lay.m3-end-to-end-test-owner.v1",
                    "verdict": failure.verdict,
                    "error": failure.message,
                    "observation_complete": false,
                    "production_authority_admitted": false,
                    "runtime_authority_changed": false,
                }),
            };
            write_integration_receipt(&receipt_path, &receipt)
                .expect("write M3 V8 end-to-end receipt");
            eprintln!("m3_v8_receipt={}", receipt_path.display());
            assert_eq!(
                receipt.get("verdict").and_then(serde_json::Value::as_str),
                Some("M3_END_TO_END_TEST_OWNER_PASS"),
                "{}",
                serde_json::to_string_pretty(&receipt).expect("render M3 V8 receipt")
            );
        }

        #[test]
        fn generation_owner_state_machine_is_fail_closed() {
            let owner = Arc::new(GenerationOwner::new(String::from("A")));
            let first = owner.borrow().expect("borrow generation A");
            let readers = (0..8)
                .map(|_| {
                    let owner = Arc::clone(&owner);
                    std::thread::spawn(move || {
                        owner.borrow().expect("concurrent generation borrow")
                    })
                })
                .map(|reader| reader.join().expect("generation reader panicked"))
                .collect::<Vec<_>>();
            assert!(readers.iter().all(|reader| reader.same_publication(&first)));
            assert_eq!(first.ordinal(), 1);

            let second = owner.publish(String::from("B")).expect("publish B");
            assert_eq!(second.ordinal(), 2);
            assert!(!second.same_publication(&first));
            assert_eq!(
                owner
                    .commit_if_current(&first, String::clone)
                    .expect("stale commit check"),
                None
            );
            assert_eq!(
                owner
                    .commit_if_current(&second, String::clone)
                    .expect("current commit check"),
                Some(String::from("B"))
            );

            let failed = owner.try_publish(|| Err("injected generation failure".to_string()));
            assert_eq!(failed.err().as_deref(), Some("injected generation failure"));
            let after_failure = owner.borrow().expect("borrow after failed generation");
            assert_eq!(after_failure.ordinal(), 2);
            assert!(after_failure.same_publication(&second));
        }
    }

    fn write_integration_receipt(path: &Path, receipt: &serde_json::Value) -> Result<(), String> {
        use std::io::Write;

        if path.exists() {
            return Err(format!(
                "integration receipt already exists: {}",
                path.display()
            ));
        }
        let parent = path
            .parent()
            .ok_or_else(|| "integration receipt has no parent".to_string())?;
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("{}: {error}", parent.display()))?;
        let temporary = path.with_extension(format!("json.tmp-{}", std::process::id()));
        let mut bytes = serde_json::to_vec_pretty(receipt).map_err(|error| error.to_string())?;
        bytes.push(b'\n');
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| format!("{}: {error}", temporary.display()))?;
        output
            .write_all(&bytes)
            .and_then(|()| output.sync_all())
            .map_err(|error| format!("{}: {error}", temporary.display()))?;
        std::fs::rename(&temporary, path)
            .map_err(|error| format!("{} -> {}: {error}", temporary.display(), path.display()))?;
        std::fs::File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| format!("{}: {error}", parent.display()))?;
        Ok(())
    }

    #[test]
    fn test_package_binding_shape_does_not_enter_sidecar_identity_search() {
        let _binding = MorphBinding {
            form_center_ref: 0,
            lemma_center_id: 1,
            feature_mask: 2,
            ..MorphBinding::default()
        };
        assert_eq!(STATE_BYTES, 8);
        assert_eq!(EDGE_BYTES, 8);
    }
}
