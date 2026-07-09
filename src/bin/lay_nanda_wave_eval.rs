use lay::config::CorrectionSafety;
use lay::eval_cases::EvalCase;
use lay::nanda_wave::{
    evaluate_wave, evaluate_wave_with_options, run_wave_trace_with_options, WaveDecision,
    WaveOptions,
};
use lay::nanda_wave::{journal, llmwave};
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::io;
use std::path::{Path, PathBuf};

#[path = "lay_nanda_wave_eval/candidate_quality.rs"]
mod candidate_quality;
#[path = "lay_nanda_wave_eval/canonical_l1_l2.rs"]
mod canonical_l1_l2;
#[path = "lay_nanda_wave_eval/canonical_l2_recent.rs"]
mod canonical_l2_recent;
#[path = "lay_nanda_wave_eval/dirty_log_collect.rs"]
mod dirty_log_collect;
#[path = "lay_nanda_wave_eval/ime_hit_rate.rs"]
mod ime_hit_rate;
#[path = "lay_nanda_wave_eval/learning_loop.rs"]
mod learning_loop;
#[path = "lay_nanda_wave_eval/real_suite.rs"]
mod real_suite;
#[path = "lay_nanda_wave_eval/status.rs"]
mod status;

const DEFAULT_LLMWAVE_SEED: &str = "data/nanda_llmwave_seed_phrases.txt";
const DEFAULT_LLMWAVE_LIVE_MIN_COUNT: usize = 1;
const DEFAULT_LLMWAVE_PROMOTION_MAX_LINES: usize = 500;
const LLMWAVE_PROMOTION_MIN_POINTS: usize = 100;
const LLMWAVE_PROMOTION_MIN_RECORDS: usize = 100;
const LLMWAVE_PROMOTION_MIN_VOCABULARY: usize = 50;
const LLMWAVE_PROMOTION_MIN_READY_PERCENT: f32 = 90.0;
const LLMWAVE_PROMOTION_MIN_TOP1_PERCENT: f32 = 50.0;
const LLMWAVE_PROMOTION_MIN_TOP3_PERCENT: f32 = 85.0;
const L2_SURFACE_MOTIF_CELL: &str = "L2SurfaceMotifCell32";
const L2_SURFACE_COMPLETION_CELL: &str = "L2SurfaceCompletionCell32";
const L2_WORD_ATTRACTOR_CELL: &str = "L2WordAttractorCell32";

fn main() -> io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if let Some(path) = arg_value(&args, "--learned") {
        env::set_var("LAY_NANDA_WAVE_MEMORY", path);
    }
    if let Some(path) = arg_value(&args, "--llmwave-memory") {
        env::set_var("LAY_LLMWAVE_MEMORY", path);
    }
    if let Some(path) = arg_value(&args, "--l2-phase-memory") {
        env::set_var("LAY_NANDA_L2_PHASE_MEMORY", path);
    }
    let disabled = arg_values(&args, "--disable-cell");
    let options = WaveOptions::with_disabled(&disabled)
        .with_llmwave_shadow(args.iter().any(|arg| arg == "--llmwave-shadow"))
        .with_llmwave_apply(args.iter().any(|arg| arg == "--llmwave-apply"));
    if let Some(path) = arg_value(&args, "--llmwave-pack-cases") {
        let Some(out) = arg_value(&args, "--out") else {
            eprintln!("--llmwave-pack-cases requires --out PATH");
            return Ok(());
        };
        pack_llmwave_cases(path, out)?;
        return Ok(());
    }
    if let Some(path) = arg_value(&args, "--llmwave-pack-text") {
        let Some(out) = arg_value(&args, "--out") else {
            eprintln!("--llmwave-pack-text requires --out PATH");
            return Ok(());
        };
        pack_llmwave_text(path, out)?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--llmwave-pack-live") {
        let out = arg_value(&args, "--out")
            .map(PathBuf::from)
            .or_else(llmwave::default_memory_path)
            .expect("default llmwave memory path");
        let seed = arg_value(&args, "--seed").unwrap_or(DEFAULT_LLMWAVE_SEED);
        let min_count = live_min_count(&args);
        pack_llmwave_live(seed, &out, min_count)?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--llmwave-learn-live") {
        let out = arg_value(&args, "--out")
            .map(PathBuf::from)
            .or_else(llmwave::default_memory_path)
            .expect("default llmwave memory path");
        let seed = arg_value(&args, "--seed").unwrap_or(DEFAULT_LLMWAVE_SEED);
        let limit = arg_value(&args, "--limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(12);
        let min_count = live_min_count(&args);
        pack_llmwave_live(seed, &out, min_count)?;
        print_llmwave_learning_report(seed, limit, min_count)?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--llmwave-learning-report") {
        let seed = arg_value(&args, "--seed").unwrap_or(DEFAULT_LLMWAVE_SEED);
        let limit = arg_value(&args, "--limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(12);
        print_llmwave_learning_report(seed, limit, live_min_count(&args))?;
        return Ok(());
    }
    if let Some(path) = arg_value(&args, "--llmwave-ingest-clean-corpus") {
        let max_records = arg_value(&args, "--max-records")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(50_000);
        let report = llmwave::ingest_clean_corpus_path(&PathBuf::from(path), max_records)?;
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }
    if let Some(path) = arg_value(&args, "--llmwave-ingest-pack-clean-corpus") {
        let out = arg_value(&args, "--out")
            .map(PathBuf::from)
            .or_else(llmwave::default_memory_path)
            .expect("default llmwave memory path");
        let seed = arg_value(&args, "--seed").unwrap_or(DEFAULT_LLMWAVE_SEED);
        let max_records = arg_value(&args, "--max-records")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(50_000);
        ingest_pack_clean_corpus(
            &PathBuf::from(path),
            seed,
            &out,
            max_records,
            live_min_count(&args),
        )?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--memory-learned-report") {
        println!(
            "{}",
            serde_json::to_string_pretty(&lay::nanda_wave::usage_memory_learned_report_json())?
        );
        return Ok(());
    }
    if let Some(path) = arg_value(&args, "--llmwave-corpus-report") {
        let test = arg_value(&args, "--test-corpus");
        let limit = arg_value(&args, "--limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(12);
        let max_lines = arg_value(&args, "--max-lines")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(20_000);
        print_llmwave_corpus_report(
            &PathBuf::from(path),
            test.map(PathBuf::from).as_deref(),
            limit,
            max_lines,
        )?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--llmwave-dirty-report") {
        let train = arg_value(&args, "--train-corpus").map(PathBuf::from);
        let include_dirty_train = args.iter().any(|arg| arg == "--include-dirty-train");
        let limit = arg_value(&args, "--limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(12);
        let max_lines = arg_value(&args, "--max-lines")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(20_000);
        print_llmwave_dirty_report(train.as_deref(), include_dirty_train, limit, max_lines)?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--llmwave-promotion-gate") {
        let train = arg_value(&args, "--train-corpus").map(PathBuf::from);
        let include_dirty_train = args.iter().any(|arg| arg == "--include-dirty-train");
        let max_lines = arg_value(&args, "--max-lines")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(DEFAULT_LLMWAVE_PROMOTION_MAX_LINES);
        print_llmwave_promotion_gate(train.as_deref(), include_dirty_train, max_lines)?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--learning-shadow-report") {
        let path = correction_learning_log_path(&args)?;
        let limit = arg_value(&args, "--limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(12);
        print_correction_learning_report(&path, live_min_count(&args), limit)?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--learning-pack-corrections") {
        let path = correction_learning_log_path(&args)?;
        let Some(out) = arg_value(&args, "--out") else {
            eprintln!("--learning-pack-corrections requires --out PATH");
            return Ok(());
        };
        pack_correction_learning(&path, &PathBuf::from(out), live_min_count(&args))?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--canonical-l1-l2-report") {
        canonical_l1_l2::print_report(&args)?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--canonical-l2-candidates") {
        canonical_l1_l2::print_candidates(&args)?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--canonical-l2-recent") {
        canonical_l2_recent::print_recent(&args)?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--l2-phase-coverage-recent") {
        canonical_l2_recent::print_phase_coverage(&args)?;
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| arg == "--l2-candidate-phase-shadow-recent")
    {
        canonical_l2_recent::print_candidate_phase_shadow(&args)?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--canonical-l2-harvest") {
        canonical_l2_recent::harvest_recent(&args)?;
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| arg == "--canonical-l2-harvest-summary")
    {
        canonical_l2_recent::print_harvest_summary(&args)?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--canonical-l2-replay") {
        canonical_l2_recent::replay_harvest(&args)?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--canonical-l2-morph-replay") {
        canonical_l2_recent::replay_harvest_with_morphology(&args)?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--status-json") {
        status::print_status_json(
            args.iter().any(|arg| arg == "--refresh-status-json"),
            args.iter().any(|arg| arg == "--full-status-json"),
        )?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--candidate-quality-report") {
        candidate_quality::print_json()?;
        return Ok(());
    }
    if let Some(text) = arg_value(&args, "--l2-form-attractor-candidates") {
        let limit = arg_value(&args, "--limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(12);
        print_l2_form_attractor_candidates(text, &options, limit)?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--ime-hit-rate-report") {
        ime_hit_rate::print_json()?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--dirty-log-eval") {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "kind": "dirty_log_eval",
                "candidate_quality": candidate_quality::report_json(),
                "ime_hit_rate": ime_hit_rate::report_json(),
                "read_as": "live dirty-log scoreboard; diagnostic only"
            }))?
        );
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--dirty-log-collect") {
        dirty_log_collect::print_json(&args)?;
        return Ok(());
    }
    if let Some(limit) = arg_value(&args, "--recent-traces") {
        print_recent_traces(parse_limit(limit, 10));
        return Ok(());
    }
    if let Some(text) = arg_value(&args, "--trace") {
        print_trace(
            text,
            &options,
            args.iter().any(|arg| arg == "--record-trace"),
        );
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--real-suite") {
        print_real_suite(
            &options,
            args.iter()
                .any(|arg| arg == "--ablation" || arg == "--ensemble-sweep"),
            args.iter().any(|arg| arg == "--show-failures"),
            args.iter().any(|arg| arg == "--show-worsened"),
            args.iter().any(|arg| arg == "--record-trace"),
        )?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--pattern-wave-ablation") {
        let suite = real_suite::load()?;
        println!("pattern_wave_ablation_suite: cases={}", suite.cases.len());
        print_cell_ablation(&suite.cases, "PatternWaveCell32", "pattern_wave");
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| arg == "--structural-relation-ablation")
    {
        let suite = real_suite::load()?;
        println!(
            "structural_relation_ablation_suite: cases={}",
            suite.cases.len()
        );
        print_cell_ablation(
            &suite.cases,
            "StructuralRelationCell32",
            "structural_relation",
        );
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--quick-ablation") {
        let suite = real_suite::load()?;
        let cases = status::status_sample_cases(&suite.cases);
        let rows = ablation_rows(&cases);
        println!(
            "quick_ablation: cases={} full_cases={}",
            cases.len(),
            suite.cases.len()
        );
        print_ablation_rows(&rows);
        print_layer_impact(&rows);
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--surface-l2-ablation") {
        let suite = real_suite::load()?;
        print_surface_l2_ablation(&status::status_sample_cases(&suite.cases));
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| arg == "--ensemble-contribution-report")
    {
        let suite = real_suite::load()?;
        let cases = if args.iter().any(|arg| arg == "--full-suite") {
            suite.cases.clone()
        } else {
            status::status_sample_cases(&suite.cases)
        };
        print_ensemble_contribution_report(&cases, suite.cases.len());
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--l2-candidate-flow-report") {
        let suite = real_suite::load()?;
        let cases = if args.iter().any(|arg| arg == "--full-suite") {
            suite.cases.clone()
        } else {
            status::status_sample_cases(&suite.cases)
        };
        let show_examples = args.iter().any(|arg| arg == "--show-examples");
        print_l2_candidate_flow_report(&cases, suite.cases.len(), &options, show_examples);
        return Ok(());
    }
    let paths = arg_values(&args, "--cases");
    if paths.is_empty() {
        eprintln!(
            "usage: lay-nanda-wave-eval --trace TEXT | --recent-traces N | --real-suite [--show-failures] [--show-worsened] | --quick-ablation | --surface-l2-ablation | --ensemble-contribution-report [--full-suite] | --l2-candidate-flow-report [--full-suite] [--show-examples] | --canonical-l1-l2-report [--probe WORD] | --canonical-l2-candidates TEXT [--limit N] | --l2-form-attractor-candidates TEXT [--limit N] | --canonical-l2-recent [--limit N] [--candidate-limit N] | --l2-phase-coverage-recent [--limit N] [--candidate-limit N] [--max-examples N] | --l2-candidate-phase-shadow-recent [--l2-phase-memory PATH] [--limit N] [--max-examples N] | --canonical-l2-harvest [--limit N] [--candidate-limit N] [--out PATH] | --canonical-l2-harvest-summary [--harvest PATH] | --canonical-l2-replay [--harvest PATH] [--min-score N] [--limit N] | --canonical-l2-morph-replay [--harvest PATH] [--min-score N] [--limit N] | --llmwave-pack-cases PATH --out PATH | --llmwave-pack-live [--out PATH] | --llmwave-learn-live [--out PATH] | --llmwave-learning-report | --llmwave-ingest-clean-corpus PATH [--max-records N] | --llmwave-ingest-pack-clean-corpus PATH [--out PATH] [--max-records N] | --memory-learned-report | --llmwave-corpus-report PATH [--test-corpus PATH] [--max-lines N] | --llmwave-dirty-report [--train-corpus PATH] [--include-dirty-train] [--max-lines N] | --llmwave-promotion-gate [--train-corpus PATH] [--include-dirty-train] [--max-lines N] | --learning-shadow-report [--learning-log PATH] | --learning-pack-corrections --out PATH [--learning-log PATH] | --candidate-quality-report | --ime-hit-rate-report | --dirty-log-eval | --dirty-log-collect [--out PATH] [--limit N] [--recent-actions PATH] [--learning-log PATH] | --cases PATH"
        );
        return Ok(());
    }
    let mut cases = Vec::new();
    for path in paths {
        cases.extend(lay::eval_cases::read_cases(&PathBuf::from(path))?);
    }
    print_cases("cases", &cases, &options);
    Ok(())
}

fn pack_llmwave_cases(path: &str, out: &str) -> io::Result<()> {
    let cases = lay::eval_cases::read_cases(&PathBuf::from(path))?;
    let text = cases
        .iter()
        .map(|case| case.expected.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let memory = llmwave::LlmWaveMemory::from_text(&text);
    llmwave::write_memory_packet(&PathBuf::from(out), &memory)?;
    println!(
        "llmwave_pack_cases: input={} output={} records={} record_bytes={}",
        path,
        out,
        memory.len(),
        llmwave::LLMWAVE_RECORD_BYTES
    );
    Ok(())
}

fn pack_llmwave_text(path: &str, out: &str) -> io::Result<()> {
    let text = std::fs::read_to_string(path)?;
    let memory = llmwave::LlmWaveMemory::from_text(&text);
    llmwave::write_memory_packet(&PathBuf::from(out), &memory)?;
    println!(
        "llmwave_pack_text: input={} output={} records={} record_bytes={}",
        path,
        out,
        memory.len(),
        llmwave::LLMWAVE_RECORD_BYTES
    );
    Ok(())
}

fn pack_llmwave_live(seed: &str, out: &std::path::Path, min_count: usize) -> io::Result<()> {
    let (memory, live_path, live_records) = build_live_llmwave_memory(seed, min_count);
    llmwave::write_memory_packet(out, &memory)?;
    println!(
        "llmwave_pack_live: seed={} live={} live_records={} min_count={} output={} records={} vocabulary={} record_bytes={}",
        seed,
        live_path,
        live_records,
        min_count,
        out.display(),
        memory.len(),
        memory.vocabulary_len(),
        llmwave::LLMWAVE_RECORD_BYTES
    );
    Ok(())
}

fn ingest_pack_clean_corpus(
    path: &Path,
    seed: &str,
    out: &Path,
    max_records: usize,
    min_count: usize,
) -> io::Result<()> {
    let ingest = llmwave::ingest_clean_corpus_path(path, max_records)?;
    let (memory, live_path, live_records) = build_live_llmwave_memory(seed, min_count);
    llmwave::write_memory_packet(out, &memory)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "kind": "llmwave_clean_corpus_pipeline",
            "ingest": ingest,
            "pack": {
                "seed": seed,
                "live": live_path,
                "live_records": live_records,
                "min_count": min_count,
                "output": out.display().to_string(),
                "records": memory.len(),
                "vocabulary": memory.vocabulary_len(),
                "record_bytes": llmwave::LLMWAVE_RECORD_BYTES
            },
            "authority": "clean corpus ingestion + pack only; runtime apply still gated by config"
        }))?
    );
    Ok(())
}

