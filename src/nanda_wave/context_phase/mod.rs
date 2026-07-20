//! Compact learned context relation memory for L3.
//!
//! Cold text is compiled into token-state centers and candidate-specific
//! context centers. The hot package stores hashes, quantized phase vectors,
//! support and learned thresholds; it stores no raw phrase or word strings.

mod compiler;
mod format;
mod online;
mod proof;
mod stream;
mod surface_field;

use std::env;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use super::phase_field::{
    add_hashed_atom, add_phase_vector, add_rotated_vector, empty_vector, hash_text,
    phase_center_from_sum, phase_micro, vector_phase_coherence, PhaseCell, PhaseCenter,
};
use crate::lexical_surface_atoms::{surface_atom_projection, SurfaceFieldEncoder};
use crate::stable_hash::mix64_golden;

pub(crate) use compiler::{
    apply_feedback_overlay, build_feedback_corpus, compile_context_phase_reader,
    compile_context_phase_reader_with_surface_field, surface_field_from_corrections_path,
};
#[cfg(test)]
pub(crate) use compiler::{compile_context_phase, ContextPhaseCompileInput};
pub(crate) use format::{read_package, write_package};
pub(crate) use proof::build_and_prove_context_phase_path_with_surface_field;
pub(crate) use proof::{
    build_and_prove_context_phase_path, prove_context_phase_package_path, prove_context_phase_path,
};
pub(crate) use surface_field::SurfaceMutationField;

