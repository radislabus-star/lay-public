use super::l3::{l3_context_field_readout, run_l3_with_options, L3_CONTEXT_FIELD_CELL};
use super::{run_wave_trace_with_options, WaveDecision, WaveOptions, WaveTrace};
use crate::eval_cases::EvalCase;
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Default)]
struct Metrics {
    context_eligible_cases: usize,
    memory_warm_cases: usize,
    cases_with_l2_candidates: usize,
    evidence_hit_cases: usize,
    sequential_evidence_cases: usize,
    scene_evidence_cases: usize,
    scene_only_cases: usize,
    authority_cases: usize,
    support_cases: usize,
    suppress_cases: usize,
    candidates_scored: usize,
    evidence_candidates: usize,
    support_candidates: usize,
    suppress_candidates: usize,
    correct_candidate_present_cases: usize,
    correct_candidate_supported_cases: usize,
    correct_candidate_suppressed_cases: usize,
    wrong_candidate_supported_cases: usize,
    candidate_lattice_drift_cases: usize,
    output_changed_cases: usize,
    improved_cases: usize,
    worsened_cases: usize,
    full_ok: usize,
    without_context_ok: usize,
}

#[derive(Default)]
struct DepthMetrics {
    cases: usize,
    evidence: usize,
    authority: usize,
    output_changed: usize,
    improved: usize,
    worsened: usize,
    full_ok: usize,
    without_context_ok: usize,
}

pub(super) fn report_json(cases: &[EvalCase], full_cases: usize) -> Value {
    super::warm_up_l3_phrase_memory();
    let without_context = WaveOptions::with_disabled(&[L3_CONTEXT_FIELD_CELL.to_string()]);
    let mut metrics = Metrics::default();
    let mut depth = BTreeMap::<&'static str, DepthMetrics>::new();

    for case in cases {
        let full = run_wave_trace_with_options(&case.original, &WaveOptions::default());
        let (_ablated_trace, ablated_decision) =
            run_l3_with_options(&case.original, &full.l2_candidates, &without_context);
        let readout = l3_context_field_readout(&case.original, &full.l2_candidates);
        let full_output = output(&full, &case.original);
        let ablated_output = decision_output(&ablated_decision, &case.original);
        let full_ok = full_output == case.expected;
        let ablated_ok = ablated_output == case.expected;

        metrics.full_ok += usize::from(full_ok);
        metrics.without_context_ok += usize::from(ablated_ok);
        metrics.cases_with_l2_candidates += usize::from(!full.l2_candidates.is_empty());

        if !readout.eligible {
            continue;
        }
        metrics.context_eligible_cases += 1;
        metrics.memory_warm_cases += usize::from(readout.memory_warm);

        let expected = case.expected.trim_end();
        let mut case_evidence = false;
        let mut case_sequential = false;
        let mut case_scene = false;
        let mut case_support = false;
        let mut case_suppress = false;
        let mut correct_present = false;
        let mut correct_supported = false;
        let mut correct_suppressed = false;
        let mut wrong_supported = false;

        for candidate in &readout.candidates {
            metrics.candidates_scored += 1;
            metrics.evidence_candidates += usize::from(candidate.evidence);
            metrics.support_candidates += usize::from(candidate.disposition == "support");
            metrics.suppress_candidates += usize::from(candidate.disposition == "suppress");
            case_evidence |= candidate.evidence;
            case_sequential |= candidate.sequential_score > 0.0;
            case_scene |= candidate.scene_score > 0.0;
            case_support |= candidate.disposition == "support";
            case_suppress |= candidate.disposition == "suppress";

            let correct = candidate.text.trim_end() == expected;
            correct_present |= correct;
            correct_supported |= correct && candidate.disposition == "support";
            correct_suppressed |= correct && candidate.disposition == "suppress";
            wrong_supported |= !correct && candidate.disposition == "support";
        }

        metrics.evidence_hit_cases += usize::from(case_evidence);
        metrics.sequential_evidence_cases += usize::from(case_sequential);
        metrics.scene_evidence_cases += usize::from(case_scene);
        metrics.scene_only_cases += usize::from(case_scene && !case_sequential);
        metrics.authority_cases += usize::from(case_support || case_suppress);
        metrics.support_cases += usize::from(case_support);
        metrics.suppress_cases += usize::from(case_suppress);
        metrics.correct_candidate_present_cases += usize::from(correct_present);
        metrics.correct_candidate_supported_cases += usize::from(correct_supported);
        metrics.correct_candidate_suppressed_cases += usize::from(correct_suppressed);
        metrics.wrong_candidate_supported_cases += usize::from(wrong_supported);

        let changed = full_output != ablated_output;
        let improved = full_ok && !ablated_ok;
        let worsened = !full_ok && ablated_ok;
        metrics.output_changed_cases += usize::from(changed);
        metrics.improved_cases += usize::from(improved);
        metrics.worsened_cases += usize::from(worsened);

        let bucket = depth_bucket(readout.context_tokens);
        let bucket = depth.entry(bucket).or_default();
        bucket.cases += 1;
        bucket.evidence += usize::from(case_evidence);
        bucket.authority += usize::from(case_support || case_suppress);
        bucket.output_changed += usize::from(changed);
        bucket.improved += usize::from(improved);
        bucket.worsened += usize::from(worsened);
        bucket.full_ok += usize::from(full_ok);
        bucket.without_context_ok += usize::from(ablated_ok);
    }

    let verdict = verdict(&metrics);
    let depth = depth
        .into_iter()
        .map(|(name, row)| {
            (
                name.to_string(),
                json!({
                    "cases": row.cases,
                    "evidence_cases": row.evidence,
                    "evidence_percent": percent(row.evidence, row.cases),
                    "authority_cases": row.authority,
                    "output_changed_cases": row.output_changed,
                    "improved_cases": row.improved,
                    "worsened_cases": row.worsened,
                    "full_ok": row.full_ok,
                    "without_context_ok": row.without_context_ok,
                    "accuracy_delta": row.full_ok as isize - row.without_context_ok as isize,
                }),
            )
        })
        .collect::<serde_json::Map<String, Value>>();

    json!({
        "kind": "l3_context_report",
        "verdict": verdict,
        "cases": cases.len(),
        "full_cases": full_cases,
        "sampled": cases.len() < full_cases,
        "proof_contract": {
            "source": "real eval suite, not live logs",
            "ablation": L3_CONTEXT_FIELD_CELL,
            "same_l2_candidate_lattice_required": true,
            "candidate_lattice_drift_cases": metrics.candidate_lattice_drift_cases,
        },
        "context_coverage": {
            "eligible_cases": metrics.context_eligible_cases,
            "eligible_percent": percent(metrics.context_eligible_cases, cases.len()),
            "memory_warm_cases": metrics.memory_warm_cases,
            "cases_with_l2_candidates": metrics.cases_with_l2_candidates,
            "evidence_hit_cases": metrics.evidence_hit_cases,
            "evidence_hit_percent_of_eligible": percent(metrics.evidence_hit_cases, metrics.context_eligible_cases),
            "sequential_evidence_cases": metrics.sequential_evidence_cases,
            "scene_evidence_cases": metrics.scene_evidence_cases,
            "scene_only_cases": metrics.scene_only_cases,
            "authority_cases": metrics.authority_cases,
            "authority_percent_of_eligible": percent(metrics.authority_cases, metrics.context_eligible_cases),
            "support_cases": metrics.support_cases,
            "suppress_cases": metrics.suppress_cases,
        },
        "candidate_field": {
            "candidates_scored": metrics.candidates_scored,
            "evidence_candidates": metrics.evidence_candidates,
            "support_candidates": metrics.support_candidates,
            "suppress_candidates": metrics.suppress_candidates,
            "correct_candidate_present_cases": metrics.correct_candidate_present_cases,
            "correct_candidate_supported_cases": metrics.correct_candidate_supported_cases,
            "correct_candidate_suppressed_cases": metrics.correct_candidate_suppressed_cases,
            "wrong_candidate_supported_cases": metrics.wrong_candidate_supported_cases,
        },
        "causal_ablation": {
            "full_ok": metrics.full_ok,
            "without_context_ok": metrics.without_context_ok,
            "accuracy_delta": metrics.full_ok as isize - metrics.without_context_ok as isize,
            "output_changed_cases": metrics.output_changed_cases,
            "improved_cases": metrics.improved_cases,
            "worsened_cases": metrics.worsened_cases,
        },
        "by_context_depth": depth,
        "read_as": "L3 is context-connected only when evidence_hit_cases is nonzero; it is decision-active only when authority and output_changed are nonzero; positive utility requires improved_cases > worsened_cases under an unchanged L2 candidate lattice"
    })
}

