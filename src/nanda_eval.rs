use crate::config::{default_typing_assist_pipeline, CorrectionSafety};
use crate::microbrain::{Expert64Cell, MicroDecisionTrace, MicrobrainOptions};
use crate::typing_assist::{
    explain_typing_assist_with_microbrain_options, explain_typing_assist_with_pipeline,
};
use crate::typing_context::typing_assist_pipeline_for_context;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalCase {
    pub original: String,
    pub expected: String,
    pub reason: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EvalStats {
    pub cases: usize,
    pub ok: usize,
    pub changed: usize,
    pub improved: usize,
    pub worsened: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReasonStats {
    pub cases: usize,
    pub baseline_ok: usize,
    pub nanda_ok: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceStats {
    pub chosen: usize,
    pub generated: usize,
    pub candidates: usize,
    pub expert_avg: BTreeMap<&'static str, f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvalResult {
    pub output: String,
    pub ok: bool,
    pub trace: Option<MicroDecisionTrace>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NandaEvalReport {
    pub baseline: Vec<EvalResult>,
    pub nanda: Vec<EvalResult>,
    pub baseline_stats: EvalStats,
    pub nanda_stats: EvalStats,
    pub compare: EvalStats,
    pub by_reason: BTreeMap<String, ReasonStats>,
    pub trace_stats: TraceStats,
}

pub fn read_cases(path: &Path) -> io::Result<Vec<EvalCase>> {
    let text = fs::read_to_string(path)?;
    if text
        .lines()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| line.starts_with("group_id\t"))
    {
        return Ok(read_grouped_training_cases(&text));
    }
    let mut cases = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 3 {
            continue;
        }
        cases.push(EvalCase {
            original: decode_fixture(cols[0]),
            expected: decode_fixture(cols[1]),
            reason: cols[2].to_string(),
        });
    }
    Ok(cases)
}

pub fn evaluate_report(
    cases: &[EvalCase],
    safety: CorrectionSafety,
    options: &MicrobrainOptions,
) -> NandaEvalReport {
    let baseline = evaluate(cases, safety, None);
    evaluate_report_with_baseline(cases, safety, &baseline, options)
}

pub fn evaluate_report_with_baseline(
    cases: &[EvalCase],
    safety: CorrectionSafety,
    baseline: &[EvalResult],
    options: &MicrobrainOptions,
) -> NandaEvalReport {
    let nanda = evaluate(cases, safety, Some(options));
    build_report(cases, baseline.to_vec(), nanda)
}

pub fn evaluate_with_packet(
    cases: &[EvalCase],
    safety: CorrectionSafety,
    disabled: &[String],
    cell: Option<Expert64Cell>,
) -> NandaEvalReport {
    let mut options = MicrobrainOptions::with_disabled(disabled);
    if let Some(cell) = cell {
        options = options.with_trained_layout_signal(cell);
    }
    evaluate_report(cases, safety, &options)
}

pub fn evaluate_with_packet_and_baseline(
    cases: &[EvalCase],
    safety: CorrectionSafety,
    baseline: &[EvalResult],
    disabled: &[String],
    cell: Option<Expert64Cell>,
) -> NandaEvalReport {
    let mut options = MicrobrainOptions::with_disabled(disabled);
    if let Some(cell) = cell {
        options = options.with_trained_layout_signal(cell);
    }
    evaluate_report_with_baseline(cases, safety, baseline, &options)
}

pub fn evaluate(
    cases: &[EvalCase],
    safety: CorrectionSafety,
    options: Option<&MicrobrainOptions>,
) -> Vec<EvalResult> {
    let configured = default_typing_assist_pipeline();
    cases
        .iter()
        .map(|case| {
            let pipeline =
                typing_assist_pipeline_for_context(true, safety, &configured, &case.original);
            let explanation = if let Some(options) = options {
                explain_typing_assist_with_microbrain_options(
                    &case.original,
                    true,
                    &pipeline,
                    options,
                )
            } else {
                explain_typing_assist_with_pipeline(&case.original, true, &pipeline)
            };
            let output = explanation
                .output
                .clone()
                .unwrap_or_else(|| case.original.clone());
            EvalResult {
                ok: output == case.expected,
                output,
                trace: explanation.microbrain,
            }
        })
        .collect()
}

pub fn render_eval_report(report: &NandaEvalReport, show_changes: bool) -> String {
    let mut out = String::new();
    push_summary(&mut out, "baseline", &report.baseline_stats);
    push_summary(&mut out, "nanda", &report.nanda_stats);
    out.push_str(&format!(
        "compare: changed={} improved={} worsened={}\n",
        report.compare.changed, report.compare.improved, report.compare.worsened
    ));
    out.push_str("by_reason:\n");
    for (reason, stats) in &report.by_reason {
        out.push_str(&format!(
            "  {reason}: baseline={}/{} nanda={}/{}\n",
            stats.baseline_ok, stats.cases, stats.nanda_ok, stats.cases
        ));
    }
    out.push_str(&format!(
        "nanda_trace: chosen={} generated={} candidates={}\n",
        report.trace_stats.chosen, report.trace_stats.generated, report.trace_stats.candidates
    ));
    out.push_str("nanda_experts_avg:\n");
    for (expert, avg) in &report.trace_stats.expert_avg {
        out.push_str(&format!("  {expert}: {avg:.3}\n"));
    }
    if show_changes {
        push_changed_cases(&mut out, &report.baseline, &report.nanda);
    }
    out
}

fn build_report(
    cases: &[EvalCase],
    baseline: Vec<EvalResult>,
    nanda: Vec<EvalResult>,
) -> NandaEvalReport {
    let baseline_stats = summarize(&baseline);
    let nanda_stats = summarize(&nanda);
    let compare = compare(&baseline, &nanda);
    let by_reason = reason_stats(cases, &baseline, &nanda);
    let trace_stats = trace_stats(&nanda);
    NandaEvalReport {
        baseline,
        nanda,
        baseline_stats,
        nanda_stats,
        compare,
        by_reason,
        trace_stats,
    }
}

fn summarize(results: &[EvalResult]) -> EvalStats {
    EvalStats {
        cases: results.len(),
        ok: results.iter().filter(|result| result.ok).count(),
        ..EvalStats::default()
    }
}

fn compare(baseline: &[EvalResult], nanda: &[EvalResult]) -> EvalStats {
    let mut stats = EvalStats {
        cases: baseline.len(),
        ..EvalStats::default()
    };
    for (base, nand) in baseline.iter().zip(nanda) {
        stats.changed += usize::from(base.output != nand.output);
        stats.improved += usize::from(!base.ok && nand.ok);
        stats.worsened += usize::from(base.ok && !nand.ok);
    }
    stats
}

fn reason_stats(
    cases: &[EvalCase],
    baseline: &[EvalResult],
    nanda: &[EvalResult],
) -> BTreeMap<String, ReasonStats> {
    let mut by_reason = BTreeMap::new();
    for ((case, base), nand) in cases.iter().zip(baseline).zip(nanda) {
        let stats: &mut ReasonStats = by_reason.entry(case.reason.clone()).or_default();
        stats.cases += 1;
        stats.baseline_ok += usize::from(base.ok);
        stats.nanda_ok += usize::from(nand.ok);
    }
    by_reason
}

fn trace_stats(results: &[EvalResult]) -> TraceStats {
    let mut chosen = 0usize;
    let mut generated = 0usize;
    let mut candidates = 0usize;
    let mut expert_scores: BTreeMap<&'static str, (usize, f32)> = BTreeMap::new();
    for result in results {
        let Some(trace) = &result.trace else {
            continue;
        };
        chosen += usize::from(trace.chosen.is_some());
        generated += trace.generated.len();
        candidates += trace.candidates.len();
        for candidate in &trace.candidates {
            for score in &candidate.expert_scores {
                let entry = expert_scores.entry(score.expert).or_default();
                entry.0 += 1;
                entry.1 += score.confidence;
            }
        }
    }
    let expert_avg = expert_scores
        .into_iter()
        .map(|(expert, (count, sum))| (expert, sum / count as f32))
        .collect();
    TraceStats {
        chosen,
        generated,
        candidates,
        expert_avg,
    }
}

fn push_summary(out: &mut String, name: &str, stats: &EvalStats) {
    out.push_str(&format!(
        "{name}: cases={} ok={}/{} {:.1}%\n",
        stats.cases,
        stats.ok,
        stats.cases,
        percent(stats.ok, stats.cases)
    ));
}

fn push_changed_cases(out: &mut String, left: &[EvalResult], right: &[EvalResult]) {
    for (idx, (left, right)) in left.iter().zip(right).enumerate() {
        if left.output == right.output && left.ok == right.ok {
            continue;
        }
        out.push_str(&format!(
            "  case#{idx} left={:?} right={:?} left_ok={} right_ok={}\n",
            left.output, right.output, left.ok, right.ok
        ));
    }
}

fn read_grouped_training_cases(text: &str) -> Vec<EvalCase> {
    let mut cases = Vec::new();
    let mut current_group = String::new();
    let mut current_original = String::new();
    let mut current_expected = String::new();
    let mut current_reason = String::new();

    for (idx, line) in text.lines().enumerate() {
        if idx == 0 || line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 8 {
            continue;
        }
        let group_id = cols[0];
        if !current_group.is_empty() && current_group != group_id {
            push_grouped_case(
                &mut cases,
                &current_original,
                &current_expected,
                &current_reason,
            );
            current_original.clear();
            current_expected.clear();
            current_reason.clear();
        }
        current_group = group_id.to_string();
        if current_original.is_empty() {
            current_original = cols[2].to_string();
        }
        if cols[5] == "1" {
            current_expected = cols[3].to_string();
            current_reason = grouped_reason(cols[4], cols[7]).to_string();
        }
    }
    push_grouped_case(
        &mut cases,
        &current_original,
        &current_expected,
        &current_reason,
    );
    cases
}

fn push_grouped_case(cases: &mut Vec<EvalCase>, original: &str, expected: &str, reason: &str) {
    if original.is_empty() || expected.is_empty() || reason.is_empty() {
        return;
    }
    cases.push(EvalCase {
        original: original.to_string(),
        expected: expected.to_string(),
        reason: reason.to_string(),
    });
}

fn grouped_reason(operation: &str, raw_reason: &str) -> &'static str {
    match operation {
        "layout" => "layout",
        "split" => "split_glued_phrase",
        "typo" => "ru_typo",
        "mixed" => "mixed_context",
        "keep" if raw_reason.contains("technical") => "technical_keep",
        "keep" => "keep",
        _ => "other",
    }
}

fn decode_fixture(value: &str) -> String {
    value.replace("\\s", " ")
}

fn percent(num: usize, den: usize) -> f64 {
    if den == 0 {
        0.0
    } else {
        num as f64 * 100.0 / den as f64
    }
}
