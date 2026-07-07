use serde_json::Value;
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

fn print_usage() {
    eprintln!("usage: lay-debug-actions --unsafe-edits | --stale-tail");
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
    if value.get("kind").and_then(Value::as_str) == Some("candidate_before_apply") {
        return value
            .get("boundary_changed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            || value
                .get("changes_non_last_word")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            || !value
                .get("safety_allow_apply")
                .and_then(Value::as_bool)
                .unwrap_or(true);
    }

    let from = value.get("from").and_then(Value::as_str).unwrap_or("");
    let to = value.get("to").and_then(Value::as_str).unwrap_or("");
    if from.is_empty() || to.is_empty() {
        return false;
    }
    let from_words = from.split_whitespace().collect::<Vec<_>>();
    let to_words = to.split_whitespace().collect::<Vec<_>>();
    let word_count_changed = from_words.len() != to_words.len();
    let replacement_shorter = to.chars().count() + 2 <= from.chars().count();
    let left_context_changed = from_words.len() >= 2
        && from_words.len() == to_words.len()
        && from_words[..from_words.len() - 1] != to_words[..to_words.len() - 1];
    let nanda_multiword = value
        .pointer("/input_gate/selected_source")
        .and_then(Value::as_str)
        == Some("nanda")
        && from_words.len() >= 2;
    let slow_output = value
        .get("output_ms")
        .and_then(Value::as_u64)
        .is_some_and(|ms| ms >= 250);

    word_count_changed
        || replacement_shorter
        || left_context_changed
        || nanda_multiword
        || slow_output
}
