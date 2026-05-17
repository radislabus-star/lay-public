use evdev::KeyCode;
use lay::config::{default_typing_assist_pipeline, CorrectionEngine};
use lay::decoder::{
    choose_ranked_scoped_tail, decode_manual_tail, decode_typing_assist_tail,
    rank_scoped_tail_candidates, CorrectionSource, DecoderAction, ManualDecodeRequest,
};
use lay::dict::{convert, Direction};
use lay::keyboard::{map_original_events, replay_layout_decision, KeyEvent};
use lay::text_edit::TextReplacement;
use lay::typing_assist::{apply_typing_assist_exact, ScopedTailOptions};

fn ascii_events(text: &str) -> Vec<KeyEvent> {
    text.chars()
        .map(|ch| {
            let (key, layout_is_ru, shift) = match ch {
                'a'..='z' | 'A'..='Z' => {
                    let key = match ch.to_ascii_lowercase() {
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
                        _ => unreachable!(),
                    };
                    (key, false, ch.is_ascii_uppercase())
                }
                'а' | 'А' => (KeyCode::KEY_F, true, ch.is_uppercase()),
                'д' | 'Д' => (KeyCode::KEY_L, true, ch.is_uppercase()),
                'е' | 'Е' => (KeyCode::KEY_T, true, ch.is_uppercase()),
                'й' | 'Й' => (KeyCode::KEY_Q, true, ch.is_uppercase()),
                'л' | 'Л' => (KeyCode::KEY_K, true, ch.is_uppercase()),
                ' ' => (KeyCode::KEY_SPACE, false, false),
                '-' => (KeyCode::KEY_MINUS, false, false),
                ';' => (KeyCode::KEY_SEMICOLON, false, false),
                other => panic!("unsupported test char {other:?}"),
            };
            KeyEvent {
                keycode: key.code(),
                shift,
                layout_is_ru,
            }
        })
        .collect()
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
        },
    })
}

#[test]
fn manual_decoder_keeps_single_word_toggle_reversible() {
    assert_eq!(
        decode_ascii_tail("good", true).action,
        DecoderAction::ReplayAll
    );
    assert_eq!(
        decode_ascii_tail("good", false).action,
        DecoderAction::ReplayAll
    );
    assert_eq!(
        decode_ascii_tail("ntrcn", true).action,
        DecoderAction::ReplayAll
    );
    assert!(decode_ascii_tail("good", true).edit.is_none());
}

#[test]
fn manual_decoder_replaces_only_bad_word_in_mixed_pair() {
    let decoded = decode_ascii_tail("good ntrcn", false);
    assert_eq!(
        decoded.action,
        DecoderAction::ReplaceText {
            replacement: "good текст".to_string(),
            source: CorrectionSource::SmartText,
        }
    );
    assert_eq!(
        decoded.edit.expect("manual edit").plan,
        TextReplacement {
            move_left: 0,
            backspaces: 5,
            insert: "текст".to_string(),
            move_right: 0,
        }
    );
}

#[test]
fn ranked_decoder_exposes_margin_for_mixed_pairs() {
    let events = ascii_events("good ntrcn");
    let options = ScopedTailOptions {
        lem_enabled: true,
        allow_layout_auto: true,
    };
    let ranked = rank_scoped_tail_candidates(&events, options).expect("ranked candidates");
    let chosen = choose_ranked_scoped_tail(&events, options).expect("confident decision");

    assert_eq!(ranked.best.text, "good текст");
    assert!(ranked.margin > 0.20, "margin was {}", ranked.margin);
    assert_eq!(chosen.best.text, ranked.best.text);
}

#[test]
fn ranked_decoder_handles_three_word_tail_without_retyping_good_prefix() {
    let decoded = decode_ascii_tail("hello good ntrcn", false);
    assert_eq!(
        decoded.action,
        DecoderAction::ReplaceText {
            replacement: "hello good текст".to_string(),
            source: CorrectionSource::SmartText,
        }
    );
    assert_eq!(
        decoded.edit.expect("manual edit").plan,
        TextReplacement {
            move_left: 0,
            backspaces: 5,
            insert: "текст".to_string(),
            move_right: 0,
        }
    );
}

#[test]
fn ranked_decoder_keeps_ascii_context_and_flips_uppercase_current_tail() {
    let decoded = decode_ascii_tail("делай KDE", false);
    assert_eq!(
        decoded.action,
        DecoderAction::ReplaceText {
            replacement: "делай ЛВУ".to_string(),
            source: CorrectionSource::SmartText,
        }
    );
    assert_eq!(
        decoded.edit.expect("manual edit").plan,
        TextReplacement {
            move_left: 0,
            backspaces: 3,
            insert: "ЛВУ".to_string(),
            move_right: 0,
        }
    );
}

#[test]
fn ranked_decoder_is_disabled_without_lem_flag() {
    let events = ascii_events("good ntrcn");
    assert!(rank_scoped_tail_candidates(
        &events,
        ScopedTailOptions {
            lem_enabled: false,
            allow_layout_auto: true,
        }
    )
    .is_none());
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
            move_left: 0,
            backspaces: 2,
            insert: "и ".to_string(),
            move_right: 0,
        }
    );
}
