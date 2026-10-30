//! Offline verification/measurement only; never delivers text or changes settings.
use lay::config::{
    default_typing_assist_pipeline, publish_runtime_config, CorrectionSafety, LayConfig,
};
use lay::correction_core::{
    resolve_text_correction, CandidateReadoutRoute, CorrectionMode, CorrectionRequest,
};
use lay::layout_fragments::{analyze, model_bytes, Boundary, Selection};
use lay::nanda_wave::WaveOptions;
use serde_json::{json, Value};
use std::{fs, hint::black_box, time::Instant};

fn resolution(text: &str) -> lay::correction_core::CorrectionResolution {
    let pipeline = default_typing_assist_pipeline();
    resolve_text_correction(CorrectionRequest {
        text,
        lexical_authority_frame: None,
        auto_replace: false,
        typing_assist: false,
        auto_switch_layout: true,
        correction_safety: CorrectionSafety::Normal,
        typing_assist_pipeline: &pipeline,
        nanda_autocorrect: false,
        nanda_candidate_route: CandidateReadoutRoute::FullWave,
        nanda_wave_options: WaveOptions::default(),
        mode: CorrectionMode::DeterministicOnly,
    })
}

fn correction(text: &str) -> Option<String> {
    resolution(text)
        .decision
        .map(|decision| decision.replacement)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: fragment_layout_proof HELDOUT.jsonl [CORE_LIMIT_PER_LANGUAGE]")?;
    let core_limit: usize = std::env::args()
        .nth(2)
        .map(|v| v.parse())
        .transpose()?
        .unwrap_or(0);
    let rows: Vec<Value> = fs::read_to_string(path)?
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    let started = Instant::now();
    let first = analyze("ghbdtn", Boundary::Closed).ok_or("embedded model rejected")?;
    let cold_us = started.elapsed().as_micros();
    if first.replacement("ghbdtn").as_deref() != Some("привет") {
        return Err("smoke projection failed".into());
    }
    let mut checked = 0;
    let mut timings = Vec::new();
    for row in &rows {
        let word = row["word"].as_str().ok_or("missing word")?;
        let projected = row["projected"].as_str().ok_or("missing projection")?;
        for input in [word, projected] {
            let started = Instant::now();
            let result = black_box(analyze(black_box(input), Boundary::Closed))
                .ok_or("proof token declined")?;
            timings.push(started.elapsed().as_nanos() as u64);
            let expected = |key: &str| {
                (
                    row[key][0].as_i64().unwrap() as i32,
                    row[key][1].as_u64().unwrap() as u8,
                )
            };
            if (
                result.russian_score.total,
                result.russian_score.status as u8,
            ) != expected("ru_score")
                || (
                    result.english_score.total,
                    result.english_score.status as u8,
                ) != expected("en_score")
            {
                return Err(format!("trainer/runtime score mismatch for {input:?}").into());
            }
            let winner = match result.selection {
                Selection::Russian => json!(0),
                Selection::English => json!(1),
                _ => Value::Null,
            };
            if winner != row["winner"] {
                return Err(format!("selector mismatch for {input:?}").into());
            }
            checked += 1;
        }
    }
    timings.sort_unstable();
    let mut core_rows = Vec::new();
    let mut smoke = Vec::new();
    let mut legacy_results = std::collections::BTreeMap::new();
    for enabled in [false, true] {
        let config = LayConfig {
            fragment_layout_autoflip: enabled,
            ..LayConfig::default()
        };
        publish_runtime_config(&config);
        for input in [
            "ghbdtn",
            "снова Ghbdtn!",
            "руддщ",
            "привезу",
            "hello",
            "git status",
        ] {
            smoke.push(json!({"fragments":enabled, "input":input, "result":correction(input)}));
        }
        for language in ["ru", "en"] {
            let mut counts = [0_usize; 5]; // total, clean preserved, restored, wrong, abstain
            let mut wrong_examples = Vec::new();
            let mut clean_errors = Vec::new();
            let mut lost_legacy_targets = Vec::new();
            let start = Instant::now();
            for row in rows
                .iter()
                .filter(|r| r["language"] == language)
                .take(core_limit)
            {
                let word = row["word"].as_str().unwrap();
                let projected = row["projected"].as_str().unwrap();
                counts[0] += 1;
                let clean = correction(word);
                counts[1] += usize::from(clean.is_none() || clean.as_deref() == Some(word));
                if clean.is_some() && clean.as_deref() != Some(word) && clean_errors.len() < 8 {
                    clean_errors.push(json!({"input":word,"actual":clean,
                        "known_russian":lay::typing_assist::is_known_russian_word_or_form(word)}));
                }
                let restored = correction(projected);
                if !enabled {
                    legacy_results.insert((language, word.to_string()), restored.clone());
                } else if legacy_results
                    .get(&(language, word.to_string()))
                    .and_then(|v| v.as_deref())
                    == Some(word)
                    && restored.as_deref() != Some(word)
                    && lost_legacy_targets.len() < 8
                {
                    lost_legacy_targets
                        .push(json!({"word":word,"input":projected,"new_result":restored}));
                }
                match restored.as_deref() {
                    Some(value) if value == word => counts[2] += 1,
                    Some(value) => {
                        counts[3] += 1;
                        if wrong_examples.len() < 4 {
                            let details = resolution(projected);
                            wrong_examples.push(json!({"input":projected, "expected":word, "actual":value,
                                "candidates":details.candidates.iter().map(|c|json!({"value":c.replacement,"source":c.source_id,"gate":format!("{:?}",c.gate)})).collect::<Vec<_>>() }));
                        }
                    }
                    None => counts[4] += 1,
                }
            }
            core_rows.push(json!({"mode": if enabled {"fragments"} else {"legacy"},
                "language": language, "total": counts[0], "clean_preserved": counts[1],
                "restored": counts[2], "wrong": counts[3], "abstain": counts[4],
                "seconds": start.elapsed().as_secs_f64(), "wrong_examples":wrong_examples,"clean_errors":clean_errors,
                "lost_legacy_targets":lost_legacy_targets}));
        }
    }
    let rss = fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find(|line| line.starts_with("VmHWM:"))
        .unwrap_or("unavailable")
        .to_string();
    println!(
        "{}",
        json!({"status":"PASS", "runtime_trainer_comparisons":checked,
        "model_bytes":model_bytes(), "cold_us":cold_us,
        "warm_median_us":timings[timings.len()/2] as f64/1000.0,
        "warm_p95_us":timings[timings.len()*95/100] as f64/1000.0,
        "rss_high_water":rss, "common_decision_core":core_rows, "smoke":smoke})
    );
    Ok(())
}
