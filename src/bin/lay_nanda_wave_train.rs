use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use lay::nanda_wave::packet::{write_learned_packet, LearnedPacketEntry};
use serde::Deserialize;

const DEFAULT_DATASET: &str = "data/nanda_training/generated_cases.tsv";
const RECENT_ACTIONS: &str = ".local/share/lay/recent_actions.jsonl";
const CORRECTIONS_LOG: &str = ".local/share/lay/corrections.jsonl";

#[derive(Debug, Clone)]
struct Learned {
    expected: String,
    operation: String,
    count: usize,
    conflicts: usize,
    live_count: usize,
}

fn main() -> io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    let dataset = arg_path(&args, "--dataset").unwrap_or_else(|| PathBuf::from(DEFAULT_DATASET));
    let out =
        arg_path(&args, "--out").unwrap_or_else(lay::nanda_wave::learned::default_memory_path);
    let include_live_actions = args.iter().any(|arg| arg == "--include-live-actions");
    let include_user_corrections = args.iter().any(|arg| arg == "--include-user-corrections");
    let mut learned = learn(&dataset)?;
    let live_report = if include_live_actions || include_user_corrections {
        add_live_learning(&mut learned, include_user_corrections)?
    } else {
        LiveLearningReport::default()
    };
    write_memory(&out, &learned)?;
    print_summary(&dataset, &out, &learned, &live_report);
    Ok(())
}

fn learn(path: &Path) -> io::Result<BTreeMap<String, Learned>> {
    let text = fs::read_to_string(path)?;
    let mut map = BTreeMap::<String, Learned>::new();
    for (idx, line) in text.lines().enumerate() {
        if idx == 0 || line.trim().is_empty() {
            continue;
        }
        let cols = line.split('\t').collect::<Vec<_>>();
        if cols.len() < 8 || cols[5] != "1" || cols[2] == cols[3] {
            continue;
        }
        let original = cols[2].trim_end();
        let expected = cols[3].trim_end();
        if original == expected {
            continue;
        }
        let entry = map.entry(original.to_string()).or_insert_with(|| Learned {
            expected: expected.to_string(),
            operation: cols[4].to_string(),
            count: 0,
            conflicts: 0,
            live_count: 0,
        });
        if entry.expected == expected {
            entry.count += 1;
        } else {
            entry.conflicts += 1;
        }
    }
    map.retain(|_, item| item.count > 0 && item.conflicts == 0);
    Ok(map)
}

#[derive(Debug, Default)]
struct LiveLearningReport {
    read: usize,
    accepted: usize,
    skipped: usize,
    user_skipped: usize,
}

#[derive(Debug, Deserialize)]
struct LiveAction {
    #[serde(default)]
    kind: String,
    #[serde(default, rename = "from")]
    from_text: String,
    #[serde(default, rename = "to")]
    to_text: String,
}

fn add_live_learning(
    learned: &mut BTreeMap<String, Learned>,
    include_user_corrections: bool,
) -> io::Result<LiveLearningReport> {
    let mut report = LiveLearningReport::default();
    for path in live_paths() {
        add_live_file(learned, &path, include_user_corrections, &mut report)?;
    }
    learned.retain(|_, item| item.count > 0 && item.conflicts == 0);
    Ok(report)
}

fn add_live_file(
    learned: &mut BTreeMap<String, Learned>,
    path: &Path,
    include_user_corrections: bool,
    report: &mut LiveLearningReport,
) -> io::Result<()> {
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(());
    };
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        report.read += 1;
        let Ok(action) = serde_json::from_str::<LiveAction>(line) else {
            report.skipped += 1;
            continue;
        };
        if !is_learnable_live_kind(&action.kind, include_user_corrections) {
            if action.kind == "user-correction" {
                report.user_skipped += 1;
            } else {
                report.skipped += 1;
            }
            continue;
        }
        let Some((from, to)) = normalized_live_pair(&action.from_text, &action.to_text) else {
            report.skipped += 1;
            continue;
        };
        let operation = operation_from_live_kind(&action.kind, &from, &to).to_string();
        let entry = learned.entry(from).or_insert_with(|| Learned {
            expected: to.clone(),
            operation,
            count: 0,
            conflicts: 0,
            live_count: 0,
        });
        if entry.expected == to {
            entry.count += 1;
            entry.live_count += 1;
            report.accepted += 1;
        } else {
            entry.conflicts += 1;
            report.skipped += 1;
        }
    }
    Ok(())
}

