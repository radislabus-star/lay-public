use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::io;
use std::thread;

use serde::{Deserialize, Serialize};

use super::super::phase_field::{
    add_cluster, add_hashed_atom, empty_vector, hash_text, margin, phase_center_from_sum,
    phase_micro, PhaseCell, PhaseCenter,
};
use super::{
    ContextCandidateProfile, ContextPhaseMode, ContextPhasePackage, TokenSemanticState, CELLS,
};

const MAX_POSITIVE_CENTERS: usize = 4;
const MAX_ANTI_CENTERS: usize = 12;
const CENTER_SPLIT_COHERENCE: f32 = 0.76;
const MAX_COMPETITORS: usize = 4;
const MAX_FRAGMENT_TOKENS: usize = 64;
const MAX_L2_TRAINING_SURFACES: usize = 4;

#[derive(Clone, Copy)]
pub(crate) struct ContextPhaseCompileInput<'a> {
    pub(crate) corpus_text: &'a str,
    pub(crate) max_fragments: usize,
    pub(crate) min_profile_support: u32,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ContextPhaseCompileReport {
    pub(crate) kind: &'static str,
    pub(crate) raw_words_stored: bool,
    pub(crate) corpus_fragments: usize,
    pub(crate) transitions: u64,
    pub(crate) semantic_states: usize,
    pub(crate) candidate_profiles: usize,
    pub(crate) positive_centers: usize,
    pub(crate) anti_centers: usize,
    pub(crate) positive_examples: u64,
    pub(crate) negative_examples: u64,
    pub(crate) l2_lattice_negative_examples: u64,
    pub(crate) global_threshold_micro: i32,
    pub(crate) competition_threshold_micro: i32,
    pub(crate) min_profile_support: u32,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ContextPhaseProofReport {
    pub(crate) kind: &'static str,
    pub(crate) train_fragments: usize,
    pub(crate) heldout_fragments: usize,
    pub(crate) evaluated_transitions: usize,
    pub(crate) full_supports: usize,
    pub(crate) full_top1: usize,
    pub(crate) full_false_supports: usize,
    pub(crate) full_false_top1: usize,
    pub(crate) no_phase_supports: usize,
    pub(crate) no_anti_top1: usize,
    pub(crate) no_anti_false_supports: usize,
    pub(crate) no_anti_false_top1: usize,
    pub(crate) no_semantic_top1: usize,
    pub(crate) phase_ablation_drop: usize,
    pub(crate) anti_ablation_drop: usize,
    pub(crate) anti_false_support_reduction: usize,
    pub(crate) anti_false_top1_reduction: usize,
    pub(crate) semantic_ablation_drop: usize,
    pub(crate) support_precision_ppm: u32,
    pub(crate) raw_words_stored: bool,
    pub(crate) min_profile_support: u32,
    pub(crate) verdict: &'static str,
}

/// Associative heldout counters. Cold proof can shard independent sentences
/// without changing the package, candidate lattice, or acceptance semantics.
#[derive(Default)]
struct ContextPhaseProofTotals {
    evaluated: usize,
    full_supports: usize,
    full_top1: usize,
    full_false_supports: usize,
    full_false_top1: usize,
    no_phase_supports: usize,
    no_anti_top1: usize,
    no_anti_false_supports: usize,
    no_anti_false_top1: usize,
    no_semantic_top1: usize,
}

impl ContextPhaseProofTotals {
    fn merge(&mut self, other: Self) {
        self.evaluated += other.evaluated;
        self.full_supports += other.full_supports;
        self.full_top1 += other.full_top1;
        self.full_false_supports += other.full_false_supports;
        self.full_false_top1 += other.full_false_top1;
        self.no_phase_supports += other.no_phase_supports;
        self.no_anti_top1 += other.no_anti_top1;
        self.no_anti_false_supports += other.no_anti_false_supports;
        self.no_anti_false_top1 += other.no_anti_false_top1;
        self.no_semantic_top1 += other.no_semantic_top1;
    }
}

/// Result of rebuilding a personal overlay from explicit IME feedback.
///
/// The resulting packet contains only the existing hashed profiles and their
/// quantized phase centers. The JSONL event source is never copied into it.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ContextPhaseFeedbackOverlayReport {
    pub(crate) kind: &'static str,
    pub(crate) raw_words_stored: bool,
    pub(crate) source_events: usize,
    pub(crate) positive_source_events: usize,
    pub(crate) negative_source_events: usize,
    pub(crate) positive_admitted: usize,
    pub(crate) negative_admitted: usize,
    pub(crate) skipped_unattested_context: usize,
    pub(crate) skipped_unattested_positive: usize,
    pub(crate) skipped_missing_profile: usize,
    pub(crate) candidate_profiles: usize,
    pub(crate) positive_centers: usize,
    pub(crate) anti_centers: usize,
}

