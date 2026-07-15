use lay::config::{default_typing_assist_pipeline, CorrectionEngine};
use lay::decoder::{
    choose_ranked_scoped_tail, decode_manual_tail, decode_typing_assist_tail,
    rank_scoped_tail_candidates, CorrectionSource, DecoderAction, ManualDecodeRequest,
};
use lay::dict::{convert, Direction};
use lay::keyboard::{map_original_events, replay_layout_decision, text_to_key_events, KeyEvent};
use lay::typing_assist::{select_typing_assist_exact, ScopedTailOptions};

#[path = "common/mod.rs"]
mod common;

fn ascii_events(text: &str) -> Vec<KeyEvent> {
    text_to_key_events(text, false).expect("decoder transition fixture must be typable")
}

fn decode_ascii_tail(text: &str, force_replay: bool) -> lay::decoder::ManualDecodeResult {
    let events = ascii_events(text);
    let original = map_original_events(&events);
    let target_is_ru = replay_layout_decision(&events).target_is_ru;
    let converted = convert(
        &original,
        if target_is_ru {
            Direction::Us2Ru
        } else {
            Direction::Ru2Us
        },
    );

    decode_manual_tail(ManualDecodeRequest {
        events: &events,
        original: &original,
        converted: &converted,
        engine: CorrectionEngine::Smart,
        force_replay,
        auto_replace: true,
        scoped_options: ScopedTailOptions {
            lem_enabled: true,
            allow_layout_auto: true,
            lem_weight: 1.0,
        },
    })
}

#[test]
fn manual_decoder_keeps_single_word_toggle_reversible() {
    for row in common::fixture_cols(include_str!(
        "fixtures/decoder_transition_manual_replay.tsv"
    )) {
        assert_eq!(row.len(), 2, "manual replay fixture must be TSV");
        let force_replay = row[1] == "true";
        let decoded = decode_ascii_tail(&row[0], force_replay);
        assert_eq!(
            decoded.action,
            DecoderAction::ReplayAll,
            "input={:?}",
            row[0]
        );
        if force_replay {
            assert!(decoded.edit.is_none(), "input={:?}", row[0]);
        }
    }
}

#[test]
fn manual_decoder_replaces_only_bad_word_in_mixed_pair() {
    let row = common::fixture_row_by_id(
        include_str!("fixtures/decoder_transition_manual_replace.tsv"),
        "mixed_pair",
    );
    let decoded = decode_ascii_tail(&row[1], false);
    assert_eq!(
        decoded.action,
        DecoderAction::ReplaceText {
            replacement: row[2].clone(),
            source: CorrectionSource::SmartText,
        }
    );
    assert_eq!(
        decoded.edit.expect("manual edit").plan,
        common::zero_edge_text_replacement(&row, 3, 4)
    );
}

#[test]
fn ranked_decoder_exposes_margin_for_mixed_pairs() {
    let row = common::fixture_row_by_id(
        include_str!("fixtures/decoder_transition_manual_replace.tsv"),
        "mixed_pair",
    );
    let events = ascii_events(&row[1]);
    let options = ScopedTailOptions {
        lem_enabled: true,
        allow_layout_auto: true,
        lem_weight: 1.0,
    };
    let ranked = rank_scoped_tail_candidates(&events, options).expect("ranked candidates");
    let chosen = choose_ranked_scoped_tail(&events, options).expect("confident decision");

    assert_eq!(ranked.best.text, row[2]);
    assert!(ranked.margin > 0.20, "margin was {}", ranked.margin);
    assert_eq!(chosen.best.text, ranked.best.text);
}

#[test]
fn ranked_decoder_handles_three_word_tail_without_retyping_good_prefix() {
    let row = common::fixture_row_by_id(
        include_str!("fixtures/decoder_transition_manual_replace.tsv"),
        "three_word_tail",
    );
    let decoded = decode_ascii_tail(&row[1], false);
    assert_eq!(
        decoded.action,
        DecoderAction::ReplaceText {
            replacement: row[2].clone(),
            source: CorrectionSource::SmartText,
        }
    );
    assert_eq!(
        decoded.edit.expect("manual edit").plan,
        common::zero_edge_text_replacement(&row, 3, 4)
    );
}

#[test]
fn ranked_decoder_keeps_ascii_context_and_flips_uppercase_current_tail() {
    let row = common::fixture_row_by_id(
        include_str!("fixtures/decoder_transition_manual_replace.tsv"),
        "uppercase_current_tail",
    );
    let decoded = decode_ascii_tail(&row[1], false);
    assert_eq!(
        decoded.action,
        DecoderAction::ReplaceText {
            replacement: row[2].clone(),
            source: CorrectionSource::SmartText,
        }
    );
    assert_eq!(
        decoded.edit.expect("manual edit").plan,
        common::zero_edge_text_replacement(&row, 3, 4)
    );
}

#[test]
fn ranked_decoder_is_disabled_without_lem_flag() {
    let row = common::fixture_row_by_id(
        include_str!("fixtures/decoder_transition_manual_replace.tsv"),
        "mixed_pair",
    );
    let events = ascii_events(&row[1]);
    assert!(rank_scoped_tail_candidates(
        &events,
        ScopedTailOptions {
            lem_enabled: false,
            allow_layout_auto: true,
            lem_weight: 1.0,
        }
    )
    .is_none());
}

#[test]
fn typing_assist_decoder_preserves_space_and_avoids_known_false_splits() {
    for input in common::fixture_lines(include_str!(
        "fixtures/decoder_transition_typing_assist_keep.txt"
    )) {
        assert_eq!(select_typing_assist_exact(&input), None, "input={input:?}");
    }
    for (input, expected) in common::fixture_cases(include_str!(
        "fixtures/decoder_transition_typing_assist_fix.tsv"
    )) {
        assert_eq!(
            select_typing_assist_exact(&input),
            Some(expected),
            "input={input:?}"
        );
    }

    let events = ascii_events("double b ");
    let plan = decode_typing_assist_tail(
        &events,
        true,
        &default_typing_assist_pipeline(),
        CorrectionSource::TypingAssist,
    );

    assert!(
        plan.is_none(),
        "lowercase visual b is ambiguous and must wait for phrase context"
    );
}
