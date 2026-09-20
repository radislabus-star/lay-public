use super::engine::{LayIbusEngine, PendingImeCompletionLearning};
use super::protocol::{
    AutocorrectSuppression, CurrentWordSuppression, ExactManualToggleSuppression,
    PendingImeAutoUndo, PendingImeAutoUndoRetry, SharedState, ShiftGestureHandoff,
};
use lay::manual_toggle::{plan_manual_toggle, ManualToggleRequest, VisibleTail};
use std::time::{Duration, Instant};

#[cfg(test)]
std::thread_local! {
    static ACCEPTED_COMPLETION_FEEDBACK: std::cell::RefCell<Vec<(String, String)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

fn publish_accepted_completion_feedback(context_tail: &str, accepted_word: &str) {
    lay::typing_cpu::TypingCpu::record_accepted_completion(context_tail, accepted_word);
    #[cfg(test)]
    ACCEPTED_COMPLETION_FEEDBACK.with(|feedback| {
        feedback
            .borrow_mut()
            .push((context_tail.to_string(), accepted_word.to_string()));
    });
}

#[cfg(test)]
pub(crate) fn take_accepted_completion_feedback() -> Vec<(String, String)> {
    ACCEPTED_COMPLETION_FEEDBACK.with(|feedback| std::mem::take(&mut *feedback.borrow_mut()))
}

const IME_AUTO_UNDO_MAX_AGE: Duration = Duration::from_secs(30);
const IME_AUTO_UNDO_RETRY_MAX_AGE: Duration = Duration::from_secs(5);
const IME_LAYOUT_HANDOFF_MAX_AGE: Duration = Duration::from_millis(700);

fn advance_suppression_revision(state: &mut SharedState) {
    state.suppression_revision = state.suppression_revision.wrapping_add(1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SurroundingSnapshotMatch {
    Exact,
    AtomicSubmission,
    TrailingBoundaryElided,
    CausalPrecondition,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExactReplayPress {
    Inactive,
    Native,
    Rejected,
}

impl LayIbusEngine {
    fn owns_shared_context_state(&self, state: &SharedState) -> bool {
        !self.context_admission_required
            || state.active_path.as_deref() == Some(self.path.as_str())
                && match self.context_owner.as_ref() {
                    Some(owner) => state.context_owner_generation == Some(owner.generation.0),
                    None => state.context_owner_generation.is_none(),
                }
    }

    pub(super) fn remember_pending_ime_auto_undo(
        &self,
        original: String,
        replacement: String,
        transition: lay::typing_cpu::ObservedSystemTransition,
    ) {
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.shift_gesture_handoff = None;
        state.pending_auto_undo_retry = None;
        state.pending_auto_undo = (original != replacement
            && !original.trim().is_empty()
            && !replacement.trim().is_empty())
        .then_some(PendingImeAutoUndo {
            original,
            replacement,
            visible_tail: self.committed_tail.buffer.clone(),
            transition,
            recorded_at: Instant::now(),
            atomic_submission_proven: false,
        });
        record_pending_ime_auto_undo_lifecycle(
            self,
            &state,
            "remember",
            if state.pending_auto_undo.is_some() {
                "stored"
            } else {
                "rejected"
            },
        );
    }

    pub(super) fn take_pending_ime_auto_undo(&self) -> Option<PendingImeAutoUndo> {
        let Ok(mut state) = self.shared.lock() else {
            return None;
        };
        let Some(pending) = state.pending_auto_undo.take() else {
            state.shift_gesture_handoff = None;
            record_pending_ime_auto_undo_lifecycle(self, &state, "take", "missing");
            return None;
        };
        state.shift_gesture_handoff = None;
        state.pending_auto_undo_retry = None;
        if let Some(reason) = pending_ime_auto_undo_invalid_reason(self, &pending) {
            record_detached_ime_auto_undo_lifecycle(self, &state, &pending, "take", reason);
            return None;
        }
        record_detached_ime_auto_undo_lifecycle(self, &state, &pending, "take", "released");
        Some(pending)
    }

    pub(super) fn restore_pending_ime_auto_undo(&self, pending: PendingImeAutoUndo) {
        if let Some(reason) = pending_ime_auto_undo_invalid_reason(self, &pending) {
            if let Ok(state) = self.shared.lock() {
                record_detached_ime_auto_undo_lifecycle(self, &state, &pending, "restore", reason);
            }
            return;
        }
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.shift_gesture_handoff = None;
        state.pending_auto_undo_retry = None;
        state.pending_auto_undo = Some(pending);
        record_pending_ime_auto_undo_lifecycle(self, &state, "restore", "stored");
    }

    pub(super) fn clear_pending_ime_auto_undo(&self, reason: &'static str) {
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        record_pending_ime_auto_undo_lifecycle(self, &state, "clear", reason);
        state.shift_gesture_handoff = None;
        state.pending_auto_undo = None;
        state.pending_auto_undo_retry = None;
    }

    /// Records exact user intent while the client publishes the post-edit
    /// surrounding text. A client that keeps returning the exact precondition
    /// may use the recorded transition as authority; unrelated snapshots never
    /// release the undo.
    pub(super) fn defer_pending_ime_auto_undo_until_visible(&self) -> bool {
        if !self.client_context.surrounding_text_supported {
            if let Ok(state) = self.shared.lock() {
                record_pending_ime_auto_undo_lifecycle(
                    self,
                    &state,
                    "defer",
                    "surrounding_text_unsupported",
                );
            }
            return false;
        }
        let Ok(mut state) = self.shared.lock() else {
            return false;
        };
        let Some(pending) = state.pending_auto_undo.as_ref() else {
            state.pending_auto_undo_retry = None;
            record_pending_ime_auto_undo_lifecycle(self, &state, "defer", "missing");
            return false;
        };
        if let Some(reason) = pending_ime_auto_undo_invalid_reason(self, pending) {
            record_pending_ime_auto_undo_lifecycle(self, &state, "defer", reason);
            state.shift_gesture_handoff = None;
            state.pending_auto_undo = None;
            state.pending_auto_undo_retry = None;
            return false;
        }
        let snapshot_match = pending_ime_auto_undo_snapshot_match(
            self.client_context.surrounding_text_snapshot.as_ref(),
            pending,
        );
        if matches!(
            snapshot_match,
            SurroundingSnapshotMatch::Exact | SurroundingSnapshotMatch::AtomicSubmission
        ) {
            state.pending_auto_undo_retry = None;
            record_pending_ime_auto_undo_lifecycle(self, &state, "defer", "exact_snapshot_ready");
            return false;
        }
        let undo_recorded_at = pending.recorded_at;
        let requested_at = Instant::now();
        state.pending_auto_undo_retry = Some(PendingImeAutoUndoRetry {
            undo_recorded_at,
            requested_at,
        });
        state.preserve_active_path_until = Some(requested_at + IME_AUTO_UNDO_RETRY_MAX_AGE);
        match snapshot_match {
            SurroundingSnapshotMatch::TrailingBoundaryElided => {
                record_pending_ime_auto_undo_lifecycle(
                    self,
                    &state,
                    "defer",
                    "boundary_elided_snapshot_ready",
                );
                false
            }
            SurroundingSnapshotMatch::CausalPrecondition => {
                record_pending_ime_auto_undo_lifecycle(
                    self,
                    &state,
                    "defer",
                    "causal_precondition_snapshot_ready",
                );
                false
            }
            SurroundingSnapshotMatch::Missing => {
                record_pending_ime_auto_undo_lifecycle(
                    self,
                    &state,
                    "defer",
                    "waiting_exact_snapshot",
                );
                true
            }
            SurroundingSnapshotMatch::Exact | SurroundingSnapshotMatch::AtomicSubmission => false,
        }
    }

    pub(super) fn pending_ime_auto_undo_retry_status(&self) -> &'static str {
        let Ok(mut state) = self.shared.lock() else {
            return "state_unavailable";
        };
        let Some(retry) = state.pending_auto_undo_retry else {
            return "none";
        };
        if retry.requested_at.elapsed() > IME_AUTO_UNDO_RETRY_MAX_AGE {
            record_pending_ime_auto_undo_lifecycle(self, &state, "retry", "expired");
            state.pending_auto_undo_retry = None;
            return "expired";
        }
        let Some(pending) = state.pending_auto_undo.as_ref() else {
            record_pending_ime_auto_undo_lifecycle(self, &state, "retry", "missing_undo");
            state.shift_gesture_handoff = None;
            state.pending_auto_undo_retry = None;
            return "missing_undo";
        };
        if retry.undo_recorded_at != pending.recorded_at {
            record_pending_ime_auto_undo_lifecycle(self, &state, "retry", "superseded");
            state.pending_auto_undo_retry = None;
            return "superseded";
        }
        if state.active_path.as_deref() != Some(self.path.as_str()) {
            return "inactive_engine";
        }
        if let Some(reason) = pending_ime_auto_undo_invalid_reason(self, pending) {
            record_pending_ime_auto_undo_lifecycle(self, &state, "retry", reason);
            state.shift_gesture_handoff = None;
            state.pending_auto_undo = None;
            state.pending_auto_undo_retry = None;
            return "invalidated";
        }
        match pending_ime_auto_undo_snapshot_match(
            self.client_context.surrounding_text_snapshot.as_ref(),
            pending,
        ) {
            SurroundingSnapshotMatch::Exact | SurroundingSnapshotMatch::AtomicSubmission => "ready",
            SurroundingSnapshotMatch::TrailingBoundaryElided => "ready_boundary_elided",
            SurroundingSnapshotMatch::CausalPrecondition => "ready_causal_precondition",
            SurroundingSnapshotMatch::Missing => "waiting_exact_snapshot",
        }
    }

    pub(super) fn pending_ime_auto_undo_uses_boundary_elided_snapshot(&self) -> bool {
        self.pending_ime_auto_undo_snapshot_match()
            == SurroundingSnapshotMatch::TrailingBoundaryElided
    }

    pub(super) fn pending_ime_auto_undo_uses_causal_precondition_snapshot(&self) -> bool {
        self.pending_ime_auto_undo_snapshot_match() == SurroundingSnapshotMatch::CausalPrecondition
    }

    fn pending_ime_auto_undo_snapshot_match(&self) -> SurroundingSnapshotMatch {
        let Ok(state) = self.shared.lock() else {
            return SurroundingSnapshotMatch::Missing;
        };
        let Some(pending) = state.pending_auto_undo.as_ref() else {
            return SurroundingSnapshotMatch::Missing;
        };
        pending_ime_auto_undo_snapshot_match(
            self.client_context.surrounding_text_snapshot.as_ref(),
            pending,
        )
    }

    pub(super) fn arm_pending_ime_completion_learning(
        &mut self,
        context_tail: String,
        typed_prefix: String,
        accepted_word: String,
        with_space: bool,
    ) {
        if !self.context_word_is_known() {
            self.committed_tail.pending_completion_learning = None;
            return;
        }
        self.committed_tail.pending_completion_learning =
            with_space.then_some(PendingImeCompletionLearning {
                context_tail,
                typed_prefix,
                accepted_word,
                editing: false,
            });
    }

    pub(super) fn begin_pending_ime_completion_edit_before_backspace(&mut self) {
        let Some(pending) = self.committed_tail.pending_completion_learning.as_mut() else {
            return;
        };
        if pending.editing {
            return;
        }
        let accepted_tail = format!("{} ", pending.accepted_word);
        if self.committed_tail.buffer.ends_with(&accepted_tail) {
            pending.editing = true;
        } else {
            self.committed_tail.pending_completion_learning = None;
        }
    }

    /// A later word or terminal punctuation confirms that the explicitly
    /// accepted completion remained useful. This does not alter visible text.
    pub(super) fn confirm_pending_ime_completion_at_stable_boundary(&mut self) {
        if !self.context_word_is_known() {
            self.committed_tail.pending_completion_learning = None;
            return;
        }
        if self
            .committed_tail
            .pending_completion_learning
            .as_ref()
            .is_some_and(|pending| pending.editing)
        {
            return;
        }
        let Some(pending) = self.committed_tail.pending_completion_learning.take() else {
            return;
        };
        let accepted_tail = format!("{} ", pending.accepted_word);
        if self.committed_tail.buffer.ends_with(&accepted_tail) {
            publish_accepted_completion_feedback(&pending.context_tail, &pending.accepted_word);
        }
    }

    pub(super) fn finalize_pending_ime_completion_edit(
        &mut self,
        tail_before_boundary: &str,
    ) -> bool {
        if !self.context_word_is_known() {
            self.committed_tail.pending_completion_learning = None;
            return false;
        }
        let is_editing = self
            .committed_tail
            .pending_completion_learning
            .as_ref()
            .is_some_and(|pending| pending.editing);
        if !is_editing {
            return false;
        }
        let pending = self
            .committed_tail
            .pending_completion_learning
            .take()
            .expect("editing completion must remain pending");
        let final_word = lay::nanda_wave::llmwave::tokenize(tail_before_boundary)
            .into_iter()
            .next_back();
        match final_word {
            Some(final_word) if final_word == pending.accepted_word => {
                publish_accepted_completion_feedback(&pending.context_tail, &pending.accepted_word);
                super::trace::record(
                    r#"{"kind":"ibus_completion_edit","status":"accepted_unchanged"}"#,
                );
            }
            Some(final_word) => {
                lay::typing_cpu::TypingCpu::record_edited_completion(
                    &pending.context_tail,
                    &pending.typed_prefix,
                    &pending.accepted_word,
                    &final_word,
                );
                super::trace::record(format!(
                    r#"{{"kind":"ibus_completion_edit","status":"finalized","suggested":{},"final":{}}}"#,
                    serde_json::to_string(&pending.accepted_word)
                        .unwrap_or_else(|_| "\"\"".to_string()),
                    serde_json::to_string(&final_word).unwrap_or_else(|_| "\"\"".to_string()),
                ));
            }
            None => {
                lay::typing_cpu::TypingCpu::record_rejected_completion(
                    &pending.context_tail,
                    &pending.accepted_word,
                );
                super::trace::record(
                    r#"{"kind":"ibus_completion_edit","status":"deleted_without_target"}"#,
                );
            }
        }
        true
    }

    pub(super) fn selected_visible_completion_suffix(&self) -> String {
        if self.selected_precognition_replacement().is_some() {
            return String::new();
        }
        visible_completion_suffix(self.selected_precognition_suffix())
    }

    pub(super) fn last_tail_token_text(&self) -> String {
        last_tail_token(&self.committed_tail.buffer)
    }

    pub(super) fn sync_tail_after_composition_commit(&mut self, text: &str) {
        self.client_context.surrounding_text_snapshot = None;
        let trailing_ws = lay::word_reader::trailing_whitespace_char_count(text);
        let committed = text.trim_end_matches(char::is_whitespace);
        if !committed.is_empty() {
            self.replace_last_tail_token_text(committed, self.composition.buffer.chars().count());
        }
        for _ in 0..trailing_ws {
            self.committed_tail.buffer.push(' ');
        }
        if trailing_ws > 0 {
            self.composition.preedit_fast.reset();
        } else {
            self.rebuild_preedit_fast_from_tail();
        }
        trim_committed_tail_buffer(&mut self.committed_tail.buffer);
        self.publish_tail_handoff();
    }

    pub(super) fn replace_last_tail_token_text(&mut self, replacement: &str, fallback_len: usize) {
        let Some((start, end)) = last_tail_token_range(&self.committed_tail.buffer) else {
            self.committed_tail.buffer.push_str(replacement);
            return;
        };
        let range_len = self.committed_tail.buffer[start..end].chars().count();
        if fallback_len > 0 && range_len != fallback_len {
            self.committed_tail.buffer.push_str(replacement);
            return;
        }
        self.committed_tail
            .buffer
            .replace_range(start..end, replacement);
    }

    pub(super) fn rebuild_preedit_fast_from_tail(&mut self) {
        self.composition.preedit_fast.reset();
        if self.committed_tail.buffer.ends_with(char::is_whitespace) {
            return;
        }
        for ch in self.last_tail_token_text().chars() {
            self.composition.preedit_fast.push(ch);
        }
    }

    pub(super) fn publish_tail_handoff(&mut self) -> bool {
        let Ok(mut state) = self.shared.lock() else {
            return false;
        };
        if !self.owns_shared_context_state(&state) {
            return false;
        }
        self.committed_tail.epoch = self.committed_tail.epoch.wrapping_add(1);
        state.handoff_tail_buffer = self.committed_tail.buffer.clone();
        state.handoff_tail_epoch = self.committed_tail.epoch;
        state.handoff_focus_receipt = self.client_context.focus_receipt.clone();
        let had_exact_epoch = state.exact_manual_toggle_handoff_epoch.take().is_some();
        let had_exact_path = state.exact_manual_toggle_handoff_path.take().is_some();
        if had_exact_epoch || had_exact_path {
            advance_suppression_revision(&mut state);
        }
        drop(state);
        self.settle_context_bridge_output()
    }

    pub(super) fn prepare_exact_manual_toggle_layout_handoff(&mut self) {
        let source_snapshot_is_exact = self.current_external_snapshot_agrees_with_owned_tail();
        let source_token = self.context_token.clone();
        let source_observation_revision = self.client_context.surrounding_observation_revision;
        if let Some(AutocorrectSuppression::ExactReplay(scope)) =
            self.committed_tail.autocorrect_suppression.clone()
        {
            if self.exact_replay_scope_is_completed(&scope) {
                self.retire_completed_exact_replay(&scope);
            }
        }
        if !self.publish_tail_handoff() {
            return;
        }
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        if !self.owns_shared_context_state(&state) {
            return;
        }
        let expires_at = Instant::now() + IME_LAYOUT_HANDOFF_MAX_AGE;
        state.preserve_active_path_until = Some(expires_at);
        state.exact_manual_toggle_handoff_epoch = Some(self.committed_tail.epoch);
        state.exact_manual_toggle_handoff_path = Some(self.path.clone());
        advance_suppression_revision(&mut state);
        drop(state);
        if source_snapshot_is_exact {
            if let (Some(admission), Some(token)) =
                (self.context_admission.as_ref(), source_token.as_ref())
            {
                let _ = admission.publish_exact_manual_snapshot(
                    token,
                    self.committed_tail.epoch,
                    self.committed_tail.buffer.clone(),
                    source_observation_revision,
                    expires_at,
                );
            }
        }
    }

    pub(super) fn exact_manual_toggle_handoff_is_live(&self) -> bool {
        let Ok(mut state) = self.shared.lock() else {
            return false;
        };
        if !self.owns_shared_context_state(&state) {
            return false;
        }
        let now = Instant::now();
        let live = state
            .preserve_active_path_until
            .is_some_and(|until| now <= until)
            && state.exact_manual_toggle_handoff_epoch == Some(self.committed_tail.epoch)
            && state.handoff_tail_epoch == self.committed_tail.epoch
            && state.handoff_tail_buffer == self.committed_tail.buffer;
        if !live {
            let had_exact_epoch = state.exact_manual_toggle_handoff_epoch.take().is_some();
            let had_exact_path = state.exact_manual_toggle_handoff_path.take().is_some();
            if had_exact_epoch || had_exact_path {
                state.preserve_active_path_until = None;
                advance_suppression_revision(&mut state);
            }
        }
        live
    }

    pub(super) fn exact_manual_toggle_handoff_is_bound_to_current_owner(&self) -> bool {
        let Ok(state) = self.shared.lock() else {
            return false;
        };
        self.owns_shared_context_state(&state)
            && state.exact_manual_toggle_handoff_epoch.is_some()
            && state.exact_manual_toggle_handoff_path.is_some()
    }

    pub(super) fn current_external_snapshot_agrees_with_owned_tail(&self) -> bool {
        if !self.client_context.surrounding_text_supported || self.committed_tail.buffer.is_empty()
        {
            return false;
        }
        let Some(snapshot) = self.client_context.surrounding_text_snapshot.as_ref() else {
            return false;
        };
        let owned_tail_chars = self.committed_tail.buffer.chars().count();
        if snapshot.has_selection()
            || snapshot.suffix_before_cursor(owned_tail_chars).as_deref()
                != Some(self.committed_tail.buffer.as_str())
        {
            return false;
        }
        let Some((token_start, token_end)) = last_tail_token_range(&self.committed_tail.buffer)
        else {
            return false;
        };
        let token_chars = self.committed_tail.buffer[token_start..token_end]
            .chars()
            .count();
        let trailing_chars = self.committed_tail.buffer[token_end..].chars().count();
        let cursor = snapshot.cursor_pos as usize;
        let Some(external_token_end) = cursor.checked_sub(trailing_chars) else {
            return false;
        };
        let Some(external_token_start) = external_token_end.checked_sub(token_chars) else {
            return false;
        };
        let left_is_boundary = external_token_start == 0
            || snapshot
                .text
                .chars()
                .nth(external_token_start - 1)
                .is_some_and(super::preedit::is_observed_word_boundary);
        let right_is_boundary = snapshot
            .text
            .chars()
            .nth(external_token_end)
            .is_none_or(super::preedit::is_observed_word_boundary);
        left_is_boundary && right_is_boundary
    }

    pub(super) fn inherited_exact_manual_snapshot_agrees_with_owned_tail(&self) -> bool {
        let Some(receipt) = self.exact_manual_target_snapshot.as_ref() else {
            return false;
        };
        receipt.target_epoch == self.committed_tail.epoch
            && receipt.source.tail_epoch == self.committed_tail.epoch
            && receipt.source.tail == self.committed_tail.buffer
            && receipt.source.expires_at > Instant::now()
            && receipt.target_observation_revision
                == self.client_context.surrounding_observation_revision
            && self.client_context.surrounding_text_supported
            && !self.client_context.surrounding_text_callback_observed
            && self.content_allows_text_assistance()
            && self.context_token.as_ref() == Some(&receipt.target_token)
            && self
                .context_admission
                .as_ref()
                .is_some_and(|admission| admission.revalidate(&receipt.target_token))
    }

    pub(super) fn clear_identity_bound_exact_manual_toggle_authority(&mut self) {
        self.exact_manual_target_snapshot = None;
        let epoch = self.committed_tail.epoch;
        let path = self.path.as_str();
        let local_suppression_is_current = self
            .committed_tail
            .autocorrect_suppression
            .as_ref()
            .is_some_and(|suppression| {
                matches!(suppression, AutocorrectSuppression::ExactReplay(scope)
                    if scope.path == path && scope.epoch == epoch)
            });
        let Ok(mut state) = self.shared.lock() else {
            if local_suppression_is_current {
                self.committed_tail.autocorrect_suppression = None;
            }
            return;
        };
        let owns_shared_state = self.owns_shared_context_state(&state);
        let handoff_is_current = owns_shared_state
            && state.exact_manual_toggle_handoff_epoch == Some(epoch)
            && state.exact_manual_toggle_handoff_path.is_some()
            && state.handoff_tail_epoch == epoch;
        if handoff_is_current {
            state.preserve_active_path_until = None;
            state.exact_manual_toggle_handoff_epoch = None;
            state.exact_manual_toggle_handoff_path = None;
            advance_suppression_revision(&mut state);
        }
        if owns_shared_state
            && local_suppression_is_current
            && state.autocorrect_suppression.as_ref()
                == self.committed_tail.autocorrect_suppression.as_ref()
        {
            state.autocorrect_suppression = None;
            advance_suppression_revision(&mut state);
        }
        if local_suppression_is_current {
            self.committed_tail.autocorrect_suppression = None;
        }
    }

    pub(super) fn consume_exact_manual_toggle_handoff(&self) {
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        let had_preserve = state.preserve_active_path_until.take().is_some();
        let had_exact_epoch = state.exact_manual_toggle_handoff_epoch.take().is_some();
        let had_exact_path = state.exact_manual_toggle_handoff_path.take().is_some();
        if had_preserve || had_exact_epoch || had_exact_path {
            advance_suppression_revision(&mut state);
        }
    }

    pub(super) fn arm_exact_manual_toggle_autocorrect_suppression(
        &mut self,
        expected_suffix: &str,
        expected_epoch: u64,
        expected_path: &str,
        expected_layout_is_ru: bool,
    ) -> bool {
        let exact_tail_suffix = last_tail_token_range(&self.committed_tail.buffer)
            .map(|(start, _)| &self.committed_tail.buffer[start..]);
        let plan = plan_manual_toggle(ManualToggleRequest {
            visible_tail: VisibleTail::ime_committed_tail(&self.committed_tail.buffer),
            current_layout_is_ru: !expected_layout_is_ru,
            preserve_trailing_whitespace: true,
        });
        if expected_suffix.is_empty()
            || exact_tail_suffix != Some(expected_suffix)
            || self.path != expected_path
            || self.layout_gesture.layout_is_ru != expected_layout_is_ru
            || self.committed_tail.epoch != expected_epoch
            || !self.committed_tail.buffer.ends_with(expected_suffix)
            || plan.as_ref().is_none_or(|plan| {
                plan.target_layout_is_ru != expected_layout_is_ru
                    || plan.backspaces as usize != expected_suffix.chars().count()
            })
        {
            return false;
        }
        if !(self.current_external_snapshot_agrees_with_owned_tail()
            || self.inherited_exact_manual_snapshot_agrees_with_owned_tail())
        {
            self.clear_identity_bound_exact_manual_toggle_authority();
            return false;
        }
        let Ok(mut state) = self.shared.lock() else {
            return false;
        };
        let live = state.active_path.as_deref() == Some(expected_path)
            && state
                .preserve_active_path_until
                .is_some_and(|until| Instant::now() <= until)
            && state.exact_manual_toggle_handoff_epoch == Some(expected_epoch)
            && state.exact_manual_toggle_handoff_path.is_some()
            && state.handoff_tail_epoch == expected_epoch
            && state.handoff_tail_buffer == self.committed_tail.buffer;
        if !live {
            drop(state);
            self.clear_identity_bound_exact_manual_toggle_authority();
            return false;
        }

        state.preserve_active_path_until = None;
        state.exact_manual_toggle_handoff_epoch = None;
        state.exact_manual_toggle_handoff_path = None;
        let plan = plan.expect("validated exact manual-toggle projection");
        let unchanged_prefix = self
            .committed_tail
            .buffer
            .strip_suffix(expected_suffix)
            .expect("validated exact suffix")
            .to_string();
        let suppression = AutocorrectSuppression::ExactReplay(ExactManualToggleSuppression {
            path: expected_path.to_string(),
            epoch: expected_epoch,
            expires_at: Instant::now() + IME_LAYOUT_HANDOFF_MAX_AGE,
            owner_lease_identity: self.client_context.runtime_owner_lease_identity,
            target_layout_is_ru: expected_layout_is_ru,
            original_tail: self.committed_tail.buffer.clone(),
            original_suffix: expected_suffix.to_string(),
            unchanged_prefix,
            replacement: plan.replacement,
        });
        state.autocorrect_suppression = Some(suppression.clone());
        advance_suppression_revision(&mut state);
        self.committed_tail.autocorrect_suppression = Some(suppression);
        self.exact_manual_target_snapshot = None;
        drop(state);
        self.invalidate_input_frame_background_work();
        self.committed_tail.pending_completion_learning = None;
        self.client_context.surrounding_text_snapshot = None;
        self.composition.pending_passthrough_preedit_clear = self.composition.preedit_visible;
        self.clear_preedit_completion_state();
        self.composition.preedit_fast.reset();
        true
    }

    fn exact_replay_scope_pair(
        &self,
    ) -> (
        Option<ExactManualToggleSuppression>,
        Option<ExactManualToggleSuppression>,
    ) {
        let local = match self.committed_tail.autocorrect_suppression.as_ref() {
            Some(AutocorrectSuppression::ExactReplay(scope)) => Some(scope.clone()),
            _ => None,
        };
        let shared = self.shared.lock().ok().and_then(|state| {
            (state.active_path.as_deref() == Some(self.path.as_str()))
                .then(|| match state.autocorrect_suppression.as_ref() {
                    Some(AutocorrectSuppression::ExactReplay(scope)) => Some(scope.clone()),
                    _ => None,
                })
                .flatten()
        });
        (local, shared)
    }

    fn exact_replay_expected_tail(
        scope: &ExactManualToggleSuppression,
        distance: usize,
    ) -> Option<String> {
        let delete_chars = scope.original_suffix.chars().count();
        let replacement_chars = scope.replacement.chars().count();
        if distance > delete_chars.saturating_add(replacement_chars) {
            return None;
        }
        if distance <= delete_chars {
            let retained = scope.original_tail.chars().count().checked_sub(distance)?;
            return Some(scope.original_tail.chars().take(retained).collect());
        }
        let inserted = distance - delete_chars;
        Some(format!(
            "{}{}",
            scope.unchanged_prefix,
            scope.replacement.chars().take(inserted).collect::<String>()
        ))
    }

    fn exact_replay_scope_is_current_except_expiry(
        &self,
        scope: &ExactManualToggleSuppression,
    ) -> bool {
        if scope.path != self.path
            || scope.owner_lease_identity != self.client_context.runtime_owner_lease_identity
            || scope.target_layout_is_ru != self.layout_gesture.layout_is_ru
            || !self.live_composition_enabled()
            || !self.client_context.surrounding_text_supported
            || self.content_is_sensitive()
            || !self.composition.buffer.is_empty()
            || self
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.has_selection())
        {
            return false;
        }
        let distance = self.committed_tail.epoch.wrapping_sub(scope.epoch) as usize;
        if Self::exact_replay_expected_tail(scope, distance).as_deref()
            != Some(self.committed_tail.buffer.as_str())
        {
            return false;
        }
        let Ok(state) = self.shared.lock() else {
            return false;
        };
        self.owns_shared_context_state(&state)
            && state.active_path.as_deref() == Some(self.path.as_str())
            && state.autocorrect_suppression.as_ref()
                == Some(&AutocorrectSuppression::ExactReplay(scope.clone()))
            && state.handoff_tail_epoch == self.committed_tail.epoch
            && state.handoff_tail_buffer == self.committed_tail.buffer
    }

    fn exact_replay_scope_is_current(&self, scope: &ExactManualToggleSuppression) -> bool {
        Instant::now() <= scope.expires_at
            && self.exact_replay_scope_is_current_except_expiry(scope)
    }

    fn revoke_exact_replay_scope(&mut self, scope: &ExactManualToggleSuppression) {
        self.invalidate_input_frame_background_work();
        self.committed_tail.pending_completion_learning = None;
        self.client_context.surrounding_text_snapshot = None;
        self.clear_preedit_completion_state();
        if self.committed_tail.autocorrect_suppression.as_ref()
            == Some(&AutocorrectSuppression::ExactReplay(scope.clone()))
        {
            self.committed_tail.autocorrect_suppression = None;
        }
        if let Ok(mut state) = self.shared.lock() {
            if self.owns_shared_context_state(&state)
                && state.active_path.as_deref() == Some(self.path.as_str())
                && scope.path == self.path
                && scope.owner_lease_identity == self.client_context.runtime_owner_lease_identity
                && state.autocorrect_suppression.as_ref()
                    == Some(&AutocorrectSuppression::ExactReplay(scope.clone()))
            {
                state.autocorrect_suppression = None;
                advance_suppression_revision(&mut state);
            }
        }
    }

    fn exact_replay_mirror_transition(
        &mut self,
        scope: &ExactManualToggleSuppression,
        append: Option<char>,
    ) -> bool {
        self.invalidate_input_frame_background_work();
        self.committed_tail.pending_completion_learning = None;
        self.client_context.surrounding_text_snapshot = None;
        self.clear_preedit_completion_state();
        match append {
            Some(ch) => self.committed_tail.buffer.push(ch),
            None => {
                self.committed_tail.buffer.pop();
            }
        }
        self.exact_replay_tail_change_quarantined = true;
        if !self.publish_tail_handoff() || !self.exact_replay_scope_is_current(scope) {
            self.revoke_exact_replay_scope(scope);
            return false;
        }
        true
    }

    fn exact_replay_scope_is_completed(&self, scope: &ExactManualToggleSuppression) -> bool {
        self.committed_tail.epoch.wrapping_sub(scope.epoch) as usize
            == scope
                .original_suffix
                .chars()
                .count()
                .saturating_add(scope.replacement.chars().count())
            && self.exact_replay_scope_is_current_except_expiry(scope)
    }

    fn retire_completed_exact_replay(&mut self, scope: &ExactManualToggleSuppression) {
        if self.committed_tail.autocorrect_suppression.as_ref()
            == Some(&AutocorrectSuppression::ExactReplay(scope.clone()))
        {
            self.committed_tail.autocorrect_suppression = None;
        }
        if let Ok(mut state) = self.shared.lock() {
            if self.owns_shared_context_state(&state)
                && state.active_path.as_deref() == Some(self.path.as_str())
                && scope.path == self.path
                && scope.owner_lease_identity == self.client_context.runtime_owner_lease_identity
                && state.autocorrect_suppression.as_ref()
                    == Some(&AutocorrectSuppression::ExactReplay(scope.clone()))
            {
                state.autocorrect_suppression = None;
                advance_suppression_revision(&mut state);
            }
        }
        self.rebuild_preedit_fast_from_tail();
        if !self.committed_tail.buffer.ends_with(char::is_whitespace) {
            self.arm_current_word_autocorrect_suppression();
        }
    }

    pub(super) fn exact_replay_quarantine_active(&self) -> bool {
        let (local, shared) = self.exact_replay_scope_pair();
        matches!((local, shared), (Some(ref local), Some(ref shared))
            if local == shared
                && (self.exact_replay_scope_is_current(local)
                    || self.exact_replay_scope_is_current_except_expiry(local)
                        && self.committed_tail.epoch.wrapping_sub(local.epoch)
                            == local.original_suffix.chars().count() as u64
                                + local.replacement.chars().count() as u64))
    }

    pub(super) fn exact_replay_contains_prior_snapshot(
        &self,
        snapshot: &super::engine::SurroundingTextSnapshot,
    ) -> bool {
        let Some(AutocorrectSuppression::ExactReplay(scope)) =
            self.committed_tail.autocorrect_suppression.as_ref()
        else {
            return false;
        };
        let deletes = scope.original_suffix.chars().count();
        let replacements = scope.replacement.chars().count();
        let distance = self.committed_tail.epoch.wrapping_sub(scope.epoch) as usize;
        let complete_distance = deletes.saturating_add(replacements);
        let snapshot_chars = snapshot.text.chars().count();
        if snapshot.has_selection()
            || snapshot.cursor_pos as usize != snapshot_chars
            || distance > complete_distance
            || (distance < complete_distance && Instant::now() > scope.expires_at)
            || !self.exact_replay_scope_is_current_except_expiry(scope)
        {
            return false;
        }
        // Only already observed delete/replay progress can explain a delayed
        // surface. Future prefixes and foreign content cannot retain provenance.
        // This remains inert until a fresh exact client receipt arrives.
        let shortest_source = scope.original_tail.chars().count() - distance.min(deletes);
        snapshot.text.starts_with(&scope.unchanged_prefix)
            && ((snapshot_chars >= shortest_source
                && scope.original_tail.starts_with(&snapshot.text))
                || (distance >= deletes
                    && snapshot
                        .text
                        .strip_prefix(&scope.unchanged_prefix)
                        .is_some_and(|suffix| {
                            suffix.chars().count() <= distance - deletes
                                && scope.replacement.starts_with(suffix)
                        })))
    }

    pub(super) fn process_exact_replay_press(
        &mut self,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> ExactReplayPress {
        let (local, shared) = self.exact_replay_scope_pair();
        let Some(scope) = local.clone().or(shared.clone()) else {
            return ExactReplayPress::Inactive;
        };
        if local.as_ref() != Some(&scope) || shared.as_ref() != Some(&scope) {
            self.revoke_exact_replay_scope(&scope);
            return ExactReplayPress::Rejected;
        }

        let delete_chars = scope.original_suffix.chars().count();
        let distance = self.committed_tail.epoch.wrapping_sub(scope.epoch) as usize;
        if self.exact_replay_scope_is_completed(&scope) {
            self.retire_completed_exact_replay(&scope);
            return ExactReplayPress::Inactive;
        }
        if !self.exact_replay_scope_is_current(&scope) {
            self.revoke_exact_replay_scope(&scope);
            return ExactReplayPress::Rejected;
        }

        if super::protocol::has_command_modifier(state) {
            self.revoke_exact_replay_scope(&scope);
            return ExactReplayPress::Rejected;
        }

        let append = if distance < delete_chars {
            (keyval == super::protocol::KEY_BACKSPACE).then_some(None)
        } else {
            let inserted = distance - delete_chars;
            let expected = scope.replacement.chars().nth(inserted);
            (self.passthrough_visible_char(keyval, keycode) == expected)
                .then_some(expected)
                .flatten()
                .map(Some)
        };
        let Some(append) = append else {
            self.revoke_exact_replay_scope(&scope);
            return ExactReplayPress::Rejected;
        };
        if !self.exact_replay_mirror_transition(&scope, append) {
            return ExactReplayPress::Rejected;
        }
        ExactReplayPress::Native
    }

    pub(super) fn revoke_exact_manual_toggle_autocorrect_suppression(
        &mut self,
        expected_epoch: u64,
        expected_path: &str,
    ) -> bool {
        if self.path != expected_path {
            return false;
        }
        let Ok(mut state) = self.shared.lock() else {
            return false;
        };
        let matches = state.autocorrect_suppression.as_ref().is_some_and(|scope| {
            matches!(scope, AutocorrectSuppression::ExactReplay(identity) if
                identity.path == expected_path && identity.epoch == expected_epoch
            )
        });
        if !matches {
            return false;
        }
        state.autocorrect_suppression = None;
        advance_suppression_revision(&mut state);
        self.committed_tail.autocorrect_suppression = None;
        true
    }

    pub(super) fn arm_current_word_autocorrect_suppression(&mut self) -> bool {
        let open_token_chars = self.current_open_token_chars();
        if open_token_chars == 0 {
            self.retire_current_word_autocorrect_suppression();
            return false;
        }
        let suppression = AutocorrectSuppression::CurrentWord(CurrentWordSuppression {
            incarnation: super::engine::next_input_identity(),
            owner_lease_identity: self.client_context.runtime_owner_lease_identity,
            open_token_chars,
        });
        let Ok(mut state) = self.shared.lock() else {
            return false;
        };
        if state.active_path.as_deref() != Some(self.path.as_str()) {
            return false;
        }
        state.autocorrect_suppression = Some(suppression.clone());
        advance_suppression_revision(&mut state);
        self.committed_tail.autocorrect_suppression = Some(suppression);
        true
    }

    pub(super) fn arm_legacy_replay_autocorrect_suppression(&mut self) -> bool {
        let Ok(mut state) = self.shared.lock() else {
            return false;
        };
        if state.active_path.as_deref() != Some(self.path.as_str()) {
            return false;
        }
        state.autocorrect_suppression = Some(AutocorrectSuppression::LegacyReplayV1);
        advance_suppression_revision(&mut state);
        self.committed_tail.autocorrect_suppression = Some(AutocorrectSuppression::LegacyReplayV1);
        true
    }

    pub(super) fn arm_local_legacy_replay_autocorrect_suppression(&mut self) {
        self.committed_tail.autocorrect_suppression = Some(AutocorrectSuppression::LegacyReplayV1);
        if let Ok(mut state) = self.shared.lock() {
            if state.active_path.as_deref() == Some(self.path.as_str()) {
                advance_suppression_revision(&mut state);
            }
        }
    }

    pub(super) fn refresh_current_word_autocorrect_suppression(&mut self) {
        let open_token_chars = self.current_open_token_chars();
        if open_token_chars == 0 {
            self.retire_current_word_autocorrect_suppression();
            return;
        }
        let Some(AutocorrectSuppression::CurrentWord(mut local)) =
            self.committed_tail.autocorrect_suppression.clone()
        else {
            return;
        };
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        if state.autocorrect_suppression.as_ref()
            != Some(&AutocorrectSuppression::CurrentWord(local))
        {
            self.committed_tail.autocorrect_suppression = None;
            return;
        }
        if local.owner_lease_identity != self.client_context.runtime_owner_lease_identity {
            self.committed_tail.autocorrect_suppression = None;
            return;
        }
        if local.open_token_chars != open_token_chars {
            local.open_token_chars = open_token_chars;
            let suppression = AutocorrectSuppression::CurrentWord(local);
            state.autocorrect_suppression = Some(suppression.clone());
            advance_suppression_revision(&mut state);
            self.committed_tail.autocorrect_suppression = Some(suppression);
        }
    }

    pub(super) fn backspace_current_word_autocorrect_suppression(&mut self) {
        let Some(AutocorrectSuppression::CurrentWord(mut local)) =
            self.committed_tail.autocorrect_suppression.clone()
        else {
            return;
        };
        if local.open_token_chars <= 1 {
            self.retire_current_word_autocorrect_suppression();
            return;
        }
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        if state.autocorrect_suppression.as_ref()
            != Some(&AutocorrectSuppression::CurrentWord(local))
            || local.owner_lease_identity != self.client_context.runtime_owner_lease_identity
        {
            self.committed_tail.autocorrect_suppression = None;
            return;
        }
        local.open_token_chars -= 1;
        let suppression = AutocorrectSuppression::CurrentWord(local);
        state.autocorrect_suppression = Some(suppression.clone());
        advance_suppression_revision(&mut state);
        self.committed_tail.autocorrect_suppression = Some(suppression);
    }

    pub(super) fn retire_current_word_autocorrect_suppression(&mut self) {
        if !matches!(
            self.committed_tail.autocorrect_suppression.as_ref(),
            Some(AutocorrectSuppression::CurrentWord(_))
        ) {
            return;
        }
        let local = self
            .committed_tail
            .autocorrect_suppression
            .take()
            .expect("CurrentWord checked above");
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        if state.autocorrect_suppression.as_ref() == Some(&local) {
            state.autocorrect_suppression = None;
            advance_suppression_revision(&mut state);
        }
    }

    fn current_open_token_chars(&self) -> usize {
        if !self.composition.preedit_fast.has_open_token() {
            return 0;
        }
        super::preedit::uncapped_open_token_chars(&self.committed_tail.buffer)
    }

    pub(super) fn close_committed_tail_field(&mut self) {
        self.committed_tail.pending_completion_learning = None;
        self.committed_tail.buffer.clear();
        self.composition.preedit_fast.reset();
        let local_suppression = self.committed_tail.autocorrect_suppression.take();
        self.composition.word_input_mode = None;
        self.committed_tail.last_input_at = None;
        self.committed_tail.last_commit_at = None;
        self.committed_tail.recent_replace = None;
        self.layout_gesture.pending_manual_toggle = false;
        self.committed_tail.pending_visible_postcondition = None;
        self.committed_tail.epoch = self.committed_tail.epoch.wrapping_add(1);
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        if !self.owns_shared_context_state(&state) {
            return;
        }
        record_pending_ime_auto_undo_lifecycle(self, &state, "clear", "close_committed_tail_field");
        state.handoff_tail_buffer.clear();
        state.handoff_tail_epoch = self.committed_tail.epoch;
        state.handoff_focus_receipt = None;
        let active_owner = state.active_path.as_deref() == Some(self.path.as_str());
        let cleared_shared = active_owner
            && (local_suppression.is_none()
                || state.autocorrect_suppression.as_ref() == local_suppression.as_ref())
            && state.autocorrect_suppression.take().is_some();
        if cleared_shared || active_owner && local_suppression.is_some() {
            advance_suppression_revision(&mut state);
        }
        state.preserve_active_path_until = None;
        state.exact_manual_toggle_handoff_epoch = None;
        state.exact_manual_toggle_handoff_path = None;
        state.pending_auto_undo = None;
        state.pending_auto_undo_retry = None;
        state.shift_gesture_handoff = None;
    }

    pub(crate) fn quarantine_visible_postcondition_mismatch(&mut self) {
        let shared = self.shared.clone();
        self.composition.buffer.clear();
        self.composition.cursor = 0;
        self.committed_tail.buffer.clear();
        self.composition.preedit_fast.reset();
        self.clear_preedit_completion_state();
        self.composition.word_input_mode = None;
        self.committed_tail.last_input_at = None;
        self.committed_tail.recent_replace = None;
        self.layout_gesture.pending_manual_toggle = false;
        let local_suppression = self.committed_tail.autocorrect_suppression.take();
        self.committed_tail.epoch = self.committed_tail.epoch.wrapping_add(1);
        if let Ok(mut state) = shared.lock() {
            if !self.owns_shared_context_state(&state) {
                return;
            }
            record_pending_ime_auto_undo_lifecycle(
                self,
                &state,
                "clear",
                "visible_postcondition_mismatch",
            );
            state.handoff_tail_buffer.clear();
            state.handoff_tail_epoch = self.committed_tail.epoch;
            state.handoff_focus_receipt = None;
            let active_owner = state.active_path.as_deref() == Some(self.path.as_str());
            let cleared_shared = active_owner
                && (local_suppression.is_none()
                    || state.autocorrect_suppression.as_ref() == local_suppression.as_ref())
                && state.autocorrect_suppression.take().is_some();
            if cleared_shared || active_owner && local_suppression.is_some() {
                advance_suppression_revision(&mut state);
            }
            state.preserve_active_path_until = None;
            state.exact_manual_toggle_handoff_epoch = None;
            state.exact_manual_toggle_handoff_path = None;
            state.pending_auto_undo = None;
            state.pending_auto_undo_retry = None;
            state.shift_gesture_handoff = None;
        };
    }

    pub(super) fn refresh_empty_tail_from_handoff(&mut self) {
        if !self.committed_tail.buffer.is_empty() {
            return;
        }
        let Ok(state) = self.shared.lock() else {
            return;
        };
        if state.handoff_tail_buffer.is_empty() {
            return;
        }
        self.committed_tail
            .buffer
            .clone_from(&state.handoff_tail_buffer);
        self.committed_tail.epoch = state.handoff_tail_epoch;
        drop(state);
        self.rebuild_preedit_fast_from_tail();
    }

    pub(super) fn clear_autocorrect_suppression_handoff(&mut self) {
        let local_suppression = self.committed_tail.autocorrect_suppression.take();
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        if !self.owns_shared_context_state(&state) {
            return;
        }
        let active_owner = state.active_path.as_deref() == Some(self.path.as_str());
        let cleared_shared = active_owner
            && (local_suppression.is_none()
                || state.autocorrect_suppression.as_ref() == local_suppression.as_ref())
            && state.autocorrect_suppression.take().is_some();
        if cleared_shared || active_owner && local_suppression.is_some() {
            advance_suppression_revision(&mut state);
        }
    }

    pub(super) fn publish_active_path_preserve_handoff(&self, until: Instant) {
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.preserve_active_path_until = Some(until);
        advance_suppression_revision(&mut state);
    }

    pub(super) fn shared_active_path_preserved(&self) -> bool {
        let Ok(mut state) = self.shared.lock() else {
            return false;
        };
        let Some(until) = state.preserve_active_path_until else {
            return false;
        };
        if Instant::now() <= until {
            return true;
        }
        state.preserve_active_path_until = None;
        state.exact_manual_toggle_handoff_epoch = None;
        state.exact_manual_toggle_handoff_path = None;
        state.shift_gesture_handoff = None;
        advance_suppression_revision(&mut state);
        false
    }

    pub(super) fn publish_shift_gesture_handoff(&self) {
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        let now = Instant::now();
        let lease_is_live = state
            .preserve_active_path_until
            .is_some_and(|until| now <= until);
        if !lease_is_live || state.pending_auto_undo.is_none() {
            state.shift_gesture_handoff = None;
            return;
        }
        if state.active_path.as_deref() != Some(self.path.as_str()) {
            return;
        }
        state.shift_gesture_handoff = Some(ShiftGestureHandoff {
            source_path: self.path.clone(),
            shift_active: self.layout_gesture.shift_active,
            shift_pressed_at: self.layout_gesture.shift_pressed_at,
            shift_used_as_modifier: self.layout_gesture.shift_used_as_modifier,
            last_shift_release_at: self.layout_gesture.last_shift_release_at,
        });
        super::trace::record(format!(
            r#"{{"kind":"ibus_shift_gesture_handoff","stage":"publish","source":"{}","shift_active":{},"used_as_modifier":{}}}"#,
            self.path, self.layout_gesture.shift_active, self.layout_gesture.shift_used_as_modifier,
        ));
    }

    pub(super) fn consume_shift_gesture_handoff(&mut self) {
        let handoff = {
            let Ok(mut state) = self.shared.lock() else {
                return;
            };
            let now = Instant::now();
            let lease_is_live = state
                .preserve_active_path_until
                .is_some_and(|until| now <= until);
            if !lease_is_live || state.pending_auto_undo.is_none() {
                state.shift_gesture_handoff = None;
                return;
            }
            if state.active_path.as_deref() != Some(self.path.as_str()) {
                return;
            }
            let Some(gesture) = state.shift_gesture_handoff.as_ref() else {
                return;
            };
            if gesture.source_path == self.path {
                return;
            }
            state.shift_gesture_handoff.take()
        };
        let Some(handoff) = handoff else {
            return;
        };
        let source_path = handoff.source_path.clone();
        self.layout_gesture.shift_active = handoff.shift_active;
        self.layout_gesture.shift_pressed_at = handoff.shift_pressed_at;
        self.layout_gesture.shift_used_as_modifier = handoff.shift_used_as_modifier;
        self.layout_gesture.last_shift_release_at = handoff.last_shift_release_at;
        super::trace::record(format!(
            r#"{{"kind":"ibus_shift_gesture_handoff","stage":"consume","source":"{}","target":"{}","shift_active":{},"used_as_modifier":{}}}"#,
            source_path,
            self.path,
            self.layout_gesture.shift_active,
            self.layout_gesture.shift_used_as_modifier,
        ));
    }
}

pub(crate) fn surrounding_snapshot_match(
    snapshot: Option<&super::engine::SurroundingTextSnapshot>,
    expected_suffix: &str,
) -> SurroundingSnapshotMatch {
    let Some(snapshot) = snapshot else {
        return SurroundingSnapshotMatch::Missing;
    };
    if snapshot.has_selection() {
        return SurroundingSnapshotMatch::Missing;
    }
    if snapshot
        .suffix_before_cursor(expected_suffix.chars().count())
        .as_deref()
        == Some(expected_suffix)
    {
        return SurroundingSnapshotMatch::Exact;
    }

    let without_boundary = expected_suffix.trim_end_matches(char::is_whitespace);
    if without_boundary.len() == expected_suffix.len() || without_boundary.is_empty() {
        return SurroundingSnapshotMatch::Missing;
    }
    if snapshot
        .suffix_before_cursor(without_boundary.chars().count())
        .as_deref()
        == Some(without_boundary)
    {
        SurroundingSnapshotMatch::TrailingBoundaryElided
    } else {
        SurroundingSnapshotMatch::Missing
    }
}

fn pending_ime_auto_undo_snapshot_match(
    snapshot: Option<&super::engine::SurroundingTextSnapshot>,
    pending: &PendingImeAutoUndo,
) -> SurroundingSnapshotMatch {
    if pending.atomic_submission_proven {
        return SurroundingSnapshotMatch::AtomicSubmission;
    }
    let replacement = surrounding_snapshot_match(snapshot, &pending.replacement);
    if replacement != SurroundingSnapshotMatch::Missing {
        return replacement;
    }
    if matches!(
        surrounding_snapshot_match(snapshot, &pending.original),
        SurroundingSnapshotMatch::Exact | SurroundingSnapshotMatch::TrailingBoundaryElided
    ) {
        SurroundingSnapshotMatch::CausalPrecondition
    } else {
        SurroundingSnapshotMatch::Missing
    }
}

fn pending_ime_auto_undo_invalid_reason(
    engine: &LayIbusEngine,
    pending: &PendingImeAutoUndo,
) -> Option<&'static str> {
    if pending.recorded_at.elapsed() > IME_AUTO_UNDO_MAX_AGE {
        return Some("expired");
    }
    if pending.visible_tail != engine.committed_tail.buffer {
        return Some("visible_tail_changed");
    }
    if !pending.visible_tail.ends_with(&pending.replacement) {
        return Some("replacement_not_tail_suffix");
    }
    None
}

fn record_pending_ime_auto_undo_lifecycle(
    engine: &LayIbusEngine,
    state: &SharedState,
    stage: &'static str,
    reason: &'static str,
) {
    let (pending_tail_chars, replacement_chars) = state
        .pending_auto_undo
        .as_ref()
        .map(|pending| {
            (
                pending.visible_tail.chars().count(),
                pending.replacement.chars().count(),
            )
        })
        .unwrap_or_default();
    super::trace::record_auto_undo_lifecycle(
        stage,
        reason,
        &engine.path,
        state.active_path.as_deref() == Some(engine.path.as_str()),
        state.pending_auto_undo.is_some(),
        state.pending_auto_undo_retry.is_some(),
        engine.committed_tail.buffer.chars().count(),
        pending_tail_chars,
        replacement_chars,
        engine
            .client_context
            .surrounding_text_snapshot
            .as_ref()
            .map_or(0, |snapshot| snapshot.text.chars().count()),
    );
}

fn record_detached_ime_auto_undo_lifecycle(
    engine: &LayIbusEngine,
    state: &SharedState,
    pending: &PendingImeAutoUndo,
    stage: &'static str,
    reason: &'static str,
) {
    super::trace::record_auto_undo_lifecycle(
        stage,
        reason,
        &engine.path,
        state.active_path.as_deref() == Some(engine.path.as_str()),
        true,
        state.pending_auto_undo_retry.is_some(),
        engine.committed_tail.buffer.chars().count(),
        pending.visible_tail.chars().count(),
        pending.replacement.chars().count(),
        engine
            .client_context
            .surrounding_text_snapshot
            .as_ref()
            .map_or(0, |snapshot| snapshot.text.chars().count()),
    );
}

pub(crate) fn record_causal_outcome(
    outcome: &str,
    pending: &super::engine::PendingVisiblePostcondition,
    observed_epoch: u64,
) {
    super::trace::record(format!(
        r#"{{"kind":"ibus_causal_outcome","outcome":"{outcome}","source":"{}","snapshot_epoch":{},"observed_epoch":{},"tail_hash":"{:016x}"}}"#,
        pending.snapshot.source.source_id(),
        pending.snapshot.revision,
        observed_epoch,
        pending.snapshot.visible_tail_hash,
    ));
}

fn visible_completion_suffix(suffix: Option<String>) -> String {
    suffix.filter(|suffix| suffix != "*").unwrap_or_default()
}

fn last_tail_token(tail: &str) -> String {
    last_tail_token_range(tail)
        .map(|(start, end)| tail[start..end].to_string())
        .unwrap_or_default()
}

fn last_tail_token_range(tail: &str) -> Option<(usize, usize)> {
    let end = tail
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| (!ch.is_whitespace()).then_some(idx + ch.len_utf8()))?;
    let start = tail[..end]
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| ch.is_whitespace().then_some(idx + ch.len_utf8()))
        .unwrap_or(0);
    Some((start, end))
}

