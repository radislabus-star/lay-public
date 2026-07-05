//! Small privacy-safe runtime counters for the tray UI.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::time::unix_timestamp;

const STATS_PATH: &str = ".local/share/lay/stats.json";

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LayStats {
    pub llm_calls: u64,
    pub learning_log_entries: u64,
    pub user_corrections: u64,
    pub promoted_rules: u64,
    pub last_llm_ts: u64,
    pub last_learning_ts: u64,
    pub last_promotion_ts: u64,
}

pub fn record_llm_call() {
    update(|stats, now| {
        stats.llm_calls = stats.llm_calls.saturating_add(1);
        stats.last_llm_ts = now;
    });
}

pub fn record_learning_log_entry(kind: &str) {
    update(|stats, now| {
        stats.learning_log_entries = stats.learning_log_entries.saturating_add(1);
        stats.last_learning_ts = now;
        if kind == "user-correction" {
            stats.user_corrections = stats.user_corrections.saturating_add(1);
        }
    });
}

pub fn record_learning_promotion() {
    update(|stats, now| {
        stats.promoted_rules = stats.promoted_rules.saturating_add(1);
        stats.last_promotion_ts = now;
    });
}

fn update(mut apply: impl FnMut(&mut LayStats, u64)) {
    let Some(path) = stats_path() else {
        return;
    };
    let mut stats = load(&path);
    apply(&mut stats, unix_timestamp());
    let _ = write_private_stats(&path, &stats);
}

fn load(path: &std::path::Path) -> LayStats {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn stats_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(STATS_PATH))
}

fn write_private_stats(path: &std::path::Path, stats: &LayStats) -> std::io::Result<()> {
    let text = serde_json::to_string_pretty(stats)?;
    crate::private_file::write_private_text(path, &format!("{text}\n"))
}

#[cfg(test)]
#[path = "stats_tests.rs"]
mod tests;
