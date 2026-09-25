use super::{
    OutcomeProof, IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH, IBUS_CAP_PREEDIT_TEXT,
    IBUS_CAP_SURROUNDING_TEXT, IBUS_INPUT_HINT_HIDDEN_TEXT, IBUS_INPUT_HINT_PRIVATE,
    IBUS_INPUT_PURPOSE_PASSWORD, IBUS_INPUT_PURPOSE_PIN,
};
use crate::atomic::{AtomicCapability, AtomicEnvelope, AtomicPriorReceipt};
use crate::context_admission::AdmissionToken;
use crate::engine::WordInputMode;
use crate::engine::{PendingSystemOutcomeFeedback, SystemOutcomeKind};
use crate::output::EngineOutput;
use crate::output::{AtomicProposal, PROPOSAL_CONSUMED_NO_EFFECT, PROPOSAL_FRAME_READY};
use crate::tail_memory::{
    record_causal_outcome, surrounding_snapshot_match, SurroundingSnapshotMatch,
};
use lay::text_edit::{VisibleTailSnapshot, VisibleTailSource};
use std::future::Future;
use std::task::Poll;
use zbus::fdo;

pub(super) fn exact_kitty_window(json: &str) -> bool {
    let Ok(window) = serde_json::from_str::<serde_json::Value>(json) else {
        return false;
    };
    window.get("appId").and_then(|value| value.as_str()) == Some("kitty.desktop")
        && window
            .get("windowId")
            .and_then(|value| value.as_str())
            .is_some_and(|value| !value.is_empty())
        && window
            .get("stableSequence")
            .and_then(|value| value.as_str())
            .is_some_and(|value| !value.is_empty())
}

async fn focused_window_is_kitty() -> bool {
    let mut request = Box::pin(async {
        let connection = zbus::Connection::session().await.ok()?;
        let reply = connection
            .call_method(
                Some("org.gnome.Shell"),
                "/io/github/radislabus_star/LayDaemon",
                Some("io.github.radislabus_star.LayDaemon"),
                "FocusedWindowInfo",
                &(),
            )
            .await
            .ok()?;
        let json = reply.body().deserialize::<String>().ok()?;
        Some(exact_kitty_window(&json))
    });
    let mut timeout = Box::pin(async_io::Timer::after(Duration::from_millis(150)));
    std::future::poll_fn(|cx| {
        if let Poll::Ready(result) = request.as_mut().poll(cx) {
            return Poll::Ready(result.unwrap_or(false));
        }
        if let Poll::Ready(_) = timeout.as_mut().poll(cx) {
            return Poll::Ready(false);
        }
        Poll::Pending
    })
    .await
}

/// A Reset invalidates the old admission token before the engine callback can
/// inspect it. This record is only a non-authoritative predecessor candidate;
/// the authenticated Reset and the next exact snapshot create new authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextResetRereceiptCandidate {
    pub(crate) token: AdmissionToken,
    pub(crate) tail_epoch: u64,
    pub(crate) token_text: String,
    pub(crate) observed_suffix_chars: u32,
    pub(crate) armed_revision: u64,
    published_preedit: Option<PublishedPreeditWitness>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingContextResetRereceipt {
    pub(crate) token: AdmissionToken,
    pub(crate) predecessor_token: AdmissionToken,
    pub(crate) tail_epoch: u64,
    pub(crate) token_text: String,
    pub(crate) observed_suffix_chars: u32,
    pub(crate) armed_revision: u64,
    pub(crate) confirmed: bool,
    published_preedit: Option<PublishedPreeditWitness>,
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
    start_tail_epoch: u64,
    anchored_token_chars: u64,
    start_committed_tail: String,
    snapshot_prefix: String,
    snapshot_suffix: String,
    start_cursor: u32,
    armed_revision: u64,
    reset_echo_epoch: Option<u64>,
}

/// Output provenance only: never a client snapshot or an edit target.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PublishedPreeditWitness {
    prefix_chars: u32,
    text: String,
}

