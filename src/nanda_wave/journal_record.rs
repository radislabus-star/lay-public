use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::signal::{WaveDecision, WavePacket, WaveTrace};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CellTraceRecord {
    pub kind: String,
    pub ts: u64,
    pub case_id: String,
    pub label: String,
    pub decision: String,
    pub original: Option<String>,
    pub expected: Option<String>,
    pub output_matches_expected: Option<bool>,
    pub chosen: Option<String>,
    #[serde(default)]
    pub candidates: Vec<CellTraceCandidate>,
    #[serde(default)]
    pub patterns: Vec<CellTracePattern>,
    pub cells: Vec<CellTraceCell>,
    pub no_raw_secret_text: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CellTraceCandidate {
    pub source: String,
    pub text: Option<String>,
    pub energy: f32,
    pub risk: f32,
    pub accepted: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CellTracePattern {
    pub cell: String,
    pub class: String,
    pub verdict: String,
    pub resonance: f32,
    pub slots: Vec<u16>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CellTraceCell {
    pub cell: String,
    pub role: String,
    pub generated: usize,
    pub accepted: usize,
    pub vetoed: usize,
    pub kept: usize,
    pub top_energy: f32,
    #[serde(default)]
    pub top_modes: Vec<CellTraceMode>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CellTraceMode {
    pub mode_id: usize,
    pub role: String,
    pub energy: f32,
    pub phase: i8,
    pub coherence: f32,
}

pub fn build_trace_record(
    ts: u64,
    case_id: String,
    label: String,
    trace: &WaveTrace,
    expected: Option<&str>,
    include_text: bool,
) -> CellTraceRecord {
    let output = trace.output();
    let ok = expected.map(|expected| output.unwrap_or(&trace.original) == expected);
    CellTraceRecord {
        kind: "nanda_wave_cell_trace".to_string(),
        ts,
        case_id,
        label,
        decision: decision_kind(&trace.decision).to_string(),
        original: include_text.then(|| trace.original.clone()),
        expected: (include_text && expected.is_some())
            .then(|| expected.unwrap_or_default().to_string()),
        output_matches_expected: ok,
        chosen: suggested_source(trace).map(ToOwned::to_owned),
        candidates: trace_candidates(trace, include_text),
        patterns: trace_patterns(trace),
        cells: trace_cells(trace),
        no_raw_secret_text: !include_text,
    }
}

fn trace_cells(trace: &WaveTrace) -> Vec<CellTraceCell> {
    let mut cells: BTreeMap<String, CellTraceCell> = BTreeMap::new();
    for packet in &trace.l1 {
        let entry = cells
            .entry(packet.cell.to_string())
            .or_insert_with(|| CellTraceCell::new(packet.cell, "signal"));
        entry.top_energy = entry.top_energy.max(packet.top_energy());
        merge_top_modes(&mut entry.top_modes, packet);
    }
    for candidate in &trace.l2_candidates {
        let entry = cells
            .entry(candidate.source.to_string())
            .or_insert_with(|| CellTraceCell::new(candidate.source, "candidate"));
        entry.generated += 1;
        entry.top_energy = entry.top_energy.max(candidate.energy);
    }
    match &trace.decision {
        WaveDecision::Suggest { .. } => {
            // A model readout is not user acceptance. The selected source is
            // recorded in `chosen`; signed memory changes only from observed
            // accept/reject/revert events.
        }
        WaveDecision::Veto { .. } => {
            cells
                .entry("TechnicalContextCell32".to_string())
                .or_insert_with(|| CellTraceCell::new("TechnicalContextCell32", "guard"))
                .vetoed += 1;
        }
        WaveDecision::Keep { .. } => {
            cells
                .entry("MeshConsensusCell32".to_string())
                .or_insert_with(|| CellTraceCell::new("MeshConsensusCell32", "consensus"))
                .kept += 1;
        }
    }
    for layer in &trace.l3 {
        let entry = cells
            .entry(layer.name.to_string())
            .or_insert_with(|| CellTraceCell::new(layer.name, l3_role(layer.name)));
        if entry.role == "l3" {
            entry.role = l3_role(layer.name).to_string();
        }
        entry.kept += usize::from(layer.name != "MeshConsensusCell32");
    }
    cells.into_values().collect()
}

fn merge_top_modes(modes: &mut Vec<CellTraceMode>, packet: &WavePacket) {
    modes.extend(packet.modes.iter().take(3).map(|mode| CellTraceMode {
        mode_id: mode.mode_id,
        role: mode.role.as_str().to_string(),
        energy: mode.energy,
        phase: mode.phase,
        coherence: mode.coherence,
    }));
    modes.sort_by(|left, right| {
        right
            .energy
            .total_cmp(&left.energy)
            .then_with(|| left.mode_id.cmp(&right.mode_id))
    });
    modes.dedup_by(|left, right| left.mode_id == right.mode_id && left.role == right.role);
    modes.truncate(6);
}

fn l3_role(name: &str) -> &str {
    match name {
        "PatternWaveCell32" => "pattern",
        "StructuralRelationCell32" => "relation",
        "L3FeedbackCell32" => "feedback",
        "LLMWaveCell32" => "memory",
        "PhraseCell32" => "phrase",
        "PhraseForecastCell32" => "forecast",
        "MeshConsensusCell32" => "consensus",
        "TechnicalContextCell32" => "guard",
        _ => "l3",
    }
}

fn trace_candidates(trace: &WaveTrace, include_text: bool) -> Vec<CellTraceCandidate> {
    trace
        .l2_candidates
        .iter()
        .map(|candidate| {
            let candidate_text = preserve_space(&trace.original, &candidate.text);
            CellTraceCandidate {
                source: candidate.source.to_string(),
                text: include_text.then_some(candidate_text.clone()),
                energy: candidate.energy,
                risk: candidate.risk,
                accepted: false,
            }
        })
        .collect()
}

fn suggested_source(trace: &WaveTrace) -> Option<&'static str> {
    let WaveDecision::Suggest { text, .. } = &trace.decision else {
        return None;
    };
    trace
        .l2_candidates
        .iter()
        .find(|candidate| preserve_space(&trace.original, &candidate.text) == *text)
        .map(|candidate| candidate.source)
}

fn trace_patterns(trace: &WaveTrace) -> Vec<CellTracePattern> {
    trace
        .l3
        .iter()
        .filter(|layer| layer.name == super::pattern_wave::PATTERN_WAVE_CELL)
        .map(|layer| CellTracePattern {
            cell: layer.name.to_string(),
            class: summary_value(&layer.summary, "class")
                .unwrap_or("unknown")
                .to_string(),
            verdict: summary_value(&layer.summary, "verdict")
                .unwrap_or("unknown")
                .to_string(),
            resonance: summary_value(&layer.summary, "resonance")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0.0),
            slots: summary_slots(&layer.summary),
        })
        .collect()
}

fn summary_value<'a>(summary: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key}=");
    summary
        .split_whitespace()
        .find_map(|part| part.strip_prefix(&prefix))
}

fn summary_slots(summary: &str) -> Vec<u16> {
    let Some(start) = summary.find("slots=[") else {
        return Vec::new();
    };
    let rest = &summary[start + "slots=[".len()..];
    let Some(end) = rest.find(']') else {
        return Vec::new();
    };
    rest[..end]
        .split(',')
        .filter_map(|item| item.parse().ok())
        .collect()
}

impl CellTraceCell {
    fn new(cell: &str, role: &str) -> Self {
        Self {
            cell: cell.to_string(),
            role: role.to_string(),
            generated: 0,
            accepted: 0,
            vetoed: 0,
            kept: 0,
            top_energy: 0.0,
            top_modes: Vec::new(),
        }
    }
}

fn decision_kind(decision: &WaveDecision) -> &'static str {
    match decision {
        WaveDecision::Suggest { .. } => "suggest",
        WaveDecision::Keep { .. } => "keep",
        WaveDecision::Veto { .. } => "veto",
    }
}

