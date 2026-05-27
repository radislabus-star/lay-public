use crate::keyboard::{preferred_layout_for_text, split_event_words, text_to_key_events, KeyEvent};

use super::{WordBuffer, MAX_REPLACE_WORDS};

impl WordBuffer {
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