fn build_live_llmwave_memory(
    seed: &str,
    min_count: usize,
) -> (llmwave::LlmWaveMemory, String, usize) {
    let mut parts = Vec::new();
    if let Ok(text) = std::fs::read_to_string(seed) {
        parts.push(text);
    }
    let live_path = llmwave::default_phrase_experience_path();
    let live_records = live_path
        .as_ref()
        .and_then(|path| llmwave::load_phrase_experience(path).ok())
        .unwrap_or_default();
    let live_text = reinforced_live_text(&live_records, min_count);
    if !live_text.trim().is_empty() {
        parts.push(live_text);
    }
    let text = parts.join("\n");
    let memory = llmwave::LlmWaveMemory::from_text(&text);
    (
        memory,
        live_path
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "none".to_string()),
        live_records.len(),
    )
}

fn print_llmwave_learning_report(seed: &str, limit: usize, min_count: usize) -> io::Result<()> {
    let seed_text = std::fs::read_to_string(seed).unwrap_or_default();
    let seed_memory = llmwave::LlmWaveMemory::from_text(&seed_text);
    let live_path = llmwave::default_phrase_experience_path();
    let live_quality = live_path
        .as_ref()
        .and_then(|path| live_phrase_quality_counts(path).ok());
    let live_records = live_path
        .as_ref()
        .and_then(|path| llmwave::load_phrase_experience(path).ok())
        .unwrap_or_default();
    let mut live_counts = BTreeMap::<String, usize>::new();
    for record in &live_records {
        *live_counts.entry(record.text.clone()).or_default() += 1;
    }
    let reinforced = reinforced_live_counts(&live_records, min_count);
    let live_text = reinforced.keys().cloned().collect::<Vec<_>>().join("\n");
    let combined_memory = llmwave::LlmWaveMemory::from_text(&format!("{seed_text}\n{live_text}"));
    println!("llmwave_learning_report:");
    println!("  seed: {seed}");
    println!(
        "  live: {}",
        live_path
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "none".to_string())
    );
    println!(
        "  seed_memory: records={} vocabulary={}",
        seed_memory.len(),
        seed_memory.vocabulary_len()
    );
    println!(
        "  combined_memory: records={} vocabulary={}",
        combined_memory.len(),
        combined_memory.vocabulary_len()
    );
    println!(
        "  live_experience: records={} unique={} reinforced={} min_count={}",
        live_records.len(),
        live_counts.len(),
        reinforced.len(),
        min_count
    );
    if let Some((accepted, rejected)) = live_quality {
        println!(
            "  live_quality: accepted={} rejected={}",
            accepted,
            rejected.values().sum::<usize>()
        );
        if !rejected.is_empty() {
            println!("  rejected:");
            for (reason, count) in rejected {
                println!("    {reason}={count}");
            }
        }
    }
    let report_phrases = reinforced_phrase_report_sample(&reinforced, limit);
    print_live_phrase_counts(&reinforced, limit);
    print_learning_deltas(&seed_memory, &combined_memory, report_phrases.iter(), limit);
    print_prediction_deltas(&seed_memory, &combined_memory, report_phrases.iter(), limit);
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
struct LlmWaveCorpusReport {
    train_input: String,
    test_input: String,
    train_lines: usize,
    test_lines: usize,
    records: usize,
    vocabulary: usize,
    prediction_points: usize,
    ready_points: usize,
    top1_hits: usize,
    top3_hits: usize,
    misses: usize,
    avg_expected_score: f32,
    avg_top_score: f32,
    examples: Vec<LlmWaveCorpusExample>,
}

