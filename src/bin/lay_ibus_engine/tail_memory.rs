use super::engine::{
    LayIbusEngine, PendingImeCompletionLearning, PendingSystemOutcomeFeedback, SystemOutcomeKind,
};
use super::protocol::{
    ExactManualToggleSuppression, PendingImeAutoUndo, PendingImeAutoUndoRetry, SharedState,
    ShiftGestureHandoff,
};
use lay::text_edit::{VisibleTailSnapshot, VisibleTailSource};
use std::time::{Duration, Instant};

const IME_AUTO_UNDO_MAX_AGE: Duration = Duration::from_secs(30);
const IME_AUTO_UNDO_RETRY_MAX_AGE: Duration = Duration::from_secs(5);
const IME_LAYOUT_HANDOFF_MAX_AGE: Duration = Duration::from_millis(700);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SurroundingSnapshotMatch {
    Exact,
    AtomicSubmission,
    TrailingBoundaryElided,
    CausalPrecondition,
    Missing,
}

impl LayIbusEngine {
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
            visible_tail: self.tail_buffer.clone(),
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
        if !self.surrounding_text_supported {
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
        let snapshot_match =
            pending_ime_auto_undo_snapshot_match(self.surrounding_text_snapshot.as_ref(), pending);
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
        match pending_ime_auto_undo_snapshot_match(self.surrounding_text_snapshot.as_ref(), pending)
        {
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
        pending_ime_auto_undo_snapshot_match(self.surrounding_text_snapshot.as_ref(), pending)
    }

    pub(super) fn arm_pending_ime_completion_learning(
        &mut self,
        context_tail: String,
        typed_prefix: String,
        accepted_word: String,
        with_space: bool,
    ) {
        self.pending_ime_completion_learning = with_space.then_some(PendingImeCompletionLearning {
            context_tail,
            typed_prefix,
            accepted_word,
            editing: false,
        });
    }

    pub(super) fn begin_pending_ime_completion_edit_before_backspace(&mut self) {
        let Some(pending) = self.pending_ime_completion_learning.as_mut() else {
            return;
        };
        if pending.editing {
            return;
        }
        let accepted_tail = format!("{} ", pending.accepted_word);
        if self.tail_buffer.ends_with(&accepted_tail) {
            pending.editing = true;
        } else {
            self.pending_ime_completion_learning = None;
        }
    }

    /// A later word or terminal punctuation confirms that the explicitly
    /// accepted completion remained useful. This does not alter visible text.
    pub(super) fn confirm_pending_ime_completion_at_stable_boundary(&mut self) {
        if self
            .pending_ime_completion_learning
            .as_ref()
            .is_some_and(|pending| pending.editing)
        {
            return;
        }
        let Some(pending) = self.pending_ime_completion_learning.take() else {
            return;
        };
        let accepted_tail = format!("{} ", pending.accepted_word);
        if self.tail_buffer.ends_with(&accepted_tail) {
            lay::typing_cpu::TypingCpu::record_accepted_completion(
                &pending.context_tail,
                &pending.accepted_word,
            );
        }
    }

    pub(super) fn finalize_pending_ime_completion_edit(
        &mut self,
        tail_before_boundary: &str,
    ) -> bool {
        let is_editing = self
            .pending_ime_completion_learning
            .as_ref()
            .is_some_and(|pending| pending.editing);
        if !is_editing {
            return false;
        }
        let pending = self
            .pending_ime_completion_learning
            .take()
            .expect("editing completion must remain pending");
        let final_word = lay::nanda_wave::llmwave::tokenize(tail_before_boundary)
            .into_iter()
            .next_back();
        match final_word {
            Some(final_word) if final_word == pending.accepted_word => {
                lay::typing_cpu::TypingCpu::record_accepted_completion(
                    &pending.context_tail,
                    &pending.accepted_word,
                );
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

    pub(super) fn arm_visible_postcondition(&mut self, dispatched_at: Instant) {
        self.arm_visible_postcondition_with_effects(dispatched_at, None, None);
    }

    pub(super) fn arm_visible_postcondition_with_feedback(
        &mut self,
        dispatched_at: Instant,
        feedback: Option<PendingSystemOutcomeFeedback>,
    ) {
        self.arm_visible_postcondition_with_effects(dispatched_at, feedback, None);
    }

    pub(super) fn arm_visible_postcondition_with_effects(
        &mut self,
        dispatched_at: Instant,
        feedback: Option<PendingSystemOutcomeFeedback>,
        layout_sync_text: Option<String>,
    ) {
        if !self.surrounding_text_supported {
            return;
        }
        self.arm_visible_postcondition_from_surrounding_dispatch(
            dispatched_at,
            feedback,
            layout_sync_text,
        );
    }

    pub(super) fn arm_visible_postcondition_from_surrounding_dispatch(
        &mut self,
        dispatched_at: Instant,
        feedback: Option<PendingSystemOutcomeFeedback>,
        layout_sync_text: Option<String>,
    ) {
        self.arm_visible_postcondition_from_surrounding_dispatch_with_snapshot(
            dispatched_at,
            feedback,
            layout_sync_text,
            None,
        );
    }

    pub(super) fn arm_exact_visible_postcondition_from_surrounding_dispatch(
        &mut self,
        dispatched_at: Instant,
        feedback: Option<PendingSystemOutcomeFeedback>,
        layout_sync_text: Option<String>,
        expected_external_snapshot: super::engine::SurroundingTextSnapshot,
    ) {
        self.arm_visible_postcondition_from_surrounding_dispatch_with_snapshot(
            dispatched_at,
            feedback,
            layout_sync_text,
            Some(expected_external_snapshot),
        );
    }

    fn arm_visible_postcondition_from_surrounding_dispatch_with_snapshot(
        &mut self,
        dispatched_at: Instant,
        feedback: Option<PendingSystemOutcomeFeedback>,
        layout_sync_text: Option<String>,
        expected_external_snapshot: Option<super::engine::SurroundingTextSnapshot>,
    ) {
        let snapshot = VisibleTailSnapshot::new(
            VisibleTailSource::ImeCommittedTail,
            self.tail_buffer.clone(),
            Some(self.path.clone()),
            self.tail_epoch,
        )
        .identity();
        self.pending_visible_postcondition = Some(super::engine::PendingVisiblePostcondition {
            expected_suffix: self.tail_buffer.clone(),
            expected_external_snapshot,
            snapshot,
            dispatched_epoch: self.tail_epoch,
            dispatched_at,
            feedback,
            layout_sync_text,
        });
    }

    pub(super) fn observe_visible_postcondition(&mut self) {
        const OBSERVATION_TIMEOUT_MS: u128 = 1500;
        const SETTLE_GRACE_MS: u128 = 500;
        let Some(pending) = self.pending_visible_postcondition.take() else {
            return;
        };
        let elapsed_ms = pending.dispatched_at.elapsed().as_millis();
        if elapsed_ms > OBSERVATION_TIMEOUT_MS || pending.dispatched_epoch != self.tail_epoch {
            record_causal_outcome("censored", &pending, self.tail_epoch);
            return;
        }
        let observed = match pending.expected_external_snapshot.as_ref() {
            Some(expected) if self.surrounding_text_snapshot.as_ref() == Some(expected) => {
                SurroundingSnapshotMatch::Exact
            }
            Some(_) => SurroundingSnapshotMatch::Missing,
            None => surrounding_snapshot_match(
                self.surrounding_text_snapshot.as_ref(),
                &pending.expected_suffix,
            ),
        };
        let status = if matches!(
            observed,
            SurroundingSnapshotMatch::Exact | SurroundingSnapshotMatch::TrailingBoundaryElided
        ) {
            self.record_observed_system_outcome(pending.feedback.as_ref());
            record_causal_outcome("confirmed_positive", &pending, self.tail_epoch);
            if let Some(text) = pending.layout_sync_text.as_deref() {
                self.sync_layout_after_committed_text(text, "visible_postcondition_confirmed");
            }
            if observed == SurroundingSnapshotMatch::TrailingBoundaryElided {
                "observed_boundary_elided"
            } else {
                "observed"
            }
        } else if elapsed_ms <= SETTLE_GRACE_MS {
            record_causal_outcome("pending_stale_observation", &pending, self.tail_epoch);
            self.pending_visible_postcondition = Some(pending);
            "pending"
        } else {
            // The compositor may report the pre-commit surrounding text once
            // before publishing the committed value. Only quarantine after the
            // bounded settle window has elapsed.
            self.quarantine_visible_postcondition_mismatch();
            record_causal_outcome("censored", &pending, self.tail_epoch);
            "mismatch"
        };
        super::trace::record(format!(
            r#"{{"kind":"ibus_visible_postcondition","status":"{status}"}}"#
        ));
    }

    fn record_observed_system_outcome(&self, feedback: Option<&PendingSystemOutcomeFeedback>) {
        let Some(feedback) = feedback else {
            return;
        };
        match feedback.kind {
            SystemOutcomeKind::LayoutProjection => {
                lay::typing_cpu::TypingCpu::record_observed_system_apply(
                    &feedback.original,
                    &feedback.replacement,
                    lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
                );
            }
            SystemOutcomeKind::Correction => {
                lay::typing_cpu::TypingCpu::record_observed_system_apply(
                    &feedback.original,
                    &feedback.replacement,
                    lay::typing_cpu::ObservedSystemTransition::Correction,
                );
            }
        }
    }

    pub(super) fn selected_visible_completion_suffix(&self) -> String {
        if self.selected_precognition_replacement().is_some() {
            return String::new();
        }
        visible_completion_suffix(self.selected_precognition_suffix())
    }

    pub(super) fn last_tail_token_text(&self) -> String {
        last_tail_token(&self.tail_buffer)
    }

    pub(super) fn sync_tail_after_composition_commit(&mut self, text: &str) {
        self.surrounding_text_snapshot = None;
        let trailing_ws = lay::word_reader::trailing_whitespace_char_count(text);
        let committed = text.trim_end_matches(char::is_whitespace);
        if !committed.is_empty() {
            self.replace_last_tail_token_text(committed, self.buffer.chars().count());
        }
        for _ in 0..trailing_ws {
            self.tail_buffer.push(' ');
        }
        if trailing_ws > 0 {
            self.preedit_fast.reset();
        } else {
            self.rebuild_preedit_fast_from_tail();
        }
        trim_committed_tail_buffer(&mut self.tail_buffer);
        self.publish_tail_handoff();
    }

    pub(super) fn replace_last_tail_token_text(&mut self, replacement: &str, fallback_len: usize) {
        let Some((start, end)) = last_tail_token_range(&self.tail_buffer) else {
            self.tail_buffer.push_str(replacement);
            return;
        };
        let range_len = self.tail_buffer[start..end].chars().count();
        if fallback_len > 0 && range_len != fallback_len {
            self.tail_buffer.push_str(replacement);
            return;
        }
        self.tail_buffer.replace_range(start..end, replacement);
    }

    pub(super) fn rebuild_preedit_fast_from_tail(&mut self) {
        self.preedit_fast.reset();
        for ch in self.last_tail_token_text().chars() {
            self.preedit_fast.push(ch);
        }
    }

    pub(super) fn publish_tail_handoff(&mut self) {
        self.tail_epoch = self.tail_epoch.wrapping_add(1);
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.handoff_tail_buffer = self.tail_buffer.clone();
        state.handoff_tail_epoch = self.tail_epoch;
        state.handoff_focus_receipt = self.focus_receipt.clone();
        state.exact_manual_toggle_handoff_epoch = None;
        state.exact_manual_toggle_handoff_path = None;
    }

    pub(super) fn prepare_exact_manual_toggle_layout_handoff(&mut self) {
        self.publish_tail_handoff();
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.preserve_active_path_until = Some(Instant::now() + IME_LAYOUT_HANDOFF_MAX_AGE);
        state.exact_manual_toggle_handoff_epoch = Some(self.tail_epoch);
        state.exact_manual_toggle_handoff_path = Some(self.path.clone());
    }

    pub(super) fn exact_manual_toggle_handoff_is_live(&self) -> bool {
        let Ok(mut state) = self.shared.lock() else {
            return false;
        };
        let now = Instant::now();
        let live = state
            .preserve_active_path_until
            .is_some_and(|until| now <= until)
            && state.exact_manual_toggle_handoff_epoch == Some(self.tail_epoch)
            && state.handoff_tail_epoch == self.tail_epoch
            && state.handoff_tail_buffer == self.tail_buffer;
        if !live {
            state.exact_manual_toggle_handoff_epoch = None;
            state.exact_manual_toggle_handoff_path = None;
        }
        live
    }

    pub(super) fn consume_exact_manual_toggle_handoff(&self) {
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.preserve_active_path_until = None;
        state.exact_manual_toggle_handoff_epoch = None;
        state.exact_manual_toggle_handoff_path = None;
    }

    pub(super) fn arm_exact_manual_toggle_autocorrect_suppression(
        &mut self,
        expected_suffix: &str,
        expected_epoch: u64,
        expected_path: &str,
        expected_layout_is_ru: bool,
    ) -> bool {
        let exact_tail_suffix =
            last_tail_token_range(&self.tail_buffer).map(|(start, _)| &self.tail_buffer[start..]);
        if expected_suffix.is_empty()
            || exact_tail_suffix != Some(expected_suffix)
            || self.path != expected_path
            || self.layout_is_ru != expected_layout_is_ru
            || self.tail_epoch != expected_epoch
            || !self.tail_buffer.ends_with(expected_suffix)
        {
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
            && state.handoff_tail_buffer == self.tail_buffer;
        if !live {
            return false;
        }

        state.preserve_active_path_until = None;
        state.exact_manual_toggle_handoff_epoch = None;
        state.exact_manual_toggle_handoff_path = None;
        state.suppress_next_committed_tail_autocorrect = true;
        state.exact_manual_toggle_suppression = Some(ExactManualToggleSuppression {
            path: expected_path.to_string(),
            epoch: expected_epoch,
            expires_at: Instant::now() + IME_LAYOUT_HANDOFF_MAX_AGE,
        });
        self.suppress_next_committed_tail_autocorrect = true;
        self.exact_manual_toggle_suppression = state.exact_manual_toggle_suppression.clone();
        true
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
        let matches = state
            .exact_manual_toggle_suppression
            .as_ref()
            .is_some_and(|identity| {
                identity.path == expected_path && identity.epoch == expected_epoch
            });
        if !matches {
            return false;
        }
        state.suppress_next_committed_tail_autocorrect = false;
        state.exact_manual_toggle_suppression = None;
        self.suppress_next_committed_tail_autocorrect = false;
        self.exact_manual_toggle_suppression = None;
        true
    }

    pub(super) fn close_committed_tail_field(&mut self) {
        self.pending_ime_completion_learning = None;
        self.tail_buffer.clear();
        self.preedit_fast.reset();
        self.suppress_next_committed_tail_autocorrect = false;
        self.exact_manual_toggle_suppression = None;
        self.word_input_mode = None;
        self.last_tail_input_at = None;
        self.last_commit_at = None;
        self.recent_committed_tail_replace = None;
        self.pending_manual_toggle = false;
        self.pending_visible_postcondition = None;
        self.tail_epoch = self.tail_epoch.wrapping_add(1);
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        record_pending_ime_auto_undo_lifecycle(self, &state, "clear", "close_committed_tail_field");
        state.handoff_tail_buffer.clear();
        state.handoff_tail_epoch = self.tail_epoch;
        state.handoff_focus_receipt = None;
        state.suppress_next_committed_tail_autocorrect = false;
        state.exact_manual_toggle_suppression = None;
        state.preserve_active_path_until = None;
        state.exact_manual_toggle_handoff_epoch = None;
        state.exact_manual_toggle_handoff_path = None;
        state.pending_auto_undo = None;
        state.pending_auto_undo_retry = None;
        state.shift_gesture_handoff = None;
    }

    fn quarantine_visible_postcondition_mismatch(&mut self) {
        let shared = self.shared.clone();
        self.buffer.clear();
        self.composition_cursor = 0;
        self.tail_buffer.clear();
        self.preedit_fast.reset();
        self.clear_preedit_completion_state();
        self.word_input_mode = None;
        self.last_tail_input_at = None;
        self.recent_committed_tail_replace = None;
        self.pending_manual_toggle = false;
        self.suppress_next_committed_tail_autocorrect = false;
        self.exact_manual_toggle_suppression = None;
        self.tail_epoch = self.tail_epoch.wrapping_add(1);
        if let Ok(mut state) = shared.lock() {
            record_pending_ime_auto_undo_lifecycle(
                self,
                &state,
                "clear",
                "visible_postcondition_mismatch",
            );
            state.handoff_tail_buffer.clear();
            state.handoff_tail_epoch = self.tail_epoch;
            state.handoff_focus_receipt = None;
            state.suppress_next_committed_tail_autocorrect = false;
            state.exact_manual_toggle_suppression = None;
            state.preserve_active_path_until = None;
            state.exact_manual_toggle_handoff_epoch = None;
            state.exact_manual_toggle_handoff_path = None;
            state.pending_auto_undo = None;
            state.pending_auto_undo_retry = None;
            state.shift_gesture_handoff = None;
        };
    }

    pub(super) fn refresh_empty_tail_from_handoff(&mut self) {
        if !self.tail_buffer.is_empty() {
            return;
        }
        let Ok(state) = self.shared.lock() else {
            return;
        };
        if state.handoff_tail_buffer.is_empty() {
            return;
        }
        self.tail_buffer.clone_from(&state.handoff_tail_buffer);
        self.tail_epoch = state.handoff_tail_epoch;
        drop(state);
        self.rebuild_preedit_fast_from_tail();
    }

    pub(super) fn publish_autocorrect_suppression_handoff(&self) {
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.suppress_next_committed_tail_autocorrect = true;
        state.exact_manual_toggle_suppression = None;
    }

    pub(super) fn take_autocorrect_suppression_handoff(&self) -> bool {
        let Ok(mut state) = self.shared.lock() else {
            return false;
        };
        let now = Instant::now();
        let exact_path_matches = state
            .exact_manual_toggle_suppression
            .as_ref()
            .is_none_or(|identity| identity.path == self.path && now <= identity.expires_at);
        let suppress = state.suppress_next_committed_tail_autocorrect && exact_path_matches;
        state.suppress_next_committed_tail_autocorrect = false;
        state.exact_manual_toggle_suppression = None;
        suppress
    }

    pub(super) fn clear_autocorrect_suppression_handoff(&self) {
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.suppress_next_committed_tail_autocorrect = false;
        state.exact_manual_toggle_suppression = None;
    }

    pub(super) fn publish_active_path_preserve_handoff(&self, until: Instant) {
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.preserve_active_path_until = Some(until);
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
            shift_active: self.shift_active,
            shift_pressed_at: self.shift_pressed_at,
            shift_used_as_modifier: self.shift_used_as_modifier,
            last_shift_release_at: self.last_shift_release_at,
        });
        super::trace::record(format!(
            r#"{{"kind":"ibus_shift_gesture_handoff","stage":"publish","source":"{}","shift_active":{},"used_as_modifier":{}}}"#,
            self.path, self.shift_active, self.shift_used_as_modifier,
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
        self.shift_active = handoff.shift_active;
        self.shift_pressed_at = handoff.shift_pressed_at;
        self.shift_used_as_modifier = handoff.shift_used_as_modifier;
        self.last_shift_release_at = handoff.last_shift_release_at;
        super::trace::record(format!(
            r#"{{"kind":"ibus_shift_gesture_handoff","stage":"consume","source":"{}","target":"{}","shift_active":{},"used_as_modifier":{}}}"#,
            source_path, self.path, self.shift_active, self.shift_used_as_modifier,
        ));
    }
}

fn surrounding_snapshot_match(
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
    if pending.visible_tail != engine.tail_buffer {
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
        engine.tail_buffer.chars().count(),
        pending_tail_chars,
        replacement_chars,
        engine
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
        engine.tail_buffer.chars().count(),
        pending.visible_tail.chars().count(),
        pending.replacement.chars().count(),
        engine
            .surrounding_text_snapshot
            .as_ref()
            .map_or(0, |snapshot| snapshot.text.chars().count()),
    );
}

fn record_causal_outcome(
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
    use crate::engine::{
        ManualToggleAuthority, PendingSystemOutcomeFeedback, SystemOutcomeKind, WordInputMode,
    };
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
        engine.surrounding_text_supported = true;
        engine.tail_buffer = "проверка ".to_string();
        engine.publish_tail_handoff();
        let epoch = engine.tail_epoch;
        engine.arm_visible_postcondition(Instant::now());
        engine.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("проверка ".to_string(), 9, 9),
        );

        engine.observe_visible_postcondition();

        assert!(engine.pending_visible_postcondition.is_none());
        assert_eq!(engine.tail_buffer, "проверка ");
        assert_eq!(engine.tail_epoch, epoch);
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
        engine.surrounding_text_supported = true;
        engine.tail_buffer = "собака ".to_string();
        engine.publish_tail_handoff();
        engine.arm_visible_postcondition(Instant::now());
        engine.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("собака".to_string(), 6, 6),
        );

        engine.observe_visible_postcondition();

        assert!(engine.pending_visible_postcondition.is_none());
        assert_eq!(engine.tail_buffer, "собака ");
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
        engine.surrounding_text_supported = false;
        engine.tail_buffer = "собака ".to_string();
        engine.publish_tail_handoff();
        engine.arm_visible_postcondition_from_surrounding_dispatch(
            Instant::now(),
            None,
            Some("собака ".to_string()),
        );

        engine.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("cj,frf".to_string(), 6, 6),
        );
        engine.observe_visible_postcondition();
        assert!(!engine.layout_is_ru);
        assert!(engine.pending_visible_postcondition.is_some());

        engine.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("собака".to_string(), 6, 6),
        );
        engine.observe_visible_postcondition();
        assert!(engine.layout_is_ru);
        assert!(engine.pending_visible_postcondition.is_none());
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
        engine.surrounding_text_supported = true;
        engine.tail_buffer = "привет".to_string();
        engine.publish_tail_handoff();
        engine.arm_exact_visible_postcondition_from_surrounding_dispatch(
            Instant::now(),
            None,
            Some("привет".to_string()),
            super::super::engine::SurroundingTextSnapshot::new("привет".to_string(), 6, 6),
        );

        engine.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("ghbdtnпривет".to_string(), 12, 12),
        );
        engine.observe_visible_postcondition();

