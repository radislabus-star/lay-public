//! Unified candidate admission for live typing.
//!
//! L1/L2/L3 may produce many plausible surfaces. This module decides which of
//! them is allowed to become a live IME completion. It does not output text and
//! does not apply edits.

use super::l2::{self, L2ImeWordCandidateKind};
use super::l4_goal_state::L4AllowedAction;
use super::l4_signed_memory::l4_signed_memory_signal_from_readout;
use crate::keyboard::is_cyrillic_letter;
use crate::typing_transition::decision::TransitionDecisionCore;
use crate::typing_transition::live_candidate::LiveCompletionProposal;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

const LIVE_COMPLETION_CACHE_LIMIT: usize = 128;
const LIVE_L2_MATERIAL_FACTOR: usize = 2;
const LIVE_L2_MATERIAL_CAP: usize = 64;

fn is_live_lexical_surface(surface: &str) -> bool {
    !surface.is_empty()
        && (surface.chars().all(is_cyrillic_letter)
            || surface.chars().all(|ch| ch.is_ascii_alphabetic()))
}

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
    rank_score: f32,
}

pub(crate) fn warm_up_live_candidate_readout() {
    let _ = live_completion_candidates(LiveCompletionRequest {
        context_prefix: "",
        partial: "пр",
        max_suffix_chars: 12,
        allow_short_lexical: true,
        limit: 12,
    });
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
    l3_evaluated: u64,
    l3_suppressed: u64,
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
    if !(1..=18).contains(&partial_len) || !is_live_lexical_surface(&partial) {
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
    let context_batch = live_context_batch_readout(&context_tokens, &raw);
    let l3_memory_warm = super::context_phase::default_memory_is_warm();
    let usage_snapshot = super::usage_prior::cached_usage_prior_snapshot();
    let usage_context = usage_snapshot.prepare_hot_context(&context_tokens);
    let state_id = crate::transition_relation::transition_state_id(&partial);
    let hidden_state_before = crate::stable_hash::mix64_golden(
        crate::nanda_wave::phase_field::hash_text(&cache_key.context_tail)
            ^ crate::nanda_wave::phase_field::hash_text(&partial).rotate_left(19),
    );
    let mut usage_supported = 0_u64;
    let mut l3_supported = 0_u64;
    let mut l3_evaluated = 0_u64;
    let mut l3_suppressed = 0_u64;
    let mut l4_signed = LiveSignedOutcomeStats::default();
    let candidates = raw
        .into_iter()
        .zip(context_batch)
        // The exact surface is an already-typed state, not a transition.
        .filter(|(candidate, _)| candidate.surface != partial)
        .filter_map(|(candidate, l3_report)| {
            let is_completion = candidate.kind == L2ImeWordCandidateKind::Completion;
            let suffix = if is_completion {
                candidate.surface.strip_prefix(&partial)?.to_string()
            } else {
                String::new()
            };
            let suffix_len = suffix.chars().count();
            if is_completion && (suffix.is_empty() || suffix_len > request.max_suffix_chars) {
                return None;
            }
            let memory_readout = usage_snapshot.hot_readout_prepared(
                &usage_context,
                "L2LiveCandidateGate32",
                "completion",
                &state_id,
                &candidate.surface,
            );
            let usage = memory_readout.word_prior;
            let context_usage = memory_readout.context_prior;
            let accepted = memory_readout.accepted_count;
            let common = crate::lexicon::is_common_ru_word(&candidate.surface)
                || crate::lexicon::is_common_en_technical_word(&candidate.surface);
            let foundation_rank = l2::l2_surface_foundation_rank(&candidate.surface);
            let l2_center_grounded = foundation_rank.is_some();
            let hot = foundation_rank.is_some_and(|rank| rank < 20_000);
            let structural = structural_support(
                candidate.score,
                candidate.l1_overlap,
                candidate.l2_overlap,
                candidate.motif_overlap,
            );
            let l3_readout = live_l3_context_score(
                l3_report.as_ref(),
                candidate.surface.chars().count(),
                partial_len,
                request.allow_short_lexical,
                usage,
                context_usage,
                l3_memory_warm,
            );
            let l3_memory_supported = !context_tokens.is_empty()
                && l3_readout.is_some_and(|readout| readout.memory_supported);
            if let Some(readout) = l3_readout {
                l3_evaluated = l3_evaluated.saturating_add(1);
                if readout.suppressed {
                    l3_suppressed = l3_suppressed.saturating_add(1);
                }
            }
            let rejected = memory_readout.rejected_prior + memory_readout.context_rejected;
            let memory_signal = l4_signed_memory_signal_from_readout(
                memory_readout,
                super::usage_prior::UsageSurfaceCoverage::default(),
            );
            l4_signed.record(&memory_signal);
            l4_signed.record_transition(&memory_signal);

            if usage >= 0.025 || context_usage >= 0.018 || accepted >= 1 {
                usage_supported = usage_supported.saturating_add(1);
            }
            if l3_memory_supported {
                l3_supported = l3_supported.saturating_add(1);
            }
            let wave_peak = super::l2_wave_peak::score_live_completion_peak(
                &partial,
                &candidate.surface,
                structural,
                usage,
                context_usage,
                accepted,
                rejected,
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
                + live_l4_signed_bias(memory_signal.signed_weight);
            let rank_score = l3_readout
                .map(|readout| base_score + readout.rank_delta)
                .unwrap_or(base_score);
            let score = rank_score.clamp(0.0, 1.0);
            Some(LiveCompletionProposal {
                state_before: hidden_state_before,
                surface: candidate.surface,
                suffix,
                score,
                source: "L2LiveCandidateGate32",
                rank_score,
                field_strength: candidate.score,
                partial_len,
                suffix_len: if is_completion { suffix_len } else { 0 },
                allow_short_lexical: request.allow_short_lexical,
                structural,
                usage,
                context_usage,
                accepted,
                common,
                hot,
                l2_center_grounded,
                l3_memory_supported,
                completed_state_known: l2_center_grounded,
                l3_relation_class: l3_report
                    .as_ref()
                    .map(|report| report.relation_class)
                    .unwrap_or_default(),
                l4_transition_state_specific: memory_signal.transition_state_specific,
                l4_transition_attract_count: memory_signal.transition_attract_count,
                l4_transition_repel_count: memory_signal.transition_repel_count,
            })
        })
        .collect::<Vec<_>>();
    let candidates = TransitionDecisionCore::select_live_completions(candidates, request.limit)
        .into_iter()
        .map(|candidate| LiveCompletionCandidate {
            surface: candidate.surface,
            suffix: candidate.suffix,
            score: candidate.score,
            source: candidate.source,
            rank_score: candidate.rank_score,
        })
        .collect::<Vec<_>>();
    store_live_completion_candidates(cache_key, &candidates);
    record_live_gate_stats(
        started,
        LiveGateRecord {
            raw_candidates: raw_count as u64,
            returned_candidates: candidates.len() as u64,
            usage_supported,
            l3_supported,
            l3_evaluated,
            l3_suppressed,
            l4_action: None,
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
    if !(1..=18).contains(&token_len) || !is_live_lexical_surface(&normalized) {
        return Vec::new();
    }

    if !l2::ime_word_candidate_memory_is_warm() {
        super::ensure_l2_ime_warmup_started();
        return Vec::new();
    }

    let material_limit = live_l2_material_limit(limit);
    let mut candidates = l2::ime_l2_word_candidates(context_prefix, &normalized, material_limit);
    candidates.extend(l2::ime_l2_completion_candidates(
        context_prefix,
        &normalized,
        material_limit,
    ));
    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.surface.cmp(&right.surface))
    });
    candidates.dedup_by(|left, right| left.surface == right.surface);
    candidates
}

fn live_l2_material_limit(limit: usize) -> usize {
    limit
        .saturating_mul(LIVE_L2_MATERIAL_FACTOR)
        .max(limit)
        .min(LIVE_L2_MATERIAL_CAP)
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
        "l3_evaluated": stats.l3_evaluated,
        "l3_suppressed": stats.l3_suppressed,
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
        l3_evaluated: stats.l3_evaluated.load(Ordering::Relaxed),
        l3_suppressed: stats.l3_suppressed.load(Ordering::Relaxed),
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

#[derive(Debug, Clone, Copy)]
struct LiveL3ContextReadout {
    rank_delta: f32,
    memory_supported: bool,
    suppressed: bool,
}

fn live_context_batch_readout(
    prefix_tokens: &[String],
    candidates: &[l2::L2ImeWordCandidate],
) -> Vec<Option<super::l3_phrase_gate::L3PhraseGateReport>> {
    if candidates.is_empty() {
        return vec![None; candidates.len()];
    }
    let surfaces = candidates
        .iter()
        .map(|candidate| candidate.surface.as_str())
        .collect::<Vec<_>>();
    super::l3_phrase_gate::evaluate_context_candidates_default(prefix_tokens, &surfaces)
}

fn live_l3_context_score(
    report: Option<&super::l3_phrase_gate::L3PhraseGateReport>,
    word_len: usize,
    partial_len: usize,
    allow_short_lexical: bool,
    usage_prior: f32,
    context_usage_prior: f32,
    memory_warm: bool,
) -> Option<LiveL3ContextReadout> {
    let min_lexical_prefix = if allow_short_lexical { 2 } else { 4 };
    let lexical_backoff_allowed = partial_len >= min_lexical_prefix
        && (partial_len >= 4 || word_len.saturating_sub(partial_len) <= 5);
    if let Some(report) = report {
        let score = report.score.clamp(0.0, 1.0);
        let (rank_delta, memory_supported, suppressed) = match report.decision {
            super::l3_phrase_gate::L3PhraseGateDecision::Support => {
                (0.08 + score * 0.24, true, false)
            }
            super::l3_phrase_gate::L3PhraseGateDecision::Neutral => (score * 0.04, false, false),
            super::l3_phrase_gate::L3PhraseGateDecision::Suppress => {
                (-(0.08 + score * 0.20), false, true)
            }
        };
        return Some(LiveL3ContextReadout {
            rank_delta: rank_delta + usage_prior + context_usage_prior,
            memory_supported,
            suppressed,
        });
    }
    if lexical_backoff_allowed || memory_warm {
        Some(LiveL3ContextReadout {
            rank_delta: if lexical_backoff_allowed {
                usage_prior + context_usage_prior
            } else {
                -0.06
            },
            memory_supported: false,
            suppressed: false,
        })
    } else {
        None
    }
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
    stats
        .l3_evaluated
        .fetch_add(record.l3_evaluated, Ordering::Relaxed);
    stats
        .l3_suppressed
        .fetch_add(record.l3_suppressed, Ordering::Relaxed);
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
    l3_evaluated: AtomicU64,
    l3_suppressed: AtomicU64,
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
    fn record(&mut self, signal: &super::l4_signed_memory::L4SignedMemorySignal) {
        if signal.signed_weight > 0.0 {
            self.attract = self.attract.saturating_add(1);
        } else if signal.signed_weight < 0.0 {
            self.repel = self.repel.saturating_add(1);
        } else {
            self.neutral = self.neutral.saturating_add(1);
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
    l3_evaluated: u64,
    l3_suppressed: u64,
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
    use crate::typing_transition::live_candidate::{
        live_completion_has_authority, live_suffix_has_display_authority,
    };

    fn request<'a>(context_prefix: &'a str, partial: &'a str) -> LiveCompletionRequest<'a> {
        LiveCompletionRequest {
            context_prefix,
            partial,
            max_suffix_chars: 24,
            allow_short_lexical: true,
            limit: 12,
        }
    }

    fn authority_proposal() -> LiveCompletionProposal {
        LiveCompletionProposal {
            state_before: crate::nanda_wave::phase_field::hash_text("test-state"),
            surface: "пример".to_string(),
            suffix: "мер".to_string(),
            score: 0.7,
            rank_score: 0.7,
            source: "test",
            partial_len: 3,
            suffix_len: 3,
            allow_short_lexical: true,
            structural: 0.6,
            usage: 0.0,
            context_usage: 0.0,
            accepted: 0,
            common: false,
            hot: false,
            l2_center_grounded: false,
            l3_memory_supported: false,
            completed_state_known: false,
            l3_relation_class: 0,
            l4_transition_state_specific: false,
            l4_transition_attract_count: 0,
            l4_transition_repel_count: 0,
        }
    }

    #[test]
    fn common_english_completion_survives_shared_gate() {
        super::super::warm_up_l2_for_ime();
        let mut input = request("", "exi");
        input.limit = 8;
        let candidates = live_completion_candidates(input);

        assert_eq!(
            candidates
                .first()
                .map(|candidate| candidate.surface.as_str()),
            Some("exit"),
            "candidates={candidates:?}"
        );
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
    fn one_letter_prefix_is_evaluated_but_requires_learned_authority() {
        super::super::warm_up_l2_for_ime();
        let material = live_l2_word_candidates("", "п", 12);
        let candidates = live_completion_candidates(request("", "п"));

        assert!(
            !material.is_empty(),
            "one-letter input must reach L2 candidate memory"
        );
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.surface.starts_with('п')),
            "one-letter readout must stay prefix preserving: {candidates:?}"
        );
    }

    #[test]
    fn live_gate_exposes_replacement_as_a_typed_non_suffix_candidate() {
        super::super::warm_up_l2_for_ime();
        let raw = live_l2_word_candidates("", "звгрузи", 12);
        let candidates = live_completion_candidates(request("", "звгрузи"));
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.surface == "загрузи" && candidate.suffix.is_empty()),
            "replacement must remain explicit rather than being forged into a suffix: raw={raw:?}, selected={candidates:?}"
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
    fn live_gate_rejects_ungrounded_ngram_continuations() {
        super::super::warm_up_l2_for_ime();
        let candidates = live_completion_candidates(request("", "провв"));
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.suffix != "раться"),
            "the synthetic проввраться continuation must not become visible: {candidates:?}"
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
        for partial in ["со", "пол", "дал", "чт"] {
            let candidates = live_l2_word_candidates("", partial, LIVE_L2_MATERIAL_CAP);
            assert!(
                candidates.len() <= LIVE_L2_MATERIAL_CAP,
                "live short-prefix readout must stay bounded for {partial:?}: {}",
                candidates.len()
            );
        }
    }

    #[test]
    fn live_gate_short_prefixes_stay_under_hot_readout_budget() {
        super::super::warm_up_l2_for_ime();
        for partial in ["бу", "де", "дел"] {
            let started = Instant::now();
            let candidates = live_completion_candidates(request("", partial));
            let elapsed_us = started.elapsed().as_micros();
            let budget_us = if cfg!(debug_assertions) {
                50_000
            } else {
                10_000
            };

            assert!(
                elapsed_us <= budget_us,
                "live short-prefix readout too slow for {partial:?}: {elapsed_us}us; candidates={candidates:?}"
            );
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.surface.starts_with(partial)),
                "short-prefix candidates must preserve prefix for {partial:?}: {candidates:?}"
            );
        }
    }

    #[test]
    fn unique_prefix_cache_misses_stay_under_hot_readout_budget() {
        super::super::warm_up_l2_for_ime();
        super::super::warm_up_l3_phrase_memory();
        let mut timings = Vec::new();
        for partial in ["пол", "цел", "рас", "оста", "дост", "остан"] {
            let started = Instant::now();
            let candidates = live_completion_candidates(request("проверяем скорость ", partial));
            timings.push((partial, started.elapsed().as_micros(), candidates.len()));
        }
        let max_us = timings
            .iter()
            .map(|(_, elapsed_us, _)| *elapsed_us)
            .max()
            .unwrap_or_default();
        let budget_us = if cfg!(debug_assertions) {
            20_000
        } else {
            1_500
        };
        eprintln!("unique live prefix timings: {timings:?}; max={max_us}us");

        assert!(
            max_us <= budget_us,
            "unique live prefix readout exceeded {budget_us}us: {timings:?}"
        );
    }

    #[test]
    fn common_completion_outranks_rare_long_surface() {
        super::super::warm_up_l2_for_ime();
        let candidates = live_completion_candidates(request("подсказка не ", "оче"));

        assert_eq!(
            candidates
                .first()
                .map(|candidate| candidate.surface.as_str()),
            Some("очень"),
            "a stable common completion must outrank a rare long surface: {candidates:?}"
        );
    }

    #[test]
    fn generated_surface_without_lexical_binding_has_no_display_authority() {
        assert!(!live_completion_has_authority(&LiveCompletionProposal {
            partial_len: 4,
            suffix_len: 5,
            structural: 0.72,
            ..authority_proposal()
        }));
    }

    #[test]
    fn single_letter_suffix_still_needs_display_authority() {
        assert!(!live_suffix_has_display_authority(
            &LiveCompletionProposal {
                suffix_len: 1,
                suffix: "е".to_string(),
                score: 1.0,
                structural: 0.20,
                ..authority_proposal()
            }
        ));
        assert!(live_suffix_has_display_authority(&LiveCompletionProposal {
            suffix_len: 1,
            suffix: "е".to_string(),
            score: 0.70,
            structural: 0.20,
            usage: 0.10,
            accepted: 2,
            ..authority_proposal()
        }));
        assert!(live_suffix_has_display_authority(&LiveCompletionProposal {
            suffix_len: 1,
            suffix: "й".to_string(),
            score: 0.55,
            structural: 0.30,
            completed_state_known: true,
            ..authority_proposal()
        }));
    }

    #[test]
    fn l2_known_state_completion_outranks_longer_prefix_branch() {
        super::super::warm_up_l2_for_ime();
        let candidates = live_completion_candidates(request("Мы с ", "тобо"));

        assert_eq!(
            candidates.first().map(|item| item.surface.as_str()),
            Some("тобой")
        );
    }

    #[test]
    fn short_mid_sentence_completion_needs_usage_authority() {
        assert!(!live_completion_has_authority(&LiveCompletionProposal {
            partial_len: 3,
            allow_short_lexical: false,
            common: true,
            hot: true,
            ..authority_proposal()
        }));
        assert!(live_completion_has_authority(&LiveCompletionProposal {
            partial_len: 3,
            allow_short_lexical: false,
            structural: 0.0,
            accepted: 1,
            ..authority_proposal()
        }));
    }

    #[test]
    fn long_surface_completion_needs_grounded_memory() {
        let ungrounded = LiveCompletionProposal {
            partial_len: 5,
            suffix_len: 6,
            structural: 0.60,
            ..authority_proposal()
        };
        assert!(!live_completion_has_authority(&ungrounded));
        assert!(live_completion_has_authority(&LiveCompletionProposal {
            l2_center_grounded: true,
            ..ungrounded.clone()
        }));
        assert!(live_completion_has_authority(&LiveCompletionProposal {
            l3_memory_supported: true,
            ..ungrounded
        }));
    }
}