fn trim_committed_tail_buffer(buffer: &mut String) {
    const LIMIT: usize = 160;
    let chars = buffer.chars().count();
    if chars <= LIMIT {
        return;
    }
    let remove = chars - LIMIT;
    if let Some((idx, _)) = buffer.char_indices().nth(remove) {
        buffer.drain(..idx);
    }
}

#[cfg(test)]
mod tests {
    use super::{last_tail_token_range, LayIbusEngine};
    use crate::context_admission::{EngineOwner, EnginePath, OwnerGeneration};
    use crate::engine::{
        ManualToggleAuthority, PendingSystemOutcomeFeedback, SurroundingTextSnapshot,
        SystemOutcomeKind, WordInputMode,
    };
    use crate::protocol::{AutocorrectSuppression, CurrentWordSuppression};
    use lay::config::LayConfig;
    use lay::text_edit::{
        decide_text_transition, LatentTextTransitionCandidate, TextTransitionDecision,
        TextTransitionIntent, TextTransitionRejection, VisibleFieldState, VisibleTailSnapshot,
        VisibleTailSource,
    };
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    #[test]
    fn committed_tail_range_keeps_separator_outside_token() {
        let tail = "file проверка ";
        let (start, end) = last_tail_token_range(tail).expect("last token");
        assert_eq!(&tail[start..end], "проверка");
        assert_eq!(lay::word_reader::trailing_whitespace_char_count(tail), 1);
    }

