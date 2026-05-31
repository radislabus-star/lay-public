use std::collections::HashMap;
use std::path::PathBuf;

use crate::data_lines::data_lines;
use crate::text_edit::TextReplacement;

const TEST_RUSSIAN_FORMS_DATA: &str = include_str!("../tests/fixtures/russian_forms.txt");
const TEST_REPLACEMENT_RULES_DATA: &str = include_str!("../tests/fixtures/replacement_rules.tsv");

pub fn russian_forms() -> impl Iterator<Item = &'static str> {
    data_lines(TEST_RUSSIAN_FORMS_DATA)
}

pub fn replacement_rules() -> HashMap<String, String> {
    data_lines(TEST_REPLACEMENT_RULES_DATA)
        .filter_map(|line| line.split_once('\t'))
        .map(|(from, to)| (from.to_string(), to.to_string()))
        .collect()
}

pub fn fixture_rows(name: &str) -> Vec<Vec<String>> {
    let path = fixture_path(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture {path:?}: {err}"));
    fixture_rows_from_str(&text)
}

pub fn fixture_row_by_id(name: &str, id: &str) -> Vec<String> {
    fixture_rows(name)
        .into_iter()
        .find(|row| row.first().is_some_and(|value| value == id))
        .unwrap_or_else(|| panic!("missing fixture row {id:?} in {name:?}"))
}

pub fn first_fixture_row(name: &str) -> Vec<String> {
    fixture_rows(name)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing fixture row in {name:?}"))
}

pub fn single_fixture_row(name: &str, width: usize) -> Vec<String> {
    let rows = fixture_rows(name);
    assert_eq!(rows.len(), 1, "fixture must contain one row: {name}");
    let row = rows.into_iter().next().expect("fixture row");
    assert_eq!(row.len(), width, "fixture row width mismatch: {name}");
    row
}

pub fn fixture_row_by_id_from_str(data: &str, id: &str) -> Vec<String> {
    fixture_rows_from_str(data)
        .into_iter()
        .find(|row| row.first().is_some_and(|value| value == id))
        .unwrap_or_else(|| panic!("missing fixture row {id:?}"))
}

pub fn first_fixture_row_from_str(data: &str) -> Vec<String> {
    fixture_rows_from_str(data)
        .into_iter()
        .next()
        .expect("missing fixture row")
}

pub fn text_replacement(
    move_left: u32,
    backspaces: u32,
    insert: impl Into<String>,
    move_right: u32,
) -> TextReplacement {
    TextReplacement {
        move_left,
        backspaces,
        insert: insert.into(),
        move_right,
    }
}

pub fn zero_edge_text_replacement(
    row: &[String],
    backspaces: usize,
    insert: usize,
) -> TextReplacement {
    text_replacement(
        0,
        row[backspaces].parse().expect("backspaces"),
        &row[insert],
        0,
    )
}

pub fn fixture_rows_from_str(data: &str) -> Vec<Vec<String>> {
    fixture_data_lines(data)
        .map(|line| line.split('\t').map(decode_fixture_field).collect())
        .collect()
}

pub fn fixture_lines_from_str(data: &str) -> Vec<String> {
    fixture_data_lines(data).map(decode_fixture_field).collect()
}

pub fn parse_bool_fixture(value: &str) -> bool {
    match value {
        "true" => true,
        "false" => false,
        other => panic!("invalid bool fixture value: {other}"),
    }
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn fixture_data_lines(data: &str) -> impl Iterator<Item = &str> {
    data.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
}

fn decode_fixture_field(value: &str) -> String {
    value.replace("\\s", " ")
}
