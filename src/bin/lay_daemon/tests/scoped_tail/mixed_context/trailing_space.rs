use super::*;

fn trailing_space_case(id: &str) -> Vec<String> {
    fixture_row_by_id("daemon_scoped_tail_trailing_space.tsv", id)
}

#[test]
fn scoped_tail_trailing_space_keeps_previous_good_word_and_flips_current_completed_latin_keys() {
    let row = trailing_space_case("ascii_title_completed");
    assert_eq!(row.len(), 10, "bad fixture row: {row:?}");
    let configured_scope: usize = row[3].parse().expect("scope");
    let buffer = typed_buffer(&[(&row[1], layout_from_fixture(&row[2]))]);

    let scope = effective_replace_words(&buffer, configured_scope, CorrectionEngine::Smart, true);
    let (events, backspaces) = buffer.what_to_replay(scope).expect("last word tail");

    assert_eq!(scope, configured_scope);
    assert_eq!(map_original_events(&events), row[4]);
    assert_eq!(decide_scoped_tail_correction(&events), Some(row[5].clone()));
    assert_eq!(backspaces, row[6].parse::<u32>().expect("backspaces"));
}

#[test]
fn scoped_tail_trailing_space_keeps_previous_russian_word_and_flips_completed_tail() {
    let row = trailing_space_case("russian_previous_completed");
    assert_eq!(row.len(), 10, "bad fixture row: {row:?}");
    let scope: usize = row[3].parse().expect("scope");
    let (_buffer, events, _) = typed_tail(
        &[(&row[1], layout_from_fixture(&row[2]))],
        scope,
        "two-word tail",
    );
    let original = map_original_events(&events);
    let replacement =
        decide_scoped_tail_correction_with_lem(&events, true).expect("smart replacement");

    assert_eq!(original, row[4]);
    assert_eq!(replacement, row[5]);
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(text_replacement_from_fixture(&row, 7, 6, 8, 9))
    );
}
