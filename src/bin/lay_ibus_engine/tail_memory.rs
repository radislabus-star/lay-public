use super::engine::{
    LayIbusEngine, PendingImeCompletionLearning, PendingSystemOutcomeFeedback, SystemOutcomeKind,
};
use super::protocol::PendingImeAutoUndo;
use lay::text_edit::{VisibleTailSnapshot, VisibleTailSource};
use std::time::{Duration, Instant};

const IME_AUTO_UNDO_MAX_AGE: Duration = Duration::from_secs(30);

impl LayIbusEngine {
    pub(super) fn remember_pending_ime_auto_undo(&self, original: String, replacement: String) {
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.pending_auto_undo = (original != replacement
            && !original.trim().is_empty()
            && !replacement.trim().is_empty())
        .then_some(PendingImeAutoUndo {
            original,
            replacement,
            recorded_at: Instant::now(),
        });
    }

    pub(super) fn take_pending_ime_auto_undo(&self) -> Option<PendingImeAutoUndo> {
        let Ok(mut state) = self.shared.lock() else {
            return None;
        };
        let pending = state.pending_auto_undo.take()?;
        if pending.recorded_at.elapsed() > IME_AUTO_UNDO_MAX_AGE
            || !self.tail_buffer.ends_with(&pending.replacement)
        {
            return None;
        }
        Some(pending)
    }

    pub(super) fn restore_pending_ime_auto_undo(&self, pending: PendingImeAutoUndo) {
        if pending.recorded_at.elapsed() > IME_AUTO_UNDO_MAX_AGE {
            return;
        }
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.pending_auto_undo = Some(pending);
    }

    pub(super) fn clear_pending_ime_auto_undo(&self) {
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.pending_auto_undo = None;
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
        self.arm_visible_postcondition_with_feedback(dispatched_at, None);
    }

    pub(super) fn arm_visible_postcondition_with_feedback(
        &mut self,
        dispatched_at: Instant,
        feedback: Option<PendingSystemOutcomeFeedback>,
    ) {
        if !self.surrounding_text_supported {
            return;
        }
        let snapshot = VisibleTailSnapshot::new(
            VisibleTailSource::ImeCommittedTail,
            self.tail_buffer.clone(),
            Some(self.path.clone()),
            self.tail_epoch,
        )
        .identity();
        self.pending_visible_postcondition = Some(super::engine::PendingVisiblePostcondition {
            expected_suffix: self.tail_buffer.clone(),
            snapshot,
            dispatched_epoch: self.tail_epoch,
            dispatched_at,
            feedback,
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
        let observed = self
            .surrounding_text_snapshot
            .as_ref()
            .and_then(|snapshot| {
                snapshot.suffix_before_cursor(pending.expected_suffix.chars().count())
            });
        let status = if observed.as_deref() == Some(pending.expected_suffix.as_str()) {
            self.record_observed_system_outcome(pending.feedback.as_ref());
            record_causal_outcome("confirmed_positive", &pending, self.tail_epoch);
            "observed"
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
                lay::typing_cpu::TypingCpu::record_accepted_layout_projection(
                    &feedback.original,
                    &feedback.replacement,
                );
            }
            SystemOutcomeKind::Correction => {
                lay::typing_cpu::TypingCpu::record_user_correction(
                    &feedback.original,
                    &feedback.replacement,
                    &feedback.replacement,
                    feedback.kind.operation(),
                );
            }
        }
    }

    pub(super) fn selected_visible_completion_suffix(&self) -> String {
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
    }

    pub(super) fn close_committed_tail_field(&mut self) {
        self.pending_ime_completion_learning = None;
        self.tail_buffer.clear();
        self.preedit_fast.reset();
        self.suppress_next_committed_tail_autocorrect = false;
        self.word_input_mode = None;
        self.last_tail_input_at = None;
        self.last_commit_at = None;
        self.recent_committed_tail_replace = None;
        self.pending_visible_postcondition = None;
        self.tail_epoch = self.tail_epoch.wrapping_add(1);
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.handoff_tail_buffer.clear();
        state.handoff_tail_epoch = self.tail_epoch;
        state.handoff_focus_receipt = None;
        state.suppress_next_committed_tail_autocorrect = false;
        state.preserve_active_path_until = None;
        state.pending_auto_undo = None;
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
        self.suppress_next_committed_tail_autocorrect = false;
        self.tail_epoch = self.tail_epoch.wrapping_add(1);
        if let Ok(mut state) = shared.lock() {
            state.handoff_tail_buffer.clear();
            state.handoff_tail_epoch = self.tail_epoch;
            state.handoff_focus_receipt = None;
            state.suppress_next_committed_tail_autocorrect = false;
            state.preserve_active_path_until = None;
            state.pending_auto_undo = None;
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
    }

    #[cfg(test)]
    pub(super) fn take_autocorrect_suppression_handoff(&self) -> bool {
        let Ok(mut state) = self.shared.lock() else {
            return false;
        };
        let suppress = state.suppress_next_committed_tail_autocorrect;
        state.suppress_next_committed_tail_autocorrect = false;
        suppress
    }

    pub(super) fn clear_autocorrect_suppression_handoff(&self) {
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.suppress_next_committed_tail_autocorrect = false;
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
        false
    }
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
        engine.remember_pending_ime_auto_undo("проверрка ".to_string(), "проверка ".to_string());

        let pending = engine
            .take_pending_ime_auto_undo()
            .expect("exact autocorrect undo");

        assert_eq!(pending.original, "проверрка ");
        assert_eq!(pending.replacement, "проверка ");
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
        engine.remember_pending_ime_auto_undo("проверрка ".to_string(), "проверка ".to_string());

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
