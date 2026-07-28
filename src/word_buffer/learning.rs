use std::time::{Duration, Instant};

use crate::keyboard::{map_original_events, KeyEvent};
use crate::text_edit::tail_chars;

use super::{
    PendingAutoUndo, PendingLearningCorrection, UserLearningCorrection, WordBuffer,
    LEARNING_FEEDBACK_MAX_AGE_SECS,
};

impl WordBuffer {
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

    pub fn pending_auto_undo_ready(&mut self) -> bool {
        let Some(undo) = self.pending_auto_undo.as_ref() else {
            return false;
        };
        if undo.started_at.elapsed() > Duration::from_secs(LEARNING_FEEDBACK_MAX_AGE_SECS) {
            self.pending_auto_undo = None;
            return false;
        }
        true
    }

    pub fn restore_pending_auto_undo(&mut self, undo: PendingAutoUndo) {
        if undo.started_at.elapsed() <= Duration::from_secs(LEARNING_FEEDBACK_MAX_AGE_SECS) {
            self.pending_auto_undo = Some(undo);
        }
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
}
