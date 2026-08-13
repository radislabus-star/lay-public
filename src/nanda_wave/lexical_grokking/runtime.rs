use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use rayon::prelude::*;

mod config;
mod contract;
mod diagnostics;
mod geometry;
mod host;
mod legacy;
mod relations;

use legacy::select_birth_atoms;
#[cfg(test)]
use legacy::should_expand_operator_lattice;

use config::{
    birth_atoms_per_channel, birth_posting_budget, first_touch_profile_word_count,
    readout_trace_enabled, readout_trace_terminal, FIRST_TOUCH_TRANSIENT_RESERVE_MIB,
};
#[cfg(test)]
use config::{DEFAULT_BIRTH_ATOMS_PER_CHANNEL, DEFAULT_BIRTH_POSTING_BUDGET};
pub(super) use contract::{AmbiguityObservation, GrokkingCandidate, ReadoutMode};
use contract::{
    AnchorSequence, BirthAtom, CachePlanOrder, FirstTouchWarmProfile, ForwardActivation,
    ObservedAtom, PreparedReadout,
};
pub(super) use diagnostics::candidate_json;
use diagnostics::percent_usize;
pub use diagnostics::{
    benchmark_diverse_restoration, benchmark_package, inspect_package_header, query_package,
    restore_surface,
};
use geometry::MAX_ANCHOR_SEQUENCE;
pub(super) use geometry::{
    ambiguity_geometry_link, damerau_distance, reconstruction_modes,
    surface_operator_reconstruction_modes, RECONSTRUCTION_MODE_DELETION,
    RECONSTRUCTION_MODE_DELETION_TRANSPOSITION, RECONSTRUCTION_MODE_DOUBLE_SUBSTITUTION,
    RECONSTRUCTION_MODE_NON_ADJACENT_TRANSPOSITION, RECONSTRUCTION_MODE_PREFIX_TRUNCATION,
    RECONSTRUCTION_MODE_SINGLE_DELETION, RECONSTRUCTION_MODE_SINGLE_SUBSTITUTION,
    RECONSTRUCTION_MODE_SUFFIX_TRUNCATION,
};
#[cfg(test)]
use geometry::{is_ordered_subsequence, is_subsequence_after_one_adjacent_swap};
pub use host::{L1RestorationHost, L1RestorationHostStats};
use relations::{RelationStore, ReverseCache};

use crate::stable_hash::mix64_golden;

use super::atoms::{
    encode_wave_surface, normalize_lexical_surface, physical_key_sequence, AtomChannel, NGramKey,
};
use super::corruption::split_scale_damages;
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
const SETTLING_ITERATIONS: u8 = 3;
const MAX_EXACT_COLLISION_OPERATOR_CHARS: usize = 16;

pub(super) struct LexicalGrokkingMemory {
    pub(super) package: LexicalGrokkingPackage,
    exact_surface_index: HashMap<u64, u32>,
    exact_surface_collisions: HashMap<u64, Vec<u32>>,
    character_anchor_by_char: HashMap<char, u32>,
    character_anchor_offsets: Vec<u32>,
    character_anchor_atoms: Vec<u32>,
    relations: RelationStore,
    reverse_cache: Mutex<ReverseCache>,
}

