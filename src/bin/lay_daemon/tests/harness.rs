use evdev::KeyCode;
use lay::config::{default_typing_assist_pipeline, TypingAssistRuleConfig};
use lay::keyboard::{keycode_to_ru_char, keycode_to_us_char, map_original_events, KeyEvent};
use lay::text_edit::TextReplacement;
use lay::typing_assist::{decide_scoped_tail_correction, split_edge_whitespace, split_ws_segments};
use lay::word_buffer::WordBuffer;

pub(super) fn ascii_hyphen_token_keycodes() -> [KeyCode; 5] {
    [
        KeyCode::KEY_W,
        KeyCode::KEY_I,
        KeyCode::KEY_MINUS,
        KeyCode::KEY_F,
        KeyCode::KEY_I,
    ]
}

pub(super) fn typing_pipeline_with_disabled(disabled: &[&str]) -> Vec<TypingAssistRuleConfig> {
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

pub(super) fn typing_pipeline_with_only(enabled: &str) -> Vec<TypingAssistRuleConfig> {
    default_typing_assist_pipeline()
        .into_iter()
        .map(|mut rule| {
            rule.enabled = rule.id == enabled;
            rule
        })
        .collect()
}

pub(super) fn typing_pipeline_with_first(first: &str) -> Vec<TypingAssistRuleConfig> {
    let mut rules = default_typing_assist_pipeline();
    for rule in &mut rules {
        rule.priority += 10;
        if rule.id == first {
            rule.priority = 1;
        }
    }
    rules
}

pub(super) fn push_key_events(
    buffer: &mut WordBuffer,
    keys: &[(KeyCode, bool)],
    layout_is_ru: bool,
) {
    for (key, shift) in keys {
        buffer.push(key_event_with_shift(*key, *shift, layout_is_ru));
    }
}

pub(super) fn key_event(key: KeyCode, layout_is_ru: bool) -> KeyEvent {
    key_event_with_shift(key, false, layout_is_ru)
}

pub(super) fn key_event_with_shift(key: KeyCode, shift: bool, layout_is_ru: bool) -> KeyEvent {
    KeyEvent {
        keycode: key.code(),
        shift,
        layout_is_ru,
    }
}

pub(super) fn key_events(keys: &[KeyCode], layout_is_ru: bool) -> Vec<KeyEvent> {
    keys.iter()
        .map(|key| key_event(*key, layout_is_ru))
        .collect()
}

pub(super) fn push_text_as_layout(buffer: &mut WordBuffer, text: &str, layout_is_ru: bool) {
    for ch in text.chars() {
        if ch == ' ' {
            buffer.handle_space();
        } else {
            buffer.push(text_key_event(ch, layout_is_ru));
        }
    }
}

pub(super) fn typed_buffer(parts: &[(&str, bool)]) -> WordBuffer {
    let mut buffer = WordBuffer::new();
    for (text, layout_is_ru) in parts {
        push_text_as_layout(&mut buffer, text, *layout_is_ru);
    }
    buffer
}

pub(super) fn layout_from_fixture(value: &str) -> bool {
    match value {
        "ru" => true,
        "us" => false,
        other => panic!("bad layout fixture value {other:?}"),
    }
}

pub(super) fn typed_buffer_from_fixture_parts(parts: &str) -> WordBuffer {
    typed_buffer_from_fixture_sequence(parts, '|')
}

pub(super) fn typed_buffer_from_semicolon_fixture(parts: &str) -> WordBuffer {
    typed_buffer_from_fixture_sequence(parts, ';')
}

pub(super) fn typed_buffer_from_fixture_sequence(parts: &str, delimiter: char) -> WordBuffer {
    let mut buffer = WordBuffer::new();
    for part in parts.split(delimiter) {
        let (text, layout) = part
            .rsplit_once('@')
            .unwrap_or_else(|| panic!("bad fixture buffer part {part:?}"));
        push_text_as_layout(&mut buffer, text, layout_from_fixture(layout));
    }
    buffer
}

pub(super) fn text_replacement_from_fixture(
    row: &[String],
    move_left: usize,
    backspaces: usize,
    insert: usize,
    move_right: usize,
) -> TextReplacement {
    text_replacement(
        row[move_left].parse().expect("move_left"),
        row[backspaces].parse().expect("backspaces"),
        &row[insert],
        row[move_right].parse().expect("move_right"),
    )
}

pub(super) fn text_replacement(
    move_left: u32,
    backspaces: u32,
    insert: impl Into<String>,
    move_right: u32,
) -> TextReplacement {
    TextReplacement {
        move_left,
        backspaces,
        insert: insert.into(),
        move_right,
    }
}

pub(super) fn text_replacement_zero_edges(
    row: &[String],
    backspaces: usize,
    insert: usize,
) -> TextReplacement {
    text_replacement(
        0,
        row[backspaces].parse().expect("backspaces"),
        &row[insert],
        0,
    )
}

pub(super) fn typed_tail(
    parts: &[(&str, bool)],
    scope: usize,
    expect: &str,
) -> (WordBuffer, Vec<KeyEvent>, u32) {
    let buffer = typed_buffer(parts);
    let (events, backspaces) = buffer.what_to_replay(scope).expect(expect);
    (buffer, events, backspaces)
}

pub(super) fn apply_typing_assist_to_text_tail_with<F>(text: &str, apply_exact: F) -> Option<String>
where
    F: Fn(&str) -> Option<String> + Copy,
{
    apply_exact(text).or_else(|| {
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
            if let Some(replacement) = apply_exact(&format!("{suffix}{trailing}")) {
                return Some(format!("{leading}{prefix}{replacement}"));
            }
        }

        None
    })
}

pub(super) fn assert_smart_pair(
    left: &str,
    left_layout_is_ru: bool,
    current_typed: &str,
    current_layout_is_ru: bool,
    expected: &str,
) {
    let (_buffer, events, _) = typed_tail(
        &[
            (left, left_layout_is_ru),
            (" ", left_layout_is_ru),
            (current_typed, current_layout_is_ru),
        ],
        2,
        "two-word tail",
    );
    let original = map_original_events(&events);
    let got = decide_scoped_tail_correction(&events).unwrap_or(original.clone());

    assert_eq!(got, expected, "original tail: {original:?}");
}

pub(super) fn map_target_events(events: &[KeyEvent], target_is_ru: bool) -> String {
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

pub(super) fn text_key_event(ch: char, layout_is_ru: bool) -> KeyEvent {
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
                return key_event_with_shift(*key, shift, layout_is_ru);
            }
        }
    }

    panic!("no key event for {ch:?} in layout_is_ru={layout_is_ru}");
}
