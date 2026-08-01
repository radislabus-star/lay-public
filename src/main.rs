//! lay — Caramba/Punto-style конвертер раскладки клавиатуры.
//!
//! Двухрежимная логика:
//! 1. Словарная конвертация US ↔ RU (микросекунды, детерминированно).
//! 2. Гибридный smart/model-режим включается явно через `--smart`.

use clap::{Parser, ValueEnum};
use std::io::{self, IsTerminal, Read};
use std::process;

use lay::{config, correction_core, dict, llm, typing_assist, typing_context};

#[derive(Parser, Debug)]
#[command(
    name = "lay",
    version,
    about = "Keyboard layout switcher and typing helper"
)]
struct Args {
    /// Текст для конвертации (если пусто — читаем stdin или --clipboard).
    text: Vec<String>,

    /// Читать из/писать в буфер обмена.
    #[arg(short, long)]
    clipboard: bool,

    /// Принудительно использовать LLM (даже если словарь дал хороший результат).
    #[arg(short, long)]
    smart: bool,

    /// Не использовать LLM ни при каких условиях.
    #[arg(long)]
    no_llm: bool,

    /// Legacy option: сохранён для совместимости, в простом режиме LLM не включается автоматически.
    #[arg(long, default_value_t = 0.7)]
    threshold: f32,

    /// Печатать какой метод сработал.
    #[arg(short, long)]
    verbose: bool,

    /// Объяснить решение автокоррекции after-space для текста.
    #[arg(long)]
    explain_correct: bool,

    /// Восстановить одно испорченное слово через общий correction core.
    #[arg(long)]
    restore_word: bool,

    /// Явно выбрать route для correction core explain/restore.
    #[arg(long, value_enum)]
    candidate_route: Option<CandidateRouteArg>,

    /// Сравнить `full-wave` против выбранного route на одном и том же входе.
    #[arg(long)]
    compare_candidate_routes: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum CandidateRouteArg {
    #[value(alias = "l2-field-shadow")]
    CanonicalL2Field,
    FullWave,
}

impl CandidateRouteArg {
    fn into_route(self) -> correction_core::CandidateReadoutRoute {
        match self {
            Self::CanonicalL2Field => correction_core::CandidateReadoutRoute::CanonicalL2Field,
            Self::FullWave => correction_core::CandidateReadoutRoute::FullWave,
        }
    }
}

fn main() {
    let args = Args::parse();

    let text = if args.clipboard {
        match read_clipboard() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("⚠ не удалось прочитать буфер: {e}");
                process::exit(1);
            }
        }
    } else if !args.text.is_empty() {
        args.text.join(" ")
    } else if io::stdin().is_terminal() {
        eprintln!("Использование: lay <текст>  |  lay --clipboard  |  echo '...' | lay");
        process::exit(1);
    } else {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s).ok();
        s
    };

    if text.trim().is_empty() {
        process::exit(0);
    }

    if args.explain_correct {
        let cfg = config::LayConfig::load();
        let route = selected_candidate_route(
            &args,
            correction_core::CandidateReadoutRoute::live_default(),
        );
        let explanation = explain_typing_assist_like_runtime(&text, &cfg);
        print_typing_explanation(&explanation);
        print_nanda_explanation(&text, &cfg, route);
        print_correction_core_explanation(&text, &cfg, route);
        return;
    }

    if args.compare_candidate_routes {
        let cfg = config::LayConfig::load();
        let target_route = selected_candidate_route(
            &args,
            correction_core::CandidateReadoutRoute::live_default(),
        );
        let report = compare_candidate_routes(&text, &cfg, target_route);
        println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("serialize compare report")
        );
        return;
    }

    if args.restore_word {
        let cfg = config::LayConfig::load();
        let route = selected_candidate_route(
            &args,
            correction_core::CandidateReadoutRoute::live_default(),
        );
        let words = text
            .lines()
            .map(str::trim)
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>();
        if words.is_empty()
            || words
                .iter()
                .any(|word| word.split_whitespace().count() != 1)
        {
            eprintln!("--restore-word принимает одно слово или поток из одного слова на строку");
            process::exit(2);
        }

        let mut restored_words = Vec::with_capacity(words.len());
        for word in words {
            let (restored, source) = restore_word(word, &cfg, route);
            if args.verbose {
                eprintln!("[{source}] {word:?} -> {restored:?}");
            }
            restored_words.push(restored);
        }
        print!("{}", restored_words.join("\n"));
        return;
    }

    let (result, method) = convert(&text, &args);

    if args.clipboard {
        if let Err(e) = write_clipboard(&result) {
            eprintln!("⚠ не удалось записать в буфер: {e}");
            process::exit(1);
        }
        if args.verbose {
            eprintln!("[{method}] {text:?} → {result:?}");
        } else {
            eprintln!("✓ в буфере обмена ({method})");
        }
    } else {
        if args.verbose {
            eprintln!("[{method}]");
        }
        print!("{result}");
        if text.ends_with('\n') && !result.ends_with('\n') {
            println!();
        }
    }
}

