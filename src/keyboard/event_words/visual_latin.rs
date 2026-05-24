use evdev::KeyCode;

use super::super::keymap::{keycode_to_ru_char, keycode_to_us_char};
use super::super::text_input::is_cyrillic_letter;
use super::super::KeyEvent;
use super::mapping::original_event_char;

pub fn mixed_visual_latin_word_target_layout(word: &[KeyEvent]) -> Option<bool> {
    if word.is_empty()
        || word
            .iter()
            .any(|event| event.keycode == KeyCode::KEY_SPACE.code())
    {
        return None;
    }

    let first_layout = word.first()?.layout_is_ru;
    if word.iter().all(|event| event.layout_is_ru == first_layout) {
        return None;
    }

    let mut latin_count = 0usize;
    let mut same_key_homoglyph_count = 0usize;
    let mut other_cyrillic_count = 0usize;

    for event in word {
        let ch = original_event_char(event)?;
        if ch.is_ascii_alphabetic() {
            latin_count += 1;
        } else if is_cyrillic_letter(ch) {
            if same_key_latin_cyrillic_homoglyph(event) {
                same_key_homoglyph_count += 1;
            } else {
                other_cyrillic_count += 1;
            }
        }
    }

    if latin_count >= 2 && same_key_homoglyph_count > 0 && other_cyrillic_count == 0 {
        Some(true)
    } else {
        None
    }
}

fn same_key_latin_cyrillic_homoglyph(event: &KeyEvent) -> bool {
    matches!(
        (
            keycode_to_us_char(event.keycode, event.shift),
            keycode_to_ru_char(event.keycode, event.shift),
        ),
        (Some('c' | 'C'), Some('с' | 'С'))
    )
}
