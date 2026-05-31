use super::*;

fn mixed_prefix_case(id: &str) -> Vec<String> {
    fixture_row_by_id("daemon_scoped_tail_mixed_prefix.tsv", id)
}

#[test]
fn scoped_tail_collapses_cyrillic_prefix_before_ascii_hyphen_tail() {
    let row = mixed_prefix_case("prefix_ascii_tail");
    assert_eq!(row.len(), 10, "bad fixture row: {row:?}");
    let buffer = typed_buffer_from_fixture_parts(&row[1]);
    let scope: usize = row[2].parse().expect("scope");
    let (events, _) = buffer.what_to_replay(scope).expect("two-word tail");

    assert_eq!(map_original_events(&events), row[3]);
    assert_eq!(decide_scoped_tail_correction(&events), Some(row[4].clone()));
}

fn assert_hyphen_case_keeps_undo(id: &str) {
    let row = mixed_prefix_case(id);
    assert_eq!(row.len(), 10, "bad fixture row: {row:?}");
    let mut buffer = typed_buffer_from_fixture_parts(&row[1]);
    let scope: usize = row[2].parse().expect("scope");

    let (events, _) = buffer.what_to_replay(scope).expect("two-word tail");
    let original = map_original_events(&events);
    let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");
    let plan = plan_text_replacement(&original, &replacement).expect("minimal plan");

    assert_eq!(original, row[3]);
    assert_eq!(replacement, row[4]);
    assert_eq!(plan, text_replacement_zero_edges(&row, 5, 6));
    assert!(buffer.remember_replacement_last_word_for_replay(&events, &plan, &replacement));

    let (undo_events, undo_backspaces) = buffer.what_to_replay(2).expect("undo tail");
    let undo_decision = replay_layout_decision(&undo_events);
    assert_eq!(map_original_events(&undo_events), row[7]);
    assert_eq!(
        undo_backspaces,
        row[8].parse::<u32>().expect("undo backspaces")
    );
    assert!(!undo_decision.target_is_ru);
    assert_eq!(map_events_to_layout(&undo_events, false), row[9]);
    assert!(buffer.replay_toggle_ready());
}

#[test]
fn scoped_tail_repairs_mixed_cyrillic_prefix_ascii_hyphen_word_and_keeps_undo() {
    assert_hyphen_case_keeps_undo("accusative_hyphen");
}

#[test]
fn scoped_tail_repairs_mixed_cyrillic_prefix_ascii_hyphen_dative_word() {
    assert_hyphen_case_keeps_undo("dative_hyphen");
}
