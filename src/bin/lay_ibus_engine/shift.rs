use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::composition_commit::ActiveCompositionCommit;
use super::engine::LayIbusEngine;

impl LayIbusEngine {
    pub(super) async fn handle_shift_release(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<bool> {
        if self.shift_used_as_modifier {
            self.shift_used_as_modifier = false;
            self.last_shift_release_at = None;
            return Ok(false);
        }
        let now = std::time::Instant::now();
        let double_tap = self
            .last_shift_release_at
            .is_some_and(|last| now.duration_since(last) <= super::engine::DOUBLE_SHIFT_WINDOW);
        self.last_shift_release_at = Some(now);
        if !double_tap {
            return Ok(false);
        }

        self.last_shift_release_at = None;
        if self.buffer.is_empty() {
            return self.toggle_committed_tail(emitter).await;
        }

        let converted = self.double_shift_replacement(&self.buffer);
        if converted == self.buffer {
            return Ok(false);
        }
        let original = std::mem::replace(&mut self.buffer, converted);
        self.composition_cursor = self.buffer.chars().count();
        let replacement = self.buffer.clone();
        self.replace_last_tail_token_text(&replacement, original.chars().count());
        self.commit_active_composition(emitter, ActiveCompositionCommit::plain())
            .await?;
        self.trace_key("double_shift_commit", 0, 0, true, None);
        Ok(true)
    }
}
