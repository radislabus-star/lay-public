use super::native_cursor_edit::{CursorEditOperation, ManualCursorEditReceipt};
use super::{
    OutcomeProof, IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH, IBUS_CAP_PREEDIT_TEXT,
    IBUS_CAP_SURROUNDING_TEXT, IBUS_INPUT_HINT_HIDDEN_TEXT, IBUS_INPUT_HINT_PRIVATE,
    IBUS_INPUT_PURPOSE_PASSWORD, IBUS_INPUT_PURPOSE_PIN,
};
use crate::atomic::{AtomicCapability, AtomicEnvelope, AtomicPriorReceipt};
use crate::context_admission::{
    AdmissionToken, ContextAdmissionAdapter, PendingFence, RequestGeneration,
};
use crate::engine::WordInputMode;
use crate::engine::{PendingSystemOutcomeFeedback, SystemOutcomeKind};
use crate::output::EngineOutput;
use crate::output::{AtomicProposal, PROPOSAL_CONSUMED_NO_EFFECT, PROPOSAL_FRAME_READY};
use crate::tail_memory::{
    record_causal_outcome, surrounding_snapshot_match, SurroundingSnapshotMatch,
};
use lay::text_edit::{VisibleTailSnapshot, VisibleTailSource};
use zbus::fdo;

include!("native_cursor_observation.rs");

/// A Reset invalidates the old admission token before the engine callback can
/// inspect it. This record is only a non-authoritative predecessor candidate;
/// the authenticated Reset and the next exact snapshot create new authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextResetRereceiptCandidate {
    // Original confirmed client observation carried through an owned boundary.
    owned_preedit_return_snapshot: Option<SurroundingTextSnapshot>,
    completed_replay: Option<crate::protocol::ExactManualToggleSuppression>,
    confirmed_native_replay_reset_ack: Option<ConfirmedNativeReplayResetAck>,
    native_mutation_reset_evidence: Option<NativeMutationResetEvidence>,
    pub(crate) token: AdmissionToken,
    pub(crate) tail_epoch: u64,
    pub(crate) token_text: String,
    pub(crate) observed_suffix_chars: u32,
    pub(crate) armed_revision: u64,
    published_preedit: Option<PublishedPreeditWitness>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingContextResetRereceipt {
    completed_replay: Option<crate::protocol::ExactManualToggleSuppression>,
    confirmed_native_replay_reset_ack: Option<ConfirmedNativeReplayResetAck>,
    native_mutation_reset_evidence: Option<NativeMutationResetEvidence>,
    pub(crate) token: AdmissionToken,
    pub(crate) predecessor_token: AdmissionToken,
    pub(crate) tail_epoch: u64,
    pub(crate) token_text: String,
    pub(crate) observed_suffix_chars: u32,
    pub(crate) armed_revision: u64,
    pub(crate) confirmed: bool,
    // Only an authenticated local append sets this. Its former client snapshot
    // was invalidated by the owned commit, so the first new full receipt is
    // distinct from refreshing an already observed full token.
    pub(crate) awaiting_owned_append_receipt: bool,
    // Inert one-shot echo of a previously confirmed managed word. It never
    // grants authority until the immediately following exact full receipt.
    strict_prefix_echo_revision: Option<u64>,
    published_preedit: Option<PublishedPreeditWitness>,
}

/// One Reset echo of an independently confirmed, fully completed native replay.
/// The existing receipt owns token, epoch, revision and text. A spent replay
/// scope alone is insufficient: rejection can also consume its source token.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfirmedNativeReplayResetAck {
    focus_serial: u64,
    runtime_owner_lease_identity: u64,
    layout_generation: u64,
}

/// A native deletion is inert until the first whole actual shortened receipt.
/// Only that receipt can acknowledge one later Reset; it never re-arms on refresh.
#[derive(Debug, Clone, PartialEq, Eq)]
enum NativeMutationResetEvidence {
    // Inert manual-only editing provenance shares the existing single slot;
    // it is never an ordinary confirmed erasure/reset grant.
    CursorEditing(Box<ManualCursorEditReceipt>),
    Awaiting {
        actual_before: SurroundingTextSnapshot,
        effect_token: AdmissionToken,
        focus_serial: u64,
        runtime_owner_lease_identity: u64,
        layout_generation: u64,
    },
    Confirmed {
        receipt_token: AdmissionToken,
        focus_serial: u64,
        runtime_owner_lease_identity: u64,
        layout_generation: u64,
    },
}

impl NativeMutationResetEvidence {
    fn source_token(&self) -> &AdmissionToken {
        match self {
            Self::CursorEditing(receipt) => &receipt.token,
            Self::Awaiting { effect_token, .. } => effect_token,
            Self::Confirmed { receipt_token, .. } => receipt_token,
        }
    }

    fn identity(&self) -> (u64, u64, u64) {
        match self {
            Self::CursorEditing(receipt) => (
                receipt.focus_serial,
                receipt.runtime_owner_lease_identity,
                receipt.layout_generation,
            ),
            Self::Awaiting {
                focus_serial,
                runtime_owner_lease_identity,
                layout_generation,
                ..
            }
            | Self::Confirmed {
                focus_serial,
                runtime_owner_lease_identity,
                layout_generation,
                ..
            } => (
                *focus_serial,
                *runtime_owner_lease_identity,
                *layout_generation,
            ),
        }
    }
}

/// Exact caret boundary observed before the first locally managed character.
/// Firefox can clear the live surrounding snapshot with Reset before it
/// publishes the committed word. This witness remains usable only while every
/// later character is the next local CommitText in the same focus/owner chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManagedWordStartWitness {
    pub(crate) identity: u64,
    focus_receipt: Option<String>,
    focus_serial: u64,
    runtime_owner_lease_identity: u64,
    layout_generation: u64,
    exact_refresh_at_start: bool,
    start_tail_epoch: u64,
    // Actual owned manual handoffs advance the mirror epoch without a key.
    // Keep this adjustment separate from Space/deletion start evidence.
    manual_tail_epoch_offset: u64,
    anchored_token_chars: u64,
    start_committed_tail: String,
    snapshot_prefix: String,
    snapshot_suffix: String,
    start_cursor: u32,
    armed_revision: u64,
    reset_echo_epoch: Option<u64>,
    // Inert provenance for the already confirmed word before this preedit.
    // It cannot authorize an edit of either word while the preedit is open.
    closed_word_predecessor: Option<Box<ContextResetRereceiptCandidate>>,
}

/// Output provenance only: never a client snapshot or an edit target.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PublishedPreeditWitness {
    prefix_chars: u32,
    text: String,
}

fn retired_preedit_right_edge_is_safe(snapshot: &SurroundingTextSnapshot, end: usize) -> bool {
    let mut right = snapshot.text.chars().skip(end);
    match right.next() {
        None => true,
        Some('\u{200b}') => right.next().is_none(),
        Some(ch) => crate::preedit::is_observed_word_boundary(ch),
    }
}

fn snapshot_matches_retired_preedit(
    snapshot: &SurroundingTextSnapshot,
    pending: &PendingContextResetRereceipt,
) -> bool {
    let Some(published) = pending.published_preedit.as_ref() else {
        return false;
    };
    if snapshot.has_selection()
        || published.prefix_chars > pending.observed_suffix_chars
        || !pending
            .token_text
            .chars()
            .take(published.prefix_chars as usize)
            .eq(published.text.chars().take(published.prefix_chars as usize))
    {
        return false;
    }
    let cursor = snapshot.cursor_pos as usize;
    let presentation_chars = published.text.chars().count();
    let old_prefix_chars = published.prefix_chars as usize;
    let current_chars = pending.observed_suffix_chars as usize;
    let old_suffix_chars = presentation_chars.saturating_sub(old_prefix_chars);
    if old_prefix_chars == current_chars {
        // HidePreeditText can be followed by one last echo of the current
        // display-only publication. Keep it solely as an inert witness; the
        // pending manual gesture still requires a later exact token receipt.
        let Some(start) = cursor.checked_sub(current_chars) else {
            return false;
        };
        return snapshot.suffix_before_cursor(current_chars).as_deref()
            == Some(pending.token_text.as_str())
            && (start == 0
                || snapshot
                    .text
                    .chars()
                    .nth(start - 1)
                    .is_some_and(crate::preedit::is_observed_word_boundary))
            && snapshot
                .text
                .chars()
                .skip(start)
                .take(presentation_chars)
                .eq(published.text.chars())
            && retired_preedit_right_edge_is_safe(snapshot, start + presentation_chars);
    }
    // The old display suffix can remain after one owned next character.
    // This witness does not authorize an edit without a fresh exact receipt.
    let old_suffix_after_append = old_suffix_chars > 0
        && cursor >= current_chars
        && snapshot.suffix_before_cursor(current_chars).as_deref()
            == Some(pending.token_text.as_str())
        && (cursor == current_chars
            || snapshot
                .text
                .chars()
                .nth(cursor - current_chars - 1)
                .is_some_and(crate::preedit::is_observed_word_boundary))
        && snapshot
            .text
            .chars()
            .skip(cursor)
            .take(old_suffix_chars)
            .eq(published.text.chars().skip(old_prefix_chars))
        && retired_preedit_right_edge_is_safe(snapshot, cursor + old_suffix_chars);
    if old_suffix_after_append {
        return true;
    }
    // Firefox can report the same retired publication with either the old
    // insertion cursor or the cursor advanced by our observed append. Both
    // retain only an inert witness until the exact committed text arrives.
    [pending.observed_suffix_chars, published.prefix_chars]
        .into_iter()
        .any(|prefix_chars| {
            let Some(start) = cursor.checked_sub(prefix_chars as usize) else {
                return false;
            };
            let end = start + presentation_chars;
            cursor <= end
                && (start == 0
                    || snapshot
                        .text
                        .chars()
                        .nth(start - 1)
                        .is_some_and(crate::preedit::is_observed_word_boundary))
                && snapshot
                    .text
                    .chars()
                    .skip(start)
                    .take(presentation_chars)
                    .eq(published.text.chars())
                && retired_preedit_right_edge_is_safe(snapshot, end)
        })
}

fn snapshot_exactly_bounds_token(
    snapshot: &SurroundingTextSnapshot,
    token_text: &str,
    chars: usize,
) -> bool {
    if chars == 0
        || snapshot.has_selection()
        || snapshot.suffix_before_cursor(chars).as_deref() != Some(token_text)
    {
        return false;
    }
    let start = snapshot.cursor_pos as usize - chars;
    let left_is_boundary = start == 0
        || snapshot
            .text
            .chars()
            .nth(start - 1)
            .is_some_and(crate::preedit::is_observed_word_boundary);
    let right_is_boundary = snapshot
        .text
        .chars()
        .nth(snapshot.cursor_pos as usize)
        .is_none_or(crate::preedit::is_observed_word_boundary);
    left_is_boundary && right_is_boundary
}

fn snapshot_has_transient_zero_width_boundary(
    snapshot: &SurroundingTextSnapshot,
    pending: &PendingContextResetRereceipt,
) -> bool {
    let chars = pending.observed_suffix_chars as usize;
    if chars == 0
        || snapshot.has_selection()
        || snapshot.suffix_before_cursor(chars).as_deref() != Some(pending.token_text.as_str())
    {
        return false;
    }
    let cursor = snapshot.cursor_pos as usize;
    let start = cursor - chars;
    let left_is_boundary = start == 0
        || snapshot
            .text
            .chars()
            .nth(start - 1)
            .is_some_and(crate::preedit::is_observed_word_boundary);
    let mut right = snapshot.text.chars().skip(cursor);
    left_is_boundary
        && right.next() == Some('\u{200b}')
        && right
            .next()
            .is_none_or(crate::preedit::is_observed_word_boundary)
}

fn snapshot_exactly_bounds_strict_token_prefix(
    snapshot: &SurroundingTextSnapshot,
    token_text: &str,
) -> bool {
    if snapshot.has_selection() {
        return false;
    }
    let Some(cursor_byte) = snapshot
        .text
        .char_indices()
        .map(|(offset, _)| offset)
        .chain(std::iter::once(snapshot.text.len()))
        .nth(snapshot.cursor_pos as usize)
    else {
        return false;
    };
    let prefix = snapshot.text[..cursor_byte]
        .rsplit(crate::preedit::is_observed_word_boundary)
        .next()
        .unwrap_or_default();
    !prefix.is_empty()
        && prefix.len() < token_text.len()
        && token_text.starts_with(prefix)
        && snapshot.text[cursor_byte..]
            .chars()
            .next()
            .is_none_or(crate::preedit::is_observed_word_boundary)
}

fn should_apply_auto_undo_before_postcondition(retry_status: &str) -> bool {
    retry_status == "ready_causal_precondition"
}

#[cfg(test)]
mod causal_precondition_order_tests {
    use super::should_apply_auto_undo_before_postcondition;

    #[test]
    fn causal_precondition_undo_precedes_stale_postcondition_quarantine() {
        assert!(should_apply_auto_undo_before_postcondition(
            "ready_causal_precondition"
        ));
        assert!(!should_apply_auto_undo_before_postcondition("ready"));
        assert!(!should_apply_auto_undo_before_postcondition(
            "ready_boundary_elided"
        ));
        assert!(!should_apply_auto_undo_before_postcondition(
            "waiting_exact_snapshot"
        ));
    }
}

use std::time::{Duration, Instant};

use crate::context_admission::{
    ActivationOutcome, EnginePath, KeyCallback, KeyCallbackObservation, LayoutIntentToken,
    SettledWordState, WordCompleteness, WordScope,
};
use crate::engine::{next_input_identity, LayIbusEngine};
use crate::protocol::{
    has_command_modifier, is_accept_completion_with_space_key, is_key_press, is_shift_key,
    AutocorrectSuppression, KEY_BACKSPACE, KEY_DOWN, KEY_ENTER, KEY_KP_ENTER, KEY_LEFT, KEY_RIGHT,
    KEY_SPACE, KEY_TAB, KEY_UP,
};
use crate::trace;

impl LayIbusEngine {
    pub(crate) async fn observe_context_focus_out(
        &mut self,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
    ) -> bool {
        self.try_install_pending_context_activation();
        let Some(admission) = self.context_admission.clone() else {
            if self.context_admission_required {
                self.context_handoff_sealed = false;
                self.revoke_context_word();
            }
            return !self.context_admission_required;
        };
        let Some(owner) = self.context_owner.clone() else {
            self.context_handoff_sealed = false;
            self.revoke_context_word();
            return false;
        };
        let observed = match admission.observe_callback(header, callback_entered).await {
            Ok(observed) => observed,
            Err(error) => {
                trace::record(format!(
                    r#"{{"kind":"ibus_focus","stage":"focus_out_stamp_refused","reason":"{error}"}}"#,
                ));
                self.context_handoff_sealed = false;
                self.revoke_context_word();
                return false;
            }
        };
        if admission.callback_is_stale_for(&owner, &observed) {
            trace::record(format!(
                r#"{{"kind":"ibus_focus","stage":"focus_out_stale_owner","serial":{}}}"#,
                observed.header.serial,
            ));
            return false;
        }
        if matches!(observed.header.member.as_str(), "FocusOut" | "FocusOutId")
            && admission.revocation_matches(&owner, &observed)
        {
            // Ingress already revoked this exact old owner. Retire its local
            // preedit before normal blur cleanup, preserving later received
            // focus metadata without admitting any old-word text handoff.
            self.context_handoff_sealed = false;
            self.strip_legacy_word_preedit_mirror();
            self.retire_context_word_locally();
            return true;
        }
        let accepted_focus = matches!(observed.header.member.as_str(), "FocusOut" | "FocusOutId")
            && admission.focus_out(&owner, &observed);
        if accepted_focus && self.composition.legacy_word_preedit_active {
            // FocusOut cancels client preedit. Remove its lexical mirror before
            // the admission reducer can seal or transfer committed-tail state.
            // The settled lineage described the canceled word, so this focus
            // transition must become source-free instead of sealing a prefix
            // under that stale lineage.
            self.strip_legacy_word_preedit_mirror();
            self.composition.preedit_visible = false;
            self.publish_tail_handoff();
            // Keep the committed prefix as inert data, but revoke every local
            // word witness. A broad adapter revoke would also erase a later
            // FocusIn already received for the pending successor.
            self.retire_context_word_locally();
            admission.retire_owned_preedit_after_focus_out(&owner, &observed);
            return true;
        }
        let accepted =
            accepted_focus && admission.seal_source(&owner, self.committed_tail.epoch, &observed);
        self.context_handoff_sealed = accepted;
        if !accepted {
            trace::record(format!(
                r#"{{"kind":"ibus_focus","stage":"source_seal_refused","serial":{},"accepted_focus":{},"tail_epoch":{},"owner_generation":{}}}"#,
                observed.header.serial,
                accepted_focus,
                self.committed_tail.epoch,
                owner.generation.0,
            ));
            if admission.focus_out_retired_by_later_ingress(&owner, &observed) {
                // The observer already received a later word revocation. Drop
                // only this old local path: revoking the adapter here would
                // erase the later FocusIn/Reset stamps before their callbacks.
                self.discard_context_activation();
                trace::record(
                    r#"{"kind":"ibus_focus","stage":"focus_out_retired_by_later_ingress"}"#,
                );
                return false;
            }
            self.revoke_context_word();
        }
        accepted
    }

    pub(crate) async fn observe_context_disable(
        &mut self,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
    ) -> bool {
        self.try_install_pending_context_activation();
        let Some(admission) = self.context_admission.clone() else {
            if self.context_admission_required {
                self.context_handoff_sealed = false;
                self.revoke_context_word();
            }
            return !self.context_admission_required;
        };
        let Some(owner) = self.context_owner.clone() else {
            self.context_handoff_sealed = false;
            self.revoke_context_word();
            return false;
        };
        let observed = match admission.observe_callback(header, callback_entered).await {
            Ok(observed) => observed,
            Err(error) => {
                trace::record(format!(
                    r#"{{"kind":"ibus_focus","stage":"disable_stamp_refused","reason":"{error}"}}"#,
                ));
                self.context_handoff_sealed = false;
                self.revoke_context_word();
                return false;
            }
        };
        if admission.callback_is_stale_for(&owner, &observed) {
            return false;
        }
        if observed.header.member == "Disable" && admission.revocation_matches(&owner, &observed) {
            self.context_handoff_sealed = false;
            // The observer's exact revocation already ended the old word.
            // A second global revoke would erase a received successor focus.
            self.strip_legacy_word_preedit_mirror();
            self.retire_context_word_locally();
            return true;
        }
        let accepted = observed.header.member == "Disable" && admission.disable(&owner, &observed);
        if !accepted {
            self.context_handoff_sealed = false;
            if admission.disable_retired_by_later_ingress(&owner, &observed) {
                // This exact source ticket is already revoked. A second global
                // revocation would erase later authenticated FocusIn stamps.
                // Retire only the matching local source, granting no handoff.
                self.discard_context_activation();
                trace::record(
                    r#"{"kind":"ibus_focus","stage":"disable_retired_by_later_ingress"}"#,
                );
                return false;
            }
            self.revoke_context_word();
        }
        accepted
    }

    pub(crate) async fn observe_context_revocation(
        &mut self,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
    ) -> ContextRevocationReceipt {
        if !self.context_admission_required {
            self.fail_context_activation_local();
            return ContextRevocationReceipt::Cleanup;
        }
        let reset_rereceipt_candidate = self.capture_context_reset_rereceipt_candidate();
        let reset_rereceipt_candidate = reset_rereceipt_candidate.filter(|candidate| {
            (candidate.completed_replay.is_none()
                && candidate.confirmed_native_replay_reset_ack.is_none()
                && candidate.native_mutation_reset_evidence.is_none())
                || header
                    .member()
                    .is_some_and(|member| member.as_str() == "Reset")
        });
        self.context_handoff_sealed = false;
        let mut installed_post_reset_scope = false;
        let mut matched_reset = false;
        if let (Some(admission), Some(owner)) =
            (self.context_admission.clone(), self.context_owner.clone())
        {
            match admission.observe_callback(header, callback_entered).await {
                Ok(observed) if admission.callback_is_stale(&observed) => {
                    return ContextRevocationReceipt::Refused
                }
                Ok(observed) if admission.revocation_matches(&owner, &observed) => {
                    matched_reset = true;
                    self.context_token = admission.current_token();
                    if let Some(token) = self.context_token.as_ref() {
                        self.context_word_scope = Some(token.word_scope());
                        installed_post_reset_scope = true;
                    }
                    self.arm_context_reset_rereceipt(reset_rereceipt_candidate, &owner);
                }
                Ok(_) => return ContextRevocationReceipt::Refused,
                Err(_) if admission.current_owner().as_ref() != Some(&owner) => {
                    return ContextRevocationReceipt::Refused
                }
                Err(_) => {
                    admission.revoke_current_owner(&owner);
                    self.context_token = None;
                }
            }
        }
        if !installed_post_reset_scope {
            if let Some(scope) = self.context_word_scope.as_mut() {
                scope.revoke_for_input_gap();
            }
        }
        self.committed_tail.pending_completion_learning = None;
        self.clear_preedit_completion_state();
        if matched_reset {
            ContextRevocationReceipt::Matched
        } else {
            ContextRevocationReceipt::Cleanup
        }
    }

    pub(crate) fn context_word_is_known(&self) -> bool {
        if !self.context_admission_required {
            return true;
        }
        let Some(admission) = self.context_admission.as_ref() else {
            return false;
        };
        self.context_word_scope.as_ref().is_some_and(|scope| {
            scope.lineage().completeness == WordCompleteness::KnownStart
                && self.context_token.as_ref().is_some_and(|token| {
                    token.matches_word_scope(scope) && admission.revalidate(token)
                })
        })
    }

    pub(crate) fn capture_stock_observed_boundary_source(
        &self,
        ch: char,
    ) -> Option<crate::engine::StockObservedBoundarySource> {
        if ch != ' '
            || !self.client_context.surrounding_text_callback_observed
            || self.client_context.exact_surrounding_refresh_available
            || !self.context_admission_required
            || self.context_handoff_sealed
            || self.atomic.active
            || self.content_is_sensitive()
            || !self.composition.buffer.is_empty()
            || self.committed_tail.buffer.ends_with(char::is_whitespace)
            || !self.current_external_snapshot_agrees_with_owned_tail()
            || matches!(
                self.committed_tail.autocorrect_suppression,
                Some(AutocorrectSuppression::ExactReplay(_))
            )
            || self.exact_replay_quarantine_active()
            || self.exact_replay_tail_change_quarantined
        {
            return None;
        }
        // The first word can still be UnknownStart before Space. Retaining
        // this real snapshot is inert; readout separately requires KnownStart
        // after the boundary and revalidates the live successor token.
        let (Some(admission), Some(token), Some(scope)) = (
            self.context_admission.as_ref(),
            self.context_token.as_ref(),
            self.context_word_scope.as_ref(),
        ) else {
            return None;
        };
        if !token.matches_word_scope(scope) || !admission.revalidate(token) {
            return None;
        }
        Some(crate::engine::StockObservedBoundarySource {
            focus_serial: self.client_context.focus_serial,
            runtime_owner_lease_identity: self.client_context.runtime_owner_lease_identity,
            // push_tail_char publishes exactly one successful local effect.
            tail_epoch: self.committed_tail.epoch.wrapping_add(1),
            observation_revision: self.client_context.surrounding_observation_revision,
            tail: format!("{} ", self.committed_tail.buffer),
            observed_external_prefix: self.current_full_external_prefix(),
        })
    }

    /// A stock client can expose the locally tracked word of this live field
    /// to an explicit manual replay when no widget receipt is available.
    /// This is source ownership, not a client delivery acknowledgement.
    pub(crate) fn context_stock_owned_manual_source_is_current(&self) -> bool {
        // A callback-capable widget needs exact current-word evidence before
        // our successful Space. A historical callback flag is insufficient.
        let locally_committed_boundary = self.committed_tail.last_commit_at.is_some()
            && self
                .committed_tail
                .observed_boundary_source
                .as_ref()
                .is_some_and(|source| {
                    source.focus_serial == self.client_context.focus_serial
                        && source.runtime_owner_lease_identity
                            == self.client_context.runtime_owner_lease_identity
                        && source.tail_epoch == self.committed_tail.epoch
                        && source.observation_revision
                            == self.client_context.surrounding_observation_revision
                        && source.tail == self.committed_tail.buffer
                })
            && !matches!(
                self.committed_tail.autocorrect_suppression,
                Some(AutocorrectSuppression::ExactReplay(_))
            )
            && !self.exact_replay_quarantine_active()
            && !self.exact_replay_tail_change_quarantined;
        self.context_admission_required
            && !self.context_handoff_sealed
            && !self.atomic.active
            && !self.content_is_sensitive()
            && !self.client_context.exact_surrounding_refresh_available
            && (!self.client_context.surrounding_text_callback_observed
                || locally_committed_boundary)
            && self.client_context.surrounding_text_snapshot.is_none()
            && self.client_context.managed_commit_snapshot_floor.is_none()
            && self.composition.buffer.is_empty()
            && !self.committed_tail.buffer.is_empty()
            && self.context_word_is_known()
    }

    /// A client can accept an append-only completion while the whole
    /// uncommitted word is this IME's live preedit. An exact-refresh client
    /// additionally requires the observed start transferred at the first key.
    /// This does not authorize replacement of already committed client text.
    pub(crate) fn context_owns_active_preedit_append_completion(&self) -> bool {
        if self.context_handoff_sealed
            || self.atomic.active
            || self.content_is_sensitive()
            || (self.client_context.exact_surrounding_refresh_available
                && !self.owned_preedit_start_boundary_is_current())
            || self
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .is_some_and(SurroundingTextSnapshot::has_selection)
            || !self.composition.legacy_word_preedit_active
            || !self.exact_owned_legacy_preedit_is_current()
            || !self.composition.preedit_visible
            || self.composition.preedit_dirty
            || self.composition.preedit_display_only_pending
            || self.composition.buffer.is_empty()
            || self.composition.cursor != self.composition.buffer.chars().count()
            || self.selected_precognition_replacement().is_some()
            || self.selected_visible_completion_suffix().is_empty()
        {
            return false;
        }
        let (Some(scope), Some(token), Some(admission)) = (
            self.context_word_scope.as_ref(),
            self.context_token.as_ref(),
            self.context_admission.as_ref(),
        ) else {
            return false;
        };
        scope.lineage().completeness == WordCompleteness::UnknownStart
            && token.matches_word_scope(scope)
            && admission.revalidate(token)
    }

    /// This engine may own every character of its visible uncommitted word.
    /// A marked client also needs the unchanged observed start receipt; the
    /// grant remains limited to this exact word and current context token.
    pub(crate) fn context_owns_active_preedit_manual_toggle(&self) -> bool {
        if self.context_handoff_sealed
            || self.atomic.active
            || self.content_is_sensitive()
            || (self.client_context.exact_surrounding_refresh_available
                && !self.marked_owned_preedit_manual_boundary_is_current())
            || self
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .is_some_and(SurroundingTextSnapshot::has_selection)
            || !self.composition.legacy_word_preedit_active
            || !self.exact_owned_legacy_preedit_is_current()
            || !self.composition.preedit_visible
            || self.composition.preedit_dirty
            || self.composition.preedit_display_only_pending
            || self.composition.buffer.is_empty()
            || self.composition.cursor != self.composition.buffer.chars().count()
            || self.selected_precognition_replacement().is_some()
        {
            return false;
        }
        let (Some(scope), Some(token), Some(admission)) = (
            self.context_word_scope.as_ref(),
            self.context_token.as_ref(),
            self.context_admission.as_ref(),
        ) else {
            return false;
        };
        // Both start proofs use this same live legacy-word owner. A confirmed
        // beginning retains the existing preedit and InputMode transaction.
        token.matches_word_scope(scope) && admission.revalidate(token)
    }

    // This proves only the unchanged client portion of an entirely owned word.
    // Actual manual handoffs retain a bounded local epoch adjustment.
    // Space/deletion retain their existing, separate start-witness checks.
    fn marked_owned_preedit_manual_boundary_is_current(&self) -> bool {
        self.composition
            .legacy_preedit_start_boundary
            .as_ref()
            .is_some_and(|witness| self.marked_owned_preedit_manual_boundary_matches(witness))
    }

    fn marked_owned_preedit_manual_boundary_matches(
        &self,
        witness: &ManagedWordStartWitness,
    ) -> bool {
        witness.exact_refresh_at_start
            && self.client_context.exact_surrounding_refresh_available
            // UnknownStart may use the exact entirely-owned birth proof only
            // after the preceding handled input has retired its paired release.
            // A still-held local input cannot stand in for a settled word start.
            && (self.context_word_is_known()
                || self.layout_gesture.handled_press_keycodes.is_empty())
            && self.client_context.managed_input
            && self.client_context.surrounding_text_supported
            && self.client_context.preedit_text_supported
            && self.composition.word_input_mode == Some(WordInputMode::ManagedCommit)
            && witness.anchored_token_chars == 0
            && witness.focus_receipt == self.client_context.focus_receipt
            && witness.focus_serial == self.client_context.focus_serial
            && witness.runtime_owner_lease_identity
                == self.client_context.runtime_owner_lease_identity
            && witness.layout_generation == self.layout_gesture.layout_generation
            && witness.armed_revision == self.client_context.surrounding_observation_revision
            && self.committed_tail.epoch
                == witness
                    .start_tail_epoch
                    .wrapping_add(self.composition.buffer.chars().count() as u64)
                    .wrapping_add(witness.manual_tail_epoch_offset)
            && self
                .committed_tail
                .buffer
                .strip_suffix(self.composition.buffer.as_str())
                == Some(witness.start_committed_tail.as_str())
            && self
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .is_some_and(|snapshot| {
                    !snapshot.has_selection()
                        && snapshot.cursor_pos == witness.start_cursor
                        && snapshot.text.strip_prefix(witness.snapshot_prefix.as_str())
                            == Some(witness.snapshot_suffix.as_str())
                })
    }

    // Only the existing verified owned-word producer calls this after its
    // successful local tail handoff. It cannot repair an intervening epoch.
    pub(crate) fn rebind_owned_preedit_manual_tail_epoch(
        &mut self,
        previous_epoch: u64,
        previous_chars: u64,
    ) {
        if let Some(witness) = self.composition.legacy_preedit_start_boundary.as_mut() {
            if witness.exact_refresh_at_start
                && witness
                    .start_tail_epoch
                    .wrapping_add(previous_chars)
                    .wrapping_add(witness.manual_tail_epoch_offset)
                    == previous_epoch
                && self.committed_tail.epoch == previous_epoch.wrapping_add(1)
            {
                witness.manual_tail_epoch_offset = self
                    .committed_tail
                    .epoch
                    .wrapping_sub(witness.start_tail_epoch)
                    .wrapping_sub(self.composition.buffer.chars().count() as u64);
            }
        }
    }

    // Called only after the existing owned-word and InputMode publication checks.
    // Rebind this manual-mode proof; do not advance its observation or tail epoch.
    pub(crate) fn rebind_owned_preedit_manual_layout(&mut self, previous_generation: u64) {
        if let Some(witness) = self.composition.legacy_preedit_start_boundary.as_mut() {
            if witness.exact_refresh_at_start && witness.layout_generation == previous_generation {
                witness.layout_generation = self.layout_gesture.layout_generation;
            }
        }
    }