fn explain_typing_assist_like_runtime(
    text: &str,
    cfg: &config::LayConfig,
) -> typing_assist::TypingAssistExplanation {
    let pipeline = typing_context::typing_assist_pipeline_for_context(
        cfg.auto_replace,
        active_typing_safety(cfg),
        &cfg.typing_assist_pipeline,
        text,
    );
    let whole = explain_typing_assist_runtime(text, cfg, &pipeline);
    if whole.output.is_some() {
        return whole;
    }

    for word_count in [2, 1] {
        if let Some(explanation) = explain_completed_tail(text, cfg, word_count) {
            return explanation;
        }
    }

    whole
}

fn explain_completed_tail(
    text: &str,
    cfg: &config::LayConfig,
    word_count: usize,
) -> Option<typing_assist::TypingAssistExplanation> {
    let (leading, core, trailing) = typing_assist::split_edge_whitespace(text);
    let segments = typing_assist::split_ws_segments(core);
    if segments.len() < 3 {
        return None;
    }

    let mut suffix_start = core.len();
    let mut non_ws_seen = 0;
    for (segment, is_ws) in segments.iter().rev() {
        suffix_start -= segment.len();
        if !is_ws {
            non_ws_seen += 1;
            if non_ws_seen == word_count {
                break;
            }
        }
    }
    if non_ws_seen != word_count {
        return None;
    }

    let mut suffix = String::new();
    suffix.push_str(leading);
    suffix.push_str(&core[suffix_start..]);
    suffix.push_str(trailing);

    let pipeline = typing_context::typing_assist_pipeline_for_context(
        cfg.auto_replace,
        active_typing_safety(cfg),
        &cfg.typing_assist_pipeline,
        text,
    );
    let explanation = explain_typing_assist_runtime(&suffix, cfg, &pipeline);
    explanation.output.is_some().then_some(explanation)
}

fn active_typing_safety(cfg: &config::LayConfig) -> config::CorrectionSafety {
    cfg.active_correction_safety()
}

fn selected_candidate_route(
    args: &Args,
    default: correction_core::CandidateReadoutRoute,
) -> correction_core::CandidateReadoutRoute {
    args.candidate_route
        .map(CandidateRouteArg::into_route)
        .unwrap_or(default)
}

fn compare_candidate_routes(
    text: &str,
    cfg: &config::LayConfig,
    target_route: correction_core::CandidateReadoutRoute,
) -> serde_json::Value {
    let inputs = comparison_inputs(text);
    let samples = inputs
        .iter()
        .map(|input| compare_candidate_routes_for_input(input, cfg, target_route))
        .collect::<Vec<_>>();
    let surface_diverged = samples
        .iter()
        .filter(|sample| sample["selected_surface_diverged"].as_bool() == Some(true))
        .count();
    let gate_diverged = samples
        .iter()
        .filter(|sample| sample["selected_gate_diverged"].as_bool() == Some(true))
        .count();
    let provenance_diverged = samples
        .iter()
        .filter(|sample| sample["selected_provenance_diverged"].as_bool() == Some(true))
        .count();
    let surface_identical = samples.len().saturating_sub(surface_diverged);
    serde_json::json!({
        "kind": "candidate_route_compare",
        "reference_route": route_name(correction_core::CandidateReadoutRoute::compare_reference()),
        "target_route": route_name(target_route),
        "sample_count": samples.len(),
        "selected_surface_diverged": surface_diverged,
        "selected_surface_identical": surface_identical,
        "selected_gate_diverged": gate_diverged,
        "selected_provenance_diverged": provenance_diverged,
        "samples": samples,
    })
}

