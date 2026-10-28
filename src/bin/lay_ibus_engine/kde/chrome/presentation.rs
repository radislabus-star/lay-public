//! Native Chrome presentation of already authorized output on KDE.
//! The common tail owner and final result verifier retain their contracts.
use super::{LayIbusEngine, SurroundingTextSnapshot};
use crate::engine::SystemOutcomeKind;
use lay::text_edit::VisibleTailSource;

impl LayIbusEngine {
    /// A literal append can make Chrome render the preceding NBSP as ASCII.
    /// Validate the full observed field and the existing local append; this
    /// acknowledges presentation, without projecting an observation or edit.
    pub(super) fn acknowledge_native_word_separator_reflow(
        &mut self,
        actual: &SurroundingTextSnapshot,
    ) -> bool {
        if crate::trace::enabled() {
            crate::trace::record(serde_json::json!({
                "kind": "ibus_native_document", "stage": "word_separator_observation",
                "preparation_chain_refusal": self.managed_word_start_chain_invalid_reason(false),
                "native_preparation": self.native_word_preparation_identity().is_some(),
                "native_binding_current": self.client_context.native_document_binding.as_ref().is_some_and(|binding| binding.is_current(self)),
                "native_document_current": self.native_document_is_current(),
                "actual_cursor": actual.cursor_pos,
                "tail_chars": self.committed_tail.buffer.chars().count(),
                "tail_epoch": self.committed_tail.epoch,
            }).to_string());
        }
        if !self
            .client_context
            .native_document_binding
            .as_ref()
            .is_some_and(|binding| binding.is_current(self))
            || self.context_handoff_sealed
            || !self.client_context.managed_input
            || self.composition.word_input_mode != Some(crate::engine::WordInputMode::ManagedCommit)
            || !self.composition.buffer.is_empty()
            || actual.has_selection()
            || actual.cursor_pos as usize > actual.text.chars().count()
            || self.native_word_preparation_identity().is_none()
        {
            return false;
        }
        let Some(effect) = self.client_context.managed_commit_snapshot_floor else {
            return false;
        };
        if effect.focus_serial != self.client_context.focus_serial
            || effect.runtime_owner_lease_identity
                != self.client_context.runtime_owner_lease_identity
            || effect.layout_generation != self.layout_gesture.layout_generation
            || effect.tail_epoch != self.committed_tail.epoch
            || effect.observation_revision > self.client_context.surrounding_observation_revision
        {
            return false;
        }
        let token = self.last_tail_token_text();
        if token.is_empty() || !token.chars().all(char::is_alphabetic) {
            return false;
        }
        let Some(prefix) = self
            .committed_tail
            .buffer
            .strip_suffix(&token)
            .and_then(|prefix| prefix.strip_suffix('\u{a0}'))
        else {
            return false;
        };
        let Some(projected) = self.managed_word_start_preparation_snapshot() else {
            return false;
        };
        let Some(slot) = projected
            .cursor_pos
            .checked_sub(token.chars().count() as u32 + 1)
        else {
            return false;
        };
        if projected.text.chars().nth(slot as usize) != Some('\u{a0}') {
            return false;
        }
        let prefix_before_space: String = projected.text.chars().take(slot as usize).collect();
        let observed_prefix: String = actual
            .text
            .chars()
            .take(actual.cursor_pos as usize)
            .collect();
        let Some(observed_word) = observed_prefix.strip_prefix(&format!("{prefix_before_space} "))
        else {
            return false;
        };
        let witness = self.client_context.managed_word_start.as_ref().unwrap();
        if !token.starts_with(observed_word)
            || (observed_word.chars().count() as u64) < witness.anchored_token_chars()
            || !actual
                .text
                .chars()
                .skip(actual.cursor_pos as usize)
                .eq(projected.text.chars().skip(projected.cursor_pos as usize))
        {
            return false;
        }
        let actual_tail = format!("{prefix} {token}");
        if !self.acknowledge_managed_word_separator_presentation(&actual_tail) {
            return false;
        }
        self.invalidate_space_autocorrect_path();
        crate::trace::record(
            r#"{"kind":"ibus_native_document","stage":"word_separator_presentation_acknowledged","sent":160,"observed":32}"#,
        );
        true
    }

