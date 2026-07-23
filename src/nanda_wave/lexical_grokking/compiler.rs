use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use crate::stable_hash::mix64_golden;

use super::atoms::{encode_wave_surface, physical_key_sequence, AtomChannel, NGramKey};
use super::crystal::{WordCenter64, WAVE_DIMENSION};
use super::model::{
    AtomRecord, CenterPhaseProfile, DecoderNode, LexicalGrokkingPackage, PairKey, PairPhaseProfile,
    WaveCoupling, COUPLING_FLAG_CHARACTER_ANCHOR,
};
use super::ngram_graph::NGramGraph;
use super::restoration::RestorationCalibration;
use super::wave_basis::{
    compile_basis, complex_coherence_milli, expand_atom, expand_word, learn_atom_code,
    settle_word_code,
};

const MAX_FORWARD_COUPLINGS: usize = 256;
const MAX_REVERSE_LEXICAL_COUPLINGS: usize = 96;
type AntiRelation = ([i64; super::crystal::WAVE_DIMENSION], u32, u64);
type PhaseMass = [i64; WAVE_DIMENSION];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ForwardPostingPolicy {
    BaselineBounded,
    Complete,
}

impl ForwardPostingPolicy {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::BaselineBounded => "bounded_256",
            Self::Complete => "complete",
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct CompileDiagnostics {
    pub(super) forward_relations_before_policy: usize,
    pub(super) forward_relations_dropped: usize,
    pub(super) forward_atoms_above_baseline_cap: usize,
    pub(super) max_forward_degree: usize,
}

pub(super) struct CompileOutput {
    pub(super) package: LexicalGrokkingPackage,
    pub(super) diagnostics: CompileDiagnostics,
}

pub(super) struct TrainingWord {
    pub(super) terminal_id: u32,
    pub(super) surface: String,
    pub(super) training_surfaces: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default)]
struct CouplingStats {
    observations: u32,
    weighted_observations: u32,
    position_sum: u64,
}

#[derive(Clone)]
struct ResolvedSurface {
    atoms: Vec<(u32, u16, u8, AtomChannel)>,
    physical_keys: Vec<u32>,
}

struct L11Banks {
    profiles: Vec<CenterPhaseProfile>,
    positive: Vec<WordCenter64>,
    anti: Vec<WordCenter64>,
    hard_negative: Vec<WordCenter64>,
    keyboard_geometry: Vec<u32>,
}

#[cfg(test)]
pub(super) fn compile(words: &[TrainingWord]) -> Result<LexicalGrokkingPackage, String> {
    Ok(compile_with_policy(words, ForwardPostingPolicy::BaselineBounded)?.package)
}

