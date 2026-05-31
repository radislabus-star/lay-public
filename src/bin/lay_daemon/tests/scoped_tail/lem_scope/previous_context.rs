use super::*;

fn previous_context_case(id: &str) -> Vec<String> {
    fixture_row_by_id("daemon_scoped_tail_previous_context.tsv", id)
}

#[test]
fn scoped_tail_does_not_turn_valid_ascii_hyphen_tail_into_bad_russian() {
    let row = fixture_rows("daemon_scoped_tail_ascii_hyphen_guard.tsv")
        .into_iter()
        .next()
        .expect("ascii hyphen guard fixture");
    assert_eq!(row.len(), 7, "bad fixture row: {row:?}");
    let scope: usize = row[6].parse().expect("scope");
    let buffer = typed_buffer(&[
        (&row[0], layout_from_fixture(&row[1])),
        (" ", layout_from_fixture(&row[1])),
        (&row[2], layout_from_fixture(&row[3])),
        (&row[4], layout_from_fixture(&row[5])),
    ]);
    let current_events = buffer.what_to_replay(1).expect("current word").0.to_vec();
    let (events, _) = buffer.what_to_replay(scope).expect("two-word tail");
    let left = &row[0];
    let current_original = format!("{}{}", row[2], row[4]);
    let current_wrong_layout = map_events_to_layout(&current_events, true);

    assert_eq!(
        map_original_events(&events),
        format!("{left} {current_original}")
    );
    assert_ne!(
        decide_scoped_tail_correction(&events),
        Some(format!("{left} {current_wrong_layout}"))
    );
}

#[test]
fn scoped_tail_converts_confident_bad_previous_word() {
    let row = previous_context_case("confident_bad_previous");
    assert_eq!(row.len(), 5, "bad fixture row: {row:?}");
    let scope: usize = row[3].parse().expect("scope");
    let (_buffer, events, _) = typed_tail(
        &[(&row[1], layout_from_fixture(&row[2]))],
        scope,
        "two-word tail",
    );

    assert_eq!(decide_scoped_tail_correction(&events), Some(row[4].clone()));
}

#[test]
fn scoped_tail_keeps_unknown_previous_word() {
    let row = previous_context_case("unknown_previous");
    assert_eq!(row.len(), 5, "bad fixture row: {row:?}");
    let scope: usize = row[3].parse().expect("scope");
    let (_buffer, events, _) = typed_tail(
        &[(&row[1], layout_from_fixture(&row[2]))],
        scope,
        "two-word tail",
    );

    assert_eq!(decide_scoped_tail_correction(&events), Some(row[4].clone()));
}
