use super::output::EngineOutput;
use std::time::Instant;
use zbus::fdo;

use super::composition_commit::ActiveCompositionCommit;
use super::engine::{LayIbusEngine, WordInputMode};
use super::protocol::{
    has_command_modifier, KEY_BACKSPACE, KEY_DOWN, KEY_ENTER, KEY_KP_ENTER, KEY_LEFT, KEY_RIGHT,
    KEY_SPACE, KEY_TAB, KEY_UP,
};

impl LayIbusEngine {
    pub(super) async fn process_pressed_key(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> fdo::Result<bool> {
        let pressed_started = Instant::now();
        if self.composition.pending_passthrough_preedit_clear {
            self.clear_preedit(emitter).await?;
            self.composition.pending_passthrough_preedit_clear = false;
        }
        self.clear_pending_ime_auto_undo("next_pressed_key");
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
            if !self.composition.buffer.is_empty() {
                self.commit_active_composition(emitter, ActiveCompositionCommit::plain())
                    .await?;
                self.trace_key("enter_commit_passthrough", keyval, keycode, false, None);
                return Ok(false);
            }
            let tail_before_boundary = self.committed_tail.buffer.clone();
            self.finalize_pending_ime_completion_edit(&tail_before_boundary);
            self.close_committed_tail_field();
            self.clear_preedit(emitter).await?;
            self.trace_key("enter", keyval, keycode, false, None);
            return Ok(false);
        }
        if keyval == KEY_SPACE {
            let space_started = Instant::now();
            if self.composition.buffer.is_empty() {
                let initial_mode = self.initial_word_input_mode();
                let mode = *self.composition.word_input_mode.get_or_insert(initial_mode);
                let setup_us = space_started.elapsed().as_micros();
                if mode == WordInputMode::ManagedCommit {
                    if self.take_manual_toggle_autocorrect_suppression() {
                        super::trace::record(
                            r#"{"kind":"ibus_space_autocorrect","status":"manual_toggle_suppressed"}"#,
                        );
                        self.clear_preedit(emitter).await?;
                        self.cancel_precognition_display_generation();
                        let commit_started = Instant::now();
                        self.commit_managed_passthrough_char(emitter, ' ').await?;
                        let commit_us = commit_started.elapsed().as_micros();
                        super::trace::record_space_autocorrect_timing(
                            "manual_toggle_suppressed",
                            0,
                            0,
                            space_started.elapsed().as_micros(),
                        );
                        super::trace::record_space_key_timing(
                            "managed_manual_toggle_suppressed",
                            setup_us,
                            0,
                            commit_us,
                            space_started.elapsed().as_micros(),
                        );
                        self.trace_key("space_managed_commit", keyval, keycode, true, Some(' '));
                        return Ok(true);
                    }
                    let autocorrect_started = Instant::now();
                    let frame = self.capture_input_frame_identity();
                    let lookup = frame
                        .as_ref()
                        .map(|identity| self.take_space_autocorrect_lease(identity));
                    self.clear_preedit(emitter).await?;
                    self.cancel_precognition_display_generation();
                    let autocorrected = match (frame.as_ref(), lookup) {
                        (Some(identity), Some(lookup)) => {
                            self.autocorrect_committed_token_on_space(emitter, identity, lookup)
                                .await?
                        }
                        _ => false,
                    };
                    if autocorrected {
                        if let Some(identity) = frame.as_ref() {
                            self.invalidate_space_autocorrect_lease(identity);
                        }
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
                    if let Some(identity) = frame.as_ref() {
                        self.invalidate_space_autocorrect_lease(identity);
                    }
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
                self.clear_preedit(emitter).await?;
                self.cancel_precognition_display_generation();
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
            if !self.composition.buffer.is_empty() {
                self.commit_active_composition(emitter, ActiveCompositionCommit::plain())
                    .await?;
                self.trace_key("non_printable_commit", keyval, keycode, false, None);
                return Ok(false);
            }
            if !self.composition.preedit_suffix.is_empty() {
                self.clear_preedit(emitter).await?;
                self.committed_tail.buffer.clear();
                self.composition.preedit_fast.reset();
                self.publish_tail_handoff();
            }
            self.trace_key("non_printable", keyval, keycode, false, None);
            return Ok(false);
        };
        if ch.is_alphabetic() || is_completion_learning_boundary(ch) {
            self.confirm_pending_ime_completion_at_stable_boundary();
        }
        if self.composition.buffer.is_empty() {
            let initial_mode = self.initial_word_input_mode();
            let mode = *self.composition.word_input_mode.get_or_insert(initial_mode);
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
                super::trace::record_printable_key_timing(
                    "terminal_passthrough",
                    pressed_started.elapsed().as_micros(),
                );
                return Ok(false);
            }
            self.commit_managed_passthrough_char(emitter, ch).await?;
            self.trace_key("printable_managed_commit", keyval, keycode, true, Some(ch));
            super::trace::record_printable_key_timing(
                "managed_commit",
                pressed_started.elapsed().as_micros(),
            );
            return Ok(true);
        }
        self.insert_composition_char(ch);
        let frame = self.capture_input_frame_identity();
        self.update_composition_preedit_after_visible_input(emitter, frame)
            .await?;
        self.trace_key("printable", keyval, keycode, true, Some(ch));
        super::trace::record_printable_key_timing(
            "active_composition",
            pressed_started.elapsed().as_micros(),
        );
        Ok(true)
    }

    async fn commit_space(&mut self, emitter: &mut EngineOutput<'_, '_>) -> fdo::Result<bool> {
        if self.composition.buffer.is_empty() {
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
                && source.contains("self.take_space_autocorrect_lease(identity)")
                && source.contains("self.cancel_precognition_display_generation();")
                && source.contains("autocorrect_committed_token_on_space("),
            "managed Space must close verified token transitions through the shared decision core"
        );
    }

    #[test]
    fn managed_space_takes_lease_before_closing_display_generation() {
        let source = include_str!("managed.rs");
        let route = source
            .split("if mode == WordInputMode::ManagedCommit")
            .nth(1)
            .expect("managed Space route")
            .split("let autocorrect_started = Instant::now();")
            .nth(1)
            .expect("ordinary autocorrect route")
            .split("self.push_tail_char(' ')")
            .next()
            .expect("managed route body");
        let take = route
            .find("take_space_autocorrect_lease")
            .expect("lease take");
        let close = route
            .find("cancel_precognition_display_generation")
            .expect("display generation close");

        assert!(
            take < close,
            "Space must take its exact lease before display close"
        );
    }

    #[test]
    fn managed_space_consumes_manual_toggle_suppression_before_lookup() {
        let source = include_str!("managed.rs");
        let route = source
            .split("if mode == WordInputMode::ManagedCommit")
            .nth(1)
            .expect("managed Space route")
            .split("self.push_tail_char(' ')")
            .next()
            .expect("managed route body");
        let suppression = route
            .find("take_manual_toggle_autocorrect_suppression")
            .expect("manual-toggle suppression");
        let lookup = route
            .find("take_space_autocorrect_lease")
            .expect("space lookup");

        assert!(suppression < lookup);
        assert!(route.contains("manual_toggle_suppressed"));
    }
}
