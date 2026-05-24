use evdev::KeyCode;

use crate::keyboard::{mark_word_layout, KeyEvent};

use super::{WordBuffer, MAX_REPLACE_WORDS};

impl WordBuffer {
    pub fn last_completed_words_events(&self, count: usize) -> Option<Vec<KeyEvent>> {
        if (!self.prev_had_trailing_space && self.current.is_empty())
            || count == 0
            || self.prev_words.len() < count
        {
            return None;
        }

        let mut events = Vec::new();
        for word in self.prev_words.iter().skip(self.prev_words.len() - count) {
            push_space_between_words(&mut events);
            events.extend(word.iter().copied());
        }
        events.push(space_event());
        Some(events)
    }

    pub fn mark_replayed_layout(&mut self, replace_words: usize, layout_is_ru: bool) {
        let replace_words = replace_words.clamp(1, MAX_REPLACE_WORDS);
        if !self.current.is_empty() {
            let take_prev = replace_words.saturating_sub(1).min(self.prev_words.len());
            let first_prev = self.prev_words.len() - take_prev;
            for word in self.prev_words.iter_mut().skip(first_prev) {
                mark_word_layout(word, layout_is_ru);
            }
            mark_word_layout(&mut self.current, layout_is_ru);
        } else if self.prev_had_trailing_space && !self.prev_words.is_empty() {
            let take_prev = replace_words.min(self.prev_words.len());
            let first_prev = self.prev_words.len() - take_prev;
            for word in self.prev_words.iter_mut().skip(first_prev) {
                mark_word_layout(word, layout_is_ru);
            }
        }
        self.replay_toggle_ready = true;
    }

    pub fn replay_toggle_ready(&self) -> bool {
        self.replay_toggle_ready
    }

    pub fn what_to_replay(&self, replace_words: usize) -> Option<(Vec<KeyEvent>, u32)> {
        let replace_words = if self.replay_toggle_ready {
            1
        } else {
            replace_words.clamp(1, MAX_REPLACE_WORDS)
        };

        if !self.current.is_empty() {
            let take_prev = replace_words.saturating_sub(1).min(self.prev_words.len());
            let mut events = Vec::new();
            for word in self
                .prev_words
                .iter()
                .skip(self.prev_words.len() - take_prev)
            {
                push_space_between_words(&mut events);
                events.extend(word.iter().copied());
            }
            push_space_between_completed_and_current(&mut events);
            events.extend(self.current.iter().copied());
            let n = events.len() as u32;
            Some((events, n))
        } else if self.prev_had_trailing_space && !self.prev_words.is_empty() {
            let take_prev = replace_words.min(self.prev_words.len());
            let mut events = Vec::new();
            for word in self
                .prev_words
                .iter()
                .skip(self.prev_words.len() - take_prev)
            {
                push_space_between_words(&mut events);
                events.extend(word.iter().copied());
            }
            events.push(space_event());
            let n = events.len() as u32;
            Some((events, n))
        } else {
            None
        }
    }
}

fn push_space_between_words(events: &mut Vec<KeyEvent>) {
    if !events.is_empty() {
        events.push(space_event());
    }
}

fn push_space_between_completed_and_current(events: &mut Vec<KeyEvent>) {
    if !events.is_empty() {
        events.push(space_event());
    }
}

fn space_event() -> KeyEvent {
    KeyEvent {
        keycode: KeyCode::KEY_SPACE.code(),
        shift: false,
        layout_is_ru: false,
    }
}
