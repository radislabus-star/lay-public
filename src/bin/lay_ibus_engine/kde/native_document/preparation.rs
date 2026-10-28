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

    /// Native proof selects the presentation; the common witness and tail
    /// owners retain their original identity, revision and atomic update gates.
    pub(crate) fn acknowledge_managed_word_separator_presentation(&mut self, actual: &str) -> bool {
        if self.native_word_preparation_identity().is_none() {
            return false;
        }
        let token = self.last_tail_token_text();
        let Some(witness) = self
            .client_context
            .managed_word_start
            .as_ref()
            .and_then(|witness| witness.with_observed_separator_presentation(actual, &token))
        else {
            return false;
        };
        if !self.acknowledge_tail_word_separator_presentation(actual) {
            return false;
        }
        self.client_context.managed_word_start = Some(witness);
        true
    }

    /// Reuse the canonical zero-character boundary witness after native
    /// presentation acknowledgement, without creating an input revision.
    pub(crate) fn arm_native_observed_word_boundary(&mut self) {
        if !self.native_document_is_current()
            || !self.committed_tail.buffer.ends_with(char::is_whitespace)
            || self.committed_tail.pending_visible_postcondition.is_some()
        {
            return;
        }
        if let Some(witness) = self.managed_word_start_from_current_snapshot("") {
            self.client_context.managed_word_start = Some(witness);
        }
    }

    /// A native boundary may arrive after one completed local append. Retain
    /// that actual boundary and exact effect as inert preparation; the pending
    /// letter is never inserted into the observed client snapshot.
    pub(crate) fn arm_native_observed_first_append_boundary(&mut self) -> bool {
        if !self.native_document_is_current()
            || self.client_context.exact_surrounding_refresh_available
            || self.composition.word_input_mode != Some(crate::engine::WordInputMode::ManagedCommit)
            || self.committed_tail.pending_visible_postcondition.is_some()
            || self.native_word_preparation_identity().is_some()
        {
            return false;
        }
        let token = self.last_tail_token_text();
        if token.chars().count() != 1 || !token.chars().all(char::is_alphabetic) {
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
            || effect.observation_revision >= self.client_context.surrounding_observation_revision
        {
            return false;
        }
        let Some(body) = self
            .committed_tail
            .buffer
            .strip_suffix(&token)
            .and_then(|prefix| prefix.strip_suffix([' ', '\u{a0}']))
        else {
            return false;
        };
        let actual = self
            .client_context
            .surrounding_text_snapshot
            .as_ref()
            .unwrap();
        let separator = actual
            .cursor_pos
            .checked_sub(1)
            .and_then(|slot| actual.text.chars().nth(slot as usize));
        let Some(separator @ (' ' | '\u{a0}')) = separator else {
            return false;
        };
        let prefix = format!("{body}{separator}");
        let Some(start_epoch) = effect.tail_epoch.checked_sub(1) else {
            return false;
        };
        let Some(witness) =
            self.observed_word_start_from_snapshot_tail("", false, &prefix, start_epoch)
        else {
            return false;
        };
        if !self.acknowledge_tail_word_separator_presentation(&format!("{prefix}{token}")) {
            return false;
        }
        self.client_context.managed_word_start = Some(witness);
        self.invalidate_space_autocorrect_path();
        crate::trace::record(
            r#"{"kind":"ibus_native_document","stage":"late_word_boundary_observed","observed_word_chars":0,"edit_authority":false}"#,
        );
        true
    }
}
