use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const DEFAULT_RECENT_ACTIONS_PATH: &str = ".local/share/lay/recent_actions.jsonl";
const DEFAULT_CORRECTIONS_PATH: &str = ".local/share/lay/corrections.jsonl";
const DEFAULT_LIMIT: usize = 2_000;
const MAX_TEXT_CHARS: usize = 180;
const MAX_WORDS: usize = 12;
const SAMPLE_LIMIT: usize = 18;

#[derive(Debug, Clone, Serialize)]
struct DirtyLogPair {
    kind: &'static str,
    ts: u64,
    source_log: &'static str,
    signal: &'static str,
    train_role: &'static str,
    quarantine_reason: String,
    original: String,
    expected: String,
    operation: String,
    count_weight: usize,
    replace_words: usize,
    candidate_count: usize,
    source_id: String,
    error_class: String,
    action_operator: String,
    action_proof: String,
    posterior_milli: Option<i64>,
    decision_rank_milli: Option<i64>,
    risk_milli: Option<i64>,
    usage_prior_milli: Option<i64>,
    context_prior_milli: Option<i64>,
    l2_wave_peak_milli: Option<i64>,
    l2_wave_peak_reason: String,
    l3_phrase_milli: Option<i64>,
    l3_phrase_decision: String,
    l4_signed_milli: Option<i64>,
    l4_signed_reason: String,
    transition_verified: Option<bool>,
    left_context_changed: bool,
    boundary_changed: bool,
    word_count_changed: bool,
    safety_allow_apply: Option<bool>,
    safety_reason: String,
}

#[derive(Debug, Default)]
struct CollectStats {
    raw_lines: usize,
    invalid_lines: usize,
    skipped_pairs: usize,
}

#[derive(Debug, Default)]
struct Collector {
    pairs: Vec<DirtyLogPair>,
    recent: CollectStats,
    corrections: CollectStats,
    by_signal: BTreeMap<String, usize>,
    by_train_role: BTreeMap<String, usize>,
    by_quarantine_reason: BTreeMap<String, usize>,
    by_operation: BTreeMap<String, usize>,
    by_source_id: BTreeMap<String, usize>,
    by_error_class: BTreeMap<String, usize>,
    by_safety_reason: BTreeMap<String, usize>,
    low_posterior: usize,
    high_risk: usize,
    boundary_changed: usize,
    left_context_changed: usize,
    unsafe_edit_plan: usize,
}

pub(crate) fn print_json(args: &[String]) -> io::Result<()> {
    let limit = arg_value(args, "--limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(DEFAULT_LIMIT);
    let recent_path = arg_value(args, "--recent-actions")
        .map(PathBuf::from)
        .or_else(|| home_path(DEFAULT_RECENT_ACTIONS_PATH));
    let corrections_path = arg_value(args, "--learning-log")
        .map(PathBuf::from)
        .or_else(|| home_path(DEFAULT_CORRECTIONS_PATH));
    let out = arg_value(args, "--out").map(PathBuf::from);
    let report = collect(recent_path.as_deref(), corrections_path.as_deref(), limit)?;
    if let Some(path) = out.as_deref() {
        write_pairs(path, &report.pairs)?;
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&report_json(&report, recent_path, corrections_path, out))?
    );
    Ok(())
}

fn collect(
    recent_path: Option<&Path>,
    corrections_path: Option<&Path>,
    limit: usize,
) -> io::Result<Collector> {
    let mut collector = Collector::default();
    if let Some(path) = corrections_path {
        if let Ok(text) = fs::read_to_string(path) {
            collect_corrections_text(&mut collector, &text, limit);
        }
    }
    if let Some(path) = recent_path {
        if let Ok(text) = fs::read_to_string(path) {
            collect_recent_text(&mut collector, &text, limit);
        }
    }
    Ok(collector)
}

