use super::*;
use crate::keyboard::text_to_key_events;

fn ascii_events(text: &str) -> Vec<KeyEvent> {
    text_to_key_events(text, false).expect("engine fixture must be typable")
}

fn ru_events(text: &str) -> Vec<KeyEvent> {
    text_to_key_events(text, true).expect("engine fixture must be typable")
}

#[test]
fn manual_engine_is_platform_neutral_for_replay() {
    let events = ascii_events("good");
    let decision = decide_manual_correction(
        ManualCorrectionInput {
            events: &events,
            original: "good",
            converted: "пщщв",
        },
        ManualCorrectionPolicy {
            engine: CorrectionEngine::Smart,
            force_replay: true,
            auto_replace: true,
            scoped_options: ScopedTailOptions::default(),
        },
    );

    assert_eq!(decision.action, DecoderAction::ReplayAll);
    assert!(decision.replay_target_is_ru);
    assert_eq!(decision.output_text, "пщщв");
}

#[test]
fn manual_engine_keeps_good_prefix_for_smart_text() {
    let events = ascii_events("good ntrcn");
    let decision = decide_manual_correction(
        ManualCorrectionInput {
            events: &events,
            original: "good ntrcn",
            converted: "пщщв текст",
        },
        ManualCorrectionPolicy {
            engine: CorrectionEngine::Smart,
            force_replay: false,
            auto_replace: true,
            scoped_options: ScopedTailOptions {
                lem_enabled: true,
                allow_layout_auto: true,
            },
        },
    );

    assert_eq!(decision.output_text, "good текст");
    assert!(decision.output_target_is_ru);
    assert!(decision.edit.is_some());
}

#[test]
fn manual_engine_replays_single_cyrillic_layout_word_to_ascii() {
    let events = ru_events("тфтвф");
    let decision = decide_manual_correction(
        ManualCorrectionInput {
            events: &events,
            original: "тфтвф",
            converted: "nanda",
        },
        ManualCorrectionPolicy {
            engine: CorrectionEngine::Smart,
            force_replay: false,
            auto_replace: true,
            scoped_options: ScopedTailOptions {
                lem_enabled: true,
                allow_layout_auto: true,
            },
        },
    );

    assert_eq!(decision.output_text, "nanda");
    assert!(!decision.output_target_is_ru);
}
