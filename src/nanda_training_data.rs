use crate::dict;
use crate::microbrain::Expert64TrainingRow;
use crate::text_metrics::without_whitespace;
use std::fs;
use std::io;
use std::path::Path;

pub fn read_training_rows(path: &Path) -> io::Result<Vec<Expert64TrainingRow>> {
    let mut rows = Vec::new();
    let text = fs::read_to_string(path)?;
    let first_data = text
        .lines()
        .find(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .unwrap_or_default();
    if first_data.starts_with("group_id\t") {
        read_labeled_rows(&text, &mut rows);
    } else {
        read_fixture_rows(path, &text, &mut rows);
    }
    rows.sort_by(|left, right| left.group_id.cmp(&right.group_id));
    Ok(rows)
}

pub fn add_training_group(
    rows: &mut Vec<Expert64TrainingRow>,
    group_id: &str,
    original: &str,
    expected: &str,
) {
    rows.push(Expert64TrainingRow {
        group_id: group_id.to_string(),
        original: original.to_string(),
        candidate: original.to_string(),
        operation: "keep".to_string(),
        label: original == expected,
    });
    if original != expected {
        rows.push(Expert64TrainingRow {
            group_id: group_id.to_string(),
            original: original.to_string(),
            candidate: expected.to_string(),
            operation: operation_for(original, expected).to_string(),
            label: true,
        });
    }
    let flipped = dict::convert(original, dict::detect_direction(original));
    if flipped != original && flipped != expected {
        rows.push(Expert64TrainingRow {
            group_id: group_id.to_string(),
            original: original.to_string(),
            candidate: flipped,
            operation: "layout".to_string(),
            label: false,
        });
    }
}

pub fn operation_for(original: &str, candidate: &str) -> &'static str {
    if original == candidate {
        "keep"
    } else if dict::convert(original, dict::detect_direction(original)) == candidate {
        "layout"
    } else if without_whitespace(original) == without_whitespace(candidate) {
        "split"
    } else {
        "typo"
    }
}

fn read_labeled_rows(text: &str, rows: &mut Vec<Expert64TrainingRow>) {
    for (idx, line) in text.lines().enumerate() {
        if idx == 0 || line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 8 {
            continue;
        }
        rows.push(Expert64TrainingRow {
            group_id: cols[0].to_string(),
            original: cols[2].to_string(),
            candidate: cols[3].to_string(),
            operation: cols[4].to_string(),
            label: cols[5] == "1",
        });
    }
}

fn read_fixture_rows(path: &Path, text: &str, rows: &mut Vec<Expert64TrainingRow>) {
    for (idx, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 2 {
            continue;
        }
        let original = decode_fixture(cols[0]);
        let expected = decode_fixture(cols[1]);
        add_training_group(
            rows,
            &format!("{}:{idx}", path.display()),
            &original,
            &expected,
        );
    }
}

fn decode_fixture(value: &str) -> String {
    value.replace("\\s", " ")
}
