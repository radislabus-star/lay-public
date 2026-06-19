//! Rolling correction-action journal for tray diagnostics.
//!
//! This is not a keylog: only successful lay actions are recorded, and the file
//! is capped to a small number of lines so it cannot grow without bound.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const ACTIONS_PATH: &str = ".local/share/lay/recent_actions.jsonl";
const KEEP_LINES: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RecentAction<'a> {
    pub ts: u64,
    pub kind: &'a str,
    pub from: &'a str,
    pub to: &'a str,
    pub replace_words: usize,
    pub words: usize,
    pub elapsed_ms: u128,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_ms: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_ms: Option<u128>,
    pub undo_available: bool,
}

pub fn record_action(
    kind: &str,
    from: &str,
    to: &str,
    replace_words: usize,
    words: usize,
    elapsed_ms: u128,
    undo_available: bool,
) {
    record_action_with_stages(
        kind,
        from,
        to,
        replace_words,
        words,
        elapsed_ms,
        None,
        None,
        undo_available,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn record_action_with_stages(
    kind: &str,
    from: &str,
    to: &str,
    replace_words: usize,
    words: usize,
    elapsed_ms: u128,
    decision_ms: Option<u128>,
    output_ms: Option<u128>,
    undo_available: bool,
) {
    if from == to || from.trim().is_empty() || to.trim().is_empty() {
        return;
    }
    if !crate::config::LayConfig::load().debug_action_log {
        return;
    }
    let Some(path) = actions_path() else {
        return;
    };
    let action = RecentAction {
        ts: unix_timestamp(),
        kind,
        from,
        to,
        replace_words,
        words,
        elapsed_ms,
        decision_ms,
        output_ms,
        undo_available,
    };
    record_action_to_path(&path, &action, KEEP_LINES);
}

pub fn record_action_to_path(path: &Path, action: &RecentAction<'_>, keep_lines: usize) {
    let Ok(mut line) = serde_json::to_string(action) else {
        return;
    };
    line.push('\n');
    if crate::private_file::append_private_text(path, &line).is_ok() {
        compact_action_log(path, keep_lines);
    }
}

fn compact_action_log(path: &Path, keep_lines: usize) {
    if keep_lines == 0 {
        let _ = std::fs::remove_file(path);
        return;
    }
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    let lines: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    if lines.len() <= keep_lines {
        return;
    }
    let start = lines.len() - keep_lines;
    let compacted = format!("{}\n", lines[start..].join("\n"));
    let _ = crate::private_file::write_private_text(path, &compacted);
}

fn actions_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(ACTIONS_PATH))
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
#[path = "action_log_tests.rs"]
mod tests;
