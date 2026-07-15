use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

const RECENT_ACTIONS: &str = ".local/share/lay/recent_actions.jsonl";
const IBUS_TRACE: &str = ".local/share/lay/ibus_engine_debug.jsonl";

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_usage();
        return;
    }
    if args.iter().any(|arg| arg == "--unsafe-edits") {
        print_matching_jsonl(recent_actions_path(), unsafe_edit);
        return;
    }
    if args.iter().any(|arg| arg == "--unsafe-scoreboard") {
        print_unsafe_scoreboard(recent_actions_path(), false);
        return;
    }
    if args.iter().any(|arg| arg == "--unsafe-gate") {
        print_unsafe_scoreboard(recent_actions_path(), true);
        return;
    }
    if args.iter().any(|arg| arg == "--candidate-report") {
        print_candidate_report(recent_actions_path());
        return;
    }
    if args.iter().any(|arg| arg == "--transition-replay") {
        print_transition_replay(recent_actions_path());
        return;
    }
    if args.iter().any(|arg| arg == "--stale-tail") {
        print_matching_jsonl(ibus_trace_path(), stale_tail_guard);
        return;
    }
    print_usage();
    std::process::exit(2);
}

fn print_matching_jsonl(path: PathBuf, predicate: fn(&Value) -> bool) {
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!("cannot read {}", path.display());
        std::process::exit(1);
    };

    for line in text.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if !predicate(&value) {
            continue;
        }
        println!(
            "{}",
            serde_json::to_string(&value).unwrap_or_else(|_| line.to_string())
        );
    }
}

fn print_unsafe_scoreboard(path: PathBuf, fail_on_unsafe: bool) {
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!("cannot read {}", path.display());
        std::process::exit(1);
    };
    let mut scoreboard = UnsafeScoreboard::default();
    for line in text.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        scoreboard.inspect(&value);
    }
    println!("{}", scoreboard.to_json(&path));
    if fail_on_unsafe && !scoreboard.passes_gate() {
        std::process::exit(1);
    }
}

fn print_candidate_report(path: PathBuf) {
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!("cannot read {}", path.display());
        std::process::exit(1);
    };
    let mut report = CandidateReport::default();
    for line in text.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        report.inspect(&value);
    }
    println!("{}", report.to_json(&path));
}

fn print_transition_replay(path: PathBuf) {
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!("cannot read {}", path.display());
        std::process::exit(1);
    };
    let mut replay = TransitionReplay::default();
    for line in text.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        replay.inspect(&value);
    }
    println!("{}", replay.to_json(&path));
    if !replay.passes_gate() {
        std::process::exit(1);
    }
}

fn print_usage() {
    eprintln!(
        "usage: lay-debug-actions --unsafe-edits | --unsafe-scoreboard | --unsafe-gate | --candidate-report | --transition-replay | --stale-tail"
    );
}

fn recent_actions_path() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(RECENT_ACTIONS)
}

fn ibus_trace_path() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(IBUS_TRACE)
}

fn stale_tail_guard(value: &Value) -> bool {
    value.get("kind").and_then(Value::as_str) == Some("ibus_committed_tail_replace_guard")
        && value
            .get("reason")
            .and_then(Value::as_str)
            .is_some_and(|reason| reason.contains("stale"))
}

fn unsafe_edit(value: &Value) -> bool {
    !unsafe_reasons(value).is_empty()
}

fn blocked_unsafe_candidate(value: &Value) -> bool {
    value.get("kind").and_then(Value::as_str) == Some("candidate_before_apply")
        && value.get("action_kind").and_then(Value::as_str) == Some("block_unsafe")
        && !value
            .get("safety_allow_apply")
            .and_then(Value::as_bool)
            .unwrap_or(true)
}