fn output(trace: &WaveTrace, original: &str) -> String {
    trace
        .output()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| original.to_string())
}

fn decision_output(decision: &WaveDecision, original: &str) -> String {
    match decision {
        WaveDecision::Suggest { text, .. } => text.clone(),
        WaveDecision::Keep { .. } | WaveDecision::Veto { .. } => original.to_string(),
    }
}

fn depth_bucket(context_tokens: usize) -> &'static str {
    match context_tokens {
        0..=2 => "2",
        3 => "3",
        4..=7 => "4-7",
        _ => "8+",
    }
}

fn percent(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 * 100.0 / denominator as f64
    }
}

fn verdict(metrics: &Metrics) -> &'static str {
    if metrics.candidate_lattice_drift_cases > 0 {
        "INVALID_CANDIDATE_LATTICE_DRIFT"
    } else if metrics.context_eligible_cases == 0 {
        "NO_CONTEXT_CASES"
    } else if metrics.evidence_hit_cases == 0 {
        "L3_CONTEXT_DISCONNECTED"
    } else if metrics.authority_cases == 0 || metrics.output_changed_cases == 0 {
        "L3_CONTEXT_OBSERVED_NOT_DECISIVE"
    } else if metrics.improved_cases > metrics.worsened_cases {
        "L3_CONTEXT_ACTIVE_POSITIVE"
    } else {
        "L3_CONTEXT_ACTIVE_WATCH"
    }
}
