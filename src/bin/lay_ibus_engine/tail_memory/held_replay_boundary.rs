//! Final boundary of the existing paired native replay, never a result ACK.
use super::*;
use crate::context_admission::{ExactManualSourceProvenance, ObservedReplayBoundary};
use crate::output::EngineOutput;
use crate::protocol::{ExactReplayBoundaryHold, KEY_SPACE, RELEASE_MASK};
use zbus::fdo;

impl LayIbusEngine {
    fn exact_replay_has_single_local_boundary(scope: &ExactManualToggleSuppression) -> bool {
        scope.source_provenance == Some(ExactManualSourceProvenance::ObservedWordAndLocalSpace)
            && scope
                .replacement
                .strip_suffix(' ')
                .is_some_and(|word| !word.is_empty() && !word.ends_with(char::is_whitespace))
    }

    pub(super) fn hold_exact_replay_boundary(
        &mut self,
        scope: &ExactManualToggleSuppression,
    ) -> bool {
        let projected_changes =
            scope.original_suffix.chars().count() + scope.replacement.chars().count();
        if self.atomic.active
            || self.context_handoff_sealed
            || self.client_context.exact_surrounding_refresh_available
            || scope.source_token.is_none()
            || scope.observed_external_prefix.is_none()
            || !Self::exact_replay_has_single_local_boundary(scope)
            || scope.native_press_count.checked_add(1) != Some(projected_changes)
            || !self.exact_replay_scope_is_current(scope)
        {
            return false;
        }
        let (Some(admission), Some(token), Some(word_scope)) = (
            self.context_admission.as_ref(),
            self.context_token.as_ref(),
            self.context_word_scope.as_ref(),
        ) else {
            return false;
        };
        // This key's callback is still outstanding. Its actual settlement must
        // remain text-inert before bridge validation can authorize any output.
        if !token.matches_word_scope(word_scope) || !admission.revalidate(token) {
            return false;
        }
        let mut held = scope.clone();
        held.held_boundary = Some(Box::new(ExactReplayBoundaryHold {
            token: token.clone(),
            focus_serial: self.client_context.focus_serial,
            layout_generation: self.layout_gesture.layout_generation,
            tail_epoch: self.committed_tail.epoch,
            observation_revision: self.client_context.surrounding_observation_revision,
            dispatched: false,
        }));
        let Ok(mut shared) = self.shared.lock() else {
            return false;
        };
        if !self.owns_shared_context_state(&shared)
            || shared.autocorrect_suppression.as_ref()
                != Some(&AutocorrectSuppression::ExactReplay(scope.clone()))
        {
            return false;
        }
        shared.autocorrect_suppression = Some(AutocorrectSuppression::ExactReplay(held.clone()));
        advance_suppression_revision(&mut shared);
        self.committed_tail.autocorrect_suppression =
            Some(AutocorrectSuppression::ExactReplay(held));
        true
    }

    fn held_replay_scope(&self) -> Option<ExactManualToggleSuppression> {
        match self.committed_tail.autocorrect_suppression.as_ref()? {
            AutocorrectSuppression::ExactReplay(scope) if scope.held_boundary.is_some() => {
                Some(scope.clone())
            }
            _ => None,
        }
    }

    pub(crate) fn has_held_replay_boundary(&self) -> bool {
        self.held_replay_scope().is_some()
    }

    /// Current field identity only; the native mirror is never a text source.
    pub(crate) fn held_replay_boundary_awaits_source(&self) -> bool {
        self.held_replay_scope()
            .as_ref()
            .is_some_and(|scope| self.held_replay_boundary_is_current(scope))
    }

    pub(crate) fn cancel_held_replay_boundary(&mut self) {
        if let Some(scope) = self.held_replay_scope() {
            self.revoke_exact_replay_scope(&scope);
        }
    }

