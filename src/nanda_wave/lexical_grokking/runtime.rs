use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::io;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::stable_hash::mix64_golden;

use super::atoms::{
    encode_wave_surface, normalize_lexical_surface, physical_key_sequence, AtomChannel, NGramKey,
};
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
const DEFAULT_BIRTH_ATOMS_PER_CHANNEL: usize = 4;
const MAX_BIRTH_ATOMS_PER_CHANNEL: usize = 32;
const DEFAULT_BIRTH_POSTING_BUDGET: usize = 131_072;
const MAX_BIRTH_POSTING_BUDGET: usize = 131_072;
const SETTLING_ITERATIONS: u8 = 3;
const MAX_ANCHOR_SEQUENCE: usize = 32;
const MAX_EXACT_COLLISION_OPERATOR_CHARS: usize = 16;
const DEFAULT_REVERSE_CACHE_MIB: usize = 16;
pub(super) const RECONSTRUCTION_MODE_DELETION: u8 = 1;
pub(super) const RECONSTRUCTION_MODE_DELETION_TRANSPOSITION: u8 = 2;
pub(super) const RECONSTRUCTION_MODE_SUFFIX_TRUNCATION: u8 = 4;
pub(super) const RECONSTRUCTION_MODE_PREFIX_TRUNCATION: u8 = 8;
pub(super) const RECONSTRUCTION_MODE_SINGLE_DELETION: u8 = 16;
pub(super) const RECONSTRUCTION_MODE_SINGLE_SUBSTITUTION: u8 = 32;
pub(super) const RECONSTRUCTION_MODE_DOUBLE_SUBSTITUTION: u8 = 64;
pub(super) const RECONSTRUCTION_MODE_NON_ADJACENT_TRANSPOSITION: u8 = 128;

pub fn query_package(
    package_path: &Path,
    surface: &str,
    limit: usize,
) -> io::Result<serde_json::Value> {
    let memory = LexicalGrokkingMemory::load(package_path).map_err(io::Error::other)?;
    let candidates = memory
        .readout(surface, limit, ReadoutMode::Full)
        .into_iter()
        .map(|candidate| candidate_json(&memory, candidate))
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "package": package_path,
        "surface": surface,
        "terminal_count": memory.package.terminal_count(),
        "candidates": candidates,
    }))
}

pub fn restore_surface(
    package_path: &Path,
    surface: &str,
    limit: usize,
) -> io::Result<serde_json::Value> {
    let host = L1RestorationHost::load(package_path)?;
    Ok(host.restore(surface, limit))
}

pub fn inspect_package_header(package_path: &Path) -> io::Result<serde_json::Value> {
    use std::io::Read;

    let mut file = std::fs::File::open(package_path)?;
    let mut header = [0_u8; 192];
    file.read_exact(&mut header)?;
    if v8::is_v8(&header) {
        let artifact = V8Artifact::load(package_path).map_err(io::Error::other)?;
        let package = artifact.decode_base().map_err(io::Error::other)?;
        return Ok(serde_json::json!({
            "format": "V8",
            "corpus_fingerprint": package.corpus_hash,
            "terminal_count": package.terminal_count(),
            "package_bytes": file.metadata()?.len(),
            "forward_relations": artifact.forward_relation_count(),
            "reverse_relations": artifact.reverse_relation_count(),
        }));
    }
    let (corpus_fingerprint, terminal_count, declared_bytes) =
        super::format::inspect_header(&header).map_err(io::Error::other)?;
    let actual_bytes = file.metadata()?.len();
    if declared_bytes != actual_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("L1.1 package size mismatch: header={declared_bytes} actual={actual_bytes}"),
        ));
    }
    Ok(serde_json::json!({
        "corpus_fingerprint": corpus_fingerprint,
        "terminal_count": terminal_count,
        "package_bytes": actual_bytes,
    }))
}

fn restoration_candidate_json(
    memory: &LexicalGrokkingMemory,
    candidate: super::restoration::RestorationCandidate,
) -> serde_json::Value {
    serde_json::json!({
        "terminal_id": candidate.terminal_id,
        "surface": memory.decode_terminal(candidate.terminal_id),
        "evidence": candidate.evidence,
    })
}

pub fn benchmark_package(
    package_path: &Path,
    surface: &str,
    iterations: usize,
    limit: usize,
) -> io::Result<serde_json::Value> {
    let host = L1RestorationHost::load(package_path)?;
    for _ in 0..16 {
        std::hint::black_box(benchmark_host_once(&host, surface, limit));
    }
    let mut elapsed_us = Vec::with_capacity(iterations);
    let mut checksum = 0_u64;
    for _ in 0..iterations {
        let started = Instant::now();
        let first_terminal = benchmark_host_once(&host, surface, limit);
        elapsed_us.push(started.elapsed().as_micros() as u64);
        checksum ^= first_terminal;
    }
    elapsed_us.sort_unstable();
    let stats = host.stats();
    Ok(serde_json::json!({
        "package": package_path,
        "surface": surface,
        "iterations": iterations,
        "limit": limit,
        "terminal_count": host.terminal_count(),
        "manifest_generation": stats.manifest_generation,
        "delta_count": stats.delta_count,
        "tombstone_count": stats.tombstone_count,
        "p50_us": percentile(&elapsed_us, 50),
        "p90_us": percentile(&elapsed_us, 90),
        "p99_us": percentile(&elapsed_us, 99),
        "max_us": elapsed_us.last().copied().unwrap_or_default(),
        "checksum": checksum,
    }))
}

pub fn benchmark_diverse_restoration(
    package_path: &Path,
    surfaces_path: &Path,
    limit: usize,
) -> io::Result<serde_json::Value> {
    let memory = LexicalGrokkingMemory::load(package_path).map_err(io::Error::other)?;
    let surfaces = std::fs::read_to_string(surfaces_path)?
        .lines()
        .map(str::trim)
        .filter(|surface| !surface.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if surfaces.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "diverse restoration benchmark requires at least one surface",
        ));
    }
    for surface in surfaces.iter().take(32) {
        std::hint::black_box(memory.readout(surface, limit, ReadoutMode::Full));
    }
    let mut readout_elapsed_us = surfaces
        .iter()
        .map(|surface| {
            let started = Instant::now();
            std::hint::black_box(memory.readout(surface, limit, ReadoutMode::Full));
            started.elapsed().as_micros() as u64
        })
        .collect::<Vec<_>>();
    readout_elapsed_us.sort_unstable();
    for surface in surfaces.iter().take(32) {
        let mut candidates = memory.readout(surface, limit, ReadoutMode::Full);
        std::hint::black_box(memory.classify_restoration(
            surface,
            &mut candidates,
            memory.package.restoration_calibration,
        ));
    }
    let mut elapsed_us = surfaces
        .iter()
        .map(|surface| {
            let started = Instant::now();
            let mut candidates = memory.readout(surface, limit, ReadoutMode::Full);
            std::hint::black_box(memory.classify_restoration(
                surface,
                &mut candidates,
                memory.package.restoration_calibration,
            ));
            started.elapsed().as_micros() as u64
        })
        .collect::<Vec<_>>();
    elapsed_us.sort_unstable();
    Ok(serde_json::json!({
        "package": package_path,
        "surfaces": surfaces_path,
        "sample_count": surfaces.len(),
        "limit": limit,
        "readout_p50_us": percentile(&readout_elapsed_us, 50),
        "readout_p90_us": percentile(&readout_elapsed_us, 90),
        "readout_p99_us": percentile(&readout_elapsed_us, 99),
        "readout_max_us": readout_elapsed_us.last().copied().unwrap_or_default(),
        "p50_us": percentile(&elapsed_us, 50),
        "p90_us": percentile(&elapsed_us, 90),
        "p99_us": percentile(&elapsed_us, 99),
        "max_us": elapsed_us.last().copied().unwrap_or_default(),
    }))
}

fn benchmark_host_once(host: &L1RestorationHost, surface: &str, limit: usize) -> u64 {
    if host.overlays.is_empty() && host.tombstones.is_empty() {
        let candidates = host.memory.readout(surface, limit, ReadoutMode::Full);
        std::hint::black_box(
            candidates
                .first()
                .map(|candidate| u64::from(candidate.terminal_id))
                .unwrap_or_default(),
        )
    } else {
        let candidates = host.lattice_seed_rows(surface, limit);
        std::hint::black_box(
            candidates
                .first()
                .map(|candidate| u64::from(candidate.0))
                .unwrap_or_default(),
        )
    }
}

fn candidate_json(
    memory: &LexicalGrokkingMemory,
    candidate: GrokkingCandidate,
) -> serde_json::Value {
    serde_json::json!({
        "terminal_id": candidate.terminal_id,
        "surface": memory.decode_terminal(candidate.terminal_id),
        "atom_hits": candidate.atom_hits,
        "surface_hits": candidate.surface_hits,
        "keyboard_hits": candidate.keyboard_hits,
        "structural_milli": candidate.structural_milli,
        "position_milli": candidate.position_milli,
        "legacy_sequence_milli": candidate.legacy_sequence_milli,
        "sequence_milli": candidate.sequence_milli,
        "forward_milli": candidate.forward_milli,
        "backward_milli": candidate.backward_milli,
        "positive_milli": candidate.positive_milli,
        "positive_subcenter_milli": candidate.positive_subcenter_milli,
        "anti_milli": candidate.anti_milli,
        "anti_subcenter_milli": candidate.anti_subcenter_milli,
        "hard_negative_milli": candidate.hard_negative_milli,
        "ambiguity_milli": candidate.ambiguity_milli,
        "ambiguity_threshold_milli": candidate.ambiguity_threshold_milli,
        "ambiguity_linked": candidate.ambiguity_linked,
        "ambiguity_shell": candidate.ambiguity_shell,
        "reconstruction_only": candidate.reconstruction_only,
        "pairwise_loss_milli": candidate.pairwise_loss_milli,
        "crystallization_wins": candidate.crystallization_wins,
        "crystallization_required": candidate.crystallization_required,
        "crystallization_margin_milli": candidate.crystallization_margin_milli,
        "crystallization_complete": candidate.crystallization_complete,
        "crystallization_known_edges": candidate.crystallization_known_edges,
        "crystallization_unknown_edges": candidate.crystallization_unknown_edges,
        "crystallization_tied_edges": candidate.crystallization_tied_edges,
        "crystallization_conflicts": candidate.crystallization_conflicts,
        "crystallization_cycles": candidate.crystallization_cycles,
        "length_milli": candidate.length_milli,
        "geometry_distance": candidate.geometry_distance,
        "reconstruction_modes": candidate.reconstruction_modes,
        "settled_energy": candidate.settled_energy,
        "legacy_settled_energy": candidate.legacy_settled_energy,
        "length_relation": candidate.length_relation,
        "exact_reconstruction": candidate.exact_reconstruction,
        "settling_iterations": candidate.settling_iterations,
    })
}

fn percentile(sorted: &[u64], percentile: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let index = (sorted.len() - 1).saturating_mul(percentile) / 100;
    sorted[index]
}

fn readout_trace_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("LAY_L11_READOUT_TRACE").is_some())
}

fn readout_trace_terminal() -> Option<u32> {
    static VALUE: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    *VALUE.get_or_init(|| {
        std::env::var("LAY_L11_READOUT_TRACE_TERMINAL")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
    })
}

fn birth_atoms_per_channel() -> usize {
    static VALUE: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *VALUE.get_or_init(|| {
        std::env::var("LAY_L11_BIRTH_ATOMS_PER_CHANNEL")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(DEFAULT_BIRTH_ATOMS_PER_CHANNEL)
            .clamp(1, MAX_BIRTH_ATOMS_PER_CHANNEL)
    })
}

fn birth_posting_budget() -> usize {
    static VALUE: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *VALUE.get_or_init(|| {
        std::env::var("LAY_L11_BIRTH_POSTING_BUDGET")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(DEFAULT_BIRTH_POSTING_BUDGET)
            .clamp(1, MAX_BIRTH_POSTING_BUDGET)
    })
}