fn unsafe_reasons(value: &Value) -> Vec<&'static str> {
    let mut reasons = Vec::new();
    if value.get("kind").and_then(Value::as_str) == Some("candidate_before_apply") {
        if blocked_unsafe_candidate(value) {
            return reasons;
        }
        if value
            .get("boundary_changed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            || value
                .get("word_count_changed")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        {
            reasons.push("boundary_changed");
        }
        if value
            .get("changes_non_last_word")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            || value
                .get("would_touch_words")
                .and_then(Value::as_u64)
                .is_some_and(|words| words > 1)
        {
            reasons.push("multiword_touch");
        }
        if !value
            .get("safety_allow_apply")
            .and_then(Value::as_bool)
            .unwrap_or(true)
        {
            reasons.push("safety_block");
        }
        if value
            .get("transition_left_context_changed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            reasons.push("transition_left_context_changed");
        }
        if value
            .get("transition_left_context_changed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            && !value
                .get("transition_verified")
                .and_then(Value::as_bool)
                .unwrap_or(true)
        {
            reasons.push("unverified_transition");
        }
        append_selected_candidate_reasons(value.get("input_gate"), &mut reasons);
        return reasons;
    }

    let from = value.get("from").and_then(Value::as_str).unwrap_or("");
    let to = value.get("to").and_then(Value::as_str).unwrap_or("");
    if from.is_empty() || to.is_empty() {
        append_selected_candidate_reasons(value.get("input_gate"), &mut reasons);
        return reasons;
    }
    let from_words = from.split_whitespace().collect::<Vec<_>>();
    let to_words = to.split_whitespace().collect::<Vec<_>>();
    let word_count_changed = from_words.len() != to_words.len();
    let left_context_changed = from_words.len() >= 2
        && from_words.len() == to_words.len()
        && from_words[..from_words.len() - 1] != to_words[..to_words.len() - 1];
    let nanda_multiword = value
        .pointer("/input_gate/selected_source")
        .and_then(Value::as_str)
        == Some("nanda")
        && from_words.len() >= 2;
    if word_count_changed {
        reasons.push("word_count_changed");
    }
    if left_context_changed {
        reasons.push("left_context_changed");
    }
    if nanda_multiword {
        reasons.push("nanda_multiword");
    }
    append_selected_candidate_reasons(value.get("input_gate"), &mut reasons);
    reasons
}

fn is_slow_output(value: &Value) -> bool {
    value
        .get("output_ms")
        .and_then(Value::as_u64)
        .is_some_and(|ms| ms >= 250)
}

fn append_selected_candidate_reasons(gate: Option<&Value>, reasons: &mut Vec<&'static str>) {
    let Some(gate) = gate else {
        return;
    };
    let Some(candidates) = gate.get("candidate_scores").and_then(Value::as_array) else {
        return;
    };
    let Some(selected) = candidates.iter().find(|candidate| {
        candidate
            .get("selected")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }) else {
        return;
    };
    if selected
        .get("edit_transition_left_context_changed")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        reasons.push("selected_left_context_changed");
    }
    if selected
        .get("edit_transition_left_context_changed")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        && !selected
            .get("edit_transition_verified")
            .and_then(Value::as_bool)
            .unwrap_or(true)
    {
        reasons.push("selected_unverified_transition");
    }
}

#[derive(Default)]
struct UnsafeScoreboard {
    records: usize,
    unsafe_records: usize,
    gate_failures: usize,
    blocked_unsafe: usize,
    candidate_before_apply: usize,
    action_records: usize,
    boundary_changed: usize,
    multiword_touch: usize,
    transition_left_context_changed: usize,
    unverified_transition: usize,
    selected_left_context_changed: usize,
    selected_unverified_transition: usize,
    nanda_multiword: usize,
    slow_output: usize,
    mutation_routes: BTreeMap<String, usize>,
}

#[derive(Default)]
struct CandidateReport {
    records: usize,
    gate_records: usize,
    total_candidates: u64,
    selected_sources: BTreeMap<String, usize>,
    selected_error_classes: BTreeMap<String, usize>,
    selected_gate_actions: BTreeMap<String, usize>,
    bayes_selected: usize,
    deterministic_candidates: u64,
    nanda_candidates: u64,
    apply_candidates: u64,
    suggest_only_candidates: u64,
    veto_candidates: u64,
    max_output_ms: u64,
}

