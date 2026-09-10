use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use zbus::fdo;

use super::context_admission::{AdmissionToken, KeyCallback};
use super::engine::{DeferredLearningAction, LayIbusEngine};
use super::output::{
    AtomicEffectBuilder, AtomicProposal, EngineOutput, PROPOSAL_CONSUMED_NO_EFFECT,
    PROPOSAL_FRAME_READY, PROPOSAL_NATIVE_UNHANDLED,
};
use super::protocol::{is_key_press, AutocorrectSuppression, SharedState, KEY_BACKSPACE};

pub(crate) type AtomicEnvelope = (u64, u64, u64, u64, u64, u64, Vec<u8>);
pub(crate) type AtomicProfile = (u32, u32, Vec<u8>, u32, u32, u8, u32);
pub(crate) type AtomicLease = (u64, u64, u64, u64, u64, bool, Vec<u8>);
pub(crate) type AtomicCapability = (AtomicProfile, AtomicLease);
pub(crate) type AtomicPriorReceipt = (u8, u64, Vec<u8>);

const PROTOCOL_VERSION: u32 = 1;
const ADAPTER_KIND: u32 = 196_609;
const TRANSACTION_KIND: u32 = 196_610;
const EFFECT_MASK: u32 = 0x0f;
const MAX_EFFECTS: u8 = 3;
const GUARANTEE_FLAGS: u32 = 0x3f;
const DIGEST_BYTES: usize = 32;
const PRODUCTION_DIGEST: [u8; DIGEST_BYTES] = [
    0xec, 0xf4, 0x3b, 0x4c, 0x0c, 0x4c, 0xeb, 0xae, 0x8d, 0xb1, 0x56, 0x02, 0xa8, 0xc1, 0x44, 0x50,
    0xcb, 0x89, 0x89, 0xc2, 0x73, 0xc5, 0x20, 0x8c, 0xb1, 0x3a, 0x45, 0x26, 0x47, 0x07, 0x4a, 0xf7,
];

const RECEIPT_NONE: u8 = 0;
const RECEIPT_REFUSED_ZERO_EFFECT: u8 = 1;
const RECEIPT_SUBMITTED_ATOMIC: u8 = 2;
const RECEIPT_CONSUMED_NO_EFFECT: u8 = 3;
const RECEIPT_FOCUS_LINEAGE_TERMINATED: u8 = 4;
const RECEIPT_SUBMISSION_UNCERTAIN_NO_RETRY: u8 = 5;

struct PendingAtomicTransition {
    transaction_identity: u64,
    daemon_focus_epoch: u64,
    disposition: u8,
    event_keycode: u32,
    event_keyval: u32,
    event_state: u32,
    event_is_press: bool,
    base: AtomicBaseSnapshot,
    admission_token: Option<AdmissionToken>,
    context_callback: Option<KeyCallback>,
    context_tail_before: String,
    speculative: LayIbusEngine,
}

#[derive(Clone)]
struct AtomicBaseSnapshot {
    suppression_revision: u64,
    active_path: Option<String>,
    handoff_tail_buffer: String,
    handoff_tail_epoch: u64,
    handoff_focus_receipt: Option<String>,
    runtime_owner_lease_identity: u64,
}

#[derive(Clone)]
struct AtomicSuppressionBlock {
    suppression: Option<AutocorrectSuppression>,
    revision: u64,
    preserve_active_path_until: Option<std::time::Instant>,
    exact_handoff_epoch: Option<u64>,
    exact_handoff_path: Option<String>,
}

fn pending_transitions() -> &'static Mutex<HashMap<String, PendingAtomicTransition>> {
    static PENDING: OnceLock<Mutex<HashMap<String, PendingAtomicTransition>>> = OnceLock::new();
    PENDING.get_or_init(|| Mutex::new(HashMap::new()))
}

impl LayIbusEngine {
    #[cfg(test)]
    pub(crate) async fn process_atomic_key_event(
        &mut self,
        keyval: u32,
        keycode: u32,
        state: u32,
        envelope: AtomicEnvelope,
        capability: AtomicCapability,
        prior_receipt: AtomicPriorReceipt,
    ) -> fdo::Result<AtomicProposal> {
        self.process_atomic_key_event_with_context_tail(
            keyval,
            keycode,
            state,
            envelope,
            capability,
            prior_receipt,
        )
        .await
        .map(|(proposal, _)| proposal)
    }

