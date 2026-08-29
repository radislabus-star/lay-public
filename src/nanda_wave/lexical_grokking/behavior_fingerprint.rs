//! Offline semantic fingerprint for move-only L1.1 refactors.
//!
//! This module observes the production reader but owns no runtime authority.

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::path::Path;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::runtime::{
    benchmark_diverse_restoration, candidate_json, candidate_order, GrokkingCandidate,
    L1RestorationHost, LexicalGrokkingMemory, ReadoutMode,
};

struct RouteDigest {
    rows: usize,
    digest: Sha256,
}

impl RouteDigest {
    fn new() -> Self {
        Self {
            rows: 0,
            digest: Sha256::new(),
        }
    }

    fn update(&mut self, value: &impl serde::Serialize) -> io::Result<()> {
        let bytes = serde_json::to_vec(value).map_err(io::Error::other)?;
        self.digest.update((bytes.len() as u64).to_le_bytes());
        self.digest.update(bytes);
        self.rows = self.rows.saturating_add(1);
        Ok(())
    }

    fn finish(self) -> Value {
        json!({
            "rows": self.rows,
            "sha256": format!("{:x}", self.digest.finalize()),
            "framing": "u64_le_byte_length_then_canonical_compact_json",
        })
    }
}

const LATENCY_CLASSES: [&str; 13] = [
    "adjacent_transposition",
    "double_substitution",
    "extra_letter",
    "layout_projection",
    "letter_substitution",
    "missing_letter",
    "non_adjacent_transposition",
    "omission_transposition",
    "prefix_truncation",
    "punctuation_suffix",
    "repeated_fragment",
    "sparse_multi_omission",
    "suffix_truncation",
];

const MODES: [(ReadoutMode, &str); 8] = [
    (ReadoutMode::Full, "Full"),
    (ReadoutMode::WithoutAnti, "WithoutAnti"),
    (ReadoutMode::WithoutPhase, "WithoutPhase"),
    (ReadoutMode::WithoutSequence, "WithoutSequence"),
    (
        ReadoutMode::WithoutSequenceCertificate,
        "WithoutSequenceCertificate",
    ),
    (ReadoutMode::LegacySequence, "LegacySequence"),
    (ReadoutMode::WithoutPairwise, "WithoutPairwise"),
    (ReadoutMode::WithoutPosition, "WithoutPosition"),
];

#[derive(Clone)]
struct ReplayCase {
    origin: &'static str,
    class: Option<&'static str>,
    surface: String,
    source_index: usize,
}