#[derive(Debug, Deserialize)]
struct FeedbackEvent {
    kind: String,
    word: String,
    #[serde(default)]
    context: Vec<String>,
}

#[derive(Default)]
struct SemanticBuilder {
    sum: Vec<PhaseCell>,
    support: u32,
}

#[derive(Default)]
struct ProfileBuilder {
    positive: Vec<PhaseCenter>,
    negative: Vec<PhaseCenter>,
    positive_examples: u32,
    negative_examples: u32,
    positive_vectors: Vec<Vec<PhaseCell>>,
    negative_vectors: Vec<Vec<PhaseCell>>,
}

pub(crate) fn compile_context_phase(
    input: ContextPhaseCompileInput<'_>,
) -> (ContextPhasePackage, ContextPhaseCompileReport) {
    let min_profile_support = input.min_profile_support.max(2);
    let sequences = corpus_sequences(input.corpus_text, input.max_fragments);
    let semantic_states = compile_semantic_states(&sequences);
    let semantic_package = ContextPhasePackage {
        semantic_states,
        ..ContextPhasePackage::default()
    };
    let mut builders = BTreeMap::<u64, ProfileBuilder>::new();
    let competitor_cache = compile_l2_competitor_cache(&sequences);
    let mut transitions = 0_u64;

    for tokens in &sequences {
        for index in 1..tokens.len() {
            let context = &tokens[..index];
            let target = &tokens[index];
            let vector =
                semantic_package.candidate_relation_vector(context, target, ContextPhaseMode::Full);
            let target_hash = hash_text(target);
            let profile = builders.entry(target_hash).or_default();
            add_cluster(
                &mut profile.positive,
                &vector,
                MAX_POSITIVE_CENTERS,
                CENTER_SPLIT_COHERENCE,
            );
            profile.positive_examples = profile.positive_examples.saturating_add(1);
            if profile.positive_vectors.len() < 64 {
                profile.positive_vectors.push(vector.clone());
            }
            transitions = transitions.saturating_add(1);

            // Train destructive interference against the same candidate topology
            // that the live L2 field produces from damaged input. A lexicon-neighbour
            // list is not enough: it misses short, high-energy false attractors.
            let competitors = competitor_cache
                .get(target)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            for competitor in competitors.iter() {
                let profile = builders.entry(hash_text(competitor)).or_default();
                let competitor_vector = semantic_package.candidate_relation_vector(
                    context,
                    competitor,
                    ContextPhaseMode::Full,
                );
                add_cluster(
                    &mut profile.negative,
                    &competitor_vector,
                    MAX_ANTI_CENTERS,
                    CENTER_SPLIT_COHERENCE,
                );
                profile.negative_examples = profile.negative_examples.saturating_add(1);
                if profile.negative_vectors.len() < 64 {
                    profile.negative_vectors.push(competitor_vector);
                }
            }
        }
    }

    let mut profiles = builders
        .into_iter()
        .filter_map(|(token_hash, builder)| {
            (builder.positive_examples >= min_profile_support).then(|| {
                let threshold_micro = learned_threshold(&builder);
                ContextCandidateProfile {
                    token_hash,
                    positive_examples: builder.positive_examples,
                    negative_examples: builder.negative_examples,
                    threshold_micro,
                    positive: builder.positive,
                    negative: builder.negative,
                }
            })
        })
        .collect::<Vec<_>>();
    profiles.sort_by_key(|profile| profile.token_hash);
    let global_threshold_micro = learned_global_threshold(&profiles);
    let mut package = ContextPhasePackage {
        semantic_states: semantic_package.semantic_states,
        profiles,
        transitions,
        corpus_fragments: sequences.len().min(u32::MAX as usize) as u32,
        global_threshold_micro,
        competition_threshold_micro: 1,
    };
    package.competition_threshold_micro =
        learned_competition_threshold(&package, &sequences, &competitor_cache);

    let report = ContextPhaseCompileReport {
        kind: "l3_context_phase_compile",
        raw_words_stored: false,
        corpus_fragments: sequences.len(),
        transitions,
        semantic_states: package.semantic_states.len(),
        candidate_profiles: package.profiles.len(),
        positive_centers: package
            .profiles
            .iter()
            .map(|profile| profile.positive.len())
            .sum(),
        anti_centers: package
            .profiles
            .iter()
            .map(|profile| profile.negative.len())
            .sum(),
        positive_examples: package
            .profiles
            .iter()
            .map(|profile| u64::from(profile.positive_examples))
            .sum(),
        negative_examples: package
            .profiles
            .iter()
            .map(|profile| u64::from(profile.negative_examples))
            .sum(),
        l2_lattice_negative_examples: competitor_cache
            .values()
            .map(|competitors| competitors.len() as u64)
            .sum(),
        global_threshold_micro: package.global_threshold_micro,
        competition_threshold_micro: package.competition_threshold_micro,
        min_profile_support,
    };
    (package, report)
}