pub(super) fn compile_with_policy(
    words: &[TrainingWord],
    forward_policy: ForwardPostingPolicy,
) -> Result<CompileOutput, String> {
    validate_words(words)?;
    let graph = compile_graph(words)?;
    let resolved = resolve_training_surfaces(&graph, words);
    let mut atom_flags = vec![0_u8; graph.atom_count as usize];
    for surface in resolved.iter().flatten() {
        for (atom_id, _, _, channel) in &surface.atoms {
            atom_flags[*atom_id as usize] = coupling_flag(*channel);
        }
    }
    let mut per_word = vec![BTreeMap::<u32, CouplingStats>::new(); words.len()];
    let mut atom_word_support = vec![0_u32; graph.atom_count as usize];
    for (word_index, surfaces) in resolved.iter().enumerate() {
        for surface in surfaces {
            let mut seen = BTreeSet::new();
            for (atom_id, position, weight, _) in surface.atoms.iter().copied() {
                let stats = per_word[word_index].entry(atom_id).or_default();
                stats.observations = stats.observations.saturating_add(1);
                stats.weighted_observations = stats
                    .weighted_observations
                    .saturating_add(u32::from(weight));
                stats.position_sum = stats.position_sum.saturating_add(u64::from(position));
                seen.insert(atom_id);
            }
            for atom_id in seen {
                atom_word_support[atom_id as usize] =
                    atom_word_support[atom_id as usize].saturating_add(1);
            }
        }
    }

    let mut by_atom = vec![Vec::<WaveCoupling>::new(); graph.atom_count as usize];
    let mut by_word = vec![Vec::<WaveCoupling>::new(); words.len()];
    for (word_id, stats) in per_word.iter().enumerate() {
        let surface_count = resolved[word_id].len().max(1) as u32;
        for (atom_id, stats) in stats {
            if atom_flags[*atom_id as usize] != 0 {
                continue;
            }
            let strength = coupling_strength(
                *stats,
                surface_count,
                atom_word_support[*atom_id as usize],
                words.len(),
            );
            let position_mode = average_position(*stats);
            let phase_relation = position_phase(position_mode);
            by_atom[*atom_id as usize].push(WaveCoupling {
                peer_id: word_id as u32,
                strength,
                phase_relation,
                position_mode,
                flags: 0,
            });
        }
    }
    // Forward memory learns every observed corruption. Backward memory is the
    // clean reference configuration that the reconstruction wave must restore.
    for (word_id, surfaces) in resolved.iter().enumerate() {
        let surface_count = surfaces.len().max(1) as u32;
        let Some(clean) = surfaces.first() else {
            continue;
        };
        for (atom_id, position, _, _) in clean.atoms.iter().copied() {
            let stats = per_word[word_id].get(&atom_id).copied().unwrap_or_default();
            let strength = coupling_strength(
                stats,
                surface_count,
                atom_word_support[atom_id as usize],
                words.len(),
            );
            let position_mode = (position / 257).min(255) as u8;
            by_word[word_id].push(WaveCoupling {
                peer_id: atom_id,
                strength,
                phase_relation: position_phase(position_mode),
                position_mode,
                flags: atom_flags[atom_id as usize],
            });
        }
    }
    let forward_relations_before_policy = by_atom.iter().map(Vec::len).sum::<usize>();
    let forward_atoms_above_baseline_cap = by_atom
        .iter()
        .filter(|couplings| couplings.len() > MAX_FORWARD_COUPLINGS)
        .count();
    let max_forward_degree = by_atom.iter().map(Vec::len).max().unwrap_or_default();
    for couplings in &mut by_atom {
        couplings.sort_unstable_by(coupling_order);
        if forward_policy == ForwardPostingPolicy::BaselineBounded {
            couplings.truncate(MAX_FORWARD_COUPLINGS);
        }
    }
    for couplings in &mut by_word {
        couplings.sort_unstable_by(coupling_order);
        let anchor_count = couplings.iter().take_while(|item| item.flags != 0).count();
        couplings.truncate(anchor_count.saturating_add(MAX_REVERSE_LEXICAL_COUPLINGS));
    }
    if by_atom
        .iter()
        .any(|couplings| couplings.len() > u16::MAX as usize)
    {
        return Err(
            "complete forward postings require the scale AtomRecord with u32 counts".to_string(),
        );
    }
    let retained_forward_relations = by_atom.iter().map(Vec::len).sum::<usize>();
    let diagnostics = CompileDiagnostics {
        forward_relations_before_policy,
        forward_relations_dropped: forward_relations_before_policy
            .saturating_sub(retained_forward_relations),
        forward_atoms_above_baseline_cap,
        max_forward_degree,
    };

    let (decoder_nodes, decoder_terminals) = compile_decoder(words)?;
    let basis = compile_basis();
    let mut forward_couplings = Vec::new();
    let atoms = by_atom
        .iter()
        .enumerate()
        .map(|(atom_id, couplings)| {
            let coupling_start = forward_couplings.len() as u32;
            forward_couplings.extend_from_slice(couplings);
            AtomRecord {
                wave_code: learn_atom_code(couplings),
                coupling_start,
                coupling_count: couplings.len() as u16,
                support: atom_word_support[atom_id].min(u16::MAX as u32) as u16,
            }
        })
        .collect::<Vec<_>>();

    let mut reverse_couplings = Vec::new();
    let mut centers = Vec::with_capacity(words.len());
    for (word_id, couplings) in by_word.iter().enumerate() {
        let coupling_start = reverse_couplings.len() as u32;
        reverse_couplings.extend_from_slice(couplings);
        let mut center = WordCenter64 {
            coupling_start,
            coupling_count: couplings.len() as u16,
            crystal_support: resolved[word_id].len().min(u16::MAX as usize) as u16,
            stability: stability(resolved[word_id].len()),
            decoder_terminal: decoder_terminals[word_id],
            surface_len: words[word_id].surface.chars().count().min(u8::MAX as usize) as u8,
            flags: super::model::surface_script_flags(&words[word_id].surface),
            ..WordCenter64::default()
        };
        settle_word_code(
            &mut center,
            couplings
                .iter()
                .filter(|coupling| coupling.flags == 0)
                .flat_map(|coupling| {
                    atoms[coupling.peer_id as usize]
                        .wave_code
                        .components
                        .into_iter()
                        .map(move |component| {
                            (
                                component.basis,
                                i32::from(component.coefficient)
                                    .saturating_mul(i32::from(coupling.strength)),
                            )
                        })
                }),
        );
        centers.push(center);
    }

    let all_anti_by_word = discover_anti_centers(&resolved, &by_atom, &atoms);
    let mut anti_by_word = all_anti_by_word
        .iter()
        .map(|bank| {
            bank.iter()
                .copied()
                .filter(|center| center.crystal_support >= 2)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let (pair_profiles, pair_centers) = compile_pair_profiles(&anti_by_word);
    let mut anti_centers = Vec::new();
    for (word_id, anti) in anti_by_word.iter_mut().enumerate() {
        anti.truncate(4);
        centers[word_id].anti_start = anti_centers.len() as u32;
        centers[word_id].anti_count = anti.len().min(u8::MAX as usize) as u8;
        anti_centers.extend_from_slice(anti);
    }
    let l11 = compile_l11_subcenters(&resolved, &atoms, &all_anti_by_word);
    let restoration_calibration = calibrate_l11(
        &resolved,
        &atoms,
        &by_word,
        &basis,
        &l11.profiles,
        &l11.positive,
    );

    Ok(CompileOutput {
        package: LexicalGrokkingPackage {
            corpus_hash: corpus_hash(words),
            graph,
            basis,
            atoms,
            forward_couplings,
            reverse_couplings,
            anti_centers,
            pair_profiles,
            pair_centers,
            center_phase_profiles: l11.profiles,
            positive_subcenters: l11.positive,
            anti_subcenters: l11.anti,
            hard_negative_subcenters: l11.hard_negative,
            keyboard_geometry_units: l11.keyboard_geometry,
            restoration_calibration,
            centers,
            decoder_nodes,
        },
        diagnostics,
    })
}

fn validate_words(words: &[TrainingWord]) -> Result<(), String> {
    if words.is_empty() {
        return Err("L1 crystal compiler received no words".to_string());
    }
    for (expected, word) in words.iter().enumerate() {
        if word.terminal_id as usize != expected {
            return Err("L1 crystal terminal IDs must be dense and ordered".to_string());
        }
    }
    Ok(())
}

fn compile_graph(words: &[TrainingWord]) -> Result<NGramGraph, String> {
    let keys = words
        .iter()
        .flat_map(|word| std::iter::once(&word.surface).chain(word.training_surfaces.iter()))
        .flat_map(|surface| encode_wave_surface(surface))
        .map(|atom| atom.key)
        .collect::<BTreeSet<NGramKey>>();
    NGramGraph::compile(keys)
}

fn resolve_training_surfaces(
    graph: &NGramGraph,
    words: &[TrainingWord],
) -> Vec<Vec<ResolvedSurface>> {
    words
        .iter()
        .map(|word| {
            std::iter::once(&word.surface)
                .chain(word.training_surfaces.iter())
                .map(|surface| ResolvedSurface {
                    atoms: encode_wave_surface(surface)
                        .into_iter()
                        .filter_map(|atom| {
                            graph.atom_id(atom.key).map(|atom_id| {
                                (atom_id, atom.position, atom.weight, atom.key.channel)
                            })
                        })
                        .collect(),
                    physical_keys: physical_key_sequence(surface),
                })
                .collect()
        })
        .collect()
}

fn coupling_strength(
    stats: CouplingStats,
    surface_count: u32,
    atom_support: u32,
    word_count: usize,
) -> u8 {
    let reliability = stats.observations.saturating_mul(255) / surface_count.max(1);
    let specificity =
        ((word_count as u32 + 1).saturating_mul(32) / atom_support.max(1)).clamp(32, 255);
    ((reliability.saturating_mul(specificity) / 255).clamp(1, 255)) as u8
}

fn average_position(stats: CouplingStats) -> u8 {
    let average = stats.position_sum / u64::from(stats.observations.max(1));
    (average / 257).min(255) as u8
}

fn position_phase(position: u8) -> i8 {
    (i16::from(position) - 128).clamp(-127, 127) as i8
}

fn stability(surface_count: usize) -> u8 {
    surface_count.saturating_mul(16).min(255) as u8
}

fn coupling_order(left: &WaveCoupling, right: &WaveCoupling) -> std::cmp::Ordering {
    (right.flags != 0)
        .cmp(&(left.flags != 0))
        .then_with(|| {
            if left.flags != 0 && right.flags != 0 {
                left.position_mode.cmp(&right.position_mode)
            } else {
                right.strength.cmp(&left.strength)
            }
        })
        .then_with(|| left.peer_id.cmp(&right.peer_id))
}

fn discover_anti_centers(
    resolved: &[Vec<ResolvedSurface>],
    by_atom: &[Vec<WaveCoupling>],
    atoms: &[AtomRecord],
) -> Vec<Vec<WordCenter64>> {
    let workers = thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(resolved.len().max(1));
    let mut relations = vec![Vec::<(u32, AntiRelation)>::new(); resolved.len()];
    let next_target = AtomicUsize::new(0);
    thread::scope(|scope| {
        let (sender, receiver) = std::sync::mpsc::sync_channel(workers.saturating_mul(2));
        let handles = (0..workers)
            .map(|_| {
                let sender = sender.clone();
                let next_target = &next_target;
                scope.spawn(move || loop {
                    let target = next_target.fetch_add(1, Ordering::Relaxed);
                    let Some(surfaces) = resolved.get(target) else {
                        break;
                    };
                    let target_relations =
                        discover_target_relations(target as u32, surfaces, by_atom, atoms);
                    if sender.send((target as u32, target_relations)).is_err() {
                        return;
                    }
                })
            })
            .collect::<Vec<_>>();
        drop(sender);
        for (target, target_relations) in receiver {
            for (competitor, relation) in target_relations {
                let bank = &mut relations[competitor as usize];
                bank.push((target, relation));
                bank.sort_unstable_by(anti_relation_order);
            }
        }
        for handle in handles {
            handle.join().expect("L1 anti-center worker panicked");
        }
    });
    relations
        .into_iter()
        .map(|relations| {
            relations
                .into_iter()
                .map(|(winner, (mass, support, _))| {
                    let mut center = WordCenter64 {
                        crystal_support: support.min(u16::MAX as u32) as u16,
                        // Anti centers do not decode text; this field identifies
                        // the competing winner that activates the counter-wave.
                        decoder_terminal: winner,
                        ..WordCenter64::default()
                    };
                    settle_word_code(
                        &mut center,
                        mass.into_iter().enumerate().map(|(basis, value)| {
                            (
                                basis as u16,
                                value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
                            )
                        }),
                    );
                    center
                })
                .collect()
        })
        .collect()
}

fn compile_l11_subcenters(
    resolved: &[Vec<ResolvedSurface>],
    atoms: &[AtomRecord],
    directional_anti: &[Vec<WordCenter64>],
) -> L11Banks {
    const MAX_POSITIVE_SUBCENTERS: usize = 4;
    const MAX_ANTI_SUBCENTERS: usize = 4;
    const MAX_HARD_NEGATIVE_SUBCENTERS: usize = 2;

    let mut profiles = Vec::with_capacity(resolved.len());
    let mut positive = Vec::new();
    let mut anti = Vec::new();
    let mut hard_negative = Vec::new();
    let mut keyboard_geometry = Vec::new();
    for (terminal_id, surfaces) in resolved.iter().enumerate() {
        let positive_start = positive.len() as u32;
        let fit_surfaces = surfaces
            .iter()
            .enumerate()
            .filter(|(index, _)| surfaces.len() < 5 || index % 5 != 0)
            .map(|(_, surface)| surface.clone())
            .collect::<Vec<_>>();
        let learned_positive = cluster_positive_subcenters(
            &fit_surfaces,
            atoms,
            terminal_id as u32,
            MAX_POSITIVE_SUBCENTERS,
        );
        positive.extend_from_slice(&learned_positive);

        let anti_start = anti.len() as u32;
        anti.extend(
            directional_anti[terminal_id]
                .iter()
                .copied()
                .filter(|center| center.crystal_support >= 2)
                .take(MAX_ANTI_SUBCENTERS),
        );
        let hard_negative_start = hard_negative.len() as u32;
        hard_negative.extend(
            directional_anti[terminal_id]
                .iter()
                .copied()
                .filter(|center| center.crystal_support == 1)
                .take(MAX_HARD_NEGATIVE_SUBCENTERS),
        );
        let keyboard_geometry_start = keyboard_geometry.len() as u32;
        if let Some(clean) = surfaces.first() {
            keyboard_geometry.extend_from_slice(&clean.physical_keys);
        }
        profiles.push(CenterPhaseProfile {
            positive_start,
            anti_start,
            hard_negative_start,
            keyboard_geometry_start,
            positive_count: learned_positive.len() as u8,
            anti_count: (anti.len() as u32 - anti_start) as u8,
            hard_negative_count: (hard_negative.len() as u32 - hard_negative_start) as u8,
            keyboard_geometry_count: (keyboard_geometry.len() as u32 - keyboard_geometry_start)
                as u8,
            flags: super::model::CENTER_PHASE_FLAG_PHYSICAL_KEY_GEOMETRY,
        });
    }
    L11Banks {
        profiles,
        positive,
        anti,
        hard_negative,
        keyboard_geometry,
    }
}

fn cluster_positive_subcenters(
    surfaces: &[ResolvedSurface],
    atoms: &[AtomRecord],
    terminal_id: u32,
    limit: usize,
) -> Vec<WordCenter64> {
    let masses = surfaces
        .iter()
        .map(|surface| surface_phase_mass(surface, atoms))
        .collect::<Vec<_>>();
    if masses.is_empty() || limit == 0 {
        return Vec::new();
    }

    let mut seeds = vec![0_usize];
    while seeds.len() < limit.min(masses.len()) {
        let next = (0..masses.len())
            .filter(|candidate| !seeds.contains(candidate))
            .min_by(|left, right| {
                let left_coherence = seeds
                    .iter()
                    .map(|seed| phase_mass_coherence_milli(&masses[*left], &masses[*seed]))
                    .max()
                    .unwrap_or_default();
                let right_coherence = seeds
                    .iter()
                    .map(|seed| phase_mass_coherence_milli(&masses[*right], &masses[*seed]))
                    .max()
                    .unwrap_or_default();
                left_coherence
                    .cmp(&right_coherence)
                    .then_with(|| left.cmp(right))
            });
        let Some(next) = next else {
            break;
        };
        seeds.push(next);
    }

    let mut centroids = seeds.iter().map(|index| masses[*index]).collect::<Vec<_>>();
    let mut supports = vec![0_u32; centroids.len()];
    for _ in 0..2 {
        let mut accumulated = vec![[0_i64; WAVE_DIMENSION]; centroids.len()];
        supports.fill(0);
        for mass in &masses {
            let owner = centroids
                .iter()
                .enumerate()
                .max_by(|(left_index, left), (right_index, right)| {
                    phase_mass_coherence_milli(mass, left)
                        .cmp(&phase_mass_coherence_milli(mass, right))
                        .then_with(|| right_index.cmp(left_index))
                })
                .map(|(index, _)| index)
                .unwrap_or_default();
            supports[owner] = supports[owner].saturating_add(1);
            for (slot, value) in accumulated[owner].iter_mut().zip(mass) {
                *slot = slot.saturating_add(*value);
            }
        }
        for (centroid, (mass, support)) in centroids
            .iter_mut()
            .zip(accumulated.into_iter().zip(supports.iter().copied()))
        {
            if support > 0 {
                *centroid = mass;
            }
        }
    }

    centroids
        .into_iter()
        .zip(supports)
        .filter(|(_, support)| *support > 0)
        .map(|(mass, support)| phase_center(mass, support, terminal_id))
        .collect()
}

fn phase_center(mass: PhaseMass, support: u32, owner: u32) -> WordCenter64 {
    let mut center = WordCenter64 {
        crystal_support: support.min(u16::MAX as u32) as u16,
        decoder_terminal: owner,
        stability: stability(support as usize),
        ..WordCenter64::default()
    };
    settle_word_code(
        &mut center,
        mass.into_iter().enumerate().map(|(basis, value)| {
            (
                basis as u16,
                value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
            )
        }),
    );
    center
}

fn surface_phase_mass(surface: &ResolvedSurface, atoms: &[AtomRecord]) -> PhaseMass {
    let mut mass = [0_i64; WAVE_DIMENSION];
    for (atom_id, _, weight, channel) in &surface.atoms {
        if coupling_flag(*channel) != 0 {
            continue;
        }
        for component in atoms[*atom_id as usize].wave_code.components {
            mass[component.basis as usize] = mass[component.basis as usize].saturating_add(
                i64::from(component.coefficient).saturating_mul(i64::from(*weight)),
            );
        }
    }
    mass
}

fn phase_mass_coherence_milli(left: &PhaseMass, right: &PhaseMass) -> u16 {
    let mut dot = 0_i128;
    let mut left_mass = 0_u128;
    let mut right_mass = 0_u128;
    for (left, right) in left.iter().zip(right) {
        let left = i128::from(*left);
        let right = i128::from(*right);
        dot = dot.saturating_add(left.saturating_mul(right));
        left_mass = left_mass.saturating_add((left * left) as u128);
        right_mass = right_mass.saturating_add((right * right) as u128);
    }
    if left_mass == 0 || right_mass == 0 {
        return 0;
    }
    let denominator = integer_sqrt(left_mass.saturating_mul(right_mass)).max(1) as i128;
    ((dot.saturating_mul(500) / denominator) + 500).clamp(0, 1_000) as u16
}

fn integer_sqrt(value: u128) -> u128 {
    if value < 2 {
        return value;
    }
    let mut estimate = 1_u128 << ((128 - value.leading_zeros() as usize).div_ceil(2));
    loop {
        let next = (estimate + value / estimate) / 2;
        if next >= estimate {
            return estimate;
        }
        estimate = next;
    }
}

fn calibrate_l11(
    resolved: &[Vec<ResolvedSurface>],
    atoms: &[AtomRecord],
    reverse: &[Vec<WaveCoupling>],
    basis: &[super::crystal::ComplexBasisWave],
    profiles: &[CenterPhaseProfile],
    positive_subcenters: &[WordCenter64],
) -> RestorationCalibration {
    let mut distances = Vec::new();
    let mut positive = Vec::new();
    let mut backward = Vec::new();
    for (terminal_id, surfaces) in resolved.iter().enumerate() {
        let expected_character_sequence = surfaces
            .first()
            .map(|surface| anchor_sequence(surface, AtomChannel::CharacterAnchor))
            .unwrap_or_default();
        let expected_keyboard_sequence = surfaces
            .first()
            .map(|surface| surface.physical_keys.clone())
            .unwrap_or_default();
        let profile = profiles[terminal_id];
        let positive_bank = positive_subcenters
            .get(
                profile.positive_start as usize
                    ..profile.positive_start as usize + profile.positive_count as usize,
            )
            .unwrap_or_default();
        for surface in surfaces
            .iter()
            .enumerate()
            .filter_map(|(index, surface)| (index % 5 == 0).then_some(surface))
        {
            let observed_character_sequence =
                anchor_sequence(surface, AtomChannel::CharacterAnchor);
            let observed_keyboard_sequence = &surface.physical_keys;
            distances.push(
                super::runtime::damerau_distance(
                    &observed_character_sequence,
                    &expected_character_sequence,
                )
                .min(super::runtime::damerau_distance(
                    observed_keyboard_sequence,
                    &expected_keyboard_sequence,
                ))
                .min(u8::MAX as usize) as u8,
            );
            let (surface_re, surface_im) = expanded_surface_wave(surface, atoms, basis);
            positive.push(
                positive_bank
                    .iter()
                    .map(|center| {
                        let (center_re, center_im) = expand_word(basis, *center);
                        complex_coherence_milli(&surface_re, &surface_im, &center_re, &center_im)
                    })
                    .max()
                    .unwrap_or_default(),
            );
            backward.push(backward_coherence_milli(surface, &reverse[terminal_id]));
        }
    }
    RestorationCalibration {
        max_geometry_distance: high_quantile(&mut distances, 999),
        min_positive_milli: low_quantile(&mut positive, 0),
        min_backward_milli: low_quantile(&mut backward, 0),
    }
}

fn anchor_sequence(surface: &ResolvedSurface, selected_channel: AtomChannel) -> Vec<u32> {
    let mut anchors = surface
        .atoms
        .iter()
        .filter(|(_, _, _, channel)| *channel == selected_channel)
        .map(|(atom_id, position, _, _)| (*position, *atom_id))
        .collect::<Vec<_>>();
    anchors.sort_unstable();
    anchors
        .into_iter()
        .take(32)
        .map(|(_, atom_id)| atom_id)
        .collect()
}

fn expanded_surface_wave(
    surface: &ResolvedSurface,
    atoms: &[AtomRecord],
    basis: &[super::crystal::ComplexBasisWave],
) -> ([i32; WAVE_DIMENSION], [i32; WAVE_DIMENSION]) {
    let mut re = [0_i32; WAVE_DIMENSION];
    let mut im = [0_i32; WAVE_DIMENSION];
    for (atom_id, _, weight, channel) in &surface.atoms {
        if coupling_flag(*channel) == 0 {
            expand_atom(
                basis,
                atoms[*atom_id as usize].wave_code,
                &mut re,
                &mut im,
                i32::from(*weight),
            );
        }
    }
    (re, im)
}

fn backward_coherence_milli(surface: &ResolvedSurface, reverse: &[WaveCoupling]) -> u16 {
    let observed = surface
        .atoms
        .iter()
        .map(|(atom_id, position, _, _)| (*atom_id, ((*position / 257).min(255)) as u8))
        .collect::<BTreeMap<_, _>>();
    let lexical = reverse.iter().filter(|coupling| coupling.flags == 0);
    let expected = lexical
        .clone()
        .map(|coupling| u64::from(coupling.strength))
        .sum::<u64>()
        .max(1);
    let observed_mass = lexical
        .filter_map(|coupling| {
            let position = observed.get(&coupling.peer_id)?;
            let coherence =
                256_u16.saturating_sub(u16::from(position.abs_diff(coupling.position_mode)));
            Some(u64::from(coupling.strength).saturating_mul(u64::from(coherence)))
        })
        .sum::<u64>();
    (observed_mass.saturating_mul(1_000) / expected.saturating_mul(256)).min(1_000) as u16
}

fn high_quantile<T: Ord + Copy + Default>(values: &mut [T], permille: usize) -> T {
    if values.is_empty() {
        return T::default();
    }
    values.sort_unstable();
    values[(values.len() - 1).saturating_mul(permille.min(1_000)) / 1_000]
}

fn low_quantile<T: Ord + Copy + Default>(values: &mut [T], permille: usize) -> T {
    high_quantile(values, permille)
}

fn compile_pair_profiles(
    directional_by_loser: &[Vec<WordCenter64>],
) -> (Vec<PairPhaseProfile>, Vec<WordCenter64>) {
    let mut banks = BTreeMap::<PairKey, (Vec<WordCenter64>, Vec<WordCenter64>)>::new();
    for (loser, directional) in directional_by_loser.iter().enumerate() {
        let loser = loser as u32;
        for center in directional {
            let winner = center.decoder_terminal;
            let Some(key) = PairKey::new(loser, winner) else {
                continue;
            };
            let banks = banks.entry(key).or_default();
            if winner == key.low_terminal {
                banks.0.push(*center);
            } else {
                banks.1.push(*center);
            }
        }
    }

    let mut profiles = Vec::with_capacity(banks.len());
    let mut centers = Vec::new();
    for (key, (mut low_wins, mut high_wins)) in banks {
        low_wins.sort_unstable_by(pair_center_order);
        high_wins.sort_unstable_by(pair_center_order);
        // One aggregate center currently represents each learned directional
        // mode. The count fields preserve a bounded subcenter extension point.
        low_wins.truncate(1);
        high_wins.truncate(1);
        let low_wins_start = centers.len() as u32;
        centers.extend_from_slice(&low_wins);
        let high_wins_start = centers.len() as u32;
        centers.extend_from_slice(&high_wins);
        profiles.push(PairPhaseProfile {
            key,
            low_wins_start,
            high_wins_start,
            low_wins_count: low_wins.len() as u16,
            high_wins_count: high_wins.len() as u16,
        });
    }
    (profiles, centers)
}

fn pair_center_order(left: &WordCenter64, right: &WordCenter64) -> std::cmp::Ordering {
    right
        .crystal_support
        .cmp(&left.crystal_support)
        .then_with(|| left.decoder_terminal.cmp(&right.decoder_terminal))
}

fn anti_relation_order(
    left: &(u32, AntiRelation),
    right: &(u32, AntiRelation),
) -> std::cmp::Ordering {
    right
        .1
         .1
        .cmp(&left.1 .1)
        .then_with(|| right.1 .2.cmp(&left.1 .2))
        .then_with(|| left.0.cmp(&right.0))
}

fn discover_target_relations(
    target: u32,
    surfaces: &[ResolvedSurface],
    by_atom: &[Vec<WaveCoupling>],
    atoms: &[AtomRecord],
) -> Vec<(u32, AntiRelation)> {
    let mut relations = BTreeMap::new();
    for surface in surfaces.iter().skip(1) {
        let scores = forward_scores(surface, by_atom);
        let Some(target_score) = scores.get(&target).copied() else {
            continue;
        };
        let mut competitors = scores
            .into_iter()
            .filter(|(word, score)| *word != target && *score >= target_score)
            .collect::<Vec<_>>();
        competitors.sort_unstable_by(|left, right| {
            right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0))
        });
        for (competitor, score) in competitors.into_iter().take(4) {
            let relation = relations.entry(competitor).or_insert((
                [0_i64; super::crystal::WAVE_DIMENSION],
                0_u32,
                0_u64,
            ));
            relation.1 = relation.1.saturating_add(1);
            relation.2 = relation.2.saturating_add(u64::from(score));
            for (atom_id, _, weight, channel) in &surface.atoms {
                if coupling_flag(*channel) != 0 {
                    continue;
                }
                for component in atoms[*atom_id as usize].wave_code.components {
                    relation.0[component.basis as usize] = relation.0[component.basis as usize]
                        .saturating_add(
                            i64::from(component.coefficient).saturating_mul(i64::from(*weight)),
                        );
                }
            }
        }
    }
    relations.into_iter().collect()
}