    pub(super) async fn process_atomic_key_event_with_context_tail(
        &mut self,
        keyval: u32,
        keycode: u32,
        state: u32,
        envelope: AtomicEnvelope,
        capability: AtomicCapability,
        prior_receipt: AtomicPriorReceipt,
    ) -> fdo::Result<(AtomicProposal, String)> {
        let mut context_tail_before = self.committed_tail.buffer.clone();
        if !valid_request(&envelope, &capability) {
            self.discard_atomic_pending();
            return Ok((native_unhandled(), context_tail_before));
        }
        self.atomic.active = true;
        let prior_settled = self.settle_atomic_pending(envelope.3, &prior_receipt);
        // The prior receipt may advance both the admitted word lineage and the
        // committed tail. This is the preimage for the current native key.
        context_tail_before = self.committed_tail.buffer.clone();
        if !prior_settled {
            return Ok((native_unhandled(), context_tail_before));
        }
        let admission_token = self.live_context_token();
        if self.context_admission_required && admission_token.is_none() {
            self.discard_atomic_pending();
            return Ok((native_unhandled(), context_tail_before));
        }
        self.consume_shift_gesture_handoff();
        #[cfg(test)]
        if let Some(before_capture) = self.atomic.before_capture.take() {
            before_capture();
        }

        let Some((mut speculative, base)) = self.deep_atomic_clone_with_live_owner_base() else {
            return Ok((native_unhandled(), context_tail_before));
        };
        speculative.atomic.speculation = true;
        speculative.atomic.deferred_layout_actions.clear();
        speculative.atomic.deferred_learning_actions.clear();

        let profile = &capability.0;
        let lease = &capability.1;
        let mut builder = AtomicEffectBuilder::new(profile.4, u32::from(profile.5), lease.5);
        let handled = {
            let mut output = EngineOutput::atomic(&mut builder);
            speculative
                .process_key_event_with_output(&mut output, keyval, keycode, state)
                .await?
        };
        let proposal = builder.finish(handled);

        if matches!(
            proposal.0,
            PROPOSAL_FRAME_READY | PROPOSAL_CONSUMED_NO_EFFECT
        ) {
            let pending = PendingAtomicTransition {
                transaction_identity: envelope.0,
                daemon_focus_epoch: envelope.3,
                disposition: proposal.0,
                event_keycode: keycode,
                event_keyval: keyval,
                event_state: state,
                event_is_press: is_key_press(state),
                base,
                admission_token,
                context_callback: None,
                context_tail_before: context_tail_before.clone(),
                speculative,
            };
            pending_transitions()
                .lock()
                .expect("lay atomic pending state poisoned")
                .insert(self.path.clone(), pending);
        } else {
            self.commit_native_observation(&speculative);
            self.commit_native_tail_observation(
                &speculative,
                keyval,
                state,
                admission_token.as_ref(),
            );
            self.publish_shift_gesture_handoff();
        }
        Ok((proposal, context_tail_before))
    }

    pub(crate) fn discard_atomic_pending(&self) {
        pending_transitions()
            .lock()
            .expect("lay atomic pending state poisoned")
            .remove(&self.path);
    }

    pub(crate) fn bind_atomic_context_callback(&self, callback: Option<KeyCallback>) -> bool {
        let Ok(mut pending) = pending_transitions().lock() else {
            return false;
        };
        let Some(transition) = pending.get_mut(&self.path) else {
            return false;
        };
        transition.context_callback = callback;
        true
    }

    #[cfg(test)]
    fn deep_atomic_clone(&self) -> Self {
        self.deep_atomic_clone_with_base().0
    }

    #[cfg(test)]
    fn deep_atomic_clone_with_base(&self) -> (Self, AtomicBaseSnapshot) {
        let mut speculative = self.clone();
        speculative.context_bridge_token = None;
        let shared = self.shared.lock().expect("lay ime state poisoned").clone();
        let base = self.atomic_base_snapshot(&shared);
        speculative.shared = Arc::new(Mutex::new(shared));
        (speculative, base)
    }

    fn deep_atomic_clone_with_live_owner_base(&self) -> Option<(Self, AtomicBaseSnapshot)> {
        let mut speculative = self.clone();
        speculative.context_bridge_token = None;
        let (shared_snapshot, base) = {
            let shared = self.shared.lock().ok()?;
            if shared.active_path.as_deref() != Some(self.path.as_str()) {
                return None;
            }
            (shared.clone(), self.atomic_base_snapshot(&shared))
        };
        speculative.shared = Arc::new(Mutex::new(shared_snapshot));
        Some((speculative, base))
    }

    fn atomic_base_snapshot(&self, shared: &SharedState) -> AtomicBaseSnapshot {
        AtomicBaseSnapshot {
            suppression_revision: shared.suppression_revision,
            active_path: shared.active_path.clone(),
            handoff_tail_buffer: shared.handoff_tail_buffer.clone(),
            handoff_tail_epoch: shared.handoff_tail_epoch,
            handoff_focus_receipt: shared.handoff_focus_receipt.clone(),
            runtime_owner_lease_identity: self.client_context.runtime_owner_lease_identity,
        }
    }

    fn settle_atomic_pending(
        &mut self,
        current_daemon_focus_epoch: u64,
        receipt: &AtomicPriorReceipt,
    ) -> bool {
        let pending = pending_transitions()
            .lock()
            .expect("lay atomic pending state poisoned")
            .remove(&self.path);
        let Some(mut pending) = pending else {
            return receipt.0 == RECEIPT_NONE && receipt.1 == 0 && receipt.2.is_empty();
        };

        if pending.daemon_focus_epoch != current_daemon_focus_epoch
            || receipt.1 != pending.transaction_identity
            || receipt.2.len() != DIGEST_BYTES
            || !receipt_is_compatible(pending.disposition, receipt.0)
        {
            self.revoke_context_word();
            return false;
        }
        if let Some(token) = pending.admission_token.as_ref() {
            if !self
                .context_admission
                .as_ref()
                .is_some_and(|admission| admission.revalidate(token))
            {
                super::trace::record(
                    r#"{"kind":"ibus_atomic_settlement","status":"context_generation_censored"}"#,
                );
                self.revoke_context_word();
                return false;
            }
        } else if self.context_admission_required {
            self.revoke_context_word();
            return false;
        }

        let callback = pending.context_callback.clone();
        let event = (
            pending.event_keyval,
            pending.event_keycode,
            pending.event_state,
            pending.context_tail_before.clone(),
        );
        let applied = match receipt.0 {
            RECEIPT_SUBMITTED_ATOMIC => {
                if pending.event_is_press {
                    pending
                        .speculative
                        .layout_gesture
                        .handled_press_keycodes
                        .remove(&pending.event_keycode);
                }
                self.commit_atomic_speculation(pending.speculative, pending.base, true)
            }
            RECEIPT_CONSUMED_NO_EFFECT => {
                self.commit_atomic_speculation(pending.speculative, pending.base, false)
            }
            RECEIPT_REFUSED_ZERO_EFFECT
            | RECEIPT_FOCUS_LINEAGE_TERMINATED
            | RECEIPT_SUBMISSION_UNCERTAIN_NO_RETRY => true,
            _ => false,
        };
        if !applied {
            self.revoke_context_word();
            return false;
        }
        if matches!(
            receipt.0,
            RECEIPT_SUBMITTED_ATOMIC | RECEIPT_CONSUMED_NO_EFFECT
        ) {
            self.settle_context_key_callback(
                callback.as_ref(),
                event.0,
                event.1,
                event.2,
                &event.3,
                true,
            );
        } else {
            self.settle_context_callback_without_input(callback.as_ref());
            // Refused atomic output can still reach the client natively;
            // uncertain submission and terminated focus also leave no exact
            // tail proof. Retain the mirror, but retire its word authority.
            self.revoke_context_word();
        }
        true
    }