    #[test]
    fn visible_postcondition_is_consumed_only_for_same_epoch() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            shared.clone(),
            true,
            true,
            LayConfig::default(),
        );
        assert!(engine.bind_focus_path());
        engine.client_context.surrounding_text_supported = true;
        engine.committed_tail.buffer = "проверка ".to_string();
        engine.publish_tail_handoff();
        let epoch = engine.committed_tail.epoch;
        engine.arm_visible_postcondition(Instant::now());
        engine.client_context.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("проверка ".to_string(), 9, 9),
        );

        engine.observe_visible_postcondition();

        assert!(engine
            .committed_tail
            .pending_visible_postcondition
            .is_none());
        assert_eq!(engine.committed_tail.buffer, "проверка ");
        assert_eq!(engine.committed_tail.epoch, epoch);
        let state = shared.lock().expect("lay ime state poisoned");
        assert_eq!(state.handoff_tail_buffer, "проверка ");
        assert_eq!(state.handoff_tail_epoch, epoch);
    }

    #[test]
    fn visible_postcondition_accepts_a_client_elided_trailing_boundary() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            shared,
            true,
            true,
            LayConfig::default(),
        );
        assert!(engine.bind_focus_path());
        engine.client_context.surrounding_text_supported = true;
        engine.committed_tail.buffer = "собака ".to_string();
        engine.publish_tail_handoff();
        engine.arm_visible_postcondition(Instant::now());
        engine.client_context.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("собака".to_string(), 6, 6),
        );

        engine.observe_visible_postcondition();

        assert!(engine
            .committed_tail
            .pending_visible_postcondition
            .is_none());
        assert_eq!(engine.committed_tail.buffer, "собака ");
    }

    #[test]
    fn layout_sync_waits_for_the_committed_text_postcondition() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );
        // The output plan captured SurroundingText authority before dispatch;
        // a transient capability update cannot erase that causal receipt.
        engine.client_context.surrounding_text_supported = false;
        engine.committed_tail.buffer = "собака ".to_string();
        engine.publish_tail_handoff();
        engine.arm_visible_postcondition_from_surrounding_dispatch(
            Instant::now(),
            None,
            Some("собака ".to_string()),
        );

        engine.client_context.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("cj,frf".to_string(), 6, 6),
        );
        engine.observe_visible_postcondition();
        assert!(!engine.layout_gesture.layout_is_ru);
        assert!(engine
            .committed_tail
            .pending_visible_postcondition
            .is_some());

        engine.client_context.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("собака".to_string(), 6, 6),
        );
        engine.observe_visible_postcondition();
        assert!(engine.layout_gesture.layout_is_ru);
        assert!(engine
            .committed_tail
            .pending_visible_postcondition
            .is_none());
    }

    #[test]
    fn exact_postcondition_rejects_the_transient_appended_replacement() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );
        engine.client_context.surrounding_text_supported = true;
        engine.committed_tail.buffer = "привет".to_string();
        engine.publish_tail_handoff();
        engine.arm_exact_visible_postcondition_from_surrounding_dispatch(
            Instant::now(),
            None,
            Some("привет".to_string()),
            super::super::engine::SurroundingTextSnapshot::new("привет".to_string(), 6, 6),
        );

        engine.client_context.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("ghbdtnпривет".to_string(), 12, 12),
        );
        engine.observe_visible_postcondition();

        assert!(!engine.layout_gesture.layout_is_ru);
        assert!(engine
            .committed_tail
            .pending_visible_postcondition
            .is_some());

        engine.client_context.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("привет".to_string(), 6, 6),
        );
        engine.observe_visible_postcondition();

        assert!(engine.layout_gesture.layout_is_ru);
        assert!(engine
            .committed_tail
            .pending_visible_postcondition
            .is_none());
    }

    #[test]
    fn pending_system_feedback_waits_for_visible_postcondition() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig::default(),
        );
        engine.client_context.surrounding_text_supported = true;
        engine.committed_tail.buffer = "давай ".to_string();
        engine.publish_tail_handoff();
        engine.arm_visible_postcondition_from_surrounding_dispatch(
            Instant::now(),
            Some(PendingSystemOutcomeFeedback {
                original: "lfdfq".to_string(),
                replacement: "давай".to_string(),
                source: VisibleTailSource::ImeCommittedTail,
                kind: SystemOutcomeKind::LayoutProjection,
            }),
            None,
        );

        let pending = engine
            .committed_tail
            .pending_visible_postcondition
            .as_ref()
            .expect("feedback must wait for observation");
        assert_eq!(
            pending
                .feedback
                .as_ref()
                .map(|item| item.replacement.as_str()),
            Some("давай")
        );
        assert_eq!(pending.snapshot.source, VisibleTailSource::ImeCommittedTail);
        assert_eq!(pending.snapshot.revision, engine.committed_tail.epoch);
        assert_ne!(pending.snapshot.visible_tail_hash, 0);
    }

    #[test]
    fn visible_postcondition_mismatch_quarantines_tail_and_composition_authority() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            shared.clone(),
            true,
            true,
            LayConfig::default(),
        );
        assert!(engine.bind_focus_path());
        engine.client_context.surrounding_text_supported = true;
        engine.composition.buffer = "stale".to_string();
        engine.composition.cursor = 5;
        engine.committed_tail.buffer = "ghbdtn".to_string();
        engine.rebuild_preedit_fast_from_tail();
        engine.composition.word_input_mode = Some(WordInputMode::ManagedCommit);
        engine.publish_tail_handoff();
        assert!(engine.arm_current_word_autocorrect_suppression());
        let epoch = engine.committed_tail.epoch;
        engine.arm_visible_postcondition(Instant::now() - Duration::from_millis(501));
        engine.client_context.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("ghjdt! ".to_string(), 7, 7),
        );

        engine.observe_visible_postcondition();

        assert!(engine
            .committed_tail
            .pending_visible_postcondition
            .is_none());
        assert!(engine.composition.buffer.is_empty());
        assert_eq!(engine.composition.cursor, 0);
        assert!(engine.committed_tail.buffer.is_empty());
        assert_eq!(engine.composition.preedit_fast.token(), "");
        assert_eq!(engine.composition.word_input_mode, None);
        assert!(engine.committed_tail.autocorrect_suppression.is_none());
        assert_eq!(
            engine.manual_toggle_authority(),
            ManualToggleAuthority::DaemonWordBuffer
        );
        assert_eq!(engine.committed_tail.epoch, epoch.wrapping_add(1));
        let state = shared.lock().expect("lay ime state poisoned");
        assert!(state.handoff_tail_buffer.is_empty());
        assert_eq!(state.handoff_tail_epoch, engine.committed_tail.epoch);
        assert!(state.autocorrect_suppression.is_none());
    }

    #[test]
    fn visible_postcondition_mismatch_blocks_repeated_stale_receipt() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig::default(),
        );
        engine.client_context.surrounding_text_supported = true;
        engine.committed_tail.buffer = "ghbdtn ".to_string();
        engine.publish_tail_handoff();
        let stale_epoch = engine.committed_tail.epoch;
        engine.arm_visible_postcondition(Instant::now() - Duration::from_millis(501));
        engine.client_context.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("ghjdt! ".to_string(), 7, 7),
        );

        engine.observe_visible_postcondition();

        let state = VisibleFieldState::committed_tail(
            engine.committed_tail.buffer.clone(),
            Some(engine.path.clone()),
        )
        .with_epoch(engine.committed_tail.epoch);
        let stale_request = LatentTextTransitionCandidate::new(
            VisibleTailSource::ImeCommittedTail,
            7,
            "привет ",
            TextTransitionIntent::ImeManualToggle,
            Some(VisibleTailSnapshot::new(
                VisibleTailSource::ImeCommittedTail,
                "ghbdtn ",
                Some(engine.path.clone()),
                stale_epoch,
            )),
        );

        assert!(matches!(
            decide_text_transition(&state, stale_request),
            TextTransitionDecision::Reject {
                rejection: TextTransitionRejection::StaleVisibleRevision { expected, actual },
                action: None
            } if expected == stale_epoch && actual == engine.committed_tail.epoch
        ));
    }

    #[test]
    fn early_stale_postcondition_waits_for_committed_surrounding_text() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            shared.clone(),
            true,
            true,
            LayConfig::default(),
        );
        engine.client_context.surrounding_text_supported = true;
        engine.committed_tail.buffer = "вот ".to_string();
        engine.publish_tail_handoff();
        let epoch = engine.committed_tail.epoch;
        engine.arm_visible_postcondition(Instant::now());
        engine.client_context.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("djn".to_string(), 3, 3),
        );

        engine.observe_visible_postcondition();

        assert!(engine
            .committed_tail
            .pending_visible_postcondition
            .is_some());
        assert_eq!(engine.committed_tail.buffer, "вот ");
        assert_eq!(engine.committed_tail.epoch, epoch);
        assert_eq!(
            shared
                .lock()
                .expect("lay ime state poisoned")
                .handoff_tail_buffer,
            "вот "
        );

        engine.client_context.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("вот ".to_string(), 4, 4),
        );
        engine.observe_visible_postcondition();

        assert!(engine
            .committed_tail
            .pending_visible_postcondition
            .is_none());
        assert_eq!(engine.committed_tail.buffer, "вот ");
        assert_eq!(engine.committed_tail.epoch, epoch);
    }

    #[test]
    fn pending_ime_auto_undo_restores_exact_original_surface() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig::default(),
        );
        engine.committed_tail.buffer = "проверка ".to_string();
        engine.remember_pending_ime_auto_undo(
            "проверрка ".to_string(),
            "проверка ".to_string(),
            lay::typing_cpu::ObservedSystemTransition::Correction,
        );

        let pending = engine
            .take_pending_ime_auto_undo()
            .expect("exact autocorrect undo");

        assert_eq!(pending.original, "проверрка ");
        assert_eq!(pending.replacement, "проверка ");
    }

    #[test]
    fn pending_ime_auto_undo_accepts_exact_causal_precondition_snapshot() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );
        assert!(engine.bind_focus_path());
        engine.committed_tail.buffer = "собака ".to_string();
        engine.publish_tail_handoff();
        engine.remember_pending_ime_auto_undo(
            "cj,frf ".to_string(),
            "собака ".to_string(),
            lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
        );
        engine.client_context.surrounding_text_supported = true;
        engine.client_context.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("cj,frf".to_string(), 6, 6),
        );

        assert!(!engine.defer_pending_ime_auto_undo_until_visible());
        assert!(engine.pending_ime_auto_undo_uses_causal_precondition_snapshot());
        assert_eq!(
            engine.pending_ime_auto_undo_retry_status(),
            "ready_causal_precondition"
        );
    }

    #[test]
    fn pending_ime_auto_undo_rejects_unrelated_surrounding_snapshot() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );
        assert!(engine.bind_focus_path());
        engine.committed_tail.buffer = "собака ".to_string();
        engine.publish_tail_handoff();
        engine.remember_pending_ime_auto_undo(
            "cj,frf ".to_string(),
            "собака ".to_string(),
            lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
        );
        engine.client_context.surrounding_text_supported = true;
        engine.client_context.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("другой".to_string(), 6, 6),
        );

        assert!(engine.defer_pending_ime_auto_undo_until_visible());
        assert!(!engine.pending_ime_auto_undo_uses_causal_precondition_snapshot());
        assert_eq!(
            engine.pending_ime_auto_undo_retry_status(),
            "waiting_exact_snapshot"
        );
    }

    #[test]
    fn pending_ime_auto_undo_rejects_a_changed_visible_tail() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig::default(),
        );
        engine.committed_tail.buffer = "проверка дальше".to_string();
        engine.remember_pending_ime_auto_undo(
            "проверрка ".to_string(),
            "проверка ".to_string(),
            lay::typing_cpu::ObservedSystemTransition::Correction,
        );

        assert!(engine.take_pending_ime_auto_undo().is_none());
    }

    #[test]
    fn committed_space_keeps_next_word_separated_in_tail_memory() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig::default(),
        );
        for ch in "печатаетт".chars() {
            engine.insert_composition_char(ch);
        }
        engine.sync_tail_after_composition_commit("печатается ");
        engine.insert_composition_char('т');
        engine.insert_composition_char('ы');
        assert_eq!(engine.committed_tail.buffer, "печатается ты");
        assert_eq!(engine.composition.preedit_fast.token(), "ты");
    }

    #[test]
    fn focus_reset_preserves_just_typed_passthrough_tail() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );

        engine.push_tail_char('g');
        engine.reset_for_ibus_focus_change();

        assert_eq!(engine.committed_tail.buffer, "g");
        assert_eq!(engine.composition.preedit_fast.token(), "g");
    }

    #[test]
    fn focus_reset_clears_stale_passthrough_tail() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );

        engine.push_tail_char('g');
        engine.committed_tail.last_input_at = Some(Instant::now() - Duration::from_millis(900));
        engine.reset_for_ibus_focus_change();

        assert!(engine.committed_tail.buffer.is_empty());
        assert_eq!(engine.composition.preedit_fast.token(), "");
    }

    #[test]
    fn ibus_soft_reset_preserves_tail_for_manual_toggle() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );

        engine.composition.word_input_mode = Some(WordInputMode::ManagedCommit);
        for ch in "ghbdtn".chars() {
            engine.push_tail_char(ch);
        }
        engine.reset_for_ibus_soft_reset();

        assert_eq!(engine.committed_tail.buffer, "ghbdtn");
        assert_eq!(engine.composition.preedit_fast.token(), "ghbdtn");
        assert_eq!(
            engine.composition.word_input_mode,
            Some(WordInputMode::ManagedCommit)
        );
    }

    #[test]
    fn tab_completion_learning_stays_pending_until_the_next_word() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig::default(),
        );

        engine.arm_pending_ime_completion_learning(
            "ну".to_string(),
            "д".to_string(),
            "да".to_string(),
            true,
        );

        let pending = engine
            .committed_tail
            .pending_completion_learning
            .as_ref()
            .expect("Tab completion must remain provisional");
        assert_eq!(pending.context_tail, "ну");
        assert_eq!(pending.typed_prefix, "д");
        assert_eq!(pending.accepted_word, "да");
        assert!(!pending.editing);
    }

    #[test]
    fn backspace_turns_tab_completion_into_an_edit_trajectory() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig::default(),
        );
        engine.committed_tail.buffer = "это было прекрасный ".to_string();
        engine.arm_pending_ime_completion_learning(
            "это было".to_string(),
            "прек".to_string(),
            "прекрасный".to_string(),
            true,
        );

        engine.begin_pending_ime_completion_edit_before_backspace();
        engine.backspace_committed_tail_only();
        engine.begin_pending_ime_completion_edit_before_backspace();

        let pending = engine
            .committed_tail
            .pending_completion_learning
            .as_ref()
            .expect("edited completion must survive every Backspace until a boundary");
        assert!(pending.editing);
        assert_eq!(pending.typed_prefix, "прек");
        assert_eq!(pending.accepted_word, "прекрасный");
        assert_eq!(engine.committed_tail.buffer, "это было прекрасный");

        engine.backspace_committed_tail_only();
        engine.backspace_committed_tail_only();
        engine.push_tail_char('о');
        engine.push_tail_char(' ');

        assert_eq!(engine.committed_tail.buffer, "это было прекрасно ");
        assert!(engine.committed_tail.pending_completion_learning.is_none());
    }

    #[test]
    fn gtk_soft_resets_preserve_only_an_active_completion_edit() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig::default(),
        );
        engine.committed_tail.buffer = "это было прекрасный ".to_string();
        engine.arm_pending_ime_completion_learning(
            "это было".to_string(),
            "прек".to_string(),
            "прекрасный".to_string(),
            true,
        );

        engine.reset_for_ibus_soft_reset();
        assert!(engine.committed_tail.pending_completion_learning.is_none());

        engine.arm_pending_ime_completion_learning(
            "это было".to_string(),
            "прек".to_string(),
            "прекрасный".to_string(),
            true,
        );
        for _ in 0..3 {
            engine.begin_pending_ime_completion_edit_before_backspace();
            engine.backspace_committed_tail_only();
            engine.reset_for_ibus_soft_reset();
            assert!(engine
                .committed_tail
                .pending_completion_learning
                .as_ref()
                .is_some_and(|pending| pending.editing));
        }

        engine.push_tail_char('о');
        engine.push_tail_char(' ');

        assert_eq!(engine.committed_tail.buffer, "это было прекрасно ");
        assert!(engine.committed_tail.pending_completion_learning.is_none());
    }

    #[test]
    fn focus_reset_discards_pending_tab_completion_without_learning() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig::default(),
        );
        engine.arm_pending_ime_completion_learning(
            "ну".to_string(),
            "д".to_string(),
            "да".to_string(),
            true,
        );

        engine.reset_for_ibus_focus_change();

        assert!(engine.committed_tail.pending_completion_learning.is_none());
    }

    #[test]
    fn ibus_soft_reset_preserves_manual_toggle_autocorrect_suppression() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );
        assert!(engine.bind_focus_path());
        engine.push_tail_char('x');
        assert!(engine.arm_current_word_autocorrect_suppression());
        engine.reset_for_ibus_soft_reset();

        assert!(matches!(
            engine.committed_tail.autocorrect_suppression.as_ref(),
            Some(AutocorrectSuppression::CurrentWord(_))
        ));
    }

    #[test]
    fn focus_reset_without_preserve_clears_shared_autocorrect_suppression() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            shared.clone(),
            false,
            true,
            LayConfig::default(),
        );
        assert!(engine.bind_focus_path());
        engine.push_tail_char('x');
        assert!(engine.arm_current_word_autocorrect_suppression());
        engine.committed_tail.last_input_at = None;
        engine.committed_tail.last_commit_at = None;

        engine.reset_for_ibus_focus_change();

        let state = shared.lock().expect("lay ime state poisoned");
        assert!(state.autocorrect_suppression.is_none());
    }

    #[test]
    fn close_committed_tail_field_clears_shared_tail_and_preserve_window() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            shared.clone(),
            false,
            true,
            LayConfig::default(),
        );
        engine.committed_tail.buffer = "file проверка".to_string();
        engine.publish_tail_handoff();
        engine.publish_active_path_preserve_handoff(Instant::now() + Duration::from_millis(100));

        engine.close_committed_tail_field();

        let state = shared.lock().expect("lay ime state poisoned");
        assert!(engine.committed_tail.buffer.is_empty());
        assert!(state.handoff_tail_buffer.is_empty());
        assert!(state.preserve_active_path_until.is_none());
    }

    #[test]
    fn active_path_preserve_handoff_is_shared_between_engine_objects() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let publisher = LayIbusEngine::new(
            "/publisher".to_string(),
            shared.clone(),
            false,
            true,
            LayConfig::default(),
        );
        let reader = LayIbusEngine::new(
            "/reader".to_string(),
            shared,
            false,
            true,
            LayConfig::default(),
        );

        publisher.publish_active_path_preserve_handoff(Instant::now() + Duration::from_millis(100));

        assert!(reader.shared_active_path_preserved());
    }

    #[test]
    fn exact_manual_toggle_handoff_preserves_tail_and_epoch_for_target_engine() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut source = LayIbusEngine::new(
            "/engine/us".to_string(),
            shared.clone(),
            false,
            true,
            LayConfig::default(),
        );
        assert!(source.bind_focus_path());
        source.committed_tail.buffer = "ghbdtn".to_string();
        source.prepare_exact_manual_toggle_layout_handoff();
        let leased_epoch = source.committed_tail.epoch;
        source.reset_for_ibus_soft_reset();
        assert_eq!(source.committed_tail.epoch, leased_epoch);

        let mut target = LayIbusEngine::new(
            "/engine/ru".to_string(),
            shared.clone(),
            true,
            true,
            LayConfig::default(),
        );
        assert!(target.bind_focus_path());
        target.reset_for_ibus_soft_reset();

        assert_eq!(target.committed_tail.buffer, "ghbdtn");
        assert_eq!(target.committed_tail.epoch, leased_epoch);
        assert!(target.committed_tail.autocorrect_suppression.is_none());
        assert!(shared
            .lock()
            .expect("lay ime state poisoned")
            .autocorrect_suppression
            .is_none());

        target.consume_exact_manual_toggle_handoff();
        target.reset_for_ibus_soft_reset();
        assert_eq!(target.committed_tail.epoch, leased_epoch.wrapping_add(1));
    }

    #[test]
    fn exact_manual_toggle_suppression_requires_and_consumes_the_exact_handoff() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut source = LayIbusEngine::new(
            "/engine/us".to_string(),
            shared.clone(),
            false,
            true,
            LayConfig::default(),
        );
        assert!(source.bind_focus_path());
        source.committed_tail.buffer = "ghbdtn".to_string();
        source.prepare_exact_manual_toggle_layout_handoff();
        let leased_epoch = source.committed_tail.epoch;

        let mut target = LayIbusEngine::new(
            "/engine/ru".to_string(),
            shared.clone(),
            true,
            true,
            LayConfig::default(),
        );
        assert!(target.bind_focus_path());
        target.reset_for_ibus_soft_reset();
        target.set_client_capabilities(1 << 5);
        target.client_context.surrounding_text_snapshot =
            Some(SurroundingTextSnapshot::new("ghbdtn".to_string(), 6, 6));

        assert!(!target.arm_exact_manual_toggle_autocorrect_suppression(
            "hbdtn",
            leased_epoch,
            "/engine/ru",
            true,
        ));
        assert!(!target.arm_exact_manual_toggle_autocorrect_suppression(
            "ghbdtn",
            leased_epoch.wrapping_add(1),
            "/engine/ru",
            true,
        ));
        assert!(!target.arm_exact_manual_toggle_autocorrect_suppression(
            "ghbdtn",
            leased_epoch,
            "/engine/us",
            true,
        ));
        assert!(!target.arm_exact_manual_toggle_autocorrect_suppression(
            "ghbdtn",
            leased_epoch,
            "/engine/ru",
            false,
        ));
        assert!(target.arm_exact_manual_toggle_autocorrect_suppression(
            "ghbdtn",
            leased_epoch,
            "/engine/ru",
            true,
        ));

        let state = shared.lock().expect("lay ime state poisoned");
        assert!(matches!(
            target.committed_tail.autocorrect_suppression.as_ref(),
            Some(AutocorrectSuppression::ExactReplay(_))
        ));
        assert!(matches!(
            state.autocorrect_suppression.as_ref(),
            Some(AutocorrectSuppression::ExactReplay(_))
        ));
        assert!(state.preserve_active_path_until.is_none());
        assert!(state.exact_manual_toggle_handoff_epoch.is_none());
        let Some(AutocorrectSuppression::ExactReplay(suppression)) =
            state.autocorrect_suppression.as_ref()
        else {
            panic!("expected exact suppression");
        };
        assert_eq!(suppression.path, "/engine/ru");
        assert_eq!(suppression.epoch, leased_epoch);
        assert!(suppression.expires_at > Instant::now());
        drop(state);

        assert!(!target.revoke_exact_manual_toggle_autocorrect_suppression(
            leased_epoch.wrapping_add(1),
            "/engine/ru",
        ));
        assert!(
            !target.revoke_exact_manual_toggle_autocorrect_suppression(leased_epoch, "/engine/us",)
        );
        assert!(
            target.revoke_exact_manual_toggle_autocorrect_suppression(leased_epoch, "/engine/ru",)
        );
        assert!(target.committed_tail.autocorrect_suppression.is_none());
        let state = shared.lock().expect("lay ime state poisoned");
        assert!(state.autocorrect_suppression.is_none());
    }

    #[test]
    fn focus_engine_can_refresh_empty_tail_from_shared_handoff() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut publisher = LayIbusEngine::new(
            "/publisher".to_string(),
            shared.clone(),
            false,
            true,
            LayConfig::default(),
        );
        publisher.committed_tail.buffer = "вот ".to_string();
        publisher.publish_tail_handoff();
        let published_epoch = publisher.committed_tail.epoch;
        let mut reader = LayIbusEngine::new(
            "/reader".to_string(),
            shared,
            true,
            true,
            LayConfig::default(),
        );
        reader.committed_tail.buffer.clear();

        reader.refresh_empty_tail_from_handoff();

        assert_eq!(reader.committed_tail.buffer, "вот ");
        assert_eq!(reader.committed_tail.epoch, published_epoch);
        assert_eq!(reader.last_tail_token_text(), "вот");
        assert_eq!(reader.composition.preedit_fast.token(), "");
        assert!(!reader.composition.preedit_fast.has_open_token());
        assert_eq!(
            reader.manual_toggle_authority(),
            ManualToggleAuthority::DaemonWordBuffer
        );

        reader.set_client_capabilities(1 << 5);
        reader.push_tail_char('д');
        assert_eq!(reader.committed_tail.buffer, "вот д");
        assert_eq!(reader.composition.preedit_fast.token(), "д");
        assert!(reader.composition.preedit_fast.has_open_token());
        assert_eq!(
            reader.manual_toggle_authority(),
            ManualToggleAuthority::ImeCommittedTail
        );
    }

    #[test]
    fn stale_source_exact_cleanup_cannot_clear_target_current_word_scope() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut source = LayIbusEngine::new_from_component(
            "/stale/source".to_string(),
            shared.clone(),
            None,
            "lay-ime-us",
            true,
            LayConfig::default(),
        );
        source.context_owner = Some(EngineOwner {
            path: EnginePath::new(source.path.clone()).unwrap(),
            generation: OwnerGeneration(1),
        });
        source.committed_tail.buffer = "a".to_string();
        source.committed_tail.epoch = 41;

        let mut target = LayIbusEngine::new_from_component(
            "/current/target".to_string(),
            shared.clone(),
            None,
            "lay-ime-ru",
            true,
            LayConfig::default(),
        );
        target.context_owner = Some(EngineOwner {
            path: EnginePath::new(target.path.clone()).unwrap(),
            generation: OwnerGeneration(2),
        });
        target.committed_tail.buffer = source.committed_tail.buffer.clone();
        target.committed_tail.epoch = source.committed_tail.epoch;
        let target_lease = target.client_context.runtime_owner_lease_identity;
        source.client_context.runtime_owner_lease_identity = target_lease;
        let current_scope = AutocorrectSuppression::CurrentWord(CurrentWordSuppression {
            incarnation: 73,
            owner_lease_identity: target_lease,
            open_token_chars: 1,
        });
        source.committed_tail.autocorrect_suppression = Some(current_scope.clone());
        target.committed_tail.autocorrect_suppression = Some(current_scope.clone());
        {
            let mut state = shared.lock().unwrap();
            state.active_path = Some(target.path.clone());
            state.context_owner_generation = Some(2);
            state.handoff_tail_buffer = "a".to_string();
            state.handoff_tail_epoch = 41;
            state.preserve_active_path_until = Some(Instant::now() + Duration::from_secs(1));
            state.exact_manual_toggle_handoff_epoch = Some(41);
            state.exact_manual_toggle_handoff_path = Some(source.path.clone());
            state.autocorrect_suppression = Some(current_scope.clone());
        }

        source.clear_identity_bound_exact_manual_toggle_authority();

        assert_eq!(
            source.committed_tail.autocorrect_suppression,
            Some(current_scope.clone())
        );
        assert_eq!(
            target.committed_tail.autocorrect_suppression,
            Some(current_scope.clone())
        );
        {
            let state = shared.lock().unwrap();
            assert_eq!(state.autocorrect_suppression, Some(current_scope.clone()));
            assert_eq!(state.exact_manual_toggle_handoff_epoch, Some(41));
            assert_eq!(
                state.exact_manual_toggle_handoff_path.as_deref(),
                Some(source.path.as_str())
            );
        }

        target.clear_identity_bound_exact_manual_toggle_authority();

        assert_eq!(
            target.committed_tail.autocorrect_suppression,
            Some(current_scope.clone())
        );
        let state = shared.lock().unwrap();
        assert_eq!(state.autocorrect_suppression, Some(current_scope));
        assert!(state.exact_manual_toggle_handoff_epoch.is_none());
        assert!(state.exact_manual_toggle_handoff_path.is_none());
    }

    #[test]
    fn empty_focus_reset_does_not_overwrite_preserved_shared_tail() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut publisher = LayIbusEngine::new(
            "/publisher".to_string(),
            shared.clone(),
            false,
            true,
            LayConfig::default(),
        );
        publisher.committed_tail.buffer = "вот ".to_string();
        publisher.publish_tail_handoff();
        publisher.publish_active_path_preserve_handoff(Instant::now() + Duration::from_millis(100));
        let mut empty_engine = LayIbusEngine::new(
            "/empty".to_string(),
            shared.clone(),
            true,
            true,
            LayConfig::default(),
        );
        empty_engine.committed_tail.buffer.clear();

        empty_engine.reset_for_ibus_focus_change();

        let state = shared.lock().expect("lay ime state poisoned");
        assert_eq!(state.handoff_tail_buffer, "вот ");
    }

    #[test]
    fn whitespace_closes_current_word_input_mode() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );

        engine.composition.word_input_mode = Some(WordInputMode::ManagedCommit);
        engine.push_tail_char('a');
        engine.push_tail_char(' ');

        assert_eq!(engine.composition.word_input_mode, None);
    }

    #[test]
    fn fresh_focus_reset_preserves_current_word_input_mode() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );

        engine.composition.word_input_mode = Some(WordInputMode::ManagedCommit);
        engine.push_tail_char('f');
        engine.reset_for_ibus_focus_change();

        assert_eq!(engine.committed_tail.buffer, "f");
        assert_eq!(
            engine.composition.word_input_mode,
            Some(WordInputMode::ManagedCommit)
        );
    }
}
