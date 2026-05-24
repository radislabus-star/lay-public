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

#[test]
fn uppercase_shift_layout_word_gets_contextual_candidate() {
    let input = "HF<JNF ";
    let pipeline = lay::typing_context::typing_assist_pipeline_for_context(
        true,
        lay::config::CorrectionSafety::Normal,
        &default_typing_assist_pipeline(),
        input,
    );

    assert_eq!(
        apply_typing_assist_with_pipeline(input, true, &pipeline),
        Some("РАБОТА ".to_string())
    );

    let leading_shift_input = "<FYRF ";
    let leading_shift_pipeline = lay::typing_context::typing_assist_pipeline_for_context(
        true,
        lay::config::CorrectionSafety::Normal,
        &default_typing_assist_pipeline(),
        leading_shift_input,
    );
    assert_eq!(
        apply_typing_assist_with_pipeline(leading_shift_input, true, &leading_shift_pipeline),
        Some("БАНКА ".to_string())
    );
}

#[test]
fn deferred_typing_assist_uses_widest_confident_completed_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "HF<JNF NTCN CFV", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "x", false);

    let correction =
        find_typing_assist_correction(&buffer, true, 3).expect("three-word correction");
    assert_eq!(map_original_events(&correction.events), "HF<JNF NTCN CFV ");
    assert_eq!(correction.edit.original, "HF<JNF NTCN CFV ");
    assert_eq!(correction.edit.replacement, "РАБОТА ТЕСТ САМ ");

    let shifted = lay::text_edit::offset_replacement_plan_for_cursor(
        &correction.edit.plan,
        typing_assist_cursor_offset_after_space(buffer.current_len()),
    );
    assert_eq!(
        shifted,
        TextReplacement {
            move_left: 2,
            backspaces: 15,
            insert: "РАБОТА ТЕСТ САМ".to_string(),
            move_right: 2,
        }
    );
}

#[test]
fn deferred_typing_assist_respects_single_word_scope() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "порт", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "port", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "gjhn", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "x", false);

    let correction =
        find_typing_assist_correction(&buffer, true, 1).expect("single-word correction");
    assert_eq!(map_original_events(&correction.events), "gjhn ");
    assert_eq!(correction.edit.original, "gjhn ");
    assert_eq!(correction.edit.replacement, "порт ");

    let shifted = lay::text_edit::offset_replacement_plan_for_cursor(
        &correction.edit.plan,
        typing_assist_cursor_offset_after_space(buffer.current_len()),
    );
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

#[test]
fn deferred_typing_assist_keeps_left_context_after_previous_autoreplace() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "порт", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "зщке", true);
    buffer.handle_space();

    let previous_events = buffer
        .last_completed_words_events(1)
        .expect("previous completed word");
    assert_eq!(map_original_events(&previous_events), "зщке ");
    assert!(buffer.remember_replacement_last_word_for_replay(
        &previous_events,
        &TextReplacement {
            move_left: 1,
            backspaces: 4,
            insert: "port".to_string(),
            move_right: 1,
        },
        "port ",
    ));

    let context_events = buffer
        .last_completed_words_events(2)
        .expect("preserved left context");
    assert_eq!(map_original_events(&context_events), "порт port ");

    push_text_as_layout(&mut buffer, "gjhn", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "x", false);

    let correction =
        find_typing_assist_correction(&buffer, true, 1).expect("single-word correction");
    assert_eq!(map_original_events(&correction.events), "gjhn ");
    assert_eq!(correction.edit.replacement, "порт ");
}
