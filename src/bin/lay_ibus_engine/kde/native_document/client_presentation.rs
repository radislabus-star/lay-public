//! Native Chrome presentation of already authorized output on KDE.
//! The common tail owner and final result verifier retain their contracts.
use super::{LayIbusEngine, SurroundingTextSnapshot};
use crate::engine::SystemOutcomeKind;
use lay::text_edit::VisibleTailSource;

impl LayIbusEngine {
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