fn preserve_space(original: &str, candidate: &str) -> String {
    if original.ends_with(' ') && !candidate.ends_with(' ') {
        format!("{candidate} ")
    } else {
        candidate.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::trace::run_wave_trace;

    #[test]
    fn trace_record_lists_participating_cells() {
        let trace = run_wave_trace("html djn ");
        let record = build_trace_record(
            1,
            "case".to_string(),
            "label".to_string(),
            &trace,
            None,
            false,
        );
        assert!(record
            .cells
            .iter()
            .any(|cell| cell.cell == "KeyboardCell32"));
        assert!(record
            .cells
            .iter()
            .any(|cell| cell.cell == "LayoutWordCell32"));
    }

    #[test]
    fn trace_record_text_policy_controls_raw_words() {
        let trace = run_wave_trace("html djn api ");
        let private = build_trace_record(
            1,
            "case".to_string(),
            "label".to_string(),
            &trace,
            Some("html вот api "),
            false,
        );
        assert_eq!(private.original, None);
        assert_eq!(private.expected, None);
        assert!(private
            .candidates
            .iter()
            .all(|candidate| candidate.text.is_none()));
        assert!(private
            .patterns
            .iter()
            .all(|pattern| pattern.cell == super::super::pattern_wave::PATTERN_WAVE_CELL));
        assert!(private.no_raw_secret_text);

        let readable = build_trace_record(
            1,
            "case".to_string(),
            "label".to_string(),
            &trace,
            Some("html вот api "),
            true,
        );
        assert_eq!(readable.original.as_deref(), Some("html djn api "));
        assert_eq!(readable.expected.as_deref(), Some("html вот api "));
        assert!(readable
            .candidates
            .iter()
            .any(|candidate| candidate.text.is_some()));
        assert!(!readable.no_raw_secret_text);
    }
}
