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
        let layout_tail = lay::dict::convert(ascii_tail, lay::dict::Direction::Us2Ru);
        let typed_text = format!("{left} {middle} {layout_tail} ");

        let (_buffer, events, _) =
            typed_tail(&[(&typed_text, true)], 3, "completed three-word tail");
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
                lem_weight: 1.0,
            },
        });

        assert_eq!(original, format!("{left} {middle} {layout_tail} "));
        assert_eq!(
            decoded.action,
            DecoderAction::ReplaceText {
                replacement: expected.clone(),
                source: CorrectionSource::SmartText,
            }
        );
        assert_eq!(
            decoded.edit.map(|edit| edit.plan),
            Some(text_replacement(
                1,
                layout_tail.chars().count() as u32,
                ascii_tail,
                1,
            ))
        );
    }
}