    /// Deliver the already authorized final key before waiting for a client
    /// observation. Its callback stays handled so it cannot be delivered twice.
    pub(crate) async fn dispatch_held_replay_boundary(
        &mut self,
        output: &mut EngineOutput<'_, '_>,
        keycode: u32,
        state: u32,
    ) -> fdo::Result<()> {
        let Some(mut scope) = self.held_replay_scope().filter(|scope| {
            self.held_replay_boundary_delivery_is_current(scope)
                && scope
                    .held_boundary
                    .as_ref()
                    .is_some_and(|held| !held.dispatched)
        }) else {
            return Err(fdo::Error::Failed(
                "replay boundary delivery is stale".into(),
            ));
        };
        let token = scope
            .held_boundary
            .as_ref()
            .expect("held scope checked")
            .token
            .clone();
        let mut bridge = self.begin_context_bridge_output(Some(&token));
        output.require_surrounding_text().await?;
        for key_state in [state, state | RELEASE_MASK] {
            if !bridge.held_replay_boundary_delivery_is_current(&scope) {
                return Err(fdo::Error::Failed(
                    "replay boundary owner changed before delivery".into(),
                ));
            }
            output
                .forward_key_event(KEY_SPACE, keycode, key_state)
                .await?;
        }
        if !bridge.held_replay_boundary_delivery_is_current(&scope) {
            return Err(fdo::Error::Failed(
                "replay boundary owner changed during delivery".into(),
            ));
        }
        let previous = scope.clone();
        scope
            .held_boundary
            .as_mut()
            .expect("held scope checked")
            .dispatched = true;
        {
            let mut shared = bridge
                .shared
                .lock()
                .map_err(|_| fdo::Error::Failed("replay state unavailable".into()))?;
            if !bridge.owns_shared_context_state(&shared)
                || shared.autocorrect_suppression.as_ref()
                    != Some(&AutocorrectSuppression::ExactReplay(previous))
            {
                return Err(fdo::Error::Failed(
                    "replay boundary pair changed during delivery".into(),
                ));
            }
            shared.autocorrect_suppression =
                Some(AutocorrectSuppression::ExactReplay(scope.clone()));
            advance_suppression_revision(&mut shared);
        }
        bridge.committed_tail.autocorrect_suppression =
            Some(AutocorrectSuppression::ExactReplay(scope));
        bridge.complete();
        Ok(())
    }

    fn held_replay_boundary_matches_current_frame(
        &self,
        scope: &ExactManualToggleSuppression,
    ) -> bool {
        let Some(held) = scope.held_boundary.as_ref() else {
            return false;
        };
        let projected_changes =
            scope.original_suffix.chars().count() + scope.replacement.chars().count();
        self.held_replay_scope().as_ref() == Some(scope)
            && !self.atomic.active
            && !self.context_handoff_sealed
            && !self.client_context.exact_surrounding_refresh_available
            && held.focus_serial == self.client_context.focus_serial
            && held.layout_generation == self.layout_gesture.layout_generation
            && held.tail_epoch == self.committed_tail.epoch
            && held.observation_revision <= self.client_context.surrounding_observation_revision
            && self.context_token.as_ref() == Some(&held.token)
            && scope.source_token.is_some()
            && scope.observed_external_prefix.is_some()
            && Self::exact_replay_has_single_local_boundary(scope)
            && scope.native_press_count.checked_add(1) == Some(projected_changes)
            && self.exact_replay_scope_is_current(scope)
            && self
                .context_word_scope
                .as_ref()
                .is_some_and(|word| held.token.matches_word_scope(word))
    }

    /// The current key callback is still outstanding. Its existing delivery
    /// lease permits forwarding; it must not be mistaken for a settled source.
    fn held_replay_boundary_delivery_is_current(
        &self,
        scope: &ExactManualToggleSuppression,
    ) -> bool {
        self.held_replay_boundary_matches_current_frame(scope)
            && scope.held_boundary.as_ref().is_some_and(|held| {
                self.context_admission
                    .as_ref()
                    .is_some_and(|admission| admission.revalidate(&held.token))
            })
    }

    fn held_replay_boundary_is_current(&self, scope: &ExactManualToggleSuppression) -> bool {
        self.held_replay_boundary_matches_current_frame(scope)
            && scope.held_boundary.as_ref().is_some_and(|held| {
                self.context_admission.as_ref().is_some_and(|admission| {
                    admission
                        .exact_replay_source_token(&held.token, held.tail_epoch)
                        .is_some()
                })
            })
    }

