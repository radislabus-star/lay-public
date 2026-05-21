//! Buffered physical word history shared by desktop frontends.
//!
//! The daemon owns event listening and text execution. This module owns only the
//! small state machine around "current word", completed words, replay toggles
//! and pending user-learning feedback.

use evdev::KeyCode;
use std::time::{Duration, Instant};

use crate::keyboard::{
    map_events_to_layout, map_original_events, mark_word_layout, split_event_words,
    text_to_key_events, KeyEvent,
};
use crate::text_edit::{tail_chars, TextReplacement};

pub const MAX_REPLACE_WORDS: usize = 8;
const LEARNING_FEEDBACK_MAX_AGE_SECS: u64 = 30;

#[derive(Debug)]
pub struct WordBuffer {
    current: Vec<KeyEvent>,
    prev_words: Vec<Vec<KeyEvent>>,
    prev_had_trailing_space: bool,
    replay_toggle_ready: bool,
    pending_learning: Option<PendingLearningCorrection>,
    pending_auto_undo: Option<PendingAutoUndo>,
}

#[derive(Debug, Clone)]
pub struct PendingAutoUndo {
    pub lay_kind: String,
    pub original: String,
    pub replacement: String,
    pub replace_words: usize,
    pub words: usize,
    started_at: Instant,
}

#[derive(Debug, Clone)]
struct PendingLearningCorrection {
    lay_kind: String,
    lay_from: String,
    lay_to: String,
    replace_words: usize,
    words: usize,
    started_at: Instant,
    deleted_chars: u32,
    typed: Vec<KeyEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserLearningCorrection {
    pub lay_kind: String,
    pub lay_from: String,
    pub lay_to: String,
    pub from: String,
    pub to: String,
    pub replace_words: usize,
    pub words: usize,
}

impl WordBuffer {
    #[inline]
    pub fn new() -> Self {
        Self {
            current: Vec::with_capacity(32),
            prev_words: Vec::with_capacity(MAX_REPLACE_WORDS),
            prev_had_trailing_space: false,
            replay_toggle_ready: false,
            pending_learning: None,
            pending_auto_undo: None,
        }
    }

    #[inline]
    pub fn current_len(&self) -> usize {
        self.current.len()
    }

    #[inline]
    pub fn current_is_empty(&self) -> bool {
        self.current.is_empty()
    }

    #[inline]
    pub fn current_last_keycode(&self) -> Option<u16> {
        self.current.last().map(|event| event.keycode)
    }

    #[inline]
    pub fn prev_words_len(&self) -> usize {
        self.prev_words.len()
    }

    #[inline]
    pub fn prev_had_trailing_space(&self) -> bool {
        self.prev_had_trailing_space
    }

    #[inline]
    pub fn prev_word_events(&self, index: usize) -> Option<&[KeyEvent]> {
        self.prev_words.get(index).map(Vec::as_slice)
    }

    #[inline]
    pub fn push(&mut self, e: KeyEvent) {
        self.current.push(e);
        self.prev_had_trailing_space = false;
        self.replay_toggle_ready = false;
        self.pending_auto_undo = None;
    }

    #[inline]
    pub fn handle_space(&mut self) {
        if !self.current.is_empty() {
            self.prev_words.push(std::mem::take(&mut self.current));
            if self.prev_words.len() > MAX_REPLACE_WORDS {
                self.prev_words.remove(0);
            }
            self.prev_had_trailing_space = true;
        }
    }

    #[inline]
    pub fn reset_all(&mut self) {
        self.current.clear();
        self.prev_words.clear();
        self.prev_had_trailing_space = false;
        self.replay_toggle_ready = false;
        self.pending_auto_undo = None;
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.current.is_empty() && self.prev_words.is_empty()
    }

