use super::output::EngineOutput;
use zbus::fdo;

use super::engine::LayIbusEngine;
use super::protocol::{KEY_LEFT, KEY_RIGHT, KEY_UP};

impl LayIbusEngine {
    pub(super) async fn backspace(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
    ) -> fdo::Result<bool> {
        self.invalidate_input_frame_background_work();
        self.composition.preedit_dirty = false;
        if !self.composition.buffer.is_empty() {
            if self.composition.cursor == 0 {
                self.clear_preedit(emitter).await?;
                self.composition.buffer.clear();
                self.composition.cursor = 0;
                self.composition.preedit_suffix.clear();
                self.composition.preedit_candidates.clear();
                self.composition.preedit_replacement_targets.clear();
                self.composition.preedit_candidate_index = 0;
                self.composition.preedit_fast.reset();
                return Ok(false);
            }
            let byte_idx = char_to_byte_idx(&self.composition.buffer, self.composition.cursor - 1);
            self.composition.buffer.remove(byte_idx);
            self.composition.cursor -= 1;
            self.sync_tail_from_composition();
            self.update_composition_preedit(emitter).await?;
            return Ok(true);
        }
        // LIVE CONTRACT - verified in WeChat on 2026-08-01.
        // The suffix is virtual preedit, but the typed prefix is already
        // committed. Keep this exact ownership order: hide preedit, update the
        // committed-tail mirror, then return handled=false. Do not republish a
        // suffix in this event. Otherwise WeChat consumes Backspace as preedit
        // cancellation and leaves the real prefix character intact.
        self.clear_preedit(emitter).await?;
        self.backspace_committed_tail_only();
        Ok(false)
    }

    pub(super) fn backspace_committed_tail_only(&mut self) {
        self.committed_tail.buffer.pop();
        self.composition.preedit_fast.backspace();
        self.clear_preedit_completion_state();
        if self.last_tail_token_text().is_empty() {
            self.composition.word_input_mode = None;
        }
        self.publish_tail_handoff();
    }

    pub(super) async fn move_composition_cursor(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
        keyval: u32,
    ) -> fdo::Result<bool> {
        let retired = self.retire_pending_precognition(emitter).await?;
        if self.composition.buffer.is_empty() {
            self.forget_committed_tail_after_passive_cursor_move();
            if !retired {
                self.clear_preedit(emitter).await?;
            }
            return Ok(false);
        }
        let len = self.composition.buffer.chars().count();
        match keyval {
            KEY_LEFT if self.composition.cursor > 0 => self.composition.cursor -= 1,
            KEY_RIGHT if self.composition.cursor < len => self.composition.cursor += 1,
            _ => {}
        }
        self.update_composition_preedit(emitter).await?;
        Ok(true)
    }

    pub(super) async fn select_precognition_candidate(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
        keyval: u32,
    ) -> fdo::Result<bool> {
        let refreshed_pending = self.composition.preedit_display_only_pending;
        if refreshed_pending {
            // Up/Down selects but never accepts. Rebuild authority from the
            // current token, cancel the older worker, and cycle that exact list.
            self.cancel_precognition_display_generation();
            self.composition.preedit_display_only_pending = false;
            self.refresh_precognition_candidates();
        }
        let step = if keyval == KEY_UP { -1 } else { 1 };
        if self.composition.buffer.is_empty() {
            if !self.cycle_precognition_candidate(step) {
                if refreshed_pending {
                    self.composition.preedit_display_only_pending = true;
                    self.retire_pending_precognition(emitter).await?;
                }
                self.forget_committed_tail_after_passive_cursor_move();
                return Ok(false);
            }
            if refreshed_pending {
                self.publish_selected_precognition_candidate(emitter)
                    .await?;
            } else {
                self.update_precognition_preedit(emitter).await?;
            }
            return Ok(true);
        }
        if !self.cycle_precognition_candidate(step) {
            if refreshed_pending {
                self.composition.preedit_display_only_pending = true;
                self.retire_pending_precognition(emitter).await?;
            }
            return Ok(false);
        }
        if refreshed_pending {
            self.publish_selected_precognition_candidate(emitter)
                .await?;
        } else {
            self.update_composition_preedit(emitter).await?;
        }
        Ok(true)
    }

    pub(super) fn forget_committed_tail_after_passive_cursor_move(&mut self) {
        self.close_committed_tail_field();
    }

