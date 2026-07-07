use crate::config::CorrectionSafety;
use crate::dict::{convert, detect_direction};
use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::{
    is_common_en_guard_prefix, is_common_en_technical_word, is_common_ru_word,
    is_ru_live_protected_word, is_ru_one_letter_function_word, is_ru_short_function_word,
    is_user_protected_word, visual_b_after_ascii_replacement, visual_b_default_replacement,
};
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::text_case::apply_word_case;
use crate::text_metrics::damerau_levenshtein;
use crate::typing_candidate::TypingCandidateFamily;
use crate::typing_context;
use crate::typing_pipeline::explain_typing_assist_with_pipeline;
use crate::word_reader::{split_last_ws_token, split_word_punctuation, split_ws_segments};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use super::context::{TailContext, TokenKind};
use super::feedback::{apply_l3_feedback, L3Feedback};
use super::l2_center_memory::{L2CenterMemory, L2CenterMemoryConfig};
use super::lexical_attractor::{lexical_attractor_candidates, LEXICAL_ATTRACTOR_CELL};
use super::options::WaveOptions;
use super::pattern_memory::{apply_pattern_memory, PATTERN_MEMORY_CELL};
use super::signal::{WavePacket, WordCandidate};

static SURFACE_MOTIF_MEMORY: OnceLock<L2CenterMemory> = OnceLock::new();
static BROAD_PREFIX_INDEX: OnceLock<super::l2_broad_index::L2BroadPrefixIndex> = OnceLock::new();
static L2_SHORT_POSITION_SEED_INDEX: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();

const MAX_LAYOUT_SCAN_CANDIDATES: usize = 4;
const MAX_TAUGHT_CANDIDATES: usize = 6;
const L2_ACTIVE_SOURCE_TARGET: usize = 1_000_000;
const L2_RUNTIME_WORD_LIMIT: usize = 100_000;
const L2_FOUNDATION_SOURCE_LIMIT: usize = 100_000;
const L2_FOUNDATION_LIVE_SCAN_LIMIT: usize = 100_000;
const L2_USAGE_WORD_LIMIT: usize = 5_000;
const L2_CASE_WORD_LIMIT: usize = 200;
const L2_BROAD_PREFIX_SCAN_LIMIT: usize = 384;
const L2_SURFACE_FOUNDATION_RU_DATA: &str =
    include_str!("../../data/lexicon/l2_surface_foundation_ru_100k.txt");
const L2_SURFACE_HOT_RU_DATA: &str = include_str!("../../data/lexicon/l2_surface_hot_ru.txt");
pub(super) const L2_SURFACE_MOTIF_CELL: &str = "L2SurfaceMotifCell32";
pub(super) const L2_SURFACE_COMPLETION_CELL: &str = "L2SurfaceCompletionCell32";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum L2ImeWordCandidateKind {
    Completion,
    Replacement,
}

#[derive(Clone, Debug, PartialEq)]
pub struct L2ImeWordCandidate {
    pub surface: String,
    pub kind: L2ImeWordCandidateKind,
    pub score: u32,
    pub l1_overlap: usize,
    pub l2_overlap: usize,
    pub motif_overlap: usize,
    pub usage_prior: f32,
    pub context_prior: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct L2SurfaceMemoryStatus {
    pub active_source_target: usize,
    pub hot_center_words: usize,
    pub hot_center_records: usize,
    pub hot_center_motifs: usize,
    pub hot_center_token_refs: usize,
    pub hot_center_bytes: usize,
    pub broad_source_words: usize,
    pub broad_prefix_keys: usize,
    pub broad_word_refs: usize,
    pub foundation_source_limit: usize,
    pub foundation_live_scan_limit: usize,
    pub generated_forms_loaded: bool,
    pub generated_forms_words: usize,
}

struct TaughtCandidateInput<'a> {
    original: &'a str,
    context: &'a TailContext,
    l1: &'a [WavePacket],
    options: &'a WaveOptions,
    replacement: &'a str,
    family: TypingCandidateFamily,
    score: f64,
    chosen: bool,
}

pub fn run_l2(original: &str, l1: &[WavePacket]) -> Vec<WordCandidate> {
    run_l2_with_options(original, l1, &WaveOptions::default())
}

pub fn run_l2_with_options(
    original: &str,
    l1: &[WavePacket],
    options: &WaveOptions,
) -> Vec<WordCandidate> {
    run_l2_refined_with_feedback(original, l1, options, &L3Feedback::default())
}

pub(super) fn warm_up_surface_motif_memory() {
    let _ = surface_motif_memory().center_count();
}

pub(super) fn warm_up_ime_word_candidate_memory() {
    // Live IME must not build broad/full L2 indexes during startup. The hot
    // path uses a bounded corpus bootstrap and only reads heavy memories after
    // another route has already warmed them.
}

pub fn l2_surface_memory_status() -> L2SurfaceMemoryStatus {
    let hot = surface_motif_memory();
    let broad = broad_prefix_index().stats();
    let generated_forms_loaded =
        crate::russian_lexicon::russian_generated_form_dictionary_is_warm();
    let generated_forms_words = if generated_forms_loaded {
        crate::russian_lexicon::russian_generated_form_dictionary().len()
    } else {
        0
    };
    L2SurfaceMemoryStatus {
        active_source_target: L2_ACTIVE_SOURCE_TARGET,
        hot_center_words: hot.source_word_count(),
        hot_center_records: hot.word_records().len(),
        hot_center_motifs: hot.center_count(),
        hot_center_token_refs: hot.token_refs().len(),
        hot_center_bytes: hot.hot_bytes(),
        broad_source_words: broad.source_words,
        broad_prefix_keys: broad.prefix_keys,
        broad_word_refs: broad.word_refs,
        foundation_source_limit: L2_FOUNDATION_SOURCE_LIMIT,
        foundation_live_scan_limit: L2_FOUNDATION_LIVE_SCAN_LIMIT,
        generated_forms_loaded,
        generated_forms_words,
    }
}

pub fn ime_l2_word_candidates(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    if limit == 0 {
        return Vec::new();
    }
    let normalized = token.to_lowercase();
    let token_len = normalized.chars().count();
    if !(2..=18).contains(&token_len) || !normalized.chars().all(is_cyrillic_letter) {
        return Vec::new();
    }
    let context_tokens = super::llmwave::tokenize(context_prefix);
    let usage = super::usage_prior::cached_usage_prior_snapshot();
    let mut candidates = surface_motif_memory()
        .surface_candidates_for_text_with_usage(
            &normalized,
            limit.saturating_mul(4).max(limit),
            &usage,
        )
        .into_iter()
        .map(|candidate| {
            let candidate_len = candidate.word.chars().count();
            let kind = if candidate.word.starts_with(&normalized) && candidate_len > token_len {
                L2ImeWordCandidateKind::Completion
            } else {
                L2ImeWordCandidateKind::Replacement
            };
            let usage_prior = usage.word_prior(&candidate.word);
            let context_prior = usage.context_word_prior(&context_tokens, &candidate.word);
            L2ImeWordCandidate {
                surface: candidate.word,
                kind,
                score: candidate.score,
                l1_overlap: candidate.l1_overlap,
                l2_overlap: candidate.l2_overlap,
                motif_overlap: candidate.motif_overlap,
                usage_prior,
                context_prior,
            }
        })
        .collect::<Vec<_>>();
    extend_ime_l2_prefix_material(&mut candidates, context_prefix, &normalized, limit);
    sort_and_truncate_ime_l2_candidates(&mut candidates, &usage, limit);
    candidates
}

fn sort_and_truncate_ime_l2_candidates(
    candidates: &mut Vec<L2ImeWordCandidate>,
    usage: &super::usage_prior::UsagePriorSnapshot,
    limit: usize,
) {
    candidates.sort_by(|left, right| {
        l2_ime_word_candidate_score(right, &usage)
            .cmp(&l2_ime_word_candidate_score(left, &usage))
            .then_with(|| right.motif_overlap.cmp(&left.motif_overlap))
            .then_with(|| right.l2_overlap.cmp(&left.l2_overlap))
            .then_with(|| right.l1_overlap.cmp(&left.l1_overlap))
            .then_with(|| {
                left.surface
                    .chars()
                    .count()
                    .cmp(&right.surface.chars().count())
            })
            .then_with(|| left.surface.cmp(&right.surface))
    });
    candidates.dedup_by(|left, right| left.surface == right.surface);
    candidates.truncate(limit);
}

fn extend_ime_l2_prefix_material(
    candidates: &mut Vec<L2ImeWordCandidate>,
    context_prefix: &str,
    token: &str,
    limit: usize,
) {
    if ime_l2_completion_count(candidates) >= limit {
        return;
    }
    let material_limit = limit.saturating_mul(2).max(limit);
    for candidate in ime_l2_generated_form_prefix_candidates(context_prefix, token, material_limit) {
        push_unique_ime_l2_candidate(candidates, candidate);
        if ime_l2_completion_count(candidates) >= limit.saturating_mul(2).max(limit) {
            return;
        }
    }
    for candidate in ime_l2_foundation_prefix_candidates(context_prefix, token, material_limit) {
        push_unique_ime_l2_candidate(candidates, candidate);
        if ime_l2_completion_count(candidates) >= limit.saturating_mul(2).max(limit) {
            return;
        }
    }
    if token.chars().count() <= 4 {
        for candidate in ime_l2_short_seed_word_candidates(context_prefix, token, material_limit) {
            push_unique_ime_l2_candidate(candidates, candidate);
            if ime_l2_completion_count(candidates) >= limit.saturating_mul(2).max(limit) {
                return;
            }
        }
    }
}

fn ime_l2_completion_count(candidates: &[L2ImeWordCandidate]) -> usize {
    candidates
        .iter()
        .filter(|candidate| candidate.kind == L2ImeWordCandidateKind::Completion)
        .count()
}

fn push_unique_ime_l2_candidate(
    candidates: &mut Vec<L2ImeWordCandidate>,
    candidate: L2ImeWordCandidate,
) {
    if candidates
        .iter()
        .any(|existing| existing.surface == candidate.surface)
    {
        return;
    }
    candidates.push(candidate);
}

pub fn ime_l2_short_seed_word_candidates(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    if limit == 0 {
        return Vec::new();
    }
    let normalized = token.to_lowercase();
    let token_len = normalized.chars().count();
    if !(2..=4).contains(&token_len) || !normalized.chars().all(is_cyrillic_letter) {
        return Vec::new();
    }
    let Some(words) = l2_short_position_seed_index().get(&normalized) else {
        return Vec::new();
    };
    let context_tokens = super::llmwave::tokenize(context_prefix);
    let usage = super::usage_prior::cached_usage_prior_snapshot();
    let mut words = words.to_vec();
    words.sort_by(|left, right| {
        compare_l2_words_by_usage(right, left, &context_tokens, &usage)
            .then_with(|| left.chars().count().cmp(&right.chars().count()))
            .then_with(|| left.cmp(right))
    });
    words
        .iter()
        .take(limit)
        .map(|word| {
            let usage_prior = usage.word_prior(word);
            let context_prior = usage.context_word_prior(&context_tokens, word);
            L2ImeWordCandidate {
                surface: word.clone(),
                kind: L2ImeWordCandidateKind::Completion,
                score: 520,
                l1_overlap: token_len,
                l2_overlap: 0,
                motif_overlap: 0,
                usage_prior,
                context_prior,
            }
        })
        .collect()
}

pub fn ime_l2_foundation_prefix_candidates(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    ime_l2_foundation_prefix_candidates_from_index(
        broad_prefix_index(),
        context_prefix,
        token,
        limit,
    )
}

fn ime_l2_foundation_prefix_candidates_from_index(
    index: &super::l2_broad_index::L2BroadPrefixIndex,
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    if limit == 0 {
        return Vec::new();
    }
    let normalized = token.to_lowercase();
    let token_len = normalized.chars().count();
    if !(2..=18).contains(&token_len) || !normalized.chars().all(is_cyrillic_letter) {
        return Vec::new();
    }
    let context_tokens = super::llmwave::tokenize(context_prefix);
    let usage = super::usage_prior::cached_usage_prior_snapshot();
    let mut words = index
        .prefix_candidates(
            &normalized,
            token_len + 1,
            32,
            limit
                .saturating_mul(16)
                .clamp(L2_BROAD_PREFIX_SCAN_LIMIT, L2_FOUNDATION_LIVE_SCAN_LIMIT),
        )
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    words.sort_by(|left, right| {
        compare_l2_words_by_usage(right, left, &context_tokens, &usage)
            .then_with(|| {
                crate::lexicon::is_common_ru_word(right)
                    .cmp(&crate::lexicon::is_common_ru_word(left))
            })
            .then_with(|| left.chars().count().cmp(&right.chars().count()))
            .then_with(|| left.cmp(right))
    });
    words.truncate(limit);
    words
        .into_iter()
        .map(|word| {
            let usage_prior = usage.word_prior(&word);
            let context_prior = usage.context_word_prior(&context_tokens, &word);
            L2ImeWordCandidate {
                surface: word,
                kind: L2ImeWordCandidateKind::Completion,
                score: 610,
                l1_overlap: token_len,
                l2_overlap: 0,
                motif_overlap: 0,
                usage_prior,
                context_prior,
            }
        })
        .collect()
}

