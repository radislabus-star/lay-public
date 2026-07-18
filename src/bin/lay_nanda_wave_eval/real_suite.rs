use lay::dict::{convert, Direction};
use lay::eval_cases::EvalCase;
use std::fs;
use std::io;
use std::path::Path;

pub(crate) struct RealSuite {
    pub(crate) cases: Vec<EvalCase>,
    pub(crate) sources: Vec<RealSuiteSource>,
}

pub(crate) struct RealSuiteSource {
    pub(crate) path: &'static str,
    pub(crate) cases: usize,
}

pub(crate) fn load() -> io::Result<RealSuite> {
    let mut suite = RealSuite {
        cases: Vec::new(),
        sources: Vec::new(),
    };
    add_three_col(
        &mut suite,
        "data/neural_arbiter/holdout.tsv",
        "neural_holdout",
    )?;
    add_three_col(
        &mut suite,
        "data/neural_arbiter/dataset.tsv",
        "training_groups",
    )?;
    add_two_col(
        &mut suite,
        "tests/fixtures/daemon_typing_assist_layout_explicit.tsv",
        "layout_explicit",
    )?;
    add_two_col(
        &mut suite,
        "tests/fixtures/daemon_typing_assist_glued_split_fix.tsv",
        "split_glued_phrase",
    )?;
    add_two_col(
        &mut suite,
        "tests/fixtures/daemon_typing_assist_regression_fix.tsv",
        "ru_typo",
    )?;
    add_two_col(
        &mut suite,
        "tests/fixtures/daemon_typing_assist_missing_letter_core.tsv",
        "ru_typo_missing_letter",
    )?;
    add_two_col(
        &mut suite,
        "tests/fixtures/daemon_typing_assist_repeated_letter.tsv",
        "ru_typo_repeated_letter",
    )?;
    add_two_col(
        &mut suite,
        "tests/fixtures/daemon_typing_assist_hard_sign_fix.tsv",
        "ru_typo_hard_sign",
    )?;
    add_two_col(
        &mut suite,
        "tests/fixtures/daemon_typing_assist_transposition.tsv",
        "ru_typo_transposition",
    )?;
    add_two_col(
        &mut suite,
        "tests/fixtures/nanda_l3_context_heldout.tsv",
        "l3_context_heldout",
    )?;
    add_keep_lines(
        &mut suite,
        "tests/fixtures/daemon_typing_assist_missing_letter_keep.txt",
        "ru_typo_keep",
    )?;
    add_keep_lines(
        &mut suite,
        "tests/fixtures/daemon_typing_assist_repeated_letter_keep.txt",
        "ru_typo_keep",
    )?;
    add_keep_lines(
        &mut suite,
        "tests/fixtures/daemon_typing_assist_valid_word_keep.txt",
        "ru_typo_keep",
    )?;
    add_keep_lines(
        &mut suite,
        "tests/fixtures/typing_assist_autocorrect_keep.txt",
        "autocorrect_keep",
    )?;
    add_two_col(
        &mut suite,
        "tests/fixtures/typing_assist_live_spacing.tsv",
        "live_spacing",
    )?;
    add_keep_lines(
        &mut suite,
        "tests/fixtures/typing_assist_shell_keep.txt",
        "shell_keep",
    )?;
    add_keep_lines(
        &mut suite,
        "tests/fixtures/typing_assist_cli_commands.txt",
        "technical_keep",
    )?;
    add_short_alternating(
        &mut suite,
        "tests/fixtures/typing_assist_short_alternating_pairs.tsv",
    )?;
    add_ru_to_en_synthetic(
        &mut suite,
        "tests/fixtures/typing_assist_ru_to_en_synthetic.txt",
    )?;
    add_three_col(
        &mut suite,
        "data/nanda_wave_synthetic_cases.tsv",
        "nanda_wave_synthetic",
    )?;
    add_three_col(
        &mut suite,
        "data/nanda_training/generated_cases.tsv",
        "nanda_generated_training",
    )?;
    Ok(suite)
}

fn add_three_col(
    suite: &mut RealSuite,
    path: &'static str,
    fallback_reason: &str,
) -> io::Result<()> {
    let before = suite.cases.len();
    for line in read_lines(path)? {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() >= 8 {
            add_grouped_positive(suite, &cols);
        } else if cols.len() >= 3 {
            suite.cases.push(EvalCase {
                original: decode(cols[0]),
                expected: decode(cols[1]),
                reason: cols.get(2).copied().unwrap_or(fallback_reason).to_string(),
            });
        }
    }
    record_source(suite, path, before);
    Ok(())
}