    fn held_replay_word_is_observed(&self, scope: &ExactManualToggleSuppression) -> bool {
        let (Some(prefix), Some(word), Some(snapshot)) = (
            scope.observed_external_prefix.as_ref(),
            scope.replacement.strip_suffix(' '),
            self.client_context.surrounding_text_snapshot.as_ref(),
        ) else {
            return false;
        };
        !snapshot.has_selection()
            && snapshot.cursor_pos as usize == snapshot.text.chars().count()
            && snapshot.text == format!("{prefix}{}{word}", scope.unchanged_prefix)
    }

    pub(crate) async fn complete_held_replay_boundary(
        &mut self,
        _output: &mut EngineOutput<'_, '_>,
    ) -> fdo::Result<bool> {
        let Some(scope) = self.held_replay_scope() else {
            return Ok(false);
        };
        if !self.held_replay_boundary_is_current(&scope) {
            self.revoke_exact_replay_scope(&scope);
            return Ok(false);
        }
        if !self.held_replay_word_is_observed(&scope) {
            if self
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .is_some_and(|snapshot| {
                    snapshot.has_selection()
                        || snapshot.cursor_pos as usize != snapshot.text.chars().count()
                        || !self.exact_replay_contains_prior_snapshot(snapshot)
                })
            {
                self.revoke_exact_replay_scope(&scope);
            }
            return Ok(false);
        }
        if !scope
            .held_boundary
            .as_ref()
            .is_some_and(|held| held.dispatched)
        {
            return Ok(false);
        }
        let boundary = ObservedReplayBoundary {
            source_epoch: scope.epoch,
            word_epoch: self.committed_tail.epoch,
            projected_changes: scope.original_suffix.chars().count()
                + scope.replacement.chars().count(),
            observation_revision: self.client_context.surrounding_observation_revision,
            retained_boundary: self.committed_tail.buffer.chars().count() as u32,
        };
        let word_token = scope
            .held_boundary
            .as_ref()
            .expect("held scope checked")
            .token
            .clone();
        let mut bridge = self.begin_context_bridge_output(Some(&word_token));
        // Forwarding alone did not authorize a source. The actual complete
        // word and the current paired delivery are both required here.
        let next_token = bridge
            .held_replay_boundary_is_current(&scope)
            .then(|| bridge.context_admission.as_ref())
            .flatten()
            .and_then(|admission| {
                admission.settle_observed_replay_boundary(
                    scope.source_token.as_deref()?,
                    &scope.held_boundary.as_ref()?.token,
                    boundary,
                )
            });
        let Some(next_token) = next_token else {
            bridge.revoke_exact_replay_scope(&scope);
            bridge.revoke_context_word();
            return Err(fdo::Error::Failed(
                "replay boundary owner changed during output".into(),
            ));
        };
        bridge.committed_tail.buffer.push(' ');
        if !bridge.publish_tail_handoff_at_epoch(boundary.word_epoch + 1) {
            bridge.revoke_exact_replay_scope(&scope);
            bridge.revoke_context_word();
            return Err(fdo::Error::Failed(
                "replay boundary publication rejected".into(),
            ));
        }
        bridge.context_word_scope = Some(next_token.word_scope());
        bridge.context_token = Some(next_token.clone());
        if bridge.context_bridge_token.is_some() {
            bridge.context_bridge_token = Some(next_token);
        }
        bridge.revoke_exact_replay_scope(&scope);
        bridge.exact_replay_tail_change_quarantined = false;
        bridge.committed_tail.last_commit_at = Some(Instant::now());
        bridge.committed_tail.observed_boundary_source =
            Some(crate::engine::StockObservedBoundarySource {
                focus_serial: bridge.client_context.focus_serial,
                runtime_owner_lease_identity: bridge.client_context.runtime_owner_lease_identity,
                tail_epoch: bridge.committed_tail.epoch,
                observation_revision: boundary.observation_revision,
                tail: bridge.committed_tail.buffer.clone(),
                observed_external_prefix: scope.observed_external_prefix.clone(),
            });
        bridge.complete();
        Ok(true)
    }
}
