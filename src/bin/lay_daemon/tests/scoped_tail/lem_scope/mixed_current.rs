use super::*;

fn mixed_current_case(id: &str) -> Vec<String> {
    fixture_row_by_id("daemon_scoped_tail_mixed_current.tsv", id)
}

#[test]
fn scoped_tail_handles_three_completed_words_with_typo() {
    let row = mixed_current_case("three_completed_with_typo");
    assert_eq!(row.len(), 5, "bad fixture row: {row:?}");
    let buffer = typed_buffer_from_fixture_parts(&row[1]);
    let expected_scope: usize = row[2].parse().expect("scope");

    let scope = effective_replace_words(&buffer, 3, CorrectionEngine::Smart, true);
    let (events, _) = buffer.what_to_replay(scope).expect("three-word tail");

    assert_eq!(scope, expected_scope);
    assert_eq!(map_original_events(&events), row[3]);
    assert_eq!(
        decide_scoped_tail_correction_with_lem(&events, true),
        Some(row[4].clone())
    );
}

#[test]
fn scoped_tail_keeps_live_and_flips_russian_current_tail() {
    let row = mixed_current_case("live_current_tail");
    assert_eq!(row.len(), 5, "bad fixture row: {row:?}");
    let buffer = typed_buffer_from_fixture_parts(&row[1]);
    let scope: usize = row[2].parse().expect("scope");
    let (events, _) = buffer.what_to_replay(scope).expect("two-word tail");

    assert_eq!(map_original_events(&events), row[3]);
    assert_eq!(decide_scoped_tail_correction(&events), Some(row[4].clone()));
}

#[test]
fn scoped_tail_normalizes_mixed_current_word_to_last_layout() {
    let row = mixed_current_case("mixed_current_last_layout");
    assert_eq!(row.len(), 5, "bad fixture row: {row:?}");
    let buffer = typed_buffer_from_fixture_parts(&row[1]);
    let scope: usize = row[2].parse().expect("scope");
    let (events, _) = buffer.what_to_replay(scope).expect("two-word tail");

    assert_eq!(map_original_events(&events), row[3]);
    assert_eq!(decide_scoped_tail_correction(&events), Some(row[4].clone()));
}

#[test]
fn scoped_tail_normalizes_single_mixed_word_to_dominant_layout() {
    let row = mixed_current_case("mixed_single_word_dominant_ru");
    assert_eq!(row.len(), 5, "bad fixture row: {row:?}");
    let buffer = typed_buffer_from_fixture_parts(&row[1]);
    let scope: usize = row[2].parse().expect("scope");
    let (events, backspaces) = buffer.what_to_replay(scope).expect("single mixed word");

    assert_eq!(backspaces, row[3].chars().count() as u32);
    assert_eq!(map_original_events(&events), row[3]);
    assert_eq!(decide_scoped_tail_correction(&events), Some(row[4].clone()));
}

#[test]
fn scoped_tail_repairs_mixed_previous_ru_word_and_flips_current_tail() {
    let row = mixed_current_case("mixed_previous_ru_word");
    assert_eq!(row.len(), 5, "bad fixture row: {row:?}");
    let buffer = typed_buffer_from_fixture_parts(&row[1]);
    let scope: usize = row[2].parse().expect("scope");
    let (events, _) = buffer.what_to_replay(scope).expect("two-word tail");

    assert_eq!(map_original_events(&events), row[3]);
    assert_eq!(decide_scoped_tail_correction(&events), Some(row[4].clone()));
}
