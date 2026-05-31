use crate::config::{
    default_typing_assist_pipeline, typing_assist_pipeline_for_policy, CorrectionSafety,
};
use crate::decoder::{decode_typing_assist_tail, CorrectionSource};
use crate::keyboard::{text_to_key_events, KeyEvent};
use crate::typing_assist_test_fixtures::{fixture_lines_from_str, fixture_rows_from_str};
use crate::typing_context::{
    completed_tail_context, should_enable_ascii_to_ru_layout, typing_assist_pipeline_for_context,
};
use crate::word_buffer::WordBuffer;

const CONTEXT_ENABLED_CASES: &str = include_str!("../tests/fixtures/typing_context_enabled.txt");
const CONTEXT_DISABLED_CASES: &str = include_str!("../tests/fixtures/typing_context_disabled.txt");
const CONTEXT_WINDOW_CASES: &str =
    include_str!("../tests/fixtures/daemon_typing_assist_context_window.tsv");

fn push_visible_text(buffer: &mut WordBuffer, text: &str) {
    for segment in text.split_inclusive(' ') {
        let word = segment.trim_end_matches(' ');
        let layout_is_ru = !word.is_ascii();
        push_text_as_layout(buffer, segment, layout_is_ru);
    }
}

fn push_text_as_layout(buffer: &mut WordBuffer, text: &str, layout_is_ru: bool) {
    for event in text_key_events(text, layout_is_ru) {
        if event.keycode == evdev::KeyCode::KEY_SPACE.code() {
            buffer.handle_space();
        } else {
            buffer.push(event);
        }
    }
}

fn text_key_events(text: &str, layout_is_ru: bool) -> Vec<KeyEvent> {
    text_to_key_events(text, layout_is_ru).unwrap_or_else(|| {
        panic!("failed to create key events for {text:?} layout_is_ru={layout_is_ru}")
    })
}

fn pipeline_is_sorted(pipeline: &[crate::config::TypingAssistRuleConfig]) -> bool {
    pipeline.windows(2).all(|pair| {
        pair[0].priority < pair[1].priority
            || (pair[0].priority == pair[1].priority && pair[0].id <= pair[1].id)
    })
}

#[test]
fn russian_context_enables_ascii_to_ru_layout_rule() {
    let enabled_cases = fixture_lines_from_str(CONTEXT_ENABLED_CASES);
    let mut enabled_cases = enabled_cases.iter();
    let first_context = enabled_cases.next().expect("enabled context fixture");
    let pipeline = typing_assist_pipeline_for_context(
        true,
        CorrectionSafety::Normal,
        &default_typing_assist_pipeline(),
        first_context,
    );

    assert!(pipeline
        .iter()
        .find(|rule| rule.id == "layout_en_to_ru")
        .is_some_and(|rule| !rule.enabled));
    assert!(pipeline
        .iter()
        .find(|rule| rule.id == "contextual_layout_en_to_ru")
        .is_some_and(|rule| rule.enabled));
    assert!(pipeline_is_sorted(&pipeline));
    assert!(should_enable_ascii_to_ru_layout(first_context));
    for context in enabled_cases {
        assert!(should_enable_ascii_to_ru_layout(context), "{context:?}");
    }
}

#[test]
fn no_context_or_english_context_keeps_ascii_to_ru_disabled() {
    let base = typing_assist_pipeline_for_policy(
        true,
        CorrectionSafety::Normal,
        &default_typing_assist_pipeline(),
    );
    assert!(base
        .iter()
        .find(|rule| rule.id == "layout_en_to_ru")
        .is_some_and(|rule| !rule.enabled));

    for context in fixture_lines_from_str(CONTEXT_DISABLED_CASES) {
        assert!(
            !should_enable_ascii_to_ru_layout(&context),
            "context={context:?}"
        );
        let pipeline = typing_assist_pipeline_for_context(
            true,
            CorrectionSafety::Normal,
            &default_typing_assist_pipeline(),
            &context,
        );
        assert!(
            pipeline
                .iter()
                .all(|rule| rule.id != "contextual_layout_en_to_ru"),
            "context={context:?}"
        );
    }
}

#[test]
fn explicit_user_disabled_rule_stays_disabled() {
    let context = fixture_lines_from_str(CONTEXT_ENABLED_CASES)
        .into_iter()
        .next()
        .expect("enabled context fixture");
    let mut configured = default_typing_assist_pipeline();
    configured
        .iter_mut()
        .find(|rule| rule.id == "layout_en_to_ru")
        .expect("layout_en_to_ru rule")
        .enabled = false;

    let pipeline =
        typing_assist_pipeline_for_context(true, CorrectionSafety::Normal, &configured, &context);

    assert!(pipeline
        .iter()
        .all(|rule| rule.id != "contextual_layout_en_to_ru"));
}

#[test]
fn completed_tail_context_keeps_left_russian_context() {
    for row in fixture_rows_from_str(CONTEXT_WINDOW_CASES) {
        assert_eq!(row.len(), 4, "context window fixture must have 4 columns");

        let mut buffer = WordBuffer::new();
        push_visible_text(&mut buffer, &row[0]);
        push_text_as_layout(&mut buffer, &row[1], false);

        let events = buffer
            .last_completed_words_events(1)
            .expect("completed last word");
        let context = completed_tail_context(&buffer, 1, &events);
        assert_eq!(context, row[2]);

        let pipeline = typing_assist_pipeline_for_context(
            true,
            CorrectionSafety::Normal,
            &default_typing_assist_pipeline(),
            &context,
        );
        let edit =
            decode_typing_assist_tail(&events, true, &pipeline, CorrectionSource::TypingAssist)
                .expect("last word correction");

        assert_eq!(edit.replacement, row[3]);
    }
}
