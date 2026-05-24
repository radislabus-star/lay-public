//! Physical keycode mapping shared by desktop adapters.
//!
//! The daemon still owns evdev listening and uinput replay. This module only
//! describes how stored physical key events map to US/RU text.

#[derive(Clone, Copy, Debug)]
pub struct KeyEvent {
    pub keycode: u16,
    pub shift: bool,
    pub layout_is_ru: bool,
}

#[derive(Debug, Clone)]
pub struct TextInputRun {
    pub target_is_ru: bool,
    pub events: Vec<KeyEvent>,
}

mod event_words;
mod keymap;
mod text_input;

pub use event_words::{
    is_layout_decision_key, map_events_to_layout, map_opposite_events, map_original_events,
    mark_word_layout, mixed_visual_latin_word_target_layout, original_event_char,
    replay_layout_decision, split_event_words, ReplayLayoutDecision,
};
pub use keymap::{is_typing_key, keycode_to_ru_char, keycode_to_us_char};
pub use text_input::{
    is_cyrillic_letter, preferred_layout_for_text, text_to_key_events, text_to_uinput_runs,
};

#[cfg(test)]
#[path = "keyboard_tests.rs"]
mod tests;
