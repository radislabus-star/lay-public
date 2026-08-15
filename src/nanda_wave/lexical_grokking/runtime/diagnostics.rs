use std::io;
use std::path::Path;
use std::time::Instant;

use sha2::{Digest, Sha256};

use super::super::v8::{self, V8Artifact};
use super::super::v9;
use super::{GrokkingCandidate, L1RestorationHost, LexicalGrokkingMemory, ReadoutMode};

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
    if v9::is_v9(&header) {
        let loaded = v9::load(package_path).map_err(io::Error::other)?;
        return Ok(serde_json::json!({
            "format": "V9",
            "corpus_fingerprint": loaded.package.corpus_hash,
            "terminal_count": loaded.package.terminal_count(),
            "atom_count": loaded.package.atoms.len(),
            "package_bytes": loaded.header.file_bytes,
            "compact_base_bytes": loaded.header.base_bytes,
            "exact_support_overflow_atoms": loaded.header.overflow_count,
            "maximum_exact_support": loaded.support.metrics.maximum_exact_support,
            "checksum": loaded.header.checksum,
            "forward_relations": 0,
            "reverse_relations": 0,
        }));
    }
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
        super::super::format::inspect_header(&header).map_err(io::Error::other)?;
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

pub(super) fn restoration_candidate_json(
    memory: &LexicalGrokkingMemory,
    candidate: super::super::restoration::RestorationCandidate,
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
    let birth_profile = memory.birth_profile(&surfaces);
    for surface in surfaces.iter().take(32) {
        std::hint::black_box(memory.readout(surface, limit, ReadoutMode::Full));
    }
    let mut raw_readout_elapsed_us = surfaces
        .iter()
        .map(|surface| {
            let started = Instant::now();
            std::hint::black_box(memory.readout(surface, limit, ReadoutMode::Full));
            started.elapsed().as_micros() as u64
        })
        .collect::<Vec<_>>();
    raw_readout_elapsed_us.sort_unstable();
    let warmup_started = Instant::now();
    let warmup = memory.warm_first_touch()?;
    let background_warmup_ms = warmup_started.elapsed().as_millis() as u64;
    let mut readout_elapsed_us = surfaces
        .iter()
        .map(|surface| {
            let started = Instant::now();
            std::hint::black_box(memory.readout(surface, limit, ReadoutMode::Full));
            started.elapsed().as_micros() as u64
        })
        .collect::<Vec<_>>();
    readout_elapsed_us.sort_unstable();
    let mut elapsed_us = surfaces
        .iter()
        .map(|surface| {
            let started = Instant::now();
            std::hint::black_box(memory.restoration_readout(surface, limit));
            started.elapsed().as_micros() as u64
        })
        .collect::<Vec<_>>();
    elapsed_us.sort_unstable();
    let candidate_sha256 = candidate_fingerprint(&memory, &surfaces, limit)?;
    Ok(serde_json::json!({
        "package": package_path,
        "surfaces": surfaces_path,
        "sample_count": surfaces.len(),
        "limit": limit,
        "raw_readout_p50_us": percentile(&raw_readout_elapsed_us, 50),
        "raw_readout_p90_us": percentile(&raw_readout_elapsed_us, 90),
        "raw_readout_p99_us": percentile(&raw_readout_elapsed_us, 99),
        "raw_readout_max_us": raw_readout_elapsed_us.last().copied().unwrap_or_default(),
        "background_warmup_ms": background_warmup_ms,
        "warmup": warmup,
        "birth_profile": birth_profile,
        "readout_p50_us": percentile(&readout_elapsed_us, 50),
        "readout_p90_us": percentile(&readout_elapsed_us, 90),
        "readout_p99_us": percentile(&readout_elapsed_us, 99),
        "readout_max_us": readout_elapsed_us.last().copied().unwrap_or_default(),
        "p50_us": percentile(&elapsed_us, 50),
        "p90_us": percentile(&elapsed_us, 90),
        "p99_us": percentile(&elapsed_us, 99),
        "max_us": elapsed_us.last().copied().unwrap_or_default(),
        "candidate_sha256": candidate_sha256,
    }))
}

fn candidate_fingerprint(
    memory: &LexicalGrokkingMemory,
    surfaces: &[String],
    limit: usize,
) -> io::Result<String> {
    let mut digest = Sha256::new();
    for surface in surfaces {
        digest.update((surface.len() as u64).to_le_bytes());
        digest.update(surface.as_bytes());
        let candidates = memory.readout(surface, limit, ReadoutMode::Full);
        let bytes = serde_json::to_vec(
            &candidates
                .into_iter()
                .map(|candidate| candidate_json(memory, candidate))
                .collect::<Vec<_>>(),
        )
        .map_err(io::Error::other)?;
        digest.update((bytes.len() as u64).to_le_bytes());
        digest.update(bytes);
    }
    Ok(format!("{:x}", digest.finalize()))
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

pub(in crate::nanda_wave::lexical_grokking) fn candidate_json(
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

pub(super) fn percent_usize(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 * 100.0 / denominator as f64
    }
}