    // Diagnostic metadata only, after an existing manual-mode refusal. Never
    // call this on successful ingress/publication or use it as an admission.
    pub(crate) fn trace_marked_preedit_binding_refusal(&self, stage: &str) {
        if !trace::enabled() || !self.client_context.exact_surrounding_refresh_available {
            return;
        }
        let witness = self.composition.legacy_preedit_start_boundary.as_ref();
        let snapshot = self.client_context.surrounding_text_snapshot.as_ref();
        trace::record(serde_json::json!({
            "kind": "ibus_marked_preedit_binding_refusal",
            "stage": stage,
            "word_known": self.context_word_is_known(),
            "managed_input": self.client_context.managed_input,
            "managed_word_mode": self.composition.word_input_mode == Some(WordInputMode::ManagedCommit),
            "surrounding_supported": self.client_context.surrounding_text_supported,
            "preedit_supported": self.client_context.preedit_text_supported,
            "witness_present": witness.is_some(),
            "birth_exact": witness.is_some_and(|w| w.exact_refresh_at_start),
            "zero_anchor": witness.is_some_and(|w| w.anchored_token_chars == 0),
            "focus_receipt_matches": witness.is_some_and(|w| w.focus_receipt == self.client_context.focus_receipt),
            "focus_serial_matches": witness.is_some_and(|w| w.focus_serial == self.client_context.focus_serial),
            "lease_matches": witness.is_some_and(|w| w.runtime_owner_lease_identity == self.client_context.runtime_owner_lease_identity),
            "layout_matches": witness.is_some_and(|w| w.layout_generation == self.layout_gesture.layout_generation),
            "revision_matches": witness.is_some_and(|w| w.armed_revision == self.client_context.surrounding_observation_revision),
            "handled_input_retired": self.layout_gesture.handled_press_keycodes.is_empty(),
            "tail_epoch_matches": witness.is_some_and(|w| self.committed_tail.epoch == w.start_tail_epoch.wrapping_add(self.composition.buffer.chars().count() as u64).wrapping_add(w.manual_tail_epoch_offset)),
            "birth_tail_epoch": witness.map(|w| w.start_tail_epoch),
            "manual_tail_epoch_offset": witness.map(|w| w.manual_tail_epoch_offset),
            "current_tail_epoch": self.committed_tail.epoch,
            "tail_prefix_matches": witness.is_some_and(|w| self.committed_tail.buffer.strip_suffix(self.composition.buffer.as_str()) == Some(w.start_committed_tail.as_str())),
            "snapshot_present": snapshot.is_some(),
            "cursor_matches": witness.zip(snapshot).is_some_and(|(w, s)| s.cursor_pos == w.start_cursor),
            "snapshot_prefix_matches": witness.zip(snapshot).is_some_and(|(w, s)| s.text.strip_prefix(w.snapshot_prefix.as_str()) == Some(w.snapshot_suffix.as_str())),
            "snapshot_unselected": snapshot.is_some_and(|s| !s.has_selection()),
            "birth_revision": witness.map(|w| w.armed_revision),
            "current_revision": self.client_context.surrounding_observation_revision,
            "birth_layout_generation": witness.map(|w| w.layout_generation),
            "current_layout_generation": self.layout_gesture.layout_generation,
        }).to_string());
    }

    /// An unknown beginning can support a suffix suggestion and an explicit
    /// append, without authorizing replacement of the complete word.
    pub(crate) fn context_observed_suffix_is_current(&self) -> bool {
        self.context_observed_suffix_readout_is_current()
            && (self.client_context.surrounding_text_snapshot.is_some()
                || (!self.client_context.surrounding_text_supported
                    && self.terminal_committed_tail_executor_available()))
    }

    /// Readout and explicit append do not require an executable deletion route.
    /// Whole-word consumers retain the stronger predicate above.
    pub(crate) fn context_observed_suffix_readout_is_current(&self) -> bool {
        if self.native_cursor_edit_receipt().is_some() {
            return false;
        }
        if self.context_handoff_sealed
            || self.atomic.active
            || !self.composition.buffer.is_empty()
            || self.content_is_sensitive()
            || self.committed_tail.buffer.ends_with(char::is_whitespace)
        {
            return false;
        }
        let Some(scope) = self.context_word_scope.as_ref() else {
            return false;
        };
        let token_text = self.last_tail_token_text();
        let chars = token_text.chars().count();
        if scope.lineage().completeness != WordCompleteness::UnknownStart
            || chars == 0
            // A fresh Reset receipt can prove this suffix while its replacement
            // scope still counts only later keys. Readout must not settle it.
            || (chars > scope.lineage().observed_suffix_chars as usize
                && !self.context_reset_rereceipt_exact_manual_handoff_allowed()
                && !self.native_managed_readout_is_current())
            || !self.context_token.as_ref().is_some_and(|token| {
                token.matches_word_scope(scope)
                    && self
                        .context_admission
                        .as_ref()
                        .is_some_and(|admission| admission.revalidate(token))
            })
        {
            return false;
        }
        let Some(snapshot) = self.client_context.surrounding_text_snapshot.as_ref() else {
            return self.uses_native_terminal_input() && self.client_context.cursor_cell_width > 0;
        };
        if snapshot.has_selection()
            || snapshot.suffix_before_cursor(chars).as_deref() != Some(token_text.as_str())
        {
            return false;
        }
        let start = snapshot.cursor_pos as usize - chars;
        let left_is_boundary = start == 0
            || snapshot
                .text
                .chars()
                .nth(start - 1)
                .is_some_and(crate::preedit::is_observed_word_boundary);
        let right_is_boundary = snapshot
            .text
            .chars()
            .nth(snapshot.cursor_pos as usize)
            .is_none_or(crate::preedit::is_observed_word_boundary);
        // A fragment edge permits only suffix readout/append. It does not
        // establish the absolute beginning required for whole-word edits.
        left_is_boundary && right_is_boundary
    }

    /// Automatic replacement needs the complete suffix observed in this word
    /// lineage. A shorter retained tail remains valid for display or explicit
    /// append, but is not a word-level autocorrection target.
    pub(crate) fn context_observed_suffix_is_exact_current(&self) -> bool {
        if !self.context_observed_suffix_is_current() {
            return false;
        }
        let chars = self.last_tail_token_text().chars().count();
        self.context_word_scope.as_ref().is_some_and(|scope| {
            chars > 0 && chars == scope.lineage().observed_suffix_chars as usize
        })
    }

    pub(crate) fn exact_marked_surrounding_suffix_is_current(&self) -> bool {
        if !self.client_context.exact_surrounding_refresh_available
            || !self.client_context.surrounding_text_supported
            || !self.context_observed_suffix_is_current()
        {
            return false;
        }
        self.context_observed_suffix_is_exact_current()
            || self.context_reset_rereceipt_exact_manual_handoff_allowed()
    }

    /// A managed CommitText client can outlive the callback admission that
    /// initiated the word. Its exact widget snapshot is still an independent
    /// complete-word witness when both word boundaries and the current local
    /// committed tail agree.
    pub(crate) fn exact_managed_surrounding_word_is_current(&self) -> bool {
        if !self.client_context.managed_input
            || self.composition.word_input_mode != Some(WordInputMode::ManagedCommit)
            || !self.client_context.surrounding_text_supported
            || self.atomic.active
            || !self.composition.buffer.is_empty()
            || self.content_is_sensitive()
            || self.committed_tail.buffer.ends_with(char::is_whitespace)
        {
            return false;
        }
        // A client without synchronous exact-refresh support may still send
        // a real post-CommitText receipt. It must be newer than this exact
        // local commit in the same focus and admission lineage; the old
        // snapshot cannot authorize the next character's Space edit.
        if !self.client_context.exact_surrounding_refresh_available {
            let fresh_local_receipt = self
                .client_context
                .managed_commit_snapshot_floor
                .is_some_and(|effect| {
                    effect.focus_serial == self.client_context.focus_serial
                        && effect.tail_epoch == self.committed_tail.epoch
                        && self.client_context.surrounding_observation_revision
                            > effect.observation_revision
                });
            if !fresh_local_receipt
                || self.context_handoff_sealed
                || self.live_context_token().is_none()
            {
                return false;
            }
        }
        let token = self.last_tail_token_text();
        let token_chars = token.chars().count();
        if token_chars == 0 {
            return false;
        }
        let Some(snapshot) = self.client_context.surrounding_text_snapshot.as_ref() else {
            return false;
        };
        if snapshot.has_selection()
            || snapshot.suffix_before_cursor(token_chars).as_deref() != Some(token.as_str())
        {
            return false;
        }
        let cursor = snapshot.cursor_pos as usize;
        let start = cursor - token_chars;
        let left_is_boundary = start == 0
            || snapshot
                .text
                .chars()
                .nth(start - 1)
                .is_some_and(crate::preedit::is_observed_word_boundary);
        let right_is_boundary = snapshot
            .text
            .chars()
            .nth(cursor)
            .is_none_or(crate::preedit::is_observed_word_boundary);
        left_is_boundary && right_is_boundary
    }

    /// Revalidates a locally appended managed word against the exact boundary
    /// that preceded it. The tail epoch makes delete/retype and equal-text ABA
    /// sequences fail even when their final text happens to match.
    pub(crate) fn managed_word_start_is_current(&self) -> bool {
        self.managed_word_start_invalid_reason().is_none()
    }

