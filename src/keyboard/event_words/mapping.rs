use super::super::keymap::{keycode_to_ru_char, keycode_to_us_char};
use super::super::KeyEvent;

#[inline]
pub fn map_original_events(events: &[KeyEvent]) -> String {
    events
        .iter()
        .filter_map(|ev| {
            if ev.layout_is_ru {
                keycode_to_ru_char(ev.keycode, ev.shift)
            } else {
                keycode_to_us_char(ev.keycode, ev.shift)
            }
        })
        .collect()
}

#[inline]
pub fn map_opposite_events(events: &[KeyEvent]) -> String {
    events
        .iter()
        .filter_map(|ev| {
            if ev.layout_is_ru {
                keycode_to_us_char(ev.keycode, ev.shift)
            } else {
                keycode_to_ru_char(ev.keycode, ev.shift)
            }
        })
        .collect()
}

#[inline]
pub fn map_events_to_layout(events: &[KeyEvent], target_is_ru: bool) -> String {
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

pub fn original_event_char(event: &KeyEvent) -> Option<char> {
    if event.layout_is_ru {
        keycode_to_ru_char(event.keycode, event.shift)
    } else {
        keycode_to_us_char(event.keycode, event.shift)
    }
}
