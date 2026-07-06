use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const RECENT_ACTIONS_PATH: &str = ".local/share/lay/recent_actions.jsonl";
const RECENT_LIMIT: usize = 500;

pub(crate) fn report_json() -> serde_json::Value {
    let Some(path) = recent_actions_path() else {
        return json!({
            "kind": "candidate_quality_report",
            "status": "unavailable",
            "reason": "HOME is not set"
        });
    };
    let Ok(text) = fs::read_to_string(&path) else {
        return json!({
            "kind": "candidate_quality_report",
            "status": "missing",
            "source": path.display().to_string()
        });
    };
    report_from_text(&text, RECENT_LIMIT, &path)
}

pub(crate) fn print_json() -> io::Result<()> {
    println!("{}", serde_json::to_string_pretty(&report_json())?);
    Ok(())
}

fn recent_actions_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(RECENT_ACTIONS_PATH))
}

#[derive(Debug, Default)]
struct CandidateQualityReport {
    records_seen: usize,
    records_used: usize,
    action_records: usize,
    candidate_before_apply_records: usize,
    gate_records: usize,
    selected_apply_records: usize,
    unsafe_edit_plan: usize,
    boundary_changed: usize,
    multiword_touch: usize,
    trace_output_mismatch: usize,
    trace_output_extra_context: usize,
    weak_bayes_apply: usize,
    bayes_unsupported_apply: usize,
    l3_weak_context_apply: usize,
    good_candidate_bad_rank: usize,
    no_candidates: usize,
    arbitration_records: usize,
    arbitration_selected: usize,
    arbitration_no_candidate_scores: usize,
    arbitration_no_selected: usize,
    arbitration_selected_not_top: usize,
    arbitration_selected_low_posterior: usize,
    arbitration_selected_high_risk: usize,
    arbitration_selected_unsafe_edit: usize,
    arbitration_nanda_extra_context: usize,
    arbitration_left_context_changed: usize,
    arbitration_unverified_transition: usize,
    nanda_apply: usize,
    deterministic_apply: usize,
    slow_output: usize,
    source_counts: BTreeMap<String, usize>,
    error_class_counts: BTreeMap<String, usize>,
    action_kind_counts: BTreeMap<String, usize>,
    safety_reason_counts: BTreeMap<String, usize>,
    class_counts: BTreeMap<&'static str, usize>,
}

impl CandidateQualityReport {
    fn add_class(&mut self, class: &'static str) {
        *self.class_counts.entry(class).or_default() += 1;
    }

    fn add_source(&mut self, source: &str) {
        if !source.is_empty() {
            *self.source_counts.entry(source.to_string()).or_default() += 1;
        }
    }

    fn add_error_class(&mut self, error_class: &str) {
        if !error_class.is_empty() {
            *self
                .error_class_counts
                .entry(error_class.to_string())
                .or_default() += 1;
        }
    }

    fn add_action_kind(&mut self, action_kind: &str) {
        if !action_kind.is_empty() {
            *self
                .action_kind_counts
                .entry(action_kind.to_string())
                .or_default() += 1;
        }
    }

    fn add_safety_reason(&mut self, reason: &str) {
        if !reason.is_empty() {
            *self
                .safety_reason_counts
                .entry(reason.to_string())
                .or_default() += 1;
        }
    }
}

fn report_from_text(text: &str, limit: usize, path: &Path) -> serde_json::Value {
    let mut report = CandidateQualityReport::default();
    let lines = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    report.records_seen = lines.len();
    let start = lines.len().saturating_sub(limit);
    for line in &lines[start..] {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        report.records_used += 1;
        inspect_record(&mut report, &value);
    }
    json!({
        "kind": "candidate_quality_report",
        "status": "ok",
        "source": path.display().to_string(),
        "window": {
            "records_seen": report.records_seen,
            "records_used": report.records_used,
            "limit": limit
        },
        "records": {
            "actions": report.action_records,
            "candidate_before_apply": report.candidate_before_apply_records,
            "with_gate": report.gate_records,
            "selected_apply": report.selected_apply_records
        },
        "classes": {
            "unsafe_edit_plan": report.unsafe_edit_plan,
            "boundary_changed": report.boundary_changed,
            "multiword_touch": report.multiword_touch,
            "trace_output_mismatch": report.trace_output_mismatch,
            "trace_output_extra_context": report.trace_output_extra_context,
            "weak_bayes_apply": report.weak_bayes_apply,
            "bayes_unsupported_apply": report.bayes_unsupported_apply,
            "l3_weak_context_apply": report.l3_weak_context_apply,
            "good_candidate_bad_rank": report.good_candidate_bad_rank,
            "no_candidates": report.no_candidates,
            "slow_output": report.slow_output
        },
        "candidate_arbitration": {
            "records": report.arbitration_records,
            "selected": report.arbitration_selected,
            "no_candidate_scores": report.arbitration_no_candidate_scores,
            "no_selected_candidate": report.arbitration_no_selected,
            "selected_not_top": report.arbitration_selected_not_top,
            "selected_low_posterior": report.arbitration_selected_low_posterior,
            "selected_high_risk": report.arbitration_selected_high_risk,
            "selected_unsafe_edit": report.arbitration_selected_unsafe_edit,
            "nanda_extra_context": report.arbitration_nanda_extra_context,
            "left_context_changed": report.arbitration_left_context_changed,
            "unverified_transition": report.arbitration_unverified_transition,
            "read_as": "why the chosen candidate won or should not have won; diagnostic only"
        },
        "source_mix": {
            "deterministic_apply": report.deterministic_apply,
            "nanda_apply": report.nanda_apply,
            "by_source": report.source_counts
        },
        "edit_actions": report.action_kind_counts,
        "safety_reasons": report.safety_reason_counts,
        "error_classes": report.error_class_counts,
        "class_counts": report.class_counts,
        "read_as": "diagnostic only; this report does not change runtime decisions"
    })
}

