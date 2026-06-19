use lay::config::CorrectionSafety;
use lay::eval_cases::EvalCase;
use lay::nanda_wave::context::{MAX_CONTEXT_TOKENS, MIN_CONTEXT_TOKENS};
use lay::nanda_wave::{evaluate_wave, evaluate_wave_with_options, journal, resonance_memory};
use lay::nanda_wave::{run_wave_trace_with_options, WaveDecision, WaveOptions};
use lay::{config, typing_assist, typing_context};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::real_suite;

pub(crate) fn print_status_json(refresh: bool) -> io::Result<()> {
    let cache = status_cache_path();
    if !refresh {
        if let Ok(text) = fs::read_to_string(&cache) {
            if let Ok(mut value) = serde_json::from_str::<Value>(&text) {
                refresh_live_status_fields(&mut value);
                println!("{}", serde_json::to_string_pretty(&value)?);
                return Ok(());
            }
            println!("{text}");
            return Ok(());
        }
    }
    let value = build_status_json(refresh)?;
    if let Some(parent) = cache.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&value)?;
    fs::write(cache, format!("{text}\n"))?;
    println!("{text}");
    Ok(())
}

pub(crate) fn evaluate_deterministic(
    cases: &[EvalCase],
    safety: CorrectionSafety,
) -> Vec<EvalResult> {
    let configured = config::default_typing_assist_pipeline();
    cases
        .iter()
        .map(|case| {
            let pipeline = typing_context::typing_assist_pipeline_for_context(
                true,
                safety,
                &configured,
                &case.original,
            );
            let output =
                typing_assist::explain_typing_assist_with_pipeline(&case.original, true, &pipeline)
                    .output
                    .unwrap_or_else(|| case.original.clone());
            EvalResult {
                ok: output == case.expected,
                output,
            }
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EvalResult {
    pub(crate) output: String,
    pub(crate) ok: bool,
}

fn refresh_live_status_fields(value: &mut Value) {
    if value.get("kind").and_then(Value::as_str) != Some("nanda_wave_status") {
        return;
    }
    let scoreboard = journal::load_scoreboard();
    let resonance_memory = resonance_memory::load_resonance_memory();
    value["cell_scoreboard"] = scoreboard_json(&scoreboard);
    value["resonance_memory"] = resonance_memory_json(&resonance_memory);
    value["live_scoreboard_refreshed_at_unix"] = json!(unix_now());
    value["source"] = json!("lay-nanda-wave-eval --status-json (cached gate + live scoreboard)");
}

fn build_status_json(full: bool) -> io::Result<serde_json::Value> {
    let suite = real_suite::load()?;
    let status_cases = if full {
        suite.cases.clone()
    } else {
        status_sample_cases(&suite.cases)
    };
    let options = WaveOptions::default();
    let safety = CorrectionSafety::Experimental;
    let baseline = evaluate_deterministic(&status_cases, safety);
    let (wave, wave_stats) = evaluate_wave_with_options(&status_cases, &options);
    let baseline_ok = baseline.iter().filter(|result| result.ok).count();
    let worsened = status_cases
        .iter()
        .zip(baseline.iter())
        .zip(wave.iter())
        .filter(|((_, base), wave)| base.ok && !wave.ok)
        .count();
    let promotion_status = if wave_stats.ok >= baseline_ok && worsened == 0 {
        "gate_green_but_manual_review_required"
    } else {
        "trace_only_do_not_promote"
    };
    let mode_status = if promotion_status == "gate_green_but_manual_review_required" {
        "ensemble_mode_candidate"
    } else {
        "ensemble_mode_not_found"
    };
    let ablation = ablation_json(&status_cases);
    let candidate_stats = candidate_stats_json(&status_cases, &options);
    let layer_impact = layer_impact_json(&ablation);
    let scoreboard = journal::load_scoreboard();
    let resonance_memory = resonance_memory::load_resonance_memory();
    Ok(json!({
        "kind": "nanda_wave_status",
        "generated_at_unix": unix_now(),
        "source": "lay-nanda-wave-eval --status-json",
        "cell": {
            "name": "NandaCell32v0",
            "bytes": 32768,
            "mode_bytes": 8,
            "modes": 2048,
            "top_k": 8,
            "sparse_probes": 64
        },
        "gate": {
            "cases": status_cases.len(),
            "full_cases": suite.cases.len(),
            "sampled": status_cases.len() < suite.cases.len(),
            "baseline_ok": baseline_ok,
            "wave_ok": wave_stats.ok,
            "wave_changed": wave_stats.changed,
            "worsened_vs_baseline": worsened,
            "promotion_status": promotion_status,
            "mode_status": mode_status
        },
        "context": {
            "window_tokens": MAX_CONTEXT_TOKENS,
            "min_phrase_tokens": MIN_CONTEXT_TOKENS,
            "token_kinds": [
                "cyrillic_word",
                "ascii_word",
                "technical_ascii",
                "number",
                "punctuation",
                "mixed",
                "other"
            ]
        },
        "zones": [
            {"id": "sensors", "label": "Сенсоры", "layer": "L1"},
            {"id": "candidates", "label": "Кандидаты", "layer": "L2"},
            {"id": "consensus", "label": "Согласование", "layer": "L3"}
        ],
        "cells": wave_cell_json(&ablation),
        "ablation": ablation,
        "layer_impact": layer_impact,
        "candidate_stats": candidate_stats,
        "cell_scoreboard": scoreboard_json(&scoreboard),
        "resonance_memory": resonance_memory_json(&resonance_memory),
        "sources": suite.sources.iter().map(|source| {
            json!({"path": source.path, "cases": source.cases})
        }).collect::<Vec<_>>()
    }))
}

fn scoreboard_json(scoreboard: &journal::CellScoreboard) -> serde_json::Value {
    json!({
        "records": scoreboard.records,
        "updated_at": scoreboard.updated_at,
        "cells": scoreboard.cells.iter().map(|(cell, score)| {
            json!({
                "cell": cell,
                "generated": score.generated,
                "accepted": score.accepted,
                "vetoed": score.vetoed,
                "kept": score.kept,
                "ok": score.ok,
                "bad": score.bad,
                "last_seen": score.last_seen,
                "status": score.status()
            })
        }).collect::<Vec<_>>()
    })
}

fn resonance_memory_json(memory: &resonance_memory::ResonanceMemory) -> serde_json::Value {
    let mut entries = memory.entries.values().collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .trust
            .total_cmp(&left.trust)
            .then_with(|| right.seen.cmp(&left.seen))
            .then_with(|| left.key.cmp(&right.key))
    });
    json!({
        "kind": memory.kind,
        "records": memory.records,
        "updated_at": memory.updated_at,
        "entries": memory.entries.len(),
        "top": entries.into_iter().take(12).map(|entry| {
            json!({
                "key": entry.key,
                "cell": entry.cell,
                "role": entry.role,
                "mode_id": entry.mode_id,
                "seen": entry.seen,
                "reinforced": entry.reinforced,
                "suppressed": entry.suppressed,
                "observed": entry.observed,
                "trust": entry.trust,
                "energy_ema": entry.energy_ema,
                "coherence_ema": entry.coherence_ema,
                "status": entry.status()
            })
        }).collect::<Vec<_>>()
    })
}

pub(crate) fn status_sample_cases(cases: &[EvalCase]) -> Vec<EvalCase> {
    const STATUS_SAMPLE_LIMIT: usize = 16;
    const PER_REASON_LIMIT: usize = 2;
    if cases.len() <= STATUS_SAMPLE_LIMIT {
        return cases.to_vec();
    }
    let mut ordered = cases.to_vec();
    ordered.sort_by(|left, right| {
        left.original
            .chars()
            .count()
            .cmp(&right.original.chars().count())
            .then_with(|| left.reason.cmp(&right.reason))
    });
    let mut sample = Vec::new();
    let mut per_reason: BTreeMap<String, usize> = BTreeMap::new();
    for case in &ordered {
        let count = per_reason.entry(case.reason.clone()).or_default();
        if *count >= PER_REASON_LIMIT {
            continue;
        }
        sample.push(case.clone());
        *count += 1;
        if sample.len() >= STATUS_SAMPLE_LIMIT {
            break;
        }
    }
    if sample.len() < STATUS_SAMPLE_LIMIT {
        for case in &ordered {
            if sample.iter().any(|item| item == case) {
                continue;
            }
            sample.push(case.clone());
            if sample.len() >= STATUS_SAMPLE_LIMIT {
                break;
            }
        }
    }
    sample
}

#[derive(Default)]
struct CandidateSourceStats {
    generated: usize,
    accepted: usize,
    vetoed: usize,
    kept: usize,
}

fn candidate_stats_json(cases: &[EvalCase], options: &WaveOptions) -> Vec<serde_json::Value> {
    let mut stats: BTreeMap<String, CandidateSourceStats> = BTreeMap::new();
    for source in [
        "LayoutWordCell32",
        "ShortTokenCell32",
        "BoundaryCell32",
        "PhraseCell32",
        "GrammarCell32",
        "TechTokenCell32",
        "LearnedMemoryCell32",
        "CommonRuFixCell32",
        "PhraseMemoryCell32",
        "UserMemoryCell32",
        "SemanticWordCell32",
    ] {
        stats.entry(source.to_string()).or_default();
    }
    for case in cases {
        let trace = run_wave_trace_with_options(&case.original, options);
        for candidate in &trace.l2_candidates {
            stats
                .entry(candidate.source.to_string())
                .or_default()
                .generated += 1;
        }
        match trace.decision {
            WaveDecision::Apply { ref text, .. } => {
                if let Some(source) = trace
                    .l2_candidates
                    .iter()
                    .find(|candidate| {
                        preserve_status_space(&case.original, &candidate.text) == *text
                    })
                    .map(|candidate| candidate.source.to_string())
                {
                    stats.entry(source).or_default().accepted += 1;
                }
            }
            WaveDecision::Veto { .. } => {
                if let Some(source) = trace
                    .l2_candidates
                    .first()
                    .map(|candidate| candidate.source)
                {
                    stats.entry(source.to_string()).or_default().vetoed += 1;
                }
            }
            WaveDecision::Keep { .. } => {
                if let Some(source) = trace
                    .l2_candidates
                    .first()
                    .map(|candidate| candidate.source)
                {
                    stats.entry(source.to_string()).or_default().kept += 1;
                }
            }
        }
    }
    stats
        .into_iter()
        .map(|(source, stats)| {
            json!({
                "source": source,
                "generated": stats.generated,
                "accepted": stats.accepted,
                "vetoed": stats.vetoed,
                "kept": stats.kept
            })
        })
        .collect()
}

fn preserve_status_space(original: &str, candidate: &str) -> String {
    if original.ends_with(' ') && !candidate.ends_with(' ') {
        format!("{candidate} ")
    } else {
        candidate.to_string()
    }
}

fn ablation_json(cases: &[EvalCase]) -> Vec<serde_json::Value> {
    let (_base, base_stats) = evaluate_wave(cases);
    super::wave_cells()
        .iter()
        .map(|cell| {
            let options = WaveOptions::with_disabled(&[cell.to_string()]);
            let (_results, stats) = evaluate_wave_with_options(cases, &options);
            let delta = stats.ok as isize - base_stats.ok as isize;
            json!({
                "cell": cell,
                "ok": stats.ok,
                "cases": stats.cases,
                "changed": stats.changed,
                "delta": delta,
                "alive": delta < 0
            })
        })
        .collect()
}

fn layer_impact_json(ablation: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let mut layers: BTreeMap<&'static str, isize> = BTreeMap::new();
    for item in ablation {
        let Some(cell) = item["cell"].as_str() else {
            continue;
        };
        let delta = item["delta"].as_i64().unwrap_or(0).min(0) as isize;
        *layers.entry(super::wave_cell_meta(cell).layer).or_default() += delta;
    }
    ["L1", "L2", "L3"]
        .into_iter()
        .map(|layer| {
            json!({
                "layer": layer,
                "delta": layers.get(layer).copied().unwrap_or(0)
            })
        })
        .collect()
}

fn wave_cell_json(ablation: &[serde_json::Value]) -> Vec<serde_json::Value> {
    super::wave_cells()
        .iter()
        .map(|name| {
            let meta = super::wave_cell_meta(name);
            let delta = ablation
                .iter()
                .find(|item| item["cell"].as_str() == Some(name))
                .and_then(|item| item["delta"].as_i64())
                .unwrap_or(0);
            json!({
                "name": name,
                "label": meta.label,
                "role": meta.role,
                "zone": meta.zone,
                "layer": meta.layer,
                "amp": if delta < 0 { 0.9 } else { 0.3 },
                "phase": meta.phase,
                "delta": delta,
                "alive": delta < 0
            })
        })
        .collect()
}

fn status_cache_path() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache/lay/nanda_wave_status.json")
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
