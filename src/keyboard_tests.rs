use super::*;
use evdev::KeyCode;

fn us_event(key: KeyCode) -> KeyEvent {
    KeyEvent {
        keycode: key.code(),
        shift: false,
        layout_is_ru: false,
    }
}

fn ru_event(key: KeyCode, shift: bool) -> KeyEvent {
    KeyEvent {
        keycode: key.code(),
        shift,
        layout_is_ru: true,
    }
}

fn text_events(text: &str, layout_is_ru: bool) -> Vec<KeyEvent> {
    text_to_key_events(text, layout_is_ru).expect("keyboard fixture must be typable")
}

#[test]
fn maps_wrong_layout_word_to_russian_target() {
    let events = text_events("ltkfq", false);

    assert_eq!(map_original_events(&events), "ltkfq");
    assert_eq!(map_events_to_layout(&events, true), "делай");
    assert_eq!(map_opposite_events(&events), "делай");
}

#[test]
fn maps_shifted_ru_currency_key_to_us_dollar_on_replay() {
    let events = [
        ru_event(KeyCode::KEY_4, false),
        ru_event(KeyCode::KEY_0, false),
        ru_event(KeyCode::KEY_0, false),
        ru_event(KeyCode::KEY_0, false),
        ru_event(KeyCode::KEY_4, true),
    ];

    assert_eq!(map_original_events(&events), "4000;");
    assert_eq!(map_events_to_layout(&events, false), "4000$");
    assert_eq!(map_opposite_events(&events), "4000$");
    assert_eq!(
        replay_layout_decision(&events),
        ReplayLayoutDecision {
            target_is_ru: false,
            mixed_layouts: false,
        }
    );
}

#[test]
fn text_insert_can_type_russian_shifted_punctuation_on_ru_layout() {
    let runs = text_to_uinput_runs("4000; 50%", true).expect("typable text");

    assert_eq!(runs.len(), 1);
    assert!(runs[0].target_is_ru);
    assert_eq!(map_events_to_layout(&runs[0].events, true), "4000; 50%");
}

#[test]
fn typing_key_excludes_shift_and_includes_space() {
    assert!(is_typing_key(KeyCode::KEY_A));
    assert!(is_typing_key(KeyCode::KEY_SPACE));
    assert!(!is_typing_key(KeyCode::KEY_LEFTSHIFT));
}

#[test]
fn splits_text_insert_into_layout_runs() {
    let runs = text_to_uinput_runs("Привет Double", true).expect("typable text");

    assert_eq!(runs.len(), 2);
    assert!(runs[0].target_is_ru);
    assert!(!runs[1].target_is_ru);
    assert_eq!(map_events_to_layout(&runs[0].events, true), "Привет ");
    assert_eq!(map_events_to_layout(&runs[1].events, false), "Double");
}

#[test]
fn layout_decision_ignores_space() {
    let events = [
        KeyEvent {
            keycode: KeyCode::KEY_A.code(),
            shift: false,
            layout_is_ru: false,
        },
        KeyEvent {
            keycode: KeyCode::KEY_SPACE.code(),
            shift: false,
            layout_is_ru: true,
        },
    ];

    assert!(!is_layout_decision_key(KeyCode::KEY_SPACE));
    assert_eq!(
        replay_layout_decision(&events),
        ReplayLayoutDecision {
            target_is_ru: true,
            mixed_layouts: false,
        }
    );
}

#[test]
fn splits_event_words_without_trailing_space_word() {
    let events = text_events("a b ", false);
    let words = split_event_words(&events).expect("words");

    assert_eq!(words.len(), 2);
    assert_eq!(map_original_events(words[0]), "a");
    assert_eq!(map_original_events(words[1]), "b");
}

#[test]
fn marks_only_typing_keys_layout() {
    let mut events = [
        us_event(KeyCode::KEY_A),
        KeyEvent {
            keycode: KeyCode::KEY_LEFTSHIFT.code(),
            shift: false,
            layout_is_ru: false,
        },
    ];

    mark_word_layout(&mut events, true);

    assert!(events[0].layout_is_ru);
    assert!(!events[1].layout_is_ru);
}
