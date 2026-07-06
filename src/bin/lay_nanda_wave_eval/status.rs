use lay::config::CorrectionSafety;
use lay::eval_cases::EvalCase;
use lay::nanda_wave::context::{MAX_CONTEXT_TOKENS, MIN_CONTEXT_TOKENS};
use lay::nanda_wave::{
    evaluate_wave, evaluate_wave_with_options, journal, llmwave, resonance_memory,
};
use lay::nanda_wave::{l2, run_wave_trace_with_options, WaveDecision, WaveOptions};
use lay::{config, typing_assist, typing_context};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use super::candidate_quality;
use super::real_suite;

const L2_SURFACE_MOTIF_CELL: &str = "L2SurfaceMotifCell32";
const L2_SURFACE_COMPLETION_CELL: &str = "L2SurfaceCompletionCell32";

pub(crate) fn print_status_json(refresh: bool, full: bool) -> io::Result<()> {
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
    let value = build_status_json(full)?;
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
    value["l4_state_map"] = l4_state_map_json();
    value["candidate_gate"] = lay::nanda_wave::candidate_gate::live_candidate_gate_stats_json();
    value["preedit_live"] = preedit_live_json();
    value["candidate_quality"] = candidate_quality::report_json();
    value["live_scoreboard_refreshed_at_unix"] = json!(lay::time::unix_timestamp());
    value["source"] = json!("lay-nanda-wave-eval --status-json (cached gate + live scoreboard)");
}

fn build_status_json(full: bool) -> io::Result<serde_json::Value> {
    let cfg = config::LayConfig::load();
    let llmwave_apply_runtime = cfg.llmwave_shadow && cfg.llmwave_apply;
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
    let llmwave_memory = llmwave::load_default_memory();
    Ok(json!({
        "kind": "nanda_wave_status",
        "generated_at_unix": lay::time::unix_timestamp(),
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
        "llmwave": {
            "enabled_by_default": true,
            "shadow_runtime": cfg.llmwave_shadow,
            "apply_runtime": llmwave_apply_runtime,
            "admission": {
                "live_authority": llmwave_apply_runtime,
                "authority_scope": "L3 candidate feedback; edit-plan safety remains final authority",
                "reason": if llmwave_apply_runtime {
                    "LLMWave is the default L3 feedback authority after promotion gate; output still passes safety"
                } else {
                    "LLMWave memory is loaded but apply_runtime is disabled in config"
                },
                "gate_command": "lay-nanda-wave-eval --llmwave-promotion-gate --train-corpus corpus/project_gutenberg_ru.txt --include-dirty-train",
                "thresholds": {
                    "min_prediction_points": super::LLMWAVE_PROMOTION_MIN_POINTS,
                    "min_records": super::LLMWAVE_PROMOTION_MIN_RECORDS,
                    "min_vocabulary": super::LLMWAVE_PROMOTION_MIN_VOCABULARY,
                    "min_ready_percent": super::LLMWAVE_PROMOTION_MIN_READY_PERCENT,
                    "min_top1_percent": super::LLMWAVE_PROMOTION_MIN_TOP1_PERCENT,
                    "min_top3_percent": super::LLMWAVE_PROMOTION_MIN_TOP3_PERCENT
                },
                "stages": [
                    "books/corpus",
                    "compact wave memory",
                    "dirty eval",
                    "shadow",
                    "promotion",
                    "live"
                ]
            },
            "contract": {
                "model_id": llmwave::contract().model_id,
                "schema_id": llmwave::contract().schema_id,
                "tokenizer_id": llmwave::contract().tokenizer_id,
                "record_bytes": llmwave::contract().record_bytes,
                "hot_path": llmwave::contract().hot_path
            },
            "memory": {
                "path": llmwave::default_memory_path()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "none".to_string()),
                "records": llmwave_memory.len(),
                "vocabulary": llmwave_memory.vocabulary_len(),
                "loaded": !llmwave_memory.is_empty()
            },
            "phrase_probe": llmwave_phrase_probe_json(&llmwave_memory)
        },
        "l2_surface_memory": l2_surface_memory_json(),
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
        "l4_state_map": l4_state_map_json(),
        "candidate_gate": lay::nanda_wave::candidate_gate::live_candidate_gate_stats_json(),
        "preedit_live": preedit_live_json(),
        "candidate_quality": candidate_quality::report_json(),
        "sources": suite.sources.iter().map(|source| {
            json!({"path": source.path, "cases": source.cases})
        }).collect::<Vec<_>>()
    }))
}

