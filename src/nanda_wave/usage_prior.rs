use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::time::unix_timestamp;

#[cfg(not(test))]
const USAGE_EVENTS_PATH: &str = ".local/share/lay/nanda_wave/word_usage_events.jsonl";
#[cfg(not(test))]
const USAGE_COUNTS_PATH: &str = ".local/share/lay/nanda_wave/word_usage_counts.json";
#[cfg(not(test))]
const LEGACY_USAGE_PRIOR_PATH: &str = ".local/share/lay/learning_candidates.json";
const USAGE_EVENTS_MAX_BYTES: u64 = 500 * 1024;
const USAGE_EVENTS_FULL_REBUILD_MAX_BYTES: u64 = 8 * 1024 * 1024;
const USAGE_COUNTS_SCHEMA_VERSION: u32 = 3;
const USAGE_COUNTS_MAX_WORDS: usize = 10_000;
const USAGE_COUNTS_MAX_ACCEPTED_WORDS: usize = 5_000;
const USAGE_COUNTS_MAX_CONTEXT_WORDS: usize = 12_000;
const USAGE_REFRESH_INTERVAL: Duration = Duration::from_millis(1000);
const CONTEXT_WORDS: usize = 5;
const MIN_CONTEXT_NGRAM: usize = 1;

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
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum UsageEventKind {
    #[default]
    Typed,
    AcceptedFix,
    AcceptedIme,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
struct UsageCounts {
    words: HashMap<String, u32>,
    #[serde(default)]
    accepted_words: HashMap<String, u32>,
    context_words: HashMap<String, u32>,
}

#[derive(Debug, Clone, Default)]
pub struct UsagePriorSnapshot {
    counts: UsageCounts,
}

impl UsagePriorSnapshot {
    pub fn word_prior(&self, word: &str) -> f32 {
        let lower = normalize_word(word);
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
        let lower = normalize_word(word);
        if lower.is_empty() || context.is_empty() {
            return 0.0;
        }
        context_ngram_prior_from_counts(&self.counts, context, &lower)
    }

    pub fn accepted_word_count(&self, word: &str) -> u32 {
        let lower = normalize_word(word);
        if lower.is_empty() {
            return 0;
        }
        self.counts
            .accepted_words
            .get(&lower)
            .copied()
            .unwrap_or_default()
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
    counts: UsageCounts,
}

pub(crate) fn record_typed_tail_if_enabled(tail: &str) {
    if !usage_learning_enabled() {
        return;
    }
    let Some((context, word)) = context_and_last_word(tail) else {
        return;
    };
    append_usage_event(UsageEvent {
        ts: unix_timestamp(),
        kind: UsageEventKind::Typed,
        word: Some(word),
        context,
        from: None,
        to: None,
    });
}

pub(crate) fn record_accepted_fix_if_enabled(from: &str, to: &str) {
    if !usage_learning_enabled() || from == to {
        return;
    }
    let to_words = normalized_words(to);
    if to_words.is_empty() {
        return;
    }
    let context = to_words
        .iter()
        .rev()
        .skip(1)
        .take(CONTEXT_WORDS)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>();
    for word in to_words {
        append_usage_event(UsageEvent {
            ts: unix_timestamp(),
            kind: UsageEventKind::AcceptedFix,
            word: Some(word),
            context: context.clone(),
            from: Some(from.trim().to_string()),
            to: Some(to.trim().to_string()),
        });
    }
}

pub(crate) fn record_accepted_ime_if_enabled(context_tail: &str, accepted_text: &str) {
    if !usage_learning_enabled() {
        return;
    }
    let accepted_words = normalized_words(accepted_text);
    if accepted_words.is_empty() {
        return;
    }
    let context = normalized_words(context_tail)
        .into_iter()
        .rev()
        .take(CONTEXT_WORDS)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>();
    for word in accepted_words {
        append_usage_event(UsageEvent {
            ts: unix_timestamp(),
            kind: UsageEventKind::AcceptedIme,
            word: Some(word),
            context: context.clone(),
            from: None,
            to: Some(accepted_text.trim().to_string()),
        });
    }
}

pub(crate) fn word_usage_prior(word: &str) -> f32 {
    let lower = normalize_word(word);
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
    let lower = normalize_word(word);
    if lower.is_empty() || context.is_empty() {
        return 0.0;
    }
    let counts = usage_counts();
    context_ngram_prior_from_counts(&counts, context, &lower)
}

fn word_prior_from_count(count: u32) -> f32 {
    ((count as f32 + 1.0).ln() * 0.026).clamp(0.0, 0.14)
}

fn usage_learning_enabled() -> bool {
    let config = crate::config::LayConfig::load();
    config.learning_log || config.nanda_precognition || config.nanda_autocorrect
}

fn usage_counts() -> UsageCounts {
    let cache = usage_cache();
    let Ok(mut cache) = cache.lock() else {
        return UsageCounts::default();
    };
    if cache
        .loaded_at
        .is_some_and(|loaded_at| loaded_at.elapsed() < USAGE_REFRESH_INTERVAL)
    {
        return cache.counts.clone();
    }
    cache.counts = load_usage_counts();
    cache.loaded_at = Some(Instant::now());
    cache.counts.clone()
}

pub(crate) fn word_usage_prior_cached(word: &str) -> f32 {
    let lower = normalize_word(word);
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
    let lower = normalize_word(word);
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
    let lower = normalize_word(word);
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
    let text = usage_events_path()
        .and_then(|path| read_usage_events_text(&path))
        .unwrap_or_default();
    let bytes = text.len() as u64;
    let parsed = usage_events_from_jsonl(&text).count();
    let counts = load_usage_counts();
    (bytes, parsed, counts.words.len())
}

fn refresh_usage_counts_from_disk() -> UsageCounts {
    let counts = load_usage_counts();
    if let Ok(mut cache) = usage_cache().lock() {
        cache.counts = counts.clone();
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
    context_ngram_keys(context)
        .into_iter()
        .filter_map(|context_key| {
            let ngram_len = context_key.split_whitespace().count();
            let key = context_word_key(&context_key, word);
            counts
                .context_words
                .get(&key)
                .copied()
                .map(|count| (count, ngram_len))
        })
        .map(|(count, ngram_len)| {
            let ngram_weight = 0.014 + ngram_len as f32 * 0.006;
            ((count as f32 + 1.0).ln() * ngram_weight).min(0.11)
        })
        .sum::<f32>()
        .clamp(0.0, 0.24)
}

fn cached_usage_counts() -> UsageCounts {
    let Ok(cache) = usage_cache().lock() else {
        return UsageCounts::default();
    };
    cache.counts.clone()
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
    let Some(word) = event.word.as_deref().map(normalize_word) else {
        return;
    };
    if word.is_empty() {
        return;
    }
    let weight = match event.kind {
        UsageEventKind::Typed => 1,
        UsageEventKind::AcceptedFix => 3,
        UsageEventKind::AcceptedIme => 2,
    };
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
    if crate::private_file::append_private_text(&path, &line).is_ok() {
        compact_usage_events_if_needed(&path);
        refresh_usage_cache_after_write(&event);
        persist_cached_usage_counts_snapshot(&path);
    }
}

fn refresh_usage_cache_after_write(event: &UsageEvent) {
    let Ok(mut cache) = usage_cache().lock() else {
        return;
    };
    if cache.loaded_at.is_none() {
        cache.counts = UsageCounts::default();
    }
    add_usage_event_count(&mut cache.counts, event);
    cache.loaded_at = Some(Instant::now());
}

fn adjacent_usage_event_is_duplicate(path: &Path, event: &UsageEvent) -> bool {
    let Some(text) = read_usage_events_text(path) else {
        return false;
    };
    let Some(line) = text.lines().rev().find(|line| !line.trim().is_empty()) else {
        return false;
    };
    let Ok(previous) = serde_json::from_str::<UsageEvent>(line) else {
        return false;
    };
    usage_event_payload_eq(&previous, event)
}

fn usage_event_payload_eq(left: &UsageEvent, right: &UsageEvent) -> bool {
    left.kind == right.kind
        && left.word == right.word
        && left.context == right.context
        && left.from == right.from
        && left.to == right.to
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

fn persist_cached_usage_counts_snapshot(events_path: &Path) {
    let source_len = std::fs::metadata(events_path)
        .map(|meta| meta.len())
        .unwrap_or_default();
    let Ok(cache) = usage_cache().lock() else {
        return;
    };
    persist_usage_counts_snapshot(&cache.counts, source_len);
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

fn context_and_last_word(text: &str) -> Option<(Vec<String>, String)> {
    let words = normalized_words(text);
    let (word, context) = words.split_last()?;
    let context = context
        .iter()
        .rev()
        .take(CONTEXT_WORDS)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>();
    Some((context, word.clone()))
}

fn normalized_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|token| {
            let word = normalize_word(token);
            (!word.is_empty()).then_some(word)
        })
        .collect()
}

fn normalize_word(word: &str) -> String {
    let trimmed = word
        .trim()
        .trim_matches(|ch: char| !ch.is_alphabetic() && ch != '-');
    if trimmed.chars().filter(|ch| ch.is_alphabetic()).count() < 2 {
        return String::new();
    }
    trimmed.to_lowercase()
}

fn context_ngram_keys(context: &[String]) -> Vec<String> {
    let normalized = context
        .iter()
        .filter_map(|word| {
            let word = normalize_word(word);
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

        assert_eq!(counts.words.get("дождь"), Some(&6));
        assert_eq!(
            counts
                .context_words
                .get("на улице идёт\u{1f}дождь")
                .copied(),
            Some(3)
        );
        assert_eq!(
            counts.context_words.get("идёт\u{1f}дождь").copied(),
            Some(3)
        );
        assert_eq!(
            counts.context_words.get("улице идёт\u{1f}дождь").copied(),
            Some(3)
        );
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
    fn adjacent_duplicate_usage_events_ignore_timestamp() {
        let first = UsageEvent {
            ts: 1,
            kind: UsageEventKind::Typed,
            word: Some("лог".to_string()),
            context: vec!["смотри".to_string()],
            from: None,
            to: None,
        };
        let second = UsageEvent {
            ts: 2,
            ..first.clone()
        };

        assert!(usage_event_payload_eq(&first, &second));
    }

    #[test]
    fn context_and_last_word_uses_recent_context() {
        let (context, word) = context_and_last_word("на улице опять идёт дождь ").unwrap();

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