    pub(crate) fn managed_word_start_invalid_reason(&self) -> Option<&'static str> {
        self.managed_word_start_chain_invalid_reason(true)
    }

    fn managed_word_start_chain_invalid_reason(
        &self,
        require_exact_refresh: bool,
    ) -> Option<&'static str> {
        let Some(witness) = self.client_context.managed_word_start.as_ref() else {
            return Some("missing");
        };
        if !self.client_context.managed_input {
            return Some("unmanaged");
        }
        if self.composition.word_input_mode != Some(WordInputMode::ManagedCommit) {
            return Some("mode");
        }
        if require_exact_refresh && !self.client_context.exact_surrounding_refresh_available {
            return Some("refresh");
        }
        if !self.client_context.surrounding_text_supported {
            return Some("surrounding");
        }
        if self.atomic.active {
            return Some("atomic");
        }
        if !self.composition.buffer.is_empty() {
            return Some("composition");
        }
        if self.content_is_sensitive() {
            return Some("sensitive");
        }
        if self.committed_tail.buffer.ends_with(char::is_whitespace) {
            return Some("boundary");
        }
        if witness.focus_serial != self.client_context.focus_serial {
            return Some("focus");
        }
        if witness.runtime_owner_lease_identity != self.client_context.runtime_owner_lease_identity
        {
            return Some("owner");
        }
        if witness.layout_generation != self.layout_gesture.layout_generation {
            return Some("layout");
        }
        if self.client_context.surrounding_observation_revision < witness.armed_revision {
            return Some("revision");
        }
        let token = self.last_tail_token_text();
        let token_chars = token.chars().count() as u64;
        if token_chars == 0 {
            return Some("empty_token");
        }
        if self.committed_tail.buffer != format!("{}{}", witness.start_committed_tail, token) {
            return Some("tail_chain");
        }
        if token_chars < witness.anchored_token_chars {
            return Some("anchor_length");
        }
        if self.committed_tail.epoch
            != witness
                .start_tail_epoch
                .wrapping_add(token_chars - witness.anchored_token_chars)
        {
            return Some("epoch");
        }
        None
    }

    /// Reconstructs the exact widget state implied by the observed start
    /// boundary plus the verified local CommitText chain. This is edit
    /// authority only for the current managed-word-start identity; it is never
    /// installed as a newly observed client snapshot.
    pub(crate) fn managed_word_start_projected_snapshot(&self) -> Option<SurroundingTextSnapshot> {
        self.managed_word_start_is_current().then_some(())?;
        self.managed_word_start_preparation_snapshot()
    }

    /// Inert input material only. Native callers must separately validate the
    /// observation owner; this never makes a client snapshot or edit receipt.
    pub(crate) fn managed_word_start_preparation_identity(&self) -> Option<u64> {
        self.managed_word_start_chain_invalid_reason(false)
            .is_none()
            .then_some(())?;
        let witness = self.client_context.managed_word_start.as_ref()?;
        (!witness.exact_refresh_at_start && self.committed_tail.epoch > witness.start_tail_epoch)
            .then_some(witness.identity)
    }

    pub(crate) fn managed_word_start_preparation_snapshot(
        &self,
    ) -> Option<SurroundingTextSnapshot> {
        self.managed_word_start_chain_invalid_reason(false)
            .is_none()
            .then_some(())?;
        let witness = self.client_context.managed_word_start.as_ref()?;
        let token = self.last_tail_token_text();
        let cursor = witness
            .start_cursor
            .checked_add(u32::try_from(token.chars().count()).ok()?)?;
        Some(SurroundingTextSnapshot::new(
            format!(
                "{}{}{}",
                witness.snapshot_prefix, token, witness.snapshot_suffix
            ),
            cursor,
            cursor,
        ))
    }

    pub(crate) fn arm_managed_commit_reset_echo(&mut self) {
        if self.managed_word_start_is_current() {
            if let Some(witness) = self.client_context.managed_word_start.as_mut() {
                witness.reset_echo_epoch = Some(self.committed_tail.epoch);
            }
        }
    }

    pub(crate) fn take_managed_commit_reset_echo(&mut self) -> bool {
        if !self.managed_word_start_is_current() {
            return false;
        }
        if !self
            .committed_tail
            .last_commit_at
            .is_some_and(|at| at.elapsed() <= Duration::from_millis(700))
        {
            return false;
        }
        self.client_context
            .managed_word_start
            .as_mut()
            .and_then(|witness| witness.reset_echo_epoch.take())
            == Some(self.committed_tail.epoch)
    }

    /// Source-free callback activation can install its authenticated owner
    /// after the exact empty boundary but before the first managed CommitText.
    /// Rebind only across that zero-character gap while the focus receipt and
    /// surrounding-observation revision are unchanged. Once local input starts
    /// this transition is permanently unavailable.
    pub(crate) fn rebind_managed_word_start_before_first_commit(&mut self) {
        let Some(witness) = self.client_context.managed_word_start.as_ref() else {
            return;
        };
        if self.committed_tail.buffer != witness.start_committed_tail
            || !self.last_tail_token_text().is_empty()
        {
            return;
        }
        let unchanged_boundary = self.composition.word_input_mode
            == Some(WordInputMode::ManagedCommit)
            && self.composition.buffer.is_empty()
            && self.client_context.focus_receipt == witness.focus_receipt
            && self.client_context.surrounding_observation_revision == witness.armed_revision
            && self.client_context.managed_input
            && self.client_context.exact_surrounding_refresh_available
            && self.client_context.surrounding_text_supported
            && !self.atomic.active
            && !self.content_is_sensitive();
        if !unchanged_boundary {
            self.client_context.managed_word_start = None;
            return;
        }
        if let Some(witness) = self.client_context.managed_word_start.as_mut() {
            witness.identity = crate::engine::next_input_identity();
            witness.focus_serial = self.client_context.focus_serial;
            witness.runtime_owner_lease_identity = self.client_context.runtime_owner_lease_identity;
            witness.layout_generation = self.layout_gesture.layout_generation;
            witness.start_tail_epoch = self.committed_tail.epoch;
            witness.reset_echo_epoch = None;
        }
    }

    /// Transfer an already observed start into the composition owner before
    /// its first local key clears the widget snapshot. This never projects an
    /// uncommitted character into the managed CommitText witness.
    pub(crate) fn capture_legacy_preedit_start_boundary(&self) -> Option<ManagedWordStartWitness> {
        if self.composition.word_input_mode != Some(WordInputMode::ManagedCommit)
            || (!self.committed_tail.buffer.is_empty()
                && !self.committed_tail.buffer.ends_with(char::is_whitespace))
        {
            return None;
        }
        self.observed_word_start_from_current_snapshot("", false)
    }

    pub(crate) fn capture_space_boundary_pair_scope(
        &self,
    ) -> Option<crate::engine::SpaceBoundaryPairScope> {
        use crate::engine::{SpaceBoundaryPairProvenance, SpaceBoundaryPairScope};
        if self.content_is_sensitive() || self.context_handoff_sealed {
            return None;
        }
        let tail = &self.committed_tail.buffer;
        // Only the final two plain Cyrillic words and one observed Space
        // belong to this mechanism; no newline/punctuation/multi-space scope.
        let (prefix, right) = tail.rsplit_once([' ', '\u{a0}'])?;
        let separator = tail[prefix.len()..].chars().next()?;
        let left = prefix.rsplit(char::is_whitespace).next()?;
        if !lay::word_reader::is_cyrillic_letters_only(left)
            || !lay::word_reader::is_cyrillic_letters_only(right)
        {
            return None;
        }
        let pair = format!("{left}{separator}{right}");
        let pair_chars = pair.chars().count();
        let start = tail.chars().count().checked_sub(pair_chars)?;
        let snapshot_proves = |snapshot: &SurroundingTextSnapshot, suffix: &str| {
            let count = suffix.chars().count();
            let cursor = snapshot.cursor_pos as usize;
            !snapshot.has_selection()
                && snapshot.suffix_before_cursor(count).as_deref() == Some(suffix)
                && cursor.checked_sub(count).is_some_and(|start| {
                    start == 0
                        || snapshot
                            .text
                            .chars()
                            .nth(start - 1)
                            .is_some_and(crate::preedit::is_observed_word_boundary)
                })
                && snapshot
                    .text
                    .chars()
                    .nth(cursor)
                    .is_none_or(crate::preedit::is_observed_word_boundary)
        };
        let provenance = if let Some(witness_identity) = self.native_word_preparation_identity() {
            let snapshot = self.managed_word_start_preparation_snapshot()?;
            if !snapshot_proves(&snapshot, &pair) {
                return None;
            }
            SpaceBoundaryPairProvenance::NativePrepared {
                witness_identity,
                cursor: snapshot.cursor_pos,
            }
        } else if self.composition.legacy_word_preedit_active {
            let witness = self.composition.legacy_preedit_start_boundary.as_ref()?;
            if !self.owned_preedit_start_boundary_matches(witness)
                || self.composition.buffer != right
                || self.composition.cursor != right.chars().count()
            {
                return None;
            }
            let before_left = witness
                .snapshot_prefix
                .strip_suffix(&format!("{left}{separator}"))?;
            if (!before_left.is_empty()
                && !before_left
                    .chars()
                    .last()
                    .is_some_and(crate::preedit::is_observed_word_boundary))
                || !witness
                    .snapshot_suffix
                    .chars()
                    .next()
                    .is_none_or(crate::preedit::is_observed_word_boundary)
            {
                return None;
            }
            SpaceBoundaryPairProvenance::OwnedPreedit {
                witness_identity: witness.identity,
                cursor: witness.start_cursor,
                preedit: right.to_string(),
            }
        } else if self.uses_native_terminal_input() {
            let scope = self.context_word_scope.as_ref()?;
            let token = self.live_context_token()?;
            let floor = scope.lineage().observed_boundary_floor?;
            let observed_separator = start > floor as usize
                && tail
                    .chars()
                    .nth(start - 1)
                    .is_some_and(crate::preedit::is_observed_word_boundary);
            let observed_first_word =
                scope
                    .lineage()
                    .observed_first_word
                    .is_some_and(|(word_start, separator)| {
                        start == usize::from(word_start)
                            && start + left.chars().count() == usize::from(separator)
                    });
            if !token.matches_word_scope(scope)
                || scope.lineage().completeness != WordCompleteness::KnownStart
                || !(observed_separator || observed_first_word)
            {
                return None;
            }
            SpaceBoundaryPairProvenance::Observed {
                token,
                boundary_floor: floor,
            }
        } else {
            let cursor = if self.exact_managed_surrounding_word_is_current() {
                let snapshot = self.client_context.surrounding_text_snapshot.as_ref()?;
                if !snapshot_proves(snapshot, &pair) {
                    return None;
                }
                snapshot.cursor_pos
            } else {
                self.managed_word_start_is_current().then_some(())?;
                let witness = self.client_context.managed_word_start.as_ref()?;
                let client_prefix = pair.strip_suffix(right)?;
                let before_left = witness.snapshot_prefix.strip_suffix(client_prefix)?;
                if (!before_left.is_empty()
                    && !before_left
                        .chars()
                        .last()
                        .is_some_and(crate::preedit::is_observed_word_boundary))
                    || !witness
                        .snapshot_suffix
                        .chars()
                        .next()
                        .is_none_or(crate::preedit::is_observed_word_boundary)
                {
                    return None;
                }
                witness
                    .start_cursor
                    .checked_add(u32::try_from(right.chars().count()).ok()?)?
            };
            SpaceBoundaryPairProvenance::Client {
                revision: self.client_context.surrounding_observation_revision,
                cursor,
            }
        };
        Some(SpaceBoundaryPairScope {
            text: pair,
            provenance,
        })
    }

    /// Reconstruct at dispatch only. Pair identities keep bounded text and
    /// existing revision/witness IDs instead of cloning a whole client field
    /// on every printable key.
    pub(crate) fn space_pair_client_snapshot(
        &self,
        scope: &crate::engine::SpaceBoundaryPairScope,
    ) -> Option<SurroundingTextSnapshot> {
        if self.capture_space_boundary_pair_scope().as_ref() != Some(scope) {
            return None;
        }
        match &scope.provenance {
            crate::engine::SpaceBoundaryPairProvenance::NativePrepared {
                witness_identity,
                cursor,
            } => self.native_word_preparation_pair_snapshot(*witness_identity, *cursor),
            crate::engine::SpaceBoundaryPairProvenance::Client { .. } => {
                if self.exact_managed_surrounding_word_is_current() {
                    self.client_context.surrounding_text_snapshot.clone()
                } else {
                    self.managed_word_start_projected_snapshot()
                }
            }
            crate::engine::SpaceBoundaryPairProvenance::OwnedPreedit { .. } => {
                let witness = self.composition.legacy_preedit_start_boundary.as_ref()?;
                Some(SurroundingTextSnapshot::new(
                    format!("{}{}", witness.snapshot_prefix, witness.snapshot_suffix),
                    witness.start_cursor,
                    witness.start_cursor,
                ))
            }
            crate::engine::SpaceBoundaryPairProvenance::Observed { .. } => None,
        }
    }

    fn owned_preedit_start_boundary_is_current(&self) -> bool {
        let Some(witness) = self.composition.legacy_preedit_start_boundary.as_ref() else {
            return false;
        };
        self.owned_preedit_start_boundary_matches(witness)
    }

    /// A post-commit expectation is not a pre-dispatch field observation.
    /// Preserve an observed old prefix exactly. With no initial observation,
    /// only an exact whole-field receipt can confirm the owned word commit.
    pub(crate) fn owned_preedit_space_commit_expectation(
        &self,
        text: &str,
        verified_edit: Option<&lay::text_edit::AuthorizedEdit>,
    ) -> Option<SurroundingTextSnapshot> {
        let word = text.strip_suffix(' ')?;
        let text_is_authorized = match verified_edit {
            Some(edit) => {
                edit.backend() == lay::text_edit::TextEditBackend::Ime
                    && edit.action().from_text() == self.composition.buffer
                    && edit.action().to_text() == text
            }
            None => word == self.composition.buffer,
        };
        if self.context_handoff_sealed
            || self.atomic.active
            || self.content_is_sensitive()
            || !text_is_authorized
            || word.is_empty()
            || word.chars().any(char::is_whitespace)
            || self.composition.cursor != self.composition.buffer.chars().count()
        {
            return None;
        }
        let expected = match (
            self.composition.legacy_preedit_start_boundary.as_ref(),
            self.client_context.surrounding_text_snapshot.as_ref(),
        ) {
            (Some(witness), Some(before)) => {
                if !self.owned_preedit_start_boundary_is_current()
                    || !witness.snapshot_suffix.is_empty()
                    || before.text != witness.snapshot_prefix
                    || before.cursor_pos != witness.start_cursor
                    || before.has_selection()
                    || before.cursor_pos as usize != before.text.chars().count()
                {
                    return None;
                }
                format!("{}{text}", witness.snapshot_prefix)
            }
            (None, None) => {
                // Do not infer an empty field or a KnownStart. This candidate
                // can only be acknowledged if the *entire* later client field
                // equals this owner's emitted word and one boundary. A known
                // but stale/incomplete start must never enter this branch.
                if self.client_context.exact_surrounding_refresh_available
                    || !self.client_context.managed_input
                    || !self.client_context.surrounding_text_supported
                    || !self.client_context.preedit_text_supported
                    || self.composition.word_input_mode != Some(WordInputMode::ManagedCommit)
                    || !self.exact_owned_legacy_preedit_is_current()
                    || self.committed_tail.buffer != self.composition.buffer
                    || self.live_context_token().is_none()
                {
                    return None;
                }
                text.to_string()
            }
            _ => return None,
        };
        let cursor = u32::try_from(expected.chars().count()).ok()?;
        Some(SurroundingTextSnapshot::new(expected, cursor, cursor))
    }

    fn owned_preedit_start_boundary_matches(&self, witness: &ManagedWordStartWitness) -> bool {
        let word = &self.composition.buffer;
        let chars = word.chars().count() as u64;
        self.client_context.managed_input
            && self.composition.word_input_mode == Some(WordInputMode::ManagedCommit)
            && self.client_context.surrounding_text_supported
            && self.composition.legacy_word_preedit_active
            && chars > 0
            && witness.anchored_token_chars == 0
            && witness.focus_receipt == self.client_context.focus_receipt
            && witness.focus_serial == self.client_context.focus_serial
            && witness.runtime_owner_lease_identity
                == self.client_context.runtime_owner_lease_identity
            && witness.layout_generation == self.layout_gesture.layout_generation
            && witness.armed_revision == self.client_context.surrounding_observation_revision
            && self.committed_tail.epoch == witness.start_tail_epoch.wrapping_add(chars)
            && self.committed_tail.buffer == format!("{}{}", witness.start_committed_tail, word)
    }

    /// A local end-caret Backspace may shorten this owner's uncommitted word.
    /// Capture only the already current boundary and token; a client deletion
    /// or stale input chain cannot manufacture a new observed start.
    pub(crate) fn prepare_owned_preedit_local_backspace(
        &self,
    ) -> Option<(ManagedWordStartWitness, String, u64)> {
        // A current observed start already bounds this uncommitted segment.
        // Local erasure does not need a client capability for a fresh receipt.
        // Pending suffix display does not change ownership of a local erase.
        // This retains inert start evidence; the manual producer still checks
        // all publication guards before a later mode operation.
        let marked_manual_word = self.marked_owned_preedit_manual_boundary_is_current();
        if (!self.owned_preedit_start_boundary_is_current() && !marked_manual_word)
            || !self.exact_owned_legacy_preedit_is_current()
            || self.context_handoff_sealed
            || self.atomic.active
            || self.content_is_sensitive()
            || self.composition.buffer.is_empty()
            || self.composition.cursor != self.composition.buffer.chars().count()
            || self
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .is_some_and(SurroundingTextSnapshot::has_selection)
        {
            return None;
        }
        let scope = self.context_word_scope.as_ref()?;
        let token = self.live_context_token()?;
        if (scope.lineage().completeness != WordCompleteness::UnknownStart && !marked_manual_word)
            || !token.matches_word_scope(scope)
        {
            return None;
        }
        Some((
            self.composition.legacy_preedit_start_boundary.clone()?,
            self.composition.buffer.clone(),
            self.committed_tail.epoch,
        ))
    }

    pub(crate) fn finish_owned_preedit_local_backspace(
        &mut self,
        receipt: Option<(ManagedWordStartWitness, String, u64)>,
    ) {
        let Some((mut witness, mut before, before_epoch)) = receipt else {
            return;
        };
        before.pop();
        if self.composition.buffer != before
            || before.is_empty()
            || self.composition.cursor != before.chars().count()
            || self.committed_tail.epoch != before_epoch.wrapping_add(1)
        {
            return;
        }
        witness.start_tail_epoch = self
            .committed_tail
            .epoch
            .wrapping_sub(before.chars().count() as u64);
        witness.manual_tail_epoch_offset = 0;
        let Some(scope) = self.context_word_scope.as_ref() else {
            return;
        };
        let Some(token) = self.live_context_token() else {
            return;
        };
        if self.owned_preedit_start_boundary_matches(&witness)
            && (scope.lineage().completeness == WordCompleteness::UnknownStart
                || self.marked_owned_preedit_manual_boundary_matches(&witness))
            && token.matches_word_scope(scope)
        {
            self.composition.legacy_preedit_start_boundary = Some(witness);
        }
    }

    fn arm_managed_word_start_from_current_snapshot(&mut self) {
        if let Some(witness) =
            self.managed_word_start_from_current_snapshot(&self.last_tail_token_text())
        {
            self.client_context.managed_word_start = Some(witness);
        }
    }

    fn managed_word_start_from_current_snapshot(
        &self,
        token: &str,
    ) -> Option<ManagedWordStartWitness> {
        self.observed_word_start_from_current_snapshot(token, !self.native_document_is_current())
    }

    fn observed_word_start_from_current_snapshot(
        &self,
        token: &str,
        require_exact_refresh: bool,
    ) -> Option<ManagedWordStartWitness> {
        if !self.client_context.managed_input
            || (require_exact_refresh && !self.client_context.exact_surrounding_refresh_available)
            || !self.client_context.surrounding_text_supported
            || self.context_handoff_sealed
            || self.atomic.active
            || !self.composition.buffer.is_empty()
            || self.content_is_sensitive()
        {
            return None;
        }
        let snapshot = self.client_context.surrounding_text_snapshot.as_ref()?;
        let text_chars = snapshot.text.chars().collect::<Vec<_>>();
        let cursor = snapshot.cursor_pos as usize;
        if snapshot.has_selection() || cursor > text_chars.len() {
            return None;
        }
        let token_chars = token.chars().count();
        let start = cursor.checked_sub(token_chars)?;
        let left_is_boundary = start == 0
            || text_chars
                .get(start - 1)
                .copied()
                .is_some_and(crate::preedit::is_observed_word_boundary);
        let right_is_boundary = text_chars
            .get(cursor)
            .copied()
            .is_none_or(crate::preedit::is_observed_word_boundary);
        let retained_tail_chars = self.committed_tail.buffer.chars().count();
        if !left_is_boundary
            || !right_is_boundary
            || (token_chars > 0
                && self.composition.word_input_mode != Some(WordInputMode::ManagedCommit))
            || snapshot
                .suffix_before_cursor(retained_tail_chars)
                .as_deref()
                != Some(self.committed_tail.buffer.as_str())
        {
            return None;
        }
        let start_committed_tail = self.committed_tail.buffer.strip_suffix(token)?;
        Some(ManagedWordStartWitness {
            identity: crate::engine::next_input_identity(),
            focus_receipt: self.client_context.focus_receipt.clone(),
            focus_serial: self.client_context.focus_serial,
            runtime_owner_lease_identity: self.client_context.runtime_owner_lease_identity,
            layout_generation: self.layout_gesture.layout_generation,
            exact_refresh_at_start: self.client_context.exact_surrounding_refresh_available,
            start_tail_epoch: self.committed_tail.epoch,
            manual_tail_epoch_offset: 0,
            anchored_token_chars: token_chars as u64,
            start_committed_tail: start_committed_tail.to_string(),
            snapshot_prefix: text_chars[..start].iter().collect(),
            snapshot_suffix: text_chars[cursor..].iter().collect(),
            start_cursor: start as u32,
            armed_revision: self.client_context.surrounding_observation_revision,
            reset_echo_epoch: None,
            closed_word_predecessor: (token_chars == 0
                && self.context_reset_rereceipt_exact_manual_handoff_allowed()
                && self.committed_tail.buffer.ends_with(char::is_whitespace))
            .then(|| self.capture_owned_context_reset_rereceipt_candidate())
            .flatten()
            .map(|mut predecessor| {
                predecessor.owned_preedit_return_snapshot =
                    self.client_context.surrounding_text_snapshot.clone();
                Box::new(predecessor)
            }),
        })
    }

    fn current_snapshot_matches_managed_word_start(&self) -> bool {
        let Some(witness) = self.client_context.managed_word_start.as_ref() else {
            return false;
        };
        let Some(snapshot) = self.client_context.surrounding_text_snapshot.as_ref() else {
            return false;
        };
        let native_preparation = self.native_word_preparation_identity().is_some();
        if snapshot.has_selection() || !(self.managed_word_start_is_current() || native_preparation)
        {
            return false;
        }
        let token = self.last_tail_token_text();
        let expected_prefix = format!("{}{}", witness.snapshot_prefix, token);
        let expected_cursor = witness
            .start_cursor
            .saturating_add(u32::try_from(token.chars().count()).unwrap_or(u32::MAX));
        let cursor = snapshot.cursor_pos as usize;
        let text_chars = snapshot.text.chars().collect::<Vec<_>>();
        if cursor > text_chars.len() {
            return false;
        }
        let actual_prefix = text_chars[..cursor].iter().collect::<String>();
        let actual_suffix = text_chars[cursor..].iter().collect::<String>();
        if snapshot.cursor_pos == expected_cursor && actual_prefix == expected_prefix {
            return if native_preparation {
                actual_suffix == witness.snapshot_suffix
            } else {
                actual_suffix.ends_with(&witness.snapshot_suffix)
            };
        }
        // A native read may still expose an earlier prefix of our exact local
        // append chain. Retain data only; the full actual-snapshot equality at
        // edit dispatch remains false until every committed character arrives.
        let earlier_prefix = actual_prefix.strip_prefix(witness.snapshot_prefix.as_str());
        let retained = native_preparation
            && actual_suffix == witness.snapshot_suffix
            && earlier_prefix.is_some_and(|observed| {
                token.starts_with(observed)
                    && observed.chars().count() as u64 >= witness.anchored_token_chars
                    && observed.chars().count() < token.chars().count()
            });
        if retained && crate::trace::enabled() {
            crate::trace::record(
                serde_json::json!({
                    "kind": "ibus_native_preparation_progress",
                    "observed_word_chars": earlier_prefix.map(|word| word.chars().count()),
                    "prepared_word_chars": token.chars().count(),
                    "tail_epoch": self.committed_tail.epoch,
                    "edit_authority": false,
                })
                .to_string(),
            );
        }
        retained
    }

    fn reconcile_managed_word_start_after_surrounding_observation(&mut self) {
        if self.client_context.managed_word_start.is_some() {
            if self.current_snapshot_matches_managed_word_start() {
                if let Some(witness) = self.client_context.managed_word_start.as_mut() {
                    witness.armed_revision = self.client_context.surrounding_observation_revision;
                }
                return;
            }
            self.client_context.managed_word_start = None;
        }
        self.arm_managed_word_start_from_current_snapshot();
    }

    pub(crate) fn context_observed_suffix_exact_manual_handoff_allowed(&self) -> bool {
        if !self.client_context.surrounding_text_supported
            || !self.context_observed_suffix_is_current()
        {
            return false;
        }
        let Some(scope) = self.context_word_scope.as_ref() else {
            return false;
        };
        let chars = self.last_tail_token_text().chars().count();
        chars > 0 && chars == scope.lineage().observed_suffix_chars as usize
    }

    pub(crate) fn context_exact_manual_handoff_bounds_unknown_suffix(&self) -> bool {
        if self.context_handoff_sealed
            || self.atomic.active
            || !self.composition.buffer.is_empty()
            || self.content_is_sensitive()
        {
            return false;
        }
        let Some(scope) = self.context_word_scope.as_ref() else {
            return false;
        };
        let chars = self.last_tail_with_boundary().chars().count();
        if scope.lineage().completeness != WordCompleteness::UnknownStart
            || chars == 0
            || chars != scope.lineage().observed_suffix_chars as usize
            || !self.context_token.as_ref().is_some_and(|token| {
                token.matches_word_scope(scope)
                    && self
                        .context_admission
                        .as_ref()
                        .is_some_and(|admission| admission.revalidate(token))
            })
        {
            return false;
        }
        self.exact_manual_toggle_handoff_is_live()
    }

    /// Unknown word beginnings do not authorize generic edits.
    /// A terminal can still project its complete observed suffix on an explicit
    /// manual request, using the same settled context and erase geometry.
    pub(crate) fn context_allows_manual_toggle(&self) -> bool {
        if self.context_word_is_known() {
            return true;
        }
        if self.native_manual_word_is_current() {
            return true;
        }
        if self.context_handoff_sealed
            || !self.composition.buffer.is_empty()
            || self.content_is_sensitive()
            || !self.terminal_committed_tail_executor_available()
        {
            return false;
        }
        let Some(scope) = self.context_word_scope.as_ref() else {
            return false;
        };
        let suffix_chars = self.last_tail_token_text().chars().count();
        suffix_chars > 0
            && suffix_chars <= scope.lineage().observed_suffix_chars as usize
            && self.context_token.as_ref().is_some_and(|token| {
                token.matches_word_scope(scope)
                    && self
                        .context_admission
                        .as_ref()
                        .is_some_and(|admission| admission.revalidate(token))
            })
    }

    pub(crate) fn live_context_token(&self) -> Option<AdmissionToken> {
        let admission = self.context_admission.as_ref()?;
        let token = self.context_token.as_ref()?;
        admission.revalidate(token).then(|| token.clone())
    }

    pub(crate) fn live_layout_intent_token(&self) -> Option<LayoutIntentToken> {
        let admission = self.context_admission.as_ref()?;
        let owner = self.context_owner.as_ref()?;
        admission.current_layout_intent_token_for(owner)
    }

    pub(crate) async fn activate_context_from_header(
        &mut self,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
        native_context: Option<&str>,
    ) -> bool {
        self.observe_context_activation_from_header(header, callback_entered, native_context)
            .await
            == ContextActivationObservation::Activated
    }

    async fn observe_context_activation_from_header(
        &mut self,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
        native_context: Option<&str>,
    ) -> ContextActivationObservation {
        let checked_compatibility_focus = native_context.is_none()
            && self.config.active_layout_backend() == lay::desktop::LayoutBackend::Kde;
        let Some(admission) = self.context_admission.clone() else {
            if self.context_admission_required {
                self.fail_context_activation();
            }
            return ContextActivationObservation::Refused;
        };
        let Some(target_path) = EnginePath::new(self.path.clone()) else {
            self.fail_context_activation();
            return ContextActivationObservation::Refused;
        };
        let observed = match admission.observe_callback(header, callback_entered).await {
            Ok(observed) => observed,
            // Missing/evicted metadata cannot identify a failed acquisition.
            // In particular, it cannot revoke the successor now on this engine.
            Err(_) if checked_compatibility_focus => {
                return ContextActivationObservation::Ignored;
            }
            Err(_) => {
                self.fail_context_activation();
                return ContextActivationObservation::Refused;
            }
        };
        if native_context.is_some() {
            self.try_install_pending_context_activation();
        }
        if let (Some(owner), Some(context_path)) = (self.context_owner.as_ref(), native_context) {
            let Some(context) = admission.context_key(context_path.to_string()) else {
                self.fail_context_activation();
                return ContextActivationObservation::Refused;
            };
            if let Some(enriched) =
                admission.enrich_native_activation(owner, &target_path, &context, &observed)
            {
                return if enriched {
                    self.context_token = admission.current_token();
                    ContextActivationObservation::Activated
                } else {
                    self.fail_context_activation();
                    ContextActivationObservation::Refused
                };
            }
        }
        if let Some(context_path) = native_context {
            let Some(context) = admission.context_key(context_path.to_string()) else {
                self.fail_context_activation();
                return ContextActivationObservation::Refused;
            };
            if admission.enrich_pending_native_activation(&target_path, context, &observed) {
                self.fail_context_activation_local();
                return ContextActivationObservation::Activated;
            }
        }
        let started = if let Some(context_path) = native_context {
            admission
                .context_key(context_path.to_string())
                .ok_or(())
                .and_then(|context| {
                    admission
                        .start_native_activation(target_path, context, observed.position)
                        .map_err(|_| ())
                })
        } else {
            let backend = self.config.active_layout_backend();
            if backend == lay::desktop::LayoutBackend::Kde {
                match admission.start_compatibility_activation_with_deadline(
                    target_path,
                    observed.position,
                    Self::kde_chrome_activation_deadline(backend),
                ) {
                    // A queued old focus is metadata, not a new activation or
                    // a reason to revoke the later, already pending request.
                    Err(crate::context_admission::AdapterError::SupersededFocus) => {
                        return ContextActivationObservation::Ignored;
                    }
                    result => result,
                }
            } else {
                admission.start_compatibility_activation(target_path, observed.position)
            }
            .map_err(|_| ())
        };
        if started.is_err() {
            self.fail_context_activation();
            return ContextActivationObservation::Refused;
        }
        // The adapter owns the one-shot Get/marker future. Its completion task
        // and normal callbacks consume the same target-bound ready slot. Until
        // installation, input follows the literal UnknownStart route.
        self.fail_context_activation_local();
        ContextActivationObservation::Activated
    }

    pub(crate) async fn deliver_ready_context_activation(
        connection: &zbus::Connection,
        admission: &ContextAdmissionAdapter,
        target_path: &EnginePath,
        request: RequestGeneration,
        fence: PendingFence,
    ) {
        let Ok(interface) = connection
            .object_server()
            .interface::<_, LayIbusEngine>(target_path.as_str())
            .await
        else {
            return;
        };
        let mut engine = interface.get_mut().await;
        if engine.path != target_path.as_str()
            || engine.atomic.active
            || engine
                .context_admission
                .as_ref()
                .is_none_or(|current| !current.same_instance(admission))
        {
            return;
        }
        // Obtain the full outcome only AFTER both interface awaits. A Reset
        // may have replaced it; a callback may have consumed it; a successor
        // request on this path must never be installed by the old task.
        let Some(outcome) = admission.pending_activation_for_request(target_path, request, fence)
        else {
            return;
        };
        if !engine.install_ready_context_activation(outcome) {
            return;
        }
        // No native Get/marker RPC or adapter mutex is held here. Only the
        // existing guarded metadata signal publisher awaits under this guard.
        let mut output = EngineOutput::legacy(interface.signal_emitter());
        engine
            .publish_pending_input_mode_property(&mut output)
            .await;
    }

    pub(crate) fn try_install_pending_context_activation(&mut self) {
        let Some(admission) = self.context_admission.clone() else {
            return;
        };
        let Some(target_path) = EnginePath::new(self.path.clone()) else {
            self.fail_context_activation();
            return;
        };
        if let Some(outcome) = admission.pending_activation_for(&target_path) {
            self.install_ready_context_activation(outcome);
        }
    }

    fn pending_literal_tail_receipt(
        &self,
        outcome: &ActivationOutcome,
    ) -> Option<crate::context_admission::PendingLiteralInputReceipt> {
        let pending = self.composition.pending_activation_preedit.as_ref()?;
        let composition_matches = if self.composition.buffer.is_empty() {
            !self.composition.legacy_word_preedit_active
        } else {
            self.composition.legacy_word_preedit_active
                && self.composition.preedit_visible
                && self.composition.cursor <= self.composition.buffer.chars().count()
                && self
                    .committed_tail
                    .buffer
                    .ends_with(&self.composition.buffer)
        };
        (self.context_owner.is_none()
            && !self.atomic.active
            && !self.content_is_sensitive()
            && self.client_context.preedit_text_supported
            && pending.focus_serial == self.client_context.focus_serial
            && pending.observed_tail_epoch == Some(self.committed_tail.epoch)
            && composition_matches
            && self.committed_tail.buffer.chars().count() <= crate::preedit::PREEDIT_TAIL_LIMIT
            && self
                .committed_tail
                .buffer
                .chars()
                .any(crate::preedit::is_observed_word_boundary)
            && self.context_admission.as_ref().is_some_and(|admission| {
                admission.activation_preserves_literal_input(&pending.receipt, outcome)
            }))
        .then(|| pending.receipt.clone())
    }

    fn install_ready_context_activation(&mut self, outcome: ActivationOutcome) -> bool {
        let Some(admission) = self.context_admission.clone() else {
            return false;
        };
        let literal_receipt = self.pending_literal_tail_receipt(&outcome);
        if !self.install_context_activation(outcome.clone()) {
            return false;
        }
        if let Some(receipt) = literal_receipt {
            let Some((scope, token)) = admission.acknowledge_activation_with_literal_input(
                &outcome,
                &receipt,
                &self.committed_tail.buffer,
                self.committed_tail.epoch,
                (
                    self.client_context.content_purpose,
                    self.client_context.content_hints,
                ),
            ) else {
                self.discard_context_activation();
                return false;
            };
            self.context_word_scope = Some(scope);
            self.context_token = Some(token);
        } else if !admission.acknowledge_activation(&outcome) {
            self.discard_context_activation();
            return false;
        }
        true
    }

    pub(crate) fn install_context_activation(&mut self, outcome: ActivationOutcome) -> bool {
        let before_focus_serial = self.client_context.focus_serial;
        // Keep this exact grant token through installation. A revocation racing
        // with publication must not be replaced by a newer unrelated token.
        let token = self
            .context_admission
            .as_ref()
            .and_then(|admission| admission.activation_outcome_token(&outcome));
        if self.context_admission_required && token.is_none() {
            self.discard_context_activation();
            return false;
        }
        let preserve_literal_tail = self.pending_literal_tail_receipt(&outcome).is_some();
        let preserve_literal_preedit = self
            .composition
            .pending_activation_preedit
            .take()
            .is_some_and(|pending| {
                self.context_owner.is_none()
                    && !self.atomic.active
                    && !self.content_is_sensitive()
                    && self.client_context.preedit_text_supported
                    && self.client_context.focus_serial == pending.focus_serial
                    && self.composition.legacy_word_preedit_active
                    && self.composition.preedit_visible
                    && !self.composition.buffer.is_empty()
                    && self.composition.cursor <= self.composition.buffer.chars().count()
                    && (preserve_literal_tail
                        || self.committed_tail.buffer == self.composition.buffer)
                    && self.context_admission.as_ref().is_some_and(|admission| {
                        admission.activation_preserves_literal_input(&pending.receipt, &outcome)
                    })
            });
        let installed_outcome = outcome.clone();
        let transferred_exact_snapshot = match &outcome {
            ActivationOutcome::Transfer(grant) => grant.exact_manual_snapshot.as_deref().cloned(),
            ActivationOutcome::SourceFree(_) | ActivationOutcome::ResetUnknown(_) => None,
        };
        let reset_unknown = matches!(&outcome, ActivationOutcome::ResetUnknown(_));
        let next_owner_lease_identity = next_input_identity();
        let (owner, activation_generation, lineage, transferred, source_free_tail_epoch, kind) =
            match outcome {
                ActivationOutcome::Transfer(grant) => {
                    if grant.target_owner.path.as_str() != self.path {
                        self.fail_context_activation();
                        return false;
                    }
                    let transferred = {
                        let Ok(mut shared) = self.shared.lock() else {
                            self.fail_context_activation();
                            return false;
                        };
                        if shared.active_path.as_deref() != Some(grant.source_owner.path.as_str())
                            || shared.context_owner_generation.is_some_and(|generation| {
                                generation != grant.source_owner.generation.0
                            })
                            || shared.handoff_tail_epoch != grant.source_tail_epoch
                        {
                            None
                        } else {
                            let preserve_exact_manual_handoff = shared
                                .preserve_active_path_until
                                .is_some_and(|until| Instant::now() <= until)
                                && shared.context_owner_generation
                                    == Some(grant.source_owner.generation.0)
                                && shared.exact_manual_toggle_handoff_epoch
                                    == Some(grant.source_tail_epoch)
                                && shared.exact_manual_toggle_handoff_path.as_deref()
                                    == Some(grant.source_owner.path.as_str())
                                && shared.handoff_tail_epoch == grant.source_tail_epoch;
                            let suppression = match shared.autocorrect_suppression.clone() {
                                Some(AutocorrectSuppression::CurrentWord(mut current)) => {
                                    current.owner_lease_identity = next_owner_lease_identity;
                                    let rebound = AutocorrectSuppression::CurrentWord(current);
                                    shared.autocorrect_suppression = Some(rebound.clone());
                                    shared.suppression_revision =
                                        shared.suppression_revision.wrapping_add(1);
                                    Some(rebound)
                                }
                                Some(AutocorrectSuppression::LegacyReplayV1) => {
                                    // Transfer already authenticates the live
                                    // source owner, field and tail epoch. V1
                                    // daemon replay has no exact-V2 snapshot
                                    // handoff; requiring one drops its guard
                                    // before the replay's trailing Space.
                                    Some(AutocorrectSuppression::LegacyReplayV1)
                                }
                                _ => {
                                    if shared.autocorrect_suppression.take().is_some() {
                                        shared.suppression_revision =
                                            shared.suppression_revision.wrapping_add(1);
                                    }
                                    None
                                }
                            };
                            shared.active_path = Some(self.path.clone());
                            shared.context_owner_generation = Some(grant.target_owner.generation.0);
                            if !preserve_exact_manual_handoff {
                                shared.preserve_active_path_until = None;
                                shared.exact_manual_toggle_handoff_epoch = None;
                                shared.exact_manual_toggle_handoff_path = None;
                            }
                            Some((
                                shared.handoff_tail_buffer.clone(),
                                shared.handoff_tail_epoch,
                                suppression,
                            ))
                        }
                    };
                    let Some(transferred) = transferred else {
                        self.fail_context_activation();
                        return false;
                    };
                    (
                        grant.target_owner,
                        grant.target_activation.generation.0,
                        grant.lineage,
                        Some(transferred),
                        None,
                        "transfer_installed",
                    )
                }
                ActivationOutcome::SourceFree(grant) | ActivationOutcome::ResetUnknown(grant) => {
                    if grant.target_owner.path.as_str() != self.path {
                        self.fail_context_activation();
                        return false;
                    }
                    if self.context_admission.as_ref().is_none_or(|admission| {
                        admission.current_owner().as_ref() != Some(&grant.target_owner)
                    }) {
                        self.fail_context_activation();
                        return false;
                    }
                    let source_free_tail_epoch = self.shared.lock().ok().and_then(|mut shared| {
                        if shared
                            .context_owner_generation
                            .is_some_and(|generation| generation > grant.target_owner.generation.0)
                        {
                            None
                        } else {
                            let tail_epoch = if reset_unknown {
                                let next_epoch = shared
                                    .handoff_tail_epoch
                                    .max(self.committed_tail.epoch)
                                    .max(grant.tail_epoch)
                                    .checked_add(1)?;
                                if !self.context_admission.as_ref().is_some_and(|admission| {
                                    admission.bind_reset_tail_epoch(&grant, next_epoch)
                                }) {
                                    return None;
                                }
                                next_epoch
                            } else if grant.tail_epoch > 0 {
                                grant.tail_epoch
                            } else {
                                let next_epoch = shared.handoff_tail_epoch.checked_add(1)?;
                                if !self.context_admission.as_ref().is_some_and(|admission| {
                                    admission.bind_source_free_tail_epoch(&grant, next_epoch)
                                }) {
                                    return None;
                                }
                                next_epoch
                            };
                            shared.active_path = Some(self.path.clone());
                            shared.context_owner_generation = Some(grant.target_owner.generation.0);
                            shared.handoff_tail_buffer.clear();
                            shared.handoff_tail_epoch = tail_epoch;
                            shared.handoff_focus_receipt = None;
                            shared.preserve_active_path_until = None;
                            shared.exact_manual_toggle_handoff_epoch = None;
                            shared.exact_manual_toggle_handoff_path = None;
                            // Ready replaces lexical ownership, but a queued
                            // authorized replay may still have native keys in
                            // flight. Retain only its exact delivery classifier;
                            // no old word, prefix observation or token transfers.
                            let delivery = match shared.autocorrect_suppression.as_ref() {
                                Some(AutocorrectSuppression::ExactReplay(scope))
                                    if self.exact_replay_delivery_is_current(scope) =>
                                {
                                    let mut next = scope.clone();
                                    next.delivery_lease_identity = Some(next_owner_lease_identity);
                                    next.source_token = None;
                                    next.observed_external_prefix = None;
                                    Some(AutocorrectSuppression::ExactReplay(next))
                                }
                                _ => None,
                            };
                            if shared.autocorrect_suppression != delivery {
                                shared.autocorrect_suppression = delivery;
                                shared.suppression_revision =
                                    shared.suppression_revision.wrapping_add(1);
                            }
                            Some(tail_epoch)
                        }
                    });
                    let Some(source_free_tail_epoch) = source_free_tail_epoch else {
                        self.discard_context_activation();
                        return false;
                    };
                    (
                        grant.target_owner,
                        grant.target_activation.generation.0,
                        grant.lineage,
                        None,
                        Some(source_free_tail_epoch),
                        if reset_unknown {
                            "reset_unknown_installed"
                        } else {
                            "source_free_installed"
                        },
                    )
                }
            };

        self.invalidate_input_frame_background_work();
        self.context_reset_rereceipt = None;
        self.client_context.focus_serial = next_input_identity();
        self.client_context.runtime_owner_lease_identity = next_owner_lease_identity;
        if !preserve_literal_preedit {
            self.composition.buffer.clear();
            self.composition.cursor = 0;
            self.composition.legacy_word_preedit_active = false;
        }
        self.clear_preedit_completion_state();
        self.composition.pending_passthrough_preedit_clear = false;
        if let Some((tail, epoch, suppression)) = transferred {
            self.committed_tail.buffer = tail;
            self.committed_tail.epoch = epoch;
            self.committed_tail.autocorrect_suppression = suppression;
        } else {
            // No old client text is imported. A complete literal span belongs
            // only to this exact acquisition; its observed separators acquire
            // authority in the reducer when the same Ready is acknowledged.
            if !preserve_literal_tail {
                self.committed_tail.buffer = if preserve_literal_preedit {
                    self.composition.buffer.clone()
                } else {
                    String::new()
                };
            }
            self.committed_tail.epoch =
                source_free_tail_epoch.expect("source-free activation owns an empty tail epoch");
            self.committed_tail.autocorrect_suppression = None;
        }
        self.committed_tail.pending_completion_learning = None;
        self.composition.word_input_mode = None;
        self.rebuild_preedit_fast_from_tail();
        let owner_generation = owner.generation.0;
        self.context_owner = Some(owner);
        self.context_word_scope = Some(WordScope::new(lineage));
        self.context_token = token;
        if let Some((purpose, hints)) = self
            .context_admission
            .as_ref()
            .and_then(|admission| admission.admitted_content_type(self.context_token.as_ref()?))
        {
            // The same admitted word carries its settled field metadata across
            // a factory handoff. Apply it before publishing imported text, so
            // a fresh engine's defaults cannot make private text public.
            self.client_context.content_purpose = purpose;
            self.client_context.content_hints = hints;
        }
        self.exact_manual_target_snapshot = transferred_exact_snapshot.and_then(|source| {
            let target_token = self.context_token.clone()?;
            if source.tail_epoch != self.committed_tail.epoch
                || source.tail != self.committed_tail.buffer
                || source.expires_at <= Instant::now()
                || !self.client_context.surrounding_text_supported
                || self.content_is_sensitive()
            {
                return None;
            }
            if self.client_context.surrounding_text_callback_observed {
                // Actual target SSI may precede activation installation. Bind
                // that observation in the same reducer rather than inheriting
                // an absent-callback receipt or granting word completeness.
                let prefix = self.current_full_external_prefix()?;
                if self.current_external_snapshot_agrees_with_owned_tail()
                    && source.observed_external_prefix.as_deref() == Some(prefix.as_str())
                {
                    let _ = self
                        .context_admission
                        .as_ref()?
                        .publish_exact_manual_snapshot(
                            &target_token,
                            source.tail_epoch,
                            source.tail,
                            source.observed_external_prefix,
                            self.client_context.surrounding_observation_revision,
                            source.expires_at,
                            crate::context_admission::ExactManualSourceProvenance::ClientSnapshot,
                        );
                }
                return None;
            }
            Some(crate::engine::ExactManualTargetSnapshotReceipt {
                source,
                target_token,
                target_epoch: self.committed_tail.epoch,
                target_observation_revision: self.client_context.surrounding_observation_revision,
            })
        });
        self.context_handoff_sealed = false;
        if self.context_admission_required
            && self.context_admission.as_ref().is_none_or(|admission| {
                !admission.activation_outcome_is_current(&installed_outcome)
            })
        {
            self.discard_context_activation();
            return false;
        }
        self.install_pending_hint_geometry(before_focus_serial, &installed_outcome);
        self.client_context.input_mode_property_refresh_pending = true;
        trace::record_context_admission(
            "activation_install",
            "",
            0,
            kind,
            Some(owner_generation),
            Some(activation_generation),
            Some(match lineage.completeness {
                WordCompleteness::KnownStart => "known_start",
                WordCompleteness::UnknownStart => "unknown_start",
            }),
        );
        true
    }

    pub(crate) fn fail_context_activation(&mut self) {
        if let (Some(admission), Some(owner)) =
            (self.context_admission.as_ref(), self.context_owner.as_ref())
        {
            admission.revoke_current_owner(owner);
        }
        self.discard_context_activation();
    }

    fn discard_context_activation(&mut self) {
        let failed_owner = self.context_owner.clone();
        self.fail_context_activation_local();
        if let (Some(failed_owner), Ok(mut shared)) = (failed_owner, self.shared.lock()) {
            if shared.active_path.as_deref() == Some(self.path.as_str())
                && shared.context_owner_generation == Some(failed_owner.generation.0)
            {
                shared.active_path = None;
                shared.context_owner_generation = None;
                shared.handoff_tail_buffer.clear();
                shared.handoff_focus_receipt = None;
                if shared.autocorrect_suppression.take().is_some() {
                    shared.suppression_revision = shared.suppression_revision.wrapping_add(1);
                }
            }
        }
    }

    fn fail_context_activation_local(&mut self) {
        self.exact_manual_target_snapshot = None;
        self.client_context.input_mode_property_refresh_pending = false;
        self.context_owner = None;
        self.context_token = None;
        self.context_word_scope = None;
        self.context_reset_rereceipt = None;
        self.context_handoff_sealed = false;
        self.committed_tail.buffer.clear();
        self.committed_tail.autocorrect_suppression = None;
        self.committed_tail.pending_completion_learning = None;
        self.clear_preedit_completion_state();
    }

    pub(crate) async fn publish_pending_input_mode_property(
        &mut self,
        output: &mut EngineOutput<'_, '_>,
    ) {
        if !self.client_context.input_mode_property_refresh_pending || !output.is_legacy() {
            return;
        }
        let Some(owner) = self.context_owner.as_ref() else {
            self.client_context.input_mode_property_refresh_pending = false;
            return;
        };
        let owner_generation = owner.generation.0;
        let current = owner.path.as_str() == self.path
            && self.context_admission.as_ref().is_some_and(|admission| {
                admission.current_owner().as_ref() == Some(owner)
                    && self.context_token.as_ref().is_some_and(|token| {
                        token.matches_owner(owner) && admission.revalidate(token)
                    })
            })
            && self.shared.lock().is_ok_and(|shared| {
                shared.active_path.as_deref() == Some(self.path.as_str())
                    && shared.context_owner_generation == Some(owner_generation)
            });
        if !current {
            self.client_context.input_mode_property_refresh_pending = false;
            return;
        }
        // The initial FocusIn/Enable registration can precede selected-source
        // installation. Publish the installed decoder using the existing
        // legacy callback emitter, including clients without surrounding text.
        // Property transport failure does not change key or edit authority.
        let published = output
            .register_input_mode(self.layout_gesture.layout_is_ru)
            .await
            .is_ok();
        if published {
            self.client_context.input_mode_property_refresh_pending = false;
            // Enable is not repeated when stock IBus attaches another client
            // to this engine. Ask that client for its actual snapshot only
            // after the existing Ready owner is installed. The request grants
            // no word-boundary authority and needs no retry or separate latch.
            if self.client_context.surrounding_text_supported
                && !self.content_is_sensitive()
                && self.live_context_token().is_some()
            {
                let _ = output.require_surrounding_text().await;
            }
        }
        if trace::enabled() {
            trace::record(format!(
                r#"{{"kind":"ibus_input_mode","stage":"post_activation_update","engine_path":{:?},"owner_generation":{owner_generation},"layout_is_ru":{},"published":{published}}}"#,
                self.path, self.layout_gesture.layout_is_ru,
            ));
        }
    }

    pub(crate) async fn begin_context_key_callback(
        &mut self,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
        atomic: bool,
    ) -> KeyCallbackObservation {
        self.try_install_pending_context_activation();
        let Some(admission) = self.context_admission.clone() else {
            return KeyCallbackObservation::Refused;
        };
        let Some(owner) = self.context_owner.clone() else {
            // The stamp rendezvous is the only await admitted while the engine
            // interface lock is held. It lets the observer poison a pending
            // transfer to UnknownStart before this key runs literally; it does
            // not wait for the compatibility property or marker fence.
            if let Ok(observed) = admission.observe_callback(header, callback_entered).await {
                if !atomic {
                    if let Some(receipt) =
                        admission.pending_literal_input_receipt(&self.path, &observed)
                    {
                        return KeyCallbackObservation::PendingLiteral(receipt);
                    }
                }
                self.try_install_pending_context_activation();
                if !atomic && admission.passive_shift_observation_for(&self.path, &observed) {
                    return KeyCallbackObservation::PassiveShift;
                }
                if let Some(owner) = self.context_owner.as_ref() {
                    if let Ok(callback) =
                        admission.key_callback_from_observed(owner, observed.clone(), atomic)
                    {
                        return KeyCallbackObservation::Owned(Box::new(callback));
                    }
                }
                admission.abandon_observed_key(&observed);
            }
            return KeyCallbackObservation::Refused;
        };
        let callback = match admission.observe_callback(header, callback_entered).await {
            Ok(observed) => {
                if !atomic {
                    if let Some(receipt) =
                        admission.pending_literal_input_receipt(&self.path, &observed)
                    {
                        return KeyCallbackObservation::PendingLiteral(receipt);
                    }
                }
                if !atomic && admission.passive_shift_observation_for(&self.path, &observed) {
                    return KeyCallbackObservation::PassiveShift;
                }
                admission.key_callback_from_observed(&owner, observed, atomic)
            }
            Err(error) => Err(error),
        };
        match callback {
            Ok(callback) => KeyCallbackObservation::Owned(Box::new(callback)),
            Err(_) => {
                self.revoke_context_word();
                KeyCallbackObservation::Refused
            }
        }
    }

    pub(crate) fn settle_context_key_callback(
        &mut self,
        callback: Option<&KeyCallback>,
        keyval: u32,
        keycode: u32,
        state: u32,
        tail_before: &str,
        handled: bool,
    ) {
        let Some(admission) = self.context_admission.clone() else {
            return;
        };
        let Some(callback) = callback else {
            self.revoke_context_word();
            return;
        };
        if !self.atomic.active && !handled {
            let validated_native_effect = is_key_press(state)
                && self.exact_replay_tail_change_quarantined
                && self.committed_tail.buffer != tail_before;
            let unchanged_release =
                !is_key_press(state) && self.committed_tail.buffer == tail_before;
            if (validated_native_effect || unchanged_release)
                && self
                    .exact_replay_reset_provenance(false)
                    .is_some_and(|replay| {
                        replay.source_token.as_deref().is_some_and(|source| {
                            admission.key_retired_by_observed_word_reset(callback, source)
                        })
                    })
            {
                // Retired native callbacks grant no authority. Keep the actual
                // queued Reset and reconcile only after its exact client receipt.
                return;
            }
        }
        if !self.atomic.active
            && !is_key_press(state)
            && tail_before == self.committed_tail.buffer
            && admission.key_observation_was_revoked(callback, self.committed_tail.epoch)
        {
            // Reset may already have retired this received release. Preserve
            // either the invalid predecessor or the actual Reset handler's
            // successor; neither is new authority from this zero-effect key.
            return;
        }
        self.advance_context_word_scope(keyval, keycode, state, tail_before, handled);
        let Some(scope) = self.context_word_scope.as_ref() else {
            return;
        };
        if !admission.settle_key_callback(
            callback,
            SettledWordState::from_scope(self.committed_tail.epoch, scope).with_content_type(
                self.client_context.content_purpose,
                self.client_context.content_hints,
            ),
        ) {
            self.revoke_context_word();
            return;
        }
        self.context_token = admission.current_token();
        if self.has_native_autocorrection_restoration() {
            self.confirm_manual_autocorrection_restoration();
        }
    }

    pub(crate) fn settle_context_callback_without_input(&mut self, callback: Option<&KeyCallback>) {
        let Some(admission) = self.context_admission.clone() else {
            return;
        };
        let (Some(callback), Some(scope)) = (callback, self.context_word_scope.as_ref()) else {
            self.revoke_context_word();
            return;
        };
        if !admission.settle_key_callback(
            callback,
            SettledWordState::from_scope(self.committed_tail.epoch, scope).with_content_type(
                self.client_context.content_purpose,
                self.client_context.content_hints,
            ),
        ) {
            self.revoke_context_word();
            return;
        }
        self.context_token = admission.current_token();
    }

    fn advance_context_word_scope(
        &mut self,
        keyval: u32,
        keycode: u32,
        state: u32,
        tail_before: &str,
        handled: bool,
    ) {
        let appended_effect = handled
            .then(|| observed_tail_append_effect(tail_before, &self.committed_tail.buffer))
            .flatten();
        if !is_key_press(state) && appended_effect.is_none() {
            return;
        }
        let is_backspace = keyval == KEY_BACKSPACE;
        let crossed_boundary = is_backspace
            && (tail_before.is_empty()
                || tail_before
                    .chars()
                    .next_back()
                    .is_some_and(crate::preedit::is_observed_word_boundary));
        let observed_boundary = !is_backspace
            && (matches!(keyval, KEY_ENTER | KEY_KP_ENTER)
                || self
                    .physical_char(keyval, keycode)
                    .is_some_and(crate::preedit::is_observed_word_boundary));
        let passive_navigation = matches!(keyval, KEY_LEFT | KEY_RIGHT | KEY_UP | KEY_DOWN);
        let unproven_external_input = !handled
            && (passive_navigation
                || keyval == KEY_TAB
                || self.physical_char(keyval, keycode).is_none());
        let printable = self.physical_char(keyval, keycode).is_some();
        let track_suffix = self.context_word_scope.as_ref().is_some_and(|scope| {
            scope.tracks_observed_suffix()
                && (scope.lineage().completeness == WordCompleteness::UnknownStart
                    || self.uses_native_terminal_input())
        });
        let observed_append = if track_suffix && printable && !is_backspace {
            observed_tail_append_length(tail_before, &self.committed_tail.buffer)
        } else {
            None
        };
        let exact_backspace = is_backspace
            && tail_before
                .char_indices()
                .next_back()
                .is_some_and(|(last, _)| self.committed_tail.buffer == tail_before[..last]);
        let retained_boundary = if crossed_boundary && exact_backspace {
            self.committed_tail
                .buffer
                .chars()
                .enumerate()
                .filter(|(_, ch)| crate::preedit::is_observed_word_boundary(*ch))
                .last()
                .and_then(|(offset, _)| u32::try_from(offset).ok())
        } else {
            None
        };
        let new_boundary = observed_boundary
            .then(|| self.committed_tail.buffer.chars().enumerate().last())
            .flatten()
            .filter(|(_, ch)| crate::preedit::is_observed_word_boundary(*ch))
            .and_then(|(offset, _)| u32::try_from(offset).ok());
        let Some(scope) = self.context_word_scope.as_mut() else {
            return;
        };
        if scope.lineage().observed_first_word.is_some()
            && tail_before != self.committed_tail.buffer
            && (self.committed_tail.buffer.chars().count() >= crate::preedit::PREEDIT_TAIL_LIMIT
                || !(self.committed_tail.buffer.starts_with(tail_before)
                    || tail_before.starts_with(&self.committed_tail.buffer)))
        {
            // Do not rebase a retained start after prefix trimming or a
            // replacement. Exact append/right Backspace preserve the range;
            // crossing its separator revokes it below.
            scope.forget_observed_first_word();
        }
        if let Some((retained_chars, appended_chars, observed_boundary)) = appended_effect {
            if let Some(boundary) = observed_boundary {
                scope.close_at_observed_boundary(self.committed_tail.epoch, Some(boundary));
            } else if track_suffix {
                scope.observe_tail_append_span(retained_chars, appended_chars);
            }
            return;
        }
        // Modifier keys themselves are layout/gesture observations, not text
        // gaps. A command-modified non-modifier key is unproven client-side
        // input and revokes the admitted word.
        if is_shift_key(keyval) {
            scope.keep_or_revoke_unknown();
            return;
        }
        if is_accept_completion_with_space_key(keyval) {
            scope.keep_or_revoke_unknown();
            return;
        }
        if has_command_modifier(state) {
            self.context_reset_rereceipt = None;
            scope.revoke_for_input_gap();
            return;
        }
        if is_backspace {
            if crossed_boundary {
                // Reuse only a separator within this lineage's observed
                // range. An empty or non-exact mirror has no such witness.
                scope.reopen_at_retained_boundary(retained_boundary);
                if trace::enabled() {
                    trace::record(format!(
                        r#"{{"kind":"ibus_context_boundary_backspace","exact_backspace":{exact_backspace},"retained_observed_start":{}}}"#,
                        scope.lineage().completeness == WordCompleteness::KnownStart,
                    ));
                }
            } else if track_suffix {
                if exact_backspace {
                    scope.observe_tail_backspace();
                } else {
                    scope.invalidate_observed_suffix();
                }
            }
            return;
        }
        if observed_boundary {
            scope.close_at_observed_boundary(self.committed_tail.epoch, new_boundary);
            return;
        }
        if unproven_external_input {
            scope.revoke_for_input_gap();
        } else if printable && track_suffix {
            if let Some(retained_chars) = observed_append {
                scope.observe_tail_append(retained_chars);
            } else {
                scope.invalidate_observed_suffix();
            }
        }
    }

    pub(crate) fn advance_context_reset_rereceipt_after_key(
        &mut self,
        keyval: u32,
        keycode: u32,
        state: u32,
        tail_before: &str,
        handled: bool,
        boundary: (
            Option<ContextResetRereceiptCandidate>,
            bool,
            Option<AdmissionToken>,
        ),
    ) {
        if self.native_cursor_edit_receipt().is_some() {
            self.rebind_native_cursor_after_key(keyval, keycode, state, handled, tail_before);
            return;
        }
        let (boundary_candidate, native_space_eligible, native_backspace_source) = boundary;
        let owned_append = handled
            || (self.exact_replay_tail_change_quarantined && self.exact_replay_quarantine_active());
        if is_key_press(state) && keyval == KEY_BACKSPACE {
            if handled {
                // Only erasing the final character of a proved local preedit
                // can return to an already confirmed closed-word boundary.
                // Key settlement has retired the pre-effect token. The original
                // observation is retained only across this proved local-only
                // cancellation. IBus may deduplicate unchanged client replies.
                self.context_reset_rereceipt = (|| {
                    let candidate = boundary_candidate?;
                    let snapshot = candidate.owned_preedit_return_snapshot?;
                    let (last, removed) = tail_before.char_indices().next_back()?;
                    let token = self.live_context_token()?;
                    let owner = self.context_owner.as_ref()?;
                    let scope = self.context_word_scope.as_ref()?;
                    if snapshot.has_selection()
                        || snapshot.cursor_pos as usize != snapshot.text.chars().count()
                        || !snapshot_exactly_bounds_token(
                            &snapshot,
                            &candidate.token_text,
                            candidate.observed_suffix_chars as usize,
                        )
                        || self
                            .client_context
                            .surrounding_text_snapshot
                            .as_ref()
                            .is_some_and(|current| current != &snapshot)
                        || has_command_modifier(state)
                        || self.context_handoff_sealed
                        || self.atomic.active
                        || self.content_is_sensitive()
                        || !self.composition.buffer.is_empty()
                        || self.composition.legacy_word_preedit_active
                        || crate::preedit::is_observed_word_boundary(removed)
                        || tail_before[..last] != self.committed_tail.buffer
                        || candidate.tail_epoch.checked_add(1) != Some(self.committed_tail.epoch)
                        || candidate.armed_revision
                            != self.client_context.surrounding_observation_revision
                        || candidate.token_text != self.last_tail_with_boundary()
                        || !candidate.token_text.ends_with(char::is_whitespace)
                        || candidate.observed_suffix_chars as usize
                            != candidate.token_text.chars().count()
                        || scope.lineage().completeness != WordCompleteness::UnknownStart
                        || scope.lineage().observed_suffix_chars != 0
                        || !candidate.token.matches_owner(owner)
                        || !token.matches_owner(owner)
                        || !token.matches_word_scope(scope)
                        || candidate.token.word_scope().lineage().generation
                            != scope.lineage().generation
                        || self
                            .context_admission
                            .as_ref()?
                            .revalidate(&candidate.token)
                    {
                        return None;
                    }
                    // Keep the real original observation and its revision;
                    // neither a reconstructed mirror nor a new client receipt.
                    self.client_context.surrounding_text_snapshot = Some(snapshot);
                    Some(PendingContextResetRereceipt {
                        completed_replay: None,
                        confirmed_native_replay_reset_ack: None,
                        native_mutation_reset_evidence: None,
                        token,
                        predecessor_token: candidate.token,
                        tail_epoch: self.committed_tail.epoch,
                        token_text: candidate.token_text,
                        observed_suffix_chars: candidate.observed_suffix_chars,
                        armed_revision: self.client_context.surrounding_observation_revision,
                        confirmed: true,
                        awaiting_owned_append_receipt: false,
                        strict_prefix_echo_revision: None,
                        published_preedit: None,
                    })
                })();
                return;
            }
            // A native deletion carries only the already confirmed predecessor.
            // The client still owns the edit; a later exact receipt must confirm
            // the shortened range before it can authorize completion acceptance.
            let shortened = (|| {
                let candidate = boundary_candidate?;
                let (last, _) = tail_before.char_indices().next_back()?;
                let token = self.live_context_token()?;
                let scope = self.context_word_scope.as_ref()?;
                let owner = self.context_owner.as_ref()?;
                let actual_before = candidate.owned_preedit_return_snapshot?;
                // Actual SSI may confirm an owned literal commit without Reset.
                // Its pre-key token lives only on this callback's stack; never
                // fabricate a Reset with an equal predecessor and successor.
                let fresh_source = native_backspace_source
                    .as_ref()
                    .filter(|source| *source == &candidate.token);
                let source_token = if let Some(source) = fresh_source {
                    source
                } else {
                    let pending = self.context_reset_rereceipt.as_ref()?;
                    if !pending.confirmed
                        || candidate.token != pending.predecessor_token
                        || candidate.tail_epoch != pending.tail_epoch
                        || candidate.token_text != pending.token_text
                        || candidate.observed_suffix_chars != pending.observed_suffix_chars
                    {
                        return None;
                    }
                    &pending.token
                };
                // KDE commits the ordinary probe letter into the document.
                // Erasing its whole one-character word can return to the
                // preceding closed word. Keep only an expected range here;
                // the existing native-erasure proof still requires the next
                // whole actual receipt before permitting any manual effect.
                let closed_return = crate::kde::native_erasure::closed_word_after_erasure(
                    self.config.active_layout_backend(),
                    &actual_before,
                    tail_before,
                    &self.committed_tail.buffer,
                    &candidate.token_text,
                    candidate.observed_suffix_chars,
                );
                let returns_to_closed_word = closed_return.is_some();
                let token_text = closed_return.unwrap_or_else(|| self.last_tail_token_text());
                let chars = token_text.chars().count();
                if crate::kde::native_erasure::supports(self.config.active_layout_backend()) {
                    trace::record(format!(
                        r#"{{"kind":"ibus_native_erasure_candidate","stage":"range","closed_return":{returns_to_closed_word},"fresh_source":{},"scope_known":{},"scope_chars":{},"returned_chars":{chars},"source_lineage_current":{},"candidate_revision_current":{},"tail_epoch_current":{},"whole_tail_current":{}}}"#,
                        fresh_source.is_some(),
                        scope.lineage().completeness == WordCompleteness::KnownStart,
                        scope.lineage().observed_suffix_chars,
                        source_token.word_scope().lineage().generation
                            == scope.lineage().generation,
                        candidate.armed_revision
                            == self.client_context.surrounding_observation_revision,
                        candidate.tail_epoch.checked_add(1) == Some(self.committed_tail.epoch),
                        actual_before.text.ends_with(tail_before),
                    ));
                }
                if handled
                    || has_command_modifier(state)
                    || !self.composition.buffer.is_empty()
                    || candidate.armed_revision
                        != self.client_context.surrounding_observation_revision
                    || tail_before[..last] != self.committed_tail.buffer
                    || !tail_before.ends_with(candidate.token_text.as_str())
                    || chars == 0
                    || (!returns_to_closed_word
                        && chars + 1 != candidate.observed_suffix_chars as usize)
                    || candidate.tail_epoch.checked_add(1) != Some(self.committed_tail.epoch)
                    || (!returns_to_closed_word && !candidate.token_text.starts_with(&token_text))
                    || (fresh_source.is_some() && !returns_to_closed_word)
                    || !token.matches_owner(owner)
                    || !source_token.matches_owner(owner)
                    || !token.matches_word_scope(scope)
                    || source_token.word_scope().lineage().generation != scope.lineage().generation
                    || (fresh_source.is_none()
                        && (scope.lineage().completeness != WordCompleteness::UnknownStart
                            || scope.lineage().observed_suffix_chars >= chars as u32))
                {
                    return None;
                }
                if actual_before.has_selection()
                    || actual_before.cursor_pos as usize != actual_before.text.chars().count()
                    || !snapshot_exactly_bounds_token(
                        &actual_before,
                        &candidate.token_text,
                        candidate.observed_suffix_chars as usize,
                    )
                {
                    return None;
                }
                Some(PendingContextResetRereceipt {
                    completed_replay: None,
                    confirmed_native_replay_reset_ack: None,
                    native_mutation_reset_evidence: Some(NativeMutationResetEvidence::Awaiting {
                        actual_before,
                        effect_token: token.clone(),
                        focus_serial: self.client_context.focus_serial,
                        runtime_owner_lease_identity: self
                            .client_context
                            .runtime_owner_lease_identity,
                        layout_generation: self.layout_gesture.layout_generation,
                    }),
                    token,
                    predecessor_token: candidate.token,
                    tail_epoch: self.committed_tail.epoch,
                    token_text,
                    observed_suffix_chars: chars as u32,
                    armed_revision: self.client_context.surrounding_observation_revision,
                    confirmed: false,
                    awaiting_owned_append_receipt: false,
                    strict_prefix_echo_revision: None,
                    published_preedit: None,
                })
            })();
            if crate::kde::native_erasure::supports(self.config.active_layout_backend()) {
                trace::record(format!(
                    r#"{{"kind":"ibus_native_erasure_candidate","stage":"armed","pending":{}}}"#,
                    shortened.is_some(),
                ));
            }
            self.context_reset_rereceipt = shortened;
            return;
        }
        // Alt's release may still carry its own modifier bit. The accepted
        // append below is the effect witness; all other command bits remain.
        let boundary_modifier_state =
            if !is_key_press(state) && is_accept_completion_with_space_key(keyval) {
                state & !crate::protocol::MOD1_MASK
            } else {
                state
            };
        // The callback-local eligibility came from the actual pre-effect Legacy
        // ManagedCommit route. False native input alone is never an owned append.
        let native_space_append = native_space_eligible
            && !handled
            && is_key_press(state)
            && keyval == KEY_SPACE
            && !self.atomic.active
            && self.composition.buffer.is_empty()
            && !self.composition.legacy_word_preedit_active
            && !self.uses_native_terminal_input()
            && !self.exact_replay_tail_change_quarantined
            && !self.exact_replay_quarantine_active()
            && !matches!(
                self.committed_tail.autocorrect_suppression.as_ref(),
                Some(AutocorrectSuppression::ExactReplay(_))
            );
        if (owned_append || native_space_append) && !has_command_modifier(boundary_modifier_state) {
            if let Some(candidate) = boundary_candidate {
                if let Some(appended) = self.committed_tail.buffer.strip_prefix(tail_before) {
                    let closes_word = !appended.is_empty()
                        && appended.ends_with(char::is_whitespace)
                        && appended
                            .trim_end_matches(char::is_whitespace)
                            .chars()
                            .all(|ch| !crate::preedit::is_observed_word_boundary(ch));
                    let native_effect_is_exact = !native_space_append
                        || (appended == " "
                            && candidate.completed_replay.is_none()
                            && candidate.tail_epoch.checked_add(1)
                                == Some(self.committed_tail.epoch)
                            && candidate.armed_revision
                                == self.client_context.surrounding_observation_revision);
                    if closes_word
                        && native_effect_is_exact
                        && tail_before.ends_with(candidate.token_text.as_str())
                        && self.committed_tail.epoch > candidate.tail_epoch
                    {
                        if let (Some(token), Some(scope), Some(owner), Some(admission)) = (
                            self.context_token.as_ref(),
                            self.context_word_scope.as_ref(),
                            self.context_owner.as_ref(),
                            self.context_admission.as_ref(),
                        ) {
                            let token_text = self.last_tail_with_boundary();
                            if scope.lineage().completeness == WordCompleteness::KnownStart
                                && token != &candidate.token
                                && candidate.token.matches_owner(owner)
                                && token.matches_owner(owner)
                                && token.matches_word_scope(scope)
                                && admission.revalidate(token)
                                && (!native_space_append || !admission.revalidate(&candidate.token))
                                && token_text.starts_with(candidate.token_text.as_str())
                            {
                                // The real append closed a proved word. Retain
                                // only its exact predecessor for a later Reset;
                                // this is not authority for the next empty word.
                                self.context_reset_rereceipt = Some(PendingContextResetRereceipt {
                                    completed_replay: None,
                                    confirmed_native_replay_reset_ack: None,
                                    native_mutation_reset_evidence: None,
                                    token: token.clone(),
                                    predecessor_token: candidate.token,
                                    tail_epoch: self.committed_tail.epoch,
                                    observed_suffix_chars: token_text.chars().count() as u32,
                                    token_text,
                                    armed_revision: self
                                        .client_context
                                        .surrounding_observation_revision,
                                    confirmed: false,
                                    awaiting_owned_append_receipt: false,
                                    strict_prefix_echo_revision: None,
                                    published_preedit: candidate.published_preedit,
                                });
                                return;
                            }
                        }
                    }
                }
            }
        }
        if self.context_reset_rereceipt.is_none() {
            return;
        }
        if !is_key_press(state) {
            // Completion is accepted on release. Its actual append/boundary
            // settlement replaces the old Reset receipt; preparation does not.
            if tail_before != self.committed_tail.buffer {
                self.context_reset_rereceipt = None;
            }
            return;
        }
        if is_shift_key(keyval) || is_accept_completion_with_space_key(keyval) {
            return;
        }
        let Some(pending) = self.context_reset_rereceipt.clone() else {
            return;
        };
        let printable = self.physical_char(keyval, keycode);
        // Exact replay returns false so the client executes the key. Its
        // validated scope can preserve lineage, never replace a client receipt.
        if !owned_append
            || has_command_modifier(state)
            || keyval == KEY_BACKSPACE
            || matches!(
                keyval,
                KEY_ENTER | KEY_KP_ENTER | KEY_LEFT | KEY_RIGHT | KEY_UP | KEY_DOWN
            )
            || printable.is_none_or(crate::preedit::is_observed_word_boundary)
            || !tail_before.ends_with(pending.token_text.as_str())
            || observed_tail_append_length(tail_before, &self.committed_tail.buffer).is_none()
        {
            self.context_reset_rereceipt = None;
            return;
        }
        let next_token_text = self.last_tail_token_text();
        let next_chars = next_token_text.chars().count();
        if next_chars != pending.observed_suffix_chars.saturating_add(1) as usize
            || !next_token_text.starts_with(pending.token_text.as_str())
            || pending.tail_epoch.checked_add(1) != Some(self.committed_tail.epoch)
        {
            self.context_reset_rereceipt = None;
            return;
        }
        let (Some(token), Some(scope)) =
            (self.context_token.clone(), self.context_word_scope.as_ref())
        else {
            self.context_reset_rereceipt = None;
            return;
        };
        if !token.matches_word_scope(scope)
            || pending.token.word_scope().lineage().generation != scope.lineage().generation
            || self.context_owner.as_ref().is_none_or(|owner| {
                !pending.token.matches_owner(owner) || !token.matches_owner(owner)
            })
            || !self
                .context_admission
                .as_ref()
                .is_some_and(|admission| admission.revalidate(&token))
        {
            self.context_reset_rereceipt = None;
            return;
        }
        self.context_reset_rereceipt = Some(PendingContextResetRereceipt {
            completed_replay: None,
            confirmed_native_replay_reset_ack: None,
            native_mutation_reset_evidence: None,
            token,
            predecessor_token: pending.predecessor_token,
            tail_epoch: self.committed_tail.epoch,
            token_text: next_token_text,
            observed_suffix_chars: next_chars as u32,
            armed_revision: self.client_context.surrounding_observation_revision,
            confirmed: pending.confirmed,
            awaiting_owned_append_receipt: pending.confirmed,
            strict_prefix_echo_revision: None,
            published_preedit: pending.published_preedit,
        });
    }

    fn last_tail_with_boundary(&self) -> String {
        let tail = &self.committed_tail.buffer;
        let end = tail.trim_end_matches(char::is_whitespace).len();
        if end == 0 {
            return String::new();
        }
        let start = tail[..end]
            .char_indices()
            .rev()
            .find_map(|(index, ch)| ch.is_whitespace().then_some(index + ch.len_utf8()))
            .unwrap_or(0);
        tail[start..].to_string()
    }

    fn capture_native_backspace_reset_rereceipt_candidate(
        &self,
    ) -> Option<ContextResetRereceiptCandidate> {
        let snapshot = self.client_context.surrounding_text_snapshot.as_ref()?;
        if crate::kde::native_erasure::supports(self.config.active_layout_backend()) {
            let floor = self.client_context.managed_commit_snapshot_floor.as_ref();
            trace::record(format!(
                r#"{{"kind":"ibus_native_erasure_candidate","stage":"source","snapshot_chars":{},"tail_chars":{},"word_chars":{},"whole_tail_current":{},"reset_source":{},"managed_input":{},"composition_empty":{},"floor_epoch_current":{},"floor_focus_current":{},"floor_lease_current":{},"floor_layout_current":{},"floor_revision_current":{}}}"#,
                snapshot.text.chars().count(),
                self.committed_tail.buffer.chars().count(),
                self.last_tail_token_text().chars().count(),
                snapshot.text.ends_with(&self.committed_tail.buffer),
                self.context_reset_rereceipt_exact_manual_handoff_allowed(),
                self.client_context.managed_input,
                self.composition.buffer.is_empty(),
                floor.is_some_and(|f| f.tail_epoch == self.committed_tail.epoch),
                floor.is_some_and(|f| f.focus_serial == self.client_context.focus_serial),
                floor.is_some_and(|f| f.runtime_owner_lease_identity
                    == self.client_context.runtime_owner_lease_identity),
                floor.is_some_and(|f| f.layout_generation == self.layout_gesture.layout_generation),
                floor.is_some_and(|f| f.observation_revision
                    < self.client_context.surrounding_observation_revision),
            ));
        }
        if snapshot.has_selection() || snapshot.cursor_pos as usize != snapshot.text.chars().count()
        {
            return None;
        }
        let mut candidate = if self.context_reset_rereceipt_exact_manual_handoff_allowed()
            && self
                .context_reset_rereceipt
                .as_ref()
                .is_some_and(|pending| self.context_token.as_ref() == Some(&pending.token))
        {
            self.capture_context_reset_rereceipt_candidate()?
        } else {
            // Existing owned-commit provenance binds this actual SSI to one
            // local append; it supplies no new fresh-read capability or owner.
            let effect = self.client_context.managed_commit_snapshot_floor.as_ref()?;
            if self.context_handoff_sealed
                || self.atomic.active
                || self.content_is_sensitive()
                || !self.composition.buffer.is_empty()
                || self.composition.legacy_word_preedit_active
                || !self.client_context.managed_input
                || effect.tail_epoch != self.committed_tail.epoch
                || effect.focus_serial != self.client_context.focus_serial
                || effect.runtime_owner_lease_identity
                    != self.client_context.runtime_owner_lease_identity
                || effect.layout_generation != self.layout_gesture.layout_generation
                || self.client_context.surrounding_observation_revision
                    <= effect.observation_revision
            {
                return None;
            }
            let token_text = self.last_tail_token_text();
            let chars = token_text.chars().count() as u32;
            let mut shortened = self.committed_tail.buffer.clone();
            shortened.pop()?;
            crate::kde::native_erasure::closed_word_after_erasure(
                self.config.active_layout_backend(),
                snapshot,
                &self.committed_tail.buffer,
                &shortened,
                &token_text,
                chars,
            )?;
            ContextResetRereceiptCandidate {
                owned_preedit_return_snapshot: None,
                completed_replay: None,
                confirmed_native_replay_reset_ack: None,
                native_mutation_reset_evidence: None,
                token: self.live_context_token()?,
                tail_epoch: self.committed_tail.epoch,
                token_text,
                observed_suffix_chars: chars,
                armed_revision: self.client_context.surrounding_observation_revision,
                published_preedit: None,
            }
        };
        candidate.owned_preedit_return_snapshot = Some(snapshot.clone());
        Some(candidate)
    }

    fn capture_owned_preedit_erasure_reset_rereceipt_candidate(
        &self,
    ) -> Option<ContextResetRereceiptCandidate> {
        let (witness, word, epoch) = self.prepare_owned_preedit_local_backspace()?;
        if word.chars().count() != 1 {
            return None;
        }
        let prior = witness.closed_word_predecessor.as_deref()?;
        let owner = self.context_owner.as_ref()?;
        let snapshot = prior.owned_preedit_return_snapshot.as_ref()?;
        if !prior.token.matches_owner(owner)
            || prior.armed_revision != witness.armed_revision
            || snapshot.has_selection()
            || snapshot.cursor_pos as usize != snapshot.text.chars().count()
            || snapshot.cursor_pos != witness.start_cursor
            || snapshot.text != format!("{}{}", witness.snapshot_prefix, witness.snapshot_suffix)
            || !snapshot_exactly_bounds_token(
                snapshot,
                &prior.token_text,
                prior.observed_suffix_chars as usize,
            )
            || !witness
                .start_committed_tail
                .ends_with(prior.token_text.as_str())
            || !prior.token_text.ends_with(char::is_whitespace)
            || prior.observed_suffix_chars as usize != prior.token_text.chars().count()
        {
            return None;
        }
        Some(ContextResetRereceiptCandidate {
            owned_preedit_return_snapshot: Some(snapshot.clone()),
            completed_replay: None,
            confirmed_native_replay_reset_ack: None,
            native_mutation_reset_evidence: None,
            token: self.live_context_token()?,
            tail_epoch: epoch,
            token_text: prior.token_text.clone(),
            observed_suffix_chars: prior.observed_suffix_chars,
            armed_revision: self.client_context.surrounding_observation_revision,
            published_preedit: None,
        })
    }

    fn capture_confirmed_owned_reset_observation(
        &self,
        native_replay_ack: Option<&ConfirmedNativeReplayResetAck>,
    ) -> Option<ContextResetRereceiptCandidate> {
        let pending = self.context_reset_rereceipt.as_ref()?;
        let snapshot = self.client_context.surrounding_text_snapshot.as_ref()?;
        let native_replay_is_current = native_replay_ack
            .is_some_and(|ack| self.confirmed_native_replay_reset_ack_is_current(ack));
        if native_replay_ack.is_some() && !native_replay_is_current {
            return None;
        }
        if !pending.confirmed
            || pending.awaiting_owned_append_receipt
            || pending.completed_replay.is_some()
            || pending.published_preedit.is_some()
            || pending.strict_prefix_echo_revision.is_some()
            || self.composition.preedit_visible
            || self.composition.legacy_word_preedit_active
            || self.committed_tail.pending_visible_postcondition.is_some()
            || ((self.exact_replay_quarantine_active()
                || self.exact_replay_tail_change_quarantined)
                && !native_replay_is_current)
            || pending.armed_revision != self.client_context.surrounding_observation_revision
            || snapshot.has_selection()
            || snapshot.cursor_pos as usize != snapshot.text.chars().count()
            || !snapshot_exactly_bounds_token(
                snapshot,
                &pending.token_text,
                pending.observed_suffix_chars as usize,
            )
        {
            return None;
        }
        // Observer-first Reset has revoked the reducer token already. Reuse
        // only this local exact predecessor, then authenticate the new token
        // through the existing matched Reset and again after output cleanup.
        let mut candidate = self.capture_owned_context_reset_rereceipt_candidate()?;
        candidate.owned_preedit_return_snapshot = Some(snapshot.clone());
        candidate.confirmed_native_replay_reset_ack = native_replay_ack.cloned();
        Some(candidate)
    }

    fn native_erasure_reset_identity_is_current(
        &self,
        evidence: &NativeMutationResetEvidence,
    ) -> bool {
        evidence.identity()
            == (
                self.client_context.focus_serial,
                self.client_context.runtime_owner_lease_identity,
                self.layout_gesture.layout_generation,
            )
    }

    fn native_erasure_reset_ack_is_current(&self, evidence: &NativeMutationResetEvidence) -> bool {
        matches!(evidence, NativeMutationResetEvidence::Confirmed { .. })
            && self.native_erasure_reset_identity_is_current(evidence)
    }

    fn confirmed_native_erasure_for_receipt(
        &self,
        pending: &PendingContextResetRereceipt,
    ) -> Option<NativeMutationResetEvidence> {
        let evidence = pending.native_mutation_reset_evidence.as_ref()?;
        let NativeMutationResetEvidence::Awaiting {
            actual_before,
            effect_token,
            focus_serial,
            runtime_owner_lease_identity,
            layout_generation,
        } = evidence
        else {
            return None;
        };
        let admission = self.context_admission.as_ref()?;
        let owner = self.context_owner.as_ref()?;
        let snapshot = self.client_context.surrounding_text_snapshot.as_ref()?;
        let mut expected = actual_before.text.clone();
        expected.pop()?;
        let direct_effect_is_current = admission.revalidate(effect_token)
            && effect_token.word_scope().lineage().generation
                == pending.token.word_scope().lineage().generation;
        // Reset legitimately advances the word lineage. Only the exact inert
        // predecessor pinned by that matched Reset can bridge this advance.
        let matched_reset_effect_is_current = &pending.predecessor_token == effect_token
            && admission.exact_replay_reset_provenance_is_current(effect_token);
        if !self.native_erasure_reset_identity_is_current(evidence)
            || !effect_token.matches_owner(owner)
            || !(direct_effect_is_current || matched_reset_effect_is_current)
            || actual_before.has_selection()
            || actual_before.cursor_pos as usize != actual_before.text.chars().count()
            || (snapshot.text != expected
                && !crate::kde::native_erasure::closed_erasure_presentation_matches(
                    self.config.active_layout_backend(),
                    &expected,
                    &snapshot.text,
                    &pending.token_text,
                ))
            || snapshot.has_selection()
            || snapshot.cursor_pos as usize != expected.chars().count()
        {
            return None;
        }
        Some(NativeMutationResetEvidence::Confirmed {
            receipt_token: pending.token.clone(),
            focus_serial: *focus_serial,
            runtime_owner_lease_identity: *runtime_owner_lease_identity,
            layout_generation: *layout_generation,
        })
    }

    fn capture_confirmed_native_erasure_reset_observation(
        &self,
    ) -> Option<ContextResetRereceiptCandidate> {
        let pending = self.context_reset_rereceipt.as_ref()?;
        let ack = pending.native_mutation_reset_evidence.as_ref()?;
        if !self.native_erasure_reset_ack_is_current(ack)
            || ack.source_token() != &pending.token
            || self.context_token.as_ref() != Some(&pending.token)
        {
            return None;
        }
        let mut candidate = self.capture_confirmed_owned_reset_observation(None)?;
        candidate.token = ack.source_token().clone();
        candidate.native_mutation_reset_evidence = Some(ack.clone());
        Some(candidate)
    }

    fn confirmed_native_replay_reset_ack_is_current(
        &self,
        ack: &ConfirmedNativeReplayResetAck,
    ) -> bool {
        ack.focus_serial == self.client_context.focus_serial
            && ack.runtime_owner_lease_identity == self.client_context.runtime_owner_lease_identity
            && ack.layout_generation == self.layout_gesture.layout_generation
            && self.spent_completed_exact_replay_scope_is_current()
    }

    fn capture_confirmed_native_replay_reset_observation(
        &self,
    ) -> Option<ContextResetRereceiptCandidate> {
        let pending = self.context_reset_rereceipt.as_ref()?;
        if self.context_token.as_ref() != Some(&pending.token) {
            return None;
        }
        let ack = pending.confirmed_native_replay_reset_ack.as_ref()?;
        self.capture_confirmed_owned_reset_observation(Some(ack))
    }

    fn capture_owned_closing_space_observation(&self) -> Option<ContextResetRereceiptCandidate> {
        // Keep the exact client boundary already acknowledged below, including
        // an owned Space rendered as NBSP. This does not confirm a new surface.
        if self.context_word_scope.as_ref()?.lineage().completeness != WordCompleteness::KnownStart
            || self.layout_gesture.handled_press_keycodes.len() != 1
            || !self.layout_gesture.handled_press_keycodes.contains(&57)
            || !self.committed_tail.buffer.ends_with(char::is_whitespace)
            || !self
                .committed_tail
                .last_commit_at
                .is_some_and(|at| at.elapsed() <= std::time::Duration::from_millis(700))
        {
            return None;
        }
        self.capture_confirmed_owned_reset_observation(None)
    }

    fn capture_owned_managed_commit_reset_observation(
        &self,
    ) -> Option<ContextResetRereceiptCandidate> {
        let effect = self.client_context.managed_commit_snapshot_floor.as_ref()?;
        if !effect.reset_echo_available
            || effect.focus_serial != self.client_context.focus_serial
            || effect.runtime_owner_lease_identity
                != self.client_context.runtime_owner_lease_identity
            || effect.layout_generation != self.layout_gesture.layout_generation
            || effect.tail_epoch != self.committed_tail.epoch
            || self.client_context.surrounding_observation_revision <= effect.observation_revision
            || self.layout_gesture.handled_press_keycodes.len() != 1
            || !self
                .layout_gesture
                .handled_press_keycodes
                .contains(&effect.keycode)
        {
            return None;
        }
        self.capture_confirmed_owned_reset_observation(None)
    }

    fn restore_owned_reset_observation(
        &mut self,
        candidate: Option<ContextResetRereceiptCandidate>,
    ) {
        let Some(candidate) = candidate else {
            return;
        };
        let native_replay_is_current = candidate
            .confirmed_native_replay_reset_ack
            .as_ref()
            .is_some_and(|ack| self.confirmed_native_replay_reset_ack_is_current(ack));
        if candidate.confirmed_native_replay_reset_ack.is_some() && !native_replay_is_current {
            return;
        }
        if candidate
            .native_mutation_reset_evidence
            .as_ref()
            .is_some_and(|ack| {
                !self.native_erasure_reset_ack_is_current(ack)
                    || ack.source_token() != &candidate.token
            })
        {
            return;
        }
        let Some(snapshot) = candidate.owned_preedit_return_snapshot else {
            return;
        };
        let Some(pending) = self.context_reset_rereceipt.as_ref() else {
            return;
        };
        if self.context_handoff_sealed
            || self.atomic.active
            || self.content_is_sensitive()
            || !self.composition.buffer.is_empty()
            || self.composition.legacy_word_preedit_active
            || self.committed_tail.pending_visible_postcondition.is_some()
            || ((self.exact_replay_quarantine_active()
                || self.exact_replay_tail_change_quarantined)
                && !native_replay_is_current)
            || !self.client_context.surrounding_text_supported
            || self.client_context.surrounding_text_snapshot.is_some()
            || self.client_context.surrounding_observation_revision != candidate.armed_revision
            || self.committed_tail.epoch != candidate.tail_epoch
            || self.last_tail_with_boundary() != candidate.token_text
            || pending.tail_epoch != candidate.tail_epoch
            || pending.token_text != candidate.token_text
            || pending.observed_suffix_chars != candidate.observed_suffix_chars
            || pending.armed_revision != candidate.armed_revision
            || pending.predecessor_token != candidate.token
            || !self.context_admission.as_ref().is_some_and(|admission| {
                admission.exact_replay_reset_provenance_is_current(&candidate.token)
            })
            || pending.confirmed
            || pending.completed_replay.is_some()
            || self.context_token.as_ref() != Some(&pending.token)
            || !self.context_reset_rereceipt_identity_is_current()
        {
            return;
        }
        // No client receipt is invented: this is the same already observed
        // whole snapshot at the same revision. Any observer ingress while
        // clear_preedit awaited makes the new token fail the existing gate.
        self.client_context.surrounding_text_snapshot = Some(snapshot);
        if let Some(pending) = self.context_reset_rereceipt.as_mut() {
            pending.confirmed = true;
        }
    }

    fn capture_pending_native_erasure_reset_candidate(
        &self,
    ) -> Option<ContextResetRereceiptCandidate> {
        if !crate::kde::native_erasure::supports(self.config.active_layout_backend()) {
            return None;
        }
        let pending = self.context_reset_rereceipt.as_ref()?;
        let evidence = pending.native_mutation_reset_evidence.as_ref()?;
        let NativeMutationResetEvidence::Awaiting { effect_token, .. } = evidence else {
            return None;
        };
        if self.context_handoff_sealed
            || self.atomic.active
            || self.content_is_sensitive()
            || !self.composition.buffer.is_empty()
            || self.composition.legacy_word_preedit_active
            || pending.confirmed
            || pending.awaiting_owned_append_receipt
            || self.context_token.as_ref() != Some(&pending.token)
            || effect_token != &pending.token
            || pending.tail_epoch != self.committed_tail.epoch
            || pending.token_text != self.last_tail_with_boundary()
            || pending.armed_revision != self.client_context.surrounding_observation_revision
            || !self.native_erasure_reset_identity_is_current(evidence)
            || !self
                .context_owner
                .as_ref()
                .is_some_and(|owner| effect_token.matches_owner(owner))
            || !self.context_admission.as_ref().is_some_and(|admission| {
                admission.exact_replay_reset_provenance_is_current(effect_token)
            })
        {
            return None;
        }
        // The observer has already recorded this same-owner Reset. Preserve
        // only the actual native-erasure witness, independently of an older
        // spent manual replay. The next whole SSI still proves its result.
        Some(ContextResetRereceiptCandidate {
            owned_preedit_return_snapshot: None,
            completed_replay: None,
            confirmed_native_replay_reset_ack: None,
            native_mutation_reset_evidence: Some(evidence.clone()),
            token: effect_token.clone(),
            tail_epoch: pending.tail_epoch,
            token_text: pending.token_text.clone(),
            observed_suffix_chars: pending.observed_suffix_chars,
            armed_revision: pending.armed_revision,
            published_preedit: None,
        })
    }

    fn capture_context_reset_rereceipt_candidate(&self) -> Option<ContextResetRereceiptCandidate> {
        self.capture_pending_native_erasure_reset_candidate()
            .or_else(|| self.capture_owned_context_reset_rereceipt_candidate())
            .or_else(|| {
                if self.context_handoff_sealed || self.atomic.active {
                    return None;
                }
                // A Reset observed during replay may have retired the ordinary
                // pre-Space candidate. A fully matched replay still carries its
                // exact source proof across the closing boundary; only the real
                // complete client receipt can confirm this inert Reset seed.
                let replay = self.exact_replay_reset_provenance(true)?;
                let token_text = self.last_tail_with_boundary();
                if token_text.is_empty() || token_text != replay.replacement {
                    return None;
                }
                Some(ContextResetRereceiptCandidate {
                    owned_preedit_return_snapshot: None,
                    token: replay.source_token.as_deref()?.clone(),
                    tail_epoch: self.committed_tail.epoch,
                    observed_suffix_chars: token_text.chars().count() as u32,
                    token_text,
                    armed_revision: self.client_context.surrounding_observation_revision,
                    published_preedit: None,
                    completed_replay: Some(replay),
                    confirmed_native_replay_reset_ack: None,
                    native_mutation_reset_evidence: None,
                })
            })
    }

    fn capture_owned_context_reset_rereceipt_candidate(
        &self,
    ) -> Option<ContextResetRereceiptCandidate> {
        if self.context_handoff_sealed
            || self.atomic.active
            || !self.composition.buffer.is_empty()
            || self.content_is_sensitive()
            || (matches!(
                self.committed_tail.autocorrect_suppression.as_ref(),
                Some(AutocorrectSuppression::ExactReplay(_))
            ) && !self.exact_replay_quarantine_active())
            || !self.client_context.surrounding_text_supported
        {
            return None;
        }
        let scope = self.context_word_scope.as_ref()?;
        let owner = self.context_owner.as_ref()?;
        let predecessor_token = self.context_token.as_ref()?;
        let token_text = self.last_tail_with_boundary();
        let observed_suffix_chars = token_text.chars().count();
        if observed_suffix_chars == 0 {
            return None;
        }
        if let Some(pending) = self.context_reset_rereceipt.as_ref().filter(|pending| {
            pending.tail_epoch == self.committed_tail.epoch
                && pending.token_text == token_text
                && pending.observed_suffix_chars as usize == observed_suffix_chars
                && self.context_token.as_ref() == Some(&pending.token)
                && pending.token.matches_word_scope(scope)
                && pending.token.matches_owner(owner)
                && pending.predecessor_token.matches_owner(owner)
                && pending.predecessor_token != pending.token
        }) {
            let same_gesture_refresh = self.layout_gesture.pending_manual_toggle
                && self
                    .layout_gesture
                    .pending_manual_refresh_at
                    .is_some_and(|queued| {
                        queued.elapsed() <= std::time::Duration::from_millis(700)
                    });
            let published_preedit = pending
                .published_preedit
                .as_ref()
                .filter(|published| {
                    (pending.confirmed || same_gesture_refresh)
                        // A locally owned next letter can shorten the same
                        // displayed word without a new exact client receipt.
                        // Its earlier prefix remains an inert publication
                        // witness across this one owner-matched Reset.
                        && published.prefix_chars <= observed_suffix_chars as u32
                        && published.text.starts_with(&token_text)
                })
                .cloned();
            return Some(ContextResetRereceiptCandidate {
                owned_preedit_return_snapshot: None,
                completed_replay: pending.completed_replay.clone(),
                confirmed_native_replay_reset_ack: pending
                    .confirmed_native_replay_reset_ack
                    .clone(),
                native_mutation_reset_evidence: pending.native_mutation_reset_evidence.clone(),
                // Several Reset ingresses may already share the latest reducer
                // token. Keep the original revoked provenance across callbacks.
                token: pending.predecessor_token.clone(),
                tail_epoch: self.committed_tail.epoch,
                token_text,
                observed_suffix_chars: observed_suffix_chars as u32,
                armed_revision: self.client_context.surrounding_observation_revision,
                published_preedit,
            });
        }
        // The observer sees Reset before this callback and has already revoked
        // the reducer token. Preserve only the exact local predecessor; the
        // owner-matching Reset and post-Reset token are checked while arming.
        if !predecessor_token.matches_owner(owner)
            || !predecessor_token.matches_word_scope(scope)
            || self.committed_tail.buffer.ends_with(char::is_whitespace)
            || (scope.lineage().completeness == WordCompleteness::UnknownStart
                && observed_suffix_chars as u32 != scope.lineage().observed_suffix_chars)
        {
            return None;
        }
        Some(ContextResetRereceiptCandidate {
            owned_preedit_return_snapshot: None,
            completed_replay: None,
            confirmed_native_replay_reset_ack: None,
            native_mutation_reset_evidence: None,
            token: predecessor_token.clone(),
            tail_epoch: self.committed_tail.epoch,
            token_text,
            observed_suffix_chars: observed_suffix_chars as u32,
            armed_revision: self.client_context.surrounding_observation_revision,
            published_preedit: None,
        })
    }

    fn arm_context_reset_rereceipt(
        &mut self,
        candidate: Option<ContextResetRereceiptCandidate>,
        owner: &crate::context_admission::EngineOwner,
    ) {
        self.context_reset_rereceipt = None;
        let (Some(mut candidate), Some(token)) = (candidate, self.context_token.clone()) else {
            trace::record(
                r#"{"kind":"ibus_context_reset_rereceipt","stage":"rejected","reason":"missing_predecessor_or_post_reset_token"}"#,
            );
            return;
        };
        if !token.matches_owner(owner)
            || token == candidate.token
            || !self
                .context_admission
                .as_ref()
                .is_some_and(|admission| admission.revalidate(&token))
        {
            trace::record(
                r#"{"kind":"ibus_context_reset_rereceipt","stage":"rejected","reason":"post_reset_identity"}"#,
            );
            return;
        }
        // Pin this effect's actual token only for the matched Reset. Preserve
        // the older generic predecessor everywhere else (including C06 deletion).
        let erasure_seed = candidate
            .native_mutation_reset_evidence
            .take()
            .and_then(|evidence| {
                if !self.native_erasure_reset_identity_is_current(&evidence)
                    || !self.context_admission.as_ref().is_some_and(|admission| {
                        admission.exact_replay_reset_provenance_is_current(evidence.source_token())
                    })
                {
                    return None;
                }
                candidate.token = evidence.source_token().clone();
                match evidence {
                    NativeMutationResetEvidence::Awaiting { .. } => Some(evidence),
                    NativeMutationResetEvidence::Confirmed { .. } => None,
                    NativeMutationResetEvidence::CursorEditing(_) => None,
                }
            });
        let tail_epoch = candidate.tail_epoch;
        let observed_suffix_chars = candidate.observed_suffix_chars;
        let same_gesture_refresh = self.layout_gesture.pending_manual_toggle
            && self
                .layout_gesture
                .pending_manual_refresh_at
                .is_some_and(|queued| queued.elapsed() <= std::time::Duration::from_millis(700));
        self.context_reset_rereceipt = Some(PendingContextResetRereceipt {
            completed_replay: candidate.completed_replay,
            confirmed_native_replay_reset_ack: None,
            native_mutation_reset_evidence: erasure_seed,
            token,
            predecessor_token: candidate.token,
            tail_epoch,
            token_text: candidate.token_text,
            observed_suffix_chars,
            armed_revision: candidate.armed_revision,
            confirmed: false,
            awaiting_owned_append_receipt: false,
            strict_prefix_echo_revision: None,
            // HidePreeditText can make Firefox Reset before its last echo of
            // this exact publication. Carry only the inert display witness
            // while the same physical gesture still owns the refresh lease.
            published_preedit: if same_gesture_refresh {
                candidate.published_preedit
            } else {
                None
            },
        });
        trace::record(format!(
            r#"{{"kind":"ibus_context_reset_rereceipt","stage":"armed","tail_epoch":{tail_epoch},"suffix_chars":{observed_suffix_chars}}}"#,
        ));
    }

    fn discard_exact_replay_reset_seed(&mut self) {
        self.client_context.native_replay_progress = None;
        if self
            .context_reset_rereceipt
            .as_ref()
            .is_some_and(|pending| pending.completed_replay.is_some())
        {
            self.context_reset_rereceipt = None;
        }
        // A capability change spends the native Reset ACK, but cannot erase
        // an already confirmed ordinary SSI when surrounding support remains.
        if let Some(pending) = self.context_reset_rereceipt.as_mut() {
            pending.confirmed_native_replay_reset_ack = None;
            pending.native_mutation_reset_evidence = None;
        }
        let replay = match self.committed_tail.autocorrect_suppression.as_ref() {
            Some(AutocorrectSuppression::ExactReplay(scope)) if scope.source_token.is_some() => {
                Some(scope.clone())
            }
            _ => None,
        };
        if let Some(replay) = replay {
            if !self.consume_exact_replay_reset_provenance(&replay) {
                // Never touch another owner's shared scope. Retire this local
                // seed even when the paired scope cannot be consumed.
                if let Some(AutocorrectSuppression::ExactReplay(scope)) =
                    self.committed_tail.autocorrect_suppression.as_mut()
                {
                    scope.source_token = None;
                }
            }
        }
    }

    fn discard_context_reset_rereceipt(&mut self) {
        if self.native_cursor_edit_receipt().is_some() {
            self.trace_native_cursor_phase("before_discard", 0, 0, 0);
        }
        self.context_reset_rereceipt = None;
        self.discard_exact_replay_reset_seed();
    }

    fn acknowledge_owned_space_boundary(&mut self) {
        let Some(pending) = self.context_reset_rereceipt.as_ref() else {
            return;
        };
        // Closing the replay with Space may already have produced its own
        // boundary token. Use the completed replay only as its effect witness;
        // retain whichever token/provenance the existing Reset path carried.
        let carried_replay = pending.completed_replay.is_some();
        let replay = pending
            .completed_replay
            .clone()
            .or_else(|| self.exact_replay_reset_provenance(true))
            .or_else(|| self.completed_exact_replay_boundary_provenance());
        if pending.confirmed
            || self.committed_tail.pending_visible_postcondition.is_some()
            || (replay.is_none()
                && (self.exact_replay_quarantine_active()
                    || self.exact_replay_tail_change_quarantined))
            || self.context_token.as_ref() != Some(&pending.token)
            || pending.armed_revision.checked_add(1)
                != Some(self.client_context.surrounding_observation_revision)
            || !(self.context_reset_rereceipt_identity_is_current()
                || self.context_reset_rereceipt_boundary_is_current(pending))
        {
            return;
        }
        let Some(word) = pending.token_text.strip_suffix(' ') else {
            return;
        };
        if !word
            .trim_end_matches([' ', '\u{a0}'])
            .chars()
            .next_back()
            .is_some_and(|ch| !crate::preedit::is_observed_word_boundary(ch))
            || pending.observed_suffix_chars as usize != pending.token_text.chars().count()
        {
            return;
        }
        let Some(owned_prefix) = self.committed_tail.buffer.strip_suffix(&pending.token_text)
        else {
            return;
        };
        let actual = if let Some(scope) = replay.as_ref() {
            let Some(prefix) = scope.observed_external_prefix.as_deref() else {
                return;
            };
            let Some(snapshot) = self.client_context.surrounding_text_snapshot.as_ref() else {
                return;
            };
            let Some(replacement) = snapshot
                .text
                .strip_prefix(&format!("{prefix}{}", scope.unchanged_prefix))
            else {
                return;
            };
            // The existing tail owner below revalidates completed provenance
            // and permits presentation changes only in its newly replayed Spaces.
            let actual_tail = format!("{}{replacement}", scope.unchanged_prefix);
            let Some(suffix) = actual_tail.strip_prefix(owned_prefix) else {
                return;
            };
            suffix.to_string()
        } else {
            format!("{word}\u{a0}")
        };
        if !self
            .client_context
            .surrounding_text_snapshot
            .as_ref()
            .is_some_and(|snapshot| {
                snapshot_exactly_bounds_token(
                    snapshot,
                    &actual,
                    pending.observed_suffix_chars as usize,
                )
            })
        {
            return;
        }
        // The receipt bounds its token suffix; the tail owner retains the full
        // owned prefix. Only newly emitted Space positions may be acknowledged,
        // using a completed paired replay when present. Keep every earlier byte
        // exact and let the existing receipt validator confirm the observation.
        let actual_tail = format!("{owned_prefix}{actual}");
        let Ok(acknowledged_replay) =
            self.acknowledge_owned_space_tail(&actual_tail, replay.as_ref())
        else {
            return;
        };
        let pending = self
            .context_reset_rereceipt
            .as_mut()
            .expect("live boundary receipt checked above");
        pending.token_text = actual;
        if carried_replay {
            pending.completed_replay = acknowledged_replay;
        }
        trace::record(format!(
            r#"{{"kind":"ibus_owned_space_boundary","stage":"client_acknowledged","sent_codepoint":32,"observed_codepoint":160,"tail_epoch":{}}}"#,
            self.committed_tail.epoch,
        ));
    }

    fn observe_context_reset_rereceipt_surrounding_text(
        &mut self,
        previous_snapshot: Option<&SurroundingTextSnapshot>,
        previous_revision: u64,
        retained_native_progress: Option<&SurroundingTextSnapshot>,
    ) {
        if self.observe_native_cursor_edit_receipt() {
            return;
        }
        self.acknowledge_owned_space_boundary();
        let Some(pending) = self.context_reset_rereceipt.clone() else {
            return;
        };
        let erasure_reset_ack = self.confirmed_native_erasure_for_receipt(&pending);
        if crate::kde::native_erasure::supports(self.config.active_layout_backend())
            && matches!(
                pending.native_mutation_reset_evidence.as_ref(),
                Some(NativeMutationResetEvidence::Awaiting { .. })
            )
            && erasure_reset_ack.is_none()
        {
            // An independently observed erasure requires its whole actual
            // result. A matching suffix must not bypass a failed full-field
            // receipt through the ordinary owned-Reset continuation below.
            self.context_reset_rereceipt = None;
            return;
        }
        // The first real SSI spends an erasure seed even when contradictory.
        // A later suffix-only match cannot create the effect's Reset ACK.
        if let Some(current) = self.context_reset_rereceipt.as_mut() {
            if matches!(
                current.native_mutation_reset_evidence,
                Some(NativeMutationResetEvidence::Awaiting { .. })
            ) {
                current.native_mutation_reset_evidence = None;
            }
        }
        // Spend the echo at the next observation, even if that observation is
        // contradictory. Other inert cache/sentinel paths cannot manufacture it.
        if let Some(current) = self.context_reset_rereceipt.as_mut() {
            current.strict_prefix_echo_revision = None;
        }
        let exact_after_managed_prefix_echo = !pending.confirmed
            && pending.strict_prefix_echo_revision.is_some_and(|revision| {
                revision.checked_add(1)
                    == Some(self.client_context.surrounding_observation_revision)
                    && pending.armed_revision.checked_add(1) == Some(revision)
                    && previous_revision == revision
                    && previous_snapshot.is_some_and(|previous| {
                        snapshot_exactly_bounds_strict_token_prefix(previous, &pending.token_text)
                            && previous.cursor_pos as usize == previous.text.chars().count()
                            && self
                                .client_context
                                .surrounding_text_snapshot
                                .as_ref()
                                .is_some_and(|current| {
                                    current.cursor_pos as usize == current.text.chars().count()
                                        && current.text.starts_with(&previous.text)
                                })
                    })
            })
            && self.managed_strict_prefix_echo_identity_is_current(&pending)
            && self.context_reset_rereceipt_matches_current_snapshot();
        let revision_gap = self
            .client_context
            .surrounding_observation_revision
            .checked_sub(pending.armed_revision);
        let advancing_native_target = revision_gap == Some(1)
            && previous_revision == pending.armed_revision
            && previous_snapshot
                .or(retained_native_progress)
                .zip(self.client_context.surrounding_text_snapshot.as_ref())
                .is_some_and(|(previous, current)| {
                    self.exact_replay_target_advances(previous, current)
                });
        if !pending.confirmed
            && revision_gap == Some(1)
            && previous_revision == pending.armed_revision
            && self.context_token.as_ref() == Some(&pending.token)
            && self.context_reset_rereceipt_boundary_is_current(&pending)
        {
            if let (Some(replay), Some(snapshot)) = (
                self.completed_exact_replay_boundary_provenance(),
                self.client_context.surrounding_text_snapshot.as_ref(),
            ) {
                let Some(external_prefix) = replay.observed_external_prefix.as_deref() else {
                    self.discard_context_reset_rereceipt();
                    return;
                };
                let prefix = format!("{external_prefix}{}", replay.unchanged_prefix);
                let expected = format!("{prefix}{}", replay.replacement);
                let prefix_chars = prefix.chars().count();
                let is_target_prefix = |surface: &SurroundingTextSnapshot| {
                    !surface.has_selection()
                        && surface.text.starts_with(&prefix)
                        && expected.starts_with(&surface.text)
                        && surface.cursor_pos as usize >= prefix_chars
                        && surface.cursor_pos as usize <= surface.text.chars().count()
                };
                // Shape also recognizes a previously retained wider caret lag;
                // it cannot itself permit retaining the next observation.
                if let Some(previous) = previous_snapshot
                    .or(retained_native_progress)
                    .filter(|p| is_target_prefix(p))
                {
                    if !is_target_prefix(snapshot) || !snapshot.text.starts_with(&previous.text) {
                        self.discard_context_reset_rereceipt();
                        return;
                    }
                }
                if is_target_prefix(snapshot)
                    && (snapshot.cursor_pos as usize == snapshot.text.chars().count()
                        || self.exact_replay_contains_prior_snapshot(snapshot)
                        || advancing_native_target)
                    && (snapshot.text != expected
                        || snapshot.cursor_pos as usize != snapshot.text.chars().count())
                {
                    // Actual completed native replay may reach the client in
                    // several ordered SSI pieces after Space settled KnownStart.
                    // Keep only its same paired lineage; each real observation
                    // advances the existing revision, without edit authority.
                    // Replacement text may precede its caret advance. Wider
                    // lag needs actual growing text and a nonregressing caret;
                    // only the exact end-caret receipt can confirm the result.
                    if let Some(current) = self.context_reset_rereceipt.as_mut() {
                        current.confirmed = false;
                        current.confirmed_native_replay_reset_ack = None;
                        current.native_mutation_reset_evidence = None;
                        current.strict_prefix_echo_revision = None;
                        current.armed_revision =
                            self.client_context.surrounding_observation_revision;
                    }
                    trace::record(
                        r#"{"kind":"ibus_context_reset_rereceipt","stage":"retained_unconfirmed","reason":"completed_native_target_progress"}"#,
                    );
                    return;
                }
            }
        }
        if (revision_gap == Some(1) || pending.confirmed && revision_gap == Some(2))
            && self.context_reset_rereceipt_identity_is_current()
            && self
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .is_some_and(|snapshot| {
                    snapshot_has_transient_zero_width_boundary(snapshot, &pending)
                })
        {
            // A client may briefly place a zero-width editor sentinel just
            // after the caret. Preserve lineage only; require a fresh exact
            // receipt before Tab or any text mutation can use it.
            if let Some(pending) = self.context_reset_rereceipt.as_mut() {
                pending.confirmed = false;
                pending.confirmed_native_replay_reset_ack = None;
                pending.native_mutation_reset_evidence = None;
                pending.armed_revision = self.client_context.surrounding_observation_revision;
            }
            trace::record(
                r#"{"kind":"ibus_context_reset_rereceipt","stage":"retained_unconfirmed","reason":"zero_width_right_boundary"}"#,
            );
            return;
        }
        if self.client_context.surrounding_observation_revision
            == pending.armed_revision.saturating_add(1)
            && (!pending.confirmed || pending.awaiting_owned_append_receipt)
            && !self.context_reset_rereceipt_matches_current_snapshot()
            && self.context_reset_rereceipt_identity_is_current()
            && self
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .is_some_and(|snapshot| {
                    self.exact_replay_contains_prior_snapshot(snapshot) || advancing_native_target
                })
        {
            if let Some(pending) = self.context_reset_rereceipt.as_mut() {
                pending.confirmed = false;
                pending.confirmed_native_replay_reset_ack = None;
                pending.native_mutation_reset_evidence = None;
                pending.armed_revision = self.client_context.surrounding_observation_revision;
            }
            trace::record(
                r#"{"kind":"ibus_context_reset_rereceipt","stage":"retained_unconfirmed","reason":"observed_replay_prior_surface"}"#,
            );
            return;
        }
        if self.client_context.surrounding_observation_revision
            == pending.armed_revision.saturating_add(1)
            && !self.context_reset_rereceipt_matches_current_snapshot()
            && (self.context_reset_rereceipt_identity_is_current()
                || self.context_reset_rereceipt_boundary_is_current(&pending))
            && self
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot_matches_retired_preedit(snapshot, &pending))
        {
            // Some asynchronous clients advance the cursor before replacing
            // their cached composition text. Retain only the observed lineage;
            // the retired publication cannot authorize any text effect.
            if let Some(pending) = self.context_reset_rereceipt.as_mut() {
                pending.confirmed = false;
                pending.confirmed_native_replay_reset_ack = None;
                pending.native_mutation_reset_evidence = None;
                pending.armed_revision = self.client_context.surrounding_observation_revision;
            }
            trace::record(
                r#"{"kind":"ibus_context_reset_rereceipt","stage":"retained_unconfirmed","reason":"published_preedit_cache"}"#,
            );
            return;
        }
        if self.client_context.surrounding_observation_revision
            == pending.armed_revision.saturating_add(1)
            && self.context_reset_rereceipt_strict_prefix_is_current(&pending)
            && (!pending.confirmed
                || pending.awaiting_owned_append_receipt
                || previous_snapshot.is_some_and(|previous| {
                    snapshot_exactly_bounds_token(
                        previous,
                        &pending.token_text,
                        pending.observed_suffix_chars as usize,
                    ) && previous.cursor_pos as usize == previous.text.chars().count()
                        && self
                            .client_context
                            .surrounding_text_snapshot
                            .as_ref()
                            .is_some_and(|current| {
                                current.cursor_pos as usize == current.text.chars().count()
                                    && previous.text.starts_with(&current.text)
                            })
                }))
        {
            let echo_revision = (pending.confirmed
                && self.managed_strict_prefix_echo_identity_is_current(&pending)
                && self
                    .client_context
                    .surrounding_text_snapshot
                    .as_ref()
                    .is_some_and(|snapshot| {
                        snapshot.cursor_pos as usize == snapshot.text.chars().count()
                    }))
            .then_some(self.client_context.surrounding_observation_revision);
            // A delayed strict prefix contradicts exact authority, but not the
            // observed token lineage. Retain it only as an inert predecessor;
            // an independently confirmed managed chain may accept only the
            // immediately following full exact receipt at the end of the
            // whole snapshot. Prefix comparison alone cannot prove unchanged
            // external context to the right of the caret. Other paths still
            // require a later authenticated Reset and exact full receipt.
            if let Some(pending) = self.context_reset_rereceipt.as_mut() {
                pending.confirmed = false;
                pending.confirmed_native_replay_reset_ack = None;
                pending.native_mutation_reset_evidence = None;
                pending.strict_prefix_echo_revision = echo_revision;
            }
            trace::record(format!(
                r#"{{"kind":"ibus_context_reset_rereceipt","stage":"retained_unconfirmed","reason":"{}"}}"#,
                if pending.confirmed {
                    "confirmed_strict_prefix_receipt"
                } else {
                    "first_strict_prefix_receipt"
                },
            ));
            return;
        }
        if pending.confirmed {
            // Owned append can carry confirmation while the prior actual
            // observation still describes the shorter predecessor. Once the
            // prior snapshot bounds this token, a refresh must preserve all
            // external context, not merely the matching suffix.
            let adjacent_snapshot_is_consistent = pending.awaiting_owned_append_receipt
                || previous_snapshot.is_some_and(|previous| {
                    self.client_context.surrounding_text_snapshot.as_ref() == Some(previous)
                        && pending.published_preedit.is_none()
                        && (self.context_reset_rereceipt_identity_is_current()
                            || (self.context_token.as_ref() == Some(&pending.token)
                                && self.context_word_scope.as_ref().is_some_and(|scope| {
                                    scope.lineage().completeness == WordCompleteness::KnownStart
                                        && scope.lineage().observed_suffix_chars == 0
                                })
                                && pending
                                    .confirmed_native_replay_reset_ack
                                    .as_ref()
                                    .is_some_and(|ack| {
                                        self.confirmed_native_replay_reset_ack_is_current(ack)
                                    })))
                });
            if previous_revision == pending.armed_revision
                && pending.armed_revision.checked_add(1)
                    == Some(self.client_context.surrounding_observation_revision)
                && adjacent_snapshot_is_consistent
                && self.context_reset_rereceipt_matches_current_snapshot()
            {
                if let Some(pending) = self.context_reset_rereceipt.as_mut() {
                    pending.published_preedit = None;
                    pending.awaiting_owned_append_receipt = false;
                    pending.armed_revision = self.client_context.surrounding_observation_revision;
                }
                trace::record(format!(
                    r#"{{"kind":"ibus_context_reset_rereceipt","stage":"advanced_confirmed","tail_epoch":{},"suffix_chars":{}}}"#,
                    pending.tail_epoch, pending.observed_suffix_chars,
                ));
                return;
            }
            self.discard_context_reset_rereceipt();
            trace::record(
                r#"{"kind":"ibus_context_reset_rereceipt","stage":"rejected","reason":"second_surrounding_receipt"}"#,
            );
            return;
        }
        if (self.client_context.surrounding_observation_revision
            != pending.armed_revision.saturating_add(1)
            && !exact_after_managed_prefix_echo)
            || !self.context_reset_rereceipt_matches_current_snapshot()
        {
            if trace::enabled() {
                let snapshot = self.client_context.surrounding_text_snapshot.as_ref();
                let surface_is_strict_prefix = snapshot.is_some_and(|snapshot| {
                    snapshot_exactly_bounds_strict_token_prefix(snapshot, &pending.token_text)
                });
                let surface_is_prior_replay = snapshot
                    .is_some_and(|snapshot| self.exact_replay_contains_prior_snapshot(snapshot));
                let (chars, cursor, selection, zero_width_only, initial_matches, ascii_latin) =
                    snapshot.map_or((0, 0, false, false, false, false), |snapshot| {
                        let first = snapshot.text.chars().next();
                        (
                            snapshot.text.chars().count(),
                            snapshot.cursor_pos,
                            snapshot.has_selection(),
                            snapshot.text == "\u{200b}",
                            first.is_some() && first == pending.token_text.chars().next(),
                            first.is_some_and(|ch| ch.is_ascii_alphabetic()),
                        )
                    });
                trace::record(format!(
                    r#"{{"kind":"ibus_context_reset_rereceipt","stage":"mismatch_diagnostic","revision_gap":{},"expected_chars":{},"snapshot_chars":{chars},"cursor":{cursor},"selection":{selection},"zero_width_only":{zero_width_only},"initial_matches":{initial_matches},"ascii_latin":{ascii_latin},"strict_prefix_surface":{surface_is_strict_prefix},"strict_prefix_with_identity":{},"prior_replay_surface":{surface_is_prior_replay},"replay_scope_present":{},"identity_current":{}}}"#,
                    self.client_context
                        .surrounding_observation_revision
                        .saturating_sub(pending.armed_revision),
                    pending.observed_suffix_chars,
                    self.context_reset_rereceipt_strict_prefix_is_current(&pending),
                    matches!(
                        self.committed_tail.autocorrect_suppression.as_ref(),
                        Some(AutocorrectSuppression::ExactReplay(_))
                    ),
                    self.context_reset_rereceipt_identity_is_current(),
                ));
            }
            self.discard_context_reset_rereceipt();
            trace::record(
                r#"{"kind":"ibus_context_reset_rereceipt","stage":"rejected","reason":"surrounding_receipt_mismatch"}"#,
            );
            return;
        }
        if exact_after_managed_prefix_echo {
            trace::record(
                r#"{"kind":"ibus_context_reset_rereceipt","stage":"managed_prefix_echo_confirmed"}"#,
            );
        }
        // Fully settled native callbacks can use the ordinary Reset receipt
        // without carrying a replay seed. Resolve its same complete paired
        // proof here; a spent source or an unchanged refresh cannot rearm it.
        let completed_replay = pending
            .completed_replay
            .clone()
            .or_else(|| self.exact_replay_reset_provenance(true))
            .or_else(|| self.completed_exact_replay_boundary_provenance());
        if let Some(replay) = completed_replay.as_ref() {
            let aligned = (|| {
                let reset_is_current = self.context_reset_rereceipt_identity_is_current();
                let boundary_is_current =
                    self.context_reset_rereceipt_boundary_is_current(&pending);
                let provenance = if reset_is_current {
                    self.exact_replay_reset_provenance(true)
                } else if boundary_is_current {
                    self.completed_exact_replay_boundary_provenance()
                } else {
                    None
                };
                if provenance.as_ref() != Some(replay) {
                    return false;
                }
                let Some(prefix) = replay.observed_external_prefix.as_ref() else {
                    return false;
                };
                let expected = format!(
                    "{}{}{}",
                    prefix, replay.unchanged_prefix, replay.replacement
                );
                let Some(snapshot) = self.client_context.surrounding_text_snapshot.as_ref() else {
                    return false;
                };
                if snapshot.text != expected
                    || snapshot.has_selection()
                    || snapshot.cursor_pos as usize != expected.chars().count()
                {
                    return false;
                }
                let (Some(source), Some(admission)) = (
                    replay.source_token.as_deref(),
                    self.context_admission.as_ref(),
                ) else {
                    return false;
                };
                let projected_changes =
                    replay.original_suffix.chars().count() + replay.replacement.chars().count();
                if reset_is_current {
                    admission.align_completed_exact_replay_epoch(
                        source,
                        &pending.token,
                        replay.epoch,
                        pending.tail_epoch,
                        projected_changes,
                    )
                } else {
                    admission.completed_exact_replay_boundary_is_current(
                        source,
                        &pending.token,
                        replay.epoch,
                        pending.tail_epoch,
                        projected_changes,
                    )
                }
            })();
            if !aligned || !self.consume_exact_replay_reset_provenance(replay) {
                self.discard_context_reset_rereceipt();
                return;
            }
        }
        if let Some(pending) = self.context_reset_rereceipt.as_mut() {
            pending.confirmed_native_replay_reset_ack =
                completed_replay
                    .as_ref()
                    .map(|_| ConfirmedNativeReplayResetAck {
                        focus_serial: self.client_context.focus_serial,
                        runtime_owner_lease_identity: self
                            .client_context
                            .runtime_owner_lease_identity,
                        layout_generation: self.layout_gesture.layout_generation,
                    });
            pending.native_mutation_reset_evidence = erasure_reset_ack;
            pending.completed_replay = None;
            pending.confirmed = true;
            pending.awaiting_owned_append_receipt = false;
            pending.published_preedit = None;
            pending.armed_revision = self.client_context.surrounding_observation_revision;
        }
        trace::record(format!(
            r#"{{"kind":"ibus_context_reset_rereceipt","stage":"confirmed","tail_epoch":{},"suffix_chars":{}}}"#,
            pending.tail_epoch, pending.observed_suffix_chars,
        ));
    }

    fn context_reset_rereceipt_strict_prefix_is_current(
        &self,
        pending: &PendingContextResetRereceipt,
    ) -> bool {
        pending.tail_epoch == self.committed_tail.epoch
            && pending.token_text == self.last_tail_with_boundary()
            && self.context_token.as_ref() == Some(&pending.token)
            && self
                .context_owner
                .as_ref()
                .is_some_and(|owner| pending.token.matches_owner(owner))
            && self
                .context_word_scope
                .as_ref()
                .is_some_and(|scope| pending.token.matches_word_scope(scope))
            && self
                .context_admission
                .as_ref()
                .is_some_and(|admission| admission.revalidate(&pending.token))
            && self
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .is_some_and(|snapshot| {
                    snapshot_exactly_bounds_strict_token_prefix(snapshot, &pending.token_text)
                })
    }

    fn managed_strict_prefix_echo_identity_is_current(
        &self,
        pending: &PendingContextResetRereceipt,
    ) -> bool {
        pending.completed_replay.is_none()
            && self.composition.word_input_mode == Some(WordInputMode::ManagedCommit)
            && !self.exact_replay_tail_change_quarantined
            && !self.exact_replay_quarantine_active()
            && !matches!(
                self.committed_tail.autocorrect_suppression.as_ref(),
                Some(AutocorrectSuppression::ExactReplay(_))
            )
            && self.shared.lock().is_ok_and(|shared| {
                !matches!(
                    shared.autocorrect_suppression.as_ref(),
                    Some(AutocorrectSuppression::ExactReplay(_))
                )
            })
            && self.context_token.as_ref() == Some(&pending.token)
            && self.context_reset_rereceipt_identity_is_current()
    }

    pub(crate) fn context_reset_rereceipt_exact_manual_handoff_allowed(&self) -> bool {
        self.context_reset_rereceipt
            .as_ref()
            .is_some_and(|pending| pending.confirmed)
            && self.context_reset_rereceipt_identity_is_current()
            && self.context_reset_rereceipt_matches_current_snapshot()
    }

    #[cfg(test)]
    pub(crate) fn context_has_native_replay_reset_ack(&self) -> bool {
        self.context_reset_rereceipt
            .as_ref()
            .is_some_and(|pending| pending.confirmed_native_replay_reset_ack.is_some())
    }

    /// Native callbacks settle the projected tail before the client necessarily
    /// reports it. Stock fields can retain their locally tracked manual source;
    /// this does not acknowledge client delivery or authorize learning.
    pub(crate) fn completed_native_replay_awaits_exact_receipt(&self) -> bool {
        if self.context_stock_owned_manual_source_is_current() {
            return false;
        }
        if self.context_word_scope.as_ref().is_none_or(|scope| {
            scope.lineage().completeness != WordCompleteness::KnownStart
                || scope.lineage().observed_suffix_chars != 0
        }) {
            return false;
        }
        let Some(AutocorrectSuppression::ExactReplay(replay)) =
            self.committed_tail.autocorrect_suppression.as_ref()
        else {
            return false;
        };
        if replay.path != self.path
            || replay.owner_lease_identity != self.client_context.runtime_owner_lease_identity
            || replay.target_layout_is_ru != self.layout_gesture.layout_is_ru
            || self.committed_tail.epoch.checked_sub(replay.epoch)
                != Some(
                    (replay.original_suffix.chars().count() + replay.replacement.chars().count())
                        as u64,
                )
        {
            return false;
        }
        !self.context_reset_rereceipt_exact_manual_handoff_allowed()
            && !self
                .context_reset_rereceipt
                .as_ref()
                .is_some_and(|pending| {
                    pending.confirmed
                        && self.context_token.as_ref() == Some(&pending.token)
                        && self.context_reset_rereceipt_matches_current_snapshot()
                        && pending
                            .confirmed_native_replay_reset_ack
                            .as_ref()
                            .is_some_and(|ack| {
                                self.confirmed_native_replay_reset_ack_is_current(ack)
                            })
                })
    }

    /// A manual gesture may ask the same client for a new exact receipt while
    /// its display-only preedit leaves one editor sentinel after the caret.
    /// This permits a refresh request only; it never authorizes deletion.
    pub(crate) fn context_reset_rereceipt_manual_refresh_allowed(&self) -> bool {
        self.context_reset_rereceipt
            .as_ref()
            .is_some_and(|pending| {
                !pending.confirmed
                    && ((self.client_context.exact_surrounding_refresh_available
                        && self.context_reset_rereceipt_identity_is_current()
                        && self.composition.preedit_visible
                        && self.composition.preedit_display_only_pending
                        && !self.composition.preedit_suffix.is_empty()
                        && self
                            .client_context
                            .surrounding_text_snapshot
                            .as_ref()
                            .is_some_and(|snapshot| {
                                snapshot_has_transient_zero_width_boundary(snapshot, pending)
                            }))
                        || (self.completed_native_replay_manual_wait_allowed()
                            && self
                                .client_context
                                .surrounding_text_snapshot
                                .as_ref()
                                .is_some_and(|snapshot| {
                                    snapshot.cursor_pos as usize == snapshot.text.chars().count()
                                        && self.exact_replay_contains_prior_snapshot(snapshot)
                                        // Ordinary clients may request a read from the whole
                                        // old surface or whole new word missing its boundary.
                                        // An intermediate replay prefix cannot start a gesture.
                                        && (self.client_context.exact_surrounding_refresh_available
                                            || match self.committed_tail.autocorrect_suppression.as_ref() {
                                                Some(AutocorrectSuppression::ExactReplay(replay)) => {
                                                    replay.observed_external_prefix.as_deref()
                                                        .and_then(|prefix| snapshot.text.strip_prefix(prefix))
                                                        .is_some_and(|relative| {
                                                            relative == replay.original_tail
                                                                || relative == format!("{}{}",
                                                                    replay.unchanged_prefix,
                                                                    replay.replacement.trim_end_matches(char::is_whitespace))
                                                        })
                                                }
                                                _ => false,
                                            })
                                })))
            })
    }

    // A completed paired replay can request another actual receipt while its
    // client still reports a prior surface. Surrounding-text support suffices
    // to request actual data; it does not imply an exact-refresh capability.
    // This grants no edit or prediction
    // authority; only the existing final exact receipt can release the gesture.
    fn completed_native_replay_manual_wait_allowed(&self) -> bool {
        let Some(pending) = self.context_reset_rereceipt.as_ref() else {
            return false;
        };
        if !self.client_context.surrounding_text_supported
            || !(self.context_reset_rereceipt_identity_is_current()
                || self.context_reset_rereceipt_boundary_is_current(pending))
        {
            return false;
        }
        if pending.confirmed {
            return self.context_reset_rereceipt_exact_manual_handoff_allowed()
                && pending
                    .confirmed_native_replay_reset_ack
                    .as_ref()
                    .is_some_and(|ack| self.confirmed_native_replay_reset_ack_is_current(ack));
        }
        self.completed_exact_replay_boundary_provenance()
            .or_else(|| self.exact_replay_reset_provenance(true))
            .is_some()
    }

    pub(crate) fn context_reset_rereceipt_manual_wait_allowed(&self) -> bool {
        self.context_reset_rereceipt_computation_allowed()
            || self.completed_native_replay_manual_wait_allowed()
    }

    /// A single editor sentinel after the caret can invalidate the surrounding
    /// receipt while leaving a completion published for this exact token on
    /// screen. It proves only an explicit append of that visible suffix.
    pub(crate) fn context_reset_rereceipt_visible_append_suffix(&self) -> Option<String> {
        let pending = self.context_reset_rereceipt.as_ref()?;
        let suffix = &self.composition.preedit_suffix;
        if pending.confirmed
            || !self.context_reset_rereceipt_identity_is_current()
            || !self.composition.preedit_visible
            || !self.composition.preedit_display_only_pending
            || self.composition.preedit_dirty
            || !self.composition.preedit_candidates.is_empty()
            || self.selected_precognition_replacement().is_some()
            || suffix.is_empty()
            || suffix == "*"
            || self.client_context.surrounding_observation_revision != pending.armed_revision
            || !self
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .is_some_and(|snapshot| {
                    snapshot_has_transient_zero_width_boundary(snapshot, pending)
                })
        {
            return None;
        }
        let published = pending.published_preedit.as_ref()?;
        let displayed = self.visible_precognition_suffix(suffix.clone());
        (published.prefix_chars == pending.observed_suffix_chars
            && published.text == format!("{}{displayed}", pending.token_text))
        .then(|| suffix.clone())
    }

    /// The observed suffix may be computed before the client supplies its exact
    /// receipt. This does not grant display or acceptance authority.
    pub(crate) fn context_reset_rereceipt_computation_allowed(&self) -> bool {
        self.context_reset_rereceipt_identity_is_current()
            && !self.exact_replay_quarantine_active()
            && !self.committed_tail.buffer.ends_with(char::is_whitespace)
    }

    pub(crate) fn context_reset_rereceipt_space_identity_token(&self) -> Option<AdmissionToken> {
        let pending = self.context_reset_rereceipt.as_ref()?;
        self.context_reset_rereceipt_identity_is_current()
            .then(|| pending.predecessor_token.clone())
    }

    pub(crate) fn record_context_reset_preedit_publication(&mut self, text: &str, cursor: u32) {
        // A legacy key publishes its shortened completion before the post-key
        // Reset lineage is advanced. During that callback the tail is one
        // owned character ahead of pending; after settlement both epochs
        // match. Preserve the prior surface in either phase only when this
        // publication is its exact shortening. It remains an inert witness.
        let tail_token = self.last_tail_token_text();
        let retain_identical_unconfirmed_surface = cursor == 0
            && !text.is_empty()
            && self
                .context_reset_rereceipt
                .as_ref()
                .is_some_and(|pending| {
                    !pending.confirmed
                        && pending.published_preedit.as_ref().is_some_and(|published| {
                            published.prefix_chars as usize <= pending.token_text.chars().count()
                                && published
                                    .text
                                    .chars()
                                    .skip(published.prefix_chars as usize)
                                    .eq(text.chars())
                        })
                });
        let retain_unconfirmed_prior_surface = cursor == 0
            && !text.is_empty()
            && self
                .context_reset_rereceipt
                .as_ref()
                .is_some_and(|pending| {
                    !pending.confirmed
                        && pending.published_preedit.as_ref().is_some_and(|published| {
                            let prefix_chars = published.prefix_chars as usize;
                            let prefix_is_owned = prefix_chars
                                <= pending.token_text.chars().count()
                                && pending
                                    .token_text
                                    .chars()
                                    .take(prefix_chars)
                                    .eq(published.text.chars().take(prefix_chars));
                            prefix_is_owned
                                && ((published.prefix_chars < pending.observed_suffix_chars
                                    && self.context_reset_rereceipt_identity_is_current())
                                    || (pending.tail_epoch.checked_add(1)
                                        == Some(self.committed_tail.epoch)
                                        && tail_token.starts_with(&pending.token_text)
                                        && tail_token.chars().count()
                                            == pending.observed_suffix_chars as usize + 1))
                        })
                });
        let retain_retired_surface = cursor == 0
            && !text.is_empty()
            && self
                .context_reset_rereceipt
                .as_ref()
                .is_some_and(|pending| {
                    pending.confirmed
                        && pending.published_preedit.as_ref().is_some_and(|published| {
                            published.text.strip_prefix(&tail_token) == Some(text)
                                && ((self.context_reset_rereceipt_identity_is_current()
                                    && published.prefix_chars < pending.observed_suffix_chars)
                                    || (pending.tail_epoch.checked_add(1)
                                        == Some(self.committed_tail.epoch)
                                        && tail_token.starts_with(&pending.token_text)
                                        && tail_token.chars().count()
                                            == pending.observed_suffix_chars as usize + 1
                                        && published.prefix_chars <= pending.observed_suffix_chars))
                        })
                });
        let published = (cursor == 0
            && !text.is_empty()
            && self.context_reset_rereceipt_exact_manual_handoff_allowed())
        .then_some(self.context_reset_rereceipt.as_ref())
        .flatten()
        .filter(|pending| {
            pending.observed_suffix_chars as usize + text.chars().count()
                <= crate::preedit::PREEDIT_TAIL_LIMIT
        })
        .map(|pending| PublishedPreeditWitness {
            prefix_chars: pending.observed_suffix_chars,
            text: format!("{}{text}", pending.token_text),
        });
        if let Some(pending) = self.context_reset_rereceipt.as_mut() {
            if published.is_some()
                || !(retain_retired_surface
                    || retain_identical_unconfirmed_surface
                    || retain_unconfirmed_prior_surface)
            {
                pending.published_preedit = published;
            }
        }
    }

    fn context_reset_rereceipt_identity_is_current(&self) -> bool {
        if self.native_cursor_edit_receipt().is_some() {
            return false;
        }
        let Some(pending) = self.context_reset_rereceipt.as_ref() else {
            return false;
        };
        if self.context_handoff_sealed
            || self.atomic.active
            || !self.composition.buffer.is_empty()
            || self.content_is_sensitive()
            || self.committed_tail.epoch != pending.tail_epoch
            || self.last_tail_with_boundary() != pending.token_text
        {
            return false;
        }
        let Some(owner) = self.context_owner.as_ref() else {
            return false;
        };
        let Some(admission) = self.context_admission.as_ref() else {
            return false;
        };
        if pending.predecessor_token == pending.token
            || !pending.predecessor_token.matches_owner(owner)
            || !pending.token.matches_owner(owner)
            || admission.revalidate(&pending.predecessor_token)
            || !admission.revalidate(&pending.token)
        {
            return false;
        }
        let Some(scope) = self.context_word_scope.as_ref() else {
            return false;
        };
        if !pending.token.matches_word_scope(scope)
            || scope.lineage().completeness != WordCompleteness::UnknownStart
            || scope.lineage().observed_suffix_chars >= pending.observed_suffix_chars
        {
            return false;
        }
        true
    }

    fn context_reset_rereceipt_boundary_is_current(
        &self,
        pending: &PendingContextResetRereceipt,
    ) -> bool {
        if pending.confirmed
            || self.context_handoff_sealed
            || self.atomic.active
            || !self.composition.buffer.is_empty()
            || self.content_is_sensitive()
            || !self.committed_tail.buffer.ends_with(char::is_whitespace)
            || self.committed_tail.epoch != pending.tail_epoch
            || self.last_tail_with_boundary() != pending.token_text
            || pending.observed_suffix_chars as usize != pending.token_text.chars().count()
            || self.context_token.as_ref() != Some(&pending.token)
        {
            return false;
        }
        let (Some(owner), Some(scope), Some(admission)) = (
            self.context_owner.as_ref(),
            self.context_word_scope.as_ref(),
            self.context_admission.as_ref(),
        ) else {
            return false;
        };
        scope.lineage().completeness == WordCompleteness::KnownStart
            && pending.token.matches_owner(owner)
            && pending.token.matches_word_scope(scope)
            && pending.predecessor_token.matches_owner(owner)
            && pending.predecessor_token != pending.token
            && !admission.revalidate(&pending.predecessor_token)
            && admission.revalidate(&pending.token)
    }

    pub(crate) fn consume_context_reset_rereceipt_for_exact_manual_handoff(&mut self) -> bool {
        if !self.context_reset_rereceipt_exact_manual_handoff_allowed() {
            self.context_reset_rereceipt = None;
            return false;
        }
        let Some(pending) = self.context_reset_rereceipt.take() else {
            return false;
        };
        if self.context_bridge_token.as_ref() != Some(&pending.token) {
            self.revoke_context_word();
            return false;
        }
        let (Some(admission), Some(owner)) =
            (self.context_admission.clone(), self.context_owner.clone())
        else {
            return false;
        };
        let tail_epoch = self.committed_tail.epoch;
        let content_purpose = self.client_context.content_purpose;
        let content_hints = self.client_context.content_hints;
        let Some(settled) = self.context_word_scope.as_mut().and_then(|scope| {
            scope
                .observe_soft_reset_rereceipt(pending.observed_suffix_chars)
                .then(|| {
                    SettledWordState::from_scope(tail_epoch, scope)
                        .with_content_type(content_purpose, content_hints)
                })
        }) else {
            return false;
        };
        let Some(token) = admission.settle_soft_reset_rereceipt(&owner, &pending.token, settled)
        else {
            if let Some(scope) = self.context_word_scope.as_mut() {
                scope.revoke_for_input_gap();
            }
            self.context_token = None;
            return false;
        };
        self.context_bridge_token = Some(token.clone());
        self.context_token = Some(token);
        trace::record(format!(
            r#"{{"kind":"ibus_context_reset_rereceipt","stage":"consumed","tail_epoch":{},"suffix_chars":{}}}"#,
            pending.tail_epoch, pending.observed_suffix_chars,
        ));
        true
    }

    fn context_reset_rereceipt_matches_current_snapshot(&self) -> bool {
        let Some(pending) = self.context_reset_rereceipt.as_ref() else {
            return false;
        };
        let Some(snapshot) = self.client_context.surrounding_text_snapshot.as_ref() else {
            return false;
        };
        snapshot_exactly_bounds_token(
            snapshot,
            &pending.token_text,
            pending.observed_suffix_chars as usize,
        )
    }

    fn retire_context_word_locally(&mut self) {
        self.committed_tail.observed_boundary_source = None;
        self.layout_gesture.native_letter_release_focus.clear();
        self.exact_manual_target_snapshot = None;
        self.discard_context_reset_rereceipt();
        if let Some(scope) = self.context_word_scope.as_mut() {
            scope.revoke_for_input_gap();
        }
        self.context_token = None;
        self.committed_tail.pending_completion_learning = None;
        self.clear_preedit_completion_state();
    }

    pub(crate) fn revoke_context_word(&mut self) {
        self.retire_context_word_locally();
        if let (Some(admission), Some(owner)) =
            (self.context_admission.as_ref(), self.context_owner.as_ref())
        {
            if admission.ready_reset_replaces(owner) {
                // The observer already discarded this word and published its
                // source-free replacement. Do not let the delayed Set handler
                // revoke that successor before it can be installed.
                self.context_token = None;
            } else {
                admission.revoke_current_owner(owner);
                self.context_token = admission.current_token();
            }
        } else {
            self.context_token = None;
        }
    }
}