#[expect(clippy::too_many_arguments, reason = "proof inputs remain explicit")]
pub fn fingerprint_l1_behavior(
    package_path: &Path,
    surfaces_path: &Path,
    corpus_path: &Path,
    heldout_path: &Path,
    output_path: &Path,
    limit: usize,
    clean_per_language: usize,
    collision_samples: usize,
) -> io::Result<Value> {
    if limit == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "L1.1 behavior fingerprint limit must be positive",
        ));
    }
    let cases = replay_cases(
        surfaces_path,
        corpus_path,
        heldout_path,
        clean_per_language,
        collision_samples,
    )?;
    let memory = LexicalGrokkingMemory::load(package_path).map_err(io::Error::other)?;
    let host = L1RestorationHost::load(package_path)?;
    let mut route_digests = [
        "readout_modes",
        "query",
        "restore",
        "host_lattice",
        "service_lattice",
        "restoration_permutation",
    ]
    .into_iter()
    .map(|route| (route, RouteDigest::new()))
    .collect::<BTreeMap<_, _>>();
    let mut permutation_failures = Vec::new();
    let mut observed_verdicts = BTreeMap::<String, usize>::new();
    let mode_values = MODES.iter().map(|(mode, _)| *mode).collect::<Vec<_>>();

    for (replay_index, case) in cases.iter().enumerate() {
        let readouts = memory.readout_modes(&case.surface, limit, &mode_values);
        let raw_modes = MODES
            .iter()
            .zip(&readouts)
            .map(|((_, name), candidates)| {
                (
                    (*name).to_string(),
                    Value::Array(
                        candidates
                            .iter()
                            .copied()
                            .map(|candidate| full_candidate_json(&memory, candidate))
                            .collect(),
                    ),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        route_digests
            .get_mut("readout_modes")
            .unwrap()
            .update(&json!({
                "replay_index": replay_index,
                "origin": case.origin,
                "class": case.class,
                "source_index": case.source_index,
                "surface": case.surface,
                "modes": raw_modes,
            }))?;

        let full = &readouts[0];
        route_digests.get_mut("query").unwrap().update(&json!({
            "package": package_path,
            "surface": case.surface,
            "terminal_count": memory.package.terminal_count(),
            "candidates": full
                .iter()
                .copied()
                .map(|candidate| candidate_json(&memory, candidate))
                .collect::<Vec<_>>(),
        }))?;

        let restore = host.restore(&case.surface, limit);
        if let Some(verdict) = restore.pointer("/result/verdict").and_then(Value::as_str) {
            *observed_verdicts.entry(verdict.to_string()).or_default() += 1;
        }
        route_digests.get_mut("restore").unwrap().update(&restore)?;
        route_digests
            .get_mut("host_lattice")
            .unwrap()
            .update(&host.lattice(&case.surface, limit))?;
        route_digests
            .get_mut("service_lattice")
            .unwrap()
            .update(&json!({
                "type": "lattice",
                "seeds": host
                    .typed_lattice_seed_rows(&case.surface, limit)
                    .into_iter()
                    .map(|(terminal_id, surface, authority, score_milli)| json!({
                        "terminal_id": terminal_id,
                        "surface": surface,
                        "authority": authority,
                        "score_milli": score_milli,
                    }))
                    .collect::<Vec<_>>(),
            }))?;

        let forward = restoration_after_phase(&memory, &case.surface, full.clone());
        let mut reversed = full.clone();
        reversed.reverse();
        reversed.sort_unstable_by(candidate_order);
        let reverse = restoration_after_phase(&memory, &case.surface, reversed);
        if forward != reverse {
            permutation_failures.push(json!({
                "replay_index": replay_index,
                "surface": case.surface,
                "forward": forward,
                "reverse": reverse,
            }));
        }
        route_digests
            .get_mut("restoration_permutation")
            .unwrap()
            .update(&json!({
                "replay_index": replay_index,
                "surface": case.surface,
                "readout": forward,
            }))?;
    }

    let stats = serde_json::to_value(host.stats()).map_err(io::Error::other)?;
    let mut benchmark = benchmark_diverse_restoration(package_path, surfaces_path, limit)?;
    strip_dynamic_benchmark_fields(&mut benchmark);
    let route_fingerprints = route_digests
        .into_iter()
        .map(|(route, digest)| (route, digest.finish()))
        .collect::<BTreeMap<_, _>>();
    let projection = json!({
        "schema": "lay.l11.behavior-fingerprint.v1",
        "contract": {
            "limit": limit,
            "modes": MODES.iter().map(|(_, name)| *name).collect::<Vec<_>>(),
            "dynamic_fields_excluded": [
                "all elapsed, percentile, and maximum timing fields",
                "background warmup duration",
                "process id, uptime, request counters, RSS, PSS, and swap",
            ],
            "candidate_order_is_semantic": true,
            "runtime_authority_changed": false,
        },
        "inputs": {
            "package": file_identity(package_path)?,
            "surfaces": file_identity(surfaces_path)?,
            "corpus": file_identity(corpus_path)?,
            "heldout": file_identity(heldout_path)?,
        },
        "selection": {
            "replay_cases": cases.len(),
            "damaged_cases": cases.iter().filter(|case| case.origin == "fixed_latency").count(),
            "clean_ru_cases": cases.iter().filter(|case| case.origin == "clean_ru").count(),
            "clean_en_cases": cases.iter().filter(|case| case.origin == "clean_en").count(),
            "heldout_surface_collision_cases": cases
                .iter()
                .filter(|case| case.origin == "heldout_surface_collision")
                .count(),
            "observed_restore_verdicts": observed_verdicts,
        },
        "host_stats": stats,
        "benchmark_stable_projection": benchmark,
        "route_fingerprints": route_fingerprints,
        "permutation": {
            "cases": cases.len(),
            "failures": permutation_failures,
        },
    });
    let semantic_sha256 = json_sha256(&projection)?;
    let output_bytes = write_projection(output_path, &projection)?;
    let output_file_sha256 = sha256_file(output_path)?;
    Ok(json!({
        "schema": "lay.l11.behavior-fingerprint-receipt.v1",
        "verdict": if permutation_failures.is_empty() {
            "PASS_BEHAVIOR_FROZEN"
        } else {
            "FAIL_CANDIDATE_PERMUTATION"
        },
        "semantic_sha256": semantic_sha256,
        "route_fingerprints": projection["route_fingerprints"],
        "output": output_path,
        "output_bytes": output_bytes,
        "output_file_sha256": output_file_sha256,
        "replay_cases": cases.len(),
        "permutation_failures": permutation_failures.len(),
        "runtime_authority_changed": false,
    }))
}

fn replay_cases(
    surfaces_path: &Path,
    corpus_path: &Path,
    heldout_path: &Path,
    clean_per_language: usize,
    collision_samples: usize,
) -> io::Result<Vec<ReplayCase>> {
    let damaged = read_nonempty_lines(surfaces_path)?;
    let mut cases = damaged
        .into_iter()
        .enumerate()
        .map(|(index, surface)| ReplayCase {
            origin: "fixed_latency",
            class: Some(LATENCY_CLASSES[index % LATENCY_CLASSES.len()]),
            surface,
            source_index: index,
        })
        .collect::<Vec<_>>();

    let corpus = read_nonempty_lines(corpus_path)?;
    let ru = corpus
        .iter()
        .enumerate()
        .filter(|(_, surface)| !surface.is_ascii())
        .map(|(index, surface)| (index, surface.clone()))
        .collect::<Vec<_>>();
    let en = corpus
        .iter()
        .enumerate()
        .filter(|(_, surface)| {
            surface
                .chars()
                .any(|character| character.is_ascii_alphabetic())
        })
        .filter(|(_, surface)| surface.is_ascii())
        .map(|(index, surface)| (index, surface.clone()))
        .collect::<Vec<_>>();
    cases.extend(select_evenly(&ru, clean_per_language).into_iter().map(
        |(source_index, surface)| ReplayCase {
            origin: "clean_ru",
            class: None,
            surface,
            source_index,
        },
    ));
    cases.extend(select_evenly(&en, clean_per_language).into_iter().map(
        |(source_index, surface)| ReplayCase {
            origin: "clean_en",
            class: None,
            surface,
            source_index,
        },
    ));

    let heldout = read_nonempty_lines(heldout_path)?;
    let mut counts = HashMap::<String, usize>::new();
    for surface in &heldout {
        *counts.entry(surface.clone()).or_default() += 1;
    }
    let collisions = heldout
        .into_iter()
        .enumerate()
        .filter(|(_, surface)| counts.get(surface).copied().unwrap_or_default() > 1)
        .collect::<Vec<_>>();
    cases.extend(
        select_evenly(&collisions, collision_samples)
            .into_iter()
            .map(|(source_index, surface)| ReplayCase {
                origin: "heldout_surface_collision",
                class: None,
                surface,
                source_index,
            }),
    );
    Ok(cases)
}

fn select_evenly(rows: &[(usize, String)], count: usize) -> Vec<(usize, String)> {
    let count = count.min(rows.len());
    if count == 0 {
        return Vec::new();
    }
    (0..count)
        .map(|sample| {
            let index = sample.saturating_mul(rows.len()).saturating_add(count / 2) / count;
            rows[index.min(rows.len() - 1)].clone()
        })
        .collect()
}

fn restoration_after_phase(
    memory: &LexicalGrokkingMemory,
    surface: &str,
    mut candidates: Vec<GrokkingCandidate>,
) -> Value {
    serde_json::to_value(memory.classify_restoration(
        surface,
        &mut candidates,
        memory.package.restoration_calibration,
    ))
    .expect("restoration readout must serialize")
}

fn full_candidate_json(memory: &LexicalGrokkingMemory, candidate: GrokkingCandidate) -> Value {
    json!({
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
        "settling_iterations": candidate.settling_iterations,
        "exact_reconstruction": candidate.exact_reconstruction,
    })
}

fn strip_dynamic_benchmark_fields(value: &mut Value) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    object.retain(|key, _| {
        !matches!(
            key.as_str(),
            "raw_readout_p50_us"
                | "raw_readout_p90_us"
                | "raw_readout_p99_us"
                | "raw_readout_max_us"
                | "background_warmup_ms"
                | "readout_p50_us"
                | "readout_p90_us"
                | "readout_p99_us"
                | "readout_max_us"
                | "p50_us"
                | "p90_us"
                | "p99_us"
                | "max_us"
        )
    });
}

fn read_nonempty_lines(path: &Path) -> io::Result<Vec<String>> {
    Ok(std::fs::read_to_string(path)?
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

fn file_identity(path: &Path) -> io::Result<Value> {
    Ok(json!({
        "path": path,
        "bytes": std::fs::metadata(path)?.len(),
        "sha256": sha256_file(path)?,
    }))
}

fn json_sha256(value: &impl serde::Serialize) -> io::Result<String> {
    let bytes = serde_json::to_vec(value).map_err(io::Error::other)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn sha256_file(path: &Path) -> io::Result<String> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn write_projection(path: &Path, projection: &Value) -> io::Result<usize> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("tmp");
    let mut bytes = serde_json::to_vec_pretty(projection).map_err(io::Error::other)?;
    bytes.push(b'\n');
    let mut file = File::create(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    std::fs::rename(&temporary, path)?;
    Ok(bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_benchmark_fields_do_not_enter_semantic_projection() {
        let mut value = json!({
            "candidate_sha256": "stable",
            "p99_us": 17,
            "background_warmup_ms": 31,
            "warmup": {"posting_cache_entries": 4},
        });
        strip_dynamic_benchmark_fields(&mut value);
        assert_eq!(
            value,
            json!({
                "candidate_sha256": "stable",
                "warmup": {"posting_cache_entries": 4},
            })
        );
    }

    #[test]
    fn even_selection_is_deterministic_and_bounded() {
        let rows = (0..10)
            .map(|index| (index, index.to_string()))
            .collect::<Vec<_>>();
        assert_eq!(select_evenly(&rows, 3), select_evenly(&rows, 3));
        assert_eq!(select_evenly(&rows, 20).len(), rows.len());
        assert!(select_evenly(&rows, 0).is_empty());
    }
}