fn collect_recent_text(collector: &mut Collector, text: &str, limit: usize) {
    let lines = bounded_tail_lines(text, limit);
    for line in lines {
        collector.recent.raw_lines += 1;
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            collector.recent.invalid_lines += 1;
            continue;
        };
        let kind = value.get("kind").and_then(Value::as_str).unwrap_or("");
        match kind {
            "typing-assist" => {
                if let Some(pair) = pair_from_recent(&value, "applied_observed") {
                    collector.add(pair);
                } else {
                    collector.recent.skipped_pairs += 1;
                }
            }
            "layout-replay" => {
                if let Some(pair) = pair_from_layout_replay(&value) {
                    collector.add(pair);
                } else {
                    collector.recent.skipped_pairs += 1;
                }
            }
            "candidate_before_apply" => {
                inspect_candidate_before_apply(collector, &value);
            }
            _ => {}
        }
    }
}

fn collect_corrections_text(collector: &mut Collector, text: &str, limit: usize) {
    let lines = bounded_tail_lines(text, limit);
    for line in lines {
        collector.corrections.raw_lines += 1;
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            collector.corrections.invalid_lines += 1;
            continue;
        };
        let kind = value.get("kind").and_then(Value::as_str).unwrap_or("");
        if kind == "user-correction" {
            if let Some(pair) = pair_from_correction(&value, "user_accepted_fix", "from", "to") {
                collector.add(pair);
            } else {
                collector.corrections.skipped_pairs += 1;
            }
            if value.get("lay_from").is_some() && value.get("lay_to").is_some() {
                if let Some(pair) =
                    pair_from_correction(&value, "user_rejected_lay_output", "lay_from", "lay_to")
                {
                    collector.add(pair);
                } else {
                    collector.corrections.skipped_pairs += 1;
                }
            }
        } else if kind == "typing-assist" {
            if let Some(pair) = pair_from_correction(&value, "applied_observed", "from", "to") {
                collector.add(pair);
            } else {
                collector.corrections.skipped_pairs += 1;
            }
        }
    }
}

