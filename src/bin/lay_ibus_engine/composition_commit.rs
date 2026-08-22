use super::output::EngineOutput;
use std::time::Instant;
use zbus::fdo;

use super::engine::LayIbusEngine;
use super::text::make_ibus_text;
use super::trace;
use lay::text_edit::AuthorizedEdit;

enum ActiveCompositionAuthority {
    UserInput,
    VerifiedEdit(Box<AuthorizedEdit>),
}

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
            // Space only finalizes active IME composition. The verified
            // autocorrect route is committed-tail Space, while IME candidates
            // are accepted explicitly through Tab/Alt.
            autocorrect: false,
        }
    }
}

impl LayIbusEngine {
    pub(super) async fn commit_verified_active_composition(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
        authorized_edit: AuthorizedEdit,
    ) -> fdo::Result<()> {
        let text = authorized_edit.action().to_text().to_string();
        self.commit_authorized_active_composition_text(
            emitter,
            text,
            false,
            false,
            ActiveCompositionAuthority::VerifiedEdit(Box::new(authorized_edit)),
        )
        .await
    }

    pub(super) async fn commit_active_composition(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
        request: ActiveCompositionCommit,
    ) -> fdo::Result<()> {
        self.commit_active_composition_with_suffix(
            emitter,
            request.with_space,
            &request.suffix,
            request.sync_layout,
            request.autocorrect,
            ActiveCompositionAuthority::UserInput,
        )
        .await
    }

    pub(super) async fn accept_completion(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
        with_space: bool,
    ) -> fdo::Result<bool> {
        if self.buffer.is_empty() {
            if self.accept_stuck_tail(emitter, with_space).await? {
                return Ok(true);
            }
            return Ok(false);
        }

        let replacement = self
            .selected_precognition_replacement()
            .map(ToOwned::to_owned);
        let suffix = self.selected_visible_completion_suffix();
        if replacement.is_none() && suffix.is_empty() {
            return Ok(false);
        }

        let accepted_word = replacement.unwrap_or_else(|| format!("{}{}", self.buffer, suffix));
        let typed_prefix = self.buffer.clone();
        let accepted_text = if with_space {
            format!("{accepted_word} ")
        } else {
            accepted_word.clone()
        };
        let action = lay::text_edit::plan_ime_candidate_accept_edit(
            "ibus-active-composition-candidate-accept",
            900,
            self.buffer.clone(),
            accepted_text.clone(),
        );
        lay::action_log::record_candidate_edit_action_before_apply(
            &action,
            lay::action_log::MutationLogRoute::IME_ACTIVE_COMPOSITION,
            None,
        );
        let backend_action =
            lay::text_edit::authorize_backend_edit(lay::text_edit::TextEditBackend::Ime, action);
        let Some(authorized_edit) = backend_action.into_authorized() else {
            trace::record(r#"{"kind":"ibus_completion_accept_blocked"}"#);
            return Ok(false);
        };
        let context_tail = self
            .tail_buffer
            .strip_suffix(&typed_prefix)
            .unwrap_or(self.tail_buffer.as_str())
            .trim_end()
            .to_string();
        trace::record_completion_accept(
            "active_composition",
            accepted_word
                .chars()
                .count()
                .saturating_sub(self.buffer.chars().count()),
            with_space,
        );
        self.commit_authorized_active_composition_text(
            emitter,
            accepted_text,
            false,
            false,
            ActiveCompositionAuthority::VerifiedEdit(Box::new(authorized_edit)),
        )
        .await?;
        self.arm_pending_ime_completion_learning(
            context_tail,
            typed_prefix,
            accepted_word,
            with_space,
        );
        Ok(true)
    }

    pub(super) async fn accept_completion_with_space(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
    ) -> fdo::Result<bool> {
        let handled = self.accept_completion(emitter, true).await?;
        self.trace_key("alt_accept", 0, 0, handled, None);
        Ok(handled)
    }

    pub(super) async fn commit_managed_passthrough_char(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
        ch: char,
    ) -> fdo::Result<()> {
        emitter
            .commit_text(make_ibus_text(ch.to_string()))
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        self.last_commit_at = Some(Instant::now());
        self.push_tail_char(ch);
        let frame = self.capture_input_frame_identity();
        if !ch.is_whitespace() {
            if let Some(identity) = frame.as_ref() {
                self.schedule_space_autocorrect_prefetch(identity);
            }
        } else {
            self.invalidate_space_autocorrect_path();
        }
        self.refresh_precognition_after_visible_input(emitter, frame)
            .await?;
        Ok(())
    }

    pub(super) async fn observe_terminal_passthrough_char(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
        ch: char,
    ) -> fdo::Result<()> {
        self.push_tail_char(ch);
        let frame = self.capture_input_frame_identity();
        self.refresh_precognition_after_visible_input(emitter, frame)
            .await
    }

    /// Finalizes the currently active IME preedit composition.
    ///
    /// This is intentionally separate from `replace_committed_tail()`: this
    /// path commits text that the client still treats as live preedit, while
    /// committed-tail replacement edits text that is already in the widget.
    async fn commit_active_composition_with_suffix(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
        with_space: bool,
        suffix: &str,
        sync_layout: bool,
        autocorrect: bool,
        authority: ActiveCompositionAuthority,
    ) -> fdo::Result<()> {
        let mut text = self.buffer.clone();
        text.push_str(suffix);
        if with_space {
            text.push(' ');
        }
        self.commit_authorized_active_composition_text(
            emitter,
            text,
            sync_layout,
            autocorrect,
            authority,
        )
        .await
    }

    async fn commit_authorized_active_composition_text(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
        mut text: String,
        sync_layout: bool,
        autocorrect: bool,
        mut authority: ActiveCompositionAuthority,
    ) -> fdo::Result<()> {
        let started_at = Instant::now();
        if !authority.matches_text(&text) {
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
                    decision.action,
                );
                if let Some(edit) = backend_action.into_authorized() {
                    text = edit.action().to_text().to_string();
                    authority = ActiveCompositionAuthority::VerifiedEdit(Box::new(edit));
                } else {
                    trace::record(r#"{"kind":"ibus_active_composition_autocorrect_blocked"}"#);
                }
            }
        }
        let decision_ms = decision_started_at.elapsed().as_micros() as u64;
        if !authority.matches_text(&text) {
            trace::record(r#"{"kind":"ibus_active_composition_authorized_text_mismatch"}"#);
            return Ok(());
        }
        let clear_started_at = Instant::now();
        self.clear_preedit(emitter).await?;
        let clear_us = clear_started_at.elapsed().as_micros() as u64;
        let output_started_at = Instant::now();
        if let Err(error) = emitter.commit_text(make_ibus_text(text.clone())).await {
            let _ = self.update_composition_preedit(emitter).await;
            return Err(fdo::Error::Failed(error.to_string()));
        }
        let output_ms = output_started_at.elapsed().as_micros() as u64;
        if sync_layout {
            self.sync_layout_after_committed_text(&text, "active_composition");
        }
        self.sync_tail_after_active_composition_commit(&text);
        self.buffer.clear();
        self.composition_cursor = 0;
        self.arm_visible_postcondition(Instant::now());
        if text.ends_with(char::is_whitespace) {
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
                active_layout_is_ru: Some(self.layout_is_ru),
            },
        )
    }

