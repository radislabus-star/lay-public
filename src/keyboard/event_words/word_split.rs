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

pub fn mark_single_current_word_layout_if_stale(
    events: &mut [KeyEvent],
    current_layout_is_ru: bool,
) -> bool {
    if events.is_empty()
        || events
            .iter()
            .any(|event| event.keycode == KeyCode::KEY_SPACE.code())
    {
        return false;
    }

    let mut typing_layouts = events
        .iter()
        .filter(|event| is_typing_key(KeyCode::new(event.keycode)))
        .map(|event| event.layout_is_ru);
    let Some(first_layout) = typing_layouts.next() else {
        return false;
    };
    if first_layout == current_layout_is_ru || typing_layouts.any(|layout| layout != first_layout) {
        return false;
    }

    mark_word_layout(events, current_layout_is_ru);
    true
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
