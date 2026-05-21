//! lay — Caramba/Punto-style конвертер раскладки клавиатуры.
//!
//! Двухрежимная логика:
//! 1. Словарная конвертация US ↔ RU (микросекунды, детерминированно).
//! 2. Гибридный smart/model-режим включается явно через `--smart`.

use clap::Parser;
use std::io::{self, IsTerminal, Read};
use std::process;

use lay::{config, dict, llm, typing_assist};

#[derive(Parser, Debug)]
#[command(
    name = "lay",
    version,
    about = "Layout switcher: 'Ye djn ghbvth' → 'Ну вот пример'"
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
        let pipeline = config::typing_assist_pipeline_for_policy(
            cfg.auto_replace,
            cfg.active_correction_safety(),
            &cfg.typing_assist_pipeline,
        );
        let explanation = typing_assist::explain_typing_assist_with_pipeline(
            &text,
            cfg.auto_switch_layout,
            &pipeline,
        );
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
    match &explanation.output {
        Some(output) => println!("output: {:?}", output),
        None => println!("output: none"),
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
