use super::*;
use crate::keyboard::KeyEvent;
use evdev::KeyCode;

fn ascii_events(text: &str) -> Vec<KeyEvent> {
    text.chars()
        .filter_map(|ch| {
            let key = match ch {
                'a' => KeyCode::KEY_A,
                'c' => KeyCode::KEY_C,
                'd' => KeyCode::KEY_D,
                'e' => KeyCode::KEY_E,
                'g' => KeyCode::KEY_G,
                'n' => KeyCode::KEY_N,
                'o' => KeyCode::KEY_O,
                'r' => KeyCode::KEY_R,
                't' => KeyCode::KEY_T,
                'x' => KeyCode::KEY_X,
                ' ' => KeyCode::KEY_SPACE,
                _ => return None,
            };
            Some(KeyEvent {
                keycode: key.code(),
                shift: false,
                layout_is_ru: false,
            })
        })
        .collect()
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
