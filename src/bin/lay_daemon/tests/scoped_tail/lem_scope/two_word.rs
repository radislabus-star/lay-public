use super::*;

#[test]
fn scoped_tail_uses_lem_for_two_word_mixed_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "good", false);
    buffer.handle_space();
    for ch in "ntrcn".chars() {
        buffer.push(text_key_event(ch, false));
    }

    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
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
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "file", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "щт", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "щаа", true);

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
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "щт", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "щаа", true);

    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
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
    let cases = [
        ("ВСЁ", "ДЕЛАЙ", "ЛВУ", "KDE"),
        ("НУЖНО", "ДЕЛАТЬ", "ТЕАЫ", "NTFS"),
        ("ПРОСТО", "ДЕЛАЙ", "СЗГ", "CPU"),
    ];

    for (left1, left2, typed_tail, expected_tail) in cases {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, left1, true);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, left2, true);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, typed_tail, true);

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
            Some(TextReplacement {
                move_left: 0,
                backspaces: typed_tail.chars().count() as u32,
                insert: expected_tail.to_string(),
                move_right: 0,
            })
        );
    }
}

#[test]
fn scoped_tail_converts_apostrophe_layout_word_as_letter() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "'nj", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "ckjdj", false);

    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let original = map_original_events(&events);

    assert_eq!(original, "'nj ckjdj");
    assert_eq!(
        decide_scoped_tail_correction_with_lem(&events, true),
        Some("это слово".to_string())
    );
}