fn forward_scores(surface: &ResolvedSurface, by_atom: &[Vec<WaveCoupling>]) -> BTreeMap<u32, u32> {
    let mut scores = BTreeMap::new();
    for (atom_id, position, weight, _) in surface.atoms.iter().copied() {
        let observed = (position / 257).min(255) as u8;
        for coupling in &by_atom[atom_id as usize] {
            let position_factor =
                256_u32.saturating_sub(u32::from(observed.abs_diff(coupling.position_mode)));
            let contribution = u32::from(coupling.strength)
                .saturating_mul(u32::from(weight))
                .saturating_mul(position_factor);
            *scores.entry(coupling.peer_id).or_default() += contribution;
        }
    }
    scores
}

fn coupling_flag(channel: AtomChannel) -> u8 {
    match channel {
        AtomChannel::CharacterAnchor => COUPLING_FLAG_CHARACTER_ANCHOR,
        _ => 0,
    }
}

fn compile_decoder(words: &[TrainingWord]) -> Result<(Vec<DecoderNode>, Vec<u32>), String> {
    let mut nodes = vec![DecoderNode {
        parent: u32::MAX,
        symbol: 0,
    }];
    let mut children = vec![BTreeMap::<u32, u32>::new()];
    let mut terminals = Vec::with_capacity(words.len());
    for word in words {
        let mut node = 0_u32;
        for symbol in word.surface.chars().map(|ch| ch as u32) {
            let next = if let Some(next) = children[node as usize].get(&symbol) {
                *next
            } else {
                let next = u32::try_from(nodes.len())
                    .map_err(|_| "decoder node count exceeds u32".to_string())?;
                nodes.push(DecoderNode {
                    parent: node,
                    symbol,
                });
                children.push(BTreeMap::new());
                children[node as usize].insert(symbol, next);
                next
            };
            node = next;
        }
        terminals.push(node);
    }
    Ok((nodes, terminals))
}

fn corpus_hash(words: &[TrainingWord]) -> u64 {
    let mut state = 0x4c31_4352_5953_3032_u64;
    for word in words {
        for byte in word.surface.as_bytes() {
            state = mix64_golden(state ^ u64::from(*byte));
        }
        state = mix64_golden(state ^ u64::from(word.terminal_id));
    }
    state
}