#[derive(Debug, Clone, PartialEq)]
struct LlmWaveCorpusExample {
    prefix: String,
    expected: String,
    top: String,
    top_score: f32,
    expected_score: f32,
    top3: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LlmWavePromotionGate {
    verdict: LlmWavePromotionVerdict,
    reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LlmWavePromotionVerdict {
    PassShadow,
    Watch,
}

impl LlmWavePromotionVerdict {
    fn as_str(self) -> &'static str {
        match self {
            Self::PassShadow => "PASS-shadow",
            Self::Watch => "WATCH",
        }
    }
}

fn print_llmwave_corpus_report(
    train_path: &Path,
    test_path: Option<&Path>,
    limit: usize,
    max_lines: usize,
) -> io::Result<()> {
    let train_text = std::fs::read_to_string(train_path)?;
    let (test_input, test_text) = match test_path {
        Some(path) => (path.display().to_string(), std::fs::read_to_string(path)?),
        None => (train_path.display().to_string(), train_text.clone()),
    };
    let report = llmwave_corpus_report_from_text(
        train_path.display().to_string(),
        train_text,
        test_input,
        test_text,
        limit,
        max_lines,
    );
    print_llmwave_corpus_report_rows("llmwave_corpus_report", &report);
    Ok(())
}

fn print_llmwave_dirty_report(
    train_path: Option<&Path>,
    include_dirty_train: bool,
    limit: usize,
    max_lines: usize,
) -> io::Result<()> {
    let dirty = dirty_log_corpus_text(max_lines)?;
    let (mut train_input, mut train_text) = match train_path {
        Some(path) => (path.display().to_string(), std::fs::read_to_string(path)?),
        None => ("dirty_logs".to_string(), dirty.clone()),
    };
    if include_dirty_train && train_path.is_some() {
        train_input = format!("{train_input}+dirty_logs");
        train_text.push('\n');
        train_text.push_str(&dirty);
    }
    let report = llmwave_corpus_report_from_text(
        train_input,
        train_text,
        "dirty_logs".to_string(),
        dirty,
        limit,
        max_lines,
    );
    print_llmwave_corpus_report_rows("llmwave_dirty_report", &report);
    Ok(())
}

fn print_llmwave_promotion_gate(
    train_path: Option<&Path>,
    include_dirty_train: bool,
    max_lines: usize,
) -> io::Result<()> {
    let dirty = dirty_log_corpus_text(max_lines)?;
    let (mut train_input, mut train_text) = match train_path {
        Some(path) => (path.display().to_string(), std::fs::read_to_string(path)?),
        None => ("dirty_logs".to_string(), dirty.clone()),
    };
    if include_dirty_train && train_path.is_some() {
        train_input = format!("{train_input}+dirty_logs");
        train_text.push('\n');
        train_text.push_str(&dirty);
    }
    let report = llmwave_corpus_report_from_text(
        train_input,
        train_text,
        "dirty_logs".to_string(),
        dirty,
        0,
        max_lines,
    );
    let gate = llmwave_promotion_gate(&report);
    println!("llmwave_promotion_gate:");
    print_llmwave_promotion_gate_rows(&report, gate, "  ");
    Ok(())
}

fn llmwave_corpus_report_from_text(
    train_input: String,
    train_text: String,
    test_input: String,
    test_text: String,
    limit: usize,
    max_lines: usize,
) -> LlmWaveCorpusReport {
    let memory = llmwave::LlmWaveMemory::from_text(&train_text);
    let train_lines = non_empty_line_count(&train_text);
    let test_lines = non_empty_line_count(&test_text).min(max_lines);
    let mut prediction_points = 0usize;
    let mut ready_points = 0usize;
    let mut top1_hits = 0usize;
    let mut top3_hits = 0usize;
    let mut expected_score_sum = 0.0f32;
    let mut top_score_sum = 0.0f32;
    let mut examples = Vec::new();

    for line in test_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(max_lines)
    {
        let tokens = llmwave::tokenize(line);
        if tokens.len() < 3 {
            continue;
        }
        for idx in 1..tokens.len() {
            let prefix_tokens = &tokens[..idx];
            let expected = &tokens[idx];
            prediction_points += 1;
            let prefix = prefix_tokens.join(" ");
            let predictions = memory.predict_phrase(&prefix, 1, 3);
            if predictions.is_empty() {
                maybe_push_corpus_example(
                    &mut examples,
                    limit,
                    prefix,
                    expected.clone(),
                    "none".to_string(),
                    0.0,
                    0.0,
                    Vec::new(),
                );
                continue;
            }
            ready_points += 1;
            let top_next = prediction_next_token(&predictions[0])
                .unwrap_or_default()
                .to_string();
            let top_score = predictions[0].score;
            let expected_score = memory
                .score_next_token_report(prefix_tokens, expected)
                .map(|score| score.score)
                .unwrap_or(0.0);
            let top3 = predictions
                .iter()
                .filter_map(prediction_next_token)
                .map(str::to_string)
                .collect::<Vec<_>>();
            let hit1 = top_next == *expected;
            let hit3 = top3.iter().any(|item| item == expected);
            top1_hits += usize::from(hit1);
            top3_hits += usize::from(hit3);
            expected_score_sum += expected_score;
            top_score_sum += top_score;
            if !hit1 {
                maybe_push_corpus_example(
                    &mut examples,
                    limit,
                    prefix,
                    expected.clone(),
                    top_next,
                    top_score,
                    expected_score,
                    top3,
                );
            }
        }
    }

    let misses = prediction_points.saturating_sub(top1_hits);
    let denominator = prediction_points.max(1) as f32;
    LlmWaveCorpusReport {
        train_input,
        test_input,
        train_lines,
        test_lines,
        records: memory.len(),
        vocabulary: memory.vocabulary_len(),
        prediction_points,
        ready_points,
        top1_hits,
        top3_hits,
        misses,
        avg_expected_score: expected_score_sum / denominator,
        avg_top_score: top_score_sum / denominator,
        examples,
    }
}

fn print_llmwave_corpus_report_rows(label: &str, report: &LlmWaveCorpusReport) {
    let gate = llmwave_promotion_gate(report);
    println!("{label}:");
    println!("  train_input: {}", report.train_input);
    println!("  test_input: {}", report.test_input);
    println!(
        "  train_lines={} test_lines={} records={} vocabulary={}",
        report.train_lines, report.test_lines, report.records, report.vocabulary
    );
    println!(
        "  prediction_points={} ready_points={} top1_hits={} top3_hits={} misses={}",
        report.prediction_points,
        report.ready_points,
        report.top1_hits,
        report.top3_hits,
        report.misses
    );
    println!(
        "  top1={:.2}% top3={:.2}% ready={:.2}% avg_expected_score={:.3} avg_top_score={:.3}",
        corpus_percent(report.top1_hits, report.prediction_points),
        corpus_percent(report.top3_hits, report.prediction_points),
        corpus_percent(report.ready_points, report.prediction_points),
        report.avg_expected_score,
        report.avg_top_score
    );
    print_llmwave_promotion_gate_rows(report, gate, "  ");
    println!("  misses_examples:");
    if report.examples.is_empty() {
        println!("    none");
    }
    for example in &report.examples {
        println!(
            "    prefix={:?} expected={:?} top={:?} top_score={:.3} expected_score={:.3} top3={:?}",
            example.prefix,
            example.expected,
            example.top,
            example.top_score,
            example.expected_score,
            example.top3
        );
    }
}

fn llmwave_promotion_gate(report: &LlmWaveCorpusReport) -> LlmWavePromotionGate {
    if report.prediction_points < LLMWAVE_PROMOTION_MIN_POINTS {
        return LlmWavePromotionGate {
            verdict: LlmWavePromotionVerdict::Watch,
            reason: "not_enough_prediction_points",
        };
    }
    if report.records < LLMWAVE_PROMOTION_MIN_RECORDS {
        return LlmWavePromotionGate {
            verdict: LlmWavePromotionVerdict::Watch,
            reason: "not_enough_memory_records",
        };
    }
    if report.vocabulary < LLMWAVE_PROMOTION_MIN_VOCABULARY {
        return LlmWavePromotionGate {
            verdict: LlmWavePromotionVerdict::Watch,
            reason: "not_enough_vocabulary",
        };
    }
    if corpus_percent(report.ready_points, report.prediction_points)
        < LLMWAVE_PROMOTION_MIN_READY_PERCENT
    {
        return LlmWavePromotionGate {
            verdict: LlmWavePromotionVerdict::Watch,
            reason: "ready_coverage_too_low",
        };
    }
    if corpus_percent(report.top1_hits, report.prediction_points)
        < LLMWAVE_PROMOTION_MIN_TOP1_PERCENT
    {
        return LlmWavePromotionGate {
            verdict: LlmWavePromotionVerdict::Watch,
            reason: "top1_too_low",
        };
    }
    if corpus_percent(report.top3_hits, report.prediction_points)
        < LLMWAVE_PROMOTION_MIN_TOP3_PERCENT
    {
        return LlmWavePromotionGate {
            verdict: LlmWavePromotionVerdict::Watch,
            reason: "top3_too_low",
        };
    }
    LlmWavePromotionGate {
        verdict: LlmWavePromotionVerdict::PassShadow,
        reason: "dirty_eval_passed_shadow_thresholds",
    }
}

fn print_llmwave_promotion_gate_rows(
    report: &LlmWaveCorpusReport,
    gate: LlmWavePromotionGate,
    indent: &str,
) {
    println!("{indent}promotion_gate:");
    println!("{indent}  verdict: {}", gate.verdict.as_str());
    println!("{indent}  reason: {}", gate.reason);
    let cfg = lay::config::LayConfig::load();
    let live_authority = gate.verdict == LlmWavePromotionVerdict::PassShadow
        && cfg.llmwave_shadow
        && cfg.llmwave_apply;
    println!("{indent}  live_authority: {live_authority}");
    println!(
        "{indent}  live_authority_reason: {}",
        if live_authority {
            "promotion gate passed and llmwave apply is enabled; edit-plan safety remains final"
        } else {
            "requires PASS-shadow plus llmwave_shadow=true and llmwave_apply=true"
        }
    );
    println!("{indent}  thresholds:");
    println!(
        "{indent}    min_prediction_points={} min_records={} min_vocabulary={}",
        LLMWAVE_PROMOTION_MIN_POINTS,
        LLMWAVE_PROMOTION_MIN_RECORDS,
        LLMWAVE_PROMOTION_MIN_VOCABULARY
    );
    println!(
        "{indent}    min_ready={:.2}% min_top1={:.2}% min_top3={:.2}%",
        LLMWAVE_PROMOTION_MIN_READY_PERCENT,
        LLMWAVE_PROMOTION_MIN_TOP1_PERCENT,
        LLMWAVE_PROMOTION_MIN_TOP3_PERCENT
    );
    println!(
        "{indent}  actual: prediction_points={} records={} vocabulary={} ready={:.2}% top1={:.2}% top3={:.2}%",
        report.prediction_points,
        report.records,
        report.vocabulary,
        corpus_percent(report.ready_points, report.prediction_points),
        corpus_percent(report.top1_hits, report.prediction_points),
        corpus_percent(report.top3_hits, report.prediction_points)
    );
}

fn dirty_log_corpus_text(max_lines: usize) -> io::Result<String> {
    let mut lines = Vec::new();
    if let Some(path) = default_recent_actions_path() {
        collect_json_string_fields(
            &path,
            &["to", "expected", "text", "replacement", "inserted_text"],
            &mut lines,
            max_lines,
        )?;
    }
    if lines.len() < max_lines {
        if let Some(path) = learning_loop::default_correction_log_path() {
            collect_json_string_fields(&path, &["to", "lay_to"], &mut lines, max_lines)?;
        }
    }
    if lines.len() < max_lines {
        if let Some(path) = llmwave::default_phrase_experience_path() {
            if let Ok(text) = llmwave::load_phrase_experience_text(&path) {
                for line in text.lines() {
                    if lines.len() >= max_lines {
                        break;
                    }
                    lines.push(line.to_string());
                }
            }
        }
    }
    Ok(lines
        .into_iter()
        .filter(|line| llmwave::tokenize(line).len() >= 2)
        .take(max_lines)
        .collect::<Vec<_>>()
        .join("\n"))
}

fn collect_json_string_fields(
    path: &Path,
    fields: &[&str],
    out: &mut Vec<String>,
    max_lines: usize,
) -> io::Result<()> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(());
    };
    collect_json_string_fields_from_str(&text, fields, out, max_lines);
    Ok(())
}

