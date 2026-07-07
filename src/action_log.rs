//! Rolling correction-action journal for tray diagnostics.
//!
//! This is not a keylog: only successful lay actions are recorded, and the file
//! is capped to a small number of lines so it cannot grow without bound.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::time::unix_timestamp;

const ACTIONS_PATH: &str = ".local/share/lay/recent_actions.jsonl";
const TIMING_PROFILE_PATH: &str = ".local/share/lay/timing_profile.jsonl";
const NANDA_DIRTY_TASKS_PATH: &str = ".local/share/lay/nanda_wave/dirty_tasks.jsonl";
const MAX_LOGGED_CANDIDATE_SCORES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MutationLogRoute(&'static str);

impl MutationLogRoute {
    pub const AUTO_UNDO: Self = Self("auto_undo");
    pub const ENTER_AUTOCORRECT: Self = Self("enter_autocorrect");
    pub const IME_ACTIVE_COMPOSITION: Self = Self("ime_active_composition");
    pub const IME_COMMITTED_TAIL: Self = Self("ime_committed_tail");
    pub const MANUAL_NATIVE_REPLACE: Self = Self("manual_native_replace");
    pub const MANUAL_TEXT_REPLACE: Self = Self("manual_text_replace");
    pub const TYPING_ASSIST_IME: Self = Self("typing_assist_ime");
    pub const TYPING_ASSIST_MINIMAL: Self = Self("typing_assist_minimal");

    #[cfg(test)]
    pub const TEST: Self = Self("test_mutation_route");

    pub fn as_str(self) -> &'static str {
        self.0
    }
}

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scoreboard: Option<RecentActionGateScoreboard>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) candidate_scores: Vec<RecentActionCandidateScore>,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct RecentActionCandidateScore {
    pub(crate) replacement: String,
    pub(crate) source: String,
    pub(crate) source_id: String,
    pub(crate) error_class: String,
    pub(crate) action_operator: String,
    pub(crate) action_proof: String,
    pub(crate) edit_transition_operator: String,
    pub(crate) edit_transition_proof: String,
    pub(crate) edit_transition_verified: bool,
    pub(crate) edit_transition_left_context_changed: bool,
    pub(crate) edit_transition_changed_tokens: usize,
    pub(crate) edit_shape: String,
    pub(crate) preservation_milli: i16,
    pub(crate) lost_mass_milli: i16,
    pub(crate) added_mass_milli: i16,
    pub(crate) operator_fit_milli: i16,
    pub(crate) shortcut_risk_milli: i16,
    pub(crate) anti_wave_milli: i16,
    pub(crate) explanation_score_milli: i16,
    pub(crate) gate_action: String,
    pub(crate) gate_reason: String,
    pub(crate) likelihood_milli: i16,
    pub(crate) usage_prior_milli: i16,
    pub(crate) context_prior_milli: i16,
    pub(crate) risk_milli: i16,
    pub(crate) posterior_milli: i16,
    pub(crate) selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RecentActionGateScoreboard {
    pub total_candidates: usize,
    pub apply_candidates: usize,
    pub suggest_only_candidates: usize,
    pub keep_original_candidates: usize,
    pub veto_candidates: usize,
    pub deterministic_candidates: usize,
    pub nanda_candidates: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_bayes_posterior_milli: Option<i16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct DirtyTaskRecord<'a> {
    ts: u64,
    kind: &'static str,
    action_kind: &'a str,
    from: &'a str,
    to: &'a str,
    replace_words: usize,
    words: usize,
    gate: &'a RecentActionGateTrace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct CandidateBeforeApplyRecord<'a> {
    ts: u64,
    kind: &'static str,
    mutation_route: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    action_kind: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    action_source: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    action_confidence_milli: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transition_operator: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transition_proof: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transition_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transition_left_context_changed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transition_changed_tokens: Option<usize>,
    from: &'a str,
    to: &'a str,
    edit_plan: EditPlanRecord<'a>,
    deleted_text: &'a str,
    inserted_text: &'a str,
    multiword_original: bool,
    deleted_contains_space: bool,
    inserted_contains_space: bool,
    insertion_splits_word: bool,
    word_count_changed: bool,
    boundary_changed: bool,
    changes_non_last_word: bool,
    would_touch_words: usize,
    safety_allow_apply: bool,
    safety_reason: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_gate: Option<RecentActionGateTrace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct EditPlanRecord<'a> {
    move_left: u32,
    backspaces: u32,
    insert: &'a str,
    move_right: u32,
}

impl RecentActionGateTrace {
    pub fn from_input_gate(trace: &crate::input_gate::InputGateDecisionTrace) -> Self {
        Self {
            stage: input_gate_stage_name(trace.stage).to_string(),
            input_class: trace.input_class.map(|class| class.as_str().to_string()),
            candidate_count: trace.candidate_count,
            scoreboard: Some(RecentActionGateScoreboard {
                total_candidates: trace.scoreboard.total_candidates,
                apply_candidates: trace.scoreboard.apply_candidates,
                suggest_only_candidates: trace.scoreboard.suggest_only_candidates,
                keep_original_candidates: trace.scoreboard.keep_original_candidates,
                veto_candidates: trace.scoreboard.veto_candidates,
                deterministic_candidates: trace.scoreboard.deterministic_candidates,
                nanda_candidates: trace.scoreboard.nanda_candidates,
                selected_bayes_posterior_milli: trace.scoreboard.selected_bayes_posterior_milli,
            }),
            candidate_scores: trace
                .candidate_scores
                .iter()
                .take(MAX_LOGGED_CANDIDATE_SCORES)
                .map(|score| RecentActionCandidateScore {
                    replacement: score.replacement.clone(),
                    source: correction_source_name(score.source).to_string(),
                    source_id: score.source_id.clone(),
                    error_class: score.error_class.as_str().to_string(),
                    action_operator: score.action_operator.to_string(),
                    action_proof: score.action_proof.to_string(),
                    edit_transition_operator: score.edit_transition_operator.to_string(),
                    edit_transition_proof: score.edit_transition_proof.to_string(),
                    edit_transition_verified: score.edit_transition_verified,
                    edit_transition_left_context_changed: score
                        .edit_transition_left_context_changed,
                    edit_transition_changed_tokens: score.edit_transition_changed_tokens,
                    edit_shape: score.edit_shape.to_string(),
                    preservation_milli: score.preservation_milli,
                    lost_mass_milli: score.lost_mass_milli,
                    added_mass_milli: score.added_mass_milli,
                    operator_fit_milli: score.operator_fit_milli,
                    shortcut_risk_milli: score.shortcut_risk_milli,
                    anti_wave_milli: score.anti_wave_milli,
                    explanation_score_milli: score.explanation_score_milli,
                    gate_action: gate_action_name(score.gate_action).to_string(),
                    gate_reason: score.gate_reason.to_string(),
                    likelihood_milli: score.likelihood_milli,
                    usage_prior_milli: score.usage_prior_milli,
                    context_prior_milli: score.context_prior_milli,
                    risk_milli: score.risk_milli,
                    posterior_milli: score.posterior_milli,
                    selected: score.selected,
                })
                .collect(),
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

    pub fn selected_transition_audit(&self) -> crate::text_edit::TransitionAudit {
        let Some(selected) = self.candidate_scores.iter().find(|score| score.selected) else {
            return crate::text_edit::TransitionAudit::none();
        };
        crate::text_edit::TransitionAudit::proven(
            selected.edit_transition_operator.clone(),
            selected.edit_transition_proof.clone(),
            selected.edit_transition_verified,
            selected.edit_transition_left_context_changed,
            selected.edit_transition_changed_tokens,
        )
    }
}

pub fn record_candidate_before_apply(
    mutation_route: MutationLogRoute,
    from: &str,
    to: &str,
    plan: &crate::text_edit::TextReplacement,
    safety: &crate::text_edit::EditPlanSafetyReport,
    input_gate: Option<RecentActionGateTrace>,
) {
    record_candidate_before_apply_inner(
        mutation_route,
        from,
        to,
        plan,
        safety,
        None,
        None,
        None,
        None,
        input_gate,
    );
}

pub fn record_candidate_edit_action_before_apply(
    action: &crate::text_edit::EditAction,
    mutation_route: MutationLogRoute,
    input_gate: Option<RecentActionGateTrace>,
) {
    let (Some(plan), Some(safety)) = (action.plan.as_ref(), action.safety.as_ref()) else {
        return;
    };
    record_candidate_before_apply_inner(
        mutation_route,
        &action.from_text,
        &action.to_text,
        plan,
        safety,
        Some(action.kind.as_str()),
        Some(action.source.as_str()),
        Some(action.confidence_milli),
        Some(&action.transition),
        input_gate,
    );
}

#[allow(clippy::too_many_arguments)]
fn record_candidate_before_apply_inner(
    mutation_route: MutationLogRoute,
    from: &str,
    to: &str,
    plan: &crate::text_edit::TextReplacement,
    safety: &crate::text_edit::EditPlanSafetyReport,
    action_kind: Option<&'static str>,
    action_source: Option<&str>,
    action_confidence_milli: Option<i16>,
    transition: Option<&crate::text_edit::TransitionAudit>,
    input_gate: Option<RecentActionGateTrace>,
) {
    if !crate::config::LayConfig::load().debug_action_log {
        return;
    }
    let Some(path) = actions_path() else {
        return;
    };
    let record = CandidateBeforeApplyRecord {
        ts: unix_timestamp(),
        kind: "candidate_before_apply",
        mutation_route: mutation_route.as_str(),
        action_kind,
        action_source,
        action_confidence_milli,
        transition_operator: transition.and_then(|audit| audit.operator.as_deref()),
        transition_proof: transition.and_then(|audit| audit.proof.as_deref()),
        transition_verified: transition.and_then(|audit| audit.verified),
        transition_left_context_changed: transition.and_then(|audit| audit.left_context_changed),
        transition_changed_tokens: transition.and_then(|audit| audit.changed_tokens),
        from,
        to,
        edit_plan: EditPlanRecord {
            move_left: plan.move_left,
            backspaces: plan.backspaces,
            insert: &plan.insert,
            move_right: plan.move_right,
        },
        deleted_text: &safety.deleted_text,
        inserted_text: &safety.inserted_text,
        multiword_original: from.split_whitespace().count() > 1,
        deleted_contains_space: safety.deleted_contains_space,
        inserted_contains_space: safety.inserted_contains_space,
        insertion_splits_word: safety.insertion_splits_word,
        word_count_changed: safety.word_count_changed,
        boundary_changed: safety.boundary_changed,
        changes_non_last_word: safety.changes_non_last_word,
        would_touch_words: safety.would_touch_words,
        safety_allow_apply: safety.allow_apply,
        safety_reason: safety.reason,
        input_gate,
    };
    if let Ok(line) = serde_json::to_string(&record) {
        crate::debug_log::append_private_line(path, line);
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
    crate::nanda_wave::record_accepted_fix_usage(from, to);
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
    record_dirty_task_if_useful(&action);
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

fn record_dirty_task_if_useful(action: &RecentAction<'_>) {
    let Some(gate) = action.input_gate.as_ref() else {
        return;
    };
    if gate.selected_gate_action.as_deref() != Some("apply") {
        return;
    }
    if gate.selected_source_id.is_none() && gate.selected_source.is_none() {
        return;
    }
    let Some(path) = home_relative_path(NANDA_DIRTY_TASKS_PATH) else {
        return;
    };
    let record = DirtyTaskRecord {
        ts: action.ts,
        kind: "lay_dirty_task_v1",
        action_kind: action.kind,
        from: action.from,
        to: action.to,
        replace_words: action.replace_words,
        words: action.words,
        gate,
    };
    if let Ok(line) = serde_json::to_string(&record) {
        crate::debug_log::append_private_line(path, line);
    }
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

#[cfg(test)]
#[path = "action_log_tests.rs"]
mod tests;
