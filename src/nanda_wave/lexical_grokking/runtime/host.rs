use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::super::atoms::normalize_lexical_surface;
use super::super::{composite, restoration, v8};
use super::diagnostics::{candidate_json, restoration_candidate_json};
use super::{GrokkingCandidate, LexicalGrokkingMemory, ReadoutMode};

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
    pub(super) memory: LexicalGrokkingMemory,
    pub(super) overlays: Vec<L1OverlayMemory>,
    pub(super) tombstones: BTreeSet<String>,
    manifest_generation: u64,
}

pub(super) struct L1OverlayMemory {
    terminal_offset: u32,
    memory: LexicalGrokkingMemory,
}

impl L1RestorationHost {
    pub fn load(package_path: &Path) -> io::Result<Self> {
        let Some(spec) = composite::load_spec(package_path)? else {
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

    pub fn warm_first_touch(&self) -> io::Result<serde_json::Value> {
        let mut packages = Vec::with_capacity(self.overlays.len().saturating_add(1));
        packages.push(self.memory.warm_first_touch()?);
        for overlay in &self.overlays {
            packages.push(overlay.memory.warm_first_touch()?);
        }
        Ok(serde_json::json!({
            "package_count": packages.len(),
            "packages": packages,
        }))
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
        let (_candidates, readout) = self.memory.restoration_readout(surface, limit.max(1));
        let result = match readout {
            restoration::RestorationReadout::Winner { candidate } => {
                serde_json::json!({
                    "verdict": "winner",
                    "authority": true,
                    "candidate": restoration_candidate_json(&self.memory, candidate),
                })
            }
            restoration::RestorationReadout::Tied {
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
            restoration::RestorationReadout::TiedOverflow {
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
            restoration::RestorationReadout::Abstain {
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
        self.lattice_seed_rows_with_parallel_packages(surface, limit, true)
    }

    pub fn typed_lattice_seed_rows(
        &self,
        surface: &str,
        limit: usize,
    ) -> Vec<(u32, String, bool, u32)> {
        if self.is_composite() {
            return self
                .lattice_seed_rows(surface, limit)
                .into_iter()
                .map(|(terminal_id, surface, score_milli)| {
                    (terminal_id, surface, false, score_milli)
                })
                .collect();
        }

        let limit = limit.max(1);
        let (candidates, readout) = self.memory.restoration_readout(surface, limit);
        let authority_terminal = match readout {
            restoration::RestorationReadout::Winner { candidate } => Some(candidate.terminal_id),
            restoration::RestorationReadout::Tied { .. }
            | restoration::RestorationReadout::TiedOverflow { .. }
            | restoration::RestorationReadout::Abstain { .. } => None,
        };
        candidates
            .into_iter()
            .filter_map(|candidate| {
                let terminal_id = candidate.terminal_id;
                Some((
                    terminal_id,
                    self.memory.decode_terminal(terminal_id)?,
                    authority_terminal == Some(terminal_id),
                    lattice_seed_score(candidate),
                ))
            })
            .take(limit)
            .collect()
    }

    pub(in crate::nanda_wave::lexical_grokking) fn lattice_seed_rows_batched(
        &self,
        surface: &str,
        limit: usize,
    ) -> Vec<(u32, String, u32)> {
        self.lattice_seed_rows_with_parallel_packages(surface, limit, false)
    }

    fn lattice_seed_rows_with_parallel_packages(
        &self,
        surface: &str,
        limit: usize,
        parallel_packages: bool,
    ) -> Vec<(u32, String, u32)> {
        let limit = limit.max(1);
        let exact_terminal = self.terminal_for_exact_surface(surface);
        if limit == 1 {
            if let Some(terminal_id) = exact_terminal {
                if let Some(surface) = self.decode_terminal(terminal_id) {
                    return vec![(terminal_id, surface, u32::MAX)];
                }
            }
        }
        let mut rows = if parallel_packages {
            if let [overlay] = self.overlays.as_slice() {
                let (mut base, delta) = v8::runtime_pool_install(|| {
                    rayon::join(
                        || memory_seed_rows(&self.memory, 0, surface, limit),
                        || {
                            memory_seed_rows(
                                &overlay.memory,
                                overlay.terminal_offset,
                                surface,
                                limit,
                            )
                        },
                    )
                });
                base.extend(delta);
                base
            } else {
                self.sequential_seed_rows(surface, limit)
            }
        } else {
            self.sequential_seed_rows(surface, limit)
        };
        rows.retain(|row| !self.is_tombstoned(&row.surface));
        rows.sort_unstable_by(|left, right| {
            (right.terminal_id == exact_terminal.unwrap_or(u32::MAX))
                .cmp(&(left.terminal_id == exact_terminal.unwrap_or(u32::MAX)))
                .then_with(|| left.local_rank.cmp(&right.local_rank))
                .then_with(|| left.geometry_distance.cmp(&right.geometry_distance))
                .then_with(|| right.score_milli.cmp(&left.score_milli))
                .then_with(|| left.surface.cmp(&right.surface))
                .then_with(|| left.terminal_id.cmp(&right.terminal_id))
        });
        let mut seen = BTreeSet::new();
        rows.retain(|row| seen.insert(normalize_lexical_surface(&row.surface)));
        rows.truncate(limit);
        rows.into_iter()
            .map(|row| (row.terminal_id, row.surface, row.score_milli))
            .collect()
    }

    fn sequential_seed_rows(&self, surface: &str, limit: usize) -> Vec<LatticeSeedRow> {
        let mut rows = memory_seed_rows(&self.memory, 0, surface, limit);
        for overlay in &self.overlays {
            rows.extend(memory_seed_rows(
                &overlay.memory,
                overlay.terminal_offset,
                surface,
                limit,
            ));
        }
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
                    .memory
                    .exact_surface_collisions
                    .values()
                    .map(Vec::len)
                    .sum::<usize>()
                + self
                    .overlays
                    .iter()
                    .map(|overlay| {
                        overlay.memory.exact_surface_index.len()
                            + overlay
                                .memory
                                .exact_surface_collisions
                                .values()
                                .map(Vec::len)
                                .sum::<usize>()
                    })
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

struct LatticeSeedRow {
    terminal_id: u32,
    surface: String,
    score_milli: u32,
    geometry_distance: u8,
    local_rank: u16,
}

fn memory_seed_rows(
    memory: &LexicalGrokkingMemory,
    terminal_offset: u32,
    surface: &str,
    limit: usize,
) -> Vec<LatticeSeedRow> {
    memory
        .readout(surface, limit, ReadoutMode::Full)
        .into_iter()
        .enumerate()
        .filter_map(|(local_rank, candidate)| {
            let terminal_id = terminal_offset.checked_add(candidate.terminal_id)?;
            let surface = memory.decode_terminal(candidate.terminal_id)?;
            Some(LatticeSeedRow {
                terminal_id,
                surface,
                score_milli: lattice_seed_score(candidate),
                geometry_distance: candidate.geometry_distance,
                local_rank: local_rank.min(u16::MAX as usize) as u16,
            })
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
