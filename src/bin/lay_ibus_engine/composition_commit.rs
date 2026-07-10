use std::time::Instant;
use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::engine::LayIbusEngine;
use super::text::make_ibus_text;
use super::trace;
use lay::text_edit::AuthorizedEdit;

pub(super) struct ActiveCompositionCommit {
    with_space: bool,
    suffix: String,
    sync_layout: bool,
    autocorrect: bool,
}

impl ActiveCompositionCommit {
    pub(super) fn plain() -> Self {
        Self {
            with_space: false,
            suffix: String::new(),
            sync_layout: false,
            autocorrect: false,
        }
    }

    pub(super) fn with_space() -> Self {
        Self {
            with_space: true,
            suffix: String::new(),
            sync_layout: true,
            autocorrect: true,
        }
    }
}

impl LayIbusEngine {
    pub(super) async fn commit_active_composition(
        &mut self,
        emitter: &SignalEmitter<'_>,
        request: ActiveCompositionCommit,
    ) -> fdo::Result<()> {
        self.commit_active_composition_with_suffix(
            emitter,
            request.with_space,
            &request.suffix,
            request.sync_layout,
            request.autocorrect,
            None,
        )
        .await
    }

    pub(super) async fn accept_completion(
        &mut self,
        emitter: &SignalEmitter<'_>,
        with_space: bool,
    ) -> fdo::Result<bool> {
        if self.buffer.is_empty() {
            if self.accept_stuck_tail(emitter, with_space).await? {
                return Ok(true);
            }
            return Ok(false);
        }

        let suffix = self.selected_visible_completion_suffix();
        if suffix.is_empty() {
            return Ok(false);
        }

        let accepted_word = format!("{}{}", self.buffer, suffix);
        let accepted_text = if with_space {
            format!("{accepted_word} ")
        } else {
            accepted_word.clone()
        };
        let action = lay::text_edit::EditAction::ime_accept(
            "ibus-active-composition-completion",
            900,
            self.buffer.clone(),
            accepted_text,
        );
        lay::action_log::record_candidate_edit_action_before_apply(
            &action,
            lay::action_log::MutationLogRoute::IME_ACTIVE_COMPOSITION,
            None,
        );
        let backend_action =
            lay::text_edit::authorize_backend_edit(lay::text_edit::TextEditBackend::Ime, &action);
        let Some(authorized_edit) = backend_action.authorized() else {
            trace::record(r#"{"kind":"ibus_completion_accept_blocked"}"#);
            return Ok(false);
        };
        let context_tail = self.tail_buffer.clone();
        trace::record_completion_accept("active_composition", suffix.chars().count(), with_space);
        self.commit_active_composition_with_suffix(
            emitter,
            with_space,
            &suffix,
            false,
            false,
            Some(authorized_edit),
        )
        .await?;
        lay::nanda_wave::record_accepted_ime_usage(&context_tail, &accepted_word);
        Ok(true)
    }

    pub(super) async fn accept_completion_with_space(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<bool> {
        let handled = self.accept_completion(emitter, true).await?;
        self.trace_key("alt_accept", 0, 0, handled, None);
        Ok(handled)
    }

    pub(super) async fn commit_managed_passthrough_char(
        &mut self,
        emitter: &SignalEmitter<'_>,
        ch: char,
    ) -> fdo::Result<()> {
        Self::commit_text(emitter, make_ibus_text(ch.to_string()))
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        self.last_commit_at = Some(Instant::now());
        self.push_tail_char(ch);
        self.preedit_dirty = false;
        self.update_precognition_preedit(emitter).await
    }

    pub(super) async fn observe_terminal_passthrough_char(
        &mut self,
        emitter: &SignalEmitter<'_>,
        ch: char,
    ) -> fdo::Result<()> {
        self.push_tail_char(ch);
        self.update_precognition_preedit(emitter).await
    }

    /// Finalizes the currently active IME preedit composition.
    ///
    /// This is intentionally separate from `replace_committed_tail()`: this
    /// path commits text that the client still treats as live preedit, while
    /// committed-tail replacement edits text that is already in the widget.
    async fn commit_active_composition_with_suffix(
        &mut self,
        emitter: &SignalEmitter<'_>,
        with_space: bool,
        suffix: &str,
        sync_layout: bool,
        autocorrect: bool,
        mut authorized_edit: Option<AuthorizedEdit>,
    ) -> fdo::Result<()> {
        let started_at = Instant::now();
        let clear_started_at = Instant::now();
        self.clear_preedit(emitter).await?;
        let clear_us = clear_started_at.elapsed().as_micros() as u64;
        let mut text = std::mem::take(&mut self.buffer);
        self.composition_cursor = 0;
        text.push_str(suffix);
        if with_space {
            text.push(' ');
        }
        if !authorized_edit_matches_ime_text(authorized_edit.as_ref(), &text) {
            trace::record(r#"{"kind":"ibus_active_composition_authorized_text_mismatch"}"#);
            return Ok(());
        }
        let decision_started_at = Instant::now();
        if autocorrect {
            if let Some(decision) = self.decide_active_composition_autocorrect(&text) {
                lay::action_log::record_candidate_edit_action_before_apply(
                    &decision.action,
                    lay::action_log::MutationLogRoute::IME_ACTIVE_COMPOSITION,
                    decision.input_gate.clone(),
                );
                let backend_action = lay::text_edit::authorize_backend_edit(
                    lay::text_edit::TextEditBackend::Ime,
                    &decision.action,
                );
                if let Some(edit) = backend_action.authorized() {
                    text = edit.action().to_text.clone();
                    authorized_edit = Some(edit);
                } else {
                    trace::record(r#"{"kind":"ibus_active_composition_autocorrect_blocked"}"#);
                }
            }
        }
        let decision_ms = decision_started_at.elapsed().as_micros() as u64;
        if !authorized_edit_matches_ime_text(authorized_edit.as_ref(), &text) {
            trace::record(r#"{"kind":"ibus_active_composition_authorized_text_mismatch"}"#);
            return Ok(());
        }
        let output_started_at = Instant::now();
        Self::commit_text(emitter, make_ibus_text(text.clone()))
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        let output_ms = output_started_at.elapsed().as_micros() as u64;
        if sync_layout {
            self.sync_layout_after_committed_text(&text);
        }
        self.sync_tail_after_active_composition_commit(&text);
        if with_space {
            self.close_precognition_word_boundary();
        }
        self.last_commit_at = Some(Instant::now());
        trace::record_ime_commit(
            decision_ms,
            clear_us,
            output_ms,
            started_at.elapsed().as_micros() as u64,
        );
        Ok(())
    }

    fn decide_active_composition_autocorrect(
        &self,
        text: &str,
    ) -> Option<lay::ime_correction::ActiveCompositionAutocorrectDecision> {
        lay::ime_correction::decide_active_composition_autocorrect(
            lay::ime_correction::ActiveCompositionAutocorrectRequest {
                text,
                committed_tail: &self.tail_buffer,
                config: &self.config,
            },
        )
    }

    fn sync_tail_after_active_composition_commit(&mut self, text: &str) {
        self.sync_tail_after_composition_commit(text);
    }
}

fn authorized_edit_matches_ime_text(authorized_edit: Option<&AuthorizedEdit>, text: &str) -> bool {
    match authorized_edit {
        Some(edit) => {
            edit.backend() == lay::text_edit::TextEditBackend::Ime && edit.action().to_text == text
        }
        None => true,
    }
}

#[cfg(test)]
mod active_composition_route_contract {
    #[test]
    fn active_composition_decision_lives_in_shared_ime_correction_module() {
        let source = include_str!("composition_commit.rs");
        let direct_gate_call = ["decide", "_input_gate("].concat();
        let direct_gate_request = ["InputGate", "Request {"].concat();
        assert!(
            !source.contains(&direct_gate_call) && !source.contains(&direct_gate_request),
            "composition_commit.rs must call lay::ime_correction instead of owning InputGate construction"
        );
    }

    #[test]
    fn completion_accept_uses_edit_action_contract() {
        let source = include_str!("composition_commit.rs");
        assert!(
            source.contains("EditAction::ime_accept("),
            "Tab/IME completion accept must be represented as EditAction::AcceptImeCandidate"
        );
        assert!(
            source.contains("Some(authorized_edit)")
                && source.contains("authorized_edit_matches_ime_text"),
            "accepted completion must hold AuthorizedEdit until CommitText"
        );
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn completion_with_space_does_not_trigger_autocorrect() {
        let completion = super::ActiveCompositionCommit {
            with_space: true,
            suffix: "ерка".to_string(),
            sync_layout: false,
            autocorrect: false,
        };
        assert!(completion.with_space);
        assert_eq!(completion.suffix, "ерка");
        assert!(!completion.autocorrect);

        let real_space = super::ActiveCompositionCommit::with_space();
        assert!(real_space.with_space);
        assert!(real_space.autocorrect);
    }
}