fn pair_from_recent(value: &Value, signal: &'static str) -> Option<DirtyLogPair> {
    let from = value.get("from").and_then(Value::as_str)?;
    let to = value.get("to").and_then(Value::as_str)?;
    if !valid_pair(from, to) {
        return None;
    }
    let gate = value.get("input_gate");
    let selected = gate.and_then(selected_candidate_score);
    let source_id = gate
        .and_then(|gate| gate.get("selected_source_id"))
        .and_then(Value::as_str)
        .or_else(|| {
            selected
                .and_then(|item| item.get("source_id"))
                .and_then(Value::as_str)
        })
        .unwrap_or("");
    let error_class = gate
        .and_then(|gate| gate.get("selected_error_class"))
        .and_then(Value::as_str)
        .or_else(|| {
            selected
                .and_then(|item| item.get("error_class"))
                .and_then(Value::as_str)
        })
        .unwrap_or("");
    let (train_role, quarantine_reason) = train_role_and_quarantine(
        signal,
        source_id,
        from,
        to,
        &classify_operation(from, to, source_id, error_class),
    );
    Some(DirtyLogPair {
        kind: "dirty_log_pair_v1",
        ts: value.get("ts").and_then(Value::as_u64).unwrap_or(0),
        source_log: "recent_actions",
        signal,
        train_role,
        quarantine_reason,
        original: from.to_string(),
        expected: to.to_string(),
        operation: classify_operation(from, to, source_id, error_class),
        count_weight: value
            .get("replace_words")
            .or_else(|| value.get("words"))
            .and_then(Value::as_u64)
            .unwrap_or(1)
            .max(1) as usize,
        replace_words: value
            .get("replace_words")
            .or_else(|| value.get("words"))
            .and_then(Value::as_u64)
            .unwrap_or(1)
            .max(1) as usize,
        candidate_count: gate
            .and_then(|gate| gate.get("candidate_count"))
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize,
        source_id: source_id.to_string(),
        error_class: error_class.to_string(),
        action_operator: selected_string(selected, "action_operator"),
        action_proof: selected_string(selected, "action_proof"),
        posterior_milli: selected_i64(selected, "posterior_milli").or_else(|| {
            gate.and_then(|gate| {
                gate.get("scoreboard")?
                    .get("selected_bayes_posterior_milli")?
                    .as_i64()
            })
        }),
        decision_rank_milli: selected_i64(selected, "decision_rank_milli"),
        risk_milli: selected_i64(selected, "risk_milli"),
        usage_prior_milli: selected_i64(selected, "usage_prior_milli"),
        context_prior_milli: selected_i64(selected, "context_prior_milli"),
        l2_wave_peak_milli: selected_i64(selected, "l2_wave_peak_milli"),
        l2_wave_peak_reason: selected_string(selected, "l2_wave_peak_reason"),
        l3_phrase_milli: selected_i64(selected, "l3_phrase_milli"),
        l3_phrase_decision: selected_string(selected, "l3_phrase_decision"),
        l4_signed_milli: selected_i64(selected, "l4_signed_milli"),
        l4_signed_reason: selected_string(selected, "l4_signed_reason"),
        transition_verified: selected
            .and_then(|item| item.get("edit_transition_verified"))
            .and_then(Value::as_bool),
        left_context_changed: selected
            .and_then(|item| item.get("edit_transition_left_context_changed"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        boundary_changed: false,
        word_count_changed: word_count_changed(from, to),
        safety_allow_apply: None,
        safety_reason: String::new(),
    })
}

fn pair_from_layout_replay(value: &Value) -> Option<DirtyLogPair> {
    let from = value.get("from").and_then(Value::as_str)?;
    let to = value.get("to").and_then(Value::as_str)?;
    if !valid_pair(from, to) {
        return None;
    }
    Some(DirtyLogPair {
        kind: "dirty_log_pair_v1",
        ts: value.get("ts").and_then(Value::as_u64).unwrap_or(0),
        source_log: "recent_actions",
        signal: "manual_layout_replay",
        train_role: "positive",
        quarantine_reason: String::new(),
        original: from.to_string(),
        expected: to.to_string(),
        operation: "layout".to_string(),
        count_weight: 1,
        replace_words: value
            .get("replace_words")
            .or_else(|| value.get("words"))
            .and_then(Value::as_u64)
            .unwrap_or(1)
            .max(1) as usize,
        candidate_count: 0,
        source_id: "manual_layout_replay".to_string(),
        error_class: "layout".to_string(),
        action_operator: "manual_replay".to_string(),
        action_proof: "manual_replay".to_string(),
        posterior_milli: None,
        decision_rank_milli: None,
        risk_milli: None,
        usage_prior_milli: None,
        context_prior_milli: None,
        l2_wave_peak_milli: None,
        l2_wave_peak_reason: String::new(),
        l3_phrase_milli: None,
        l3_phrase_decision: String::new(),
        l4_signed_milli: None,
        l4_signed_reason: String::new(),
        transition_verified: None,
        left_context_changed: false,
        boundary_changed: word_count_changed(from, to),
        word_count_changed: word_count_changed(from, to),
        safety_allow_apply: None,
        safety_reason: String::new(),
    })
}

fn pair_from_correction(
    value: &Value,
    signal: &'static str,
    from_key: &str,
    to_key: &str,
) -> Option<DirtyLogPair> {
    let from = value.get(from_key).and_then(Value::as_str)?;
    let to = value.get(to_key).and_then(Value::as_str)?;
    if !valid_pair(from, to) {
        return None;
    }
    let lay_kind = value
        .get("lay_kind")
        .and_then(Value::as_str)
        .unwrap_or("user-correction");
    let operation = classify_operation(from, to, lay_kind, "");
    let (train_role, quarantine_reason) =
        train_role_and_quarantine(signal, lay_kind, from, to, &operation);
    Some(DirtyLogPair {
        kind: "dirty_log_pair_v1",
        ts: value.get("ts").and_then(Value::as_u64).unwrap_or(0),
        source_log: "corrections",
        signal,
        train_role,
        quarantine_reason,
        original: from.to_string(),
        expected: to.to_string(),
        operation,
        count_weight: value
            .get("replace_words")
            .or_else(|| value.get("words"))
            .and_then(Value::as_u64)
            .unwrap_or(1)
            .max(1) as usize,
        replace_words: value
            .get("replace_words")
            .or_else(|| value.get("words"))
            .and_then(Value::as_u64)
            .unwrap_or(1)
            .max(1) as usize,
        candidate_count: 0,
        source_id: lay_kind.to_string(),
        error_class: String::new(),
        action_operator: String::new(),
        action_proof: String::new(),
        posterior_milli: None,
        decision_rank_milli: None,
        risk_milli: None,
        usage_prior_milli: None,
        context_prior_milli: None,
        l2_wave_peak_milli: None,
        l2_wave_peak_reason: String::new(),
        l3_phrase_milli: None,
        l3_phrase_decision: String::new(),
        l4_signed_milli: None,
        l4_signed_reason: String::new(),
        transition_verified: None,
        left_context_changed: false,
        boundary_changed: word_count_changed(from, to),
        word_count_changed: word_count_changed(from, to),
        safety_allow_apply: None,
        safety_reason: String::new(),
    })
}

fn inspect_candidate_before_apply(collector: &mut Collector, value: &Value) {
    let safety_allow_apply = value
        .get("safety_allow_apply")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let boundary_changed = value
        .get("boundary_changed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let left_context_changed = value
        .get("transition_left_context_changed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let word_count_changed = value
        .get("word_count_changed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let safety_reason = value
        .get("safety_reason")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !safety_allow_apply || boundary_changed || left_context_changed {
        collector.unsafe_edit_plan += 1;
    }
    if boundary_changed || word_count_changed {
        collector.boundary_changed += 1;
    }
    if left_context_changed {
        collector.left_context_changed += 1;
    }
    if !safety_reason.is_empty() {
        *collector
            .by_safety_reason
            .entry(safety_reason.to_string())
            .or_default() += 1;
    }
}

impl Collector {
    fn add(&mut self, pair: DirtyLogPair) {
        *self.by_signal.entry(pair.signal.to_string()).or_default() += 1;
        *self
            .by_train_role
            .entry(pair.train_role.to_string())
            .or_default() += 1;
        if !pair.quarantine_reason.is_empty() {
            *self
                .by_quarantine_reason
                .entry(pair.quarantine_reason.clone())
                .or_default() += 1;
        }
        *self.by_operation.entry(pair.operation.clone()).or_default() += 1;
        if !pair.source_id.is_empty() {
            *self.by_source_id.entry(pair.source_id.clone()).or_default() += 1;
        }
        if !pair.error_class.is_empty() {
            *self
                .by_error_class
                .entry(pair.error_class.clone())
                .or_default() += 1;
        }
        if pair.posterior_milli.is_some_and(|value| value < 520) {
            self.low_posterior += 1;
        }
        if pair.risk_milli.is_some_and(|value| value >= 250) {
            self.high_risk += 1;
        }
        if pair.boundary_changed {
            self.boundary_changed += 1;
        }
        if pair.left_context_changed {
            self.left_context_changed += 1;
        }
        self.pairs.push(pair);
    }
}

fn report_json(
    report: &Collector,
    recent_path: Option<PathBuf>,
    corrections_path: Option<PathBuf>,
    out: Option<PathBuf>,
) -> Value {
    let samples = report
        .pairs
        .iter()
        .take(SAMPLE_LIMIT)
        .map(|pair| {
            json!({
                "signal": pair.signal,
                "train_role": pair.train_role,
                "quarantine_reason": pair.quarantine_reason,
                "operation": pair.operation,
                "source_id": pair.source_id,
                "original": pair.original,
                "expected": pair.expected,
                "posterior_milli": pair.posterior_milli,
                "l2_wave_peak_reason": pair.l2_wave_peak_reason,
                "l3_phrase_decision": pair.l3_phrase_decision,
                "l4_signed_reason": pair.l4_signed_reason
            })
        })
        .collect::<Vec<_>>();
    json!({
        "kind": "dirty_log_collect_report",
        "status": "ok",
        "read_as": "dirty-log pair corpus for L2/L3/L4 replay and learned transition memory; diagnostic only",
        "sources": {
            "recent_actions": recent_path.map(|path| path.display().to_string()),
            "corrections": corrections_path.map(|path| path.display().to_string())
        },
        "window": {
            "recent_raw_lines": report.recent.raw_lines,
            "recent_invalid_lines": report.recent.invalid_lines,
            "recent_skipped_pairs": report.recent.skipped_pairs,
            "correction_raw_lines": report.corrections.raw_lines,
            "correction_invalid_lines": report.corrections.invalid_lines,
            "correction_skipped_pairs": report.corrections.skipped_pairs
        },
        "pairs": {
            "total": report.pairs.len(),
            "by_signal": report.by_signal,
            "by_train_role": report.by_train_role,
            "by_quarantine_reason": report.by_quarantine_reason,
            "by_operation": report.by_operation
        },
        "candidate_lanes": {
            "by_source_id": report.by_source_id,
            "by_error_class": report.by_error_class,
            "low_posterior": report.low_posterior,
            "high_risk": report.high_risk
        },
        "edit_safety": {
            "unsafe_edit_plan": report.unsafe_edit_plan,
            "boundary_changed": report.boundary_changed,
            "left_context_changed": report.left_context_changed,
            "by_safety_reason": report.by_safety_reason
        },
        "out": out.map(|path| path.display().to_string()),
        "written": report.pairs.len(),
        "samples": samples
    })
}

fn write_pairs(path: &Path, pairs: &[DirtyLogPair]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut text = String::new();
    for pair in pairs {
        text.push_str(&serde_json::to_string(pair)?);
        text.push('\n');
    }
    fs::write(path, text)
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

fn selected_i64(selected: Option<&Value>, key: &str) -> Option<i64> {
    selected?.get(key)?.as_i64()
}

fn selected_string(selected: Option<&Value>, key: &str) -> String {
    selected
        .and_then(|item| item.get(key))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn classify_operation(
    original: &str,
    expected: &str,
    source_id: &str,
    error_class: &str,
) -> String {
    if source_id.contains("layout") || error_class == "layout" {
        return "layout".to_string();
    }
    if word_count_changed(original, expected) {
        return "boundary".to_string();
    }
    if is_adjacent_transposition(original, expected) {
        return "transposition".to_string();
    }
    if original
        .trim()
        .chars()
        .count()
        .abs_diff(expected.trim().chars().count())
        == 1
    {
        return "single_char_edit".to_string();
    }
    if has_mixed_layout(original) || has_mixed_layout(expected) {
        return "mixed_layout".to_string();
    }
    "surface_typo".to_string()
}

fn train_role_and_quarantine(
    signal: &'static str,
    source_id: &str,
    original: &str,
    expected: &str,
    operation: &str,
) -> (&'static str, String) {
    if signal == "user_rejected_lay_output" {
        return ("negative", String::new());
    }
    if source_id == "smart-text" && signal == "user_accepted_fix" {
        return ("review", "external_smart_text_source".to_string());
    }
    if operation == "boundary" && signal != "manual_layout_replay" {
        return ("review", "boundary_needs_transition_proof".to_string());
    }
    if operation == "mixed_layout" && signal != "manual_layout_replay" {
        return ("review", "mixed_layout_dirty_pair".to_string());
    }
    if original
        .split_whitespace()
        .count()
        .max(expected.split_whitespace().count())
        > 4
    {
        return ("review", "wide_context_pair".to_string());
    }
    ("positive", String::new())
}

fn is_adjacent_transposition(original: &str, expected: &str) -> bool {
    let original = original.trim().chars().collect::<Vec<_>>();
    let expected = expected.trim().chars().collect::<Vec<_>>();
    if original.len() != expected.len() || original.len() < 2 {
        return false;
    }
    let diffs = original
        .iter()
        .zip(expected.iter())
        .enumerate()
        .filter_map(|(index, (left, right))| (left != right).then_some(index))
        .collect::<Vec<_>>();
    diffs.len() == 2
        && diffs[1] == diffs[0] + 1
        && original[diffs[0]] == expected[diffs[1]]
        && original[diffs[1]] == expected[diffs[0]]
}

fn has_mixed_layout(text: &str) -> bool {
    let has_latin = text.chars().any(|ch| ch.is_ascii_alphabetic());
    let has_cyrillic = text.chars().any(lay::keyboard::is_cyrillic_letter);
    has_latin && has_cyrillic
}

fn valid_pair(original: &str, expected: &str) -> bool {
    let original = original.trim();
    let expected = expected.trim();
    !original.is_empty()
        && !expected.is_empty()
        && original != expected
        && original.chars().count() <= MAX_TEXT_CHARS
        && expected.chars().count() <= MAX_TEXT_CHARS
        && original.split_whitespace().count().max(1) <= MAX_WORDS
        && expected.split_whitespace().count().max(1) <= MAX_WORDS
        && !contains_unsafe_training_text(original)
        && !contains_unsafe_training_text(expected)
}

fn contains_unsafe_training_text(text: &str) -> bool {
    text.contains("://")
        || text.contains('@')
        || text.contains('=')
        || text.chars().any(char::is_control)
        || text
            .split_whitespace()
            .any(|token| token.chars().filter(|ch| ch.is_ascii_punctuation()).count() >= 4)
}

fn word_count_changed(original: &str, expected: &str) -> bool {
    original.split_whitespace().count() != expected.split_whitespace().count()
}

fn bounded_tail_lines(text: &str, limit: usize) -> Vec<&str> {
    let lines = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    let start = lines.len().saturating_sub(limit.max(1));
    lines[start..].to_vec()
}

fn arg_value(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find_map(|pair| (pair[0] == name).then(|| pair[1].clone()))
}

fn home_path(relative: &str) -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(relative))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_typing_assist_with_selected_candidate_lanes() {
        let text = r#"{"ts":1,"kind":"typing-assist","from":"звгрузи ","to":"загрузи ","replace_words":1,"input_gate":{"candidate_count":1,"selected_source_id":"L2SurfaceMotifCell32","selected_error_class":"composite-typo","scoreboard":{"selected_bayes_posterior_milli":640},"candidate_scores":[{"replacement":"загрузи ","source_id":"L2SurfaceMotifCell32","error_class":"composite-typo","action_operator":"fix_typo","action_proof":"typo","posterior_milli":640,"decision_rank_milli":800,"risk_milli":0,"usage_prior_milli":120,"context_prior_milli":80,"l2_wave_peak_milli":900,"l2_wave_peak_reason":"l2_wave_center_resonance","l3_phrase_milli":30,"l3_phrase_decision":"support","l4_signed_milli":20,"l4_signed_reason":"learned_transition_attracts","selected":true}]}}"#;
        let mut collector = Collector::default();
        collect_recent_text(&mut collector, text, 100);
        assert_eq!(collector.pairs.len(), 1);
        let pair = &collector.pairs[0];
        assert_eq!(pair.signal, "applied_observed");
        assert_eq!(pair.operation, "surface_typo");
        assert_eq!(pair.source_id, "L2SurfaceMotifCell32");
        assert_eq!(pair.posterior_milli, Some(640));
        assert_eq!(pair.l3_phrase_decision, "support");
    }

    #[test]
    fn collects_user_accepted_and_rejected_pairs() {
        let text = r#"{"ts":2,"kind":"user-correction","from":"рвено ","to":"верно ","replace_words":1,"lay_kind":"typing-assist","lay_from":"верно ","lay_to":"рвено "}"#;
        let mut collector = Collector::default();
        collect_corrections_text(&mut collector, text, 100);
        assert_eq!(collector.pairs.len(), 2);
        assert_eq!(collector.by_signal["user_accepted_fix"], 1);
        assert_eq!(collector.by_signal["user_rejected_lay_output"], 1);
    }

    #[test]
    fn collects_layout_replay_as_manual_transition() {
        let text =
            r#"{"ts":3,"kind":"layout-replay","from":"ghbdtn","to":"привет","replace_words":1}"#;
        let mut collector = Collector::default();
        collect_recent_text(&mut collector, text, 100);
        assert_eq!(collector.pairs.len(), 1);
        assert_eq!(collector.pairs[0].signal, "manual_layout_replay");
        assert_eq!(collector.pairs[0].operation, "layout");
    }

    #[test]
    fn tracks_unsafe_edit_plan_without_training_it_as_positive() {
        let text = r#"{"ts":4,"kind":"candidate_before_apply","from":"два слова","to":"одно","boundary_changed":true,"transition_left_context_changed":true,"safety_allow_apply":false,"safety_reason":"low_confidence_wide_edit"}"#;
        let mut collector = Collector::default();
        collect_recent_text(&mut collector, text, 100);
        assert!(collector.pairs.is_empty());
        assert_eq!(collector.unsafe_edit_plan, 1);
        assert_eq!(collector.boundary_changed, 1);
        assert_eq!(collector.left_context_changed, 1);
        assert_eq!(collector.by_safety_reason["low_confidence_wide_edit"], 1);
    }
}
