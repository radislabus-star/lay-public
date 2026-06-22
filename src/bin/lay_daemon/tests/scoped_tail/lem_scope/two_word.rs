use super::*;

#[test]
fn scoped_tail_uses_lem_for_two_word_mixed_tail() {
    let (_buffer, events, _) = typed_tail(&[("good ntrcn", false)], 2, "two-word tail");
    let words = split_event_words(&events).expect("split words");
    let ranked = lay::lem::rank_candidates(
        &map_original_events(&events),
        scoped_tail_lem_candidates(&words, true, true),
    );

    assert_eq!(map_original_events(&events), "good ntrcn");
    assert_eq!(ranked[0].text, "good текст");
    assert_eq!(
        decide_scoped_tail_correction_with_lem(&events, true),
        Some("good текст".to_string())
    );
}

#[test]
fn scoped_tail_flips_short_english_layout_pair_in_ascii_context() {
    let mut buffer = typed_buffer(&[("file ", false), ("щт щаа", true)]);

    let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
    let words = split_event_words(&events).expect("split words");
    let candidates = scoped_tail_lem_candidates(&words, true, true);

    assert_eq!(map_original_events(&events), "file щт щаа");
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate == "file on off"),
        "short completed layout word must be offered to LEM: {candidates:?}"
    );
    assert_eq!(
        decide_scoped_tail_correction_with_lem(&events, true),
        Some("file on off".to_string())
    );

    buffer.handle_space();
    let (completed_events, _) = buffer.what_to_replay(3).expect("completed tail");
    assert_eq!(map_original_events(&completed_events), "file щт щаа ");
    assert_eq!(
        decide_scoped_tail_correction_with_lem(&completed_events, true),
        Some("file on off ".to_string())
    );

    let original = map_original_events(&completed_events);
    let target_is_ru = replay_layout_decision(&completed_events).target_is_ru;
    let converted = map_events_to_layout(&completed_events, target_is_ru);
    let decoded = decode_manual_tail(ManualDecodeRequest {
        events: &completed_events,
        original: &original,
        converted: &converted,
        engine: CorrectionEngine::Smart,
        force_replay: false,
        auto_replace: true,
        scoped_options: ScopedTailOptions {
            lem_enabled: true,
            allow_layout_auto: true,
            lem_weight: 1.0,
        },
    });

    assert_eq!(
        decoded.action,
        DecoderAction::ReplaceText {
            replacement: "file on off ".to_string(),
            source: CorrectionSource::SmartText,
        }
    );
}

#[test]
fn scoped_tail_flips_short_english_layout_pair_without_context() {
    let (_buffer, events, _) = typed_tail(&[("щт щаа", true)], 2, "two-word tail");
    let words = split_event_words(&events).expect("split words");
    let candidates = scoped_tail_lem_candidates(&words, true, true);

    assert_eq!(map_original_events(&events), "щт щаа");
    assert!(
        candidates.iter().any(|candidate| candidate == "on off"),
        "short completed layout pair must be offered to LEM: {candidates:?}"
    );
    assert_eq!(
        decide_scoped_tail_correction_with_lem(&events, true),
        Some("on off".to_string())
    );
}

#[test]
fn scoped_tail_keeps_good_russian_context_and_flips_current_acronym() {
    for row in fixture_rows("daemon_scoped_tail_acronym_context.tsv") {
        assert_eq!(row.len(), 4, "bad fixture row: {row:?}");
        let left1 = &row[0];
        let left2 = &row[1];
        let typed_tail = &row[2];
        let expected_tail = &row[3];
        let buffer = typed_buffer(&[
            (left1, true),
            (" ", true),
            (left2, true),
            (" ", true),
            (typed_tail, true),
        ]);

        let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
        let original = map_original_events(&events);
        let expected = format!("{left1} {left2} {expected_tail}");

        assert_eq!(
            decide_scoped_tail_correction_with_lem(&events, true),
            Some(expected.clone()),
            "original={original:?}"
        );
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some(expected.clone()),
            "original={original:?}"
        );
        assert_eq!(
            plan_text_replacement(&original, &expected),
            Some(text_replacement(
                0,
                typed_tail.chars().count() as u32,
                expected_tail,
                0,
            ))
        );
    }
}

#[test]
fn scoped_tail_converts_apostrophe_layout_word_as_letter() {
    let (_buffer, events, _) = typed_tail(&[("'nj ckjdj", false)], 2, "two-word tail");
    let original = map_original_events(&events);

    assert_eq!(original, "'nj ckjdj");
    assert_eq!(
        decide_scoped_tail_correction_with_lem(&events, true),
        Some("это слово".to_string())
    );
}