fn collect_json_string_fields_from_str(
    text: &str,
    fields: &[&str],
    out: &mut Vec<String>,
    max_lines: usize,
) {
    let log_lines = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    for line in log_lines.into_iter().rev() {
        if out.len() >= max_lines {
            break;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        collect_json_value_string_fields(&value, fields, out, max_lines);
    }
}

fn collect_json_value_string_fields(
    value: &Value,
    fields: &[&str],
    out: &mut Vec<String>,
    max_lines: usize,
) {
    if out.len() >= max_lines {
        return;
    }
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                if out.len() >= max_lines {
                    return;
                }
                if fields.contains(&key.as_str()) {
                    if let Some(text) = item.as_str() {
                        if !text.trim().is_empty() {
                            out.push(text.to_string());
                        }
                    }
                }
                collect_json_value_string_fields(item, fields, out, max_lines);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_json_value_string_fields(item, fields, out, max_lines);
                if out.len() >= max_lines {
                    return;
                }
            }
        }
        _ => {}
    }
}

fn default_recent_actions_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".local/share/lay/recent_actions.jsonl"))
}

fn prediction_next_token(prediction: &llmwave::LlmWavePhrasePrediction) -> Option<&str> {
    prediction.tokens.last().map(String::as_str)
}

#[allow(clippy::too_many_arguments)]
fn maybe_push_corpus_example(
    examples: &mut Vec<LlmWaveCorpusExample>,
    limit: usize,
    prefix: String,
    expected: String,
    top: String,
    top_score: f32,
    expected_score: f32,
    top3: Vec<String>,
) {
    if examples.len() >= limit {
        return;
    }
    examples.push(LlmWaveCorpusExample {
        prefix,
        expected,
        top,
        top_score,
        expected_score,
        top3,
    });
}

fn non_empty_line_count(text: &str) -> usize {
    text.lines().filter(|line| !line.trim().is_empty()).count()
}

fn corpus_percent(part: usize, total: usize) -> f32 {
    if total == 0 {
        return 0.0;
    }
    part as f32 * 100.0 / total as f32
}

fn live_min_count(args: &[String]) -> usize {
    arg_value(args, "--min-live-count")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(DEFAULT_LLMWAVE_LIVE_MIN_COUNT)
        .max(1)
}

fn correction_learning_log_path(args: &[String]) -> io::Result<PathBuf> {
    if let Some(path) = arg_value(args, "--learning-log") {
        return Ok(PathBuf::from(path));
    }
    learning_loop::default_correction_log_path().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME is not set and --learning-log was not provided",
        )
    })
}

fn print_correction_learning_report(
    path: &std::path::Path,
    min_count: usize,
    limit: usize,
) -> io::Result<()> {
    let report = learning_loop::learning_shadow_report(path, min_count)?;
    println!("nanda_correction_learning_shadow:");
    println!("  input: {}", report.input.display());
    println!(
        "  raw_lines={} invalid_lines={} experiences={}",
        report.raw_lines, report.invalid_lines, report.experiences
    );
    println!(
        "  accepted_signals={} rejected_signals={} min_count={}",
        report.accepted_signals, report.rejected_signals, report.min_count
    );
    println!(
        "  candidate_entries={} ready_entries={}",
        report.candidate_entries, report.ready_entries
    );
    if !report.by_signal.is_empty() {
        println!("  by_signal:");
        for (signal, count) in &report.by_signal {
            println!("    {signal:?}={count}");
        }
    }
    if !report.ready.is_empty() {
        println!("  ready:");
        for entry in report.ready.iter().take(limit) {
            println!(
                "    {:?} -> {:?} op={} count={}",
                entry.original, entry.expected, entry.operation, entry.count
            );
        }
    }
    if !report.rejected_pairs.is_empty() {
        println!("  rejected_pairs:");
        for entry in report.rejected_pairs.iter().take(limit) {
            println!("    {:?} -> {:?}", entry.original, entry.expected);
        }
    }
    Ok(())
}

fn pack_correction_learning(
    input: &std::path::Path,
    out: &std::path::Path,
    min_count: usize,
) -> io::Result<()> {
    let (report, write) = learning_loop::pack_correction_learning(input, out, min_count)?;
    println!(
        "nanda_correction_learning_pack: input={} output={} ready={} encoded={} skipped={} min_count={}",
        input.display(),
        out.display(),
        report.ready_entries,
        write.encoded,
        write.skipped,
        report.min_count
    );
    Ok(())
}

fn reinforced_live_text(records: &[llmwave::LlmWavePhraseExperience], min_count: usize) -> String {
    reinforced_live_counts(records, min_count)
        .keys()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
}

fn reinforced_live_counts(
    records: &[llmwave::LlmWavePhraseExperience],
    min_count: usize,
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::<String, usize>::new();
    for record in records {
        *counts.entry(record.text.clone()).or_default() += 1;
    }
    counts.retain(|_phrase, count| *count >= min_count);
    counts
}

fn live_phrase_quality_counts(
    path: &std::path::Path,
) -> io::Result<(usize, BTreeMap<String, usize>)> {
    let text = std::fs::read_to_string(path)?;
    let mut accepted = 0_usize;
    let mut rejected = BTreeMap::<String, usize>::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(record) = serde_json::from_str::<llmwave::LlmWavePhraseExperience>(line) else {
            *rejected.entry("invalid_json".to_string()).or_default() += 1;
            continue;
        };
        if let Some(reason) = llmwave::stored_phrase_experience_rejection_reason(&record) {
            *rejected.entry(reason.as_str().to_string()).or_default() += 1;
        } else {
            accepted += 1;
        }
    }
    Ok((accepted, rejected))
}

fn print_live_phrase_counts(live_counts: &BTreeMap<String, usize>, limit: usize) {
    let mut rows = live_counts.iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| right.1.cmp(left.1).then_with(|| left.0.cmp(right.0)));
    println!("  learned_phrases:");
    if rows.is_empty() {
        println!("    none");
        return;
    }
    for (phrase, count) in rows.into_iter().take(limit) {
        println!("    count={count} text={phrase:?}");
    }
}

fn reinforced_phrase_report_sample(
    live_counts: &BTreeMap<String, usize>,
    limit: usize,
) -> Vec<String> {
    let mut rows = live_counts.iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| right.1.cmp(left.1).then_with(|| left.0.cmp(right.0)));
    rows.into_iter()
        .take(limit.saturating_mul(8).clamp(16, 64))
        .map(|(phrase, _count)| phrase.clone())
        .collect()
}

fn print_learning_deltas<'a>(
    seed_memory: &llmwave::LlmWaveMemory,
    combined_memory: &llmwave::LlmWaveMemory,
    live_phrases: impl Iterator<Item = &'a String>,
    limit: usize,
) {
    println!("  l3_learning_deltas:");
    let phrases = live_phrases.map(String::as_str).collect::<Vec<_>>();
    let deltas = llmwave::learning_deltas(seed_memory, combined_memory, phrases.into_iter(), limit);
    if deltas.is_empty() {
        println!("    none");
        return;
    }
    for delta in deltas {
        println!(
            "    prefix={:?} next={:?} seed={:.3} live={:.3} support={} width={} phrase={:?}",
            delta.prefix,
            delta.next_token,
            delta.seed_score,
            delta.live_score,
            delta.live_support,
            delta.width,
            delta.phrase
        );
    }
}

fn print_prediction_deltas<'a>(
    seed_memory: &llmwave::LlmWaveMemory,
    combined_memory: &llmwave::LlmWaveMemory,
    live_phrases: impl Iterator<Item = &'a String>,
    limit: usize,
) {
    println!("  prediction_deltas:");
    let mut printed = 0usize;
    for phrase in live_phrases {
        let tokens = llmwave::tokenize(phrase);
        if tokens.len() < 3 {
            continue;
        }
        for width in (2..=4.min(tokens.len() - 1)).rev() {
            let prefix = tokens[..width].join(" ");
            let seed_top = seed_memory.predict_phrase(&prefix, 1, 1).into_iter().next();
            let combined_top = combined_memory
                .predict_phrase(&prefix, 1, 1)
                .into_iter()
                .next();
            if prediction_text(&seed_top) == prediction_text(&combined_top) {
                continue;
            }
            println!(
                "    prefix={prefix:?} seed={} live={}",
                format_prediction(seed_top.as_ref()),
                format_prediction(combined_top.as_ref())
            );
            printed += 1;
            break;
        }
        if printed >= limit {
            break;
        }
    }
    if printed == 0 {
        println!("    none");
    }
}

