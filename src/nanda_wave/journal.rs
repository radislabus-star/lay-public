use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::journal_record::build_trace_record;
use super::signal::WaveTrace;

pub use super::journal_record::{
    CellTraceCandidate, CellTraceCell, CellTraceMode, CellTracePattern, CellTraceRecord,
};

const TRACE_PATH: &str = ".local/share/lay/nanda_wave/cell_trace.jsonl";
const SCOREBOARD_PATH: &str = ".local/share/lay/nanda_wave/cell_scoreboard.json";
const MAX_TRACE_BYTES: u64 = 2 * 1024 * 1024;
const KEEP_TRACE_LINES: usize = 2500;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CellScoreboard {
    pub kind: String,
    pub updated_at: u64,
    pub records: u64,
    pub cells: BTreeMap<String, CellScore>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CellScore {
    pub generated: u64,
    pub accepted: u64,
    pub vetoed: u64,
    pub kept: u64,
    pub ok: u64,
    pub bad: u64,
    pub last_seen: u64,
}

impl CellScore {
    pub fn status(&self) -> &'static str {
        if self.bad > self.ok && self.bad > 0 {
            "подозрительная"
        } else if self.accepted > 0 || self.vetoed > 0 {
            "живая"
        } else if self.generated > 0 || self.kept > 0 {
            "след"
        } else {
            "спящая"
        }
    }
}

pub fn record_trace(
    case_id: impl Into<String>,
    label: impl Into<String>,
    trace: &WaveTrace,
    expected: Option<&str>,
) {
    record_trace_with_text_policy(case_id, label, trace, expected, false);
}

pub fn record_trace_with_text_policy(
    case_id: impl Into<String>,
    label: impl Into<String>,
    trace: &WaveTrace,
    expected: Option<&str>,
    include_text: bool,
) {
    let ts = unix_now();
    let record = build_trace_record(
        ts,
        case_id.into(),
        label.into(),
        trace,
        expected,
        include_text,
    );
    append_trace(&record);
    update_scoreboard(&record);
    super::resonance_memory::observe_record(&record);
}

pub fn load_recent_traces(limit: usize) -> Vec<CellTraceRecord> {
    let Some(path) = trace_path() else {
        return Vec::new();
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut records = text
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<CellTraceRecord>(line).ok())
        .take(limit)
        .collect::<Vec<_>>();
    records.reverse();
    records
}

pub fn load_scoreboard() -> CellScoreboard {
    scoreboard_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| CellScoreboard {
            kind: "nanda_wave_cell_scoreboard".to_string(),
            ..Default::default()
        })
}

fn append_trace(record: &CellTraceRecord) {
    let Some(path) = trace_path() else {
        return;
    };
    if let Ok(text) = serde_json::to_string(record) {
        let _ = crate::private_file::append_private_text(&path, &format!("{text}\n"));
        compact_trace_if_needed(&path);
    }
}

fn compact_trace_if_needed(path: &std::path::Path) {
    let Ok(metadata) = std::fs::metadata(path) else {
        return;
    };
    if metadata.len() <= MAX_TRACE_BYTES {
        return;
    }
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    let lines = text
        .lines()
        .rev()
        .take(KEEP_TRACE_LINES)
        .collect::<Vec<_>>();
    let compacted = lines.into_iter().rev().collect::<Vec<_>>().join("\n") + "\n";
    let _ = crate::private_file::write_private_text(path, &compacted);
}

fn update_scoreboard(record: &CellTraceRecord) {
    let Some(path) = scoreboard_path() else {
        return;
    };
    let mut scoreboard = load_scoreboard();
    scoreboard.kind = "nanda_wave_cell_scoreboard".to_string();
    scoreboard.updated_at = record.ts;
    scoreboard.records = scoreboard.records.saturating_add(1);
    let ok = record.output_matches_expected.unwrap_or(false);
    for cell in &record.cells {
        let score = scoreboard.cells.entry(cell.cell.clone()).or_default();
        score.generated = score.generated.saturating_add(cell.generated as u64);
        score.accepted = score.accepted.saturating_add(cell.accepted as u64);
        score.vetoed = score.vetoed.saturating_add(cell.vetoed as u64);
        score.kept = score.kept.saturating_add(cell.kept as u64);
        if ok {
            score.ok = score.ok.saturating_add(1);
        } else if record.output_matches_expected.is_some() {
            score.bad = score.bad.saturating_add(1);
        }
        score.last_seen = record.ts;
    }
    if let Ok(text) = serde_json::to_string_pretty(&scoreboard) {
        let _ = crate::private_file::write_private_text(&path, &format!("{text}\n"));
    }
}

fn trace_path() -> Option<PathBuf> {
    home_path(TRACE_PATH)
}

fn scoreboard_path() -> Option<PathBuf> {
    home_path(SCOREBOARD_PATH)
}

fn home_path(relative: &str) -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(relative))
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
