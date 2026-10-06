use super::*;
#[path = "residuals.rs"]
pub(crate) mod residuals;
#[path = "terminal_delivery.rs"]
mod terminal_delivery;
use crate::output::{
    AtomicEffectBuilder, AtomicProposal, EngineOutput, PROPOSAL_FRAME_READY,
    PROPOSAL_NATIVE_UNHANDLED,
};
use crate::protocol::{KEY_BACKSPACE, KEY_LEFT, KEY_LEFT_SHIFT, KEY_SPACE, KEY_TAB, RELEASE_MASK};

const RECEIPT_DIGEST: [u8; 32] = [9; 32];
const CALLBACK_BUDGET: Duration = Duration::from_secs(1);
const ENVELOPE_MUTTER_FRAME: u64 = 3;
const ENVELOPE_CLIENT: u64 = 12;
const ENVELOPE_FOCUS_EPOCH: u64 = 4;
const ENVELOPE_CONTEXT: u64 = 6;
const ENVELOPE_LEASE: u64 = 13;

#[test]
fn td121_observed_bridge_expiry_releases_only_its_own_slot() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness().await;
        complete_native_activation(&mut harness).await;
        let owner = harness.adapter.current_owner();
        let token = harness.adapter.current_token().unwrap();
        let (fence, ()) = future::zip(
            harness.adapter.begin_bridge_fence(),
            serve_ping_and_marker(&mut harness.peer),
        )
        .await;
        let fence = fence.unwrap();
        assert!(harness.observer.process_next().await.unwrap());
        {
            let slot = harness.adapter.shared.pending.lock().unwrap();
            let current = slot.as_ref().unwrap();
            assert_eq!(current.nonce, fence.nonce);
            assert!(current.ready && current.marker_observed);
        }
        // The timer has already won the wait; marker completion runs before
        // its existing error-cleanup callback. No elapsed-time claim or sleep.
        harness.adapter.expire_fence(fence);
        assert!(harness.adapter.shared.pending.lock().unwrap().is_none());
        assert_eq!(harness.adapter.current_owner(), owner);
        assert!(harness.adapter.revalidate(&token));
        assert!(harness
            .adapter
            .shared
            .ready_activation
            .lock()
            .unwrap()
            .is_none());

        for _ in 0..2 {
            let (successor, ()) = future::zip(
                harness.adapter.begin_bridge_fence(),
                serve_ping_and_marker(&mut harness.peer),
            )
            .await;
            let successor = successor.expect("expired bridge cannot leave Busy");
            assert_ne!(successor.nonce, fence.nonce);
            harness.adapter.expire_fence(fence);
            assert_eq!(
                harness
                    .adapter
                    .shared
                    .pending
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .nonce,
                successor.nonce
            );
            assert!(harness.observer.process_next().await.unwrap());
            let admitted = harness.adapter.finish_bridge_fence(successor).unwrap();
            assert!(harness.adapter.revalidate(&admitted));
            assert_eq!(harness.adapter.current_owner(), owner);
        }
    }));
}

fn bounded<T>(work: impl std::future::Future<Output = T>) -> impl std::future::Future<Output = T> {
    // Keep large nested P2P fixture futures off the default test-thread stack.
    // The same deadline and cancellation race still cover the entire work.
    future::race(Box::pin(work), async {
        async_io::Timer::after(CALLBACK_BUDGET).await;
        panic!("bounded P2P word-scope choreography timed out")
    })
}

async fn forward_marker_bounded(peer: &mut ControlledPeer) {
    bounded(forward_marker(peer)).await;
}

async fn start_source_free_pending(harness: &Harness) {
    harness
        .adapter
        .start_native_activation(
            engine_path(TARGET_PATH),
            context(CONTEXT_PATH),
            Default::default(),
        )
        .unwrap();
}