fn is_learnable_live_kind(kind: &str, include_user_corrections: bool) -> bool {
    matches!(
        kind,
        "typing-assist" | "ime-typing-assist" | "layout-replay" | "smart-text"
    ) || (include_user_corrections && kind == "user-correction")
}

fn normalized_live_pair(from: &str, to: &str) -> Option<(String, String)> {
    let from = from.trim_end();
    let to = to.trim_end();
    if from.is_empty()
        || to.is_empty()
        || from == to
        || from.chars().count() > 96
        || to.chars().count() > 96
        || from.chars().any(char::is_control)
        || to.chars().any(char::is_control)
        || from.split_whitespace().count().max(1) > 6
        || to.split_whitespace().count().max(1) > 6
    {
        return None;
    }
    Some((from.to_string(), to.to_string()))
}

fn operation_from_live_kind(kind: &str, from: &str, to: &str) -> &'static str {
    if kind == "layout-replay" || scripts_look_layout_like(from, to) {
        "layout"
    } else if from.split_whitespace().count() != to.split_whitespace().count() {
        "split"
    } else {
        "typo"
    }
}

fn scripts_look_layout_like(from: &str, to: &str) -> bool {
    let from_ascii = from.chars().any(|ch| ch.is_ascii_alphabetic());
    let from_cyr = from
        .chars()
        .any(|ch| ('а'..='я').contains(&ch) || ('А'..='Я').contains(&ch));
    let to_ascii = to.chars().any(|ch| ch.is_ascii_alphabetic());
    let to_cyr = to
        .chars()
        .any(|ch| ('а'..='я').contains(&ch) || ('А'..='Я').contains(&ch));
    (from_ascii && to_cyr) || (from_cyr && to_ascii)
}

fn live_paths() -> Vec<PathBuf> {
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };
    vec![home.join(RECENT_ACTIONS), home.join(CORRECTIONS_LOG)]
}

fn write_memory(path: &Path, learned: &BTreeMap<String, Learned>) -> io::Result<()> {
    let entries = learned
        .iter()
        .map(|(original, item)| LearnedPacketEntry {
            original: original.clone(),
            expected: item.expected.clone(),
            operation: item.operation.clone(),
            count: item.count,
        })
        .collect::<Vec<_>>();
    let report = write_learned_packet(path, &entries)?;
    println!(
        "cell32_packet: bytes={} encoded={} skipped={}",
        lay::nanda_wave::CELL32_BYTES,
        report.encoded,
        report.skipped
    );
    Ok(())
}

fn print_summary(
    dataset: &Path,
    out: &Path,
    learned: &BTreeMap<String, Learned>,
    live_report: &LiveLearningReport,
) {
    let mut by_operation = BTreeMap::<&str, usize>::new();
    let mut live_entries = 0usize;
    for item in learned.values() {
        *by_operation.entry(&item.operation).or_default() += 1;
        if item.live_count > 0 {
            live_entries += 1;
        }
    }
    println!("dataset: {}", dataset.display());
    println!("out: {}", out.display());
    println!("learned_corrections: {}", learned.len());
    if live_report.read > 0 {
        println!(
            "live_actions: read={} accepted={} skipped={} user_skipped={} live_entries={}",
            live_report.read,
            live_report.accepted,
            live_report.skipped,
            live_report.user_skipped,
            live_entries
        );
    }
    for (operation, count) in by_operation {
        println!("  {operation}: {count}");
    }
}

fn arg_path(args: &[String], name: &str) -> Option<PathBuf> {
    args.windows(2)
        .find_map(|pair| (pair[0] == name).then(|| PathBuf::from(&pair[1])))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_typing_action_is_learnable() {
        let pair = normalized_live_pair("fавтозамена ", "автозамена ").unwrap();
        assert_eq!(pair, ("fавтозамена".to_string(), "автозамена".to_string()));
        assert!(is_learnable_live_kind("ime-typing-assist", false));
        assert_eq!(
            operation_from_live_kind("ime-typing-assist", &pair.0, &pair.1),
            "layout"
        );
    }

    #[test]
    fn user_corrections_are_opt_in_for_training() {
        assert!(!is_learnable_live_kind("user-correction", false));
        assert!(is_learnable_live_kind("user-correction", true));
    }
}