fn reverse_cache_bytes() -> usize {
    static BYTES: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *BYTES.get_or_init(|| {
        std::env::var("LAY_L11_V8_REVERSE_CACHE_MIB")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(DEFAULT_REVERSE_CACHE_MIB)
            .min(128)
            .saturating_mul(1024 * 1024)
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReadoutMode {
    Full,
    WithoutAnti,
    WithoutPhase,
    WithoutSequence,
    WithoutSequenceCertificate,
    LegacySequence,
    WithoutPairwise,
    WithoutPosition,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct GrokkingCandidate {
    pub(super) terminal_id: u32,
    pub(super) atom_hits: u16,
    pub(super) surface_hits: u16,
    pub(super) keyboard_hits: u16,
    pub(super) structural_milli: u16,
    pub(super) position_milli: u16,
    pub(super) legacy_sequence_milli: u16,
    pub(super) sequence_milli: u16,
    pub(super) forward_milli: u16,
    pub(super) backward_milli: u16,
    pub(super) positive_milli: u16,
    pub(super) positive_subcenter_milli: u16,
    pub(super) anti_milli: u16,
    pub(super) anti_subcenter_milli: u16,
    pub(super) hard_negative_milli: u16,
    pub(super) ambiguity_milli: u16,
    pub(super) ambiguity_threshold_milli: u16,
    pub(super) ambiguity_linked: bool,
    pub(super) ambiguity_shell: bool,
    pub(super) reconstruction_only: bool,
    pub(super) pairwise_loss_milli: u16,
    pub(super) crystallization_wins: u8,
    pub(super) crystallization_required: u8,
    pub(super) crystallization_margin_milli: u16,
    pub(super) crystallization_complete: bool,
    pub(super) crystallization_known_edges: u16,
    pub(super) crystallization_unknown_edges: u16,
    pub(super) crystallization_tied_edges: u16,
    pub(super) crystallization_conflicts: u16,
    pub(super) crystallization_cycles: u16,
    pub(super) length_milli: u16,
    pub(super) geometry_distance: u8,
    pub(super) reconstruction_modes: u8,
    pub(super) settled_energy: i32,
    pub(super) legacy_settled_energy: i32,
    pub(super) length_relation: i8,
    pub(super) settling_iterations: u8,
    pub(super) exact_reconstruction: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct AmbiguityObservation {
    pub(super) center_index: usize,
    pub(super) owner: u32,
    pub(super) competitor: u32,
    pub(super) coherence_milli: u16,
    pub(super) structurally_applicable: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct ForwardActivation {
    mass: u64,
    hits: u16,
    surface_hits: u16,
    keyboard_hits: u16,
}

#[derive(Default)]
struct ForwardScratch {
    activations: Vec<ForwardActivation>,
    activation_epochs: Vec<u32>,
    epoch: u32,
    touched: Vec<u32>,
}

struct PreparedReadout {
    observed: BTreeMap<u32, ObservedAtom>,
    character_sequence: AnchorSequence,
    observed_char_count: u8,
    surface_re: [i32; WAVE_DIMENSION],
    surface_im: [i32; WAVE_DIMENSION],
    max_forward: u64,
    frontier: Vec<(u32, ForwardActivation)>,
    frontier_reverse: Option<Vec<Arc<[WaveCoupling]>>>,
    geometry_reserve_ids: BTreeSet<u32>,
    reconstruction_only_ids: BTreeSet<u32>,
}

thread_local! {
    static FORWARD_SCRATCH: RefCell<ForwardScratch> = RefCell::new(ForwardScratch::default());
}

#[derive(Clone, Copy, Debug)]
struct ObservedAtom {
    position: u8,
    weight: u8,
    channel: AtomChannel,
}

type BirthAtom = (usize, u32, ObservedAtom);

fn select_birth_atoms(
    birth_by_channel: &mut [Vec<BirthAtom>],
    atoms_per_channel: usize,
    posting_budget: usize,
) -> Vec<BirthAtom> {
    let mut eligible = Vec::new();
    for atoms in birth_by_channel {
        atoms.sort_unstable_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| right.2.weight.cmp(&left.2.weight))
                .then_with(|| left.1.cmp(&right.1))
        });
        eligible.extend(atoms.iter().take(atoms_per_channel).copied());
    }
    eligible.sort_unstable_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| right.2.weight.cmp(&left.2.weight))
            .then_with(|| left.1.cmp(&right.1))
    });
    let mut selected = Vec::with_capacity(eligible.len());
    let mut posting_count = 0_usize;
    for atom in eligible {
        let next = posting_count.saturating_add(atom.0);
        if !selected.is_empty() && next > posting_budget {
            continue;
        }
        posting_count = next;
        selected.push(atom);
    }
    selected
}

#[derive(Clone, Copy, Debug, Default)]
struct AnchorSequence {
    atoms: [u32; MAX_ANCHOR_SEQUENCE],
    len: u8,
}

impl AnchorSequence {
    fn as_slice(&self) -> &[u32] {
        &self.atoms[..usize::from(self.len)]
    }
}

pub(super) struct LexicalGrokkingMemory {
    pub(super) package: LexicalGrokkingPackage,
    exact_surface_index: Vec<(u64, u32)>,
    character_anchor_offsets: Vec<u32>,
    character_anchor_atoms: Vec<u32>,
    relations: RelationStore,
    reverse_cache: Mutex<ReverseCache>,
}

enum RelationStore {
    Eager,
    LazyV8(V8Artifact),
}

enum CouplingView<'a> {
    Borrowed(&'a [WaveCoupling]),
    Shared(Arc<[WaveCoupling]>),
}

impl Deref for CouplingView<'_> {
    type Target = [WaveCoupling];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(relations) => relations,
            Self::Shared(relations) => relations,
        }
    }
}

