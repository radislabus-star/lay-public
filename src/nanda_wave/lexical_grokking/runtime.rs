use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::Path;
use std::time::Instant;

use crate::stable_hash::mix64_golden;

use super::atoms::{
    encode_wave_surface, normalize_lexical_surface, physical_key_sequence, AtomChannel,
};
use super::crystal::{AmbiguityPhaseCenter64, WAVE_DIMENSION};
use super::format;
use super::model::{
    LexicalGrokkingPackage, WaveCoupling, CENTER_PHASE_FLAG_PHYSICAL_KEY_GEOMETRY,
    COUPLING_FLAG_CHARACTER_ANCHOR,
};
use super::wave_basis::{
    complex_coherence_milli, expand_atom, expand_word, pair_residual_atoms, positioned_atom_code,
};

const MAX_PHASE_FRONTIER: usize = 128;
const MAX_GEOMETRY_RESERVE: usize = 32;
const SETTLING_ITERATIONS: u8 = 3;
const MAX_ANCHOR_SEQUENCE: usize = 32;
pub(super) const RECONSTRUCTION_MODE_DELETION: u8 = 1;
pub(super) const RECONSTRUCTION_MODE_DELETION_TRANSPOSITION: u8 = 2;

pub fn query_package(
    package_path: &Path,
    surface: &str,
    limit: usize,
) -> io::Result<serde_json::Value> {
    let bytes = std::fs::read(package_path)?;
    let memory = LexicalGrokkingMemory::from_bytes(&bytes).map_err(io::Error::other)?;
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
    let bytes = std::fs::read(package_path)?;
    let memory = LexicalGrokkingMemory::from_bytes(&bytes).map_err(io::Error::other)?;
    let mut candidates = memory.readout(surface, limit.max(1), ReadoutMode::Full);
    let readout = memory.classify_restoration(
        surface,
        &mut candidates,
        memory.package.restoration_calibration,
    );
    let result = match readout {
        super::restoration::RestorationReadout::Winner { candidate } => {
            serde_json::json!({
                "verdict": "winner",
                "authority": true,
                "candidate": restoration_candidate_json(&memory, candidate),
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
                .map(|candidate| restoration_candidate_json(&memory, candidate))
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
                .map(|candidate| restoration_candidate_json(&memory, candidate))
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
                .map(|candidate| restoration_candidate_json(&memory, candidate))
                .collect::<Vec<_>>(),
        }),
    };
    Ok(serde_json::json!({
        "package": package_path,
        "input": surface,
        "terminal_count": memory.package.terminal_count(),
        "result": result,
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
) -> io::Result<serde_json::Value> {
    let bytes = std::fs::read(package_path)?;
    let memory = LexicalGrokkingMemory::from_bytes(&bytes).map_err(io::Error::other)?;
    for _ in 0..16 {
        std::hint::black_box(memory.readout(surface, 64, ReadoutMode::Full));
    }
    let mut elapsed_us = Vec::with_capacity(iterations);
    let mut checksum = 0_u64;
    for _ in 0..iterations {
        let started = Instant::now();
        let candidates = memory.readout(surface, 64, ReadoutMode::Full);
        elapsed_us.push(started.elapsed().as_micros() as u64);
        checksum ^= candidates
            .first()
            .map(|candidate| u64::from(candidate.terminal_id))
            .unwrap_or_default();
    }
    elapsed_us.sort_unstable();
    Ok(serde_json::json!({
        "surface": surface,
        "iterations": iterations,
        "terminal_count": memory.package.terminal_count(),
        "p50_us": percentile(&elapsed_us, 50),
        "p90_us": percentile(&elapsed_us, 90),
        "p99_us": percentile(&elapsed_us, 99),
        "max_us": elapsed_us.last().copied().unwrap_or_default(),
        "checksum": checksum,
    }))
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
    touched: Vec<u32>,
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
    character_anchors: Vec<Vec<u32>>,
}

impl LexicalGrokkingMemory {
    pub(super) fn from_package(package: LexicalGrokkingPackage) -> Self {
        let character_anchors = compile_character_anchors(&package);
        let exact_surface_index = compile_exact_surface_index(&character_anchors);
        Self {
            package,
            exact_surface_index,
            character_anchors,
        }
    }

    pub(super) fn into_package(self) -> LexicalGrokkingPackage {
        self.package
    }

    pub(super) fn ambiguity_center_count(&self) -> usize {
        self.package.ambiguity_subcenters.len()
    }

    pub(super) fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        Ok(Self::from_package(format::decode(bytes)?))
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
        let observed = self.resolve_surface(surface);
        if observed.is_empty() {
            return Vec::new();
        }
        let character_sequence = observed_sequence(&observed, AtomChannel::CharacterAnchor);
        let exact_terminals = self.exact_terminals(character_sequence.as_slice());
        let observed_char_count = normalize_lexical_surface(surface)
            .chars()
            .count()
            .min(u8::MAX as usize) as u8;
        let (surface_re, surface_im, max_forward, mut frontier) =
            FORWARD_SCRATCH.with_borrow_mut(|scratch| {
                if scratch.activations.len() != self.package.terminal_count() as usize {
                    scratch.activations =
                        vec![ForwardActivation::default(); self.package.terminal_count() as usize];
                    scratch.touched.clear();
                } else {
                    let mut touched = std::mem::take(&mut scratch.touched);
                    for terminal_id in touched.drain(..) {
                        scratch.activations[terminal_id as usize] = ForwardActivation::default();
                    }
                    scratch.touched = touched;
                }
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
                    for coupling in self.forward_couplings(*atom_id) {
                        let position = position_coherence(atom.position, coupling.position_mode);
                        let contribution = u64::from(coupling.strength)
                            .saturating_mul(u64::from(atom.weight))
                            .saturating_mul(u64::from(position));
                        let terminal_id = coupling.peer_id as usize;
                        if terminal_id >= scratch.activations.len() {
                            continue;
                        }
                        if scratch.activations[terminal_id].hits == 0 {
                            scratch.touched.push(coupling.peer_id);
                        }
                        let activation = &mut scratch.activations[terminal_id];
                        activation.mass = activation.mass.saturating_add(contribution);
                        activation.hits = activation.hits.saturating_add(1);
                        if is_keyboard_channel(atom.channel) {
                            activation.keyboard_hits = activation.keyboard_hits.saturating_add(1);
                        } else {
                            activation.surface_hits = activation.surface_hits.saturating_add(1);
                        }
                    }
                }
                let max_forward = scratch
                    .touched
                    .iter()
                    .map(|terminal_id| scratch.activations[*terminal_id as usize].mass)
                    .max()
                    .unwrap_or(1)
                    .max(1);
                let frontier = scratch
                    .touched
                    .iter()
                    .map(|terminal_id| (*terminal_id, scratch.activations[*terminal_id as usize]))
                    .collect::<Vec<_>>();
                (surface_re, surface_im, max_forward, frontier)
            });
        let geometry_reserve = if exact_terminals.is_empty() {
            self.geometry_reserve(&frontier, character_sequence.as_slice())
        } else {
            Vec::new()
        };
        let geometry_reserve_ids = geometry_reserve
            .iter()
            .map(|(terminal_id, _)| *terminal_id)
            .collect::<BTreeSet<_>>();
        frontier.sort_unstable_by(|left, right| {
            exact_terminals
                .contains(&right.0)
                .cmp(&exact_terminals.contains(&left.0))
                .then_with(|| right.1.mass.cmp(&left.1.mass))
                .then_with(|| right.1.hits.cmp(&left.1.hits))
                .then_with(|| left.0.cmp(&right.0))
        });
        frontier.truncate(MAX_PHASE_FRONTIER.max(limit));
        let mut retained = frontier
            .iter()
            .map(|(terminal_id, _)| *terminal_id)
            .collect::<BTreeSet<_>>();
        frontier.extend(
            geometry_reserve
                .into_iter()
                .filter(|(terminal_id, _)| retained.insert(*terminal_id)),
        );
        let observed = observed
            .into_iter()
            .filter(|(_, atom)| !is_anchor_channel(atom.channel))
            .collect::<BTreeMap<_, _>>();
        let mut candidates = frontier
            .into_iter()
            .filter_map(|(terminal_id, activation)| {
                self.settle_candidate(
                    terminal_id,
                    activation,
                    max_forward,
                    &observed,
                    &surface_re,
                    &surface_im,
                    &character_sequence,
                    observed_char_count,
                    mode,
                )
            })
            .map(|mut candidate| {
                candidate.ambiguity_shell = geometry_reserve_ids.contains(&candidate.terminal_id);
                candidate
            })
            .collect::<Vec<_>>();
        apply_structural_interference(&mut candidates);
        if mode != ReadoutMode::WithoutAnti {
            self.apply_pairwise_interference(&mut candidates, &surface_re, &surface_im);
        }
        apply_sequence_certificate_interference(&mut candidates, mode);
        if mode != ReadoutMode::WithoutPairwise {
            super::pairwise::apply_pairwise_field(
                &self.package.pair_profiles,
                &self.package.pair_centers,
                &self.package.basis,
                &mut candidates,
                &surface_re,
                &surface_im,
            );
        }
        candidates.sort_unstable_by(candidate_order);
        if mode != ReadoutMode::WithoutPosition {
            apply_position_certificate_interference(&mut candidates);
        }
        candidates.truncate(limit);
        candidates
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
                (self.character_anchors.get(*terminal as usize)?.as_slice() == observed)
                    .then_some(*terminal)
            })
            .collect()
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
            let Some(expected) = self.character_anchors.get(*terminal_id as usize) else {
                continue;
            };
            if expected.len().abs_diff(observed.len()) > maximum_distance {
                continue;
            }
            let distance = damerau_distance(observed, expected);
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
        let observed_keyboard_sequence = observed_sequence(&observed, AtomChannel::KeyboardGram);
        let observed_physical_keys = physical_key_sequence(surface);
        let observed_script_flags = super::model::surface_script_flags(surface);
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
        for candidate in candidates.iter_mut() {
            let Some(center) = self.package.centers.get(candidate.terminal_id as usize) else {
                continue;
            };
            let Some(profile) = self
                .package
                .center_phase_profiles
                .get(candidate.terminal_id as usize)
            else {
                continue;
            };
            let cross_script = center.flags != 0
                && observed_script_flags != 0
                && center.flags & observed_script_flags == 0;
            let reverse = self.reverse_couplings(*center);
            let expected_character_sequence =
                expected_sequence(reverse, COUPLING_FLAG_CHARACTER_ANCHOR);
            candidate.reconstruction_modes =
                if observed_character_sequence.as_slice().len() == normalized_char_count {
                    reconstruction_modes(
                        observed_character_sequence.as_slice(),
                        expected_character_sequence.as_slice(),
                    )
                } else {
                    0
                };
            if !cross_script {
                continue;
            }
            let start = profile.keyboard_geometry_start as usize;
            let end = start.saturating_add(profile.keyboard_geometry_count as usize);
            let Some(expected) = self.package.keyboard_geometry_units.get(start..end) else {
                continue;
            };
            if expected.is_empty() {
                continue;
            }
            let observed_geometry = if profile.flags & CENTER_PHASE_FLAG_PHYSICAL_KEY_GEOMETRY != 0
            {
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
                let Some(owner_center) = self.package.centers.get(candidate.terminal_id as usize)
                else {
                    continue;
                };
                let Some(competitor_center) =
                    self.package.centers.get(center.decoder_terminal as usize)
                else {
                    continue;
                };
                let competitor_character_sequence = expected_sequence(
                    self.reverse_couplings(*competitor_center),
                    COUPLING_FLAG_CHARACTER_ANCHOR,
                );
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
                let (residual_re, residual_im) = expanded_pair_residual_wave(
                    &observed,
                    &self.package.atoms,
                    &self.package.basis,
                    self.reverse_couplings(*owner_center),
                    self.reverse_couplings(*competitor_center),
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
            let Some(owner_center) = self.package.centers.get(owner.terminal_id as usize) else {
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
                let Some(competitor_center) = self
                    .package
                    .centers
                    .get(relation_center.decoder_terminal as usize)
                else {
                    continue;
                };
                let (residual_re, residual_im) = expanded_pair_residual_wave(
                    &observed,
                    &self.package.atoms,
                    &self.package.basis,
                    self.reverse_couplings(*owner_center),
                    self.reverse_couplings(*competitor_center),
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
        let center = *self.package.centers.get(terminal_id as usize)?;
        let reverse = self.reverse_couplings(center);
        let expected_char_count = center.surface_len;
        let anchors_cover_surface =
            character_sequence.as_slice().len() == usize::from(observed_char_count);
        let legacy_sequence_milli =
            if observed_char_count < expected_char_count && anchors_cover_surface {
                legacy_reconstruction_sequence_milli(reverse, character_sequence)
            } else {
                750
            };
        let expected_character_sequence =
            expected_sequence(reverse, COUPLING_FLAG_CHARACTER_ANCHOR);
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
                    .max(reconstruction_sequence_milli(reverse, character_sequence))
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
        let mut energy = i32::from(forward_milli) * 3;
        for _ in 0..SETTLING_ITERATIONS {
            let constructive =
                i32::from(backward_milli) * 3 + (i32::from(positive_milli) - 500).max(0) * 2;
            let destructive = i32::from(anti_milli) * 4
                + (500 - i32::from(positive_milli)).max(0) * 2
                + (1_000 - i32::from(length_milli)) * 2;
            energy =
                (energy + i32::from(forward_milli) * 3 + constructive + i32::from(length_milli)
                    - destructive)
                    / 2;
        }
        let legacy_settled_energy =
            energy.saturating_add((i32::from(legacy_sequence_milli) - 750).saturating_mul(3));
        energy = energy.saturating_add((i32::from(sequence_milli) - 750).saturating_mul(3));
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

    fn forward_couplings(&self, atom_id: u32) -> &[WaveCoupling] {
        let Some(record) = self.package.atoms.get(atom_id as usize) else {
            return &[];
        };
        let start = record.coupling_start as usize;
        let end = start.saturating_add(record.coupling_count as usize);
        self.package
            .forward_couplings
            .get(start..end)
            .unwrap_or_default()
    }

    fn reverse_couplings(&self, center: super::crystal::WordCenter64) -> &[WaveCoupling] {
        let start = center.coupling_start as usize;
        let end = start.saturating_add(center.coupling_count as usize);
        self.package
            .reverse_couplings
            .get(start..end)
            .unwrap_or_default()
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

pub(super) fn damerau_distance(left: &[u32], right: &[u32]) -> usize {
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

fn compile_character_anchors(package: &LexicalGrokkingPackage) -> Vec<Vec<u32>> {
    package
        .centers
        .iter()
        .map(|center| {
            let start = center.coupling_start as usize;
            let end = start.saturating_add(center.coupling_count as usize);
            expected_sequence(
                package
                    .reverse_couplings
                    .get(start..end)
                    .unwrap_or_default(),
                COUPLING_FLAG_CHARACTER_ANCHOR,
            )
            .as_slice()
            .to_vec()
        })
        .collect()
}

fn compile_exact_surface_index(character_anchors: &[Vec<u32>]) -> Vec<(u64, u32)> {
    let mut index = character_anchors
        .iter()
        .enumerate()
        .map(|(terminal, anchors)| (anchor_sequence_hash(anchors), terminal as u32))
        .collect::<Vec<_>>();
    index.sort_unstable();
    index
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
    fn geometry_reserve_keeps_the_nearest_basin_and_ambiguity_shell() {
        let memory = LexicalGrokkingMemory {
            package: LexicalGrokkingPackage {
                restoration_calibration: super::super::restoration::RestorationCalibration {
                    max_geometry_distance: 2,
                    ..Default::default()
                },
                ..Default::default()
            },
            exact_surface_index: Vec::new(),
            character_anchors: vec![vec![1, 9, 9], vec![1, 2, 4], vec![1, 2, 5]],
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
}
