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

#[cfg(test)]
mod tests {
    use super::map_events_to_layout;
    use crate::keyboard::text_to_key_events;

    #[test]
    fn physical_layout_projection_is_exact_and_reversible() {
        let ru = text_to_key_events("а", true).expect("Russian physical key");
        let us = text_to_key_events("f", false).expect("US physical key");
        assert_eq!(map_events_to_layout(&ru, false), "f");
        assert_eq!(map_events_to_layout(&us, true), "а");

        let ru_word = text_to_key_events("привет", true).expect("Russian physical word");
        let us_word = text_to_key_events("ghbdtn", false).expect("US physical word");
        assert_eq!(map_events_to_layout(&ru_word, false), "ghbdtn");
        assert_eq!(map_events_to_layout(&us_word, true), "привет");
    }
}
