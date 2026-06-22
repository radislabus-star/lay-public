use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::engine::LayIbusEngine;
use super::state::CommittedTailReplaceRequest;
use super::text::make_ibus_text;
use super::trace;
use lay::manual_toggle::{plan_manual_toggle, ManualToggleRequest, VisibleTail};

impl LayIbusEngine {
    pub(super) async fn accept_stuck_tail(
        &mut self,
        emitter: &SignalEmitter<'_>,
        with_space: bool,
    ) -> fdo::Result<bool> {
        if self.tail_buffer.trim().is_empty() {
            return Ok(false);
        }
        let mut text = self.selected_visible_completion_suffix();
        if text.is_empty() {
            return Ok(false);
        }
        if with_space {
            text.push(' ');
        }
        self.clear_preedit(emitter).await?;
        Self::commit_text(emitter, make_ibus_text(text.clone()))
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        self.sync_tail_after_stuck_completion(&text);
        Ok(true)
    }

    fn sync_tail_after_stuck_completion(&mut self, text: &str) {
        for ch in text.chars() {
            self.push_tail_char(ch);
        }
    }

    pub(super) async fn toggle_committed_tail_target(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<Option<bool>> {
        let Some(plan) = self.committed_tail_toggle_plan() else {
            return Ok(None);
        };
        trace::record_manual_toggle_plan(&plan);
        let handled = self
            .replace_committed_tail(
                emitter,
                CommittedTailReplaceRequest::ime_manual_toggle(
                    plan.backspaces,
                    plan.replacement.clone(),
                    plan.suppress_next_autocorrect,
                ),
            )
            .await?;
        if handled {
            self.sync_layout_after_manual_toggle(&plan.replacement);
            self.trace_key("double_shift_committed_tail", 0, 0, true, None);
        }
        Ok(handled.then_some(plan.target_layout_is_ru))
    }

    fn committed_tail_toggle_plan(&self) -> Option<lay::manual_toggle::ManualTogglePlan> {
        plan_manual_toggle(ManualToggleRequest {
            visible_tail: VisibleTail::ime_committed_tail(&self.tail_buffer),
            current_layout_is_ru: self.layout_is_ru,
            recover_missing_initial: true,
            preserve_trailing_whitespace: true,
        })
    }

    pub(super) async fn apply_pending_committed_tail_space_autocorrect(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<bool> {
        let Some(pending) = self.pending_space_committed_tail_replace.take() else {
            return Ok(false);
        };
        let handled = self
            .replace_committed_tail(
                emitter,
                CommittedTailReplaceRequest::ime_autocorrect(
                    pending.backspaces,
                    pending.replacement.clone(),
                ),
            )
            .await?;
        if handled {
            self.sync_layout_after_committed_text(&pending.replacement);
            lay::action_log::record_action(
                "ime-typing-assist",
                &format!("{} ", pending.original),
                &pending.replacement,
                1,
                1,
                pending.started_at.elapsed().as_millis(),
                true,
            );
        }
        Ok(handled)
    }

    pub(super) async fn autocorrect_committed_tail_enter(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<bool> {
        if self.take_manual_toggle_autocorrect_suppression() {
            return Ok(false);
        }
        let Some((backspaces, replacement)) = self.committed_tail_boundary_replacement(false)
        else {
            return Ok(false);
        };
        let handled = self
            .replace_committed_tail(
                emitter,
                CommittedTailReplaceRequest::ime_autocorrect(backspaces, replacement.clone()),
            )
            .await?;
        if handled {
            self.sync_layout_after_committed_text(&replacement);
        }
        Ok(handled)
    }

    fn committed_tail_boundary_replacement(
        &self,
        include_separator: bool,
    ) -> Option<(u32, String)> {
        let token = self.last_tail_token_text();
        if token.is_empty() {
            return None;
        }
        let original = format!("{token} ");
        let replacement = self.autocorrect_committed_tail_text(&original)?;
        if replacement == original {
            return None;
        }
        let replacement = if include_separator {
            replacement
        } else {
            replacement
                .trim_end_matches(char::is_whitespace)
                .to_string()
        };
        Some((token.chars().count() as u32, replacement))
    }

    fn take_manual_toggle_autocorrect_suppression(&mut self) -> bool {
        let suppress = self.suppress_next_committed_tail_autocorrect
            || self.take_autocorrect_suppression_handoff();
        self.suppress_next_committed_tail_autocorrect = false;
        suppress
    }
}

#[cfg(test)]
mod tests {
    use super::LayIbusEngine;
    use lay::config::LayConfig;
    use std::sync::{Arc, Mutex};

    fn engine() -> LayIbusEngine {
        LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                auto_replace: true,
                typing_assist: true,
                auto_switch_layout: true,
                correction_safety: "experimental".to_string(),
                nanda_autocorrect: true,
                nanda_precognition: true,
                ..LayConfig::default()
            },
        )
    }

    #[test]
    fn stuck_completion_appends_suffix_to_tail_memory() {
        let mut engine = engine();
        for ch in "пров".chars() {
            engine.push_tail_char(ch);
        }

        engine.sync_tail_after_stuck_completion("ерка ");

        assert_eq!(engine.tail_buffer, "проверка ");
        assert_eq!(engine.preedit_fast.token(), "");
    }

    #[test]
    fn enter_boundary_uses_completed_tail_autocorrect_without_inserting_space() {
        let mut engine = engine();
        for ch in "fвтозамена".chars() {
            engine.push_tail_char(ch);
        }

        assert_eq!(
            engine.committed_tail_boundary_replacement(false),
            Some((10, "автозамена".to_string()))
        );
    }

    #[test]
    fn space_boundary_repairs_duplicate_latin_prefix_before_russian_word() {
        let mut engine = engine();
        for ch in "fавтозамена".chars() {
            engine.push_tail_char(ch);
        }

        assert_eq!(
            engine.committed_tail_boundary_replacement(true),
            Some((11, "автозамена ".to_string()))
        );
    }

    #[test]
    fn manual_toggle_suppresses_next_boundary_autocorrect_once() {
        let mut engine = engine();
        engine.suppress_next_committed_tail_autocorrect = true;

        assert!(engine.take_manual_toggle_autocorrect_suppression());
        assert!(!engine.take_manual_toggle_autocorrect_suppression());
    }

    #[test]
    fn manual_toggle_suppression_survives_engine_handoff() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let engine_a = LayIbusEngine::new(
            "/test/a".to_string(),
            Arc::clone(&shared),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                ..LayConfig::default()
            },
        );
        engine_a.publish_autocorrect_suppression_handoff();

        let mut engine_b = LayIbusEngine::new(
            "/test/b".to_string(),
            shared,
            false,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                ..LayConfig::default()
            },
        );

        assert!(engine_b.take_manual_toggle_autocorrect_suppression());
        assert!(!engine_b.take_manual_toggle_autocorrect_suppression());
    }

    #[test]
    fn committed_tail_toggle_plan_uses_visible_ime_tail_not_old_daemon_buffer() {
        let mut engine = engine();
        engine.tail_buffer.push_str("вот ");
        engine.layout_is_ru = true;

        let plan = engine.committed_tail_toggle_plan().expect("toggle plan");

        assert_eq!(plan.replacement, "djn ");
        assert_eq!(plan.backspaces, 4);
    }
}
