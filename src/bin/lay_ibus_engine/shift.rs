use zbus::fdo;
use zbus::object_server::SignalEmitter;

use lay::manual_toggle::{plan_manual_toggle, ManualToggleRequest, VisibleTail};

use super::composition_commit::ActiveCompositionCommit;
use super::engine::{LayIbusEngine, ManualToggleAuthority};
use super::trace;

impl LayIbusEngine {
    pub(super) async fn manual_toggle_active_text_target(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<Option<bool>> {
        match self.manual_toggle_authority() {
            ManualToggleAuthority::DaemonWordBuffer => {
                self.defer_committed_tail_manual_toggle_to_daemon();
                self.trace_key("double_shift_defer_to_daemon", 0, 0, false, None);
                return Ok(None);
            }
            ManualToggleAuthority::ImeCommittedTail => {
                return self.toggle_committed_tail_target(emitter).await;
            }
            ManualToggleAuthority::ImeActiveComposition => {}
        }
        let Some(plan) = plan_manual_toggle(ManualToggleRequest {
            visible_tail: VisibleTail::ime_active_composition(&self.buffer),
            current_layout_is_ru: self.layout_is_ru,
            recover_missing_initial: false,
            preserve_trailing_whitespace: false,
        }) else {
            return Ok(None);
        };
        trace::record_manual_toggle_plan(&plan);
        let original = std::mem::replace(&mut self.buffer, plan.replacement.clone());
        self.composition_cursor = self.buffer.chars().count();
        self.replace_last_tail_token_text(&plan.replacement, original.chars().count());
        self.commit_active_composition(emitter, ActiveCompositionCommit::plain())
            .await?;
        self.suppress_next_committed_tail_autocorrect = plan.suppress_next_autocorrect;
        self.sync_layout_after_manual_toggle(&plan.replacement);
        self.trace_key("double_shift_commit", 0, 0, true, None);
        Ok(Some(plan.target_layout_is_ru))
    }

    fn defer_committed_tail_manual_toggle_to_daemon(&mut self) {
        self.suppress_next_committed_tail_autocorrect = true;
    }
}

#[cfg(test)]
mod tests {
    use super::LayIbusEngine;
    use lay::config::LayConfig;
    use std::sync::{Arc, Mutex};

    #[test]
    fn deferring_committed_tail_toggle_suppresses_next_boundary_autocorrect() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );

        engine.defer_committed_tail_manual_toggle_to_daemon();

        assert!(engine.suppress_next_committed_tail_autocorrect);
    }
}