#[derive(Default)]
struct ReverseCache {
    bytes: usize,
    order: VecDeque<u32>,
    entries: HashMap<u32, Arc<[WaveCoupling]>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct L1RestorationHostStats {
    pub package_path: PathBuf,
    pub package_bytes: usize,
    pub terminal_count: u32,
    pub atom_count: usize,
    pub forward_relations: usize,
    pub reverse_relations: usize,
    pub exact_surface_count: usize,
    pub character_anchor_count: usize,
    pub manifest_generation: u64,
    pub delta_count: usize,
    pub tombstone_count: usize,
}

pub struct L1RestorationHost {
    package_path: PathBuf,
    package_bytes: usize,
    memory: LexicalGrokkingMemory,
    overlays: Vec<L1OverlayMemory>,
    tombstones: BTreeSet<String>,
    manifest_generation: u64,
}

struct L1OverlayMemory {
    terminal_offset: u32,
    memory: LexicalGrokkingMemory,
}

impl LexicalGrokkingMemory {
    pub(super) fn from_package(package: LexicalGrokkingPackage) -> Self {
        let (exact_surface_index, character_anchor_offsets, character_anchor_atoms) =
            compile_surface_indices(&package);
        Self {
            package,
            exact_surface_index,
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
        let (exact_surface_index, character_anchor_offsets, character_anchor_atoms) =
            compile_surface_indices(&package);
        Ok(Self {
            package,
            exact_surface_index,
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

    pub(super) fn readout_modes(
        &self,
        surface: &str,
        limit: usize,
        modes: &[ReadoutMode],
    ) -> Vec<Vec<GrokkingCandidate>> {
        if limit == 0 {
            return vec![Vec::new(); modes.len()];
        }
        let Some(prepared) = self.prepare_readout(surface, limit) else {
            return vec![Vec::new(); modes.len()];
        };
        let mut invariant_candidates =
            self.settle_prepared_candidates(&prepared, ReadoutMode::Full);
        self.apply_restoration_geometry(surface, &mut invariant_candidates);
        modes
            .iter()
            .copied()
            .map(|mode| {
                let mut candidates = invariant_candidates.clone();
                for candidate in &mut candidates {
                    apply_settlement_mode(candidate, mode);
                }
                self.finalize_candidates_after_geometry(
                    limit,
                    mode,
                    &prepared.surface_re,
                    &prepared.surface_im,
                    &mut candidates,
                );
                candidates
            })
            .collect()
    }

    fn prepare_readout(&self, surface: &str, limit: usize) -> Option<PreparedReadout> {
        let trace_started = Instant::now();
        let observed = self.resolve_surface(surface);
        if observed.is_empty() {
            return None;
        }
        let resolve_us = trace_started.elapsed().as_micros();
        let character_sequence = observed_sequence(&observed, AtomChannel::CharacterAnchor);
        let exact_terminals = self.exact_terminals(character_sequence.as_slice());
        let observed_char_count = normalize_lexical_surface(surface)
            .chars()
            .count()
            .min(u8::MAX as usize) as u8;
        let lexical_observed = observed
            .iter()
            .filter(|(_, atom)| !is_anchor_channel(atom.channel))
            .copied()
            .collect::<BTreeMap<_, _>>();
        let mut birth_by_channel: [Vec<BirthAtom>; 12] = std::array::from_fn(|_| Vec::new());
        for (atom_id, atom) in &lexical_observed {
            birth_by_channel[atom.channel as usize].push((
                self.forward_degree(*atom_id),
                *atom_id,
                *atom,
            ));
        }
        let birth_atoms = select_birth_atoms(
            &mut birth_by_channel,
            birth_atoms_per_channel(),
            birth_posting_budget(),
        );
        let birth_postings = birth_atoms
            .iter()
            .map(|(degree, _, _)| *degree)
            .sum::<usize>();
        let birth_atom_ids = birth_atoms
            .iter()
            .map(|(_, atom_id, _)| *atom_id)
            .collect::<Vec<_>>();
        let birth_couplings = self.forward_coupling_views(&birth_atom_ids);
        let prefetch_us = trace_started.elapsed().as_micros();
        let (surface_re, surface_im, mut frontier) = FORWARD_SCRATCH.with_borrow_mut(|scratch| {
            if scratch.activations.len() != self.package.terminal_count() as usize {
                scratch.activations =
                    vec![ForwardActivation::default(); self.package.terminal_count() as usize];
                scratch.activation_epochs = vec![0; self.package.terminal_count() as usize];
                scratch.epoch = 1;
                scratch.touched.clear();
            } else {
                scratch.epoch = scratch.epoch.wrapping_add(1);
                if scratch.epoch == 0 {
                    scratch.activation_epochs.fill(0);
                    scratch.epoch = 1;
                }
                scratch.touched.clear();
            }
            let epoch = scratch.epoch;
            let mut surface_re = [0_i32; WAVE_DIMENSION];
            let mut surface_im = [0_i32; WAVE_DIMENSION];
            for (atom_id, atom) in &lexical_observed {
                let Some(record) = self.package.atoms.get(*atom_id as usize) else {
                    continue;
                };
                expand_atom(
                    &self.package.basis,
                    record.wave_code,
                    &mut surface_re,
                    &mut surface_im,
                    i32::from(atom.weight),
                );
            }
            for ((_, _, atom), couplings) in birth_atoms.iter().zip(&birth_couplings) {
                let atom_weight = u64::from(atom.weight);
                let keyboard_channel = is_keyboard_channel(atom.channel);
                for coupling in couplings.iter() {
                    let contribution = u64::from(coupling.strength)
                        * atom_weight
                        * u64::from(position_coherence(atom.position, coupling.position_mode));
                    let terminal_id = coupling.peer_id as usize;
                    if terminal_id >= scratch.activations.len() {
                        continue;
                    }
                    if scratch.activation_epochs[terminal_id] != epoch {
                        scratch.activation_epochs[terminal_id] = epoch;
                        scratch.activations[terminal_id] = ForwardActivation::default();
                        scratch.touched.push(coupling.peer_id);
                    }
                    let activation = &mut scratch.activations[terminal_id];
                    activation.mass += contribution;
                    activation.hits += 1;
                    if keyboard_channel {
                        activation.keyboard_hits += 1;
                    } else {
                        activation.surface_hits += 1;
                    }
                }
            }
            let frontier = scratch
                .touched
                .iter()
                .map(|terminal_id| (*terminal_id, scratch.activations[*terminal_id as usize]))
                .collect::<Vec<_>>();
            (surface_re, surface_im, frontier)
        });
        let forward_us = trace_started.elapsed().as_micros();
        let operator_reserve = if should_expand_operator_lattice(exact_terminals.len(), limit) {
            self.operator_reserve(surface, &lexical_observed, !exact_terminals.is_empty())
        } else {
            Vec::new()
        };
        let operator_us = trace_started.elapsed().as_micros();
        let frontier_order = |left: &(u32, ForwardActivation), right: &(u32, ForwardActivation)| {
            exact_terminals
                .contains(&right.0)
                .cmp(&exact_terminals.contains(&left.0))
                .then_with(|| right.1.mass.cmp(&left.1.mass))
                .then_with(|| right.1.hits.cmp(&left.1.hits))
                .then_with(|| left.0.cmp(&right.0))
        };
        let touched_count = frontier.len();
        if let Some(trace_terminal) = readout_trace_terminal() {
            let selected_activation = frontier
                .iter()
                .find_map(|(terminal_id, activation)| {
                    (*terminal_id == trace_terminal).then_some(*activation)
                })
                .unwrap_or_default();
            let full_activation = self.activation_for_terminal(trace_terminal, &lexical_observed);
            let expected = self.character_anchors(trace_terminal);
            let reconstruction_modes =
                reconstruction_modes(character_sequence.as_slice(), expected);
            let selected_support_atoms = birth_atoms
                .iter()
                .filter(|(_, atom_id, _)| {
                    self.forward_couplings(*atom_id)
                        .iter()
                        .any(|coupling| coupling.peer_id == trace_terminal)
                })
                .count();
            let observed_support_atoms = lexical_observed
                .keys()
                .filter(|atom_id| {
                    self.forward_couplings(**atom_id)
                        .iter()
                        .any(|coupling| coupling.peer_id == trace_terminal)
                })
                .count();
            eprintln!(
                "l11_trace_terminal terminal={} touched={} selected_hits={} selected_mass={} \
                 full_hits={} full_mass={} reconstruction_modes={} observed_support_atoms={} \
                 selected_support_atoms={}",
                trace_terminal,
                selected_activation.hits != 0,
                selected_activation.hits,
                selected_activation.mass,
                full_activation.hits,
                full_activation.mass,
                reconstruction_modes,
                observed_support_atoms,
                selected_support_atoms,
            );
        }
        if frontier.len() > MAX_RECONSTRUCTION_SCAN {
            frontier.select_nth_unstable_by(MAX_RECONSTRUCTION_SCAN, frontier_order);
            frontier.truncate(MAX_RECONSTRUCTION_SCAN);
        }
        let reconstruction_reserve =
            self.reconstruction_lane_reserve(&frontier, character_sequence.as_slice());
        let reconstruction_us = trace_started.elapsed().as_micros();
        if frontier.len() > MAX_GEOMETRY_SCAN {
            frontier.select_nth_unstable_by(MAX_GEOMETRY_SCAN, frontier_order);
            frontier.truncate(MAX_GEOMETRY_SCAN);
        }
        frontier.sort_unstable_by(frontier_order);
        let geometry_reserve = if exact_terminals.is_empty() {
            self.geometry_reserve(&frontier, character_sequence.as_slice())
        } else {
            Vec::new()
        };
        let geometry_us = trace_started.elapsed().as_micros();
        let operator_reserve_count = operator_reserve.len();
        let reconstruction_reserve_count = reconstruction_reserve.len();
        let geometry_reserve_count = geometry_reserve.len();
        let geometry_reserve_ids = operator_reserve
            .iter()
            .chain(&reconstruction_reserve)
            .chain(&geometry_reserve)
            .map(|(terminal_id, _)| *terminal_id)
            .collect::<BTreeSet<_>>();
        frontier.truncate(MAX_PHASE_FRONTIER.max(limit));
        let primary_ids = frontier
            .iter()
            .map(|(terminal_id, _)| *terminal_id)
            .collect::<BTreeSet<_>>();
        let reconstruction_only_ids = reconstruction_reserve
            .iter()
            .map(|(terminal_id, _)| *terminal_id)
            .filter(|terminal_id| !primary_ids.contains(terminal_id))
            .collect::<BTreeSet<_>>();
        let mut retained = primary_ids;
        frontier.extend(
            operator_reserve
                .into_iter()
                .filter(|(terminal_id, _)| retained.insert(*terminal_id)),
        );
        frontier.extend(
            reconstruction_reserve
                .into_iter()
                .filter(|(terminal_id, _)| retained.insert(*terminal_id)),
        );
        frontier.extend(
            geometry_reserve
                .into_iter()
                .filter(|(terminal_id, _)| retained.insert(*terminal_id)),
        );
        let mut frontier_reverse =
            self.refresh_frontier_activations(&mut frontier, &lexical_observed);
        let activation_us = trace_started.elapsed().as_micros();
        if let Some(reverse) = frontier_reverse.take() {
            let mut retained_frontier = Vec::with_capacity(frontier.len());
            let mut retained_reverse = Vec::with_capacity(reverse.len());
            for (candidate, relations) in frontier.into_iter().zip(reverse) {
                if candidate.1.hits != 0 {
                    retained_frontier.push(candidate);
                    retained_reverse.push(relations);
                }
            }
            frontier = retained_frontier;
            frontier_reverse = Some(retained_reverse);
        } else {
            frontier.retain(|(_, activation)| activation.hits != 0);
        }
        let max_forward = frontier
            .iter()
            .map(|(_, activation)| activation.mass)
            .max()
            .unwrap_or(1)
            .max(1);
        if readout_trace_enabled() {
            eprintln!(
                "l11_readout_trace resolve_us={resolve_us} prefetch_us={} forward_us={} operator_us={} \
                 reconstruction_us={} geometry_us={} activation_us={} prepare_us={} touched={} retained={} \
                 operator_reserve={} reconstruction_reserve={} geometry_reserve={} birth_atoms={} \
                 birth_postings={}",
                prefetch_us.saturating_sub(resolve_us),
                forward_us.saturating_sub(prefetch_us),
                operator_us.saturating_sub(forward_us),
                reconstruction_us.saturating_sub(operator_us),
                geometry_us.saturating_sub(reconstruction_us),
                activation_us.saturating_sub(geometry_us),
                trace_started.elapsed().as_micros(),
                touched_count,
                retained.len(),
                operator_reserve_count,
                reconstruction_reserve_count,
                geometry_reserve_count,
                birth_atoms.len(),
                birth_postings,
            );
        }
        Some(PreparedReadout {
            observed: lexical_observed,
            character_sequence,
            observed_char_count,
            surface_re,
            surface_im,
            max_forward,
            frontier,
            frontier_reverse,
            geometry_reserve_ids,
            reconstruction_only_ids,
        })
    }

    fn exact_singleton_readout(&self, surface: &str) -> Option<GrokkingCandidate> {
        let terminal_id = self.exact_terminal_for_surface(surface)?;
        let observed = self.resolve_surface(surface);
        let character_sequence = observed_sequence(&observed, AtomChannel::CharacterAnchor);
        self.package.centers.get(terminal_id as usize)?;
        let reverse = self.reverse_couplings(terminal_id);
        let observed_char_count = normalize_lexical_surface(surface)
            .chars()
            .count()
            .min(u8::MAX as usize) as u8;
        let mut surface_re = [0_i32; WAVE_DIMENSION];
        let mut surface_im = [0_i32; WAVE_DIMENSION];
        let mut activation = ForwardActivation::default();
        for (atom_id, atom) in &observed {
            if is_anchor_channel(atom.channel) {
                continue;
            }
            let record = self.package.atoms.get(*atom_id as usize)?;
            expand_atom(
                &self.package.basis,
                record.wave_code,
                &mut surface_re,
                &mut surface_im,
                i32::from(atom.weight),
            );
            let coupling = reverse
                .iter()
                .find(|coupling| coupling.flags == 0 && coupling.peer_id == *atom_id);
            let Some(coupling) = coupling else {
                continue;
            };
            let contribution = u64::from(coupling.strength)
                .saturating_mul(u64::from(atom.weight))
                .saturating_mul(u64::from(position_coherence(
                    atom.position,
                    coupling.position_mode,
                )));
            activation.mass = activation.mass.saturating_add(contribution);
            activation.hits = activation.hits.saturating_add(1);
            if is_keyboard_channel(atom.channel) {
                activation.keyboard_hits = activation.keyboard_hits.saturating_add(1);
            } else {
                activation.surface_hits = activation.surface_hits.saturating_add(1);
            }
        }
        if activation.hits == 0 {
            return None;
        }
        let observed = observed
            .into_iter()
            .filter(|(_, atom)| !is_anchor_channel(atom.channel))
            .collect::<BTreeMap<_, _>>();
        let candidate = self.settle_candidate(
            terminal_id,
            activation,
            activation.mass.max(1),
            &observed,
            &surface_re,
            &surface_im,
            &character_sequence,
            observed_char_count,
            ReadoutMode::Full,
        )?;
        let mut candidates = vec![candidate];
        self.finalize_candidates(
            surface,
            1,
            ReadoutMode::Full,
            &surface_re,
            &surface_im,
            &mut candidates,
        );
        candidates.into_iter().next()
    }

    fn exact_terminal_for_surface(&self, surface: &str) -> Option<u32> {
        let observed = self.resolve_surface(surface);
        if observed.is_empty() {
            return None;
        }
        let character_sequence = observed_sequence(&observed, AtomChannel::CharacterAnchor);
        let exact_terminals = self.exact_terminals(character_sequence.as_slice());
        (exact_terminals.len() == 1).then(|| *exact_terminals.first().expect("one terminal"))
    }

    fn finish_readout(
        &self,
        surface: &str,
        limit: usize,
        mode: ReadoutMode,
        prepared: &PreparedReadout,
    ) -> Vec<GrokkingCandidate> {
        let mut candidates = self.settle_prepared_candidates(prepared, mode);
        self.finalize_candidates(
            surface,
            limit,
            mode,
            &prepared.surface_re,
            &prepared.surface_im,
            &mut candidates,
        );
        candidates
    }

    fn settle_prepared_candidates(
        &self,
        prepared: &PreparedReadout,
        mode: ReadoutMode,
    ) -> Vec<GrokkingCandidate> {
        let settle = |(index, (terminal_id, activation)): (usize, &(u32, ForwardActivation))| {
            let mut candidate = if let Some(reverse) = prepared
                .frontier_reverse
                .as_ref()
                .and_then(|relations| relations.get(index))
            {
                self.settle_candidate_with_reverse(
                    *terminal_id,
                    *activation,
                    prepared.max_forward,
                    &prepared.observed,
                    &prepared.surface_re,
                    &prepared.surface_im,
                    &prepared.character_sequence,
                    prepared.observed_char_count,
                    mode,
                    reverse,
                )
            } else {
                self.settle_candidate(
                    *terminal_id,
                    *activation,
                    prepared.max_forward,
                    &prepared.observed,
                    &prepared.surface_re,
                    &prepared.surface_im,
                    &prepared.character_sequence,
                    prepared.observed_char_count,
                    mode,
                )
            }?;
            candidate.ambiguity_shell = prepared
                .geometry_reserve_ids
                .contains(&candidate.terminal_id);
            candidate.reconstruction_only = prepared
                .reconstruction_only_ids
                .contains(&candidate.terminal_id);
            Some(candidate)
        };
        if matches!(self.relations, RelationStore::LazyV8(_)) {
            v8::runtime_pool_install(|| {
                prepared
                    .frontier
                    .par_iter()
                    .enumerate()
                    .filter_map(settle)
                    .collect()
            })
        } else {
            prepared
                .frontier
                .iter()
                .enumerate()
                .filter_map(settle)
                .collect()
        }
    }

    fn finalize_candidates(
        &self,
        surface: &str,
        limit: usize,
        mode: ReadoutMode,
        surface_re: &[i32; WAVE_DIMENSION],
        surface_im: &[i32; WAVE_DIMENSION],
        candidates: &mut Vec<GrokkingCandidate>,
    ) {
        self.apply_restoration_geometry(surface, candidates);
        self.finalize_candidates_after_geometry(limit, mode, surface_re, surface_im, candidates);
    }

    fn finalize_candidates_after_geometry(
        &self,
        limit: usize,
        mode: ReadoutMode,
        surface_re: &[i32; WAVE_DIMENSION],
        surface_im: &[i32; WAVE_DIMENSION],
        candidates: &mut Vec<GrokkingCandidate>,
    ) {
        apply_structural_interference(candidates);
        if mode != ReadoutMode::WithoutAnti {
            self.apply_pairwise_interference(candidates, surface_re, surface_im);
        }
        apply_sequence_certificate_interference(candidates, mode);
        if mode != ReadoutMode::WithoutPairwise {
            super::pairwise::apply_pairwise_field(
                &self.package.pair_profiles,
                &self.package.pair_centers,
                &self.package.basis,
                candidates,
                surface_re,
                surface_im,
            );
        }
        candidates.sort_unstable_by(candidate_order);
        apply_geometry_certificate_interference(candidates);
        if mode != ReadoutMode::WithoutPosition {
            apply_position_certificate_interference(candidates);
        }
        if let Some(trace_terminal) = readout_trace_terminal() {
            let before = candidates
                .iter()
                .position(|candidate| candidate.terminal_id == trace_terminal)
                .map(|index| {
                    (
                        index + 1,
                        candidates[index].reconstruction_only,
                        candidates[index].settled_energy,
                    )
                });
            eprintln!(
                "l11_trace_terminal_finalize terminal={} before_truncate={before:?}",
                trace_terminal
            );
        }
        truncate_with_reconstruction_tail(candidates, limit);
        if let Some(trace_terminal) = readout_trace_terminal() {
            let after = candidates
                .iter()
                .position(|candidate| candidate.terminal_id == trace_terminal)
                .map(|index| index + 1);
            eprintln!(
                "l11_trace_terminal_finalize terminal={} after_truncate={after:?}",
                trace_terminal
            );
        }
    }

    fn exact_terminals(&self, observed: &[u32]) -> BTreeSet<u32> {
        let hash = anchor_sequence_hash(observed);
        let start = self
            .exact_surface_index
            .partition_point(|(candidate_hash, _)| *candidate_hash < hash);
        let end = self
            .exact_surface_index
            .partition_point(|(candidate_hash, _)| *candidate_hash <= hash);
        self.exact_surface_index[start..end]
            .iter()
            .filter_map(|(_, terminal)| {
                (self.character_anchors(*terminal) == observed).then_some(*terminal)
            })
            .collect()
    }

    fn record_exact_terminals_for_chars(
        &self,
        chars: &[char],
        rank: u8,
        candidates: &mut BTreeMap<u32, u8>,
    ) {
        if chars.len() > MAX_ANCHOR_SEQUENCE {
            return;
        }
        let mut anchors = [0_u32; MAX_ANCHOR_SEQUENCE];
        for (index, ch) in chars.iter().enumerate() {
            let Some(atom_id) = self.package.graph.atom_id(NGramKey {
                channel: AtomChannel::CharacterAnchor,
                len: 1,
                units: [*ch as u32, 0, 0, 0],
            }) else {
                return;
            };
            anchors[index] = atom_id;
        }
        let anchors = &anchors[..chars.len()];
        let hash = anchor_sequence_hash(anchors);
        let start = self
            .exact_surface_index
            .partition_point(|(candidate_hash, _)| *candidate_hash < hash);
        let end = self
            .exact_surface_index
            .partition_point(|(candidate_hash, _)| *candidate_hash <= hash);
        for terminal_id in
            self.exact_surface_index[start..end]
                .iter()
                .filter_map(|(_, terminal_id)| {
                    (self.character_anchors(*terminal_id) == anchors).then_some(*terminal_id)
                })
        {
            candidates
                .entry(terminal_id)
                .and_modify(|current| *current = (*current).min(rank))
                .or_insert(rank);
        }
    }

    fn reconstruction_lane_reserve(
        &self,
        frontier: &[(u32, ForwardActivation)],
        observed: &[u32],
    ) -> Vec<(u32, ForwardActivation)> {
        let mut reserve = frontier
            .iter()
            .filter_map(|(terminal_id, activation)| {
                let expected = self.character_anchors(*terminal_id);
                let modes = reconstruction_modes(observed, &expected);
                (modes != 0).then_some((modes, *terminal_id, *activation))
            })
            .collect::<Vec<_>>();
        reserve.sort_unstable_by(|left, right| {
            reconstruction_mode_rank(right.0)
                .cmp(&reconstruction_mode_rank(left.0))
                .then_with(|| right.2.mass.cmp(&left.2.mass))
                .then_with(|| right.2.hits.cmp(&left.2.hits))
                .then_with(|| left.1.cmp(&right.1))
        });
        reserve.truncate(MAX_RECONSTRUCTION_RESERVE);
        reserve
            .into_iter()
            .map(|(_, terminal_id, activation)| (terminal_id, activation))
            .collect()
    }

    fn operator_reserve(
        &self,
        surface: &str,
        observed: &BTreeMap<u32, ObservedAtom>,
        expand_inverse_lattice: bool,
    ) -> Vec<(u32, ForwardActivation)> {
        let normalized = normalize_lexical_surface(surface);
        let chars = normalized.chars().collect::<Vec<_>>();
        if chars.is_empty() || chars.len() > MAX_ANCHOR_SEQUENCE {
            return Vec::new();
        }

        let mut ranked = BTreeMap::<u32, u8>::new();
        let raw_projected = crate::dict::convert(
            surface.trim(),
            crate::dict::detect_direction(surface.trim()),
        );
        let raw_projected = normalize_lexical_surface(&raw_projected);
        let raw_projected_chars = raw_projected.chars().collect::<Vec<_>>();
        if raw_projected_chars != chars {
            self.record_exact_operator_candidates(&raw_projected_chars, 0, &mut ranked);
        }
        let projected =
            crate::dict::convert(&normalized, crate::dict::detect_direction(&normalized));
        let projected_chars = projected.chars().collect::<Vec<_>>();
        if projected_chars != chars {
            self.record_exact_operator_candidates(&projected_chars, 0, &mut ranked);
        }

        let predecessors = chars
            .iter()
            .copied()
            .map(crate::nanda_wave::surface_damage::alphabet_predecessor)
            .collect::<Vec<_>>();
        for first in 0..chars.len() {
            let Some(first_value) = predecessors[first] else {
                continue;
            };
            for second in first + 1..chars.len() {
                let Some(second_value) = predecessors[second] else {
                    continue;
                };
                let mut repaired = chars.clone();
                repaired[first] = first_value;
                repaired[second] = second_value;
                self.record_exact_operator_candidates(&repaired, 1, &mut ranked);
            }
        }
        for (index, predecessor) in predecessors.into_iter().enumerate() {
            let Some(predecessor) = predecessor else {
                continue;
            };
            let mut repaired = chars.clone();
            repaired[index] = predecessor;
            self.record_exact_operator_candidates(&repaired, 2, &mut ranked);
        }
        for first in 0..chars.len() {
            for second in first + 1..chars.len() {
                if chars[first] == chars[second] {
                    continue;
                }
                let mut repaired = chars.clone();
                repaired.swap(first, second);
                self.record_exact_operator_candidates(&repaired, 3, &mut ranked);
            }
        }
        for index in 0..chars.len() {
            let mut repaired = chars.clone();
            repaired.remove(index);
            self.record_exact_operator_candidates(&repaired, 4, &mut ranked);
        }
        if chars.len() >= 2 {
            for index in 0..chars.len() - 1 {
                let mut repaired = chars.clone();
                repaired.drain(index..=index + 1);
                self.record_exact_operator_candidates(&repaired, 5, &mut ranked);
            }
        }
        if expand_inverse_lattice && chars.len() <= MAX_EXACT_COLLISION_OPERATOR_CHARS {
            self.record_inverse_operator_candidates(&chars, &mut ranked);
        }
        if let Some(trace_terminal) = readout_trace_terminal() {
            eprintln!(
                "l11_trace_operator terminal={} raw_projected={} ranked={:?}",
                trace_terminal,
                raw_projected,
                ranked.get(&trace_terminal),
            );
        }

        let mut reserve = ranked
            .into_iter()
            .filter_map(|(terminal_id, rank)| {
                let activation = self.activation_for_terminal(terminal_id, observed);
                (activation.hits != 0).then_some((rank, terminal_id, activation))
            })
            .collect::<Vec<_>>();
        reserve.sort_unstable_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| right.2.mass.cmp(&left.2.mass))
                .then_with(|| right.2.hits.cmp(&left.2.hits))
                .then_with(|| left.1.cmp(&right.1))
        });
        reserve.truncate(MAX_OPERATOR_RESERVE);
        reserve
            .into_iter()
            .map(|(_, terminal_id, activation)| (terminal_id, activation))
            .collect()
    }

    fn record_inverse_operator_candidates(
        &self,
        chars: &[char],
        candidates: &mut BTreeMap<u32, u8>,
    ) {
        let Some(alphabet) = chars
            .iter()
            .find_map(|ch| crate::nanda_wave::surface_damage::alphabet_for(*ch))
        else {
            return;
        };
        for insert_at in 0..=chars.len() {
            for inserted in alphabet.chars() {
                let mut repaired = Vec::with_capacity(chars.len() + 1);
                repaired.extend_from_slice(&chars[..insert_at]);
                repaired.push(inserted);
                repaired.extend_from_slice(&chars[insert_at..]);
                self.record_exact_operator_candidates(&repaired, 3, candidates);
                for swap_at in 0..repaired.len().saturating_sub(1) {
                    if repaired[swap_at] == repaired[swap_at + 1] {
                        continue;
                    }
                    repaired.swap(swap_at, swap_at + 1);
                    self.record_exact_operator_candidates(&repaired, 4, candidates);
                    repaired.swap(swap_at, swap_at + 1);
                }
            }
        }
    }

    fn record_exact_operator_candidates(
        &self,
        chars: &[char],
        rank: u8,
        candidates: &mut BTreeMap<u32, u8>,
    ) {
        self.record_exact_terminals_for_chars(chars, rank, candidates);
    }

    fn activation_for_terminal(
        &self,
        terminal_id: u32,
        observed: &BTreeMap<u32, ObservedAtom>,
    ) -> ForwardActivation {
        let reverse = self.reverse_couplings(terminal_id);
        self.activation_for_terminal_with_reverse(terminal_id, observed, &reverse)
    }

    fn activation_for_terminal_with_reverse(
        &self,
        terminal_id: u32,
        observed: &BTreeMap<u32, ObservedAtom>,
        reverse: &[WaveCoupling],
    ) -> ForwardActivation {
        let Some(_) = self.package.centers.get(terminal_id as usize) else {
            return ForwardActivation::default();
        };
        let mut activation = ForwardActivation::default();
        for coupling in reverse.iter().filter(|coupling| coupling.flags == 0) {
            let Some(atom) = observed.get(&coupling.peer_id) else {
                continue;
            };
            let contribution = u64::from(coupling.strength)
                .saturating_mul(u64::from(atom.weight))
                .saturating_mul(u64::from(position_coherence(
                    atom.position,
                    coupling.position_mode,
                )));
            activation.mass = activation.mass.saturating_add(contribution);
            activation.hits = activation.hits.saturating_add(1);
            if is_keyboard_channel(atom.channel) {
                activation.keyboard_hits = activation.keyboard_hits.saturating_add(1);
            } else {
                activation.surface_hits = activation.surface_hits.saturating_add(1);
            }
        }
        activation
    }

    fn geometry_reserve(
        &self,
        frontier: &[(u32, ForwardActivation)],
        observed: &[u32],
    ) -> Vec<(u32, ForwardActivation)> {
        let maximum_distance =
            usize::from(self.package.restoration_calibration.max_geometry_distance);
        let mut minimum_distance = usize::MAX;
        let mut measured = Vec::new();
        for (terminal_id, activation) in frontier {
            let expected = self.character_anchors(*terminal_id);
            if expected.len().abs_diff(observed.len()) > maximum_distance {
                continue;
            }
            let distance = damerau_distance(observed, &expected);
            if distance > maximum_distance {
                continue;
            }
            minimum_distance = minimum_distance.min(distance);
            measured.push((distance, *terminal_id, *activation));
        }
        let ambiguity_shell = minimum_distance.saturating_add(1).min(maximum_distance);
        measured.retain(|(distance, _, _)| *distance <= ambiguity_shell);
        measured.sort_unstable_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| right.2.mass.cmp(&left.2.mass))
                .then_with(|| right.2.hits.cmp(&left.2.hits))
                .then_with(|| left.1.cmp(&right.1))
        });
        measured.truncate(MAX_GEOMETRY_RESERVE);
        measured
            .into_iter()
            .map(|(_, terminal_id, activation)| (terminal_id, activation))
            .collect()
    }

    pub(super) fn classify_restoration(
        &self,
        surface: &str,
        candidates: &mut [GrokkingCandidate],
        calibration: super::restoration::RestorationCalibration,
    ) -> super::restoration::RestorationReadout {
        self.apply_l11_phase_evidence(surface, candidates);
        super::restoration::classify(candidates, calibration)
    }

    pub(super) fn apply_l11_phase_evidence(
        &self,
        surface: &str,
        candidates: &mut [GrokkingCandidate],
    ) {
        if self.package.center_phase_profiles.is_empty() {
            return;
        }
        let observed = self.resolve_surface(surface);
        let observed_character_sequence =
            observed_sequence(&observed, AtomChannel::CharacterAnchor);
        let normalized_char_count = normalize_lexical_surface(surface).chars().count();
        let character_anchors_cover_surface =
            observed_character_sequence.as_slice().len() == normalized_char_count;
        let mut surface_re = [0_i32; WAVE_DIMENSION];
        let mut surface_im = [0_i32; WAVE_DIMENSION];
        for (atom_id, atom) in &observed {
            if is_anchor_channel(atom.channel) {
                continue;
            }
            let Some(record) = self.package.atoms.get(*atom_id as usize) else {
                continue;
            };
            expand_atom(
                &self.package.basis,
                record.wave_code,
                &mut surface_re,
                &mut surface_im,
                i32::from(atom.weight),
            );
        }
        let minimum_geometry = candidates
            .iter()
            .map(|candidate| candidate.geometry_distance)
            .min()
            .unwrap_or(u8::MAX);
        let present = candidates
            .iter()
            .filter(|candidate| candidate.geometry_distance == minimum_geometry)
            .map(|candidate| candidate.terminal_id)
            .collect::<BTreeSet<_>>();
        let mut ambiguity_links = Vec::new();
        for candidate in candidates.iter_mut() {
            if !present.contains(&candidate.terminal_id) {
                continue;
            }
            let Some(profile) = self
                .package
                .center_phase_profiles
                .get(candidate.terminal_id as usize)
            else {
                continue;
            };
            candidate.positive_subcenter_milli = max_subcenter_coherence(
                &self.package.positive_subcenters,
                profile.positive_start,
                profile.positive_count,
                &self.package.basis,
                &surface_re,
                &surface_im,
                None,
            );
            candidate.anti_subcenter_milli = max_subcenter_coherence(
                &self.package.anti_subcenters,
                profile.anti_start,
                profile.anti_count,
                &self.package.basis,
                &surface_re,
                &surface_im,
                Some(&present),
            );
            candidate.hard_negative_milli = max_subcenter_coherence(
                &self.package.hard_negative_subcenters,
                profile.hard_negative_start,
                profile.hard_negative_count,
                &self.package.basis,
                &surface_re,
                &surface_im,
                Some(&present),
            );
            let ambiguity_start = profile.ambiguity_start as usize;
            let ambiguity_end = ambiguity_start.saturating_add(profile.ambiguity_count as usize);
            let mut geometry_linked_competitors = BTreeSet::new();
            for center in self
                .package
                .ambiguity_subcenters
                .get(ambiguity_start..ambiguity_end)
                .unwrap_or_default()
            {
                let relation = AmbiguityPhaseCenter64::from_record(*center);
                let threshold = relation.threshold_milli();
                if geometry_linked_competitors.contains(&center.decoder_terminal) {
                    continue;
                }
                let Some(_) = self.package.centers.get(candidate.terminal_id as usize) else {
                    continue;
                };
                let Some(_) = self.package.centers.get(center.decoder_terminal as usize) else {
                    continue;
                };
                let competitor_reverse = self.reverse_couplings(center.decoder_terminal);
                let competitor_character_sequence =
                    expected_sequence(&competitor_reverse, COUPLING_FLAG_CHARACTER_ANCHOR);
                let competitor_geometry = damerau_distance(
                    observed_character_sequence.as_slice(),
                    competitor_character_sequence.as_slice(),
                )
                .min(u8::MAX as usize) as u8;
                let geometry_link = character_anchors_cover_surface
                    && ambiguity_geometry_link(
                        candidate.geometry_distance,
                        competitor_geometry,
                        self.package.restoration_calibration.max_geometry_distance,
                    );
                if !candidate.exact_reconstruction && geometry_link {
                    if geometry_linked_competitors.insert(center.decoder_terminal) {
                        ambiguity_links.push((candidate.terminal_id, center.decoder_terminal));
                    }
                    continue;
                }
                if threshold == 0 {
                    continue;
                }
                let owner_reverse = self.reverse_couplings(candidate.terminal_id);
                let competitor_reverse = self.reverse_couplings(center.decoder_terminal);
                let (residual_re, residual_im) = expanded_pair_residual_wave(
                    &observed,
                    &self.package.atoms,
                    &self.package.basis,
                    &owner_reverse,
                    &competitor_reverse,
                );
                let (center_re, center_im) = expand_word(&self.package.basis, *center);
                let coherence =
                    complex_coherence_milli(&residual_re, &residual_im, &center_re, &center_im);
                candidate.ambiguity_milli = candidate.ambiguity_milli.max(coherence);
                candidate.ambiguity_threshold_milli =
                    candidate.ambiguity_threshold_milli.max(threshold);
                let phase_link = threshold != 0 && coherence >= threshold;
                if !candidate.exact_reconstruction && phase_link {
                    ambiguity_links.push((candidate.terminal_id, center.decoder_terminal));
                }
            }
        }
        for (owner, competitor) in ambiguity_links {
            let Some(owner_index) = candidates
                .iter()
                .position(|candidate| candidate.terminal_id == owner)
            else {
                continue;
            };
            let Some(competitor_index) = candidates
                .iter()
                .position(|candidate| candidate.terminal_id == competitor)
            else {
                candidates[owner_index].ambiguity_linked = true;
                continue;
            };
            if candidates[owner_index]
                .geometry_distance
                .abs_diff(candidates[competitor_index].geometry_distance)
                > 1
            {
                continue;
            }
            candidates[owner_index].ambiguity_linked = true;
            candidates[competitor_index].ambiguity_linked = true;
            let basin_distance = candidates[owner_index]
                .geometry_distance
                .min(candidates[competitor_index].geometry_distance);
            candidates[owner_index].geometry_distance = basin_distance;
            candidates[competitor_index].geometry_distance = basin_distance;
        }
        super::pairwise::apply_restoration_dominance_certificate(
            &self.package.pair_profiles,
            &self.package.pair_centers,
            &self.package.basis,
            candidates,
            &surface_re,
            &surface_im,
        );
    }

    fn apply_restoration_geometry(&self, surface: &str, candidates: &mut [GrokkingCandidate]) {
        let observed = self.resolve_surface(surface);
        let observed_character_sequence =
            observed_sequence(&observed, AtomChannel::CharacterAnchor);
        let normalized_char_count = normalize_lexical_surface(surface).chars().count();
        let character_anchors_cover_surface =
            observed_character_sequence.as_slice().len() == normalized_char_count;
        let observed_keyboard_sequence = observed_sequence(&observed, AtomChannel::KeyboardGram);
        let observed_physical_keys = physical_key_sequence(surface);
        let observed_script_flags = super::model::surface_script_flags(surface);
        for candidate in candidates {
            let Some(center) = self.package.centers.get(candidate.terminal_id as usize) else {
                continue;
            };
            let profile = self
                .package
                .center_phase_profiles
                .get(candidate.terminal_id as usize);
            let reverse = self.reverse_couplings(candidate.terminal_id);
            let expected_character_sequence =
                expected_sequence(&reverse, COUPLING_FLAG_CHARACTER_ANCHOR);
            candidate.reconstruction_modes = if character_anchors_cover_surface {
                reconstruction_modes(
                    observed_character_sequence.as_slice(),
                    expected_character_sequence.as_slice(),
                )
            } else {
                0
            };
            let expected_surface = self.decode_terminal(candidate.terminal_id);
            if let Some(expected_surface) = expected_surface.as_deref() {
                candidate.reconstruction_modes |=
                    surface_operator_reconstruction_modes(surface, expected_surface);
            }
            let cross_script = center.flags != 0
                && observed_script_flags != 0
                && center.flags & observed_script_flags == 0;
            if !cross_script {
                continue;
            }
            let generated_geometry;
            let (expected, uses_physical_keys) = if let Some(profile) = profile {
                let start = profile.keyboard_geometry_start as usize;
                let end = start.saturating_add(profile.keyboard_geometry_count as usize);
                let Some(expected) = self.package.keyboard_geometry_units.get(start..end) else {
                    continue;
                };
                (
                    expected,
                    profile.flags & CENTER_PHASE_FLAG_PHYSICAL_KEY_GEOMETRY != 0,
                )
            } else {
                let Some(expected_surface) = expected_surface.as_deref() else {
                    continue;
                };
                generated_geometry = physical_key_sequence(expected_surface);
                (generated_geometry.as_slice(), true)
            };
            if expected.is_empty() {
                continue;
            }
            let observed_geometry = if uses_physical_keys {
                observed_physical_keys.as_slice()
            } else {
                observed_keyboard_sequence.as_slice()
            };
            if observed_geometry.is_empty() {
                continue;
            }
            candidate.geometry_distance = candidate
                .geometry_distance
                .min(damerau_distance(observed_geometry, expected).min(u8::MAX as usize) as u8);
        }
    }

    pub(super) fn ambiguity_observations(
        &self,
        surface: &str,
        candidates: &[GrokkingCandidate],
    ) -> Vec<AmbiguityObservation> {
        let observed = self.resolve_surface(surface);
        if observed.is_empty() || candidates.is_empty() {
            return Vec::new();
        }
        let minimum_geometry = candidates
            .iter()
            .map(|candidate| candidate.geometry_distance)
            .min()
            .unwrap_or(u8::MAX);
        let mut observations = Vec::new();
        for owner in candidates
            .iter()
            .filter(|candidate| candidate.geometry_distance == minimum_geometry)
        {
            let Some(_) = self.package.centers.get(owner.terminal_id as usize) else {
                continue;
            };
            let Some(profile) = self
                .package
                .center_phase_profiles
                .get(owner.terminal_id as usize)
            else {
                continue;
            };
            let start = profile.ambiguity_start as usize;
            let end = start.saturating_add(profile.ambiguity_count as usize);
            for (offset, relation_center) in self
                .package
                .ambiguity_subcenters
                .get(start..end)
                .unwrap_or_default()
                .iter()
                .enumerate()
            {
                let Some(_) = self
                    .package
                    .centers
                    .get(relation_center.decoder_terminal as usize)
                else {
                    continue;
                };
                let owner_reverse = self.reverse_couplings(owner.terminal_id);
                let competitor_reverse = self.reverse_couplings(relation_center.decoder_terminal);
                let (residual_re, residual_im) = expanded_pair_residual_wave(
                    &observed,
                    &self.package.atoms,
                    &self.package.basis,
                    &owner_reverse,
                    &competitor_reverse,
                );
                let (center_re, center_im) = expand_word(&self.package.basis, *relation_center);
                let coherence =
                    complex_coherence_milli(&residual_re, &residual_im, &center_re, &center_im);
                let structurally_applicable = candidates
                    .iter()
                    .find(|candidate| candidate.terminal_id == relation_center.decoder_terminal)
                    .is_some_and(|competitor| {
                        owner
                            .geometry_distance
                            .abs_diff(competitor.geometry_distance)
                            <= 1
                    });
                observations.push(AmbiguityObservation {
                    center_index: start + offset,
                    owner: owner.terminal_id,
                    competitor: relation_center.decoder_terminal,
                    coherence_milli: coherence,
                    structurally_applicable,
                });
            }
        }
        observations
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

    fn resolve_surface(&self, surface: &str) -> Vec<(u32, ObservedAtom)> {
        encode_wave_surface(surface)
            .into_iter()
            .filter_map(|atom| {
                self.package.graph.atom_id(atom.key).map(|atom_id| {
                    (
                        atom_id,
                        ObservedAtom {
                            position: (atom.position / 257).min(255) as u8,
                            weight: atom.weight,
                            channel: atom.key.channel,
                        },
                    )
                })
            })
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    fn settle_candidate(
        &self,
        terminal_id: u32,
        activation: ForwardActivation,
        max_forward: u64,
        observed: &BTreeMap<u32, ObservedAtom>,
        surface_re: &[i32; WAVE_DIMENSION],
        surface_im: &[i32; WAVE_DIMENSION],
        character_sequence: &AnchorSequence,
        observed_char_count: u8,
        mode: ReadoutMode,
    ) -> Option<GrokkingCandidate> {
        let reverse = self.reverse_couplings(terminal_id);
        self.settle_candidate_with_reverse(
            terminal_id,
            activation,
            max_forward,
            observed,
            surface_re,
            surface_im,
            character_sequence,
            observed_char_count,
            mode,
            &reverse,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn settle_candidate_with_reverse(
        &self,
        terminal_id: u32,
        activation: ForwardActivation,
        max_forward: u64,
        observed: &BTreeMap<u32, ObservedAtom>,
        surface_re: &[i32; WAVE_DIMENSION],
        surface_im: &[i32; WAVE_DIMENSION],
        character_sequence: &AnchorSequence,
        observed_char_count: u8,
        mode: ReadoutMode,
        reverse: &[WaveCoupling],
    ) -> Option<GrokkingCandidate> {
        let center = *self.package.centers.get(terminal_id as usize)?;
        let expected_char_count = center.surface_len;
        let anchors_cover_surface =
            character_sequence.as_slice().len() == usize::from(observed_char_count);
        let legacy_sequence_milli =
            if observed_char_count < expected_char_count && anchors_cover_surface {
                legacy_reconstruction_sequence_milli(&reverse, character_sequence)
            } else {
                750
            };
        let expected_character_sequence =
            expected_sequence(&reverse, COUPLING_FLAG_CHARACTER_ANCHOR);
        let character_distance = damerau_distance(
            character_sequence.as_slice(),
            expected_character_sequence.as_slice(),
        );
        let geometry_distance = character_distance.min(u8::MAX as usize) as u8;
        let exact_reconstruction = observed_char_count == expected_char_count
            && character_sequence.as_slice() == expected_character_sequence.as_slice();
        let position_milli = if observed_char_count == expected_char_count
            && anchors_cover_surface
            && activation.surface_hits > activation.keyboard_hits
        {
            exact_position_coherence_milli(
                character_sequence.as_slice(),
                expected_character_sequence.as_slice(),
            )
        } else {
            0
        };
        let sequence_milli = match mode {
            ReadoutMode::WithoutSequence => 750,
            ReadoutMode::LegacySequence => legacy_sequence_milli,
            _ if anchors_cover_surface && activation.surface_hits > activation.keyboard_hits => {
                legacy_sequence_milli
                    .max(reconstruction_sequence_milli(&reverse, character_sequence))
            }
            _ => legacy_sequence_milli,
        };
        let lexical_reverse = reverse.iter().filter(|coupling| coupling.flags == 0);
        let expected_mass = lexical_reverse
            .clone()
            .map(|coupling| u64::from(coupling.strength))
            .sum::<u64>()
            .max(1);
        let backward_mass =
            lexical_reverse
                .filter_map(|coupling| {
                    let atom = observed.get(&coupling.peer_id)?;
                    Some(u64::from(coupling.strength).saturating_mul(u64::from(
                        position_coherence(atom.position, coupling.position_mode),
                    )))
                })
                .sum::<u64>();
        let forward_milli = (activation.mass.saturating_mul(1_000) / max_forward) as u16;
        let backward_milli = (backward_mass.saturating_mul(1_000)
            / expected_mass.saturating_mul(256))
        .min(1_000) as u16;
        let positive_milli = if mode == ReadoutMode::WithoutPhase {
            500
        } else {
            let (center_re, center_im) = expand_word(&self.package.basis, center);
            complex_coherence_milli(surface_re, surface_im, &center_re, &center_im)
        };
        let anti_milli = 0;
        let length_milli = 1_000_u16.saturating_sub(
            u16::from(observed_char_count.abs_diff(expected_char_count)).saturating_mul(180),
        );
        let energy =
            base_settled_energy(forward_milli, backward_milli, positive_milli, length_milli);
        let legacy_settled_energy = with_sequence_energy(energy, legacy_sequence_milli);
        let energy = with_sequence_energy(energy, sequence_milli);
        Some(GrokkingCandidate {
            terminal_id,
            atom_hits: activation.hits,
            surface_hits: activation.surface_hits,
            keyboard_hits: activation.keyboard_hits,
            structural_milli: 0,
            position_milli,
            legacy_sequence_milli,
            sequence_milli,
            forward_milli,
            backward_milli,
            positive_milli,
            positive_subcenter_milli: 0,
            anti_milli,
            anti_subcenter_milli: 0,
            hard_negative_milli: 0,
            ambiguity_milli: 0,
            ambiguity_threshold_milli: 0,
            ambiguity_linked: false,
            ambiguity_shell: false,
            reconstruction_only: false,
            pairwise_loss_milli: 0,
            crystallization_wins: 0,
            crystallization_required: 0,
            crystallization_margin_milli: 0,
            crystallization_complete: false,
            crystallization_known_edges: 0,
            crystallization_unknown_edges: 0,
            crystallization_tied_edges: 0,
            crystallization_conflicts: 0,
            crystallization_cycles: 0,
            length_milli,
            geometry_distance,
            reconstruction_modes: 0,
            settled_energy: energy,
            legacy_settled_energy,
            length_relation: length_relation(observed_char_count, expected_char_count),
            settling_iterations: SETTLING_ITERATIONS,
            exact_reconstruction,
        })
    }

    fn forward_coupling_views(&self, atom_ids: &[u32]) -> Vec<CouplingView<'_>> {
        match &self.relations {
            RelationStore::Eager => atom_ids
                .iter()
                .map(|atom_id| self.forward_couplings(*atom_id))
                .collect(),
            RelationStore::LazyV8(artifact) => artifact
                .postings(atom_ids)
                .unwrap_or_else(|_| {
                    atom_ids
                        .iter()
                        .map(|_| Arc::from(Vec::<WaveCoupling>::new()))
                        .collect()
                })
                .into_iter()
                .map(CouplingView::Shared)
                .collect(),
        }
    }

    fn refresh_frontier_activations(
        &self,
        frontier: &mut [(u32, ForwardActivation)],
        observed: &BTreeMap<u32, ObservedAtom>,
    ) -> Option<Vec<Arc<[WaveCoupling]>>> {
        if matches!(self.relations, RelationStore::Eager) || frontier.len() < 2 {
            for (terminal_id, activation) in frontier {
                *activation = self.activation_for_terminal(*terminal_id, observed);
            }
            return None;
        }
        let refreshed = v8::runtime_pool_install(|| {
            frontier
                .par_iter()
                .map(|(terminal_id, _)| {
                    let reverse = self.reverse_couplings_shared(*terminal_id);
                    let activation =
                        self.activation_for_terminal_with_reverse(*terminal_id, observed, &reverse);
                    (activation, reverse)
                })
                .collect::<Vec<_>>()
        });
        let mut reverse = Vec::with_capacity(refreshed.len());
        for ((_, activation), (refreshed_activation, relations)) in
            frontier.iter_mut().zip(refreshed)
        {
            *activation = refreshed_activation;
            reverse.push(relations);
        }
        Some(reverse)
    }

    fn forward_couplings(&self, atom_id: u32) -> CouplingView<'_> {
        match &self.relations {
            RelationStore::Eager => {
                let Some(record) = self.package.atoms.get(atom_id as usize) else {
                    return CouplingView::Borrowed(&[]);
                };
                let start = record.coupling_start as usize;
                let end = start.saturating_add(record.coupling_count as usize);
                CouplingView::Borrowed(
                    self.package
                        .forward_couplings
                        .get(start..end)
                        .unwrap_or_default(),
                )
            }
            RelationStore::LazyV8(artifact) => CouplingView::Shared(
                artifact
                    .posting(atom_id)
                    .unwrap_or_else(|_| Arc::from(Vec::<WaveCoupling>::new())),
            ),
        }
    }

    pub(super) fn forward_degree(&self, atom_id: u32) -> usize {
        match &self.relations {
            RelationStore::Eager => self
                .package
                .atoms
                .get(atom_id as usize)
                .map(|record| record.coupling_count as usize)
                .unwrap_or_default(),
            RelationStore::LazyV8(artifact) => artifact.posting_degree(atom_id),
        }
    }

    pub(super) fn forward_relation_count(&self) -> usize {
        match &self.relations {
            RelationStore::Eager => self.package.forward_couplings.len(),
            RelationStore::LazyV8(artifact) => artifact.forward_relation_count(),
        }
    }

    pub(super) fn reverse_relation_count(&self) -> usize {
        match &self.relations {
            RelationStore::Eager => self.package.reverse_couplings.len(),
            RelationStore::LazyV8(artifact) => artifact.reverse_relation_count(),
        }
    }

    fn reverse_couplings(&self, terminal_id: u32) -> CouplingView<'_> {
        if matches!(self.relations, RelationStore::Eager) {
            let Some(center) = self.package.centers.get(terminal_id as usize) else {
                return CouplingView::Borrowed(&[]);
            };
            let start = center.coupling_start as usize;
            let end = start.saturating_add(center.coupling_count as usize);
            return CouplingView::Borrowed(
                self.package
                    .reverse_couplings
                    .get(start..end)
                    .unwrap_or_default(),
            );
        }
        CouplingView::Shared(self.reverse_couplings_shared(terminal_id))
    }

    fn reverse_couplings_shared(&self, terminal_id: u32) -> Arc<[WaveCoupling]> {
        let cache_budget = reverse_cache_bytes();
        if cache_budget != 0 {
            if let Ok(cache) = self.reverse_cache.lock() {
                if let Some(relations) = cache.entries.get(&terminal_id) {
                    return Arc::clone(relations);
                }
            }
        }
        let relations: Arc<[WaveCoupling]> =
            format::reconstruct_compact_center_reverse(&self.package, terminal_id)
                .unwrap_or_default()
                .into();
        let relation_bytes = relations
            .len()
            .saturating_mul(std::mem::size_of::<WaveCoupling>());
        if relation_bytes <= cache_budget {
            if let Ok(mut cache) = self.reverse_cache.lock() {
                if let Some(existing) = cache.entries.get(&terminal_id) {
                    return Arc::clone(existing);
                }
                while cache.bytes.saturating_add(relation_bytes) > cache_budget {
                    let Some(evicted_id) = cache.order.pop_front() else {
                        break;
                    };
                    let Some(evicted) = cache.entries.remove(&evicted_id) else {
                        continue;
                    };
                    cache.bytes = cache.bytes.saturating_sub(
                        evicted
                            .len()
                            .saturating_mul(std::mem::size_of::<WaveCoupling>()),
                    );
                }
                cache.bytes = cache.bytes.saturating_add(relation_bytes);
                cache.order.push_back(terminal_id);
                cache.entries.insert(terminal_id, Arc::clone(&relations));
            }
        }
        relations
    }

    fn apply_pairwise_interference(
        &self,
        candidates: &mut [GrokkingCandidate],
        surface_re: &[i32; WAVE_DIMENSION],
        surface_im: &[i32; WAVE_DIMENSION],
    ) {
        let present = candidates
            .iter()
            .map(|candidate| candidate.terminal_id)
            .collect::<BTreeSet<_>>();
        for candidate in candidates {
            let Some(center) = self.package.centers.get(candidate.terminal_id as usize) else {
                continue;
            };
            let start = center.anti_start as usize;
            let end = start.saturating_add(center.anti_count as usize);
            let pressure = self
                .package
                .anti_centers
                .get(start..end)
                .unwrap_or_default()
                .iter()
                .filter(|anti_center| present.contains(&anti_center.decoder_terminal))
                .map(|anti_center| {
                    let (anti_re, anti_im) = expand_word(&self.package.basis, *anti_center);
                    complex_coherence_milli(surface_re, surface_im, &anti_re, &anti_im)
                        .saturating_sub(candidate.positive_milli.saturating_add(24))
                        .saturating_mul(10)
                        .min(1_000)
                })
                .max()
                .unwrap_or_default();
            candidate.anti_milli = pressure;
            candidate.settled_energy = candidate
                .settled_energy
                .saturating_sub(i32::from(pressure).saturating_mul(4));
            candidate.legacy_settled_energy = candidate
                .legacy_settled_energy
                .saturating_sub(i32::from(pressure).saturating_mul(4));
        }
    }
}

