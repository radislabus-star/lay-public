use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::thread;
use std::time::Instant;

use crate::stable_hash::mix64_golden;
use crate::text_metrics::{damerau_levenshtein, damerau_levenshtein_bounded};

use super::anti_postings::{
    score_strength_position, AntiPostingCursor, AntiPostingIndex, TERMINAL_BLOCK_SIZE,
};
use super::atoms::{encode_wave_surface, physical_key_sequence, AtomChannel, NGramKey};
use super::crystal::{AmbiguityPhaseCenter64, WordCenter64, WAVE_DIMENSION};
use super::model::{
    AtomRecord, CenterPhaseProfile, DecoderNode, LexicalGrokkingPackage, PairKey, PairPhaseProfile,
    WaveCoupling, COUPLING_FLAG_CHARACTER_ANCHOR,
};
use super::ngram_graph::NGramGraph;
use super::posting_spool::PostingSpool;
use super::restoration::{tied_energy_winner, RestorationCalibration};
use super::runtime::{LexicalGrokkingMemory, ReadoutMode};
use super::training_budget::checkpoint;
use super::training_corpus::{TrainingCorpus, TrainingCorpusWord};
use super::wave_basis::{
    compile_basis, complex_coherence_milli, expand_atom, expand_word, pair_residual_atoms,
    positioned_atom_code, settle_word_code,
};

#[cfg(test)]
pub(super) use super::training_corpus::TrainingWord;

const MAX_FORWARD_COUPLINGS: usize = 256;
const MAX_REVERSE_LEXICAL_COUPLINGS: usize = 96;
const MAX_DIRECTIONAL_ANTI_RELATIONS: usize = 16;
const ANTI_PROGRESS_INTERVAL: usize = 1_000;
const SUBCENTER_PROGRESS_INTERVAL: usize = 1_000;
const SUBCENTER_WORK_CHUNK: usize = 512;
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
    ambiguity: Vec<WordCenter64>,
    keyboard_geometry: Vec<u32>,
}

struct DenseScoreAccumulator {
    scores: Vec<u32>,
    epochs: Vec<u32>,
    touched: Vec<u32>,
    epoch: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct AntiSearchStats {
    surfaces: u64,
    posting_entries: u64,
    posting_entries_skipped: u64,
    exact_candidates: u64,
    dense_fallbacks: u64,
    block_checks: u64,
    block_skips: u64,
}

#[derive(Default)]
struct SubcenterSearchStats {
    edit_distance_calls: AtomicU64,
}

struct WandCursor<'a> {
    postings: AntiPostingCursor<'a>,
    upper_bound: u64,
    cached_block: u32,
    cached_block_upper_bound: u64,
}

impl WandCursor<'_> {
    fn current_terminal(&self) -> Option<u32> {
        self.postings.current_terminal()
    }

    fn advance_to(&mut self, terminal_id: u32) -> usize {
        self.postings.advance_to(terminal_id)
    }

    fn consume_current(&mut self) -> u32 {
        self.postings.consume_current()
    }

    fn block_upper_bound(&mut self, block: u32) -> u64 {
        if self.cached_block != block {
            self.cached_block = block;
            self.cached_block_upper_bound = self.postings.block_upper_bound(block);
        }
        self.cached_block_upper_bound
    }

    fn score_terminal(&self, terminal_id: u32) -> Option<u32> {
        self.postings.score_terminal(terminal_id)
    }
}

impl DenseScoreAccumulator {
    fn new(terminal_count: usize) -> Self {
        Self {
            scores: vec![0; terminal_count],
            epochs: vec![0; terminal_count],
            touched: Vec::new(),
            epoch: 0,
        }
    }

    fn begin_surface(&mut self) {
        self.touched.clear();
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.epochs.fill(0);
            self.epoch = 1;
        }
    }

    fn add(&mut self, terminal_id: u32, contribution: u32) {
        let index = terminal_id as usize;
        if self.epochs[index] != self.epoch {
            self.epochs[index] = self.epoch;
            self.scores[index] = 0;
            self.touched.push(terminal_id);
        }
        self.scores[index] = self.scores[index].wrapping_add(contribution);
    }

    fn get(&self, terminal_id: u32) -> Option<u32> {
        let index = terminal_id as usize;
        (self.epochs.get(index).copied() == Some(self.epoch)).then(|| self.scores[index])
    }

    fn top_competitors(&self, target: u32, minimum_score: u32, limit: usize) -> Vec<(u32, u32)> {
        let mut top = Vec::with_capacity(limit);
        for terminal_id in self.touched.iter().copied() {
            if terminal_id == target {
                continue;
            }
            let score = self.scores[terminal_id as usize];
            if score < minimum_score {
                continue;
            }
            let position = top
                .iter()
                .position(|(other_terminal, other_score)| {
                    score > *other_score || (score == *other_score && terminal_id < *other_terminal)
                })
                .unwrap_or(top.len());
            if position < limit {
                top.insert(position, (terminal_id, score));
                top.truncate(limit);
            }
        }
        top
    }
}

#[cfg(test)]
mod dense_score_tests {
    use super::{
        accumulate_forward_scores, bounded_anti_surface_indices, wand_top_competitors,
        wand_top_lattice_competitors, AntiPostingIndex, AntiSearchStats, DenseScoreAccumulator,
        ResolvedSurface,
    };
    use crate::nanda_wave::lexical_grokking::atoms::AtomChannel;
    use crate::nanda_wave::lexical_grokking::model::{AtomRecord, WaveCoupling};
    use std::collections::BTreeMap;

