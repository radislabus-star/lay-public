use lay::config::CorrectionSafety;
use lay::microbrain::{
    default_trained_layout_signal_path, evaluate_expert64_layout_signal,
    train_expert64_layout_signal, Expert64Cell, Expert64TrainingRow,
};
use lay::nanda_eval::{
    evaluate, evaluate_with_packet_and_baseline, read_cases, render_eval_report,
};
use lay::nanda_training_data::{add_training_group, read_training_rows};
use lay::private_file;
use serde::Deserialize;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_DATASET: &str = "data/neural_arbiter/dataset.tsv";
const DEFAULT_HOLDOUT: &str = "data/neural_arbiter/holdout.tsv";
const DEFAULT_REPORT_DIR: &str = "target/nanda/reports";
const LEARNING_LOG: &str = ".local/share/lay/corrections.jsonl";

#[derive(Debug, Deserialize)]
struct LearningEntry {
    kind: Option<String>,
    from: String,
    to: String,
    #[serde(default)]
    lay_kind: Option<String>,
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let dataset = arg_path(&args, "--dataset").unwrap_or_else(|| PathBuf::from(DEFAULT_DATASET));
    let holdout = arg_path(&args, "--holdout").unwrap_or_else(|| PathBuf::from(DEFAULT_HOLDOUT));
    let report_dir =
        arg_path(&args, "--report-dir").unwrap_or_else(|| PathBuf::from(DEFAULT_REPORT_DIR));
    let epochs = arg_value(&args, "--epochs")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(32);
    let promote = args.iter().any(|arg| arg == "--promote");
    let dry_run = !promote;

    fs::create_dir_all(&report_dir)?;
    let stamp = now_stamp();
    let candidate_packet = report_dir.join(format!("layout_signal_64k_candidate_{stamp}.ex64"));
    let report_path = report_dir.join(format!("layout_signal_64k_candidate_{stamp}.md"));

    let mut rows = read_training_rows(&dataset)?;
    let dataset_rows = rows.len();
    let learning_rows = append_learning_rows(&mut rows);
    rows.sort_by(|left, right| left.group_id.cmp(&right.group_id));
    let holdout_rows = read_training_rows(&holdout)?;

    let (cell, train_report) = train_expert64_layout_signal(&rows, epochs);
    let holdout_report = evaluate_expert64_layout_signal(&cell, &holdout_rows, epochs);
    cell.write(&candidate_packet)?;
    let loaded = Expert64Cell::read(&candidate_packet)?;
    let candidate_bytes = fs::metadata(&candidate_packet)?.len();

    let eval_cases = read_cases(&dataset)?;
    let safety = CorrectionSafety::Experimental;
    let baseline = evaluate(&eval_cases, safety, None);
    let current_report = evaluate_with_packet_and_baseline(
        &eval_cases,
        safety,
        &baseline,
        &[],
        read_runtime_cell(default_trained_layout_signal_path().as_path()),
    );
    let candidate_report = evaluate_with_packet_and_baseline(
        &eval_cases,
        safety,
        &baseline,
        &[],
        Some(loaded.clone()),
    );
    let eval_current = render_eval_report(&current_report, true);
    let eval_candidate = render_eval_report(&candidate_report, true);
    let candidate_ok = candidate_report.nanda_stats.ok;
    let current_ok = current_report.nanda_stats.ok;
    let worsened = candidate_report.compare.worsened;
    let should_promote = promote && candidate_ok >= current_ok && worsened == 0;

    if should_promote {
        let runtime_path = default_trained_layout_signal_path();
        private_file::write_private_bytes(&runtime_path, &loaded.to_bytes())?;
    }

    let report = render_report(LoopReport {
        dataset: &dataset,
        holdout: &holdout,
        candidate_packet: &candidate_packet,
        runtime_packet: &default_trained_layout_signal_path(),
        epochs,
        dataset_rows,
        learning_rows,
        candidate_bytes,
        train_rows: train_report.rows,
        train_groups: train_report.groups,
        train_acc: train_report.accuracy,
        train_group_acc: train_report.group_accuracy,
        holdout_rows: holdout_report.rows,
        holdout_groups: holdout_report.groups,
        holdout_acc: holdout_report.accuracy,
        holdout_group_acc: holdout_report.group_accuracy,
        current_eval: &eval_current,
        candidate_eval: &eval_candidate,
        dry_run,
        promoted: should_promote,
    });
    fs::write(&report_path, report)?;
    println!("report: {}", report_path.display());
    println!("candidate: {}", candidate_packet.display());
    println!("dataset_rows: {dataset_rows}");
    println!("learning_rows: {learning_rows}");
    println!("candidate_ok: {candidate_ok}");
    println!("current_ok: {current_ok}");
    println!("promoted: {should_promote}");
    if dry_run {
        println!("mode: dry-run; runtime packet was not changed");
    }
    Ok(())
}