#[test]
fn completed_acquisition_releases_pending_and_keeps_exact_finish_contract() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness().await;
        start_source_free_pending(&harness).await;
        forward_marker_bounded(&mut harness.peer).await;
        let pending_nonce = harness
            .adapter
            .shared
            .pending
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .nonce;
        let original_request = harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .request
            .as_ref()
            .unwrap()
            .generation;
        assert!(matches!(
            harness.adapter.begin_activation_request(
                engine_path(TARGET_PATH),
                BarrierNonce(pending_nonce.0 + 1),
                ReceiptOrigin::Native,
                Default::default(),
                None,
            ),
            Err(AdapterError::Busy)
        ));
        assert_eq!(
            harness
                .adapter
                .shared
                .reducer
                .lock()
                .unwrap()
                .request
                .as_ref()
                .unwrap()
                .generation,
            original_request
        );
        assert_eq!(
            harness
                .adapter
                .shared
                .pending
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .nonce,
            pending_nonce
        );
        assert!(harness.observer.process_next().await.unwrap());
        let ready = harness
            .adapter
            .shared
            .ready_activation
            .lock()
            .unwrap()
            .clone()
            .unwrap();
        assert!(harness.adapter.shared.pending.lock().unwrap().is_none());
        let wrong = PendingFence {
            nonce: BarrierNonce(ready.fence.nonce.0 + 1),
            ..ready.fence
        };
        assert!(harness.adapter.complete_activation(wrong).await.is_err());
        assert!(harness
            .adapter
            .activation_outcome_is_current(&ready.outcome));
        assert!(harness
            .adapter
            .complete_activation(ready.fence)
            .await
            .is_ok());
        assert!(harness
            .adapter
            .complete_activation(ready.fence)
            .await
            .is_err());
    }));
}

#[test]
fn next_factory_keeps_ready_predecessor_until_delayed_source_installation() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness().await;
        let shared = Arc::new(Mutex::new(SharedState::default()));
        shared.lock().unwrap().handoff_tail_epoch = 41;
        let mut source = LayIbusEngine::new_from_component(
            TARGET_PATH.to_string(),
            shared.clone(),
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        start_source_free_pending(&harness).await;
        forward_marker_bounded(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
        let predecessor = harness
            .adapter
            .shared
            .ready_activation
            .lock()
            .unwrap()
            .clone()
            .unwrap();
        assert!(source.context_owner.is_none());
        let next_path = "/io/github/radislabus_star/LayIme/engine/next";
        let factory = Message::method_call("/org/freedesktop/IBus/Factory", "CreateEngine")
            .unwrap()
            .interface(FACTORY_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(4_100).unwrap())
            .with_flags(zbus::message::Flags::NoReplyExpected)
            .unwrap()
            .build(&"lay-us")
            .unwrap();
        send_manually_dispatched_callback(
            &mut harness.peer,
            &factory,
            "/org/freedesktop/IBus/Factory",
            FACTORY_INTERFACE,
        )
        .await;
        assert!(harness.observer.process_next().await.unwrap());
        let callback = harness
            .adapter
            .begin_factory_callback(&factory.header(), Instant::now(), profile("lay-us"))
            .await
            .unwrap();
        assert!(harness
            .adapter
            .bind_factory_target(&callback, engine_path(next_path)));
        let focus_out = method_message(
            DISPATCH_SENDER,
            4_101,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "FocusOut",
        );
        send_manually_dispatched_callback(
            &mut harness.peer,
            &focus_out,
            TARGET_PATH,
            ENGINE_INTERFACE,
        )
        .await;
        assert!(harness.observer.process_next().await.unwrap());
        let disable = method_message(
            DISPATCH_SENDER,
            4_102,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "Disable",
        );
        send_manually_dispatched_callback(
            &mut harness.peer,
            &disable,
            TARGET_PATH,
            ENGINE_INTERFACE,
        )
        .await;
        assert!(harness.observer.process_next().await.unwrap());
        let focus_in = method_message(
            DISPATCH_SENDER,
            4_103,
            next_path,
            ENGINE_INTERFACE,
            "FocusInId",
        );
        send_manually_dispatched_callback(
            &mut harness.peer,
            &focus_in,
            next_path,
            ENGINE_INTERFACE,
        )
        .await;
        assert!(harness.observer.process_next().await.unwrap());
        let observed = harness
            .adapter
            .observe_callback(&focus_in.header(), Instant::now())
            .await
            .unwrap();
        harness
            .adapter
            .start_native_activation(
                engine_path(next_path),
                context(CONTEXT_PATH),
                observed.position,
            )
            .unwrap();
        forward_marker_bounded(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
        let pending = harness
            .adapter
            .shared
            .pending
            .lock()
            .unwrap()
            .clone()
            .unwrap();
        assert_ne!(pending.nonce, predecessor.fence.nonce);
        assert_eq!(
            harness
                .adapter
                .shared
                .ready_activation
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .owner,
            predecessor.owner
        );
        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(next_path))
            .unwrap()
            .is_none());
        // An old completed fence's timer cannot erase its successor's work.
        harness.adapter.expire_fence(predecessor.fence);
        assert!(pending.marker_observed);
        assert!(matches!(pending.kind, FenceKind::Acquisition { .. }));
        // Matching observed Acquisition retains its separate ready owner.
        harness.adapter.expire_fence(PendingFence {
            nonce: pending.nonce,
            deadline: pending.deadline,
        });
        assert_eq!(
            harness
                .adapter
                .shared
                .ready_activation
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .owner,
            predecessor.owner
        );
        assert_eq!(
            harness
                .adapter
                .shared
                .pending
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .nonce,
            pending.nonce
        );
        assert!(
            source
                .observe_context_focus_out(&focus_out.header(), Instant::now())
                .await
        );
        assert!(source.context_owner.is_some());
        assert!(source.committed_tail.epoch > 41);
        assert!(source.committed_tail.buffer.is_empty());
        assert!(!source.context_word_is_known());
        assert!(harness.adapter.shared.pending.lock().unwrap().is_none());
        let outcome = harness
            .adapter
            .try_finish_activation_for(&engine_path(next_path))
            .unwrap()
            .expect("successor becomes ready after exact source installation/seal");
        let mut target = LayIbusEngine::new_from_component(
            next_path.to_string(),
            shared.clone(),
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        assert!(target.install_context_activation(outcome));
        assert!(target.live_context_token().is_some());
        assert!(!target.context_word_is_known());
        assert!(target.committed_tail.buffer.is_empty());
        assert_eq!(
            shared.lock().unwrap().active_path.as_deref(),
            Some(next_path)
        );
        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .is_none());
        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(next_path))
            .unwrap()
            .is_none());
    }));
}

