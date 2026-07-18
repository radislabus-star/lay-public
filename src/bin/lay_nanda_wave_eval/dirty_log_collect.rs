use lay::config::{default_typing_assist_pipeline, CorrectionSafety, TypingAssistRuleConfig};
use lay::correction_core::{
    resolve_text_correction, CandidateGateAction, CorrectionDecisionSource, CorrectionMode,
    CorrectionRequest, CorrectionResolution, UnifiedCorrectionCandidate,
};
use lay::nanda_wave::WaveOptions;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const DEFAULT_RECENT_ACTIONS_PATH: &str = ".local/share/lay/recent_actions.jsonl";
const DEFAULT_CORRECTIONS_PATH: &str = ".local/share/lay/corrections.jsonl";
const DEFAULT_LIMIT: usize = 2_000;
const DEFAULT_REPLAY_LIMIT: usize = 30;
const MAX_TEXT_CHARS: usize = 180;
const MAX_WORDS: usize = 12;
const SAMPLE_LIMIT: usize = 18;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DirtyLogPair {
    kind: String,
    ts: u64,
    source_log: String,
    signal: String,
    train_role: String,
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
    #[serde(default)]
    l2_transition_phase_milli: Option<i64>,
    #[serde(default)]
    l2_transition_phase_verdict: String,
    #[serde(default)]
    l2_transition_phase_surfaces: Option<u64>,
    l3_phrase_milli: Option<i64>,
    l3_phrase_decision: String,
    l4_signed_milli: Option<i64>,
    l4_signed_reason: String,
    #[serde(default)]
    l4_surface_status: String,
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

pub(crate) fn print_replay_json(args: &[String], options: &WaveOptions) -> io::Result<()> {
    let limit = arg_value(args, "--limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(DEFAULT_REPLAY_LIMIT);
    let max_examples = arg_value(args, "--max-examples")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(SAMPLE_LIMIT);
    let max_eval = arg_value(args, "--max-eval")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(limit);
    let train_role = arg_value(args, "--train-role").unwrap_or_else(|| "all".to_string());
    let input = arg_value(args, "--input").map(PathBuf::from);
    let phase_memory = arg_value(args, "--l2-phase-memory").map(PathBuf::from);
    let mut pairs = if let Some(path) = input.as_deref() {
        read_pairs(path, limit)?
    } else {
        collect(
            home_path(DEFAULT_RECENT_ACTIONS_PATH).as_deref(),
            home_path(DEFAULT_CORRECTIONS_PATH).as_deref(),
            limit,
        )?
        .pairs
    };
    let latest_state_only = args.iter().any(|arg| arg == "--latest-state-only");
    if latest_state_only {
        pairs = latest_transition_states(pairs);
    }
    if train_role != "all" {
        pairs.retain(|pair| pair.train_role == train_role);
    }
    pairs.truncate(max_eval);
    let phase_evaluator =
        lay::nanda_wave::L2TransitionPhaseShadowEvaluator::load(phase_memory.as_deref());
    if args.iter().any(|arg| arg == "--phase-only") {
        let phase_report = replay_phase_pairs(&pairs, &phase_evaluator, max_examples);
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "kind": "dirty_log_transition_phase_replay",
                "status": "ok",
                "input": input.map(|path| path.display().to_string()).unwrap_or_else(|| "live_logs".to_string()),
                "pairs": pairs.len(),
                "latest_state_only": latest_state_only,
                "read_as": "fast phase-only replay; full L2/L3 candidate generation is intentionally skipped",
                "transition_phase_shadow": phase_replay_report_json(&phase_report, phase_memory),
            }))?
        );
        return Ok(());
    }
    let report = replay_pairs(&pairs, options, max_examples, &phase_evaluator);
    println!(
        "{}",
        serde_json::to_string_pretty(&replay_report_json(
            &report,
            input,
            phase_memory,
            pairs.len(),
            latest_state_only,
        ))?
    );
    Ok(())
}

fn latest_transition_states(pairs: Vec<DirtyLogPair>) -> Vec<DirtyLogPair> {
    let mut latest = BTreeMap::<String, DirtyLogPair>::new();
    for pair in pairs {
        let key = transition_state_replay_key(&pair);
        match latest.get(&key) {
            Some(existing) if existing.ts > pair.ts => {}
            _ => {
                latest.insert(key, pair);
            }
        }
    }
    latest.into_values().collect()
}

fn transition_state_replay_key(pair: &DirtyLogPair) -> String {
    let operation = if pair.action_operator.trim().is_empty() {
        pair.operation.as_str()
    } else {
        pair.action_operator.as_str()
    };
    format!(
        "{}\u{1e}{}\u{1e}{}",
        pair.original.trim().to_lowercase(),
        pair.expected.trim().to_lowercase(),
        lay::nanda_wave::infer_l2_transition_operator(&pair.original, &pair.expected, operation,)
    )
}

pub(crate) fn pack_usage_json(args: &[String]) -> io::Result<()> {
    let Some(input) = arg_value(args, "--input").map(PathBuf::from) else {
        eprintln!("--dirty-log-pack-usage requires --input PATH");
        return Ok(());
    };
    let Some(out) = arg_value(args, "--out").map(PathBuf::from) else {
        eprintln!("--dirty-log-pack-usage requires --out PATH");
        return Ok(());
    };
    let limit = arg_value(args, "--limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(DEFAULT_LIMIT);
    let mut pairs = read_pairs(&input, limit)?;
    let latest_state_only = args.iter().any(|arg| arg == "--latest-state-only");
    if latest_state_only {
        pairs = latest_transition_states(pairs);
    }
    let events = usage_events_from_pairs(&pairs);
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut text = String::new();
    for event in &events {
        text.push_str(&serde_json::to_string(event)?);
        text.push('\n');
    }
    fs::write(&out, text)?;
    let accepted = events
        .iter()
        .filter(|event| event.get("kind").and_then(Value::as_str) == Some("accepted_fix"))
        .count();
    let rejected = events
        .iter()
        .filter(|event| event.get("kind").and_then(Value::as_str) == Some("rejected_candidate"))
        .count();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "kind": "dirty_log_usage_pack_report",
            "latest_state_only": latest_state_only,
            "status": "ok",
            "input": input.display().to_string(),
            "out": out.display().to_string(),
            "pairs": pairs.len(),
            "events": events.len(),
            "accepted_fix_events": accepted,
            "rejected_candidate_events": rejected,
            "read_as": "shadow usage-event pack from dirty-log pairs; does not mutate live user memory"
        }))?
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
    collector.reconcile_conflicting_evidence();
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
            if !is_exact_lay_undo(&value) {
                let pair = full_user_target(&value)
                    .and_then(|target| {
                        let lay_from = value.get("lay_from")?.as_str()?;
                        pair_from_correction_text(&value, "user_accepted_fix", lay_from, &target)
                    })
                    .or_else(|| pair_from_correction(&value, "user_accepted_fix", "from", "to"));
                if let Some(pair) = pair {
                    collector.add(pair);
                } else {
                    collector.corrections.skipped_pairs += 1;
                }
            }
            if value.get("lay_from").is_some()
                && value.get("lay_to").is_some()
                && !lay_output_matches_user_target(&value)
            {
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
        } else if kind == "layout-replay" {
            if let Some(pair) = pair_from_layout_replay(&value) {
                collector.add(pair);
            } else {
                collector.corrections.skipped_pairs += 1;
            }
        }
    }
}

