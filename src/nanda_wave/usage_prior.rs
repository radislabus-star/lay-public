use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::time::unix_timestamp;
#[cfg(test)]
use crate::typing_memory;
use crate::typing_memory::{
    changed_target_indexes, normalize_memory_word, normalized_words, TypingMemoryEvent,
    TypingMemoryEventKind,
};

#[cfg(not(test))]
const USAGE_EVENTS_PATH: &str = ".local/share/lay/nanda_wave/word_usage_events.jsonl";
#[cfg(not(test))]
const USAGE_COUNTS_PATH: &str = ".local/share/lay/nanda_wave/word_usage_counts.json";
#[cfg(not(test))]
const LEGACY_USAGE_PRIOR_PATH: &str = ".local/share/lay/learning_candidates.json";
const USAGE_EVENTS_MAX_BYTES: u64 = 500 * 1024;
const USAGE_EVENTS_FULL_REBUILD_MAX_BYTES: u64 = 8 * 1024 * 1024;
const USAGE_COUNTS_SCHEMA_VERSION: u32 = 9;
const USAGE_COUNTS_MAX_WORDS: usize = 10_000;
const USAGE_COUNTS_MAX_ACCEPTED_WORDS: usize = 5_000;
const USAGE_COUNTS_MAX_CONTEXT_WORDS: usize = 12_000;
const USAGE_COUNTS_MAX_REJECTED_WORDS: usize = 5_000;
const USAGE_COUNTS_MAX_REJECTED_CONTEXT_WORDS: usize = 12_000;
const USAGE_COUNTS_MAX_TRANSITION_STATES: usize = 24_000;
const USAGE_REFRESH_INTERVAL: Duration = Duration::from_millis(1000);
#[cfg(not(test))]
const USAGE_PERSIST_INTERVAL: Duration = Duration::from_millis(1000);
const CONTEXT_WORDS: usize = 5;
const MIN_CONTEXT_NGRAM: usize = 1;
const TRANSITION_ANY: &str = "*";