fn legacy_message(serial: u32) -> Message {
    legacy_key_message_at(serial, TARGET_PATH, KEY_SPACE, 57, 0)
}

fn legacy_key_message_at(
    serial: u32,
    path: &str,
    keyval: u32,
    keycode: u32,
    state: u32,
) -> Message {
    Message::method_call(path, "ProcessKeyEvent")
        .unwrap()
        .interface(ENGINE_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(std::num::NonZeroU32::new(serial).unwrap())
        .with_flags(zbus::message::Flags::NoReplyExpected)
        .unwrap()
        .build(&(keyval, keycode, state))
        .unwrap()
}

async fn run_received_legacy_key(
    harness: &Harness,
    engine: &mut LayIbusEngine,
    key: &Message,
    keyval: u32,
    keycode: u32,
    state: u32,
) -> bool {
    let emitter = zbus::object_server::SignalEmitter::new(&harness.connection, TARGET_PATH)
        .expect("legacy signal emitter");
    bounded(engine.process_key_event(key.header(), emitter, keyval, keycode, state))
        .await
        .expect("legacy ProcessKeyEvent result")
}

async fn legacy_key(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    serial: u32,
    keyval: u32,
    keycode: u32,
    state: u32,
) -> bool {
    let key = legacy_key_message_at(serial, TARGET_PATH, keyval, keycode, state);
    harness.peer.detached_callback_reply_serials.insert(serial);
    harness.peer.connection.send(&key).await.unwrap();
    // Semantic callback fixtures own a received observer stamp before the
    // engine callback consumes it. Rendezvous wait and timeout schedules have
    // dedicated adapter tests; racing every semantic fixture against the 1 ms
    // product deadline only measures host scheduling jitter.
    assert!(bounded(harness.observer.process_next())
        .await
        .expect("legacy key observer result"));
    let emitter = zbus::object_server::SignalEmitter::new(&harness.connection, TARGET_PATH)
        .expect("legacy signal emitter");
    bounded(engine.process_key_event(key.header(), emitter, keyval, keycode, state))
        .await
        .expect("legacy ProcessKeyEvent result")
}

pub(super) fn assert_input_mode_update(
    message: &Message,
    engine: &LayIbusEngine,
    expected_mode: bool,
) {
    let header = message.header();
    assert_eq!(header.message_type(), zbus::message::Type::Signal);
    assert_eq!(header.path().unwrap().as_str(), engine.path);
    assert_eq!(header.interface().unwrap().as_str(), ENGINE_INTERFACE);
    assert_eq!(header.member().unwrap().as_str(), "UpdateProperty");
    let owner = engine.context_owner.as_ref().expect("installed mode owner");
    assert_eq!(owner.path.as_str(), engine.path);
    assert_eq!(
        engine
            .context_admission
            .as_ref()
            .unwrap()
            .current_owner()
            .as_ref(),
        Some(owner)
    );
    assert!(engine.live_context_token().is_some());
    let shared = engine.shared.lock().unwrap();
    assert_eq!(shared.active_path.as_deref(), Some(engine.path.as_str()));
    assert_eq!(shared.context_owner_generation, Some(owner.generation.0));
    let body = message.body();
    let (value,): (zbus::zvariant::Value<'_>,) = body.deserialize().unwrap();
    let zbus::zvariant::Value::Structure(property) = value else {
        panic!("UpdateProperty must carry a single IBusProperty");
    };
    let fields = property.fields();
    assert_eq!(fields.len(), 12);
    assert!(matches!(&fields[0], zbus::zvariant::Value::Str(v) if v.as_str() == "IBusProperty"));
    assert!(matches!(&fields[2], zbus::zvariant::Value::Str(v) if v.as_str() == "InputMode"));
    assert!(matches!(&fields[3], zbus::zvariant::Value::U32(0)));
    assert!(matches!(&fields[7], zbus::zvariant::Value::Bool(true)));
    assert!(matches!(&fields[8], zbus::zvariant::Value::Bool(false)));
    assert!(matches!(&fields[9], zbus::zvariant::Value::U32(0)));
    for index in [4, 11] {
        let zbus::zvariant::Value::Value(text) = &fields[index] else {
            panic!("InputMode label and symbol must be IBusText variants");
        };
        assert_eq!(
            crate::ibus_interface::ibus_text_value_to_string(text.as_ref()).as_deref(),
            Some(if expected_mode { "RU" } else { "EN" })
        );
    }
}

pub(super) async fn expect_activation_input_mode_update(
    harness: &mut Harness,
    engine: &LayIbusEngine,
    expected_mode: bool,
) {
    let update = bounded(next_peer_message(&mut harness.peer)).await;
    assert_input_mode_update(&update, engine, expected_mode);
}

pub(super) async fn next_legacy_text_effect(
    peer: &mut ControlledPeer,
    engine: &LayIbusEngine,
    expected_mode: bool,
) -> Message {
    let first = bounded(next_peer_message(peer)).await;
    if first
        .header()
        .member()
        .is_some_and(|member| member.as_str() == "UpdateProperty")
    {
        assert_input_mode_update(&first, engine, expected_mode);
        // Exactly one admitted metadata frame may precede the text effect.
        // A second property or any other signal is returned to the unchanged
        // CommitText assertion and fails there; no arbitrary skip loop.
        bounded(next_peer_message(peer)).await
    } else {
        first
    }
}

// ADR 2026-10-06-native-space-observed-boundary: ordinary Legacy NoApply
// passes the one original Space. The pre-key display state fixes whether its
// close must emit two clear/hide signals; no Commit/Delete may be present.
pub(super) async fn expect_legacy_native_space(
    harness: &mut Harness,
    engine: &LayIbusEngine,
    expected_mode: bool,
    was_visible: bool,
) {
    let effects = terminal_delivery::legacy_effects(harness).await;
    let first = if effects.first().is_some_and(|message| {
        message
            .header()
            .member()
            .is_some_and(|member| member.as_str() == "UpdateProperty")
    }) {
        assert_input_mode_update(&effects[0], engine, expected_mode);
        1
    } else {
        0
    };
    let text_effects = &effects[first..];
    if !was_visible {
        assert!(
            text_effects.is_empty(),
            "native Space with no display must emit no text effect"
        );
        return;
    }
    assert_eq!(
        text_effects.len(),
        2,
        "native Space closes exactly one visible display"
    );
    for effect in text_effects {
        assert_eq!(effect.header().message_type(), Type::Signal);
        assert_eq!(effect.header().path().unwrap().as_str(), engine.path);
        assert_eq!(
            effect.header().interface().unwrap().as_str(),
            ENGINE_INTERFACE
        );
    }
    assert_eq!(
        text_effects[0].header().member().unwrap().as_str(),
        "UpdatePreeditText"
    );
    assert_eq!(
        text_effects[1].header().member().unwrap().as_str(),
        "HidePreeditText"
    );
    let body = text_effects[0].body();
    let (text, cursor, visible, mode) = body
        .deserialize::<(zbus::zvariant::Value<'_>, u32, bool, u32)>()
        .unwrap();
    assert_eq!(
        crate::ibus_interface::ibus_text_value_to_string(&text),
        Some(String::new())
    );
    assert_eq!((cursor, visible, mode), (0, false, 0));
    assert!(text_effects[1].body().deserialize::<()>().is_ok());
}

async fn expect_legacy_commit(
    peer: &mut ControlledPeer,
    engine: &LayIbusEngine,
    expected_mode: bool,
) {
    let commit = next_legacy_text_effect(peer, engine, expected_mode).await;
    assert_eq!(commit.header().message_type(), zbus::message::Type::Signal);
    assert_eq!(
        commit.header().interface().unwrap().as_str(),
        ENGINE_INTERFACE
    );
    assert_eq!(commit.header().member().unwrap().as_str(), "CommitText");
}

async fn atomic_callback(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    callback_identity: (u32, u64),
    keyval: u32,
    keycode: u32,
    state: u32,
    prior: (u8, u64, Vec<u8>),
) -> AtomicProposal {
    let (serial, transaction) = callback_identity;
    let key = method_message(
        DISPATCH_SENDER,
        serial,
        TARGET_PATH,
        ENGINE_INTERFACE,
        "ProcessKeyEventAtomicV1",
    );
    send_manually_dispatched_callback(&mut harness.peer, &key, TARGET_PATH, ENGINE_INTERFACE).await;
    bounded(async {
        let (proposal, observed) = future::zip(
            crate::window_interaction::WindowInteraction::process_atomic_key(
                engine,
                &key.header(),
                keyval,
                keycode,
                state,
                (
                    transaction,
                    ENVELOPE_MUTTER_FRAME,
                    ENVELOPE_CLIENT,
                    ENVELOPE_FOCUS_EPOCH,
                    ENVELOPE_CONTEXT,
                    ENVELOPE_LEASE,
                    vec![8; 32],
                ),
                td120_test_atomic_capability(),
                prior,
            ),
            harness.observer.process_next(),
        )
        .await;
        assert!(observed.unwrap(), "observer must receive the real callback");
        proposal.unwrap()
    })
    .await
}

fn prior_from(proposal: &AtomicProposal, transaction: u64) -> (u8, u64, Vec<u8>) {
    if proposal.0 == PROPOSAL_FRAME_READY {
        (2, transaction, RECEIPT_DIGEST.to_vec())
    } else {
        (0, 0, Vec::new())
    }
}

struct AtomicDriver<'a> {
    harness: &'a mut Harness,
    engine: &'a mut LayIbusEngine,
    serial: u32,
    transaction: u64,
    prior: (u8, u64, Vec<u8>),
}

impl<'a> AtomicDriver<'a> {
    fn new(harness: &'a mut Harness, engine: &'a mut LayIbusEngine, seed: u32) -> Self {
        Self {
            harness,
            engine,
            serial: seed,
            transaction: u64::from(seed),
            prior: (0, 0, Vec::new()),
        }
    }

    async fn press(&mut self, keyval: u32, keycode: u32, state: u32) -> AtomicProposal {
        let transaction = self.transaction;
        let proposal = atomic_callback(
            self.harness,
            self.engine,
            (self.serial, transaction),
            keyval,
            keycode,
            state,
            self.prior.clone(),
        )
        .await;
        self.prior = prior_from(&proposal, transaction);
        self.serial += 1;
        self.transaction += 1;
        assert!(
            self.engine.live_context_token().is_some(),
            "valid atomic envelope must retain the live source-free owner"
        );
        proposal
    }
}

async fn start_source_free_unknown(harness: &mut Harness, engine: &mut LayIbusEngine) {
    start_source_free_pending(harness).await;
    forward_marker_bounded(&mut harness.peer).await;
    assert!(bounded(harness.observer.process_next())
        .await
        .expect("marker observer result"));
    assert!(engine.live_composition_enabled());
}

#[test]
fn legacy_marker_before_space_press_promotes_known_start() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;

        let input_mode_before_key = engine.layout_gesture.layout_is_ru;
        let native_space_was_visible = engine.composition.preedit_visible;
        assert!(!legacy_key(&mut harness, &mut engine, 1_780, KEY_SPACE, 57, 0).await);
        expect_legacy_native_space(
            &mut harness,
            &engine,
            input_mode_before_key,
            native_space_was_visible,
        )
        .await;
        assert_eq!(engine.committed_tail.buffer, " ");
        assert!(engine.live_context_token().is_some());
        assert!(engine.context_word_is_known());

        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;

        let shift_mode_before_key = engine.layout_gesture.layout_is_ru;
        assert!(!legacy_key(&mut harness, &mut engine, 1_798, KEY_LEFT_SHIFT, 42, 0,).await);
        assert!(
            !legacy_key(
                &mut harness,
                &mut engine,
                1_799,
                KEY_LEFT_SHIFT,
                42,
                RELEASE_MASK,
            )
            .await
        );
        assert!(engine.committed_tail.buffer.is_empty());
        assert!(engine.live_context_token().is_some());
        assert!(!engine.context_word_is_known());
        let property = bounded(next_peer_message(&mut harness.peer)).await;
        assert_input_mode_update(&property, &engine, shift_mode_before_key);

        let (fence, ()) = future::zip(
            harness.adapter.begin_bridge_fence(),
            serve_ping_and_marker(&mut harness.peer),
        )
        .await;
        let fence = fence.expect("legacy readiness bridge fence");
        assert!(bounded(harness.observer.process_next())
            .await
            .expect("bridge marker observer result"));
        let token = harness
            .adapter
            .finish_bridge_fence(fence)
            .expect("legacy readiness bridge token");
        assert!(harness.adapter.revalidate(&token));
        assert!(!engine.context_word_is_known());

        let input_mode_before_key = engine.layout_gesture.layout_is_ru;
        let native_space_was_visible = engine.composition.preedit_visible;
        assert!(!legacy_key(&mut harness, &mut engine, 1_800, KEY_SPACE, 57, 0).await);
        expect_legacy_native_space(
            &mut harness,
            &engine,
            input_mode_before_key,
            native_space_was_visible,
        )
        .await;
        assert_eq!(engine.committed_tail.buffer, " ");
        assert!(engine.live_context_token().is_some());
        assert!(engine.context_word_is_known());
    }));
}

