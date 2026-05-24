//! Buffered physical word history shared by desktop frontends.
//!
//! The daemon owns event listening and text execution. This module owns only the
//! small state machine around "current word", completed words, replay toggles
//! and pending user-learning feedback.

mod learning;
mod replay_memory;
mod replay_scope;
mod visible_text_memory;

use std::time::Instant;

use crate::keyboard::KeyEvent;

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
}

impl Default for WordBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "word_buffer_tests.rs"]
mod tests;