struct LoopReport<'a> {
    dataset: &'a Path,
    holdout: &'a Path,
    candidate_packet: &'a Path,
    runtime_packet: &'a Path,
    epochs: usize,
    dataset_rows: usize,
    learning_rows: usize,
    candidate_bytes: u64,
    train_rows: usize,
    train_groups: usize,
    train_acc: f32,
    train_group_acc: f32,
    holdout_rows: usize,
    holdout_groups: usize,
    holdout_acc: f32,
    holdout_group_acc: f32,
    current_eval: &'a str,
    candidate_eval: &'a str,
    dry_run: bool,
    promoted: bool,
}

fn render_report(report: LoopReport<'_>) -> String {
    format!(
        r#"# NANDA Trainer Loop Report

## Inputs

- dataset: `{}`
- holdout: `{}`
- epochs: `{}`
- dataset rows: `{}`
- learning rows added: `{}`
- candidate packet: `{}`
- runtime packet: `{}`
- packet bytes: `{}`
- mode: `{}`

The report intentionally does not include raw learning-log text.

## Training

- train rows: `{}`
- train groups: `{}`
- train accuracy: `{:.3}`
- train group accuracy: `{:.3}`
- holdout rows: `{}`
- holdout groups: `{}`
- holdout accuracy: `{:.3}`
- holdout group accuracy: `{:.3}`

## Current Runtime Packet Eval

```text
{}
```

## Candidate Packet Eval

```text
{}
```

## Decision

- promoted: `{}`

Promotion is allowed only with `--promote`, no regressions, and candidate score
not worse than the current runtime packet.
"#,
        report.dataset.display(),
        report.holdout.display(),
        report.epochs,
        report.dataset_rows,
        report.learning_rows,
        report.candidate_packet.display(),
        report.runtime_packet.display(),
        report.candidate_bytes,
        if report.dry_run {
            "dry-run"
        } else {
            "promote-requested"
        },
        report.train_rows,
        report.train_groups,
        report.train_acc,
        report.train_group_acc,
        report.holdout_rows,
        report.holdout_groups,
        report.holdout_acc,
        report.holdout_group_acc,
        trim_eval(report.current_eval),
        trim_eval(report.candidate_eval),
        report.promoted
    )
}

fn append_learning_rows(rows: &mut Vec<Expert64TrainingRow>) -> usize {
    let Some(path) = learning_log_path() else {
        return 0;
    };
    let Ok(content) = fs::read_to_string(&path) else {
        return 0;
    };
    let mut added = 0usize;
    for (idx, line) in content.lines().enumerate() {
        let Ok(entry) = serde_json::from_str::<LearningEntry>(line) else {
            continue;
        };
        if !is_safe_learning_entry(&entry) {
            continue;
        }
        let before = rows.len();
        add_training_group(rows, &format!("learning:{idx}"), &entry.from, &entry.to);
        added += rows.len().saturating_sub(before);
    }
    added
}

fn is_safe_learning_entry(entry: &LearningEntry) -> bool {
    if entry.from == entry.to || entry.from.trim().is_empty() || entry.to.trim().is_empty() {
        return false;
    }
    if entry.from.chars().count() > 80 || entry.to.chars().count() > 80 {
        return false;
    }
    if entry.from.contains('\n') || entry.to.contains('\n') {
        return false;
    }
    matches!(
        entry.kind.as_deref(),
        Some("manual") | Some("user-correction") | Some("double-shift") | Some("auto-undo")
    ) || entry.lay_kind.is_some()
}

fn read_runtime_cell(path: &Path) -> Option<Expert64Cell> {
    if !path.exists() {
        return None;
    }
    Expert64Cell::read(path).ok()
}

fn trim_eval(text: &str) -> String {
    text.lines().take(80).collect::<Vec<_>>().join("\n")
}

fn learning_log_path() -> Option<PathBuf> {
    let home = env::var_os("HOME").map(PathBuf::from)?;
    Some(home.join(LEARNING_LOG))
}

fn now_stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    secs.to_string()
}

fn arg_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == name)
        .map(|window| window[1].as_str())
}

fn arg_path(args: &[String], name: &str) -> Option<PathBuf> {
    arg_value(args, name).map(PathBuf::from)
}