    pub(super) fn insert_composition_char(&mut self, ch: char) {
        let len = self.composition.buffer.chars().count();
        self.composition.cursor = self.composition.cursor.min(len);
        if self.composition.cursor == len {
            self.composition.buffer.push(ch);
            self.composition.cursor += 1;
            self.push_tail_char(ch);
            return;
        }
        let byte_idx = char_to_byte_idx(&self.composition.buffer, self.composition.cursor);
        self.composition.buffer.insert(byte_idx, ch);
        self.composition.cursor += 1;
        self.sync_tail_from_composition();
    }

    pub(super) fn sync_tail_from_composition(&mut self) {
        let replacement = self.composition.buffer.clone();
        self.replace_last_tail_token_text(&replacement, 0);
        self.composition.preedit_fast.reset();
        for ch in self.composition.buffer.chars() {
            self.composition.preedit_fast.push(ch);
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
        assert_eq!(engine.composition.buffer, "abc");
        assert_eq!(engine.composition.cursor, 3);
        engine.composition.cursor -= 1;
        engine.insert_composition_char('X');
        assert_eq!(engine.composition.buffer, "abXc");
        assert_eq!(engine.composition.cursor, 3);
        let byte_idx = char_to_byte_idx(&engine.composition.buffer, engine.composition.cursor - 1);
        engine.composition.buffer.remove(byte_idx);
        engine.composition.cursor -= 1;
        engine.sync_tail_from_composition();
        assert_eq!(engine.composition.buffer, "abc");
        assert_eq!(engine.composition.cursor, 2);
        assert_eq!(engine.committed_tail.buffer, "abc");
    }

    #[test]
    fn composition_cursor_backspace_edits_before_cursor() {
        let mut engine = engine();
        for ch in "abcd".chars() {
            engine.insert_composition_char(ch);
        }
        engine.composition.cursor = 2;
        let byte_idx = char_to_byte_idx(&engine.composition.buffer, engine.composition.cursor - 1);
        engine.composition.buffer.remove(byte_idx);
        engine.composition.cursor -= 1;
        engine.sync_tail_from_composition();
        assert_eq!(engine.composition.buffer, "acd");
        assert_eq!(engine.composition.cursor, 1);
        assert_eq!(engine.committed_tail.buffer, "acd");
    }

    #[test]
    fn composition_cursor_at_start_does_not_swallow_backspace() {
        let mut engine = engine();
        for ch in "abc".chars() {
            engine.insert_composition_char(ch);
        }
        engine.composition.cursor = 0;

        assert_eq!(engine.composition.buffer, "abc");
        assert_eq!(engine.composition.cursor, 0);
    }

    #[test]
    fn empty_composition_backspace_updates_memory_but_stays_unhandled() {
        let mut engine = engine();
        for ch in "тест".chars() {
            engine.push_tail_char(ch);
        }
        engine.backspace_committed_tail_only();
        assert_eq!(engine.composition.buffer, "");
        assert_eq!(engine.committed_tail.buffer, "тес");
        assert_eq!(engine.composition.preedit_fast.token(), "тес");
    }

    #[test]
    fn wechat_live_contract_backspace_dismisses_completion_and_deletes_one_prefix_char() {
        let mut engine = engine();
        for ch in "прек".chars() {
            engine.push_tail_char(ch);
        }
        engine.composition.preedit_suffix = "расный".to_string();
        engine.composition.preedit_candidates = vec!["расный".to_string()];
        engine.composition.preedit_replacement_targets = vec![None];

        engine.backspace_committed_tail_only();

        assert_eq!(engine.committed_tail.buffer, "пре");
        assert_eq!(engine.composition.preedit_fast.token(), "пре");
        assert!(engine.composition.preedit_suffix.is_empty());
        assert!(engine.composition.preedit_candidates.is_empty());
        assert!(engine.composition.preedit_replacement_targets.is_empty());
    }

    #[test]
    fn passive_cursor_move_forgets_committed_tail() {
        let mut engine = engine();
        for ch in "ищем ".chars() {
            engine.push_tail_char(ch);
        }

        engine.forget_committed_tail_after_passive_cursor_move();

        assert!(engine.committed_tail.buffer.is_empty());
        assert!(engine.composition.preedit_fast.token().is_empty());
    }
}
