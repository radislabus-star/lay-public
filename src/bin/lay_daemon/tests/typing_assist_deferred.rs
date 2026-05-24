use super::*;

#[test]
fn deferred_typing_assist_stays_valid_after_next_word_started() {
    assert!(!should_run_deferred_typing_assist_after_space(
        false, true, false
    ));
    assert!(!should_run_deferred_typing_assist_after_space(
        true, false, false
    ));
    assert!(!should_run_deferred_typing_assist_after_space(
        true, true, true
    ));
    assert!(should_run_deferred_typing_assist_after_space(
        true, true, false
    ));
    assert_eq!(typing_assist_cursor_offset_after_space(0), 0);
    assert_eq!(typing_assist_cursor_offset_after_space(3), 3);
}

#[test]
fn deferred_typing_assist_can_plan_previous_word_behind_current_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "gjhn", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "x", false);

    let events = buffer
        .last_completed_words_events(1)
        .expect("completed word");
    let context = lay::typing_context::completed_tail_context(&buffer, 1, &events);
    let pipeline = lay::typing_context::typing_assist_pipeline_for_context(
        true,
        lay::config::CorrectionSafety::Normal,
        &default_typing_assist_pipeline(),
        &context,
    );
    let edit = lay::decoder::decode_typing_assist_tail(
        &events,
        true,
        &pipeline,
        CorrectionSource::TypingAssist,
    )
    .expect("previous word correction");
    let shifted = lay::text_edit::offset_replacement_plan_for_cursor(
        &edit.plan,
        typing_assist_cursor_offset_after_space(buffer.current_len()),
    );

    assert_eq!(edit.original, "gjhn ");
    assert_eq!(edit.replacement, "порт ");
    assert_eq!(
        shifted,
        TextReplacement {
            move_left: 2,
            backspaces: 4,
            insert: "порт".to_string(),
            move_right: 2,
        }
    );
}