        assert!(!engine.layout_is_ru);
        assert!(engine.pending_visible_postcondition.is_some());

        engine.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("привет".to_string(), 6, 6),
        );
        engine.observe_visible_postcondition();

        assert!(engine.layout_is_ru);
        assert!(engine.pending_visible_postcondition.is_none());
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
        engine.surrounding_text_supported = true;
        engine.tail_buffer = "давай ".to_string();
        engine.publish_tail_handoff();
        engine.arm_visible_postcondition_with_feedback(
            Instant::now(),
            Some(PendingSystemOutcomeFeedback {
                original: "lfdfq".to_string(),
                replacement: "давай".to_string(),
                source: VisibleTailSource::ImeCommittedTail,
                kind: SystemOutcomeKind::LayoutProjection,
            }),
        );

        let pending = engine
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
        assert_eq!(pending.snapshot.revision, engine.tail_epoch);
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
        engine.surrounding_text_supported = true;
        engine.buffer = "stale".to_string();
        engine.composition_cursor = 5;
        engine.tail_buffer = "ghbdtn ".to_string();
        engine.rebuild_preedit_fast_from_tail();
        engine.word_input_mode = Some(WordInputMode::ManagedCommit);
        engine.suppress_next_committed_tail_autocorrect = true;
        engine.publish_tail_handoff();
        let epoch = engine.tail_epoch;
        engine.arm_visible_postcondition(Instant::now() - Duration::from_millis(501));
        engine.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("ghjdt! ".to_string(), 7, 7),
        );

        engine.observe_visible_postcondition();

        assert!(engine.pending_visible_postcondition.is_none());
        assert!(engine.buffer.is_empty());
        assert_eq!(engine.composition_cursor, 0);
        assert!(engine.tail_buffer.is_empty());
        assert_eq!(engine.preedit_fast.token(), "");
        assert_eq!(engine.word_input_mode, None);
        assert!(!engine.suppress_next_committed_tail_autocorrect);
        assert_eq!(
            engine.manual_toggle_authority(),
            ManualToggleAuthority::DaemonWordBuffer
        );
        assert_eq!(engine.tail_epoch, epoch.wrapping_add(1));
        let state = shared.lock().expect("lay ime state poisoned");
        assert!(state.handoff_tail_buffer.is_empty());
        assert_eq!(state.handoff_tail_epoch, engine.tail_epoch);
        assert!(!state.suppress_next_committed_tail_autocorrect);
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
        engine.surrounding_text_supported = true;
        engine.tail_buffer = "ghbdtn ".to_string();
        engine.publish_tail_handoff();
        let stale_epoch = engine.tail_epoch;
        engine.arm_visible_postcondition(Instant::now() - Duration::from_millis(501));
        engine.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("ghjdt! ".to_string(), 7, 7),
        );

        engine.observe_visible_postcondition();

        let state = VisibleFieldState::committed_tail(
            engine.tail_buffer.clone(),
            Some(engine.path.clone()),
        )
        .with_epoch(engine.tail_epoch);
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
            } if expected == stale_epoch && actual == engine.tail_epoch
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
        engine.surrounding_text_supported = true;
        engine.tail_buffer = "вот ".to_string();
        engine.publish_tail_handoff();
        let epoch = engine.tail_epoch;
        engine.arm_visible_postcondition(Instant::now());
        engine.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("djn".to_string(), 3, 3),
        );

        engine.observe_visible_postcondition();

        assert!(engine.pending_visible_postcondition.is_some());
        assert_eq!(engine.tail_buffer, "вот ");
        assert_eq!(engine.tail_epoch, epoch);
        assert_eq!(
            shared
                .lock()
                .expect("lay ime state poisoned")
                .handoff_tail_buffer,
            "вот "
        );

        engine.surrounding_text_snapshot = Some(
            super::super::engine::SurroundingTextSnapshot::new("вот ".to_string(), 4, 4),
        );
        engine.observe_visible_postcondition();

        assert!(engine.pending_visible_postcondition.is_none());
        assert_eq!(engine.tail_buffer, "вот ");
        assert_eq!(engine.tail_epoch, epoch);
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
        engine.tail_buffer = "проверка ".to_string();
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
        engine.tail_buffer = "собака ".to_string();
        engine.publish_tail_handoff();
        engine.remember_pending_ime_auto_undo(
            "cj,frf ".to_string(),
            "собака ".to_string(),
            lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
        );
        engine.surrounding_text_supported = true;
        engine.surrounding_text_snapshot = Some(
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
        engine.tail_buffer = "собака ".to_string();
        engine.publish_tail_handoff();
        engine.remember_pending_ime_auto_undo(
            "cj,frf ".to_string(),
            "собака ".to_string(),
            lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
        );
        engine.surrounding_text_supported = true;
        engine.surrounding_text_snapshot = Some(
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
        engine.tail_buffer = "проверка дальше".to_string();
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
        assert_eq!(engine.tail_buffer, "печатается ты");
        assert_eq!(engine.preedit_fast.token(), "ты");
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

        assert_eq!(engine.tail_buffer, "g");
        assert_eq!(engine.preedit_fast.token(), "g");
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
        engine.last_tail_input_at = Some(Instant::now() - Duration::from_millis(900));
        engine.reset_for_ibus_focus_change();

        assert!(engine.tail_buffer.is_empty());
        assert_eq!(engine.preedit_fast.token(), "");
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

        engine.word_input_mode = Some(WordInputMode::ManagedCommit);
        for ch in "ghbdtn".chars() {
            engine.push_tail_char(ch);
        }
        engine.reset_for_ibus_soft_reset();

        assert_eq!(engine.tail_buffer, "ghbdtn");
        assert_eq!(engine.preedit_fast.token(), "ghbdtn");
        assert_eq!(engine.word_input_mode, Some(WordInputMode::ManagedCommit));
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
            .pending_ime_completion_learning
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
        engine.tail_buffer = "это было прекрасный ".to_string();
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
            .pending_ime_completion_learning
            .as_ref()
            .expect("edited completion must survive every Backspace until a boundary");
        assert!(pending.editing);
        assert_eq!(pending.typed_prefix, "прек");
        assert_eq!(pending.accepted_word, "прекрасный");
        assert_eq!(engine.tail_buffer, "это было прекрасный");

        engine.backspace_committed_tail_only();
        engine.backspace_committed_tail_only();
        engine.push_tail_char('о');
        engine.push_tail_char(' ');

        assert_eq!(engine.tail_buffer, "это было прекрасно ");
        assert!(engine.pending_ime_completion_learning.is_none());
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
        engine.tail_buffer = "это было прекрасный ".to_string();
        engine.arm_pending_ime_completion_learning(
            "это было".to_string(),
            "прек".to_string(),
            "прекрасный".to_string(),
            true,
        );

        engine.reset_for_ibus_soft_reset();
        assert!(engine.pending_ime_completion_learning.is_none());

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
                .pending_ime_completion_learning
                .as_ref()
                .is_some_and(|pending| pending.editing));
        }

        engine.push_tail_char('о');
        engine.push_tail_char(' ');

        assert_eq!(engine.tail_buffer, "это было прекрасно ");
        assert!(engine.pending_ime_completion_learning.is_none());
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

        assert!(engine.pending_ime_completion_learning.is_none());
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

        engine.suppress_next_committed_tail_autocorrect = true;
        engine.reset_for_ibus_soft_reset();

        assert!(engine.suppress_next_committed_tail_autocorrect);
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
        engine.publish_autocorrect_suppression_handoff();

        engine.reset_for_ibus_focus_change();

        let state = shared.lock().expect("lay ime state poisoned");
        assert!(!state.suppress_next_committed_tail_autocorrect);
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
        engine.tail_buffer = "file проверка".to_string();
        engine.publish_tail_handoff();
        engine.publish_active_path_preserve_handoff(Instant::now() + Duration::from_millis(100));

        engine.close_committed_tail_field();

        let state = shared.lock().expect("lay ime state poisoned");
        assert!(engine.tail_buffer.is_empty());
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
        source.tail_buffer = "ghbdtn".to_string();
        source.prepare_exact_manual_toggle_layout_handoff();
        let leased_epoch = source.tail_epoch;
        source.reset_for_ibus_soft_reset();
        assert_eq!(source.tail_epoch, leased_epoch);

        let mut target = LayIbusEngine::new(
            "/engine/ru".to_string(),
            shared.clone(),
            true,
            true,
            LayConfig::default(),
        );
        assert!(target.bind_focus_path());
        target.reset_for_ibus_soft_reset();

        assert_eq!(target.tail_buffer, "ghbdtn");
        assert_eq!(target.tail_epoch, leased_epoch);
        assert!(!target.suppress_next_committed_tail_autocorrect);
        assert!(
            !shared
                .lock()
                .expect("lay ime state poisoned")
                .suppress_next_committed_tail_autocorrect
        );

        target.consume_exact_manual_toggle_handoff();
        target.reset_for_ibus_soft_reset();
        assert_eq!(target.tail_epoch, leased_epoch.wrapping_add(1));
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
        source.tail_buffer = "ghbdtn".to_string();
        source.prepare_exact_manual_toggle_layout_handoff();
        let leased_epoch = source.tail_epoch;

        let mut target = LayIbusEngine::new(
            "/engine/ru".to_string(),
            shared.clone(),
            true,
            true,
            LayConfig::default(),
        );
        assert!(target.bind_focus_path());
        target.reset_for_ibus_soft_reset();

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
        assert!(target.suppress_next_committed_tail_autocorrect);
        assert!(state.suppress_next_committed_tail_autocorrect);
        assert!(state.preserve_active_path_until.is_none());
        assert!(state.exact_manual_toggle_handoff_epoch.is_none());
        let suppression = state
            .exact_manual_toggle_suppression
            .as_ref()
            .expect("exact suppression");
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
        assert!(!target.suppress_next_committed_tail_autocorrect);
        let state = shared.lock().expect("lay ime state poisoned");
        assert!(!state.suppress_next_committed_tail_autocorrect);
        assert!(state.exact_manual_toggle_suppression.is_none());
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
        publisher.tail_buffer = "вот ".to_string();
        publisher.publish_tail_handoff();
        let mut reader = LayIbusEngine::new(
            "/reader".to_string(),
            shared,
            true,
            true,
            LayConfig::default(),
        );
        reader.tail_buffer.clear();

        reader.refresh_empty_tail_from_handoff();

        assert_eq!(reader.tail_buffer, "вот ");
        assert_eq!(reader.preedit_fast.token(), "вот");
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
        publisher.tail_buffer = "вот ".to_string();
        publisher.publish_tail_handoff();
        publisher.publish_active_path_preserve_handoff(Instant::now() + Duration::from_millis(100));
        let mut empty_engine = LayIbusEngine::new(
            "/empty".to_string(),
            shared.clone(),
            true,
            true,
            LayConfig::default(),
        );
        empty_engine.tail_buffer.clear();

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

        engine.word_input_mode = Some(WordInputMode::ManagedCommit);
        engine.push_tail_char('a');
        engine.push_tail_char(' ');

        assert_eq!(engine.word_input_mode, None);
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

        engine.word_input_mode = Some(WordInputMode::ManagedCommit);
        engine.push_tail_char('f');
        engine.reset_for_ibus_focus_change();

        assert_eq!(engine.tail_buffer, "f");
        assert_eq!(engine.word_input_mode, Some(WordInputMode::ManagedCommit));
    }
}
