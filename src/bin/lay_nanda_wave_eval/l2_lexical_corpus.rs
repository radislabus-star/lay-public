use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

const DEFAULT_CANDIDATE_LIMIT: usize = 16;
const DEFAULT_EXAMPLE_LIMIT: usize = 12;
const WORK_CHUNK_ROWS: usize = 32;

#[derive(Debug)]
struct EvalRow {
    language: &'static str,
    operation: String,
    input: String,
    expected: String,
}

#[derive(Debug, Default, Serialize)]
struct Metrics {
    rows: usize,
    candidate_rows: usize,
    expected_any: usize,
    expected_top1: usize,
    expected_top3: usize,
    wrong_top1: usize,
    candidate_total: usize,
    #[serde(skip)]
    latency_us: Vec<u64>,
    missed_examples: Vec<String>,
}

#[derive(Debug, Default)]
struct EvalAccumulator {
    total: Metrics,
    by_operation: BTreeMap<String, Metrics>,
    by_language: BTreeMap<String, Metrics>,
}

pub(crate) fn print_json(args: &[String]) -> io::Result<()> {
    let input = super::arg_value(args, "--l2-lexical-corpus-eval")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing eval corpus"))?;
    let candidate_limit = usize_arg(args, "--candidate-limit", DEFAULT_CANDIDATE_LIMIT, 1, 64);
    let example_limit = usize_arg(args, "--max-examples", DEFAULT_EXAMPLE_LIMIT, 0, 100);
    let jobs = usize_arg(
        args,
        "--jobs",
        std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1),
        1,
        256,
    );
    let row_limit = super::arg_value(args, "--limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(usize::MAX);
    let (rows, malformed_rows, skipped_layout_rows) = load_rows(&input, row_limit)?;

    // Artifact loading and page faults are startup costs, not candidate latency.
    let _ = lay::nanda_wave::l2::correction_l2_word_candidates("", "проверка", candidate_limit);
    let _ = lay::nanda_wave::l2::correction_l2_word_candidates("", "example", candidate_limit);
    let memory = lay::nanda_wave::eval::runtime_l2_lexical_memory_stats();
    let rows = Arc::new(rows);
    let started = Instant::now();
    let report = evaluate_rows(Arc::clone(&rows), jobs, candidate_limit, example_limit);
    let elapsed_ms = started.elapsed().as_millis();
    let hot_latency_us = measure_hot_latency(&rows, candidate_limit, 2_000);
    let artifact = std::env::var_os("LAY_L2_LEXICAL_PHASE_MEMORY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("default"));

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "lay.l2-lexical-corpus-eval.v1",
            "verdict": if report.total.rows == 0 { "WATCH_NO_ELIGIBLE_ROWS" } else { "SHADOW_MEASURED" },
            "input": input,
            "artifact": artifact,
            "memory": memory,
            "jobs": jobs,
            "candidate_limit": candidate_limit,
            "elapsed_ms": elapsed_ms,
            "throughput_rows_per_second": if elapsed_ms == 0 { 0.0 } else { report.total.rows as f64 * 1_000.0 / elapsed_ms as f64 },
            "hot_latency_us": latency_json(&hot_latency_us),
            "malformed_rows": malformed_rows,
            "skipped_layout_rows": skipped_layout_rows,
            "corruption_rows_are_eval_only": true,
            "live_authority": false,
            "total": metrics_json(&report.total),
            "by_operation": metrics_map_json(&report.by_operation),
            "by_language": metrics_map_json(&report.by_language),
        }))?
    );
    Ok(())
}

