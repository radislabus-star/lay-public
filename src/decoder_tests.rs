use super::*;
use crate::config::{default_typing_assist_pipeline, CorrectionEngine};
use crate::keyboard::KeyEvent;
use crate::text_edit::{
    committed_separator_is_preserved, replacement_plan_matches, TextReplacement,
};
use crate::typing_assist::ScopedTailOptions;
use evdev::KeyCode;

fn ev(keycode: KeyCode, layout_is_ru: bool) -> KeyEvent {
    KeyEvent {
        keycode: keycode.code(),
        shift: false,
        layout_is_ru,
    }
}

fn events_for_ascii(text: &str) -> Vec<KeyEvent> {
    text.chars()
        .filter_map(|ch| {
            let key = match ch {
                'a' => KeyCode::KEY_A,
                'b' => KeyCode::KEY_B,
                'c' => KeyCode::KEY_C,
                'd' => KeyCode::KEY_D,
                'e' => KeyCode::KEY_E,
                'f' => KeyCode::KEY_F,
                'g' => KeyCode::KEY_G,
                'h' => KeyCode::KEY_H,
                'i' => KeyCode::KEY_I,
                'j' => KeyCode::KEY_J,
                'k' => KeyCode::KEY_K,
                'l' => KeyCode::KEY_L,
                'm' => KeyCode::KEY_M,
                'n' => KeyCode::KEY_N,
                'o' => KeyCode::KEY_O,
                'p' => KeyCode::KEY_P,
                'q' => KeyCode::KEY_Q,
                'r' => KeyCode::KEY_R,
                's' => KeyCode::KEY_S,
                't' => KeyCode::KEY_T,
                'u' => KeyCode::KEY_U,
                'v' => KeyCode::KEY_V,
                'w' => KeyCode::KEY_W,
                'x' => KeyCode::KEY_X,
                'y' => KeyCode::KEY_Y,
                'z' => KeyCode::KEY_Z,
                ' ' => KeyCode::KEY_SPACE,
                _ => return None,
            };
            Some(ev(key, false))
        })
        .collect()
}

#[test]
fn manual_decoder_keeps_replay_as_explicit_user_command() {
    let events = events_for_ascii("good");
    let result = decode_manual_tail(ManualDecodeRequest {
        events: &events,
        original: "good",
        converted: "пщщв",
        engine: CorrectionEngine::Smart,
        force_replay: true,
        auto_replace: true,
        scoped_options: ScopedTailOptions::default(),
    });

    assert_eq!(result.action, DecoderAction::ReplayAll);
}

#[test]
fn manual_decoder_uses_smart_tail_for_mixed_two_words() {
    let events = events_for_ascii("good ntrcn");
    let result = decode_manual_tail(ManualDecodeRequest {
        events: &events,
        original: "good ntrcn",
        converted: "пщщв текст",
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
            replacement: "good текст".to_string(),
            source: CorrectionSource::SmartText,
        }
    );
    assert_eq!(
        result.edit.expect("manual edit").plan,
        TextReplacement {
            move_left: 0,
            backspaces: 5,
            insert: "текст".to_string(),
            move_right: 0,
        }
    );
}

#[test]
fn typing_assist_decoder_reemits_committed_space_boundary() {
    let events = events_for_ascii("double b ");
    let plan = decode_typing_assist_tail(
        &events,
        true,
        &default_typing_assist_pipeline(),
        CorrectionSource::TypingAssist,
    )
    .expect("assist plan");

    assert_eq!(plan.replacement, "double и ");
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
    assert_eq!(
        plan.plan,
        TextReplacement {
            move_left: 1,
            backspaces: 1,
            insert: "и".to_string(),
            move_right: 1,
        }
    );
    assert!(plan.source.needs_undo_checkpoint());
}

#[test]
fn typing_assist_context_decoder_keeps_edit_to_last_tail() {
    let events = events_for_ascii("b ");
    let plan = decode_typing_assist_tail_with_context(
        &events,
        "css b ",
        true,
        &default_typing_assist_pipeline(),
        CorrectionSource::TypingAssist,
    )
    .expect("assist plan");

    assert_eq!(plan.original, "b ");
    assert_eq!(plan.replacement, "и ");
    assert_eq!(
        plan.plan,
        TextReplacement {
            move_left: 1,
            backspaces: 1,
            insert: "и".to_string(),
            move_right: 1,
        }
    );
    assert!(plan.preserves_committed_separator());
}
