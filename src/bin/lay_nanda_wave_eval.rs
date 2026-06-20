use lay::config::CorrectionSafety;
use lay::eval_cases::EvalCase;
use lay::nanda_wave::{
    evaluate_wave, evaluate_wave_with_options, run_wave_trace_with_options, WaveDecision,
    WaveOptions,
};
use lay::nanda_wave::{journal, llmwave};
use std::collections::BTreeMap;
use std::env;
use std::io;
use std::path::PathBuf;

#[path = "lay_nanda_wave_eval/real_suite.rs"]
mod real_suite;
#[path = "lay_nanda_wave_eval/status.rs"]
mod status;

fn main() -> io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if let Some(path) = arg_value(&args, "--learned") {
        env::set_var("LAY_NANDA_WAVE_MEMORY", path);
    }
    if let Some(path) = arg_value(&args, "--llmwave-memory") {
        env::set_var("LAY_LLMWAVE_MEMORY", path);
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
    if args.iter().any(|arg| arg == "--status-json") {
        status::print_status_json(
            args.iter().any(|arg| arg == "--refresh-status-json"),
            args.iter().any(|arg| arg == "--full-status-json"),
        )?;
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
    let paths = arg_values(&args, "--cases");
    if paths.is_empty() {
        eprintln!(
            "usage: lay-nanda-wave-eval --trace TEXT | --recent-traces N | --real-suite | --quick-ablation | --llmwave-pack-cases PATH --out PATH | --cases PATH"
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
        .filter(|(case, base)| base.ok && run_wave_output(&case.original, options) != case.expected)
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

fn run_wave_output(original: &str, options: &WaveOptions) -> String {
    let trace = run_wave_trace_with_options(original, options);
    trace
        .output()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| original.to_string())
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
