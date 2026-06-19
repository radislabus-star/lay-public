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
        if self.tail_buffer.trim().is_empty() || self.preedit_suffix.is_empty() {
            return Ok(false);
        }
        let mut text = self.last_tail_token_text();
        if text.is_empty() {
            return Ok(false);
        }
        if with_space {
            text.push(' ');
        }
        self.clear_preedit(emitter).await?;
        Self::commit_text(emitter, make_ibus_text(text))
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        self.tail_buffer.clear();
        self.preedit_fast.reset();
        Ok(true)
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
            .replace_committed_tail(emitter, backspaces as u32, replacement)
            .await?;
        if handled {
            self.trace_key("double_shift_committed_tail", 0, 0, true, None);
        }
        Ok(handled)
    }
}
