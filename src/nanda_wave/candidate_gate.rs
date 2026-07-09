//! Unified candidate admission for live typing.
//!
//! L1/L2/L3 may produce many plausible surfaces. This module decides which of
//! them is allowed to become a live IME completion. It does not output text and
//! does not apply edits.

use crate::keyboard::is_cyrillic_letter;

use super::l2::{self, L2ImeWordCandidateKind};
use super::l4_goal_state::{derive_l4_scene_state, L4AllowedAction, L4SceneStateInput};
use super::l4_signed_memory::{l4_signed_memory_signal, L4SignedMemoryInput};
use super::l4_signed_outcome::{l4_signed_outcome, L4OutcomePolarity, L4SignedOutcomeInput};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

const LIVE_COMPLETION_CACHE_LIMIT: usize = 128;
const LIVE_L2_MATERIAL_FACTOR: usize = 2;
const LIVE_L2_MATERIAL_CAP: usize = 64;

#[derive(Debug, Clone, PartialEq)]
pub struct LiveCompletionRequest<'a> {
    pub context_prefix: &'a str,
    pub partial: &'a str,
    pub max_suffix_chars: usize,
    pub allow_short_lexical: bool,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiveCompletionCandidate {
    pub surface: String,
    pub suffix: String,
    pub score: f32,
    pub source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LiveCompletionCacheKey {
    context_tail: String,
    partial: String,
    max_suffix_chars: usize,
    allow_short_lexical: bool,
    limit: usize,
}

#[derive(Debug, Clone)]
struct LiveCompletionCacheEntry {
    key: LiveCompletionCacheKey,
    candidates: Vec<LiveCompletionCandidate>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct LiveCandidateGateStats {
    requests: u64,
    raw_candidates: u64,
    returned_candidates: u64,
    no_candidate: u64,
    usage_supported: u64,
    l3_supported: u64,
    l4_suggest: u64,
    l4_wait: u64,
    l4_block: u64,
    l4_attract: u64,
    l4_neutral: u64,
    l4_repel: u64,
    l4_transition_hits: u64,
    l4_transition_repels: u64,
    l4_scene_memory_hits: u64,
    cache_hits: u64,
    cache_misses: u64,
    total_us: u64,
    max_us: u64,
}

pub fn live_completion_candidates(
    request: LiveCompletionRequest<'_>,
) -> Vec<LiveCompletionCandidate> {
    let started = Instant::now();
    if request.limit == 0 || request.max_suffix_chars == 0 {
        record_live_gate_stats(
            started,
            LiveGateRecord {
                cache_hit: false,
                ..LiveGateRecord::default()
            },
        );
        return Vec::new();
    }
    let partial = request.partial.to_lowercase();
    let partial_len = partial.chars().count();
    if !(2..=18).contains(&partial_len) || !partial.chars().all(is_cyrillic_letter) {
        record_live_gate_stats(
            started,
            LiveGateRecord {
                cache_hit: false,
                ..LiveGateRecord::default()
            },
        );
        return Vec::new();
    }
    if !l2::ime_word_candidate_memory_is_warm() {
        super::ensure_l2_ime_warmup_started();
        record_live_gate_stats(
            started,
            LiveGateRecord {
                cache_hit: false,
                ..LiveGateRecord::default()
            },
        );
        return Vec::new();
    }
    let cache_key = LiveCompletionCacheKey {
        context_tail: live_completion_context_tail(request.context_prefix),
        partial: partial.clone(),
        max_suffix_chars: request.max_suffix_chars,
        allow_short_lexical: request.allow_short_lexical,
        limit: request.limit,
    };
    if let Some(cached) = cached_live_completion_candidates(&cache_key) {
        record_live_gate_stats(
            started,
            LiveGateRecord {
                returned_candidates: cached.len() as u64,
                cache_hit: true,
                ..LiveGateRecord::default()
            },
        );
        return cached;
    }

    let raw = live_l2_word_candidates(request.context_prefix, &partial, request.limit);

    let raw_count = raw.len();
    let context_tokens = super::llmwave::tokenize(request.context_prefix);
    let scene_state = derive_l4_scene_state(L4SceneStateInput {
        context_prefix: request.context_prefix,
        current_word: &partial,
        candidate_count: raw_count,
    });
    let usage_snapshot = super::usage_prior::cached_usage_prior_snapshot();
    let mut usage_supported = 0_u64;
    let mut l3_supported = 0_u64;
    let mut l4_scene_memory_supported = 0_u64;
    let mut l4_signed = LiveSignedOutcomeStats::default();
    let mut candidates = raw
        .into_iter()
        .filter(|candidate| candidate.kind == L2ImeWordCandidateKind::Completion)
        .filter_map(|candidate| {
            let suffix = candidate.surface.strip_prefix(&partial)?.to_string();
            let suffix_len = suffix.chars().count();
            if suffix.is_empty() || suffix_len > request.max_suffix_chars {
                return None;
            }
            let usage = usage_snapshot.word_prior(&candidate.surface);
            let context_usage =
                usage_snapshot.context_word_prior(&context_tokens, &candidate.surface);
            let accepted = usage_snapshot.accepted_word_count(&candidate.surface);
            let common = crate::lexicon::is_common_ru_word(&candidate.surface);
            let hot = crate::lexicon::is_ime_hot_ru_word(&candidate.surface);
            let structural = structural_support(
                candidate.score,
                candidate.l1_overlap,
                candidate.l2_overlap,
                candidate.motif_overlap,
            );
            let l3_score = live_l3_context_score(
                &context_tokens,
                &candidate.surface,
                partial_len,
                request.allow_short_lexical,
                &usage_snapshot,
            );
            let scene_memory_score = if l3_score.is_none() {
                live_l4_scene_memory_score(&context_tokens, &candidate.surface)
            } else {
                None
            };
            let memory_signal = l4_signed_memory_signal(L4SignedMemoryInput {
                context: &context_tokens,
                source: "L2LiveCandidateGate32",
                operation: "completion",
                word: &candidate.surface,
                usage: &usage_snapshot,
            });
            let signed = l4_signed_outcome(L4SignedOutcomeInput {
                scene: &scene_state,
                candidate: &candidate.surface,
                suffix: &suffix,
                partial_len,
                structural,
                usage,
                context_usage,
                accepted,
                learned_attraction: memory_signal.attraction,
                learned_repulsion: memory_signal.repulsion,
            });
            l4_signed.record(signed.polarity);
            l4_signed.record_transition(&memory_signal);

            if !live_completion_has_authority(LiveCompletionAuthority {
                partial_len,
                suffix_len,
                allow_short_lexical: request.allow_short_lexical,
                structural,
                usage,
                context_usage,
                accepted,
                common,
                hot,
            }) {
                return None;
            }

            if usage >= 0.025 || context_usage >= 0.018 || accepted >= 1 {
                usage_supported = usage_supported.saturating_add(1);
            }
            if l3_score.is_some() {
                l3_supported = l3_supported.saturating_add(1);
            }
            if scene_memory_score.is_some() {
                l4_scene_memory_supported = l4_scene_memory_supported.saturating_add(1);
            }
            let wave_peak = super::l2_wave_peak::score_live_completion_peak(
                request.context_prefix,
                &partial,
                &candidate.surface,
                structural,
                usage,
                context_usage,
                accepted,
            );

            let base_score = 0.22
                + structural
                + wave_peak.rank_bonus
                + usage * 2.30
                + context_usage * 3.20
                + (accepted.min(20) as f32 * 0.030)
                + if common { 0.055 } else { 0.0 }
                + if hot { 0.045 } else { 0.0 }
                + (partial_len.min(8) as f32 * 0.018)
                + live_l4_scene_bias(scene_state.allowed_action, scene_state.confidence)
                + live_l4_scene_memory_bias(scene_memory_score)
                + live_l4_signed_bias(signed.signed_weight);
            let score = l3_score
                .map(|score| score.max(base_score))
                .unwrap_or(base_score)
                .clamp(0.0, 1.0);
            if !live_suffix_has_display_authority(LiveSuffixAuthority {
                suffix_len,
                suffix: &suffix,
                score,
                structural,
                usage,
                context_usage,
                accepted,
            }) {
                return None;
            }

            Some(LiveCompletionCandidate {
                surface: candidate.surface,
                suffix,
                score,
                source: "L2LiveCandidateGate32",
            })
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| {
                left.suffix
                    .chars()
                    .count()
                    .cmp(&right.suffix.chars().count())
            })
            .then_with(|| left.surface.cmp(&right.surface))
    });
    candidates.dedup_by(|left, right| left.surface == right.surface || left.suffix == right.suffix);
    candidates.truncate(request.limit);
    store_live_completion_candidates(cache_key, &candidates);
    l4_signed.scene_memory_hits = l4_scene_memory_supported;
    record_live_gate_stats(
        started,
        LiveGateRecord {
            raw_candidates: raw_count as u64,
            returned_candidates: candidates.len() as u64,
            usage_supported,
            l3_supported,
            l4_action: Some(scene_state.allowed_action),
            l4_signed,
            cache_hit: false,
        },
    );
    candidates
}

fn live_l2_word_candidates(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<l2::L2ImeWordCandidate> {
    if limit == 0 {
        return Vec::new();
    }
    let normalized = token.to_lowercase();
    let token_len = normalized.chars().count();
    if !(2..=18).contains(&token_len) || !normalized.chars().all(is_cyrillic_letter) {
        return Vec::new();
    }

    if !l2::ime_word_candidate_memory_is_warm() {
        super::ensure_l2_ime_warmup_started();
        return Vec::new();
    }

    let material_limit = live_l2_material_limit(limit);
    let mut candidates = Vec::new();
    for candidate in
        l2::ime_l2_surface_decoder_candidates(context_prefix, &normalized, material_limit)
    {
        push_unique_live_l2_candidate(&mut candidates, candidate);
        if candidates.len() >= material_limit {
            return candidates;
        }
    }
    for candidate in
        l2::ime_l2_generated_form_prefix_candidates(context_prefix, &normalized, material_limit)
    {
        push_unique_live_l2_candidate(&mut candidates, candidate);
        if candidates.len() >= material_limit {
            return candidates;
        }
    }
    for candidate in
        l2::ime_l2_foundation_prefix_candidates(context_prefix, &normalized, material_limit)
    {
        push_unique_live_l2_candidate(&mut candidates, candidate);
        if candidates.len() >= material_limit {
            return candidates;
        }
    }
    if token_len <= 4 {
        for candidate in
            l2::ime_l2_short_seed_word_candidates(context_prefix, &normalized, material_limit)
        {
            push_unique_live_l2_candidate(&mut candidates, candidate);
            if candidates.len() >= material_limit {
                break;
            }
        }
    }
    candidates
}

fn live_l2_material_limit(limit: usize) -> usize {
    limit
        .saturating_mul(LIVE_L2_MATERIAL_FACTOR)
        .max(limit)
        .min(LIVE_L2_MATERIAL_CAP)
}

fn push_unique_live_l2_candidate(
    candidates: &mut Vec<l2::L2ImeWordCandidate>,
    candidate: l2::L2ImeWordCandidate,
) {
    if candidates
        .iter()
        .any(|existing| existing.surface == candidate.surface)
    {
        return;
    }
    candidates.push(candidate);
}

pub fn live_candidate_gate_stats_json() -> serde_json::Value {
    let stats = live_candidate_gate_stats();
    let avg_us = stats.total_us.checked_div(stats.requests).unwrap_or(0);
    serde_json::json!({
        "requests": stats.requests,
        "raw_candidates": stats.raw_candidates,
        "returned_candidates": stats.returned_candidates,
        "no_candidate": stats.no_candidate,
        "usage_supported": stats.usage_supported,
        "l3_supported": stats.l3_supported,
        "l4_scene": {
            "suggest": stats.l4_suggest,
            "wait": stats.l4_wait,
            "block": stats.l4_block,
        },
        "l4_signed_outcome": {
            "attract": stats.l4_attract,
            "neutral": stats.l4_neutral,
            "repel": stats.l4_repel,
            "transition_hits": stats.l4_transition_hits,
            "transition_repels": stats.l4_transition_repels,
        },
        "l4_scene_memory": {
            "hits": stats.l4_scene_memory_hits,
            "authority": "weak bias only; display authority and edit-plan safety stay final"
        },
        "live_completion_cache": {
            "hits": stats.cache_hits,
            "misses": stats.cache_misses
        },
        "authority_contract": "L4 signed state is bias only; live candidate authority and edit-plan safety remain final",
        "avg_us": avg_us,
        "max_us": stats.max_us,
    })
}

fn live_candidate_gate_stats() -> LiveCandidateGateStats {
    let stats = live_stats();
    LiveCandidateGateStats {
        requests: stats.requests.load(Ordering::Relaxed),
        raw_candidates: stats.raw_candidates.load(Ordering::Relaxed),
        returned_candidates: stats.returned_candidates.load(Ordering::Relaxed),
        no_candidate: stats.no_candidate.load(Ordering::Relaxed),
        usage_supported: stats.usage_supported.load(Ordering::Relaxed),
        l3_supported: stats.l3_supported.load(Ordering::Relaxed),
        l4_suggest: stats.l4_suggest.load(Ordering::Relaxed),
        l4_wait: stats.l4_wait.load(Ordering::Relaxed),
        l4_block: stats.l4_block.load(Ordering::Relaxed),
        l4_attract: stats.l4_attract.load(Ordering::Relaxed),
        l4_neutral: stats.l4_neutral.load(Ordering::Relaxed),
        l4_repel: stats.l4_repel.load(Ordering::Relaxed),
        l4_transition_hits: stats.l4_transition_hits.load(Ordering::Relaxed),
        l4_transition_repels: stats.l4_transition_repels.load(Ordering::Relaxed),
        l4_scene_memory_hits: stats.l4_scene_memory_hits.load(Ordering::Relaxed),
        cache_hits: stats.cache_hits.load(Ordering::Relaxed),
        cache_misses: stats.cache_misses.load(Ordering::Relaxed),
        total_us: stats.total_us.load(Ordering::Relaxed),
        max_us: stats.max_us.load(Ordering::Relaxed),
    }
}

fn live_l3_context_score(
    prefix_tokens: &[String],
    word: &str,
    partial_len: usize,
    allow_short_lexical: bool,
    usage: &super::usage_prior::UsagePriorSnapshot,
) -> Option<f32> {
    let min_lexical_prefix = if allow_short_lexical { 2 } else { 4 };
    let word_len = word.chars().count();
    let lexical_backoff_allowed = partial_len >= min_lexical_prefix
        && (partial_len >= 4 || word_len.saturating_sub(partial_len) <= 5);
    let usage_prior = usage.word_prior(word);
    let context_usage_prior = usage.context_word_prior(prefix_tokens, word);
    if super::llmwave::default_memory_is_warm() {
        return super::llmwave::with_default_memory(|memory| {
            if let Some(report) = memory.score_next_token_report(prefix_tokens, word) {
                return (report.score >= 0.18).then_some(
                    (0.62 + report.score * 0.34 + usage_prior + context_usage_prior)
                        .clamp(0.0, 1.0),
                );
            }
            lexical_backoff_allowed.then_some(
                (0.28 + partial_len as f32 * 0.035 + usage_prior + context_usage_prior)
                    .clamp(0.0, 0.70),
            )
        });
    }
    lexical_backoff_allowed.then_some(
        (0.28 + partial_len as f32 * 0.035 + usage_prior + context_usage_prior).clamp(0.0, 0.70),
    )
}

fn live_l4_scene_memory_score(prefix_tokens: &[String], word: &str) -> Option<f32> {
    if prefix_tokens.len() < 3 || !super::llmwave::default_memory_is_warm() {
        return None;
    }
    super::llmwave::with_default_memory(|memory| {
        memory
            .score_scene_token_report(prefix_tokens, word)
            .and_then(|report| (report.score >= 0.16 && report.support > 0).then_some(report.score))
    })
}

fn record_live_gate_stats(started: Instant, record: LiveGateRecord) {
    let elapsed_us = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    let stats = live_stats();
    stats.requests.fetch_add(1, Ordering::Relaxed);
    stats
        .raw_candidates
        .fetch_add(record.raw_candidates, Ordering::Relaxed);
    stats
        .returned_candidates
        .fetch_add(record.returned_candidates, Ordering::Relaxed);
    if record.returned_candidates == 0 {
        stats.no_candidate.fetch_add(1, Ordering::Relaxed);
    }
    stats
        .usage_supported
        .fetch_add(record.usage_supported, Ordering::Relaxed);
    stats
        .l3_supported
        .fetch_add(record.l3_supported, Ordering::Relaxed);
    match record.l4_action {
        Some(L4AllowedAction::Suggest) => {
            stats.l4_suggest.fetch_add(1, Ordering::Relaxed);
        }
        Some(L4AllowedAction::Wait) => {
            stats.l4_wait.fetch_add(1, Ordering::Relaxed);
        }
        Some(L4AllowedAction::Block) => {
            stats.l4_block.fetch_add(1, Ordering::Relaxed);
        }
        None => {}
    }
    stats
        .l4_attract
        .fetch_add(record.l4_signed.attract, Ordering::Relaxed);
    stats
        .l4_neutral
        .fetch_add(record.l4_signed.neutral, Ordering::Relaxed);
    stats
        .l4_repel
        .fetch_add(record.l4_signed.repel, Ordering::Relaxed);
    stats
        .l4_transition_hits
        .fetch_add(record.l4_signed.transition_hits, Ordering::Relaxed);
    stats
        .l4_transition_repels
        .fetch_add(record.l4_signed.transition_repels, Ordering::Relaxed);
    stats
        .l4_scene_memory_hits
        .fetch_add(record.l4_signed.scene_memory_hits, Ordering::Relaxed);
    if record.cache_hit {
        stats.cache_hits.fetch_add(1, Ordering::Relaxed);
    } else {
        stats.cache_misses.fetch_add(1, Ordering::Relaxed);
    }
    stats.total_us.fetch_add(elapsed_us, Ordering::Relaxed);
    update_max_atomic(&stats.max_us, elapsed_us);
}

fn live_stats() -> &'static LiveCandidateGateAtomicStats {
    static STATS: OnceLock<LiveCandidateGateAtomicStats> = OnceLock::new();
    STATS.get_or_init(LiveCandidateGateAtomicStats::default)
}

#[derive(Default)]
struct LiveCandidateGateAtomicStats {
    requests: AtomicU64,
    raw_candidates: AtomicU64,
    returned_candidates: AtomicU64,
    no_candidate: AtomicU64,
    usage_supported: AtomicU64,
    l3_supported: AtomicU64,
    l4_suggest: AtomicU64,
    l4_wait: AtomicU64,
    l4_block: AtomicU64,
    l4_attract: AtomicU64,
    l4_neutral: AtomicU64,
    l4_repel: AtomicU64,
    l4_transition_hits: AtomicU64,
    l4_transition_repels: AtomicU64,
    l4_scene_memory_hits: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    total_us: AtomicU64,
    max_us: AtomicU64,
}

#[derive(Debug, Clone, Copy, Default)]
struct LiveSignedOutcomeStats {
    attract: u64,
    neutral: u64,
    repel: u64,
    transition_hits: u64,
    transition_repels: u64,
    scene_memory_hits: u64,
}

impl LiveSignedOutcomeStats {
    fn record(&mut self, polarity: L4OutcomePolarity) {
        match polarity {
            L4OutcomePolarity::Attract => self.attract = self.attract.saturating_add(1),
            L4OutcomePolarity::Neutral => self.neutral = self.neutral.saturating_add(1),
            L4OutcomePolarity::Repel => self.repel = self.repel.saturating_add(1),
        }
    }

    fn record_transition(&mut self, signal: &super::l4_signed_memory::L4SignedMemorySignal) {
        if signal.transition_attract_count > 0 || signal.transition_repel_count > 0 {
            self.transition_hits = self.transition_hits.saturating_add(1);
        }
        if signal.transition_repulsion > signal.transition_attraction
            && signal.transition_repel_count > 0
        {
            self.transition_repels = self.transition_repels.saturating_add(1);
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct LiveGateRecord {
    raw_candidates: u64,
    returned_candidates: u64,
    usage_supported: u64,
    l3_supported: u64,
    l4_action: Option<L4AllowedAction>,
    l4_signed: LiveSignedOutcomeStats,
    cache_hit: bool,
}

fn live_completion_context_tail(context_prefix: &str) -> String {
    let mut tokens = context_prefix
        .split_whitespace()
        .rev()
        .take(5)
        .collect::<Vec<_>>();
    tokens.reverse();
    tokens.join(" ")
}

fn cached_live_completion_candidates(
    key: &LiveCompletionCacheKey,
) -> Option<Vec<LiveCompletionCandidate>> {
    let Ok(mut cache) = live_completion_cache().lock() else {
        return None;
    };
    let index = cache.iter().position(|entry| &entry.key == key)?;
    let entry = cache.remove(index)?;
    let candidates = entry.candidates.clone();
    cache.push_back(entry);
    Some(candidates)
}

fn store_live_completion_candidates(
    key: LiveCompletionCacheKey,
    candidates: &[LiveCompletionCandidate],
) {
    let Ok(mut cache) = live_completion_cache().lock() else {
        return;
    };
    if let Some(index) = cache.iter().position(|entry| entry.key == key) {
        cache.remove(index);
    }
    cache.push_back(LiveCompletionCacheEntry {
        key,
        candidates: candidates.to_vec(),
    });
    while cache.len() > LIVE_COMPLETION_CACHE_LIMIT {
        cache.pop_front();
    }
}

fn live_completion_cache() -> &'static Mutex<VecDeque<LiveCompletionCacheEntry>> {
    static CACHE: OnceLock<Mutex<VecDeque<LiveCompletionCacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn update_max_atomic(target: &AtomicU64, value: u64) {
    let mut current = target.load(Ordering::Relaxed);
    while value > current {
        match target.compare_exchange_weak(current, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(next) => current = next,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct LiveCompletionAuthority {
    partial_len: usize,
    suffix_len: usize,
    allow_short_lexical: bool,
    structural: f32,
    usage: f32,
    context_usage: f32,
    accepted: u32,
    common: bool,
    hot: bool,
}

fn live_completion_has_authority(input: LiveCompletionAuthority) -> bool {
    let usage_signal = input.usage >= 0.025 || input.context_usage >= 0.018 || input.accepted >= 1;
    let lexical_signal = input.common || input.hot;
    let structural_signal = input.structural >= 0.34;

    if input.partial_len <= 2 {
        return input.allow_short_lexical
            && (usage_signal
                || structural_signal
                || input.context_usage >= 0.018
                || input.hot
                || input.common)
            && input.suffix_len <= 8;
    }
    if input.partial_len == 3 {
        if !input.allow_short_lexical {
            return usage_signal;
        }
        return usage_signal
            || structural_signal
            || (input.allow_short_lexical && lexical_signal && input.suffix_len <= 7);
    }
    if input.partial_len == 4 {
        return usage_signal || structural_signal || lexical_signal;
    }
    usage_signal || structural_signal || lexical_signal
}

#[derive(Debug, Clone, Copy)]
struct LiveSuffixAuthority<'a> {
    suffix_len: usize,
    suffix: &'a str,
    score: f32,
    structural: f32,
    usage: f32,
    context_usage: f32,
    accepted: u32,
}

fn live_suffix_has_display_authority(input: LiveSuffixAuthority<'_>) -> bool {
    if input.suffix_len != 1 {
        return true;
    }
    if matches!(input.suffix, "и" | "я") {
        return true;
    }
    input.accepted >= 2
        || input.context_usage >= 0.060
        || input.usage >= 0.095
        || (input.score >= 0.90 && input.structural >= 0.46)
}

fn live_l4_scene_bias(action: L4AllowedAction, confidence: f32) -> f32 {
    match action {
        L4AllowedAction::Suggest => 0.030 * confidence,
        L4AllowedAction::Wait => -0.020 * confidence,
        L4AllowedAction::Block => -0.060 * confidence,
    }
}

fn live_l4_scene_memory_bias(score: Option<f32>) -> f32 {
    score
        .map(|score| (score * 0.075).clamp(0.0, 0.090))
        .unwrap_or(0.0)
}

fn live_l4_signed_bias(signed_weight: f32) -> f32 {
    (signed_weight * 0.085).clamp(-0.080, 0.080)
}

fn structural_support(
    score: u32,
    l1_overlap: usize,
    l2_overlap: usize,
    motif_overlap: usize,
) -> f32 {
    let score_part = (score as f32 / 1600.0).clamp(0.0, 0.45);
    let l1_part = (l1_overlap.min(10) as f32) * 0.012;
    let l2_part = (l2_overlap.min(8) as f32) * 0.025;
    let motif_part = (motif_overlap.min(4) as f32) * 0.055;
    (score_part + l1_part + l2_part + motif_part).clamp(0.0, 0.72)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request<'a>(context_prefix: &'a str, partial: &'a str) -> LiveCompletionRequest<'a> {
        LiveCompletionRequest {
            context_prefix,
            partial,
            max_suffix_chars: 24,
            allow_short_lexical: true,
            limit: 12,
        }
    }

    #[test]
    fn live_gate_returns_prefix_preserving_candidates() {
        super::super::warm_up_l2_for_ime();
        let candidates = live_completion_candidates(request("я хочу ", "пров"));
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.surface.starts_with("пров")),
            "live IME must only show prefix-preserving completions: {candidates:?}"
        );
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.surface.starts_with("провер")),
            "expected useful провер* candidates, got {candidates:?}"
        );
    }

    #[test]
    fn live_gate_allows_authorized_short_prefix_candidates() {
        super::super::warm_up_l2_for_ime();
        let center_candidates = l2::ime_l2_word_candidates("я хочу ", "пр", 12);
        assert!(
            center_candidates
                .iter()
                .any(|candidate| candidate.kind == L2ImeWordCandidateKind::Completion),
            "short prefixes should be visible to L2 center memory, not prefix fallback: {center_candidates:?}"
        );

        let live_candidates = live_completion_candidates(request("я хочу ", "пр"));
        assert!(
            live_candidates
                .iter()
                .all(|candidate| candidate.surface.starts_with("пр")),
            "short-prefix candidates must preserve the typed prefix: {live_candidates:?}"
        );
    }

    #[test]
    fn live_gate_keeps_replacement_out_of_suffix_lane() {
        let candidates = live_completion_candidates(request("", "звгрузи"));
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.surface != "загрузи"),
            "replacement must wait for boundary autocorrect, not live suffix: {candidates:?}"
        );
    }

    #[test]
    fn live_gate_prefers_exact_prefix_continuation_from_l2_center() {
        super::super::warm_up_l2_for_ime();
        let candidates = live_completion_candidates(request("как будто нет ", "кандидат"));
        assert!(
            candidates
                .first()
                .is_some_and(|candidate| candidate.surface.starts_with("кандидат")),
            "кандидат should get L2 center continuations first, got {candidates:?}"
        );
    }

    #[test]
    fn live_gate_records_candidate_metrics_without_raw_text() {
        let before = live_candidate_gate_stats();
        let _ = live_completion_candidates(request("я хочу ", "пров"));
        let after = live_candidate_gate_stats();

        assert!(after.requests > before.requests);
        assert!(after.raw_candidates >= before.raw_candidates);
        assert!(after.returned_candidates >= before.returned_candidates);
        assert!(after.total_us >= before.total_us);
    }

    #[test]
    fn live_gate_caps_short_prefix_material_pool() {
        super::super::warm_up_l2_for_ime();
        let before = live_candidate_gate_stats();
        for partial in ["со", "пол", "дал", "чт"] {
            let _ = live_completion_candidates(request("", partial));
        }
        let after = live_candidate_gate_stats();
        let raw_delta = after.raw_candidates.saturating_sub(before.raw_candidates);

        assert!(
            raw_delta <= (LIVE_L2_MATERIAL_CAP as u64) * 4,
            "live short-prefix readout must stay bounded: raw_delta={raw_delta}"
        );
    }

    #[test]
    fn single_letter_suffix_still_needs_display_authority() {
        assert!(!live_suffix_has_display_authority(LiveSuffixAuthority {
            suffix_len: 1,
            suffix: "е",
            score: 1.0,
            structural: 0.20,
            usage: 0.0,
            context_usage: 0.0,
            accepted: 0,
        }));
        assert!(live_suffix_has_display_authority(LiveSuffixAuthority {
            suffix_len: 1,
            suffix: "е",
            score: 0.70,
            structural: 0.20,
            usage: 0.10,
            context_usage: 0.0,
            accepted: 2,
        }));
    }

    #[test]
    fn short_mid_sentence_completion_needs_usage_authority() {
        assert!(!live_completion_has_authority(LiveCompletionAuthority {
            partial_len: 3,
            suffix_len: 3,
            allow_short_lexical: false,
            structural: 0.60,
            usage: 0.0,
            context_usage: 0.0,
            accepted: 0,
            common: true,
            hot: true,
        }));
        assert!(live_completion_has_authority(LiveCompletionAuthority {
            partial_len: 3,
            suffix_len: 3,
            allow_short_lexical: false,
            structural: 0.0,
            usage: 0.0,
            context_usage: 0.0,
            accepted: 1,
            common: false,
            hot: false,
        }));
    }
}