    pub fn last_completed_words_events(&self, count: usize) -> Option<Vec<KeyEvent>> {
        if (!self.prev_had_trailing_space && self.current.is_empty())
            || count == 0
            || self.prev_words.len() < count
        {
            return None;
        }

        let mut events = Vec::new();
        for word in self.prev_words.iter().skip(self.prev_words.len() - count) {
            if !events.is_empty() {
                events.push(KeyEvent {
                    keycode: KeyCode::KEY_SPACE.code(),
                    shift: false,
                    layout_is_ru: false,
                });
            }
            events.extend(word.iter().copied());
        }
        events.push(KeyEvent {
            keycode: KeyCode::KEY_SPACE.code(),
            shift: false,
            layout_is_ru: false,
        });
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

        let target_layout = crate::keyboard::preferred_layout_for_text(replacement, true);
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
            let target_layout = crate::keyboard::preferred_layout_for_text(word, true);
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
        let Some(events) =
            text_to_key_events(text, crate::keyboard::preferred_layout_for_text(text, true))
        else {
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

    pub fn remember_pending_learning_correction(
        &mut self,
        lay_kind: &str,
        lay_from: &str,
        lay_to: &str,
        replace_words: usize,
        words: usize,
    ) {
        if lay_from == lay_to || lay_from.trim().is_empty() || lay_to.trim().is_empty() {
            self.pending_learning = None;
            return;
        }

        self.pending_learning = Some(PendingLearningCorrection {
            lay_kind: lay_kind.to_string(),
            lay_from: lay_from.to_string(),
            lay_to: lay_to.to_string(),
            replace_words,
            words,
            started_at: Instant::now(),
            deleted_chars: 0,
            typed: Vec::new(),
        });
    }

    pub fn clear_pending_learning(&mut self) {
        self.pending_learning = None;
    }

    pub fn remember_pending_auto_undo(
        &mut self,
        lay_kind: &str,
        original: &str,
        replacement: &str,
        replace_words: usize,
        words: usize,
    ) {
        if original == replacement || original.trim().is_empty() || replacement.trim().is_empty() {
            self.pending_auto_undo = None;
            return;
        }

        self.pending_auto_undo = Some(PendingAutoUndo {
            lay_kind: lay_kind.to_string(),
            original: original.to_string(),
            replacement: replacement.to_string(),
            replace_words,
            words,
            started_at: Instant::now(),
        });
    }

    pub fn take_pending_auto_undo(&mut self) -> Option<PendingAutoUndo> {
        let undo = self.pending_auto_undo.take()?;
        if undo.started_at.elapsed() > Duration::from_secs(LEARNING_FEEDBACK_MAX_AGE_SECS) {
            return None;
        }
        Some(undo)
    }

    pub fn note_learning_backspace(&mut self) {
        let Some(pending) = self.pending_learning.as_mut() else {
            return;
        };
        if pending.started_at.elapsed() > Duration::from_secs(LEARNING_FEEDBACK_MAX_AGE_SECS) {
            self.pending_learning = None;
            return;
        }
        pending.deleted_chars = pending.deleted_chars.saturating_add(1);
        pending.typed.clear();
    }

    pub fn note_learning_typed(&mut self, event: KeyEvent) {
        let Some(pending) = self.pending_learning.as_mut() else {
            return;
        };
        if pending.started_at.elapsed() > Duration::from_secs(LEARNING_FEEDBACK_MAX_AGE_SECS) {
            self.pending_learning = None;
            return;
        }
        if pending.deleted_chars == 0 {
            self.pending_learning = None;
            return;
        }
        pending.typed.push(event);
    }

    pub fn take_user_learning_correction(
        &mut self,
        include_trailing_space: bool,
    ) -> Option<UserLearningCorrection> {
        let pending = self.pending_learning.take()?;
        if pending.deleted_chars == 0 || pending.typed.is_empty() {
            return None;
        }

        let from = tail_chars(&pending.lay_to, pending.deleted_chars as usize);
        let mut to = map_original_events(&pending.typed);
        let lay_to_ends_with_space = pending
            .lay_to
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace);
        if include_trailing_space && lay_to_ends_with_space {
            to.push(' ');
        }

        if from == to || from.trim().is_empty() || to.trim().is_empty() {
            return None;
        }

        Some(UserLearningCorrection {
            lay_kind: pending.lay_kind,
            lay_from: pending.lay_from,
            lay_to: pending.lay_to,
            from,
            to,
            replace_words: pending.replace_words,
            words: pending.words,
        })
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
                if !events.is_empty() {
                    events.push(KeyEvent {
                        keycode: KeyCode::KEY_SPACE.code(),
                        shift: false,
                        layout_is_ru: false,
                    });
                }
                events.extend(word.iter().copied());
            }
            if !events.is_empty() {
                events.push(KeyEvent {
                    keycode: KeyCode::KEY_SPACE.code(),
                    shift: false,
                    layout_is_ru: false,
                });
            }
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
                if !events.is_empty() {
                    events.push(KeyEvent {
                        keycode: KeyCode::KEY_SPACE.code(),
                        shift: false,
                        layout_is_ru: false,
                    });
                }
                events.extend(word.iter().copied());
            }
            events.push(KeyEvent {
                keycode: KeyCode::KEY_SPACE.code(),
                shift: false,
                layout_is_ru: false,
            });
            let n = events.len() as u32;
            Some((events, n))
        } else {
            None
        }
    }
}

impl Default for WordBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "word_buffer_tests.rs"]
mod tests;