#[derive(Debug, serde::Deserialize)]
struct LearningCandidate {
    to: String,
    count: u32,
    #[serde(default)]
    promoted: bool,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
struct UsageEvent {
    ts: u64,
    kind: UsageEventKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    word: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    context: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    surface: Option<String>,
}

impl UsageEvent {
    fn from_typing_memory_event(event: &TypingMemoryEvent) -> Self {
        Self {
            ts: unix_timestamp(),
            kind: match event.kind {
                TypingMemoryEventKind::Typed => UsageEventKind::Typed,
                TypingMemoryEventKind::AcceptedFix => UsageEventKind::AcceptedFix,
                TypingMemoryEventKind::AcceptedIme => UsageEventKind::AcceptedIme,
                TypingMemoryEventKind::RejectedIme => UsageEventKind::RejectedIme,
                TypingMemoryEventKind::RejectedCandidate => UsageEventKind::RejectedCandidate,
            },
            word: Some(event.word.clone()),
            context: event.context.clone(),
            from: event.from.clone(),
            to: event.to.clone(),
            source: Some(event.source.clone()),
            operation: Some(event.operation.clone()),
            surface: event.surface.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum UsageEventKind {
    #[default]
    Typed,
    AcceptedFix,
    AcceptedIme,
    RejectedIme,
    RejectedCandidate,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
struct UsageCounts {
    words: HashMap<String, u32>,
    #[serde(default)]
    accepted_words: HashMap<String, u32>,
    context_words: HashMap<String, u32>,
    #[serde(default)]
    rejected_words: HashMap<String, u32>,
    #[serde(default)]
    rejected_context_words: HashMap<String, u32>,
    #[serde(default)]
    transition_observed: HashMap<String, u32>,
    #[serde(default)]
    transition_attract: HashMap<String, u32>,
    #[serde(default)]
    transition_repel: HashMap<String, u32>,
    #[serde(default)]
    surface_observed: HashMap<String, u32>,
    #[serde(default)]
    surface_attract: HashMap<String, u32>,
    #[serde(default)]
    surface_repel: HashMap<String, u32>,
}

#[derive(Debug, Clone, Default)]
pub struct UsagePriorSnapshot {
    counts: Arc<UsageCounts>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct UsageStateMapSummary {
    pub(crate) source_bytes: u64,
    pub(crate) parsed_events: usize,
    pub(crate) word_states: usize,
    pub(crate) accepted_word_states: usize,
    pub(crate) context_word_states: usize,
    pub(crate) rejected_word_states: usize,
    pub(crate) rejected_context_word_states: usize,
    pub(crate) signed_word_states: usize,
    pub(crate) transition_states: usize,
    pub(crate) transition_observed_states: usize,
    pub(crate) transition_attract_states: usize,
    pub(crate) transition_repel_states: usize,
    pub(crate) transition_signed_states: usize,
    pub(crate) transition_conflict_states: usize,
    pub(crate) surface_states: usize,
    pub(crate) surface_observed_states: usize,
    pub(crate) surface_covered_states: usize,
    pub(crate) surface_repelled_states: usize,
    pub(crate) surface_conflict_states: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct UsageTransitionSignal {
    pub(crate) attraction: f32,
    pub(crate) repulsion: f32,
    pub(crate) signed_weight: f32,
    pub(crate) attract_count: u32,
    pub(crate) repel_count: u32,
    pub(crate) state_specific: bool,
    pub(crate) reason: &'static str,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct UsageSurfaceCoverage {
    pub(crate) observed: u32,
    pub(crate) accepted: u32,
    pub(crate) rejected: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct UsageHotReadout {
    pub(crate) word_prior: f32,
    pub(crate) context_prior: f32,
    pub(crate) rejected_prior: f32,
    pub(crate) context_rejected: f32,
    pub(crate) accepted_count: u32,
    pub(crate) rejected_count: u32,
    pub(crate) transition: UsageTransitionSignal,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct UsageHotContext {
    context_keys: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct UsageCandidatePrior {
    pub(crate) word_prior: f32,
    pub(crate) context_prior: f32,
    pub(crate) accepted_count: u32,
}

impl UsagePriorSnapshot {
    pub(crate) fn surface_coverage(&self, surface: &str) -> UsageSurfaceCoverage {
        UsageSurfaceCoverage {
            observed: self
                .counts
                .surface_observed
                .get(surface)
                .copied()
                .unwrap_or_default(),
            accepted: self
                .counts
                .surface_attract
                .get(surface)
                .copied()
                .unwrap_or_default(),
            rejected: self
                .counts
                .surface_repel
                .get(surface)
                .copied()
                .unwrap_or_default(),
        }
    }
    pub fn word_prior(&self, word: &str) -> f32 {
        let lower = normalize_memory_word(word);
        if lower.is_empty() {
            return 0.0;
        }
        self.counts
            .words
            .get(&lower)
            .copied()
            .map(word_prior_from_count)
            .unwrap_or(0.0)
    }

    pub fn context_word_prior(&self, context: &[String], word: &str) -> f32 {
        let lower = normalize_memory_word(word);
        if lower.is_empty() || context.is_empty() {
            return 0.0;
        }
        context_ngram_prior_from_counts(&self.counts, context, &lower)
    }

    pub fn accepted_word_count(&self, word: &str) -> u32 {
        let lower = normalize_memory_word(word);
        if lower.is_empty() {
            return 0;
        }
        self.counts
            .accepted_words
            .get(&lower)
            .copied()
            .unwrap_or_default()
    }

    pub(crate) fn rejected_word_prior(&self, word: &str) -> f32 {
        let lower = normalize_memory_word(word);
        if lower.is_empty() {
            return 0.0;
        }
        self.counts
            .rejected_words
            .get(&lower)
            .copied()
            .map(rejected_prior_from_count)
            .unwrap_or(0.0)
    }

    pub(crate) fn context_rejected_word_prior(&self, context: &[String], word: &str) -> f32 {
        let lower = normalize_memory_word(word);
        if lower.is_empty() || context.is_empty() {
            return 0.0;
        }
        context_ngram_prior_from_map(&self.counts.rejected_context_words, context, &lower, 0.012)
    }

    pub(crate) fn hot_readout(
        &self,
        context: &[String],
        source: &str,
        operation: &str,
        state_word: &str,
        candidate_text: &str,
    ) -> UsageHotReadout {
        let prepared = self.prepare_hot_context(context);
        self.hot_readout_prepared(&prepared, source, operation, state_word, candidate_text)
    }

    pub(crate) fn prepare_hot_context(&self, context: &[String]) -> UsageHotContext {
        UsageHotContext {
            context_keys: context_ngram_keys(context),
        }
    }

    pub(crate) fn candidate_prior_prepared(
        &self,
        context: &UsageHotContext,
        normalized_word: &str,
    ) -> UsageCandidatePrior {
        if normalized_word.is_empty() {
            return UsageCandidatePrior::default();
        }
        UsageCandidatePrior {
            word_prior: self
                .counts
                .words
                .get(normalized_word)
                .copied()
                .map(word_prior_from_count)
                .unwrap_or_default(),
            context_prior: context_ngram_prior_from_keys(
                &self.counts.context_words,
                &context.context_keys,
                normalized_word,
                0.020,
            ),
            accepted_count: self
                .counts
                .accepted_words
                .get(normalized_word)
                .copied()
                .unwrap_or_default(),
        }
    }

    pub(crate) fn hot_readout_prepared(
        &self,
        context: &UsageHotContext,
        source: &str,
        operation: &str,
        state_word: &str,
        candidate_text: &str,
    ) -> UsageHotReadout {
        let lower = normalized_words(candidate_text)
            .into_iter()
            .next_back()
            .unwrap_or_default();
        if lower.is_empty() {
            return UsageHotReadout::default();
        }
        let transition_target = crate::transition_relation::transition_target_id(candidate_text);
        let context_keys = &context.context_keys;
        UsageHotReadout {
            word_prior: self
                .counts
                .words
                .get(&lower)
                .copied()
                .map(word_prior_from_count)
                .unwrap_or_default(),
            context_prior: context_ngram_prior_from_keys(
                &self.counts.context_words,
                context_keys,
                &lower,
                0.020,
            ),
            rejected_prior: self
                .counts
                .rejected_words
                .get(&lower)
                .copied()
                .map(rejected_prior_from_count)
                .unwrap_or_default(),
            context_rejected: context_ngram_prior_from_keys(
                &self.counts.rejected_context_words,
                context_keys,
                &lower,
                0.012,
            ),
            accepted_count: self
                .counts
                .accepted_words
                .get(&lower)
                .copied()
                .unwrap_or_default(),
            rejected_count: self
                .counts
                .rejected_words
                .get(&lower)
                .copied()
                .unwrap_or_default(),
            transition: transition_signal_from_counts_for_word(
                &self.counts,
                context_keys,
                source,
                operation,
                state_word,
                &transition_target,
            ),
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
struct PersistedUsageCounts {
    #[serde(default)]
    schema_version: u32,
    source_len: u64,
    counts: UsageCounts,
}

#[derive(Debug, Default)]
struct UsageCache {
    loaded_at: Option<Instant>,
    counts: Arc<UsageCounts>,
}

#[cfg(not(test))]
struct UsagePersistLine {
    path: PathBuf,
    line: String,
}

#[cfg(not(test))]
static USAGE_PERSIST_SENDER: OnceLock<Sender<UsagePersistLine>> = OnceLock::new();
static LAST_USAGE_EVENT: OnceLock<Mutex<Option<UsageEvent>>> = OnceLock::new();

pub(crate) fn record_typed_tail_if_enabled(tail: &str) {
    if !usage_learning_enabled() {
        return;
    }
    let Some(event) = TypingMemoryEvent::typed_tail(tail) else {
        return;
    };
    record_typing_memory_event_if_enabled(&event);
}

pub(crate) fn record_accepted_fix_if_enabled(from: &str, to: &str) {
    if !usage_learning_enabled() || from == to {
        return;
    }
    for event in TypingMemoryEvent::accepted_fix(from, to) {
        record_typing_memory_event_if_enabled(&event);
    }
    super::llmwave::record_phrase_experience("space", to);
}

pub(crate) fn record_accepted_ime_if_enabled(context_tail: &str, accepted_text: &str) {
    if !usage_learning_enabled() {
        return;
    }
    for event in TypingMemoryEvent::accepted_ime(context_tail, accepted_text) {
        record_typing_memory_event_if_enabled(&event);
    }
    let phrase = [context_tail.trim(), accepted_text.trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    super::llmwave::record_phrase_experience("space", &phrase);
}

pub(crate) fn record_rejected_ime_if_enabled(context_tail: &str, rejected_text: &str) {
    if !usage_learning_enabled() {
        return;
    }
    for event in TypingMemoryEvent::rejected_ime(context_tail, rejected_text) {
        record_typing_memory_event_if_enabled(&event);
    }
}

pub(crate) fn record_rejected_candidate_if_enabled(
    context_tail: &str,
    rejected_text: &str,
    source: &str,
    operation: &str,
) {
    if !usage_learning_enabled() {
        return;
    }
    for event in
        TypingMemoryEvent::rejected_candidate(context_tail, rejected_text, source, operation)
    {
        record_typing_memory_event_if_enabled(&event);
    }
}

pub(crate) fn record_typing_memory_event_if_enabled(event: &TypingMemoryEvent) {
    if !usage_learning_enabled() {
        return;
    }
    append_usage_event(UsageEvent::from_typing_memory_event(event));
}

pub(crate) fn word_usage_prior(word: &str) -> f32 {
    let lower = normalize_memory_word(word);
    if lower.is_empty() {
        return 0.0;
    }
    let counts = usage_counts();
    let Some(count) = counts.words.get(&lower).copied() else {
        return 0.0;
    };
    word_prior_from_count(count)
}

pub(crate) fn context_word_usage_prior(context: &[String], word: &str) -> f32 {
    let lower = normalize_memory_word(word);
    if lower.is_empty() || context.is_empty() {
        return 0.0;
    }
    let counts = usage_counts();
    context_ngram_prior_from_counts(&counts, context, &lower)
}

fn word_prior_from_count(count: u32) -> f32 {
    ((count as f32 + 1.0).ln() * 0.036).clamp(0.0, 0.22)
}

fn usage_learning_enabled() -> bool {
    let config = crate::config::LayConfig::load();
    config.learning_log || config.nanda_precognition || config.nanda_autocorrect
}

fn usage_counts() -> Arc<UsageCounts> {
    let cache = usage_cache();
    let Ok(mut cache) = cache.lock() else {
        return Arc::new(UsageCounts::default());
    };
    if cache
        .loaded_at
        .is_some_and(|loaded_at| loaded_at.elapsed() < USAGE_REFRESH_INTERVAL)
    {
        return Arc::clone(&cache.counts);
    }
    cache.counts = Arc::new(load_usage_counts());
    cache.loaded_at = Some(Instant::now());
    Arc::clone(&cache.counts)
}

pub(crate) fn word_usage_prior_cached(word: &str) -> f32 {
    let lower = normalize_memory_word(word);
    if lower.is_empty() {
        return 0.0;
    }
    let counts = cached_usage_counts();
    counts
        .words
        .get(&lower)
        .copied()
        .map(word_prior_from_count)
        .unwrap_or(0.0)
}

pub(crate) fn accepted_word_usage_count_cached(word: &str) -> u32 {
    let lower = normalize_memory_word(word);
    if lower.is_empty() {
        return 0;
    }
    let Ok(cache) = usage_cache().lock() else {
        return 0;
    };
    cache
        .counts
        .accepted_words
        .get(&lower)
        .copied()
        .unwrap_or(0)
}

pub(crate) fn context_word_usage_prior_cached(context: &[String], word: &str) -> f32 {
    let lower = normalize_memory_word(word);
    if lower.is_empty() || context.is_empty() {
        return 0.0;
    }
    let counts = cached_usage_counts();
    context_ngram_prior_from_counts(&counts, context, &lower)
}

pub(crate) fn cached_usage_prior_snapshot() -> UsagePriorSnapshot {
    UsagePriorSnapshot {
        counts: cached_usage_counts(),
    }
}

#[cfg(test)]
pub(crate) fn snapshot_from_usage_events_for_tests(text: &str) -> UsagePriorSnapshot {
    let mut counts = UsageCounts::default();
    add_usage_event_counts(&mut counts, text);
    UsagePriorSnapshot {
        counts: Arc::new(counts),
    }
}

pub(crate) fn l2_surface_words_by_usage(limit: usize) -> Vec<String> {
    if limit == 0 {
        return Vec::new();
    }
    let counts = refresh_usage_counts_from_disk();
    let mut words = usage_surface_words_from_counts(counts);
    words.truncate(limit);
    words
}

pub(crate) fn usage_debug_summary() -> (u64, usize, usize) {
    let summary = usage_state_map_summary();
    (
        summary.source_bytes,
        summary.parsed_events,
        summary.word_states,
    )
}

pub(crate) fn usage_state_map_summary() -> UsageStateMapSummary {
    let text = usage_events_path()
        .and_then(|path| read_usage_events_text(&path))
        .unwrap_or_default();
    let counts = load_usage_counts();
    UsageStateMapSummary {
        source_bytes: text.len() as u64,
        parsed_events: usage_events_from_jsonl(&text).count(),
        word_states: counts.words.len(),
        accepted_word_states: counts.accepted_words.len(),
        context_word_states: counts.context_words.len(),
        rejected_word_states: counts.rejected_words.len(),
        rejected_context_word_states: counts.rejected_context_words.len(),
        signed_word_states: counts
            .accepted_words
            .keys()
            .chain(counts.rejected_words.keys())
            .collect::<HashSet<_>>()
            .len(),
        transition_states: counts
            .transition_observed
            .keys()
            .chain(counts.transition_attract.keys())
            .chain(counts.transition_repel.keys())
            .collect::<HashSet<_>>()
            .len(),
        transition_observed_states: counts.transition_observed.len(),
        transition_attract_states: counts.transition_attract.len(),
        transition_repel_states: counts.transition_repel.len(),
        transition_signed_states: counts
            .transition_attract
            .keys()
            .chain(counts.transition_repel.keys())
            .collect::<HashSet<_>>()
            .len(),
        transition_conflict_states: counts
            .transition_attract
            .keys()
            .filter(|key| counts.transition_repel.contains_key(*key))
            .count(),
        surface_states: counts
            .surface_observed
            .keys()
            .chain(counts.surface_attract.keys())
            .chain(counts.surface_repel.keys())
            .collect::<HashSet<_>>()
            .len(),
        surface_observed_states: counts.surface_observed.len(),
        surface_covered_states: counts.surface_attract.len(),
        surface_repelled_states: counts.surface_repel.len(),
        surface_conflict_states: counts
            .surface_attract
            .keys()
            .filter(|key| counts.surface_repel.contains_key(*key))
            .count(),
    }
}

pub fn usage_memory_learned_report_json() -> serde_json::Value {
    let text = usage_events_path()
        .and_then(|path| read_usage_events_text(&path))
        .unwrap_or_default();
    let counts = load_usage_counts();
    let summary = usage_state_map_summary();
    serde_json::json!({
        "kind": "typing_memory_learned_report",
        "status": "ok",
        "source": "word_usage_events.jsonl + word_usage_counts.json",
        "summary": {
            "source_bytes": summary.source_bytes,
            "parsed_events": summary.parsed_events,
            "word_states": summary.word_states,
            "accepted_word_states": summary.accepted_word_states,
            "context_word_states": summary.context_word_states,
            "rejected_word_states": summary.rejected_word_states,
            "rejected_context_word_states": summary.rejected_context_word_states,
            "signed_word_states": summary.signed_word_states,
            "transition_states": summary.transition_states,
            "transition_observed_states": summary.transition_observed_states,
            "transition_attract_states": summary.transition_attract_states,
            "transition_repel_states": summary.transition_repel_states,
            "transition_signed_states": summary.transition_signed_states,
            "transition_conflict_states": summary.transition_conflict_states,
            "surface_states": summary.surface_states,
            "surface_observed_states": summary.surface_observed_states,
            "surface_covered_states": summary.surface_covered_states,
            "surface_repelled_states": summary.surface_repelled_states,
            "surface_conflict_states": summary.surface_conflict_states
        },
        "learned_top": {
            "accepted_words": top_count_json(&counts.accepted_words, 12),
            "rejected_words": top_count_json(&counts.rejected_words, 12),
            "context_words": top_count_json(&counts.context_words, 12),
            "transition_attract": top_count_json(&counts.transition_attract, 12),
            "transition_repel": top_count_json(&counts.transition_repel, 12),
            "surface_covered": top_count_json(&counts.surface_attract, 12),
            "surface_repelled": top_count_json(&counts.surface_repel, 12)
        },
        "hot_readout": {
            "mode": "UsagePriorSnapshot::hot_readout",
            "single_pass": true,
            "uses": ["word_prior", "context_prior", "rejected_prior", "context_rejected", "accepted_count", "rejected_count", "transition_signal", "surface_frontier"]
        },
        "events_tail_bytes": text.len(),
        "authority": "ranking signal only; edit safety gate remains final"
    })
}

fn refresh_usage_counts_from_disk() -> UsageCounts {
    let counts = load_usage_counts();
    if let Ok(mut cache) = usage_cache().lock() {
        cache.counts = Arc::new(counts.clone());
        cache.loaded_at = Some(Instant::now());
    }
    counts
}

fn usage_surface_words_from_counts(counts: UsageCounts) -> Vec<String> {
    let word_counts = counts.words;
    let mut words = counts
        .accepted_words
        .into_iter()
        .filter(|(word, count)| {
            *count >= 1
                && (2..=32).contains(&word.chars().count())
                && word.chars().all(|ch| ch.is_alphabetic() || ch == '-')
        })
        .map(|(word, accepted_count)| {
            let typed_count = word_counts.get(&word).copied().unwrap_or_default();
            let score = accepted_count
                .saturating_mul(4)
                .saturating_add(typed_count.min(50));
            (word, score)
        })
        .collect::<Vec<_>>();
    words.sort_by(|(left_word, left_count), (right_word, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_word.chars().count().cmp(&right_word.chars().count()))
            .then_with(|| left_word.cmp(right_word))
    });
    words.into_iter().map(|(word, _)| word).collect()
}

fn context_ngram_prior_from_counts(counts: &UsageCounts, context: &[String], word: &str) -> f32 {
    let context_keys = context_ngram_keys(context);
    context_ngram_prior_from_keys(&counts.context_words, &context_keys, word, 0.020)
}

fn context_ngram_prior_from_map(
    source: &HashMap<String, u32>,
    context: &[String],
    word: &str,
    base_weight: f32,
) -> f32 {
    let context_keys = context_ngram_keys(context);
    context_ngram_prior_from_keys(source, &context_keys, word, base_weight)
}

fn context_ngram_prior_from_keys(
    source: &HashMap<String, u32>,
    context_keys: &[String],
    word: &str,
    base_weight: f32,
) -> f32 {
    context_keys
        .iter()
        .filter_map(|context_key| {
            let ngram_len = context_key.split_whitespace().count();
            let key = context_word_key(context_key, word);
            source.get(&key).copied().map(|count| (count, ngram_len))
        })
        .map(|(count, ngram_len)| {
            let ngram_weight = base_weight + ngram_len as f32 * 0.010;
            ((count as f32 + 1.0).ln() * ngram_weight).min(0.18)
        })
        .sum::<f32>()
        .clamp(0.0, 0.34)
}

fn cached_usage_counts() -> Arc<UsageCounts> {
    let Ok(mut cache) = usage_cache().lock() else {
        return Arc::new(UsageCounts::default());
    };
    // The hot readout is allowed to be the first usage-memory consumer.
    // Returning the default Arc here made L4 depend on an unrelated warmup
    // route and left first-word decisions blind until another action loaded
    // the persisted state.
    ensure_usage_cache_initialized(&mut cache, load_usage_counts);
    cache.counts.clone()
}

fn ensure_usage_cache_initialized(cache: &mut UsageCache, load: impl FnOnce() -> UsageCounts) {
    if cache.loaded_at.is_some() {
        return;
    }
    cache.counts = Arc::new(load());
    cache.loaded_at = Some(Instant::now());
}

fn usage_cache() -> &'static Mutex<UsageCache> {
    static CACHE: OnceLock<Mutex<UsageCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(UsageCache::default()))
}

fn load_usage_counts() -> UsageCounts {
    let mut counts = UsageCounts::default();
    if let Some(text) =
        legacy_usage_prior_path().and_then(|path| std::fs::read_to_string(path).ok())
    {
        add_legacy_usage_counts(&mut counts, &text);
    }
    merge_usage_counts(&mut counts, load_usage_event_counts());
    counts
}

fn load_usage_event_counts() -> UsageCounts {
    let Some(path) = usage_events_path() else {
        return UsageCounts::default();
    };
    let source_len = std::fs::metadata(&path)
        .map(|meta| meta.len())
        .unwrap_or_default();
    if let Some(snapshot) = load_usage_counts_snapshot(source_len) {
        return snapshot;
    }

    let text = if source_len <= USAGE_EVENTS_FULL_REBUILD_MAX_BYTES {
        read_full_text_lossy(&path)
    } else {
        read_usage_events_text(&path)
    };
    let mut counts = UsageCounts::default();
    if let Some(text) = text {
        add_usage_event_counts(&mut counts, &text);
    }
    persist_usage_counts_snapshot(&counts, source_len);
    counts
}

fn merge_usage_counts(target: &mut UsageCounts, source: UsageCounts) {
    for (word, count) in source.words {
        *target.words.entry(word).or_default() = target
            .words
            .get(&word)
            .copied()
            .unwrap_or_default()
            .saturating_add(count);
    }
    for (word, count) in source.accepted_words {
        *target.accepted_words.entry(word).or_default() = target
            .accepted_words
            .get(&word)
            .copied()
            .unwrap_or_default()
            .saturating_add(count);
    }
    for (key, count) in source.context_words {
        *target.context_words.entry(key).or_default() = target
            .context_words
            .get(&key)
            .copied()
            .unwrap_or_default()
            .saturating_add(count);
    }
    for (word, count) in source.rejected_words {
        *target.rejected_words.entry(word.clone()).or_default() = target
            .rejected_words
            .get(&word)
            .copied()
            .unwrap_or_default()
            .saturating_add(count);
    }
    for (key, count) in source.rejected_context_words {
        *target
            .rejected_context_words
            .entry(key.clone())
            .or_default() = target
            .rejected_context_words
            .get(&key)
            .copied()
            .unwrap_or_default()
            .saturating_add(count);
    }
    merge_count_map(&mut target.transition_observed, source.transition_observed);
    merge_count_map(&mut target.transition_attract, source.transition_attract);
    merge_count_map(&mut target.transition_repel, source.transition_repel);
}

fn merge_count_map(target: &mut HashMap<String, u32>, source: HashMap<String, u32>) {
    for (key, count) in source {
        *target.entry(key.clone()).or_default() = target
            .get(&key)
            .copied()
            .unwrap_or_default()
            .saturating_add(count);
    }
}

fn add_legacy_usage_counts(counts: &mut UsageCounts, text: &str) {
    for (word, count) in legacy_usage_counts_from_json(text) {
        *counts.words.entry(word).or_default() = counts
            .words
            .get(&word)
            .copied()
            .unwrap_or_default()
            .saturating_add(count);
    }
}

fn add_usage_event_counts(counts: &mut UsageCounts, text: &str) {
    for event in usage_events_from_jsonl(text) {
        add_usage_event_count(counts, &event);
    }
}

fn add_usage_event_count(counts: &mut UsageCounts, event: &UsageEvent) {
    let Some(word) = event.word.as_deref().map(normalize_memory_word) else {
        return;
    };
    if word.is_empty() {
        return;
    }
    if matches!(event.kind, UsageEventKind::RejectedCandidate)
        && !event_word_is_changed_target(event, &word)
    {
        return;
    }
    let state_word = event_state_word(event);
    let transition_target = event_transition_target(event, &word);
    let transition_context = event_transition_context(event);
    if matches!(
        event.kind,
        UsageEventKind::RejectedIme | UsageEventKind::RejectedCandidate
    ) {
        let weight = rejected_usage_weight(event.kind);
        if let Some(surface) = event.surface.as_deref() {
            *counts
                .surface_observed
                .entry(surface.to_string())
                .or_default() += weight;
        }
        add_rejected_word_state(
            counts,
            RejectedStateEvidence {
                context: &event.context,
                source: event_source(event),
                operation: event_operation(event),
                state_word: &state_word,
                rejected: &word,
                transition_context: &transition_context,
                transition_target: &transition_target,
                weight,
                transition_weight: event_transition_weight(event, weight),
                record_transition: true,
            },
        );
        return;
    }
    let weight = match event.kind {
        UsageEventKind::Typed => 1,
        UsageEventKind::AcceptedFix => 6,
        UsageEventKind::AcceptedIme => 5,
        UsageEventKind::RejectedIme | UsageEventKind::RejectedCandidate => {
            unreachable!("handled before positive count")
        }
    };
    if let Some(surface) = event.surface.as_deref() {
        *counts
            .surface_observed
            .entry(surface.to_string())
            .or_default() += weight;
        if matches!(
            event.kind,
            UsageEventKind::AcceptedFix | UsageEventKind::AcceptedIme
        ) {
            *counts
                .surface_attract
                .entry(surface.to_string())
                .or_default() += weight;
        }
    }
    *counts.words.entry(word.clone()).or_default() = counts
        .words
        .get(&word)
        .copied()
        .unwrap_or_default()
        .saturating_add(weight);
    if !matches!(event.kind, UsageEventKind::Typed) {
        *counts.accepted_words.entry(word.clone()).or_default() = counts
            .accepted_words
            .get(&word)
            .copied()
            .unwrap_or_default()
            .saturating_add(weight);
    }

    for context_key in context_ngram_keys(&event.context) {
        let key = context_word_key(&context_key, &word);
        *counts.context_words.entry(key.clone()).or_default() = counts
            .context_words
            .get(&key)
            .copied()
            .unwrap_or_default()
            .saturating_add(weight);
    }
    let source = event_source(event);
    let operation = event_operation(event);
    add_transition_counts(
        &mut counts.transition_observed,
        &transition_context,
        source,
        operation,
        &state_word,
        &transition_target,
        event_transition_weight(event, weight),
    );
    if !matches!(event.kind, UsageEventKind::Typed) {
        add_transition_counts(
            &mut counts.transition_attract,
            &transition_context,
            source,
            operation,
            &state_word,
            &transition_target,
            event_transition_weight(event, weight),
        );
    }

    if matches!(event.kind, UsageEventKind::AcceptedFix) {
        add_rejected_fix_sources(counts, event, weight, source, operation);
    }
}

fn rejected_usage_weight(kind: UsageEventKind) -> u32 {
    match kind {
        UsageEventKind::RejectedCandidate => 8,
        UsageEventKind::RejectedIme => 8,
        _ => 0,
    }
}

fn usage_events_from_jsonl(text: &str) -> impl Iterator<Item = UsageEvent> + '_ {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<UsageEvent>(line).ok())
}

fn legacy_usage_counts_from_json(text: &str) -> HashMap<String, u32> {
    let Ok(records) = serde_json::from_str::<HashMap<String, LearningCandidate>>(text) else {
        return HashMap::new();
    };
    let mut counts = HashMap::<String, u32>::new();
    for record in records.into_values() {
        let token = normalized_words(&record.to)
            .into_iter()
            .next_back()
            .unwrap_or_default();
        if token.is_empty() {
            continue;
        }
        let weight = record
            .count
            .saturating_add(if record.promoted { 3 } else { 0 });
        *counts.entry(token.clone()).or_default() = counts
            .get(&token)
            .copied()
            .unwrap_or_default()
            .saturating_add(weight);
    }
    counts
}

fn add_rejected_fix_sources(
    counts: &mut UsageCounts,
    event: &UsageEvent,
    weight: u32,
    source: &str,
    operation: &str,
) {
    let Some(from) = event.from.as_deref() else {
        return;
    };
    let accepted = event
        .to
        .as_deref()
        .map(normalized_words)
        .unwrap_or_default()
        .into_iter()
        .collect::<HashSet<_>>();
    for rejected in normalized_words(from)
        .into_iter()
        .filter(|word| !accepted.contains(word))
    {
        add_rejected_word_state(
            counts,
            RejectedStateEvidence {
                context: &event.context,
                source,
                operation,
                state_word: &rejected,
                rejected: &rejected,
                transition_context: &event.context,
                transition_target: &rejected,
                weight,
                transition_weight: weight,
                record_transition: false,
            },
        );
    }
}

struct RejectedStateEvidence<'a> {
    context: &'a [String],
    source: &'a str,
    operation: &'a str,
    state_word: &'a str,
    rejected: &'a str,
    transition_context: &'a [String],
    transition_target: &'a str,
    weight: u32,
    transition_weight: u32,
    record_transition: bool,
}

fn add_rejected_word_state(counts: &mut UsageCounts, evidence: RejectedStateEvidence<'_>) {
    let RejectedStateEvidence {
        context,
        source,
        operation,
        state_word,
        rejected,
        transition_context,
        transition_target,
        weight,
        transition_weight,
        record_transition,
    } = evidence;
    *counts
        .rejected_words
        .entry(rejected.to_string())
        .or_default() = counts
        .rejected_words
        .get(rejected)
        .copied()
        .unwrap_or_default()
        .saturating_add(weight);
    for context_key in context_ngram_keys(context) {
        let key = context_word_key(&context_key, rejected);
        *counts
            .rejected_context_words
            .entry(key.clone())
            .or_default() = counts
            .rejected_context_words
            .get(&key)
            .copied()
            .unwrap_or_default()
            .saturating_add(weight);
    }
    if record_transition {
        add_transition_counts(
            &mut counts.transition_repel,
            transition_context,
            source,
            operation,
            state_word,
            transition_target,
            transition_weight,
        );
    }
}

fn event_word_is_changed_target(event: &UsageEvent, word: &str) -> bool {
    let (Some(from), Some(to)) = (event.from.as_deref(), event.to.as_deref()) else {
        return true;
    };
    let from_words = normalized_words(from);
    let to_words = normalized_words(to);
    changed_target_indexes(&from_words, &to_words)
        .into_iter()
        .any(|index| to_words.get(index).is_some_and(|target| target == word))
}

fn event_transition_target(event: &UsageEvent, fallback_word: &str) -> String {
    let target = match (event.from.as_deref(), event.to.as_deref()) {
        (Some(from), Some(to)) => crate::typing_memory::transition_target_text(from, to),
        (_, Some(to)) => to.to_string(),
        _ => fallback_word.to_string(),
    };
    crate::transition_relation::transition_target_id(&target)
}

fn event_transition_context(event: &UsageEvent) -> Vec<String> {
    match (event.from.as_deref(), event.to.as_deref()) {
        (Some(from), Some(to)) => crate::typing_memory::transition_context_words(from, to),
        _ => event.context.clone(),
    }
}

fn event_transition_weight(event: &UsageEvent, weight: u32) -> u32 {
    let event_count = match (event.from.as_deref(), event.to.as_deref()) {
        (Some(from), Some(to)) => {
            let from_words = normalized_words(from);
            let to_words = normalized_words(to);
            changed_target_indexes(&from_words, &to_words).len()
        }
        (_, Some(to)) => normalized_words(to).len(),
        _ => 1,
    }
    .max(1) as u32;
    weight.saturating_add(event_count - 1) / event_count
}

fn event_state_word(event: &UsageEvent) -> String {
    event
        .from
        .as_deref()
        .map(crate::transition_relation::transition_state_id)
        .unwrap_or_else(|| TRANSITION_ANY.to_string())
}

fn rejected_prior_from_count(count: u32) -> f32 {
    ((count as f32 + 1.0).ln() * 0.040).clamp(0.0, 0.26)
}

fn transition_signal_from_counts_for_word(
    counts: &UsageCounts,
    context_keys: &[String],
    source: &str,
    operation: &str,
    state_word: &str,
    word: &str,
) -> UsageTransitionSignal {
    let exact_keys =
        transition_lookup_keys_from_context_keys(context_keys, source, operation, state_word, word);
    let (mut attract_count, mut repel_count) = transition_counts_for_keys(counts, &exact_keys);
    let state_specific = state_word != TRANSITION_ANY && (attract_count > 0 || repel_count > 0);
    if attract_count == 0 && repel_count == 0 && state_word != TRANSITION_ANY {
        let fallback_keys = transition_lookup_keys_from_context_keys(
            context_keys,
            source,
            operation,
            TRANSITION_ANY,
            word,
        );
        (attract_count, repel_count) = transition_counts_for_keys(counts, &fallback_keys);
    }
    let attraction = transition_attraction_from_count(attract_count);
    let repulsion = transition_repulsion_from_count(repel_count);
    let signed_weight = (attraction - repulsion).clamp(-1.0, 1.0);
    let reason = if repel_count > 0 && repulsion > attraction {
        "transition_repels"
    } else if attract_count > 0 && attraction > repulsion {
        "transition_attracts"
    } else if attract_count > 0 || repel_count > 0 {
        "transition_conflict"
    } else {
        "transition_empty"
    };
    UsageTransitionSignal {
        attraction,
        repulsion,
        signed_weight,
        attract_count,
        repel_count,
        state_specific,
        reason,
    }
}

fn transition_counts_for_keys(counts: &UsageCounts, keys: &[String]) -> (u32, u32) {
    let attract = keys
        .iter()
        .filter_map(|key| counts.transition_attract.get(key).copied())
        .max()
        .unwrap_or_default();
    let repel = keys
        .iter()
        .filter_map(|key| counts.transition_repel.get(key).copied())
        .max()
        .unwrap_or_default();
    (attract, repel)
}

fn transition_attraction_from_count(count: u32) -> f32 {
    if count == 0 {
        return 0.0;
    }
    ((count as f32 + 1.0).ln() * 0.050).clamp(0.0, 0.32)
}

fn transition_repulsion_from_count(count: u32) -> f32 {
    if count == 0 {
        return 0.0;
    }
    ((count as f32 + 1.0).ln() * 0.060).clamp(0.0, 0.38)
}

fn append_usage_event(event: UsageEvent) {
    let Some(path) = usage_events_path() else {
        return;
    };
    let _ = usage_counts();
    if adjacent_usage_event_is_duplicate(&path, &event) {
        return;
    }
    let Ok(mut line) = serde_json::to_string(&event) else {
        return;
    };
    line.push('\n');
    refresh_usage_cache_after_write(&event);
    enqueue_usage_persist(path, line);
}

fn refresh_usage_cache_after_write(event: &UsageEvent) {
    let Ok(mut cache) = usage_cache().lock() else {
        return;
    };
    if cache.loaded_at.is_none() {
        cache.counts = Arc::new(UsageCounts::default());
    }
    add_usage_event_count(Arc::make_mut(&mut cache.counts), event);
    cache.loaded_at = Some(Instant::now());
}

fn adjacent_usage_event_is_duplicate(path: &Path, event: &UsageEvent) -> bool {
    let last = LAST_USAGE_EVENT.get_or_init(|| Mutex::new(read_last_usage_event(path)));
    let Ok(mut last) = last.lock() else {
        return false;
    };
    if last
        .as_ref()
        .is_some_and(|previous| usage_event_payload_eq(previous, event))
    {
        return true;
    }
    *last = Some(event.clone());
    false
}

fn read_last_usage_event(path: &Path) -> Option<UsageEvent> {
    let text = read_usage_events_text(path)?;
    let line = text.lines().rev().find(|line| !line.trim().is_empty())?;
    serde_json::from_str(line).ok()
}

fn usage_event_payload_eq(left: &UsageEvent, right: &UsageEvent) -> bool {
    left.kind == right.kind
        && left.word == right.word
        && left.context == right.context
        && left.from == right.from
        && left.to == right.to
        && left.source == right.source
        && left.operation == right.operation
}

#[cfg(not(test))]
fn enqueue_usage_persist(path: PathBuf, line: String) {
    let sender = USAGE_PERSIST_SENDER.get_or_init(spawn_usage_persist_writer);
    let _ = sender.send(UsagePersistLine { path, line });
}

#[cfg(test)]
fn enqueue_usage_persist(path: PathBuf, line: String) {
    if crate::private_file::append_private_text(&path, &line).is_ok() {
        compact_usage_events_if_needed(&path);
    }
}

#[cfg(not(test))]
fn spawn_usage_persist_writer() -> Sender<UsagePersistLine> {
    let (sender, receiver) = mpsc::channel::<UsagePersistLine>();
    std::thread::Builder::new()
        .name("lay-usage-persist".to_string())
        .spawn(move || {
            let mut pending = HashMap::<PathBuf, String>::new();
            loop {
                match receiver.recv_timeout(USAGE_PERSIST_INTERVAL) {
                    Ok(record) => pending
                        .entry(record.path)
                        .or_default()
                        .push_str(&record.line),
                    Err(RecvTimeoutError::Timeout) => flush_usage_persist(&mut pending),
                    Err(RecvTimeoutError::Disconnected) => {
                        flush_usage_persist(&mut pending);
                        break;
                    }
                }
            }
        })
        .expect("spawn lay usage persistence writer");
    sender
}

#[cfg(not(test))]
fn flush_usage_persist(pending: &mut HashMap<PathBuf, String>) {
    for (path, text) in std::mem::take(pending) {
        if crate::private_file::append_private_text(&path, &text).is_err() {
            continue;
        }
        compact_usage_events_if_needed(&path);
        let _ = load_usage_event_counts();
    }
}

fn compact_usage_events_if_needed(path: &Path) {
    let Ok(meta) = std::fs::metadata(path) else {
        return;
    };
    if meta.len() <= USAGE_EVENTS_MAX_BYTES {
        return;
    }
    let Some(text) = read_usage_events_text(path) else {
        return;
    };
    let compacted = keep_jsonl_tail_bytes(&text, USAGE_EVENTS_MAX_BYTES as usize);
    let _ = crate::private_file::write_private_text(path, &compacted);
}

fn read_usage_events_text(path: &Path) -> Option<String> {
    read_tail_text_lossy(path, USAGE_EVENTS_MAX_BYTES as usize)
}

fn read_full_text_lossy(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

fn read_tail_text_lossy(path: &Path, max_bytes: usize) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let start = bytes.len().saturating_sub(max_bytes);
    let text = String::from_utf8_lossy(&bytes[start..]).into_owned();
    if start == 0 {
        return Some(text);
    }
    Some(
        text.find('\n')
            .map(|index| text[index + 1..].to_string())
            .unwrap_or(text),
    )
}

fn load_usage_counts_snapshot(source_len: u64) -> Option<UsageCounts> {
    let path = usage_counts_path()?;
    let text = std::fs::read_to_string(path).ok()?;
    let snapshot = serde_json::from_str::<PersistedUsageCounts>(&text).ok()?;
    (snapshot.schema_version == USAGE_COUNTS_SCHEMA_VERSION && snapshot.source_len == source_len)
        .then_some(snapshot.counts)
}

fn persist_usage_counts_snapshot(counts: &UsageCounts, source_len: u64) {
    let Some(path) = usage_counts_path() else {
        return;
    };
    let snapshot = PersistedUsageCounts {
        schema_version: USAGE_COUNTS_SCHEMA_VERSION,
        source_len,
        counts: compact_usage_counts_for_persist(counts),
    };
    let Ok(mut text) = serde_json::to_string(&snapshot) else {
        return;
    };
    text.push('\n');
    let _ = crate::private_file::write_private_text(&path, &text);
}

fn compact_usage_counts_for_persist(counts: &UsageCounts) -> UsageCounts {
    UsageCounts {
        words: top_count_entries(&counts.words, USAGE_COUNTS_MAX_WORDS),
        accepted_words: top_count_entries(&counts.accepted_words, USAGE_COUNTS_MAX_ACCEPTED_WORDS),
        context_words: top_count_entries(&counts.context_words, USAGE_COUNTS_MAX_CONTEXT_WORDS),
        rejected_words: top_count_entries(&counts.rejected_words, USAGE_COUNTS_MAX_REJECTED_WORDS),
        rejected_context_words: top_count_entries(
            &counts.rejected_context_words,
            USAGE_COUNTS_MAX_REJECTED_CONTEXT_WORDS,
        ),
        transition_observed: top_count_entries(
            &counts.transition_observed,
            USAGE_COUNTS_MAX_TRANSITION_STATES,
        ),
        transition_attract: top_count_entries(
            &counts.transition_attract,
            USAGE_COUNTS_MAX_TRANSITION_STATES,
        ),
        transition_repel: top_count_entries(
            &counts.transition_repel,
            USAGE_COUNTS_MAX_TRANSITION_STATES,
        ),
        surface_observed: top_count_entries(
            &counts.surface_observed,
            USAGE_COUNTS_MAX_TRANSITION_STATES,
        ),
        surface_attract: top_count_entries(
            &counts.surface_attract,
            USAGE_COUNTS_MAX_TRANSITION_STATES,
        ),
        surface_repel: top_count_entries(&counts.surface_repel, USAGE_COUNTS_MAX_TRANSITION_STATES),
    }
}

fn top_count_entries(source: &HashMap<String, u32>, limit: usize) -> HashMap<String, u32> {
    if source.len() <= limit {
        return source.clone();
    }
    let mut entries = source
        .iter()
        .map(|(key, count)| (key.clone(), *count))
        .collect::<Vec<_>>();
    entries.sort_by(|(left_key, left_count), (right_key, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_key.cmp(right_key))
    });
    entries.truncate(limit);
    entries.into_iter().collect()
}

fn keep_jsonl_tail_bytes(content: &str, max_bytes: usize) -> String {
    if content.len() <= max_bytes {
        return content.to_string();
    }
    let start = content.len().saturating_sub(max_bytes);
    let start = content
        .char_indices()
        .find_map(|(idx, _)| (idx >= start).then_some(idx))
        .unwrap_or(start);
    let start = content[..start]
        .rfind('\n')
        .map(|idx| idx + 1)
        .unwrap_or(start);
    content[start..].to_string()
}

fn context_ngram_keys(context: &[String]) -> Vec<String> {
    let normalized = context
        .iter()
        .filter_map(|word| {
            let word = normalize_memory_word(word);
            (!word.is_empty()).then_some(word)
        })
        .collect::<Vec<_>>();
    let max_len = normalized.len().min(CONTEXT_WORDS);
    (MIN_CONTEXT_NGRAM..=max_len)
        .filter_map(|len| {
            let start = normalized.len().saturating_sub(len);
            let key = normalized[start..].join(" ");
            (!key.is_empty()).then_some(key)
        })
        .collect()
}

fn context_word_key(context: &str, word: &str) -> String {
    format!("{context}\u{1f}{word}")
}

fn add_transition_counts(
    target: &mut HashMap<String, u32>,
    context: &[String],
    source: &str,
    operation: &str,
    state_word: &str,
    word: &str,
    weight: u32,
) {
    for key in transition_record_keys(context, source, operation, state_word, word) {
        *target.entry(key.clone()).or_default() = target
            .get(&key)
            .copied()
            .unwrap_or_default()
            .saturating_add(weight);
    }
}

fn transition_record_keys(
    context: &[String],
    source: &str,
    operation: &str,
    state_word: &str,
    word: &str,
) -> Vec<String> {
    let context_keys = context_ngram_keys(context);
    let mut keys = transition_lookup_keys_from_context_keys(
        &context_keys,
        source,
        operation,
        state_word,
        word,
    );
    if state_word != TRANSITION_ANY {
        keys.extend(transition_lookup_keys_from_context_keys(
            &context_keys,
            source,
            operation,
            TRANSITION_ANY,
            word,
        ));
    }
    keys.sort();
    keys.dedup();
    keys
}

fn transition_lookup_keys_from_context_keys(
    context_keys: &[String],
    source: &str,
    operation: &str,
    state_word: &str,
    word: &str,
) -> Vec<String> {
    let mut keys = Vec::new();
    let contexts = if context_keys.is_empty() {
        vec![String::new()]
    } else {
        context_keys.to_vec()
    };
    for context_key in contexts {
        keys.push(transition_key(
            &context_key,
            source,
            operation,
            state_word,
            word,
        ));
        keys.push(transition_key(
            &context_key,
            TRANSITION_ANY,
            operation,
            state_word,
            word,
        ));
        keys.push(transition_key(
            &context_key,
            TRANSITION_ANY,
            TRANSITION_ANY,
            state_word,
            word,
        ));
    }
    keys.sort();
    keys.dedup();
    keys
}

fn transition_key(
    context: &str,
    source: &str,
    operation: &str,
    state_word: &str,
    word: &str,
) -> String {
    format!("{context}\u{1e}{source}\u{1e}{operation}\u{1f}{state_word}\u{1d}{word}")
}

fn top_count_json(source: &HashMap<String, u32>, limit: usize) -> Vec<serde_json::Value> {
    let mut entries = source
        .iter()
        .map(|(key, count)| (key.as_str(), *count))
        .collect::<Vec<_>>();
    entries.sort_by(|(left_key, left_count), (right_key, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_key.cmp(right_key))
    });
    entries
        .into_iter()
        .take(limit)
        .map(|(key, count)| serde_json::json!({ "key": key, "count": count }))
        .collect()
}

fn event_source(event: &UsageEvent) -> &str {
    event.source.as_deref().unwrap_or(match event.kind {
        UsageEventKind::Typed => "user",
        UsageEventKind::AcceptedFix => "autocorrect",
        UsageEventKind::AcceptedIme | UsageEventKind::RejectedIme => "ime",
        UsageEventKind::RejectedCandidate => "candidate",
    })
}

fn event_operation(event: &UsageEvent) -> &str {
    event.operation.as_deref().unwrap_or(match event.kind {
        UsageEventKind::Typed => "typed",
        UsageEventKind::AcceptedFix => "replacement",
        UsageEventKind::AcceptedIme | UsageEventKind::RejectedIme => "completion",
        UsageEventKind::RejectedCandidate => "candidate",
    })
}

fn usage_events_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("LAY_NANDA_WORD_USAGE_EVENTS").map(PathBuf::from) {
        return Some(path);
    }
    #[cfg(test)]
    {
        None
    }
    #[cfg(not(test))]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(USAGE_EVENTS_PATH))
    }
}

fn usage_counts_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("LAY_NANDA_WORD_USAGE_COUNTS").map(PathBuf::from) {
        return Some(path);
    }
    #[cfg(test)]
    {
        None
    }
    #[cfg(not(test))]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(USAGE_COUNTS_PATH))
    }
}

fn legacy_usage_prior_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("LAY_NANDA_USAGE_PRIOR").map(PathBuf::from) {
        return Some(path);
    }
    #[cfg(test)]
    {
        None
    }
    #[cfg(not(test))]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(LEGACY_USAGE_PRIOR_PATH))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_hot_readout_initializes_persisted_usage_memory_once() {
        let mut cache = UsageCache::default();
        let mut loads = 0;
        ensure_usage_cache_initialized(&mut cache, || {
            loads += 1;
            let mut counts = UsageCounts::default();
            counts.rejected_words.insert("ошибка".to_string(), 8);
            counts
        });
        ensure_usage_cache_initialized(&mut cache, || {
            panic!("an initialized hot cache must not reload on every readout")
        });

        assert_eq!(loads, 1);
        assert_eq!(cache.counts.rejected_words.get("ошибка"), Some(&8));
        assert!(cache.loaded_at.is_some());
    }