fn comparison_inputs(text: &str) -> Vec<String> {
    let lines = text
        .lines()
        .map(str::to_string)
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    if lines.is_empty() {
        vec![text.to_string()]
    } else if lines.len() == 1 && !text.contains('\n') {
        vec![text.to_string()]
    } else {
        lines
    }
}

fn compare_candidate_routes_for_input(
    input: &str,
    cfg: &config::LayConfig,
    target_route: correction_core::CandidateReadoutRoute,
) -> serde_json::Value {
    let compact = resolve_with_route(
        input,
        cfg,
        correction_core::CandidateReadoutRoute::compare_reference(),
    );
    let target = resolve_with_route(input, cfg, target_route);
    let compact_selected = compact.selected.as_ref();
    let target_selected = target.selected.as_ref();
    let selected_surface_diverged = selected_surface_diverged(compact_selected, target_selected);
    let selected_gate_diverged = selected_gate_diverged(compact_selected, target_selected);
    let selected_provenance_diverged =
        selected_provenance_diverged(compact_selected, target_selected);
    serde_json::json!({
        "input": input,
        "input_class": compact.event.input_class.as_str(),
        "selected_surface_diverged": selected_surface_diverged,
        "selected_gate_diverged": selected_gate_diverged,
        "selected_provenance_diverged": selected_provenance_diverged,
        "reference": resolution_summary_json(
            correction_core::CandidateReadoutRoute::compare_reference(),
            &compact,
        ),
        "target": resolution_summary_json(target_route, &target),
    })
}

fn selected_surface_diverged(
    left: Option<&correction_core::UnifiedCorrectionCandidate>,
    right: Option<&correction_core::UnifiedCorrectionCandidate>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.replacement != right.replacement,
        (None, None) => false,
        _ => true,
    }
}

fn selected_gate_diverged(
    left: Option<&correction_core::UnifiedCorrectionCandidate>,
    right: Option<&correction_core::UnifiedCorrectionCandidate>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            left.gate.action != right.gate.action
                || left.gate.reason != right.gate.reason
                || left.error_class != right.error_class
        }
        (None, None) => false,
        _ => true,
    }
}

fn selected_provenance_diverged(
    left: Option<&correction_core::UnifiedCorrectionCandidate>,
    right: Option<&correction_core::UnifiedCorrectionCandidate>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            left.source != right.source || left.source_id != right.source_id
        }
        (None, None) => false,
        _ => true,
    }
}

fn restore_word(
    word: &str,
    cfg: &config::LayConfig,
    route: correction_core::CandidateReadoutRoute,
) -> (String, &'static str) {
    let completed = format!("{word} ");
    let resolution = correction_core::resolve_text_correction(correction_core::CorrectionRequest {
        text: &completed,
        auto_replace: true,
        typing_assist: true,
        auto_switch_layout: true,
        correction_safety: active_typing_safety(cfg),
        typing_assist_pipeline: &cfg.typing_assist_pipeline,
        nanda_autocorrect: true,
        nanda_candidate_route: route,
        nanda_wave_options: cfg.active_nanda_wave_options(),
        mode: correction_core::CorrectionMode::DeterministicThenNanda,
    });

    match resolution.selected {
        Some(candidate) => (candidate.replacement.trim().to_string(), "correction-core"),
        None => (word.to_string(), "keep"),
    }
}