fn l2_surface_memory_json() -> serde_json::Value {
    let status = l2::l2_surface_memory_status();
    json!({
        "active_source_target": status.active_source_target,
        "hot_center": {
            "words": status.hot_center_words,
            "records": status.hot_center_records,
            "motifs": status.hot_center_motifs,
            "token_refs": status.hot_center_token_refs,
            "compact_bytes": status.hot_center_bytes
        },
        "broad_prefix_readout": {
            "source_words": status.broad_source_words,
            "prefix_keys": status.broad_prefix_keys,
            "word_refs": status.broad_word_refs,
            "foundation_source_limit": status.foundation_source_limit,
            "foundation_live_scan_limit": status.foundation_live_scan_limit
        },
        "generated_forms": {
            "loaded": status.generated_forms_loaded,
            "words": status.generated_forms_words,
            "authority": "cold lexical source; not promoted into hot L2 center heap"
        },
        "contract": {
            "hot_memory": "L1/L2 centers, token refs and phase-like fingerprints",
            "surface_text": "materialized only for returned candidates",
            "million_target": "source/readout budget, not raw Vec<String> in hot path"
        }
    })
}

#[derive(Debug, Default, Clone, Copy)]
struct PreeditLiveStats {
    sessions: usize,
    accepted: usize,
    abandoned: usize,
}

impl PreeditLiveStats {
    fn acceptance_percent(self) -> Option<f64> {
        (self.sessions > 0).then(|| self.accepted as f64 / self.sessions as f64 * 100.0)
    }
}

fn preedit_live_json() -> serde_json::Value {
    let text = preedit_trace_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .unwrap_or_default();
    let stats = preedit_live_stats_from_text(&text);
    json!({
        "kind": "ime_precognition_live",
        "sessions": stats.sessions,
        "accepted": stats.accepted,
        "abandoned": stats.abandoned,
        "acceptance_percent": stats.acceptance_percent(),
        "candidate_gate": lay::nanda_wave::candidate_gate::live_candidate_gate_stats_json(),
        "correction_gate": lay::correction_core::correction_gate_stats_json(),
    })
}

fn preedit_live_stats_from_text(text: &str) -> PreeditLiveStats {
    let mut stats = PreeditLiveStats::default();
    let mut active = false;
    for line in text.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        match value.get("kind").and_then(Value::as_str) {
            Some("ibus_preedit") => {
                let stage = value.get("stage").and_then(Value::as_str);
                let visible = value
                    .get("visible")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let chars = value.get("chars").and_then(Value::as_u64).unwrap_or(0);
                if stage == Some("show") && visible && chars > 0 {
                    if !active {
                        stats.sessions += 1;
                        active = true;
                    }
                } else if stage == Some("clear") && active {
                    stats.abandoned += 1;
                    active = false;
                }
            }
            Some("ibus_completion_accept") => {
                stats.accepted += 1;
                active = false;
            }
            _ => {}
        }
    }
    stats
}

fn preedit_trace_path() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".local/share/lay/ibus_engine_debug.jsonl"))
}