impl LexicalGrokkingMemory {
    pub(super) fn from_package(package: LexicalGrokkingPackage) -> Self {
        let (
            exact_surface_index,
            exact_surface_collisions,
            character_anchor_by_char,
            character_anchor_offsets,
            character_anchor_atoms,
        ) = compile_surface_indices(&package);
        Self {
            package,
            exact_surface_index,
            exact_surface_collisions,
            character_anchor_by_char,
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
        let (
            exact_surface_index,
            exact_surface_collisions,
            character_anchor_by_char,
            character_anchor_offsets,
            character_anchor_atoms,
        ) = compile_surface_indices(&package);
        Ok(Self {
            package,
            exact_surface_index,
            exact_surface_collisions,
            character_anchor_by_char,
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

    pub(super) fn warm_first_touch(&self) -> io::Result<serde_json::Value> {
        let RelationStore::LazyV8(artifact) = &self.relations else {
            return Ok(serde_json::json!({
                "format": "eager",
                "eligible_atoms": 0,
                "eligible_relations": 0,
            }));
        };
        let profile = self.first_touch_warm_profile(artifact.posting_cache_budget_bytes());
        let stats = artifact
            .warm_first_touch(&profile.atom_ids)
            .map_err(io::Error::other)?;
        Ok(serde_json::json!({
            "format": "v8",
            "sampled_words": profile.sampled_words,
            "damage_surfaces": profile.damage_surfaces,
            "observed_atoms": profile.observed_atoms,
            "protected_budget_bytes": profile.protected_budget_bytes,
            "eligible_atoms": stats.eligible_atoms,
            "eligible_relations": stats.eligible_relations,
            "posting_cache_bytes": stats.posting_cache_bytes,
            "posting_cache_entries": stats.posting_cache_entries,
            "protected_cache_bytes": stats.protected_cache_bytes,
            "protected_cache_entries": stats.protected_cache_entries,
            "shard_cache_bytes": stats.shard_cache_bytes,
            "shard_cache_entries": stats.shard_cache_entries,
        }))
    }

    fn first_touch_warm_profile(&self, cache_budget_bytes: usize) -> FirstTouchWarmProfile {
        let terminal_count = self.package.terminal_count() as usize;
        let sampled_words = first_touch_profile_word_count().min(terminal_count);
        let mut atom_uses = BTreeMap::<u32, usize>::new();
        let mut damage_surfaces = 0_usize;
        for sample in 0..sampled_words {
            let terminal_id = sample
                .saturating_mul(terminal_count)
                .saturating_add(sampled_words / 2)
                / sampled_words.max(1);
            let Some(word) = self.decode_terminal(terminal_id as u32) else {
                continue;
            };
            let (training, heldout) = split_scale_damages(&word, false);
            let mut by_class = BTreeMap::<&'static str, String>::new();
            for example in training.into_iter().chain(heldout) {
                by_class
                    .entry(example.class)
                    .and_modify(|surface| {
                        if example.surface < *surface {
                            *surface = example.surface.clone();
                        }
                    })
                    .or_insert(example.surface);
            }
            for surface in by_class.into_values() {
                damage_surfaces += 1;
                let observed = self.resolve_surface(&surface);
                let mut by_channel: [Vec<BirthAtom>; 12] = std::array::from_fn(|_| Vec::new());
                for (atom_id, atom) in observed
                    .iter()
                    .filter(|(_, atom)| !is_anchor_channel(atom.channel))
                {
                    by_channel[atom.channel as usize].push((
                        self.forward_degree(*atom_id),
                        *atom_id,
                        *atom,
                    ));
                }
                for (_, atom_id, _) in select_birth_atoms(
                    &mut by_channel,
                    birth_atoms_per_channel(),
                    birth_posting_budget(),
                ) {
                    *atom_uses.entry(atom_id).or_default() += 1;
                }
            }
        }
        let observed_atoms = atom_uses.len();
        let mut ranked = atom_uses
            .into_iter()
            .map(|(atom_id, uses)| (uses, self.forward_degree(atom_id), atom_id))
            .filter(|(_, degree, _)| *degree != 0)
            .collect::<Vec<_>>();
        ranked.sort_unstable_by(|left, right| {
            right
                .0
                .cmp(&left.0)
                .then_with(|| right.1.cmp(&left.1))
                .then_with(|| left.2.cmp(&right.2))
        });
        let reserve = FIRST_TOUCH_TRANSIENT_RESERVE_MIB.saturating_mul(1024 * 1024);
        let protected_budget_bytes =
            cache_budget_bytes.saturating_sub(reserve.min(cache_budget_bytes));
        let mut protected_bytes = 0_usize;
        let mut atom_ids = Vec::new();
        for (_, degree, atom_id) in ranked {
            let posting_bytes = degree.saturating_mul(std::mem::size_of::<WaveCoupling>());
            if posting_bytes > protected_budget_bytes.saturating_sub(protected_bytes) {
                continue;
            }
            protected_bytes = protected_bytes.saturating_add(posting_bytes);
            atom_ids.push(atom_id);
        }
        FirstTouchWarmProfile {
            atom_ids,
            sampled_words,
            damage_surfaces,
            observed_atoms,
            protected_budget_bytes,
        }
    }

    fn birth_profile(&self, surfaces: &[String]) -> serde_json::Value {
        let mut uses = BTreeMap::<u32, (usize, AtomChannel)>::new();
        let mut selected_references = 0_usize;
        let mut selected_relations = 0_usize;
        for surface in surfaces {
            let observed = self.resolve_surface(surface);
            let mut by_channel: [Vec<BirthAtom>; 12] = std::array::from_fn(|_| Vec::new());
            for (atom_id, atom) in observed
                .iter()
                .filter(|(_, atom)| !is_anchor_channel(atom.channel))
            {
                by_channel[atom.channel as usize].push((
                    self.forward_degree(*atom_id),
                    *atom_id,
                    *atom,
                ));
            }
            for (degree, atom_id, atom) in select_birth_atoms(
                &mut by_channel,
                birth_atoms_per_channel(),
                birth_posting_budget(),
            ) {
                selected_references += 1;
                selected_relations = selected_relations.saturating_add(degree);
                uses.entry(atom_id)
                    .and_modify(|entry| entry.0 += 1)
                    .or_insert((1, atom.channel));
            }
        }
        let unique_relations = uses
            .keys()
            .map(|atom_id| self.forward_degree(*atom_id))
            .sum::<usize>();
        let mut hottest = uses
            .iter()
            .map(|(atom_id, (uses, channel))| {
                let atom_id = *atom_id;
                let uses = *uses;
                let channel = *channel;
                let degree = self.forward_degree(atom_id);
                (uses.saturating_mul(degree), atom_id, uses, degree, channel)
            })
            .collect::<Vec<_>>();
        hottest.sort_unstable_by(|left, right| {
            right
                .0
                .cmp(&left.0)
                .then_with(|| right.3.cmp(&left.3))
                .then_with(|| left.1.cmp(&right.1))
        });
        let cache_plans = [96_usize, 128]
            .into_iter()
            .map(|budget_mib| {
                let budget_bytes = budget_mib.saturating_mul(1024 * 1024);
                serde_json::json!({
                    "budget_mib": budget_mib,
                    "support": self.simulate_posting_cache_plan(
                        &uses,
                        selected_references,
                        selected_relations,
                        budget_bytes,
                        CachePlanOrder::Support,
                    ),
                    "degree": self.simulate_posting_cache_plan(
                        &uses,
                        selected_references,
                        selected_relations,
                        budget_bytes,
                        CachePlanOrder::Degree,
                    ),
                    "oracle": self.simulate_posting_cache_plan(
                        &uses,
                        selected_references,
                        selected_relations,
                        budget_bytes,
                        CachePlanOrder::ObservedUses,
                    ),
                })
            })
            .collect::<Vec<_>>();
        serde_json::json!({
            "selected_references": selected_references,
            "selected_relations": selected_relations,
            "unique_atoms": hottest.len(),
            "unique_relations": unique_relations,
            "unique_decoded_bytes": unique_relations.saturating_mul(std::mem::size_of::<WaveCoupling>()),
            "cache_plans": cache_plans,
            "top_expected_decode_work": hottest
                .into_iter()
                .take(64)
                .map(|(work, atom_id, uses, degree, channel)| serde_json::json!({
                    "atom_id": atom_id,
                    "channel": format!("{channel:?}"),
                    "uses": uses,
                    "degree": degree,
                    "expected_decode_work": work,
                    "decoded_bytes": degree.saturating_mul(std::mem::size_of::<WaveCoupling>()),
                }))
                .collect::<Vec<_>>(),
        })
    }

    fn simulate_posting_cache_plan(
        &self,
        observed_uses: &BTreeMap<u32, (usize, AtomChannel)>,
        selected_references: usize,
        selected_relations: usize,
        budget_bytes: usize,
        order: CachePlanOrder,
    ) -> serde_json::Value {
        let mut atoms = self
            .package
            .atoms
            .iter()
            .enumerate()
            .map(|(atom_id, record)| {
                let atom_id = atom_id as u32;
                let degree = self.forward_degree(atom_id);
                let uses = observed_uses
                    .get(&atom_id)
                    .map(|entry| entry.0)
                    .unwrap_or_default();
                (atom_id, usize::from(record.support), degree, uses)
            })
            .collect::<Vec<_>>();
        atoms.sort_unstable_by(|left, right| {
            let left_key = match order {
                CachePlanOrder::Support => left.1,
                CachePlanOrder::Degree => left.2,
                CachePlanOrder::ObservedUses => left.3,
            };
            let right_key = match order {
                CachePlanOrder::Support => right.1,
                CachePlanOrder::Degree => right.2,
                CachePlanOrder::ObservedUses => right.3,
            };
            right_key
                .cmp(&left_key)
                .then_with(|| right.2.cmp(&left.2))
                .then_with(|| left.0.cmp(&right.0))
        });
        let mut bytes = 0_usize;
        let mut atom_count = 0_usize;
        let mut relation_count = 0_usize;
        let mut observed_reference_hits = 0_usize;
        let mut observed_relation_hits = 0_usize;
        for (_, _, degree, uses) in atoms {
            let posting_bytes = degree.saturating_mul(std::mem::size_of::<WaveCoupling>());
            if posting_bytes > budget_bytes.saturating_sub(bytes) {
                continue;
            }
            bytes = bytes.saturating_add(posting_bytes);
            atom_count += 1;
            relation_count = relation_count.saturating_add(degree);
            observed_reference_hits = observed_reference_hits.saturating_add(uses);
            observed_relation_hits =
                observed_relation_hits.saturating_add(uses.saturating_mul(degree));
        }
        serde_json::json!({
            "atom_count": atom_count,
            "relation_count": relation_count,
            "bytes": bytes,
            "observed_reference_coverage_percent": percent_usize(
                observed_reference_hits,
                selected_references,
            ),
            "observed_decode_work_coverage_percent": percent_usize(
                observed_relation_hits,
                selected_relations,
            ),
        })
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

    fn finish_readout(
        &self,
        surface: &str,
        limit: usize,
        mode: ReadoutMode,
        prepared: &PreparedReadout,
    ) -> Vec<GrokkingCandidate> {
        let trace_started = Instant::now();
        let mut candidates = self.settle_prepared_candidates(prepared, mode);
        let settle_us = trace_started.elapsed().as_micros();
        self.apply_restoration_geometry(surface, &mut candidates);
        let geometry_us = trace_started.elapsed().as_micros();
        self.finalize_candidates_after_geometry(
            limit,
            mode,
            &prepared.surface_re,
            &prepared.surface_im,
            &mut candidates,
        );
        if readout_trace_enabled() {
            let finish_us = trace_started.elapsed().as_micros();
            eprintln!(
                "l11_finish_trace settle_us={} geometry_us={} finalize_us={} finish_us={} candidates={}",
                settle_us,
                geometry_us.saturating_sub(settle_us),
                finish_us.saturating_sub(geometry_us),
                finish_us,
                candidates.len(),
            );
        }
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

fn compile_surface_indices(
    package: &LexicalGrokkingPackage,
) -> (
    HashMap<u64, u32>,
    HashMap<u64, Vec<u32>>,
    HashMap<char, u32>,
    Vec<u32>,
    Vec<u32>,
) {
    let mut index = HashMap::with_capacity(package.centers.len());
    let mut collisions = HashMap::<u64, Vec<u32>>::new();
    let mut anchor_by_char = HashMap::new();
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
                anchor_by_char.entry(ch).or_insert(atom_id);
                anchors.atoms[position] = atom_id;
                anchors.len = anchors.len.saturating_add(1);
            }
        }
        if complete && !anchors.as_slice().is_empty() {
            let hash = anchor_sequence_hash(anchors.as_slice());
            match index.entry(hash) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(terminal as u32);
                }
                std::collections::hash_map::Entry::Occupied(_) => {
                    collisions.entry(hash).or_default().push(terminal as u32);
                }
            }
            atoms.extend_from_slice(anchors.as_slice());
        }
        offsets.push(atoms.len() as u32);
    }
    (index, collisions, anchor_by_char, offsets, atoms)
}

fn anchor_sequence_hash(sequence: &[u32]) -> u64 {
    let mut state = mix64_golden(0x4c31_4558_4143_5431 ^ sequence.len() as u64);
    for atom in sequence {
        state = mix64_golden(state ^ u64::from(*atom));
    }
    state
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
            exact_surface_index: HashMap::new(),
            exact_surface_collisions: HashMap::new(),
            character_anchor_by_char: HashMap::new(),
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