/// Layers explicit user IME outcomes onto a canonical context package.
///
/// This is intentionally narrower than corpus compilation: feedback can only
/// reinforce or suppress a profile already grounded by the clean corpus. A
/// noisy live word can therefore never create a new L3 authority by itself.
pub(crate) fn apply_feedback_overlay(
    package: &mut ContextPhasePackage,
    events_text: &str,
) -> io::Result<ContextPhaseFeedbackOverlayReport> {
    let mut report = ContextPhaseFeedbackOverlayReport {
        kind: "l3_context_phase_feedback_overlay",
        raw_words_stored: false,
        source_events: 0,
        positive_source_events: 0,
        negative_source_events: 0,
        positive_admitted: 0,
        negative_admitted: 0,
        skipped_unattested_context: 0,
        skipped_unattested_positive: 0,
        skipped_missing_profile: 0,
        candidate_profiles: package.profiles.len(),
        positive_centers: 0,
        anti_centers: 0,
    };

    for (line_number, line) in events_text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let event: FeedbackEvent = serde_json::from_str(line).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "invalid typing feedback JSONL at line {}: {error}",
                    line_number + 1
                ),
            )
        })?;
        let polarity = match event.kind.as_str() {
            "accepted_ime" | "confirmed_ime_prediction" => {
                report.positive_source_events += 1;
                1_i8
            }
            "rejected_ime" | "rejected_candidate" => {
                report.negative_source_events += 1;
                -1_i8
            }
            _ => continue,
        };
        report.source_events += 1;

        let context = event
            .context
            .iter()
            .map(|word| crate::typing_memory::normalize_memory_word(word))
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>();
        if context.is_empty()
            || !crate::typing_memory::phrase_is_attested_for_learning(&context.join(" "))
        {
            report.skipped_unattested_context += 1;
            continue;
        }
        let candidate = crate::typing_memory::normalize_memory_word(&event.word);
        if candidate.is_empty() {
            report.skipped_missing_profile += 1;
            continue;
        }
        // Positive experience is only safe when the complete observed phrase is
        // lexically attested. A rejected candidate is allowed to be unknown: it
        // is destructive evidence, never an authority to promote a word.
        if polarity > 0 {
            let mut phrase = context.clone();
            phrase.push(candidate.clone());
            if !crate::typing_memory::phrase_is_attested_for_learning(&phrase.join(" ")) {
                report.skipped_unattested_positive += 1;
                continue;
            }
        }
        let token_hash = hash_text(&candidate);
        let Some(index) = package
            .profiles
            .binary_search_by_key(&token_hash, |profile| profile.token_hash)
            .ok()
        else {
            report.skipped_missing_profile += 1;
            continue;
        };
        let vector =
            package.candidate_relation_vector(&context, &candidate, ContextPhaseMode::Full);
        let profile = &mut package.profiles[index];
        if polarity > 0 {
            add_cluster(
                &mut profile.positive,
                &vector,
                MAX_POSITIVE_CENTERS,
                CENTER_SPLIT_COHERENCE,
            );
            profile.positive_examples = profile.positive_examples.saturating_add(1);
            report.positive_admitted += 1;
        } else {
            add_cluster(
                &mut profile.negative,
                &vector,
                MAX_ANTI_CENTERS,
                CENTER_SPLIT_COHERENCE,
            );
            profile.negative_examples = profile.negative_examples.saturating_add(1);
            report.negative_admitted += 1;
        }
    }
    package.transitions = package.transitions.saturating_add(
        u64::try_from(
            report
                .positive_admitted
                .saturating_add(report.negative_admitted),
        )
        .unwrap_or(u64::MAX),
    );
    report.positive_centers = package
        .profiles
        .iter()
        .map(|profile| profile.positive.len())
        .sum();
    report.anti_centers = package
        .profiles
        .iter()
        .map(|profile| profile.negative.len())
        .sum();
    Ok(report)
}