impl L1RestorationHost {
    pub fn load(package_path: &Path) -> io::Result<Self> {
        let Some(spec) = super::composite::load_spec(package_path)? else {
            let package_bytes = std::fs::metadata(package_path)?.len() as usize;
            let memory = LexicalGrokkingMemory::load(package_path).map_err(io::Error::other)?;
            return Ok(Self {
                package_path: package_path.to_path_buf(),
                package_bytes,
                memory,
                overlays: Vec::new(),
                tombstones: BTreeSet::new(),
                manifest_generation: 0,
            });
        };
        let memory = LexicalGrokkingMemory::load(&spec.base_path).map_err(io::Error::other)?;
        let mut terminal_offset = memory.package.terminal_count();
        let mut package_bytes =
            spec.manifest_bytes as usize + std::fs::metadata(&spec.base_path)?.len() as usize;
        let mut overlays = Vec::with_capacity(spec.delta_paths.len());
        for delta_path in &spec.delta_paths {
            let delta = LexicalGrokkingMemory::load(delta_path).map_err(io::Error::other)?;
            let next_offset = terminal_offset
                .checked_add(delta.package.terminal_count())
                .ok_or_else(|| io::Error::other("L1.1 composite terminal ID overflow"))?;
            package_bytes = package_bytes
                .checked_add(std::fs::metadata(delta_path)?.len() as usize)
                .ok_or_else(|| io::Error::other("L1.1 composite byte count overflow"))?;
            overlays.push(L1OverlayMemory {
                terminal_offset,
                memory: delta,
            });
            terminal_offset = next_offset;
        }
        Ok(Self {
            package_path: package_path.to_path_buf(),
            package_bytes,
            memory,
            overlays,
            tombstones: spec.tombstones,
            manifest_generation: spec.generation,
        })
    }

