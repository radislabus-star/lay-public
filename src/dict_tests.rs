use super::*;
use crate::typing_assist_test_fixtures::{fixture_row_by_id, fixture_rows};

#[test]
fn us_to_ru_basic() {
    assert_convert_fixture("basic_us_to_ru");
}

#[test]
fn ru_to_us_basic() {
    assert_convert_fixture("basic_ru_to_us");
}

#[test]
fn warm_reverse_projection_is_read_only_after_startup() {
    assert_ne!(warm_up_ru_to_us(), 0);
    assert_eq!(convert_ru_to_us_if_warm("Згыр").as_deref(), Some("Push"));
}

#[test]
fn detect() {
    for row in fixture_rows("dict_detect.tsv") {
        assert_eq!(row.len(), 2, "dict detect fixture must be TSV");
        assert_eq!(
            detect_direction(&row[0]),
            parse_direction(&row[1]),
            "input={:?}",
            row[0]
        );
    }
}

#[test]
fn preserves_unknown_chars() {
    assert_convert_fixture("preserves_unknown_chars");
}

#[test]
fn us_shift_punctuation_maps_to_physical_ru_letters() {
    assert_convert_fixture("shift_punctuation_us_to_ru");
    assert_convert_fixture("shift_punctuation_ru_to_us");
}

#[test]
fn scalar_projection_matches_string_conversion_for_mapped_and_unmapped_symbols() {
    let symbols = (0_u32..=0x7f)
        .chain(0x400..=0x4ff)
        .filter_map(char::from_u32);
    for direction in [Direction::Us2Ru, Direction::Ru2Us] {
        for symbol in symbols.clone() {
            let projected = convert(&symbol.to_string(), direction);
            assert_eq!(
                projected.chars().collect::<Vec<_>>(),
                [project_char(symbol, direction)]
            );
        }
    }
}

fn assert_convert_fixture(label: &str) {
    let row = fixture_row_by_id("dict_convert.tsv", label);
    assert_eq!(row.len(), 4, "dict convert fixture must be TSV");
    assert_eq!(
        convert(&row[2], parse_direction(&row[1])),
        row[3],
        "label={label}"
    );
}

fn parse_direction(value: &str) -> Direction {
    match value {
        "Us2Ru" => Direction::Us2Ru,
        "Ru2Us" => Direction::Ru2Us,
        other => panic!("unknown dict direction: {other}"),
    }
}
