//! Compact learned context relation memory for L3.
//!
//! Cold text is compiled into token-state centers and candidate-specific
//! context centers. The hot package stores hashes, quantized phase vectors,
//! support and learned thresholds; it stores no raw phrase or word strings.

mod compiler;
mod composite;
mod format;
mod online;
mod proof;
mod sentence;
mod stream;
mod surface_field;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};
use std::{env, io};

use super::phase_field::{
    add_hashed_atom, add_phase_vector, add_rotated_vector, empty_vector, hash_text,
    phase_center_from_sum, phase_micro, vector_phase_coherence, PhaseCell, PhaseCenter,
};
use crate::lexical_surface_atoms::{surface_atom_projection, SurfaceFieldEncoder};
use crate::stable_hash::mix64_golden;
use sha2::{Digest, Sha256};

pub(crate) use compiler::{
    apply_feedback_overlay, build_feedback_corpus,
    compile_context_phase_delta_reader_with_projection_base, compile_context_phase_reader,
    compile_context_phase_reader_with_surface_field, surface_field_from_corrections_path,
};
#[cfg(test)]
pub(crate) use compiler::{compile_context_phase, ContextPhaseCompileInput};
pub(crate) use composite::{
    admit_delta, admit_delta_with_full_proof, compact_manifest, initialize_manifest,
    snapshot_manifest, snapshot_manifest_with_delta, L3CompositeMemory,
};
pub(crate) use format::{encode_package, read_package, write_package};
pub(crate) use proof::build_and_prove_context_phase_path_with_surface_field;
pub(crate) use proof::{
    build_and_prove_context_phase_path, prove_context_phase_package_delta,
    prove_context_phase_package_delta_path, prove_context_phase_package_path,
    prove_context_phase_package_path_with_surface_field, prove_context_phase_path,
};
pub(crate) use sentence::{
    build_and_prove_sentence_context_path, compile_supervised_relation_delta,
    prove_sentence_context_delta_path,
};
pub(crate) use surface_field::SurfaceMutationField;

pub(crate) const MAGIC: &[u8; 8] = b"LAYL3P01";
pub(crate) const CELLS: usize = 64;
pub(crate) const MAX_CONTEXT_TOKENS: usize = 16;
pub(crate) const MAX_CONTEXT_ATOMS: usize = MAX_CONTEXT_TOKENS * 2;
const MAX_PAIR_CANDIDATES: usize = 8;
pub(super) const SIGNATURE_SCHEMA_LEGACY: u32 = 1;
const SIGNATURE_SCHEMA_MORPHOLOGY_ENDING: u32 = 2;
pub(super) const SIGNATURE_SCHEMA_MORPHOLOGY_PHASE: u32 = 3;
pub(super) const SIGNATURE_SCHEMA_RELATION_ROLES: u32 = 4;
// Pairwise memory is keyed by compact hashes and bounded phase banks, never
// text. One candidate pair can be valid in several incompatible sentence
// scenes, so each direction retains a small multimodal attractor bank.
pub(super) const MAX_EXACT_PAIR_PROFILES: usize = 65_536;
pub(super) const MAX_RELATION_PAIR_PROFILES: usize = 16_384;
pub(crate) const MAX_PAIR_PROFILES: usize = MAX_EXACT_PAIR_PROFILES + MAX_RELATION_PAIR_PROFILES;
/// Signature profiles are compact L2-center classes, not lexical entries.
/// They let L3 transfer context evidence across words with the same observed
/// surface state while keeping exact word profiles as the only support owner.
pub(crate) const MAX_SIGNATURE_PROFILES: usize = 16_384;
pub(crate) const MAX_PAIR_CENTERS_PER_BANK: usize = 16;
pub(crate) const MAX_HARD_PAIR_CENTERS_PER_BANK: usize = 4;
const PAIR_CENTER_SPLIT_COHERENCE: f32 = 0.76;
const MIN_DIRECTIONAL_PAIR_SUPPORT: u32 = 2;

pub(crate) fn package_sha256(package: &ContextPhasePackage) -> String {
    sha256_hex(&encode_package(package))
}