    fn commit_native_observation(&mut self, speculative: &LayIbusEngine) {
        self.layout_gesture.shift_active = speculative.layout_gesture.shift_active;
        self.layout_gesture.shift_used_as_modifier =
            speculative.layout_gesture.shift_used_as_modifier;
        self.layout_gesture.shift_pressed_at = speculative.layout_gesture.shift_pressed_at;
        self.layout_gesture.last_shift_release_at =
            speculative.layout_gesture.last_shift_release_at;
        self.layout_gesture.alt_completion_active =
            speculative.layout_gesture.alt_completion_active;
        self.layout_gesture.alt_used_as_modifier = speculative.layout_gesture.alt_used_as_modifier;
    }

    fn commit_native_tail_observation(
        &mut self,
        speculative: &LayIbusEngine,
        keyval: u32,
        state: u32,
        admission_token: Option<&AdmissionToken>,
    ) {
        if keyval != KEY_BACKSPACE
            || !is_key_press(state)
            || !self.composition.buffer.is_empty()
            || speculative.committed_tail.buffer == self.committed_tail.buffer
        {
            return;
        }
        let mut expected = self.committed_tail.buffer.clone();
        expected.pop();
        if speculative.committed_tail.buffer != expected
            || (self.context_admission_required
                && !admission_token.is_some_and(|token| {
                    self.context_admission
                        .as_ref()
                        .is_some_and(|admission| admission.revalidate(token))
                }))
        {
            return;
        }
        self.backspace_committed_tail_only();
    }

    fn commit_atomic_speculation(
        &mut self,
        mut speculative: LayIbusEngine,
        base: AtomicBaseSnapshot,
        submitted_atomic_frame: bool,
    ) -> bool {
        let newer_live_surrounding = (self.client_context.surrounding_observation_revision
            > speculative.client_context.surrounding_observation_revision)
            .then(|| {
                (
                    self.client_context.surrounding_text_supported,
                    self.client_context.surrounding_text_snapshot.clone(),
                    self.client_context.surrounding_observation_revision,
                )
            });
        let live_shared = Arc::clone(&self.shared);
        let speculative_shared = speculative
            .shared
            .lock()
            .expect("lay speculative state poisoned")
            .clone();
        let mut speculative_shared = speculative_shared;
        let mut live_shared_guard = live_shared.lock().expect("lay ime state poisoned");
        let owner_tail_is_unchanged = base.active_path.as_deref() == Some(self.path.as_str())
            && live_shared_guard.active_path == base.active_path
            && live_shared_guard.handoff_tail_buffer == base.handoff_tail_buffer
            && live_shared_guard.handoff_tail_epoch == base.handoff_tail_epoch
            && live_shared_guard.handoff_focus_receipt == base.handoff_focus_receipt
            && self.client_context.runtime_owner_lease_identity
                == base.runtime_owner_lease_identity;
        if !owner_tail_is_unchanged {
            super::trace::record(
                r#"{"kind":"ibus_atomic_settlement","status":"owner_or_tail_conflict_censored"}"#,
            );
            self.committed_tail.autocorrect_suppression = None;
            #[cfg(test)]
            self.atomic
                .settlement_feedback_events
                .lock()
                .expect("atomic feedback events")
                .push("censored");
            if live_shared_guard.active_path.as_deref() == Some(self.path.as_str()) {
                self.committed_tail
                    .buffer
                    .clone_from(&live_shared_guard.handoff_tail_buffer);
                self.committed_tail.epoch = live_shared_guard.handoff_tail_epoch;
                self.client_context.focus_receipt = live_shared_guard.handoff_focus_receipt.clone();
                drop(live_shared_guard);
                self.rebuild_preedit_fast_from_tail();
            } else {
                drop(live_shared_guard);
                self.committed_tail.buffer.clear();
                self.composition.preedit_fast.reset();
                self.composition.word_input_mode = None;
            }
            return false;
        }
        if live_shared_guard.suppression_revision != base.suppression_revision {
            super::trace::record(
                r#"{"kind":"ibus_atomic_settlement","status":"live_suppression_preserved"}"#,
            );
            let live_local = match live_shared_guard.autocorrect_suppression.as_ref() {
                None if matches!(
                    self.committed_tail.autocorrect_suppression.as_ref(),
                    Some(AutocorrectSuppression::LegacyReplayV1)
                ) =>
                {
                    self.committed_tail.autocorrect_suppression.clone()
                }
                _ => live_shared_guard.autocorrect_suppression.clone(),
            };
            let live_block = AtomicSuppressionBlock {
                suppression: live_shared_guard.autocorrect_suppression.clone(),
                revision: live_shared_guard.suppression_revision,
                preserve_active_path_until: live_shared_guard.preserve_active_path_until,
                exact_handoff_epoch: live_shared_guard.exact_manual_toggle_handoff_epoch,
                exact_handoff_path: live_shared_guard.exact_manual_toggle_handoff_path.clone(),
            };
            speculative_shared.autocorrect_suppression = live_block.suppression.clone();
            speculative_shared.suppression_revision = live_block.revision;
            speculative_shared.preserve_active_path_until = live_block.preserve_active_path_until;
            speculative_shared.exact_manual_toggle_handoff_epoch = live_block.exact_handoff_epoch;
            speculative_shared.exact_manual_toggle_handoff_path = live_block.exact_handoff_path;
            speculative.committed_tail.autocorrect_suppression = live_local;
        }
        speculative.shared = Arc::clone(&live_shared);
        speculative.atomic.speculation = false;
        *live_shared_guard = speculative_shared;
        drop(live_shared_guard);
        *self = speculative;
        if submitted_atomic_frame {
            if let Ok(mut shared) = self.shared.lock() {
                if let Some(pending) = shared.pending_auto_undo.as_mut() {
                    pending.atomic_submission_proven = true;
                }
            }
        }
        if let Some((supported, snapshot, revision)) = newer_live_surrounding {
            self.client_context.surrounding_text_supported = supported;
            self.client_context.surrounding_text_snapshot = snapshot;
            self.client_context.surrounding_observation_revision = revision;
            self.observe_visible_postcondition();
        }
        self.apply_deferred_layout_actions();
        self.apply_deferred_learning_actions();
        true
    }

