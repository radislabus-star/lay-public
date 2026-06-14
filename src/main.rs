//! lay — Caramba/Punto-style конвертер раскладки клавиатуры.
//!
//! Двухрежимная логика:
//! 1. Словарная конвертация US ↔ RU (микросекунды, детерминированно).
//! 2. Гибридный smart/model-режим включается явно через `--smart`.

use clap::Parser;
use std::io::{self, IsTerminal, Read};
use std::process;

use lay::{config, dict, llm, microbrain, nanda_profile, typing_assist, typing_context};

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

    /// Включить экспериментальный microbrain/NANDA scorer для --explain-correct.
    #[arg(long)]
    microbrain: bool,

    /// Отключить microbrain-эксперта для ablation, можно указать несколько раз.
    #[arg(long)]
    disable_expert: Vec<String>,

    /// Показать CPU/cache профиль для NANDA Expert64.
    #[arg(long)]
    nanda_profile: bool,

    /// Показать подробное состояние NANDA Expert64.
    #[arg(long)]
    nanda_status: bool,
}

fn main() {
    let args = Args::parse();

    if args.nanda_profile {
        println!(
            "{}",
            nanda_profile::NandaCpuProfile::detect().compact_text()
        );
        return;
    }
    if args.nanda_status {
        println!("{}", microbrain::nanda_status_text());
        return;
    }

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
        let mut cfg = config::LayConfig::load();
        if args.microbrain {
            cfg.microbrain = true;
        }
        let explanation = explain_typing_assist_like_runtime(&text, &cfg, &args.disable_expert);
        print_typing_explanation(&explanation);
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
    disabled_experts: &[String],
) -> typing_assist::TypingAssistExplanation {
    let pipeline = typing_context::typing_assist_pipeline_for_context(
        cfg.auto_replace,
        active_typing_safety(cfg),
        &cfg.typing_assist_pipeline,
        text,
    );
    let whole = explain_typing_assist_runtime(text, cfg, &pipeline, disabled_experts);
    if whole.output.is_some() {
        return whole;
    }

    for word_count in [2, 1] {
        if let Some(explanation) = explain_completed_tail(text, cfg, word_count, disabled_experts) {
            return explanation;
        }
    }

    whole
}

fn explain_completed_tail(
    text: &str,
    cfg: &config::LayConfig,
    word_count: usize,
    disabled_experts: &[String],
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
    let explanation = explain_typing_assist_runtime(&suffix, cfg, &pipeline, disabled_experts);
    explanation.output.is_some().then_some(explanation)
}

fn active_typing_safety(cfg: &config::LayConfig) -> config::CorrectionSafety {
    if (cfg.microbrain || cfg.nanda_autocorrect) && cfg.auto_replace {
        config::CorrectionSafety::Experimental
    } else {
        cfg.active_correction_safety()
    }
}

fn explain_typing_assist_runtime(
    text: &str,
    cfg: &config::LayConfig,
    pipeline: &[config::TypingAssistRuleConfig],
    disabled_experts: &[String],
) -> typing_assist::TypingAssistExplanation {
    if cfg.microbrain || cfg.nanda_autocorrect {
        let options = microbrain::MicrobrainOptions::with_disabled(disabled_experts);
        typing_assist::explain_typing_assist_with_microbrain_options(
            text,
            cfg.auto_switch_layout,
            pipeline,
            &options,
        )
    } else {
        typing_assist::explain_typing_assist_with_pipeline(text, cfg.auto_switch_layout, pipeline)
    }
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
    if let Some(trace) = &explanation.microbrain {
        println!("microbrain:");
        println!("  chosen: {:?}", trace.chosen);
        println!("  no_raw_secret_text: {}", trace.no_raw_secret_text);
        if !trace.disabled_experts.is_empty() {
            println!("  disabled_experts: {:?}", trace.disabled_experts);
        }
        if !trace.generated.is_empty() {
            println!("  generated:");
            for generated in &trace.generated {
                println!(
                    "    {:?} source={} action={:?} reason={}",
                    generated.text, generated.source, generated.action, generated.reason_code
                );
            }
        }
        for candidate in &trace.candidates {
            println!(
                "  candidate {:?} source={} action={:?} confidence={:.3}",
                candidate.candidate, candidate.source, candidate.action, candidate.confidence
            );
            for score in &candidate.expert_scores {
                println!(
                    "    {} {:.3} {}",
                    score.expert, score.confidence, score.reason_code
                );
            }
            for tick in &candidate.mesh_ticks {
                println!(
                    "    mesh tick={} confidence={:.3} coherence={:.3} reason={}",
                    tick.tick, tick.confidence, tick.board.coherence, tick.reason_code
                );
            }
        }
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
