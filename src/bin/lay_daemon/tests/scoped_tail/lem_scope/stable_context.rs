use super::*;

#[test]
fn scoped_tail_keeps_stable_russian_context_before_current_layout_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "чем", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "ещё", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "луеен", true);

    let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
    let original = map_original_events(&events);
    let words = split_event_words(&events).expect("split words");
    let candidates = scoped_tail_lem_candidates(&words, true, true);
    let replacement =
        decide_scoped_tail_correction_with_lem(&events, true).expect("smart replacement");

    assert_eq!(original, "чем ещё луеен");
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate == "чем ещё ketty"),
        "expected current-tail flip candidate, got {candidates:?}"
    );
    assert!(
        !candidates
            .iter()
            .any(|candidate| candidate == "xtv to` ketty"),
        "stable completed Russian context must not be fully flipped: {candidates:?}"
    );
    assert_eq!(replacement, "чем ещё ketty");
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(TextReplacement {
            move_left: 0,
            backspaces: 5,
            insert: "ketty".to_string(),
            move_right: 0,
        })
    );
}

#[test]
fn scoped_tail_keeps_stable_russian_context_before_current_layout_tail_with_chto() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "что", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "ещё", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "луеен", true);

    let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
    let original = map_original_events(&events);
    let replacement =
        decide_scoped_tail_correction_with_lem(&events, true).expect("smart replacement");

    assert_eq!(original, "что ещё луеен");
    assert_eq!(replacement, "что ещё ketty");
}

#[test]
fn manual_decoder_keeps_stable_russian_context_before_completed_layout_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "чем", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "ещё", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "луеен", true);
    buffer.handle_space();

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
        },
    });

    assert_eq!(scope, 3);
    assert_eq!(original, "чем ещё луеен ");
    assert_eq!(replay_target, "xtv to` ketty ");
    assert_eq!(
        decoded.action,
        DecoderAction::ReplaceText {
            replacement: "чем ещё ketty ".to_string(),
            source: CorrectionSource::SmartText,
        }
    );
    assert_eq!(
        decoded.edit.expect("manual edit").plan,
        TextReplacement {
            move_left: 1,
            backspaces: 5,
            insert: "ketty".to_string(),
            move_right: 1,
        }
    );
}
