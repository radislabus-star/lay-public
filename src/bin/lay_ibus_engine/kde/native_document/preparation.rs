//! Read-only Space work for an observed word followed by owned CommitText.
//! The retained boundary is data provenance. A fresh actual native document
//! remains necessary before any prepared action can edit the field.

use crate::engine::LayIbusEngine;

impl LayIbusEngine {
    /// Only this current actual KDE field can extend the existing boundary
    /// protocol. The already committed native word must never be republished.
    pub(crate) fn native_space_boundary_continuation_is_current(
        &self,
        identity: &crate::engine::InputFrameIdentity,
    ) -> bool {
        self.native_document_is_current()
            && self.native_word_preparation_edit_is_current()
            && self.composition.buffer.is_empty()
            && !self.composition.legacy_word_preedit_active
            && self.composition.word_input_mode == Some(crate::engine::WordInputMode::ManagedCommit)
            && self.input_frame_authority_matches(identity)
            && self.capture_space_boundary_pair_scope() == identity.space_boundary_pair
            && identity.space_boundary_pair.as_ref().is_some_and(|pair| {
                matches!(
                    pair.provenance,
                    crate::engine::SpaceBoundaryPairProvenance::NativePrepared { .. }
                )
            })
    }

    /// Receipt ownership can remain current while its actual Space observation
    /// is pending. A different native field cannot consume the retained work.
    pub(crate) fn native_space_boundary_owner_is_current(
        &self,
        expected: Option<&super::NativeDocumentBinding>,
        frame: &crate::engine::InputFrameIdentity,
    ) -> bool {
        expected.is_some_and(|expected| {
            let Some(current) = self.context_token.as_ref() else {
                return false;
            };
            let Some(boundary) = frame
                .committed_tail
                .strip_suffix(' ')
                .and_then(|prefix| u32::try_from(prefix.chars().count()).ok())
            else {
                return false;
            };
            expected.tail_epoch == frame.tail_epoch
                && self.committed_tail.epoch == frame.tail_epoch
                && (current == &expected.token
                    || current.is_observed_boundary_successor_of(
                        &expected.token,
                        frame.tail_epoch,
                        boundary,
                    ))
                && expected.observation_owner_is_current_for_token(self, current)
                && self
                    .client_context
                    .native_document_binding
                    .as_ref()
                    .is_some_and(|actual| actual.identity() == expected.identity())
        })
    }

    /// Current owned appends can permit one more source read, never an edit.
    pub(super) fn native_owned_append_preparation_is_current(&self) -> bool {
        (self.has_unfinished_us_managed_input()
            || self
                .capture_space_boundary_pair_scope()
                .is_some_and(|scope| {
                    matches!(
                        scope.provenance,
                        crate::engine::SpaceBoundaryPairProvenance::NativePrepared { .. }
                    )
                }))
            && self.native_word_preparation_identity().is_some()
    }

    /// A coherent native read may still precede our latest CommitText. Only
    /// the same owned append chain can request another actual read; the
    /// projected state remains inert and is never installed as an observation.
    pub(super) fn native_owned_append_observation_is_pending(
        &self,
        snapshot: &super::SurroundingTextSnapshot,
    ) -> bool {
        if !self.native_owned_append_preparation_is_current() || snapshot.has_selection() {
            return false;
        }
        let Some(expected) = self.managed_word_start_preparation_snapshot() else {
            return false;
        };
        let cursor = snapshot.cursor_pos as usize;
        let expected_cursor = expected.cursor_pos as usize;
        let Some(word_start) =
            expected_cursor.checked_sub(self.last_tail_token_text().chars().count())
        else {
            return false;
        };
        cursor >= word_start
            && cursor < expected_cursor
            && snapshot
                .text
                .chars()
                .take(cursor)
                .eq(expected.text.chars().take(cursor))
            && snapshot
                .text
                .chars()
                .skip(cursor)
                .eq(expected.text.chars().skip(expected_cursor))
    }

    pub(crate) fn native_word_preparation_identity(&self) -> Option<u64> {
        let binding = self.client_context.native_document_binding.as_ref()?;
        if !binding.observation_owner_is_current(self)
            || self.context_handoff_sealed
            || self.client_context.exact_surrounding_refresh_available
            || self
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.has_selection())
        {
            return None;
        }
        self.managed_word_start_preparation_identity()
    }

    pub(crate) fn native_word_preparation_edit_is_current(&self) -> bool {
        self.native_word_preparation_identity().is_some()
            && self.native_manual_word_is_current()
            && self.managed_word_start_preparation_snapshot().as_ref()
                == self.client_context.surrounding_text_snapshot.as_ref()
    }

    pub(crate) fn native_word_preparation_pair_snapshot(
        &self,
        witness_identity: u64,
        cursor: u32,
    ) -> Option<super::SurroundingTextSnapshot> {
        if self.native_word_preparation_identity() != Some(witness_identity)
            || !self.native_word_preparation_edit_is_current()
        {
            return None;
        }
        self.client_context
            .surrounding_text_snapshot
            .as_ref()
            .filter(|snapshot| snapshot.cursor_pos == cursor)
            .cloned()
    }
}