pub(crate) fn prove_context_phase(input: ContextPhaseCompileInput<'_>) -> ContextPhaseProofReport {
    let sequences = corpus_sequences(input.corpus_text, input.max_fragments);
    let (heldout, train): (Vec<_>, Vec<_>) = sequences
        .into_iter()
        .enumerate()
        .partition(|(index, _)| index % 5 == 4);
    let train_sequences = train
        .into_iter()
        .map(|(_, tokens)| tokens)
        .collect::<Vec<_>>();
    let heldout_sequences = heldout
        .into_iter()
        .map(|(_, tokens)| tokens)
        .collect::<Vec<_>>();
    let train_text = train_sequences
        .iter()
        .map(|tokens| tokens.join(" "))
        .collect::<Vec<_>>()
        .join(".\n");
    let (package, _) = compile_context_phase(ContextPhaseCompileInput {
        corpus_text: &train_text,
        max_fragments: 0,
        min_profile_support: input.min_profile_support,
    });
    let workers = context_phase_worker_count(heldout_sequences.len());
    let chunk_size = heldout_sequences.len().div_ceil(workers);
    let totals = thread::scope(|scope| {
        let mut tasks = Vec::new();
        for chunk in heldout_sequences.chunks(chunk_size) {
            tasks.push(scope.spawn(|| evaluate_heldout_sequences(&package, chunk)));
        }
        let mut totals = ContextPhaseProofTotals::default();
        for task in tasks {
            totals.merge(task.join().expect("L3 context proof worker panicked"));
        }
        totals
    });
    let ContextPhaseProofTotals {
        evaluated,
        full_supports,
        full_top1,
        full_false_supports,
        full_false_top1,
        no_phase_supports,
        no_anti_top1,
        no_anti_false_supports,
        no_anti_false_top1,
        no_semantic_top1,
    } = totals;
    let phase_ablation_drop = full_supports.saturating_sub(no_phase_supports);
    let anti_ablation_drop = full_top1.saturating_sub(no_anti_top1);
    let anti_false_support_reduction = no_anti_false_supports.saturating_sub(full_false_supports);
    let anti_false_top1_reduction = no_anti_false_top1.saturating_sub(full_false_top1);
    let semantic_ablation_drop = full_top1.saturating_sub(no_semantic_top1);
    let support_precision_ppm = ((full_supports as u64 * 1_000_000)
        / (full_supports + full_false_supports).max(1) as u64)
        .min(u64::from(u32::MAX)) as u32;
    let verdict = if evaluated > 0
        && full_supports > 0
        && full_false_top1 == 0
        && phase_ablation_drop > 0
        && semantic_ablation_drop > 0
    {
        "PASS"
    } else {
        "WATCH"
    };
    ContextPhaseProofReport {
        kind: "l3_context_phase_heldout_proof",
        train_fragments: train_sequences.len(),
        heldout_fragments: heldout_sequences.len(),
        evaluated_transitions: evaluated,
        full_supports,
        full_top1,
        full_false_supports,
        full_false_top1,
        no_phase_supports,
        no_anti_top1,
        no_anti_false_supports,
        no_anti_false_top1,
        no_semantic_top1,
        phase_ablation_drop,
        anti_ablation_drop,
        anti_false_support_reduction,
        anti_false_top1_reduction,
        semantic_ablation_drop,
        support_precision_ppm,
        raw_words_stored: false,
        min_profile_support: input.min_profile_support.max(2),
        verdict,
    }
}