    #[test]
    fn legacy_usage_prior_counts_target_words() {
        let counts = legacy_usage_counts_from_json(
            r#"{
                "а\u001fпроверка": {"to":"проверка","count":2,"promoted":false},
                "б\u001fпроверка": {"to":"проверка","count":1,"promoted":true},
                "в\u001f": {"to":"","count":99,"promoted":true}
            }"#,
        );

        assert_eq!(counts.get("проверка"), Some(&6));
        assert!(!counts.contains_key(""));
    }

    #[test]
    fn usage_events_count_typed_fix_and_ime_words() {
        let text = r#"{"ts":1,"kind":"typed","word":"дождь","context":["на","улице","идёт"]}
{"ts":2,"kind":"accepted_fix","word":"дождь","from":"дожть","to":"дождь"}
{"ts":3,"kind":"accepted_ime","word":"дождь","context":["на","улице","идёт"],"to":"дождь"}
"#;
        let mut counts = UsageCounts::default();
        add_usage_event_counts(&mut counts, text);

        assert_eq!(counts.words.get("дождь"), Some(&12));
        assert_eq!(
            counts
                .context_words
                .get("на улице идёт\u{1f}дождь")
                .copied(),
            Some(6)
        );
        assert_eq!(
            counts.context_words.get("идёт\u{1f}дождь").copied(),
            Some(6)
        );
        assert_eq!(
            counts.context_words.get("улице идёт\u{1f}дождь").copied(),
            Some(6)
        );
        assert_eq!(counts.rejected_words.get("дожть"), Some(&6));
    }

