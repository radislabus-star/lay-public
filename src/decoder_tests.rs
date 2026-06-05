use super::*;
use crate::config::{default_typing_assist_pipeline, CorrectionEngine};
use crate::keyboard::KeyEvent;
use crate::text_edit::{committed_separator_is_preserved, replacement_plan_matches};
use crate::typing_assist::ScopedTailOptions;
use crate::typing_assist_test_fixtures::{
    first_fixture_row_from_str, fixture_row_by_id_from_str, fixture_rows_from_str,
    text_replacement, zero_edge_text_replacement,
};

const MANUAL_LEM_CASES: &str = include_str!("../tests/fixtures/decoder_manual_lem_cases.tsv");
const MANUAL_REPLAY_CASES: &str =
    include_str!("../tests/fixtures/decoder_transition_manual_replay.tsv");
const MANUAL_REPLACE_CASES: &str =
    include_str!("../tests/fixtures/decoder_transition_manual_replace.tsv");
const VISUAL_B_CASES: &str = include_str!("../tests/fixtures/decoder_transition_visual_b.tsv");
const CONTEXT_VISUAL_B_CASES: &str = include_str!("../tests/fixtures/decoder_context_visual_b.tsv");

fn events_for_ascii(text: &str) -> Vec<KeyEvent> {
    crate::keyboard::text_to_key_events(text, false).expect("decoder fixture must be typable")
}

#[test]
fn manual_decoder_keeps_replay_as_explicit_user_command() {
    let row = fixture_rows_from_str(MANUAL_REPLAY_CASES)
        .into_iter()
        .find(|row| {
            row.first().is_some_and(|value| value == "good")
                && row.get(1).is_some_and(|value| value == "true")
        })
        .expect("forced replay fixture");
    let events = events_for_ascii(&row[0]);
    let result = decode_manual_tail(ManualDecodeRequest {
        events: &events,
        original: &row[0],
        converted: &crate::dict::convert(&row[0], crate::dict::Direction::Us2Ru),
        engine: CorrectionEngine::Smart,
        force_replay: true,
        auto_replace: true,
        scoped_options: ScopedTailOptions::default(),
    });

    assert_eq!(result.action, DecoderAction::ReplayAll);
}

#[test]
fn manual_decoder_does_not_apply_visual_b_auto_replace_to_replay() {
    let row = fixture_row_by_id_from_str(MANUAL_REPLAY_CASES, "b");
    let events = events_for_ascii(&row[0]);
    let converted = crate::dict::convert(&row[0], crate::dict::Direction::Us2Ru);
    let result = decode_manual_tail(ManualDecodeRequest {
        events: &events,
        original: &row[0],
        converted: &converted,
        engine: CorrectionEngine::Replay,
        force_replay: false,
        auto_replace: true,
        scoped_options: ScopedTailOptions::default(),
    });

    assert_eq!(converted, "и");
    assert_eq!(result.action, DecoderAction::ReplayAll);
    assert!(result.edit.is_none());
}

#[test]
fn manual_decoder_uses_smart_tail_for_mixed_two_words() {
    let row = fixture_row_by_id_from_str(MANUAL_REPLACE_CASES, "mixed_pair");
    let events = events_for_ascii(&row[1]);
    let result = decode_manual_tail(ManualDecodeRequest {
        events: &events,
        original: &row[1],
        converted: &crate::dict::convert(&row[1], crate::dict::Direction::Us2Ru),
        engine: CorrectionEngine::Smart,
        force_replay: false,
        auto_replace: false,
        scoped_options: ScopedTailOptions {
            lem_enabled: true,
            allow_layout_auto: true,
        },
    });

    assert_eq!(
        result.action,
        DecoderAction::ReplaceText {
            replacement: row[2].clone(),
            source: CorrectionSource::SmartText,
        }
    );
    assert_eq!(
        result.edit.expect("manual edit").plan,
        zero_edge_text_replacement(&row, 3, 4)
    );
}

#[test]
fn manual_decoder_lem_fixture_cases_choose_expected_tail() {
    for (label, original, converted, expected) in manual_lem_fixture_cases() {
        let events = events_for_ascii(&original);
        let result = decode_manual_tail(ManualDecodeRequest {
            events: &events,
            original: &original,
            converted: &converted,
            engine: CorrectionEngine::Smart,
            force_replay: false,
            auto_replace: false,
            scoped_options: ScopedTailOptions {
                lem_enabled: true,
                allow_layout_auto: true,
            },
        });

        assert_eq!(
            result.action,
            DecoderAction::ReplaceText {
                replacement: expected,
                source: CorrectionSource::SmartText,
            },
            "case={label}"
        );
    }
}

#[test]
fn typing_assist_decoder_reemits_committed_space_boundary() {
    let row = first_fixture_row_from_str(VISUAL_B_CASES);
    let events = events_for_ascii(&row[0]);
    let plan = decode_typing_assist_tail(
        &events,
        true,
        &default_typing_assist_pipeline(),
        CorrectionSource::TypingAssist,
    )
    .expect("assist plan");

    assert_eq!(plan.replacement, row[1]);
    assert!(committed_separator_is_preserved(
        &plan.original,
        &plan.replacement
    ));
    assert!(replacement_plan_matches(
        &plan.original,
        &plan.replacement,
        &plan.plan
    ));
    assert!(plan.plan_matches_replacement());
    assert!(plan.preserves_committed_separator());
    assert_eq!(plan.verified_plan_for_cursor(0), Some(plan.plan.clone()));
    assert_eq!(
        plan.verified_plan_for_cursor(3),
        Some(text_replacement(4, 1, &row[2], 4))
    );
    assert_eq!(plan.plan, text_replacement(1, 1, &row[2], 1));
    assert!(plan.source.needs_undo_checkpoint());
}

#[test]
fn committed_decoder_plan_normalizes_replacement_and_preserves_screen_state() {
    let plan = DecoderEditPlan::committed_tail(
        CorrectionTrigger::AfterSpace,
        "aa ",
        "bb",
        CorrectionSource::TypingAssist,
    )
    .expect("committed plan");

    assert_eq!(plan.replacement, "bb ");
    assert!(plan.preserves_committed_separator());
    assert!(plan.plan_matches_replacement());
    assert!(replacement_plan_matches(
        &plan.original,
        &plan.replacement,
        &plan.plan
    ));
    assert_eq!(
        crate::text_edit::apply_replacement_plan_to_text(&plan.original, &plan.plan),
        plan.replacement
    );
}

fn manual_lem_fixture_cases() -> impl Iterator<Item = (String, String, String, String)> {
    fixture_rows_from_str(MANUAL_LEM_CASES)
        .into_iter()
        .map(|row| {
            assert_eq!(row.len(), 4, "bad decoder manual LEM fixture row");
            (
                row[0].clone(),
                row[1].clone(),
                row[2].clone(),
                row[3].clone(),
            )
        })
}

#[test]
fn typing_assist_context_decoder_keeps_edit_to_last_tail() {
    let row = first_fixture_row_from_str(CONTEXT_VISUAL_B_CASES);
    let events = events_for_ascii(&row[0]);
    let plan = decode_typing_assist_tail_with_context(
        &events,
        &row[1],
        true,
        &default_typing_assist_pipeline(),
        CorrectionSource::TypingAssist,
    )
    .expect("assist plan");

    assert_eq!(plan.original, row[0]);
    assert_eq!(plan.replacement, row[2]);
    assert_eq!(plan.plan, text_replacement(1, 1, &row[3], 1));
    assert!(plan.preserves_committed_separator());
}
