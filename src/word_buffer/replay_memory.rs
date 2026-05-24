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
            return self.remember_visible_replacement_tail_for_replay(original_events, replacement);
        }
        if plan.backspaces == 0 {
            return false;
        }
        if plan.move_right != 0 && plan.move_right != trailing_ws_chars {
            return false;
        }
        self.remember_visible_replacement_tail_for_replay(original_events, replacement)
    }

    pub fn remember_completed_replacement_words_for_replay(&mut self, replacement: &str) -> bool {
        let replacement_ends_with_space = replacement
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace);
        let Some(mut words) = replacement_word_events(replacement) else {
            return false;
        };

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

    pub fn remember_visible_replacement_tail_for_replay(
        &mut self,
        original_events: &[KeyEvent],
        replacement: &str,
    ) -> bool {
        let Some(original_words) = split_event_words(original_events) else {
            return false;
        };
        let original_word_count = original_words.len();
        if original_word_count == 0 {
            return false;
        }

        let Some(replacement_words) = replacement_word_events(replacement) else {
            return false;
        };
        if replacement_words.is_empty() {
            return false;
        }

        let original_ends_with_space = original_events
            .last()
            .is_some_and(|event| event.keycode == KeyCode::KEY_SPACE.code());
        let replacement_ends_with_space = replacement
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace);

        if original_ends_with_space {
            if self.prev_words.len() < original_word_count {
                return false;
            }
            truncate_prev_words_suffix(&mut self.prev_words, original_word_count);
            self.current.clear();
            self.remember_replacement_words(replacement_words, replacement_ends_with_space);
            return true;
        }

        if !self.current.is_empty() {
            let previous_words_in_tail = original_word_count.saturating_sub(1);
            if self.prev_words.len() < previous_words_in_tail {
                return false;
            }
            truncate_prev_words_suffix(&mut self.prev_words, previous_words_in_tail);
            self.remember_replacement_words(replacement_words, replacement_ends_with_space);
            return true;
        }

        if self.prev_words.len() < original_word_count {
            return false;
        }
        truncate_prev_words_suffix(&mut self.prev_words, original_word_count);
        self.remember_replacement_words(replacement_words, replacement_ends_with_space);
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
    fn remember_replacement_words(
        &mut self,
        mut replacement_words: Vec<Vec<KeyEvent>>,
        replacement_ends_with_space: bool,
    ) {
        if replacement_ends_with_space {
            self.current.clear();
            append_completed_words(&mut self.prev_words, replacement_words);
            self.prev_had_trailing_space = true;
        } else {
            let current = replacement_words.pop();
            append_completed_words(&mut self.prev_words, replacement_words);
            if let Some(current) = current {
                self.current = current;
            } else {
                self.current.clear();
            }
            self.prev_had_trailing_space = false;
        }
        self.replay_toggle_ready = true;
    }
}

fn replacement_word_events(replacement: &str) -> Option<Vec<Vec<KeyEvent>>> {
    replacement
        .split_whitespace()
        .map(|word| {
            let target_layout = preferred_layout_for_text(word, true);
            let mut events = text_to_key_events(word, target_layout)?;
            mark_word_layout(&mut events, target_layout);
            Some(events)
        })
        .collect()
}

fn truncate_prev_words_suffix(prev_words: &mut Vec<Vec<KeyEvent>>, count: usize) {
    let keep = prev_words.len().saturating_sub(count);
    prev_words.truncate(keep);
}

fn append_completed_words(prev_words: &mut Vec<Vec<KeyEvent>>, words: Vec<Vec<KeyEvent>>) {
    prev_words.extend(words);
    if prev_words.len() > MAX_REPLACE_WORDS {
        let keep_from = prev_words.len() - MAX_REPLACE_WORDS;
        prev_words.drain(0..keep_from);
    }
}
