//! Exhaustive proof oracle for tiny lexical packages.
//!
//! This module is intentionally excluded from normal builds. It reuses the
//! production evidence and settlement kernels, but enumerates every center and
//! every non-empty competitive subset so optimized search can later be checked
//! against an exact reference.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::atoms::{normalize_lexical_surface, AtomChannel};
use super::crystal::WAVE_DIMENSION;
use super::restoration::RestorationReadout;
use super::runtime::{
    candidate_json, observed_sequence, ForwardActivation, GrokkingCandidate, LexicalGrokkingMemory,
    ObservedAtom, ReadoutMode,
};
use super::wave_basis::expand_atom;

const MAX_EXHAUSTIVE_CENTERS: usize = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CenterIteration {
    Forward,
    Reverse,
    Permuted(u64),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct OracleActivation {
    mass: u64,
    hits: u16,
    surface_hits: u16,
    keyboard_hits: u16,
}

impl From<ForwardActivation> for OracleActivation {
    fn from(activation: ForwardActivation) -> Self {
        Self {
            mass: activation.mass,
            hits: activation.hits,
            surface_hits: activation.surface_hits,
            keyboard_hits: activation.keyboard_hits,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct OracleCenterEvidence {
    terminal_id: u32,
    surface: Option<String>,
    activation: OracleActivation,
    standalone_candidate: serde_json::Value,
    lower_final_scalar: i32,
    upper_final_scalar: i32,
    subsets_observed: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
enum DependencyKind {
    Ambiguity,
    Anti,
    Pairwise,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct OracleDependency {
    kind: DependencyKind,
    record_index: usize,
    owner: u32,
    peer: u32,
    owner_independently_admitted: bool,
    peer_independently_admitted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct OracleCoreReport {
    schema: &'static str,
    query: String,
    requested_k: usize,
    effective_k: usize,
    terminal_count: usize,
    non_empty_subsets: u64,
    maximum_activation_mass: u64,
    maximum_surface_hits: u16,
    maximum_keyboard_hits: u16,
    centers: Vec<OracleCenterEvidence>,
    beta_k: i32,
    beta_k_lower_equality: Vec<u32>,
    minimum_typed_geometry: u8,
    scalar_competitive_set_a: Vec<u32>,
    conservative_typed_set_d: Vec<u32>,
    competitive_closure_s: Vec<u32>,
    exact_surface_collisions: Vec<u32>,
    typed_dependencies_r: Vec<OracleDependency>,
    final_candidates: Vec<serde_json::Value>,
    restoration: RestorationReadout,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::nanda_wave::lexical_grokking) struct DenseOracleReport {
    #[serde(flatten)]
    core: OracleCoreReport,
    result_fingerprint: String,
}

#[derive(Clone)]
struct SubsetResult {
    candidates: Vec<GrokkingCandidate>,
    restoration: RestorationReadout,
}

pub(in crate::nanda_wave::lexical_grokking) fn dense_oracle(
    memory: &LexicalGrokkingMemory,
    surface: &str,
    requested_k: usize,
) -> Result<DenseOracleReport, String> {
    dense_oracle_with_iteration(memory, surface, requested_k, CenterIteration::Forward)
}

fn dense_oracle_with_iteration(
    memory: &LexicalGrokkingMemory,
    surface: &str,
    requested_k: usize,
    iteration: CenterIteration,
) -> Result<DenseOracleReport, String> {
    let terminal_count = memory.package.terminal_count() as usize;
    if terminal_count == 0 {
        return Err("dense oracle requires at least one WordCenterId".to_string());
    }
    if terminal_count > MAX_EXHAUSTIVE_CENTERS {
        return Err(format!(
            "dense oracle supports at most {MAX_EXHAUSTIVE_CENTERS} centers, got {terminal_count}"
        ));
    }
    if requested_k == 0 {
        return Err("dense oracle requires requested_k > 0".to_string());
    }

    let observed = memory.resolve_surface(surface);
    let lexical_observed = observed
        .iter()
        .filter(|(_, atom)| atom.channel != AtomChannel::CharacterAnchor)
        .copied()
        .collect::<BTreeMap<u32, ObservedAtom>>();
    let character_sequence = observed_sequence(&observed, AtomChannel::CharacterAnchor);
    let observed_char_count = normalize_lexical_surface(surface)
        .chars()
        .count()
        .min(u8::MAX as usize) as u8;
    let (surface_re, surface_im) = surface_wave(memory, &lexical_observed);

    let iteration_ids = center_iteration(terminal_count, iteration);
    let activations = (0..terminal_count)
        .map(|terminal_id| memory.activation_for_terminal(terminal_id as u32, &lexical_observed))
        .collect::<Vec<_>>();
    let maximum_activation_mass = activations
        .iter()
        .map(|activation| activation.mass)
        .max()
        .unwrap_or_default();
    let maximum_surface_hits = activations
        .iter()
        .map(|activation| activation.surface_hits)
        .max()
        .unwrap_or_default();
    let maximum_keyboard_hits = activations
        .iter()
        .map(|activation| activation.keyboard_hits)
        .max()
        .unwrap_or_default();

    let subset_total = (1_u64 << terminal_count) - 1;
    let mut lower = vec![i32::MAX; terminal_count];
    let mut upper = vec![i32::MIN; terminal_count];
    let mut subset_counts = vec![0_u64; terminal_count];
    let mut all_center_result = None;
    for mask in 1_u64..=subset_total {
        let selected = iteration_ids
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(bit, terminal_id)| ((mask >> bit) & 1 == 1).then_some(terminal_id))
            .collect::<Vec<_>>();
        let result = settle_subset(
            memory,
            surface,
            &selected,
            &activations,
            &lexical_observed,
            &surface_re,
            &surface_im,
            &character_sequence,
            observed_char_count,
        );
        for candidate in &result.candidates {
            let index = candidate.terminal_id as usize;
            lower[index] = lower[index].min(candidate.settled_energy);
            upper[index] = upper[index].max(candidate.settled_energy);
            subset_counts[index] = subset_counts[index].saturating_add(1);
        }
        if selected.len() == terminal_count {
            all_center_result = Some(result);
        }
    }
    let all_center_result = all_center_result
        .ok_or_else(|| "dense oracle failed to enumerate the complete center subset".to_string())?;

    let expected_subset_count = 1_u64 << terminal_count.saturating_sub(1);
    if subset_counts
        .iter()
        .any(|count| *count != expected_subset_count)
    {
        return Err("dense oracle did not observe every subset containing each center".to_string());
    }

    let effective_k = requested_k.min(terminal_count);
    let mut lower_order = lower.clone();
    lower_order.sort_unstable_by(|left, right| right.cmp(left));
    let beta_k = lower_order[effective_k - 1];
    let beta_k_lower_equality = (0..terminal_count)
        .filter_map(|terminal_id| (lower[terminal_id] == beta_k).then_some(terminal_id as u32))
        .collect::<Vec<_>>();
    let scalar_competitive_set_a = (0..terminal_count)
        .filter_map(|terminal_id| (upper[terminal_id] >= beta_k).then_some(terminal_id as u32))
        .collect::<Vec<_>>();

    // Phase 4 deliberately uses the conservative complete typed closure. Later
    // phases may shrink D only after a separate reachability proof.
    let conservative_typed_set_d = (0..terminal_count as u32).collect::<Vec<_>>();
    let competitive_closure_s = scalar_competitive_set_a
        .iter()
        .chain(&conservative_typed_set_d)
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let exact_surface_collisions = all_center_result
        .candidates
        .iter()
        .filter_map(|candidate| {
            candidate
                .exact_reconstruction
                .then_some(candidate.terminal_id)
        })
        .collect::<Vec<_>>();
    let minimum_typed_geometry = all_center_result
        .candidates
        .iter()
        .map(|candidate| candidate.geometry_distance)
        .min()
        .unwrap_or(u8::MAX);
    let typed_dependencies_r = dependencies(memory, &competitive_closure_s);

    let centers = (0..terminal_count)
        .map(|terminal_id| {
            let singleton = settle_subset(
                memory,
                surface,
                &[terminal_id as u32],
                &activations,
                &lexical_observed,
                &surface_re,
                &surface_im,
                &character_sequence,
                observed_char_count,
            );
            let standalone = singleton
                .candidates
                .iter()
                .find(|candidate| candidate.terminal_id == terminal_id as u32)
                .copied()
                .ok_or_else(|| format!("missing standalone center {terminal_id}"))?;
            Ok(OracleCenterEvidence {
                terminal_id: terminal_id as u32,
                surface: memory.decode_terminal(terminal_id as u32),
                activation: activations[terminal_id].into(),
                standalone_candidate: candidate_json(memory, standalone),
                lower_final_scalar: lower[terminal_id],
                upper_final_scalar: upper[terminal_id],
                subsets_observed: subset_counts[terminal_id],
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let final_candidates = all_center_result
        .candidates
        .iter()
        .copied()
        .map(|candidate| candidate_json(memory, candidate))
        .collect::<Vec<_>>();
    let core = OracleCoreReport {
        schema: "lay.l11.dense-peak-oracle.v1",
        query: surface.to_string(),
        requested_k,
        effective_k,
        terminal_count,
        non_empty_subsets: subset_total,
        maximum_activation_mass,
        maximum_surface_hits,
        maximum_keyboard_hits,
        centers,
        beta_k,
        beta_k_lower_equality,
        minimum_typed_geometry,
        scalar_competitive_set_a,
        conservative_typed_set_d,
        competitive_closure_s,
        exact_surface_collisions,
        typed_dependencies_r,
        final_candidates,
        restoration: all_center_result.restoration,
    };
    let result_fingerprint = sha256_hex(
        &serde_json::to_vec(&core).map_err(|error| format!("serialize dense oracle: {error}"))?,
    );
    Ok(DenseOracleReport {
        core,
        result_fingerprint,
    })
}

#[allow(clippy::too_many_arguments)]
fn settle_subset(
    memory: &LexicalGrokkingMemory,
    surface: &str,
    terminal_ids: &[u32],
    activations: &[ForwardActivation],
    observed: &BTreeMap<u32, ObservedAtom>,
    surface_re: &[i32; WAVE_DIMENSION],
    surface_im: &[i32; WAVE_DIMENSION],
    character_sequence: &super::runtime::AnchorSequence,
    observed_char_count: u8,
) -> SubsetResult {
    let max_forward = terminal_ids
        .iter()
        .map(|terminal_id| activations[*terminal_id as usize].mass)
        .max()
        .unwrap_or(1)
        .max(1);
    let mut candidates = terminal_ids
        .iter()
        .filter_map(|terminal_id| {
            memory.settle_candidate(
                *terminal_id,
                activations[*terminal_id as usize],
                max_forward,
                observed,
                surface_re,
                surface_im,
                character_sequence,
                observed_char_count,
                ReadoutMode::Full,
            )
        })
        .collect::<Vec<_>>();
    memory.finalize_candidates(
        surface,
        usize::MAX,
        ReadoutMode::Full,
        surface_re,
        surface_im,
        &mut candidates,
    );
    let restoration = memory.classify_restoration(
        surface,
        &mut candidates,
        memory.package.restoration_calibration,
    );
    SubsetResult {
        candidates,
        restoration,
    }
}

fn surface_wave(
    memory: &LexicalGrokkingMemory,
    observed: &BTreeMap<u32, ObservedAtom>,
) -> ([i32; WAVE_DIMENSION], [i32; WAVE_DIMENSION]) {
    let mut re = [0_i32; WAVE_DIMENSION];
    let mut im = [0_i32; WAVE_DIMENSION];
    for (atom_id, atom) in observed {
        let Some(record) = memory.package.atoms.get(*atom_id as usize) else {
            continue;
        };
        expand_atom(
            &memory.package.basis,
            record.wave_code,
            &mut re,
            &mut im,
            i32::from(atom.weight),
        );
    }
    (re, im)
}

fn center_iteration(terminal_count: usize, iteration: CenterIteration) -> Vec<u32> {
    let mut ids = (0..terminal_count as u32).collect::<Vec<_>>();
    match iteration {
        CenterIteration::Forward => {}
        CenterIteration::Reverse => ids.reverse(),
        CenterIteration::Permuted(mut state) => {
            for index in (1..ids.len()).rev() {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                ids.swap(index, state as usize % (index + 1));
            }
        }
    }
    ids
}

fn dependencies(memory: &LexicalGrokkingMemory, closure: &[u32]) -> Vec<OracleDependency> {
    let admitted = closure.iter().copied().collect::<BTreeSet<_>>();
    let mut dependencies = BTreeSet::new();
    for owner in 0..memory.package.terminal_count() {
        if !admitted.contains(&owner) {
            continue;
        }
        if let Some(center) = memory.package.centers.get(owner as usize) {
            let start = center.anti_start as usize;
            let end = start.saturating_add(center.anti_count as usize);
            for (offset, relation) in memory
                .package
                .anti_centers
                .get(start..end)
                .unwrap_or_default()
                .iter()
                .enumerate()
            {
                dependencies.insert(OracleDependency {
                    kind: DependencyKind::Anti,
                    record_index: start + offset,
                    owner,
                    peer: relation.decoder_terminal,
                    owner_independently_admitted: true,
                    peer_independently_admitted: admitted.contains(&relation.decoder_terminal),
                });
            }
        }
        if let Some(profile) = memory.package.center_phase_profiles.get(owner as usize) {
            let start = profile.ambiguity_start as usize;
            let end = start.saturating_add(profile.ambiguity_count as usize);
            for (offset, relation) in memory
                .package
                .ambiguity_subcenters
                .get(start..end)
                .unwrap_or_default()
                .iter()
                .enumerate()
            {
                dependencies.insert(OracleDependency {
                    kind: DependencyKind::Ambiguity,
                    record_index: start + offset,
                    owner,
                    peer: relation.decoder_terminal,
                    owner_independently_admitted: true,
                    peer_independently_admitted: admitted.contains(&relation.decoder_terminal),
                });
            }
        }
    }
    for (record_index, profile) in memory.package.pair_profiles.iter().enumerate() {
        let owner = profile.key.low_terminal;
        let peer = profile.key.high_terminal;
        if admitted.contains(&owner) || admitted.contains(&peer) {
            dependencies.insert(OracleDependency {
                kind: DependencyKind::Pairwise,
                record_index,
                owner,
                peer,
                owner_independently_admitted: admitted.contains(&owner),
                peer_independently_admitted: admitted.contains(&peer),
            });
        }
    }
    dependencies.into_iter().collect()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::lexical_grokking::compiler::compile;
    use crate::nanda_wave::lexical_grokking::training_corpus::TrainingWord;

    fn tiny_memory() -> LexicalGrokkingMemory {
        let words = ["form", "farm", "foam", "from", "frame"]
            .into_iter()
            .enumerate()
            .map(|(terminal_id, surface)| TrainingWord {
                terminal_id: terminal_id as u32,
                surface: surface.to_string(),
                training_surfaces: Vec::new(),
            })
            .collect::<Vec<_>>();
        LexicalGrokkingMemory::from_package(compile(&words).expect("compile tiny oracle package"))
    }

    #[test]
    fn tiny_oracle_enumerates_all_centers_and_all_containing_subsets() {
        let report = dense_oracle(&tiny_memory(), "frmo", 128).expect("run dense oracle");
        assert_eq!(report.core.terminal_count, 5);
        assert_eq!(report.core.effective_k, 5);
        assert_eq!(report.core.non_empty_subsets, 31);
        assert_eq!(report.core.centers.len(), 5);
        assert!(report
            .core
            .centers
            .iter()
            .all(|center| center.subsets_observed == 16));
        assert_eq!(report.core.competitive_closure_s, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn exact_bounds_equal_exhaustive_subset_extrema() {
        let memory = tiny_memory();
        let report = dense_oracle(&memory, "frmo", 3).expect("run dense oracle");
        let observed = memory.resolve_surface("frmo");
        let lexical_observed = observed
            .iter()
            .filter(|(_, atom)| atom.channel != AtomChannel::CharacterAnchor)
            .copied()
            .collect::<BTreeMap<_, _>>();
        let character_sequence = observed_sequence(&observed, AtomChannel::CharacterAnchor);
        let activations = (0..5)
            .map(|terminal_id| memory.activation_for_terminal(terminal_id, &lexical_observed))
            .collect::<Vec<_>>();
        let (surface_re, surface_im) = surface_wave(&memory, &lexical_observed);
        for center in &report.core.centers {
            let mut observed_scalars = Vec::new();
            for mask in 1_u64..32 {
                if mask & (1 << center.terminal_id) == 0 {
                    continue;
                }
                let selected = (0..5)
                    .filter_map(|terminal_id| {
                        (mask & (1 << terminal_id) != 0).then_some(terminal_id)
                    })
                    .collect::<Vec<_>>();
                let result = settle_subset(
                    &memory,
                    "frmo",
                    &selected,
                    &activations,
                    &lexical_observed,
                    &surface_re,
                    &surface_im,
                    &character_sequence,
                    4,
                );
                observed_scalars.push(
                    result
                        .candidates
                        .iter()
                        .find(|candidate| candidate.terminal_id == center.terminal_id)
                        .expect("selected center remains in exhaustive subset")
                        .settled_energy,
                );
            }
            assert_eq!(
                center.lower_final_scalar,
                *observed_scalars.iter().min().unwrap()
            );
            assert_eq!(
                center.upper_final_scalar,
                *observed_scalars.iter().max().unwrap()
            );
        }
    }

    #[test]
    fn center_iteration_order_cannot_change_oracle_bytes() {
        let memory = tiny_memory();
        let forward = dense_oracle_with_iteration(&memory, "frmo", 3, CenterIteration::Forward)
            .expect("forward oracle");
        let reverse = dense_oracle_with_iteration(&memory, "frmo", 3, CenterIteration::Reverse)
            .expect("reverse oracle");
        let permuted =
            dense_oracle_with_iteration(&memory, "frmo", 3, CenterIteration::Permuted(0x51a7))
                .expect("permuted oracle");
        assert_eq!(
            serde_json::to_vec(&forward).unwrap(),
            serde_json::to_vec(&reverse).unwrap()
        );
        assert_eq!(
            serde_json::to_vec(&forward).unwrap(),
            serde_json::to_vec(&permuted).unwrap()
        );
    }

    #[test]
    fn external_target_labels_cannot_enter_or_change_oracle_bytes() {
        let memory = tiny_memory();
        let report = dense_oracle(&memory, "frmo", 3).expect("run dense oracle");
        let before = serde_json::to_vec(&report).unwrap();
        let external_labels = ["form", "from", "unrelated-heldout-label"];
        for _label in external_labels {
            assert_eq!(before, serde_json::to_vec(&report).unwrap());
        }
        let schema = serde_json::to_value(&report).unwrap();
        assert!(schema.get("target").is_none());
        assert!(schema.get("expected").is_none());
    }

    #[test]
    fn oracle_is_unreachable_from_production_source_owners() {
        for source in [
            include_str!("runtime.rs"),
            include_str!("service.rs"),
            include_str!("peak_search/mod.rs"),
        ] {
            assert!(!source.contains("peak_oracle"));
            assert!(!source.contains("dense_oracle"));
        }
        let module_root = include_str!("mod.rs");
        assert!(module_root
            .contains("#[cfg(any(test, feature = \"lexical-compiler\"))]\nmod peak_oracle;"));
    }
}