fn load_rows(path: &Path, limit: usize) -> io::Result<(Vec<EvalRow>, usize, usize)> {
    let reader = BufReader::new(File::open(path)?);
    let mut rows = Vec::new();
    let mut malformed = 0usize;
    let mut skipped_layout = 0usize;
    for (line_index, line) in reader.lines().enumerate() {
        let line = line?;
        if line_index == 0 && line.starts_with("group_id\t") {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() < 8 {
            malformed += 1;
            continue;
        }
        if columns[5] != "1" {
            continue;
        }
        if columns[4] == "layout" {
            // Layout projection has a separate typed operator and must not distort lexical recall.
            skipped_layout += 1;
            continue;
        }
        let language = if columns[3].chars().all(|ch| ch.is_ascii_alphabetic()) {
            "en"
        } else {
            "ru"
        };
        rows.push(EvalRow {
            language,
            operation: columns[4].to_string(),
            input: columns[2].to_string(),
            expected: columns[3].to_string(),
        });
        if rows.len() >= limit {
            break;
        }
    }
    Ok((rows, malformed, skipped_layout))
}

fn evaluate_rows(
    rows: Arc<Vec<EvalRow>>,
    jobs: usize,
    candidate_limit: usize,
    example_limit: usize,
) -> EvalAccumulator {
    let cursor = Arc::new(AtomicUsize::new(0));
    let mut workers = Vec::with_capacity(jobs);
    for _ in 0..jobs {
        let rows = Arc::clone(&rows);
        let cursor = Arc::clone(&cursor);
        workers.push(std::thread::spawn(move || {
            let mut report = EvalAccumulator::default();
            loop {
                let start = cursor.fetch_add(WORK_CHUNK_ROWS, Ordering::Relaxed);
                if start >= rows.len() {
                    break;
                }
                for row in &rows[start..rows.len().min(start + WORK_CHUNK_ROWS)] {
                    let started = Instant::now();
                    let candidates = lay::nanda_wave::l2::correction_l2_word_candidates(
                        "",
                        &row.input,
                        candidate_limit,
                    );
                    let latency_us = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
                    observe(
                        &mut report.total,
                        row,
                        &candidates,
                        latency_us,
                        example_limit,
                    );
                    observe(
                        report
                            .by_operation
                            .entry(row.operation.clone())
                            .or_default(),
                        row,
                        &candidates,
                        latency_us,
                        example_limit,
                    );
                    observe(
                        report
                            .by_language
                            .entry(row.language.to_string())
                            .or_default(),
                        row,
                        &candidates,
                        latency_us,
                        example_limit,
                    );
                }
            }
            report
        }));
    }

    let mut combined = EvalAccumulator::default();
    for worker in workers {
        merge_accumulator(
            &mut combined,
            worker.join().expect("lexical eval worker panicked"),
        );
    }
    truncate_examples(&mut combined, example_limit);
    combined
}

fn measure_hot_latency(rows: &[EvalRow], candidate_limit: usize, sample_limit: usize) -> Vec<u64> {
    if rows.is_empty() || sample_limit == 0 {
        return Vec::new();
    }
    let sample_size = rows.len().min(sample_limit);
    let stride = rows.len().div_ceil(sample_size);
    let mut latency = Vec::with_capacity(sample_size);
    for row in rows.iter().step_by(stride).take(sample_size) {
        let started = Instant::now();
        let _ = lay::nanda_wave::l2::correction_l2_word_candidates("", &row.input, candidate_limit);
        latency.push(started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64);
    }
    latency
}

fn observe(
    metrics: &mut Metrics,
    row: &EvalRow,
    candidates: &[lay::nanda_wave::l2::L2ImeWordCandidate],
    latency_us: u64,
    example_limit: usize,
) {
    metrics.rows += 1;
    metrics.candidate_total += candidates.len();
    metrics.latency_us.push(latency_us);
    if !candidates.is_empty() {
        metrics.candidate_rows += 1;
    }
    let expected_rank = candidates
        .iter()
        .position(|candidate| candidate.surface == row.expected);
    if expected_rank.is_some() {
        metrics.expected_any += 1;
    }
    if expected_rank == Some(0) {
        metrics.expected_top1 += 1;
    } else if !candidates.is_empty() {
        metrics.wrong_top1 += 1;
    }
    if expected_rank.is_some_and(|rank| rank < 3) {
        metrics.expected_top3 += 1;
    }
    if expected_rank.is_none() && metrics.missed_examples.len() < example_limit {
        metrics.missed_examples.push(format!(
            "{} -> {} [{}] top={}",
            row.input,
            row.expected,
            row.operation,
            candidates
                .first()
                .map(|candidate| candidate.surface.as_str())
                .unwrap_or("none")
        ));
    }
}

fn merge_accumulator(target: &mut EvalAccumulator, source: EvalAccumulator) {
    merge_metrics(&mut target.total, source.total);
    for (key, metrics) in source.by_operation {
        merge_metrics(target.by_operation.entry(key).or_default(), metrics);
    }
    for (key, metrics) in source.by_language {
        merge_metrics(target.by_language.entry(key).or_default(), metrics);
    }
}

fn merge_metrics(target: &mut Metrics, mut source: Metrics) {
    target.rows += source.rows;
    target.candidate_rows += source.candidate_rows;
    target.expected_any += source.expected_any;
    target.expected_top1 += source.expected_top1;
    target.expected_top3 += source.expected_top3;
    target.wrong_top1 += source.wrong_top1;
    target.candidate_total += source.candidate_total;
    target.latency_us.append(&mut source.latency_us);
    target.missed_examples.append(&mut source.missed_examples);
}

fn truncate_examples(report: &mut EvalAccumulator, limit: usize) {
    report.total.missed_examples.truncate(limit);
    for metrics in report.by_operation.values_mut() {
        metrics.missed_examples.truncate(limit);
    }
    for metrics in report.by_language.values_mut() {
        metrics.missed_examples.truncate(limit);
    }
}

fn metrics_map_json(metrics: &BTreeMap<String, Metrics>) -> serde_json::Value {
    serde_json::Value::Object(
        metrics
            .iter()
            .map(|(key, value)| (key.clone(), metrics_json(value)))
            .collect(),
    )
}

fn metrics_json(metrics: &Metrics) -> serde_json::Value {
    let mut latency = metrics.latency_us.clone();
    latency.sort_unstable();
    serde_json::json!({
        "rows": metrics.rows,
        "candidate_rows": metrics.candidate_rows,
        "candidate_rate_pct": percent(metrics.candidate_rows, metrics.rows),
        "coverage": metrics.expected_any,
        "coverage_pct": percent(metrics.expected_any, metrics.rows),
        "top1": metrics.expected_top1,
        "top1_pct": percent(metrics.expected_top1, metrics.rows),
        "top3": metrics.expected_top3,
        "top3_pct": percent(metrics.expected_top3, metrics.rows),
        "wrong_top1": metrics.wrong_top1,
        "average_candidates": if metrics.rows == 0 { 0.0 } else { metrics.candidate_total as f64 / metrics.rows as f64 },
        "parallel_query_wall_us": latency_json(&latency),
        "missed_examples": metrics.missed_examples,
    })
}

fn latency_json(latency: &[u64]) -> serde_json::Value {
    let mut latency = latency.to_vec();
    latency.sort_unstable();
    serde_json::json!({
        "samples": latency.len(),
        "p50": percentile(&latency, 50),
        "p90": percentile(&latency, 90),
        "p99": percentile(&latency, 99),
        "max": latency.last().copied().unwrap_or(0),
    })
}

fn percentile(sorted: &[u64], percentile: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let index = sorted.len().saturating_mul(percentile).saturating_add(99) / 100;
    sorted[index.saturating_sub(1).min(sorted.len() - 1)]
}

fn percent(value: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        value as f64 * 100.0 / total as f64
    }
}

