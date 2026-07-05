use super::*;
use crate::boundary_runtime::{handle_hard_boundary_if_needed, HardBoundaryContext};
use crate::pending_typing_assist::PendingTypingAssist;

fn deferred_case(label: &str) -> Vec<String> {
    fixture_row_by_id("daemon_typing_assist_deferred_cases.tsv", label)
}

fn deferred_context_memory_case(label: &str) -> Vec<String> {
    fixture_row_by_id("daemon_typing_assist_deferred_context_memory.tsv", label)
}

fn typing_assist_cursor_offset_after_space(current_len: usize) -> u32 {
    current_len.min(u32::MAX as usize) as u32
}

fn deferred_plan(row: &[String]) -> TextReplacement {
    text_replacement_from_fixture(row, 7, 8, 9, 10)
}

fn context_memory_plan(row: &[String]) -> TextReplacement {
    text_replacement_from_fixture(row, 4, 5, 6, 7)
}

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
fn pending_typing_assist_waits_for_space_release_before_output() {
    let row = deferred_case("space_release");
    let buffer = typed_buffer_from_semicolon_fixture(&row[1]);

    let correction =
        find_typing_assist_correction(&buffer, true, 1).expect("prepared completed word");
    let mut pending = PendingTypingAssist::new(correction);

    assert!(!pending.ready_to_apply());
    pending.note_visible_char();
    assert!(!pending.ready_to_apply());
    pending.note_separator_released();
    assert!(pending.ready_to_apply());
}

#[test]
fn hard_boundary_drops_pending_typing_assist_without_unsafe_output() {
    let row = deferred_case("space_release");
    let mut buffer = typed_buffer_from_semicolon_fixture(&row[1]);
    let correction =
        find_typing_assist_correction(&buffer, true, 1).expect("prepared completed word");
    let mut pending = Some(PendingTypingAssist::new(correction));
    let mut ignore_current_token_until_space = false;
    let mut events_since_word_start = 3;

    assert!(handle_hard_boundary_if_needed(
        KeyCode::KEY_ENTER,
        1,
        HardBoundaryContext {
            buffer: &mut buffer,
            pending_typing_assist_after_space: &mut pending,
            ignore_current_token_until_space: &mut ignore_current_token_until_space,
            events_since_word_start: &mut events_since_word_start,
            verbose: false,
        },
    ));

    assert!(pending.is_none());
    assert!(buffer.is_empty());
    assert_eq!(events_since_word_start, 0);
    assert!(!ignore_current_token_until_space);
}

#[test]
fn typing_assist_tail_can_read_left_context_without_replacing_it() {
    let row = deferred_case("context_visual_b");
    let buffer = typed_buffer_from_semicolon_fixture(&row[1]);

    let correction =
        find_typing_assist_correction(&buffer, true, 1).expect("prepared completed word");
    assert_eq!(map_original_events(&correction.events), row[3]);
    assert_eq!(correction.edit.original, row[4]);
    assert_eq!(correction.edit.replacement, row[5]);
    assert_eq!(correction.edit.plan, deferred_plan(&row));
}

#[test]
fn deferred_typing_assist_can_plan_previous_word_behind_current_tail() {
    let row = deferred_case("previous_word_behind_current");
    let buffer = typed_buffer_from_semicolon_fixture(&row[1]);

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
        row[6].parse().expect("cursor_offset"),
    );

    assert_eq!(edit.original, row[4]);
    assert_eq!(edit.replacement, row[5]);
    assert_eq!(shifted, deferred_plan(&row));
}

#[test]
fn uppercase_shift_layout_word_gets_contextual_candidate() {
    let row = deferred_case("uppercase_word");
    let input = &row[1];
    let pipeline = lay::typing_context::typing_assist_pipeline_for_context(
        true,
        lay::config::CorrectionSafety::Normal,
        &default_typing_assist_pipeline(),
        input,
    );

    assert_eq!(
        apply_typing_assist_with_pipeline(input, true, &pipeline),
        Some(row[5].clone())
    );

    let leading_shift_row = deferred_case("leading_shift_word");
    let leading_shift_input = &leading_shift_row[1];
    let leading_shift_pipeline = lay::typing_context::typing_assist_pipeline_for_context(
        true,
        lay::config::CorrectionSafety::Normal,
        &default_typing_assist_pipeline(),
        leading_shift_input,
    );
    assert_eq!(
        apply_typing_assist_with_pipeline(leading_shift_input, true, &leading_shift_pipeline),
        Some(leading_shift_row[5].clone())
    );
}

#[test]
fn deferred_typing_assist_does_not_autocorrect_three_word_tail() {
    let row = deferred_case("wide_three_word_tail");
    let buffer = typed_buffer_from_semicolon_fixture(&row[1]);

    assert!(find_typing_assist_correction(&buffer, true, 3).is_none());
}

#[test]
fn deferred_typing_assist_respects_single_word_scope() {
    let row = deferred_case("single_word_scope");
    let buffer = typed_buffer_from_semicolon_fixture(&row[1]);

    let correction =
        find_typing_assist_correction(&buffer, true, 1).expect("single-word correction");
    assert_eq!(map_original_events(&correction.events), row[3]);
    assert_eq!(correction.edit.original, row[4]);
    assert_eq!(correction.edit.replacement, row[5]);

    let shifted = lay::text_edit::offset_replacement_plan_for_cursor(
        &correction.edit.plan,
        row[6].parse().expect("cursor_offset"),
    );
    assert_eq!(shifted, deferred_plan(&row));
}

#[test]
fn deferred_typing_assist_snapshot_survives_extra_space_and_next_word() {
    let row = deferred_case("snapshot_extra_space");
    let mut buffer = typed_buffer_from_semicolon_fixture(&row[1]);

    let pending = find_typing_assist_correction(&buffer, true, row[2].parse().expect("scope"))
        .expect("prepared completed word");
    assert_eq!(map_original_events(&pending.events), row[3]);
    assert_eq!(pending.edit.original, row[4]);
    assert_eq!(pending.edit.replacement, row[5]);

    buffer.handle_space();
    push_text_as_layout(&mut buffer, "x", false);
    let shifted = lay::text_edit::offset_replacement_plan_for_cursor(
        &pending.edit.plan,
        row[6].parse().expect("cursor_offset"),
    );

    assert_eq!(shifted, deferred_plan(&row));
}

#[test]
fn deferred_typing_assist_keeps_left_context_after_previous_autoreplace() {
    let row = deferred_context_memory_case("left_context_after_autoreplace");
    assert_eq!(row.len(), 12, "bad deferred context-memory row: {row:?}");
    let mut buffer = typed_buffer_from_semicolon_fixture(&row[1]);

    let previous_events = buffer
        .last_completed_words_events(1)
        .expect("previous completed word");
    assert_eq!(map_original_events(&previous_events), row[2]);
    assert!(buffer.remember_replacement_last_word_for_replay(
        &previous_events,
        &context_memory_plan(&row),
        &row[3],
    ));

    let context_events = buffer
        .last_completed_words_events(2)
        .expect("preserved left context");
    assert_eq!(map_original_events(&context_events), row[8]);

    push_text_as_layout(&mut buffer, &row[9], false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "x", false);

    let correction =
        find_typing_assist_correction(&buffer, true, 1).expect("single-word correction");
    assert_eq!(map_original_events(&correction.events), row[10]);
    assert_eq!(correction.edit.replacement, row[11]);
}