fn observed_tail_append_length(before: &str, after: &str) -> Option<u32> {
    let (last, _) = after.char_indices().next_back()?;
    let prefix = &after[..last];
    let retained_chars = after.chars().count();
    (prefix == before
        || (retained_chars == crate::preedit::PREEDIT_TAIL_LIMIT && before.ends_with(prefix)))
    .then_some(retained_chars as u32)
}

fn observed_tail_append_effect(before: &str, after: &str) -> Option<(u32, u32, Option<u32>)> {
    if before == after {
        return None;
    }
    if let Some(appended) = after
        .strip_prefix(before)
        .filter(|appended| !appended.is_empty())
    {
        let before_chars = before.chars().count();
        let retained_chars = after.chars().count();
        let appended_chars = appended.chars().count();
        let boundary = appended
            .chars()
            .enumerate()
            .filter(|(_, ch)| crate::preedit::is_observed_word_boundary(*ch))
            .last()
            .and_then(|(offset, _)| u32::try_from(before_chars.saturating_add(offset)).ok());
        return u32::try_from(retained_chars)
            .ok()
            .zip(u32::try_from(appended_chars).ok())
            .map(|(retained, appended)| (retained, appended, boundary));
    }
    observed_tail_append_length(before, after).map(|retained| {
        let boundary = after
            .chars()
            .enumerate()
            .last()
            .filter(|(_, ch)| crate::preedit::is_observed_word_boundary(*ch))
            .and_then(|(offset, _)| u32::try_from(offset).ok());
        (retained, 1, boundary)
    })
}

