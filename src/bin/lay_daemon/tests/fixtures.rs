use std::path::PathBuf;

pub(super) fn fixture_rows(name: &str) -> Vec<Vec<String>> {
    fixture_rows_from_str(&fixture_text(name))
}

pub(super) fn fixture_lines(name: &str) -> Vec<String> {
    fixture_lines_from_str(&fixture_text(name))
}

pub(super) fn fixture_row_by_id(name: &str, id: &str) -> Vec<String> {
    fixture_rows(name)
        .into_iter()
        .find(|row| row.first().is_some_and(|value| value == id))
        .unwrap_or_else(|| panic!("missing fixture row {id:?} in {name:?}"))
}

pub(super) fn first_fixture_row(name: &str) -> Vec<String> {
    fixture_rows(name)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing fixture row in {name:?}"))
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn fixture_text(name: &str) -> String {
    let path = fixture_path(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture {path:?}: {err}"))
}

fn fixture_rows_from_str(data: &str) -> Vec<Vec<String>> {
    fixture_data_lines(data)
        .map(|line| line.split('\t').map(decode_fixture_field).collect())
        .collect()
}

fn fixture_lines_from_str(data: &str) -> Vec<String> {
    fixture_data_lines(data).map(decode_fixture_field).collect()
}

fn fixture_data_lines(data: &str) -> impl Iterator<Item = &str> {
    data.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
}

fn decode_fixture_field(value: &str) -> String {
    value.replace("\\s", " ")
}
