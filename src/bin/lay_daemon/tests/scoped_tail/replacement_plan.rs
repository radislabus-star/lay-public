use super::*;

#[test]
fn replacement_plan_fixture_cases_match_minimal_edits() {
    for row in fixture_rows("daemon_scoped_tail_replacement_plan.tsv") {
        assert_eq!(row.len(), 7, "bad replacement-plan fixture row: {row:?}");
        let original = &row[1];
        let replacement = &row[2];
        assert_eq!(
            plan_text_replacement(original, replacement),
            Some(text_replacement_from_fixture(&row, 3, 4, 5, 6)),
            "case={}",
            row[0]
        );
    }
}

#[test]
fn pending_auto_undo_restores_full_original_text() {
    let row = fixture_rows("daemon_scoped_tail_pending_auto_undo.tsv")
        .into_iter()
        .next()
        .expect("pending auto undo fixture");
    assert_eq!(row.len(), 7, "bad pending-auto-undo fixture row: {row:?}");
    let mut buffer = WordBuffer::new();
    buffer.remember_pending_auto_undo(&row[0], &row[1], &row[2], 1, 1);

    let undo = buffer.take_pending_auto_undo().expect("pending undo");
    assert_eq!(
        undo.replacement_plan(),
        text_replacement_from_fixture(&row, 3, 4, 5, 6)
    );
}

#[test]
fn opposite_events_flip_each_key_own_layout_for_smart_mixed_tail() {
    let events = [
        key_event(KeyCode::KEY_H, true),
        key_event(KeyCode::KEY_R, true),
        key_event(KeyCode::KEY_F, true),
        key_event(KeyCode::KEY_SPACE, false),
        key_event(KeyCode::KEY_G, false),
        key_event(KeyCode::KEY_H, false),
        key_event(KeyCode::KEY_J, false),
    ];

    assert_eq!(map_original_events(&events), "рка ghj");
    assert_eq!(map_opposite_events(&events), "hrf про");
}

#[test]
fn smart_insert_layout_follows_result_text_tail() {
    for row in fixture_rows("daemon_scoped_tail_layout_preference.tsv") {
        assert_eq!(row.len(), 3, "bad layout-preference fixture row: {row:?}");
        let fallback_is_ru = match row[1].as_str() {
            "ru" => true,
            "us" => false,
            _ => panic!("unknown fallback layout {:?}", row[1]),
        };
        let expected_is_ru: bool = row[2].parse().expect("expected bool");
        assert_eq!(
            preferred_layout_for_text(&row[0], fallback_is_ru),
            expected_is_ru,
            "text={:?}",
            row[0]
        );
    }
}

#[test]
fn target_layout_matches_cache_contract() {
    assert_eq!(target_layout(true), ("ru", "xkb:ru::rus"));
    assert_eq!(target_layout(false), ("us", "xkb:us::eng"));
}
