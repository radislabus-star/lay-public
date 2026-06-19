use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::journal::{CellTraceCell, CellTraceMode, CellTracePattern, CellTraceRecord};

const MEMORY_PATH: &str = ".local/share/lay/nanda_wave/resonance_memory.json";

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ResonanceMemory {
    pub kind: String,
    pub updated_at: u64,
    pub records: u64,
    pub entries: BTreeMap<String, ResonanceEntry>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ResonanceEntry {
    pub key: String,
    pub cell: String,
    pub role: String,
    pub mode_id: Option<usize>,
    pub seen: u64,
    pub accepted: u64,
    pub vetoed: u64,
    pub kept: u64,
    pub reinforced: u64,
    pub suppressed: u64,
    pub observed: u64,
    pub energy_ema: f32,
    pub coherence_ema: f32,
    pub trust: f32,
    pub last_seen: u64,
}

impl ResonanceEntry {
    pub fn status(&self) -> &'static str {
        if self.suppressed > self.reinforced && self.suppressed > 0 {
            "suppressed"
        } else if self.reinforced > 0 {
            "reinforced"
        } else if self.accepted > 0 || self.vetoed > 0 {
            "committed"
        } else if self.seen > 0 {
            "observed"
        } else {
            "cold"
        }
    }
}

pub fn observe_record(record: &CellTraceRecord) {
    let Some(path) = memory_path() else {
        return;
    };
    let mut memory = load_resonance_memory();
    observe_record_into(&mut memory, record);
    if let Ok(text) = serde_json::to_string_pretty(&memory) {
        let _ = crate::private_file::write_private_text(&path, &format!("{text}\n"));
    }
}

pub fn load_resonance_memory() -> ResonanceMemory {
    memory_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| ResonanceMemory {
            kind: "nanda_wave_resonance_memory_v30".to_string(),
            ..Default::default()
        })
}

pub fn observe_record_into(memory: &mut ResonanceMemory, record: &CellTraceRecord) {
    memory.kind = "nanda_wave_resonance_memory_v30".to_string();
    memory.updated_at = record.ts;
    memory.records = memory.records.saturating_add(1);
    for cell in &record.cells {
        observe_cell(memory, cell, record);
        for mode in &cell.top_modes {
            observe_mode(memory, cell, mode, record);
        }
    }
    for pattern in &record.patterns {
        observe_pattern(memory, pattern, record);
    }
}

fn observe_cell(memory: &mut ResonanceMemory, cell: &CellTraceCell, record: &CellTraceRecord) {
    let key = format!("cell:{}:{}", cell.role, cell.cell);
    let entry = memory
        .entries
        .entry(key.clone())
        .or_insert_with(|| new_entry(key, &cell.cell, &cell.role, None));
    apply_observation(entry, cell, record, cell.top_energy, cell.top_energy);
}

fn observe_mode(
    memory: &mut ResonanceMemory,
    cell: &CellTraceCell,
    mode: &CellTraceMode,
    record: &CellTraceRecord,
) {
    let key = format!("mode:{}:{}#{}", mode.role, cell.cell, mode.mode_id);
    let entry = memory
        .entries
        .entry(key.clone())
        .or_insert_with(|| new_entry(key, &cell.cell, &mode.role, Some(mode.mode_id)));
    apply_observation(entry, cell, record, mode.energy, mode.coherence);
}

fn observe_pattern(
    memory: &mut ResonanceMemory,
    pattern: &CellTracePattern,
    record: &CellTraceRecord,
) {
    let slot_key = pattern
        .slots
        .iter()
        .map(u16::to_string)
        .collect::<Vec<_>>()
        .join(".");
    let key = format!("pattern:{}:{}:{}", pattern.class, pattern.verdict, slot_key);
    let entry = memory.entries.entry(key.clone()).or_insert_with(|| {
        new_entry(
            key,
            &pattern.cell,
            "pattern",
            pattern.slots.first().copied().map(usize::from),
        )
    });
    let cell = CellTraceCell {
        cell: pattern.cell.clone(),
        role: "pattern".to_string(),
        generated: 1,
        accepted: usize::from(record.decision == "apply"),
        vetoed: usize::from(record.decision == "veto"),
        kept: usize::from(record.decision == "keep"),
        top_energy: pattern.resonance,
        top_modes: Vec::new(),
    };
    apply_observation(entry, &cell, record, pattern.resonance, pattern.resonance);
}