    pub(super) fn record_reverted_system_apply(
        &mut self,
        original: &str,
        rejected: &str,
        transition: lay::typing_cpu::ObservedSystemTransition,
    ) {
        if self.atomic.speculation {
            self.atomic.deferred_learning_actions.push(
                DeferredLearningAction::RevertedSystemApply {
                    original: original.to_string(),
                    rejected: rejected.to_string(),
                    transition,
                },
            );
        } else {
            lay::typing_cpu::TypingCpu::record_reverted_system_apply(
                original, rejected, transition,
            );
        }
    }

    fn apply_deferred_learning_actions(&mut self) {
        for action in std::mem::take(&mut self.atomic.deferred_learning_actions) {
            match action {
                DeferredLearningAction::RevertedSystemApply {
                    original,
                    rejected,
                    transition,
                } => {
                    #[cfg(test)]
                    self.atomic
                        .settlement_feedback_events
                        .lock()
                        .expect("atomic feedback events")
                        .push("reverted");
                    lay::typing_cpu::TypingCpu::record_reverted_system_apply(
                        &original, &rejected, transition,
                    )
                }
            }
        }
    }
}

fn receipt_is_compatible(disposition: u8, receipt: u8) -> bool {
    match disposition {
        PROPOSAL_FRAME_READY => matches!(
            receipt,
            RECEIPT_REFUSED_ZERO_EFFECT
                | RECEIPT_SUBMITTED_ATOMIC
                | RECEIPT_FOCUS_LINEAGE_TERMINATED
                | RECEIPT_SUBMISSION_UNCERTAIN_NO_RETRY
        ),
        PROPOSAL_CONSUMED_NO_EFFECT => matches!(
            receipt,
            RECEIPT_CONSUMED_NO_EFFECT | RECEIPT_FOCUS_LINEAGE_TERMINATED
        ),
        _ => false,
    }
}

fn valid_request(envelope: &AtomicEnvelope, capability: &AtomicCapability) -> bool {
    let profile = &capability.0;
    let lease = &capability.1;
    envelope.0 > 0
        && envelope.1 > 0
        && envelope.2 > 0
        && envelope.3 > 0
        && envelope.4 > 0
        && envelope.5 > 0
        && envelope.6.len() == DIGEST_BYTES
        && profile.0 == PROTOCOL_VERSION
        && profile.1 == ADAPTER_KIND
        && profile.2.as_slice() == PRODUCTION_DIGEST
        && profile.3 == TRANSACTION_KIND
        && profile.4 & !EFFECT_MASK == 0
        && profile.5 > 0
        && profile.5 <= MAX_EFFECTS
        && profile.6 == GUARANTEE_FLAGS
        && lease.0 > 0
        && lease.1 == envelope.2
        && lease.2 == envelope.5
        // Mutter lineage and IBus focus epoch are independent namespaces.
        && lease.3 > 0
        && lease.4 > 0
        && (!lease.5 || lease.6.len() == DIGEST_BYTES)
        && (lease.5 || lease.6.is_empty())
}

fn native_unhandled() -> AtomicProposal {
    (PROPOSAL_NATIVE_UNHANDLED, Vec::new())
}

#[cfg(test)]
pub(crate) fn td120_test_atomic_capability() -> AtomicCapability {
    (
        (
            PROTOCOL_VERSION,
            ADAPTER_KIND,
            PRODUCTION_DIGEST.to_vec(),
            TRANSACTION_KIND,
            EFFECT_MASK,
            MAX_EFFECTS,
            GUARANTEE_FLAGS,
        ),
        (11, 12, 13, 14, 15, true, vec![7; DIGEST_BYTES]),
    )
}

