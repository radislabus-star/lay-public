use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::engine::LayIbusEngine;
use super::protocol::{KEY_LEFT, KEY_RIGHT, KEY_UP};

impl LayIbusEngine {
    pub(super) async fn backspace(&mut self, emitter: &SignalEmitter<'_>) -> fdo::Result<bool> {
        self.preedit_dirty = false;
        if !self.buffer.is_empty() {
            if self.composition_cursor == 0 {
                self.clear_preedit(emitter).await?;
                self.buffer.clear();
                self.composition_cursor = 0;
                self.preedit_suffix.clear();
                self.preedit_candidates.clear();
                self.preedit_replacement_targets.clear();
                self.preedit_candidate_index = 0;
                self.preedit_fast.reset();
                return Ok(false);
            }
            let byte_idx = char_to_byte_idx(&self.buffer, self.composition_cursor - 1);
            self.buffer.remove(byte_idx);
            self.composition_cursor -= 1;
            self.sync_tail_from_composition();
            self.update_composition_preedit(emitter).await?;
            return Ok(true);
        }
        // The visible completion is virtual preedit, while the typed prefix is
        // already committed. Some clients (notably WeChat) otherwise consume
        // Backspace as preedit cancellation and leave the real prefix intact.
        // Hide the suffix first and do not republish it during the same key
        // event, so this Backspace reaches the committed character.
        self.clear_preedit(emitter).await?;
        self.backspace_committed_tail_only();
        Ok(false)
    }

    pub(super) fn backspace_committed_tail_only(&mut self) {
        self.tail_buffer.pop();
        self.preedit_fast.backspace();
        self.clear_preedit_completion_state();
        if self.last_tail_token_text().is_empty() {
            self.word_input_mode = None;
        }
        self.publish_tail_handoff();
    }

    pub(super) async fn move_composition_cursor(
        &mut self,
        emitter: &SignalEmitter<'_>,
        keyval: u32,
    ) -> fdo::Result<bool> {
        if self.buffer.is_empty() {
            self.forget_committed_tail_after_passive_cursor_move();
            self.clear_preedit(emitter).await?;
            return Ok(false);
        }
        let len = self.buffer.chars().count();
        match keyval {
            KEY_LEFT if self.composition_cursor > 0 => self.composition_cursor -= 1,
            KEY_RIGHT if self.composition_cursor < len => self.composition_cursor += 1,
            _ => {}
        }
        self.update_composition_preedit(emitter).await?;
        Ok(true)
    }

    pub(super) async fn select_precognition_candidate(
        &mut self,
        emitter: &SignalEmitter<'_>,
        keyval: u32,
    ) -> fdo::Result<bool> {
        let step = if keyval == KEY_UP { -1 } else { 1 };
        if self.buffer.is_empty() {
            if !self.cycle_precognition_candidate(step) {
                self.forget_committed_tail_after_passive_cursor_move();
                return Ok(false);
            }
            self.update_precognition_preedit(emitter).await?;
            return Ok(true);
        }
        if !self.cycle_precognition_candidate(step) {
            return Ok(false);
        }
        self.update_composition_preedit(emitter).await?;
        Ok(true)
    }

    pub(super) fn forget_committed_tail_after_passive_cursor_move(&mut self) {
        self.close_committed_tail_field();
    }

    pub(super) fn insert_composition_char(&mut self, ch: char) {
        let len = self.buffer.chars().count();
        self.composition_cursor = self.composition_cursor.min(len);
        if self.composition_cursor == len {
            self.buffer.push(ch);
            self.composition_cursor += 1;
            self.push_tail_char(ch);
            return;
        }
        let byte_idx = char_to_byte_idx(&self.buffer, self.composition_cursor);
        self.buffer.insert(byte_idx, ch);
        self.composition_cursor += 1;
        self.sync_tail_from_composition();
    }

