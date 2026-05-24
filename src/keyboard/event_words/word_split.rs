use evdev::KeyCode;

use super::super::keymap::is_typing_key;
use super::super::KeyEvent;

pub fn mark_word_layout(word: &mut [KeyEvent], layout_is_ru: bool) {
    for event in word {
        if is_typing_key(KeyCode::new(event.keycode)) {
            event.layout_is_ru = layout_is_ru;
        }
    }
}

pub fn split_event_words(events: &[KeyEvent]) -> Option<Vec<&[KeyEvent]>> {
    if events.is_empty() {
        return None;
    }

    let end = if events
        .last()
        .is_some_and(|event| event.keycode == KeyCode::KEY_SPACE.code())
    {
        events.len().saturating_sub(1)
    } else {
        events.len()
    };
    if end == 0 {
        return None;
    }

    let mut words = Vec::new();
    let mut start = 0;
    for (idx, event) in events.iter().take(end).enumerate() {
        if event.keycode == KeyCode::KEY_SPACE.code() {
            if start < idx {
                words.push(&events[start..idx]);
            }
            start = idx + 1;
        }
    }
    if start < end {
        words.push(&events[start..end]);
    }

    (!words.is_empty()).then_some(words)
}