    #[test]
    fn accepted_fix_creates_negative_trace_for_corrected_away_word_only() {
        let text = r#"{"ts":1,"kind":"accepted_fix","word":"отравим","context":["мы"],"from":"мы отвравим","to":"мы отравим"}
"#;
        let mut counts = UsageCounts::default();
        add_usage_event_counts(&mut counts, text);

        assert_eq!(counts.accepted_words.get("отравим"), Some(&6));
        assert_eq!(counts.rejected_words.get("отвравим"), Some(&6));
        assert!(!counts.rejected_words.contains_key("мы"));
        assert_eq!(
            counts
                .rejected_context_words
                .get("мы\u{1f}отвравим")
                .copied(),
            Some(6)
        );
    }

    #[test]
    fn state_map_summary_counts_signed_word_states() {
        let text = r#"{"ts":1,"kind":"accepted_fix","word":"отравим","context":["мы"],"from":"мы отвравим","to":"мы отравим"}
{"ts":2,"kind":"accepted_ime","word":"дождь","context":["идёт"],"to":"дождь"}
"#;
        let mut counts = UsageCounts::default();
        add_usage_event_counts(&mut counts, text);

        let signed_word_states = counts
            .accepted_words
            .keys()
            .chain(counts.rejected_words.keys())
            .collect::<HashSet<_>>()
            .len();

        assert_eq!(counts.accepted_words.len(), 2);
        assert_eq!(counts.rejected_words.len(), 1);
        assert_eq!(signed_word_states, 3);
        assert!(!counts.transition_attract.is_empty());
        assert!(counts.transition_repel.is_empty());
    }

