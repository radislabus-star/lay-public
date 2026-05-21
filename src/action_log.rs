//! Rolling correction-action journal for tray diagnostics.
//!
//! This is not a keylog: only successful lay actions are recorded, and the file
//! is capped to a small number of lines so it cannot grow without bound.

use serde::{Deserialize, Serialize};
use std::io::Write;
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
    if from == to || from.trim().is_empty() || to.trim().is_empty() {
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
        undo_available,
    };
    record_action_to_path(&path, &action, KEEP_LINES);
}

pub fn record_action_to_path(path: &Path, action: &RecentAction<'_>, keep_lines: usize) {
    if let Some(parent) = path.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return;
        }
    }
    let Ok(mut line) = serde_json::to_string(action) else {
        return;
    };
    line.push('\n');
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
    {
        if file.write_all(line.as_bytes()).is_ok() {
            compact_action_log(path, keep_lines);
        }
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
    let _ = std::fs::write(path, compacted);
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
