use super::*;

#[test]
fn manual_decoder_keeps_completed_russian_preposition_before_completed_ascii_tail() {
    for row in fixture_rows("daemon_scoped_tail_completed_titlecase.tsv") {
        assert_eq!(
            row.len(),
            3,
            "fixture format: left<TAB>middle<TAB>ascii_tail"
        );
        let left = &row[0];
        let middle = &row[1];
        let ascii_tail = &row[2];
        let typed_tail = lay::dict::convert(ascii_tail, lay::dict::Direction::Us2Ru);
        let mut buffer = WordBuffer::new();

        push_text_as_layout(&mut buffer, left, true);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, middle, true);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, &typed_tail, true);
        buffer.handle_space();

        let (events, _) = buffer.what_to_replay(3).expect("completed three-word tail");
        let original = map_original_events(&events);
        let target_is_ru = replay_layout_decision(&events).target_is_ru;
        let replay_target = map_events_to_layout(&events, target_is_ru);
        let expected = format!("{left} {middle} {ascii_tail} ");
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
            },
        });

        assert_eq!(original, format!("{left} {middle} {typed_tail} "));
        assert_eq!(
            decoded.action,
            DecoderAction::ReplaceText {
                replacement: expected.clone(),
                source: CorrectionSource::SmartText,
            }
        );
        assert_eq!(
            decoded.edit.map(|edit| edit.plan),
            Some(TextReplacement {
                move_left: 1,
                backspaces: typed_tail.chars().count() as u32,
                insert: ascii_tail.to_string(),
                move_right: 1,
            })
        );
    }
}
