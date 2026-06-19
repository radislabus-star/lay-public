use super::*;

fn manual_decision_case(id: &str) -> Vec<String> {
    fixture_row_by_id("daemon_scoped_tail_manual_decision.tsv", id)
}

fn expected_correction_action(row: &[String]) -> Correction {
    assert_eq!(row.len(), 5, "bad fixture row: {row:?}");
    match row[3].as_str() {
        "replay" => Correction::ReplayAll,
        "insert" => Correction::InsertText(row[4].clone()),
        other => panic!("unknown expected action {other:?}"),
    }
}

#[test]
fn smart_decision_replays_single_valid_word_as_manual_toggle() {
    let row = manual_decision_case("single_valid_word");

    assert_eq!(
        decide_correction(&row[1], &row[2], CorrectionEngine::Smart),
        expected_correction_action(&row)
    );
}

#[test]
fn single_word_wrong_layout_replay_target_is_opposite_layout() {
    let (_buffer, events, backspaces) = typed_tail(&[("ltkfq", false)], 1, "single word");
    let decision = replay_layout_decision(&events);

    assert_eq!(backspaces, 5);
    assert_eq!(map_original_events(&events), "ltkfq");
    assert!(decision.target_is_ru);
    assert_eq!(map_target_events(&events, decision.target_is_ru), "делай");
    let row = manual_decision_case("single_wrong_layout");
    assert_eq!(
        decide_correction(&row[1], &row[2], CorrectionEngine::Smart),
        expected_correction_action(&row)
    );

    let row = manual_decision_case("single_wrong_layout_ru_to_en");
    assert_eq!(
        decide_correction(&row[1], &row[2], CorrectionEngine::Smart),
        expected_correction_action(&row)
    );
}

#[test]
fn single_currency_tail_replays_ru_semicolon_as_us_dollar() {
    let mut buffer = WordBuffer::new();
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_4, false),
            (KeyCode::KEY_0, false),
            (KeyCode::KEY_0, false),
            (KeyCode::KEY_0, false),
            (KeyCode::KEY_4, true),
        ],
        true,
    );
    let (events, backspaces) = buffer.what_to_replay(1).expect("single word");
    let decision = replay_layout_decision(&events);
    let original = map_original_events(&events);
    let target = map_target_events(&events, decision.target_is_ru);

    assert_eq!(backspaces, 5);
    assert_eq!(original, "4000;");
    assert!(!decision.target_is_ru);
    assert_eq!(target, "4000$");
    assert_eq!(
        decide_correction(&original, &target, CorrectionEngine::Smart),
        Correction::ReplayAll
    );
}

#[test]
fn smart_decision_replays_single_cyrillic_acronym_as_manual_toggle() {
    let events = [
        key_event_with_shift(KeyCode::KEY_L, true, true),
        key_event_with_shift(KeyCode::KEY_L, true, true),
        key_event_with_shift(KeyCode::KEY_M, true, true),
    ];
    let decision = replay_layout_decision(&events);
    let original = map_original_events(&events);
    let target = map_events_to_layout(&events, decision.target_is_ru);

    assert_eq!(original, "ДДЬ");
    assert_eq!(target, "LLM");
    assert!(!decision.target_is_ru);
    assert_eq!(
        decide_correction(&original, &target, CorrectionEngine::Smart),
        Correction::ReplayAll
    );
}

#[test]
fn smart_decision_replays_two_valid_words_as_manual_toggle() {
    let row = manual_decision_case("two_valid_words");

    assert_eq!(
        decide_correction(&row[1], &row[2], CorrectionEngine::Smart),
        expected_correction_action(&row)
    );
}

#[test]
fn smart_decision_replays_valid_russian_preposition_phrase_as_manual_toggle() {
    let row = manual_decision_case("valid_preposition_phrase");

    assert_eq!(
        decide_correction(&row[1], &row[2], CorrectionEngine::Smart),
        expected_correction_action(&row)
    );
}

#[test]
fn smart_decision_converts_mixed_layout_neighbor_only() {
    for id in ["mixed_neighbor_short", "mixed_neighbor_word"] {
        let row = manual_decision_case(id);

        assert_eq!(
            decide_correction(&row[1], &row[2], CorrectionEngine::Smart),
            expected_correction_action(&row)
        );
    }
}

#[test]
fn smart_decision_replays_protected_ascii_span_as_manual_toggle() {
    let row = manual_decision_case("protected_ascii_span");

    assert_eq!(
        decide_correction(&row[1], &row[2], CorrectionEngine::Smart),
        expected_correction_action(&row)
    );
}

#[test]
fn smart_decision_repairs_brand_plus_letter_inside_larger_tail() {
    let row = manual_decision_case("brand_plus_letter");

    assert_eq!(
        decide_correction(&row[1], &row[2], CorrectionEngine::Smart),
        expected_correction_action(&row)
    );
}