fn prediction_text(prediction: &Option<llmwave::LlmWavePhrasePrediction>) -> Option<&str> {
    prediction.as_ref().map(|item| item.text.as_str())
}

fn format_prediction(prediction: Option<&llmwave::LlmWavePhrasePrediction>) -> String {
    prediction
        .map(|item| {
            format!(
                "{:?} score={:.3} support={}",
                item.text, item.score, item.support
            )
        })
        .unwrap_or_else(|| "none".to_string())
}

fn print_recent_traces(limit: usize) {
    let records = journal::load_recent_traces(limit);
    if records.is_empty() {
        println!("recent_traces: none");
        return;
    }
    println!("recent_traces: {}", records.len());
    for record in records {
        let original = record
            .original
            .as_deref()
            .map(|value| format!("{value:?}"))
            .unwrap_or_else(|| format!("len-hidden no_raw={}", record.no_raw_secret_text));
        let expected = record
            .expected
            .as_deref()
            .map(|value| format!("{value:?}"))
            .unwrap_or_else(|| "hidden".to_string());
        println!(
            "- ts={} decision={} chosen={} original={} expected={}",
            record.ts,
            record.decision,
            record.chosen.as_deref().unwrap_or("none"),
            original,
            expected
        );
        if record.candidates.is_empty() {
            println!("  candidates: none-recorded");
        } else {
            println!("  candidates:");
            for candidate in &record.candidates {
                let marker = if candidate.accepted { "*" } else { " " };
                let text = candidate
                    .text
                    .as_deref()
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_else(|| "hidden".to_string());
                println!(
                    "  {marker} {} energy={:.3} risk={:.3} text={}",
                    candidate.source, candidate.energy, candidate.risk, text
                );
            }
        }
        let cells = record
            .cells
            .iter()
            .filter(|cell| {
                cell.generated > 0
                    || cell.accepted > 0
                    || cell.vetoed > 0
                    || cell.kept > 0
                    || cell.role == "signal"
            })
            .map(|cell| {
                format!(
                    "{}:{} g={} a={} v={} k={} e={:.3}",
                    cell.cell,
                    cell.role,
                    cell.generated,
                    cell.accepted,
                    cell.vetoed,
                    cell.kept,
                    cell.top_energy
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        println!("  cells: {cells}");
    }
}

fn print_trace(text: &str, options: &WaveOptions, record_trace: bool) {
    let trace = run_wave_trace_with_options(text, options);
    if record_trace {
        journal::record_trace("manual_trace", "trace", &trace, None);
    }
    println!("original: {:?}", trace.original);
    if !options.disabled().is_empty() {
        println!("disabled: {}", options.disabled().join(", "));
    }
    println!("L1 packets: {}", trace.l1.len());
    for packet in trace.l1.iter().take(8) {
        let modes = packet
            .modes
            .iter()
            .take(3)
            .map(|mode| {
                format!(
                    "{}#{}:{}:{:.3}",
                    mode.cell,
                    mode.mode_id,
                    mode.role.as_str(),
                    mode.energy
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        println!("  {} -> {}", packet.cell, modes);
    }
    println!("L2 candidates: {}", trace.l2_candidates.len());
    for candidate in &trace.l2_candidates {
        println!(
            "  {:?} source={} energy={:.3} risk={:.3}",
            candidate.text, candidate.source, candidate.energy, candidate.risk
        );
    }
    println!("L3:");
    for layer in &trace.l3 {
        println!("  {}: {}", layer.name, layer.summary);
    }
    if options.llmwave_shadow() || options.llmwave_apply() {
        let memory = llmwave::load_default_memory();
        let predictions = memory.predict_phrase(text, 3, 5);
        if !predictions.is_empty() {
            println!("L3 phrase predictions:");
            for prediction in predictions {
                println!(
                    "  {:?} score={:.3} support={}",
                    prediction.text, prediction.score, prediction.support
                );
            }
        }
    }
    match trace.decision {
        WaveDecision::Apply { text, confidence } => {
            println!("decision: apply {:?} confidence={confidence:.3}", text);
        }
        WaveDecision::Keep { reason } => println!("decision: keep {reason}"),
        WaveDecision::Veto { reason } => println!("decision: veto {reason}"),
    }
}

fn print_real_suite(
    options: &WaveOptions,
    ablation: bool,
    show_failures: bool,
    show_worsened: bool,
    record_trace: bool,
) -> io::Result<()> {
    let suite = real_suite::load()?;
    println!("nanda_wave_real_suite:");
    println!("  cases: {}", suite.cases.len());
    println!("  sources:");
    for source in &suite.sources {
        println!("    {}: {}", source.path, source.cases);
    }
    print_cases("wave", &suite.cases, options);

    let safety = CorrectionSafety::Experimental;
    let baseline = status::evaluate_deterministic(&suite.cases, safety);
    print_eval_summary("deterministic", &baseline);

    let (wave, wave_stats) = evaluate_wave_with_options(&suite.cases, options);
    print_reason_breakdown(&suite.cases, &baseline, &wave);
    if record_trace {
        for (idx, case) in suite.cases.iter().enumerate() {
            let trace = run_wave_trace_with_options(&case.original, options);
            journal::record_trace(
                format!("real_suite:{idx}"),
                case.reason.clone(),
                &trace,
                Some(&case.expected),
            );
        }
    }
    let baseline_ok = baseline.iter().filter(|result| result.ok).count();
    let worsened = suite
        .cases
        .iter()
        .zip(baseline.iter())
        .zip(wave.iter())
        .filter(|((case, base), wave)| base.ok && wave.output != case.expected)
        .count();
    println!(
        "gate: baseline_ok={baseline_ok}/{} wave_ok={}/{} wave_worsened_vs_baseline={}",
        suite.cases.len(),
        wave_stats.ok,
        wave_stats.cases,
        worsened
    );
    print_gate_status(suite.cases.len(), baseline_ok, wave_stats.ok, worsened);
    if ablation {
        let ablation_rows = ablation_rows(&suite.cases);
        print_ablation_rows(&ablation_rows);
        print_layer_impact(&ablation_rows);
        print_cell_ablation(&suite.cases, "PatternWaveCell32", "pattern_wave");
        print_cell_ablation(
            &suite.cases,
            "StructuralRelationCell32",
            "structural_relation",
        );
    }
    if show_failures {
        print_failures(&suite.cases, &wave);
    }
    if show_worsened {
        print_worsened(&suite.cases, &baseline, &wave);
    }
    Ok(())
}

fn print_gate_status(cases: usize, baseline_ok: usize, wave_ok: usize, worsened: usize) {
    let gate_green = wave_ok >= baseline_ok && worsened == 0;
    if gate_green {
        println!("promotion_status: gate_green_but_manual_review_required");
        println!("mode_status: ensemble_mode_candidate");
    } else {
        println!("promotion_status: trace_only_do_not_promote");
        println!(
            "mode_status: ensemble_mode_not_found reason=wave_ok:{wave_ok}/{cases} baseline_ok:{baseline_ok}/{cases} worsened:{worsened}"
        );
    }
}

fn print_cases(label: &str, cases: &[EvalCase], options: &WaveOptions) {
    let (_results, stats) = evaluate_wave_with_options(cases, options);
    println!(
        "{label}: cases={} ok={}/{} changed={}",
        stats.cases, stats.ok, stats.cases, stats.changed
    );
}

#[derive(Debug, Clone)]
struct AblationRow {
    cell: &'static str,
    ok: usize,
    cases: usize,
    changed: usize,
    delta: isize,
}

fn ablation_rows(cases: &[EvalCase]) -> Vec<AblationRow> {
    let (_base, base_stats) = evaluate_wave(cases);
    wave_cells()
        .iter()
        .map(|cell| {
            let options = WaveOptions::with_disabled(&[cell.to_string()]);
            let (_results, stats) = evaluate_wave_with_options(cases, &options);
            AblationRow {
                cell,
                ok: stats.ok,
                cases: stats.cases,
                changed: stats.changed,
                delta: stats.ok as isize - base_stats.ok as isize,
            }
        })
        .collect()
}

fn print_ablation_rows(rows: &[AblationRow]) {
    println!("wave_ablation:");
    for row in rows {
        println!(
            "  without {}: ok={}/{} delta={:+} changed={}",
            row.cell, row.ok, row.cases, row.delta, row.changed
        );
    }
}

fn print_layer_impact(rows: &[AblationRow]) {
    let mut layers: BTreeMap<&'static str, isize> = BTreeMap::new();
    for row in rows {
        *layers.entry(wave_cell_meta(row.cell).layer).or_default() += row.delta.min(0);
    }
    println!("layer_impact:");
    for layer in ["L1", "L2", "L3"] {
        println!(
            "  {layer}: delta={:+}",
            layers.get(layer).copied().unwrap_or(0)
        );
    }
}

fn print_surface_l2_ablation(cases: &[EvalCase]) {
    let mut cases = cases.to_vec();
    cases.extend([
        EvalCase {
            original: "звгрузи ".to_string(),
            expected: "загрузи ".to_string(),
            reason: "surface_l2_probe".to_string(),
        },
        EvalCase {
            original: "делай проверк ".to_string(),
            expected: "делай проверка ".to_string(),
            reason: "surface_l2_probe".to_string(),
        },
        EvalCase {
            original: "пукнут ".to_string(),
            expected: "пукнут ".to_string(),
            reason: "surface_l2_keep_probe".to_string(),
        },
    ]);

    struct Scenario {
        label: &'static str,
        disabled: Vec<String>,
    }

    let scenarios = [
        Scenario {
            label: "full",
            disabled: Vec::new(),
        },
        Scenario {
            label: "without_surface_typo",
            disabled: vec![L2_SURFACE_MOTIF_CELL.to_string()],
        },
        Scenario {
            label: "without_surface_completion",
            disabled: vec![L2_SURFACE_COMPLETION_CELL.to_string()],
        },
        Scenario {
            label: "without_surface_l2",
            disabled: surface_l2_cells(),
        },
        Scenario {
            label: "l2_surface_only",
            disabled: wave_cells()
                .iter()
                .filter(|cell| {
                    **cell != L2_SURFACE_MOTIF_CELL
                        && **cell != L2_SURFACE_COMPLETION_CELL
                        && **cell != L2_WORD_ATTRACTOR_CELL
                })
                .map(|cell| (*cell).to_string())
                .collect(),
        },
        Scenario {
            label: "without_l3_consensus",
            disabled: [
                "TechnicalContextCell32",
                "PhraseForecastCell32",
                "PatternWaveCell32",
                "StructuralRelationCell32",
                "PhraseCell32",
                "MeshConsensusCell32",
            ]
            .iter()
            .map(|cell| (*cell).to_string())
            .collect(),
        },
    ];

    println!("surface_l2_ablation: cases={}", cases.len());
    for scenario in scenarios {
        let options = WaveOptions::with_disabled(&scenario.disabled);
        let (results, stats) = evaluate_wave_with_options(&cases, &options);
        let surface_candidates = cases
            .iter()
            .map(|case| run_wave_trace_with_options(&case.original, &options))
            .map(|trace| {
                trace
                    .l2_candidates
                    .iter()
                    .filter(|candidate| {
                        candidate.source == L2_SURFACE_MOTIF_CELL
                            || candidate.source == L2_SURFACE_COMPLETION_CELL
                            || candidate.source == L2_WORD_ATTRACTOR_CELL
                    })
                    .count()
            })
            .sum::<usize>();
        let changed_from_original = results
            .iter()
            .zip(&cases)
            .filter(|(result, case)| result.output != case.original)
            .count();
        println!(
            "  {}: ok={}/{} changed={} surface_candidates={} disabled={}",
            scenario.label,
            stats.ok,
            stats.cases,
            changed_from_original,
            surface_candidates,
            if scenario.disabled.is_empty() {
                "none".to_string()
            } else {
                scenario.disabled.join(",")
            }
        );
    }
}

fn print_ensemble_contribution_report(cases: &[EvalCase], full_cases: usize) {
    let safety = CorrectionSafety::Experimental;
    let baseline = status::evaluate_deterministic(cases, safety);
    let baseline_ok = baseline.iter().filter(|result| result.ok).count();

    println!(
        "ensemble_contribution_report: cases={} full_cases={} sampled={}",
        cases.len(),
        full_cases,
        cases.len() < full_cases
    );
    println!(
        "  deterministic_no_lem_baseline: ok={}/{} {:.1}%",
        baseline_ok,
        cases.len(),
        percent(baseline_ok, cases.len())
    );
    println!("  note: live daemon LEM is scoped-tail runtime; this baseline is typing-assist only");

    for scenario in contribution_scenarios() {
        let report = contribution_report_for_scenario(cases, &baseline, &scenario);
        print_contribution_report(&report);
    }
}

fn print_l2_form_attractor_candidates(
    text: &str,
    options: &WaveOptions,
    limit: usize,
) -> io::Result<()> {
    let trace = run_wave_trace_with_options(text, options);
    let candidates = trace
        .l2_candidates
        .iter()
        .filter(|candidate| candidate.source == L2_WORD_ATTRACTOR_CELL)
        .take(limit)
        .map(|candidate| {
            serde_json::json!({
                "text": candidate.text,
                "source": candidate.source,
                "energy": candidate.energy,
                "risk": candidate.risk,
                "support": candidate.support,
            })
        })
        .collect::<Vec<_>>();
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "kind": "l2_form_attractor_candidates",
            "input": text,
            "source": L2_WORD_ATTRACTOR_CELL,
            "read_as": "clean corpus word-form attractor; dirty input is only a probe, not training authority",
            "candidates": candidates,
        }))?
    );
    Ok(())
}

#[derive(Debug, Clone)]
struct ContributionScenario {
    label: &'static str,
    options: WaveOptions,
}

#[derive(Debug, Clone)]
struct ContributionReport {
    label: &'static str,
    cases: usize,
    ok: usize,
    changed: usize,
    delta_vs_baseline: isize,
    improved_vs_baseline: usize,
    worsened_vs_baseline: usize,
    total_l2_candidates: usize,
    applied: usize,
    applied_sources: BTreeMap<String, usize>,
    disabled: Vec<String>,
    l2_weight: f32,
    l3_weight: f32,
}

fn contribution_scenarios() -> Vec<ContributionScenario> {
    let l3_disabled = l3_support_cells();
    let surface_disabled = surface_l2_cells();
    let l1_sensors = l1_sensor_cells();
    let surface_only_disabled = wave_cells()
        .iter()
        .filter(|cell| {
            **cell != L2_SURFACE_MOTIF_CELL
                && **cell != L2_SURFACE_COMPLETION_CELL
                && **cell != "MeshConsensusCell32"
                && !l1_sensors.iter().any(|sensor| sensor == *cell)
        })
        .map(|cell| (*cell).to_string())
        .collect::<Vec<_>>();

    vec![
        ContributionScenario {
            label: "nanda_full_ensemble",
            options: WaveOptions::default(),
        },
        ContributionScenario {
            label: "nanda_l2_mesh_only",
            options: WaveOptions::with_disabled(&l3_disabled).with_layer_weights(1.0, 0.0),
        },
        ContributionScenario {
            label: "nanda_l2_l3_without_surface_l2",
            options: WaveOptions::with_disabled(&surface_disabled),
        },
        ContributionScenario {
            label: "nanda_surface_l2_only_with_mesh",
            options: WaveOptions::with_disabled(&surface_only_disabled)
                .with_layer_weights(1.0, 0.0),
        },
    ]
}

fn l3_support_cells() -> Vec<String> {
    [
        "TechnicalContextCell32",
        "PhraseForecastCell32",
        "PatternWaveCell32",
        "StructuralRelationCell32",
        "PhraseCell32",
    ]
    .iter()
    .map(|cell| (*cell).to_string())
    .collect()
}

fn l1_sensor_cells() -> Vec<String> {
    [
        "Utf8Cell32",
        "ScriptCell32",
        "KeyboardCell32",
        "BoundaryCell32",
    ]
    .iter()
    .map(|cell| (*cell).to_string())
    .collect()
}

fn surface_l2_cells() -> Vec<String> {
    [
        L2_SURFACE_MOTIF_CELL,
        L2_SURFACE_COMPLETION_CELL,
        L2_WORD_ATTRACTOR_CELL,
    ]
    .iter()
    .map(|cell| (*cell).to_string())
    .collect()
}

fn contribution_report_for_scenario(
    cases: &[EvalCase],
    baseline: &[status::EvalResult],
    scenario: &ContributionScenario,
) -> ContributionReport {
    let mut ok = 0;
    let mut changed = 0;
    let mut improved_vs_baseline = 0;
    let mut worsened_vs_baseline = 0;
    let mut total_l2_candidates = 0;
    let mut applied = 0;
    let mut applied_sources = BTreeMap::new();

    for (case, baseline) in cases.iter().zip(baseline) {
        let trace = run_wave_trace_with_options(&case.original, &scenario.options);
        let output = trace
            .output()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| case.original.clone());
        let case_ok = output == case.expected;
        ok += usize::from(case_ok);
        changed += usize::from(output != case.original);
        improved_vs_baseline += usize::from(!baseline.ok && case_ok);
        worsened_vs_baseline += usize::from(baseline.ok && output != case.expected);
        total_l2_candidates += trace.l2_candidates.len();
        if let Some(source) = applied_source_for_trace(&trace, &output) {
            applied += 1;
            *applied_sources.entry(source.to_string()).or_default() += 1;
        }
    }

    let baseline_ok = baseline.iter().filter(|result| result.ok).count();
    ContributionReport {
        label: scenario.label,
        cases: cases.len(),
        ok,
        changed,
        delta_vs_baseline: ok as isize - baseline_ok as isize,
        improved_vs_baseline,
        worsened_vs_baseline,
        total_l2_candidates,
        applied,
        applied_sources,
        disabled: scenario.options.disabled().to_vec(),
        l2_weight: scenario.options.l2_weight(),
        l3_weight: scenario.options.l3_weight(),
    }
}

fn applied_source_for_trace<'a>(
    trace: &'a lay::nanda_wave::WaveTrace,
    output: &str,
) -> Option<&'a str> {
    if !matches!(trace.decision, WaveDecision::Apply { .. }) {
        return None;
    }
    let output = output.trim_end();
    trace
        .l2_candidates
        .iter()
        .find(|candidate| candidate.text == output)
        .map(|candidate| candidate.source)
}

