use super::*;

fn stale_layout_case(id: &str) -> Vec<String> {
    fixture_row_by_id("daemon_scoped_tail_stale_layout.tsv", id)
}

fn single_char_from_fixture(value: &str) -> char {
    let mut chars = value.chars();
    let ch = chars
        .next()
        .unwrap_or_else(|| panic!("empty char fixture value"));
    assert!(
        chars.next().is_none(),
        "fixture value must contain one char: {value:?}"
    );
    ch
}

#[test]
fn scoped_tail_keeps_completed_russian_y_word_and_flips_current_latin_brand() {
    let row = stale_layout_case("completed_russian_y_word");
    assert_eq!(row.len(), 12, "bad fixture row: {row:?}");
    let scope: usize = row[7].parse().expect("scope");
    let (_buffer, events, _) = typed_tail(
        &[(&row[1], layout_from_fixture(&row[2]))],
        scope,
        "three-word tail",
    );
    let original = map_original_events(&events);
    let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");

    assert_eq!(original, row[8]);
    assert_eq!(replacement, row[9]);
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(text_replacement_zero_edges(&row, 10, 11))
    );
}

#[test]
fn scoped_tail_repairs_stale_layout_flag_inside_completed_russian_word() {
    let row = stale_layout_case("stale_layout_inside_word");
    assert_eq!(row.len(), 12, "bad fixture row: {row:?}");
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, &row[1], layout_from_fixture(&row[2]));
    buffer.push(text_key_event(
        single_char_from_fixture(&row[3]),
        layout_from_fixture(&row[4]),
    ));
    push_text_as_layout(&mut buffer, &row[5], layout_from_fixture(&row[6]));
    let scope: usize = row[7].parse().expect("scope");
    let (events, _) = buffer.what_to_replay(scope).expect("three-word tail");
    let original = map_original_events(&events);
    let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");

    assert_eq!(original, row[8]);
    assert_eq!(replacement, row[9]);
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(text_replacement_zero_edges(&row, 10, 11))
    );
}