#[cfg(test)]
#[path = "../context_runtime/tests.rs"]
mod tests;

impl LayIbusEngine {
    pub(crate) async fn refresh_terminal_focus_if_ready(&mut self) {
        let focus_serial = self.client_context.focus_serial;
        let focus_receipt = self.terminal_input_focus_receipt();
        if !(5..=32).contains(&self.client_context.cursor_cell_width)
            || !self.client_context.preedit_text_supported
            || self.client_context.exact_surrounding_refresh_available
            || self.client_context.content_purpose != 0
            || focus_receipt.is_none()
            || self.client_context.terminal_focus_receipt.as_deref() == focus_receipt.as_deref()
            || self.client_context.terminal_focus_probe_serial == Some(focus_serial)
        {
            return;
        }
        self.client_context.terminal_focus_probe_serial = Some(focus_serial);
        let backend = self.config.active_layout_backend();
        let started = std::time::Instant::now();
        let is_terminal = super::desktop::focused_window_is_native_terminal(backend).await;
        if self.finish_terminal_focus_probe(focus_receipt.as_deref(), is_terminal) {
            let source = if backend == lay::desktop::LayoutBackend::Kde {
                "canonical_kwin_terminal_window"
            } else {
                "gnome_exact_kitty_window"
            };
            trace::record(format!(
                r#"{{"kind":"ibus_terminal_focus","source":"{source}","elapsed_us":{}}}"#,
                started.elapsed().as_micros(),
            ));
        }
    }