fn print_contribution_report(report: &ContributionReport) {
    let avg_candidates = if report.cases == 0 {
        0.0
    } else {
        report.total_l2_candidates as f64 / report.cases as f64
    };
    let sources = if report.applied_sources.is_empty() {
        "none".to_string()
    } else {
        report
            .applied_sources
            .iter()
            .map(|(source, count)| format!("{source}:{count}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    let disabled = if report.disabled.is_empty() {
        "none".to_string()
    } else {
        report.disabled.join(",")
    };

    println!(
        "  {}: ok={}/{} delta_vs_baseline={:+} improved={} worsened={} changed={} applied={} avg_l2_candidates={:.2} l2_weight={:.2} l3_weight={:.2}",
        report.label,
        report.ok,
        report.cases,
        report.delta_vs_baseline,
        report.improved_vs_baseline,
        report.worsened_vs_baseline,
        report.changed,
        report.applied,
        avg_candidates,
        report.l2_weight,
        report.l3_weight
    );
    println!("    applied_sources: {sources}");
    println!("    disabled: {disabled}");
}

fn print_l2_candidate_flow_report(
    cases: &[EvalCase],
    full_cases: usize,
    options: &WaveOptions,
    show_examples: bool,
) {
    let dirty_cases = cases
        .iter()
        .filter(|case| case.original != case.expected)
        .collect::<Vec<_>>();
    let mut stats: BTreeMap<String, L2CandidateFlowStats> = BTreeMap::new();
    let mut cases_with_l2 = 0usize;
    let mut no_l2_candidates = 0usize;
    let mut expected_candidate_present = 0usize;
    let mut expected_candidate_missing = 0usize;
    let mut expected_candidate_applied = 0usize;
    let mut present_but_not_applied = 0usize;
    let mut nonfinal_apply = 0usize;
    let mut examples = L2CandidateFlowExamples::default();

    for case in &dirty_cases {
        let trace = run_wave_trace_with_options(&case.original, options);
        if trace.l2_candidates.is_empty() {
            no_l2_candidates += 1;
            examples.no_l2.push(flow_example(case, &trace, None));
            continue;
        }
        cases_with_l2 += 1;

        for candidate in &trace.l2_candidates {
            stats
                .entry(candidate.source.to_string())
                .or_default()
                .generated += 1;
        }
        if let Some(first) = trace.l2_candidates.first() {
            stats.entry(first.source.to_string()).or_default().first += 1;
        }

        let expected_sources = trace
            .l2_candidates
            .iter()
            .filter(|candidate| {
                candidate_output_for_original(&case.original, &candidate.text) == case.expected
            })
            .map(|candidate| candidate.source.to_string())
            .collect::<Vec<_>>();
        if expected_sources.is_empty() {
            expected_candidate_missing += 1;
            examples
                .missing_expected
                .push(flow_example(case, &trace, None));
        } else {
            expected_candidate_present += 1;
            for source in &expected_sources {
                stats.entry(source.clone()).or_default().expected_present += 1;
            }
        }

        match trace.decision {
            WaveDecision::Apply { ref text, .. } => {
                let applied_source = applied_source_for_trace(&trace, text);
                if let Some(source) = applied_source {
                    stats.entry(source.to_string()).or_default().accepted += 1;
                }
                if text == &case.expected {
                    expected_candidate_applied += 1;
                } else {
                    nonfinal_apply += 1;
                    if let Some(source) = applied_source {
                        stats.entry(source.to_string()).or_default().nonfinal_apply += 1;
                    }
                    examples
                        .nonfinal_apply
                        .push(flow_example(case, &trace, Some(text.as_str())));
                }
                if text != &case.expected && !expected_sources.is_empty() {
                    present_but_not_applied += 1;
                    for source in &expected_sources {
                        stats
                            .entry(source.clone())
                            .or_default()
                            .expected_present_not_applied += 1;
                    }
                    examples.present_not_applied.push(flow_example(
                        case,
                        &trace,
                        Some(text.as_str()),
                    ));
                }
            }
            WaveDecision::Keep { .. } | WaveDecision::Veto { .. } => {
                if !expected_sources.is_empty() {
                    present_but_not_applied += 1;
                    for source in &expected_sources {
                        stats
                            .entry(source.clone())
                            .or_default()
                            .expected_present_not_applied += 1;
                    }
                    examples
                        .present_not_applied
                        .push(flow_example(case, &trace, None));
                }
            }
        }
    }

    println!(
        "l2_candidate_flow_report: cases={} full_cases={} sampled={} dirty_cases={}",
        cases.len(),
        full_cases,
        cases.len() < full_cases,
        dirty_cases.len()
    );
    println!("  authority: analysis_only_no_runtime_change");
    println!("  cases_with_l2_candidates: {cases_with_l2}");
    println!("  no_l2_candidates: {no_l2_candidates}");
    println!("  expected_candidate_present: {expected_candidate_present}");
    println!("  expected_candidate_missing: {expected_candidate_missing}");
    println!("  expected_candidate_applied: {expected_candidate_applied}");
    println!("  expected_present_but_not_applied: {present_but_not_applied}");
    println!("  nonfinal_apply: {nonfinal_apply}");
    println!(
        "  note: nonfinal_apply means output != expected; multi-error rows can be partial fixes"
    );
    println!("  sources:");
    for (source, row) in stats {
        if row.is_empty() {
            continue;
        }
        println!(
            "    {source}: generated={} first={} expected_present={} expected_present_not_applied={} accepted={} nonfinal_apply={}",
            row.generated,
            row.first,
            row.expected_present,
            row.expected_present_not_applied,
            row.accepted,
            row.nonfinal_apply
        );
    }
    if show_examples {
        print_flow_examples("no_l2", &examples.no_l2);
        print_flow_examples("missing_expected", &examples.missing_expected);
        print_flow_examples("present_not_applied", &examples.present_not_applied);
        print_flow_examples("nonfinal_apply", &examples.nonfinal_apply);
    }
}

#[derive(Debug, Default, Clone)]
struct L2CandidateFlowStats {
    generated: usize,
    first: usize,
    expected_present: usize,
    expected_present_not_applied: usize,
    accepted: usize,
    nonfinal_apply: usize,
}

impl L2CandidateFlowStats {
    fn is_empty(&self) -> bool {
        self.generated == 0
            && self.first == 0
            && self.expected_present == 0
            && self.expected_present_not_applied == 0
            && self.accepted == 0
            && self.nonfinal_apply == 0
    }
}

#[derive(Debug, Default)]
struct L2CandidateFlowExamples {
    no_l2: Vec<L2CandidateFlowExample>,
    missing_expected: Vec<L2CandidateFlowExample>,
    present_not_applied: Vec<L2CandidateFlowExample>,
    nonfinal_apply: Vec<L2CandidateFlowExample>,
}

#[derive(Debug, Clone)]
struct L2CandidateFlowExample {
    original: String,
    expected: String,
    decision: String,
    output: String,
    first_candidate: String,
    top_sources: String,
}

fn flow_example(
    case: &EvalCase,
    trace: &lay::nanda_wave::WaveTrace,
    output: Option<&str>,
) -> L2CandidateFlowExample {
    L2CandidateFlowExample {
        original: case.original.clone(),
        expected: case.expected.clone(),
        decision: decision_label(&trace.decision).to_string(),
        output: output
            .map(ToOwned::to_owned)
            .or_else(|| trace.output().map(ToOwned::to_owned))
            .unwrap_or_else(|| "keep".to_string()),
        first_candidate: trace
            .l2_candidates
            .first()
            .map(candidate_short)
            .unwrap_or_else(|| "none".to_string()),
        top_sources: trace
            .l2_candidates
            .iter()
            .take(4)
            .map(|candidate| candidate.source)
            .collect::<Vec<_>>()
            .join(","),
    }
}

fn print_flow_examples(label: &str, examples: &[L2CandidateFlowExample]) {
    println!("  examples_{label}:");
    if examples.is_empty() {
        println!("    none");
        return;
    }
    for example in examples.iter().take(8) {
        println!(
            "    {:?} -> expected {:?} | decision={} output={:?} first={} sources={}",
            example.original,
            example.expected,
            example.decision,
            example.output,
            example.first_candidate,
            example.top_sources
        );
    }
}

fn decision_label(decision: &WaveDecision) -> &'static str {
    match decision {
        WaveDecision::Apply { .. } => "apply",
        WaveDecision::Keep { .. } => "keep",
        WaveDecision::Veto { .. } => "veto",
    }
}

fn candidate_short(candidate: &lay::nanda_wave::WordCandidate) -> String {
    format!(
        "{}:{:?}:e{:.2}:r{:.2}",
        candidate.source, candidate.text, candidate.energy, candidate.risk
    )
}

fn candidate_output_for_original(original: &str, candidate: &str) -> String {
    if original.ends_with(' ') && !candidate.ends_with(' ') {
        format!("{candidate} ")
    } else {
        candidate.to_string()
    }
}

fn print_cell_ablation(cases: &[EvalCase], cell: &str, label: &str) {
    let full_options = WaveOptions::default();
    let disabled_options = WaveOptions::with_disabled(&[cell.to_string()]);
    let (full_results, full_stats) = evaluate_wave_with_options(cases, &full_options);
    let (disabled_results, disabled_stats) = evaluate_wave_with_options(cases, &disabled_options);
    println!(
        "{label}_ablation: full_ok={}/{} without_ok={}/{} delta_without={:+}",
        full_stats.ok,
        full_stats.cases,
        disabled_stats.ok,
        disabled_stats.cases,
        disabled_stats.ok as isize - full_stats.ok as isize
    );

    let mut classes: BTreeMap<&str, (usize, usize, isize, usize)> = BTreeMap::new();
    for ((case, full), disabled) in cases.iter().zip(&full_results).zip(&disabled_results) {
        let entry = classes.entry(case.reason.as_str()).or_default();
        entry.0 += usize::from(full.ok);
        entry.1 += usize::from(disabled.ok);
        entry.2 += ok_as_isize(disabled.ok) - ok_as_isize(full.ok);
        entry.3 += usize::from(full.output != disabled.output);
    }

    println!("{label}_class_delta:");
    for (reason, (full_ok, without_ok, delta, changed)) in classes {
        if delta != 0 || changed != 0 {
            println!(
                "  {reason}: full_ok={full_ok} without_ok={without_ok} delta_without={delta:+} output_changed={changed}"
            );
        }
    }
}

fn ok_as_isize(ok: bool) -> isize {
    if ok {
        1
    } else {
        0
    }
}

fn print_eval_summary(label: &str, results: &[status::EvalResult]) {
    let ok = results.iter().filter(|result| result.ok).count();
    println!(
        "{label}: cases={} ok={}/{} {:.1}%",
        results.len(),
        ok,
        results.len(),
        percent(ok, results.len())
    );
}

fn print_reason_breakdown(
    cases: &[EvalCase],
    baseline: &[status::EvalResult],
    wave: &[lay::nanda_wave::WaveEvalResult],
) {
    #[derive(Default)]
    struct Row {
        cases: usize,
        baseline_ok: usize,
        wave_ok: usize,
        changed: usize,
    }
    let mut rows = BTreeMap::<String, Row>::new();
    for ((case, base), wave) in cases.iter().zip(baseline).zip(wave) {
        let row = rows.entry(case.reason.clone()).or_default();
        row.cases += 1;
        row.baseline_ok += usize::from(base.ok);
        row.wave_ok += usize::from(wave.ok);
        row.changed += usize::from(wave.output != case.original);
    }
    println!("per_class:");
    for (reason, row) in rows {
        let delta = row.wave_ok as isize - row.baseline_ok as isize;
        println!(
            "  {reason}: cases={} baseline={}/{} wave={}/{} delta={:+} changed={}",
            row.cases, row.baseline_ok, row.cases, row.wave_ok, row.cases, delta, row.changed
        );
    }
}

fn percent(num: usize, den: usize) -> f64 {
    if den == 0 {
        0.0
    } else {
        num as f64 * 100.0 / den as f64
    }
}

struct WaveCellMeta {
    label: &'static str,
    role: &'static str,
    zone: &'static str,
    layer: &'static str,
    phase: f32,
}

fn wave_cell_meta(name: &str) -> WaveCellMeta {
    match name {
        "Utf8Cell32" => WaveCellMeta {
            label: "UTF-8",
            role: "любой символ",
            zone: "sensors",
            layer: "L1",
            phase: 0.15,
        },
        "ScriptCell32" => WaveCellMeta {
            label: "Письмо",
            role: "кириллица / латиница / регистр",
            zone: "sensors",
            layer: "L1",
            phase: 0.85,
        },
        "KeyboardCell32" => WaveCellMeta {
            label: "Клавиши",
            role: "RU/EN клавиатурная мода",
            zone: "sensors",
            layer: "L1",
            phase: 1.55,
        },
        "BoundaryCell32" => WaveCellMeta {
            label: "Границы",
            role: "пробелы / пунктуация / край слова",
            zone: "sensors",
            layer: "L1",
            phase: 2.30,
        },
        "LayoutWordCell32" => WaveCellMeta {
            label: "Раскладка",
            role: "кандидат раскладки",
            zone: "candidates",
            layer: "L2",
            phase: 0.55,
        },
        "ShortTokenCell32" => WaveCellMeta {
            label: "Короткий токен",
            role: "одна буква: варианты и/в/оставить",
            zone: "candidates",
            layer: "L2",
            phase: 0.95,
        },
        "TechTokenCell32" => WaveCellMeta {
            label: "Тех. токен",
            role: "технический токен",
            zone: "candidates",
            layer: "L2",
            phase: 1.35,
        },
        "LearnedMemoryCell32" => WaveCellMeta {
            label: "Память",
            role: "выученный кандидат",
            zone: "candidates",
            layer: "L2",
            phase: 1.75,
        },
        "CommonRuFixCell32" => WaveCellMeta {
            label: "Рус. правка",
            role: "выученная опечатка",
            zone: "candidates",
            layer: "L2",
            phase: 1.92,
        },
        "PhraseMemoryCell32" => WaveCellMeta {
            label: "Память фраз",
            role: "выученная склейка / фраза",
            zone: "candidates",
            layer: "L2",
            phase: 2.02,
        },
        "UserMemoryCell32" => WaveCellMeta {
            label: "Пользователь",
            role: "личная правка",
            zone: "candidates",
            layer: "L2",
            phase: 2.08,
        },
        "SemanticWordCell32" => WaveCellMeta {
            label: "Смысл-слово",
            role: "ближайшее слово из контекста",
            zone: "candidates",
            layer: "L2",
            phase: 2.10,
        },
        "L2SurfaceMotifCell32" => WaveCellMeta {
            label: "L2 форма",
            role: "слово по центрам формы",
            zone: "candidates",
            layer: "L2",
            phase: 2.16,
        },
        "L2SurfaceCompletionCell32" => WaveCellMeta {
            label: "L2 окончание",
            role: "дописать слово по форме",
            zone: "candidates",
            layer: "L2",
            phase: 2.20,
        },
        "L2WordAttractorCell32" => WaveCellMeta {
            label: "L2 аттрактор",
            role: "полное слово из corpus/L1/L2 центра",
            zone: "candidates",
            layer: "L2",
            phase: 2.24,
        },
        "TechnicalContextCell32" => WaveCellMeta {
            label: "Защита",
            role: "контекст защиты",
            zone: "consensus",
            layer: "L3",
            phase: 2.00,
        },
        "PhraseForecastCell32" => WaveCellMeta {
            label: "Прекогниция",
            role: "фразовая волна до конца мысли",
            zone: "consensus",
            layer: "L3",
            phase: 2.45,
        },
        "PatternWaveCell32" => WaveCellMeta {
            label: "Паттерн",
            role: "локальные волны + общая форма",
            zone: "consensus",
            layer: "L3",
            phase: 2.58,
        },
        "StructuralRelationCell32" => WaveCellMeta {
            label: "Связи",
            role: "роли токенов и маршруты фразы",
            zone: "consensus",
            layer: "L3",
            phase: 2.62,
        },
        "PhraseCell32" => WaveCellMeta {
            label: "Фраза",
            role: "фразовая связность",
            zone: "consensus",
            layer: "L3",
            phase: 2.70,
        },
        "GrammarCell32" => WaveCellMeta {
            label: "Грамматика",
            role: "согласование русской фразы",
            zone: "candidates",
            layer: "L2",
            phase: 2.95,
        },
        "MeshConsensusCell32" => WaveCellMeta {
            label: "Mesh",
            role: "согласование решения",
            zone: "consensus",
            layer: "L3",
            phase: 3.20,
        },
        _ => WaveCellMeta {
            label: "Cell",
            role: "неизвестная роль",
            zone: "consensus",
            layer: "L3",
            phase: 0.0,
        },
    }
}

fn print_failures(cases: &[EvalCase], wave: &[lay::nanda_wave::WaveEvalResult]) {
    println!("wave_failures:");
    for (idx, (case, result)) in cases.iter().zip(wave).enumerate() {
        if result.ok {
            continue;
        }
        println!(
            "  case#{idx} reason={} original={:?} expected={:?} wave={:?}",
            case.reason, case.original, case.expected, result.output
        );
    }
}

fn print_worsened(
    cases: &[EvalCase],
    baseline: &[status::EvalResult],
    wave: &[lay::nanda_wave::WaveEvalResult],
) {
    println!("wave_worsened:");
    for (idx, ((case, base), wave)) in cases.iter().zip(baseline).zip(wave).enumerate() {
        if !base.ok || wave.ok {
            continue;
        }
        println!(
            "  case#{idx} reason={} original={:?} expected={:?} wave={:?}",
            case.reason, case.original, case.expected, wave.output
        );
    }
}

fn wave_cells() -> &'static [&'static str] {
    &[
        "Utf8Cell32",
        "ScriptCell32",
        "KeyboardCell32",
        "BoundaryCell32",
        "LayoutWordCell32",
        "ShortTokenCell32",
        "TechTokenCell32",
        "LearnedMemoryCell32",
        "CommonRuFixCell32",
        "PhraseMemoryCell32",
        "UserMemoryCell32",
        "SemanticWordCell32",
        "L2SurfaceMotifCell32",
        "L2SurfaceCompletionCell32",
        "L2WordAttractorCell32",
        "TechnicalContextCell32",
        "PhraseForecastCell32",
        "PatternWaveCell32",
        "StructuralRelationCell32",
        "PhraseCell32",
        "GrammarCell32",
        "MeshConsensusCell32",
    ]
}