fn new_entry(key: String, cell: &str, role: &str, mode_id: Option<usize>) -> ResonanceEntry {
    ResonanceEntry {
        key,
        cell: cell.to_string(),
        role: role.to_string(),
        mode_id,
        trust: 0.5,
        ..Default::default()
    }
}

fn apply_observation(
    entry: &mut ResonanceEntry,
    cell: &CellTraceCell,
    record: &CellTraceRecord,
    energy: f32,
    coherence: f32,
) {
    entry.seen = entry.seen.saturating_add(1);
    entry.accepted = entry.accepted.saturating_add(cell.accepted as u64);
    entry.vetoed = entry.vetoed.saturating_add(cell.vetoed as u64);
    entry.kept = entry.kept.saturating_add(cell.kept as u64);
    match record.output_matches_expected {
        Some(true) => entry.reinforced = entry.reinforced.saturating_add(1),
        Some(false) => entry.suppressed = entry.suppressed.saturating_add(1),
        None => entry.observed = entry.observed.saturating_add(1),
    }
    entry.energy_ema = ema(entry.energy_ema, energy, entry.seen);
    entry.coherence_ema = ema(entry.coherence_ema, coherence, entry.seen);
    entry.trust = trust(entry);
    entry.last_seen = record.ts;
}

fn ema(previous: f32, value: f32, seen: u64) -> f32 {
    if seen <= 1 {
        value.clamp(0.0, 1.0)
    } else {
        (previous * 0.82 + value.clamp(0.0, 1.0) * 0.18).clamp(0.0, 1.0)
    }
}

fn trust(entry: &ResonanceEntry) -> f32 {
    let positive = entry.reinforced as f32
        + entry.accepted as f32 * 0.35
        + entry.vetoed as f32 * 0.20
        + entry.observed as f32 * 0.05;
    let negative = entry.suppressed as f32;
    ((positive + 1.0) / (positive + negative + 2.0)).clamp(0.0, 1.0)
}

fn memory_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(MEMORY_PATH))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::journal::{CellTraceCandidate, CellTraceMode};

    #[test]
    fn accepted_mode_reinforces_resonance_memory() {
        let record = test_record(true);
        let mut memory = ResonanceMemory::default();
        observe_record_into(&mut memory, &record);

        let mode = memory
            .entries
            .get("mode:keyboard:KeyboardCell32#7")
            .expect("mode entry");
        assert_eq!(mode.reinforced, 1);
        assert_eq!(mode.suppressed, 0);
        assert!(mode.trust > 0.5);
        assert_eq!(mode.status(), "reinforced");
    }

    #[test]
    fn failed_mode_suppresses_resonance_memory() {
        let record = test_record(false);
        let mut memory = ResonanceMemory::default();
        observe_record_into(&mut memory, &record);

        let mode = memory
            .entries
            .get("mode:keyboard:KeyboardCell32#7")
            .expect("mode entry");
        assert_eq!(mode.reinforced, 0);
        assert_eq!(mode.suppressed, 1);
        assert!(mode.trust < 0.5);
        assert_eq!(mode.status(), "suppressed");
    }

    fn test_record(ok: bool) -> CellTraceRecord {
        CellTraceRecord {
            kind: "nanda_wave_cell_trace".to_string(),
            ts: 10,
            case_id: "case".to_string(),
            label: "label".to_string(),
            decision: "apply".to_string(),
            original: None,
            expected: None,
            output_matches_expected: Some(ok),
            chosen: Some("LayoutWordCell32".to_string()),
            candidates: vec![CellTraceCandidate {
                source: "LayoutWordCell32".to_string(),
                text: None,
                energy: 0.8,
                risk: 0.1,
                accepted: true,
            }],
            cells: vec![CellTraceCell {
                cell: "KeyboardCell32".to_string(),
                role: "signal".to_string(),
                generated: 0,
                accepted: 1,
                vetoed: 0,
                kept: 0,
                top_energy: 0.9,
                top_modes: vec![CellTraceMode {
                    mode_id: 7,
                    role: "keyboard".to_string(),
                    energy: 0.9,
                    phase: 12,
                    coherence: 0.8,
                }],
            }],
            patterns: Vec::new(),
            no_raw_secret_text: true,
        }
    }
}