fn snapshot_matches_retired_preedit(
    snapshot: &SurroundingTextSnapshot,
    pending: &PendingContextResetRereceipt,
) -> bool {
    let Some(published) = pending.published_preedit.as_ref() else {
        return false;
    };
    if snapshot.has_selection()
        || published.prefix_chars >= pending.observed_suffix_chars
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
        && snapshot
            .text
            .chars()
            .nth(cursor + old_suffix_chars)
            .is_none_or(crate::preedit::is_observed_word_boundary);
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
                && snapshot
                    .text
                    .chars()
                    .nth(end)
                    .is_none_or(crate::preedit::is_observed_word_boundary)
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
    ActivationOutcome, EnginePath, KeyCallback, LayoutIntentToken, SettledWordState,
    WordCompleteness, WordScope,
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
            Err(_) => {
                self.context_handoff_sealed = false;
                self.revoke_context_word();
                return false;
            }
        };
        if admission.callback_is_stale_for(&owner, &observed) {
            return false;
        }
        if matches!(observed.header.member.as_str(), "FocusOut" | "FocusOutId")
            && admission.revocation_matches(&owner, &observed)
        {
            // Authenticated loss of focus still requires local cleanup. The
            // return value admits that lifecycle event, not a text handoff.
            self.context_handoff_sealed = false;
            self.revoke_context_word();
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
            self.context_handoff_sealed = false;
            self.context_reset_rereceipt = None;
            self.discard_legacy_word_preedit_ownership();
            return true;
        }
        let accepted =
            accepted_focus && admission.seal_source(&owner, self.committed_tail.epoch, &observed);
        self.context_handoff_sealed = accepted;
        if !accepted {
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
            Err(_) => {
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
            self.revoke_context_word();
            return true;
        }
        let accepted = observed.header.member == "Disable" && admission.disable(&owner, &observed);
        if !accepted {
            self.context_handoff_sealed = false;
            self.revoke_context_word();
        }
        accepted
    }

    pub(crate) async fn observe_context_revocation(
        &mut self,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
    ) -> bool {
        if !self.context_admission_required {
            self.fail_context_activation_local();
            return true;
        }
        let reset_rereceipt_candidate = self.capture_context_reset_rereceipt_candidate();
        self.context_handoff_sealed = false;
        let mut installed_post_reset_scope = false;
        if let (Some(admission), Some(owner)) =
            (self.context_admission.clone(), self.context_owner.clone())
        {
            match admission.observe_callback(header, callback_entered).await {
                Ok(observed) if admission.callback_is_stale(&observed) => return false,
                Ok(observed) if admission.revocation_matches(&owner, &observed) => {
                    self.context_token = admission.current_token();
                    if let Some(token) = self.context_token.as_ref() {
                        self.context_word_scope = Some(token.word_scope());
                        installed_post_reset_scope = true;
                    }
                    self.arm_context_reset_rereceipt(reset_rereceipt_candidate, &owner);
                }
                Ok(_) => return false,
                Err(_) if admission.current_owner().as_ref() != Some(&owner) => return false,
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
        true
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

    /// An unknown beginning can support a suffix suggestion and an explicit
    /// append, without authorizing replacement of the complete word.
    pub(crate) fn context_observed_suffix_is_current(&self) -> bool {
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
                && !self.context_reset_rereceipt_exact_manual_handoff_allowed())
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
            return !self.client_context.surrounding_text_supported
                && self.terminal_committed_tail_executor_available();
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
            || !self.client_context.exact_surrounding_refresh_available
            || !self.client_context.surrounding_text_supported
            || self.atomic.active
            || !self.composition.buffer.is_empty()
            || self.content_is_sensitive()
            || self.committed_tail.buffer.ends_with(char::is_whitespace)
        {
            return false;
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
        let Some(witness) = self.client_context.managed_word_start.as_ref() else {
            return Some("missing");
        };
        if !self.client_context.managed_input {
            return Some("unmanaged");
        }
        if self.composition.word_input_mode != Some(WordInputMode::ManagedCommit) {
            return Some("mode");
        }
        if !self.client_context.exact_surrounding_refresh_available {
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

    fn arm_managed_word_start_from_current_snapshot(&mut self) {
        if !self.client_context.managed_input
            || !self.client_context.exact_surrounding_refresh_available
            || !self.client_context.surrounding_text_supported
            || self.context_handoff_sealed
            || self.atomic.active
            || !self.composition.buffer.is_empty()
            || self.content_is_sensitive()
        {
            return;
        }
        let Some(snapshot) = self.client_context.surrounding_text_snapshot.as_ref() else {
            return;
        };
        let text_chars = snapshot.text.chars().collect::<Vec<_>>();
        let cursor = snapshot.cursor_pos as usize;
        if snapshot.has_selection() || cursor > text_chars.len() {
            return;
        }
        let token = self.last_tail_token_text();
        let token_chars = token.chars().count();
        let Some(start) = cursor.checked_sub(token_chars) else {
            return;
        };
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
            return;
        }
        let Some(start_committed_tail) = self.committed_tail.buffer.strip_suffix(&token) else {
            return;
        };
        self.client_context.managed_word_start = Some(ManagedWordStartWitness {
            identity: crate::engine::next_input_identity(),
            focus_receipt: self.client_context.focus_receipt.clone(),
            focus_serial: self.client_context.focus_serial,
            runtime_owner_lease_identity: self.client_context.runtime_owner_lease_identity,
            layout_generation: self.layout_gesture.layout_generation,
            start_tail_epoch: self.committed_tail.epoch,
            anchored_token_chars: token_chars as u64,
            start_committed_tail: start_committed_tail.to_string(),
            snapshot_prefix: text_chars[..start].iter().collect(),
            snapshot_suffix: text_chars[cursor..].iter().collect(),
            start_cursor: start as u32,
            armed_revision: self.client_context.surrounding_observation_revision,
            reset_echo_epoch: None,
        });
    }

    fn current_snapshot_matches_managed_word_start(&self) -> bool {
        let Some(witness) = self.client_context.managed_word_start.as_ref() else {
            return false;
        };
        let Some(snapshot) = self.client_context.surrounding_text_snapshot.as_ref() else {
            return false;
        };
        if snapshot.has_selection() || !self.managed_word_start_is_current() {
            return false;
        }
        let token = self.last_tail_token_text();
        let expected_prefix = format!("{}{}", witness.snapshot_prefix, token);
        let expected_cursor = witness
            .start_cursor
            .saturating_add(u32::try_from(token.chars().count()).unwrap_or(u32::MAX));
        let cursor = snapshot.cursor_pos as usize;
        let text_chars = snapshot.text.chars().collect::<Vec<_>>();
        cursor <= text_chars.len()
            && snapshot.cursor_pos == expected_cursor
            && text_chars[..cursor].iter().collect::<String>() == expected_prefix
            && text_chars[cursor..]
                .iter()
                .collect::<String>()
                .ends_with(&witness.snapshot_suffix)
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
        let Some(admission) = self.context_admission.clone() else {
            if self.context_admission_required {
                self.fail_context_activation();
            }
            return false;
        };
        let Some(target_path) = EnginePath::new(self.path.clone()) else {
            self.fail_context_activation();
            return false;
        };
        let observed = match admission.observe_callback(header, callback_entered).await {
            Ok(observed) => observed,
            Err(_) => {
                self.fail_context_activation();
                return false;
            }
        };
        if native_context.is_some() {
            self.try_install_pending_context_activation();
        }
        if let (Some(owner), Some(context_path)) = (self.context_owner.as_ref(), native_context) {
            let Some(context) = admission.context_key(context_path.to_string()) else {
                self.fail_context_activation();
                return false;
            };
            if let Some(enriched) =
                admission.enrich_native_activation(owner, &target_path, &context, &observed)
            {
                return if enriched {
                    self.context_token = admission.current_token();
                    true
                } else {
                    self.fail_context_activation();
                    false
                };
            }
        }
        if let Some(context_path) = native_context {
            let Some(context) = admission.context_key(context_path.to_string()) else {
                self.fail_context_activation();
                return false;
            };
            if admission.enrich_pending_native_activation(&target_path, context, &observed) {
                self.fail_context_activation_local();
                return true;
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
            admission
                .start_compatibility_activation(target_path, observed.position)
                .map_err(|_| ())
        };
        if started.is_err() {
            self.fail_context_activation();
            return false;
        }
        // The adapter owns the one-shot Get/marker future. Until a later
        // callback consumes its target-bound result, this engine has no text
        // authority and all input follows the literal UnknownStart route.
        self.fail_context_activation_local();
        true
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
            if self.install_context_activation(outcome.clone())
                && !admission.acknowledge_activation(&outcome)
            {
                self.discard_context_activation();
            }
        }
    }

    pub(crate) fn install_context_activation(&mut self, outcome: ActivationOutcome) -> bool {
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
                            if shared.autocorrect_suppression.take().is_some() {
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
        self.composition.buffer.clear();
        self.composition.cursor = 0;
        self.composition.legacy_word_preedit_active = false;
        self.clear_preedit_completion_state();
        self.composition.pending_passthrough_preedit_clear = false;
        if let Some((tail, epoch, suppression)) = transferred {
            self.committed_tail.buffer = tail;
            self.committed_tail.epoch = epoch;
            self.committed_tail.autocorrect_suppression = suppression;
        } else {
            self.committed_tail.buffer.clear();
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
        self.exact_manual_target_snapshot = transferred_exact_snapshot.and_then(|source| {
            let target_token = self.context_token.clone()?;
            (source.tail_epoch == self.committed_tail.epoch
                && source.tail == self.committed_tail.buffer
                && source.expires_at > Instant::now()
                && self.client_context.surrounding_text_supported
                && !self.client_context.surrounding_text_callback_observed
                && self.content_allows_text_assistance())
            .then_some(crate::engine::ExactManualTargetSnapshotReceipt {
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

    pub(crate) async fn begin_context_key_callback(
        &mut self,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
        atomic: bool,
    ) -> Option<KeyCallback> {
        self.try_install_pending_context_activation();
        let admission = self.context_admission.clone()?;
        let Some(owner) = self.context_owner.clone() else {
            // The stamp rendezvous is the only await admitted while the engine
            // interface lock is held. It lets the observer poison a pending
            // transfer to UnknownStart before this key runs literally; it does
            // not wait for the compatibility property or marker fence.
            if let Ok(observed) = admission.observe_callback(header, callback_entered).await {
                self.try_install_pending_context_activation();
                if let Some(owner) = self.context_owner.as_ref() {
                    if let Ok(callback) =
                        admission.key_callback_from_observed(owner, observed.clone(), atomic)
                    {
                        return Some(callback);
                    }
                }
                admission.abandon_observed_key(&observed);
            }
            return None;
        };
        let callback = if atomic {
            admission
                .begin_atomic_key_callback(&owner, header, callback_entered)
                .await
        } else {
            admission
                .begin_key_callback(&owner, header, callback_entered)
                .await
        };
        match callback {
            Ok(callback) => Some(callback),
            Err(_) => {
                self.revoke_context_word();
                None
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
        let track_suffix = self
            .context_word_scope
            .as_ref()
            .is_some_and(|scope| scope.lineage().completeness == WordCompleteness::UnknownStart);
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
        if let Some((retained_chars, appended_chars, observed_boundary)) = appended_effect {
            if let Some(boundary) = observed_boundary {
                scope.close_at_observed_boundary(self.committed_tail.epoch, Some(boundary));
            } else if scope.lineage().completeness == WordCompleteness::UnknownStart {
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
            } else if scope.lineage().completeness == WordCompleteness::UnknownStart {
                if exact_backspace {
                    scope.observe_tail_backspace();
                } else {
                    scope.revoke_for_input_gap();
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
        } else if printable && scope.lineage().completeness == WordCompleteness::UnknownStart {
            if let Some(retained_chars) = observed_append {
                scope.observe_tail_append(retained_chars);
            } else {
                scope.revoke_for_input_gap();
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
        boundary_candidate: Option<ContextResetRereceiptCandidate>,
    ) {
        let owned_append = handled
            || (self.exact_replay_tail_change_quarantined && self.exact_replay_quarantine_active());
        // Alt's release may still carry its own modifier bit. The accepted
        // append below is the effect witness; all other command bits remain.
        let boundary_modifier_state =
            if !is_key_press(state) && is_accept_completion_with_space_key(keyval) {
                state & !crate::protocol::MOD1_MASK
            } else {
                state
            };
        if owned_append && !has_command_modifier(boundary_modifier_state) {
            if let Some(candidate) = boundary_candidate {
                if let Some(appended) = self.committed_tail.buffer.strip_prefix(tail_before) {
                    let closes_word = !appended.is_empty()
                        && appended.ends_with(char::is_whitespace)
                        && appended
                            .trim_end_matches(char::is_whitespace)
                            .chars()
                            .all(|ch| !crate::preedit::is_observed_word_boundary(ch));
                    if closes_word
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
                                && token_text.starts_with(candidate.token_text.as_str())
                            {
                                // The real append closed a proved word. Retain
                                // only its exact predecessor for a later Reset;
                                // this is not authority for the next empty word.
                                self.context_reset_rereceipt = Some(PendingContextResetRereceipt {
                                    token: token.clone(),
                                    predecessor_token: candidate.token,
                                    tail_epoch: self.committed_tail.epoch,
                                    observed_suffix_chars: token_text.chars().count() as u32,
                                    token_text,
                                    armed_revision: self
                                        .client_context
                                        .surrounding_observation_revision,
                                    confirmed: false,
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
            token,
            predecessor_token: pending.predecessor_token,
            tail_epoch: self.committed_tail.epoch,
            token_text: next_token_text,
            observed_suffix_chars: next_chars as u32,
            armed_revision: self.client_context.surrounding_observation_revision,
            confirmed: pending.confirmed,
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

    fn capture_context_reset_rereceipt_candidate(&self) -> Option<ContextResetRereceiptCandidate> {
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
            let published_preedit = pending
                .published_preedit
                .as_ref()
                .filter(|published| {
                    pending.confirmed
                        && published.prefix_chars == observed_suffix_chars as u32
                        && published.text.starts_with(&token_text)
                })
                .cloned();
            return Some(ContextResetRereceiptCandidate {
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
        let (Some(candidate), Some(token)) = (candidate, self.context_token.clone()) else {
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
        let tail_epoch = candidate.tail_epoch;
        let observed_suffix_chars = candidate.observed_suffix_chars;
        self.context_reset_rereceipt = Some(PendingContextResetRereceipt {
            token,
            predecessor_token: candidate.token,
            tail_epoch,
            token_text: candidate.token_text,
            observed_suffix_chars,
            armed_revision: candidate.armed_revision,
            confirmed: false,
            published_preedit: None,
        });
        trace::record(format!(
            r#"{{"kind":"ibus_context_reset_rereceipt","stage":"armed","tail_epoch":{tail_epoch},"suffix_chars":{observed_suffix_chars}}}"#,
        ));
    }

    pub(crate) fn observe_context_reset_rereceipt_surrounding_text(&mut self) {
        let Some(pending) = self.context_reset_rereceipt.clone() else {
            return;
        };
        let revision_gap = self
            .client_context
            .surrounding_observation_revision
            .checked_sub(pending.armed_revision);
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
                pending.armed_revision = self.client_context.surrounding_observation_revision;
            }
            trace::record(
                r#"{"kind":"ibus_context_reset_rereceipt","stage":"retained_unconfirmed","reason":"zero_width_right_boundary"}"#,
            );
            return;
        }
        if self.client_context.surrounding_observation_revision
            == pending.armed_revision.saturating_add(1)
            && !self.context_reset_rereceipt_matches_current_snapshot()
            && self.context_reset_rereceipt_identity_is_current()
            && self
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .is_some_and(|snapshot| self.exact_replay_contains_prior_snapshot(snapshot))
        {
            if let Some(pending) = self.context_reset_rereceipt.as_mut() {
                pending.confirmed = false;
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
        {
            // A delayed strict prefix contradicts exact authority, but not the
            // observed token lineage. Retain it only as an inert predecessor;
            // recovery still requires a later authenticated Reset and exact
            // full receipt.
            if let Some(pending) = self.context_reset_rereceipt.as_mut() {
                pending.confirmed = false;
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
            if self.client_context.surrounding_observation_revision
                == pending.armed_revision.saturating_add(1)
                && self.context_reset_rereceipt_matches_current_snapshot()
            {
                if let Some(pending) = self.context_reset_rereceipt.as_mut() {
                    pending.published_preedit = None;
                }
                trace::record(format!(
                    r#"{{"kind":"ibus_context_reset_rereceipt","stage":"advanced_confirmed","tail_epoch":{},"suffix_chars":{}}}"#,
                    pending.tail_epoch, pending.observed_suffix_chars,
                ));
                return;
            }
            self.context_reset_rereceipt = None;
            trace::record(
                r#"{"kind":"ibus_context_reset_rereceipt","stage":"rejected","reason":"second_surrounding_receipt"}"#,
            );
            return;
        }
        if self.client_context.surrounding_observation_revision
            != pending.armed_revision.saturating_add(1)
            || !self.context_reset_rereceipt_matches_current_snapshot()
        {
            self.context_reset_rereceipt = None;
            trace::record(
                r#"{"kind":"ibus_context_reset_rereceipt","stage":"rejected","reason":"surrounding_receipt_mismatch"}"#,
            );
            return;
        }
        if let Some(pending) = self.context_reset_rereceipt.as_mut() {
            pending.confirmed = true;
            pending.published_preedit = None;
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

    pub(crate) fn context_reset_rereceipt_exact_manual_handoff_allowed(&self) -> bool {
        self.context_reset_rereceipt
            .as_ref()
            .is_some_and(|pending| pending.confirmed)
            && self.context_reset_rereceipt_identity_is_current()
            && self.context_reset_rereceipt_matches_current_snapshot()
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
                || !(retain_retired_surface || retain_identical_unconfirmed_surface)
            {
                pending.published_preedit = published;
            }
        }
    }

    fn context_reset_rereceipt_identity_is_current(&self) -> bool {
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

    pub(crate) fn revoke_context_word(&mut self) {
        self.exact_manual_target_snapshot = None;
        self.context_reset_rereceipt = None;
        if let Some(scope) = self.context_word_scope.as_mut() {
            scope.revoke_for_input_gap();
        }
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
        self.committed_tail.pending_completion_learning = None;
        self.clear_preedit_completion_state();
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
        let legacy_word_preedit_is_supported = self.client_context.preedit_text_supported
            && !self.client_context.surrounding_text_supported
            && !self.client_context.exact_surrounding_refresh_available;
        if legacy_word_preedit_was_supported != legacy_word_preedit_is_supported
            || exact_surrounding_refresh_was_available
                != self.client_context.exact_surrounding_refresh_available
        {
            self.client_context.managed_word_start = None;
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
            self.context_reset_rereceipt = None;
            self.advance_surrounding_observation_revision();
        }
        if !surrounding_text_was_supported
            && self.client_context.surrounding_text_supported
            && self.composition.word_input_mode == Some(WordInputMode::TerminalPassthrough)
        {
            self.composition.word_input_mode = Some(WordInputMode::ManagedCommit);
            self.composition.pending_passthrough_preedit_clear = true;
        }
        if !self.client_context.surrounding_text_supported {
            self.client_context.surrounding_text_snapshot = None;
            self.client_context.managed_word_start = None;
            self.layout_gesture.pending_manual_toggle = false;
        }
    }
    pub(crate) fn set_content_type_state(&mut self, purpose: u32, hints: u32) {
        if self.client_context.content_purpose == purpose
            && self.client_context.content_hints == hints
        {
            return;
        }
        self.context_reset_rereceipt = None;
        if self.client_context.content_purpose != purpose {
            self.composition.word_input_mode = None;
        }
        self.client_context.content_purpose = purpose;
        self.client_context.content_hints = hints;
        self.client_context.managed_word_start = None;
        self.invalidate_input_frame_background_work();
        self.clear_preedit_completion_state();
        if self.content_is_sensitive() {
            self.composition.buffer.clear();
            self.composition.cursor = 0;
            self.composition.legacy_word_preedit_active = false;
            self.client_context.surrounding_text_snapshot = None;
            self.close_committed_tail_field();
        }
    }
    pub(crate) fn content_is_sensitive(&self) -> bool {
        matches!(
            self.client_context.content_purpose,
            IBUS_INPUT_PURPOSE_PASSWORD | IBUS_INPUT_PURPOSE_PIN
        ) || self.client_context.content_hints
            & (IBUS_INPUT_HINT_PRIVATE | IBUS_INPUT_HINT_HIDDEN_TEXT)
            != 0
    }
    pub(crate) fn content_allows_text_assistance(&self) -> bool {
        !self.content_is_sensitive()
    }
    pub(crate) fn observe_external_surrounding_text(
        &mut self,
        snapshot: Option<SurroundingTextSnapshot>,
    ) {
        self.client_context.surrounding_text_callback_observed = true;
        self.advance_surrounding_observation_revision();
        self.client_context.surrounding_text_supported = true;
        self.client_context.surrounding_text_snapshot = if self.content_is_sensitive() {
            None
        } else {
            snapshot
        };
        self.bind_exact_replay_external_prefix_from_snapshot();
        self.reconcile_managed_word_start_after_surrounding_observation();
        self.observe_context_reset_rereceipt_surrounding_text();
    }
    pub(crate) fn advance_surrounding_observation_revision(&mut self) {
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

#[derive(Clone)]
pub(crate) struct ClientContextState {
    pub(crate) focus_receipt: Option<String>,
    pub(crate) focus_serial: u64,
    pub(crate) runtime_owner_lease_identity: u64,
    pub(crate) cursor_cell_width: i32,
    pub(crate) kitty_focus_probe_serial: Option<u64>,
    pub(crate) kitty_terminal_focus_receipt: Option<String>,
    pub(crate) content_purpose: u32,
    pub(crate) content_hints: u32,
    pub(crate) surrounding_text_supported: bool,
    pub(crate) preedit_text_supported: bool,
    pub(crate) exact_surrounding_refresh_available: bool,
    pub(crate) surrounding_text_snapshot: Option<SurroundingTextSnapshot>,
    pub(crate) surrounding_observation_revision: u64,
    pub(crate) managed_word_start: Option<ManagedWordStartWitness>,
    pub(crate) surrounding_text_callback_observed: bool,
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
            kitty_focus_probe_serial: None,
            kitty_terminal_focus_receipt: None,
            content_purpose: 0,
            content_hints: 0,
            surrounding_text_supported: false,
            preedit_text_supported: false,
            exact_surrounding_refresh_available: false,
            surrounding_text_snapshot: None,
            surrounding_observation_revision: 0,
            managed_word_start: None,
            surrounding_text_callback_observed: false,
            factory_engine_profile,
            managed_input,
        }
    }

    pub(super) fn finish_kitty_focus_probe(
        &mut self,
        expected_focus_receipt: Option<&str>,
        is_kitty: bool,
    ) -> bool {
        let Some(expected_focus_receipt) = expected_focus_receipt else {
            return false;
        };
        if self.focus_receipt.as_deref() != Some(expected_focus_receipt) {
            return false;
        }
        self.kitty_focus_probe_serial = Some(self.focus_serial);
        self.kitty_terminal_focus_receipt = is_kitty.then(|| expected_focus_receipt.to_string());
        is_kitty
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
        let callback = engine
            .begin_context_key_callback(header, callback_entered, false)
            .await;
        if trace::enabled() {
            trace::record_context_admission(
                "legacy_callback_admission",
                "ProcessKeyEvent",
                callback_serial,
                if callback.is_some() {
                    "accepted"
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
        let tail_before = engine.committed_tail.buffer.clone();
        let boundary_candidate = (callback.is_some()
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
        .flatten();
        engine.exact_replay_tail_change_quarantined = false;
        engine.context_callback_entered = Some(callback_entered);
        engine.consume_shift_gesture_handoff();
        let mut result = engine
            .process_key_event_with_output(output, keyval, keycode, state)
            .await;
        if result.is_ok() {
            let handled = result.as_ref().is_ok_and(|handled| *handled);
            engine.settle_context_key_callback(
                callback.as_ref(),
                keyval,
                keycode,
                state,
                &tail_before,
                handled,
            );
            engine.advance_context_reset_rereceipt_after_key(
                keyval,
                keycode,
                state,
                &tail_before,
                handled,
                boundary_candidate,
            );
            if is_key_press(state)
                && tail_before != engine.committed_tail.buffer
                && !std::mem::take(&mut engine.exact_replay_tail_change_quarantined)
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
            .await;
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
        output: Option<&mut EngineOutput<'_, '_>>,
    ) -> fdo::Result<LifecycleReceipt> {
        match event {
            WindowLifecycleEvent::FocusIn { header } => {
                let changed = if engine.context_admission_required {
                    engine
                        .activate_context_from_header(header, Instant::now(), None)
                        .await
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
                if !engine
                    .observe_context_revocation(header, Instant::now())
                    .await
                {
                    return Ok(LifecycleReceipt::Refused);
                }
                engine.discard_atomic_pending();
                trace::record(r#"{"kind":"ibus_focus","stage":"reset"}"#);
                let held_shift = engine.layout_gesture.shift_active;
                let cleared = if engine.atomic.active {
                    Ok(())
                } else {
                    let output = output.ok_or_else(|| {
                        fdo::Error::Failed("Reset requires the existing output boundary".into())
                    })?;
                    engine.clear_preedit(output).await
                };
                engine.reset_for_ibus_soft_reset();
                engine.layout_gesture.shift_active = held_shift;
                cleared?;
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
                let unchanged = if let (Some(admission), Some(path), Some(header)) = (
                    engine.context_admission.clone(),
                    EnginePath::new(engine.path.clone()),
                    header,
                ) {
                    match admission
                        .observe_content_type_callback(&path, value, header, Instant::now())
                        .await
                    {
                        Ok(Some(unchanged)) => unchanged,
                        Ok(None) => return Ok(ObservationReceipt::Refused),
                        Err(_) => false,
                    }
                } else {
                    false
                };
                if !unchanged {
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
                let focus_serial = engine.client_context.focus_serial;
                let focus_receipt = engine.client_context.focus_receipt.clone();
                if (5..=32).contains(&width)
                    && engine.client_context.preedit_text_supported
                    && !engine.client_context.surrounding_text_supported
                    && engine.client_context.content_purpose == 0
                    && focus_receipt.is_some()
                    && engine
                        .client_context
                        .kitty_terminal_focus_receipt
                        .as_deref()
                        != focus_receipt.as_deref()
                    && engine.client_context.kitty_focus_probe_serial != Some(focus_serial)
                {
                    engine.client_context.kitty_focus_probe_serial = Some(focus_serial);
                    let is_kitty = focused_window_is_kitty().await;
                    if engine
                        .client_context
                        .finish_kitty_focus_probe(focus_receipt.as_deref(), is_kitty)
                    {
                        trace::record(
                            r#"{"kind":"ibus_terminal_focus","source":"gnome_exact_kitty_window"}"#,
                        );
                    }
                }
                if !engine.atomic.active {
                    let output = output.as_deref_mut().ok_or_else(|| {
                        fdo::Error::Failed("cursor observation requires output".into())
                    })?;
                    engine.flush_dirty_preedit(output).await?;
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
        engine.client_context.kitty_focus_probe_serial = None;
        engine.client_context.kitty_terminal_focus_receipt = None;
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
                snapshot,
                dispatched_epoch: self.committed_tail.epoch,
                dispatched_at,
                feedback,
                layout_sync_text,
            });
    }
    pub(crate) fn observe_visible_postcondition(&mut self) -> OutcomeProof {
        const OBSERVATION_TIMEOUT_MS: u128 = 1500;
        const SETTLE_GRACE_MS: u128 = 500;
        let Some(pending) = self.committed_tail.pending_visible_postcondition.take() else {
            return OutcomeProof::Rejected;
        };
        let elapsed_ms = pending.dispatched_at.elapsed().as_millis();
        if elapsed_ms > OBSERVATION_TIMEOUT_MS
            || pending.dispatched_epoch != self.committed_tail.epoch
        {
            record_causal_outcome("censored", &pending, self.committed_tail.epoch);
            return OutcomeProof::ExistingPostconditionCensored;
        }
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
        &self,
        feedback: Option<&PendingSystemOutcomeFeedback>,
    ) {
        if !self.context_word_is_known() {
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
                lay::typing_cpu::TypingCpu::record_observed_system_apply(
                    &feedback.original,
                    &feedback.replacement,
                    lay::typing_cpu::ObservedSystemTransition::Correction,
                );
            }
        }
    }
}
