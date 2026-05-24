use evdev::KeyCode;

use crate::keyboard::{
    map_events_to_layout, map_original_events, mark_word_layout, preferred_layout_for_text,
    split_event_words, text_to_key_events, KeyEvent,
};
use crate::text_edit::TextReplacement;

use super::{WordBuffer, MAX_REPLACE_WORDS};

impl WordBuffer {
    pub fn remember_inserted_tail_for_replay(
        &mut self,
        original_events: &[KeyEvent],
        plan: &TextReplacement,
        inserted_layout_is_ru: bool,
    ) -> bool {
        if plan.move_right != 0 || plan.insert.is_empty() {
            return false;
        }

        let replaced_len = plan.backspaces as usize;
        if replaced_len == 0 || replaced_len > original_events.len() {
            return false;
        }

        let start = original_events.len() - replaced_len;
        let mut tail = original_events[start..].to_vec();
        if tail.is_empty()
            || tail
                .iter()
                .any(|ev| ev.keycode == KeyCode::KEY_SPACE.code())
        {
            return false;
        }

        mark_word_layout(&mut tail, inserted_layout_is_ru);
        if map_original_events(&tail) != plan.insert {
            return false;
        }

        self.current = tail;
        self.prev_words.clear();
        self.prev_had_trailing_space = false;
        self.replay_toggle_ready = true;
        true
    }

    pub fn remember_inserted_last_word_for_replay(
        &mut self,
        original_events: &[KeyEvent],
        plan: &TextReplacement,
    ) -> bool {
        if plan.move_right != 0 || plan.insert.trim().is_empty() {
            return false;
        }

        let Some(inserted_word) = plan.insert.split_whitespace().next_back() else {
            return false;
        };
        if inserted_word.is_empty() {
            return false;
        }

        let Some(words) = split_event_words(original_events) else {
            return false;
        };
        for word in words.iter().rev() {
            for target_is_ru in [false, true] {
                if map_events_to_layout(word, target_is_ru) != inserted_word {
                    continue;
                }

                let mut tail = (*word).to_vec();
                mark_word_layout(&mut tail, target_is_ru);
                self.current = tail;
                self.prev_words.clear();
                self.prev_had_trailing_space = false;
                self.replay_toggle_ready = true;
                return true;
            }
        }

        false
    }

    pub fn remember_replacement_last_word_for_replay(
        &mut self,
        original_events: &[KeyEvent],
        plan: &TextReplacement,
        replacement: &str,
    ) -> bool {
        let trailing_ws_chars = replacement
            .chars()
            .rev()
            .take_while(|ch| ch.is_whitespace())
            .count() as u32;
        let original = map_original_events(original_events);
        let original_body_spaces = original
            .trim_end_matches(char::is_whitespace)
            .chars()
            .filter(|ch| ch.is_whitespace())
            .count();
        let replacement_body_spaces = replacement
            .trim_end_matches(char::is_whitespace)
            .chars()
            .filter(|ch| ch.is_whitespace())
            .count();
        if plan.move_right > trailing_ws_chars && replacement_body_spaces > original_body_spaces {
            return self.remember_completed_replacement_words_for_replay(replacement);
        }
        if plan.backspaces == 0 {
            return false;
        }
        if plan.move_right != 0 && plan.move_right != trailing_ws_chars {
            return false;
        }

        let Some(inserted_word) = replacement.split_whitespace().next_back() else {
            return false;
        };
        if inserted_word.is_empty() {
            return false;
        }
        let replacement_ends_with_space = replacement
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace);

        let Some(words) = split_event_words(original_events) else {
            return false;
        };
        for word in words.iter().rev() {
            for target_is_ru in [false, true] {
                if map_events_to_layout(word, target_is_ru) != inserted_word {
                    continue;
                }

                let mut tail = (*word).to_vec();
                mark_word_layout(&mut tail, target_is_ru);
                return self.remember_replacement_tail_events(tail, replacement_ends_with_space);
            }
        }

        let target_layout = preferred_layout_for_text(replacement, true);
        let Some(mut tail) = text_to_key_events(inserted_word, target_layout) else {
            return false;
        };
        mark_word_layout(&mut tail, target_layout);
        self.remember_replacement_tail_events(tail, replacement_ends_with_space)
    }

    pub fn remember_completed_replacement_words_for_replay(&mut self, replacement: &str) -> bool {
        let replacement_ends_with_space = replacement
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace);
        let mut words = Vec::new();

        for word in replacement.split_whitespace() {
            let target_layout = preferred_layout_for_text(word, true);
            let Some(mut events) = text_to_key_events(word, target_layout) else {
                return false;
            };
            mark_word_layout(&mut events, target_layout);
            words.push(events);
        }

        if words.is_empty() {
            return false;
        }

        if words.len() > MAX_REPLACE_WORDS {
            let keep_from = words.len() - MAX_REPLACE_WORDS;
            words.drain(0..keep_from);
        }

        self.prev_words = words;
        self.prev_had_trailing_space = replacement_ends_with_space;
        self.replay_toggle_ready = true;
        true
    }

    pub fn remember_visible_text_for_correction(&mut self, text: &str) -> bool {
        let Some(events) = text_to_key_events(text, preferred_layout_for_text(text, true)) else {
            return false;
        };
        let Some(words) = split_event_words(&events) else {
            return false;
        };
        let text_ends_with_space = text.chars().next_back().is_some_and(char::is_whitespace);
        let mut owned_words: Vec<Vec<KeyEvent>> = words.iter().map(|word| word.to_vec()).collect();

        if owned_words.len() > MAX_REPLACE_WORDS {
            let keep_from = owned_words.len() - MAX_REPLACE_WORDS;
            owned_words.drain(0..keep_from);
        }

        self.prev_words.clear();
        self.current.clear();
        if text_ends_with_space {
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
        self.replay_toggle_ready = false;
        self.pending_auto_undo = None;
        true
    }

    fn remember_replacement_tail_events(
        &mut self,
        tail: Vec<KeyEvent>,
        replacement_ends_with_space: bool,
    ) -> bool {
        self.prev_words.clear();
        if replacement_ends_with_space {
            self.current.clear();
            self.prev_words.push(tail);
            self.prev_had_trailing_space = true;
        } else {
            self.current = tail;
            self.prev_had_trailing_space = false;
        }
        self.replay_toggle_ready = true;
        true
    }
}