    /// An owned closing Space can reach Chrome through literal input rather
    /// than a pending edit. Recognize only that tail slot; keep all earlier
    /// owned characters exact and leave scope, epoch and layout unchanged.
    pub(super) fn acknowledge_native_closing_space(&mut self) -> bool {
        if !self.native_document_is_current()
            || self.context_handoff_sealed
            || self.committed_tail.pending_visible_postcondition.is_some()
        {
            return false;
        }
        let Some(body) = self.committed_tail.buffer.strip_suffix(' ') else {
            return false;
        };
        let actual_tail = format!("{body}\u{a0}");
        if !self
            .client_context
            .surrounding_text_snapshot
            .as_ref()
            .is_some_and(|actual| {
                !actual.has_selection()
                    && actual
                        .suffix_before_cursor(self.committed_tail.buffer.chars().count())
                        .as_deref()
                        == Some(actual_tail.as_str())
            })
        {
            return false;
        }
        if self
            .acknowledge_owned_space_tail(&actual_tail, None)
            .is_err()
        {
            return false;
        }
        self.arm_native_observed_word_boundary();
        crate::trace::record(
            r#"{"kind":"ibus_native_document","stage":"closing_space_presentation_acknowledged","sent":32,"observed":160}"#,
        );
        true
    }

    /// Chrome can render a layout edit's newly emitted final Space as NBSP.
    /// Compare the complete field, caret and selection; only that emitted slot
    /// may differ. Then reuse the tail owner's existing presentation receipt.
    pub(super) fn acknowledge_native_layout_space(&mut self) -> bool {
        let Some(pending) = self.committed_tail.pending_visible_postcondition.as_ref() else {
            return false;
        };
        if !self.native_document_is_current()
            || self.context_handoff_sealed
            || pending.dispatched_epoch != self.committed_tail.epoch
            || pending.snapshot.source != VisibleTailSource::ImeCommittedTail
            || pending.expected_suffix != self.committed_tail.buffer
            || pending.feedback.as_ref().is_some_and(|feedback| {
                !matches!(feedback.kind, SystemOutcomeKind::LayoutProjection)
            })
            || !pending.native_document.as_ref().is_some_and(|expected| {
                expected.is_current(self)
                    && self
                        .client_context
                        .native_document_binding
                        .as_ref()
                        .is_some_and(|actual| actual.identity == expected.identity)
            })
        {
            return false;
        }
        let (Some(expected), Some(body), Some(sent)) = (
            pending.expected_external_snapshot.as_ref(),
            pending.expected_suffix.strip_suffix(' '),
            pending
                .layout_sync_text
                .as_deref()
                .filter(|sent| sent.ends_with(' ')),
        ) else {
            return false;
        };
        if expected.has_selection()
            || expected
                .suffix_before_cursor(sent.chars().count())
                .as_deref()
                != Some(sent)
        {
            return false;
        }
        let Some(slot) = expected.cursor_pos.checked_sub(1).map(|p| p as usize) else {
            return false;
        };
        let mut text: Vec<_> = expected.text.chars().collect();
        if text.get(slot) != Some(&' ') {
            return false;
        }
        text[slot] = '\u{a0}';
        let observed = SurroundingTextSnapshot::new(
            text.into_iter().collect(),
            expected.cursor_pos,
            expected.anchor_pos,
        );
        if self.client_context.surrounding_text_snapshot.as_ref() != Some(&observed) {
            return false;
        }
        let actual_tail = format!("{body}\u{a0}");
        if self
            .acknowledge_owned_space_tail(&actual_tail, None)
            .is_err()
        {
            return false;
        }
        // The owner call above retains the pending result under this exclusive
        // engine borrow. Its original deadline, feedback and scope stay intact.
        if let Some(pending) = self.committed_tail.pending_visible_postcondition.as_mut() {
            pending.expected_suffix = actual_tail;
            pending.expected_external_snapshot = Some(observed);
        }
        crate::trace::record(
            r#"{"kind":"ibus_native_document","stage":"layout_space_presentation_acknowledged","sent":32,"observed":160}"#,
        );
        true
    }
}