fn llmwave_phrase_probe_json(memory: &llmwave::LlmWaveMemory) -> Vec<serde_json::Value> {
    ["html", "на улице опять идёт", "я хочу"]
        .iter()
        .map(|prefix| {
            let predictions = memory.predict_phrase(prefix, 3, 3);
            json!({
                "prefix": prefix,
                "predictions": predictions.into_iter().map(|prediction| {
                    json!({
                        "text": prediction.text,
                        "score": prediction.score,
                        "support": prediction.support
                    })
                }).collect::<Vec<_>>()
            })
        })
        .collect()
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
        L2_SURFACE_MOTIF_CELL,
        L2_SURFACE_COMPLETION_CELL,
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

#[derive(Debug, Default, serde::Deserialize)]
struct StatusUsageCountsFile {
    counts: StatusUsageCounts,
}

#[derive(Debug, Default, serde::Deserialize)]
struct StatusUsageCounts {
    #[serde(default)]
    accepted_words: BTreeMap<String, u32>,
    #[serde(default)]
    context_words: BTreeMap<String, u32>,
    #[serde(default)]
    rejected_words: BTreeMap<String, u32>,
    #[serde(default)]
    rejected_context_words: BTreeMap<String, u32>,
    #[serde(default)]
    transition_observed: BTreeMap<String, u32>,
    #[serde(default)]
    transition_attract: BTreeMap<String, u32>,
    #[serde(default)]
    transition_repel: BTreeMap<String, u32>,
}

fn l4_state_map_json() -> Value {
    let (source_bytes, parsed_events, word_states) = lay::nanda_wave::usage_debug_summary();
    let counts = status_usage_counts();
    let signed_word_states = counts
        .accepted_words
        .keys()
        .chain(counts.rejected_words.keys())
        .collect::<BTreeSet<_>>()
        .len();
    let transition_states = counts
        .transition_observed
        .keys()
        .chain(counts.transition_attract.keys())
        .chain(counts.transition_repel.keys())
        .collect::<BTreeSet<_>>()
        .len();
    let transition_signed_states = counts
        .transition_attract
        .keys()
        .chain(counts.transition_repel.keys())
        .collect::<BTreeSet<_>>()
        .len();
    let transition_conflict_states = counts
        .transition_attract
        .keys()
        .filter(|key| counts.transition_repel.contains_key(*key))
        .count();
    let neutral = word_states.saturating_sub(signed_word_states);
    let transition_neutral = transition_states.saturating_sub(transition_signed_states);
    json!({
        "kind": "l4_signed_state_map",
        "source": "word_usage_events.jsonl -> usage_counts v5",
        "source_bytes": source_bytes,
        "parsed_events": parsed_events,
        "word_states": word_states,
        "accepted_word_states": counts.accepted_words.len(),
        "context_word_states": counts.context_words.len(),
        "rejected_word_states": counts.rejected_words.len(),
        "rejected_context_word_states": counts.rejected_context_words.len(),
        "signed_word_states": signed_word_states,
        "transition_states": transition_states,
        "transition_observed_states": counts.transition_observed.len(),
        "transition_attract_states": counts.transition_attract.len(),
        "transition_repel_states": counts.transition_repel.len(),
        "transition_signed_states": transition_signed_states,
        "transition_conflict_states": transition_conflict_states,
        "polarity": {
            "attract": counts.accepted_words.len(),
            "neutral": neutral,
            "repel": counts.rejected_words.len()
        },
        "transition_shadow": {
            "mode": "self_shadow_over_usage_counts",
            "state_hits": transition_signed_states,
            "state_repels": counts.transition_repel.len(),
            "state_false_push": transition_conflict_states,
            "neutral": transition_neutral,
            "ready": transition_signed_states > 0
        },
        "scene_memory": {
            "mode": "whole_context_token_field",
            "source": "phrase_memory.llmw.bin scene-token readout",
            "authority": "weak bias only; candidate authority and edit-plan safety remain final"
        },
        "contract": {
            "positive_trace": "accepted_ime / accepted_fix target",
            "negative_trace": "accepted_fix corrected-away source word",
            "authority": "bias only; safety/edit gates remain final authority"
        }
    })
}

fn status_usage_counts() -> StatusUsageCounts {
    let path = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local/share/lay/nanda_wave/word_usage_counts.json");
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<StatusUsageCountsFile>(&text).ok())
        .map(|file| file.counts)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{l4_state_map_json, preedit_live_stats_from_text};

    #[test]
    fn preedit_live_stats_count_sessions_not_each_show() {
        let text = r#"
{"kind":"ibus_preedit","stage":"show","visible":true,"chars":2,"cursor_pos":0,"text":"ка"}
{"kind":"ibus_preedit","stage":"show","visible":true,"chars":1,"cursor_pos":0,"text":"а"}
{"kind":"ibus_completion_accept","source":"active_composition","suffix_chars":1,"with_space":true}
{"kind":"ibus_preedit","stage":"clear","visible":false,"chars":0,"cursor_pos":0,"text":null}
{"kind":"ibus_preedit","stage":"show","visible":true,"chars":3,"cursor_pos":0,"text":"ить"}
{"kind":"ibus_preedit","stage":"clear","visible":false,"chars":0,"cursor_pos":0,"text":null}
"#;

        let stats = preedit_live_stats_from_text(text);

        assert_eq!(stats.sessions, 2);
        assert_eq!(stats.accepted, 1);
        assert_eq!(stats.abandoned, 1);
        assert_eq!(stats.acceptance_percent(), Some(50.0));
    }

    #[test]
    fn l4_state_map_status_has_contract() {
        let value = l4_state_map_json();

        assert_eq!(value["kind"], "l4_signed_state_map");
        assert_eq!(
            value["contract"]["authority"],
            "bias only; safety/edit gates remain final authority"
        );
        assert_eq!(value["scene_memory"]["mode"], "whole_context_token_field");
        assert_eq!(
            value["scene_memory"]["authority"],
            "weak bias only; candidate authority and edit-plan safety remain final"
        );
    }
}