impl CandidateReport {
    fn inspect(&mut self, value: &Value) {
        self.records += 1;
        if let Some(ms) = value.get("output_ms").and_then(Value::as_u64) {
            self.max_output_ms = self.max_output_ms.max(ms);
        }
        let Some(gate) = value.get("input_gate") else {
            return;
        };
        self.gate_records += 1;
        let scoreboard = gate.get("scoreboard");
        self.total_candidates = self.total_candidates.saturating_add(
            scoreboard
                .and_then(|scoreboard| scoreboard.get("total_candidates"))
                .and_then(Value::as_u64)
                .unwrap_or_else(|| {
                    gate.get("candidate_count")
                        .and_then(Value::as_u64)
                        .unwrap_or_default()
                }),
        );
        self.deterministic_candidates = self.deterministic_candidates.saturating_add(
            scoreboard
                .and_then(|scoreboard| scoreboard.get("deterministic_candidates"))
                .and_then(Value::as_u64)
                .unwrap_or_default(),
        );
        self.nanda_candidates = self.nanda_candidates.saturating_add(
            scoreboard
                .and_then(|scoreboard| scoreboard.get("nanda_candidates"))
                .and_then(Value::as_u64)
                .unwrap_or_default(),
        );
        self.apply_candidates = self.apply_candidates.saturating_add(
            scoreboard
                .and_then(|scoreboard| scoreboard.get("apply_candidates"))
                .and_then(Value::as_u64)
                .unwrap_or_default(),
        );
        self.suggest_only_candidates = self.suggest_only_candidates.saturating_add(
            scoreboard
                .and_then(|scoreboard| scoreboard.get("suggest_only_candidates"))
                .and_then(Value::as_u64)
                .unwrap_or_default(),
        );
        self.veto_candidates = self.veto_candidates.saturating_add(
            scoreboard
                .and_then(|scoreboard| scoreboard.get("veto_candidates"))
                .and_then(Value::as_u64)
                .unwrap_or_default(),
        );
        if scoreboard
            .and_then(|scoreboard| scoreboard.get("selected_bayes_posterior_milli"))
            .and_then(Value::as_i64)
            .is_some()
        {
            self.bayes_selected += 1;
        }
        if let Some(source) = gate.get("selected_source").and_then(Value::as_str) {
            *self.selected_sources.entry(source.to_string()).or_insert(0) += 1;
        }
        if let Some(error_class) = gate.get("selected_error_class").and_then(Value::as_str) {
            *self
                .selected_error_classes
                .entry(error_class.to_string())
                .or_insert(0) += 1;
        }
        if let Some(action) = gate.get("selected_gate_action").and_then(Value::as_str) {
            *self
                .selected_gate_actions
                .entry(action.to_string())
                .or_insert(0) += 1;
        }
    }

    fn to_json(&self, path: &std::path::Path) -> String {
        let avg_candidates = self
            .total_candidates
            .checked_div(self.gate_records as u64)
            .unwrap_or_default();
        serde_json::json!({
            "kind": "candidate_quality_report",
            "source": path.display().to_string(),
            "records": {
                "total": self.records,
                "with_input_gate": self.gate_records
            },
            "candidate_count": {
                "total": self.total_candidates,
                "avg_per_gate_record": avg_candidates
            },
            "candidate_sources": {
                "deterministic": self.deterministic_candidates,
                "nanda": self.nanda_candidates,
            },
            "gate_actions": {
                "apply": self.apply_candidates,
                "suggest_only": self.suggest_only_candidates,
                "veto": self.veto_candidates,
            },
            "selected": {
                "sources": self.selected_sources,
                "error_classes": self.selected_error_classes,
                "gate_actions": self.selected_gate_actions,
                "bayes_posterior_records": self.bayes_selected
            },
            "latency": {
                "max_output_ms": self.max_output_ms
            },
            "read_as": "recent_actions candidate scoreboard; use with --unsafe-gate for edit safety"
        })
        .to_string()
    }
}

#[derive(Default)]
struct TransitionReplay {
    records: usize,
    candidate_before_apply: usize,
    applied_actions: usize,
    input_gate_records: usize,
    selected_apply: usize,
    false_apply_candidates: usize,
    missed_good_candidates: usize,
    left_context_mutations: usize,
    unverified_left_context_mutations: usize,
    unverified_transitions: usize,
    unsafe_multiword: usize,
}

impl TransitionReplay {
    fn passes_gate(&self) -> bool {
        self.false_apply_candidates == 0
            && self.unverified_left_context_mutations == 0
            && self.unverified_transitions == 0
    }