fn arg_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == name)
        .map(|window| window[1].as_str())
}

fn arg_values(args: &[String], name: &str) -> Vec<String> {
    args.windows(2)
        .filter(|window| window[0] == name)
        .map(|window| window[1].clone())
        .collect()
}

fn parse_limit(value: &str, fallback: usize) -> usize {
    value.parse::<usize>().unwrap_or(fallback).clamp(1, 50)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn llmwave_corpus_report_measures_next_word_hits() {
        let report = llmwave_corpus_report_from_text(
            "train".to_string(),
            "на улице идет дождь\nя хочу проверить ввод\n".to_string(),
            "test".to_string(),
            "на улице идет дождь\nна улице идет снег\n".to_string(),
            8,
            100,
        );

        assert!(report.prediction_points > 0);
        assert!(report.ready_points > 0);
        assert!(report.top1_hits > 0);
        assert!(report.misses > 0);
        assert!(report
            .examples
            .iter()
            .any(|example| example.expected == "снег"));
    }

    #[test]
    fn dirty_log_field_collector_reads_json_strings() {
        let mut lines = Vec::new();
        collect_json_string_fields_from_str(
            r#"{"to":"проверить ввод","expected":"на улице дождь"}"#,
            &["to", "expected"],
            &mut lines,
            10,
        );

        assert_eq!(lines.len(), 2);
        assert!(lines.contains(&"проверить ввод".to_string()));
        assert!(lines.contains(&"на улице дождь".to_string()));
    }

    #[test]
    fn llmwave_promotion_gate_blocks_tiny_reports() {
        let report = LlmWaveCorpusReport {
            train_input: "train".to_string(),
            test_input: "test".to_string(),
            train_lines: 1,
            test_lines: 1,
            records: LLMWAVE_PROMOTION_MIN_RECORDS,
            vocabulary: LLMWAVE_PROMOTION_MIN_VOCABULARY,
            prediction_points: LLMWAVE_PROMOTION_MIN_POINTS - 1,
            ready_points: LLMWAVE_PROMOTION_MIN_POINTS - 1,
            top1_hits: LLMWAVE_PROMOTION_MIN_POINTS - 1,
            top3_hits: LLMWAVE_PROMOTION_MIN_POINTS - 1,
            misses: 0,
            avg_expected_score: 1.0,
            avg_top_score: 1.0,
            examples: Vec::new(),
        };

        let gate = llmwave_promotion_gate(&report);
        assert_eq!(gate.verdict, LlmWavePromotionVerdict::Watch);
        assert_eq!(gate.reason, "not_enough_prediction_points");
    }

    #[test]
    fn llmwave_promotion_gate_promotes_shadow_only_after_thresholds() {
        let points = LLMWAVE_PROMOTION_MIN_POINTS;
        let report = LlmWaveCorpusReport {
            train_input: "train".to_string(),
            test_input: "test".to_string(),
            train_lines: 20,
            test_lines: 20,
            records: LLMWAVE_PROMOTION_MIN_RECORDS,
            vocabulary: LLMWAVE_PROMOTION_MIN_VOCABULARY,
            prediction_points: points,
            ready_points: points,
            top1_hits: points,
            top3_hits: points,
            misses: 0,
            avg_expected_score: 1.0,
            avg_top_score: 1.0,
            examples: Vec::new(),
        };

        let gate = llmwave_promotion_gate(&report);
        assert_eq!(gate.verdict, LlmWavePromotionVerdict::PassShadow);
        assert_eq!(gate.reason, "dirty_eval_passed_shadow_thresholds");
    }
}
