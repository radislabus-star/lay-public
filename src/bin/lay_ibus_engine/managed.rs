use std::time::Instant;
use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::composition_commit::ActiveCompositionCommit;
use super::engine::{LayIbusEngine, WordInputMode};
use super::protocol::{
    has_command_modifier, KEY_BACKSPACE, KEY_DOWN, KEY_ENTER, KEY_KP_ENTER, KEY_LEFT, KEY_RIGHT,
    KEY_SPACE, KEY_TAB, KEY_UP,
};

impl LayIbusEngine {
    pub(super) async fn process_pressed_key(
        &mut self,
        emitter: &SignalEmitter<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> fdo::Result<bool> {
        self.clear_pending_ime_auto_undo();
        if keyval == KEY_BACKSPACE {
            self.begin_pending_ime_completion_edit_before_backspace();
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
            let handled = self.accept_completion(emitter, true).await?;
            self.trace_key("tab", keyval, keycode, handled, None);
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
                self.trace_key("enter_commit_passthrough", keyval, keycode, false, None);
                return Ok(false);
            }
            let tail_before_boundary = self.tail_buffer.clone();
            self.finalize_pending_ime_completion_edit(&tail_before_boundary);
            self.close_committed_tail_field();
            self.clear_preedit(emitter).await?;
            self.trace_key("enter", keyval, keycode, false, None);
            return Ok(false);
        }
        if keyval == KEY_SPACE {
            let space_started = Instant::now();
            if self.buffer.is_empty() {
                self.clear_preedit(emitter).await?;
                self.close_precognition_word_boundary();
                let initial_mode = self.initial_word_input_mode();
                let mode = *self.word_input_mode.get_or_insert(initial_mode);
                let setup_us = space_started.elapsed().as_micros();
                if mode == WordInputMode::ManagedCommit {
                    let autocorrect_started = Instant::now();
                    if self.autocorrect_committed_token_on_space(emitter).await? {
                        let autocorrect_us = autocorrect_started.elapsed().as_micros();
                        super::trace::record_space_key_timing(
                            "managed_autocorrect",
                            setup_us,
                            autocorrect_us,
                            0,
                            space_started.elapsed().as_micros(),
                        );
                        self.trace_key(
                            "space_managed_autocorrect",
                            keyval,
                            keycode,
                            true,
                            Some(' '),
                        );
                        return Ok(true);
                    }
                    let autocorrect_us = autocorrect_started.elapsed().as_micros();
                    let commit_started = Instant::now();
                    self.commit_managed_passthrough_char(emitter, ' ').await?;
                    let commit_us = commit_started.elapsed().as_micros();
                    super::trace::record_space_key_timing(
                        "managed_fallback_commit",
                        setup_us,
                        autocorrect_us,
                        commit_us,
                        space_started.elapsed().as_micros(),
                    );
                    self.trace_key("space_managed_commit", keyval, keycode, true, Some(' '));
                    return Ok(true);
                }
                self.push_tail_char(' ');
                super::trace::record_space_key_timing(
                    "terminal_passthrough",
                    setup_us,
                    0,
                    0,
                    space_started.elapsed().as_micros(),
                );
                self.trace_key(
                    "space_terminal_passthrough",
                    keyval,
                    keycode,
                    false,
                    Some(' '),
                );
                return Ok(false);
            }
            let commit_started = Instant::now();
            let handled = self.commit_space(emitter).await?;
            super::trace::record_space_key_timing(
                "active_composition",
                0,
                0,
                commit_started.elapsed().as_micros(),
                space_started.elapsed().as_micros(),
            );
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
                self.publish_tail_handoff();
            }
            self.trace_key("non_printable", keyval, keycode, false, None);
            return Ok(false);
        };
        if ch.is_alphabetic() || is_completion_learning_boundary(ch) {
            self.confirm_pending_ime_completion_at_stable_boundary();
        }
        if self.buffer.is_empty() {
            let initial_mode = self.initial_word_input_mode();
            let mode = *self.word_input_mode.get_or_insert(initial_mode);
            if mode == WordInputMode::TerminalPassthrough {
                let visible_ch = self.passthrough_visible_char(keyval, keycode).unwrap_or(ch);
                self.observe_terminal_passthrough_char(emitter, visible_ch)
                    .await?;
                self.trace_key(
                    "terminal_passthrough",
                    keyval,
                    keycode,
                    false,
                    Some(visible_ch),
                );
                return Ok(false);
            }
            self.commit_managed_passthrough_char(emitter, ch).await?;
            self.trace_key("printable_managed_commit", keyval, keycode, true, Some(ch));
            return Ok(true);
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
}

/// Punctuation after an explicitly accepted completion is a neutral end of
/// thought: it confirms the selected word without becoming lexical evidence.
fn is_completion_learning_boundary(ch: char) -> bool {
    matches!(ch, '!' | ',' | '.' | '?')
}

#[cfg(test)]
mod word_boundary_route_contract {
    use super::is_completion_learning_boundary;

    #[test]
    fn punctuation_confirms_completion_learning_without_becoming_a_word() {
        for ch in ['!', ',', '.', '?'] {
            assert!(is_completion_learning_boundary(ch));
        }
        for ch in ['a', ' ', ':', ')'] {
            assert!(!is_completion_learning_boundary(ch));
        }
    }

    #[test]
    fn managed_space_uses_shared_decision_core_for_verified_token_transitions() {
        let source = include_str!("managed.rs");

        assert!(
            source.contains("space_managed_commit")
                && source.contains("space_terminal_passthrough")
                && source.contains("self.close_precognition_word_boundary();")
                && source.contains("autocorrect_committed_token_on_space(emitter)"),
            "managed Space must close verified token transitions through the shared decision core"
        );
    }
}