pub fn ime_l2_generated_form_prefix_candidates(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    if limit == 0 {
        return Vec::new();
    }
    let normalized = token.to_lowercase();
    let token_len = normalized.chars().count();
    if !(3..=18).contains(&token_len) || !normalized.chars().all(is_cyrillic_letter) {
        return Vec::new();
    }
    if !crate::russian_lexicon::russian_generated_form_dictionary_is_warm() {
        return Vec::new();
    }
    let max_len = (token_len + 12).min(32);
    let mut words = crate::russian_lexicon::russian_generated_form_dictionary().prefix_words(
        &normalized,
        token_len + 1,
        max_len,
        limit.saturating_mul(4).max(limit),
    );
    let context_tokens = super::llmwave::tokenize(context_prefix);
    let usage = super::usage_prior::cached_usage_prior_snapshot();
    words.sort_by(|left, right| {
        compare_l2_words_by_usage(right, left, &context_tokens, &usage)
            .then_with(|| {
                crate::lexicon::is_common_ru_word(right)
                    .cmp(&crate::lexicon::is_common_ru_word(left))
            })
            .then_with(|| left.chars().count().cmp(&right.chars().count()))
            .then_with(|| left.cmp(right))
    });
    words.truncate(limit);
    words
        .into_iter()
        .map(|word| {
            let usage_prior = usage.word_prior(&word);
            let context_prior = usage.context_word_prior(&context_tokens, &word);
            L2ImeWordCandidate {
                surface: word,
                kind: L2ImeWordCandidateKind::Completion,
                score: 650,
                l1_overlap: token_len,
                l2_overlap: 0,
                motif_overlap: 0,
                usage_prior,
                context_prior,
            }
        })
        .collect()
}

fn l2_short_position_seed_index() -> &'static HashMap<String, Vec<String>> {
    L2_SHORT_POSITION_SEED_INDEX.get_or_init(|| {
        let mut index = HashMap::<String, Vec<String>>::new();
        for word in runtime_l2_surface_words() {
            let len = word.chars().count();
            if !(3..=18).contains(&len) || !word.chars().all(is_cyrillic_letter) {
                continue;
            }
            for prefix_len in 2..=4.min(len.saturating_sub(1)) {
                let key = word.chars().take(prefix_len).collect::<String>();
                index.entry(key).or_default().push(word.clone());
            }
        }
        let usage = super::usage_prior::cached_usage_prior_snapshot();
        for words in index.values_mut() {
            words.sort_by(|left, right| {
                usage
                    .word_prior(right)
                    .total_cmp(&usage.word_prior(left))
                    .then_with(|| {
                        crate::lexicon::is_common_ru_word(right)
                            .cmp(&crate::lexicon::is_common_ru_word(left))
                    })
                    .then_with(|| left.chars().count().cmp(&right.chars().count()))
                    .then_with(|| left.cmp(right))
            });
            words.truncate(16);
        }
        index
    })
}

fn l2_ime_word_candidate_score(
    candidate: &L2ImeWordCandidate,
    usage: &super::usage_prior::UsagePriorSnapshot,
) -> u32 {
    let prior = ((candidate.usage_prior * 1600.0 + candidate.context_prior * 2600.0)
        .round()
        .clamp(0.0, 820.0) as u32)
        .saturating_add(usage.accepted_word_count(&candidate.surface).min(40) * 18);
    let kind_bonus = match candidate.kind {
        L2ImeWordCandidateKind::Completion => 80,
        L2ImeWordCandidateKind::Replacement => 0,
    };
    candidate
        .score
        .saturating_add(prior)
        .saturating_add(kind_bonus)
}

fn compare_l2_words_by_usage(
    left: &str,
    right: &str,
    context_tokens: &[String],
    usage: &super::usage_prior::UsagePriorSnapshot,
) -> std::cmp::Ordering {
    l2_word_usage_rank(left, context_tokens, usage).cmp(&l2_word_usage_rank(
        right,
        context_tokens,
        usage,
    ))
}

fn l2_word_usage_rank(
    word: &str,
    context_tokens: &[String],
    usage: &super::usage_prior::UsagePriorSnapshot,
) -> u32 {
    let usage_prior = usage.word_prior(word);
    let context_prior = usage.context_word_prior(context_tokens, word);
    let accepted = usage.accepted_word_count(word).min(40);
    ((usage_prior * 1600.0 + context_prior * 2600.0)
        .round()
        .clamp(0.0, 820.0) as u32)
        .saturating_add(accepted * 18)
}

