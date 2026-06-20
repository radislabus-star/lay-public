use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::engine::LayIbusEngine;
use super::text::make_ibus_text;
use lay::manual_toggle::{plan_manual_toggle, ManualToggleRequest, ManualToggleRoute};
use std::time::Instant;

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

    pub(super) async fn toggle_committed_tail(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<bool> {
        let Some(plan) = self.committed_tail_toggle_plan() else {
            return Ok(false);
        };
        let handled = self
            .replace_committed_tail(emitter, plan.backspaces, plan.replacement.clone())
            .await?;
        if handled {
            self.suppress_next_committed_tail_autocorrect = plan.suppress_next_autocorrect;
            self.sync_layout_after_manual_toggle(&plan.replacement);
            self.trace_key("double_shift_committed_tail", 0, 0, true, None);
        }
        Ok(handled)
    }

    fn committed_tail_toggle_plan(&self) -> Option<lay::manual_toggle::ManualTogglePlan> {
        plan_manual_toggle(ManualToggleRequest {
            tail: &self.tail_buffer,
            current_layout_is_ru: self.layout_is_ru,
            route: ManualToggleRoute::ImeCommittedTail,
            recover_missing_initial: true,
            preserve_trailing_whitespace: true,
        })
    }

    pub(super) async fn autocorrect_committed_tail_space(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<bool> {
        if self.take_manual_toggle_autocorrect_suppression() {
            return Ok(false);
        }
        let started_at = Instant::now();
        let Some((backspaces, replacement)) = self.committed_tail_boundary_replacement(true) else {
            return Ok(false);
        };
        let original = self.last_tail_token_text();
        let handled = self
            .replace_committed_tail(emitter, backspaces, replacement.clone())
            .await?;
        if handled {
            self.sync_layout_after_committed_text(&replacement);
            lay::action_log::record_action(
                "ime-typing-assist",
                &format!("{original} "),
                &replacement,
                1,
                1,
                started_at.elapsed().as_millis(),
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
            .replace_committed_tail(emitter, backspaces, replacement.clone())
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
        let suppress = self.suppress_next_committed_tail_autocorrect;
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
    fn double_shift_recovers_missing_initial_ascii_layout_letter() {
        let mut engine = engine();
        for ch in "hbdtn".chars() {
            engine.push_tail_char(ch);
        }

        let plan = engine.committed_tail_toggle_plan().expect("toggle plan");
        assert_eq!(plan.backspaces, 6);
        assert_eq!(plan.replacement, "привет");
    }

    #[test]
    fn double_shift_recovers_missing_initial_autozamena_letter() {
        let mut engine = engine();
        for ch in "dnjpfvtyf".chars() {
            engine.push_tail_char(ch);
        }

        let plan = engine.committed_tail_toggle_plan().expect("toggle plan");
        assert_eq!(plan.backspaces, 10);
        assert_eq!(plan.replacement, "автозамена");
    }

    #[test]
    fn double_shift_does_not_recover_missing_initial_for_short_ascii_tail() {
        let mut engine = engine();
        for ch in "в ima".chars() {
            engine.push_tail_char(ch);
        }

        let plan = engine
            .committed_tail_toggle_plan()
            .expect("plain double shift still toggles the short tail");

        assert_eq!(plan.backspaces, 3);
        assert_ne!(plan.replacement, "dime");
    }

    #[test]
    fn manual_toggle_suppresses_next_boundary_autocorrect_once() {
        let mut engine = engine();
        engine.suppress_next_committed_tail_autocorrect = true;

        assert!(engine.take_manual_toggle_autocorrect_suppression());
        assert!(!engine.take_manual_toggle_autocorrect_suppression());
    }
}