    #[test]
    fn dense_scores_preserve_reference_top_k_and_tie_break() {
        let contributions = [
            (0, 10_u32),
            (3, 8),
            (2, 20),
            (1, 7),
            (1, 13),
            (4, 9),
            (3, 12),
        ];
        let mut reference = BTreeMap::<u32, u32>::new();
        let mut dense = DenseScoreAccumulator::new(5);
        dense.begin_surface();
        for (terminal_id, contribution) in contributions {
            *reference.entry(terminal_id).or_default() += contribution;
            dense.add(terminal_id, contribution);
        }

        let target_score = reference[&0];
        let mut expected = reference
            .into_iter()
            .filter(|(terminal_id, score)| *terminal_id != 0 && *score >= target_score)
            .collect::<Vec<_>>();
        expected.sort_unstable_by(|left, right| {
            right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0))
        });
        expected.truncate(3);

        assert_eq!(dense.get(0), Some(target_score));
        assert_eq!(dense.top_competitors(0, target_score, 3), expected);
    }

    #[test]
    fn dense_scores_reset_by_epoch_without_clearing_the_arrays() {
        let mut dense = DenseScoreAccumulator::new(4);
        dense.begin_surface();
        dense.add(0, 10);
        dense.add(1, 20);
        assert_eq!(dense.top_competitors(0, 10, 4), vec![(1, 20)]);

        dense.begin_surface();
        dense.add(2, 30);
        assert_eq!(dense.get(0), None);
        assert_eq!(dense.get(1), None);
        assert_eq!(dense.get(2), Some(30));
    }

    #[test]
    fn dense_score_epoch_wrap_does_not_revive_stale_scores() {
        let mut dense = DenseScoreAccumulator::new(2);
        dense.begin_surface();
        dense.add(0, 10);
        dense.epoch = u32::MAX;

        dense.begin_surface();
        dense.add(1, 20);
        assert_eq!(dense.get(0), None);
        assert_eq!(dense.get(1), Some(20));
    }

    #[test]
    fn bounded_anti_surface_sampling_is_deterministic_and_covers_the_global_modes() {
        assert_eq!(bounded_anti_surface_indices(7, 0, 1), Vec::<usize>::new());
        assert_eq!(
            bounded_anti_surface_indices(7, 4, usize::MAX),
            vec![0, 1, 2, 3]
        );
        assert_eq!(
            bounded_anti_surface_indices(91, 13, 1),
            bounded_anti_surface_indices(91, 13, 1)
        );

        let covered = (0..1_000_u32)
            .flat_map(|target| bounded_anti_surface_indices(target, 13, 1))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(covered, (0..13).collect());
    }

    #[test]
    fn wand_matches_dense_oracle_across_targets_and_surface_positions() {
        let terminal_count = 257_u32;
        let mut atoms = Vec::new();
        let mut forward = Vec::new();
        let mut maximum_strengths = Vec::new();
        for atom_id in 0..7_u32 {
            let start = forward.len();
            let mut maximum_strength = 0_u8;
            for terminal_id in 0..terminal_count {
                if (terminal_id + atom_id * 3) % (atom_id + 3) == 0 {
                    continue;
                }
                let strength = 1 + ((terminal_id * 17 + atom_id * 29) % 251) as u8;
                maximum_strength = maximum_strength.max(strength);
                forward.push(WaveCoupling {
                    peer_id: terminal_id,
                    strength,
                    phase_relation: 0,
                    position_mode: ((terminal_id * 11 + atom_id * 19) % 256) as u8,
                    flags: 0,
                });
            }
            atoms.push(AtomRecord {
                coupling_start: start as u32,
                coupling_count: (forward.len() - start) as u32,
                ..AtomRecord::default()
            });
            maximum_strengths.push(maximum_strength);
        }

        let mut block_skips = 0_u64;
        let anti_postings = AntiPostingIndex::build(&atoms, &forward).unwrap();
        for position_bias in [0_u16, 5_000, 30_000, 60_000] {
            let surface = ResolvedSurface {
                atoms: (0..7_u32)
                    .map(|atom_id| {
                        (
                            atom_id,
                            position_bias.saturating_add(atom_id as u16 * 311),
                            1 + (atom_id % 3) as u8,
                            AtomChannel::CharacterGram,
                        )
                    })
                    .collect(),
                physical_keys: Vec::new(),
            };
            for target in 0..terminal_count {
                let mut dense = DenseScoreAccumulator::new(terminal_count as usize);
                accumulate_forward_scores(&surface, &atoms, &forward, &mut dense);
                let expected = dense
                    .get(target)
                    .map(|score| dense.top_competitors(target, score, 4))
                    .unwrap_or_default();
                let mut stats = AntiSearchStats::default();
                let actual = wand_top_competitors(
                    &surface,
                    target,
                    &maximum_strengths,
                    &anti_postings,
                    &mut stats,
                )
                .expect("bounded test scores cannot overflow");
                block_skips = block_skips.saturating_add(stats.block_skips);
                assert_eq!(
                    actual, expected,
                    "target={target} position_bias={position_bias}"
                );

                let expected_lattice = dense.top_competitors(target, 0, 17);
                let mut lattice_stats = AntiSearchStats::default();
                let actual_lattice = wand_top_lattice_competitors(
                    &surface,
                    target,
                    &maximum_strengths,
                    &anti_postings,
                    17,
                    &mut lattice_stats,
                )
                .expect("bounded test scores cannot overflow");
                assert_eq!(
                    actual_lattice, expected_lattice,
                    "lattice target={target} position_bias={position_bias}"
                );
            }
        }
        assert!(
            block_skips > 0,
            "oracle fixture must exercise block skipping"
        );
    }

    #[test]
    fn wand_falls_back_when_wrapping_scores_break_monotonic_bounds() {
        let atoms = vec![
            AtomRecord {
                coupling_start: 0,
                coupling_count: 1,
                ..AtomRecord::default()
            };
            300
        ];
        let forward = vec![
            WaveCoupling {
                peer_id: 0,
                strength: u8::MAX,
                phase_relation: 0,
                position_mode: 0,
                flags: 0,
            };
            300
        ];
        let surface = ResolvedSurface {
            atoms: (0..300_u32)
                .map(|atom_id| (atom_id, 0, u8::MAX, AtomChannel::CharacterGram))
                .collect(),
            physical_keys: Vec::new(),
        };
        let mut stats = AntiSearchStats::default();
        let anti_postings = AntiPostingIndex::build(&atoms, &forward).unwrap();
        assert!(
            wand_top_competitors(&surface, 0, &vec![u8::MAX; 300], &anti_postings, &mut stats,)
                .is_err()
        );
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct CouplingRange {
    start: u32,
    count: u16,
}

struct PostingCompileShard {
    start_word: usize,
    spool: PostingSpool,
    reverse_couplings: Vec<WaveCoupling>,
    reverse_ranges: Vec<CouplingRange>,
}

#[cfg(test)]
pub(super) fn compile(words: &[TrainingWord]) -> Result<LexicalGrokkingPackage, String> {
    Ok(compile_with_policy(words, ForwardPostingPolicy::BaselineBounded)?.package)
}

#[cfg(test)]
pub(super) fn compile_with_policy(
    words: &[TrainingWord],
    forward_policy: ForwardPostingPolicy,
) -> Result<CompileOutput, String> {
    let corpus = TrainingCorpus::from_words(words)?;
    compile_training_corpus_with_policy(&corpus, forward_policy)
}

#[cfg(test)]
pub(super) fn compile_training_corpus_with_policy(
    corpus: &TrainingCorpus,
    forward_policy: ForwardPostingPolicy,
) -> Result<CompileOutput, String> {
    compile_training_corpus_with_policy_in(
        corpus,
        forward_policy,
        &std::env::temp_dir().join("lay-l11-training"),
    )
}

pub(super) fn compile_training_corpus_with_policy_in(
    corpus: &TrainingCorpus,
    forward_policy: ForwardPostingPolicy,
    workspace: &Path,
) -> Result<CompileOutput, String> {
    checkpoint("compiler_validate")?;
    validate_corpus(corpus)?;
    let words = corpus.words();
    checkpoint("ngram_graph_collect")?;
    let graph = compile_graph(corpus)?;
    checkpoint("ngram_graph_compiled")?;
    checkpoint("atom_support")?;
    let (atom_flags, atom_word_support) = compile_atom_support(corpus, &graph)?;
    checkpoint("atom_support_complete")?;

    checkpoint("posting_spool")?;
    let (posting_spool, reverse_couplings, reverse_ranges) =
        compile_posting_spool(corpus, &graph, &atom_flags, &atom_word_support, workspace)?;
    checkpoint("posting_spool_complete")?;
    let materialized = posting_spool.materialize(
        &atom_word_support,
        (forward_policy == ForwardPostingPolicy::BaselineBounded).then_some(MAX_FORWARD_COUPLINGS),
    )?;
    checkpoint("posting_materialized")?;
    let diagnostics = CompileDiagnostics {
        forward_relations_before_policy: materialized.relations_before_policy,
        forward_relations_dropped: materialized.relations_dropped,
        forward_atoms_above_baseline_cap: materialized.atoms_above_policy_cap,
        max_forward_degree: materialized.max_forward_degree,
    };
    let atoms = materialized.atoms;
    let forward_couplings = materialized.forward_couplings;
    let maximum_forward_strengths = materialized.maximum_strengths;
    let anti_postings = AntiPostingIndex::build(&atoms, &forward_couplings)?;
    eprintln!(
        "l11_anti_postings blocks={} payloads={} resident_bytes={}",
        anti_postings.block_count(),
        anti_postings.payload_count(),
        anti_postings.resident_bytes(),
    );
    checkpoint("anti_postings_compiled")?;

    let (decoder_nodes, decoder_terminals) = compile_decoder(words)?;
    let basis = compile_basis();
    let mut centers = Vec::with_capacity(words.len());
    for (word_id, range) in reverse_ranges.iter().copied().enumerate() {
        let couplings = coupling_slice(&reverse_couplings, range);
        let mut center = WordCenter64 {
            coupling_start: range.start,
            coupling_count: range.count,
            crystal_support: 1,
            stability: stability(1),
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
    checkpoint("primary_centers_complete")?;

    checkpoint("anti_center_discovery")?;
    let all_anti_by_word = discover_anti_centers(
        corpus,
        &graph,
        &atoms,
        &forward_couplings,
        &maximum_forward_strengths,
        &anti_postings,
    );
    checkpoint("anti_centers_complete")?;
    let mut anti_by_word = all_anti_by_word
        .iter()
        .map(|bank| {
            bank.iter()
                .copied()
                .filter(|center| center.crystal_support >= 2)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let ambiguous_owners = corpus.ambiguous_surface_owners();
    let l11 = compile_l11_subcenters(
        corpus,
        &graph,
        &atoms,
        &reverse_couplings,
        &reverse_ranges,
        &all_anti_by_word,
        &ambiguous_owners,
    );
    checkpoint("l11_subcenters_complete")?;
    if let Some(target_limit) = shadow_subcenter_target_limit(words.len()) {
        return Err(format!(
            "L1.1 shadow subcenter probe complete: targets={target_limit}; package intentionally not emitted"
        ));
    }
    let (pair_profiles, pair_centers) = compile_pair_profiles(&anti_by_word);
    let mut anti_centers = Vec::new();
    for (word_id, anti) in anti_by_word.iter_mut().enumerate() {
        anti.truncate(4);
        centers[word_id].anti_start = anti_centers.len() as u32;
        centers[word_id].anti_count = anti.len().min(u8::MAX as usize) as u8;
        anti_centers.extend_from_slice(anti);
    }
    let restoration_calibration = calibrate_l11(
        corpus,
        &graph,
        &atoms,
        &reverse_couplings,
        &reverse_ranges,
        &basis,
        &l11.profiles,
        &l11.positive,
    );
    checkpoint("restoration_calibration_complete")?;

    let package = LexicalGrokkingPackage {
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
        ambiguity_subcenters: l11.ambiguity,
        keyboard_geometry_units: l11.keyboard_geometry,
        restoration_calibration,
        centers,
        decoder_nodes,
    };
    let memory = LexicalGrokkingMemory::from_package(package);
    let ambiguity_thresholds =
        calibrate_l11_ambiguity_thresholds(corpus, &ambiguous_owners, &memory);
    checkpoint("ambiguity_calibration_complete")?;
    let mut package = memory.into_package();
    for (center, threshold) in package
        .ambiguity_subcenters
        .iter_mut()
        .zip(ambiguity_thresholds)
    {
        center.coupling_count = threshold;
    }
    let memory = LexicalGrokkingMemory::from_package(package);
    let min_tied_energy_margin =
        calibrate_l11_tied_energy_margin(corpus, &ambiguous_owners, &memory);
    checkpoint("tied_margin_calibration_complete")?;
    let mut package = memory.into_package();
    package.restoration_calibration.min_tied_energy_margin = min_tied_energy_margin;

    Ok(CompileOutput {
        package,
        diagnostics,
    })
}

fn compile_posting_spool(
    corpus: &TrainingCorpus,
    graph: &NGramGraph,
    atom_flags: &[u8],
    atom_word_support: &[u32],
    workspace: &Path,
) -> Result<(PostingSpool, Vec<WaveCoupling>, Vec<CouplingRange>), String> {
    std::fs::create_dir_all(workspace).map_err(|error| {
        format!(
            "create L1 compiler workspace {}: {error}",
            workspace.display()
        )
    })?;
    let words = corpus.words();
    let workers = worker_count(words.len());
    let chunk_size = words.len().div_ceil(workers);
    eprintln!(
        "l11_posting_spool words={} atoms={} workers={workers} chunk_size={chunk_size}",
        words.len(),
        graph.atom_count
    );
    let mut shards = thread::scope(|scope| {
        words
            .chunks(chunk_size)
            .enumerate()
            .map(|(worker, chunk)| {
                let start_word = worker.saturating_mul(chunk_size);
                let worker_workspace = workspace.join(format!("worker-{worker:03}"));
                scope.spawn(move || {
                    compile_posting_shard(
                        corpus,
                        graph,
                        atom_flags,
                        atom_word_support,
                        &worker_workspace,
                        start_word,
                        chunk,
                    )
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| "L1 posting-spool worker panicked".to_string())?
            })
            .collect::<Result<Vec<_>, String>>()
    })?;
    shards.sort_unstable_by_key(|shard| shard.start_word);

    let mut reverse_couplings = Vec::new();
    let mut reverse_ranges = Vec::with_capacity(words.len());
    let mut spools = Vec::with_capacity(shards.len());
    for mut shard in shards {
        let coupling_offset = u32::try_from(reverse_couplings.len())
            .map_err(|_| "L1 reverse coupling start exceeds u32".to_string())?;
        for range in &mut shard.reverse_ranges {
            range.start = range
                .start
                .checked_add(coupling_offset)
                .ok_or_else(|| "L1 reverse coupling range exceeds u32".to_string())?;
        }
        reverse_couplings.append(&mut shard.reverse_couplings);
        reverse_ranges.append(&mut shard.reverse_ranges);
        spools.push(shard.spool);
    }
    let posting_spool = PostingSpool::merge(spools)?;
    Ok((posting_spool, reverse_couplings, reverse_ranges))
}

fn compile_posting_shard(
    corpus: &TrainingCorpus,
    graph: &NGramGraph,
    atom_flags: &[u8],
    atom_word_support: &[u32],
    workspace: &Path,
    start_word: usize,
    words: &[TrainingCorpusWord],
) -> Result<PostingCompileShard, String> {
    let total_words = corpus.words().len();
    let mut spool = PostingSpool::create(workspace, graph.atom_count as usize)?;
    let mut reverse_couplings = Vec::new();
    let mut reverse_ranges = Vec::with_capacity(words.len());
    for (offset, word) in words.iter().enumerate() {
        let word_id = start_word.saturating_add(offset);
        let clean = resolve_surface(graph, corpus.clean_surface(word));
        let mut stats_by_atom = BTreeMap::<u32, CouplingStats>::new();
        for (atom_id, position, weight, _) in clean.atoms.iter().copied() {
            let stats = stats_by_atom.entry(atom_id).or_default();
            stats.observations = stats.observations.saturating_add(1);
            stats.weighted_observations = stats
                .weighted_observations
                .saturating_add(u32::from(weight));
            stats.position_sum = stats.position_sum.saturating_add(u64::from(position));
        }
        let surface_count = 1;
        for (atom_id, stats) in &stats_by_atom {
            if atom_flags[*atom_id as usize] != 0 {
                continue;
            }
            let strength = coupling_strength(
                *stats,
                surface_count,
                atom_word_support[*atom_id as usize],
                total_words,
            );
            let position_mode = average_position(*stats);
            let phase_relation = position_phase(position_mode);
            spool.push(
                *atom_id,
                WaveCoupling {
                    peer_id: word_id as u32,
                    strength,
                    phase_relation,
                    position_mode,
                    flags: 0,
                },
            )?;
        }
        let mut reverse = Vec::new();
        for (atom_id, position, _, _) in clean.atoms.iter().copied() {
            let stats = stats_by_atom.get(&atom_id).copied().unwrap_or_default();
            let strength = coupling_strength(
                stats,
                surface_count,
                atom_word_support[atom_id as usize],
                total_words,
            );
            let position_mode = (position / 257).min(255) as u8;
            reverse.push(WaveCoupling {
                peer_id: atom_id,
                strength,
                phase_relation: position_phase(position_mode),
                position_mode,
                flags: atom_flags[atom_id as usize],
            });
        }
        reverse.sort_unstable_by(coupling_order);
        let anchor_count = reverse.iter().take_while(|item| item.flags != 0).count();
        reverse.truncate(anchor_count.saturating_add(MAX_REVERSE_LEXICAL_COUPLINGS));
        let start = u32::try_from(reverse_couplings.len())
            .map_err(|_| "L1 reverse coupling start exceeds u32".to_string())?;
        let count = u16::try_from(reverse.len())
            .map_err(|_| "L1 reverse coupling count exceeds u16".to_string())?;
        reverse_couplings.extend_from_slice(&reverse);
        reverse_ranges.push(CouplingRange { start, count });
    }
    Ok(PostingCompileShard {
        start_word,
        spool,
        reverse_couplings,
        reverse_ranges,
    })
}

#[derive(Default)]
struct AmbiguityCalibrationSamples {
    desired_fit: Vec<u16>,
    desired_calibration: Vec<u16>,
    undesired_fit: Vec<u16>,
    undesired_calibration: Vec<u16>,
}

fn calibrate_l11_ambiguity_thresholds(
    corpus: &TrainingCorpus,
    ambiguous_owners: &HashMap<&str, Vec<u32>>,
    memory: &LexicalGrokkingMemory,
) -> Vec<u16> {
    const CANDIDATE_LIMIT: usize = 64;

    let words = corpus.words();
    let workers = thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(words.len().max(1));
    let partial = thread::scope(|scope| {
        (0..workers)
            .map(|worker| {
                scope.spawn(move || {
                    let mut samples = Vec::<(usize, bool, bool, u16)>::new();
                    for word in words.iter().skip(worker).step_by(workers) {
                        for surface in corpus.training_surfaces(word) {
                            let candidates =
                                memory.readout(surface, CANDIDATE_LIMIT, ReadoutMode::Full);
                            let calibration = calibration_surface_hash(surface) % 5 == 0;
                            for observation in memory.ambiguity_observations(surface, &candidates) {
                                if !observation.structurally_applicable
                                    || !objective_contains(
                                        ambiguous_owners,
                                        surface,
                                        word.terminal_id,
                                        observation.owner,
                                    )
                                {
                                    continue;
                                }
                                samples.push((
                                    observation.center_index,
                                    objective_contains(
                                        ambiguous_owners,
                                        surface,
                                        word.terminal_id,
                                        observation.competitor,
                                    ),
                                    calibration,
                                    observation.coherence_milli,
                                ));
                            }
                        }
                    }
                    samples
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|handle| handle.join().unwrap_or_default())
            .collect::<Vec<_>>()
    });
    let mut samples = (0..memory.ambiguity_center_count())
        .map(|_| AmbiguityCalibrationSamples::default())
        .collect::<Vec<_>>();
    for (center_index, desired, calibration, coherence) in partial {
        let Some(center) = samples.get_mut(center_index) else {
            continue;
        };
        match (desired, calibration) {
            (true, true) => center.desired_calibration.push(coherence),
            (true, false) => center.desired_fit.push(coherence),
            (false, true) => center.undesired_calibration.push(coherence),
            (false, false) => center.undesired_fit.push(coherence),
        }
    }
    samples
        .into_iter()
        .map(calibrate_ambiguity_center_threshold)
        .collect()
}

fn calibrate_ambiguity_center_threshold(mut samples: AmbiguityCalibrationSamples) -> u16 {
    const UNIQUE_PRESERVATION_PERMILLE: usize = 955;

    let desired = if samples.desired_calibration.is_empty() {
        &mut samples.desired_fit
    } else {
        &mut samples.desired_calibration
    };
    if desired.is_empty() {
        return 0;
    }
    let undesired = if samples.undesired_calibration.is_empty() {
        &mut samples.undesired_fit
    } else {
        &mut samples.undesired_calibration
    };
    let threshold = if undesired.is_empty() {
        low_quantile(desired, 100).max(1)
    } else {
        let safe = high_quantile(undesired, UNIQUE_PRESERVATION_PERMILLE);
        if safe == 1_000 {
            return 0;
        }
        safe + 1
    };
    if desired.iter().any(|coherence| *coherence >= threshold) {
        threshold
    } else {
        0
    }
}

fn validate_corpus(corpus: &TrainingCorpus) -> Result<(), String> {
    let words = corpus.words();
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

fn compile_graph(corpus: &TrainingCorpus) -> Result<NGramGraph, String> {
    let words = corpus.words();
    let workers = worker_count(words.len());
    let chunk_size = words.len().div_ceil(workers);
    eprintln!(
        "l11_ngram_graph words={} workers={workers} chunk_size={chunk_size}",
        words.len()
    );
    let partial = thread::scope(|scope| {
        words
            .chunks(chunk_size)
            .map(|chunk| {
                scope.spawn(move || {
                    let mut keys = BTreeSet::<NGramKey>::new();
                    for word in chunk {
                        keys.extend(
                            encode_wave_surface(corpus.clean_surface(word))
                                .into_iter()
                                .map(|atom| atom.key),
                        );
                    }
                    keys
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| "L1 n-gram graph worker panicked".to_string())
            })
            .collect::<Result<Vec<_>, String>>()
    })?;
    let mut keys = BTreeSet::<NGramKey>::new();
    for mut shard in partial {
        keys.append(&mut shard);
    }
    NGramGraph::compile_sorted_unique(keys)
}

fn compile_atom_support(
    corpus: &TrainingCorpus,
    graph: &NGramGraph,
) -> Result<(Vec<u8>, Vec<u32>), String> {
    let words = corpus.words();
    let atom_count = graph.atom_count as usize;
    let workers = worker_count(words.len());
    let chunk_size = words.len().div_ceil(workers);
    eprintln!(
        "l11_atom_support words={} atoms={atom_count} workers={workers} chunk_size={chunk_size}",
        words.len()
    );
    let partial = thread::scope(|scope| {
        words
            .chunks(chunk_size)
            .map(|chunk| {
                scope.spawn(move || {
                    let mut flags = vec![0_u8; atom_count];
                    let mut support = vec![0_u32; atom_count];
                    for word in chunk {
                        for atom in encode_wave_surface(corpus.clean_surface(word)) {
                            let Some(atom_id) = graph.atom_id(atom.key) else {
                                continue;
                            };
                            let index = atom_id as usize;
                            flags[index] = coupling_flag(atom.key.channel);
                            support[index] = support[index].saturating_add(1);
                        }
                    }
                    (flags, support)
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| "L1 atom-support worker panicked".to_string())
            })
            .collect::<Result<Vec<_>, String>>()
    })?;
    let mut flags = vec![0_u8; atom_count];
    let mut support = vec![0_u32; atom_count];
    for (shard_flags, shard_support) in partial {
        for (target, value) in flags.iter_mut().zip(shard_flags) {
            *target |= value;
        }
        for (target, value) in support.iter_mut().zip(shard_support) {
            *target = target.saturating_add(value);
        }
    }
    Ok((flags, support))
}

fn worker_count(item_count: usize) -> usize {
    let workers = thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(item_count.max(1));
    #[cfg(test)]
    {
        return workers.min(2);
    }
    #[cfg(not(test))]
    workers
}

fn shadow_subcenter_target_limit(word_count: usize) -> Option<usize> {
    std::env::var("LAY_L11_SHADOW_SUBCENTER_TARGET_LIMIT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|limit| *limit > 0 && *limit < word_count)
}

fn resolve_word_surfaces(
    graph: &NGramGraph,
    corpus: &TrainingCorpus,
    word: &TrainingCorpusWord,
) -> Vec<ResolvedSurface> {
    corpus
        .word_surfaces(word)
        .map(|surface| resolve_surface(graph, surface))
        .collect()
}

fn resolve_surface(graph: &NGramGraph, surface: &str) -> ResolvedSurface {
    ResolvedSurface {
        atoms: encode_wave_surface(surface)
            .into_iter()
            .filter_map(|atom| {
                graph
                    .atom_id(atom.key)
                    .map(|atom_id| (atom_id, atom.position, atom.weight, atom.key.channel))
            })
            .collect(),
        physical_keys: physical_key_sequence(surface),
    }
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

fn coupling_slice(couplings: &[WaveCoupling], range: CouplingRange) -> &[WaveCoupling] {
    let start = range.start as usize;
    let end = start.saturating_add(range.count as usize);
    couplings.get(start..end).unwrap_or_default()
}

fn atom_couplings<'a>(
    atoms: &[AtomRecord],
    couplings: &'a [WaveCoupling],
    atom_id: u32,
) -> &'a [WaveCoupling] {
    let Some(atom) = atoms.get(atom_id as usize) else {
        return &[];
    };
    let start = atom.coupling_start as usize;
    let end = start.saturating_add(atom.coupling_count as usize);
    couplings.get(start..end).unwrap_or_default()
}

fn discover_anti_centers(
    corpus: &TrainingCorpus,
    graph: &NGramGraph,
    atoms: &[AtomRecord],
    forward_couplings: &[WaveCoupling],
    maximum_forward_strengths: &[u8],
    anti_postings: &AntiPostingIndex,
) -> Vec<Vec<WordCenter64>> {
    let words = corpus.words();
    let target_count = std::env::var("LAY_L11_SHADOW_ANTI_TARGET_LIMIT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|limit| *limit > 0)
        .unwrap_or(words.len())
        .min(words.len());
    let maximum_surfaces_per_target = std::env::var("LAY_L11_SHADOW_ANTI_SURFACES_PER_TARGET")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|limit| *limit > 0)
        .unwrap_or(usize::MAX);
    let workers = thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(target_count.max(1));
    if target_count != words.len() || maximum_surfaces_per_target != usize::MAX {
        eprintln!(
            concat!(
                "l11_anti_discovery fixed_target_range=0..{} workers={} ",
                "surfaces_per_target={} shadow_only=true"
            ),
            target_count,
            workers,
            if maximum_surfaces_per_target == usize::MAX {
                "all".to_string()
            } else {
                maximum_surfaces_per_target.to_string()
            }
        );
    }
    let mut relations = vec![Vec::<(u32, AntiRelation)>::new(); words.len()];
    let next_target = AtomicUsize::new(0);
    let completed_targets = AtomicUsize::new(0);
    let completed_surfaces = AtomicU64::new(0);
    let posting_entries = AtomicU64::new(0);
    let posting_entries_skipped = AtomicU64::new(0);
    let exact_candidates = AtomicU64::new(0);
    let dense_fallbacks = AtomicU64::new(0);
    let block_checks = AtomicU64::new(0);
    let block_skips = AtomicU64::new(0);
    let anti_started = Instant::now();
    thread::scope(|scope| {
        let (sender, receiver) = std::sync::mpsc::sync_channel(workers.saturating_mul(2));
        let handles = (0..workers)
            .map(|_| {
                let sender = sender.clone();
                let next_target = &next_target;
                let completed_targets = &completed_targets;
                let completed_surfaces = &completed_surfaces;
                let posting_entries = &posting_entries;
                let posting_entries_skipped = &posting_entries_skipped;
                let exact_candidates = &exact_candidates;
                let dense_fallbacks = &dense_fallbacks;
                let block_checks = &block_checks;
                let block_skips = &block_skips;
                let anti_started = &anti_started;
                scope.spawn(move || {
                    let mut scores = DenseScoreAccumulator::new(words.len());
                    loop {
                        let target = next_target.fetch_add(1, Ordering::Relaxed);
                        if target >= target_count {
                            break;
                        }
                        let word = &words[target];
                        let surfaces = resolve_word_surfaces(graph, corpus, word);
                        let target_relations = discover_target_relations(
                            target as u32,
                            &surfaces,
                            atoms,
                            forward_couplings,
                            maximum_forward_strengths,
                            anti_postings,
                            &mut scores,
                            maximum_surfaces_per_target,
                        );
                        completed_surfaces
                            .fetch_add(target_relations.1.surfaces, Ordering::Relaxed);
                        posting_entries
                            .fetch_add(target_relations.1.posting_entries, Ordering::Relaxed);
                        posting_entries_skipped.fetch_add(
                            target_relations.1.posting_entries_skipped,
                            Ordering::Relaxed,
                        );
                        exact_candidates
                            .fetch_add(target_relations.1.exact_candidates, Ordering::Relaxed);
                        dense_fallbacks
                            .fetch_add(target_relations.1.dense_fallbacks, Ordering::Relaxed);
                        block_checks.fetch_add(target_relations.1.block_checks, Ordering::Relaxed);
                        block_skips.fetch_add(target_relations.1.block_skips, Ordering::Relaxed);
                        if sender.send((target as u32, target_relations.0)).is_err() {
                            return;
                        }
                        let completed = completed_targets.fetch_add(1, Ordering::Relaxed) + 1;
                        if completed % ANTI_PROGRESS_INTERVAL == 0 || completed == target_count {
                            eprintln!(
                                concat!(
                                    "l11_anti_discovery targets={} total={} ",
                                    "percent_milli={} surfaces={} posting_entries={} ",
                                    "posting_entries_skipped={} exact_candidates={} ",
                                    "dense_fallbacks={} block_checks={} block_skips={} ",
                                    "elapsed_ms={}"
                                ),
                                completed,
                                target_count,
                                completed.saturating_mul(100_000) / target_count.max(1),
                                completed_surfaces.load(Ordering::Relaxed),
                                posting_entries.load(Ordering::Relaxed),
                                posting_entries_skipped.load(Ordering::Relaxed),
                                exact_candidates.load(Ordering::Relaxed),
                                dense_fallbacks.load(Ordering::Relaxed),
                                block_checks.load(Ordering::Relaxed),
                                block_skips.load(Ordering::Relaxed),
                                anti_started.elapsed().as_millis(),
                            );
                        }
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
                bank.truncate(MAX_DIRECTIONAL_ANTI_RELATIONS);
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
    corpus: &TrainingCorpus,
    graph: &NGramGraph,
    atoms: &[AtomRecord],
    reverse_couplings: &[WaveCoupling],
    reverse_ranges: &[CouplingRange],
    directional_anti: &[Vec<WordCenter64>],
    surface_owners: &HashMap<&str, Vec<u32>>,
) -> L11Banks {
    let words = corpus.words();
    let target_count = shadow_subcenter_target_limit(words.len()).unwrap_or(words.len());
    let workers = worker_count(target_count);
    let chunk_size = SUBCENTER_WORK_CHUNK.min(target_count.max(1));
    let work_chunks = target_count.div_ceil(chunk_size);
    eprintln!(
        concat!(
            "l11_subcenters words={} targets={} workers={} chunk_size={} ",
            "work_chunks={} evidence_only_ambiguity=true shadow_only={}"
        ),
        words.len(),
        target_count,
        workers,
        chunk_size,
        work_chunks,
        target_count != words.len(),
    );
    let next_start = AtomicUsize::new(0);
    let completed_words = AtomicUsize::new(0);
    let search_stats = SubcenterSearchStats::default();
    let subcenter_started = Instant::now();
    let mut shards = thread::scope(|scope| {
        let (sender, receiver) = std::sync::mpsc::sync_channel(workers.saturating_mul(2));
        let handles = (0..workers)
            .map(|_| {
                let sender = sender.clone();
                let next_start = &next_start;
                let completed_words = &completed_words;
                let search_stats = &search_stats;
                let subcenter_started = &subcenter_started;
                scope.spawn(move || loop {
                    let start = next_start.fetch_add(chunk_size, Ordering::Relaxed);
                    if start >= target_count {
                        break;
                    }
                    let end = start.saturating_add(chunk_size).min(target_count);
                    let shard = compile_l11_subcenter_range(
                        corpus,
                        graph,
                        atoms,
                        reverse_couplings,
                        reverse_ranges,
                        directional_anti,
                        surface_owners,
                        start,
                        end,
                        target_count,
                        completed_words,
                        search_stats,
                        subcenter_started,
                    );
                    if sender.send((start, shard)).is_err() {
                        break;
                    }
                })
            })
            .collect::<Vec<_>>();
        drop(sender);
        let shards = receiver.into_iter().collect::<Vec<_>>();
        for handle in handles {
            handle.join().expect("L1.1 subcenter worker panicked");
        }
        shards
    });
    shards.sort_unstable_by_key(|(start, _)| *start);
    eprintln!(
        "l11_subcenters_search evidence_only_ambiguity=true edit_distance_calls={} elapsed_ms={}",
        search_stats.edit_distance_calls.load(Ordering::Relaxed),
        subcenter_started.elapsed().as_millis(),
    );

    let mut merged = L11Banks {
        profiles: Vec::with_capacity(words.len()),
        positive: Vec::new(),
        anti: Vec::new(),
        hard_negative: Vec::new(),
        ambiguity: Vec::new(),
        keyboard_geometry: Vec::new(),
    };
    for (_, mut shard) in shards {
        let positive_offset = merged.positive.len() as u32;
        let anti_offset = merged.anti.len() as u32;
        let hard_negative_offset = merged.hard_negative.len() as u32;
        let ambiguity_offset = merged.ambiguity.len() as u32;
        let keyboard_geometry_offset = merged.keyboard_geometry.len() as u32;
        for profile in &mut shard.profiles {
            profile.positive_start = profile.positive_start.saturating_add(positive_offset);
            profile.anti_start = profile.anti_start.saturating_add(anti_offset);
            profile.hard_negative_start = profile
                .hard_negative_start
                .saturating_add(hard_negative_offset);
            profile.ambiguity_start = profile.ambiguity_start.saturating_add(ambiguity_offset);
            profile.keyboard_geometry_start = profile
                .keyboard_geometry_start
                .saturating_add(keyboard_geometry_offset);
        }
        merged.profiles.append(&mut shard.profiles);
        merged.positive.append(&mut shard.positive);
        merged.anti.append(&mut shard.anti);
        merged.hard_negative.append(&mut shard.hard_negative);
        merged.ambiguity.append(&mut shard.ambiguity);
        merged
            .keyboard_geometry
            .append(&mut shard.keyboard_geometry);
    }
    merged
}

#[allow(clippy::too_many_arguments)]
fn compile_l11_subcenter_range(
    corpus: &TrainingCorpus,
    graph: &NGramGraph,
    atoms: &[AtomRecord],
    reverse_couplings: &[WaveCoupling],
    reverse_ranges: &[CouplingRange],
    directional_anti: &[Vec<WordCenter64>],
    surface_owners: &HashMap<&str, Vec<u32>>,
    start: usize,
    end: usize,
    total_words: usize,
    completed_words: &AtomicUsize,
    search_stats: &SubcenterSearchStats,
    subcenter_started: &Instant,
) -> L11Banks {
    const MAX_POSITIVE_SUBCENTERS: usize = 4;
    const MAX_ANTI_SUBCENTERS: usize = 4;
    const MAX_HARD_NEGATIVE_SUBCENTERS: usize = 2;
    const MAX_AMBIGUITY_SUBCENTERS: usize = 8;
    const MAX_AMBIGUITY_SUBCENTERS_PER_RELATION: usize = 2;

    let words = corpus.words();
    let range_len = end.saturating_sub(start);
    let mut profiles = Vec::with_capacity(range_len);
    let mut positive = Vec::with_capacity(range_len);
    let mut anti = Vec::with_capacity(range_len);
    let mut hard_negative = Vec::with_capacity(range_len / 4);
    let mut ambiguity = Vec::with_capacity(range_len);
    let mut keyboard_geometry = Vec::with_capacity(range_len.saturating_mul(12));
    for terminal_id in start..end {
        let word = &words[terminal_id];
        let surfaces = resolve_word_surfaces(graph, corpus, word);
        let owner_reverse = coupling_slice(reverse_couplings, reverse_ranges[terminal_id]);
        let positive_start = positive.len() as u32;
        let fit_masses = surfaces
            .iter()
            .enumerate()
            .filter(|(index, _)| surfaces.len() < 5 || index % 5 != 0)
            .map(|(_, surface)| surface_phase_mass(surface, atoms))
            .collect::<Vec<_>>();
        let learned_positive =
            cluster_phase_masses(&fit_masses, terminal_id as u32, MAX_POSITIVE_SUBCENTERS);
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
        let ambiguity_start = ambiguity.len() as u32;
        let mut ambiguity_fit = BTreeMap::<u32, Vec<usize>>::new();
        for (surface_index, surface_name) in corpus.word_surfaces(word).enumerate() {
            let Some(owners) = surface_owners.get(surface_name) else {
                continue;
            };
            if owners.len() < 2 || calibration_surface_hash(surface_name) % 5 == 0 {
                continue;
            }
            for competitor in owners {
                if *competitor != terminal_id as u32 {
                    ambiguity_fit
                        .entry(*competitor)
                        .or_default()
                        .push(surface_index);
                }
            }
        }
        let mut learned_ambiguity = Vec::new();
        for (competitor, relation_surface_indices) in &ambiguity_fit {
            let residuals = relation_surface_indices
                .iter()
                .filter_map(|surface_index| surfaces.get(*surface_index))
                .map(|surface| {
                    pair_residual_phase_mass(
                        surface,
                        atoms,
                        owner_reverse,
                        coupling_slice(reverse_couplings, reverse_ranges[*competitor as usize]),
                    )
                })
                .collect::<Vec<_>>();
            let relation_centers = cluster_phase_masses(
                &residuals,
                *competitor,
                MAX_AMBIGUITY_SUBCENTERS_PER_RELATION,
            );
            if relation_centers.is_empty() {
                continue;
            }
            learned_ambiguity.extend(relation_centers.into_iter().map(|center| {
                AmbiguityPhaseCenter64::from_record(center)
                    .with_threshold_milli(1)
                    .record()
            }));
        }
        if let Some(clean) = surfaces.first() {
            let mut represented = learned_ambiguity
                .iter()
                .map(|center| center.decoder_terminal)
                .collect::<BTreeSet<_>>();
            for competitor in nearby_directional_competitors(
                terminal_id as u32,
                words,
                &directional_anti[terminal_id],
                MAX_AMBIGUITY_SUBCENTERS,
                search_stats,
            ) {
                if !represented.insert(competitor) {
                    continue;
                }
                let residual = pair_residual_phase_mass(
                    clean,
                    atoms,
                    owner_reverse,
                    coupling_slice(reverse_couplings, reverse_ranges[competitor as usize]),
                );
                learned_ambiguity.push(
                    AmbiguityPhaseCenter64::from_record(phase_center(residual, 1, competitor))
                        .with_threshold_milli(1)
                        .record(),
                );
            }
        }
        search_stats
            .edit_distance_calls
            .fetch_add(learned_ambiguity.len() as u64, Ordering::Relaxed);
        let mut ranked_ambiguity = learned_ambiguity
            .into_iter()
            .map(|center| {
                (
                    damerau_levenshtein(
                        &word.surface,
                        &words[center.decoder_terminal as usize].surface,
                    ),
                    center,
                )
            })
            .collect::<Vec<_>>();
        ranked_ambiguity.sort_unstable_by(|(left_distance, left), (right_distance, right)| {
            left_distance
                .cmp(right_distance)
                .then_with(|| right.crystal_support.cmp(&left.crystal_support))
                .then_with(|| left.decoder_terminal.cmp(&right.decoder_terminal))
                .then_with(|| left.encode().cmp(&right.encode()))
        });
        ranked_ambiguity.truncate(MAX_AMBIGUITY_SUBCENTERS);
        let learned_ambiguity = ranked_ambiguity
            .into_iter()
            .map(|(_, center)| center)
            .collect::<Vec<_>>();
        ambiguity.extend_from_slice(&learned_ambiguity);
        let keyboard_geometry_start = keyboard_geometry.len() as u32;
        if let Some(clean) = surfaces.first() {
            keyboard_geometry.extend_from_slice(&clean.physical_keys);
        }
        profiles.push(CenterPhaseProfile {
            positive_start,
            anti_start,
            hard_negative_start,
            keyboard_geometry_start,
            ambiguity_start,
            positive_count: learned_positive.len() as u8,
            anti_count: (anti.len() as u32 - anti_start) as u8,
            hard_negative_count: (hard_negative.len() as u32 - hard_negative_start) as u8,
            keyboard_geometry_count: (keyboard_geometry.len() as u32 - keyboard_geometry_start)
                as u8,
            flags: super::model::CENTER_PHASE_FLAG_PHYSICAL_KEY_GEOMETRY,
            ambiguity_count: learned_ambiguity.len() as u8,
            min_ambiguity_milli: 0,
        });
        let completed = completed_words.fetch_add(1, Ordering::Relaxed) + 1;
        if completed % SUBCENTER_PROGRESS_INTERVAL == 0 || completed == total_words {
            eprintln!(
                "l11_subcenters_progress words={completed} total={total_words} \
                 percent_milli={} elapsed_ms={}",
                completed.saturating_mul(100_000) / total_words.max(1),
                subcenter_started.elapsed().as_millis()
            );
        }
    }
    L11Banks {
        profiles,
        positive,
        anti,
        hard_negative,
        ambiguity,
        keyboard_geometry,
    }
}

fn nearby_directional_competitors(
    owner: u32,
    words: &[TrainingCorpusWord],
    directional: &[WordCenter64],
    limit: usize,
    search_stats: &SubcenterSearchStats,
) -> Vec<u32> {
    const MAX_PAIR_DISTANCE: usize = 4;

    let mut candidates = directional
        .iter()
        .map(|center| center.decoder_terminal)
        .filter(|candidate| *candidate != owner)
        .filter_map(|candidate| {
            search_stats
                .edit_distance_calls
                .fetch_add(1, Ordering::Relaxed);
            let distance = damerau_levenshtein_bounded(
                &words[owner as usize].surface,
                &words[candidate as usize].surface,
                MAX_PAIR_DISTANCE,
            )?;
            Some((distance, candidate))
        })
        .collect::<Vec<_>>();
    candidates.sort_unstable();
    candidates.truncate(limit);
    candidates
        .into_iter()
        .map(|(_, candidate)| candidate)
        .collect()
}

fn cluster_phase_masses(masses: &[PhaseMass], terminal_id: u32, limit: usize) -> Vec<WordCenter64> {
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
        for mass in masses {
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

fn pair_residual_phase_mass(
    surface: &ResolvedSurface,
    atoms: &[AtomRecord],
    owner_reverse: &[WaveCoupling],
    competitor_reverse: &[WaveCoupling],
) -> PhaseMass {
    let residual = pair_residual_atoms(
        surface
            .atoms
            .iter()
            .filter(|(_, _, _, channel)| coupling_flag(*channel) == 0)
            .map(|(atom_id, position, _, _)| (*atom_id, (position / 257).min(255) as u8)),
        owner_reverse
            .iter()
            .filter(|coupling| coupling.flags == 0)
            .map(|coupling| (coupling.peer_id, coupling.position_mode)),
        competitor_reverse
            .iter()
            .filter(|coupling| coupling.flags == 0)
            .map(|coupling| (coupling.peer_id, coupling.position_mode)),
    );
    let mut mass = [0_i64; WAVE_DIMENSION];
    for relation in residual {
        let Some(atom) = atoms.get(relation.atom_id as usize) else {
            continue;
        };
        for component in positioned_atom_code(atom.wave_code, relation.position_mode).components {
            mass[component.basis as usize] = mass[component.basis as usize].saturating_add(
                i64::from(component.coefficient).saturating_mul(i64::from(relation.coefficient)),
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
    corpus: &TrainingCorpus,
    graph: &NGramGraph,
    atoms: &[AtomRecord],
    reverse_couplings: &[WaveCoupling],
    reverse_ranges: &[CouplingRange],
    basis: &[super::crystal::ComplexBasisWave],
    profiles: &[CenterPhaseProfile],
    positive_subcenters: &[WordCenter64],
) -> RestorationCalibration {
    let words = corpus.words();
    let mut distances = Vec::new();
    let mut positive = Vec::new();
    let mut backward = Vec::new();
    for (terminal_id, word) in words.iter().enumerate() {
        let surfaces = resolve_word_surfaces(graph, corpus, word);
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
            backward.push(backward_coherence_milli(
                surface,
                coupling_slice(reverse_couplings, reverse_ranges[terminal_id]),
            ));
        }
    }
    RestorationCalibration {
        max_geometry_distance: high_quantile(&mut distances, 999),
        min_positive_milli: low_quantile(&mut positive, 0),
        min_backward_milli: low_quantile(&mut backward, 0),
        min_tied_energy_margin: 0,
    }
}

fn calibrate_l11_tied_energy_margin(
    corpus: &TrainingCorpus,
    ambiguous_owners: &HashMap<&str, Vec<u32>>,
    memory: &LexicalGrokkingMemory,
) -> u16 {
    const CALIBRATION_BUCKETS: u64 = 5;
    const CANDIDATE_LIMIT: usize = 64;

    let words = corpus.words();
    let workers = thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(words.len().max(1));
    let partial = thread::scope(|scope| {
        (0..workers)
            .map(|worker| {
                scope.spawn(move || {
                    let mut maximum_unsafe_margin = None::<u16>;
                    let mut maximum_safe_margin = None::<u16>;
                    for word in words.iter().skip(worker).step_by(workers) {
                        for surface in corpus.training_surfaces(word) {
                            if calibration_surface_hash(surface) % CALIBRATION_BUCKETS != 0 {
                                continue;
                            }
                            let mut candidates =
                                memory.readout(surface, CANDIDATE_LIMIT, ReadoutMode::Full);
                            memory.apply_l11_phase_evidence(surface, &mut candidates);
                            let Some((winner, margin)) = tied_energy_winner(&candidates) else {
                                continue;
                            };
                            let mutates_surface = memory
                                .decode_terminal(winner)
                                .is_some_and(|decoded| decoded != surface);
                            let unsafe_authority =
                                if let Some(objective_targets) = ambiguous_owners.get(surface) {
                                    if objective_targets.len() == 1 {
                                        !objective_targets.contains(&winner)
                                    } else {
                                        mutates_surface
                                    }
                                } else {
                                    winner != word.terminal_id
                                };
                            if unsafe_authority {
                                maximum_unsafe_margin =
                                    Some(maximum_unsafe_margin.unwrap_or_default().max(margin));
                            } else {
                                maximum_safe_margin =
                                    Some(maximum_safe_margin.unwrap_or_default().max(margin));
                            }
                        }
                    }
                    (maximum_unsafe_margin, maximum_safe_margin)
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| handle.join().expect("L1.1 calibration worker panicked"))
            .collect::<Vec<_>>()
    });
    let maximum_unsafe_margin = partial.iter().filter_map(|item| item.0).max();
    let maximum_safe_margin = partial.iter().filter_map(|item| item.1).max();

    let Some(maximum_safe_margin) = maximum_safe_margin else {
        return 0;
    };
    let threshold = match maximum_unsafe_margin {
        Some(u16::MAX) => return 0,
        Some(margin) => margin + 1,
        None => 1,
    };
    if maximum_safe_margin < threshold {
        0
    } else {
        threshold
    }
}

fn calibration_surface_hash(surface: &str) -> u64 {
    let mut state = 0x4c31_315f_4341_4c31_u64;
    for byte in surface.bytes() {
        state = mix64_golden(state ^ u64::from(byte));
    }
    state
}

fn objective_contains(
    ambiguous_owners: &HashMap<&str, Vec<u32>>,
    surface: &str,
    default_owner: u32,
    candidate: u32,
) -> bool {
    ambiguous_owners
        .get(surface)
        .map_or(candidate == default_owner, |owners| {
            owners.contains(&candidate)
        })
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
    atoms: &[AtomRecord],
    forward_couplings: &[WaveCoupling],
    maximum_forward_strengths: &[u8],
    anti_postings: &AntiPostingIndex,
    scores: &mut DenseScoreAccumulator,
    maximum_surfaces: usize,
) -> (Vec<(u32, AntiRelation)>, AntiSearchStats) {
    let mut relations = BTreeMap::new();
    let mut search_stats = AntiSearchStats::default();
    let damaged_surfaces = surfaces.get(1..).unwrap_or_default();
    for surface_index in
        bounded_anti_surface_indices(target, damaged_surfaces.len(), maximum_surfaces)
    {
        let surface = &damaged_surfaces[surface_index];
        search_stats.surfaces = search_stats.surfaces.saturating_add(1);
        let competitors = match wand_top_competitors(
            surface,
            target,
            maximum_forward_strengths,
            anti_postings,
            &mut search_stats,
        ) {
            Ok(competitors) => competitors,
            Err(()) => {
                search_stats.dense_fallbacks = search_stats.dense_fallbacks.saturating_add(1);
                accumulate_forward_scores(surface, atoms, forward_couplings, scores);
                let Some(target_score) = scores.get(target) else {
                    continue;
                };
                scores.top_competitors(target, target_score, 4)
            }
        };
        for (competitor, score) in competitors {
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
    (relations.into_iter().collect(), search_stats)
}

fn bounded_anti_surface_indices(
    target: u32,
    surface_count: usize,
    maximum_surfaces: usize,
) -> Vec<usize> {
    if surface_count == 0 {
        return Vec::new();
    }
    let limit = maximum_surfaces.min(surface_count);
    if limit == surface_count {
        return (0..surface_count).collect();
    }
    let mut state = mix64_golden(u64::from(target) ^ 0x4c31_315f_414e_5449);
    if limit == 1 {
        return vec![state as usize % surface_count];
    }
    let mut selected = BTreeSet::new();
    while selected.len() < limit {
        selected.insert(state as usize % surface_count);
        state = mix64_golden(state);
    }
    selected.into_iter().collect()
}

fn wand_top_competitors(
    surface: &ResolvedSurface,
    target: u32,
    maximum_forward_strengths: &[u8],
    anti_postings: &AntiPostingIndex,
    stats: &mut AntiSearchStats,
) -> Result<Vec<(u32, u32)>, ()> {
    let cursors = build_wand_cursors(surface, maximum_forward_strengths, anti_postings, stats)?;
    let Some(target_score) = exact_terminal_score(&cursors, target) else {
        return Ok(Vec::new());
    };
    Ok(wand_collect_top(cursors, target, target_score, 4, stats))
}

#[cfg(test)]
fn wand_top_lattice_competitors(
    surface: &ResolvedSurface,
    target: u32,
    maximum_forward_strengths: &[u8],
    anti_postings: &AntiPostingIndex,
    limit: usize,
    stats: &mut AntiSearchStats,
) -> Result<Vec<(u32, u32)>, ()> {
    let cursors = build_wand_cursors(surface, maximum_forward_strengths, anti_postings, stats)?;
    Ok(wand_collect_top(cursors, target, 0, limit, stats))
}

fn build_wand_cursors<'a>(
    surface: &ResolvedSurface,
    maximum_forward_strengths: &[u8],
    anti_postings: &'a AntiPostingIndex,
    stats: &mut AntiSearchStats,
) -> Result<Vec<WandCursor<'a>>, ()> {
    let mut cursors = Vec::with_capacity(surface.atoms.len());
    let mut total_upper_bound = 0_u64;
    for (atom_id, position, weight, _) in surface.atoms.iter().copied() {
        let observed_position = (position / 257).min(255) as u8;
        let postings = anti_postings.cursor(atom_id, observed_position, weight);
        if postings.current_terminal().is_none() {
            continue;
        }
        let upper_bound = u64::from(
            maximum_forward_strengths
                .get(atom_id as usize)
                .copied()
                .unwrap_or_default(),
        )
        .saturating_mul(u64::from(weight))
        .saturating_mul(256);
        total_upper_bound = total_upper_bound.saturating_add(upper_bound);
        stats.posting_entries = stats
            .posting_entries
            .saturating_add(u64::from(postings.posting_count()));
        cursors.push(WandCursor {
            postings,
            upper_bound,
            cached_block: u32::MAX,
            cached_block_upper_bound: 0,
        });
    }
    // Dense scoring intentionally preserves legacy wrapping addition. WAND
    // requires monotonic non-negative sums, so an overflowing query falls
    // back to the exact dense oracle.
    if total_upper_bound > u64::from(u32::MAX) {
        return Err(());
    }
    Ok(cursors)
}

fn wand_collect_top(
    mut cursors: Vec<WandCursor<'_>>,
    target: u32,
    minimum_score: u32,
    limit: usize,
    stats: &mut AntiSearchStats,
) -> Vec<(u32, u32)> {
    if limit == 0 {
        return Vec::new();
    }
    let mut threshold = minimum_score;
    let mut top = Vec::with_capacity(limit);
    loop {
        cursors.retain(|cursor| cursor.current_terminal().is_some());
        if cursors.is_empty() {
            break;
        }
        cursors.sort_unstable_by_key(|cursor| cursor.current_terminal().unwrap_or(u32::MAX));
        let mut cumulative_bound = 0_u64;
        let pivot = cursors.iter().position(|cursor| {
            cumulative_bound = cumulative_bound.saturating_add(cursor.upper_bound);
            cumulative_bound >= u64::from(threshold)
        });
        let Some(pivot) = pivot else {
            break;
        };
        let pivot_terminal = cursors[pivot]
            .current_terminal()
            .expect("active WAND cursor has a terminal");
        let first_terminal = cursors[0]
            .current_terminal()
            .expect("active WAND cursor has a terminal");
        if first_terminal != pivot_terminal {
            for cursor in cursors.iter_mut().take(pivot) {
                let skipped = cursor.advance_to(pivot_terminal);
                stats.posting_entries_skipped =
                    stats.posting_entries_skipped.saturating_add(skipped as u64);
            }
            continue;
        }

        stats.block_checks = stats.block_checks.saturating_add(1);
        let terminal_block = pivot_terminal / TERMINAL_BLOCK_SIZE;
        let block_upper_bound = cursors
            .iter_mut()
            .map(|cursor| cursor.block_upper_bound(terminal_block))
            .fold(0_u64, u64::saturating_add);
        if block_upper_bound < u64::from(threshold) {
            stats.block_skips = stats.block_skips.saturating_add(1);
            skip_terminal_block(&mut cursors, terminal_block, stats);
            continue;
        }

        let mut score = 0_u32;
        for cursor in &mut cursors {
            if cursor.current_terminal() == Some(pivot_terminal) {
                score = score.wrapping_add(cursor.consume_current());
            }
        }
        stats.exact_candidates = stats.exact_candidates.saturating_add(1);
        if pivot_terminal == target || score < minimum_score {
            continue;
        }
        insert_top_competitor(&mut top, pivot_terminal, score, limit);
        if top.len() == limit {
            threshold = minimum_score.max(top[limit - 1].1);
        }
    }
    top
}

fn skip_terminal_block(
    cursors: &mut [WandCursor<'_>],
    terminal_block: u32,
    stats: &mut AntiSearchStats,
) {
    let next_block = block_end_terminal(terminal_block).saturating_add(1);
    for cursor in cursors {
        if cursor
            .current_terminal()
            .is_some_and(|terminal| terminal / TERMINAL_BLOCK_SIZE == terminal_block)
        {
            let skipped = cursor.advance_to(next_block);
            stats.posting_entries_skipped =
                stats.posting_entries_skipped.saturating_add(skipped as u64);
        }
    }
}

fn block_end_terminal(block: u32) -> u32 {
    block
        .saturating_add(1)
        .saturating_mul(TERMINAL_BLOCK_SIZE)
        .saturating_sub(1)
}

fn exact_terminal_score(cursors: &[WandCursor<'_>], terminal_id: u32) -> Option<u32> {
    let mut found = false;
    let mut score = 0_u32;
    for cursor in cursors {
        let Some(contribution) = cursor.score_terminal(terminal_id) else {
            continue;
        };
        found = true;
        score = score.wrapping_add(contribution);
    }
    found.then_some(score)
}

fn insert_top_competitor(top: &mut Vec<(u32, u32)>, terminal_id: u32, score: u32, limit: usize) {
    let position = top
        .iter()
        .position(|(other_terminal, other_score)| {
            score > *other_score || (score == *other_score && terminal_id < *other_terminal)
        })
        .unwrap_or(top.len());
    if position < limit {
        top.insert(position, (terminal_id, score));
        top.truncate(limit);
    }
}

fn score_contribution(observed_position: u8, weight: u8, coupling: WaveCoupling) -> u32 {
    score_strength_position(
        observed_position,
        weight,
        coupling.strength,
        coupling.position_mode,
    )
}

fn accumulate_forward_scores(
    surface: &ResolvedSurface,
    atoms: &[AtomRecord],
    forward_couplings: &[WaveCoupling],
    scores: &mut DenseScoreAccumulator,
) {
    scores.begin_surface();
    for (atom_id, position, weight, _) in surface.atoms.iter().copied() {
        let observed = (position / 257).min(255) as u8;
        for coupling in atom_couplings(atoms, forward_couplings, atom_id) {
            let contribution = score_contribution(observed, weight, *coupling);
            scores.add(coupling.peer_id, contribution);
        }
    }
}

fn coupling_flag(channel: AtomChannel) -> u8 {
    match channel {
        AtomChannel::CharacterAnchor => COUPLING_FLAG_CHARACTER_ANCHOR,
        _ => 0,
    }
}

fn compile_decoder(words: &[TrainingCorpusWord]) -> Result<(Vec<DecoderNode>, Vec<u32>), String> {
    let mut nodes = vec![DecoderNode {
        parent: u32::MAX,
        symbol: 0,
    }];
    let mut transitions = HashMap::<(u32, u32), u32>::new();
    let mut terminals = Vec::with_capacity(words.len());
    for word in words {
        let mut node = 0_u32;
        for symbol in word.surface.chars().map(|ch| ch as u32) {
            let next = if let Some(next) = transitions.get(&(node, symbol)) {
                *next
            } else {
                let next = u32::try_from(nodes.len())
                    .map_err(|_| "decoder node count exceeds u32".to_string())?;
                nodes.push(DecoderNode {
                    parent: node,
                    symbol,
                });
                transitions.insert((node, symbol), next);
                next
            };
            node = next;
        }
        terminals.push(node);
    }
    Ok((nodes, terminals))
}

fn corpus_hash(words: &[TrainingCorpusWord]) -> u64 {
    let mut state = 0x4c31_4352_5953_3032_u64;
    for word in words {
        for byte in word.surface.as_bytes() {
            state = mix64_golden(state ^ u64::from(*byte));
        }
        state = mix64_golden(state ^ u64::from(word.terminal_id));
    }
    state
}
