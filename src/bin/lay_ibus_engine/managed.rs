use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::composition_commit::ActiveCompositionCommit;
use super::engine::LayIbusEngine;
use super::protocol::{
    has_command_modifier, is_accept_completion_with_space_key, KEY_BACKSPACE, KEY_DOWN, KEY_ENTER,
    KEY_KP_ENTER, KEY_LEFT, KEY_RIGHT, KEY_SPACE, KEY_TAB, KEY_UP,
};

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

    pub(super) async fn process_pressed_key(
        &mut self,
        emitter: &SignalEmitter<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> fdo::Result<bool> {
        if keyval == KEY_BACKSPACE {
            let handled = self.backspace(emitter).await?;
            self.trace_key("backspace", keyval, keycode, handled, None);
            return Ok(handled);
        }
        if keyval == KEY_LEFT || keyval == KEY_RIGHT {
            let handled = self.move_composition_cursor(emitter, keyval).await?;
            self.trace_key("cursor", keyval, keycode, handled, None);
            return Ok(handled);
        }
        if keyval == KEY_UP || keyval == KEY_DOWN {
            let handled = self.select_precognition_candidate(emitter, keyval).await?;
            self.trace_key("candidate_select", keyval, keycode, handled, None);
            return Ok(handled);
        }
        if keyval == KEY_TAB {
            let handled = self.accept_completion(emitter, false).await?;
            self.trace_key("tab", keyval, keycode, handled, None);
            return Ok(handled);
        }
        if is_accept_completion_with_space_key(keyval) {
            let handled = self.accept_completion(emitter, true).await?;
            self.trace_key("alt_accept", keyval, keycode, handled, None);
            return Ok(handled);
        }
        if has_command_modifier(state) {
            self.trace_key("command_passthrough", keyval, keycode, false, None);
            return Ok(false);
        }
        if keyval == KEY_ENTER || keyval == KEY_KP_ENTER {
            if !self.buffer.is_empty() {
                self.commit_active_composition(emitter, ActiveCompositionCommit::plain())
                    .await?;
                Self::forward_key_event(emitter, keyval, keycode, state)
                    .await
                    .map_err(|e| fdo::Error::Failed(e.to_string()))?;
                self.trace_key("enter_forward", keyval, keycode, true, None);
                return Ok(true);
            }
            self.tail_buffer.clear();
            self.preedit_fast.reset();
            self.clear_preedit(emitter).await?;
            self.trace_key("enter", keyval, keycode, false, None);
            return Ok(false);
        }
        if keyval == KEY_SPACE {
            let handled = self.commit_space(emitter).await?;
            self.trace_key("space", keyval, keycode, handled, Some(' '));
            return Ok(handled);
        }
        let Some(ch) = self.physical_char(keyval, keycode) else {
            if !self.buffer.is_empty() {
                self.commit_active_composition(emitter, ActiveCompositionCommit::plain())
                    .await?;
                self.trace_key("non_printable_commit", keyval, keycode, false, None);
                return Ok(false);
            }
            if !self.preedit_suffix.is_empty() {
                self.clear_preedit(emitter).await?;
                self.tail_buffer.clear();
                self.preedit_fast.reset();
            }
            self.trace_key("non_printable", keyval, keycode, false, None);
            return Ok(false);
        };
        if self.buffer.is_empty() && !can_start_ime_composition(ch) {
            self.trace_key("printable_passthrough", keyval, keycode, false, Some(ch));
            return Ok(false);
        }
        self.insert_composition_char(ch);
        self.update_composition_preedit(emitter).await?;
        self.trace_key("printable", keyval, keycode, true, Some(ch));
        Ok(true)
    }

    async fn commit_space(&mut self, emitter: &SignalEmitter<'_>) -> fdo::Result<bool> {
        if self.buffer.is_empty() {
            return Ok(false);
        }
        self.commit_active_composition(emitter, ActiveCompositionCommit::with_space())
            .await?;
        Ok(true)
    }

    async fn accept_completion(
        &mut self,
        emitter: &SignalEmitter<'_>,
        with_space: bool,
    ) -> fdo::Result<bool> {
        if self.buffer.is_empty() {
            return self.accept_stuck_tail(emitter, with_space).await;
        }

        let suffix = self.selected_visible_completion_suffix();
        if suffix.is_empty() && !with_space {
            return Ok(false);
        }

        self.commit_active_composition(
            emitter,
            ActiveCompositionCommit::with_completion(suffix, with_space),
        )
        .await?;
        Ok(true)
    }
}

fn can_start_ime_composition(ch: char) -> bool {
    ch.is_alphabetic() || is_ru_letter_punctuation_key(ch)
}

fn is_ru_letter_punctuation_key(ch: char) -> bool {
    matches!(
        ch,
        '`' | '~' | '[' | '{' | ']' | '}' | ';' | ':' | '\'' | '"' | ',' | '<' | '.' | '>'
    )
}

#[cfg(test)]
mod tests {
    use super::can_start_ime_composition;

    #[test]
    fn ime_starts_on_letters_and_ru_letter_punctuation_keys() {
        assert!(can_start_ime_composition('n'));
        assert!(can_start_ime_composition('т'));
        assert!(can_start_ime_composition('\''));
        assert!(can_start_ime_composition('['));
        assert!(can_start_ime_composition(';'));
        assert!(can_start_ime_composition(','));
    }

    #[test]
    fn ime_does_not_start_on_plain_special_symbols() {
        for ch in [
            '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '=', '/', '?', '\\', '|',
        ] {
            assert!(!can_start_ime_composition(ch), "{ch}");
        }
    }
}