    pub fn reload(&mut self, package_path: &Path) -> io::Result<()> {
        *self = Self::load(package_path)?;
        Ok(())
    }

    pub fn package_path(&self) -> &Path {
        &self.package_path
    }

    pub fn corpus_fingerprint(&self) -> u64 {
        self.memory.package.corpus_hash
    }

    pub fn terminal_count(&self) -> u32 {
        self.overlays
            .last()
            .map(|overlay| {
                overlay
                    .terminal_offset
                    .saturating_add(overlay.memory.package.terminal_count())
            })
            .unwrap_or_else(|| self.memory.package.terminal_count())
    }

    pub fn decode_terminal(&self, terminal_id: u32) -> Option<String> {
        let surface = if terminal_id < self.memory.package.terminal_count() {
            self.memory.decode_terminal(terminal_id)
        } else {
            self.overlays.iter().find_map(|overlay| {
                let local_id = terminal_id.checked_sub(overlay.terminal_offset)?;
                (local_id < overlay.memory.package.terminal_count())
                    .then(|| overlay.memory.decode_terminal(local_id))
                    .flatten()
            })
        }?;
        (!self.is_tombstoned(&surface)).then_some(surface)
    }

    pub fn terminal_for_exact_surface(&self, surface: &str) -> Option<u32> {
        if self.is_tombstoned(surface) {
            return None;
        }
        self.overlays
            .iter()
            .rev()
            .find_map(|overlay| {
                overlay
                    .memory
                    .exact_terminal_for_surface(surface)
                    .and_then(|terminal_id| overlay.terminal_offset.checked_add(terminal_id))
            })
            .or_else(|| self.memory.exact_terminal_for_surface(surface))
    }

