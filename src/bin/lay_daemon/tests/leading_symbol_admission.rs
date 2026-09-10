use super::*;
use crate::buffer_filter_runtime::{should_skip_buffer_input, BufferFilterContext};

fn observe_character(
    buffer: &mut WordBuffer,
    event: lay::keyboard::KeyEvent,
    modifiers: &ShiftState,
) {
    let key = KeyCode::new(event.keycode);
    if should_skip_buffer_input(BufferFilterContext {
        key,
        code: event.keycode,
        shift_state: modifiers,
        verbose: false,
    }) {
        return;
    }
    if key == KeyCode::KEY_SPACE {
        buffer.handle_space();
    } else {
        buffer.push(event);
    }
}

#[test]
fn leading_symbols_replay_exact_visible_suffix_and_round_trip() {
    let mut failures = Vec::new();
    let mut cases = 0;
    for (symbol_key, shifted, symbol) in [
        (KeyCode::KEY_MINUS, false, "-"),
        (KeyCode::KEY_MINUS, true, "_"),
        (KeyCode::KEY_EQUAL, false, "="),
        (KeyCode::KEY_EQUAL, true, "+"),
    ] {
        for (layout_is_ru, letter, target_letter, letter_key, previous) in [
            (true, "з", "p", KeyCode::KEY_P, "привет "),
            (false, "f", "а", KeyCode::KEY_F, "hello "),
        ] {
            for prefix in ["", previous] {
                for trailing in ["", " "] {
                    cases += 1;
                    let mut buffer = WordBuffer::new();
                    push_text_as_layout(&mut buffer, prefix, layout_is_ru);
                    let mut modifiers = ShiftState::default();
                    modifiers.update(KeyCode::KEY_LEFTSHIFT, i32::from(shifted));
                    observe_character(
                        &mut buffer,
                        key_event_with_shift(symbol_key, shifted, layout_is_ru),
                        &modifiers,
                    );
                    modifiers.update(KeyCode::KEY_LEFTSHIFT, 0);
                    observe_character(&mut buffer, key_event(letter_key, layout_is_ru), &modifiers);
                    if !trailing.is_empty() {
                        observe_character(
                            &mut buffer,
                            key_event(KeyCode::KEY_SPACE, layout_is_ru),
                            &modifiers,
                        );
                    }
                    let source = format!("{symbol}{letter}{trailing}");
                    let target = format!("{symbol}{target_letter}{trailing}");
                    let visible = format!("{prefix}{source}");
                    let Some((events, erase)) = buffer.what_to_replay(1) else {
                        failures.push(format!("{visible:?}: no replay"));
                        continue;
                    };
                    let actual_source = map_original_events(&events);
                    let actual_target = map_events_to_layout(&events, !layout_is_ru);
                    let expected_erase = source.chars().count() as u32;
                    let retained: String = visible
                        .chars()
                        .take(visible.chars().count().saturating_sub(erase as usize))
                        .collect();
                    let actual_visible = format!("{retained}{actual_target}");
                    if actual_source != source
                        || actual_target != target
                        || erase != expected_erase
                        || actual_visible != format!("{prefix}{target}")
                    {
                        failures.push(format!(
                            "{visible:?}: source={actual_source:?}, target={actual_target:?}, \
                             erase={erase}, visible={actual_visible:?}"
                        ));
                    }
                    buffer.mark_replayed_layout(1, !layout_is_ru);
                    let (reverse, reverse_erase) = buffer.what_to_replay(1).expect("same scope");
                    if map_original_events(&reverse) != target
                        || map_events_to_layout(&reverse, layout_is_ru) != source
                        || reverse_erase != expected_erase
                    {
                        failures.push(format!("{visible:?}: reverse scope drift"));
                    }
                }
            }
        }
    }
    assert_eq!(cases, 32);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn symbol_shortcuts_stay_filtered_without_poisoning_following_typing() {
    for modifier in [
        KeyCode::KEY_LEFTCTRL,
        KeyCode::KEY_LEFTALT,
        KeyCode::KEY_LEFTMETA,
    ] {
        for symbol in [KeyCode::KEY_MINUS, KeyCode::KEY_EQUAL] {
            let mut buffer = WordBuffer::new();
            let mut modifiers = ShiftState::default();
            modifiers.update(modifier, 1);
            observe_character(&mut buffer, key_event(symbol, false), &modifiers);
            assert!(buffer.is_empty(), "shortcut {modifier:?}+{symbol:?}");
            modifiers.update(modifier, 0);
            observe_character(&mut buffer, key_event(KeyCode::KEY_A, false), &modifiers);
            let (events, erase) = buffer.what_to_replay(1).expect("fresh typing");
            assert_eq!(map_original_events(&events), "a");
            assert_eq!(map_events_to_layout(&events, true), "ф");
            assert_eq!(erase, 1);
        }
    }
}

#[test]
fn admitted_clean_symbol_tokens_do_not_authorize_typing_assistance() {
    for input in [
        "--help ",
        "--force ",
        "_name ",
        "+x ",
        "=42 ",
        "git --help ",
        "git push --force ",
        "hello _name ",
        "hello +x ",
        "hello =42 ",
    ] {
        let mut buffer = WordBuffer::new();
        for character in input.chars() {
            let event = if character == ' ' {
                key_event(KeyCode::KEY_SPACE, false)
            } else {
                text_key_event(character, false)
            };
            let mut modifiers = ShiftState::default();
            modifiers.update(KeyCode::KEY_LEFTSHIFT, i32::from(event.shift));
            observe_character(&mut buffer, event, &modifiers);
        }
        let (events, erase) = buffer.what_to_replay(1).expect("observed token");
        let tail = format!("{} ", input.split_whitespace().last().unwrap());
        assert_eq!(map_original_events(&events), tail);
        assert_eq!(erase as usize, tail.chars().count());
        // Exercise both layout settings and the one/two-word decoder scopes.
        // The existing test runtime enables normal typing assistance but
        // disables Nanda; production-model coverage is recorded separately.
        for allow_layout_auto in [false, true] {
            for max_words in [1, 2, 3] {
                assert!(
                    find_typing_assist_correction(&buffer, allow_layout_auto, max_words).is_none(),
                    "clean token {input:?}, auto-layout={allow_layout_auto}, scope={max_words}"
                );
            }
        }
    }
}