    #[test]
    fn transition_signal_attracts_accepted_completion() {
        let usage = snapshot_from_usage_events_for_tests(
            r#"{"ts":1,"kind":"accepted_ime","word":"дождь","context":["на","улице","идёт"],"to":"дождь","source":"ime","operation":"completion"}
"#,
        );
        let context = ["на", "улице", "идёт"].map(String::from);
        let signal = usage
            .hot_readout(
                &context,
                "L2LiveCandidateGate32",
                "completion",
                "д",
                "дождь",
            )
            .transition;

        assert!(signal.attraction > signal.repulsion);
        assert!(signal.signed_weight > 0.0);
        assert_eq!(signal.reason, "transition_attracts");
    }

    #[test]
    fn transition_signal_attracts_accepted_state_change() {
        let usage = snapshot_from_usage_events_for_tests(
            r#"{"ts":1,"kind":"accepted_fix","word":"отравим","context":["мы"],"from":"мы отвравим","to":"мы отравим","source":"autocorrect","operation":"replacement"}
"#,
        );
        let context = ["мы"].map(String::from);
        let signal = usage
            .hot_readout(
                &context,
                "SemanticWordCell32",
                "replacement",
                "отвравим",
                "отравим",
            )
            .transition;

        assert!(signal.attraction > signal.repulsion);
        assert!(signal.signed_weight > 0.0);
        assert_eq!(signal.reason, "transition_attracts");
    }