    pub fn restore(&self, surface: &str, limit: usize) -> serde_json::Value {
        if self.is_composite() {
            let candidates = self
                .lattice_seed_rows(surface, limit.max(1))
                .into_iter()
                .map(|(terminal_id, surface, score_milli)| {
                    serde_json::json!({
                        "terminal_id": terminal_id,
                        "surface": surface,
                        "score_milli": score_milli,
                    })
                })
                .collect::<Vec<_>>();
            let verdict = if candidates.is_empty() {
                "abstain"
            } else {
                "lattice"
            };
            return serde_json::json!({
                "package": self.package_path,
                "input": surface,
                "terminal_count": self.terminal_count(),
                "manifest_generation": self.manifest_generation,
                "result": {
                    "verdict": verdict,
                    "authority": false,
                    "reason": "append_only_overlay_requires_composite_proof",
                    "candidates": candidates,
                },
            });
        }
        let mut candidates = self
            .memory
            .readout(surface, limit.max(1), ReadoutMode::Full);
        let readout = self.memory.classify_restoration(
            surface,
            &mut candidates,
            self.memory.package.restoration_calibration,
        );
        let result = match readout {
            super::restoration::RestorationReadout::Winner { candidate } => {
                serde_json::json!({
                    "verdict": "winner",
                    "authority": true,
                    "candidate": restoration_candidate_json(&self.memory, candidate),
                })
            }
            super::restoration::RestorationReadout::Tied {
                geometry_distance,
                candidates,
            } => serde_json::json!({
                "verdict": "tied",
                "authority": false,
                "geometry_distance": geometry_distance,
                "candidates": candidates
                    .into_iter()
                    .map(|candidate| restoration_candidate_json(&self.memory, candidate))
                    .collect::<Vec<_>>(),
            }),
            super::restoration::RestorationReadout::TiedOverflow {
                geometry_distance,
                total_candidates,
                candidates,
            } => serde_json::json!({
                "verdict": "tied_overflow",
                "authority": false,
                "geometry_distance": geometry_distance,
                "total_candidates": total_candidates,
                "candidates": candidates
                    .into_iter()
                    .map(|candidate| restoration_candidate_json(&self.memory, candidate))
                    .collect::<Vec<_>>(),
            }),
            super::restoration::RestorationReadout::Abstain {
                reason,
                geometry_distance,
                candidates,
            } => serde_json::json!({
                "verdict": "abstain",
                "authority": false,
                "reason": reason,
                "geometry_distance": geometry_distance,
                "candidates": candidates
                    .into_iter()
                    .map(|candidate| restoration_candidate_json(&self.memory, candidate))
                    .collect::<Vec<_>>(),
            }),
        };
        serde_json::json!({
            "package": self.package_path,
            "input": surface,
            "terminal_count": self.memory.package.terminal_count(),
            "result": result,
        })
    }

    pub fn lattice(&self, surface: &str, limit: usize) -> serde_json::Value {
        if self.is_composite() {
            let candidates = self
                .lattice_seed_rows(surface, limit.max(1))
                .into_iter()
                .map(|(terminal_id, surface, score_milli)| {
                    serde_json::json!({
                        "terminal_id": terminal_id,
                        "surface": surface,
                        "score_milli": score_milli,
                    })
                })
                .collect::<Vec<_>>();
            return serde_json::json!({
                "package": self.package_path,
                "input": surface,
                "terminal_count": self.terminal_count(),
                "manifest_generation": self.manifest_generation,
                "result": {
                    "verdict": "lattice",
                    "authority": false,
                    "candidates": candidates,
                },
            });
        }
        let candidates = self
            .memory
            .readout(surface, limit.max(1), ReadoutMode::Full)
            .into_iter()
            .map(|candidate| candidate_json(&self.memory, candidate))
            .collect::<Vec<_>>();
        serde_json::json!({
            "package": self.package_path,
            "input": surface,
            "terminal_count": self.memory.package.terminal_count(),
            "result": {
                "verdict": "lattice",
                "authority": false,
                "candidates": candidates,
            },
        })
    }

    pub fn lattice_seed_rows(&self, surface: &str, limit: usize) -> Vec<(u32, String, u32)> {
        let limit = limit.max(1);
        let mut rows = memory_seed_rows(&self.memory, 0, surface, limit);
        for overlay in &self.overlays {
            rows.extend(memory_seed_rows(
                &overlay.memory,
                overlay.terminal_offset,
                surface,
                limit,
            ));
        }
        rows.retain(|(_, candidate_surface, _)| !self.is_tombstoned(candidate_surface));
        rows.sort_unstable_by(|left, right| {
            right
                .2
                .cmp(&left.2)
                .then_with(|| left.1.cmp(&right.1))
                .then_with(|| left.0.cmp(&right.0))
        });
        let mut seen = BTreeSet::new();
        rows.retain(|(_, candidate_surface, _)| {
            seen.insert(normalize_lexical_surface(candidate_surface))
        });
        rows.truncate(limit);
        rows
    }

    pub fn stats(&self) -> L1RestorationHostStats {
        L1RestorationHostStats {
            package_path: self.package_path.clone(),
            package_bytes: self.package_bytes,
            terminal_count: self.terminal_count(),
            atom_count: self.memory.package.atoms.len()
                + self
                    .overlays
                    .iter()
                    .map(|overlay| overlay.memory.package.atoms.len())
                    .sum::<usize>(),
            forward_relations: self.memory.forward_relation_count()
                + self
                    .overlays
                    .iter()
                    .map(|overlay| overlay.memory.forward_relation_count())
                    .sum::<usize>(),
            reverse_relations: self.memory.reverse_relation_count()
                + self
                    .overlays
                    .iter()
                    .map(|overlay| overlay.memory.reverse_relation_count())
                    .sum::<usize>(),
            exact_surface_count: self.memory.exact_surface_index.len()
                + self
                    .overlays
                    .iter()
                    .map(|overlay| overlay.memory.exact_surface_index.len())
                    .sum::<usize>(),
            character_anchor_count: self.terminal_count() as usize,
            manifest_generation: self.manifest_generation,
            delta_count: self.overlays.len(),
            tombstone_count: self.tombstones.len(),
        }
    }

    fn is_tombstoned(&self, surface: &str) -> bool {
        self.tombstones
            .contains(&normalize_lexical_surface(surface))
    }

    fn is_composite(&self) -> bool {
        self.manifest_generation != 0
    }
}

fn memory_seed_rows(
    memory: &LexicalGrokkingMemory,
    terminal_offset: u32,
    surface: &str,
    limit: usize,
) -> Vec<(u32, String, u32)> {
    memory
        .readout(surface, limit, ReadoutMode::Full)
        .into_iter()
        .filter_map(|candidate| {
            let terminal_id = terminal_offset.checked_add(candidate.terminal_id)?;
            let surface = memory.decode_terminal(candidate.terminal_id)?;
            Some((terminal_id, surface, lattice_seed_score(candidate)))
        })
        .collect()
}

fn lattice_seed_score(candidate: GrokkingCandidate) -> u32 {
    let geometry_bonus =
        256_u64.saturating_sub(u64::from(candidate.geometry_distance).saturating_mul(24));
    u64::from(candidate.positive_milli)
        .saturating_add(u64::from(candidate.backward_milli))
        .saturating_add(u64::from(candidate.crystallization_margin_milli))
        .saturating_add(geometry_bonus)
        .saturating_sub(u64::from(candidate.anti_milli))
        .saturating_sub(u64::from(candidate.hard_negative_milli))
        .saturating_sub(u64::from(candidate.ambiguity_milli) / 2)
        .min(u64::from(u32::MAX)) as u32
}