fn evaluate_heldout_sequences(
    package: &ContextPhasePackage,
    sequences: &[Vec<String>],
) -> ContextPhaseProofTotals {
    let mut totals = ContextPhaseProofTotals::default();
    for tokens in sequences {
        for index in 1..tokens.len() {
            let target = &tokens[index];
            let competitors = l2_lattice_competitors(&tokens[..index], target, MAX_COMPETITORS);
            if competitors.is_empty() {
                continue;
            }
            let mut candidates = Vec::with_capacity(competitors.len() + 1);
            candidates.push(target.as_str());
            candidates.extend(competitors.iter().map(String::as_str));
            let full = package.score_candidates_with_mode(
                &tokens[..index],
                &candidates,
                ContextPhaseMode::Full,
            );
            if !full.first().is_some_and(|readout| readout.profile_present) {
                continue;
            }
            totals.evaluated += 1;
            totals.full_supports += full.first().is_some_and(|readout| {
                readout.disposition == super::ContextPhaseDisposition::Support
            }) as usize;
            totals.full_top1 += correct_is_unique_top(&full) as usize;
            totals.full_false_supports += full
                .iter()
                .skip(1)
                .filter(|readout| readout.disposition == super::ContextPhaseDisposition::Support)
                .count();
            totals.full_false_top1 += false_candidate_wins(&full) as usize;
            let no_phase = package.score_candidates_with_mode(
                &tokens[..index],
                &candidates,
                ContextPhaseMode::NoPhase,
            );
            totals.no_phase_supports += no_phase.first().is_some_and(|readout| {
                readout.disposition == super::ContextPhaseDisposition::Support
            }) as usize;
            let no_anti = package.score_candidates_with_mode(
                &tokens[..index],
                &candidates,
                ContextPhaseMode::NoAnti,
            );
            totals.no_anti_top1 += correct_is_unique_top(&no_anti) as usize;
            totals.no_anti_false_supports += no_anti
                .iter()
                .skip(1)
                .filter(|readout| readout.disposition == super::ContextPhaseDisposition::Support)
                .count();
            totals.no_anti_false_top1 += false_candidate_wins(&no_anti) as usize;
            let no_semantic = package.score_candidates_with_mode(
                &tokens[..index],
                &candidates,
                ContextPhaseMode::NoSemanticState,
            );
            totals.no_semantic_top1 += correct_is_unique_top(&no_semantic) as usize;
        }
    }
    totals
}

fn correct_is_unique_top(readouts: &[super::ContextPhaseReadout]) -> bool {
    let Some(correct) = readouts.first() else {
        return false;
    };
    correct.disposition == super::ContextPhaseDisposition::Support
        && readouts
            .iter()
            .skip(1)
            .all(|readout| readout.margin_micro < correct.margin_micro)
}

fn false_candidate_wins(readouts: &[super::ContextPhaseReadout]) -> bool {
    let Some(correct) = readouts.first() else {
        return false;
    };
    readouts.iter().skip(1).any(|candidate| {
        candidate.disposition == super::ContextPhaseDisposition::Support
            && candidate.margin_micro >= correct.margin_micro
    })
}