    pub(super) fn sync_tail_from_composition(&mut self) {
        let replacement = self.buffer.clone();
        self.replace_last_tail_token_text(&replacement, 0);
        self.preedit_fast.reset();
        for ch in self.buffer.chars() {
            self.preedit_fast.push(ch);
        }
    }
}

fn char_to_byte_idx(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(idx, _)| idx)
        .unwrap_or(text.len())
}

#[cfg(test)]
mod tests {
    use super::{char_to_byte_idx, LayIbusEngine};
    use lay::config::LayConfig;
    use std::sync::{Arc, Mutex};

    fn engine() -> LayIbusEngine {
        LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        )
    }

    #[test]
    fn composition_cursor_edits_inside_buffer_without_committing() {
        let mut engine = engine();
        for ch in "abc".chars() {
            engine.insert_composition_char(ch);
        }
        assert_eq!(engine.buffer, "abc");
        assert_eq!(engine.composition_cursor, 3);
        engine.composition_cursor -= 1;
        engine.insert_composition_char('X');
        assert_eq!(engine.buffer, "abXc");
        assert_eq!(engine.composition_cursor, 3);
        let byte_idx = char_to_byte_idx(&engine.buffer, engine.composition_cursor - 1);
        engine.buffer.remove(byte_idx);
        engine.composition_cursor -= 1;
        engine.sync_tail_from_composition();
        assert_eq!(engine.buffer, "abc");
        assert_eq!(engine.composition_cursor, 2);
        assert_eq!(engine.tail_buffer, "abc");
    }

    #[test]
    fn composition_cursor_backspace_edits_before_cursor() {
        let mut engine = engine();
        for ch in "abcd".chars() {
            engine.insert_composition_char(ch);
        }
        engine.composition_cursor = 2;
        let byte_idx = char_to_byte_idx(&engine.buffer, engine.composition_cursor - 1);
        engine.buffer.remove(byte_idx);
        engine.composition_cursor -= 1;
        engine.sync_tail_from_composition();
        assert_eq!(engine.buffer, "acd");
        assert_eq!(engine.composition_cursor, 1);
        assert_eq!(engine.tail_buffer, "acd");
    }

    #[test]
    fn composition_cursor_at_start_does_not_swallow_backspace() {
        let mut engine = engine();
        for ch in "abc".chars() {
            engine.insert_composition_char(ch);
        }
        engine.composition_cursor = 0;

        assert_eq!(engine.buffer, "abc");
        assert_eq!(engine.composition_cursor, 0);
    }

    #[test]
    fn empty_composition_backspace_updates_memory_but_stays_unhandled() {
        let mut engine = engine();
        for ch in "тест".chars() {
            engine.push_tail_char(ch);
        }
        engine.backspace_committed_tail_only();
        assert_eq!(engine.buffer, "");
        assert_eq!(engine.tail_buffer, "тес");
        assert_eq!(engine.preedit_fast.token(), "тес");
    }

    #[test]
    fn committed_tail_backspace_dismisses_virtual_completion_before_editing_prefix() {
        let mut engine = engine();
        for ch in "прек".chars() {
            engine.push_tail_char(ch);
        }
        engine.preedit_suffix = "расный".to_string();
        engine.preedit_candidates = vec!["расный".to_string()];
        engine.preedit_replacement_targets = vec![None];

        engine.backspace_committed_tail_only();

        assert_eq!(engine.tail_buffer, "пре");
        assert_eq!(engine.preedit_fast.token(), "пре");
        assert!(engine.preedit_suffix.is_empty());
        assert!(engine.preedit_candidates.is_empty());
        assert!(engine.preedit_replacement_targets.is_empty());
    }

    #[test]
    fn passive_cursor_move_forgets_committed_tail() {
        let mut engine = engine();
        for ch in "ищем ".chars() {
            engine.push_tail_char(ch);
        }

        engine.forget_committed_tail_after_passive_cursor_move();

        assert!(engine.tail_buffer.is_empty());
        assert!(engine.preedit_fast.token().is_empty());
    }
}
