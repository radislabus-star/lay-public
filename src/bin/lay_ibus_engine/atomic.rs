use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use zbus::fdo;

use super::engine::{DeferredLearningAction, LayIbusEngine};
use super::output::{
    AtomicEffectBuilder, AtomicProposal, EngineOutput, PROPOSAL_CONSUMED_NO_EFFECT,
    PROPOSAL_FRAME_READY, PROPOSAL_NATIVE_UNHANDLED,
};
use super::protocol::is_key_press;

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
    event_is_press: bool,
    speculative: LayIbusEngine,
}

fn pending_transitions() -> &'static Mutex<HashMap<String, PendingAtomicTransition>> {
    static PENDING: OnceLock<Mutex<HashMap<String, PendingAtomicTransition>>> = OnceLock::new();
    PENDING.get_or_init(|| Mutex::new(HashMap::new()))
}

impl LayIbusEngine {
    pub(crate) async fn process_atomic_key_event(
        &mut self,
        keyval: u32,
        keycode: u32,
        state: u32,
        envelope: AtomicEnvelope,
        capability: AtomicCapability,
        prior_receipt: AtomicPriorReceipt,
    ) -> fdo::Result<AtomicProposal> {
        if !valid_request(&envelope, &capability) {
            self.discard_atomic_pending();
            return Ok(native_unhandled());
        }

        self.atomic_route_active = true;
        if !self.settle_atomic_pending(envelope.3, &prior_receipt) {
            return Ok(native_unhandled());
        }
        self.consume_shift_gesture_handoff();

        let mut speculative = self.deep_atomic_clone();
        speculative.atomic_speculation = true;
        speculative.deferred_layout_actions.clear();
        speculative.deferred_learning_actions.clear();

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
                event_is_press: is_key_press(state),
                speculative,
            };
            pending_transitions()
                .lock()
                .expect("lay atomic pending state poisoned")
                .insert(self.path.clone(), pending);
        } else {
            self.commit_native_observation(&speculative);
            self.publish_shift_gesture_handoff();
        }
        Ok(proposal)
    }

    pub(crate) fn discard_atomic_pending(&self) {
        pending_transitions()
            .lock()
            .expect("lay atomic pending state poisoned")
            .remove(&self.path);
    }

    fn deep_atomic_clone(&self) -> Self {
        let mut speculative = self.clone();
        let shared = self.shared.lock().expect("lay ime state poisoned").clone();
        speculative.shared = Arc::new(Mutex::new(shared));
        speculative
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
            return false;
        }

        match receipt.0 {
            RECEIPT_SUBMITTED_ATOMIC => {
                if pending.event_is_press {
                    pending
                        .speculative
                        .handled_press_keycodes
                        .remove(&pending.event_keycode);
                }
                self.commit_atomic_speculation(pending.speculative, true);
                true
            }
            RECEIPT_CONSUMED_NO_EFFECT => {
                self.commit_atomic_speculation(pending.speculative, false);
                true
            }
            RECEIPT_REFUSED_ZERO_EFFECT
            | RECEIPT_FOCUS_LINEAGE_TERMINATED
            | RECEIPT_SUBMISSION_UNCERTAIN_NO_RETRY => true,
            _ => false,
        }
    }

    fn commit_native_observation(&mut self, speculative: &LayIbusEngine) {
        self.shift_active = speculative.shift_active;
        self.shift_used_as_modifier = speculative.shift_used_as_modifier;
        self.shift_pressed_at = speculative.shift_pressed_at;
        self.last_shift_release_at = speculative.last_shift_release_at;
        self.alt_completion_active = speculative.alt_completion_active;
        self.alt_used_as_modifier = speculative.alt_used_as_modifier;
    }

    fn commit_atomic_speculation(
        &mut self,
        mut speculative: LayIbusEngine,
        submitted_atomic_frame: bool,
    ) {
        let newer_live_surrounding = (self.surrounding_observation_revision
            > speculative.surrounding_observation_revision)
            .then(|| {
                (
                    self.surrounding_text_supported,
                    self.surrounding_text_snapshot.clone(),
                    self.surrounding_observation_revision,
                )
            });
        let live_shared = Arc::clone(&self.shared);
        let speculative_shared = speculative
            .shared
            .lock()
            .expect("lay speculative state poisoned")
            .clone();
        speculative.shared = Arc::clone(&live_shared);
        speculative.atomic_speculation = false;
        *live_shared.lock().expect("lay ime state poisoned") = speculative_shared;
        *self = speculative;
        if submitted_atomic_frame {
            if let Ok(mut shared) = self.shared.lock() {
                if let Some(pending) = shared.pending_auto_undo.as_mut() {
                    pending.atomic_submission_proven = true;
                }
            }
        }
        if let Some((supported, snapshot, revision)) = newer_live_surrounding {
            self.surrounding_text_supported = supported;
            self.surrounding_text_snapshot = snapshot;
            self.surrounding_observation_revision = revision;
            self.observe_visible_postcondition();
        }
        self.apply_deferred_layout_actions();
        self.apply_deferred_learning_actions();
    }

    pub(super) fn record_reverted_system_apply(
        &mut self,
        original: &str,
        rejected: &str,
        transition: lay::typing_cpu::ObservedSystemTransition,
    ) {
        if self.atomic_speculation {
            self.deferred_learning_actions
                .push(DeferredLearningAction::RevertedSystemApply {
                    original: original.to_string(),
                    rejected: rejected.to_string(),
                    transition,
                });
        } else {
            lay::typing_cpu::TypingCpu::record_reverted_system_apply(
                original, rejected, transition,
            );
        }
    }

    fn apply_deferred_learning_actions(&mut self) {
        for action in std::mem::take(&mut self.deferred_learning_actions) {
            match action {
                DeferredLearningAction::RevertedSystemApply {
                    original,
                    rejected,
                    transition,
                } => lay::typing_cpu::TypingCpu::record_reverted_system_apply(
                    &original, &rejected, transition,
                ),
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
        LayIbusEngine::new(
            format!(
                "/io/github/radislabus_star/LayIme/test/{}",
                NEXT_PATH.fetch_add(1, Ordering::Relaxed)
            ),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            config,
        )
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
        let mut speculative = live.deep_atomic_clone();
        speculative.buffer = "speculative".to_string();
        speculative
            .shared
            .lock()
            .expect("speculative state")
            .active_path = Some("/speculative".to_string());

        assert!(live.buffer.is_empty());
        assert!(live
            .shared
            .lock()
            .expect("live state")
            .active_path
            .is_none());
        assert!(!Arc::ptr_eq(&live.shared, &speculative.shared));
    }

    #[test]
    fn receipt_matrix_commits_only_compatible_success() {
        let mut live = engine();
        let mut accepted = live.deep_atomic_clone();
        accepted.buffer = "accepted".to_string();
        pending_transitions().lock().expect("pending state").insert(
            live.path.clone(),
            PendingAtomicTransition {
                transaction_identity: 91,
                daemon_focus_epoch: 17,
                disposition: PROPOSAL_FRAME_READY,
                event_keycode: 30,
                event_is_press: true,
                speculative: accepted,
            },
        );

        assert!(
            live.settle_atomic_pending(17, &(RECEIPT_SUBMITTED_ATOMIC, 91, vec![3; DIGEST_BYTES]))
        );
        assert_eq!(live.buffer, "accepted");

        let mut refused = live.deep_atomic_clone();
        refused.buffer = "must-not-commit".to_string();
        pending_transitions().lock().expect("pending state").insert(
            live.path.clone(),
            PendingAtomicTransition {
                transaction_identity: 92,
                daemon_focus_epoch: 17,
                disposition: PROPOSAL_FRAME_READY,
                event_keycode: 31,
                event_is_press: true,
                speculative: refused,
            },
        );
        assert!(live.settle_atomic_pending(
            17,
            &(RECEIPT_REFUSED_ZERO_EFFECT, 92, vec![4; DIGEST_BYTES])
        ));
        assert_eq!(live.buffer, "accepted");
    }

    #[test]
    fn submitted_press_receipt_clears_only_its_exact_handled_marker() {
        let mut live = engine();
        let mut paired = live.deep_atomic_clone();
        paired.handled_press_keycodes.insert(30);
        paired.handled_press_keycodes.insert(31);
        pending_transitions().lock().expect("pending state").insert(
            live.path.clone(),
            PendingAtomicTransition {
                transaction_identity: 94,
                daemon_focus_epoch: 19,
                disposition: PROPOSAL_FRAME_READY,
                event_keycode: 30,
                event_is_press: true,
                speculative: paired,
            },
        );

        assert!(
            live.settle_atomic_pending(19, &(RECEIPT_SUBMITTED_ATOMIC, 94, vec![6; DIGEST_BYTES]),)
        );
        assert!(!live.handled_press_keycodes.contains(&30));
        assert!(live.handled_press_keycodes.contains(&31));

        let mut release = live.deep_atomic_clone();
        release.handled_press_keycodes.insert(32);
        pending_transitions().lock().expect("pending state").insert(
            live.path.clone(),
            PendingAtomicTransition {
                transaction_identity: 95,
                daemon_focus_epoch: 19,
                disposition: PROPOSAL_FRAME_READY,
                event_keycode: 32,
                event_is_press: false,
                speculative: release,
            },
        );
        assert!(
            live.settle_atomic_pending(19, &(RECEIPT_SUBMITTED_ATOMIC, 95, vec![7; DIGEST_BYTES]),)
        );
        assert!(live.handled_press_keycodes.contains(&32));
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
                event_is_press: false,
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
        speculative.buffer = "forbidden".to_string();
        speculative.shift_active = true;
        speculative.shift_pressed_at = Some(std::time::Instant::now());
        speculative.alt_completion_active = true;

        live.commit_native_observation(&speculative);

        assert!(live.buffer.is_empty());
        assert!(live.shift_active);
        assert!(live.shift_pressed_at.is_some());
        assert!(live.alt_completion_active);
    }

    #[test]
    fn double_shift_produces_one_speculative_atomic_frame() {
        let mut live = engine();
        live.layout_is_ru = false;
        live.buffer = "ghbdtn".to_string();
        live.composition_cursor = live.buffer.chars().count();
        let key = super::super::protocol::KEY_LEFT_SHIFT;
        let release = super::super::protocol::RELEASE_MASK;

        for (transaction, state) in [(101, 0), (102, release), (103, 0)] {
            if state == release {
                live.shift_pressed_at =
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

        live.shift_pressed_at = Some(std::time::Instant::now() - std::time::Duration::from_secs(2));

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
        assert_eq!(live.buffer, "ghbdtn");
        live.discard_atomic_pending();
    }

    #[test]
    fn mixed_shift_sides_cannot_complete_double_left_shift() {
        let mut live = engine();
        live.layout_is_ru = false;
        live.buffer = "ghbdtn".to_string();
        live.composition_cursor = live.buffer.chars().count();
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

        assert_eq!(live.buffer, "ghbdtn");
    }
}

#[cfg(test)]
#[path = "atomic/proof.rs"]
mod proof;