pub fn run_l2_refined_with_feedback(
    original: &str,
    l1: &[WavePacket],
    options: &WaveOptions,
    feedback: &L3Feedback,
) -> Vec<WordCandidate> {
    let tail = original.trim_end();
    let Some((prefix, token)) = split_last_ws_token(tail) else {
        return Vec::new();
    };
    let context = TailContext::from_text(tail);
    let mut candidates = Vec::new();
    let timing_enabled = std::env::var_os("LAY_NANDA_L2_TIMING").is_some();
    let mut timing_last = std::time::Instant::now();
    macro_rules! mark_timing {
        ($stage:literal) => {
            if timing_enabled {
                let now = std::time::Instant::now();
                eprintln!(
                    "lay_nanda_l2_timing stage={} elapsed_us={} candidates={}",
                    $stage,
                    now.duration_since(timing_last).as_micros(),
                    candidates.len()
                );
                timing_last = now;
            }
        };
    }
    if options.is_enabled("LayoutWordCell32") {
        if let Some(candidate) = layout_candidate(prefix, token, &context, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
        for candidate in layout_scan_candidates(tail, &context, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("layout");
    if options.is_enabled("ShortTokenCell32") {
        for candidate in short_token_candidates(prefix, token, &context, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("short-token");
    if options.is_enabled("TechTokenCell32") {
        if let Some(candidate) = technical_keep_candidate(token, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
        if let Some(candidate) = technical_context_keep_candidate(tail, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("tech-token");
    let boundary_scan = if options.is_enabled("BoundaryCell32") {
        boundary_scan_candidates(tail, l1, &context)
    } else {
        Vec::new()
    };
    let mut has_l2_surface_motif_candidate = false;
    if boundary_scan.is_empty()
        && (options.is_enabled(L2_SURFACE_MOTIF_CELL)
            || options.is_enabled(L2_SURFACE_COMPLETION_CELL))
    {
        for candidate in surface_motif_word_candidates(prefix, token, &context, l1, options) {
            has_l2_surface_motif_candidate |= candidate.source == L2_SURFACE_MOTIF_CELL;
            push_unique_candidate(&mut candidates, candidate);
        }
        if options.is_enabled(L2_SURFACE_MOTIF_CELL) {
            for candidate in surface_motif_scan_candidates(tail, l1, &context) {
                has_l2_surface_motif_candidate |= candidate.source == L2_SURFACE_MOTIF_CELL;
                push_unique_candidate(&mut candidates, candidate);
            }
        }
    }
    mark_timing!("surface-motif");
    let has_explicit_boundary_split = token.chars().all(is_cyrillic_letter)
        && light_boundary_replacement(&token.to_lowercase()).is_some();
    if options.is_enabled("BoundaryCell32") {
        if !has_l2_surface_motif_candidate || has_explicit_boundary_split {
            for candidate in boundary_split_candidates(prefix, token, l1, &context) {
                push_unique_candidate(&mut candidates, candidate);
            }
        }
        for candidate in boundary_scan {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("boundary");
    if options.is_enabled(LEXICAL_ATTRACTOR_CELL) {
        for candidate in lexical_attractor_candidates(tail, &context) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("lexical-attractor");
    if memory_cells_enabled(options) {
        for candidate in super::learned::learned_candidates(tail)
            .into_iter()
            .filter(|candidate| input_source_enabled(candidate.source, options))
        {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("learned-memory");
    if options.is_enabled(super::context_wave::SEMANTIC_WORD_SOURCE) {
        for candidate in super::context_wave::semantic_word_candidates(tail) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("semantic-word");
    if options.is_enabled("PhraseCell32") {
        for candidate in customs_actor_phrase_candidates(tail, &context, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("phrase");
    if options.is_enabled(super::context_wave::PHRASE_FORECAST_CELL) && options.llmwave_shadow() {
        let memory = phrase_forecast_memory();
        for candidate in super::llmwave::phrase_forecast_candidates(tail, &memory) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("phrase-forecast");
    if options.is_enabled("GrammarCell32") {
        for candidate in grammar_agreement_candidates(tail, &context, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("grammar");
    if should_run_taught_candidates(token, options) {
        for candidate in taught_candidates(tail, &context, l1, options) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("taught");
    if options.is_enabled(PATTERN_MEMORY_CELL) {
        let report = apply_pattern_memory(tail, &mut candidates);
        if report.applied > 0 {
            candidates.iter_mut().for_each(|candidate| {
                candidate.support.push(format!(
                    "pattern-memory-report:records={} applied={}",
                    report.records, report.applied
                ));
            });
        }
    }
    mark_timing!("pattern-memory");
    apply_l2_phase_shadow(tail, &mut candidates, options);
    mark_timing!("l2-phase-shadow");
    apply_l2_weight(&mut candidates, options);
    candidates.sort_by(|left, right| {
        right
            .energy
            .total_cmp(&left.energy)
            .then_with(|| left.risk.total_cmp(&right.risk))
    });
    apply_l3_feedback(&mut candidates, feedback);
    candidates.sort_by(|left, right| {
        right
            .energy
            .total_cmp(&left.energy)
            .then_with(|| left.risk.total_cmp(&right.risk))
    });
    mark_timing!("feedback-sort");
    if timing_enabled {
        let _ = timing_last.elapsed();
    }
    candidates
}

fn apply_l2_weight(candidates: &mut [WordCandidate], options: &WaveOptions) {
    if (options.l2_weight() - 1.0).abs() < f32::EPSILON {
        return;
    }
    for candidate in candidates {
        candidate.energy = options.scale_l2_energy(candidate.energy);
        candidate
            .support
            .push(format!("l2-weight:{:.2}", options.l2_weight()));
    }
}

fn apply_l2_phase_shadow(
    original: &str,
    candidates: &mut Vec<WordCandidate>,
    options: &WaveOptions,
) {
    if !options.l2_phase_shadow() {
        return;
    }
    let mut package_loaded_any = false;
    for candidate in candidates.iter_mut() {
        let operation = l2_phase_operation(candidate.source);
        let (loaded, margin_micro, admitted) =
            super::l2_candidate_phase_shadow(original, &candidate.text, operation);
        package_loaded_any |= loaded;
        candidate.support.push(format!(
            "l2-phase:loaded={} margin={} admitted={}",
            loaded, margin_micro, admitted
        ));
    }
    if !options.l2_phase_apply() || !package_loaded_any {
        return;
    }
    for candidate in candidates.iter_mut() {
        if candidate_has_l2_phase_admission(candidate) {
            candidate.energy = (candidate.energy + 0.025).min(1.0);
        }
    }
    candidates.retain(|candidate| {
        candidate_has_l2_phase_admission(candidate) || !l2_phase_apply_source(candidate.source)
    });
}

fn candidate_has_l2_phase_admission(candidate: &WordCandidate) -> bool {
    candidate
        .support
        .iter()
        .any(|item| item.contains("l2-phase:loaded=true") && item.contains("admitted=true"))
}

fn l2_phase_operation(source: &str) -> &'static str {
    match source {
        "LayoutWordCell32" | "LearnedMemoryCell32" => "layout",
        "BoundaryCell32" | "PhraseMemoryCell32" => "split",
        L2_SURFACE_COMPLETION_CELL => "completion",
        _ => "typo",
    }
}

fn l2_phase_apply_source(source: &str) -> bool {
    matches!(
        source,
        L2_SURFACE_MOTIF_CELL
            | L2_SURFACE_COMPLETION_CELL
            | "CommonRuFixCell32"
            | "PhraseCell32"
            | "GrammarCell32"
    )
}

#[cfg(not(test))]
fn phrase_forecast_memory() -> super::llmwave::LlmWaveMemory {
    super::llmwave::load_default_memory()
}

#[cfg(test)]
fn phrase_forecast_memory() -> super::llmwave::LlmWaveMemory {
    super::llmwave::load_default_memory_uncached()
}

fn push_unique_candidate(candidates: &mut Vec<WordCandidate>, candidate: WordCandidate) {
    if candidates
        .iter()
        .any(|item| item.text == candidate.text && item.source == candidate.source)
    {
        return;
    }
    candidates.push(candidate);
}

fn taught_candidates(
    original: &str,
    context: &TailContext,
    l1: &[WavePacket],
    options: &WaveOptions,
) -> Vec<WordCandidate> {
    let configured = crate::config::default_typing_assist_pipeline();
    let pipeline = typing_context::typing_assist_pipeline_for_context(
        true,
        CorrectionSafety::Experimental,
        &configured,
        original,
    );
    let explanation = explain_typing_assist_with_pipeline(original, true, &pipeline);
    if explanation.output.is_none() {
        return Vec::new();
    }
    let mut candidates = Vec::new();
    if let Some(chosen) = explanation.chosen.as_ref() {
        if let Some(candidate) = taught_word_candidate(TaughtCandidateInput {
            original,
            context,
            l1,
            options,
            replacement: &chosen.replacement,
            family: chosen.score.family,
            score: chosen.score.total,
            chosen: true,
        }) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    for evaluation in &explanation.evaluations {
        if evaluation.rejected.is_some() {
            continue;
        }
        let Some(candidate) = evaluation.candidate.as_ref() else {
            continue;
        };
        if let Some(candidate) = taught_word_candidate(TaughtCandidateInput {
            original,
            context,
            l1,
            options,
            replacement: &candidate.replacement,
            family: candidate.score.family,
            score: candidate.score.total,
            chosen: false,
        }) {
            push_unique_candidate(&mut candidates, candidate);
        }
        if candidates.len() >= MAX_TAUGHT_CANDIDATES {
            break;
        }
    }
    candidates
}

fn memory_cells_enabled(options: &WaveOptions) -> bool {
    options.is_enabled("LearnedMemoryCell32")
        || options.is_enabled("CommonRuFixCell32")
        || options.is_enabled("PhraseMemoryCell32")
        || options.is_enabled("UserMemoryCell32")
}

fn input_source_enabled(source: &str, options: &WaveOptions) -> bool {
    match source {
        "LearnedMemoryCell32" => options.is_enabled("LearnedMemoryCell32"),
        "CommonRuFixCell32" => options.is_enabled("CommonRuFixCell32"),
        "PhraseMemoryCell32" => options.is_enabled("PhraseMemoryCell32"),
        "UserMemoryCell32" => options.is_enabled("UserMemoryCell32"),
        L2_SURFACE_MOTIF_CELL => options.is_enabled(L2_SURFACE_MOTIF_CELL),
        L2_SURFACE_COMPLETION_CELL => options.is_enabled(L2_SURFACE_COMPLETION_CELL),
        source if source == PATTERN_MEMORY_CELL => false,
        _ => true,
    }
}

fn should_run_taught_candidates(token: &str, options: &WaveOptions) -> bool {
    if options.is_enabled(L2_SURFACE_MOTIF_CELL)
        && token.chars().count() >= 4
        && token.chars().all(is_cyrillic_letter)
    {
        return false;
    }
    true
}

fn taught_word_candidate(input: TaughtCandidateInput<'_>) -> Option<WordCandidate> {
    let replacement = input.replacement.trim_end();
    if replacement == input.original.trim_end() {
        return None;
    }
    if matches!(input.family, TypingCandidateFamily::Layout)
        && known_short_russian_token_blocks_layout(input.original.trim_end())
        && !short_cyrillic_layout_technical_allowed(replacement)
    {
        return None;
    }
    let source = match input.family {
        TypingCandidateFamily::Layout if input.options.is_enabled("LayoutWordCell32") => {
            "LayoutWordCell32"
        }
        TypingCandidateFamily::Structural if input.options.is_enabled("BoundaryCell32") => {
            "BoundaryCell32"
        }
        TypingCandidateFamily::Typo | TypingCandidateFamily::Exact
            if is_phrase_grammar_candidate(input.context, input.original, replacement)
                && input.options.is_enabled("GrammarCell32") =>
        {
            "GrammarCell32"
        }
        TypingCandidateFamily::Typo
        | TypingCandidateFamily::Visual
        | TypingCandidateFamily::Exact
        | TypingCandidateFamily::Cleanup
            if input.options.is_enabled("PhraseCell32") =>
        {
            "PhraseCell32"
        }
        _ => return None,
    };
    if source == "PhraseCell32" && unsafe_single_token_phrase_typo(input.original, replacement) {
        return None;
    }
    Some(WordCandidate {
        text: replacement.to_string(),
        source,
        energy: taught_energy(input.score, source, input.l1, input.chosen),
        risk: taught_risk(
            input.family,
            source,
            input.original,
            replacement,
            input.chosen,
        ),
        support: candidate_support(input.l1, input.context),
    })
}

fn layout_candidate(
    prefix: &str,
    token: &str,
    context: &TailContext,
    l1: &[WavePacket],
) -> Option<WordCandidate> {
    if token.chars().count() < 2 {
        return None;
    }
    if is_common_en_technical_word(&token.to_ascii_lowercase()) {
        return None;
    }
    if technical_context_blocks_layout(prefix, token) {
        return None;
    }
    let converted = convert(token, detect_direction(token));
    if converted == token {
        return None;
    }
    if context.token_count() < 2
        && token.chars().count() > 3
        && !is_common_en_technical_word(&converted.to_ascii_lowercase())
    {
        return None;
    }
    if known_short_russian_token_blocks_layout(token)
        && !short_cyrillic_layout_technical_allowed(&converted)
    {
        return None;
    }
    if !layout_candidate_allowed(token, &converted) {
        return None;
    }
    if !language_allows_layout(token, &converted) {
        return None;
    }
    let energy = l1_energy(l1, "KeyboardCell32").max(0.35);
    let risk = layout_risk(token, &converted, context);
    if energy <= risk {
        return None;
    }
    Some(WordCandidate {
        text: format!("{prefix}{converted}"),
        source: "LayoutWordCell32",
        energy,
        risk,
        support: candidate_support(l1, context),
    })
}

fn layout_scan_candidates(
    tail: &str,
    context: &TailContext,
    l1: &[WavePacket],
) -> Vec<WordCandidate> {
    let tokens = tail.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 2 || tokens.len() > 15 {
        return Vec::new();
    }
    let mut candidates = Vec::new();
    for idx in (0..tokens.len()).rev() {
        let token = tokens[idx];
        if token.chars().count() < 2 {
            continue;
        }
        if is_common_en_technical_word(&token.to_ascii_lowercase()) {
            continue;
        }
        let prefix = if idx == 0 { "" } else { tokens[idx - 1] };
        if technical_context_blocks_layout(prefix, token) {
            continue;
        }
        let converted = convert(token, detect_direction(token));
        if converted == token
            || !layout_candidate_allowed(token, &converted)
            || !language_allows_layout(token, &converted)
        {
            continue;
        }
        if known_short_russian_token_blocks_layout(token)
            && !short_cyrillic_layout_technical_allowed(&converted)
        {
            continue;
        }
        let mut replaced = tokens
            .iter()
            .map(|item| (*item).to_string())
            .collect::<Vec<_>>();
        replaced[idx] = converted;
        let text = replaced.join(" ");
        let energy = l1_energy(l1, "KeyboardCell32").max(0.35);
        let risk = (layout_risk(token, &replaced[idx], context) + 0.08).min(0.90);
        if energy <= risk {
            continue;
        }
        candidates.push(WordCandidate {
            text,
            source: "LayoutWordCell32",
            energy,
            risk,
            support: candidate_support(l1, context),
        });
        if candidates.len() >= MAX_LAYOUT_SCAN_CANDIDATES {
            break;
        }
    }
    candidates
}

fn language_allows_layout(token: &str, converted: &str) -> bool {
    let token_ascii = token.chars().all(|ch| ch.is_ascii_alphabetic());
    let converted_cyrillic = converted.chars().all(is_cyrillic_letter);
    if token_ascii && converted_cyrillic {
        return is_common_ru_word(&converted.to_lowercase());
    }
    true
}

fn short_token_candidates(
    prefix: &str,
    token: &str,
    context: &TailContext,
    l1: &[WavePacket],
) -> Vec<WordCandidate> {
    let clean = token.trim_matches(|ch: char| ch.is_ascii_punctuation());
    if clean.chars().count() != 1 || !clean.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return Vec::new();
    }
    if context.token_count() < 2 || technical_context_blocks_layout(prefix, token) {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    let converted = convert(clean, detect_direction(clean));
    let converted_lower = converted.to_lowercase();
    if converted != clean
        && converted.chars().all(is_cyrillic_letter)
        && (is_ru_short_function_word(&converted_lower) || is_common_ru_word(&converted_lower))
    {
        candidates.push(short_token_candidate(ShortTokenCandidateInput {
            prefix,
            token,
            replacement: &converted,
            reason: "keyboard-short-token",
            energy_floor: 0.90,
            risk: short_token_risk(context, token, "keyboard"),
            l1,
            context,
        }));
    }

    if clean.eq_ignore_ascii_case("b") {
        for (replacement, reason) in [
            (visual_b_default_replacement(), "visual-b-default"),
            (visual_b_after_ascii_replacement(), "visual-b-after-ascii"),
        ] {
            if replacement != converted
                && !candidates.iter().any(|item| {
                    item.text
                        .split_whitespace()
                        .last()
                        .is_some_and(|last| last == replacement)
                })
            {
                candidates.push(short_token_candidate(ShortTokenCandidateInput {
                    prefix,
                    token,
                    replacement,
                    reason,
                    energy_floor: 0.76,
                    risk: short_token_risk(context, token, "visual"),
                    l1,
                    context,
                }));
            }
        }
    }
    candidates
}

struct ShortTokenCandidateInput<'a> {
    prefix: &'a str,
    token: &'a str,
    replacement: &'a str,
    reason: &'a str,
    energy_floor: f32,
    risk: f32,
    l1: &'a [WavePacket],
    context: &'a TailContext,
}

fn short_token_candidate(input: ShortTokenCandidateInput<'_>) -> WordCandidate {
    let replacement = if input.token.chars().next().is_some_and(char::is_uppercase) {
        input.replacement.to_uppercase()
    } else {
        input.replacement.to_string()
    };
    WordCandidate {
        text: format!("{}{}", input.prefix, replacement),
        source: "ShortTokenCell32",
        energy: l1_energy(input.l1, "KeyboardCell32").max(input.energy_floor),
        risk: input.risk,
        support: {
            let mut support = candidate_support(input.l1, input.context);
            support.push(input.reason.to_string());
            support
        },
    }
}

fn short_token_risk(context: &TailContext, token: &str, mode: &str) -> f32 {
    let technical_context = context.has_technical_context();
    let ascii_context = context.tokens.iter().any(|item| {
        matches!(item.kind, TokenKind::AsciiWord | TokenKind::TechnicalAscii)
            && !item.text.eq_ignore_ascii_case(token)
    });
    let cyrillic_context = context
        .tokens
        .iter()
        .any(|item| item.kind == TokenKind::CyrillicWord);
    let mut risk: f32 = match mode {
        "visual" => 0.30,
        _ => 0.18,
    };
    if technical_context {
        risk += 0.35;
    }
    if ascii_context && !cyrillic_context {
        risk += 0.28;
    }
    if cyrillic_context {
        risk -= 0.08;
    }
    risk.clamp(0.05, 0.85)
}

fn technical_context_blocks_layout(prefix: &str, token: &str) -> bool {
    if !token.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return false;
    }
    let Some(previous) = previous_token(prefix) else {
        return false;
    };
    is_common_en_guard_prefix(&previous.to_ascii_lowercase()) && token.chars().count() >= 3
}

fn known_short_russian_token_blocks_layout(token: &str) -> bool {
    let lower = token.to_lowercase();
    token.chars().count() <= 3
        && lower.chars().all(is_cyrillic_letter)
        && (is_common_ru_word(&lower)
            || surface_motif_known_surface(&lower)
            || token.chars().all(is_cyrillic_letter))
}

fn short_cyrillic_layout_technical_allowed(converted: &str) -> bool {
    matches!(
        converted.to_ascii_lowercase().as_str(),
        "api" | "css" | "eng" | "git" | "go" | "lay" | "log" | "md" | "ms" | "rus" | "ssh" | "vpn"
    )
}

fn surface_motif_word_candidates(
    prefix: &str,
    token: &str,
    context: &TailContext,
    l1: &[WavePacket],
    options: &WaveOptions,
) -> Vec<WordCandidate> {
    if context.has_technical_context() {
        return Vec::new();
    }
    let (leading, word, trailing) = split_word_punctuation(token);
    let normalized = word.to_lowercase();
    let len = normalized.chars().count();
    if !(2..=18).contains(&len) || !normalized.chars().all(is_cyrillic_letter) {
        return Vec::new();
    }

    let surface_candidates = surface_motif_memory().surface_candidates_for_text(&normalized, 24);
    let mut out = Vec::new();
    if options.is_enabled(L2_SURFACE_MOTIF_CELL) && surface_candidates.is_empty() {
        if let Some(candidate) = repeated_letter_surface_candidate(
            prefix,
            leading,
            word,
            trailing,
            &normalized,
            l1,
            context,
        ) {
            out.push(candidate);
        }
    }
    let empty_fuzzy_authority = Vec::new();
    for candidate in &surface_candidates {
        let candidate_len = candidate.word.chars().count();
        let distance = damerau_levenshtein(&normalized, &candidate.word);
        let is_completion = candidate.word.starts_with(&normalized) && candidate_len > len;

        if options.is_enabled(L2_SURFACE_MOTIF_CELL)
            && len >= 4
            && !is_common_ru_word(&normalized)
            && !surface_motif_stable_existing_word(&normalized)
            && surface_motif_typo_has_authority(
                &normalized,
                &candidate.word,
                candidate.score,
                &surface_candidates,
                &empty_fuzzy_authority,
            )
            && (!fuzzy_surface_candidate_blocked(word, &normalized, &candidate.word)
                || repeated_all_caps_surface_allowed(word, &normalized, &candidate.word))
            && surface_motif_typo_allowed(
                &normalized,
                &candidate.word,
                len,
                distance,
                candidate.score,
            )
        {
            out.push(surface_motif_candidate(SurfaceMotifCandidateInput {
                prefix,
                leading,
                word,
                trailing,
                replacement_lower: &candidate.word,
                source: L2_SURFACE_MOTIF_CELL,
                score: candidate.score,
                l1_overlap: candidate.l1_overlap,
                l2_overlap: candidate.l2_overlap,
                motif_overlap: candidate.motif_overlap,
                prefix_match: candidate.prefix_match,
                distance,
                risk: surface_motif_typo_risk(context, distance),
                l1,
                context,
            }));
            if out.len() >= 8 {
                break;
            }
            continue;
        }

        if is_completion
            && options.is_enabled(L2_SURFACE_COMPLETION_CELL)
            && len >= 2
            && !surface_motif_stable_existing_word(&normalized)
            && candidate_len.saturating_sub(len) <= 10
        {
            out.push(surface_motif_candidate(SurfaceMotifCandidateInput {
                prefix,
                leading,
                word,
                trailing,
                replacement_lower: &candidate.word,
                source: L2_SURFACE_COMPLETION_CELL,
                score: candidate.score,
                l1_overlap: candidate.l1_overlap,
                l2_overlap: candidate.l2_overlap,
                motif_overlap: candidate.motif_overlap,
                prefix_match: candidate.prefix_match,
                distance,
                risk: 0.06,
                l1,
                context,
            }));
        }
    }
    if options.is_enabled(L2_SURFACE_MOTIF_CELL)
        && out.is_empty()
        && len >= 4
        && !surface_motif_stable_existing_word(&normalized)
    {
        let mut fuzzy_authority = crate::ru_typo::fuzzy_known_word_candidates(&normalized);
        fuzzy_authority.sort_by(|left, right| {
            let left_distance = damerau_levenshtein(&normalized, left);
            let right_distance = damerau_levenshtein(&normalized, right);
            left_distance
                .cmp(&right_distance)
                .then_with(|| left.chars().count().cmp(&right.chars().count()))
                .then_with(|| left.cmp(right))
        });
        for replacement_lower in fuzzy_authority.iter().take(4) {
            let distance = damerau_levenshtein(&normalized, replacement_lower);
            let score = 940u32.saturating_sub(distance.min(4) as u32 * 40);
            if surface_motif_typo_has_authority(
                &normalized,
                replacement_lower,
                score,
                &surface_candidates,
                &fuzzy_authority,
            ) && !fuzzy_surface_candidate_blocked(word, &normalized, replacement_lower)
                && surface_motif_typo_allowed(&normalized, replacement_lower, len, distance, score)
            {
                out.push(surface_motif_candidate(SurfaceMotifCandidateInput {
                    prefix,
                    leading,
                    word,
                    trailing,
                    replacement_lower,
                    source: L2_SURFACE_MOTIF_CELL,
                    score,
                    l1_overlap: 0,
                    l2_overlap: 0,
                    motif_overlap: 0,
                    prefix_match: false,
                    distance,
                    risk: surface_motif_typo_risk(context, distance),
                    l1,
                    context,
                }));
                break;
            }
        }
    }
    out
}

fn repeated_letter_surface_candidate(
    prefix: &str,
    leading: &str,
    word: &str,
    trailing: &str,
    normalized: &str,
    l1: &[WavePacket],
    context: &TailContext,
) -> Option<WordCandidate> {
    if normalized.chars().count() < 3 || is_common_ru_word(normalized) {
        return None;
    }
    if !has_adjacent_repeated_char(normalized) {
        return None;
    }
    let replacement = crate::ru_typo::correct_repeated_letter(word)?;
    let replacement_lower = replacement.to_lowercase();
    if replacement_lower == normalized || !is_known_russian_word_or_form(&replacement_lower) {
        return None;
    }
    let distance = damerau_levenshtein(normalized, &replacement_lower);
    if distance == 0 || distance > 3 {
        return None;
    }
    Some(surface_motif_candidate(SurfaceMotifCandidateInput {
        prefix,
        leading,
        word,
        trailing,
        replacement_lower: &replacement_lower,
        source: L2_SURFACE_MOTIF_CELL,
        score: 940,
        l1_overlap: 0,
        l2_overlap: 0,
        motif_overlap: 0,
        prefix_match: false,
        distance,
        risk: 0.08,
        l1,
        context,
    }))
}

fn has_adjacent_repeated_char(word: &str) -> bool {
    let mut prev = None;
    for ch in word.chars() {
        if prev == Some(ch) {
            return true;
        }
        prev = Some(ch);
    }
    false
}

fn surface_motif_typo_has_authority(
    original: &str,
    candidate: &str,
    score: u32,
    surface_candidates: &[super::l2_center_memory::L2SurfaceCandidate],
    fuzzy_authority: &[String],
) -> bool {
    let candidate_distance = damerau_levenshtein(original, candidate);
    let l2_surface_match = surface_candidates
        .iter()
        .any(|surface| surface.word == candidate && surface.score == score);
    if l2_surface_match
        && surface_motif_typo_allowed(
            original,
            candidate,
            original.chars().count(),
            candidate_distance,
            score,
        )
    {
        return true;
    }
    if surface_candidates.is_empty()
        && fuzzy_authority.len() == 1
        && fuzzy_authority
            .first()
            .is_some_and(|word| word == candidate)
        && candidate_distance == 1
    {
        return true;
    }
    if score < 880 || !fuzzy_authority.iter().any(|word| word == candidate) {
        return false;
    }
    let original_len = original.chars().count();
    !surface_candidates.iter().any(|other| {
        if other.word == candidate {
            return false;
        }
        let other_distance = damerau_levenshtein(original, &other.word);
        surface_motif_typo_allowed(
            original,
            &other.word,
            original_len,
            other_distance,
            other.score,
        ) && other_distance <= candidate_distance
            && other.score.saturating_add(24) >= score
    })
}

fn repeated_all_caps_surface_allowed(
    original_word: &str,
    original_lower: &str,
    candidate: &str,
) -> bool {
    original_word
        .chars()
        .all(|ch| !ch.is_alphabetic() || ch.is_uppercase())
        && has_adjacent_repeated_char(original_lower)
        && damerau_levenshtein(original_lower, candidate) <= 3
}

fn fuzzy_surface_candidate_blocked(
    original_word: &str,
    original_lower: &str,
    candidate: &str,
) -> bool {
    if is_user_protected_word(original_lower) || is_ru_live_protected_word(original_lower) {
        return true;
    }
    if original_word
        .chars()
        .all(|ch| !ch.is_alphabetic() || ch.is_uppercase())
    {
        return true;
    }
    if crate::ru_typo::rewrites_protected_pattern_term_stem(original_lower, candidate) {
        return true;
    }
    if same_stem_inflection_rewrite(original_lower, candidate) {
        return true;
    }
    looks_like_live_russian_it_verb(original_lower) && !candidate.ends_with("ит")
}

fn surface_motif_stable_existing_word(word: &str) -> bool {
    is_common_ru_word(word)
        || is_user_protected_word(word)
        || is_ru_live_protected_word(word)
        || (surface_motif_strict_known_surface(word)
            && !russian_zero_a_ya_stem_has_known_lemma(word))
        || russian_zero_o_form_has_known_lemma(word)
        || russian_future_ut_form_has_known_infinitive(word)
}

fn russian_zero_a_ya_stem_has_known_lemma(word: &str) -> bool {
    word.chars().count() >= 5
        && word.chars().last().is_some_and(is_russian_consonant_for_l2)
        && (surface_motif_known_surface(&format!("{word}а"))
            || surface_motif_known_surface(&format!("{word}я")))
}

fn russian_zero_o_form_has_known_lemma(word: &str) -> bool {
    word.chars().count() >= 4
        && word.chars().last().is_some_and(is_russian_consonant_for_l2)
        && surface_motif_known_surface(&format!("{word}о"))
}

fn russian_future_ut_form_has_known_infinitive(word: &str) -> bool {
    let Some(stem) = word.strip_suffix("ут") else {
        return false;
    };
    stem.chars().count() >= 3 && surface_motif_known_surface(&format!("{stem}уть"))
}

fn surface_motif_known_surface(word: &str) -> bool {
    surface_motif_strict_known_surface(word) || runtime_l2_surface_word_set().contains(word)
}

fn surface_motif_strict_known_surface(word: &str) -> bool {
    is_common_ru_word(word)
        || is_ru_live_protected_word(word)
        || is_user_protected_word(word)
        || crate::russian_lexicon::russian_dictionary().contains(word)
}

fn is_russian_consonant_for_l2(ch: char) -> bool {
    is_cyrillic_letter(ch)
        && !matches!(
            ch,
            'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ы' | 'э' | 'ю' | 'я' | 'ь' | 'ъ'
        )
}

fn same_stem_inflection_rewrite(original: &str, candidate: &str) -> bool {
    same_stem_suffix_rewrite(
        original,
        candidate,
        &[
            "ыми", "ими", "ого", "его", "ому", "ему", "ом", "ем", "ой", "ей", "ая", "яя", "ое",
            "ее", "ые", "ие", "ых", "их", "ый", "ий",
        ],
    ) || same_stem_suffix_rewrite(
        original,
        candidate,
        &[
            "ешься",
            "ишься",
            "ется",
            "ются",
            "ался",
            "алась",
            "ались",
            "алось",
            "ился",
            "илась",
            "ились",
            "илось",
            "аете",
            "аешь",
            "айте",
            "ают",
            "ешь",
            "ишь",
            "ает",
            "ать",
            "ить",
            "еть",
            "уть",
            "нут",
            "ют",
            "ут",
            "ет",
            "ит",
            "ай",
            "ил",
            "ла",
            "ли",
            "ло",
            "у",
        ],
    )
}

fn same_stem_suffix_rewrite(original: &str, candidate: &str, suffixes: &[&'static str]) -> bool {
    let Some((original_stem, original_suffix)) = split_known_suffix(original, suffixes) else {
        return false;
    };
    let Some((candidate_stem, candidate_suffix)) = split_known_suffix(candidate, suffixes) else {
        return false;
    };
    original_suffix != candidate_suffix
        && original_stem == candidate_stem
        && original_stem.chars().count() >= 3
}

fn split_known_suffix<'a>(
    word: &'a str,
    suffixes: &[&'static str],
) -> Option<(&'a str, &'static str)> {
    suffixes.iter().find_map(|suffix| {
        let stem = word.strip_suffix(suffix)?;
        (!stem.is_empty()).then_some((stem, *suffix))
    })
}

fn looks_like_live_russian_it_verb(word: &str) -> bool {
    word.chars().count() >= 5
        && word.ends_with("ит")
        && !word.ends_with("оит")
        && !word.ends_with("еит")
        && !word.ends_with("аит")
}

struct SurfaceMotifCandidateInput<'a> {
    prefix: &'a str,
    leading: &'a str,
    word: &'a str,
    trailing: &'a str,
    replacement_lower: &'a str,
    source: &'static str,
    score: u32,
    l1_overlap: usize,
    l2_overlap: usize,
    motif_overlap: usize,
    prefix_match: bool,
    distance: usize,
    risk: f32,
    l1: &'a [WavePacket],
    context: &'a TailContext,
}

fn surface_motif_candidate(input: SurfaceMotifCandidateInput<'_>) -> WordCandidate {
    let replacement_word = apply_word_case(input.word, input.replacement_lower);
    let energy =
        l1_energy(input.l1, "ScriptCell32").max((input.score as f32 / 900.0).clamp(0.42, 0.95));
    WordCandidate {
        text: format!(
            "{}{}{}{}",
            input.prefix, input.leading, replacement_word, input.trailing
        ),
        source: input.source,
        energy,
        risk: input.risk,
        support: {
            let mut support = candidate_support(input.l1, input.context);
            support.push(format!(
                "l2-surface:score={} l1_overlap={} l2_overlap={} motif_overlap={} prefix={} distance={}",
                input.score,
                input.l1_overlap,
                input.l2_overlap,
                input.motif_overlap,
                input.prefix_match,
                input.distance
            ));
            support
        },
    }
}

fn surface_motif_typo_allowed(
    input: &str,
    candidate: &str,
    input_len: usize,
    distance: usize,
    score: u32,
) -> bool {
    distance == 1
        || is_single_adjacent_transposition(input, candidate)
        || (input_len >= 6 && distance == 2 && score >= 300)
        || (input_len >= 8 && distance == 3 && score >= 380)
}

fn is_single_adjacent_transposition(input: &str, candidate: &str) -> bool {
    let mut left = input.chars().collect::<Vec<_>>();
    let right = candidate.chars().collect::<Vec<_>>();
    if left.len() != right.len() || left.len() < 2 || left == right {
        return false;
    }
    for index in 0..left.len() - 1 {
        left.swap(index, index + 1);
        if left == right {
            return true;
        }
        left.swap(index, index + 1);
    }
    false
}

fn surface_motif_typo_risk(context: &TailContext, distance: usize) -> f32 {
    let phrase_bonus = if context.token_count() >= 2 {
        -0.03
    } else {
        0.05
    };
    (0.10 + distance as f32 * 0.06 + phrase_bonus).clamp(0.06, 0.40)
}

fn surface_motif_memory() -> &'static L2CenterMemory {
    SURFACE_MOTIF_MEMORY.get_or_init(|| {
        let timing_enabled = std::env::var_os("LAY_NANDA_L2_TIMING").is_some();
        let started = std::time::Instant::now();
        let words = runtime_l2_surface_words();
        if timing_enabled {
            eprintln!(
                "lay_nanda_l2_timing stage=surface-bank elapsed_us={} words={}",
                started.elapsed().as_micros(),
                words.len()
            );
        }
        let build_started = std::time::Instant::now();
        let memory = L2CenterMemory::build(
            words.iter().map(String::as_str),
            L2CenterMemoryConfig {
                l1_config: super::l1_center_memory::L1CenterMemoryConfig {
                    min_center_support: 2,
                    max_centers: 48_000,
                },
                motif_len: 3,
                min_motif_support: 2,
                max_motifs: 64_000,
            },
        );
        if timing_enabled {
            eprintln!(
                "lay_nanda_l2_timing stage=surface-memory-build elapsed_us={} centers={} words={}",
                build_started.elapsed().as_micros(),
                memory.center_count(),
                words.len()
            );
        }
        drop(words);
        trim_allocator_after_l2_surface_build();
        memory
    })
}

#[cfg(target_os = "linux")]
fn trim_allocator_after_l2_surface_build() {
    unsafe {
        libc::malloc_trim(0);
    }
}

#[cfg(not(target_os = "linux"))]
fn trim_allocator_after_l2_surface_build() {}

fn broad_prefix_index() -> &'static super::l2_broad_index::L2BroadPrefixIndex {
    BROAD_PREFIX_INDEX.get_or_init(|| {
        super::l2_broad_index::L2BroadPrefixIndex::build(&[
            L2_SURFACE_FOUNDATION_RU_DATA,
            L2_SURFACE_HOT_RU_DATA,
        ])
    })
}

fn runtime_l2_surface_word_set() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| runtime_l2_surface_words().into_iter().collect())
}

fn runtime_l2_surface_words() -> Vec<String> {
    let mut words = Vec::new();
    let mut seen = HashSet::new();
    collect_runtime_l2_words(
        super::usage_prior::l2_surface_words_by_usage(L2_USAGE_WORD_LIMIT),
        &mut words,
        &mut seen,
    );
    collect_runtime_l2_case_words(
        include_str!("../../data/nanda_wave_synthetic_cases.tsv"),
        1,
        L2_CASE_WORD_LIMIT,
        &mut words,
        &mut seen,
    );
    collect_runtime_l2_generated_positive_words(
        include_str!("../../data/nanda_training/generated_cases.tsv"),
        L2_CASE_WORD_LIMIT,
        &mut words,
        &mut seen,
    );
    collect_runtime_l2_training_words(
        crate::lexicon::common_ru_words_iter().map(str::to_string),
        &mut words,
        &mut seen,
    );
    fill_balanced_runtime_l2_surface_words(
        data_words(L2_SURFACE_HOT_RU_DATA)
            .chain(data_words(L2_SURFACE_FOUNDATION_RU_DATA).take(L2_FOUNDATION_SOURCE_LIMIT)),
        L2_RUNTIME_WORD_LIMIT,
        &mut words,
        &mut seen,
    );

    words.truncate(L2_RUNTIME_WORD_LIMIT);
    words
}

fn fill_balanced_runtime_l2_surface_words<I>(
    source: I,
    limit: usize,
    words: &mut Vec<String>,
    seen: &mut HashSet<String>,
) where
    I: IntoIterator<Item = String>,
{
    let remaining = limit.saturating_sub(words.len());
    if remaining == 0 {
        return;
    }
    for word in super::surface_bank::balanced_l2_surface_words(source, remaining.saturating_mul(3))
    {
        if seen.insert(word.clone()) {
            words.push(word);
            if words.len() >= limit {
                break;
            }
        }
    }
}

fn collect_runtime_l2_training_words<I>(
    source: I,
    words: &mut Vec<String>,
    seen: &mut HashSet<String>,
) where
    I: IntoIterator<Item = String>,
{
    for word in source {
        if let Some(normalized) = super::surface_bank::normalize_l2_training_surface_word(&word) {
            if seen.insert(normalized.clone()) {
                words.push(normalized);
            }
        }
    }
}

fn collect_runtime_l2_words<I>(source: I, words: &mut Vec<String>, seen: &mut HashSet<String>)
where
    I: IntoIterator<Item = String>,
{
    for word in source {
        if let Some(normalized) = super::surface_bank::normalize_l2_surface_word(&word) {
            if seen.insert(normalized.clone()) {
                words.push(normalized);
            }
        }
    }
}

fn data_words(data: &str) -> impl Iterator<Item = String> + '_ {
    data.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
}

fn collect_runtime_l2_case_words(
    text: &str,
    expected_col: usize,
    max_new_words: usize,
    words: &mut Vec<String>,
    seen: &mut HashSet<String>,
) {
    let start_len = words.len();
    for line in text
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
    {
        if words.len().saturating_sub(start_len) >= max_new_words {
            break;
        }
        let cols = line.split('\t').collect::<Vec<_>>();
        if let Some(expected) = cols.get(expected_col) {
            collect_runtime_l2_text_words(&decode_fixture_spaces(expected), words, seen);
        }
    }
}

fn collect_runtime_l2_generated_positive_words(
    text: &str,
    max_new_words: usize,
    words: &mut Vec<String>,
    seen: &mut HashSet<String>,
) {
    let start_len = words.len();
    for line in text.lines().skip(1) {
        if words.len().saturating_sub(start_len) >= max_new_words {
            break;
        }
        let cols = line.split('\t').collect::<Vec<_>>();
        if cols.len() >= 6 && cols[5] == "1" {
            collect_runtime_l2_text_words(&decode_fixture_spaces(cols[3]), words, seen);
        }
    }
}

fn collect_runtime_l2_text_words(text: &str, words: &mut Vec<String>, seen: &mut HashSet<String>) {
    collect_runtime_l2_words(
        text.split_whitespace().map(|token| {
            token
                .chars()
                .filter(|ch| ch.is_alphabetic() || *ch == '-')
                .flat_map(char::to_lowercase)
                .collect::<String>()
        }),
        words,
        seen,
    );
}

fn decode_fixture_spaces(text: &str) -> String {
    text.replace("\\s", " ")
}

fn customs_actor_phrase_candidates(
    tail: &str,
    context: &TailContext,
    l1: &[WavePacket],
) -> Vec<WordCandidate> {
    if context.has_technical_context() || context.tokens.len() < 4 {
        return Vec::new();
    }
    let Some(previous) = context.previous() else {
        return Vec::new();
    };
    let Some(last) = context.last() else {
        return Vec::new();
    };
    if clean_ru_token(&previous.text) != "таможен" || clean_ru_token(&last.text) != "мы" {
        return Vec::new();
    }
    if !has_customs_actor_context(context) {
        return Vec::new();
    }
    let mut tokens = tail
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    if tokens.len() < 2 {
        return Vec::new();
    }
    let previous_idx = tokens.len() - 2;
    let Some(replacement) = replace_cyrillic_core(&tokens[previous_idx], "таможим") else {
        return Vec::new();
    };
    tokens[previous_idx] = replacement;
    vec![WordCandidate {
        text: tokens.join(" "),
        source: "PhraseCell32",
        energy: l1_energy(l1, "ScriptCell32")
            .max(l1_energy(l1, "BoundaryCell32"))
            .max(0.88),
        risk: 0.13,
        support: {
            let mut support = candidate_support(l1, context);
            support.push("customs-actor-phrase".to_string());
            support.push("previous=таможен last=мы replacement=таможим".to_string());
            support
        },
    }]
}

fn has_customs_actor_context(context: &TailContext) -> bool {
    context.tokens.iter().any(|token| {
        let token = clean_ru_token(&token.text);
        token.contains("поставщик")
            || token.contains("цен")
            || token.contains("склад")
            || token.contains("покупател")
            || token.contains("накладн")
            || token.contains("меркур")
            || token.contains("логист")
            || token.contains("достав")
    })
}

fn replace_cyrillic_core(token: &str, replacement: &str) -> Option<String> {
    let start = token.find(is_cyrillic_letter)?;
    let end = token
        .char_indices()
        .rev()
        .find(|(_idx, ch)| is_cyrillic_letter(*ch))
        .map(|(idx, ch)| idx + ch.len_utf8())?;
    if start >= end {
        return None;
    }
    let replacement = if token[start..end]
        .chars()
        .next()
        .is_some_and(char::is_uppercase)
    {
        capitalize_first(replacement)
    } else {
        replacement.to_string()
    };
    Some(format!(
        "{}{}{}",
        &token[..start],
        replacement,
        &token[end..]
    ))
}

fn capitalize_first(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_uppercase().chain(chars).collect()
}

fn grammar_agreement_candidates(
    tail: &str,
    context: &TailContext,
    l1: &[WavePacket],
) -> Vec<WordCandidate> {
    if context.has_technical_context() {
        return Vec::new();
    }
    let Some(previous) = context.previous() else {
        return Vec::new();
    };
    let Some(last) = context.last() else {
        return Vec::new();
    };
    if previous.kind != TokenKind::CyrillicWord || last.kind != TokenKind::CyrillicWord {
        return Vec::new();
    }
    let previous = clean_ru_token(&previous.text);
    let last_clean = clean_ru_token(&last.text);
    let Some(replacement) = preposition_case_completion(&previous, &last_clean)
        .or_else(|| agree_adjective_like_tail(&previous, &last_clean))
    else {
        return Vec::new();
    };
    if replacement == last_clean {
        return Vec::new();
    }
    let Some((prefix, _token)) = split_last_ws_token(tail.trim_end()) else {
        return Vec::new();
    };
    vec![WordCandidate {
        text: format!("{prefix}{replacement}"),
        source: "GrammarCell32",
        energy: l1_energy(l1, "ScriptCell32")
            .max(l1_energy(l1, "BoundaryCell32"))
            .max(0.84),
        risk: 0.13,
        support: vec![
            "grammar-agreement".to_string(),
            format!("previous={previous:?} last={last_clean:?} replacement={replacement:?}"),
        ],
    }]
}

fn preposition_case_completion(previous: &str, word: &str) -> Option<String> {
    if !crate::lexicon::is_ru_short_preposition(previous)
        && !is_ru_one_letter_function_word(previous)
    {
        return None;
    }
    if word.chars().count() < 5 || is_common_ru_word(word) || is_known_russian_word_or_form(word) {
        return None;
    }
    let replacement = format!("{word}и");
    if is_known_russian_word_or_form(&replacement)
        || is_common_ru_word(&replacement)
        || word.ends_with("ани")
        || word.ends_with("ени")
    {
        return Some(replacement);
    }
    None
}

fn agree_adjective_like_tail(previous: &str, word: &str) -> Option<String> {
    if previous.chars().count() < 4 || word.chars().count() < 6 {
        return None;
    }
    if is_common_ru_word(word) {
        return None;
    }
    if has_russian_verb_tail(previous) {
        return None;
    }
    let stem = word
        .strip_suffix("ительные")
        .map(|stem| (stem, "ительный"))
        .or_else(|| word.strip_suffix("альные").map(|stem| (stem, "альный")))
        .or_else(|| word.strip_suffix("ные").map(|stem| (stem, "ный")))
        .or_else(|| word.strip_suffix("ые").map(|stem| (stem, "ый")))?;
    if !looks_like_singular_anchor(previous) {
        return None;
    }
    if looks_like_plural_anchor(previous) {
        return None;
    }
    Some(format!("{}{}", stem.0, stem.1))
}

fn has_russian_verb_tail(word: &str) -> bool {
    const VERB_TAILS: &[&str] = &[
        "ет", "ит", "ют", "ут", "ат", "ят", "ем", "им", "ешь", "ишь", "ете", "ите", "ал", "ала",
        "ило", "или", "ил", "ено", "ена", "ены", "ает", "яет", "ует",
    ];
    VERB_TAILS.iter().any(|tail| word.ends_with(tail))
}

fn looks_like_plural_anchor(word: &str) -> bool {
    word.ends_with("ые")
        || word.ends_with("ие")
        || word.ends_with("ых")
        || word.ends_with("их")
        || word.ends_with("ыми")
        || word.ends_with("ими")
}

fn looks_like_singular_anchor(word: &str) -> bool {
    let Some(last) = word.chars().last() else {
        return false;
    };
    matches!(
        last,
        'б' | 'в'
            | 'г'
            | 'д'
            | 'ж'
            | 'з'
            | 'й'
            | 'к'
            | 'л'
            | 'м'
            | 'н'
            | 'п'
            | 'р'
            | 'с'
            | 'т'
            | 'ф'
            | 'х'
            | 'ц'
            | 'ч'
            | 'ш'
            | 'щ'
            | 'о'
            | 'е'
    )
}

fn clean_ru_token(token: &str) -> String {
    token
        .trim_matches(|ch: char| ch.is_ascii_punctuation() || matches!(ch, '«' | '»' | '“' | '”'))
        .to_lowercase()
}

fn layout_candidate_allowed(token: &str, converted: &str) -> bool {
    let token_ascii = token.chars().all(|ch| ch.is_ascii_alphabetic());
    let token_cyrillic = token.chars().all(is_cyrillic_letter);
    let converted_ascii = converted.chars().all(|ch| ch.is_ascii_alphabetic());
    let converted_cyrillic = converted.chars().all(is_cyrillic_letter);

    if token_ascii && converted_cyrillic {
        return true;
    }
    if token_cyrillic && converted_ascii {
        let token_lower = token.to_lowercase();
        if is_user_protected_word(&token_lower) || surface_motif_known_surface(&token_lower) {
            return false;
        }
        return is_common_en_technical_word(&converted.to_ascii_lowercase());
    }
    false
}

fn technical_keep_candidate(token: &str, l1: &[WavePacket]) -> Option<WordCandidate> {
    if !is_common_en_technical_word(&token.to_ascii_lowercase()) {
        return None;
    }
    Some(WordCandidate {
        text: token.to_string(),
        source: "TechTokenCell32",
        energy: l1_energy(l1, "ScriptCell32").max(0.8),
        risk: 0.05,
        support: top_support(l1),
    })
}

fn boundary_split_candidates(
    prefix: &str,
    token: &str,
    l1: &[WavePacket],
    context: &TailContext,
) -> Vec<WordCandidate> {
    if context.has_technical_context() || !token.chars().all(is_cyrillic_letter) {
        return Vec::new();
    }
    let normalized = token.to_lowercase();
    if normalized.chars().count() < 6
        || is_common_ru_word(&normalized)
        || is_ru_live_protected_word(&normalized)
    {
        return Vec::new();
    }
    if surface_motif_known_surface(&normalized) {
        return Vec::new();
    }
    if let Some(replacement) = light_boundary_replacement(&normalized) {
        return vec![WordCandidate {
            text: format!("{prefix}{}", apply_word_case(token, &replacement)),
            source: "BoundaryCell32",
            energy: l1_energy(l1, "BoundaryCell32").max(0.99),
            risk: 0.04,
            support: {
                let mut support = candidate_support(l1, context);
                support.push("light-boundary-split".to_string());
                support.push(format!("word={normalized:?} replacement={replacement:?}"));
                support
            },
        }];
    }
    if let Some(replacement) = crate::phrase_reader::correct_glued_russian_phrase(&normalized) {
        if replacement != normalized {
            return vec![WordCandidate {
                text: format!("{prefix}{}", apply_word_case(token, &replacement)),
                source: "BoundaryCell32",
                energy: l1_energy(l1, "BoundaryCell32").max(0.86),
                risk: 0.08,
                support: {
                    let mut support = candidate_support(l1, context);
                    support.push("direct-glued-phrase-boundary".to_string());
                    support.push(format!("word={normalized:?} replacement={replacement:?}"));
                    support
                },
            }];
        }
    }
    let chars = normalized.chars().collect::<Vec<_>>();
    let mut fuzzy_typo_candidates: Option<Vec<String>> = None;
    let mut candidates = Vec::new();
    for split in 1..chars.len() {
        let left = chars[..split].iter().collect::<String>();
        let right = chars[split..].iter().collect::<String>();
        if left.chars().count() > 2 && right.chars().count() < 3 {
            continue;
        }
        let short_function_boundary =
            left.chars().count() == 1 && is_ru_one_letter_function_word(&left);
        if short_function_boundary && fuzzy_typo_candidates.is_none() {
            fuzzy_typo_candidates = Some(crate::ru_typo::fuzzy_known_word_candidates(&normalized));
        }
        if short_function_boundary
            && fuzzy_typo_candidates
                .as_ref()
                .is_some_and(|candidates| !candidates.is_empty())
            && !strong_boundary_right_anchor(&right)
        {
            continue;
        }
        let known_left = short_function_boundary;
        let known_right = surface_motif_known_surface(&right);
        if !known_left || !known_right {
            continue;
        }
        let (energy, risk, reason) = if short_function_boundary {
            (
                l1_energy(l1, "BoundaryCell32").max(0.99),
                0.04,
                "hidden-short-function-boundary",
            )
        } else {
            (
                l1_energy(l1, "BoundaryCell32").max(0.78),
                if left.chars().count() <= 2 {
                    0.18
                } else {
                    0.12
                },
                "dictionary-split",
            )
        };
        candidates.push(WordCandidate {
            text: format!("{prefix}{left} {right}"),
            source: "BoundaryCell32",
            energy,
            risk,
            support: vec![reason.to_string(), format!("left={left:?} right={right:?}")],
        });
        if candidates.len() >= 3 {
            break;
        }
    }
    candidates
}

fn light_boundary_replacement(word: &str) -> Option<String> {
    let chars = word.chars().collect::<Vec<_>>();
    let mut best = None::<(usize, String)>;
    for split in 1..chars.len() {
        let left = chars[..split].iter().collect::<String>();
        let right = chars[split..].iter().collect::<String>();
        if left.chars().count() > 3 || right.chars().count() < 3 {
            continue;
        }
        if left.chars().count() > 1 && crate::lexicon::is_ru_short_preposition(&left) {
            continue;
        }
        let known_left_function = is_ru_one_letter_function_word(&left);
        let known_left_pronoun = crate::lexicon::is_ru_short_pronoun(&left);
        let known_left_common = is_common_ru_word(&left);
        let known_left = known_left_function || known_left_pronoun || known_left_common;
        let known_right = surface_motif_known_surface(&right);
        if known_left && known_right {
            let score = boundary_split_score(
                left.chars().count(),
                right.chars().count(),
                known_left_function,
                known_left_pronoun || known_left_common,
                is_common_ru_word(&right),
            );
            let replacement = format!("{left} {right}");
            let replace_best = match best.as_ref() {
                Some((best_score, _)) => score > *best_score,
                None => true,
            };
            if replace_best {
                best = Some((score, replacement));
            }
        }
    }
    best.map(|(_, replacement)| replacement)
}

fn boundary_split_score(
    left_len: usize,
    right_len: usize,
    left_function: bool,
    left_common: bool,
    right_common: bool,
) -> usize {
    let mut score = right_len.min(12);
    if left_common {
        score += 20;
    }
    if right_common {
        score += 10;
    }
    if left_function {
        score += 4;
    }
    score + left_len.min(4) * 3
}

fn boundary_scan_candidates(
    tail: &str,
    l1: &[WavePacket],
    context: &TailContext,
) -> Vec<WordCandidate> {
    if context.has_technical_context() {
        return Vec::new();
    }
    let segments = split_ws_segments(tail);
    if segments.len() < 3 || context.token_count() > 15 {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for (idx, (segment, is_ws)) in segments.iter().enumerate().rev() {
        if *is_ws {
            continue;
        }
        let (leading, word, trailing) = split_word_punctuation(segment);
        if word.is_empty() || !word.chars().all(is_cyrillic_letter) {
            continue;
        }
        let previous = previous_word_segment(&segments, idx);
        let Some(replacement) = contextual_boundary_replacement_for_word(word, previous)
            .or_else(|| boundary_replacement_for_word(word))
        else {
            continue;
        };
        if replacement == word {
            continue;
        }
        let text = replace_segment_word(&segments, idx, leading, &replacement, trailing);
        candidates.push(WordCandidate {
            text,
            source: "BoundaryCell32",
            energy: l1_energy(l1, "BoundaryCell32").max(0.82),
            risk: 0.10,
            support: {
                let mut support = candidate_support(l1, context);
                support.push("tail-boundary-scan".to_string());
                support.push(format!("word={word:?} replacement={replacement:?}"));
                support
            },
        });
        if candidates.len() >= 4 {
            return candidates;
        }
    }

    for window in word_segment_windows(&segments).into_iter().rev() {
        let pair_text = format!(
            "{}{}{}",
            segments[window.left_idx].0, segments[window.ws_idx].0, segments[window.right_idx].0
        );
        let Some(replacement) = crate::phrase_reader::correct_split_word_pair(&pair_text) else {
            continue;
        };
        if replacement == pair_text {
            continue;
        }
        candidates.push(WordCandidate {
            text: replace_segment_window(
                &segments,
                window.left_idx,
                window.right_idx,
                &replacement,
            ),
            source: "BoundaryCell32",
            energy: l1_energy(l1, "BoundaryCell32").max(0.80),
            risk: 0.12,
            support: {
                let mut support = candidate_support(l1, context);
                support.push("tail-split-pair-scan".to_string());
                support.push(format!("pair={pair_text:?} replacement={replacement:?}"));
                support
            },
        });
        if candidates.len() >= 4 {
            break;
        }
    }

    candidates
}

fn boundary_replacement_for_word(word: &str) -> Option<String> {
    crate::phrase_reader::correct_glued_russian_phrase(word).or_else(|| {
        let lower = word.to_lowercase();
        if lower.chars().count() < 6
            || is_common_ru_word(&lower)
            || is_known_russian_word_or_form(&lower)
        {
            return None;
        }
        let chars = lower.chars().collect::<Vec<_>>();
        for split in 1..chars.len() {
            let left = chars[..split].iter().collect::<String>();
            let right = chars[split..].iter().collect::<String>();
            if left.chars().count() > 2 && right.chars().count() < 3 {
                continue;
            }
            let known_left = left.chars().count() == 1 && is_ru_one_letter_function_word(&left);
            let known_right = is_common_ru_word(&right) || is_known_russian_word_or_form(&right);
            if known_left && known_right {
                let replacement = format!("{left} {right}");
                return Some(apply_word_case(word, &replacement));
            }
        }
        None
    })
}

fn contextual_boundary_replacement_for_word(word: &str, previous: Option<&str>) -> Option<String> {
    let previous = previous?.to_lowercase();
    if !crate::phrase_lexicon::is_short_russian_function_word(&previous) {
        return None;
    }

    let lower = word.to_lowercase();
    let chars = lower.chars().collect::<Vec<_>>();
    for split in 1..chars.len() {
        let left = chars[..split].iter().collect::<String>();
        let right = chars[split..].iter().collect::<String>();
        if !crate::lexicon::is_ru_short_pronoun(&left) {
            continue;
        }
        if !(right == "есть" || is_common_ru_word(&right) || is_known_russian_word_or_form(&right))
        {
            continue;
        }
        let replacement = format!("{left} {right}");
        return Some(apply_word_case(word, &replacement));
    }
    None
}

fn surface_motif_scan_candidates(
    tail: &str,
    l1: &[WavePacket],
    context: &TailContext,
) -> Vec<WordCandidate> {
    if context.has_technical_context() || context.token_count() > 15 {
        return Vec::new();
    }
    let segments = split_ws_segments(tail);
    if segments.len() < 3 {
        return Vec::new();
    }
    let mut candidates = Vec::new();
    for (idx, (segment, is_ws)) in segments.iter().enumerate().rev() {
        if *is_ws {
            continue;
        }
        let (leading, word, trailing) = split_word_punctuation(segment);
        if word.is_empty() || !word.chars().all(is_cyrillic_letter) {
            continue;
        }
        let Some(replacement) = surface_replacement_for_word(word) else {
            continue;
        };
        if replacement == word {
            continue;
        }
        let lower = word.to_lowercase();
        let replacement_lower = replacement.to_lowercase();
        let distance = damerau_levenshtein(&lower, &replacement_lower);
        if distance == 0 || distance > 3 {
            continue;
        }
        candidates.push(WordCandidate {
            text: replace_segment_word(&segments, idx, leading, &replacement, trailing),
            source: L2_SURFACE_MOTIF_CELL,
            energy: l1_energy(l1, "ScriptCell32").max(0.78),
            risk: surface_motif_typo_risk(context, distance),
            support: {
                let mut support = candidate_support(l1, context);
                support.push("tail-surface-scan".to_string());
                support.push(format!(
                    "word={word:?} replacement={replacement:?} distance={distance}"
                ));
                support
            },
        });
        if candidates.len() >= 4 {
            break;
        }
    }
    candidates
}

fn surface_replacement_for_word(word: &str) -> Option<String> {
    crate::ru_typo::correct_repeated_letter(word)
        .or_else(|| crate::ru_typo::correct_adjacent_transposition(word))
        .or_else(|| crate::ru_typo::correct_missing_letter(word))
}

struct SegmentWindow {
    left_idx: usize,
    ws_idx: usize,
    right_idx: usize,
}

fn word_segment_windows(segments: &[(&str, bool)]) -> Vec<SegmentWindow> {
    segments
        .windows(3)
        .enumerate()
        .filter_map(|(idx, window)| {
            let [left, ws, right] = window else {
                return None;
            };
            (!left.1 && ws.1 && !right.1).then_some(SegmentWindow {
                left_idx: idx,
                ws_idx: idx + 1,
                right_idx: idx + 2,
            })
        })
        .collect()
}

fn replace_segment_word(
    segments: &[(&str, bool)],
    target_idx: usize,
    leading: &str,
    replacement: &str,
    trailing: &str,
) -> String {
    let mut out = String::new();
    for (idx, (segment, _)) in segments.iter().enumerate() {
        if idx == target_idx {
            out.push_str(leading);
            out.push_str(replacement);
            out.push_str(trailing);
        } else {
            out.push_str(segment);
        }
    }
    out
}

fn replace_segment_window(
    segments: &[(&str, bool)],
    left_idx: usize,
    right_idx: usize,
    replacement: &str,
) -> String {
    let mut out = String::new();
    let mut idx = 0;
    while idx < segments.len() {
        if idx == left_idx {
            out.push_str(replacement);
            idx = right_idx + 1;
        } else {
            out.push_str(segments[idx].0);
            idx += 1;
        }
    }
    out
}

fn previous_word_segment<'a>(
    segments: &'a [(&'a str, bool)],
    before_idx: usize,
) -> Option<&'a str> {
    segments[..before_idx]
        .iter()
        .rev()
        .find_map(|(segment, is_ws)| {
            if *is_ws {
                return None;
            }
            let (_, word, _) = split_word_punctuation(segment);
            (!word.is_empty()).then_some(word)
        })
}

fn strong_boundary_right_anchor(lower: &str) -> bool {
    lower.chars().count() >= 5
        && (lower.ends_with("ах") || lower.ends_with("ях"))
        && (is_common_ru_word(lower) || is_known_russian_word_or_form(lower))
}

fn technical_context_keep_candidate(text: &str, l1: &[WavePacket]) -> Option<WordCandidate> {
    if !looks_like_shell_or_technical_phrase(text) {
        return None;
    }
    Some(WordCandidate {
        text: text.to_string(),
        source: "TechTokenCell32",
        energy: l1_energy(l1, "ScriptCell32").max(0.92),
        risk: 0.02,
        support: top_support(l1),
    })
}

fn looks_like_shell_or_technical_phrase(text: &str) -> bool {
    let mut tokens = text.split_whitespace().peekable();
    let Some(first) = tokens.peek().copied() else {
        return false;
    };
    if !is_common_en_technical_word(&first.to_ascii_lowercase()) {
        return false;
    }
    text.contains(" -")
        || text.contains(" --")
        || text.contains("&&")
        || text.contains("://")
        || text.contains('/')
        || text.contains('=')
}

fn previous_token(prefix: &str) -> Option<&str> {
    prefix.split_whitespace().last()
}

fn layout_risk(token: &str, converted: &str, context: &TailContext) -> f32 {
    let short: f32 = if token.chars().count() <= 2 {
        0.35
    } else {
        0.10
    };
    let technical: f32 = if is_common_en_technical_word(&token.to_ascii_lowercase())
        || is_common_en_technical_word(&converted.to_ascii_lowercase())
    {
        0.20
    } else {
        0.0
    };
    let context_bonus = context.mixed_language_score();
    (short + technical - context_bonus).clamp(0.0, 0.85)
}

fn taught_energy(score: f64, source: &str, l1: &[WavePacket], chosen: bool) -> f32 {
    let base = match source {
        "LayoutWordCell32" => l1_energy(l1, "KeyboardCell32"),
        "BoundaryCell32" => l1_energy(l1, "BoundaryCell32"),
        "GrammarCell32" => l1_energy(l1, "ScriptCell32").max(l1_energy(l1, "BoundaryCell32")),
        "PhraseCell32" => l1_energy(l1, "ScriptCell32"),
        _ => 0.5,
    };
    let score = (score / 14.0).clamp(0.25, 0.95) as f32;
    let chosen_bonus = if chosen { 0.04 } else { 0.0 };
    (base.max(score) + chosen_bonus).min(0.99)
}

fn taught_risk(
    family: TypingCandidateFamily,
    source: &str,
    original: &str,
    replacement: &str,
    chosen: bool,
) -> f32 {
    let edit_ratio = normalized_edit_ratio(original, replacement);
    let base = match family {
        TypingCandidateFamily::Layout => 0.10,
        TypingCandidateFamily::Structural => 0.08,
        TypingCandidateFamily::Typo => 0.14,
        TypingCandidateFamily::Visual | TypingCandidateFamily::Exact => 0.10,
        TypingCandidateFamily::Cleanup | TypingCandidateFamily::Unknown => 0.22,
    };
    let bad_split =
        if source == "BoundaryCell32" && compact_text(original) != compact_text(replacement) {
            0.50
        } else {
            0.0
        };
    let chosen_bonus = if chosen { -0.03 } else { 0.0 };
    (base + edit_ratio * 0.20 + bad_split + chosen_bonus).clamp(0.02, 0.85)
}

fn is_phrase_grammar_candidate(context: &TailContext, original: &str, replacement: &str) -> bool {
    if context.token_count() < 2 {
        return false;
    }
    if original.split_whitespace().count() != replacement.split_whitespace().count() {
        return false;
    }
    if context.has_technical_context() {
        return false;
    }
    let Some(previous) = context.previous() else {
        return false;
    };
    let Some(last) = context.last() else {
        return false;
    };
    if previous.kind != TokenKind::CyrillicWord || last.kind != TokenKind::CyrillicWord {
        return false;
    }
    let previous = clean_ru_token(&previous.text);
    let last = clean_ru_token(&last.text);
    let Some(replacement_last) = replacement
        .split_whitespace()
        .next_back()
        .map(clean_ru_token)
    else {
        return false;
    };
    agree_adjective_like_tail(&previous, &last).as_deref() == Some(replacement_last.as_str())
}

fn normalized_edit_ratio(original: &str, replacement: &str) -> f32 {
    let original_len = original.chars().count().max(1);
    let replacement_len = replacement.chars().count();
    original_len.abs_diff(replacement_len) as f32 / original_len as f32
}

fn compact_text(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn unsafe_single_token_phrase_typo(original: &str, replacement: &str) -> bool {
    let original = original.trim();
    let replacement = replacement.trim();
    if original.split_whitespace().count() != 1 || replacement.split_whitespace().count() != 1 {
        return false;
    }
    if original == replacement {
        return false;
    }
    let original_lower = original.to_lowercase();
    let replacement_lower = replacement.to_lowercase();
    if crate::ru_typo::rewrites_protected_pattern_term_stem(&original_lower, &replacement_lower) {
        return true;
    }
    original.chars().count() >= 4
        && original.chars().all(is_cyrillic_letter)
        && original.chars().any(char::is_uppercase)
        && original.chars().all(|ch| !ch.is_lowercase())
}

fn l1_energy(l1: &[WavePacket], cell: &str) -> f32 {
    l1.iter()
        .filter(|packet| packet.cell == cell)
        .map(WavePacket::top_energy)
        .fold(0.0, f32::max)
}

fn top_support(l1: &[WavePacket]) -> Vec<String> {
    l1.iter()
        .filter_map(|packet| packet.modes.first())
        .take(8)
        .map(|mode| mode.label())
        .collect()
}

fn candidate_support(l1: &[WavePacket], context: &TailContext) -> Vec<String> {
    let mut support = top_support(l1);
    support.push(format!("ctx:{}", context.phrase_signature()));
    support
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::l1::run_l1;

    #[test]
    fn layout_candidate_for_last_token() {
        let original = "html djn ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.text == "html вот"));
    }

    #[test]
    fn l2_weight_scales_candidate_energy() {
        let original = "html djn ";
        let l1 = run_l1(original);
        let normal = run_l2_with_options(original, &l1, &WaveOptions::default());
        let muted = run_l2_with_options(
            original,
            &l1,
            &WaveOptions::default().with_layer_weights(0.5, 1.0),
        );
        let normal_layout = normal
            .iter()
            .find(|candidate| candidate.text == "html вот")
            .expect("normal layout candidate");
        let muted_layout = muted
            .iter()
            .find(|candidate| candidate.text == "html вот")
            .expect("muted layout candidate");

        assert!(muted_layout.energy < normal_layout.energy);
        assert!(muted_layout
            .support
            .iter()
            .any(|item| item == "l2-weight:0.50"));
    }

    #[test]
    fn keeps_known_technical_ascii_token() {
        let original = "git status ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .all(|candidate| candidate.source != "LayoutWordCell32"));
    }

    #[test]
    fn technical_context_does_not_flip_argument_like_ascii() {
        let original = "vpn port ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .all(|candidate| candidate.source != "LayoutWordCell32"));
    }

    #[test]
    fn scans_previous_layout_token_before_technical_tail() {
        let original = "html djn api ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.text == "html вот api"));
    }

    #[test]
    fn exposes_current_and_previous_layout_candidates_to_mesh() {
        let original = "html djn api ашду ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.text == "html djn api file"));
        assert!(candidates
            .iter()
            .any(|candidate| candidate.text == "html вот api ашду"));
    }

    #[test]
    fn mixed_ru_en_context_does_not_emit_raw_malformed_layout_candidate() {
        let original = "тест Ghjljkbv file ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);

        assert!(candidates
            .iter()
            .all(|candidate| candidate.text != "тест Продолим file"));
    }

    #[test]
    fn guard_prefix_blocks_short_layout_argument() {
        let original = "api djn ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .all(|candidate| candidate.text != "api вот"));
    }

    #[test]
    fn does_not_flip_normal_cyrillic_word_to_ascii_noise() {
        let original = "у нас есть ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates.is_empty());
    }

    #[test]
    fn layout_word_cell_respects_known_short_russian_words() {
        let original = "ой ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .all(|candidate| candidate.text != "jq" && candidate.source != "LayoutWordCell32"));
    }

    #[test]
    fn grammar_cell_keeps_known_plural_forms_after_verbs() {
        let original = "имеет волнистые ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .all(|candidate| candidate.text != "имеет волнистый"));
    }

    #[test]
    fn boundary_cell_gets_structural_candidate() {
        let original = "у насесть ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.source == "BoundaryCell32"));
    }

    #[test]
    fn boundary_cell_splits_dictionary_glue() {
        let original = "она есть ";
        let glued = original.replace(' ', "");
        let l1 = run_l1(&glued);
        let candidates = run_l2(&glued, &l1);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.text == "она есть"));
    }

    #[test]
    fn boundary_cell_does_not_split_known_russian_word_forms() {
        for original in ["упоминай ", "поехал ", "поплыл ", "указать ", "сторона "]
        {
            let l1 = run_l1(original);
            let candidates = run_l2(original, &l1);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "BoundaryCell32"),
                "known word must not become boundary split: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn boundary_cell_recovers_one_letter_function_boundary() {
        let original = "влогах ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        let split = candidates
            .iter()
            .find(|candidate| candidate.text == "в логах")
            .expect("hidden short function boundary candidate");
        assert_eq!(split.source, "BoundaryCell32");
        assert!(
            split.energy - split.risk > 0.90,
            "split candidate must outrank single-word typo: {split:?}"
        );
    }

    #[test]
    fn boundary_cell_does_not_split_multi_letter_preposition_guesses() {
        for original in ["заполни поспорта ", "в задани "] {
            let l1 = run_l1(original);
            let candidates = run_l2(original, &l1);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "BoundaryCell32"),
                "multi-letter preposition guesses must not split automatically: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn boundary_cell_scans_glued_word_inside_tail() {
        let original = "я пишу мои слова мои предложения чтобыточно проверить дальше ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.source == "BoundaryCell32"
                    && candidate.text
                        == "я пишу мои слова мои предложения чтобы точно проверить дальше"
            }),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn boundary_cell_scans_split_pair_inside_tail() {
        let original = "сейчас думаю тако й пример работает ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.source == "BoundaryCell32"
                    && candidate.text == "сейчас думаю такой пример работает"
            }),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn boundary_cell_uses_context_to_split_known_glued_form() {
        let original = "мы должны помнить что у насесть право на информацию ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.source == "BoundaryCell32"
                    && candidate.text == "мы должны помнить что у нас есть право на информацию"
            }),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn phrase_cell_does_not_rewrite_single_all_caps_russian_terms() {
        for original in ["БЕЙСОВ ", "БЕЙСОВК ", "БЕЙСОВКИ ", "БЕЙСОВСКИ "]
        {
            let l1 = run_l1(original);
            let candidates = run_l2(original, &l1);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "PhraseCell32"),
                "all-caps term should not get PhraseCell typo candidate: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn phrase_cell_does_not_delete_n_from_pattern_terms() {
        for (original, rejected) in [
            ("патерн ", "патер"),
            ("патерна ", "патера"),
            ("патернов ", "патеров"),
        ] {
            let l1 = run_l1(original);
            let candidates = run_l2(original, &l1);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.text.trim() != rejected),
                "pattern-like term should not get n-deletion candidate: {original:?} -> {candidates:?}"
            );
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "PhraseCell32"),
                "pattern-like term should not get PhraseCell typo candidate: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn phrase_cell_gets_typo_candidate() {
        let original = "рабоатет ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.source == L2_SURFACE_MOTIF_CELL));
    }

    #[test]
    fn l2_surface_motif_cell_generates_word_candidate() {
        let original = "делай проверк ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.source == L2_SURFACE_MOTIF_CELL && candidate.text == "делай проверка"
            }),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn l2_surface_motif_cell_recovers_known_word_from_fuzzy_dictionary() {
        let original = "звгрузи ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.source == L2_SURFACE_MOTIF_CELL && candidate.text == "загрузи"
            }),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn l2_surface_motif_cell_recovers_adjacent_transposition() {
        let original = "пукнт ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.source == L2_SURFACE_MOTIF_CELL && candidate.text == "пункт"
            }),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn l2_surface_motif_does_not_treat_usage_typo_as_stable_word() {
        assert!(!surface_motif_stable_existing_word("пукнт"));
        assert!(!fuzzy_surface_candidate_blocked("пукнт", "пукнт", "пункт"));
        let fuzzy = crate::ru_typo::fuzzy_known_word_candidates("пукнт");
        assert!(fuzzy.iter().any(|candidate| candidate == "пункт"));
        assert!(surface_motif_typo_has_authority(
            "пукнт",
            "пункт",
            900,
            &[],
            &fuzzy
        ));
        assert!(surface_motif_typo_allowed("пукнт", "пункт", 5, 1, 900));
        let l1 = run_l1("пукнт");
        let context = TailContext::from_text("пукнт");
        let cell_candidates =
            surface_motif_word_candidates("", "пукнт", &context, &l1, &WaveOptions::default());
        assert!(
            cell_candidates
                .iter()
                .any(|candidate| candidate.text == "пункт"),
            "cell_candidates={cell_candidates:?}"
        );
    }

    #[test]
    fn l2_surface_motif_memory_recovers_missing_letter_without_fuzzy_route() {
        let candidates = surface_motif_memory().surface_candidates_for_text("звгрузи", 8);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.word == "загрузи"),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn ime_l2_word_candidates_return_whole_words_not_suffixes() {
        let candidates = ime_l2_word_candidates("я хочу ", "пров", 8);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.kind == L2ImeWordCandidateKind::Completion
                    && candidate.surface.starts_with("провер")
            }),
            "L2 IME candidates must expose complete word surfaces, got {candidates:?}"
        );
        assert!(
            candidates
                .iter()
                .all(|candidate| !candidate.surface.starts_with("ер")),
            "L2 must not return display suffixes as word candidates: {candidates:?}"
        );
    }

    #[test]
    fn ime_l2_word_candidates_keep_replacements_distinct_from_completions() {
        let candidates = ime_l2_word_candidates("", "звгрузи", 8);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.kind == L2ImeWordCandidateKind::Replacement
                    && candidate.surface == "загрузи"
            }),
            "noisy input should produce a whole-word replacement candidate, got {candidates:?}"
        );
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.surface != "агрузи"),
            "replacement candidates must not be converted into suffix fragments: {candidates:?}"
        );
    }

    #[test]
    fn l2_surface_motif_memory_recovers_common_shadow_words() {
        for (input, expected) in [("эсперемнт", "эксперимент"), ("ффективная", "эффективная")]
        {
            let candidates = surface_motif_memory().surface_candidates_for_text(input, 32);
            assert!(
                candidates
                    .iter()
                    .any(|candidate| candidate.word == expected),
                "input={input} expected={expected} candidates={candidates:?}"
            );
        }
    }

    #[test]
    fn l2_surface_motif_cell_promotes_common_shadow_words() {
        for (input, expected) in [
            ("эсперемнт ", "эксперимент"),
            ("ффективная ", "эффективная"),
        ] {
            let l1 = run_l1(input);
            let candidates = run_l2(input, &l1);
            let surface_candidates =
                surface_motif_memory().surface_candidates_for_text(input.trim(), 24);
            assert!(
                candidates.iter().any(|candidate| {
                    candidate.source == L2_SURFACE_MOTIF_CELL && candidate.text == expected
                }),
                "input={input} expected={expected} candidates={candidates:?} surface_candidates={surface_candidates:?}"
            );
        }
    }

    #[test]
    fn l2_surface_motif_cell_repairs_repeated_letter_all_caps_word() {
        let original = "ТРУССС ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.source == L2_SURFACE_MOTIF_CELL && candidate.text == "ТРУС"
            }),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn l2_surface_motif_cell_does_not_rewrite_known_word_without_context() {
        let original = "пукнут ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.source != L2_SURFACE_MOTIF_CELL),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn l2_surface_completion_cell_is_separate_from_typo_candidate() {
        let original = "делай пров ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.source == L2_SURFACE_COMPLETION_CELL),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn grammar_cell_does_not_fake_unknown_phrase_candidate() {
        let original = "фразы связанности ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .all(|candidate| candidate.source != "GrammarCell32"));
    }

    #[test]
    fn grammar_cell_generates_agreement_candidate() {
        let original = "расчёт приблизительные ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates.iter().any(|candidate| {
            candidate.source == "GrammarCell32" && candidate.text == "расчёт приблизительный"
        }));
    }

    #[test]
    fn grammar_cell_completes_preposition_case_tail() {
        let original = "в задани ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates.iter().any(|candidate| {
            candidate.source == "GrammarCell32" && candidate.text == "в задании"
        }));
    }

    #[test]
    fn phrase_cell_generates_customs_actor_candidate() {
        let original = "Поставщик говорит что цена до склада нашего покупателя но таможен мы! ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates.iter().any(|candidate| {
            candidate.source == "PhraseCell32"
                && candidate.text
                    == "Поставщик говорит что цена до склада нашего покупателя но таможим мы!"
        }));
    }

    #[test]
    fn phrase_cell_does_not_rewrite_customs_actor_without_right_anchor() {
        let original = "Поставщик говорит что цена до склада нашего покупателя но таможен ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .all(|candidate| !candidate.text.contains("таможим")));
    }

    #[test]
    fn phrase_cell_does_not_rewrite_customs_actor_without_domain_context() {
        let original = "я сказал что странно но таможен мы! ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .all(|candidate| !candidate.text.contains("таможим")));
    }

    #[test]
    fn l2_exposes_l3_phrase_forecast_candidate_when_llmwave_is_enabled() {
        let memory = super::super::llmwave::LlmWaveMemory::from_text("на улице опять идёт дождь");
        let path =
            std::env::temp_dir().join(format!("lay-l2-llmwave-{}.llmw.bin", std::process::id()));
        super::super::llmwave::write_memory_packet(&path, &memory).unwrap();
        std::env::set_var("LAY_LLMWAVE_MEMORY", &path);

        let original = "на улице опять идёт д";
        let l1 = run_l1(original);
        let options = crate::nanda_wave::WaveOptions::default().with_llmwave_shadow(true);
        let candidates = run_l2_with_options(original, &l1, &options);
        std::env::remove_var("LAY_LLMWAVE_MEMORY");
        let _ = std::fs::remove_file(path);

        assert!(candidates.iter().any(|candidate| {
            candidate.source == crate::nanda_wave::context_wave::PHRASE_FORECAST_CELL
                && candidate.text == "на улице опять идёт дождь"
        }));
    }

    #[test]
    fn grammar_cell_keeps_plural_anchor_phrases() {
        for original in ["первые которые ", "такие условие ", "другие перемнные "]
        {
            let l1 = run_l1(original);
            let candidates = run_l2(original, &l1);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "GrammarCell32"),
                "plural anchor phrase should not get grammar candidate: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn grammar_cell_keeps_neuter_nouns_ending_with_ie() {
        for original in ["обратил внимание ", "срабатывает переварачивание "]
        {
            let l1 = run_l1(original);
            let candidates = run_l2(original, &l1);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "GrammarCell32"),
                "neuter noun should not get adjective agreement candidate: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn grammar_cell_keeps_neutral_clause_context() {
        for original in ["там недоказно ", "что там недоказно "] {
            let l1 = run_l1(original);
            let candidates = run_l2(original, &l1);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "GrammarCell32"),
                "neutral clause should not get grammar agreement candidate: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn technical_cell_protects_shell_phrase() {
        let original = "git checkout -b new ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.source == "TechTokenCell32"));
    }

    #[test]
    fn layout_cell_does_not_overrule_teacher_for_plain_ascii() {
        let original = "ola ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates.is_empty());
    }

    #[test]
    fn short_token_cell_exposes_keyboard_and_visual_hypotheses() {
        let original = "пер b ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates.iter().any(|candidate| {
            candidate.source == "ShortTokenCell32" && candidate.text == "пер и"
        }));
        assert!(candidates.iter().any(|candidate| {
            candidate.source == "ShortTokenCell32" && candidate.text == "пер в"
        }));
    }

    #[test]
    fn short_token_cell_marks_ascii_context_as_risky() {
        let original = "vitamin B ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        let short = candidates
            .iter()
            .find(|candidate| {
                candidate.source == "ShortTokenCell32" && candidate.text == "vitamin И"
            })
            .expect("short token candidate");
        assert!(short.risk >= 0.40);
    }
}
