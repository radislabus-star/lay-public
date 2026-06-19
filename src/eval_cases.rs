use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalCase {
    pub original: String,
    pub expected: String,
    pub reason: String,
}

pub fn read_cases(path: &Path) -> io::Result<Vec<EvalCase>> {
    let text = fs::read_to_string(path)?;
    if text
        .lines()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| line.starts_with("group_id\t"))
    {
        return Ok(read_grouped_training_cases(&text));
    }
    let mut cases = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 3 {
            continue;
        }
        cases.push(EvalCase {
            original: decode_fixture(cols[0]),
            expected: decode_fixture(cols[1]),
            reason: cols[2].to_string(),
        });
    }
    Ok(cases)
}

fn read_grouped_training_cases(text: &str) -> Vec<EvalCase> {
    let mut cases = Vec::new();
    let mut current_group = String::new();
    let mut current_original = String::new();
    let mut current_expected = String::new();
    let mut current_reason = String::new();

    for (idx, line) in text.lines().enumerate() {
        if idx == 0 || line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 8 {
            continue;
        }
        let group_id = cols[0];
        if !current_group.is_empty() && current_group != group_id {
            push_grouped_case(
                &mut cases,
                &current_original,
                &current_expected,
                &current_reason,
            );
            current_original.clear();
            current_expected.clear();
            current_reason.clear();
        }
        current_group = group_id.to_string();
        if current_original.is_empty() {
            current_original = cols[2].to_string();
        }
        if cols[5] == "1" {
            current_expected = cols[3].to_string();
            current_reason = grouped_reason(cols[4], cols[7]).to_string();
        }
    }
    push_grouped_case(
        &mut cases,
        &current_original,
        &current_expected,
        &current_reason,
    );
    cases
}

fn push_grouped_case(cases: &mut Vec<EvalCase>, original: &str, expected: &str, reason: &str) {
    if original.is_empty() || expected.is_empty() || reason.is_empty() {
        return;
    }
    cases.push(EvalCase {
        original: original.to_string(),
        expected: expected.to_string(),
        reason: reason.to_string(),
    });
}

fn grouped_reason(operation: &str, raw_reason: &str) -> &'static str {
    match operation {
        "layout" => "layout",
        "split" => "split_glued_phrase",
        "typo" => "ru_typo",
        "mixed" => "mixed_context",
        "keep" if raw_reason.contains("technical") => "technical_keep",
        "keep" => "keep",
        _ => "other",
    }
}

fn decode_fixture(value: &str) -> String {
    value.replace("\\s", " ")
}