fn resolve_with_route(
    text: &str,
    cfg: &config::LayConfig,
    route: correction_core::CandidateReadoutRoute,
) -> correction_core::CorrectionResolution {
    correction_core::resolve_text_correction(correction_core::CorrectionRequest {
        text,
        auto_replace: true,
        typing_assist: true,
        auto_switch_layout: true,
        correction_safety: active_typing_safety(cfg),
        typing_assist_pipeline: &cfg.typing_assist_pipeline,
        nanda_autocorrect: true,
        nanda_candidate_route: route,
        nanda_wave_options: cfg.active_nanda_wave_options(),
        mode: correction_core::CorrectionMode::DeterministicThenNanda,
    })
}

fn resolution_summary_json(
    route: correction_core::CandidateReadoutRoute,
    resolution: &correction_core::CorrectionResolution,
) -> serde_json::Value {
    serde_json::json!({
        "route": route_name(route),
        "candidate_count": resolution.candidates.len(),
        "scoreboard": {
            "total_candidates": resolution.scoreboard.total_candidates,
            "apply_candidates": resolution.scoreboard.apply_candidates,
            "suggest_only_candidates": resolution.scoreboard.suggest_only_candidates,
            "keep_original_candidates": resolution.scoreboard.keep_original_candidates,
            "veto_candidates": resolution.scoreboard.veto_candidates,
        },
        "selected": resolution.selected.as_ref().map(candidate_summary_json),
        "candidates": resolution
            .candidates
            .iter()
            .map(candidate_summary_json)
            .collect::<Vec<_>>(),
    })
}

fn candidate_summary_json(
    candidate: &correction_core::UnifiedCorrectionCandidate,
) -> serde_json::Value {
    serde_json::json!({
        "replacement": candidate.replacement,
        "source": format!("{:?}", candidate.source),
        "source_id": candidate.source_id,
        "error_class": candidate.error_class.as_str(),
        "gate_action": format!("{:?}", candidate.gate.action),
        "gate_reason": candidate.gate.reason,
    })
}

fn route_name(route: correction_core::CandidateReadoutRoute) -> &'static str {
    match route {
        correction_core::CandidateReadoutRoute::CanonicalL2Field => "canonical-l2-field",
        correction_core::CandidateReadoutRoute::FullWave => "full-wave",
    }
}

fn explain_typing_assist_runtime(
    text: &str,
    cfg: &config::LayConfig,
    pipeline: &[config::TypingAssistRuleConfig],
) -> typing_assist::TypingAssistExplanation {
    typing_assist::explain_typing_assist_with_pipeline(text, cfg.auto_switch_layout, pipeline)
}

fn print_typing_explanation(explanation: &typing_assist::TypingAssistExplanation) {
    println!("original: {:?}", explanation.original);
    println!("core: {:?}", explanation.core);
    println!("allow_layout_auto: {}", explanation.allow_layout_auto);
    println!("rules:");

    for eval in &explanation.evaluations {
        let status = if !eval.enabled {
            "disabled".to_string()
        } else if let Some(reason) = &eval.rejected {
            format!("rejected: {reason}")
        } else {
            "accepted".to_string()
        };

        if let Some(candidate) = &eval.candidate {
            println!(
                "  {} priority={} {} -> {:?} score={:.3}",
                eval.id, eval.priority, status, candidate.replacement, candidate.score.total
            );
        } else {
            println!("  {} priority={} {}", eval.id, eval.priority, status);
        }
    }

    match &explanation.chosen {
        Some(chosen) => println!(
            "chosen: {} -> {:?} score={:.3}",
            chosen.rule_id, chosen.replacement, chosen.score.total
        ),
        None => println!("chosen: none"),
    }
    match (&explanation.second, explanation.margin) {
        (Some(second), Some(margin)) => println!(
            "second: {} -> {:?} score={:.3} margin={:.3}",
            second.rule_id, second.replacement, second.score.total, margin
        ),
        (None, Some(margin)) => println!("second: none margin={margin:.3}"),
        _ => {}
    }
    if let Some(confidence) = explanation.confidence(1.0) {
        println!("confidence: {confidence:?}");
    }
    match &explanation.output {
        Some(output) => println!("output: {:?}", output),
        None => println!("output: none"),
    }
}