pub(crate) const MAGIC: &[u8; 8] = b"LAYL3P01";
pub(crate) const CELLS: usize = 64;
pub(crate) const MAX_CONTEXT_TOKENS: usize = 16;
const MAX_PAIR_CANDIDATES: usize = 8;
// Pairwise memory is keyed by compact hashes and bounded phase banks, never
// text. Preserve a broad relation vocabulary, but hold one phase mode per
// direction: runtime needs a compact attractor, not training-history detail.
pub(super) const MAX_EXACT_PAIR_PROFILES: usize = 65_536;
pub(super) const MAX_RELATION_PAIR_PROFILES: usize = 16_384;
pub(crate) const MAX_PAIR_PROFILES: usize = MAX_EXACT_PAIR_PROFILES + MAX_RELATION_PAIR_PROFILES;
pub(crate) const MAX_PAIR_CENTERS_PER_BANK: usize = 1;
pub(crate) const MAX_HARD_PAIR_CENTERS_PER_BANK: usize = 1;
const PAIR_CENTER_SPLIT_COHERENCE: f32 = 0.76;

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
    pub(crate) pair_profiles: Vec<ContextPairPhaseProfile>,
    pub(crate) transitions: u64,
    pub(crate) corpus_fragments: u32,
    pub(crate) global_threshold_micro: i32,
    pub(crate) competition_threshold_micro: i32,
    pub(crate) pairwise_threshold_micro: i32,
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
        let surface_count = u32::try_from(shards.len()).unwrap_or(u32::MAX);
        let required = min_surface_support.clamp(1, surface_count.max(1));
        let mut profile_surfaces = std::collections::BTreeMap::<u64, u32>::new();
        let mut pair_surfaces = std::collections::BTreeMap::<(u64, u64), u32>::new();
        for shard in &shards {
            for profile in &shard.profiles {
                *profile_surfaces.entry(profile.token_hash).or_default() += 1;
            }
            for pair in &shard.pair_profiles {
                *pair_surfaces
                    .entry((pair.low_hash, pair.high_hash))
                    .or_default() += 1;
            }
        }
        let mut merged = Self::default();
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
        self.profiles.is_empty()
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
        if self.is_empty() || context_tokens.is_empty() {
            return vec![ContextPhaseReadout::default(); candidates.len()];
        }
        let mut readouts = candidates
            .iter()
            .map(|candidate| {
                let vector = self.candidate_relation_vector(context_tokens, candidate, mode);
                self.raw_readout(&vector, candidate, mode)
            })
            .collect::<Vec<_>>();
        let context_known_tokens =
            context_tokens_for_authority(self, context_tokens).min(u16::MAX as usize) as u16;
        let context_token_count = context_tokens.len().min(u16::MAX as usize) as u16;
        for readout in &mut readouts {
            readout.context_tokens = context_token_count;
            readout.context_known_tokens = context_known_tokens;
        }

        let pairwise = if self.pair_profiles.is_empty() || !mode.pairwise_enabled() {
            PairwiseDominance::default()
        } else {
            let scene = self.context_vector(context_tokens, mode);
            let mut pair_candidates = std::collections::BTreeMap::<u64, (i64, u64)>::new();
            for (candidate, readout) in candidates.iter().zip(&readouts) {
                let hash = candidate_token_hash(candidate);
                let signature = candidate_l2_signature(candidate);
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
            self.pairwise_dominance(&scene, &pair_lattice, mode)
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
            (pairwise_certified && readout.positive_examples >= 2 && !provisional_conflict)
                || (unary_competition_ready
                    && readout.positive_examples >= 2
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
            let context_ready = readout.context_known_tokens >= 2
                && usize::from(readout.context_known_tokens) * 2
                    >= usize::from(readout.context_tokens);
            let anti_margin_ready = readout.anti_micro.saturating_sub(readout.positive_micro)
                >= i64::from(self.competition_threshold_micro.max(1));
            let pair_blocked = pairwise.blocks(candidate_token_hash(candidates[index]));
            let pairwise_certified = pairwise_certificate
                .is_some_and(|winner| winner == candidate_token_hash(candidates[index]));
            readout.pairwise_blocked = pair_blocked;
            readout.pairwise_certified = pairwise_certified;
            readout.pairwise_conflict = pairwise
                .conflicts
                .contains(&candidate_token_hash(candidates[index]));
            readout.pairwise_known_edges = pairwise.known_edges;
            readout.pairwise_unknown_edges = pairwise.unknown_edges;
            readout.pairwise_cycle_members = pairwise.cycle_members;
            readout.disposition =
                if is_best_token && best_support_ready && context_ready && !pair_blocked {
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

    fn pair_edge_for_profile(
        &self,
        scene: &[PhaseCell],
        profile: &ContextPairPhaseProfile,
        hard_enabled: bool,
    ) -> PairEdgeOutcome {
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
            return PairEdgeOutcome::Conflict;
        }
        // A hard bank is a counter-wave: it may remove a known false winner,
        // but support still requires the winner's unary L3 evidence below.
        if hard_low >= threshold {
            return PairEdgeOutcome::LowWins;
        }
        if hard_high >= threshold {
            return PairEdgeOutcome::HighWins;
        }
        let low = strongest_center_with_min_support(scene, &profile.low_wins, 2);
        let high = strongest_center_with_min_support(scene, &profile.high_wins, 2);
        let low_score = low.map(|(score, _)| score).unwrap_or(0.0);
        let high_score = high.map(|(score, _)| score).unwrap_or(0.0);
        if low_score <= 0.0 && high_score <= 0.0 {
            return PairEdgeOutcome::Unknown;
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
            return PairEdgeOutcome::Tie;
        }
        if low_score > high_score {
            PairEdgeOutcome::LowWins
        } else {
            PairEdgeOutcome::HighWins
        }
    }

    fn pairwise_dominance(
        &self,
        scene: &[PhaseCell],
        lattice: &[(u64, (i64, u64))],
        mode: ContextPhaseMode,
    ) -> PairwiseDominance {
        let mut dominance = PairwiseDominance::default();
        if !mode.pairwise_enabled() {
            return dominance;
        }
        let transformed_scene = pairwise_scene(scene, mode);
        let scene = transformed_scene.as_deref().unwrap_or(scene);
        let mut edges = std::collections::BTreeMap::<u64, std::collections::BTreeSet<u64>>::new();
        dominance.lattice_size = lattice.len().min(u8::MAX as usize) as u8;
        for left in 0..lattice.len() {
            for right in left + 1..lattice.len() {
                let (left_hash, (_, left_signature)) = lattice[left];
                let (right_hash, (_, right_signature)) = lattice[right];
                let outcome = self.pair_edge(
                    scene,
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

    fn raw_readout(
        &self,
        vector: &[PhaseCell],
        candidate: &str,
        mode: ContextPhaseMode,
    ) -> ContextPhaseReadout {
        let token = crate::word_reader::last_text_word(candidate).unwrap_or_default();
        let token_hash = hash_text(&token.to_lowercase());
        let Some(profile) = self.profile(token_hash) else {
            return ContextPhaseReadout {
                package_loaded: true,
                ..ContextPhaseReadout::default()
            };
        };
        if mode == ContextPhaseMode::NoPhase {
            return ContextPhaseReadout {
                package_loaded: true,
                profile_present: true,
                threshold_micro: i64::from(profile.threshold_micro),
                positive_examples: profile.positive_examples,
                negative_examples: profile.negative_examples,
                positive_centers: profile.positive.len().min(u8::MAX as usize) as u8,
                anti_centers: profile
                    .negative
                    .len()
                    .saturating_add(profile.hard_negative.len())
                    .min(u8::MAX as usize) as u8,
                semantic_support: self
                    .semantic_state(token_hash)
                    .map(|state| state.support)
                    .unwrap_or_default(),
                relation_class: relation_class(token_hash, 0),
                ..ContextPhaseReadout::default()
            };
        }
        let (positive, positive_center_support) =
            strongest_center(vector, &profile.positive).unwrap_or_default();
        let (anti, anti_center_support) = if mode == ContextPhaseMode::NoAnti {
            (0.0, 0)
        } else {
            // A generic L2 competitor says only that this word lost to some
            // other word in one scene. That relation belongs to PairKey and
            // must never become a unary veto on the word everywhere else.
            // The only candidate-local destructive authority is a witnessed
            // false winner, retained in the hard bank below.
            strongest_center(vector, &profile.hard_negative).unwrap_or_default()
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
        let hashes = context_tokens
            .iter()
            .map(|token| hash_text(token))
            .collect::<Vec<_>>();
        canonical_scene_wave(&hashes, mode, |hash| {
            self.semantic_state(hash)
                .map(|state| (state.center.as_slice(), state.support))
        })
    }

    pub(super) fn candidate_relation_vector(
        &self,
        context_tokens: &[String],
        candidate: &str,
        mode: ContextPhaseMode,
    ) -> Vec<PhaseCell> {
        let mut vector = self.context_vector(context_tokens, mode);
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
    mut semantic_lookup: F,
) -> Vec<PhaseCell>
where
    F: FnMut(u64) -> Option<(&'a [PhaseCell], u32)>,
{
    let mut vector = empty_vector(CELLS);
    let start = context_hashes.len().saturating_sub(MAX_CONTEXT_TOKENS);
    for (offset, token_hash) in context_hashes[start..].iter().rev().copied().enumerate() {
        let position = offset as u64 + 1;
        let recency = 1.0 / (position as f32).sqrt();
        let semantic_state = (mode != ContextPhaseMode::NoSemanticState)
            .then(|| semantic_lookup(token_hash))
            .flatten();
        let (surface_weight, semantic_weight) = semantic_state
            .map(|(_, support)| semantic_relation_weights(support))
            .unwrap_or((1.0, 0.0));
        add_hashed_atom(
            &mut vector,
            token_hash ^ 0x0043_4f4e_5445_5854,
            position ^ token_hash.rotate_left(13),
            recency * surface_weight,
        );
        if let Some((center, _)) = semantic_state {
            add_rotated_vector(
                &mut vector,
                center,
                position ^ 0x0053_454d_414e_5449,
                recency * semantic_weight,
            );
        }
    }
    phase_center_from_sum(&vector)
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

/// A bounded observation of the existing L2 surface field. It deliberately
/// keeps no candidate text: only center coverage, motif support and residual
/// pressure are projected into a stable relation state.
pub(super) fn candidate_l2_signature(candidate: &str) -> u64 {
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
    // A compact L1 wave spectrum prevents a broad L2 coverage class from
    // aliasing unrelated word forms. It is a lossy phase projection, not a
    // retained n-gram list or a word identity.
    let mut spectrum = [0_i16; 4];
    for atom in SurfaceFieldEncoder::encode(&token).atoms() {
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
    let surface_shape = strongest.into_iter().take(2).enumerate().fold(
        0_u64,
        |value, (ordinal, (magnitude, lane, positive))| {
            value
                ^ ((lane | (u64::from(positive) << 2) | (u64::from(magnitude.min(3)) << 3))
                    .rotate_left((ordinal * 7) as u32))
        },
    );
    mix64_golden(packed ^ surface_shape.rotate_left(17) ^ 0x004c_325f_5354_4154)
}

fn context_tokens_for_authority(package: &ContextPhasePackage, context_tokens: &[String]) -> usize {
    context_tokens
        .iter()
        .filter(|token| package.semantic_state(hash_text(token)).is_some())
        .count()
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

static DEFAULT_MEMORY: OnceLock<ContextPhasePackage> = OnceLock::new();
static DEFAULT_MEMORY_WARM: AtomicBool = AtomicBool::new(false);

pub(crate) fn warm_default_memory() {
    let _ = default_memory();
}

pub(crate) fn default_memory_is_warm() -> bool {
    DEFAULT_MEMORY_WARM.load(Ordering::Acquire)
}

pub(crate) fn default_memory() -> &'static ContextPhasePackage {
    DEFAULT_MEMORY.get_or_init(|| {
        let memory = read_package(&default_memory_path()).unwrap_or_default();
        DEFAULT_MEMORY_WARM.store(true, Ordering::Release);
        memory
    })
}

pub(crate) fn readout_default_candidates(
    original: &str,
    replacements: &[&str],
) -> Vec<ContextPhaseReadout> {
    readout_candidates_with_package(default_memory(), original, replacements)
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
    let original_tokens = super::llmwave::tokenize(original);
    let mut context = original_tokens.clone();
    context.pop();
    let candidate_tokens = replacements
        .iter()
        .map(|replacement| context_preserving_candidate_token(&context, replacement))
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

fn context_preserving_candidate_token(context: &[String], replacement: &str) -> Option<String> {
    let tokens = super::llmwave::tokenize(replacement);
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
