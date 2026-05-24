use super::*;
use evdev::KeyCode;
use lay::config::{
    default_typing_assist_pipeline, default_typing_assist_rules, normalize_typing_assist_pipeline,
    typing_assist_pipeline_for_auto_replace, CorrectionEngine, LayConfig, TypingAssistRuleConfig,
};
use lay::correction::Correction;
use lay::decoder::{decode_manual_tail, CorrectionSource, DecoderAction, ManualDecodeRequest};
use lay::desktop::{
    is_ru_layout_id, normalize_layout_id, parse_setxkbmap_layout, resolve_layout_backend,
    LayoutBackend,
};
use lay::keyboard::{
    is_cyrillic_letter, is_layout_decision_key, keycode_to_ru_char, keycode_to_us_char,
    map_events_to_layout, map_opposite_events, map_original_events, preferred_layout_for_text,
    replay_layout_decision, split_event_words, text_to_uinput_runs, KeyEvent, ReplayLayoutDecision,
};
use lay::text_edit::{plan_committed_tail_replacement, plan_text_replacement, TextReplacement};
use lay::typing_assist::{
    apply_typing_assist_with_pipeline, are_ru_keyboard_neighbors,
    correct_duplicate_layout_prefix_on_ascii_token, correct_extra_letters, correct_missing_letter,
    correct_wrong_layout_ascii_technical_token, decide_completed_scope_word, decide_correction,
    decide_scoped_tail_correction, decide_scoped_tail_correction_with_lem, effective_replace_words,
    is_ascii_technical_token, is_known_russian_word_or_form, promoted_replacement_for_token,
    remember_promoted_replacement, repair_cyrillic_prefix_before_ascii_tail,
    russian_generated_form_dictionary, scoped_tail_lem_candidates,
    should_force_replay_for_short_fragment, should_keep_plain_cyrillic_before_ascii_technical,
    split_edge_whitespace, split_ws_segments, ScopedTailOptions,
};
use lay::word_buffer::{UserLearningCorrection, WordBuffer};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use std::sync::Once;
use std::time::{Duration, Instant};

fn seed_test_replacements() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        for row in fixture_rows("daemon_seed_replacements.tsv") {
            assert_eq!(row.len(), 2, "seed replacement fixture must be TSV");
            remember_promoted_replacement(&row[0], &row[1]);
        }
    });
}

fn fixture_rows(name: &str) -> Vec<Vec<String>> {
    let path = fixture_path(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture {path:?}: {err}"))
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| line.split('\t').map(decode_fixture_field).collect())
        .collect()
}