fn add_grouped_positive(suite: &mut RealSuite, cols: &[&str]) {
    if cols.get(5) != Some(&"1") {
        return;
    }
    let reason = match cols.get(4).copied().unwrap_or_default() {
        "layout" => "layout",
        "split" => "split_glued_phrase",
        "typo" => "ru_typo",
        "mixed" => "mixed_context",
        "keep" => "keep",
        _ => "training_group",
    };
    suite.cases.push(EvalCase {
        original: cols.get(2).copied().unwrap_or_default().to_string(),
        expected: cols.get(3).copied().unwrap_or_default().to_string(),
        reason: reason.to_string(),
    });
}

fn add_two_col(suite: &mut RealSuite, path: &'static str, reason: &str) -> io::Result<()> {
    let before = suite.cases.len();
    for line in read_lines(path)? {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 2 {
            continue;
        }
        suite.cases.push(EvalCase {
            original: decode(cols[0]),
            expected: decode(cols[1]),
            reason: reason.to_string(),
        });
    }
    record_source(suite, path, before);
    Ok(())
}

fn add_keep_lines(suite: &mut RealSuite, path: &'static str, reason: &str) -> io::Result<()> {
    let before = suite.cases.len();
    for line in read_lines(path)? {
        let text = decode(&line);
        suite.cases.push(EvalCase {
            original: text.clone(),
            expected: text,
            reason: reason.to_string(),
        });
    }
    record_source(suite, path, before);
    Ok(())
}

fn add_short_alternating(suite: &mut RealSuite, path: &'static str) -> io::Result<()> {
    let before = suite.cases.len();
    let pairs = read_lines(path)?
        .into_iter()
        .filter_map(|line| {
            let cols: Vec<&str> = line.split('\t').collect();
            (cols.len() >= 2).then(|| (cols[0].to_string(), cols[1].to_string()))
        })
        .collect::<Vec<_>>();
    let expected = pairs
        .iter()
        .flat_map(|(ru, en)| [ru.as_str(), en.as_str()])
        .collect::<Vec<_>>()
        .join(" ")
        + " ";
    let original_ru_wrong_us = pairs
        .iter()
        .flat_map(|(ru, en)| [convert(ru, Direction::Ru2Us), en.clone()])
        .collect::<Vec<_>>()
        .join(" ")
        + " ";
    let original_en_wrong_ru = pairs
        .iter()
        .flat_map(|(ru, en)| [ru.clone(), convert(en, Direction::Us2Ru)])
        .collect::<Vec<_>>()
        .join(" ")
        + " ";
    let original_all_wrong = pairs
        .iter()
        .flat_map(|(ru, en)| [convert(ru, Direction::Ru2Us), convert(en, Direction::Us2Ru)])
        .collect::<Vec<_>>()
        .join(" ")
        + " ";
    for (reason, original) in [
        ("short_alternating_ru_wrong", original_ru_wrong_us),
        ("short_alternating_en_wrong", original_en_wrong_ru),
        ("short_alternating_all_wrong", original_all_wrong),
    ] {
        suite.cases.push(EvalCase {
            original,
            expected: expected.clone(),
            reason: reason.to_string(),
        });
    }
    record_source(suite, path, before);
    Ok(())
}

fn add_ru_to_en_synthetic(suite: &mut RealSuite, path: &'static str) -> io::Result<()> {
    let before = suite.cases.len();
    for token in read_lines(path)? {
        suite.cases.push(EvalCase {
            original: convert(&token, Direction::Us2Ru) + " ",
            expected: token + " ",
            reason: "synthetic_ru_to_en_technical".to_string(),
        });
    }
    record_source(suite, path, before);
    Ok(())
}

fn read_lines(path: &str) -> io::Result<Vec<String>> {
    let text = fs::read_to_string(Path::new(path))?;
    Ok(text
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .map(ToOwned::to_owned)
        .collect())
}

fn record_source(suite: &mut RealSuite, path: &'static str, before: usize) {
    suite.sources.push(RealSuiteSource {
        path,
        cases: suite.cases.len().saturating_sub(before),
    });
}

fn decode(value: &str) -> String {
    value.replace("\\s", " ")
}
