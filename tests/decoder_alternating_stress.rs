mod common;

use common::fixture_tagged_cases;
use lay::config::CorrectionEngine;
use lay::decoder::DecoderAction;
use lay::engine::{decide_manual_correction, ManualCorrectionInput, ManualCorrectionPolicy};
use lay::keyboard::{
    map_events_to_layout, map_original_events, replay_layout_decision, text_to_key_events,
};

const DECODER_ALTERNATING_STRESS_CASES: &str =
    include_str!("fixtures/decoder_alternating_stress.tsv");

fn decode_manual_visible_tail(input: &str) -> String {
    let events = text_to_key_events(input, false).expect("fixture must map to key events");
    let original = map_original_events(&events);
    let replay = replay_layout_decision(&events);
    let converted = map_events_to_layout(&events, replay.target_is_ru);
    let decision = decide_manual_correction(
        ManualCorrectionInput {
            events: &events,
            original: &original,
            converted: &converted,
        },
        ManualCorrectionPolicy {
            engine: CorrectionEngine::Smart,
            force_replay: false,
            auto_replace: true,
        },
    );

    match decision.action {
        DecoderAction::KeepOriginal => original,
        DecoderAction::ReplayAll => converted,
        DecoderAction::ReplaceText { replacement, .. } => replacement,
    }
}

#[test]
fn manual_decoder_alternating_language_cases_choose_tokenwise_tail() {
    for (class, input, expected) in fixture_tagged_cases(DECODER_ALTERNATING_STRESS_CASES) {
        let got = decode_manual_visible_tail(&input);
        assert_eq!(got, expected, "class={class} input={input:?}");
        assert_eq!(
            expected.split_whitespace().count(),
            got.split_whitespace().count(),
            "class={class} word count changed: got={got:?}"
        );
    }
}