#[test]
fn legacy_pre_marker_space_schedules_remain_unknown_until_next_boundary() {
    zbus::block_on(bounded(async {
        // The observer receives the press before the marker, but the actual
        // legacy handler starts only after the marker has made the activation
        // ready. Installation cannot retrospectively authorize that press.
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_pending(&harness).await;
        let key = legacy_message(1_900);
        send_manually_dispatched_callback(&mut harness.peer, &key, TARGET_PATH, ENGINE_INTERFACE)
            .await;
        assert!(bounded(harness.observer.process_next())
            .await
            .expect("pre-marker press observer result"));
        forward_marker_bounded(&mut harness.peer).await;
        assert!(bounded(harness.observer.process_next())
            .await
            .expect("marker observer result"));

        let input_mode_before_key = engine.layout_gesture.layout_is_ru;
        let native_space_was_visible = engine.composition.preedit_visible;
        assert!(!run_received_legacy_key(&harness, &mut engine, &key, KEY_SPACE, 57, 0).await);
        expect_legacy_native_space(
            &mut harness,
            &engine,
            input_mode_before_key,
            native_space_was_visible,
        )
        .await;
        assert_eq!(engine.committed_tail.buffer, " ");
        assert!(engine.live_context_token().is_some());
        assert!(!engine.context_word_is_known());

        let input_mode_before_key = engine.layout_gesture.layout_is_ru;
        let native_space_was_visible = engine.composition.preedit_visible;
        assert!(!legacy_key(&mut harness, &mut engine, 1_901, KEY_SPACE, 57, 0).await);
        expect_legacy_native_space(
            &mut harness,
            &engine,
            input_mode_before_key,
            native_space_was_visible,
        )
        .await;
        assert!(engine.context_word_is_known());

        // The private-client trace also permits a tighter schedule: the press
        // native press is observed before readiness, then source-free installation is
        // consumed by its matching release. Installation, not release logic,
        // clears the unowned local tail and completeness stays UnknownStart.
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_pending(&harness).await;
        let input_mode_before_key = engine.layout_gesture.layout_is_ru;
        let native_space_was_visible = engine.composition.preedit_visible;
        assert!(!legacy_key(&mut harness, &mut engine, 1_910, KEY_SPACE, 57, 0).await);
        assert_eq!(engine.committed_tail.buffer, " ");
        assert!(engine.live_context_token().is_none());

        forward_marker_bounded(&mut harness.peer).await;
        assert!(bounded(harness.observer.process_next())
            .await
            .expect("marker observer result"));
        expect_legacy_native_space(
            &mut harness,
            &engine,
            input_mode_before_key,
            native_space_was_visible,
        )
        .await;
        assert!(
            !legacy_key(
                &mut harness,
                &mut engine,
                1_911,
                KEY_SPACE,
                57,
                RELEASE_MASK,
            )
            .await
        );
        assert!(engine.committed_tail.buffer.is_empty());
        assert!(engine.live_context_token().is_some());
        assert!(!engine.context_word_is_known());

        let input_mode_before_key = engine.layout_gesture.layout_is_ru;
        let native_space_was_visible = engine.composition.preedit_visible;
        assert!(!legacy_key(&mut harness, &mut engine, 1_912, KEY_SPACE, 57, 0).await);
        expect_legacy_native_space(
            &mut harness,
            &engine,
            input_mode_before_key,
            native_space_was_visible,
        )
        .await;
        assert_eq!(engine.committed_tail.buffer, " ");
        assert!(engine.context_word_is_known());
    }));
}