    pub(crate) fn set_client_capabilities(&mut self, caps: u32) {
        let surrounding_text_was_supported = self.client_context.surrounding_text_supported;
        let preedit_text_was_supported = self.client_context.preedit_text_supported;
        let exact_surrounding_refresh_was_available =
            self.client_context.exact_surrounding_refresh_available;
        let legacy_word_preedit_was_supported = preedit_text_was_supported
            && !surrounding_text_was_supported
            && !exact_surrounding_refresh_was_available;
        self.client_context.surrounding_text_supported = caps & IBUS_CAP_SURROUNDING_TEXT != 0;
        self.client_context.preedit_text_supported = caps & IBUS_CAP_PREEDIT_TEXT != 0;
        self.client_context.exact_surrounding_refresh_available =
            caps & IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH != 0;
        if surrounding_text_was_supported != self.client_context.surrounding_text_supported
            || preedit_text_was_supported != self.client_context.preedit_text_supported
            || exact_surrounding_refresh_was_available
                != self.client_context.exact_surrounding_refresh_available
        {
            self.cancel_native_autocorrection_restoration();
        }
        if preedit_text_was_supported != self.client_context.preedit_text_supported {
            self.discard_exact_replay_reset_seed();
            self.composition.legacy_preedit_start_boundary = None;
        }
        let legacy_word_preedit_is_supported = self.client_context.preedit_text_supported
            && !self.client_context.surrounding_text_supported
            && !self.client_context.exact_surrounding_refresh_available;
        if legacy_word_preedit_was_supported != legacy_word_preedit_is_supported
            || exact_surrounding_refresh_was_available
                != self.client_context.exact_surrounding_refresh_available
        {
            self.discard_exact_replay_reset_seed();
            self.client_context.managed_word_start = None;
            self.composition.legacy_preedit_start_boundary = None;
            self.invalidate_space_autocorrect_path();
            if preedit_text_was_supported
                && !self.client_context.preedit_text_supported
                && self.composition.legacy_word_preedit_active
            {
                self.cancel_precognition_display_generation();
                self.discard_legacy_word_preedit_ownership();
            }
        }
        if surrounding_text_was_supported != self.client_context.surrounding_text_supported {
            self.discard_context_reset_rereceipt();
            self.advance_surrounding_observation_revision();
        }
        if !surrounding_text_was_supported
            && self.client_context.surrounding_text_supported
            && self.composition.word_input_mode == Some(WordInputMode::TerminalPassthrough)
            && !self.uses_native_terminal_input()
        {
            self.composition.word_input_mode = Some(WordInputMode::ManagedCommit);
            // Retire only an already visible native-mode preedit. A worker
            // may publish the first current suggestion after this capability
            // callback; the next key must not clear that new publication.
            self.composition.pending_passthrough_preedit_clear =
                self.composition.suggestion_display_visible();
        }
        if !self.client_context.surrounding_text_supported {
            self.revoke_pending_fresh_manual_source();
            self.client_context.surrounding_text_snapshot = None;
            self.client_context.managed_word_start = None;
            self.layout_gesture.pending_manual_toggle = false;
            self.layout_gesture.pending_manual_refresh_at = None;
        }
    }
    pub(crate) fn set_content_type_state(&mut self, purpose: u32, hints: u32) {
        if self.client_context.content_purpose == purpose
            && self.client_context.content_hints == hints
        {
            return;
        }
        let personalization_was_allowed = self.content_allows_personalization();
        self.cancel_native_autocorrection_restoration();
        self.discard_context_reset_rereceipt();
        if self.client_context.content_purpose != purpose {
            self.composition.word_input_mode = None;
        }
        self.client_context.content_purpose = purpose;
        self.client_context.content_hints = hints;
        self.client_context.managed_word_start = None;
        self.invalidate_input_frame_background_work();
        self.clear_preedit_completion_state();
        if !self.content_allows_personalization() {
            self.committed_tail.pending_completion_learning = None;
            self.atomic.deferred_learning_actions.clear();
            if let Some(pending) = self.committed_tail.pending_visible_postcondition.as_mut() {
                pending.feedback = None;
            }
        }
        if self.content_is_sensitive()
            || personalization_was_allowed != self.content_allows_personalization()
        {
            // Retire only our volatile context. Private text must not survive
            // into a later public writer; this does not edit the client field.
            self.composition.buffer.clear();
            self.composition.cursor = 0;
            self.composition.legacy_word_preedit_active = false;
            self.client_context.surrounding_text_snapshot = None;
            self.atomic.deferred_learning_actions.clear();
            self.close_committed_tail_field();
        }
    }
    pub(crate) fn content_is_sensitive(&self) -> bool {
        matches!(
            self.client_context.content_purpose,
            IBUS_INPUT_PURPOSE_PASSWORD | IBUS_INPUT_PURPOSE_PIN
        ) || self.client_context.content_hints & IBUS_INPUT_HINT_HIDDEN_TEXT != 0
    }
    pub(crate) fn content_allows_personalization(&self) -> bool {
        !self.content_is_sensitive()
            && self.client_context.content_hints & IBUS_INPUT_HINT_PRIVATE == 0
    }
    pub(crate) fn content_allows_text_assistance(&self) -> bool {
        self.content_allows_personalization()
    }
    pub(crate) fn observe_external_surrounding_text(
        &mut self,
        snapshot: Option<SurroundingTextSnapshot>,
    ) {
        self.client_context.native_document_binding = None;
        // The inherited receiver receipt is current until this callback. Save
        // it before publishing the callback fact that retires its inheritance.
        let manual_snapshot = self.current_exact_manual_source_snapshot();
        self.client_context.surrounding_text_callback_observed = true;
        self.client_context.surrounding_text_supported = true;
        self.observe_document_snapshot_with_receipt(snapshot, manual_snapshot);
    }