fn print_nanda_explanation(
    text: &str,
    cfg: &config::LayConfig,
    route: correction_core::CandidateReadoutRoute,
) {
    if !cfg.nanda_autocorrect {
        println!("nanda: disabled");
        return;
    }
    let resolution = correction_core::resolve_text_correction(correction_core::CorrectionRequest {
        text,
        auto_replace: cfg.auto_replace,
        typing_assist: cfg.typing_assist,
        auto_switch_layout: cfg.auto_switch_layout,
        correction_safety: active_typing_safety(cfg),
        typing_assist_pipeline: &cfg.typing_assist_pipeline,
        nanda_autocorrect: cfg.nanda_autocorrect,
        nanda_candidate_route: route,
        nanda_wave_options: cfg.active_nanda_wave_options(),
        mode: correction_core::CorrectionMode::NandaOnly,
    });
    println!(
        "nanda: candidates={} apply={} suggest={} veto={}",
        resolution.scoreboard.total_candidates,
        resolution.scoreboard.apply_candidates,
        resolution.scoreboard.suggest_only_candidates,
        resolution.scoreboard.veto_candidates
    );
    match &resolution.selected {
        Some(candidate) => println!(
            "nanda: transition-apply {:?} source={} gate={:?}/{}",
            candidate.replacement,
            candidate.source_id,
            candidate.gate.action,
            candidate.gate.reason
        ),
        None => println!("nanda: transition-keep"),
    }
}

fn print_correction_core_explanation(
    text: &str,
    cfg: &config::LayConfig,
    route: correction_core::CandidateReadoutRoute,
) {
    let resolution = correction_core::resolve_text_correction(correction_core::CorrectionRequest {
        text,
        auto_replace: cfg.auto_replace,
        typing_assist: cfg.typing_assist,
        auto_switch_layout: cfg.auto_switch_layout,
        correction_safety: active_typing_safety(cfg),
        typing_assist_pipeline: &cfg.typing_assist_pipeline,
        nanda_autocorrect: cfg.nanda_autocorrect,
        nanda_candidate_route: route,
        nanda_wave_options: cfg.active_nanda_wave_options(),
        mode: correction_core::CorrectionMode::DeterministicThenNanda,
    });

    println!("correction_core:");
    println!(
        "  input_class={} word={:?}",
        resolution.event.input_class.as_str(),
        resolution.event.current_word
    );
    if resolution.candidates.is_empty() {
        println!("  candidates: none");
    } else {
        println!("  candidates:");
        for candidate in &resolution.candidates {
            println!(
                "    source={:?}:{} class={} gate={:?}/{} -> {:?}",
                candidate.source,
                candidate.source_id,
                candidate.error_class.as_str(),
                candidate.gate.action,
                candidate.gate.reason,
                candidate.replacement
            );
        }
    }
    match &resolution.selected {
        Some(candidate) => println!(
            "  selected: {:?}:{} -> {:?}",
            candidate.source, candidate.source_id, candidate.replacement
        ),
        None => println!("  selected: none"),
    }
}

fn convert(text: &str, args: &Args) -> (String, &'static str) {
    let direction = dict::detect_direction(text);
    let dict_result = dict::convert(text, direction);

    if args.no_llm {
        return (dict_result, "dict");
    }

    if args.smart {
        return match llm::convert_hybrid(text, &dict_result) {
            Ok(Some(result)) => (result, "llm-hybrid"),
            _ => (dict_result, "dict-fallback"),
        };
    }

    (dict_result, "dict")
}

fn read_clipboard() -> Result<String, Box<dyn std::error::Error>> {
    let mut cb = arboard::Clipboard::new()?;
    Ok(cb.get_text()?)
}

fn write_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut cb = arboard::Clipboard::new()?;
    cb.set_text(text.to_string())?;
    Ok(())
}