    #[test]
    fn context_ngram_keys_cover_recent_suffixes() {
        let context = ["на", "улице", "опять", "идёт"].map(String::from);

        assert_eq!(
            context_ngram_keys(&context),
            [
                "идёт",
                "опять идёт",
                "улице опять идёт",
                "на улице опять идёт"
            ]
        );
    }

    #[test]
    fn context_ngram_prior_scores_partial_context_match() {
        let mut counts = UsageCounts::default();
        add_usage_event_counts(
            &mut counts,
            r#"{"ts":1,"kind":"accepted_ime","word":"дождь","context":["на","улице","опять","идёт"],"to":"дождь"}
"#,
        );

        let close_context = ["вечером", "опять", "идёт"].map(String::from);
        let far_context = ["в", "другом", "месте"].map(String::from);

        let close_score = context_ngram_prior_from_counts(&counts, &close_context, "дождь");
        let far_score = context_ngram_prior_from_counts(&counts, &far_context, "дождь");

        assert!(close_score > 0.0);
        assert_eq!(far_score, 0.0);
        assert!(close_score > far_score);
    }

    #[test]
    fn usage_surface_words_promote_repeated_local_words() {
        let mut counts = UsageCounts::default();
        add_usage_event_counts(
            &mut counts,
            r#"{"ts":1,"kind":"typed","word":"комитет"}
{"ts":2,"kind":"accepted_fix","word":"комитет","from":"коммит","to":"комитет"}
{"ts":3,"kind":"typed","word":"x"}
"#,
        );

        let words = usage_surface_words_from_counts(counts);

        assert_eq!(words.first().map(String::as_str), Some("комитет"));
        assert!(!words.iter().any(|word| word == "x"));
    }

