use evdev::KeyCode;

use crate::keyboard::{
    map_original_events, preferred_layout_for_text, split_event_words, text_to_key_events, KeyEvent,
};

use super::{WordBuffer, MAX_REPLACE_WORDS};

impl WordBuffer {
    pub fn visible_tail_text(&self, max_words: usize) -> Option<String> {
        if max_words == 0 || (self.current.is_empty() && self.prev_words.is_empty()) {
            return None;
        }
        let current_words = usize::from(!self.current.is_empty());
        let take_prev = max_words
            .saturating_sub(current_words)
            .min(self.prev_words.len());
        let mut events = Vec::new();
        for word in self
            .prev_words
            .iter()
            .skip(self.prev_words.len() - take_prev)
        {
            if !events.is_empty() {
                events.push(visible_tail_space_event());
            }
            events.extend(word.iter().copied());
        }
        if !self.current.is_empty() {
            if !events.is_empty() {
                events.push(visible_tail_space_event());
            }
            events.extend(self.current.iter().copied());
        } else if self.prev_had_trailing_space && !events.is_empty() {
            events.push(visible_tail_space_event());
        }
        (!events.is_empty()).then(|| map_original_events(&events))
    }

    pub fn remember_visible_text_for_correction(&mut self, text: &str) -> bool {
        let Some(events) = text_to_key_events(text, preferred_layout_for_text(text, true)) else {
            return false;
        };
        let Some(words) = split_event_words(&events) else {
            return false;
        };
        let mut owned_words: Vec<Vec<KeyEvent>> = words.iter().map(|word| word.to_vec()).collect();
        if owned_words.len() > MAX_REPLACE_WORDS {
            let keep_from = owned_words.len() - MAX_REPLACE_WORDS;
            owned_words.drain(0..keep_from);
        }

        self.prev_words.clear();
        self.current.clear();
        if text_ends_with_space(text) {
            self.prev_words = owned_words;
            self.prev_had_trailing_space = true;
        } else {
            let Some(current) = owned_words.pop() else {
                return false;
            };
            self.prev_words = owned_words;
            self.current = current;
            self.prev_had_trailing_space = false;
        }
        self.replay_toggle_words = 0;
        self.pending_auto_undo = None;
        true
    }
}

fn text_ends_with_space(text: &str) -> bool {
    text.chars().next_back().is_some_and(char::is_whitespace)
}

fn visible_tail_space_event() -> KeyEvent {
    KeyEvent {
        keycode: KeyCode::KEY_SPACE.code(),
        shift: false,
        layout_is_ru: false,
    }
}