fn expanded_pair_residual_wave(
    observed: &[(u32, ObservedAtom)],
    atoms: &[super::model::AtomRecord],
    basis: &[super::crystal::ComplexBasisWave],
    owner_reverse: &[WaveCoupling],
    competitor_reverse: &[WaveCoupling],
) -> ([i32; WAVE_DIMENSION], [i32; WAVE_DIMENSION]) {
    let residual = pair_residual_atoms(
        observed
            .iter()
            .filter(|(_, atom)| !is_anchor_channel(atom.channel))
            .map(|(atom_id, atom)| (*atom_id, atom.position)),
        owner_reverse
            .iter()
            .filter(|coupling| coupling.flags == 0)
            .map(|coupling| (coupling.peer_id, coupling.position_mode)),
        competitor_reverse
            .iter()
            .filter(|coupling| coupling.flags == 0)
            .map(|coupling| (coupling.peer_id, coupling.position_mode)),
    );
    let mut re = [0_i32; WAVE_DIMENSION];
    let mut im = [0_i32; WAVE_DIMENSION];
    for relation in residual {
        if let Some(atom) = atoms.get(relation.atom_id as usize) {
            expand_atom(
                basis,
                positioned_atom_code(atom.wave_code, relation.position_mode),
                &mut re,
                &mut im,
                relation.coefficient,
            );
        }
    }
    (re, im)
}

fn max_subcenter_coherence(
    centers: &[super::crystal::WordCenter64],
    start: u32,
    count: u8,
    basis: &[super::crystal::ComplexBasisWave],
    surface_re: &[i32; WAVE_DIMENSION],
    surface_im: &[i32; WAVE_DIMENSION],
    active_owners: Option<&BTreeSet<u32>>,
) -> u16 {
    centers
        .get(start as usize..start as usize + count as usize)
        .unwrap_or_default()
        .iter()
        .filter(|center| {
            active_owners.map_or(true, |owners| owners.contains(&center.decoder_terminal))
        })
        .map(|center| {
            let (center_re, center_im) = expand_word(basis, *center);
            complex_coherence_milli(surface_re, surface_im, &center_re, &center_im)
        })
        .max()
        .unwrap_or_default()
}

pub(super) fn damerau_distance(left: &[u32], right: &[u32]) -> usize {
    const STACK_WIDTH: usize = MAX_ANCHOR_SEQUENCE + 1;
    if right.len() >= STACK_WIDTH {
        return damerau_distance_heap(left, right);
    }
    let mut previous_previous = [0_usize; STACK_WIDTH];
    let mut previous = [0_usize; STACK_WIDTH];
    let mut current = [0_usize; STACK_WIDTH];
    for (column, slot) in previous.iter_mut().take(right.len() + 1).enumerate() {
        *slot = column;
    }
    for row in 1..=left.len() {
        current[0] = row;
        for column in 1..=right.len() {
            let substitution = usize::from(left[row - 1] != right[column - 1]);
            let mut distance = previous[column]
                .saturating_add(1)
                .min(current[column - 1].saturating_add(1))
                .min(previous[column - 1].saturating_add(substitution));
            if row > 1
                && column > 1
                && left[row - 1] == right[column - 2]
                && left[row - 2] == right[column - 1]
            {
                distance = distance.min(previous_previous[column - 2].saturating_add(1));
            }
            current[column] = distance;
        }
        std::mem::swap(&mut previous_previous, &mut previous);
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}

fn damerau_distance_heap(left: &[u32], right: &[u32]) -> usize {
    let width = right.len() + 1;
    let mut matrix = vec![0_usize; (left.len() + 1) * width];
    for row in 0..=left.len() {
        matrix[row * width] = row;
    }
    for (column, slot) in matrix.iter_mut().take(width).enumerate() {
        *slot = column;
    }
    for row in 1..=left.len() {
        for column in 1..=right.len() {
            let substitution = usize::from(left[row - 1] != right[column - 1]);
            let mut distance = matrix[(row - 1) * width + column]
                .saturating_add(1)
                .min(matrix[row * width + column - 1].saturating_add(1))
                .min(matrix[(row - 1) * width + column - 1].saturating_add(substitution));
            if row > 1
                && column > 1
                && left[row - 1] == right[column - 2]
                && left[row - 2] == right[column - 1]
            {
                distance = distance.min(matrix[(row - 2) * width + column - 2].saturating_add(1));
            }
            matrix[row * width + column] = distance;
        }
    }
    matrix[left.len() * width + right.len()]
}

pub(super) fn reconstruction_modes(observed: &[u32], expected: &[u32]) -> u8 {
    let missing = expected.len().saturating_sub(observed.len());
    if !(1..=2).contains(&missing) {
        return 0;
    }
    let mut modes = 0;
    let ordered_subsequence = is_ordered_subsequence(observed, expected);
    if missing == 1 && ordered_subsequence {
        if observed == &expected[..expected.len() - 1] {
            modes |= RECONSTRUCTION_MODE_SUFFIX_TRUNCATION;
        } else if observed == &expected[1..] {
            modes |= RECONSTRUCTION_MODE_PREFIX_TRUNCATION;
        } else {
            modes |= RECONSTRUCTION_MODE_SINGLE_DELETION;
        }
    }
    if missing == 2 && ordered_subsequence {
        modes |= RECONSTRUCTION_MODE_DELETION;
    }
    if missing == 1
        && !ordered_subsequence
        && is_subsequence_after_one_adjacent_swap(observed, expected)
    {
        modes |= RECONSTRUCTION_MODE_DELETION_TRANSPOSITION;
    }
    modes
}

pub(super) fn surface_operator_reconstruction_modes(observed: &str, expected: &str) -> u8 {
    let observed = normalize_lexical_surface(observed)
        .chars()
        .collect::<Vec<_>>();
    let expected = normalize_lexical_surface(expected)
        .chars()
        .collect::<Vec<_>>();
    if observed.len() != expected.len() || observed == expected {
        return 0;
    }

    let mismatches = observed
        .iter()
        .zip(&expected)
        .enumerate()
        .filter_map(|(index, (observed, expected))| (observed != expected).then_some(index))
        .collect::<Vec<_>>();
    match mismatches.as_slice() {
        [index]
            if crate::nanda_wave::surface_damage::alphabet_successor(expected[*index])
                == Some(observed[*index]) =>
        {
            RECONSTRUCTION_MODE_SINGLE_SUBSTITUTION
        }
        [first, second] => {
            let mut modes = 0;
            if crate::nanda_wave::surface_damage::alphabet_successor(expected[*first])
                == Some(observed[*first])
                && crate::nanda_wave::surface_damage::alphabet_successor(expected[*second])
                    == Some(observed[*second])
            {
                modes |= RECONSTRUCTION_MODE_DOUBLE_SUBSTITUTION;
            }
            if expected[*first] == observed[*second] && expected[*second] == observed[*first] {
                modes |= RECONSTRUCTION_MODE_NON_ADJACENT_TRANSPOSITION;
            }
            modes
        }
        _ => 0,
    }
}

fn is_ordered_subsequence(needle: &[u32], haystack: &[u32]) -> bool {
    let mut next = 0;
    for symbol in haystack {
        if needle.get(next) == Some(symbol) {
            next += 1;
        }
    }
    next == needle.len()
}

fn is_subsequence_after_one_adjacent_swap(observed: &[u32], expected: &[u32]) -> bool {
    if observed.len() < 2
        || expected.len() != observed.len().saturating_add(1)
        || observed.len() > MAX_ANCHOR_SEQUENCE
        || expected.len() > MAX_ANCHOR_SEQUENCE
    {
        return false;
    }

    fn visit(
        observed: &[u32],
        expected: &[u32],
        observed_index: usize,
        expected_index: usize,
        skipped: bool,
        swapped: bool,
    ) -> bool {
        if observed_index == observed.len() && expected_index == expected.len() {
            skipped && swapped
        } else {
            (!skipped
                && expected_index < expected.len()
                && visit(
                    observed,
                    expected,
                    observed_index,
                    expected_index + 1,
                    true,
                    swapped,
                ))
                || (observed_index < observed.len()
                    && expected_index < expected.len()
                    && observed[observed_index] == expected[expected_index]
                    && visit(
                        observed,
                        expected,
                        observed_index + 1,
                        expected_index + 1,
                        skipped,
                        swapped,
                    ))
                || (!swapped
                    && observed_index + 1 < observed.len()
                    && expected_index + 1 < expected.len()
                    && observed[observed_index] == expected[expected_index + 1]
                    && observed[observed_index + 1] == expected[expected_index]
                    && visit(
                        observed,
                        expected,
                        observed_index + 2,
                        expected_index + 2,
                        skipped,
                        true,
                    ))
                || (!skipped
                    && !swapped
                    && observed_index + 1 < observed.len()
                    && expected_index + 2 < expected.len()
                    && observed[observed_index] == expected[expected_index + 2]
                    && observed[observed_index + 1] == expected[expected_index]
                    && visit(
                        observed,
                        expected,
                        observed_index + 2,
                        expected_index + 3,
                        true,
                        true,
                    ))
        }
    }

    visit(observed, expected, 0, 0, false, false)
}

pub(super) fn apply_position_certificate_interference(candidates: &mut [GrokkingCandidate]) {
    const MIN_COHERENCE: u16 = 600;
    const MIN_MARGIN: u16 = 100;
    const EQUAL_LENGTH_ENERGY_LEASE: i32 = 250;
    const CROSS_LENGTH_ENERGY_LEASE: i32 = 850;
    const STRONG_SEQUENCE_COHERENCE: u16 = 800;

    let Some(incumbent) = candidates.first().copied() else {
        return;
    };
    if incumbent.exact_reconstruction
        || (incumbent.length_relation != 0 && incumbent.sequence_milli == 1_000)
    {
        return;
    }
    let mut evidence = candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| candidate.position_milli >= MIN_COHERENCE)
        .map(|(index, candidate)| (index, candidate.position_milli, candidate.terminal_id))
        .collect::<Vec<_>>();
    evidence
        .sort_unstable_by(|left, right| right.1.cmp(&left.1).then_with(|| left.2.cmp(&right.2)));
    let Some((winner, coherence, _)) = evidence.first().copied() else {
        return;
    };
    let runner_up = evidence.get(1).map(|item| item.1).unwrap_or_default();
    if coherence.saturating_sub(runner_up) < MIN_MARGIN || winner == 0 {
        return;
    }
    let winner_candidate = candidates[winner];
    if winner_candidate.geometry_distance > incumbent.geometry_distance {
        return;
    }
    let energy_deficit = incumbent
        .settled_energy
        .saturating_sub(winner_candidate.settled_energy);
    if incumbent.length_relation == 0 {
        if winner_candidate.sequence_milli < incumbent.sequence_milli
            || (winner_candidate.sequence_milli == incumbent.sequence_milli
                && energy_deficit > EQUAL_LENGTH_ENERGY_LEASE)
        {
            return;
        }
    } else {
        if incumbent.sequence_milli > STRONG_SEQUENCE_COHERENCE
            && winner_candidate.sequence_milli < incumbent.sequence_milli
        {
            return;
        }
        if energy_deficit > CROSS_LENGTH_ENERGY_LEASE {
            return;
        }
    }
    candidates[..=winner].rotate_right(1);
}

pub(super) fn apply_geometry_certificate_interference(candidates: &mut [GrokkingCandidate]) {
    const MAX_ENERGY_DEFICIT: i32 = 1_000;

    let Some(incumbent) = candidates.first().copied() else {
        return;
    };
    if incumbent.exact_reconstruction {
        return;
    }
    let mut evidence = candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| geometry_certificate_rank(candidate) != 0)
        .collect::<Vec<_>>();
    evidence.sort_unstable_by(|left, right| {
        geometry_certificate_rank(right.1)
            .cmp(&geometry_certificate_rank(left.1))
            .then_with(|| left.1.geometry_distance.cmp(&right.1.geometry_distance))
            .then_with(|| right.1.settled_energy.cmp(&left.1.settled_energy))
            .then_with(|| left.1.terminal_id.cmp(&right.1.terminal_id))
    });
    let Some((winner, candidate)) = evidence.first().copied() else {
        return;
    };
    if winner == 0 {
        return;
    }
    let candidate_rank = geometry_certificate_rank(candidate);
    let incumbent_rank = geometry_certificate_rank(&incumbent);
    if candidate_rank < incumbent_rank {
        return;
    }
    if candidate.geometry_distance > incumbent.geometry_distance
        && !geometry_certificate_can_cross_distance(candidate)
    {
        return;
    }
    if candidate_rank == incumbent_rank
        && candidate.geometry_distance >= incumbent.geometry_distance
    {
        return;
    }
    let energy_deficit = incumbent
        .settled_energy
        .saturating_sub(candidate.settled_energy);
    if candidate.reconstruction_modes & RECONSTRUCTION_MODE_DELETION != 0
        && incumbent.reconstruction_modes
            & (RECONSTRUCTION_MODE_SINGLE_DELETION
                | RECONSTRUCTION_MODE_PREFIX_TRUNCATION
                | RECONSTRUCTION_MODE_SUFFIX_TRUNCATION)
            != 0
        && incumbent.geometry_distance < candidate.geometry_distance
        && energy_deficit > 0
    {
        return;
    }
    if candidate.geometry_distance > incumbent.geometry_distance
        && energy_deficit > geometry_certificate_cross_distance_lease(candidate)
    {
        return;
    }
    if candidate_rank == incumbent_rank && energy_deficit > MAX_ENERGY_DEFICIT {
        return;
    }
    candidates[..=winner].rotate_right(1);
}

fn geometry_certificate_rank(candidate: &GrokkingCandidate) -> u8 {
    const DIRECT_SURFACE_MODES: u8 = RECONSTRUCTION_MODE_SINGLE_SUBSTITUTION
        | RECONSTRUCTION_MODE_DOUBLE_SUBSTITUTION
        | RECONSTRUCTION_MODE_NON_ADJACENT_TRANSPOSITION;
    if candidate.reconstruction_modes & DIRECT_SURFACE_MODES != 0 {
        7
    } else if candidate.reconstruction_modes & RECONSTRUCTION_MODE_DELETION_TRANSPOSITION != 0 {
        6
    } else if candidate.keyboard_hits > candidate.surface_hits {
        5
    } else if candidate.reconstruction_modes
        & (RECONSTRUCTION_MODE_PREFIX_TRUNCATION | RECONSTRUCTION_MODE_SUFFIX_TRUNCATION)
        != 0
    {
        3
    } else if candidate.reconstruction_modes
        & (RECONSTRUCTION_MODE_DELETION | RECONSTRUCTION_MODE_SINGLE_DELETION)
        != 0
    {
        2
    } else if candidate.reconstruction_modes != 0 {
        1
    } else {
        0
    }
}

