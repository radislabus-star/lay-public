use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::engine::LayIbusEngine;
use super::text::make_ibus_text;

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
        let token = self.last_tail_token_text();
        if token.is_empty() {
            return Ok(false);
        }
        let converted = self.double_shift_replacement(&token);
        if converted == token {
            return Ok(false);
        }
        let trailing_ws = self.tail_trailing_whitespace_chars();
        let mut replacement = converted;
        for _ in 0..trailing_ws {
            replacement.push(' ');
        }
        let backspaces = token.chars().count().saturating_add(trailing_ws);
        let handled = self
            .replace_committed_tail(emitter, backspaces as u32, replacement.clone())
            .await?;
        if handled {
            self.sync_layout_after_manual_toggle(&replacement);
            self.trace_key("double_shift_committed_tail", 0, 0, true, None);
        }
        Ok(handled)
    }

    pub(super) async fn autocorrect_committed_tail_space(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<bool> {
        let token = self.last_tail_token_text();
        if token.is_empty() {
            return Ok(false);
        }
        let original = format!("{token} ");
        let Some(replacement) = self.autocorrect_committed_tail_text(&original) else {
            return Ok(false);
        };
        if replacement == original {
            return Ok(false);
        }
        let handled = self
            .replace_committed_tail(emitter, token.chars().count() as u32, replacement.clone())
            .await?;
        if handled {
            self.sync_layout_after_committed_text(&replacement);
        }
        Ok(handled)
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
}
