use evdev::KeyCode;
use lay::config::{default_typing_assist_pipeline, CorrectionEngine};
use lay::decoder::{
    decode_manual_tail, decode_typing_assist_tail, CorrectionSource, DecoderAction,
    ManualDecodeRequest,
};
use lay::dict::{convert, Direction};
use lay::keyboard::{map_original_events, replay_layout_decision, KeyEvent};
use lay::text_edit::TextReplacement;
use lay::typing_assist::{apply_typing_assist_exact, ScopedTailOptions};

fn ev(keycode: KeyCode, layout_is_ru: bool) -> KeyEvent {
    KeyEvent {
        keycode: keycode.code(),
        shift: false,
        layout_is_ru,
    }
}

fn ascii_events(text: &str) -> Vec<KeyEvent> {
    text.chars()
        .map(|ch| {
            let key = match ch {
                'a' | 'A' => KeyCode::KEY_A,
                'b' | 'B' => KeyCode::KEY_B,
                'c' | 'C' => KeyCode::KEY_C,
                'd' | 'D' => KeyCode::KEY_D,
                'e' | 'E' => KeyCode::KEY_E,
                'f' | 'F' => KeyCode::KEY_F,
                'g' | 'G' => KeyCode::KEY_G,
                'h' | 'H' => KeyCode::KEY_H,
                'i' | 'I' => KeyCode::KEY_I,
                'j' | 'J' => KeyCode::KEY_J,
                'k' | 'K' => KeyCode::KEY_K,
                'l' | 'L' => KeyCode::KEY_L,
                'm' | 'M' => KeyCode::KEY_M,
                'n' | 'N' => KeyCode::KEY_N,
                'o' | 'O' => KeyCode::KEY_O,
                'p' | 'P' => KeyCode::KEY_P,
                'q' | 'Q' => KeyCode::KEY_Q,
                'r' | 'R' => KeyCode::KEY_R,
                's' | 'S' => KeyCode::KEY_S,
                't' | 'T' => KeyCode::KEY_T,
                'u' | 'U' => KeyCode::KEY_U,
                'v' | 'V' => KeyCode::KEY_V,
                'w' | 'W' => KeyCode::KEY_W,
                'x' | 'X' => KeyCode::KEY_X,
                'y' | 'Y' => KeyCode::KEY_Y,
                'z' | 'Z' => KeyCode::KEY_Z,
                ' ' => KeyCode::KEY_SPACE,
                '-' => KeyCode::KEY_MINUS,
                ';' => KeyCode::KEY_SEMICOLON,
                other => panic!("unsupported test char {other:?}"),
            };
            ev(key, false)
        })
        .collect()
}

fn decode_ascii_tail(text: &str, force_replay: bool) -> DecoderAction {
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
        },
    })
    .action
}

#[test]
fn manual_decoder_keeps_single_word_toggle_reversible() {
    assert_eq!(decode_ascii_tail("good", true), DecoderAction::ReplayAll);
    assert_eq!(decode_ascii_tail("good", false), DecoderAction::ReplayAll);
    assert_eq!(decode_ascii_tail("ntrcn", true), DecoderAction::ReplayAll);
}

#[test]
fn manual_decoder_replaces_only_bad_word_in_mixed_pair() {
    assert_eq!(
        decode_ascii_tail("good ntrcn", false),
        DecoderAction::ReplaceText {
            replacement: "good текст".to_string(),
            source: CorrectionSource::SmartText,
        }
    );
}

#[test]
fn typing_assist_decoder_preserves_space_and_avoids_known_false_splits() {
    assert_eq!(apply_typing_assist_exact("я язык "), None);
    assert_eq!(apply_typing_assist_exact("про сою "), None);
    assert_eq!(apply_typing_assist_exact("15р-16р "), None);
    assert_eq!(
        apply_typing_assist_exact("у насесть "),
        Some("у нас есть ".to_string())
    );

    let events = ascii_events("double b ");
    let plan = decode_typing_assist_tail(
        &events,
        true,
        &default_typing_assist_pipeline(),
        CorrectionSource::TypingAssist,
    )
    .expect("visual b replacement");

    assert_eq!(plan.replacement, "double и ");
    assert_eq!(
        plan.plan,
        TextReplacement {
            move_left: 1,
            backspaces: 1,
            insert: "и".to_string(),
            move_right: 1,
        }
    );
}