fn usize_arg(args: &[String], name: &str, default: usize, min: usize, max: usize) -> usize {
    super::arg_value(args, name)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
        .clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_loader_keeps_only_positive_non_layout_rows() {
        let path =
            std::env::temp_dir().join(format!("lay-l2-lexical-corpus-{}.tsv", std::process::id()));
        std::fs::write(
            &path,
            concat!(
                "group_id\tcontext\toriginal\tcandidate\toperation\tlabel\tsource\treason\n",
                "a\t\tврмея\tвремя\tadjacent_transposition\t1\tclean\tpositive\n",
                "a\t\tврмея\tвроде\tadjacent_transposition\t0\tclean\tnegative\n",
                "b\t\tdownload\tвщцтдщфв\tlayout\t1\tclean\tpositive\n",
            ),
        )
        .expect("write fixture");

        let (rows, malformed, skipped_layout) = load_rows(&path, usize::MAX).expect("load rows");
        std::fs::remove_file(path).ok();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].input, "врмея");
        assert_eq!(rows[0].expected, "время");
        assert_eq!(malformed, 0);
        assert_eq!(skipped_layout, 1);
    }

    #[test]
    fn percentile_uses_nearest_rank() {
        let values = (1..=100).collect::<Vec<_>>();
        assert_eq!(percentile(&values, 50), 50);
        assert_eq!(percentile(&values, 99), 99);
    }
}
