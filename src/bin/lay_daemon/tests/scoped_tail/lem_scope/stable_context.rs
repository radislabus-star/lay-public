use super::*;

#[test]
fn scoped_tail_keeps_stable_russian_context_before_current_layout_tail() {
    let row = fixture_rows("daemon_scoped_tail_stable_current.tsv")
        .into_iter()
        .find(|row| row.get(2).is_some_and(|value| !value.is_empty()))
        .expect("stable current fixture with candidate checks");
    assert_eq!(row.len(), 7, "bad fixture row: {row:?}");

    let input = &row[0];
    let original_expected = &row[1];
    let expected_candidate = &row[2];
    let forbidden_candidate = &row[3];
    let replacement_expected = &row[4];
    let (_buffer, events, _) = typed_tail(&[(input, true)], 3, "three-word tail");
    let original = map_original_events(&events);
    let words = split_event_words(&events).expect("split words");
    let candidates = scoped_tail_lem_candidates(&words, true, true);
    let replacement =
        decide_scoped_tail_correction_with_lem(&events, true).expect("smart replacement");

    assert_eq!(original, *original_expected);
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate == expected_candidate),
        "expected current-tail flip candidate, got {candidates:?}"
    );
    assert!(
        !candidates
            .iter()
            .any(|candidate| candidate == forbidden_candidate),
        "stable completed Russian context must not be fully flipped: {candidates:?}"
    );
    assert_eq!(replacement, *replacement_expected);
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(text_replacement_zero_edges(&row, 6, 5))
    );
}

#[test]
fn scoped_tail_keeps_stable_russian_context_before_current_layout_tail_with_chto() {
    let row = fixture_rows("daemon_scoped_tail_stable_current.tsv")
        .into_iter()
        .find(|row| row.get(2).is_some_and(|value| value.is_empty()))
        .expect("stable current fixture without candidate checks");
    assert_eq!(row.len(), 7, "bad fixture row: {row:?}");

    let (_buffer, events, _) = typed_tail(&[(&row[0], true)], 3, "three-word tail");
    let replacement =
        decide_scoped_tail_correction_with_lem(&events, true).expect("smart replacement");

    assert_eq!(map_original_events(&events), row[1]);
    assert_eq!(replacement, row[4]);
}

#[test]
fn manual_decoder_keeps_stable_russian_context_before_completed_layout_tail() {
    let row = fixture_rows("daemon_scoped_tail_stable_completed.tsv")
        .into_iter()
        .next()
        .expect("stable completed fixture");
    assert_eq!(row.len(), 9, "bad fixture row: {row:?}");

    let input = &row[0];
    let expected_scope: usize = row[1].parse().expect("scope");
    let original_expected = &row[2];
    let replay_target_expected = &row[3];
    let replacement_expected = &row[4];
    let buffer = typed_buffer(&[(input, true)]);

    let scope = effective_replace_words(&buffer, 3, CorrectionEngine::Smart, true);
    let (events, _) = buffer.what_to_replay(scope).expect("three-word tail");
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
        scoped_options: ScopedTailOptions {
            lem_enabled: true,
            allow_layout_auto: true,
            lem_weight: 1.0,
        },
    });

    assert_eq!(scope, expected_scope);
    assert_eq!(original, *original_expected);
    assert_eq!(replay_target, *replay_target_expected);
    assert_eq!(
        decoded.action,
        DecoderAction::ReplaceText {
            replacement: replacement_expected.clone(),
            source: CorrectionSource::SmartText,
        }
    );
    assert_eq!(
        decoded.edit.expect("manual edit").plan,
        text_replacement_from_fixture(&row, 5, 6, 7, 8)
    );
}