fn compile_semantic_states(sequences: &[Vec<String>]) -> Vec<TokenSemanticState> {
    let mut builders = BTreeMap::<u64, SemanticBuilder>::new();
    for tokens in sequences {
        for (index, token) in tokens.iter().enumerate() {
            let token_hash = hash_text(token);
            let builder = builders
                .entry(token_hash)
                .or_insert_with(|| SemanticBuilder {
                    sum: empty_vector(CELLS),
                    support: 0,
                });
            builder.support = builder.support.saturating_add(1);
            let start = index.saturating_sub(4);
            let end = (index + 5).min(tokens.len());
            for (neighbor_index, neighbor) in tokens[start..end].iter().enumerate() {
                let absolute = start + neighbor_index;
                if absolute == index {
                    continue;
                }
                let relative = absolute as isize - index as isize;
                let position = relative.unsigned_abs() as u64;
                let direction = if relative < 0 { 0x4c } else { 0x52 };
                add_hashed_atom(
                    &mut builder.sum,
                    hash_text(neighbor) ^ (direction << 56),
                    token_hash ^ position.rotate_left(11),
                    1.0 / (position as f32).sqrt(),
                );
            }
        }
    }
    builders
        .into_iter()
        .filter(|(_, builder)| builder.support >= 2)
        .map(|(token_hash, builder)| TokenSemanticState {
            token_hash,
            support: builder.support,
            center: phase_center_from_sum(&builder.sum),
        })
        .collect()
}

fn learned_threshold(builder: &ProfileBuilder) -> i32 {
    let mut positive = builder
        .positive_vectors
        .iter()
        .map(|vector| margin(vector, &builder.positive, &builder.negative))
        .collect::<Vec<_>>();
    let mut negative = builder
        .negative_vectors
        .iter()
        .map(|vector| margin(vector, &builder.positive, &builder.negative))
        .collect::<Vec<_>>();
    positive.sort_by(f32::total_cmp);
    negative.sort_by(f32::total_cmp);
    let positive_floor = percentile(&positive, 10).unwrap_or(0.0);
    let negative_ceiling = percentile(&negative, 90).unwrap_or(0.0);
    let threshold = if !negative.is_empty() && positive_floor > negative_ceiling {
        negative_ceiling + (positive_floor - negative_ceiling) * 0.50
    } else {
        positive_floor * 0.70
    };
    phase_micro(threshold).clamp(i32::MIN as i64, i32::MAX as i64) as i32
}

fn learned_global_threshold(profiles: &[ContextCandidateProfile]) -> i32 {
    let mut thresholds = profiles
        .iter()
        .filter(|profile| profile.positive_examples >= 2)
        .map(|profile| profile.threshold_micro)
        .collect::<Vec<_>>();
    thresholds.sort_unstable();
    percentile_i32(&thresholds, 25).unwrap_or(1).max(1)
}

fn learned_competition_threshold(
    package: &ContextPhasePackage,
    sequences: &[Vec<String>],
    competitors: &BTreeMap<String, Vec<String>>,
) -> i32 {
    let mut correct_gaps = Vec::new();
    let mut wrong_gaps = Vec::new();
    for tokens in sequences.iter().take(10_000) {
        for index in 1..tokens.len() {
            let target = &tokens[index];
            let Some(negative) = competitors.get(target) else {
                continue;
            };
            if negative.is_empty() {
                continue;
            }
            let mut candidates = Vec::with_capacity(negative.len() + 1);
            candidates.push(target.as_str());
            candidates.extend(negative.iter().map(String::as_str));
            let readouts = package.score_candidates_with_mode(
                &tokens[..index],
                &candidates,
                ContextPhaseMode::Full,
            );
            let correct = readouts
                .first()
                .map(|item| item.margin_micro)
                .unwrap_or_default();
            let wrong = readouts
                .iter()
                .skip(1)
                .map(|item| item.margin_micro)
                .max()
                .unwrap_or(i64::MIN / 2);
            if correct > wrong {
                correct_gaps.push((correct - wrong).min(i64::from(i32::MAX)) as i32);
            } else if wrong > correct {
                wrong_gaps.push((wrong - correct).min(i64::from(i32::MAX)) as i32);
            }
        }
    }
    correct_gaps.sort_unstable();
    wrong_gaps.sort_unstable();
    let positive_floor = percentile_i32(&correct_gaps, 25).unwrap_or(1).max(1);
    let negative_ceiling = percentile_i32(&wrong_gaps, 90).unwrap_or(0).max(0);
    if positive_floor > negative_ceiling {
        negative_ceiling + (positive_floor - negative_ceiling) / 2
    } else {
        // The field has not separated this candidate family enough to grant a
        // narrow winning margin. Keep the learned positive floor and abstain
        // on close competition instead of promoting a false attractor.
        positive_floor
    }
}