    /// Applies actual document data from an explicitly identified transport.
    /// Native callers do not set either IBus capability/callback flag.
    pub(crate) fn observe_document_snapshot(&mut self, snapshot: Option<SurroundingTextSnapshot>) {
        let manual_snapshot = self.current_exact_manual_source_snapshot();
        self.observe_document_snapshot_with_receipt(snapshot, manual_snapshot);
    }

    fn observe_document_snapshot_with_receipt(
        &mut self,
        snapshot: Option<SurroundingTextSnapshot>,
        manual_snapshot: Option<super::super::context_admission::ExactManualSourceSnapshotReceipt>,
    ) {
        // Capture the separate Space proof before advancing its revision. An
        // actual prefix/preedit projection can retain this current inert start
        // without requiring the unrelated marked manual-toggle producer.
        let owned_space_pair_was_current = !self.atomic.active
            && self.composition.legacy_word_preedit_active
            && self.capture_space_boundary_pair_scope().is_some();
        let owned_prefix_was_current = self.client_context.exact_surrounding_refresh_available
            && self.context_owns_active_preedit_manual_toggle();
        let previous_revision = self.client_context.surrounding_observation_revision;
        self.advance_surrounding_observation_revision();
        let current_snapshot = if self.content_is_sensitive() {
            None
        } else {
            snapshot
        };
        let previous_snapshot = std::mem::replace(
            &mut self.client_context.surrounding_text_snapshot,
            current_snapshot,
        );
        // Same-field SSI may arrive before or after target installation. Move
        // only an existing live exact receipt to this actual matching revision;
        // matching text alone cannot recreate authority or extend its expiry.
        if let Some(receipt) = manual_snapshot {
            if previous_revision.checked_add(1)
                == Some(self.client_context.surrounding_observation_revision)
                && receipt.tail_epoch == self.committed_tail.epoch
                && receipt.tail == self.committed_tail.buffer
                && receipt.observed_external_prefix.is_some()
                && self.current_full_external_prefix() == receipt.observed_external_prefix
                && self.current_external_snapshot_agrees_with_owned_tail()
                && self.exact_manual_toggle_handoff_is_live()
            {
                if let (Some(admission), Some(token)) =
                    (self.context_admission.as_ref(), self.context_token.as_ref())
                {
                    let _ = admission.publish_exact_manual_snapshot(
                        token,
                        receipt.tail_epoch,
                        receipt.tail,
                        receipt.observed_external_prefix,
                        self.client_context.surrounding_observation_revision,
                        receipt.expires_at,
                        self.document_source_provenance(),
                    );
                }
            }
        }
        let owned_space_pair_rereceipt = owned_space_pair_was_current
            && self
                .composition
                .legacy_preedit_start_boundary
                .as_ref()
                .is_some_and(|witness| {
                    self.client_context
                        .surrounding_text_snapshot
                        .as_ref()
                        .is_some_and(|snapshot| {
                            if snapshot.has_selection() {
                                return false;
                            }
                            let Some(after_prefix) =
                                snapshot.text.strip_prefix(witness.snapshot_prefix.as_str())
                            else {
                                return false;
                            };
                            (snapshot.cursor_pos == witness.start_cursor
                                && after_prefix == witness.snapshot_suffix)
                                || (u32::try_from(self.composition.buffer.chars().count())
                                    .ok()
                                    .and_then(|chars| witness.start_cursor.checked_add(chars))
                                    == Some(snapshot.cursor_pos)
                                    && after_prefix.strip_prefix(self.composition.buffer.as_str())
                                        == Some(witness.snapshot_suffix.as_str()))
                        })
                });
        // Only carry an already-current witness across this adjacent actual
        // receipt. Matching text can never repair a stale/missing start proof.
        if ((owned_prefix_was_current
            && previous_snapshot.is_some()
            && previous_snapshot == self.client_context.surrounding_text_snapshot)
            || owned_space_pair_rereceipt)
            && previous_revision.checked_add(1)
                == Some(self.client_context.surrounding_observation_revision)
        {
            if let Some(witness) = self.composition.legacy_preedit_start_boundary.as_mut() {
                if witness.armed_revision == previous_revision {
                    witness.armed_revision = self.client_context.surrounding_observation_revision;
                }
            }
        }
        // A native replay invalidates current client text on every key. Keep
        // its last actual observation only as an inert adjacent-progress proof;
        // never restore it as the current snapshot or an authorized edit range.
        // The paired scope may still be receiving replacement keys; completion
        // is required for the final receipt, not for retaining inert progress.
        let retained_progress =
            self.client_context
                .native_replay_progress
                .take()
                .filter(|receipt| {
                    receipt.observation_revision == previous_revision
                        && receipt.focus_serial == self.client_context.focus_serial
                        && receipt.layout_generation == self.layout_gesture.layout_generation
                        && self
                            .completed_exact_replay_boundary_provenance()
                            .or_else(|| self.exact_replay_reset_provenance(false))
                            .as_ref()
                            == Some(&receipt.scope)
                });
        self.bind_exact_replay_external_prefix_from_snapshot();
        self.reconcile_managed_word_start_after_surrounding_observation();
        self.observe_context_reset_rereceipt_surrounding_text(
            previous_snapshot.as_ref(),
            previous_revision,
            retained_progress.as_ref().map(|receipt| &receipt.snapshot),
        );
        self.confirm_manual_autocorrection_restoration();
    }
    pub(crate) fn advance_surrounding_observation_revision(&mut self) {
        self.committed_tail.observed_boundary_source = None;
        self.exact_manual_target_snapshot = None;
        if let (Some(admission), Some(owner)) =
            (self.context_admission.as_ref(), self.context_owner.as_ref())
        {
            admission.invalidate_exact_manual_snapshot(owner);
        }
        self.client_context.surrounding_observation_revision = self
            .client_context
            .surrounding_observation_revision
            .saturating_add(1);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SurroundingTextSnapshot {
    pub(crate) text: String,
    pub(crate) cursor_pos: u32,
    pub(crate) anchor_pos: u32,
}
impl SurroundingTextSnapshot {
    pub(crate) fn new(text: String, cursor_pos: u32, anchor_pos: u32) -> Self {
        Self {
            text,
            cursor_pos,
            anchor_pos,
        }
    }

    pub(crate) fn suffix_before_cursor(&self, chars: usize) -> Option<String> {
        if chars == 0 {
            return Some(String::new());
        }
        let cursor = self.cursor_pos as usize;
        if cursor < chars || self.text.chars().count() < cursor {
            return None;
        }
        Some(
            self.text
                .chars()
                .take(cursor)
                .skip(cursor - chars)
                .collect(),
        )
    }

    pub(crate) fn has_selection(&self) -> bool {
        self.cursor_pos != self.anchor_pos
    }
}

/// Metadata of the existing successful managed CommitText, never a client
/// observation. Its first matched Reset spends eligibility even without SSI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ManagedCommitSnapshotFloor {
    pub(crate) focus_serial: u64,
    pub(crate) runtime_owner_lease_identity: u64,
    pub(crate) layout_generation: u64,
    pub(crate) tail_epoch: u64,
    pub(crate) observation_revision: u64,
    pub(crate) keycode: u32,
    pub(crate) reset_echo_available: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct NativeReplayProgressReceipt {
    pub(crate) scope: crate::protocol::ExactManualToggleSuppression,
    pub(crate) snapshot: SurroundingTextSnapshot,
    pub(crate) observation_revision: u64,
    pub(crate) focus_serial: u64,
    pub(crate) layout_generation: u64,
}

#[derive(Clone)]
pub(crate) struct ClientContextState {
    pub(crate) focus_receipt: Option<String>,
    pub(crate) focus_serial: u64,
    pub(crate) runtime_owner_lease_identity: u64,
    pub(crate) cursor_cell_width: i32,
    pub(crate) hint_cursor_geometry: Option<std::sync::Arc<crate::hint_view::HintCursorGeometry>>,
    pub(crate) terminal_focus_probe_serial: Option<u64>,
    pub(crate) terminal_focus_receipt: Option<String>,
    pub(crate) content_purpose: u32,
    pub(crate) content_hints: u32,
    pub(crate) surrounding_text_supported: bool,
    pub(crate) preedit_text_supported: bool,
    pub(crate) exact_surrounding_refresh_available: bool,
    pub(crate) surrounding_text_snapshot: Option<SurroundingTextSnapshot>,
    /// Provenance of the actual document snapshot above. The legacy snapshot
    /// field name does not turn native data into an IBus callback/capability.
    pub(crate) native_document_binding: Option<super::NativeDocumentBinding>,
    pub(crate) native_replay_progress: Option<NativeReplayProgressReceipt>,
    pub(crate) surrounding_observation_revision: u64,
    /// Latest successful local managed effect; only a later actual exact
    /// callback can bind text to it. Reset eligibility does not grant authority.
    pub(crate) managed_commit_snapshot_floor: Option<ManagedCommitSnapshotFloor>,
    pub(crate) managed_word_start: Option<ManagedWordStartWitness>,
    pub(crate) surrounding_text_callback_observed: bool,
    /// Initial registration can precede installed activation. The next live
    /// legacy callback publishes the installed mode without requiring text.
    pub(crate) input_mode_property_refresh_pending: bool,
    pub(crate) factory_engine_profile: lay::exact_layout_authority::FactoryEngineProfile,
    pub(crate) managed_input: bool,
}
impl ClientContextState {
    pub(crate) fn new(
        focus_receipt: Option<String>,
        factory_engine_profile: lay::exact_layout_authority::FactoryEngineProfile,
        managed_input: bool,
    ) -> Self {
        Self {
            focus_receipt,
            focus_serial: crate::engine::next_input_identity(),
            runtime_owner_lease_identity: crate::engine::next_input_identity(),
            cursor_cell_width: 0,
            hint_cursor_geometry: None,
            terminal_focus_probe_serial: None,
            terminal_focus_receipt: None,
            content_purpose: 0,
            content_hints: 0,
            surrounding_text_supported: false,
            preedit_text_supported: false,
            exact_surrounding_refresh_available: false,
            surrounding_text_snapshot: None,
            native_document_binding: None,
            native_replay_progress: None,
            surrounding_observation_revision: 0,
            managed_commit_snapshot_floor: None,
            managed_word_start: None,
            surrounding_text_callback_observed: false,
            input_mode_property_refresh_pending: false,
            factory_engine_profile,
            managed_input,
        }
    }
}

pub(crate) struct WindowInteraction;

pub(crate) enum WindowLifecycleEvent<'header, 'message> {
    FocusIn {
        header: &'header zbus::message::Header<'message>,
    },
    FocusInId {
        header: &'header zbus::message::Header<'message>,
        object_path: String,
        client: String,
    },
    FocusOut {
        header: &'header zbus::message::Header<'message>,
    },
    Disable {
        header: &'header zbus::message::Header<'message>,
    },
    Reset {
        header: &'header zbus::message::Header<'message>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContextRevocationReceipt {
    Refused,
    Cleanup,
    Matched,
}

/// Ignored metadata has no authority to run a later field's focus cleanup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextActivationObservation {
    Activated,
    Refused,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LifecycleReceipt {
    Focused { changed: bool },
    FocusedOut,
    Disabled,
    Reset,
    Refused,
}

pub(crate) enum WindowFactEvent<'header, 'message> {
    Capabilities(u32),
    ContentType {
        value: (u32, u32),
        header: Option<&'header zbus::message::Header<'message>>,
    },
    CursorGeometry {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
    SurroundingText(Option<SurroundingTextSnapshot>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ObservationReceipt {
    Capabilities,
    ContentType,
    CursorGeometry,
    SurroundingText(OutcomeProof),
    Refused,
}

impl WindowInteraction {
    pub(crate) fn observe_existing_postcondition(engine: &mut LayIbusEngine) -> OutcomeProof {
        engine.observe_visible_postcondition()
    }

    pub(crate) async fn process_legacy_key(
        engine: &mut LayIbusEngine,
        header: &zbus::message::Header<'_>,
        output: &mut EngineOutput<'_, '_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> fdo::Result<bool> {
        if !engine.legacy_key_route_allowed() {
            trace::record(r#"{"kind":"ibus_legacy_key_blocked","owner":"atomic"}"#);
            return Ok(false);
        }
        let callback_entered = Instant::now();
        let callback_serial = header.primary().serial_num().get();
        if trace::enabled() {
            let owner_generation = engine
                .context_admission
                .as_ref()
                .and_then(|admission| admission.current_owner())
                .map(|owner| owner.generation.0);
            let activation_generation = engine
                .context_admission
                .as_ref()
                .and_then(|admission| admission.current_activation_generation());
            trace::record_context_admission(
                "legacy_callback_enter",
                "ProcessKeyEvent",
                callback_serial,
                if is_key_press(state) {
                    "press"
                } else {
                    "release"
                },
                owner_generation,
                activation_generation,
                None,
            );
        }
        if crate::protocol::is_text_inert_legacy_release(keyval, state) {
            engine.try_install_pending_context_activation();
            let managed_release = engine.client_context.managed_input
                && engine
                    .layout_gesture
                    .handled_press_keycodes
                    .contains(&keycode);
            let native_release = engine.live_composition_enabled()
                && engine
                    .layout_gesture
                    .native_letter_release_focus
                    .get(&keycode)
                    == Some(&engine.client_context.focus_serial)
                && engine.uses_native_terminal_input()
                && engine.capture_space_autocorrect_frame_identity().is_some();
            let cursor_release = engine.native_cursor_release_is_current(keyval, keycode);
            let completed = (managed_release || native_release || cursor_release)
                && engine.context_owner.as_ref().is_some_and(|owner| {
                    engine.context_token.as_ref().is_some_and(|token| {
                        token.matches_owner(owner)
                            // A consumed managed press has no client key-up
                            // effect. Its current admission token is required,
                            // but local word publication may still lag Reset.
                            && (managed_release
                                || engine
                                    .context_word_scope
                                    .as_ref()
                                    .is_some_and(|scope| token.matches_word_scope(scope)))
                            && engine.context_admission.as_ref().is_some_and(|admission| {
                                admission.record_text_inert_release_completion(
                                    header,
                                    token,
                                    (
                                        engine.client_context.content_purpose,
                                        engine.client_context.content_hints,
                                    ),
                                    keyval,
                                    keycode,
                                    state,
                                )
                            })
                    })
                });
            if completed {
                // No await occurs between recording completion and retiring the
                // existing press. Later canonical ingress consumes that exact
                // fact; no new word, epoch, owner or receive order is invented.
                engine.context_callback_entered = Some(callback_entered);
                let handled = if managed_release {
                    let handled = engine.consume_handled_release(keycode);
                    engine.trace_key("managed_release", keyval, keycode, handled, None);
                    handled
                } else {
                    engine
                        .layout_gesture
                        .native_letter_release_focus
                        .remove(&keycode);
                    false
                };
                if cursor_release {
                    engine.rebind_native_cursor_after_key(keyval, keycode, state, handled, "");
                }
                engine.publish_pending_input_mode_property(output).await;
                let result = if handled {
                    engine.flush_dirty_preedit(output).await.map(|()| handled)
                } else {
                    Ok(false)
                };
                engine.context_callback_entered = None;
                return result;
            }
        }
        let observation = engine
            .begin_context_key_callback(header, callback_entered, false)
            .await;
        let (callback, passive_shift, pending_literal) = match observation {
            KeyCallbackObservation::Owned(callback) => (Some(*callback), false, None),
            KeyCallbackObservation::PassiveShift => (None, true, None),
            KeyCallbackObservation::PendingLiteral(receipt) => (None, false, Some(receipt)),
            KeyCallbackObservation::Refused => (None, false, None),
        };
        let pending_literal_observed = pending_literal.is_some();
        let pending_preedit_start = pending_literal
            .as_ref()
            .filter(|_| {
                engine.context_owner.is_none()
                    && is_key_press(state)
                    && engine.composition.buffer.is_empty()
            })
            .map(|receipt| (receipt.clone(), engine.client_context.focus_serial));
        if passive_shift && !is_shift_key(keyval) {
            engine.revoke_context_word();
            return Ok(false);
        }
        if callback.is_some() || passive_shift {
            engine.publish_pending_input_mode_property(output).await;
        }
        if trace::enabled() {
            trace::record_context_admission(
                "legacy_callback_admission",
                "ProcessKeyEvent",
                callback_serial,
                if callback.is_some() {
                    "accepted"
                } else if passive_shift {
                    "passive_shift"
                } else {
                    "refused"
                },
                engine
                    .context_owner
                    .as_ref()
                    .map(|owner| owner.generation.0),
                engine
                    .context_admission
                    .as_ref()
                    .and_then(|admission| admission.current_activation_generation()),
                None,
            );
        }
        if callback.is_some()
            && is_key_press(state)
            && !is_shift_key(keyval)
            && !Self::refresh_native_document(engine, output).await?
        {
            return Ok(false);
        }
        // Keep transport eligibility only on this callback's stack: whitespace
        // clears word_input_mode, so post-effect mode cannot identify its route.
        let native_space_eligible = callback.is_some()
            && output.is_legacy()
            && is_key_press(state)
            && keyval == KEY_SPACE
            && !has_command_modifier(state)
            && !engine.atomic.active
            && engine.composition.buffer.is_empty()
            && !engine.composition.legacy_word_preedit_active
            && !engine.uses_native_terminal_input()
            && engine
                .composition
                .word_input_mode
                .unwrap_or_else(|| engine.initial_word_input_mode())
                == WordInputMode::ManagedCommit
            && !engine.exact_replay_tail_change_quarantined
            && !engine.exact_replay_quarantine_active()
            && !matches!(
                engine.committed_tail.autocorrect_suppression.as_ref(),
                Some(AutocorrectSuppression::ExactReplay(_))
            );
        let tail_before = engine.committed_tail.buffer.clone();
        let tail_epoch_before = engine.committed_tail.epoch;
        let native_backspace_identity = (callback.is_some()
            && is_key_press(state)
            && keyval == KEY_BACKSPACE
            && !has_command_modifier(state))
        .then(|| {
            engine
                .live_layout_intent_token()
                .map(|token| (token, engine.layout_gesture.layout_is_ru))
        })
        .flatten();
        let native_backspace_source = native_backspace_identity
            .as_ref()
            .and_then(|_| engine.live_context_token());
        let boundary_candidate = if native_backspace_identity.is_some() {
            engine
                .capture_native_backspace_reset_rereceipt_candidate()
                .or_else(|| engine.capture_owned_preedit_erasure_reset_rereceipt_candidate())
        } else {
            (callback.is_some()
                && (matches!(keyval, KEY_SPACE | KEY_TAB)
                    || is_accept_completion_with_space_key(keyval))
                && (engine.context_word_is_known()
                    || engine.context_observed_suffix_exact_manual_handoff_allowed()
                    || engine.context_reset_rereceipt_exact_manual_handoff_allowed()
                    || (keyval == KEY_SPACE
                        && is_key_press(state)
                        && engine
                            .context_reset_rereceipt
                            .as_ref()
                            .is_some_and(|pending| pending.confirmed)
                        && engine.context_reset_rereceipt_computation_allowed())
                    || engine.exact_replay_quarantine_active()))
            .then(|| engine.capture_context_reset_rereceipt_candidate())
            .flatten()
        };
        engine.exact_replay_tail_change_quarantined = false;
        engine.context_callback_entered = Some(callback_entered);
        engine.trace_client_output_metadata(
            "callback",
            "binding",
            trace::diagnostics_enabled_cached().then_some(callback_entered),
            None,
            None,
            Some(callback_serial),
        );
        engine.consume_shift_gesture_handoff();
        if callback.is_some() {
            let cursor_prepared = engine.prepare_native_cursor_edit(keyval, keycode, state);
            if !cursor_prepared && is_key_press(state) && matches!(keyval, KEY_LEFT | KEY_RIGHT) {
                engine.trace_native_cursor_phase("cursor_prepare_refused", keyval, keycode, state);
            }
        }
        let handled_release = !is_key_press(state)
            && engine
                .layout_gesture
                .handled_press_keycodes
                .contains(&keycode);
        let mut result = engine
            .process_key_event_with_output(output, keyval, keycode, state)
            .await;
        if result.is_ok() {
            let handled = result.as_ref().is_ok_and(|handled| *handled);
            let literal_append = is_key_press(state)
                && handled
                && !has_command_modifier(state)
                && engine.physical_char(keyval, keycode).is_some_and(|ch| {
                    (ch.is_alphabetic() || ch == ' ')
                        && engine.committed_tail.buffer.strip_prefix(&tail_before)
                            == Some(ch.to_string().as_str())
                });
            let literal_backspace = is_key_press(state)
                && handled
                && keyval == KEY_BACKSPACE
                && !has_command_modifier(state)
                && tail_before
                    .char_indices()
                    .next_back()
                    .is_some_and(|(last, _)| engine.committed_tail.buffer == tail_before[..last]);
            let unchanged_modifier_or_release = (!is_key_press(state) || is_shift_key(keyval))
                && engine.committed_tail.buffer == tail_before
                && engine.committed_tail.epoch == tail_epoch_before;
            if let Some(pending) = engine.composition.pending_activation_preedit.as_mut() {
                let same_acquisition = pending_literal.as_ref() == Some(&pending.receipt);
                if pending.focus_serial != engine.client_context.focus_serial
                    || pending.observed_tail_epoch != Some(tail_epoch_before)
                    || !(unchanged_modifier_or_release
                        || (same_acquisition && (literal_append || literal_backspace)))
                {
                    pending.observed_tail_epoch = None;
                } else {
                    pending.observed_tail_epoch = Some(engine.committed_tail.epoch);
                }
            }
            if handled
                && engine.composition.legacy_word_preedit_active
                && !engine.composition.buffer.is_empty()
                && engine.composition.buffer == engine.committed_tail.buffer
            {
                if let Some((receipt, focus_serial)) = pending_preedit_start {
                    engine.composition.pending_activation_preedit =
                        Some(super::super::engine::PendingActivationLiteralInput {
                            receipt,
                            focus_serial,
                            observed_tail_epoch: (tail_before.is_empty() && literal_append)
                                .then_some(engine.committed_tail.epoch),
                        });
                }
            }
            let held_replay_boundary = handled
                && keyval == KEY_SPACE
                && tail_before == engine.committed_tail.buffer
                && engine.has_held_replay_boundary();
            if held_replay_boundary {
                engine.settle_context_callback_without_input(callback.as_ref());
            } else if passive_shift {
                if handled || engine.committed_tail.buffer != tail_before {
                    engine.revoke_context_word();
                }
            } else if !pending_literal_observed {
                engine.settle_context_key_callback(
                    callback.as_ref(),
                    keyval,
                    keycode,
                    state,
                    &tail_before,
                    handled,
                );
            }
            let boundary_candidate = if keyval == KEY_BACKSPACE {
                boundary_candidate.filter(|_| {
                    native_backspace_identity
                        .as_ref()
                        .is_some_and(|(token, layout)| {
                            engine.live_layout_intent_token().as_ref() == Some(token)
                                && engine.layout_gesture.layout_is_ru == *layout
                        })
                })
            } else {
                boundary_candidate
            };
            let mut held_boundary_applied = false;
            if held_replay_boundary {
                match engine.complete_held_replay_boundary(output).await {
                    Ok(applied) => held_boundary_applied = applied,
                    Err(error) => result = Err(error),
                }
            } else {
                engine.advance_context_reset_rereceipt_after_key(
                    keyval,
                    keycode,
                    state,
                    &tail_before,
                    handled,
                    (
                        boundary_candidate,
                        native_space_eligible,
                        native_backspace_source,
                    ),
                );
            }
            if handled_release && handled {
                // Drain display only after the existing owner settles this
                // no-text-effect release. The stored frame must still match.
                if let Err(error) = engine.flush_dirty_preedit(output).await {
                    result = Err(error);
                }
            }
            if is_key_press(state)
                && !held_boundary_applied
                && tail_before != engine.committed_tail.buffer
                && !std::mem::take(&mut engine.exact_replay_tail_change_quarantined)
                && !(keyval == KEY_BACKSPACE && !handled)
            {
                let correction_frame = if (engine.uses_native_terminal_input()
                    || engine.composition.legacy_word_preedit_active)
                    && !engine.committed_tail.buffer.ends_with(char::is_whitespace)
                {
                    engine.capture_space_autocorrect_frame_identity()
                } else {
                    engine.capture_pending_reset_space_frame()
                };
                if let Some(identity) = correction_frame {
                    engine.schedule_space_autocorrect_prefetch(&identity);
                    engine.schedule_settled_owned_preedit_precognition(output, &identity);
                }
                if let Err(error) = engine.refresh_observed_suffix_precognition(output).await {
                    result = Err(error);
                }
            }
        } else {
            engine.revoke_context_word();
        }
        if let Some(receipt) = pending_literal.as_ref() {
            if let Some(admission) = engine.context_admission.clone() {
                if admission.complete_pending_literal_callback(header, receipt) && result.is_ok() {
                    // A Ready wake may already have run while this callback
                    // was queued. Reuse that same grant after its last literal
                    // callback has finished, with no timer or extra owner.
                    engine.try_install_pending_context_activation();
                }
            }
        }
        engine.context_callback_entered = None;
        result
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "preserve one-to-one forwarding of authenticated AtomicV1 wire arguments"
    )]
    pub(crate) async fn process_atomic_key(
        engine: &mut LayIbusEngine,
        header: &zbus::message::Header<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
        envelope: AtomicEnvelope,
        capability: AtomicCapability,
        prior_receipt: AtomicPriorReceipt,
    ) -> fdo::Result<AtomicProposal> {
        let callback_entered = Instant::now();
        let callback = engine
            .begin_context_key_callback(header, callback_entered, true)
            .await
            .into_key();
        engine.context_callback_entered = Some(callback_entered);
        let result = engine
            .process_atomic_key_event_with_context_tail(
                keyval,
                keycode,
                state,
                envelope,
                capability,
                prior_receipt,
            )
            .await;
        engine.context_callback_entered = None;
        match result.as_ref() {
            Ok((proposal, _))
                if matches!(
                    proposal.0,
                    PROPOSAL_FRAME_READY | PROPOSAL_CONSUMED_NO_EFFECT
                ) =>
            {
                if !engine.bind_atomic_context_callback(callback) {
                    engine.revoke_context_word();
                }
            }
            Ok((_, tail_before)) => {
                engine.settle_context_key_callback(
                    callback.as_ref(),
                    keyval,
                    keycode,
                    state,
                    tail_before,
                    false,
                );
            }
            Err(_) => engine.revoke_context_word(),
        }
        result.map(|(proposal, _)| proposal)
    }

    pub(crate) async fn observe_lifecycle(
        engine: &mut LayIbusEngine,
        event: WindowLifecycleEvent<'_, '_>,
        mut output: Option<&mut EngineOutput<'_, '_>>,
    ) -> fdo::Result<LifecycleReceipt> {
        match event {
            WindowLifecycleEvent::FocusIn { header } => {
                let changed = if engine.context_admission_required {
                    match engine
                        .observe_context_activation_from_header(header, Instant::now(), None)
                        .await
                    {
                        ContextActivationObservation::Activated => true,
                        ContextActivationObservation::Refused => false,
                        ContextActivationObservation::Ignored => {
                            return Ok(LifecycleReceipt::Focused { changed: false });
                        }
                    }
                } else {
                    engine.bind_focus_path()
                };
                Self::finish_focus_in(engine, changed);
                Ok(LifecycleReceipt::Focused { changed })
            }
            WindowLifecycleEvent::FocusInId {
                header,
                object_path,
                client,
            } => {
                let changed = engine.bind_focus_receipt(object_path, client);
                trace::record(if changed {
                    r#"{"kind":"ibus_focus","stage":"focus_in_id","receipt":"new"}"#
                } else {
                    r#"{"kind":"ibus_focus","stage":"focus_in_id","receipt":"same"}"#
                });
                let activated = if engine.context_admission_required {
                    let native_path = engine
                        .client_context
                        .focus_receipt
                        .as_deref()
                        .and_then(|receipt| receipt.split('\u{1f}').next())
                        .map(str::to_owned);
                    engine
                        .activate_context_from_header(
                            header,
                            Instant::now(),
                            native_path.as_deref(),
                        )
                        .await
                } else {
                    engine.bind_focus_path()
                };
                let changed = changed || activated;
                Self::finish_focus_in(engine, changed);
                Ok(LifecycleReceipt::Focused { changed })
            }
            WindowLifecycleEvent::FocusOut { header } => {
                if !engine
                    .observe_context_focus_out(header, Instant::now())
                    .await
                {
                    return Ok(LifecycleReceipt::Refused);
                }
                Self::finish_focus_out(engine);
                Ok(LifecycleReceipt::FocusedOut)
            }
            WindowLifecycleEvent::Disable { header } => {
                if !engine.observe_context_disable(header, Instant::now()).await {
                    return Ok(LifecycleReceipt::Refused);
                }
                engine.discard_atomic_pending();
                engine.atomic.active = false;
                engine.client_context.managed_word_start = None;
                trace::record(r#"{"kind":"ibus_focus","stage":"disable"}"#);
                engine.reset_for_ibus_soft_reset();
                Ok(LifecycleReceipt::Disabled)
            }
            WindowLifecycleEvent::Reset { header } => {
                let native_cursor_reset = engine.capture_native_cursor_reset();
                if native_cursor_reset.is_none() && engine.native_cursor_edit_receipt().is_some() {
                    engine.trace_native_cursor_phase("reset_capture_refused", 0, 0, 0);
                    trace::record(format!(
                        r#"{{"kind":"ibus_native_cursor_reset_capture","serial":{},"captured":{}}}"#,
                        header.primary().serial_num().get(),
                        native_cursor_reset.is_some(),
                    ));
                }
                let owned_observation = engine
                    .capture_owned_closing_space_observation()
                    .or_else(|| engine.capture_owned_managed_commit_reset_observation())
                    .or_else(|| engine.capture_confirmed_native_replay_reset_observation())
                    .or_else(|| engine.capture_confirmed_native_erasure_reset_observation());
                let owned_native_replay_ack = owned_observation
                    .as_ref()
                    .and_then(|candidate| candidate.confirmed_native_replay_reset_ack.clone());
                let owned_native_erasure_ack = owned_observation
                    .as_ref()
                    .and_then(|candidate| candidate.native_mutation_reset_evidence.clone());
                let owned_observation_source = owned_observation
                    .as_ref()
                    .map(|candidate| candidate.token.clone());
                let owned_preedit_source = (engine.composition.legacy_word_preedit_active
                    && owned_observation.is_none()
                    && !engine.layout_gesture.pending_manual_toggle
                    && !engine.exact_replay_quarantine_active()
                    && !engine.exact_replay_tail_change_quarantined)
                    .then(|| engine.context_token.clone())
                    .flatten();
                // Observer-first Reset already revoked the old token. Pin its
                // local owner lineage, then require an explicitly matched
                // callback and its live current token. Several queued Reset
                // callbacks may legitimately share the latest reducer token.
                let reset_identity = engine
                    .context_owner
                    .clone()
                    .zip(engine.context_token.clone())
                    .filter(|(owner, token)| {
                        engine.context_admission_required
                            && token.matches_owner(owner)
                            && engine
                                .context_word_scope
                                .as_ref()
                                .is_some_and(|scope| token.matches_word_scope(scope))
                    })
                    .map(|(owner, _)| {
                        (
                            owner,
                            engine.client_context.focus_serial,
                            engine.client_context.runtime_owner_lease_identity,
                            engine.layout_gesture.layout_generation,
                        )
                    });
                let same_reset_owner = |engine: &LayIbusEngine| {
                    reset_identity
                        .as_ref()
                        .is_some_and(|(owner, focus, lease, _)| {
                            engine.context_owner.as_ref() == Some(owner)
                                && engine.client_context.focus_serial == *focus
                                && engine.client_context.runtime_owner_lease_identity == *lease
                            // Layout/word changes revoke observations,
                            // independently of an already consumed key-up.
                        })
                };
                let handled_releases = engine.layout_gesture.handled_press_keycodes.clone();
                let reset_receipt = engine
                    .observe_context_revocation(header, Instant::now())
                    .await;
                if reset_receipt == ContextRevocationReceipt::Refused {
                    return Ok(LifecycleReceipt::Refused);
                }
                let post_reset_token = engine.live_context_token().filter(|token| {
                    reset_receipt == ContextRevocationReceipt::Matched
                        && same_reset_owner(engine)
                        && reset_identity
                            .as_ref()
                            .is_some_and(|(owner, _, _, _)| token.matches_owner(owner))
                });
                if post_reset_token.is_some() {
                    if let Some(effect) =
                        engine.client_context.managed_commit_snapshot_floor.as_mut()
                    {
                        effect.reset_echo_available = false;
                    }
                }
                engine.discard_atomic_pending();
                trace::record(r#"{"kind":"ibus_focus","stage":"reset"}"#);
                let held_shift = engine.layout_gesture.shift_active;
                let cleared = if engine.atomic.active {
                    Ok(())
                } else {
                    let output = output.as_deref_mut().ok_or_else(|| {
                        fdo::Error::Failed("Reset requires the existing output boundary".into())
                    })?;
                    engine.clear_preedit(output).await
                };
                if reset_receipt == ContextRevocationReceipt::Matched {
                    if let Some(receipt) = native_cursor_reset {
                        engine.restore_native_cursor_after_reset(receipt);
                    }
                }
                // Reset retires text authority even after an output error.
                // Keep a failed auxiliary hide pending for the next clear.
                let auxiliary_clear_pending = engine.composition.auxiliary_visible;
                let retires_owned_source = cleared.is_ok()
                    && post_reset_token.as_ref().is_some_and(|token| {
                        same_reset_owner(engine)
                            && engine.live_context_token().as_ref() == Some(token)
                    });
                let retired_owned_source = engine.reset_for_ibus_soft_reset_retiring_owned_source(
                    retires_owned_source
                        .then_some(owned_preedit_source.as_ref())
                        .flatten(),
                );
                engine.composition.auxiliary_visible = auxiliary_clear_pending;
                engine.restore_native_cursor_observation_after_reset();
                // Consumed releases are routing receipts, not text authority.
                // They have no observation TTL, and each is still consumed once.
                // Clear the unauthenticated helper's possible retention first.
                engine.layout_gesture.handled_press_keycodes.clear();
                let release_owner_is_current = |engine: &LayIbusEngine| {
                    post_reset_token.as_ref().is_some_and(|token| {
                        same_reset_owner(engine)
                            && engine
                                .context_admission
                                .as_ref()
                                .and_then(|admission| admission.current_token())
                                .is_some_and(|current| current.shares_input_activation(token))
                    })
                };
                if release_owner_is_current(engine) {
                    engine
                        .layout_gesture
                        .handled_press_keycodes
                        .extend(handled_releases);
                }
                engine.layout_gesture.shift_active = held_shift;
                cleared?;
                if post_reset_token.as_ref().is_some_and(|token| {
                    same_reset_owner(engine)
                        && reset_identity.as_ref().is_some_and(|(_, _, _, layout)| {
                            engine.layout_gesture.layout_generation == *layout
                        })
                        && engine.live_context_token().as_ref() == Some(token)
                }) {
                    engine.restore_owned_reset_observation(owned_observation);
                    if retired_owned_source {
                        if let (Some(admission), Some(source), Some(reset)) = (
                            engine.context_admission.as_ref(),
                            owned_preedit_source.as_ref(),
                            post_reset_token.as_ref(),
                        ) {
                            admission.admit_fresh_manual_source_after_owned_reset(source, reset);
                        }
                    }
                }
                // A browser can Reset in response to hiding our display-only
                // hint. The word is still open, but its prepared Space lease
                // needs a fresh client receipt after that revocation.
                if engine.context_reset_rereceipt.is_some()
                    && engine.client_context.exact_surrounding_refresh_available
                    && engine.client_context.surrounding_text_supported
                {
                    if let Some(output) = output {
                        let stage = if output.require_surrounding_text().await.is_ok() {
                            "requested_after_reset"
                        } else {
                            "reset_snapshot_request_failed"
                        };
                        trace::record(format!(
                            r#"{{"kind":"ibus_surrounding_refresh","stage":"{stage}"}}"#
                        ));
                    }
                }
                // The request can await transport while a later observer
                // revokes this token. Preserve release pairing only in the
                // same activation; never leave the old observation confirmed.
                if post_reset_token.as_ref().is_some_and(|token| {
                    !same_reset_owner(engine)
                        || engine.live_context_token().as_ref() != Some(token)
                        || owned_native_replay_ack.as_ref().is_some_and(|ack| {
                            !engine.confirmed_native_replay_reset_ack_is_current(ack)
                        })
                        || owned_native_erasure_ack
                            .as_ref()
                            .is_some_and(|ack| !engine.native_erasure_reset_ack_is_current(ack))
                        || owned_observation_source.as_ref().is_some_and(|source| {
                            !engine.context_admission.as_ref().is_some_and(|admission| {
                                admission.exact_replay_reset_provenance_is_current(source)
                            })
                        })
                }) {
                    engine.client_context.surrounding_text_snapshot = None;
                    if let Some(pending) = engine.context_reset_rereceipt.as_mut() {
                        pending.confirmed = false;
                        pending.confirmed_native_replay_reset_ack = None;
                        pending.native_mutation_reset_evidence = None;
                    }
                }
                if !release_owner_is_current(engine) {
                    engine.layout_gesture.handled_press_keycodes.clear();
                }
                Ok(LifecycleReceipt::Reset)
            }
        }
    }

    pub(crate) async fn observe_facts(
        engine: &mut LayIbusEngine,
        event: WindowFactEvent<'_, '_>,
        mut output: Option<&mut EngineOutput<'_, '_>>,
    ) -> fdo::Result<ObservationReceipt> {
        match event {
            WindowFactEvent::Capabilities(caps) => {
                engine.set_client_capabilities(caps);
                trace::record_capabilities(
                    caps,
                    engine.client_context.surrounding_text_supported,
                    engine.client_context.exact_surrounding_refresh_available,
                );
                Ok(ObservationReceipt::Capabilities)
            }
            WindowFactEvent::ContentType { value, header } => {
                let preserve_current_word = if let (Some(admission), Some(path), Some(header)) = (
                    engine.context_admission.clone(),
                    EnginePath::new(engine.path.clone()),
                    header,
                ) {
                    match admission
                        .observe_content_type_callback(&path, value, header, Instant::now())
                        .await
                    {
                        Ok(Some(preserve)) => preserve,
                        Ok(None) => return Ok(ObservationReceipt::Refused),
                        Err(_) => false,
                    }
                } else {
                    false
                };
                if !preserve_current_word {
                    engine.revoke_context_word();
                }
                engine.set_content_type_state(value.0, value.1);
                trace::record(format!(
                    r#"{{"kind":"ibus_content_type","purpose":{},"hints":{},"text_assistance":{}}}"#,
                    value.0,
                    value.1,
                    engine.content_allows_text_assistance()
                ));
                Ok(ObservationReceipt::ContentType)
            }
            WindowFactEvent::CursorGeometry {
                x,
                y,
                width,
                height,
            } => {
                engine.client_context.cursor_cell_width = width;
                trace::record_cursor_location(x, y, width, height);
                engine.refresh_terminal_focus_if_ready().await;
                engine.remember_hint_geometry(x, y, width, height);
                if !engine.atomic.active {
                    let output = output.as_deref_mut().ok_or_else(|| {
                        fdo::Error::Failed("cursor observation requires output".into())
                    })?;
                    Self::refresh_native_document_for_display(engine, output).await?;
                    engine.flush_dirty_preedit(output).await?;
                    engine.publish_pending_input_mode_property(output).await;
                    Box::pin(engine.refresh_inline_hint_geometry()).await?;
                }
                Ok(ObservationReceipt::CursorGeometry)
            }
            WindowFactEvent::SurroundingText(snapshot) => {
                let suffix_snapshot_changed = engine
                    .client_context
                    .surrounding_text_snapshot
                    .as_ref()
                    .map(|snapshot| (&snapshot.text, snapshot.cursor_pos, snapshot.anchor_pos))
                    != snapshot
                        .as_ref()
                        .map(|snapshot| (&snapshot.text, snapshot.cursor_pos, snapshot.anchor_pos));
                let exact_replay_quarantined = engine.exact_replay_quarantine_active();
                let (cursor_pos, anchor_pos) = snapshot.as_ref().map_or((0, 0), |snapshot| {
                    (snapshot.cursor_pos, snapshot.anchor_pos)
                });
                engine.observe_external_surrounding_text(snapshot);
                let retry_status = engine.pending_ime_auto_undo_retry_status();
                let sensitive = engine.content_is_sensitive();
                trace::record_surrounding_text_snapshot(
                    engine
                        .client_context
                        .surrounding_text_snapshot
                        .as_ref()
                        .map_or(0, |snapshot| snapshot.text.chars().count()),
                    if sensitive { 0 } else { cursor_pos },
                    if sensitive { 0 } else { anchor_pos },
                    retry_status,
                );
                if engine.atomic.active {
                    let outcome = Self::observe_existing_postcondition(engine);
                    return Ok(ObservationReceipt::SurroundingText(outcome));
                }
                let output = output.ok_or_else(|| {
                    fdo::Error::Failed("surrounding observation requires output".into())
                })?;
                if engine.complete_held_replay_boundary(output).await? {
                    return Ok(ObservationReceipt::SurroundingText(
                        OutcomeProof::ObservedSourceBoundary,
                    ));
                }
                if engine.commit_observed_literal_preedit(output).await? {
                    return Ok(ObservationReceipt::SurroundingText(
                        OutcomeProof::ExistingPostconditionPending,
                    ));
                }
                let boundary_applied = engine
                    .apply_space_boundary_after_client_commit(output)
                    .await?;
                if boundary_applied
                    || engine
                        .committed_tail
                        .pending_visible_postcondition
                        .as_ref()
                        .is_some_and(|receipt| receipt.space_boundary_commit.is_some())
                {
                    return Ok(ObservationReceipt::SurroundingText(
                        OutcomeProof::ExistingPostconditionPending,
                    ));
                }
                if engine
                    .apply_pending_manual_toggle_after_surrounding_snapshot(output)
                    .await?
                {
                    return Ok(ObservationReceipt::SurroundingText(OutcomeProof::Rejected));
                }
                if should_apply_auto_undo_before_postcondition(retry_status) {
                    let status = if engine.undo_last_ime_autocorrect(output).await?.is_some() {
                        "applied_after_causal_precondition_snapshot"
                    } else {
                        "causal_precondition_apply_failed"
                    };
                    trace::record_auto_undo_retry(status);
                }
                let outcome = Self::observe_existing_postcondition(engine);
                if outcome == OutcomeProof::ExistingPostconditionPending
                    && !sensitive
                    && engine.client_context.exact_surrounding_refresh_available
                    && engine
                        .committed_tail
                        .pending_visible_postcondition
                        .as_ref()
                        .is_some_and(|pending| {
                            (pending.expected_external_snapshot.is_some()
                                || pending.snapshot.source
                                    == lay::text_edit::VisibleTailSource::ImeActiveComposition)
                                && !pending.final_refresh_retry_sent
                        })
                {
                    engine
                        .committed_tail
                        .pending_visible_postcondition
                        .as_mut()
                        .expect("pending refreshable postcondition checked above")
                        .final_refresh_retry_sent = true;
                    let stage = if output.require_surrounding_text().await.is_ok() {
                        "requested_after_intermediate_snapshot"
                    } else {
                        "intermediate_snapshot_request_failed"
                    };
                    trace::record(format!(
                        r#"{{"kind":"ibus_surrounding_refresh","stage":"{stage}"}}"#
                    ));
                } else if suffix_snapshot_changed
                    && engine.completed_native_replay_awaits_exact_receipt()
                    && engine.context_reset_rereceipt_manual_refresh_allowed()
                    && (engine
                        .completed_exact_replay_boundary_provenance()
                        .is_some()
                        || engine.exact_replay_reset_provenance(true).is_some())
                {
                    // A whole actual replay surface may lag its final Space.
                    // Ask the client once per changed observation; duplicates
                    // cannot loop and this read grants no edit or receipt.
                    output.require_surrounding_text().await?;
                }
                if matches!(retry_status, "ready" | "ready_boundary_elided") {
                    let status = if engine.undo_last_ime_autocorrect(output).await?.is_some() {
                        if retry_status == "ready_boundary_elided" {
                            "applied_after_boundary_elided_snapshot"
                        } else {
                            "applied_after_exact_snapshot"
                        }
                    } else {
                        "snapshot_apply_failed"
                    };
                    trace::record_auto_undo_retry(status);
                }
                if suffix_snapshot_changed && !exact_replay_quarantined {
                    if engine.exact_marked_surrounding_suffix_is_current()
                        || engine.exact_managed_surrounding_word_is_current()
                    {
                        if let Some(identity) = engine.capture_space_autocorrect_frame_identity() {
                            engine.schedule_space_autocorrect_prefetch(&identity);
                        }
                    }
                    engine.refresh_observed_suffix_precognition(output).await?;
                }
                Ok(ObservationReceipt::SurroundingText(outcome))
            }
        }
    }

    pub(crate) fn finish_focus_in(engine: &mut LayIbusEngine, changed: bool) {
        engine.discard_atomic_pending();
        engine.atomic.active = false;
        engine.invalidate_input_frame_background_work();
        trace::record(if changed {
            r#"{"kind":"ibus_focus","stage":"focus_in","receipt":"new_path"}"#
        } else {
            r#"{"kind":"ibus_focus","stage":"focus_in","receipt":"same_path"}"#
        });
        engine.config = lay::config::LayConfig::load();
        engine.context_reset_rereceipt = None;
        engine.client_context.surrounding_text_snapshot = None;
        if !changed && !engine.context_admission_required {
            engine.refresh_empty_tail_from_handoff();
        }
    }

    pub(crate) fn finish_focus_out(engine: &mut LayIbusEngine) {
        engine.discard_atomic_pending();
        engine.atomic.active = false;
        trace::record(r#"{"kind":"ibus_focus","stage":"focus_out"}"#);
        engine.client_context.terminal_focus_probe_serial = None;
        engine.client_context.terminal_focus_receipt = None;
        let preserve_active_path = engine.context_handoff_sealed
            || !engine.context_admission_required
                && (engine.should_preserve_focus_handoff()
                    || engine.shared_active_path_preserved());
        engine.reset_for_ibus_focus_change();
        if preserve_active_path {
            return;
        }
        let mut state = engine.shared.lock().expect("lay ime state poisoned");
        if state.active_path.as_deref() == Some(engine.path.as_str())
            && (!engine.context_admission_required
                || match engine.context_owner.as_ref() {
                    Some(owner) => state.context_owner_generation == Some(owner.generation.0),
                    None => state.context_owner_generation.is_none(),
                })
        {
            state.active_path = None;
            state.context_owner_generation = None;
        }
    }
}

impl LayIbusEngine {
    pub(crate) fn arm_visible_postcondition(&mut self, dispatched_at: Instant) {
        self.arm_visible_postcondition_with_effects(dispatched_at, None, None);
    }
    pub(crate) fn arm_visible_postcondition_with_effects(
        &mut self,
        dispatched_at: Instant,
        feedback: Option<PendingSystemOutcomeFeedback>,
        layout_sync_text: Option<String>,
    ) {
        if !self.client_context.surrounding_text_supported {
            return;
        }
        self.arm_visible_postcondition_from_surrounding_dispatch(
            dispatched_at,
            feedback,
            layout_sync_text,
        );
    }
    pub(crate) fn arm_active_composition_visible_postcondition_with_effects(
        &mut self,
        dispatched_at: Instant,
        feedback: Option<PendingSystemOutcomeFeedback>,
        layout_sync_text: Option<String>,
    ) {
        self.arm_visible_postcondition_from_surrounding_dispatch_with_source(
            dispatched_at,
            feedback,
            layout_sync_text,
            None,
            VisibleTailSource::ImeActiveComposition,
        );
    }
    pub(crate) fn arm_owned_preedit_space_visible_postcondition(
        &mut self,
        dispatched_at: Instant,
        expected: SurroundingTextSnapshot,
    ) {
        self.arm_visible_postcondition_from_surrounding_dispatch_with_source(
            dispatched_at,
            None,
            None,
            Some(expected),
            VisibleTailSource::ImeActiveComposition,
        );
    }
    pub(crate) fn arm_visible_postcondition_from_surrounding_dispatch(
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
    pub(crate) fn arm_exact_visible_postcondition_from_surrounding_dispatch(
        &mut self,
        dispatched_at: Instant,
        feedback: Option<PendingSystemOutcomeFeedback>,
        layout_sync_text: Option<String>,
        expected_external_snapshot: crate::engine::SurroundingTextSnapshot,
    ) {
        self.arm_visible_postcondition_from_surrounding_dispatch_with_snapshot(
            dispatched_at,
            feedback,
            layout_sync_text,
            Some(expected_external_snapshot),
        );
    }
    pub(crate) fn arm_visible_postcondition_from_surrounding_dispatch_with_snapshot(
        &mut self,
        dispatched_at: Instant,
        feedback: Option<PendingSystemOutcomeFeedback>,
        layout_sync_text: Option<String>,
        expected_external_snapshot: Option<crate::engine::SurroundingTextSnapshot>,
    ) {
        self.arm_visible_postcondition_from_surrounding_dispatch_with_source(
            dispatched_at,
            feedback,
            layout_sync_text,
            expected_external_snapshot,
            VisibleTailSource::ImeCommittedTail,
        );
    }
    fn arm_visible_postcondition_from_surrounding_dispatch_with_source(
        &mut self,
        dispatched_at: Instant,
        feedback: Option<PendingSystemOutcomeFeedback>,
        layout_sync_text: Option<String>,
        expected_external_snapshot: Option<crate::engine::SurroundingTextSnapshot>,
        source: VisibleTailSource,
    ) {
        let snapshot = VisibleTailSnapshot::new(
            source,
            self.committed_tail.buffer.clone(),
            Some(self.path.clone()),
            self.committed_tail.epoch,
        )
        .identity();
        self.committed_tail.pending_visible_postcondition =
            Some(crate::engine::PendingVisiblePostcondition {
                expected_suffix: self.committed_tail.buffer.clone(),
                expected_external_snapshot,
                native_document: self
                    .client_context
                    .native_document_binding
                    .as_ref()
                    .map(|binding| binding.for_result(self)),
                snapshot,
                dispatched_epoch: self.committed_tail.epoch,
                dispatched_at,
                final_refresh_retry_sent: false,
                feedback,
                layout_sync_text,
                space_boundary_commit: None,
            });
    }
    pub(crate) fn observe_visible_postcondition(&mut self) -> OutcomeProof {
        const OBSERVATION_TIMEOUT_MS: u128 = 1500;
        const SETTLE_GRACE_MS: u128 = 500;
        let Some(mut pending) = self.committed_tail.pending_visible_postcondition.take() else {
            return OutcomeProof::Rejected;
        };
        let elapsed_ms = pending.dispatched_at.elapsed().as_millis();
        if elapsed_ms > OBSERVATION_TIMEOUT_MS
            || pending.dispatched_epoch != self.committed_tail.epoch
            || pending.native_document.as_ref().is_some_and(|expected| {
                !expected.is_current(self)
                    || !self.native_document_is_current()
                    || self
                        .client_context
                        .native_document_binding
                        .as_ref()
                        .is_none_or(|actual| actual.identity() != expected.identity())
            })
        {
            record_causal_outcome("censored", &pending, self.committed_tail.epoch);
            return OutcomeProof::ExistingPostconditionCensored;
        }
        self.acknowledge_owned_preedit_space_postcondition(&mut pending);
        let observed = match pending.expected_external_snapshot.as_ref() {
            Some(expected)
                if self.client_context.surrounding_text_snapshot.as_ref() == Some(expected) =>
            {
                SurroundingSnapshotMatch::Exact
            }
            Some(_) => SurroundingSnapshotMatch::Missing,
            None => surrounding_snapshot_match(
                self.client_context.surrounding_text_snapshot.as_ref(),
                &pending.expected_suffix,
            ),
        };
        let status = if matches!(
            observed,
            SurroundingSnapshotMatch::Exact | SurroundingSnapshotMatch::TrailingBoundaryElided
        ) {
            self.record_observed_system_outcome(pending.feedback.as_ref());
            record_causal_outcome("confirmed_positive", &pending, self.committed_tail.epoch);
            if let Some(text) = pending.layout_sync_text.as_deref() {
                self.sync_layout_after_committed_text(text, "visible_postcondition_confirmed");
            }
            if observed == SurroundingSnapshotMatch::TrailingBoundaryElided {
                "observed_boundary_elided"
            } else {
                "observed"
            }
        } else if elapsed_ms <= SETTLE_GRACE_MS {
            record_causal_outcome(
                "pending_stale_observation",
                &pending,
                self.committed_tail.epoch,
            );
            self.committed_tail.pending_visible_postcondition = Some(pending);
            "pending"
        } else {
            // The compositor may report the pre-commit surrounding text once
            // before publishing the committed value. Only quarantine after the
            // bounded settle window has elapsed.
            self.quarantine_visible_postcondition_mismatch();
            record_causal_outcome("censored", &pending, self.committed_tail.epoch);
            "mismatch"
        };
        crate::trace::record(format!(
            r#"{{"kind":"ibus_visible_postcondition","status":"{status}"}}"#
        ));
        match status {
            "observed" | "observed_boundary_elided" => OutcomeProof::ExistingPostconditionConfirmed,
            "pending" => OutcomeProof::ExistingPostconditionPending,
            _ => OutcomeProof::ExistingPostconditionMismatch,
        }
    }
    pub(crate) fn record_observed_system_outcome(
        &mut self,
        feedback: Option<&PendingSystemOutcomeFeedback>,
    ) {
        if !self.context_word_is_known() || !self.content_allows_personalization() {
            return;
        }
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
                self.arm_observed_autocorrection_learning(
                    &feedback.original,
                    &feedback.replacement,
                );
                lay::typing_cpu::TypingCpu::record_observed_system_apply(
                    &feedback.original,
                    &feedback.replacement,
                    lay::typing_cpu::ObservedSystemTransition::Correction,
                );
            }
        }
    }

    /// Client presentation of this receipt's newly emitted Space may differ
    /// only at that final character. Older field text remains byte-for-byte
    /// exact; this cannot authorize a correction or create learning feedback.
    fn acknowledge_owned_preedit_space_postcondition(
        &mut self,
        pending: &mut crate::engine::PendingVisiblePostcondition,
    ) {
        if pending.snapshot.source != VisibleTailSource::ImeActiveComposition
            || pending.snapshot.focus_id.as_deref() != Some(self.path.as_str())
            || pending.feedback.is_some()
            || pending.space_boundary_commit.is_some()
            || pending.expected_suffix != self.committed_tail.buffer
        {
            return;
        }
        let Some(expected) = pending.expected_external_snapshot.as_ref() else {
            return;
        };
        if expected.has_selection()
            || expected.cursor_pos as usize != expected.text.chars().count()
            || !expected.text.ends_with(&pending.expected_suffix)
        {
            return;
        }
        let Some(sent_body) = expected.text.strip_suffix(' ') else {
            return;
        };
        let Some(owned_body) = pending.expected_suffix.strip_suffix(' ') else {
            return;
        };
        let observed = SurroundingTextSnapshot::new(
            format!("{sent_body}\u{a0}"),
            expected.cursor_pos,
            expected.anchor_pos,
        );
        if self.client_context.surrounding_text_snapshot.as_ref() != Some(&observed) {
            return;
        }
        let actual_tail = format!("{owned_body}\u{a0}");
        if self
            .acknowledge_owned_space_tail(&actual_tail, None)
            .is_err()
        {
            return;
        }
        pending.expected_suffix = actual_tail;
        pending.expected_external_snapshot = Some(observed);
        trace::record(
            r#"{"kind":"ibus_owned_space_boundary","stage":"client_acknowledged","source":"active_composition","sent":32,"observed":160}"#,
        );
    }
}
