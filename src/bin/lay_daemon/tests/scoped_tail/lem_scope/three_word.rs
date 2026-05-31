use super::*;

fn three_word_case(id: &str) -> Vec<String> {
    fixture_row_by_id("daemon_scoped_tail_three_word.tsv", id)
}

#[test]
fn scoped_tail_generalizes_to_more_than_two_words() {
    let row = three_word_case("generalizes_mixed_tail");
    assert_eq!(row.len(), 10, "bad fixture row: {row:?}");
    let scope: usize = row[3].parse().expect("scope");
    let (_buffer, events, _) = typed_tail(
        &[(&row[1], layout_from_fixture(&row[2]))],
        scope,
        "three-word tail",
    );

    assert_eq!(map_original_events(&events), row[4]);
    assert_eq!(decide_scoped_tail_correction(&events), Some(row[5].clone()));
}

#[test]
fn scoped_tail_uses_lem_for_three_word_mixed_tail() {
    let row = three_word_case("mixed_layout_tail");
    assert_eq!(row.len(), 10, "bad fixture row: {row:?}");
    let scope: usize = row[3].parse().expect("scope");
    let (_buffer, events, _) = typed_tail(
        &[(&row[1], layout_from_fixture(&row[2]))],
        scope,
        "three-word tail",
    );

    assert_eq!(map_original_events(&events), row[4]);
    assert_eq!(decide_scoped_tail_correction(&events), Some(row[5].clone()));
}

#[test]
fn scoped_tail_keeps_two_russian_words_and_flips_current_english_layout_tail() {
    let row = three_word_case("stable_ru_context_current_tail");
    assert_eq!(row.len(), 10, "bad fixture row: {row:?}");
    let scope: usize = row[3].parse().expect("scope");
    let (_buffer, events, _) = typed_tail(
        &[(&row[1], layout_from_fixture(&row[2]))],
        scope,
        "three-word tail",
    );
    let original = map_original_events(&events);
    let words = split_event_words(&events).expect("split words");
    let candidates = scoped_tail_lem_candidates(&words, true, true);
    let replacement =
        decide_scoped_tail_correction_with_lem(&events, true).expect("smart replacement");

    assert_eq!(original, row[4]);
    assert!(candidates.iter().any(|candidate| candidate == &row[6]));
    assert!(
        !candidates
            .iter()
            .any(|candidate| candidate.contains(&row[7])),
        "stable completed Russian context must not be typo-corrected: {candidates:?}"
    );
    assert_eq!(replacement, row[5]);
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(text_replacement_zero_edges(&row, 9, 8))
    );
}

#[test]
fn scoped_tail_keeps_short_repeated_completed_word_and_flips_current_tail() {
    let row = three_word_case("short_repeated_context");
    assert_eq!(row.len(), 10, "bad fixture row: {row:?}");
    let scope: usize = row[3].parse().expect("scope");
    let (_buffer, events, _) = typed_tail(
        &[(&row[1], layout_from_fixture(&row[2]))],
        scope,
        "three-word tail",
    );
    let original = map_original_events(&events);
    let replacement =
        decide_scoped_tail_correction_with_lem(&events, true).expect("smart replacement");

    assert_eq!(original, row[4]);
    assert_eq!(replacement, row[5]);
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(text_replacement_zero_edges(&row, 9, 8))
    );
}