    fn inspect(&mut self, value: &Value) {
        self.records += 1;
        match value.get("kind").and_then(Value::as_str) {
            Some("candidate_before_apply") => self.candidate_before_apply += 1,
            Some("typing-assist") => self.applied_actions += 1,
            _ => {}
        }
        if value.get("input_gate").is_some() {
            self.input_gate_records += 1;
        }
        let left_context_changed = value
            .get("transition_left_context_changed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            || value
                .get("changes_non_last_word")
                .and_then(Value::as_bool)
                .unwrap_or(false);
        let transition_verified = value
            .get("transition_verified")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        if left_context_changed {
            self.left_context_mutations += 1;
        }
        if left_context_changed && !transition_verified {
            self.unverified_left_context_mutations += 1;
        }
        if !transition_verified {
            self.unverified_transitions += 1;
        }
        if value
            .get("would_touch_words")
            .and_then(Value::as_u64)
            .is_some_and(|words| words > 1)
        {
            self.unsafe_multiword += 1;
        }

        let Some(gate) = value.get("input_gate") else {
            return;
        };
        let Some(candidates) = gate.get("candidate_scores").and_then(Value::as_array) else {
            return;
        };
        for candidate in candidates {
            self.inspect_candidate(candidate);
        }
    }

    fn inspect_candidate(&mut self, candidate: &Value) {
        let selected = candidate
            .get("selected")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !selected {
            return;
        }
        let apply = candidate.get("gate_action").and_then(Value::as_str) == Some("apply")
            || candidate
                .get("selected_gate_action")
                .and_then(Value::as_str)
                == Some("apply");
        let left_context = candidate
            .get("edit_transition_left_context_changed")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let verified = candidate
            .get("edit_transition_verified")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        if apply {
            self.selected_apply += 1;
        }
        if apply && left_context && !verified {
            self.false_apply_candidates += 1;
        }
        if !apply && verified && !left_context {
            self.missed_good_candidates += 1;
        }
    }

    fn to_json(&self, path: &std::path::Path) -> String {
        serde_json::json!({
            "kind": "typing_transition_shadow_replay",
            "source": path.display().to_string(),
            "records": {
                "total": self.records,
                "candidate_before_apply": self.candidate_before_apply,
                "applied_actions": self.applied_actions,
                "with_input_gate": self.input_gate_records
            },
            "transition": {
                "selected_apply": self.selected_apply,
                "false_apply_candidates": self.false_apply_candidates,
                "missed_good_candidates": self.missed_good_candidates,
                "left_context_mutations_observed": self.left_context_mutations,
                "unverified_left_context_mutations": self.unverified_left_context_mutations,
                "unverified_transitions": self.unverified_transitions,
                "unsafe_multiword": self.unsafe_multiword
            },
            "verdict": if self.passes_gate() {
                "PASS-shadow"
            } else {
                "WATCH-shadow"
            },
            "read_as": "shadow replay over recent_actions; runtime authority is TransitionDecisionCore"
        })
        .to_string()
    }
}

impl UnsafeScoreboard {
    fn passes_gate(&self) -> bool {
        self.gate_failures == 0
    }

    fn inspect(&mut self, value: &Value) {
        self.records += 1;
        if value.get("kind").and_then(Value::as_str) == Some("candidate_before_apply") {
            self.candidate_before_apply += 1;
            let route = value
                .get("mutation_route")
                .and_then(Value::as_str)
                .unwrap_or("legacy_missing_route");
            *self.mutation_routes.entry(route.to_string()).or_insert(0) += 1;
        } else {
            self.action_records += 1;
        }
        if blocked_unsafe_candidate(value) {
            self.blocked_unsafe += 1;
        }
        let reasons = unsafe_reasons(value);
        if !reasons.is_empty() {
            self.unsafe_records += 1;
        }
        if reasons.iter().any(|reason| {
            matches!(
                *reason,
                "safety_block" | "unverified_transition" | "selected_unverified_transition"
            )
        }) {
            self.gate_failures += 1;
        }
        if is_slow_output(value) {
            self.slow_output += 1;
        }
        for reason in reasons {
            match reason {
                "boundary_changed" | "word_count_changed" => self.boundary_changed += 1,
                "multiword_touch" => self.multiword_touch += 1,
                "transition_left_context_changed" | "left_context_changed" => {
                    self.transition_left_context_changed += 1;
                }
                "unverified_transition" => self.unverified_transition += 1,
                "selected_left_context_changed" => self.selected_left_context_changed += 1,
                "selected_unverified_transition" => self.selected_unverified_transition += 1,
                "nanda_multiword" => self.nanda_multiword += 1,
                _ => {}
            }
        }
    }