fn corpus_sequences(text: &str, max_fragments: usize) -> Vec<Vec<String>> {
    text.split(['\n', '.', '!', '?', ';'])
        .filter_map(|fragment| {
            let tokens = super::super::llmwave::tokenize(fragment);
            (tokens.len() >= 3 && tokens.len() <= MAX_FRAGMENT_TOKENS).then_some(tokens)
        })
        .take(if max_fragments == 0 {
            usize::MAX
        } else {
            max_fragments
        })
        .collect()
}

fn l2_lattice_competitors(context: &[String], target: &str, limit: usize) -> Vec<String> {
    if limit == 0 {
        return Vec::new();
    }
    let context_prefix = context.join(" ");
    let mut competitors = BTreeSet::new();
    for damaged in training_surfaces(target) {
        for candidate in crate::nanda_wave::l2::correction_l2_word_candidates(
            &context_prefix,
            &damaged,
            limit.saturating_mul(4),
        ) {
            let candidate = candidate.surface.to_lowercase();
            if candidate != target {
                competitors.insert(candidate);
            }
        }
    }
    let mut competitors = competitors.into_iter().collect::<Vec<_>>();
    competitors.sort_by(|left, right| {
        crate::text_metrics::damerau_levenshtein(target, left)
            .cmp(&crate::text_metrics::damerau_levenshtein(target, right))
            .then_with(|| left.cmp(right))
    });
    competitors.truncate(limit);
    competitors
}

fn compile_l2_competitor_cache(sequences: &[Vec<String>]) -> BTreeMap<String, Vec<String>> {
    let mut first_contexts = BTreeMap::<String, Vec<String>>::new();
    for tokens in sequences {
        for index in 1..tokens.len() {
            first_contexts
                .entry(tokens[index].clone())
                .or_insert_with(|| tokens[..index].to_vec());
        }
    }
    let jobs = first_contexts.into_iter().collect::<Vec<_>>();
    if jobs.is_empty() {
        return BTreeMap::new();
    }
    let workers = context_phase_worker_count(jobs.len());
    let chunk_size = jobs.len().div_ceil(workers);
    let mut cache = BTreeMap::new();
    thread::scope(|scope| {
        let mut tasks = Vec::new();
        for chunk in jobs.chunks(chunk_size) {
            tasks.push(scope.spawn(move || {
                chunk
                    .iter()
                    .map(|(target, context)| {
                        (
                            target.clone(),
                            l2_lattice_competitors(context, target, MAX_COMPETITORS),
                        )
                    })
                    .collect::<Vec<_>>()
            }));
        }
        for task in tasks {
            for (target, competitors) in task.join().expect("L2 lattice worker panicked") {
                cache.insert(target, competitors);
            }
        }
    });
    cache
}

