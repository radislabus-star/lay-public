use super::*;

fn current_tail_case(id: &str) -> Vec<String> {
    fixture_row_by_id("daemon_scoped_tail_current_tail.tsv", id)
}

#[test]
fn scoped_tail_keeps_good_english_previous_word_and_flips_current_layout_word() {
    let row = current_tail_case("good_english_previous");
    assert_eq!(row.len(), 9, "bad fixture row: {row:?}");
    let scope: usize = row[3].parse().expect("scope");
    let (_buffer, events, _) = typed_tail(
        &[(&row[1], layout_from_fixture(&row[2]))],
        scope,
        "two-word tail",
    );

    assert_eq!(map_original_events(&events), row[4]);
    assert_eq!(decide_scoped_tail_correction(&events), Some(row[6].clone()));
}

#[test]
fn scoped_tail_keeps_good_russian_previous_word_and_flips_current_currency_symbol() {
    let row = current_tail_case("currency_symbol");
    assert_eq!(row.len(), 9, "bad fixture row: {row:?}");
    let scope: usize = row[3].parse().expect("scope");
    let (_buffer, events, _) = typed_tail(
        &[(&row[1], layout_from_fixture(&row[2]))],
        scope,
        "two-word tail",
    );
    let original = map_original_events(&events);
    let target_is_ru = replay_layout_decision(&events).target_is_ru;
    let replay_target = map_events_to_layout(&events, target_is_ru);
    let decoded = decode_manual_tail(ManualDecodeRequest {
        events: &events,
        original: &original,
        converted: &replay_target,
        engine: CorrectionEngine::Smart,
        force_replay: false,
        auto_replace: true,
    });

    assert_eq!(original, row[4]);
    assert_eq!(replay_target, row[5]);
    assert_eq!(decide_scoped_tail_correction(&events), Some(row[6].clone()));
    assert_eq!(
        decoded.action,
        DecoderAction::ReplaceText {
            replacement: row[6].clone(),
            source: CorrectionSource::SmartText,
        }
    );
    assert_eq!(
        decoded.edit.map(|edit| edit.plan),
        Some(text_replacement_zero_edges(&row, 7, 8))
    );
}

#[test]
fn scoped_tail_keeps_completed_ascii_title_word_and_flips_current_latin_keys() {
    let row = current_tail_case("ascii_title_current");
    assert_eq!(row.len(), 9, "bad fixture row: {row:?}");
    let scope: usize = row[3].parse().expect("scope");
    let (_buffer, events, _) = typed_tail(
        &[(&row[1], layout_from_fixture(&row[2]))],
        scope,
        "two-word tail",
    );

    assert_eq!(map_original_events(&events), row[4]);
    assert_eq!(decide_scoped_tail_correction(&events), Some(row[6].clone()));
    assert_eq!(
        plan_text_replacement(&row[4], &row[6]),
        Some(text_replacement_zero_edges(&row, 7, 8))
    );
}

#[test]
fn scoped_tail_keeps_single_completed_cyrillic_fragment_before_current_word() {
    let row = current_tail_case("single_cyrillic_fragment");
    assert_eq!(row.len(), 9, "bad fixture row: {row:?}");
    let scope: usize = row[3].parse().expect("scope");
    let (_buffer, events, _) = typed_tail(
        &[(&row[1], layout_from_fixture(&row[2]))],
        scope,
        "two-word tail",
    );
    let original = map_original_events(&events);
    let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");

    assert_eq!(original, row[4]);
    assert_eq!(replacement, row[6]);
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(text_replacement_zero_edges(&row, 7, 8))
    );
}
