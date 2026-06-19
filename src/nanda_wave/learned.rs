use std::collections::BTreeMap;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use super::packet::{read_learned_packet, LearnedPacketEntry};
use super::signal::WordCandidate;

static MEMORY: OnceLock<Mutex<LearnedMemoryCache>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq)]
struct LearnedEntry {
    expected: String,
    operation: String,
    count: usize,
}

#[derive(Debug, Default)]
struct LearnedMemoryCache {
    path: Option<PathBuf>,
    modified: Option<SystemTime>,
    entries: BTreeMap<String, LearnedEntry>,
}

pub fn learned_candidates(original: &str) -> Vec<WordCandidate> {
    let tail = original.trim_end();
    let Some(entry) = memory_entry(tail) else {
        return Vec::new();
    };
    if entry.expected == tail {
        return Vec::new();
    }
    let source = learned_source(tail, &entry);
    vec![WordCandidate {
        text: entry.expected.clone(),
        source,
        energy: learned_energy(&entry, source),
        risk: learned_risk(&entry, source),
        support: learned_support(tail, &entry, source),
    }]
}

pub fn learned_candidate(original: &str) -> Option<WordCandidate> {
    learned_candidates(original).into_iter().next()
}

pub fn default_memory_path() -> PathBuf {
    env::var_os("LAY_NANDA_WAVE_MEMORY")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".local/share/lay/nanda_wave/learned_memory.cell32")
        })
}

fn memory_entry(tail: &str) -> Option<LearnedEntry> {
    let path = default_memory_path();
    let mut cache = MEMORY
        .get_or_init(|| Mutex::new(LearnedMemoryCache::default()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    refresh_cache(&mut cache, path);
    cache.entries.get(tail).cloned()
}

pub(crate) fn loaded_memory_entries(path: &Path) -> Vec<LearnedPacketEntry> {
    read_learned_packet(path).unwrap_or_default()
}

fn refresh_cache(cache: &mut LearnedMemoryCache, path: PathBuf) {
    let modified = std::fs::metadata(&path)
        .and_then(|metadata| metadata.modified())
        .ok();
    if cache.path.as_ref() == Some(&path) && cache.modified == modified {
        return;
    }
    cache.path = Some(path.clone());
    cache.modified = modified;
    cache.entries = load_memory(&path).unwrap_or_default();
}

fn load_memory(path: &Path) -> Option<BTreeMap<String, LearnedEntry>> {
    let mut map = BTreeMap::new();
    for entry in loaded_memory_entries(path) {
        map.insert(
            entry.original,
            LearnedEntry {
                expected: entry.expected,
                operation: entry.operation,
                count: entry.count,
            },
        );
    }
    Some(map)
}

fn learned_source(tail: &str, entry: &LearnedEntry) -> &'static str {
    let token_count = tail
        .split_whitespace()
        .count()
        .max(entry.expected.split_whitespace().count());
    match entry.operation.as_str() {
        "split" if token_count >= 2 => "PhraseMemoryCell32",
        "typo" if token_count <= 1 => "CommonRuFixCell32",
        "layout" => "LearnedMemoryCell32",
        _ => "UserMemoryCell32",
    }
}

fn learned_energy(entry: &LearnedEntry, source: &str) -> f32 {
    let base: f32 = match source {
        "LearnedMemoryCell32" => 0.84,
        "CommonRuFixCell32" => 0.80,
        "PhraseMemoryCell32" => 0.82,
        "UserMemoryCell32" => 0.86,
        _ => 0.76,
    };
    (base + (entry.count.min(8) as f32 * 0.015)).min(0.94)
}

fn learned_risk(entry: &LearnedEntry, source: &str) -> f32 {
    match (source, entry.operation.as_str()) {
        ("LearnedMemoryCell32", "layout") => 0.08,
        ("PhraseMemoryCell32", "split") => 0.11,
        ("CommonRuFixCell32", "typo") => 0.15,
        ("UserMemoryCell32", _) => 0.12,
        (_, "layout") => 0.10,
        (_, "split") => 0.14,
        (_, "typo") => 0.18,
        _ => 0.22,
    }
}

fn learned_support(tail: &str, entry: &LearnedEntry, source: &str) -> Vec<String> {
    vec![
        "learned-memory".to_string(),
        format!("source={source}"),
        format!("operation={}", entry.operation),
        format!("count={}", entry.count),
        format!("tokens={}", tail.split_whitespace().count()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learned_energy_stays_bounded() {
        let entry = LearnedEntry {
            expected: "вот".to_string(),
            operation: "layout".to_string(),
            count: 99,
        };
        assert!(learned_energy(&entry, "LearnedMemoryCell32") <= 0.94);
        assert!(
            learned_risk(&entry, "LearnedMemoryCell32")
                < learned_energy(&entry, "LearnedMemoryCell32")
        );
    }
}