fn context_phase_worker_count(work_items: usize) -> usize {
    let requested = env::var("LAY_L3_CONTEXT_WORKERS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|workers| *workers > 0);
    requested
        .or_else(|| {
            thread::available_parallelism()
                .ok()
                .map(|value| value.get())
        })
        .unwrap_or(1)
        .min(work_items.max(1))
}

fn training_surfaces(target: &str) -> Vec<String> {
    let chars = target.chars().collect::<Vec<_>>();
    if chars.len() < 3 || !chars.iter().all(|ch| ch.is_alphabetic()) {
        return Vec::new();
    }

    let mut positions = [1, chars.len() / 2, chars.len().saturating_sub(2)]
        .into_iter()
        .filter(|position| *position > 0 && *position + 1 < chars.len())
        .collect::<Vec<_>>();
    positions.sort_unstable();
    positions.dedup();

    let mut surfaces = positions
        .into_iter()
        .map(|position| {
            chars
                .iter()
                .enumerate()
                .filter_map(|(index, ch)| (index != position).then_some(*ch))
                .collect::<String>()
        })
        .collect::<Vec<_>>();
    if chars.len() >= 4 {
        let position = chars.len() / 2 - 1;
        let mut transposed = chars.clone();
        transposed.swap(position, position + 1);
        surfaces.push(transposed.into_iter().collect());
    }
    surfaces.sort();
    surfaces.dedup();
    surfaces.truncate(MAX_L2_TRAINING_SURFACES);
    surfaces
}

fn percentile(values: &[f32], percentile: usize) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    let index = values
        .len()
        .saturating_sub(1)
        .saturating_mul(percentile.min(100))
        / 100;
    values.get(index).copied()
}

fn percentile_i32(values: &[i32], percentile: usize) -> Option<i32> {
    if values.is_empty() {
        return None;
    }
    let index = values
        .len()
        .saturating_sub(1)
        .saturating_mul(percentile.min(100))
        / 100;
    values.get(index).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiler_learns_context_centers_and_destructive_competitors() {
        let corpus = "на улице снова идет дождь. вечером на улице идет дождь. в доме снова горит свет. вечером в доме горит свет.";
        let (package, report) = compile_context_phase(ContextPhaseCompileInput {
            corpus_text: corpus,
            max_fragments: 0,
            min_profile_support: 2,
        });

        assert!(report.semantic_states > 0);
        assert!(report.candidate_profiles > 0);
        assert!(report.positive_centers > 0);
        assert!(report.l2_lattice_negative_examples > 0);
        let readouts = package.score_candidates(
            &super::super::super::llmwave::tokenize("вечером на улице идет"),
            &["дождь", "домик"],
        );
        assert!(readouts[0].profile_present);
        assert!(readouts[0].margin_micro >= readouts[1].margin_micro);
    }

    #[test]
    fn feedback_overlay_trains_existing_profiles_without_storing_live_text() {
        let corpus = concat!(
            "на улице идет дождь. ",
            "вечером на улице идет дождь. ",
            "в комнате горит свет. ",
            "вечером в комнате горит свет."
        );
        let (mut package, _) = compile_context_phase(ContextPhaseCompileInput {
            corpus_text: corpus,
            max_fragments: 0,
            min_profile_support: 2,
        });
        let rain_hash = hash_text("дождь");
        let light_hash = hash_text("свет");
        let before_rain = package
            .profile(rain_hash)
            .expect("corpus profile")
            .positive_examples;
        let before_light = package
            .profile(light_hash)
            .expect("corpus profile")
            .negative_examples;
        let events = concat!(
            r#"{"kind":"confirmed_ime_prediction","word":"дождь","context":["на","улице","идёт"]}"#,
            "\n",
            r#"{"kind":"rejected_ime","word":"свет","context":["на","улице","идёт"]}"#,
            "\n",
            r#"{"kind":"typed","word":"мусор","context":["на","улице"]}"#,
        );

        let report = apply_feedback_overlay(&mut package, events).expect("valid feedback");

        assert!(!report.raw_words_stored);
        assert_eq!(report.positive_admitted, 1);
        assert_eq!(report.negative_admitted, 1);
        assert_eq!(report.source_events, 2);
        assert_eq!(
            package.profile(rain_hash).unwrap().positive_examples,
            before_rain + 1
        );
        assert_eq!(
            package.profile(light_hash).unwrap().negative_examples,
            before_light + 1
        );
        let dir =
            std::env::temp_dir().join(format!("lay-l3-feedback-overlay-{}", std::process::id()));
        let path = dir.join("feedback.nwpc");
        std::fs::create_dir_all(&dir).unwrap();
        super::super::write_package(&path, &package).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert!(!bytes
            .windows("дождь".len())
            .any(|window| window == "дождь".as_bytes()));
        let _ = std::fs::remove_dir_all(dir);
    }
}