fn new_engine(harness: &Harness) -> LayIbusEngine {
    LayIbusEngine::new_from_component(
        TARGET_PATH.to_string(),
        Arc::new(Mutex::new(SharedState::default())),
        Some(harness.adapter.clone()),
        "lay-ime-us",
        true,
        ime_config(),
    )
}

fn committed_texts(proposal: &AtomicProposal) -> Vec<String> {
    proposal
        .1
        .iter()
        .filter(|(tag, _)| *tag == 1)
        .map(|(_, value)| String::try_from(value.clone()).expect("CommitText string"))
        .collect()
}

fn assert_literal_commit(proposal: &AtomicProposal, literal: &str) {
    assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
    assert_eq!(committed_texts(proposal), vec![literal.to_string()]);
    assert!(proposal.1.iter().all(|(tag, _)| *tag != 2));
}

#[test]
fn c20_source_free_boundary_rearms_only_the_next_word() {
    zbus::block_on(bounded(async {
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        let mut keys = AtomicDriver::new(&mut harness, &mut engine, 2_000);

        assert_literal_commit(&keys.press(u32::from(b'l'), 38, 0).await, "l");
        assert_literal_commit(&keys.press(u32::from(b'j'), 36, 0).await, "j");
        assert_literal_commit(&keys.press(u32::from(b'v'), 47, 0).await, "v");
        assert_literal_commit(&keys.press(KEY_SPACE, 57, 0).await, " ");

        let settle_space = keys.press(KEY_LEFT_SHIFT, 42, 0).await;
        assert_eq!(settle_space.0, PROPOSAL_NATIVE_UNHANDLED);
        assert!(settle_space.1.is_empty());
        assert!(keys.engine.context_word_is_known());

        let boundary_backspace = keys.press(KEY_BACKSPACE, 14, 0).await;
        assert_eq!(boundary_backspace.0, PROPOSAL_NATIVE_UNHANDLED);
        assert!(boundary_backspace.1.is_empty());
        assert!(!keys.engine.context_word_is_known());

        let tab = keys.press(KEY_TAB, 15, 0).await;
        assert_eq!(tab.0, PROPOSAL_NATIVE_UNHANDLED);
        assert!(tab.1.is_empty(), "Tab must not publish correction effects");

        let mut manual_builder = AtomicEffectBuilder::default();
        let manual = {
            let mut output = EngineOutput::atomic(&mut manual_builder);
            keys.engine
                .manual_toggle_active_text_target(&mut output)
                .await
                .expect("actual manual route")
        };
        assert_eq!(manual, None);
        assert_eq!(
            manual_builder.finish(false),
            (PROPOSAL_NATIVE_UNHANDLED, Vec::new())
        );
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    }));
}

