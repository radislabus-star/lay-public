use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;

const IBUS_DEBUG_PATH: &str = ".local/share/lay/ibus_engine_debug.jsonl";
const RECENT_LIMIT: usize = 2000;

pub(crate) fn print_json() -> io::Result<()> {
    println!("{}", serde_json::to_string_pretty(&report_json())?);
    Ok(())
}

fn report_json() -> Value {
    let Some(path) = ibus_debug_path() else {
        return json!({
            "kind": "ime_hit_rate_report",
            "status": "unavailable",
            "reason": "HOME is not set"
        });
    };
    let Ok(text) = fs::read_to_string(&path) else {
        return json!({
            "kind": "ime_hit_rate_report",
            "status": "missing",
            "source": path.display().to_string()
        });
    };
    report_from_text(&text, RECENT_LIMIT, &path)
}

fn ibus_debug_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(IBUS_DEBUG_PATH))
}

#[derive(Debug, Default)]
struct ImeHitRateReport {
    records_seen: usize,
    timing_records: usize,
    candidate_records: usize,
    no_candidate_records: usize,
    total_us: Vec<u64>,
    ascii_us: Vec<u64>,
    ru_us: Vec<u64>,
    semantic_us: Vec<u64>,
    candidate_count_sum: u64,
    candidate_count_max: u64,
    no_candidate_tokens: BTreeMap<String, usize>,
    top_candidates: BTreeMap<String, usize>,
}

fn report_from_text(text: &str, limit: usize, path: &PathBuf) -> Value {
    let lines = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    let start = lines.len().saturating_sub(limit);
    let mut report = ImeHitRateReport {
        records_seen: lines.len(),
        ..ImeHitRateReport::default()
    };
    for line in &lines[start..] {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if value.get("kind").and_then(Value::as_str) != Some("ibus_precognition_timing") {
            continue;
        }
        inspect_timing(&mut report, &value);
    }
    let candidate_rate = percent(report.candidate_records, report.timing_records);
    let no_candidate_rate = percent(report.no_candidate_records, report.timing_records);
    json!({
        "kind": "ime_hit_rate_report",
        "status": "ok",
        "source": path.display().to_string(),
        "window": {
            "records_seen": report.records_seen,
            "limit": limit,
            "timing_records": report.timing_records
        },
        "hit_rate": {
            "candidate_records": report.candidate_records,
            "no_candidate_records": report.no_candidate_records,
            "candidate_rate_percent": candidate_rate,
            "no_candidate_rate_percent": no_candidate_rate,
            "candidate_count_avg": average_count(report.candidate_count_sum, report.timing_records),
            "candidate_count_max": report.candidate_count_max
        },
        "latency_us": {
            "total": percentile_block(&mut report.total_us),
            "ascii": percentile_block(&mut report.ascii_us),
            "ru": percentile_block(&mut report.ru_us),
            "semantic": percentile_block(&mut report.semantic_us)
        },
        "no_candidate_tokens": top_counts(report.no_candidate_tokens, 12),
        "top_candidates": top_counts(report.top_candidates, 12),
        "read_as": "candidate availability and latency from live IME trace; diagnostic only"
    })
}

fn inspect_timing(report: &mut ImeHitRateReport, value: &Value) {
    report.timing_records += 1;
    let total = value.get("total_us").and_then(Value::as_u64).unwrap_or(0);
    let ascii = value.get("ascii_us").and_then(Value::as_u64).unwrap_or(0);
    let ru = value.get("ru_us").and_then(Value::as_u64).unwrap_or(0);
    let semantic = value
        .get("semantic_us")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    report.total_us.push(total);
    report.ascii_us.push(ascii);
    report.ru_us.push(ru);
    report.semantic_us.push(semantic);

    let candidates = value.get("candidates").and_then(Value::as_u64).unwrap_or(0);
    report.candidate_count_sum = report.candidate_count_sum.saturating_add(candidates);
    report.candidate_count_max = report.candidate_count_max.max(candidates);
    if candidates == 0 {
        report.no_candidate_records += 1;
        if let Some(token) = value.get("token").and_then(Value::as_str) {
            if !token.is_empty() {
                *report
                    .no_candidate_tokens
                    .entry(token.to_string())
                    .or_default() += 1;
            }
        }
    } else {
        report.candidate_records += 1;
        if let Some(top) = value.get("top").and_then(Value::as_str) {
            if !top.is_empty() {
                *report.top_candidates.entry(top.to_string()).or_default() += 1;
            }
        }
    }
}

fn percentile_block(values: &mut Vec<u64>) -> Value {
    values.sort_unstable();
    json!({
        "count": values.len(),
        "p50": percentile(values, 50),
        "p90": percentile(values, 90),
        "p99": percentile(values, 99),
        "max": values.last().copied().unwrap_or(0)
    })
}

fn percentile(values: &[u64], pct: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let index = ((values.len() - 1) * pct) / 100;
    values[index]
}

fn average_count(sum: u64, total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    ((sum as f64 / total as f64) * 100.0).round() / 100.0
}

fn percent(count: usize, total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    ((count as f64 * 10000.0 / total as f64).round()) / 100.0
}

fn top_counts(counts: BTreeMap<String, usize>, limit: usize) -> Vec<Value> {
    let mut rows = counts.into_iter().collect::<Vec<_>>();
    rows.sort_by(|(left_key, left_count), (right_key, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_key.cmp(right_key))
    });
    rows.truncate(limit);
    rows.into_iter()
        .map(|(value, count)| json!({ "value": value, "count": count }))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::report_from_text;
    use std::path::PathBuf;

    #[test]
    fn report_counts_candidates_and_latency() {
        let text = r#"
{"kind":"ibus_precognition_timing","total_us":10,"ascii_us":1,"ru_us":2,"semantic_us":3,"candidates":0,"token":"пиш","top":null}
{"kind":"ibus_precognition_timing","total_us":20,"ascii_us":2,"ru_us":3,"semantic_us":4,"candidates":2,"token":"пров","top":"ерка"}
{"kind":"other"}
"#;

        let report = report_from_text(text, 100, &PathBuf::from("ibus.jsonl"));

        assert_eq!(report["hit_rate"]["candidate_records"], 1);
        assert_eq!(report["hit_rate"]["no_candidate_records"], 1);
        assert_eq!(report["latency_us"]["total"]["max"], 20);
        assert_eq!(report["top_candidates"][0]["value"], "ерка");
    }
}