    #[test]
    fn usage_surface_words_promote_accepted_ime_word_into_hot_set() {
        let mut counts = UsageCounts::default();
        add_usage_event_counts(
            &mut counts,
            r#"{"ts":1,"kind":"accepted_ime","word":"архитектура","context":["новая"],"to":"архитектура"}
"#,
        );

        let words = usage_surface_words_from_counts(counts);

        assert_eq!(words.first().map(String::as_str), Some("архитектура"));
    }

    #[test]
    fn rejected_ime_creates_negative_trace_without_promoting_word() {
        let text = r#"{"ts":1,"kind":"rejected_ime","word":"даша","context":["ну"],"to":"даша","source":"ime","operation":"completion"}
"#;
        let mut counts = UsageCounts::default();
        add_usage_event_counts(&mut counts, text);

        assert!(!counts.words.contains_key("даша"));
        assert!(!counts.accepted_words.contains_key("даша"));
        assert_eq!(counts.rejected_words.get("даша"), Some(&8));
        assert_eq!(
            counts.rejected_context_words.get("ну\u{1f}даша").copied(),
            Some(8)
        );

        let usage = snapshot_from_usage_events_for_tests(text);
        let context = ["ну"].map(String::from);
        let signal = usage
            .hot_readout(&context, "L2LiveCandidateGate32", "completion", "*", "даша")
            .transition;
        assert!(signal.repulsion > signal.attraction);
    }