#[test]
fn c21_backspace_to_empty_does_not_manufacture_completeness() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        let mut keys = AtomicDriver::new(&mut harness, &mut engine, 2_100);

        assert_literal_commit(&keys.press(u32::from(b'l'), 38, 0).await, "l");
        let backspace = keys.press(KEY_BACKSPACE, 14, 0).await;
        assert_eq!(backspace.0, PROPOSAL_NATIVE_UNHANDLED);
        assert!(backspace.1.is_empty());
        assert!(keys.engine.committed_tail.buffer.is_empty());
        assert!(!keys.engine.context_word_is_known());

        assert_literal_commit(&keys.press(u32::from(b'j'), 36, 0).await, "j");
        assert_literal_commit(&keys.press(u32::from(b'v'), 47, 0).await, "v");
        assert!(!keys.engine.context_word_is_known());

        let tab = keys.press(KEY_TAB, 15, 0).await;
        assert_eq!(tab.0, PROPOSAL_NATIVE_UNHANDLED);
        assert!(tab.1.is_empty());
    }));
}

#[test]
fn c22_ctrl_space_and_navigation_never_rearm_unknown_start() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        let mut keys = AtomicDriver::new(&mut harness, &mut engine, 2_200);

        let ctrl_space = keys.press(KEY_SPACE, 57, 1 << 2).await;
        assert_eq!(ctrl_space.0, PROPOSAL_NATIVE_UNHANDLED);
        assert!(ctrl_space.1.is_empty());
        assert!(!keys.engine.context_word_is_known());

        let navigation = keys.press(KEY_LEFT, 105, 0).await;
        assert_eq!(navigation.0, PROPOSAL_NATIVE_UNHANDLED);
        assert!(navigation.1.is_empty());
        assert!(!keys.engine.context_word_is_known());
    }));
}

#[test]
fn c27_atomic_boundary_receipt_preserves_the_rearmed_next_word() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        let mut keys = AtomicDriver::new(&mut harness, &mut engine, 2_700);

        assert_literal_commit(&keys.press(u32::from(b'l'), 38, 0).await, "l");
        assert_literal_commit(&keys.press(KEY_SPACE, 57, 0).await, " ");
        assert_literal_commit(&keys.press(u32::from(b'j'), 36, 0).await, "j");
        assert_eq!(keys.engine.committed_tail.buffer, "l ");
        assert!(keys.engine.context_word_is_known());
        assert!(keys.engine.live_context_token().is_some());

        let settle_next_word = keys.press(KEY_LEFT_SHIFT, 42, 0).await;
        assert_eq!(settle_next_word.0, PROPOSAL_NATIVE_UNHANDLED);
        assert!(settle_next_word.1.is_empty());
        assert_eq!(keys.engine.committed_tail.buffer, "l j");
        assert!(keys.engine.context_word_is_known());
        assert!(keys.engine.live_context_token().is_some());
    }));
}
