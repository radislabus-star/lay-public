//! Rolling correction-action journal for tray diagnostics.
//!
//! This is not a keylog: only successful lay actions are recorded, and the file
//! is capped to a small number of lines so it cannot grow without bound.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const ACTIONS_PATH: &str = ".local/share/lay/recent_actions.jsonl";
const TIMING_PROFILE_PATH: &str = ".local/share/lay/timing_profile.jsonl";

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_gate: Option<RecentActionGateTrace>,
    pub undo_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RecentActionGateTrace {
    pub stage: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_class: Option<String>,
    pub candidate_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_error_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_gate_action: Option<String>,
    pub reason: String,
}

impl RecentActionGateTrace {
    pub fn from_input_gate(trace: &crate::input_gate::InputGateDecisionTrace) -> Self {
        Self {
            stage: input_gate_stage_name(trace.stage).to_string(),
            input_class: trace.input_class.map(|class| class.as_str().to_string()),
            candidate_count: trace.candidate_count,
            selected_source: trace
                .selected_source
                .map(correction_source_name)
                .map(str::to_string),
            selected_source_id: trace.selected_source_id.clone(),
            selected_error_class: trace
                .selected_error_class
                .map(|class| class.as_str().to_string()),
            selected_gate_action: trace
                .selected_gate_action
                .map(gate_action_name)
                .map(str::to_string),
            reason: trace.reason.to_string(),
        }
    }
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
    record_action_with_stages_and_gate(
        kind,
        from,
        to,
        replace_words,
        words,
        elapsed_ms,
        decision_ms,
        output_ms,
        None,
        undo_available,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn record_action_with_stages_and_gate(
    kind: &str,
    from: &str,
    to: &str,
    replace_words: usize,
    words: usize,
    elapsed_ms: u128,
    decision_ms: Option<u128>,
    output_ms: Option<u128>,
    input_gate: Option<RecentActionGateTrace>,
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
        input_gate,
        undo_available,
    };
    record_action_async_to_path(&path, &action);
}

fn input_gate_stage_name(stage: crate::input_gate::InputGateStage) -> &'static str {
    match stage {
        crate::input_gate::InputGateStage::LiveInput => "live_input",
        crate::input_gate::InputGateStage::WordBoundary => "word_boundary",
        crate::input_gate::InputGateStage::ManualToggle => "manual_toggle",
        crate::input_gate::InputGateStage::CompletionAccept => "completion_accept",
        crate::input_gate::InputGateStage::FocusOrLayout => "focus_or_layout",
    }
}

fn correction_source_name(
    source: crate::correction_core::CorrectionDecisionSource,
) -> &'static str {
    match source {
        crate::correction_core::CorrectionDecisionSource::Deterministic => "deterministic",
        crate::correction_core::CorrectionDecisionSource::Nanda => "nanda",
    }
}

fn gate_action_name(action: crate::correction_core::CandidateGateAction) -> &'static str {
    match action {
        crate::correction_core::CandidateGateAction::Apply => "apply",
        crate::correction_core::CandidateGateAction::SuggestOnly => "suggest_only",
        crate::correction_core::CandidateGateAction::KeepOriginal => "keep_original",
        crate::correction_core::CandidateGateAction::Veto => "veto",
    }
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

pub fn record_timing_profile(kind: &str, route: &str, stages: &[(&str, u128)]) {
    if !crate::config::LayConfig::load().debug_action_log {
        return;
    }
    let Some(path) = home_relative_path(TIMING_PROFILE_PATH) else {
        return;
    };
    let stage_values = stages
        .iter()
        .map(|(name, ms)| serde_json::json!({ "name": name, "ms": ms }))
        .collect::<Vec<_>>();
    let record = serde_json::json!({
        "ts": unix_timestamp(),
        "kind": kind,
        "route": route,
        "stages": stage_values,
    });
    crate::debug_log::append_private_line(path, record.to_string());
}

fn record_action_async_to_path(path: &Path, action: &RecentAction<'_>) {
    if let Ok(line) = serde_json::to_string(action) {
        crate::debug_log::append_private_line(path.to_path_buf(), line);
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
    home_relative_path(ACTIONS_PATH)
}

fn home_relative_path(relative: &str) -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(relative))
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