    #[test]
    fn typing_memory_event_routes_accepted_fix_into_usage_counts() {
        let events = TypingMemoryEvent::accepted_fix("мы отвравим", "мы отравим");
        let mut counts = UsageCounts::default();
        for event in events {
            add_usage_event_count(&mut counts, &UsageEvent::from_typing_memory_event(&event));
        }

        assert_eq!(counts.accepted_words.get("отравим"), Some(&6));
        assert_eq!(counts.rejected_words.get("отвравим"), Some(&6));
        assert!(!counts.transition_attract.is_empty());
        assert!(counts.transition_repel.is_empty());
    }

    #[test]
    fn typing_memory_event_routes_rejected_candidate_into_l4_repulsion() {
        let events = TypingMemoryEvent::rejected_candidate(
            "ну исходник",
            "ну даша",
            "L2LiveCandidateGate32",
            "completion",
        );
        let mut counts = UsageCounts::default();
        for event in events {
            add_usage_event_count(&mut counts, &UsageEvent::from_typing_memory_event(&event));
        }

        assert!(!counts.words.contains_key("даша"));
        assert_eq!(counts.rejected_words.get("даша"), Some(&8));

        let usage = UsagePriorSnapshot {
            counts: Arc::new(counts),
        };
        let context = ["ну"].map(String::from);
        let signal = usage
            .hot_readout(
                &context,
                "L2LiveCandidateGate32",
                "completion",
                &crate::transition_relation::transition_state_id("ну исходник"),
                "даша",
            )
            .transition;
        assert!(signal.repulsion > signal.attraction);
        assert_eq!(signal.reason, "transition_repels");
    }

    #[test]
    fn hot_readout_collects_usage_and_rejection_in_one_pass() {
        let usage = snapshot_from_usage_events_for_tests(
            r#"{"ts":1,"kind":"accepted_fix","word":"проверить","context":["можно"],"from":"можно проврить","to":"можно проверить","source":"autocorrect","operation":"replacement"}
{"ts":2,"kind":"rejected_candidate","word":"проврить","context":["можно"],"to":"проврить","source":"autocorrect","operation":"auto_undo"}
"#,
        );
        let context = ["можно".to_string()];

        let good = usage.hot_readout(
            &context,
            "autocorrect",
            "replacement",
            "проврить",
            "проверить",
        );
        let prepared = usage.prepare_hot_context(&context);
        let prepared_good = usage.hot_readout_prepared(
            &prepared,
            "autocorrect",
            "replacement",
            "проврить",
            "проверить",
        );
        let bad = usage.hot_readout(&context, "autocorrect", "auto_undo", "*", "проврить");

        assert_eq!(prepared_good, good);
        assert!(good.accepted_count > 0);
        assert!(good.transition.attraction > good.transition.repulsion);
        assert!(bad.rejected_count > 0);
        assert!(bad.transition.repulsion > bad.transition.attraction);
    }

    #[test]
    fn exact_state_transition_overrides_global_target_frequency() {
        let usage = snapshot_from_usage_events_for_tests(
            r#"{"ts":1,"kind":"accepted_fix","word":"так","from":"nfr","to":"так","source":"autocorrect","operation":"replacement"}
{"ts":2,"kind":"accepted_fix","word":"так","from":"другой","to":"так","source":"autocorrect","operation":"replacement"}
{"ts":3,"kind":"rejected_candidate","word":"так","from":"nfr","to":"так","source":"user_correction","operation":"replacement"}
"#,
        );

        let rejected_state = crate::transition_relation::transition_state_id("nfr");
        let fallback_state = crate::transition_relation::transition_state_id("новый");
        let rejected = usage.hot_readout(&[], "layout", "replacement", &rejected_state, "так");
        let fallback = usage.hot_readout(&[], "layout", "replacement", &fallback_state, "так");

        assert!(rejected.transition.repulsion > rejected.transition.attraction);
        assert!(rejected.transition.state_specific);
        assert!(fallback.transition.attraction > 0.0);
        assert!(!fallback.transition.state_specific);
    }

    #[test]
    fn context_rejection_does_not_leak_into_empty_context_transition() {
        let usage = snapshot_from_usage_events_for_tests(
            r#"{"ts":1,"kind":"accepted_fix","word":"проверь","from":"ghjdthm","to":"проверь","source":"manual_layout_replay","operation":"layout"}
{"ts":2,"kind":"rejected_candidate","word":"проверь","context":["gfzvnm"],"from":"gfzvnm ghjdthm","to":"gfzvnm проверь","source":"typing-assist","operation":"mixed_layout"}
"#,
        );
        let state = crate::transition_relation::transition_state_id("ghjdthm");

        let empty = usage.hot_readout(&[], "layout", "layout", &state, "проверь");
        let contextual = usage.hot_readout(
            &["gfzvnm".to_string()],
            "layout",
            "mixed_layout",
            &state,
            "проверь",
        );

        assert!(empty.transition.attraction > empty.transition.repulsion);
        assert!(contextual.transition.repulsion > contextual.transition.attraction);
    }

    #[test]
    fn adjacent_duplicate_usage_events_ignore_timestamp() {
        let first = UsageEvent {
            ts: 1,
            kind: UsageEventKind::Typed,
            word: Some("лог".to_string()),
            context: vec!["смотри".to_string()],
            from: None,
            to: None,
            source: Some("user".to_string()),
            operation: Some("typed".to_string()),
            surface: None,
        };
        let second = UsageEvent {
            ts: 2,
            ..first.clone()
        };

        assert!(usage_event_payload_eq(&first, &second));
    }

    #[test]
    fn context_and_last_word_uses_recent_context() {
        let (context, word) =
            typing_memory::context_and_last_word("на улице опять идёт дождь ").unwrap();

        assert_eq!(word, "дождь");
        assert_eq!(context, ["на", "улице", "опять", "идёт"]);
    }

    #[test]
    fn tail_compaction_keeps_recent_complete_lines() {
        let text = "one\ntwo\nthree\nfour\n";
        let compacted = keep_jsonl_tail_bytes(text, 11);

        assert_eq!(compacted, "three\nfour\n");
    }
}