fn geometry_certificate_cross_distance_lease(candidate: &GrokkingCandidate) -> i32 {
    if candidate.reconstruction_modes & RECONSTRUCTION_MODE_DELETION != 0 {
        4_000
    } else {
        1_500
    }
}

fn geometry_certificate_can_cross_distance(candidate: &GrokkingCandidate) -> bool {
    candidate.keyboard_hits > candidate.surface_hits
        || candidate.reconstruction_modes
            & (RECONSTRUCTION_MODE_DOUBLE_SUBSTITUTION
                | RECONSTRUCTION_MODE_NON_ADJACENT_TRANSPOSITION
                | RECONSTRUCTION_MODE_DELETION_TRANSPOSITION)
            != 0
        || (candidate.reconstruction_modes != 0 && candidate.sequence_milli == 1_000)
}

fn reconstruction_mode_rank(modes: u8) -> u8 {
    if modes & RECONSTRUCTION_MODE_DELETION_TRANSPOSITION != 0 {
        4
    } else if modes & RECONSTRUCTION_MODE_DELETION != 0 {
        3
    } else if modes
        & (RECONSTRUCTION_MODE_SINGLE_DELETION
            | RECONSTRUCTION_MODE_PREFIX_TRUNCATION
            | RECONSTRUCTION_MODE_SUFFIX_TRUNCATION)
        != 0
    {
        2
    } else {
        1
    }
}

fn truncate_with_reconstruction_tail(candidates: &mut Vec<GrokkingCandidate>, limit: usize) {
    if candidates.len() <= limit {
        return;
    }
    let mut reserve = candidates[limit..]
        .iter()
        .filter(|candidate| candidate.reconstruction_modes != 0 || candidate.ambiguity_shell)
        .copied()
        .collect::<Vec<_>>();
    reserve.sort_unstable_by(|left, right| {
        geometry_certificate_rank(right)
            .cmp(&geometry_certificate_rank(left))
            .then_with(|| left.geometry_distance.cmp(&right.geometry_distance))
            .then_with(|| right.settled_energy.cmp(&left.settled_energy))
            .then_with(|| left.terminal_id.cmp(&right.terminal_id))
    });
    reserve.truncate(MAX_RECONSTRUCTION_TAIL.min(limit));
    if reserve.is_empty() {
        candidates.truncate(limit);
        return;
    }
    let replaceable = candidates[..limit]
        .iter()
        .enumerate()
        .rev()
        .filter_map(|(index, candidate)| {
            (index != 0 && candidate.reconstruction_modes == 0 && !candidate.ambiguity_shell)
                .then_some(index)
        })
        .take(reserve.len())
        .collect::<BTreeSet<_>>();
    reserve.truncate(replaceable.len());
    let mut retained = candidates[..limit]
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| (!replaceable.contains(&index)).then_some(*candidate))
        .collect::<Vec<_>>();
    retained.extend(reserve);
    *candidates = retained;
}

fn should_expand_operator_lattice(exact_terminal_count: usize, limit: usize) -> bool {
    exact_terminal_count == 0 || limit > exact_terminal_count
}

fn compile_surface_indices(
    package: &LexicalGrokkingPackage,
) -> (Vec<(u64, u32)>, Vec<u32>, Vec<u32>) {
    let mut index = Vec::with_capacity(package.centers.len());
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
                anchors.atoms[position] = atom_id;
                anchors.len = anchors.len.saturating_add(1);
            }
        }
        if complete && !anchors.as_slice().is_empty() {
            index.push((anchor_sequence_hash(anchors.as_slice()), terminal as u32));
            atoms.extend_from_slice(anchors.as_slice());
        }
        offsets.push(atoms.len() as u32);
    }
    index.sort_unstable();
    (index, offsets, atoms)
}

fn anchor_sequence_hash(sequence: &[u32]) -> u64 {
    let mut state = mix64_golden(0x4c31_4558_4143_5431 ^ sequence.len() as u64);
    for atom in sequence {
        state = mix64_golden(state ^ u64::from(*atom));
    }
    state
}

pub(super) fn ambiguity_geometry_link(
    owner_distance: u8,
    competitor_distance: u8,
    max_geometry_distance: u8,
) -> bool {
    competitor_distance <= max_geometry_distance
        && owner_distance.abs_diff(competitor_distance) <= 1
}

fn apply_settlement_mode(candidate: &mut GrokkingCandidate, mode: ReadoutMode) {
    if mode == ReadoutMode::WithoutPhase {
        candidate.positive_milli = 500;
    }
    candidate.sequence_milli = match mode {
        ReadoutMode::WithoutSequence => 750,
        ReadoutMode::LegacySequence => candidate.legacy_sequence_milli,
        _ => candidate.sequence_milli,
    };
    let energy = base_settled_energy(
        candidate.forward_milli,
        candidate.backward_milli,
        candidate.positive_milli,
        candidate.length_milli,
    );
    candidate.legacy_settled_energy = with_sequence_energy(energy, candidate.legacy_sequence_milli);
    candidate.settled_energy = with_sequence_energy(energy, candidate.sequence_milli);
}

fn base_settled_energy(
    forward_milli: u16,
    backward_milli: u16,
    positive_milli: u16,
    length_milli: u16,
) -> i32 {
    let mut energy = i32::from(forward_milli) * 3;
    for _ in 0..SETTLING_ITERATIONS {
        let constructive =
            i32::from(backward_milli) * 3 + (i32::from(positive_milli) - 500).max(0) * 2;
        let destructive =
            (500 - i32::from(positive_milli)).max(0) * 2 + (1_000 - i32::from(length_milli)) * 2;
        energy = (energy + i32::from(forward_milli) * 3 + constructive + i32::from(length_milli)
            - destructive)
            / 2;
    }
    energy
}

fn with_sequence_energy(energy: i32, sequence_milli: u16) -> i32 {
    energy.saturating_add((i32::from(sequence_milli) - 750).saturating_mul(3))
}

fn apply_structural_interference(candidates: &mut [GrokkingCandidate]) {
    let max_surface_hits = candidates
        .iter()
        .map(|candidate| candidate.surface_hits)
        .max()
        .unwrap_or(1)
        .max(1);
    let max_keyboard_hits = candidates
        .iter()
        .map(|candidate| candidate.keyboard_hits)
        .max()
        .unwrap_or(1)
        .max(1);
    for candidate in candidates {
        let surface =
            u32::from(candidate.surface_hits).saturating_mul(1_000) / u32::from(max_surface_hits);
        let keyboard =
            u32::from(candidate.keyboard_hits).saturating_mul(1_000) / u32::from(max_keyboard_hits);
        let coherence = surface.max(keyboard);
        candidate.structural_milli = coherence as u16;
        // Independent atom links provide a lattice-relative constructive wave;
        // this prevents a few heavy generic links from owning the whole basin.
        candidate.settled_energy = candidate
            .settled_energy
            .saturating_add((coherence as i32 - 500).saturating_mul(3));
        candidate.legacy_settled_energy = candidate
            .legacy_settled_energy
            .saturating_add((coherence as i32 - 500).saturating_mul(3));
    }
}

pub(super) fn apply_sequence_certificate_interference(
    candidates: &mut [GrokkingCandidate],
    mode: ReadoutMode,
) {
    if matches!(
        mode,
        ReadoutMode::WithoutSequence
            | ReadoutMode::WithoutSequenceCertificate
            | ReadoutMode::LegacySequence
    ) {
        return;
    }
    let certificate_owner = candidates.iter().max_by(|left, right| {
        left.legacy_settled_energy
            .cmp(&right.legacy_settled_energy)
            .then_with(|| left.backward_milli.cmp(&right.backward_milli))
            .then_with(|| left.positive_milli.cmp(&right.positive_milli))
            .then_with(|| left.forward_milli.cmp(&right.forward_milli))
            .then_with(|| right.terminal_id.cmp(&left.terminal_id))
    });
    if !certificate_owner.is_some_and(|candidate| candidate.legacy_sequence_milli == 1_000) {
        return;
    }
    for candidate in candidates {
        if candidate.legacy_sequence_milli < 1_000 {
            candidate.sequence_milli = candidate.legacy_sequence_milli;
            candidate.settled_energy = candidate.legacy_settled_energy;
        }
    }
}

fn length_relation(observed: u8, expected: u8) -> i8 {
    match expected.cmp(&observed) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
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

fn is_anchor_channel(channel: AtomChannel) -> bool {
    channel == AtomChannel::CharacterAnchor
}

fn observed_sequence(observed: &[(u32, ObservedAtom)], channel: AtomChannel) -> AnchorSequence {
    ordered_anchor_sequence(
        observed
            .iter()
            .filter(|(_, atom)| atom.channel == channel)
            .map(|(atom_id, atom)| (atom.position, *atom_id)),
    )
}

fn reconstruction_sequence_milli(
    reverse: &[WaveCoupling],
    character_sequence: &AnchorSequence,
) -> u16 {
    let character = expected_sequence(reverse, COUPLING_FLAG_CHARACTER_ANCHOR);
    sequence_coherence_milli(character_sequence.as_slice(), character.as_slice())
}

fn legacy_reconstruction_sequence_milli(
    reverse: &[WaveCoupling],
    character_sequence: &AnchorSequence,
) -> u16 {
    let character = expected_sequence(reverse, COUPLING_FLAG_CHARACTER_ANCHOR);
    legacy_sequence_coherence_milli(character_sequence.as_slice(), character.as_slice())
}

fn expected_sequence(reverse: &[WaveCoupling], flag: u8) -> AnchorSequence {
    ordered_anchor_sequence(
        reverse
            .iter()
            .filter(|coupling| coupling.flags == flag)
            .map(|coupling| (coupling.position_mode, coupling.peer_id)),
    )
}

fn ordered_anchor_sequence(items: impl IntoIterator<Item = (u8, u32)>) -> AnchorSequence {
    let mut ordered = [(0_u8, 0_u32); MAX_ANCHOR_SEQUENCE];
    let mut len = 0;
    for item in items.into_iter().take(MAX_ANCHOR_SEQUENCE) {
        ordered[len] = item;
        len += 1;
    }
    ordered[..len].sort_unstable();
    let mut sequence = AnchorSequence {
        len: len as u8,
        ..AnchorSequence::default()
    };
    for (target, (_, atom_id)) in sequence.atoms.iter_mut().zip(ordered[..len].iter()) {
        *target = *atom_id;
    }
    sequence
}

pub(super) fn sequence_coherence_milli(observed: &[u32], expected: &[u32]) -> u16 {
    if observed.is_empty() || expected.is_empty() {
        return 750;
    }
    let common_order = longest_common_subsequence(observed, expected);
    let common_mass = multiset_intersection(observed, expected);
    let shorter = observed.len().min(expected.len()).max(1);
    let longer = observed.len().max(expected.len()).max(1);
    let order_milli = common_order.saturating_mul(1_000) / shorter;
    let mass_milli = common_mass.saturating_mul(1_000) / shorter;
    let length_milli = shorter.saturating_mul(1_000) / longer;
    if common_order == shorter && common_mass == shorter {
        1_000
    } else {
        (((order_milli * 4 + mass_milli * 4 + length_milli * 2) / 10) as u16).max(750)
    }
}

fn exact_position_coherence_milli(observed: &[u32], expected: &[u32]) -> u16 {
    if observed.is_empty() || observed.len() != expected.len() {
        return 0;
    }
    let matches = observed
        .iter()
        .zip(expected)
        .filter(|(left, right)| left == right)
        .count();
    (matches.saturating_mul(1_000) / observed.len()) as u16
}

pub(super) fn legacy_sequence_coherence_milli(observed: &[u32], expected: &[u32]) -> u16 {
    if observed.is_empty() || observed.len() >= expected.len() {
        return 750;
    }
    if longest_common_subsequence(observed, expected) == observed.len() {
        1_000
    } else {
        750
    }
}

fn longest_common_subsequence(left: &[u32], right: &[u32]) -> usize {
    let mut previous = [0_u8; MAX_ANCHOR_SEQUENCE + 1];
    let mut current = [0_u8; MAX_ANCHOR_SEQUENCE + 1];
    for left_atom in left.iter().take(MAX_ANCHOR_SEQUENCE) {
        current[0] = 0;
        for (right_index, right_atom) in right.iter().take(MAX_ANCHOR_SEQUENCE).enumerate() {
            current[right_index + 1] = if left_atom == right_atom {
                previous[right_index].saturating_add(1)
            } else {
                current[right_index].max(previous[right_index + 1])
            };
        }
        std::mem::swap(&mut previous, &mut current);
    }
    usize::from(previous[right.len().min(MAX_ANCHOR_SEQUENCE)])
}

fn multiset_intersection(left: &[u32], right: &[u32]) -> usize {
    let mut consumed = [false; MAX_ANCHOR_SEQUENCE];
    let mut common = 0_usize;
    for left_atom in left.iter().take(MAX_ANCHOR_SEQUENCE) {
        if let Some(index) = right
            .iter()
            .take(MAX_ANCHOR_SEQUENCE)
            .enumerate()
            .position(|(index, right_atom)| !consumed[index] && left_atom == right_atom)
        {
            consumed[index] = true;
            common += 1;
        }
    }
    common
}

fn position_coherence(observed: u8, expected: u8) -> u16 {
    256_u16.saturating_sub(u16::from(observed.abs_diff(expected)))
}

pub(super) fn candidate_order(
    left: &GrokkingCandidate,
    right: &GrokkingCandidate,
) -> std::cmp::Ordering {
    right
        .exact_reconstruction
        .cmp(&left.exact_reconstruction)
        .then_with(|| right.settled_energy.cmp(&left.settled_energy))
        .then_with(|| right.backward_milli.cmp(&left.backward_milli))
        .then_with(|| right.positive_milli.cmp(&left.positive_milli))
        .then_with(|| right.forward_milli.cmp(&left.forward_milli))
        .then_with(|| left.terminal_id.cmp(&right.terminal_id))
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
            exact_surface_index: Vec::new(),
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
