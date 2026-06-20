use zbus::fdo;
use zbus::object_server::SignalEmitter;

use lay::manual_toggle::{plan_manual_toggle, ManualToggleRequest, ManualToggleRoute};

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

        let Some(plan) = plan_manual_toggle(ManualToggleRequest {
            tail: &self.buffer,
            current_layout_is_ru: self.layout_is_ru,
            route: ManualToggleRoute::ImeActiveComposition,
            recover_missing_initial: false,
            preserve_trailing_whitespace: false,
        }) else {
            return Ok(false);
        };
        let original = std::mem::replace(&mut self.buffer, plan.replacement.clone());
        self.composition_cursor = self.buffer.chars().count();
        self.replace_last_tail_token_text(&plan.replacement, original.chars().count());
        self.commit_active_composition(emitter, ActiveCompositionCommit::plain())
            .await?;
        self.suppress_next_committed_tail_autocorrect = plan.suppress_next_autocorrect;
        self.sync_layout_after_manual_toggle(&plan.replacement);
        self.trace_key("double_shift_commit", 0, 0, true, None);
        Ok(true)
    }
}