fn inspect_record(report: &mut CandidateQualityReport, value: &Value) {
    let kind = value.get("kind").and_then(Value::as_str).unwrap_or("");
    if kind == "candidate_before_apply" {
        report.candidate_before_apply_records += 1;
        inspect_edit_plan(report, value);
    } else {
        report.action_records += 1;
    }
    if value
        .get("output_ms")
        .and_then(Value::as_u64)
        .is_some_and(|ms| ms >= 250)
        || value
            .get("elapsed_ms")
            .and_then(Value::as_u64)
            .is_some_and(|ms| ms >= 300)
    {
        report.slow_output += 1;
        report.add_class("slow_output");
    }
    let Some(gate) = value.get("input_gate") else {
        return;
    };
    report.gate_records += 1;
    inspect_gate(report, value, gate);
}

fn inspect_edit_plan(report: &mut CandidateQualityReport, value: &Value) {
    if let Some(action_kind) = value.get("action_kind").and_then(Value::as_str) {
        report.add_action_kind(action_kind);
        if action_kind == "block_unsafe" {
            report.unsafe_edit_plan += 1;
            report.add_class("unsafe_edit_action");
        }
    }
    if let Some(reason) = value.get("safety_reason").and_then(Value::as_str) {
        report.add_safety_reason(reason);
        if reason == "low_confidence_boundary_edit" {
            report.add_class("low_confidence_boundary_edit");
        }
        if reason == "low_confidence_wide_edit" {
            report.add_class("low_confidence_wide_edit");
        }
    }
    let safety_allow_apply = value
        .get("safety_allow_apply")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let boundary_changed = value
        .get("boundary_changed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let changes_non_last_word = value
        .get("changes_non_last_word")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let word_count_changed = value
        .get("word_count_changed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let would_touch_words = value
        .get("would_touch_words")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if !safety_allow_apply || boundary_changed || changes_non_last_word {
        report.unsafe_edit_plan += 1;
        report.add_class("unsafe_edit_plan");
    }
    if boundary_changed || word_count_changed {
        report.boundary_changed += 1;
        report.add_class("boundary_changed");
    }
    if would_touch_words > 1 || changes_non_last_word {
        report.multiword_touch += 1;
        report.add_class("multiword_touch");
    }
}

fn inspect_gate(report: &mut CandidateQualityReport, value: &Value, gate: &Value) {
    let candidate_count = gate
        .get("candidate_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if candidate_count == 0 {
        report.no_candidates += 1;
        report.add_class("no_candidates");
    }
    let selected_action = gate
        .get("selected_gate_action")
        .and_then(Value::as_str)
        .unwrap_or("");
    if selected_action == "apply" {
        report.selected_apply_records += 1;
    }
    let selected_source = gate
        .get("selected_source")
        .and_then(Value::as_str)
        .unwrap_or("");
    let selected_error_class = gate
        .get("selected_error_class")
        .and_then(Value::as_str)
        .unwrap_or("");
    report.add_source(selected_source);
    report.add_error_class(selected_error_class);
    match selected_source {
        "deterministic" => report.deterministic_apply += 1,
        "nanda" => report.nanda_apply += 1,
        _ => {}
    }
    if let Some(selected) = selected_candidate_score(gate) {
        inspect_selected_candidate(report, value, selected_source, selected);
        inspect_rank(report, gate, selected);
    }
    inspect_arbitration(report, value, gate);
}

fn selected_candidate_score(gate: &Value) -> Option<&Value> {
    gate.get("candidate_scores")?
        .as_array()?
        .iter()
        .find(|candidate| {
            candidate
                .get("selected")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
}

fn inspect_selected_candidate(
    report: &mut CandidateQualityReport,
    value: &Value,
    selected_source: &str,
    selected: &Value,
) {
    let posterior = selected
        .get("posterior_milli")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let usage = selected
        .get("usage_prior_milli")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let context = selected
        .get("context_prior_milli")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let risk = selected
        .get("risk_milli")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let replacement = selected
        .get("replacement")
        .and_then(Value::as_str)
        .unwrap_or("");
    let to = value.get("to").and_then(Value::as_str).unwrap_or("");
    let replacement_trimmed = replacement.trim();
    let to_trimmed = to.trim();
    if !to_trimmed.is_empty() && !replacement_trimmed.is_empty() {
        if replacement_trimmed.split_whitespace().count() > to_trimmed.split_whitespace().count() {
            report.trace_output_extra_context += 1;
            report.add_class("trace_output_extra_context");
        }
        if !replacement_trimmed.contains(to_trimmed) && !to_trimmed.contains(replacement_trimmed) {
            report.trace_output_mismatch += 1;
            report.add_class("trace_output_mismatch");
        }
    }
    if posterior < 300 {
        report.weak_bayes_apply += 1;
        report.add_class("weak_bayes_apply");
    }
    if usage <= 0 && context <= 0 && posterior < 520 {
        report.bayes_unsupported_apply += 1;
        report.add_class("bayes_unsupported_apply");
    }
    if selected_source == "nanda" && context <= 0 && risk > 0 {
        report.l3_weak_context_apply += 1;
        report.add_class("l3_weak_context_apply");
    }
}

fn inspect_rank(report: &mut CandidateQualityReport, gate: &Value, selected: &Value) {
    let selected_posterior = selected
        .get("posterior_milli")
        .and_then(Value::as_i64)
        .unwrap_or(i64::MIN);
    let Some(candidates) = gate.get("candidate_scores").and_then(Value::as_array) else {
        return;
    };
    let better_exists = candidates.iter().any(|candidate| {
        !candidate
            .get("selected")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            && candidate
                .get("posterior_milli")
                .and_then(Value::as_i64)
                .is_some_and(|posterior| posterior >= selected_posterior + 120)
            && candidate
                .get("gate_action")
                .and_then(Value::as_str)
                .is_some_and(|action| action == "apply" || action == "suggest_only")
    });
    if better_exists {
        report.good_candidate_bad_rank += 1;
        report.add_class("good_candidate_bad_rank");
    }
}

fn inspect_arbitration(report: &mut CandidateQualityReport, value: &Value, gate: &Value) {
    let Some(candidates) = gate.get("candidate_scores").and_then(Value::as_array) else {
        report.arbitration_no_candidate_scores += 1;
        report.add_class("arbitration_no_candidate_scores");
        return;
    };
    if candidates.is_empty() {
        report.arbitration_no_candidate_scores += 1;
        report.add_class("arbitration_no_candidate_scores");
        return;
    }
    report.arbitration_records += 1;

    let selected_index = candidates.iter().position(is_selected_candidate);
    let Some(selected_index) = selected_index else {
        report.arbitration_no_selected += 1;
        report.add_class("arbitration_no_selected_candidate");
        return;
    };
    report.arbitration_selected += 1;

    let selected = &candidates[selected_index];
    if let Some(top_index) = top_candidate_index(candidates) {
        let selected_rank = candidate_rank_tuple(selected);
        let top_rank = candidate_rank_tuple(&candidates[top_index]);
        if top_index != selected_index && top_rank.0 >= selected_rank.0 + 80 {
            report.arbitration_selected_not_top += 1;
            report.add_class("arbitration_selected_not_top");
        }
    }

    let posterior = selected
        .get("posterior_milli")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let risk = selected
        .get("risk_milli")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    if posterior < 350 {
        report.arbitration_selected_low_posterior += 1;
        report.add_class("arbitration_selected_low_posterior");
    }
    if risk >= 300 {
        report.arbitration_selected_high_risk += 1;
        report.add_class("arbitration_selected_high_risk");
    }

    let safety_allow_apply = value
        .get("safety_allow_apply")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let touches_left_context = value
        .get("changes_non_last_word")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || value
            .get("would_touch_words")
            .and_then(Value::as_u64)
            .is_some_and(|words| words > 1);
    if !safety_allow_apply || touches_left_context {
        report.arbitration_selected_unsafe_edit += 1;
        report.add_class("arbitration_selected_unsafe_edit");
    }

    let transition_left_context_changed = selected
        .get("edit_transition_left_context_changed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let transition_verified = selected
        .get("edit_transition_verified")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    if transition_left_context_changed {
        report.arbitration_left_context_changed += 1;
        report.add_class("arbitration_left_context_changed");
    }
    if transition_left_context_changed && !transition_verified {
        report.arbitration_unverified_transition += 1;
        report.add_class("arbitration_unverified_transition");
    }

    let selected_source = selected
        .get("source")
        .and_then(Value::as_str)
        .or_else(|| gate.get("selected_source").and_then(Value::as_str))
        .unwrap_or("");
    let replacement_words = selected
        .get("replacement")
        .and_then(Value::as_str)
        .unwrap_or("")
        .split_whitespace()
        .count();
    let to_words = value
        .get("to")
        .and_then(Value::as_str)
        .unwrap_or("")
        .split_whitespace()
        .count();
    if selected_source == "nanda" && replacement_words > to_words.max(1) {
        report.arbitration_nanda_extra_context += 1;
        report.add_class("arbitration_nanda_extra_context");
    }
}

fn is_selected_candidate(candidate: &Value) -> bool {
    candidate
        .get("selected")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn top_candidate_index(candidates: &[Value]) -> Option<usize> {
    candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| candidate_is_viable(candidate))
        .max_by_key(|(_, candidate)| candidate_rank_tuple(candidate))
        .map(|(index, _)| index)
}

fn candidate_is_viable(candidate: &Value) -> bool {
    candidate
        .get("gate_action")
        .and_then(Value::as_str)
        .map_or(true, |action| action == "apply" || action == "suggest_only")
}

fn candidate_rank_tuple(candidate: &Value) -> (i64, i64) {
    let posterior = candidate
        .get("posterior_milli")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let risk = candidate
        .get("risk_milli")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    (posterior, -risk)
}

#[cfg(test)]
mod tests {
    use super::report_from_text;
    use std::path::PathBuf;

    #[test]
    fn candidate_quality_report_flags_trace_and_boundary_risks() {
        let text = r#"
{"kind":"candidate_before_apply","action_kind":"replace_last_token","from":"gjhn ","to":"порт ","boundary_changed":false,"changes_non_last_word":false,"word_count_changed":false,"would_touch_words":1,"safety_allow_apply":true,"input_gate":{"selected_source":"deterministic","selected_error_class":"composite-typo","selected_gate_action":"apply","candidate_count":1,"candidate_scores":[{"replacement":"порт port порт ","source":"deterministic","posterior_milli":488,"usage_prior_milli":0,"context_prior_milli":240,"risk_milli":280,"gate_action":"apply","selected":true,"edit_transition_left_context_changed":false,"edit_transition_verified":true}]}}
{"kind":"candidate_before_apply","action_kind":"block_unsafe","from":"одно два ","to":"однотри ","boundary_changed":true,"changes_non_last_word":true,"word_count_changed":true,"would_touch_words":2,"safety_allow_apply":false,"input_gate":{"selected_source":"nanda","selected_error_class":"glued-words","selected_gate_action":"apply","candidate_count":2,"candidate_scores":[{"replacement":"однотри ","source":"nanda","posterior_milli":220,"usage_prior_milli":0,"context_prior_milli":0,"risk_milli":310,"gate_action":"apply","selected":true,"edit_transition_left_context_changed":true,"edit_transition_verified":false},{"replacement":"одно два ","source":"deterministic","posterior_milli":500,"usage_prior_milli":0,"context_prior_milli":0,"risk_milli":0,"gate_action":"suggest_only","selected":false,"edit_transition_left_context_changed":false,"edit_transition_verified":true}]}}
"#;

        let report = report_from_text(text, 20, &PathBuf::from("recent.jsonl"));

        assert_eq!(report["classes"]["trace_output_mismatch"], 0);
        assert_eq!(report["classes"]["trace_output_extra_context"], 1);
        assert_eq!(report["classes"]["unsafe_edit_plan"], 2);
        assert_eq!(report["edit_actions"]["replace_last_token"], 1);
        assert_eq!(report["edit_actions"]["block_unsafe"], 1);
        assert_eq!(report["classes"]["boundary_changed"], 1);
        assert_eq!(report["classes"]["multiword_touch"], 1);
        assert_eq!(report["classes"]["weak_bayes_apply"], 1);
        assert_eq!(report["classes"]["bayes_unsupported_apply"], 1);
        assert_eq!(report["classes"]["l3_weak_context_apply"], 1);
        assert_eq!(report["classes"]["good_candidate_bad_rank"], 1);
        assert_eq!(report["candidate_arbitration"]["records"], 2);
        assert_eq!(report["candidate_arbitration"]["selected"], 2);
        assert_eq!(report["candidate_arbitration"]["selected_not_top"], 1);
        assert_eq!(report["candidate_arbitration"]["selected_low_posterior"], 1);
        assert_eq!(report["candidate_arbitration"]["selected_high_risk"], 1);
        assert_eq!(report["candidate_arbitration"]["selected_unsafe_edit"], 1);
        assert_eq!(report["candidate_arbitration"]["left_context_changed"], 1);
        assert_eq!(report["candidate_arbitration"]["unverified_transition"], 1);
    }
}