fn fixture_lines(name: &str) -> Vec<String> {
    let path = fixture_path(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture {path:?}: {err}"))
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(decode_fixture_field)
        .collect()
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn decode_fixture_field(value: &str) -> String {
    value.replace("\\s", " ")
}

fn apply_typing_assist_exact(text: &str) -> Option<String> {
    seed_test_replacements();
    lay::typing_assist::apply_typing_assist_exact(text)
}

fn apply_typing_assist(text: &str, allow_layout_auto: bool) -> Option<String> {
    seed_test_replacements();
    lay::typing_assist::apply_typing_assist(text, allow_layout_auto)
}

fn apply_auto_replace(original: &str, target: &str) -> Option<String> {
    seed_test_replacements();
    lay::typing_assist::apply_auto_replace(original, target)
}

fn key_event(key: KeyCode, layout_is_ru: bool) -> KeyEvent {
    KeyEvent {
        keycode: key.code(),
        shift: false,
        layout_is_ru,
    }
}

fn push_keys(buffer: &mut WordBuffer, keys: &[KeyCode], layout_is_ru: bool) {
    for key in keys {
        buffer.push(key_event(*key, layout_is_ru));
    }
}

fn key_events(keys: &[KeyCode], layout_is_ru: bool) -> Vec<KeyEvent> {
    keys.iter()
        .map(|key| key_event(*key, layout_is_ru))
        .collect()
}

#[path = "tests/config_contract.rs"]
mod config_contract;
#[path = "tests/enter_autocorrect.rs"]
mod enter_autocorrect;
#[path = "tests/layout_backend.rs"]
mod layout_backend;
#[path = "tests/learning.rs"]
mod learning;
#[path = "tests/learning_log.rs"]
mod learning_log;
#[path = "tests/runtime_state.rs"]
mod runtime_state;
#[path = "tests/scoped_tail.rs"]
mod scoped_tail;
#[path = "tests/text_output_contract.rs"]
mod text_output_contract;
#[path = "tests/typing_assist_rules.rs"]
mod typing_assist_rules;

fn ascii_hyphen_token_keycodes() -> [KeyCode; 5] {
    [
        KeyCode::KEY_W,
        KeyCode::KEY_I,
        KeyCode::KEY_MINUS,
        KeyCode::KEY_F,
        KeyCode::KEY_I,
    ]
}

fn typing_pipeline_with_disabled(disabled: &[&str]) -> Vec<TypingAssistRuleConfig> {
    default_typing_assist_pipeline()
        .into_iter()
        .map(|mut rule| {
            if disabled.iter().any(|id| *id == rule.id) {
                rule.enabled = false;
            }
            rule
        })
        .collect()
}

fn typing_pipeline_with_only(enabled: &str) -> Vec<TypingAssistRuleConfig> {
    default_typing_assist_pipeline()
        .into_iter()
        .map(|mut rule| {
            rule.enabled = rule.id == enabled;
            rule
        })
        .collect()
}

fn typing_pipeline_with_first(first: &str) -> Vec<TypingAssistRuleConfig> {
    let mut rules = default_typing_assist_pipeline();
    for rule in &mut rules {
        rule.priority += 10;
        if rule.id == first {
            rule.priority = 1;
        }
    }
    rules
}

fn push_key_events(buffer: &mut WordBuffer, keys: &[(KeyCode, bool)], layout_is_ru: bool) {
    for (key, shift) in keys {
        buffer.push(KeyEvent {
            keycode: key.code(),
            shift: *shift,
            layout_is_ru,
        });
    }
}

fn text_key_event(ch: char, layout_is_ru: bool) -> KeyEvent {
    const KEYS: &[KeyCode] = &[
        KeyCode::KEY_A,
        KeyCode::KEY_B,
        KeyCode::KEY_C,
        KeyCode::KEY_D,
        KeyCode::KEY_E,
        KeyCode::KEY_F,
        KeyCode::KEY_G,
        KeyCode::KEY_H,
        KeyCode::KEY_I,
        KeyCode::KEY_J,
        KeyCode::KEY_K,
        KeyCode::KEY_L,
        KeyCode::KEY_M,
        KeyCode::KEY_N,
        KeyCode::KEY_O,
        KeyCode::KEY_P,
        KeyCode::KEY_Q,
        KeyCode::KEY_R,
        KeyCode::KEY_S,
        KeyCode::KEY_T,
        KeyCode::KEY_U,
        KeyCode::KEY_V,
        KeyCode::KEY_W,
        KeyCode::KEY_X,
        KeyCode::KEY_Y,
        KeyCode::KEY_Z,
        KeyCode::KEY_1,
        KeyCode::KEY_2,
        KeyCode::KEY_3,
        KeyCode::KEY_4,
        KeyCode::KEY_5,
        KeyCode::KEY_6,
        KeyCode::KEY_7,
        KeyCode::KEY_8,
        KeyCode::KEY_9,
        KeyCode::KEY_0,
        KeyCode::KEY_SEMICOLON,
        KeyCode::KEY_APOSTROPHE,
        KeyCode::KEY_COMMA,
        KeyCode::KEY_DOT,
        KeyCode::KEY_LEFTBRACE,
        KeyCode::KEY_RIGHTBRACE,
        KeyCode::KEY_GRAVE,
        KeyCode::KEY_SLASH,
        KeyCode::KEY_BACKSLASH,
        KeyCode::KEY_MINUS,
        KeyCode::KEY_EQUAL,
    ];

    for key in KEYS {
        for shift in [false, true] {
            let mapped = if layout_is_ru {
                keycode_to_ru_char(key.code(), shift)
            } else {
                keycode_to_us_char(key.code(), shift)
            };
            if mapped == Some(ch) {
                return KeyEvent {
                    keycode: key.code(),
                    shift,
                    layout_is_ru,
                };
            }
        }
    }

    panic!("no key event for {ch:?} in layout_is_ru={layout_is_ru}");
}

fn push_text_as_layout(buffer: &mut WordBuffer, text: &str, layout_is_ru: bool) {
    for ch in text.chars() {
        if ch == ' ' {
            buffer.handle_space();
        } else {
            buffer.push(text_key_event(ch, layout_is_ru));
        }
    }
}

fn assert_smart_pair(
    left: &str,
    left_layout_is_ru: bool,
    current_typed: &str,
    current_layout_is_ru: bool,
    expected: &str,
) {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, left, left_layout_is_ru);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, current_typed, current_layout_is_ru);
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let original = map_original_events(&events);
    let got = decide_scoped_tail_correction(&events).unwrap_or(original.clone());

    assert_eq!(got, expected, "original tail: {original:?}");
}

fn map_target_events(events: &[KeyEvent], target_is_ru: bool) -> String {
    events
        .iter()
        .filter_map(|ev| {
            if target_is_ru {
                keycode_to_ru_char(ev.keycode, ev.shift)
            } else {
                keycode_to_us_char(ev.keycode, ev.shift)
            }
        })
        .collect()
}

fn apply_typing_assist_to_text_tail(text: &str) -> Option<String> {
    apply_typing_assist_exact(text).or_else(|| {
        let (leading, core, trailing) = split_edge_whitespace(text);
        let segments = split_ws_segments(core);
        if segments.len() < 3 {
            return None;
        }

        for word_count in [2, 1] {
            let mut suffix_start = core.len();
            let mut non_ws_seen = 0;
            for (segment, is_ws) in segments.iter().rev() {
                suffix_start -= segment.len();
                if !is_ws {
                    non_ws_seen += 1;
                    if non_ws_seen == word_count {
                        break;
                    }
                }
            }

            let prefix = &core[..suffix_start];
            let suffix = &core[suffix_start..];
            if let Some(replacement) = apply_typing_assist_exact(&format!("{suffix}{trailing}")) {
                return Some(format!("{leading}{prefix}{replacement}"));
            }
        }

        None
    })
}