    fn sync_tail_after_active_composition_commit(&mut self, text: &str) {
        self.sync_tail_after_composition_commit(text);
    }
}

impl ActiveCompositionAuthority {
    fn matches_text(&self, text: &str) -> bool {
        match self {
            Self::VerifiedEdit(edit) => {
                edit.backend() == lay::text_edit::TextEditBackend::Ime
                    && edit.action().to_text() == text
            }
            Self::UserInput => true,
        }
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
            source.contains("plan_ime_candidate_accept_edit("),
            "Tab/IME candidate accept must enter the shared typed edit plan"
        );
        assert!(
            source.contains("ActiveCompositionAuthority::VerifiedEdit(Box::new(authorized_edit))")
                && source.contains("authority.matches_text"),
            "accepted completion must hold AuthorizedEdit until CommitText"
        );
    }

    #[test]
    fn space_correction_is_scheduled_before_display_projection() {
        let source = include_str!("composition_commit.rs");
        let route = source
            .split("pub(super) async fn commit_managed_passthrough_char")
            .nth(1)
            .expect("managed passthrough route")
            .split("pub(super) async fn observe_terminal_passthrough_char")
            .next()
            .expect("single route body");
        let prefetch = route
            .find("schedule_space_autocorrect_prefetch")
            .expect("Space prefetch schedule");
        let refresh = route
            .find("refresh_precognition_after_visible_input")
            .expect("visible field refresh");

        assert!(
            prefetch < refresh,
            "Space correction must receive the shared frame before display projection"
        );
        assert_eq!(
            route.matches("capture_input_frame_identity").count(),
            1,
            "one printable event must capture one shared GUI identity"
        );
        assert!(
            route.contains("schedule_space_autocorrect_prefetch(identity)")
                && route.contains("refresh_precognition_after_visible_input(emitter, frame)"),
            "correction and display must receive clones of the same captured frame"
        );
        assert!(
            !route.contains("refresh_precognition_candidates("),
            "visible input must not materialize the field synchronously"
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
        assert!(!real_space.autocorrect);
    }
}