fn is_exact_lay_undo(value: &Value) -> bool {
    let Some(lay_from) = value.get("lay_from").and_then(Value::as_str) else {
        return false;
    };
    full_user_target(value).is_some_and(|target| {
        normalized_transition_side(&target) == normalized_transition_side(lay_from)
    })
}

fn lay_output_matches_user_target(value: &Value) -> bool {
    let Some(lay_to) = value.get("lay_to").and_then(Value::as_str) else {
        return false;
    };
    full_user_target(value).is_some_and(|target| {
        normalized_transition_side(&target) == normalized_transition_side(lay_to)
    })
}

fn full_user_target(value: &Value) -> Option<String> {
    if let Some(target) = value.get("user_target").and_then(Value::as_str) {
        return Some(target.to_string());
    }
    lay::word_buffer::reconstruct_user_correction_target(
        value.get("lay_to")?.as_str()?,
        value.get("from")?.as_str()?,
        value.get("to")?.as_str()?,
    )
}

fn normalized_transition_side(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
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
        kind: "dirty_log_pair_v1".to_string(),
        ts: value.get("ts").and_then(Value::as_u64).unwrap_or(0),
        source_log: "recent_actions".to_string(),
        signal: signal.to_string(),
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
        l2_transition_phase_milli: selected_i64(selected, "l2_transition_phase_milli"),
        l2_transition_phase_verdict: selected_string(selected, "l2_transition_phase_verdict"),
        l2_transition_phase_surfaces: selected
            .and_then(|item| item.get("l2_transition_phase_surfaces"))
            .and_then(Value::as_u64),
        l3_phrase_milli: selected_i64(selected, "l3_phrase_milli"),
        l3_phrase_decision: selected_string(selected, "l3_phrase_decision"),
        l4_signed_milli: selected_i64(selected, "l4_signed_milli"),
        l4_signed_reason: selected_string(selected, "l4_signed_reason"),
        l4_surface_status: selected_string(selected, "l4_surface_status"),
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
    let (train_role, quarantine_reason) = manual_layout_train_role(from, to);
    Some(DirtyLogPair {
        kind: "dirty_log_pair_v1".to_string(),
        ts: value.get("ts").and_then(Value::as_u64).unwrap_or(0),
        source_log: "recent_actions".to_string(),
        signal: "manual_layout_replay".to_string(),
        train_role,
        quarantine_reason,
        original: from.to_string(),
        expected: to.to_string(),
        operation: "replacement".to_string(),
        count_weight: 1,
        replace_words: value
            .get("replace_words")
            .or_else(|| value.get("words"))
            .and_then(Value::as_u64)
            .unwrap_or(1)
            .max(1) as usize,
        candidate_count: 0,
        source_id: "layout".to_string(),
        error_class: "layout".to_string(),
        action_operator: "flip_layout".to_string(),
        action_proof: "layout".to_string(),
        posterior_milli: None,
        decision_rank_milli: None,
        risk_milli: None,
        usage_prior_milli: None,
        context_prior_milli: None,
        l2_wave_peak_milli: None,
        l2_wave_peak_reason: String::new(),
        l2_transition_phase_milli: None,
        l2_transition_phase_verdict: String::new(),
        l2_transition_phase_surfaces: None,
        l3_phrase_milli: None,
        l3_phrase_decision: String::new(),
        l4_signed_milli: None,
        l4_signed_reason: String::new(),
        l4_surface_status: String::new(),
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
    pair_from_correction_text(value, signal, from, to)
}

fn pair_from_correction_text(
    value: &Value,
    signal: &'static str,
    from: &str,
    to: &str,
) -> Option<DirtyLogPair> {
    if !valid_pair(from, to) {
        return None;
    }
    let lay_kind = value
        .get("lay_kind")
        .and_then(Value::as_str)
        .unwrap_or("user-correction");
    let operation = classify_operation(from, to, lay_kind, "");
    let (source_id, action_operator) = evidence_operator_metadata(lay_kind, &operation);
    let (train_role, quarantine_reason) =
        train_role_and_quarantine(signal, &source_id, from, to, &operation);
    Some(DirtyLogPair {
        kind: "dirty_log_pair_v1".to_string(),
        ts: value.get("ts").and_then(Value::as_u64).unwrap_or(0),
        source_log: "corrections".to_string(),
        signal: signal.to_string(),
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
        source_id,
        error_class: String::new(),
        action_operator,
        action_proof: String::new(),
        posterior_milli: None,
        decision_rank_milli: None,
        risk_milli: None,
        usage_prior_milli: None,
        context_prior_milli: None,
        l2_wave_peak_milli: None,
        l2_wave_peak_reason: String::new(),
        l2_transition_phase_milli: None,
        l2_transition_phase_verdict: String::new(),
        l2_transition_phase_surfaces: None,
        l3_phrase_milli: None,
        l3_phrase_decision: String::new(),
        l4_signed_milli: None,
        l4_signed_reason: String::new(),
        l4_surface_status: String::new(),
        transition_verified: None,
        left_context_changed: false,
        boundary_changed: word_count_changed(from, to),
        word_count_changed: word_count_changed(from, to),
        safety_allow_apply: None,
        safety_reason: String::new(),
    })
}

/// Correction logs describe the outer component that emitted an event. L4
/// learns transition operators, so layout evidence must bind to `flip_layout`
/// rather than the generic `typing-assist` envelope.
fn evidence_operator_metadata(source_id: &str, operation: &str) -> (String, String) {
    match operation {
        "layout" => ("layout".to_string(), "flip_layout".to_string()),
        "boundary" => ("boundary".to_string(), "boundary_transition".to_string()),
        "transposition" => (source_id.to_string(), "adjacent_transposition".to_string()),
        _ => (source_id.to_string(), "replace_current_token".to_string()),
    }
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
        *self.by_signal.entry(pair.signal.clone()).or_default() += 1;
        *self
            .by_train_role
            .entry(pair.train_role.clone())
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

    /// A later manual replay can confirm the exact transition that an earlier
    /// correction record tentatively marked as rejected. Such contradictory
    /// evidence must be visible for review, never learned as both attraction
    /// and repulsion for the same state transition.
    fn reconcile_conflicting_evidence(&mut self) {
        let mut roles = BTreeMap::<(String, String), (bool, bool, bool)>::new();
        for pair in &self.pairs {
            let entry = roles
                .entry((pair.original.clone(), pair.expected.clone()))
                .or_default();
            entry.0 |= pair.train_role == "positive";
            entry.1 |= pair.train_role == "negative";
            entry.2 |= pair.signal == "manual_layout_replay";
        }
        for pair in &mut self.pairs {
            if roles
                .get(&(pair.original.clone(), pair.expected.clone()))
                .is_some_and(|(positive, negative, manual_replay)| {
                    *negative && (*positive || *manual_replay)
                })
            {
                pair.train_role = "review".to_string();
                pair.quarantine_reason = "conflicting_transition_feedback".to_string();
            }
        }
        self.by_train_role.clear();
        self.by_quarantine_reason.clear();
        for pair in &self.pairs {
            *self
                .by_train_role
                .entry(pair.train_role.clone())
                .or_default() += 1;
            if !pair.quarantine_reason.is_empty() {
                *self
                    .by_quarantine_reason
                    .entry(pair.quarantine_reason.clone())
                    .or_default() += 1;
            }
        }
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
                "l2_transition_phase_milli": pair.l2_transition_phase_milli,
                "l2_transition_phase_verdict": pair.l2_transition_phase_verdict,
                "l2_transition_phase_surfaces": pair.l2_transition_phase_surfaces,
                "l3_phrase_decision": pair.l3_phrase_decision,
                "l4_signed_reason": pair.l4_signed_reason,
                "l4_surface_status": pair.l4_surface_status
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

fn read_pairs(path: &Path, limit: usize) -> io::Result<Vec<DirtyLogPair>> {
    let text = fs::read_to_string(path)?;
    let mut pairs = Vec::new();
    for line in bounded_tail_lines(&text, limit) {
        if let Ok(pair) = serde_json::from_str::<DirtyLogPair>(line) {
            pairs.push(pair);
        }
    }
    Ok(pairs)
}

#[derive(Debug, Default)]
struct ReplayReport {
    positive: ReplayBucket,
    negative: ReplayBucket,
    review: usize,
    by_operation: BTreeMap<String, ReplayBucket>,
    by_source: BTreeMap<String, ReplayBucket>,
    examples: ReplayExamples,
    transition_phase: PhaseReplayReport,
    phase_apply_policy: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct ReplayBucket {
    cases: usize,
    with_l2_candidates: usize,
    no_l2_candidates: usize,
    expected_present: usize,
    expected_missing: usize,
    expected_top1: usize,
    applied_expected: usize,
    present_not_applied: usize,
    applied_other: usize,
    kept_or_vetoed: usize,
}

#[derive(Debug, Default)]
struct ReplayExamples {
    missing_expected: Vec<Value>,
    present_not_applied: Vec<Value>,
    applied_other: Vec<Value>,
    negative_applied: Vec<Value>,
}

#[derive(Debug, Default)]
struct PhaseReplayReport {
    positive: PhaseReplayBucket,
    negative: PhaseReplayBucket,
    structural_positive: PhaseReplayBucket,
    structural_negative: PhaseReplayBucket,
    context_positive: PhaseReplayBucket,
    context_negative: PhaseReplayBucket,
    by_operation: BTreeMap<String, PhaseReplayBucket>,
    examples: PhaseReplayExamples,
}

#[derive(Debug, Default, Clone, Copy)]
struct PhaseReplayBucket {
    cases: usize,
    support: usize,
    repel: usize,
    unknown: usize,
    package_missing: usize,
    operator_missing: usize,
}

#[derive(Debug, Default)]
struct PhaseReplayExamples {
    positive_unknown: Vec<Value>,
    positive_repelled: Vec<Value>,
    negative_supported: Vec<Value>,
}

fn replay_pairs(
    pairs: &[DirtyLogPair],
    options: &WaveOptions,
    max_examples: usize,
    phase_evaluator: &lay::nanda_wave::L2TransitionPhaseShadowEvaluator,
) -> ReplayReport {
    let mut report = ReplayReport {
        phase_apply_policy: options.l2_phase_apply(),
        ..ReplayReport::default()
    };
    let pipeline = default_typing_assist_pipeline();
    let mut resolution_cache = BTreeMap::new();
    for pair in pairs {
        if pair.train_role == "review" {
            report.review += 1;
            continue;
        }
        let resolution = resolution_cache
            .entry(pair.original.clone())
            .or_insert_with(|| canonical_replay_resolution(&pair.original, options, &pipeline));
        let outcome = replay_one(pair, resolution);
        let bucket = if pair.train_role == "negative" {
            &mut report.negative
        } else {
            &mut report.positive
        };
        bucket.add(outcome);
        report
            .by_operation
            .entry(pair.operation.clone())
            .or_default()
            .add(outcome);
        report
            .by_source
            .entry(pair.source_id.clone())
            .or_default()
            .add(outcome);
        collect_replay_examples(
            &mut report.examples,
            pair,
            resolution,
            outcome,
            max_examples,
        );
        add_phase_replay(
            &mut report.transition_phase,
            pair,
            phase_evaluator,
            max_examples,
        );
    }
    report
}

fn canonical_replay_resolution(
    original: &str,
    options: &WaveOptions,
    pipeline: &[TypingAssistRuleConfig],
) -> CorrectionResolution {
    resolve_text_correction(CorrectionRequest {
        text: original,
        auto_replace: true,
        typing_assist: true,
        auto_switch_layout: true,
        correction_safety: CorrectionSafety::Normal,
        typing_assist_pipeline: pipeline,
        nanda_autocorrect: true,
        nanda_candidate_route: lay::correction_core::CandidateReadoutRoute::FullWave,
        nanda_wave_options: options.clone(),
        mode: CorrectionMode::DeterministicThenNanda,
    })
}

fn replay_phase_pairs(
    pairs: &[DirtyLogPair],
    phase_evaluator: &lay::nanda_wave::L2TransitionPhaseShadowEvaluator,
    max_examples: usize,
) -> PhaseReplayReport {
    let mut report = PhaseReplayReport::default();
    for pair in pairs {
        if pair.train_role != "review" {
            add_phase_replay(&mut report, pair, phase_evaluator, max_examples);
        }
    }
    report
}

fn add_phase_replay(
    report: &mut PhaseReplayReport,
    pair: &DirtyLogPair,
    phase_evaluator: &lay::nanda_wave::L2TransitionPhaseShadowEvaluator,
    max_examples: usize,
) {
    let outcome = replay_phase_one(pair, phase_evaluator);
    let bucket = if pair.train_role == "negative" {
        &mut report.negative
    } else {
        &mut report.positive
    };
    bucket.add(&outcome);
    match phase_evidence_role(pair) {
        PhaseEvidenceRole::StructuralPositive => report.structural_positive.add(&outcome),
        PhaseEvidenceRole::StructuralNegative => report.structural_negative.add(&outcome),
        PhaseEvidenceRole::ContextPositive => report.context_positive.add(&outcome),
        PhaseEvidenceRole::ContextNegative => report.context_negative.add(&outcome),
        PhaseEvidenceRole::Review => {}
    }
    report
        .by_operation
        .entry(outcome.operator.clone())
        .or_default()
        .add(&outcome);
    collect_phase_examples(&mut report.examples, pair, &outcome, max_examples);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PhaseEvidenceRole {
    StructuralPositive,
    StructuralNegative,
    ContextPositive,
    ContextNegative,
    Review,
}

fn phase_evidence_role(pair: &DirtyLogPair) -> PhaseEvidenceRole {
    if pair.train_role == "review" {
        return PhaseEvidenceRole::Review;
    }
    if pair.safety_allow_apply == Some(false) || pair.transition_verified == Some(false) {
        return PhaseEvidenceRole::StructuralNegative;
    }
    if pair.signal == "manual_layout_replay" || pair.transition_verified == Some(true) {
        return PhaseEvidenceRole::StructuralPositive;
    }
    if pair.train_role == "negative" {
        PhaseEvidenceRole::ContextNegative
    } else {
        PhaseEvidenceRole::ContextPositive
    }
}

#[derive(Debug)]
struct PhaseReplayOutcome {
    operator: String,
    verdict: &'static str,
    package_loaded: bool,
    operator_present: bool,
    margin_micro: i64,
    threshold_micro: i64,
    positive_centers: u8,
    anti_centers: u8,
    covered_surfaces: u32,
    rejected_surfaces: u32,
}

impl PhaseReplayBucket {
    fn add(&mut self, outcome: &PhaseReplayOutcome) {
        self.cases += 1;
        match outcome.verdict {
            "support" => self.support += 1,
            "repel" => self.repel += 1,
            _ => self.unknown += 1,
        }
        if !outcome.package_loaded {
            self.package_missing += 1;
        } else if !outcome.operator_present {
            self.operator_missing += 1;
        }
    }
}

fn replay_phase_one(
    pair: &DirtyLogPair,
    phase_evaluator: &lay::nanda_wave::L2TransitionPhaseShadowEvaluator,
) -> PhaseReplayOutcome {
    let operation = if pair.action_operator.trim().is_empty()
        || pair.action_operator == "other"
        || pair.action_operator == "keep"
    {
        pair.operation.as_str()
    } else {
        pair.action_operator.as_str()
    };
    let readout = phase_evaluator.readout(&pair.original, &pair.expected, operation);
    PhaseReplayOutcome {
        operator: lay::nanda_wave::infer_l2_transition_operator(
            &pair.original,
            &pair.expected,
            operation,
        )
        .to_string(),
        verdict: readout.verdict,
        package_loaded: readout.package_loaded,
        operator_present: readout.operator_present,
        margin_micro: readout.margin_micro,
        threshold_micro: readout.threshold_micro,
        positive_centers: readout.positive_centers,
        anti_centers: readout.anti_centers,
        covered_surfaces: readout.covered_surfaces,
        rejected_surfaces: readout.rejected_surfaces,
    }
}

fn collect_phase_examples(
    examples: &mut PhaseReplayExamples,
    pair: &DirtyLogPair,
    outcome: &PhaseReplayOutcome,
    max_examples: usize,
) {
    let target = match (pair.train_role.as_str(), outcome.verdict) {
        ("positive", "unknown") => Some(&mut examples.positive_unknown),
        ("positive", "repel") => Some(&mut examples.positive_repelled),
        ("negative", "support") => Some(&mut examples.negative_supported),
        _ => None,
    };
    let Some(target) = target else {
        return;
    };
    if target.len() >= max_examples {
        return;
    }
    target.push(json!({
        "role": pair.train_role,
        "signal": pair.signal,
        "operator": outcome.operator,
        "original": pair.original,
        "expected": pair.expected,
        "verdict": outcome.verdict,
        "margin_micro": outcome.margin_micro,
        "threshold_micro": outcome.threshold_micro,
        "positive_centers": outcome.positive_centers,
        "anti_centers": outcome.anti_centers,
        "covered_surfaces": outcome.covered_surfaces,
        "rejected_surfaces": outcome.rejected_surfaces,
    }));
}

#[derive(Debug, Clone, Copy)]
struct ReplayOutcome {
    has_l2: bool,
    expected_present: bool,
    expected_top1: bool,
    applied_expected: bool,
    applied_other: bool,
}

impl ReplayBucket {
    fn add(&mut self, outcome: ReplayOutcome) {
        self.cases += 1;
        if outcome.has_l2 {
            self.with_l2_candidates += 1;
        } else {
            self.no_l2_candidates += 1;
        }
        if outcome.expected_present {
            self.expected_present += 1;
        } else {
            self.expected_missing += 1;
        }
        if outcome.expected_top1 {
            self.expected_top1 += 1;
        }
        if outcome.applied_expected {
            self.applied_expected += 1;
        } else if outcome.applied_other {
            self.applied_other += 1;
        } else {
            self.kept_or_vetoed += 1;
        }
        if outcome.expected_present && !outcome.applied_expected {
            self.present_not_applied += 1;
        }
    }
}

fn replay_one(pair: &DirtyLogPair, resolution: &CorrectionResolution) -> ReplayOutcome {
    let expected_present = resolution
        .candidates
        .iter()
        .any(|candidate| candidate.replacement == pair.expected);
    let output = resolution
        .decision
        .as_ref()
        .map(|decision| decision.replacement.as_str());
    let expected_top1 = output == Some(pair.expected.as_str());
    ReplayOutcome {
        has_l2: !resolution.candidates.is_empty(),
        expected_present,
        expected_top1,
        applied_expected: output == Some(pair.expected.as_str()),
        applied_other: output.is_some_and(|text| text != pair.expected),
    }
}

fn collect_replay_examples(
    examples: &mut ReplayExamples,
    pair: &DirtyLogPair,
    resolution: &CorrectionResolution,
    outcome: ReplayOutcome,
    max_examples: usize,
) {
    if !outcome.expected_present {
        push_example(
            &mut examples.missing_expected,
            pair,
            resolution,
            max_examples,
        );
    } else if !outcome.applied_expected {
        push_example(
            &mut examples.present_not_applied,
            pair,
            resolution,
            max_examples,
        );
    }
    if outcome.applied_other {
        push_example(&mut examples.applied_other, pair, resolution, max_examples);
    }
    if pair.train_role == "negative" && outcome.applied_expected {
        push_example(
            &mut examples.negative_applied,
            pair,
            resolution,
            max_examples,
        );
    }
}

fn push_example(
    target: &mut Vec<Value>,
    pair: &DirtyLogPair,
    resolution: &CorrectionResolution,
    max_examples: usize,
) {
    if target.len() >= max_examples {
        return;
    }
    target.push(json!({
        "role": pair.train_role,
        "signal": pair.signal,
        "operation": pair.operation,
        "source_id": pair.source_id,
        "original": pair.original,
        "expected": pair.expected,
        "decision": decision_label(resolution),
        "output": resolution.decision.as_ref().map(|decision| decision.replacement.as_str()).unwrap_or("keep"),
        "selected_candidate": resolution.selected.as_ref().map(candidate_short),
        "candidate_sources": resolution.candidates.iter().take(8).map(|candidate| candidate.source_id.as_str()).collect::<Vec<_>>(),
        "candidate_ladder": resolution.candidates.iter().take(8).map(|candidate| json!({
            "output": candidate.replacement,
            "source": candidate_source_label(candidate.source),
            "source_id": candidate.source_id,
            "error_class": candidate.error_class.as_str(),
            "gate": gate_action_label(candidate.gate.action),
            "gate_reason": candidate.gate.reason,
        })).collect::<Vec<_>>()
    }));
}

fn replay_report_json(
    report: &ReplayReport,
    input: Option<PathBuf>,
    phase_memory: Option<PathBuf>,
    pairs: usize,
    latest_state_only: bool,
) -> Value {
    json!({
        "kind": "dirty_log_replay_report",
        "status": "ok",
        "input": input.map(|path| path.display().to_string()).unwrap_or_else(|| "live_logs".to_string()),
        "pairs": pairs,
        "latest_state_only": latest_state_only,
        "l2_phase_apply_policy": report.phase_apply_policy,
        "read_as": "canonical runtime shadow: the live L2 candidate lattice and TransitionDecisionCore are evaluated without physical text output",
        "runtime_path": {
            "candidate_factory": "correction_core::resolve_text_correction/L2CandidateLattice",
            "decision_authority": "typing_transition::TransitionDecisionCore",
            "usage_memory": "configured hot UsagePriorSnapshot; environment path overrides are honored",
            "physical_output": false
        },
        "positive": replay_bucket_json(report.positive),
        "negative": replay_bucket_json(report.negative),
        "review": {
            "cases": report.review,
            "read_as": "quarantined pairs are not scored as positive authority"
        },
        "state_card": replay_state_card_json(report),
        "scoreboard": {
            "positive_top1_percent": percent(report.positive.expected_top1, report.positive.cases),
            "positive_apply_accuracy_percent": percent(report.positive.applied_expected, report.positive.cases),
            "positive_candidate_coverage_percent": percent(report.positive.expected_present, report.positive.cases),
            "positive_present_but_not_applied": report.positive.present_not_applied,
            "negative_false_apply": report.negative.applied_expected,
            "negative_false_apply_percent": percent(report.negative.applied_expected, report.negative.cases),
            "missing_candidate": report.positive.expected_missing,
            "applied_other": report.positive.applied_other,
            "gate": replay_gate(report)
        },
        "transition_phase_shadow": phase_replay_report_json(
            &report.transition_phase,
            phase_memory,
        ),
        "by_operation": report.by_operation.iter().map(|(key, bucket)| (key, replay_bucket_json(*bucket))).collect::<BTreeMap<_, _>>(),
        "by_source": report.by_source.iter().map(|(key, bucket)| (key, replay_bucket_json(*bucket))).collect::<BTreeMap<_, _>>(),
        "examples": {
            "missing_expected": report.examples.missing_expected,
            "present_not_applied": report.examples.present_not_applied,
            "applied_other": report.examples.applied_other,
            "negative_applied": report.examples.negative_applied
        }
    })
}

/// Compact, denominator-explicit view for humans. The detailed buckets remain
/// below for diagnosis; this card prevents a zero false-apply count from
/// hiding missed candidates, quarantined evidence, or alternate bad applies.
fn replay_state_card_json(report: &ReplayReport) -> Value {
    let scored_cases = report.positive.cases.saturating_add(report.negative.cases);
    json!({
        "evidence": {
            "positive_cases": report.positive.cases,
            "negative_cases": report.negative.cases,
            "quarantined_cases": report.review,
            "scored_cases": scored_cases,
            "quarantined_percent": percent(report.review, scored_cases.saturating_add(report.review)),
        },
        "candidate_birth": {
            "positive_coverage_percent": percent(report.positive.expected_present, report.positive.cases),
            "positive_missing_candidates": report.positive.expected_missing,
            "positive_no_l2_candidates": report.positive.no_l2_candidates,
        },
        "decision": {
            "positive_top1_percent": percent(report.positive.expected_top1, report.positive.cases),
            "positive_applied_expected": report.positive.applied_expected,
            "positive_present_but_not_applied": report.positive.present_not_applied,
        },
        "safety": {
            "negative_false_apply": report.negative.applied_expected,
            "negative_false_apply_percent": percent(report.negative.applied_expected, report.negative.cases),
            "negative_other_apply": report.negative.applied_other,
            "negative_other_apply_percent": percent(report.negative.applied_other, report.negative.cases),
            "verdict": if report.negative.applied_expected == 0 { "PASS" } else { "VETO" },
        },
        "read_as": "shadow-only: false apply measures the labelled rejected target; other apply is a separate unsafe-candidate debt"
    })
}

fn phase_replay_report_json(report: &PhaseReplayReport, path: Option<PathBuf>) -> Value {
    json!({
        "schema": "lay.l2-transition-phase-shadow.v1",
        "memory": path.map(|path| path.display().to_string()).unwrap_or_else(|| "live_default".to_string()),
        "apply_authority": false,
        "positive": phase_replay_bucket_json(report.positive),
        "negative": phase_replay_bucket_json(report.negative),
        "evidence_ownership": {
            "structural_positive": phase_replay_bucket_json(report.structural_positive),
            "structural_negative": phase_replay_bucket_json(report.structural_negative),
            "context_positive": phase_replay_bucket_json(report.context_positive),
            "context_negative": phase_replay_bucket_json(report.context_negative),
            "read_as": "L2 owns structural transition evidence; contextual accept/reject belongs to L3/L4 and cannot train L2 anti-centers",
        },
        "scoreboard": {
            "positive_support_percent": percent(report.positive.support, report.positive.cases),
            "positive_repel_percent": percent(report.positive.repel, report.positive.cases),
            "structural_negative_support": report.structural_negative.support,
            "structural_negative_support_percent": percent(report.structural_negative.support, report.structural_negative.cases),
            "context_negative_support_needs_l3_l4": report.context_negative.support,
            "negative_repel_percent": percent(report.negative.repel, report.negative.cases),
            "gate": phase_replay_gate(report),
        },
        "by_operation": report.by_operation.iter().map(|(key, bucket)| (key, phase_replay_bucket_json(*bucket))).collect::<BTreeMap<_, _>>(),
        "examples": {
            "positive_unknown": report.examples.positive_unknown,
            "positive_repelled": report.examples.positive_repelled,
            "negative_supported": report.examples.negative_supported,
        }
    })
}

fn phase_replay_bucket_json(bucket: PhaseReplayBucket) -> Value {
    json!({
        "cases": bucket.cases,
        "support": bucket.support,
        "repel": bucket.repel,
        "unknown": bucket.unknown,
        "package_missing": bucket.package_missing,
        "operator_missing": bucket.operator_missing,
    })
}

fn phase_replay_gate(report: &PhaseReplayReport) -> &'static str {
    if report.positive.package_missing > 0 || report.negative.package_missing > 0 {
        "WATCH-phase-package-missing"
    } else if report.structural_negative.support > 0 {
        "VETO-structural-negative-phase-support"
    } else if report.structural_positive.repel > 0 {
        "WATCH-structural-positive-phase-repel"
    } else if report.structural_positive.cases == 0 {
        "WATCH-no-live-structural-proof"
    } else if report.structural_positive.support * 2 < report.structural_positive.cases {
        "WATCH-low-structural-phase-coverage"
    } else {
        "PASS-shadow"
    }
}

fn usage_events_from_pairs(pairs: &[DirtyLogPair]) -> Vec<Value> {
    let mut events = Vec::new();
    for pair in pairs {
        match pair.train_role.as_str() {
            "positive" => events.extend(accepted_usage_events(pair)),
            "negative" => events.extend(rejected_usage_events(pair)),
            _ => {}
        }
    }
    events
}

fn accepted_usage_events(pair: &DirtyLogPair) -> Vec<Value> {
    let expected_words = normalized_words(&pair.expected);
    if expected_words.is_empty() {
        return Vec::new();
    }
    let original_words = normalized_words(&pair.original);
    let indexes = changed_word_indexes(&original_words, &expected_words);
    let surface = transition_surface(pair);
    indexes
        .into_iter()
        .filter_map(|index| {
            let word = expected_words.get(index)?;
            Some(json!({
                "ts": pair.ts,
                "kind": "accepted_fix",
                "word": word,
                "context": words_before_index(&expected_words, index),
                "from": pair.original.trim(),
                "to": pair.expected.trim(),
                "source": pair.source_id,
                "operation": runtime_memory_operation(pair),
                "surface": surface
            }))
        })
        .collect()
}

fn rejected_usage_events(pair: &DirtyLogPair) -> Vec<Value> {
    let rejected_words = normalized_words(&pair.expected);
    if rejected_words.is_empty() {
        return Vec::new();
    }
    let original_words = normalized_words(&pair.original);
    let surface = transition_surface(pair);
    changed_word_indexes(&original_words, &rejected_words)
        .into_iter()
        .filter_map(|index| {
            let word = rejected_words.get(index)?;
            Some(json!({
                "ts": pair.ts,
                "kind": "rejected_candidate",
                "word": word,
                "context": words_before_index(&original_words, index.min(original_words.len())),
                "from": pair.original.trim(),
                "to": pair.expected.trim(),
                "source": pair.source_id,
                "operation": runtime_memory_operation(pair),
                "surface": surface
            }))
        })
        .collect()
}

/// The decision core evaluates all correction candidates through the same
/// replacement transition. `DirtyLogPair::operation` remains an analytical
/// class; the learned L4 event must use the runtime operation key.
fn runtime_memory_operation(_pair: &DirtyLogPair) -> &'static str {
    "replacement"
}

fn transition_surface(pair: &DirtyLogPair) -> String {
    lay::nanda_wave::transition_surface_key(
        &pair.original,
        &pair.expected,
        &pair.source_id,
        runtime_memory_operation(pair),
    )
}

fn changed_word_indexes(original_words: &[String], expected_words: &[String]) -> Vec<usize> {
    let indexes = expected_words
        .iter()
        .enumerate()
        .filter_map(|(index, word)| (original_words.get(index) != Some(word)).then_some(index))
        .collect::<Vec<_>>();
    if indexes.is_empty() {
        expected_words.len().checked_sub(1).into_iter().collect()
    } else {
        indexes
    }
}

fn words_before_index(words: &[String], index: usize) -> Vec<String> {
    words[..index]
        .iter()
        .rev()
        .take(5)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn normalized_words(text: &str) -> Vec<String> {
    text.split_whitespace().filter_map(normalize_word).collect()
}

fn normalize_word(token: &str) -> Option<String> {
    let trimmed = token.trim_matches(|ch: char| !ch.is_alphabetic() && ch != '-');
    let alpha = trimmed.chars().filter(|ch| ch.is_alphabetic()).count();
    (alpha >= 1).then(|| trimmed.to_lowercase())
}

fn replay_bucket_json(bucket: ReplayBucket) -> Value {
    json!({
        "cases": bucket.cases,
        "with_l2_candidates": bucket.with_l2_candidates,
        "no_l2_candidates": bucket.no_l2_candidates,
        "expected_present": bucket.expected_present,
        "expected_missing": bucket.expected_missing,
        "expected_top1": bucket.expected_top1,
        "applied_expected": bucket.applied_expected,
        "present_not_applied": bucket.present_not_applied,
        "applied_other": bucket.applied_other,
        "kept_or_vetoed": bucket.kept_or_vetoed
    })
}

fn replay_gate(report: &ReplayReport) -> &'static str {
    if report.negative.applied_expected > 0 {
        "WATCH-negative-false-apply"
    } else if report.positive.expected_present < report.positive.cases / 2 {
        "WATCH-low-candidate-coverage"
    } else if report.positive.present_not_applied > report.positive.applied_expected {
        "WATCH-arbitration"
    } else {
        "PASS-shadow"
    }
}

fn percent(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    (numerator as f64 * 100.0 / denominator as f64 * 100.0).round() / 100.0
}

fn decision_label(resolution: &CorrectionResolution) -> &'static str {
    if resolution.decision.is_some() {
        "apply"
    } else if resolution
        .candidates
        .iter()
        .any(|candidate| candidate.gate.action == CandidateGateAction::Veto)
    {
        "veto"
    } else if resolution
        .candidates
        .iter()
        .any(|candidate| candidate.gate.action == CandidateGateAction::SuggestOnly)
    {
        "suggest"
    } else {
        "keep"
    }
}

fn candidate_short(candidate: &UnifiedCorrectionCandidate) -> String {
    format!(
        "{}:{:?}:{}:{}",
        candidate.source_id,
        candidate.replacement,
        candidate.error_class.as_str(),
        gate_action_label(candidate.gate.action),
    )
}

fn candidate_source_label(source: CorrectionDecisionSource) -> &'static str {
    match source {
        CorrectionDecisionSource::Deterministic => "deterministic",
        CorrectionDecisionSource::Nanda => "nanda",
    }
}

fn gate_action_label(action: CandidateGateAction) -> &'static str {
    match action {
        CandidateGateAction::Eligible => "eligible",
        CandidateGateAction::SuggestOnly => "suggest_only",
        CandidateGateAction::KeepOriginal => "keep_original",
        CandidateGateAction::Veto => "veto",
    }
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
    if source_id.contains("layout")
        || error_class == "layout"
        || is_exact_layout_projection(original, expected)
    {
        return "layout".to_string();
    }
    if word_count_changed(original, expected) {
        return "boundary".to_string();
    }
    if lay::text_metrics::is_adjacent_transposition(original.trim(), expected.trim()) {
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
) -> (String, String) {
    if signal == "user_rejected_lay_output" {
        return ("negative".to_string(), String::new());
    }
    if source_id == "smart-text" && signal == "user_accepted_fix" {
        return (
            "review".to_string(),
            "external_smart_text_source".to_string(),
        );
    }
    if operation == "boundary" && signal != "manual_layout_replay" {
        return (
            "review".to_string(),
            "boundary_needs_transition_proof".to_string(),
        );
    }
    if operation == "mixed_layout" && signal != "manual_layout_replay" {
        return ("review".to_string(), "mixed_layout_dirty_pair".to_string());
    }
    if original
        .split_whitespace()
        .count()
        .max(expected.split_whitespace().count())
        > 4
    {
        return ("review".to_string(), "wide_context_pair".to_string());
    }
    if signal != "manual_layout_replay" && !positive_pair_is_trainable(original, expected) {
        return (
            "review".to_string(),
            "nonlocal_user_edit_not_candidate_evidence".to_string(),
        );
    }
    ("positive".to_string(), String::new())
}

fn positive_pair_is_trainable(original: &str, expected: &str) -> bool {
    let original_words = normalized_words(original);
    let expected_words = normalized_words(expected);
    if original_words.is_empty() || original_words.len() != expected_words.len() {
        return false;
    }
    let changed = original_words
        .iter()
        .zip(&expected_words)
        .filter(|(left, right)| left != right)
        .collect::<Vec<_>>();
    let [(original_word, expected_word)] = changed.as_slice() else {
        return false;
    };
    let min_len = original_word
        .chars()
        .count()
        .min(expected_word.chars().count());
    if min_len < 3 {
        return false;
    }
    if is_exact_layout_projection(original_word, expected_word) {
        return true;
    }
    if has_mixed_layout(original_word) || has_mixed_layout(expected_word) {
        return false;
    }
    let distance = lay::text_metrics::damerau_levenshtein(original_word, expected_word);
    let max_distance = if min_len >= 7 { 3 } else { 2 };
    !known_training_surface(original_word)
        && known_training_surface(expected_word)
        && distance <= max_distance
        && original_word
            .chars()
            .count()
            .abs_diff(expected_word.chars().count())
            <= 2
}

fn is_exact_layout_projection(original: &str, expected: &str) -> bool {
    let original = original.trim();
    let expected = expected.trim();
    !original.is_empty()
        && !expected.is_empty()
        && lay::dict::convert(original, lay::dict::detect_direction(original))
            .eq_ignore_ascii_case(expected)
}

fn known_training_surface(word: &str) -> bool {
    let lower = word.to_lowercase();
    lay::russian_lexicon::is_known_russian_word_or_form(&lower)
        || lay::lexicon::is_common_ru_word(&lower)
        || lay::lexicon::is_common_en_technical_word(&lower)
        || lay::lexicon::is_ru_technical_loanword(&lower)
}

fn manual_layout_train_role(original: &str, expected: &str) -> (String, String) {
    if manual_layout_pair_is_trainable(original, expected) {
        ("positive".to_string(), String::new())
    } else {
        (
            "review".to_string(),
            "manual_layout_short_or_technical".to_string(),
        )
    }
}

fn manual_layout_pair_is_trainable(original: &str, expected: &str) -> bool {
    let original = original.trim();
    let expected = expected.trim();
    original.split_whitespace().count() == 1
        && expected.split_whitespace().count() == 1
        && original.chars().all(|ch| ch.is_alphabetic())
        && expected.chars().all(|ch| ch.is_alphabetic())
        && original.chars().filter(|ch| ch.is_alphabetic()).count() >= 4
        && expected.chars().filter(|ch| ch.is_alphabetic()).count() >= 4
        && !looks_like_uppercase_technical_token(original)
        && !looks_like_uppercase_technical_token(expected)
}

fn looks_like_uppercase_technical_token(token: &str) -> bool {
    let letters = token
        .chars()
        .filter(|ch| ch.is_alphabetic())
        .collect::<Vec<_>>();
    letters.len() >= 2 && letters.iter().all(|ch| ch.is_uppercase())
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
        assert_eq!(collector.pairs.len(), 1);
        assert_eq!(collector.by_signal.get("user_accepted_fix"), None);
        assert_eq!(collector.by_signal["user_rejected_lay_output"], 1);
    }

    #[test]
    fn non_inverse_user_correction_keeps_positive_feedback() {
        let text = r#"{"ts":2,"kind":"user-correction","from":"рвено ","to":"верно ","replace_words":1,"lay_kind":"typing-assist","lay_from":"ревно ","lay_to":"рвено "}"#;
        let mut collector = Collector::default();
        collect_corrections_text(&mut collector, text, 100);
        assert_eq!(collector.pairs.len(), 2);
        assert_eq!(collector.by_signal["user_accepted_fix"], 1);
        assert_eq!(collector.by_signal["user_rejected_lay_output"], 1);
    }

    #[test]
    fn suffix_feedback_preserves_review_and_negative_transitions() {
        let text = r#"{"ts":2,"kind":"user-correction","from":"ло? ","to":"льно? ","replace_words":1,"lay_kind":"typing-assist","lay_from":"Праивльно? ","lay_to":"Правило? "}"#;
        let mut collector = Collector::default();
        collect_corrections_text(&mut collector, text, 100);

        assert!(collector.pairs.iter().any(|pair| {
            pair.train_role == "review"
                && pair.original == "Праивльно? "
                && pair.expected == "Правильно? "
        }));
        assert!(collector.pairs.iter().any(|pair| {
            pair.train_role == "negative"
                && pair.original == "Праивльно? "
                && pair.expected == "Правило? "
        }));
        assert!(collector
            .pairs
            .iter()
            .all(|pair| pair.original != "ло? " && pair.expected != "льно? "));
    }

    #[test]
    fn matching_final_target_confirms_lay_output_instead_of_rejecting_it() {
        let text = r#"{"ts":3,"kind":"user-correction","from":"гналом ","to":"сигналом ","replace_words":1,"lay_kind":"typing-assist","lay_from":"сиганлом ","lay_to":"сигналом ","user_target":"сигналом "}"#;
        let mut collector = Collector::default();
        collect_corrections_text(&mut collector, text, 100);

        assert_eq!(collector.pairs.len(), 1);
        assert_eq!(collector.pairs[0].signal, "user_accepted_fix");
        assert_eq!(collector.pairs[0].train_role, "positive");
        assert_eq!(collector.by_signal.get("user_rejected_lay_output"), None);
    }

    #[test]
    fn collects_layout_replay_as_manual_transition() {
        let text =
            r#"{"ts":3,"kind":"layout-replay","from":"ghbdtn","to":"привет","replace_words":1}"#;
        let mut collector = Collector::default();
        collect_recent_text(&mut collector, text, 100);
        assert_eq!(collector.pairs.len(), 1);
        assert_eq!(collector.pairs[0].signal, "manual_layout_replay");
        assert_eq!(collector.pairs[0].train_role, "positive");
        assert_eq!(collector.pairs[0].operation, "replacement");
        assert_eq!(collector.pairs[0].source_id, "layout");
        assert_eq!(collector.pairs[0].action_operator, "flip_layout");
    }

    #[test]
    fn conflicting_manual_replay_is_quarantined_from_learning() {
        let mut collector = Collector::default();
        collect_corrections_text(
            &mut collector,
            r#"{"ts":3,"kind":"user-correction","from":"ljdtcnb ","to":"довести ","lay_from":"ljdtcnb ","lay_to":"довести "}"#,
            100,
        );
        collect_recent_text(
            &mut collector,
            r#"{"ts":4,"kind":"layout-replay","from":"ljdtcnb ","to":"довести ","replace_words":1}"#,
            100,
        );

        collector.reconcile_conflicting_evidence();

        assert!(collector
            .pairs
            .iter()
            .all(|pair| pair.train_role == "review"));
        assert!(collector
            .pairs
            .iter()
            .all(|pair| pair.quarantine_reason == "conflicting_transition_feedback"));
        assert_eq!(collector.by_train_role.get("negative"), None);
        assert_eq!(collector.by_train_role["review"], 3);
    }

    #[test]
    fn technical_manual_replay_still_resolves_matching_negative_evidence() {
        let mut collector = Collector::default();
        let negative = pair_from_correction_text(
            &serde_json::json!({"lay_kind": "typing-assist"}),
            "user_rejected_lay_output",
            "СЗГ ",
            "CPU ",
        )
        .expect("valid negative pair");
        assert_eq!(negative.train_role, "negative");
        collector.add(negative);
        collect_recent_text(
            &mut collector,
            r#"{"ts":4,"kind":"layout-replay","from":"СЗГ ","to":"CPU ","replace_words":1}"#,
            100,
        );

        collector.reconcile_conflicting_evidence();

        assert!(collector
            .pairs
            .iter()
            .all(|pair| pair.train_role == "review"));
        assert!(collector
            .pairs
            .iter()
            .all(|pair| { pair.quarantine_reason == "conflicting_transition_feedback" }));
    }

    #[test]
    fn correction_layout_evidence_binds_to_layout_operator() {
        let pair = pair_from_correction_text(
            &serde_json::json!({"lay_kind": "typing-assist"}),
            "user_rejected_lay_output",
            "yfc ",
            "нас ",
        )
        .expect("valid layout correction pair");

        assert_eq!(pair.operation, "layout");
        assert_eq!(pair.source_id, "layout");
        assert_eq!(pair.action_operator, "flip_layout");
        let events = rejected_usage_events(&pair);
        assert_eq!(events[0]["source"], "layout");
        assert_eq!(events[0]["operation"], "replacement");
        assert_eq!(
            events[0]["surface"],
            lay::nanda_wave::transition_surface_key("yfc ", "нас ", "layout", "replacement")
        );
    }

    #[test]
    fn corrections_log_layout_replay_reaches_the_same_transition_memory() {
        let text =
            r#"{"ts":3,"kind":"layout-replay","from":"ltkfq","to":"делай","replace_words":1}"#;
        let mut collector = Collector::default();
        collect_corrections_text(&mut collector, text, 100);

        assert_eq!(collector.pairs.len(), 1);
        assert_eq!(collector.pairs[0].signal, "manual_layout_replay");
        assert_eq!(collector.pairs[0].source_id, "layout");
        assert_eq!(collector.pairs[0].operation, "replacement");
    }

    #[test]
    fn quarantines_short_or_technical_manual_layout_pairs() {
        let text = r#"{"ts":3,"kind":"layout-replay","from":"Д2","to":"L2","replace_words":1}
{"ts":4,"kind":"layout-replay","from":"e","to":"у","replace_words":1}
"#;
        let mut collector = Collector::default();
        collect_recent_text(&mut collector, text, 100);
        assert_eq!(collector.pairs.len(), 2);
        assert!(collector
            .pairs
            .iter()
            .all(|pair| pair.train_role == "review"));
        assert!(collector
            .pairs
            .iter()
            .all(|pair| pair.quarantine_reason == "manual_layout_short_or_technical"));
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

    #[test]
    fn latest_state_replay_uses_newest_feedback_for_same_transition() {
        let older = serde_json::json!({
            "ts": 1,
            "lay_kind": "typing-assist",
            "lay_from": "xnj ",
            "lay_to": "что "
        });
        let newer = serde_json::json!({
            "ts": 2,
            "lay_kind": "typing-assist",
            "from": "xnj ",
            "to": "что "
        });
        let rejected =
            pair_from_correction(&older, "user_rejected_lay_output", "lay_from", "lay_to").unwrap();
        let accepted = pair_from_correction(&newer, "user_accepted_fix", "from", "to").unwrap();

        let latest = latest_transition_states(vec![rejected, accepted]);

        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].train_role, "positive");
        assert_eq!(latest[0].ts, 2);
    }

    #[test]
    fn rejected_usage_records_only_the_changed_candidate_word() {
        let value = serde_json::json!({
            "ts": 3,
            "lay_kind": "typing-assist",
            "lay_from": "как попусы ",
            "lay_to": "как опусы "
        });
        let pair = pair_from_correction(&value, "user_rejected_lay_output", "lay_from", "lay_to")
            .expect("negative pair");

        let events = rejected_usage_events(&pair);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["word"], "опусы");
        assert_eq!(events[0]["context"], serde_json::json!(["как"]));
        assert!(events[0]["surface"]
            .as_str()
            .is_some_and(|value| !value.is_empty()));
    }

    #[test]
    fn broad_user_rewrite_is_review_not_positive_training() {
        for (original, expected) in [("воздействиям ", "Все "), ("вот ", "что ")]
        {
            let (role, reason) = train_role_and_quarantine(
                "user_accepted_fix",
                "typing-assist",
                original,
                expected,
                "surface_typo",
            );

            assert_eq!(role, "review", "{original:?} -> {expected:?}");
            assert_eq!(
                reason, "nonlocal_user_edit_not_candidate_evidence",
                "{original:?} -> {expected:?}"
            );
        }
    }

    #[test]
    fn local_typo_and_layout_remain_positive_training() {
        for (original, expected) in [("звгрузи ", "загрузи "), ("ghbdtn ", "привет ")]
        {
            assert!(positive_pair_is_trainable(original, expected));
        }
    }

    #[test]
    fn replay_state_card_keeps_safety_and_coverage_denominators_visible() {
        let report = ReplayReport {
            positive: ReplayBucket {
                cases: 10,
                expected_present: 7,
                expected_missing: 3,
                no_l2_candidates: 1,
                expected_top1: 6,
                applied_expected: 6,
                present_not_applied: 1,
                ..ReplayBucket::default()
            },
            negative: ReplayBucket {
                cases: 8,
                applied_other: 2,
                ..ReplayBucket::default()
            },
            review: 4,
            ..ReplayReport::default()
        };

        let card = replay_state_card_json(&report);

        assert_eq!(card["candidate_birth"]["positive_coverage_percent"], 70.0);
        assert_eq!(card["safety"]["negative_false_apply"], 0);
        assert_eq!(card["safety"]["negative_other_apply"], 2);
        assert_eq!(card["safety"]["verdict"], "PASS");
        assert_eq!(card["evidence"]["quarantined_percent"], 18.18);
    }
}
