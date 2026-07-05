use serde::Serialize;
use std::path::PathBuf;

use super::{run_wave_trace, WaveDecision, WaveTrace};
use crate::time::unix_timestamp;

const PRECOGNITION_PATH: &str = ".local/share/lay/nanda_wave/precognition.jsonl";

#[derive(Debug, Serialize)]
struct PrecognitionRecord<'a> {
    kind: &'static str,
    ts: u64,
    stage: &'a str,
    original: Option<&'a str>,
    text_len: usize,
    l1_packets: usize,
    l2_candidates: Vec<PrecognitionCandidate<'a>>,
    l3: Vec<&'a str>,
    decision: &'static str,
    output: Option<&'a str>,
    no_raw_secret_text: bool,
}

#[derive(Debug, Serialize)]
struct PrecognitionCandidate<'a> {
    source: &'a str,
    text: Option<&'a str>,
    energy: f32,
    risk: f32,
}

pub fn record_precognition_tick(stage: &str, text: &str, include_text: bool) {
    if text.trim().is_empty() {
        return;
    }
    let trace = run_wave_trace(text);
    let record = build_record(stage, text, include_text, &trace);
    let Some(path) = precognition_path() else {
        return;
    };
    if let Ok(line) = serde_json::to_string(&record) {
        crate::debug_log::append_private_line(path, line);
    }
    if is_informative_tick(stage, &trace) {
        super::journal::record_runtime_trace_with_text_policy(
            "runtime:precognition",
            format!("precognition:{stage}"),
            &trace,
            None,
            include_text,
        );
    }
}

fn is_informative_tick(stage: &str, trace: &WaveTrace) -> bool {
    stage == "space" || !trace.l2_candidates.is_empty() || trace.output().is_some()
}

fn build_record<'a>(
    stage: &'a str,
    text: &'a str,
    include_text: bool,
    trace: &'a WaveTrace,
) -> PrecognitionRecord<'a> {
    PrecognitionRecord {
        kind: "nanda_precognition_tick",
        ts: unix_timestamp(),
        stage,
        original: include_text.then_some(text),
        text_len: text.chars().count(),
        l1_packets: trace.l1.len(),
        l2_candidates: trace
            .l2_candidates
            .iter()
            .map(|candidate| PrecognitionCandidate {
                source: candidate.source,
                text: include_text.then_some(candidate.text.as_str()),
                energy: candidate.energy,
                risk: candidate.risk,
            })
            .collect(),
        l3: trace.l3.iter().map(|layer| layer.name).collect(),
        decision: decision_kind(&trace.decision),
        output: include_text.then(|| trace.output()).flatten(),
        no_raw_secret_text: !include_text,
    }
}

fn decision_kind(decision: &WaveDecision) -> &'static str {
    match decision {
        WaveDecision::Apply { .. } => "apply",
        WaveDecision::Keep { .. } => "keep",
        WaveDecision::Veto { .. } => "veto",
    }
}

fn precognition_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(PRECOGNITION_PATH))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_policy_hides_raw_precognition_text() {
        let trace = run_wave_trace("html djn ");
        let private = build_record("key", "html djn ", false, &trace);
        assert!(private.original.is_none());
        assert!(private.output.is_none());
        assert!(private
            .l2_candidates
            .iter()
            .all(|candidate| candidate.text.is_none()));

        let readable = build_record("key", "html djn ", true, &trace);
        assert_eq!(readable.original, Some("html djn "));
        assert!(readable
            .l2_candidates
            .iter()
            .any(|candidate| candidate.text.is_some()));
    }

    #[test]
    fn informative_tick_keeps_word_boundaries_and_candidates() {
        let trace = run_wave_trace("html djn ");
        assert!(is_informative_tick("space", &trace));
        assert!(is_informative_tick("key", &trace));
    }
}