#[cfg(test)]
pub(crate) fn td120_test_defer_reverted_feedback_on_pending(engine: &LayIbusEngine) {
    let mut pending = pending_transitions()
        .lock()
        .expect("lay atomic pending state poisoned");
    let transition = pending
        .get_mut(&engine.path)
        .expect("real atomic proposal must own a pending transition");
    transition
        .speculative
        .atomic
        .deferred_learning_actions
        .push(DeferredLearningAction::RevertedSystemApply {
            original: "source".to_string(),
            rejected: "target".to_string(),
            transition: lay::typing_cpu::ObservedSystemTransition::Correction,
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use lay::config::LayConfig;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn engine() -> LayIbusEngine {
        static NEXT_PATH: AtomicU64 = AtomicU64::new(1);
        let config = LayConfig {
            text_backend: "ime".to_string(),
            ..LayConfig::default()
        };
        let mut engine = LayIbusEngine::new(
            format!(
                "/io/github/radislabus_star/LayIme/test/{}",
                NEXT_PATH.fetch_add(1, Ordering::Relaxed)
            ),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            config,
        );
        assert!(engine.bind_focus_path());
        engine
    }

    fn atomic_base(engine: &LayIbusEngine) -> AtomicBaseSnapshot {
        let shared = engine.shared.lock().expect("live shared state");
        engine.atomic_base_snapshot(&shared)
    }

    fn capability() -> AtomicCapability {
        (
            (
                PROTOCOL_VERSION,
                ADAPTER_KIND,
                PRODUCTION_DIGEST.to_vec(),
                TRANSACTION_KIND,
                EFFECT_MASK,
                MAX_EFFECTS,
                GUARANTEE_FLAGS,
            ),
            (11, 12, 13, 14, 15, true, vec![7; DIGEST_BYTES]),
        )
    }

    #[test]
    fn production_request_keeps_focus_namespaces_independent() {
        let envelope = (1, 2, 12, 41, 5, 13, vec![8; DIGEST_BYTES]);
        assert!(valid_request(&envelope, &capability()));

        let mut wrong = capability();
        wrong.0 .1 += 1;
        assert!(!valid_request(&envelope, &wrong));

        let mut zero_effects = capability();
        zero_effects.0 .5 = 0;
        assert!(!valid_request(&envelope, &zero_effects));

        let mut too_many_effects = capability();
        too_many_effects.0 .5 = MAX_EFFECTS + 1;
        assert!(!valid_request(&envelope, &too_many_effects));

        let mut zero_lineage = capability();
        zero_lineage.1 .3 = 0;
        assert!(!valid_request(&envelope, &zero_lineage));

        let mut zero_daemon_focus = envelope.clone();
        zero_daemon_focus.3 = 0;
        assert!(!valid_request(&zero_daemon_focus, &capability()));
    }

    #[test]
    fn production_request_rejects_v22_profile_identity() {
        const V22_DIGEST: [u8; DIGEST_BYTES] = [
            0x14, 0x75, 0xb5, 0x80, 0xff, 0x96, 0x00, 0xcc, 0xfa, 0x84, 0xe4, 0x3e, 0xb9, 0xbd,
            0x50, 0xa6, 0x1f, 0xa0, 0xbc, 0xa8, 0x8b, 0x11, 0xd3, 0x3a, 0x10, 0x48, 0x77, 0xe3,
            0x6b, 0x39, 0xad, 0xac,
        ];
        let envelope = (1, 2, 12, 41, 5, 13, vec![8; DIGEST_BYTES]);
        let mut legacy = capability();
        legacy.0 .2 = V22_DIGEST.to_vec();

        assert!(!valid_request(&envelope, &legacy));
    }

    #[test]
    fn deep_clone_isolates_shared_and_engine_state() {
        let live = engine();
        let live_active_path = live
            .shared
            .lock()
            .expect("live state before clone")
            .active_path
            .clone();
        let mut speculative = live.deep_atomic_clone();
        speculative.composition.buffer = "speculative".to_string();
        speculative
            .shared
            .lock()
            .expect("speculative state")
            .active_path = Some("/speculative".to_string());

        assert!(live.composition.buffer.is_empty());
        assert_eq!(
            live.shared.lock().expect("live state").active_path,
            live_active_path
        );
        assert_ne!(live_active_path.as_deref(), Some("/speculative"));
        assert!(!Arc::ptr_eq(&live.shared, &speculative.shared));
    }

    #[test]
    fn receipt_matrix_commits_only_compatible_success() {
        let mut live = engine();
        let mut accepted = live.deep_atomic_clone();
        accepted.composition.buffer = "accepted".to_string();
        pending_transitions().lock().expect("pending state").insert(
            live.path.clone(),
            PendingAtomicTransition {
                transaction_identity: 91,
                daemon_focus_epoch: 17,
                disposition: PROPOSAL_FRAME_READY,
                event_keycode: 30,
                event_keyval: 0,
                event_state: 0,
                event_is_press: true,
                base: atomic_base(&live),
                admission_token: None,
                context_callback: None,
                context_tail_before: String::new(),
                speculative: accepted,
            },
        );

        assert!(
            live.settle_atomic_pending(17, &(RECEIPT_SUBMITTED_ATOMIC, 91, vec![3; DIGEST_BYTES]))
        );
        assert_eq!(live.composition.buffer, "accepted");

        let mut refused = live.deep_atomic_clone();
        refused.composition.buffer = "must-not-commit".to_string();
        pending_transitions().lock().expect("pending state").insert(
            live.path.clone(),
            PendingAtomicTransition {
                transaction_identity: 92,
                daemon_focus_epoch: 17,
                disposition: PROPOSAL_FRAME_READY,
                event_keycode: 31,
                event_keyval: 0,
                event_state: 0,
                event_is_press: true,
                base: atomic_base(&live),
                admission_token: None,
                context_callback: None,
                context_tail_before: String::new(),
                speculative: refused,
            },
        );
        assert!(live.settle_atomic_pending(
            17,
            &(RECEIPT_REFUSED_ZERO_EFFECT, 92, vec![4; DIGEST_BYTES])
        ));
        assert_eq!(live.composition.buffer, "accepted");
    }

    #[test]
    fn submitted_press_receipt_clears_only_its_exact_handled_marker() {
        let mut live = engine();
        let mut paired = live.deep_atomic_clone();
        paired.layout_gesture.handled_press_keycodes.insert(30);
        paired.layout_gesture.handled_press_keycodes.insert(31);
        pending_transitions().lock().expect("pending state").insert(
            live.path.clone(),
            PendingAtomicTransition {
                transaction_identity: 94,
                daemon_focus_epoch: 19,
                disposition: PROPOSAL_FRAME_READY,
                event_keycode: 30,
                event_keyval: 0,
                event_state: 0,
                event_is_press: true,
                base: atomic_base(&live),
                admission_token: None,
                context_callback: None,
                context_tail_before: String::new(),
                speculative: paired,
            },
        );

        assert!(
            live.settle_atomic_pending(19, &(RECEIPT_SUBMITTED_ATOMIC, 94, vec![6; DIGEST_BYTES]),)
        );
        assert!(!live.layout_gesture.handled_press_keycodes.contains(&30));
        assert!(live.layout_gesture.handled_press_keycodes.contains(&31));

        let mut release = live.deep_atomic_clone();
        release.layout_gesture.handled_press_keycodes.insert(32);
        pending_transitions().lock().expect("pending state").insert(
            live.path.clone(),
            PendingAtomicTransition {
                transaction_identity: 95,
                daemon_focus_epoch: 19,
                disposition: PROPOSAL_FRAME_READY,
                event_keycode: 32,
                event_keyval: 0,
                event_state: 0,
                event_is_press: false,
                base: atomic_base(&live),
                admission_token: None,
                context_callback: None,
                context_tail_before: String::new(),
                speculative: release,
            },
        );
        assert!(
            live.settle_atomic_pending(19, &(RECEIPT_SUBMITTED_ATOMIC, 95, vec![7; DIGEST_BYTES]),)
        );
        assert!(live.layout_gesture.handled_press_keycodes.contains(&32));
    }

    #[test]
    fn receipt_matrix_rejects_mismatch_duplicate_and_wrong_disposition() {
        let mut live = engine();
        let speculative = live.deep_atomic_clone();
        pending_transitions().lock().expect("pending state").insert(
            live.path.clone(),
            PendingAtomicTransition {
                transaction_identity: 93,
                daemon_focus_epoch: 18,
                disposition: PROPOSAL_CONSUMED_NO_EFFECT,
                event_keycode: 42,
                event_keyval: 0,
                event_state: 0,
                event_is_press: false,
                base: atomic_base(&live),
                admission_token: None,
                context_callback: None,
                context_tail_before: String::new(),
                speculative,
            },
        );

        assert!(
            !live.settle_atomic_pending(18, &(RECEIPT_SUBMITTED_ATOMIC, 93, vec![5; DIGEST_BYTES]))
        );
        assert!(!live
            .settle_atomic_pending(18, &(RECEIPT_CONSUMED_NO_EFFECT, 93, vec![5; DIGEST_BYTES])));
        assert!(live.settle_atomic_pending(18, &(RECEIPT_NONE, 0, Vec::new())));
    }

    #[test]
    fn native_unhandled_commits_only_modifier_observation() {
        let mut live = engine();
        let mut speculative = live.deep_atomic_clone();
        speculative.composition.buffer = "forbidden".to_string();
        speculative.layout_gesture.shift_active = true;
        speculative.layout_gesture.shift_pressed_at = Some(std::time::Instant::now());
        speculative.layout_gesture.alt_completion_active = true;

        live.commit_native_observation(&speculative);

        assert!(live.composition.buffer.is_empty());
        assert!(live.layout_gesture.shift_active);
        assert!(live.layout_gesture.shift_pressed_at.is_some());
        assert!(live.layout_gesture.alt_completion_active);
    }

    #[test]
    fn double_shift_produces_one_speculative_atomic_frame() {
        let mut live = engine();
        live.layout_gesture.layout_is_ru = false;
        live.composition.buffer = "ghbdtn".to_string();
        live.composition.cursor = live.composition.buffer.chars().count();
        let key = super::super::protocol::KEY_LEFT_SHIFT;
        let release = super::super::protocol::RELEASE_MASK;

        for (transaction, state) in [(101, 0), (102, release), (103, 0)] {
            if state == release {
                live.layout_gesture.shift_pressed_at =
                    Some(std::time::Instant::now() - std::time::Duration::from_secs(2));
            }
            let envelope = (transaction, 2, 12, 41, 5, 13, vec![8; DIGEST_BYTES]);
            let proposal = zbus::block_on(live.process_atomic_key_event(
                key,
                42,
                state,
                envelope,
                capability(),
                (RECEIPT_NONE, 0, Vec::new()),
            ))
            .expect("atomic shift observation");
            assert_eq!(proposal.0, PROPOSAL_NATIVE_UNHANDLED);
        }

        live.layout_gesture.shift_pressed_at =
            Some(std::time::Instant::now() - std::time::Duration::from_secs(2));

        let proposal = zbus::block_on(live.process_atomic_key_event(
            key,
            42,
            release,
            (104, 2, 12, 41, 5, 13, vec![8; DIGEST_BYTES]),
            capability(),
            (RECEIPT_NONE, 0, Vec::new()),
        ))
        .expect("atomic double shift");

        assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
        assert_eq!(live.composition.buffer, "ghbdtn");
        live.discard_atomic_pending();
    }

    #[test]
    fn mixed_shift_sides_cannot_complete_double_left_shift() {
        let mut live = engine();
        live.layout_gesture.layout_is_ru = false;
        live.composition.buffer = "ghbdtn".to_string();
        live.composition.cursor = live.composition.buffer.chars().count();
        let left = super::super::protocol::KEY_LEFT_SHIFT;
        let right = super::super::protocol::KEY_RIGHT_SHIFT;
        let release = super::super::protocol::RELEASE_MASK;

        for (transaction, key, state) in [
            (201, left, 0),
            (202, left, release),
            (203, right, 0),
            (204, right, release),
            (205, left, 0),
            (206, left, release),
        ] {
            let proposal = zbus::block_on(live.process_atomic_key_event(
                key,
                42,
                state,
                (transaction, 2, 12, 41, 5, 13, vec![8; DIGEST_BYTES]),
                capability(),
                (RECEIPT_NONE, 0, Vec::new()),
            ))
            .expect("mixed Shift observation");
            assert_eq!(proposal.0, PROPOSAL_NATIVE_UNHANDLED);
        }

        assert_eq!(live.composition.buffer, "ghbdtn");
    }

    #[test]
    fn td120_atomic_no_conflict_commit_abort_and_duplicate_preserve_effect_counts() {
        let mut live = engine();
        for ch in "abc".chars() {
            live.push_tail_char(ch);
        }
        assert!(live.arm_current_word_autocorrect_suppression());
        let proposal = zbus::block_on(live.process_atomic_key_event(
            super::super::protocol::KEY_SPACE,
            65,
            0,
            (301, 2, 12, 41, 5, 13, vec![8; DIGEST_BYTES]),
            capability(),
            (RECEIPT_NONE, 0, Vec::new()),
        ))
        .expect("atomic Space proposal");
        assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
        assert_eq!(proposal.1.iter().filter(|(tag, _)| *tag == 1).count(), 1);
        assert_eq!(live.committed_tail.buffer, "abc");
        td120_test_defer_reverted_feedback_on_pending(&live);

        let settled = zbus::block_on(live.process_atomic_key_event(
            super::super::protocol::KEY_LEFT_SHIFT,
            42,
            0,
            (302, 2, 12, 41, 5, 13, vec![8; DIGEST_BYTES]),
            capability(),
            (RECEIPT_SUBMITTED_ATOMIC, 301, vec![9; DIGEST_BYTES]),
        ))
        .expect("settle submitted Space");
        assert_eq!(settled.0, PROPOSAL_NATIVE_UNHANDLED);
        assert!(settled.1.is_empty());
        assert_eq!(live.committed_tail.buffer, "abc ");
        assert!(live
            .shared
            .lock()
            .expect("live state")
            .autocorrect_suppression
            .is_none());
        assert_eq!(
            *live
                .atomic
                .settlement_feedback_events
                .lock()
                .expect("atomic feedback events"),
            ["reverted"]
        );

        let duplicate = zbus::block_on(live.process_atomic_key_event(
            super::super::protocol::KEY_LEFT_SHIFT,
            42,
            super::super::protocol::RELEASE_MASK,
            (303, 2, 12, 41, 5, 13, vec![8; DIGEST_BYTES]),
            capability(),
            (RECEIPT_SUBMITTED_ATOMIC, 301, vec![9; DIGEST_BYTES]),
        ))
        .expect("duplicate receipt");
        assert_eq!(duplicate.0, PROPOSAL_NATIVE_UNHANDLED);
        assert!(duplicate.1.is_empty());
        assert_eq!(live.committed_tail.buffer, "abc ");
        assert_eq!(
            *live
                .atomic
                .settlement_feedback_events
                .lock()
                .expect("atomic feedback events"),
            ["reverted"]
        );

        let mut aborted = engine();
        for ch in "xyz".chars() {
            aborted.push_tail_char(ch);
        }
        let proposal = zbus::block_on(aborted.process_atomic_key_event(
            super::super::protocol::KEY_SPACE,
            65,
            0,
            (311, 2, 12, 51, 5, 13, vec![8; DIGEST_BYTES]),
            capability(),
            (RECEIPT_NONE, 0, Vec::new()),
        ))
        .expect("atomic abort proposal");
        assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
        let next = zbus::block_on(aborted.process_atomic_key_event(
            super::super::protocol::KEY_LEFT_SHIFT,
            42,
            0,
            (312, 2, 12, 51, 5, 13, vec![8; DIGEST_BYTES]),
            capability(),
            (RECEIPT_REFUSED_ZERO_EFFECT, 311, vec![9; DIGEST_BYTES]),
        ))
        .expect("settle refused Space");
        assert_eq!(next.0, PROPOSAL_NATIVE_UNHANDLED);
        assert_eq!(aborted.committed_tail.buffer, "xyz");
    }

    #[test]
    fn td120_atomic_guard_drift_preserves_newer_arm_consume_and_revoke() {
        let mut armed = engine();
        armed.push_tail_char('a');
        assert!(armed.arm_current_word_autocorrect_suppression());
        let (mut speculative, base) = armed.deep_atomic_clone_with_base();
        speculative.composition.buffer = "accepted-speculation".to_string();
        assert!(armed.arm_legacy_replay_autocorrect_suppression());
        let live_revision = armed
            .shared
            .lock()
            .expect("live state")
            .suppression_revision;
        pending_transitions().lock().expect("pending state").insert(
            armed.path.clone(),
            PendingAtomicTransition {
                transaction_identity: 321,
                daemon_focus_epoch: 61,
                disposition: PROPOSAL_FRAME_READY,
                event_keycode: 30,
                event_keyval: 0,
                event_state: 0,
                event_is_press: true,
                base,
                admission_token: None,
                context_callback: None,
                context_tail_before: String::new(),
                speculative,
            },
        );
        assert!(armed
            .settle_atomic_pending(61, &(RECEIPT_SUBMITTED_ATOMIC, 321, vec![1; DIGEST_BYTES])));
        assert_eq!(armed.composition.buffer, "accepted-speculation");
        let state = armed.shared.lock().expect("live state");
        assert_eq!(state.suppression_revision, live_revision);
        assert!(matches!(
            state.autocorrect_suppression.as_ref(),
            Some(AutocorrectSuppression::LegacyReplayV1)
        ));
        drop(state);

        let mut revoked = engine();
        revoked.push_tail_char('x');
        revoked.prepare_exact_manual_toggle_layout_handoff();
        let epoch = revoked.committed_tail.epoch;
        assert!(revoked.arm_exact_manual_toggle_autocorrect_suppression(
            "x",
            epoch,
            &revoked.path.clone(),
            true,
        ));
        let (speculative, base) = revoked.deep_atomic_clone_with_base();
        assert!(revoked
            .revoke_exact_manual_toggle_autocorrect_suppression(epoch, &revoked.path.clone(),));
        pending_transitions().lock().expect("pending state").insert(
            revoked.path.clone(),
            PendingAtomicTransition {
                transaction_identity: 322,
                daemon_focus_epoch: 62,
                disposition: PROPOSAL_CONSUMED_NO_EFFECT,
                event_keycode: 31,
                event_keyval: 0,
                event_state: 0,
                event_is_press: false,
                base,
                admission_token: None,
                context_callback: None,
                context_tail_before: String::new(),
                speculative,
            },
        );
        assert!(revoked.settle_atomic_pending(
            62,
            &(RECEIPT_CONSUMED_NO_EFFECT, 322, vec![2; DIGEST_BYTES])
        ));
        assert!(revoked
            .shared
            .lock()
            .expect("live state")
            .autocorrect_suppression
            .is_none());
        assert!(revoked.committed_tail.autocorrect_suppression.is_none());
    }

    #[test]
    fn td120_atomic_equal_final_revisions_compare_against_base() {
        let mut live = engine();
        live.push_tail_char('a');
        live.prepare_exact_manual_toggle_layout_handoff();
        let epoch = live.committed_tail.epoch;
        let path = live.path.clone();
        assert!(live.arm_exact_manual_toggle_autocorrect_suppression("a", epoch, &path, true,));
        let (mut speculative, base) = live.deep_atomic_clone_with_base();
        assert!(speculative.arm_legacy_replay_autocorrect_suppression());
        assert!(live.revoke_exact_manual_toggle_autocorrect_suppression(epoch, &path));
        let live_final_revision = live.shared.lock().expect("live state").suppression_revision;
        let speculative_final_revision = speculative
            .shared
            .lock()
            .expect("speculative state")
            .suppression_revision;
        assert_eq!(live_final_revision, speculative_final_revision);
        pending_transitions().lock().expect("pending state").insert(
            live.path.clone(),
            PendingAtomicTransition {
                transaction_identity: 323,
                daemon_focus_epoch: 63,
                disposition: PROPOSAL_CONSUMED_NO_EFFECT,
                event_keycode: 32,
                event_keyval: 0,
                event_state: 0,
                event_is_press: false,
                base,
                admission_token: None,
                context_callback: None,
                context_tail_before: String::new(),
                speculative,
            },
        );
        assert!(live.settle_atomic_pending(
            63,
            &(RECEIPT_CONSUMED_NO_EFFECT, 323, vec![3; DIGEST_BYTES])
        ));
        assert!(live
            .shared
            .lock()
            .expect("live state")
            .autocorrect_suppression
            .is_none());
        assert!(live.committed_tail.autocorrect_suppression.is_none());
    }

    #[test]
    fn td120_atomic_sensitive_callback_censors_real_pending_frame_and_feedback() {
        let mut sensitive = engine();
        sensitive.push_tail_char('a');
        assert!(sensitive.arm_current_word_autocorrect_suppression());
        let proposal = zbus::block_on(sensitive.process_atomic_key_event(
            super::super::protocol::KEY_SPACE,
            65,
            0,
            (324, 2, 12, 64, 5, 13, vec![8; DIGEST_BYTES]),
            capability(),
            (RECEIPT_NONE, 0, Vec::new()),
        ))
        .expect("real sensitive-conflict proposal");
        assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
        assert_eq!(proposal.1.iter().filter(|(tag, _)| *tag == 1).count(), 1);
        td120_test_defer_reverted_feedback_on_pending(&sensitive);
        sensitive.set_content_type_state(8, 0);
        let refused = zbus::block_on(sensitive.process_atomic_key_event(
            super::super::protocol::KEY_LEFT_SHIFT,
            42,
            0,
            (325, 2, 12, 64, 5, 13, vec![8; DIGEST_BYTES]),
            capability(),
            (RECEIPT_SUBMITTED_ATOMIC, 324, vec![4; DIGEST_BYTES]),
        ))
        .expect("sensitive callback settlement");
        assert_eq!(refused.0, PROPOSAL_NATIVE_UNHANDLED);
        assert!(refused.1.is_empty());
        assert!(sensitive.composition.buffer.is_empty());
        assert!(sensitive.atomic.deferred_layout_actions.is_empty());
        assert!(sensitive.atomic.deferred_learning_actions.is_empty());
        assert!(sensitive
            .shared
            .lock()
            .expect("live state")
            .autocorrect_suppression
            .is_none());
        assert_eq!(
            *sensitive
                .atomic
                .settlement_feedback_events
                .lock()
                .expect("atomic feedback events"),
            ["censored"]
        );

        let duplicate = zbus::block_on(sensitive.process_atomic_key_event(
            super::super::protocol::KEY_LEFT_SHIFT,
            42,
            0,
            (326, 2, 12, 64, 5, 13, vec![8; DIGEST_BYTES]),
            capability(),
            (RECEIPT_SUBMITTED_ATOMIC, 324, vec![4; DIGEST_BYTES]),
        ))
        .expect("duplicate sensitive receipt");
        assert_eq!(duplicate.0, PROPOSAL_NATIVE_UNHANDLED);
        assert!(duplicate.1.is_empty());
        assert_eq!(
            *sensitive
                .atomic
                .settlement_feedback_events
                .lock()
                .expect("atomic feedback events"),
            ["censored"]
        );
    }
}

#[cfg(test)]
#[path = "atomic/proof.rs"]
mod proof;