    fn to_json(&self, path: &std::path::Path) -> String {
        serde_json::json!({
            "kind": "unsafe_edit_scoreboard",
            "source": path.display().to_string(),
            "records": {
                "total": self.records,
                "observed_risk": self.unsafe_records,
                "gate_failures": self.gate_failures,
                "blocked_unsafe": self.blocked_unsafe,
                "candidate_before_apply": self.candidate_before_apply,
                "actions": self.action_records
            },
            "classes": {
                "boundary_changed": self.boundary_changed,
                "multiword_touch": self.multiword_touch,
                "transition_left_context_changed": self.transition_left_context_changed,
                "unverified_transition": self.unverified_transition,
                "selected_left_context_changed": self.selected_left_context_changed,
                "selected_unverified_transition": self.selected_unverified_transition,
                "nanda_multiword": self.nanda_multiword,
                "slow_output": self.slow_output
            },
            "performance": {
                "slow_output": self.slow_output,
                "slow_output_is_safety_failure": false
            },
            "mutation_routes": self.mutation_routes,
            "verdict": if self.passes_gate() { "PASS" } else { "VETO" },
            "read_as": "observed_risk includes verified boundary edits; gate_failures counts only unsafe execution that escaped verification"
        })
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{unsafe_edit, CandidateReport, TransitionReplay, UnsafeScoreboard};
    use serde_json::json;

    #[test]
    fn unsafe_edit_flags_multiword_touch_even_without_boundary_change() {
        let value = json!({
            "kind": "candidate_before_apply",
            "from": "одно два ",
            "to": "одно два ",
            "boundary_changed": false,
            "changes_non_last_word": false,
            "word_count_changed": false,
            "would_touch_words": 2,
            "safety_allow_apply": true
        });
        assert!(unsafe_edit(&value));
    }

    #[test]
    fn unsafe_edit_flags_unverified_left_context_transition() {
        let value = json!({
            "kind": "candidate_before_apply",
            "from": "одно два ",
            "to": "однотри ",
            "boundary_changed": false,
            "changes_non_last_word": false,
            "word_count_changed": false,
            "would_touch_words": 1,
            "safety_allow_apply": true,
            "transition_left_context_changed": true,
            "transition_verified": false
        });
        assert!(unsafe_edit(&value));
    }

    #[test]
    fn unsafe_scoreboard_counts_selected_transition_risk() {
        let value = json!({
            "kind": "typing-assist",
            "from": "одно два",
            "to": "одно два",
            "replace_words": 1,
            "words": 2,
            "input_gate": {
                "candidate_scores": [{
                    "selected": true,
                    "edit_transition_left_context_changed": true,
                    "edit_transition_verified": false
                }]
            }
        });
        let mut scoreboard = UnsafeScoreboard::default();
        scoreboard.inspect(&value);
        assert_eq!(scoreboard.records, 1);
        assert_eq!(scoreboard.unsafe_records, 1);
        assert_eq!(scoreboard.gate_failures, 1);
        assert!(!scoreboard.passes_gate());
        assert_eq!(scoreboard.selected_left_context_changed, 1);
        assert_eq!(scoreboard.selected_unverified_transition, 1);
    }

    #[test]
    fn unsafe_scoreboard_reports_slow_output_without_safety_failure() {
        let value = json!({
            "kind": "typing-assist",
            "from": "провека",
            "to": "проверка",
            "output_ms": 291,
            "replace_words": 1,
            "words": 1
        });
        let mut scoreboard = UnsafeScoreboard::default();
        scoreboard.inspect(&value);
        assert_eq!(scoreboard.records, 1);
        assert_eq!(scoreboard.unsafe_records, 0);
        assert_eq!(scoreboard.slow_output, 1);
    }

    #[test]
    fn unsafe_scoreboard_counts_mutation_routes() {
        let value = json!({
            "kind": "candidate_before_apply",
            "mutation_route": "ime_committed_tail",
            "from": "провека ",
            "to": "проверка ",
            "boundary_changed": false,
            "changes_non_last_word": false,
            "word_count_changed": false,
            "would_touch_words": 1,
            "safety_allow_apply": true
        });
        let mut scoreboard = UnsafeScoreboard::default();
        scoreboard.inspect(&value);
        assert_eq!(scoreboard.candidate_before_apply, 1);
        assert_eq!(
            scoreboard
                .mutation_routes
                .get("ime_committed_tail")
                .copied(),
            Some(1)
        );
    }

    #[test]
    fn unsafe_scoreboard_counts_blocked_unsafe_without_failure() {
        let value = json!({
            "kind": "candidate_before_apply",
            "action_kind": "block_unsafe",
            "mutation_route": "ime_committed_tail",
            "from": "а ",
            "to": " ",
            "boundary_changed": true,
            "changes_non_last_word": false,
            "word_count_changed": true,
            "would_touch_words": 1,
            "safety_allow_apply": false,
            "safety_reason": "unsafe_boundary_edit_without_proof"
        });
        let mut scoreboard = UnsafeScoreboard::default();
        scoreboard.inspect(&value);
        assert!(!unsafe_edit(&value));
        assert_eq!(scoreboard.candidate_before_apply, 1);
        assert_eq!(scoreboard.blocked_unsafe, 1);
        assert_eq!(scoreboard.unsafe_records, 0);
        assert_eq!(scoreboard.gate_failures, 0);
        assert!(scoreboard.passes_gate());
        assert_eq!(scoreboard.boundary_changed, 0);
    }

    #[test]
    fn unsafe_scoreboard_observes_verified_boundary_without_failing_gate() {
        let value = json!({
            "kind": "candidate_before_apply",
            "mutation_route": "typing_assist_ime",
            "boundary_changed": true,
            "changes_non_last_word": true,
            "would_touch_words": 2,
            "safety_allow_apply": true,
            "transition_left_context_changed": true,
            "transition_verified": true,
            "transition_proof": "boundary"
        });
        let mut scoreboard = UnsafeScoreboard::default();
        scoreboard.inspect(&value);

        assert_eq!(scoreboard.unsafe_records, 1);
        assert_eq!(scoreboard.gate_failures, 0);
        assert!(scoreboard.passes_gate());
    }

    #[test]
    fn candidate_report_counts_sources_and_bayes_records() {
        let value = json!({
            "kind": "typing-assist",
            "output_ms": 12,
            "input_gate": {
                "candidate_count": 4,
                "selected_source": "nanda",
                "selected_error_class": "typo",
                "selected_gate_action": "apply",
                "scoreboard": {
                    "total_candidates": 4,
                    "apply_candidates": 1,
                    "suggest_only_candidates": 2,
                    "veto_candidates": 1,
                    "deterministic_candidates": 1,
                    "nanda_candidates": 3,
                    "selected_bayes_posterior_milli": 820
                }
            }
        });
        let mut report = CandidateReport::default();
        report.inspect(&value);

        assert_eq!(report.gate_records, 1);
        assert_eq!(report.total_candidates, 4);
        assert_eq!(report.nanda_candidates, 3);
        assert_eq!(report.bayes_selected, 1);
        assert_eq!(report.selected_sources.get("nanda").copied(), Some(1));
    }

    #[test]
    fn transition_replay_flags_selected_unverified_apply() {
        let value = json!({
            "kind": "typing-assist",
            "input_gate": {
                "candidate_scores": [{
                    "selected": true,
                    "gate_action": "apply",
                    "edit_transition_left_context_changed": true,
                    "edit_transition_verified": false
                }]
            }
        });
        let mut replay = TransitionReplay::default();
        replay.inspect(&value);
        assert_eq!(replay.selected_apply, 1);
        assert_eq!(replay.false_apply_candidates, 1);
    }

    #[test]
    fn transition_replay_keeps_verified_boundary_transition_out_of_unsafe_count() {
        let value = json!({
            "kind": "candidate_before_apply",
            "transition_left_context_changed": true,
            "transition_verified": true
        });
        let mut replay = TransitionReplay::default();
        replay.inspect(&value);

        assert_eq!(replay.left_context_mutations, 1);
        assert_eq!(replay.unverified_left_context_mutations, 0);
        assert!(replay.passes_gate());
    }

    #[test]
    fn transition_replay_rejects_any_unverified_transition() {
        let value = json!({
            "kind": "candidate_before_apply",
            "transition_left_context_changed": false,
            "transition_verified": false
        });
        let mut replay = TransitionReplay::default();
        replay.inspect(&value);

        assert_eq!(replay.unverified_transitions, 1);
        assert!(!replay.passes_gate());
    }
}