pub(crate) fn package_path_sha256(path: &Path) -> io::Result<String> {
    Ok(sha256_hex(&std::fs::read(path)?))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn semantic_relation_weights(support: u32) -> (f32, f32) {
    if support < 2 {
        return (1.0, 0.0);
    }
    (1.0, 0.80)
}

fn candidate_semantic_relation_weight(support: u32) -> f32 {
    if support < 2 {
        0.0
    } else {
        0.85
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TokenSemanticState {
    pub(crate) token_hash: u64,
    pub(crate) support: u32,
    pub(crate) center: Vec<PhaseCell>,
}

#[derive(Clone, Debug)]
pub(crate) struct ContextCandidateProfile {
    pub(crate) token_hash: u64,
    pub(crate) positive_examples: u32,
    pub(crate) negative_examples: u32,
    pub(crate) threshold_micro: i32,
    pub(crate) positive: Vec<PhaseCenter>,
    pub(crate) negative: Vec<PhaseCenter>,
    /// Candidate-specific destructive centers mined only from observed L2
    /// false winners. Keeping them separate prevents broad lexical negatives
    /// from averaging away a precise wrong-attractor phase mode.
    pub(crate) hard_negative: Vec<PhaseCenter>,
}

/// Directed learned competition for one canonical L2 candidate pair. The
/// hashes are sorted only for storage; the two banks preserve who wins.
#[derive(Clone, Debug, Default)]
pub(crate) struct ContextPairPhaseProfile {
    pub(crate) low_hash: u64,
    pub(crate) high_hash: u64,
    pub(crate) low_wins: Vec<PhaseCenter>,
    pub(crate) high_wins: Vec<PhaseCenter>,
    pub(crate) hard_low_wins: Vec<PhaseCenter>,
    pub(crate) hard_high_wins: Vec<PhaseCenter>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct PairKey {
    pub(crate) low_hash: u64,
    pub(crate) high_hash: u64,
}

impl PairKey {
    pub(crate) fn new(left: u64, right: u64) -> Option<Self> {
        (left != right).then(|| Self {
            low_hash: left.min(right),
            high_hash: left.max(right),
        })
    }

    /// A generalized pair has no lexical identity in its key. Its directional
    /// banks are ordered by L2 signature, never by a word hash: a new lexical
    /// pair may carry the same two L2 roles in the opposite hash order.
    pub(crate) fn relation(
        left_hash: u64,
        left_signature: u64,
        right_hash: u64,
        right_signature: u64,
    ) -> Option<Self> {
        (left_hash != right_hash && left_signature != right_signature).then(|| {
            let low_signature = left_signature.min(right_signature);
            let high_signature = left_signature.max(right_signature);
            let relation = mix64_golden(
                low_signature ^ high_signature.rotate_left(29) ^ 0x0050_4149_5252_454c,
            )
            .max(1);
            Self {
                low_hash: 0,
                high_hash: relation,
            }
        })
    }

    fn is_relation(self) -> bool {
        self.low_hash == 0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PairEdgeOutcome {
    #[default]
    Unknown,
    Tie,
    LowWins,
    HighWins,
    Conflict,
}

#[derive(Clone, Copy, Debug, Default)]
struct PairEdgeEvidence {
    outcome: PairEdgeOutcome,
    confidence: f32,
}

#[derive(Default)]
struct PairwiseDominance {
    losses: std::collections::BTreeSet<u64>,
    conflicts: std::collections::BTreeSet<u64>,
    wins: std::collections::BTreeMap<u64, u8>,
    known_by_candidate: std::collections::BTreeMap<u64, u8>,
    lattice_size: u8,
    known_edges: u16,
    unknown_edges: u16,
    cycle_members: u16,
}

impl PairwiseDominance {
    fn blocks(&self, candidate: u64) -> bool {
        self.losses.contains(&candidate) || self.conflicts.contains(&candidate)
    }

    fn certified_winner(&self) -> Option<u64> {
        let required = self.lattice_size.checked_sub(1)?;
        self.wins.iter().find_map(|(candidate, wins)| {
            (*wins == required
                && self.known_by_candidate.get(candidate).copied() == Some(required)
                && !self.blocks(*candidate))
            .then_some(*candidate)
        })
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ContextPhasePackage {
    pub(crate) semantic_states: Vec<TokenSemanticState>,
    pub(crate) profiles: Vec<ContextCandidateProfile>,
    pub(crate) signature_profiles: Vec<ContextCandidateProfile>,
    pub(crate) pair_profiles: Vec<ContextPairPhaseProfile>,
    pub(crate) transitions: u64,
    pub(crate) corpus_fragments: u32,
    pub(crate) global_threshold_micro: i32,
    pub(crate) competition_threshold_micro: i32,
    pub(crate) pairwise_threshold_micro: i32,
    /// Versioned so a hot package always uses the same compact L2 projection
    /// that produced its signature and pairwise banks during cold learning.
    pub(crate) signature_schema: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ContextPhaseDisposition {
    Support,
    Suppress,
    Neutral,
    #[default]
    Unavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ContextPhaseReadout {
    pub(crate) package_loaded: bool,
    pub(crate) profile_present: bool,
    pub(crate) disposition: ContextPhaseDisposition,
    pub(crate) positive_micro: i64,
    pub(crate) anti_micro: i64,
    pub(crate) margin_micro: i64,
    pub(crate) threshold_micro: i64,
    pub(crate) competition_margin_micro: i64,
    pub(crate) positive_examples: u32,
    pub(crate) negative_examples: u32,
    pub(crate) positive_centers: u8,
    pub(crate) anti_centers: u8,
    pub(crate) positive_center_support: u32,
    pub(crate) anti_center_support: u32,
    pub(crate) semantic_support: u32,
    pub(crate) signature_profile_present: bool,
    pub(crate) signature_positive_micro: i64,
    pub(crate) signature_center_support: u32,
    pub(crate) relation_class: u64,
    pub(crate) context_tokens: u16,
    pub(crate) context_known_tokens: u16,
    pub(crate) pairwise_blocked: bool,
    pub(crate) pairwise_conflict: bool,
    pub(crate) pairwise_certified: bool,
    pub(crate) pairwise_known_edges: u16,
    pub(crate) pairwise_unknown_edges: u16,
    pub(crate) pairwise_cycle_members: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ContextPhaseMode {
    Full,
    NoPhase,
    NoAnti,
    NoSemanticState,
    NoSignatureProfile,
    NoPairwise,
    NoHardPairwise,
    ShuffledPairDirection,
    ShuffledPairScene,
    MagnitudeOnlyPairwise,
}

impl ContextPhaseMode {
    fn pairwise_enabled(self) -> bool {
        !matches!(self, Self::NoPhase | Self::NoPairwise)
    }

    fn hard_pairwise_enabled(self) -> bool {
        !matches!(
            self,
            Self::NoPhase | Self::NoPairwise | Self::NoHardPairwise | Self::NoAnti
        )
    }

    fn signature_profile_enabled(self) -> bool {
        !matches!(self, Self::NoPhase | Self::NoSignatureProfile)
    }
}

impl ContextPhasePackage {
    /// Deterministically merges cold compiler shards. Positive and both
    /// destructive banks remain separate, so shard merging cannot turn an
    /// anti-wave into positive support.
    #[cfg(test)]
    pub(crate) fn merge_shards(shards: Vec<Self>) -> Self {
        Self::merge_shards_with_min_surface_support(shards, 1).0
    }

    /// Merges independently compiled corpus surfaces and retains only the
    /// requested cross-surface lexical/pair relations. Semantic anchors stay
    /// broad because they encode the shared scene coordinate system; candidate
    /// and pair authority must survive more than one surface when consensus is
    /// requested. `min_surface_support = 1` is byte-for-byte behavioral
    /// compatibility with the original shard merge.
    pub(crate) fn merge_shards_with_min_surface_support(
        mut shards: Vec<Self>,
        min_surface_support: u32,
    ) -> (Self, SurfaceConsensusMergeReport) {
        shards.sort_by_key(|package| (package.corpus_fragments, package.transitions));
        let signature_schema = shards
            .first()
            .map(|package| package.signature_schema)
            .unwrap_or_default();
        debug_assert!(shards
            .iter()
            .all(|package| package.signature_schema == signature_schema));
        let surface_count = u32::try_from(shards.len()).unwrap_or(u32::MAX);
        let required = min_surface_support.clamp(1, surface_count.max(1));
        let mut profile_surfaces = std::collections::BTreeMap::<u64, u32>::new();
        let mut signature_profile_surfaces = std::collections::BTreeMap::<u64, u32>::new();
        let mut pair_surfaces = std::collections::BTreeMap::<(u64, u64), u32>::new();
        for shard in &shards {
            for profile in &shard.profiles {
                *profile_surfaces.entry(profile.token_hash).or_default() += 1;
            }
            for profile in &shard.signature_profiles {
                *signature_profile_surfaces
                    .entry(profile.token_hash)
                    .or_default() += 1;
            }
            for pair in &shard.pair_profiles {
                *pair_surfaces
                    .entry((pair.low_hash, pair.high_hash))
                    .or_default() += 1;
            }
        }
        let mut merged = Self::default();
        merged.signature_schema = signature_schema;
        for shard in shards {
            merged.transitions = merged.transitions.saturating_add(shard.transitions);
            merged.corpus_fragments = merged
                .corpus_fragments
                .saturating_add(shard.corpus_fragments);
            merged.global_threshold_micro = merged
                .global_threshold_micro
                .max(shard.global_threshold_micro);
            merged.competition_threshold_micro = merged
                .competition_threshold_micro
                .max(shard.competition_threshold_micro);
            merged.pairwise_threshold_micro = merged
                .pairwise_threshold_micro
                .max(shard.pairwise_threshold_micro);
            for state in shard.semantic_states {
                if let Some(existing) = merged
                    .semantic_states
                    .iter_mut()
                    .find(|value| value.token_hash == state.token_hash)
                {
                    for (left, right) in existing.center.iter_mut().zip(&state.center) {
                        left.re += right.re;
                        left.im += right.im;
                    }
                    existing.center = super::phase_field::phase_center_from_sum(&existing.center);
                    existing.support = existing.support.saturating_add(state.support);
                } else {
                    merged.semantic_states.push(state);
                }
            }
            for profile in shard.profiles {
                if let Some(existing) = merged
                    .profiles
                    .iter_mut()
                    .find(|value| value.token_hash == profile.token_hash)
                {
                    existing.positive_examples = existing
                        .positive_examples
                        .saturating_add(profile.positive_examples);
                    existing.negative_examples = existing
                        .negative_examples
                        .saturating_add(profile.negative_examples);
                    existing.threshold_micro =
                        existing.threshold_micro.max(profile.threshold_micro);
                    existing.positive.extend(profile.positive);
                    existing.negative.extend(profile.negative);
                    existing.hard_negative.extend(profile.hard_negative);
                } else {
                    merged.profiles.push(profile);
                }
            }
            for profile in shard.signature_profiles {
                merge_candidate_profile(
                    &mut merged.signature_profiles,
                    profile,
                    MAX_SIGNATURE_PROFILES,
                );
            }
            for pair in shard.pair_profiles {
                if let Some(existing) = merged.pair_profiles.iter_mut().find(|value| {
                    value.low_hash == pair.low_hash && value.high_hash == pair.high_hash
                }) {
                    merge_pair_bank(
                        &mut existing.low_wins,
                        pair.low_wins,
                        MAX_PAIR_CENTERS_PER_BANK,
                    );
                    merge_pair_bank(
                        &mut existing.high_wins,
                        pair.high_wins,
                        MAX_PAIR_CENTERS_PER_BANK,
                    );
                    merge_pair_bank(
                        &mut existing.hard_low_wins,
                        pair.hard_low_wins,
                        MAX_HARD_PAIR_CENTERS_PER_BANK,
                    );
                    merge_pair_bank(
                        &mut existing.hard_high_wins,
                        pair.hard_high_wins,
                        MAX_HARD_PAIR_CENTERS_PER_BANK,
                    );
                } else if merged.pair_profiles.len() < MAX_PAIR_PROFILES {
                    let mut bounded = ContextPairPhaseProfile {
                        low_hash: pair.low_hash,
                        high_hash: pair.high_hash,
                        ..ContextPairPhaseProfile::default()
                    };
                    merge_pair_bank(
                        &mut bounded.low_wins,
                        pair.low_wins,
                        MAX_PAIR_CENTERS_PER_BANK,
                    );
                    merge_pair_bank(
                        &mut bounded.high_wins,
                        pair.high_wins,
                        MAX_PAIR_CENTERS_PER_BANK,
                    );
                    merge_pair_bank(
                        &mut bounded.hard_low_wins,
                        pair.hard_low_wins,
                        MAX_HARD_PAIR_CENTERS_PER_BANK,
                    );
                    merge_pair_bank(
                        &mut bounded.hard_high_wins,
                        pair.hard_high_wins,
                        MAX_HARD_PAIR_CENTERS_PER_BANK,
                    );
                    merged.pair_profiles.push(bounded);
                }
            }
        }
        merged.semantic_states.sort_by_key(|value| value.token_hash);
        merged.profiles.sort_by_key(|value| value.token_hash);
        merged
            .signature_profiles
            .sort_by_key(|value| value.token_hash);
        merged
            .pair_profiles
            .sort_by_key(|value| (value.low_hash, value.high_hash));
        let profiles_before_consensus = merged.profiles.len();
        let pairs_before_consensus = merged.pair_profiles.len();
        if required > 1 {
            merged.profiles.retain(|profile| {
                profile_surfaces
                    .get(&profile.token_hash)
                    .copied()
                    .unwrap_or_default()
                    >= required
            });
            merged.pair_profiles.retain(|pair| {
                pair_surfaces
                    .get(&(pair.low_hash, pair.high_hash))
                    .copied()
                    .unwrap_or_default()
                    >= required
            });
            merged.signature_profiles.retain(|profile| {
                signature_profile_surfaces
                    .get(&profile.token_hash)
                    .copied()
                    .unwrap_or_default()
                    >= required
            });
        }
        let report = SurfaceConsensusMergeReport {
            surface_count,
            min_surface_support: required,
            profiles_before_consensus,
            profiles_after_consensus: merged.profiles.len(),
            pairs_before_consensus,
            pairs_after_consensus: merged.pair_profiles.len(),
        };
        (merged, report)
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.profiles.is_empty() && self.signature_profiles.is_empty()
    }

    pub(crate) fn pair_profile_counts(&self) -> (usize, usize) {
        let generalized = self
            .pair_profiles
            .iter()
            .filter(|profile| profile.low_hash == 0)
            .count();
        (
            self.pair_profiles.len().saturating_sub(generalized),
            generalized,
        )
    }

    pub(crate) fn score_candidates(
        &self,
        context_tokens: &[String],
        candidates: &[&str],
    ) -> Vec<ContextPhaseReadout> {
        self.score_candidates_with_mode(context_tokens, candidates, ContextPhaseMode::Full)
    }

    pub(crate) fn score_candidates_with_mode(
        &self,
        context_tokens: &[String],
        candidates: &[&str],
        mode: ContextPhaseMode,
    ) -> Vec<ContextPhaseReadout> {
        self.score_candidates_with_mode_and_pair_views(context_tokens, candidates, mode, None, None)
    }

    fn score_candidates_with_mode_and_pair_views(
        &self,
        context_tokens: &[String],
        candidates: &[&str],
        mode: ContextPhaseMode,
        pair_views: Option<&[Vec<String>]>,
        direct_pair_view_indices: Option<&[usize]>,
    ) -> Vec<ContextPhaseReadout> {
        if self.is_empty() || context_tokens.is_empty() {
            return vec![ContextPhaseReadout::default(); candidates.len()];
        }
        let relation_competition = self.signature_schema >= SIGNATURE_SCHEMA_RELATION_ROLES
            && candidates
                .iter()
                .any(|candidate| relation_role_candidate(candidate));
        let scene =
            self.context_vector_for_relation_roles(context_tokens, mode, relation_competition);
        let exact_scene = (relation_competition
            && candidates
                .iter()
                .any(|candidate| !relation_role_candidate(candidate)))
        .then(|| self.context_vector_for_relation_roles(context_tokens, mode, false));
        let mut readouts = candidates
            .iter()
            .map(|candidate| {
                let relation_candidate = relation_role_candidate(candidate);
                let signature_scene = if relation_competition && !relation_candidate {
                    exact_scene.as_deref().unwrap_or(&scene)
                } else {
                    &scene
                };
                // The context scene is identical for every candidate in this
                // lane. Reusing it avoids rebuilding the full L3 scene once
                // per frontier member on the synchronous IME preedit path.
                let vector =
                    self.candidate_relation_vector_from_scene(signature_scene, candidate, mode);
                self.raw_readout(
                    &vector,
                    signature_scene,
                    candidate,
                    mode,
                    relation_script_mismatch(context_tokens, candidate),
                )
            })
            .collect::<Vec<_>>();
        let context_known_tokens =
            context_tokens_for_authority(self, context_tokens).min(u16::MAX as usize) as u16;
        let context_token_count = context_tokens.len().min(u16::MAX as usize) as u16;
        for readout in &mut readouts {
            readout.context_tokens = context_token_count;
            readout.context_known_tokens = context_known_tokens;
        }

        // A single preceding token cannot define the direction of an
        // otherwise tied pair. Keep its unary ranking, but defer pairwise
        // authority until the scene has at least two tokens.
        let sentence_pair_views = pair_views.is_some();
        let anchored_sentence_competition = sentence_pair_views
            && direct_pair_view_indices.is_some_and(|indices| {
                indices
                    .iter()
                    .copied()
                    .filter(|index| {
                        matches!(
                            *index,
                            sentence::PAIR_VIEW_LEFT_EXACT | sentence::PAIR_VIEW_RIGHT_EXACT
                        )
                    })
                    .any(|view_index| {
                        (0..candidates.len()).any(|left| {
                            (left + 1..candidates.len()).any(|right| {
                                self.pair_view_profile_exists(
                                    candidate_token_hash(candidates[left]),
                                    candidate_token_hash(candidates[right]),
                                    self.candidate_signature(candidates[left]),
                                    self.candidate_signature(candidates[right]),
                                    view_index,
                                )
                            })
                        })
                    })
            });
        let pair_scenes = pair_views
            .map(|views| {
                views
                    .iter()
                    .enumerate()
                    .filter(|(_, view)| view.len() >= 2)
                    .map(|(index, view)| {
                        (
                            index,
                            self.context_vector_for_relation_roles(
                                view,
                                mode,
                                relation_competition,
                            ),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![(usize::MAX, scene.clone())]);
        let direct_pair_scenes = direct_pair_view_indices
            .map(|indices| {
                pair_scenes
                    .iter()
                    .filter(|(index, _)| indices.contains(index))
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let pairwise = if self.pair_profiles.is_empty()
            || !mode.pairwise_enabled()
            || pair_scenes.is_empty()
            || (!sentence_pair_views && context_tokens.len() < 2)
        {
            PairwiseDominance::default()
        } else {
            let mut pair_candidates = std::collections::BTreeMap::<u64, (i64, u64)>::new();
            for (candidate, readout) in candidates.iter().zip(&readouts) {
                let hash = candidate_token_hash(candidate);
                let signature = self.candidate_signature(candidate);
                pair_candidates
                    .entry(hash)
                    .and_modify(|entry| entry.0 = entry.0.max(readout.margin_micro))
                    .or_insert((readout.margin_micro, signature));
            }
            let mut pair_lattice = pair_candidates.into_iter().collect::<Vec<_>>();
            pair_lattice.sort_by(|left, right| {
                right
                    .1
                     .0
                    .cmp(&left.1 .0)
                    .then_with(|| left.0.cmp(&right.0))
            });
            pair_lattice.truncate(MAX_PAIR_CANDIDATES);
            if sentence_pair_views {
                let all = self.pairwise_dominance_across_scenes(&pair_scenes, &pair_lattice, mode);
                if direct_pair_scenes.is_empty() {
                    all
                } else {
                    // Structural/morphology views may rank the field, but a
                    // sentence winner must settle in the scene's authority
                    // views. Falling back to `all` here let two correlated
                    // punctuation crystals certify unrelated clean contexts.
                    self.pairwise_dominance_across_scenes(&direct_pair_scenes, &pair_lattice, mode)
                }
            } else {
                self.pairwise_dominance(&scene, &pair_lattice, mode)
            }
        };
        let pairwise_certificate = pairwise.certified_winner();

        let unary_ranked = survivor_ranking(candidates, &readouts, &PairwiseDominance::default());
        let ranked = survivor_ranking(candidates, &readouts, &pairwise);
        let best = pairwise_certificate
            .and_then(|winner| {
                candidates
                    .iter()
                    .enumerate()
                    .find_map(|(index, candidate)| {
                        (candidate_token_hash(candidate) == winner
                            && readouts[index].profile_present)
                            .then_some((index, readouts[index].margin_micro))
                    })
            })
            .or_else(|| ranked.first().copied());
        let runner_up = ranked
            .get(1)
            .map(|(_, score)| *score)
            .unwrap_or(i64::MIN / 2);
        let competition_margin = best
            .map(|(_, score)| score.saturating_sub(runner_up))
            .unwrap_or_default();
        let unary_competition_margin = best
            .map(|(index, score)| {
                let winner_hash = candidate_token_hash(candidates[index]);
                let runner_up = unary_ranked
                    .iter()
                    .filter(|(other, _)| candidate_token_hash(candidates[*other]) != winner_hash)
                    .map(|(_, margin)| *margin)
                    .max()
                    .unwrap_or(i64::MIN / 2);
                score.saturating_sub(runner_up)
            })
            .unwrap_or_default();
        let unary_competition_ready = unary_ranked.len() == 1
            || unary_competition_margin >= i64::from(self.competition_threshold_micro.max(1));
        let all_candidate_profiles_known = readouts
            .iter()
            .all(|readout| readout.profile_present || readout.signature_profile_present);
        let exact_candidate_profiles = readouts
            .iter()
            .filter(|readout| readout.profile_present)
            .count();
        let active_candidate_basins = readouts
            .iter()
            .filter(|readout| {
                readout.profile_present
                    && readout.positive_center_support > 0
                    && readout.positive_micro > readout.anti_micro
            })
            .count();
        let all_exact_candidate_profiles_known =
            readouts.iter().all(|readout| readout.profile_present);
        let best_support_ready = best.is_some_and(|(index, _)| {
            let readout = &readouts[index];
            let competition_threshold = i64::from(self.competition_threshold_micro.max(1));
            // A one-shot positive subcenter is a hypothesis, not a settled
            // context state. If the same candidate already carries a learned
            // counter-wave, wait for a second coherent observation instead of
            // promoting the unresolved phase conflict.
            let provisional_conflict =
                readout.positive_center_support < 2 && readout.anti_center_support > 0;
            // Finite evidence raises admission energy; it must not inflate the score.
            let support_uncertainty = (competition_threshold as f64
                / f64::from(readout.positive_examples.max(1)).sqrt())
            .ceil() as i64;
            let absolute_support_ready =
                readout.margin_micro >= readout.threshold_micro.saturating_add(support_uncertainty);
            let relative_competition_ready = unary_ranked.len() >= 2
                && unary_competition_margin
                    >= competition_threshold.saturating_add(support_uncertainty);
            // Pairwise may exclude a false competitor, but it must not erase
            // the unary relative evidence that made a survivor admissible.
            let pairwise_certified = pairwise_certificate
                .is_some_and(|winner| winner == candidate_token_hash(candidates[index]));
            let center_support_ready = readout.positive_center_support
                >= if sentence_pair_views && pairwise_certified {
                    1
                } else {
                    2
                };
            (pairwise_certified
                && (unary_competition_ready || context_tokens.len() >= 3)
                && readout.positive_examples >= 2
                && center_support_ready
                && !provisional_conflict)
                || (unary_competition_ready
                    && readout.positive_examples >= 2
                    && readout.positive_center_support >= 2
                    && !provisional_conflict
                    && (absolute_support_ready
                        || (relative_competition_ready
                            && readout.positive_micro > readout.anti_micro)))
        });

        for (index, readout) in readouts.iter_mut().enumerate() {
            if !readout.profile_present || mode == ContextPhaseMode::NoPhase {
                continue;
            }
            let is_best_token = best.is_some_and(|(best, _)| {
                candidate_token_hash(candidates[best]) == candidate_token_hash(candidates[index])
            });
            readout.competition_margin_micro = if is_best_token {
                competition_margin
            } else {
                best.map(|(_, score)| readout.margin_micro.saturating_sub(score))
                    .unwrap_or_default()
            };
            let minimum_known_tokens = if self.signature_schema >= SIGNATURE_SCHEMA_RELATION_ROLES {
                1
            } else {
                2
            };
            let context_ready = usize::from(readout.context_known_tokens) >= minimum_known_tokens
                && usize::from(readout.context_known_tokens) * 2
                    >= usize::from(readout.context_tokens)
                && (!relation_competition || usize::from(readout.context_tokens) >= 2);
            let anti_margin_ready = readout.anti_micro.saturating_sub(readout.positive_micro)
                >= i64::from(self.competition_threshold_micro.max(1));
            let pair_blocked = pairwise.blocks(candidate_token_hash(candidates[index]));
            let pairwise_certified = pairwise_certificate
                .is_some_and(|winner| winner == candidate_token_hash(candidates[index]));
            // Two active basins always need directional pair evidence. A quiet
            // learned basin also remains unresolved when the exact lexical
            // field is incomplete; signature-only presence is not enough to
            // certify ordinary word competition.
            let competition_resolved = pairwise_certified
                || (!anchored_sentence_competition
                    && active_candidate_basins <= 1
                    && (exact_candidate_profiles <= 1 || all_exact_candidate_profiles_known));
            readout.pairwise_blocked = pair_blocked;
            readout.pairwise_certified = pairwise_certified;
            readout.pairwise_conflict = pairwise
                .conflicts
                .contains(&candidate_token_hash(candidates[index]));
            readout.pairwise_known_edges = pairwise.known_edges;
            readout.pairwise_unknown_edges = pairwise.unknown_edges;
            readout.pairwise_cycle_members = pairwise.cycle_members;
            readout.disposition = if is_best_token
                && best_support_ready
                && context_ready
                && (!relation_competition || all_candidate_profiles_known)
                && competition_resolved
                && !pair_blocked
            {
                ContextPhaseDisposition::Support
            } else if context_ready
                && (anti_margin_ready
                    || (best_support_ready
                        && best.is_some_and(|(_, score)| {
                            score.saturating_sub(readout.margin_micro)
                                >= i64::from(self.competition_threshold_micro.max(1))
                        })))
            {
                ContextPhaseDisposition::Suppress
            } else {
                ContextPhaseDisposition::Neutral
            };
        }
        readouts
    }

    pub(crate) fn score_sentence_candidates(
        &self,
        scene: &sentence::SentenceContextScene,
        candidates: &[&str],
    ) -> Vec<ContextPhaseReadout> {
        let structured = self
            .semantic_state(sentence::SentenceContextScene::anchor_hash())
            .is_some_and(|state| state.support >= 2);
        if structured {
            let encoded = scene.encoded_tokens();
            let pair_views = scene
                .has_directional_context()
                .then(|| scene.pair_views())
                .unwrap_or_default();
            let direct_pair_view_indices = scene.direct_pair_view_indices();
            self.score_candidates_with_mode_and_pair_views(
                &encoded,
                candidates,
                ContextPhaseMode::Full,
                Some(&pair_views),
                Some(&direct_pair_view_indices),
            )
        } else {
            self.score_candidates(scene.legacy_left_tokens(), candidates)
        }
    }

    pub(crate) fn sentence_pair_debug(
        &self,
        scene: &sentence::SentenceContextScene,
        candidates: &[&str],
    ) -> serde_json::Value {
        serde_json::json!({
            "views": scene
                .pair_views()
                .iter()
                .enumerate()
                .map(|(index, view)| serde_json::json!({
                    "index": index,
                    "tokens": view,
                    "pair": self.pair_view_debug(view, candidates, index),
                }))
                .collect::<Vec<_>>(),
        })
    }

    pub(crate) fn pair_debug(
        &self,
        context_tokens: &[String],
        candidates: &[&str],
    ) -> serde_json::Value {
        if candidates.len() != 2 {
            return serde_json::json!({"available": false});
        }
        let left_hash = candidate_token_hash(candidates[0]);
        let right_hash = candidate_token_hash(candidates[1]);
        let Some(key) = PairKey::new(left_hash, right_hash) else {
            return serde_json::json!({"available": false});
        };
        let Some(profile) = self.pair_profile(key) else {
            return serde_json::json!({"available": false, "key": [key.low_hash, key.high_hash]});
        };
        let relation_competition = candidates
            .iter()
            .any(|candidate| relation_role_candidate(candidate));
        let scene = self.context_vector_for_relation_roles(
            context_tokens,
            ContextPhaseMode::Full,
            relation_competition,
        );
        let low = strongest_center_with_min_support(&scene, &profile.low_wins, 2);
        let high = strongest_center_with_min_support(&scene, &profile.high_wins, 2);
        serde_json::json!({
            "available": true,
            "low_hash": key.low_hash,
            "high_hash": key.high_hash,
            "low_candidate": if left_hash == key.low_hash { candidates[0] } else { candidates[1] },
            "high_candidate": if left_hash == key.high_hash { candidates[0] } else { candidates[1] },
            "low_score_ppm": low.map(|(score, _)| phase_micro(score)).unwrap_or_default(),
            "high_score_ppm": high.map(|(score, _)| phase_micro(score)).unwrap_or_default(),
            "low_local_support": low.map(|(_, support)| support).unwrap_or_default(),
            "high_local_support": high.map(|(_, support)| support).unwrap_or_default(),
            "low_bank_support": bank_support(&profile.low_wins),
            "high_bank_support": bank_support(&profile.high_wins),
            "low_centers": profile.low_wins.len(),
            "high_centers": profile.high_wins.len(),
            "pairwise_threshold_micro": self.pairwise_threshold_micro,
            "outcome": format!("{:?}", self.pair_edge_for_profile(&scene, profile, true)),
        })
    }

    fn pair_view_debug(
        &self,
        context_tokens: &[String],
        candidates: &[&str],
        view_index: usize,
    ) -> serde_json::Value {
        if candidates.len() != 2 {
            return serde_json::json!({"available": false});
        }
        let left = candidate_token_hash(candidates[0]);
        let right = candidate_token_hash(candidates[1]);
        let left_signature = self.candidate_signature(candidates[0]);
        let right_signature = self.candidate_signature(candidates[1]);
        let relation_competition = candidates
            .iter()
            .any(|candidate| relation_role_candidate(candidate));
        let scene = self.context_vector_for_relation_roles(
            context_tokens,
            ContextPhaseMode::Full,
            relation_competition,
        );
        let evidence = self.pair_view_edge_evidence(
            &scene,
            left,
            right,
            left_signature,
            right_signature,
            view_index,
            true,
        );
        serde_json::json!({
            "available": true,
            "view_index": view_index,
            "outcome": format!("{:?}", evidence.outcome),
            "confidence_micro": (evidence.confidence * 1_000_000.0).round() as i64,
        })
    }

    fn pair_edge(
        &self,
        scene: &[PhaseCell],
        left: u64,
        right: u64,
        left_signature: u64,
        right_signature: u64,
        hard_enabled: bool,
    ) -> PairEdgeOutcome {
        let Some(exact_key) = PairKey::new(left, right) else {
            return PairEdgeOutcome::Tie;
        };
        let exact = self
            .pair_profile(exact_key)
            .map(|profile| self.pair_edge_for_profile(scene, profile, hard_enabled));
        // An exact observed pair owns its known uncertainty. A generalized L2
        // relation only fills a genuine coverage gap; it never overrides an
        // exact tie or conflict.
        if !matches!(exact, None | Some(PairEdgeOutcome::Unknown)) {
            return exact.unwrap_or_default();
        }
        let Some(relation_key) = PairKey::relation(left, left_signature, right, right_signature)
        else {
            return exact.unwrap_or(PairEdgeOutcome::Unknown);
        };
        self.pair_profile(relation_key)
            .map(|profile| {
                remap_relation_outcome(
                    self.pair_edge_for_profile(scene, profile, false),
                    left,
                    right,
                    left_signature,
                    right_signature,
                )
            })
            .unwrap_or_else(|| exact.unwrap_or(PairEdgeOutcome::Unknown))
    }

    fn pair_view_edge_evidence(
        &self,
        scene: &[PhaseCell],
        left: u64,
        right: u64,
        left_signature: u64,
        right_signature: u64,
        view_index: usize,
        hard_enabled: bool,
    ) -> PairEdgeEvidence {
        let view_left = pair_view_hash(left, view_index);
        let view_right = pair_view_hash(right, view_index);
        let Some(exact_key) = PairKey::new(view_left, view_right) else {
            return PairEdgeEvidence {
                outcome: PairEdgeOutcome::Tie,
                confidence: 0.0,
            };
        };
        let exact = self.pair_profile(exact_key).map(|profile| {
            let mut evidence =
                self.pair_view_evidence_for_profile(scene, profile, view_index, hard_enabled);
            evidence.outcome =
                remap_pair_view_outcome(evidence.outcome, left, right, view_left, view_right);
            evidence
        });
        if exact.is_some_and(|evidence| evidence.outcome != PairEdgeOutcome::Unknown) {
            return exact.unwrap_or_default();
        }

        let view_left_signature = pair_view_hash(left_signature, view_index);
        let view_right_signature = pair_view_hash(right_signature, view_index);
        let Some(relation_key) = PairKey::relation(
            view_left,
            view_left_signature,
            view_right,
            view_right_signature,
        ) else {
            return exact.unwrap_or_default();
        };
        self.pair_profile(relation_key)
            .map(|profile| {
                let mut evidence =
                    self.pair_view_evidence_for_profile(scene, profile, view_index, false);
                evidence.outcome = remap_relation_outcome(
                    evidence.outcome,
                    left,
                    right,
                    view_left_signature,
                    view_right_signature,
                );
                evidence
            })
            .unwrap_or_else(|| exact.unwrap_or_default())
    }

    fn pair_view_profile_exists(
        &self,
        left: u64,
        right: u64,
        left_signature: u64,
        right_signature: u64,
        view_index: usize,
    ) -> bool {
        let view_left = pair_view_hash(left, view_index);
        let view_right = pair_view_hash(right, view_index);
        if PairKey::new(view_left, view_right).is_some_and(|key| self.pair_profile(key).is_some()) {
            return true;
        }
        let view_left_signature = pair_view_hash(left_signature, view_index);
        let view_right_signature = pair_view_hash(right_signature, view_index);
        PairKey::relation(
            view_left,
            view_left_signature,
            view_right,
            view_right_signature,
        )
        .is_some_and(|key| self.pair_profile(key).is_some())
    }

    fn pair_view_evidence_for_profile(
        &self,
        scene: &[PhaseCell],
        profile: &ContextPairPhaseProfile,
        view_index: usize,
        hard_enabled: bool,
    ) -> PairEdgeEvidence {
        let exact_anchor = matches!(
            view_index,
            sentence::PAIR_VIEW_LEFT_EXACT | sentence::PAIR_VIEW_RIGHT_EXACT
        );
        let anchored_centers = [
            strongest_directional_center(scene, &profile.low_wins),
            strongest_directional_center(scene, &profile.high_wins),
            strongest_center_with_min_support(scene, &profile.hard_low_wins, 2),
            strongest_center_with_min_support(scene, &profile.hard_high_wins, 2),
        ];
        if exact_anchor
            && anchored_centers
                .into_iter()
                .flatten()
                .all(|(score, _)| score < PAIR_CENTER_SPLIT_COHERENCE)
        {
            return PairEdgeEvidence::default();
        }
        self.pair_view_raw_evidence_for_profile(scene, profile, hard_enabled)
    }

    fn pair_view_raw_evidence_for_profile(
        &self,
        scene: &[PhaseCell],
        profile: &ContextPairPhaseProfile,
        hard_enabled: bool,
    ) -> PairEdgeEvidence {
        let gated = self.pair_edge_evidence_for_profile(scene, profile, hard_enabled);
        if matches!(
            gated.outcome,
            PairEdgeOutcome::LowWins | PairEdgeOutcome::HighWins | PairEdgeOutcome::Conflict
        ) {
            return gated;
        }
        let low = strongest_directional_center(scene, &profile.low_wins)
            .map(|(score, _)| score)
            .unwrap_or(0.0);
        let high = strongest_directional_center(scene, &profile.high_wins)
            .map(|(score, _)| score)
            .unwrap_or(0.0);
        let confidence = (low - high).abs();
        if confidence <= f32::EPSILON {
            return PairEdgeEvidence {
                outcome: if low > 0.0 || high > 0.0 {
                    PairEdgeOutcome::Tie
                } else {
                    PairEdgeOutcome::Unknown
                },
                confidence,
            };
        }
        PairEdgeEvidence {
            outcome: if low > high {
                PairEdgeOutcome::LowWins
            } else {
                PairEdgeOutcome::HighWins
            },
            confidence,
        }
    }

    fn pair_edge_for_profile(
        &self,
        scene: &[PhaseCell],
        profile: &ContextPairPhaseProfile,
        hard_enabled: bool,
    ) -> PairEdgeOutcome {
        self.pair_edge_evidence_for_profile(scene, profile, hard_enabled)
            .outcome
    }

    fn pair_edge_evidence_for_profile(
        &self,
        scene: &[PhaseCell],
        profile: &ContextPairPhaseProfile,
        hard_enabled: bool,
    ) -> PairEdgeEvidence {
        let hard_low = hard_enabled
            .then(|| strongest_center_with_min_support(scene, &profile.hard_low_wins, 2))
            .flatten()
            .map(|(score, _)| score)
            .unwrap_or(0.0);
        let hard_high = hard_enabled
            .then(|| strongest_center_with_min_support(scene, &profile.hard_high_wins, 2))
            .flatten()
            .map(|(score, _)| score)
            .unwrap_or(0.0);
        let threshold = self.pairwise_threshold_micro.max(1) as f32 / 1_000_000.0;
        if hard_low >= threshold && hard_high >= threshold {
            return PairEdgeEvidence {
                outcome: PairEdgeOutcome::Conflict,
                confidence: hard_low.min(hard_high),
            };
        }
        // A hard bank is a counter-wave: it may remove a known false winner,
        // but support still requires the winner's unary L3 evidence below.
        if hard_low >= threshold {
            return PairEdgeEvidence {
                outcome: PairEdgeOutcome::LowWins,
                confidence: hard_low,
            };
        }
        if hard_high >= threshold {
            return PairEdgeEvidence {
                outcome: PairEdgeOutcome::HighWins,
                confidence: hard_high,
            };
        }
        let low = strongest_center_with_min_support(scene, &profile.low_wins, 2);
        let high = strongest_center_with_min_support(scene, &profile.high_wins, 2);
        let low_score = low.map(|(score, _)| score).unwrap_or(0.0);
        let high_score = high.map(|(score, _)| score).unwrap_or(0.0);
        if low_score <= 0.0 && high_score <= 0.0 {
            return PairEdgeEvidence::default();
        }
        let (winner_support, directional_support) = if low_score > high_score {
            (
                low.map(|(_, support)| support).unwrap_or_default(),
                bank_support(&profile.low_wins),
            )
        } else {
            (
                high.map(|(_, support)| support).unwrap_or_default(),
                bank_support(&profile.high_wins),
            )
        };
        // A pair can legitimately occupy several scene subcenters. Its local
        // center still needs support, while the full directional bank records
        // whether that learned relation has repeatedly settled overall.
        let evidence_margin =
            directional_evidence_margin(threshold, winner_support, directional_support);
        if (low_score - high_score).abs() < evidence_margin {
            return PairEdgeEvidence {
                outcome: PairEdgeOutcome::Tie,
                confidence: (low_score - high_score).abs(),
            };
        }
        let outcome = if low_score > high_score {
            PairEdgeOutcome::LowWins
        } else {
            PairEdgeOutcome::HighWins
        };
        PairEdgeEvidence {
            outcome,
            confidence: (low_score - high_score).abs(),
        }
    }

    fn pairwise_dominance(
        &self,
        scene: &[PhaseCell],
        lattice: &[(u64, (i64, u64))],
        mode: ContextPhaseMode,
    ) -> PairwiseDominance {
        self.pairwise_dominance_across_scenes(&[(usize::MAX, scene.to_vec())], lattice, mode)
    }

    fn pairwise_dominance_across_scenes(
        &self,
        scenes: &[(usize, Vec<PhaseCell>)],
        lattice: &[(u64, (i64, u64))],
        mode: ContextPhaseMode,
    ) -> PairwiseDominance {
        let mut dominance = PairwiseDominance::default();
        if !mode.pairwise_enabled() {
            return dominance;
        }
        let scenes = scenes
            .iter()
            .map(|(view_index, scene)| {
                (
                    *view_index,
                    pairwise_scene(scene, mode).unwrap_or_else(|| scene.clone()),
                )
            })
            .collect::<Vec<_>>();
        let mut edges = std::collections::BTreeMap::<u64, std::collections::BTreeSet<u64>>::new();
        dominance.lattice_size = lattice.len().min(u8::MAX as usize) as u8;
        for left in 0..lattice.len() {
            for right in left + 1..lattice.len() {
                let (left_hash, (_, left_signature)) = lattice[left];
                let (right_hash, (_, right_signature)) = lattice[right];
                let outcome = self.pair_edge_across_scenes(
                    &scenes,
                    left_hash,
                    right_hash,
                    left_signature,
                    right_signature,
                    mode.hard_pairwise_enabled(),
                );
                let outcome = if mode == ContextPhaseMode::ShuffledPairDirection {
                    reverse_pair_outcome(outcome)
                } else {
                    outcome
                };
                match outcome {
                    PairEdgeOutcome::LowWins => {
                        dominance.known_edges = dominance.known_edges.saturating_add(1);
                        let (winner, loser) = if left_hash < right_hash {
                            (left_hash, right_hash)
                        } else {
                            (right_hash, left_hash)
                        };
                        dominance.losses.insert(loser);
                        *dominance.wins.entry(winner).or_default() += 1;
                        *dominance.known_by_candidate.entry(winner).or_default() += 1;
                        *dominance.known_by_candidate.entry(loser).or_default() += 1;
                        edges.entry(winner).or_default().insert(loser);
                    }
                    PairEdgeOutcome::HighWins => {
                        dominance.known_edges = dominance.known_edges.saturating_add(1);
                        let (winner, loser) = if left_hash < right_hash {
                            (right_hash, left_hash)
                        } else {
                            (left_hash, right_hash)
                        };
                        dominance.losses.insert(loser);
                        *dominance.wins.entry(winner).or_default() += 1;
                        *dominance.known_by_candidate.entry(winner).or_default() += 1;
                        *dominance.known_by_candidate.entry(loser).or_default() += 1;
                        edges.entry(winner).or_default().insert(loser);
                    }
                    PairEdgeOutcome::Conflict => {
                        dominance.known_edges = dominance.known_edges.saturating_add(1);
                        dominance.conflicts.insert(left_hash);
                        dominance.conflicts.insert(right_hash);
                        *dominance.known_by_candidate.entry(left_hash).or_default() += 1;
                        *dominance.known_by_candidate.entry(right_hash).or_default() += 1;
                    }
                    PairEdgeOutcome::Tie => {
                        dominance.known_edges = dominance.known_edges.saturating_add(1);
                        *dominance.known_by_candidate.entry(left_hash).or_default() += 1;
                        *dominance.known_by_candidate.entry(right_hash).or_default() += 1;
                    }
                    PairEdgeOutcome::Unknown => {
                        dominance.unknown_edges = dominance.unknown_edges.saturating_add(1);
                    }
                }
            }
        }
        // A directed cycle is unresolved competition, not evidence for an
        // arbitrary hash tie-break. Every member stays Neutral for this scene.
        for (candidate, _) in lattice {
            if pair_graph_reaches(
                &edges,
                *candidate,
                *candidate,
                &mut std::collections::BTreeSet::new(),
            ) {
                dominance.conflicts.insert(*candidate);
                dominance.cycle_members = dominance.cycle_members.saturating_add(1);
            }
        }
        dominance
    }

    fn pair_edge_across_scenes(
        &self,
        scenes: &[(usize, Vec<PhaseCell>)],
        left: u64,
        right: u64,
        left_signature: u64,
        right_signature: u64,
        hard_enabled: bool,
    ) -> PairEdgeOutcome {
        if let [(view_index, scene)] = scenes {
            if *view_index == usize::MAX {
                return self.pair_edge(
                    scene,
                    left,
                    right,
                    left_signature,
                    right_signature,
                    hard_enabled,
                );
            }
        }
        let mut low_confidence = 0.0_f32;
        let mut high_confidence = 0.0_f32;
        let mut low_views = 0_u8;
        let mut high_views = 0_u8;
        let mut tied = false;
        for (view_index, scene) in scenes {
            let evidence = self.pair_view_edge_evidence(
                scene,
                left,
                right,
                left_signature,
                right_signature,
                *view_index,
                hard_enabled,
            );
            match evidence.outcome {
                PairEdgeOutcome::Conflict => return PairEdgeOutcome::Conflict,
                PairEdgeOutcome::LowWins => {
                    low_confidence += evidence.confidence;
                    low_views = low_views.saturating_add(1);
                }
                PairEdgeOutcome::HighWins => {
                    high_confidence += evidence.confidence;
                    high_views = high_views.saturating_add(1);
                }
                PairEdgeOutcome::Tie => tied = true,
                PairEdgeOutcome::Unknown => {}
            }
        }
        let conflict_band = self.pairwise_threshold_micro.max(1) as f32 / 1_000_000.0;
        const MIN_DIRECTIONAL_VIEWS: u8 = 2;
        let outcome = if low_confidence > 0.0 && high_confidence > 0.0 {
            if (low_confidence - high_confidence).abs() < conflict_band {
                PairEdgeOutcome::Conflict
            } else if low_confidence > high_confidence
                && low_confidence - high_confidence >= conflict_band
                && low_views >= MIN_DIRECTIONAL_VIEWS
            {
                PairEdgeOutcome::LowWins
            } else if high_confidence - low_confidence >= conflict_band
                && high_views >= MIN_DIRECTIONAL_VIEWS
            {
                PairEdgeOutcome::HighWins
            } else {
                PairEdgeOutcome::Tie
            }
        } else if low_confidence >= conflict_band && low_views >= MIN_DIRECTIONAL_VIEWS {
            PairEdgeOutcome::LowWins
        } else if high_confidence >= conflict_band && high_views >= MIN_DIRECTIONAL_VIEWS {
            PairEdgeOutcome::HighWins
        } else if tied || low_confidence > 0.0 || high_confidence > 0.0 {
            PairEdgeOutcome::Tie
        } else {
            PairEdgeOutcome::Unknown
        };

        // New online sentence deltas carry an exact-neighbour anchor view. If
        // that profile exists for this pair, broad structural agreement cannot
        // certify a different scene by itself. Packages predating anchor views
        // have no such profile and retain their existing readout contract.
        let mut required = None;
        let mut required_profile_present = false;
        for (view_index, scene) in scenes.iter().filter(|(view_index, _)| {
            matches!(
                *view_index,
                sentence::PAIR_VIEW_LEFT_EXACT | sentence::PAIR_VIEW_RIGHT_EXACT
            )
        }) {
            if !self.pair_view_profile_exists(
                left,
                right,
                left_signature,
                right_signature,
                *view_index,
            ) {
                continue;
            }
            required_profile_present = true;
            let anchor = self.pair_view_edge_evidence(
                scene,
                left,
                right,
                left_signature,
                right_signature,
                *view_index,
                hard_enabled,
            );
            let direction = match anchor.outcome {
                PairEdgeOutcome::LowWins | PairEdgeOutcome::HighWins => anchor.outcome,
                PairEdgeOutcome::Conflict => return PairEdgeOutcome::Conflict,
                PairEdgeOutcome::Tie | PairEdgeOutcome::Unknown => continue,
            };
            if required.is_some_and(|existing| existing != direction) {
                return PairEdgeOutcome::Conflict;
            }
            required = Some(direction);
        }
        match required {
            Some(direction) if outcome == direction => outcome,
            Some(_)
                if matches!(
                    outcome,
                    PairEdgeOutcome::LowWins | PairEdgeOutcome::HighWins
                ) =>
            {
                PairEdgeOutcome::Conflict
            }
            Some(_) => PairEdgeOutcome::Unknown,
            None if required_profile_present => PairEdgeOutcome::Unknown,
            None => outcome,
        }
    }

    fn raw_readout(
        &self,
        vector: &[PhaseCell],
        scene: &[PhaseCell],
        candidate: &str,
        mode: ContextPhaseMode,
        relation_script_mismatch: bool,
    ) -> ContextPhaseReadout {
        let token = crate::word_reader::last_text_word(candidate).unwrap_or_default();
        let token_hash = hash_text(&token.to_lowercase());
        let exact_profile = self.profile(token_hash);
        let signature_profile = mode
            .signature_profile_enabled()
            .then(|| self.signature_profile(self.candidate_signature(candidate)))
            .flatten();
        if mode == ContextPhaseMode::NoPhase {
            return ContextPhaseReadout {
                package_loaded: true,
                ..ContextPhaseReadout::default()
            };
        }
        let Some(profile) = exact_profile else {
            // A morphology/L2 signature can transfer context pressure between
            // unseen lexical forms. It is deliberately display-ranking only:
            // without an exact profile it remains Neutral and can never grant
            // L3 Support or text-edit authority.
            let (signature_positive, signature_support) = signature_profile
                .and_then(|profile| strongest_center(scene, &profile.positive))
                .unwrap_or_default();
            let signature_anti = if mode == ContextPhaseMode::NoAnti {
                0.0
            } else {
                signature_profile
                    .and_then(|profile| strongest_center(scene, &profile.negative))
                    .map(|(coherence, _)| coherence)
                    .unwrap_or_default()
            };
            let signature_margin = phase_micro(signature_positive - signature_anti);
            return ContextPhaseReadout {
                package_loaded: true,
                // Keep this false: all authority paths require an exact
                // lexical profile. The signature contributes a ranking field,
                // not an admissible candidate profile.
                profile_present: false,
                disposition: ContextPhaseDisposition::Neutral,
                positive_micro: signature_margin,
                anti_micro: phase_micro(signature_anti),
                margin_micro: signature_margin,
                signature_profile_present: signature_profile.is_some(),
                signature_positive_micro: signature_margin,
                signature_center_support: signature_support,
                relation_class: relation_class(
                    self.candidate_signature(candidate),
                    signature_margin,
                ),
                ..ContextPhaseReadout::default()
            };
        };
        let signature_positive = signature_profile
            .and_then(|profile| strongest_center(scene, &profile.positive))
            .unwrap_or_default();
        let (exact_positive, exact_positive_center_support) =
            strongest_center(vector, &profile.positive).unwrap_or_default();
        // A signature is a transfer witness from the L2 center field. It may
        // strengthen an existing lexical profile, but cannot create Support
        // for an unseen lexical candidate on its own.
        // Transfer is admissible only where both the lexical center and its
        // L2-state center have independently settled. A provisional center
        // may be useful during cold learning, but must not redirect live
        // competition through an unproven cross-word resemblance.
        let signature_can_reinforce =
            exact_positive_center_support >= 2 && signature_positive.1 >= 2;
        let (positive, positive_center_support) =
            if signature_can_reinforce && signature_positive.0 > exact_positive {
                signature_positive
            } else {
                (exact_positive, exact_positive_center_support)
            };
        let (anti, anti_center_support) = if mode == ContextPhaseMode::NoAnti {
            (0.0, 0)
        } else {
            // A generic L2 competitor says only that this word lost to some
            // other word in one scene. That relation belongs to PairKey and
            // must never become a unary veto on the word everywhere else.
            // The only candidate-local destructive authority is a witnessed
            // false winner, retained in the hard bank below.
            let hard = strongest_center(vector, &profile.hard_negative).unwrap_or_default();
            let relation = relation_script_mismatch
                .then(|| {
                    signature_profile
                        .and_then(|profile| strongest_center(scene, &profile.negative))
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            if relation.0 > hard.0 {
                relation
            } else {
                hard
            }
        };
        let margin = positive - anti;
        let margin_micro = phase_micro(margin);
        ContextPhaseReadout {
            package_loaded: true,
            profile_present: true,
            disposition: ContextPhaseDisposition::Neutral,
            positive_micro: phase_micro(positive),
            anti_micro: phase_micro(anti),
            margin_micro,
            threshold_micro: i64::from(profile.threshold_micro.max(self.global_threshold_micro)),
            competition_margin_micro: 0,
            positive_examples: profile.positive_examples,
            negative_examples: profile.negative_examples,
            positive_centers: profile.positive.len().min(u8::MAX as usize) as u8,
            anti_centers: profile
                .negative
                .len()
                .saturating_add(profile.hard_negative.len())
                .min(u8::MAX as usize) as u8,
            positive_center_support,
            anti_center_support,
            semantic_support: self
                .semantic_state(token_hash)
                .map(|state| state.support)
                .unwrap_or_default(),
            signature_profile_present: signature_profile.is_some(),
            signature_positive_micro: phase_micro(signature_positive.0),
            signature_center_support: signature_positive.1,
            relation_class: relation_class(token_hash, margin_micro),
            context_tokens: 0,
            context_known_tokens: 0,
            pairwise_blocked: false,
            pairwise_conflict: false,
            pairwise_certified: false,
            pairwise_known_edges: 0,
            pairwise_unknown_edges: 0,
            pairwise_cycle_members: 0,
        }
    }

    pub(crate) fn context_vector(
        &self,
        context_tokens: &[String],
        mode: ContextPhaseMode,
    ) -> Vec<PhaseCell> {
        self.context_vector_for_relation_roles(context_tokens, mode, false)
    }

    fn context_vector_for_relation_roles(
        &self,
        context_tokens: &[String],
        mode: ContextPhaseMode,
        relation_roles: bool,
    ) -> Vec<PhaseCell> {
        let relation_scene =
            relation_roles && self.signature_schema >= SIGNATURE_SCHEMA_RELATION_ROLES;
        let hashes = if relation_scene {
            context_atom_hashes(context_tokens, SIGNATURE_SCHEMA_RELATION_ROLES)
        } else {
            context_atom_hashes(context_tokens, SIGNATURE_SCHEMA_MORPHOLOGY_PHASE)
        };
        if relation_scene {
            canonical_relation_scene_wave(&hashes, mode, |atom_index, hash| {
                (atom_index % 2 == 0)
                    .then(|| {
                        self.semantic_state(hash)
                            .map(|state| (state.center.as_slice(), state.support))
                    })
                    .flatten()
            })
        } else {
            canonical_scene_wave(&hashes, mode, |_, hash| {
                self.semantic_state(hash)
                    .map(|state| (state.center.as_slice(), state.support))
            })
        }
    }

    pub(super) fn candidate_relation_vector(
        &self,
        context_tokens: &[String],
        candidate: &str,
        mode: ContextPhaseMode,
    ) -> Vec<PhaseCell> {
        let scene = self.context_vector_for_relation_roles(
            context_tokens,
            mode,
            relation_role_candidate(candidate),
        );
        self.candidate_relation_vector_from_scene(&scene, candidate, mode)
    }

    fn candidate_relation_vector_from_scene(
        &self,
        scene: &[PhaseCell],
        candidate: &str,
        mode: ContextPhaseMode,
    ) -> Vec<PhaseCell> {
        let mut vector = scene.to_vec();
        if mode != ContextPhaseMode::NoSemanticState {
            let token = crate::word_reader::last_text_word(candidate).unwrap_or_default();
            let token_hash = hash_text(&token.to_lowercase());
            if let Some(state) = self.semantic_state(token_hash) {
                let semantic_weight = candidate_semantic_relation_weight(state.support);
                add_rotated_vector(
                    &mut vector,
                    &state.center,
                    token_hash ^ 0x0052_454c_4154_494f,
                    semantic_weight,
                );
            }
        }
        phase_center_from_sum(&vector)
    }

    fn semantic_state(&self, token_hash: u64) -> Option<&TokenSemanticState> {
        self.semantic_states
            .binary_search_by_key(&token_hash, |state| state.token_hash)
            .ok()
            .and_then(|index| self.semantic_states.get(index))
    }

    fn profile(&self, token_hash: u64) -> Option<&ContextCandidateProfile> {
        self.profiles
            .binary_search_by_key(&token_hash, |profile| profile.token_hash)
            .ok()
            .and_then(|index| self.profiles.get(index))
    }

    fn signature_profile(&self, signature: u64) -> Option<&ContextCandidateProfile> {
        self.signature_profiles
            .binary_search_by_key(&signature, |profile| profile.token_hash)
            .ok()
            .and_then(|index| self.signature_profiles.get(index))
    }

    fn candidate_signature(&self, candidate: &str) -> u64 {
        candidate_l2_signature_for_schema(candidate, self.signature_schema)
    }

    fn pair_profile(&self, key: PairKey) -> Option<&ContextPairPhaseProfile> {
        self.pair_profiles
            .binary_search_by_key(&(key.low_hash, key.high_hash), |profile| {
                (profile.low_hash, profile.high_hash)
            })
            .ok()
            .and_then(|index| self.pair_profiles.get(index))
    }
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub(crate) struct SurfaceConsensusMergeReport {
    pub(crate) surface_count: u32,
    pub(crate) min_surface_support: u32,
    pub(crate) profiles_before_consensus: usize,
    pub(crate) profiles_after_consensus: usize,
    pub(crate) pairs_before_consensus: usize,
    pub(crate) pairs_after_consensus: usize,
}

/// The only scene encoder used by cold learning and hot readout. Candidate
/// identity deliberately does not enter this vector: it belongs to the unary
/// profile or PairKey, never to the scene itself.
pub(super) fn canonical_scene_wave<'a, F>(
    context_hashes: &[u64],
    mode: ContextPhaseMode,
    semantic_lookup: F,
) -> Vec<PhaseCell>
where
    F: FnMut(usize, u64) -> Option<(&'a [PhaseCell], u32)>,
{
    canonical_scene_wave_scaled(context_hashes, mode, 1, 1.0, semantic_lookup)
}

pub(super) fn canonical_relation_scene_wave<'a, F>(
    context_hashes: &[u64],
    mode: ContextPhaseMode,
    semantic_lookup: F,
) -> Vec<PhaseCell>
where
    F: FnMut(usize, u64) -> Option<(&'a [PhaseCell], u32)>,
{
    canonical_scene_wave_scaled(context_hashes, mode, 2, 0.5, semantic_lookup)
}

fn canonical_scene_wave_scaled<'a, F>(
    context_hashes: &[u64],
    mode: ContextPhaseMode,
    atoms_per_token: usize,
    role_weight: f32,
    mut semantic_lookup: F,
) -> Vec<PhaseCell>
where
    F: FnMut(usize, u64) -> Option<(&'a [PhaseCell], u32)>,
{
    let mut vector = empty_vector(CELLS);
    let start = context_hashes.len().saturating_sub(MAX_CONTEXT_ATOMS);
    for (offset, (atom_index, token_hash)) in context_hashes[start..]
        .iter()
        .copied()
        .enumerate()
        .rev()
        .enumerate()
    {
        let position = (offset / atoms_per_token.max(1)) as u64 + 1;
        let recency = 1.0 / (position as f32).sqrt();
        let lane_weight = if atoms_per_token > 1 && atom_index % atoms_per_token != 0 {
            role_weight
        } else {
            1.0
        };
        let semantic_state = (mode != ContextPhaseMode::NoSemanticState)
            .then(|| semantic_lookup(atom_index, token_hash))
            .flatten();
        let (surface_weight, semantic_weight) = semantic_state
            .map(|(_, support)| semantic_relation_weights(support))
            .unwrap_or((1.0, 0.0));
        add_hashed_atom(
            &mut vector,
            token_hash ^ 0x0043_4f4e_5445_5854,
            position ^ token_hash.rotate_left(13),
            recency * surface_weight * lane_weight,
        );
        if let Some((center, _)) = semantic_state {
            add_rotated_vector(
                &mut vector,
                center,
                position ^ 0x0053_454d_414e_5449,
                recency * semantic_weight,
            );
            if atoms_per_token > 1 && atom_index % atoms_per_token == 0 {
                add_rotated_vector(
                    &mut vector,
                    center,
                    0x0047_4c4f_4241_4c53,
                    semantic_weight * 0.45,
                );
            }
        }
    }
    phase_center_from_sum(&vector)
}

fn merge_candidate_profile(
    target: &mut Vec<ContextCandidateProfile>,
    incoming: ContextCandidateProfile,
    limit: usize,
) {
    if let Some(existing) = target
        .iter_mut()
        .find(|profile| profile.token_hash == incoming.token_hash)
    {
        existing.positive_examples = existing
            .positive_examples
            .saturating_add(incoming.positive_examples);
        existing.negative_examples = existing
            .negative_examples
            .saturating_add(incoming.negative_examples);
        existing.threshold_micro = existing.threshold_micro.max(incoming.threshold_micro);
        existing.positive.extend(incoming.positive);
        existing.negative.extend(incoming.negative);
        existing.hard_negative.extend(incoming.hard_negative);
    } else if target.len() < limit {
        target.push(incoming);
    }
}

fn merge_pair_bank(target: &mut Vec<PhaseCenter>, incoming: Vec<PhaseCenter>, max_centers: usize) {
    for mut center in incoming {
        // Shard merge is cold work. Materialize only here, never on hot package
        // decode, so runtime keeps serialized phase cells compact.
        center.materialize_sum();
        let mut best: Option<(usize, f32)> = None;
        for (index, current) in target.iter_mut().enumerate() {
            current.materialize_sum();
            let coherence = vector_phase_coherence(&center.center, &current.center);
            if match best {
                Some((_, current_best)) => coherence > current_best,
                None => true,
            } {
                best = Some((index, coherence));
            }
        }
        if let Some((index, _)) =
            best.filter(|(_, coherence)| *coherence >= PAIR_CENTER_SPLIT_COHERENCE)
        {
            let current = &mut target[index];
            add_phase_vector(&mut current.sum, &center.sum);
            current.center = phase_center_from_sum(&current.sum);
            current.support = current.support.saturating_add(center.support);
        } else if target.len() < max_centers {
            target.push(center);
        }
    }
}

fn pair_graph_reaches(
    edges: &std::collections::BTreeMap<u64, std::collections::BTreeSet<u64>>,
    start: u64,
    current: u64,
    visited: &mut std::collections::BTreeSet<u64>,
) -> bool {
    if !visited.insert(current) {
        return current == start;
    }
    edges.get(&current).is_some_and(|next| {
        next.iter()
            .any(|value| pair_graph_reaches(edges, start, *value, visited))
    })
}

fn survivor_ranking(
    candidates: &[&str],
    readouts: &[ContextPhaseReadout],
    dominance: &PairwiseDominance,
) -> Vec<(usize, i64)> {
    let mut unique_tokens = std::collections::BTreeMap::<u64, (usize, i64)>::new();
    for (index, (candidate, readout)) in candidates.iter().zip(readouts).enumerate() {
        let hash = candidate_token_hash(candidate);
        // A dominated candidate is absent from the survivor ranking. Giving it
        // a synthetic zero would let a vetoed negative-margin candidate win
        // over a valid negative-margin survivor.
        // Anti-only profiles represent observed L2 competitors. They carry
        // destructive evidence but have no positive lexical state, so they
        // must never alter unary winner selection or its margin geometry.
        if readout.positive_examples >= 2 && !dominance.blocks(hash) {
            unique_tokens
                .entry(hash)
                .or_insert((index, readout.margin_micro));
        }
    }
    let mut ranked = unique_tokens.into_values().collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    ranked
}

fn reverse_pair_outcome(outcome: PairEdgeOutcome) -> PairEdgeOutcome {
    match outcome {
        PairEdgeOutcome::LowWins => PairEdgeOutcome::HighWins,
        PairEdgeOutcome::HighWins => PairEdgeOutcome::LowWins,
        other => other,
    }
}

fn remap_pair_view_outcome(
    outcome: PairEdgeOutcome,
    left_hash: u64,
    right_hash: u64,
    view_left_hash: u64,
    view_right_hash: u64,
) -> PairEdgeOutcome {
    let view_low_winner = match outcome {
        PairEdgeOutcome::LowWins => true,
        PairEdgeOutcome::HighWins => false,
        PairEdgeOutcome::Unknown | PairEdgeOutcome::Tie | PairEdgeOutcome::Conflict => {
            return outcome;
        }
    };
    let left_wins = if view_low_winner {
        view_left_hash < view_right_hash
    } else {
        view_left_hash > view_right_hash
    };
    if (left_hash < right_hash) == left_wins {
        PairEdgeOutcome::LowWins
    } else {
        PairEdgeOutcome::HighWins
    }
}

/// Generalized banks are oriented by compact L2 state signatures. Dominance
/// storage is still oriented by lexical hash for deterministic graph keys, so
/// this maps the learned signature-side winner back to that graph axis.
fn remap_relation_outcome(
    outcome: PairEdgeOutcome,
    left_hash: u64,
    right_hash: u64,
    left_signature: u64,
    right_signature: u64,
) -> PairEdgeOutcome {
    let signature_low_winner = match outcome {
        PairEdgeOutcome::LowWins => true,
        PairEdgeOutcome::HighWins => false,
        PairEdgeOutcome::Unknown | PairEdgeOutcome::Tie | PairEdgeOutcome::Conflict => {
            return outcome
        }
    };
    let left_wins = if signature_low_winner {
        left_signature < right_signature
    } else {
        left_signature > right_signature
    };
    let lexical_low_wins = if left_hash < right_hash {
        left_wins
    } else {
        !left_wins
    };
    if lexical_low_wins {
        PairEdgeOutcome::LowWins
    } else {
        PairEdgeOutcome::HighWins
    }
}

fn pairwise_scene(scene: &[PhaseCell], mode: ContextPhaseMode) -> Option<Vec<PhaseCell>> {
    if scene.is_empty() {
        return None;
    }
    match mode {
        ContextPhaseMode::ShuffledPairScene => {
            let width = scene.len();
            Some(
                (0..width)
                    .map(|index| scene[(index.wrapping_mul(17).wrapping_add(11)) % width])
                    .collect(),
            )
        }
        ContextPhaseMode::MagnitudeOnlyPairwise => Some(
            scene
                .iter()
                .map(|cell| PhaseCell {
                    re: cell.re.hypot(cell.im),
                    im: 0.0,
                })
                .collect(),
        ),
        _ => None,
    }
}

fn strongest_center(vector: &[PhaseCell], centers: &[PhaseCenter]) -> Option<(f32, u32)> {
    strongest_center_with_min_support(vector, centers, 1)
}

fn strongest_center_with_min_support(
    vector: &[PhaseCell],
    centers: &[PhaseCenter],
    minimum_support: u32,
) -> Option<(f32, u32)> {
    centers
        .iter()
        .filter(|center| center.support >= minimum_support)
        .map(|center| (center.coherence(vector), center.support))
        .max_by(|left, right| {
            left.0
                .total_cmp(&right.0)
                .then_with(|| left.1.cmp(&right.1))
        })
}

fn bank_support(centers: &[PhaseCenter]) -> u32 {
    centers
        .iter()
        .fold(0_u32, |total, center| total.saturating_add(center.support))
}

fn strongest_directional_center(
    vector: &[PhaseCell],
    centers: &[PhaseCenter],
) -> Option<(f32, u32)> {
    (bank_support(centers) >= MIN_DIRECTIONAL_PAIR_SUPPORT)
        .then(|| strongest_center_with_min_support(vector, centers, 1))
        .flatten()
}

fn directional_evidence_margin(
    threshold: f32,
    local_support: u32,
    directional_support: u32,
) -> f32 {
    // The learned pair relation earns a narrower uncertainty band only when
    // both its current phase mode and its direction have repeated. The fourth
    // root prevents a large bank from overpowering a weak local subcenter.
    let evidence = (local_support.max(1) as f32 * directional_support.max(1) as f32)
        .sqrt()
        .sqrt();
    threshold * (1.0 + 1.0 / evidence)
}

fn candidate_token_hash(candidate: &str) -> u64 {
    let token = crate::word_reader::last_text_word(candidate).unwrap_or_default();
    hash_text(&token.to_lowercase())
}

fn relation_role_candidate(candidate: &str) -> bool {
    let token = crate::word_reader::last_text_word(candidate).unwrap_or_default();
    token.chars().count() == 1 && token.chars().all(char::is_alphabetic)
}

fn relation_script_mismatch(context_tokens: &[String], candidate: &str) -> bool {
    if !relation_role_candidate(candidate) {
        return false;
    }
    let (mut cyrillic, mut latin) = (0_usize, 0_usize);
    for character in context_tokens
        .iter()
        .filter(|token| !sentence::is_sentence_marker(token))
        .flat_map(|token| token.chars())
    {
        if matches!(character, '\u{0400}'..='\u{052f}') {
            cyrillic += 1;
        } else if character.is_ascii_alphabetic() {
            latin += 1;
        }
    }
    let candidate = crate::word_reader::last_text_word(candidate).unwrap_or_default();
    let candidate_is_cyrillic = candidate
        .chars()
        .all(|character| matches!(character, '\u{0400}'..='\u{052f}'));
    let candidate_is_latin = candidate
        .chars()
        .all(|character| character.is_ascii_alphabetic());
    (candidate_is_cyrillic && latin > cyrillic) || (candidate_is_latin && cyrillic > latin)
}

/// A bounded observation of the candidate's L2 field and terminal shape.
/// The final two graphemes are a morphology projection, not lexical identity:
/// forms such as `проверки` and `перезагрузки` can share contextual evidence
/// without retaining either word at runtime.
pub(super) fn candidate_l2_signature(candidate: &str) -> u64 {
    candidate_l2_signature_for_schema(candidate, SIGNATURE_SCHEMA_MORPHOLOGY_PHASE)
}

fn candidate_l2_signature_for_schema(candidate: &str, schema: u32) -> u64 {
    let token = crate::word_reader::last_text_word(candidate)
        .unwrap_or_default()
        .to_lowercase();
    let readout = crate::nanda_wave::l2::l2_surface_phase_readout(&token);
    let coherence_band = (readout.coherence_milli() / 125).min(8) as u64;
    let packed = u64::from(readout.exact_center)
        | ((readout.l1_refs.min(31) as u64) << 1)
        | ((readout.motif_refs.min(31) as u64) << 6)
        | ((readout.covered_l1_refs.min(31) as u64) << 11)
        | ((readout.residual_l1_refs.min(31) as u64) << 16)
        | (coherence_band << 21);
    if schema == SIGNATURE_SCHEMA_LEGACY {
        return legacy_candidate_l2_signature(&token, packed);
    }
    let ending = token
        .chars()
        .rev()
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    if schema == SIGNATURE_SCHEMA_MORPHOLOGY_ENDING {
        return mix64_golden(packed ^ hash_text(&ending).rotate_left(17) ^ 0x004c_334d_4f52_5048);
    }
    // Ending alone aliases too many unrelated forms. A tiny phase class keeps
    // the transfer morphological while separating incompatible L1/L2 shapes.
    let surface_class = compact_surface_phase_class(&token);
    mix64_golden(
        packed
            ^ hash_text(&ending).rotate_left(17)
            ^ surface_class.rotate_left(31)
            ^ 0x004c_334d_4f52_5048,
    )
}

fn legacy_candidate_l2_signature(token: &str, packed: u64) -> u64 {
    mix64_golden(
        packed ^ compact_surface_phase_class(token).rotate_left(17) ^ 0x004c_325f_5354_4154,
    )
}

fn compact_surface_phase_class(token: &str) -> u64 {
    let mut spectrum = [0_i16; 4];
    for atom in SurfaceFieldEncoder::encode(token).atoms() {
        for trit in surface_atom_projection(atom.position, &atom.bytes) {
            let bucket = usize::from(trit.lane & 3);
            spectrum[bucket] = spectrum[bucket].saturating_add(i16::from(trit.value));
        }
    }
    let mut strongest = spectrum
        .iter()
        .enumerate()
        .map(|(index, value)| (value.unsigned_abs(), index as u64, *value >= 0))
        .collect::<Vec<_>>();
    strongest.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    strongest.into_iter().take(2).enumerate().fold(
        0_u64,
        |value, (ordinal, (magnitude, lane, positive))| {
            value
                ^ ((lane | (u64::from(positive) << 2) | (u64::from(magnitude.min(3)) << 3))
                    .rotate_left((ordinal * 7) as u32))
        },
    )
}

fn context_tokens_for_authority(package: &ContextPhasePackage, context_tokens: &[String]) -> usize {
    let sentence_encoder_known = package
        .semantic_state(sentence::SentenceContextScene::anchor_hash())
        .is_some_and(|state| state.support >= 2);
    context_tokens
        .iter()
        .filter(|token| {
            (sentence_encoder_known && sentence::is_sentence_structural_marker(token))
                || package.semantic_state(context_exact_hash(token)).is_some()
                || (package.signature_schema >= SIGNATURE_SCHEMA_RELATION_ROLES
                    && package
                        .semantic_state(context_role_hash(token))
                        .is_some_and(|state| state.support >= 2))
        })
        .count()
}

pub(super) fn context_exact_hash(token: &str) -> u64 {
    if token.chars().any(char::is_uppercase) {
        hash_text(&token.to_lowercase())
    } else {
        hash_text(token)
    }
}

pub(super) fn pair_view_hash(value: u64, view_index: usize) -> u64 {
    mix64_golden(
        value
            ^ 0x4c33_5041_4952_5657_u64
            ^ (view_index as u64 + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15),
    )
    .max(1)
}

pub(super) fn context_atom_hashes(tokens: &[String], schema: u32) -> Vec<u64> {
    let start = tokens.len().saturating_sub(MAX_CONTEXT_TOKENS);
    let tokens = &tokens[start..];
    let atoms_per_token = if schema >= SIGNATURE_SCHEMA_RELATION_ROLES {
        2
    } else {
        1
    };
    let mut hashes = Vec::with_capacity(tokens.len() * atoms_per_token);
    for token in tokens {
        hashes.push(context_exact_hash(token));
        if schema >= SIGNATURE_SCHEMA_RELATION_ROLES {
            hashes.push(context_role_hash(token));
        }
    }
    hashes
}

pub(super) fn context_role_hash(token: &str) -> u64 {
    let core = token.trim_matches(|ch: char| {
        ch.is_ascii_punctuation() || matches!(ch, '«' | '»' | '“' | '”' | '„' | '…')
    });
    let has_cyrillic = core.chars().any(crate::keyboard::is_cyrillic_letter);
    let has_ascii_alpha = core.chars().any(|ch| ch.is_ascii_alphabetic());
    let has_digit = core.chars().any(|ch| ch.is_ascii_digit());
    let class = if has_cyrillic && has_ascii_alpha {
        8
    } else if has_cyrillic {
        let letters = core
            .chars()
            .filter(|ch| crate::keyboard::is_cyrillic_letter(*ch))
            .count();
        if letters == 1 {
            1
        } else if letters <= 3 {
            2
        } else {
            3
        }
    } else if crate::word_recognizer::is_ascii_titlecase_token(core) {
        4
    } else if crate::word_recognizer::is_ascii_technical_or_brand_token(core) {
        5
    } else if has_ascii_alpha && core.is_ascii() {
        6
    } else if has_digit && core.chars().all(|ch| ch.is_ascii_digit()) {
        7
    } else {
        9
    };
    mix64_golden(0x4c33_524f_4c45_0000_u64 ^ class)
}

pub(super) fn tokenize_context_text(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|token| {
            let token = token.trim_matches(|ch: char| {
                ch.is_ascii_punctuation() || matches!(ch, '«' | '»' | '“' | '”' | '„' | '…')
            });
            (!token.is_empty()).then(|| token.to_string())
        })
        .collect()
}

fn relation_class(token_hash: u64, margin_micro: i64) -> u64 {
    let band = ((margin_micro / 25_000).clamp(-32, 32) + 32) as u64;
    crate::stable_hash::mix64_golden(token_hash ^ band.rotate_left(19))
}

pub(crate) fn default_memory_path() -> PathBuf {
    env::var_os("LAY_NANDA_L3_CONTEXT_MEMORY")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".local/share/lay/nanda_wave/l3_context_phase.nwpc")
        })
}

pub(crate) fn default_manifest_path() -> PathBuf {
    env::var_os("LAY_NANDA_L3_CONTEXT_MANIFEST")
        .map(PathBuf::from)
        .unwrap_or_else(|| default_memory_path().with_extension("runtime.json"))
}

static DEFAULT_MEMORY: OnceLock<RwLock<Arc<L3CompositeMemory>>> = OnceLock::new();
static DEFAULT_MEMORY_WARM: AtomicBool = AtomicBool::new(false);
static DEFAULT_MEMORY_REFRESH_CHECK_MS: AtomicU64 = AtomicU64::new(0);
static DEFAULT_MEMORY_REFRESH_IN_FLIGHT: AtomicBool = AtomicBool::new(false);

pub(crate) fn warm_default_memory() {
    let _ = default_memory_lock();
}

pub(crate) fn default_memory_is_warm() -> bool {
    DEFAULT_MEMORY_WARM.load(Ordering::Acquire)
}

fn load_default_composite() -> L3CompositeMemory {
    let manifest_path = default_manifest_path();
    if manifest_path.is_file() {
        if let Ok(memory) = L3CompositeMemory::load_manifest(&manifest_path) {
            return memory;
        }
    }
    L3CompositeMemory::from_package(&default_memory_path())
        .unwrap_or_else(|_| L3CompositeMemory::empty(default_memory_path()))
}

fn default_memory_lock() -> &'static RwLock<Arc<L3CompositeMemory>> {
    DEFAULT_MEMORY.get_or_init(|| {
        let memory = load_default_composite();
        DEFAULT_MEMORY_WARM.store(true, Ordering::Release);
        RwLock::new(Arc::new(memory))
    })
}

pub(crate) fn with_default_memory<T>(read: impl FnOnce(&ContextPhasePackage) -> T) -> T {
    maybe_reload_default_memory();
    let memory = default_memory_lock()
        .read()
        .unwrap_or_else(|error| error.into_inner());
    read(memory.package())
}

fn maybe_reload_default_memory() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64;
    let previous = DEFAULT_MEMORY_REFRESH_CHECK_MS.load(Ordering::Relaxed);
    if now.saturating_sub(previous) < 1_000
        || DEFAULT_MEMORY_REFRESH_CHECK_MS
            .compare_exchange(previous, now, Ordering::AcqRel, Ordering::Relaxed)
            .is_err()
    {
        return;
    }
    let manifest_path = default_manifest_path();
    let Ok(stamp) = composite::file_stamp(&manifest_path) else {
        return;
    };
    {
        let current = default_memory_lock()
            .read()
            .unwrap_or_else(|error| error.into_inner());
        if current.manifest_stamp == stamp {
            return;
        }
    }
    if DEFAULT_MEMORY_REFRESH_IN_FLIGHT
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        .is_err()
    {
        return;
    }
    if std::thread::Builder::new()
        .name("lay-l3-memory-refresh".to_string())
        .spawn(move || {
            struct RefreshGuard;
            impl Drop for RefreshGuard {
                fn drop(&mut self) {
                    DEFAULT_MEMORY_REFRESH_IN_FLIGHT.store(false, Ordering::Release);
                }
            }
            let _guard = RefreshGuard;
            let Ok(memory) = L3CompositeMemory::load_manifest(&manifest_path) else {
                return;
            };
            let mut current = default_memory_lock()
                .write()
                .unwrap_or_else(|error| error.into_inner());
            if current.manifest_stamp != memory.manifest_stamp {
                *current = Arc::new(memory);
            }
        })
        .is_err()
    {
        DEFAULT_MEMORY_REFRESH_IN_FLIGHT.store(false, Ordering::Release);
    }
}

pub(crate) fn reload_default_memory() -> io::Result<serde_json::Value> {
    let manifest_path = default_manifest_path();
    let memory = if manifest_path.is_file() {
        L3CompositeMemory::load_manifest(&manifest_path)?
    } else {
        L3CompositeMemory::from_package(&default_memory_path())?
    };
    let report = memory.report();
    let mut current = default_memory_lock()
        .write()
        .unwrap_or_else(|error| error.into_inner());
    *current = Arc::new(memory);
    Ok(report)
}

pub(crate) fn readout_default_candidates(
    original: &str,
    replacements: &[&str],
) -> Vec<ContextPhaseReadout> {
    with_default_memory(|package| readout_candidates_with_package(package, original, replacements))
}

/// Scores context-preserving candidate surfaces against one explicit package.
///
/// The default runtime uses this with the installed package, while proof code
/// can bind the package it is asserting. This prevents a test from silently
/// reading a different user-local memory file.
pub(crate) fn readout_candidates_with_package(
    package: &ContextPhasePackage,
    original: &str,
    replacements: &[&str],
) -> Vec<ContextPhaseReadout> {
    if let Some(projection) = sentence::project_candidate_lattice(original, replacements) {
        let valid_tokens = projection
            .candidates
            .iter()
            .filter_map(Option::as_deref)
            .collect::<Vec<_>>();
        let mut valid_readouts = package
            .score_sentence_candidates(&projection.scene, &valid_tokens)
            .into_iter();
        return projection
            .candidates
            .into_iter()
            .map(|token| {
                token
                    .map(|_| valid_readouts.next().unwrap_or_default())
                    .unwrap_or_default()
            })
            .collect();
    }
    let original_tokens = if package.signature_schema >= SIGNATURE_SCHEMA_RELATION_ROLES {
        tokenize_context_text(original)
    } else {
        super::llmwave::tokenize(original)
    };
    let mut context = original_tokens.clone();
    context.pop();
    let relation_schema = package.signature_schema >= SIGNATURE_SCHEMA_RELATION_ROLES;
    let candidate_tokens = replacements
        .iter()
        .map(|replacement| {
            context_preserving_candidate_token(&context, replacement, relation_schema)
        })
        .collect::<Vec<_>>();
    let valid_tokens = candidate_tokens
        .iter()
        .filter_map(Option::as_deref)
        .collect::<Vec<_>>();
    let mut valid_readouts = package
        .score_candidates(&context, &valid_tokens)
        .into_iter();
    candidate_tokens
        .into_iter()
        .map(|token| {
            token
                .map(|_| valid_readouts.next().unwrap_or_default())
                .unwrap_or_default()
        })
        .collect()
}

fn context_preserving_candidate_token(
    context: &[String],
    replacement: &str,
    relation_schema: bool,
) -> Option<String> {
    let tokens = if relation_schema {
        tokenize_context_text(replacement)
    } else {
        super::llmwave::tokenize(replacement)
    };
    let (candidate, prefix) = tokens.split_last()?;
    if prefix.is_empty() || prefix == context {
        Some(candidate.clone())
    } else {
        None
    }
}

pub(crate) fn package_report(path: &Path) -> serde_json::Value {
    match read_package(path) {
        Ok(package) => serde_json::json!({
            "kind": "l3_context_phase_package",
            "path": path,
            "loaded": true,
            "raw_words_stored": false,
            "cells": CELLS,
            "signature_schema": package.signature_schema,
            "semantic_states": package.semantic_states.len(),
            "candidate_profiles": package.profiles.len(),
            "pair_profiles": package.pair_profiles.len(),
            "pair_centers": package.pair_profiles.iter().map(|profile| profile.low_wins.len() + profile.high_wins.len() + profile.hard_low_wins.len() + profile.hard_high_wins.len()).sum::<usize>(),
            "positive_centers": package.profiles.iter().map(|profile| profile.positive.len()).sum::<usize>(),
            "anti_centers": package
                .profiles
                .iter()
                .map(|profile| profile.negative.len() + profile.hard_negative.len())
                .sum::<usize>(),
            "transitions": package.transitions,
            "corpus_fragments": package.corpus_fragments,
            "global_threshold_micro": package.global_threshold_micro,
            "competition_threshold_micro": package.competition_threshold_micro,
            "pairwise_threshold_micro": package.pairwise_threshold_micro,
            "bytes": std::fs::metadata(path).map(|meta| meta.len()).unwrap_or_default(),
        }),
        Err(error) => serde_json::json!({
            "kind": "l3_context_phase_package",
            "path": path,
            "loaded": false,
            "error": error.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests;
