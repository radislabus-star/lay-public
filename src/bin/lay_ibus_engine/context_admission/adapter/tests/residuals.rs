//! Final-pass adverse schedules through the production receive/callback paths.
use super::*;
use crate::bridge::LayImeBridge;
use crate::protocol::{KEY_ENTER, KEY_KP_ENTER};

async fn known_engine(harness: &mut Harness) -> LayIbusEngine {
    let mut engine = new_engine(harness);
    start_source_free_unknown(harness, &mut engine).await;
    assert!(legacy_key(harness, &mut engine, 3_000, KEY_SPACE, 57, 0).await);
    expect_legacy_commit(&mut harness.peer).await;
    assert!(legacy_key(harness, &mut engine, 3_001, KEY_SPACE, 57, RELEASE_MASK).await);
    assert!(engine.context_word_is_known());
    engine
}

async fn receive(harness: &mut Harness, serial: u32, member: &str) -> Message {
    let message = method_message(
        DISPATCH_SENDER,
        serial,
        TARGET_PATH,
        ENGINE_INTERFACE,
        member,
    );
    harness.peer.connection.send(&message).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    message
}

async fn bridge_fence(harness: &mut Harness) -> Result<AdmissionToken, AdapterError> {
    let (fence, ()) = bounded(future::zip(
        harness.adapter.begin_bridge_fence(),
        serve_ping_and_marker(&mut harness.peer),
    ))
    .await;
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    harness.adapter.finish_bridge_fence(fence.unwrap())
}

fn bridge(harness: &Harness, engine: &LayIbusEngine) -> LayImeBridge {
    LayImeBridge {
        ibus_connection: harness.connection.clone(),
        shared: engine.shared.clone(),
        context_admission_required: true,
        admission: Some(harness.adapter.clone()),
    }
}

async fn actual_focus_out(harness: &mut Harness, engine: &mut LayIbusEngine, serial: u32) {
    let focus_out = method_message(
        DISPATCH_SENDER,
        serial,
        TARGET_PATH,
        ENGINE_INTERFACE,
        "FocusOut",
    );
    harness.peer.connection.send(&focus_out).await.unwrap();
    let ((), observed) = bounded(future::zip(
        engine.focus_out(focus_out.header()),
        harness.observer.process_next(),
    ))
    .await;
    assert!(
        observed.expect("repeated FocusOut must not cancel the observer"),
        "observer stream remains live after repeated FocusOut"
    );
}

#[test]
fn residual_repeated_focus_out_revokes_locally_and_recovers_fresh_unknown_start() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = known_engine(&mut harness).await;
        let old_token = engine.live_context_token().unwrap();

        actual_focus_out(&mut harness, &mut engine, 3_002).await;
        assert!(engine.context_handoff_sealed);
        {
            let state = engine.shared.lock().unwrap();
            assert_eq!(state.active_path.as_deref(), Some(TARGET_PATH));
            assert_eq!(
                state.context_owner_generation,
                Some(old_token.owner.generation.0)
            );
        }

        // Model stale local state retained after the first sealed handoff. The
        // repeated callback is a scoped revocation and must still run the real
        // FocusOut cleanup rather than terminate the observer.
        engine.composition.buffer = "stale".to_string();
        engine.composition.preedit_visible = true;
        actual_focus_out(&mut harness, &mut engine, 3_003).await;
        assert!(!engine.context_handoff_sealed);
        assert!(engine.composition.buffer.is_empty());
        assert!(!engine.composition.preedit_visible);
        {
            let state = engine.shared.lock().unwrap();
            assert!(state.active_path.is_none());
            assert!(state.context_owner_generation.is_none());
        }
        assert!(!harness.adapter.revalidate(&old_token));

        let focus_in = receive(&mut harness, 3_004, "FocusInId").await;
        bounded(engine.focus_in_id(
            focus_in.header(),
            CONTEXT_PATH.to_string(),
            "test-client".to_string(),
        ))
        .await;
        engine.config = ime_config();
        forward_marker_bounded(&mut harness.peer).await;
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        assert!(!legacy_key(&mut harness, &mut engine, 3_005, KEY_LEFT_SHIFT, 42, 0).await);
        assert!(
            !legacy_key(
                &mut harness,
                &mut engine,
                3_006,
                KEY_LEFT_SHIFT,
                42,
                RELEASE_MASK,
            )
            .await
        );

        let fresh_token = engine.live_context_token().expect("fresh FocusIn owner");
        assert_ne!(fresh_token.owner.generation, old_token.owner.generation);
        assert_eq!(
            fresh_token.lineage.completeness,
            WordCompleteness::UnknownStart
        );
        assert!(!harness.adapter.revalidate(&old_token));

        assert!(legacy_key(&mut harness, &mut engine, 3_007, KEY_SPACE, 57, 0).await);
        expect_legacy_commit(&mut harness.peer).await;
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                3_008,
                KEY_SPACE,
                57,
                RELEASE_MASK,
            )
            .await
        );
        assert!(engine.context_word_is_known());
        assert!(bridge_fence(&mut harness).await.is_ok());
    }));
}

#[test]
fn residual_repeated_focus_out_after_reset_or_content_type_revocation_survives() {
    zbus::block_on(bounded(async {
        for content_type in [false, true] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = known_engine(&mut harness).await;
            let old_token = engine.live_context_token().unwrap();
            actual_focus_out(&mut harness, &mut engine, 3_100).await;
            assert!(engine.context_handoff_sealed);
            if content_type {
                revoke_ready(&mut harness, &old_token.owner, true).await;
            } else {
                let reset = receive(&mut harness, 3_101, "Reset").await;
                bounded(
                    engine.reset(
                        reset.header(),
                        zbus::object_server::SignalEmitter::new(
                            &harness.connection,
                            engine.path.clone(),
                        )
                        .unwrap(),
                    ),
                )
                .await
                .unwrap();
            }
            actual_focus_out(&mut harness, &mut engine, 3_102).await;
            assert!(!harness.adapter.revalidate(&old_token));
            assert!(!engine.context_handoff_sealed);
            assert!(!engine.context_word_is_known());
            let state = engine.shared.lock().unwrap();
            assert!(state.active_path.is_none());
            assert!(state.context_owner_generation.is_none());
        }
    }));
}

async fn actual_disable(harness: &mut Harness, engine: &mut LayIbusEngine, serial: u32) {
    let disable = method_message(
        DISPATCH_SENDER,
        serial,
        TARGET_PATH,
        ENGINE_INTERFACE,
        "Disable",
    );
    harness.peer.connection.send(&disable).await.unwrap();
    let ((), observed) = bounded(future::zip(
        engine.disable(disable.header()),
        harness.observer.process_next(),
    ))
    .await;
    assert!(observed.expect("lifecycle Disable refusal must not cancel observation"));
}

fn factory_message(serial: u32) -> Message {
    Message::method_call("/org/freedesktop/IBus/Factory", "CreateEngine")
        .unwrap()
        .interface(FACTORY_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .build(&"lay-us")
        .unwrap()
}

async fn legacy_key_at(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    path: &str,
    serial: u32,
    keyval: u32,
    keycode: u32,
    state: u32,
) -> bool {
    let key = method_message(
        DISPATCH_SENDER,
        serial,
        path,
        ENGINE_INTERFACE,
        "ProcessKeyEvent",
    );
    harness.peer.connection.send(&key).await.unwrap();
    let emitter = zbus::object_server::SignalEmitter::new(&harness.connection, path)
        .expect("legacy signal emitter");
    let (handled, observed) = bounded(future::zip(
        engine.process_key_event(key.header(), emitter, keyval, keycode, state),
        harness.observer.process_next(),
    ))
    .await;
    assert!(observed.expect("legacy key observer result"));
    handled.expect("legacy ProcessKeyEvent result")
}

#[derive(Clone, Copy, Debug)]
enum PendingWordLoss {
    EarlyKey,
    EarlyKeyThenTerminalContentType,
    Reset,
    ResetAfterReply,
    TerminalContentType,
    TerminalContentTypeAfterReply,
}

async fn actual_pending_word_reset(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    terminal_content_type: bool,
    serial: u32,
) {
    if terminal_content_type {
        let set = Message::method_call(TARGET_PATH, "Set")
            .unwrap()
            .interface(PROPERTIES_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(serial).unwrap())
            .build(&(
                ENGINE_INTERFACE,
                "ContentType",
                zbus::zvariant::Value::from((10u32, 0u32)),
            ))
            .unwrap();
        harness.peer.connection.send(&set).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        bounded(engine.set_content_type((10, 0), Some(set.header()))).await;
        assert_eq!(engine.client_context.content_purpose, 10);
    } else {
        let reset = receive(harness, serial, "Reset").await;
        bounded(
            engine.reset(
                reset.header(),
                zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                    .unwrap(),
            ),
        )
        .await
        .unwrap();
    }
}

async fn actual_pending_early_key(harness: &mut Harness, engine: &mut LayIbusEngine) {
    assert!(!legacy_key(harness, engine, 3_212, KEY_LEFT_SHIFT, 42, 0,).await);
    assert!(!legacy_key(harness, engine, 3_213, KEY_LEFT_SHIFT, 42, RELEASE_MASK,).await);
}

async fn held_compatibility_get_word_loss_case(word_loss: PendingWordLoss) {
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = known_engine(&mut harness).await;
    let old_token = engine.live_context_token().unwrap();
    engine.committed_tail.buffer = "old-tail-must-not-return".to_string();
    engine.rebuild_preedit_fast_from_tail();
    {
        let mut shared = engine.shared.lock().unwrap();
        shared.handoff_tail_buffer = engine.committed_tail.buffer.clone();
        shared.handoff_tail_epoch = engine.committed_tail.epoch;
    }
    assert!(engine.arm_current_word_autocorrect_suppression());

    actual_focus_out(&mut harness, &mut engine, 3_210).await;
    assert!(engine.context_handoff_sealed);

    let focus_in = receive(&mut harness, 3_211, "FocusIn").await;
    bounded(engine.focus_in_callback(focus_in.header())).await;
    engine.config = ime_config();

    let held_get = bounded(next_peer_message(&mut harness.peer)).await;
    assert_eq!(held_get.header().member().unwrap().as_str(), "Get");
    let (interface, property) = held_get.body().deserialize::<(String, String)>().unwrap();
    assert_eq!(interface, IBUS_INTERFACE);
    assert_eq!(property, "CurrentInputContext");
    let (original_request, original_nonce) = {
        let reducer = harness.adapter.shared.reducer.lock().unwrap();
        let request = reducer.request.as_ref().expect("original held-Get request");
        assert_eq!(request.origin, ReceiptOrigin::CompatibilityProperty);
        assert!(request.reply.is_none());
        (request.generation, request.nonce)
    };

    if matches!(
        word_loss,
        PendingWordLoss::EarlyKey | PendingWordLoss::EarlyKeyThenTerminalContentType
    ) {
        actual_pending_early_key(&mut harness, &mut engine).await;
    }
    match word_loss {
        PendingWordLoss::Reset => {
            actual_pending_word_reset(&mut harness, &mut engine, false, 3_214).await;
        }
        PendingWordLoss::TerminalContentType | PendingWordLoss::EarlyKeyThenTerminalContentType => {
            actual_pending_word_reset(&mut harness, &mut engine, true, 3_215).await;
        }
        PendingWordLoss::EarlyKey
        | PendingWordLoss::ResetAfterReply
        | PendingWordLoss::TerminalContentTypeAfterReply => {}
    }

    let reset_after_reply = matches!(
        word_loss,
        PendingWordLoss::ResetAfterReply | PendingWordLoss::TerminalContentTypeAfterReply
    );
    if !reset_after_reply {
        let reducer = harness.adapter.shared.reducer.lock().unwrap();
        let request = reducer
            .request
            .as_ref()
            .expect("word loss retains the original context acquisition");
        assert_eq!(request.generation, original_request);
        assert_eq!(request.nonce, original_nonce);
        assert!(request.reply.is_none());
        assert_eq!(
            reducer.lineage().completeness,
            WordCompleteness::UnknownStart
        );
    }

    let value = OwnedValue::from(ObjectPath::try_from(CONTEXT_PATH).unwrap());
    harness
        .peer
        .connection
        .reply(&held_get.header(), &value)
        .await
        .unwrap();
    let marker = bounded(next_peer_message(&mut harness.peer)).await;
    assert_eq!(marker.header().member().unwrap().as_str(), MARKER_MEMBER);
    assert_eq!(
        marker.body().deserialize::<u64>().unwrap(),
        original_nonce.0,
        "the word-loss path must retain the original marker fence"
    );
    if reset_after_reply {
        actual_pending_word_reset(
            &mut harness,
            &mut engine,
            matches!(word_loss, PendingWordLoss::TerminalContentTypeAfterReply),
            3_219,
        )
        .await;
        let reducer = harness.adapter.shared.reducer.lock().unwrap();
        let request = reducer
            .request
            .as_ref()
            .expect("marker-window reset retains the original request");
        assert_eq!(request.generation, original_request);
        assert_eq!(request.nonce, original_nonce);
        assert!(request.reply.is_some());
        assert!(request.marker_position.is_none());
        assert_eq!(
            reducer.lineage().completeness,
            WordCompleteness::UnknownStart
        );
    }
    assert!(!harness.adapter.revalidate(&old_token));
    let forwarded = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
        .unwrap()
        .sender(ADAPTER_SENDER)
        .unwrap()
        .build(&original_nonce.0)
        .unwrap();
    harness.peer.connection.send(&forwarded).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());

    assert!(!legacy_key(&mut harness, &mut engine, 3_216, KEY_LEFT_SHIFT, 42, 0,).await);
    assert!(
        !legacy_key(
            &mut harness,
            &mut engine,
            3_217,
            KEY_LEFT_SHIFT,
            42,
            RELEASE_MASK,
        )
        .await
    );
    let fresh = engine
        .live_context_token()
        .expect("the original Get and marker install a fresh context");
    assert_ne!(fresh.owner.generation, old_token.owner.generation);
    assert_eq!(fresh.lineage.completeness, WordCompleteness::UnknownStart);
    assert!(engine.committed_tail.buffer.is_empty());
    assert!(engine.committed_tail.autocorrect_suppression.is_none());
    assert!(engine.committed_tail.pending_completion_learning.is_none());
    {
        let shared = engine.shared.lock().unwrap();
        assert!(shared.handoff_tail_buffer.is_empty());
        assert!(shared.autocorrect_suppression.is_none());
    }
    assert!(engine
        .atomic
        .settlement_feedback_events
        .lock()
        .unwrap()
        .is_empty());
    assert!(bridge_fence(&mut harness).await.is_ok());

    let handled = legacy_key(&mut harness, &mut engine, 3_218, KEY_SPACE, 57, 0).await;
    if handled {
        expect_legacy_commit(&mut harness.peer).await;
    }
    assert!(engine.context_word_is_known());
    assert!(bridge_fence(&mut harness).await.is_ok());
}

#[test]
fn residual_held_compatibility_get_survives_early_key_as_empty_unknown_start() {
    zbus::block_on(bounded(held_compatibility_get_word_loss_case(
        PendingWordLoss::EarlyKey,
    )));
}

#[test]
fn residual_held_compatibility_get_survives_actual_reset_as_empty_unknown_start() {
    zbus::block_on(bounded(held_compatibility_get_word_loss_case(
        PendingWordLoss::Reset,
    )));
}

#[test]
fn residual_held_compatibility_get_survives_actual_terminal_content_type_as_empty_unknown_start() {
    zbus::block_on(bounded(held_compatibility_get_word_loss_case(
        PendingWordLoss::TerminalContentType,
    )));
}

#[test]
fn residual_held_compatibility_get_survives_early_key_then_terminal_content_type() {
    zbus::block_on(bounded(held_compatibility_get_word_loss_case(
        PendingWordLoss::EarlyKeyThenTerminalContentType,
    )));
}

#[test]
fn residual_compatibility_marker_window_survives_actual_reset_or_terminal_content_type() {
    zbus::block_on(bounded(async {
        held_compatibility_get_word_loss_case(PendingWordLoss::ResetAfterReply).await;
        held_compatibility_get_word_loss_case(PendingWordLoss::TerminalContentTypeAfterReply).await;
    }));
}

#[test]
fn residual_compatibility_refocus_without_disable_keeps_original_get_and_word() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = known_engine(&mut harness).await;
        let old_token = engine.live_context_token().unwrap();
        engine.committed_tail.buffer = "retained-word".to_string();
        engine.rebuild_preedit_fast_from_tail();
        {
            let mut shared = engine.shared.lock().unwrap();
            shared.handoff_tail_buffer = engine.committed_tail.buffer.clone();
            shared.handoff_tail_epoch = engine.committed_tail.epoch;
        }
        assert!(engine.arm_current_word_autocorrect_suppression());

        actual_focus_out(&mut harness, &mut engine, 3_220).await;
        let focus_in = receive(&mut harness, 3_221, "FocusIn").await;
        bounded(engine.focus_in_callback(focus_in.header())).await;
        engine.config = ime_config();

        let held_get = bounded(next_peer_message(&mut harness.peer)).await;
        assert_eq!(held_get.header().member().unwrap().as_str(), "Get");
        let (request_generation, nonce) = {
            let reducer = harness.adapter.shared.reducer.lock().unwrap();
            let request = reducer
                .request
                .as_ref()
                .expect("compatibility refocus request");
            assert_eq!(request.origin, ReceiptOrigin::CompatibilityProperty);
            (request.generation, request.nonce)
        };
        let value = OwnedValue::from(ObjectPath::try_from(CONTEXT_PATH).unwrap());
        harness
            .peer
            .connection
            .reply(&held_get.header(), &value)
            .await
            .unwrap();
        let marker = bounded(next_peer_message(&mut harness.peer)).await;
        assert_eq!(marker.header().member().unwrap().as_str(), MARKER_MEMBER);
        assert_eq!(marker.body().deserialize::<u64>().unwrap(), nonce.0);
        {
            let reducer = harness.adapter.shared.reducer.lock().unwrap();
            let request = reducer.request.as_ref().expect("original request retained");
            assert_eq!(request.generation, request_generation);
            assert_eq!(request.nonce, nonce);
            assert!(request.reply.is_some());
        }
        let forwarded = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
            .unwrap()
            .sender(ADAPTER_SENDER)
            .unwrap()
            .build(&nonce.0)
            .unwrap();
        harness.peer.connection.send(&forwarded).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());

        assert!(!legacy_key(&mut harness, &mut engine, 3_222, KEY_LEFT_SHIFT, 42, 0,).await);
        assert!(
            !legacy_key(
                &mut harness,
                &mut engine,
                3_223,
                KEY_LEFT_SHIFT,
                42,
                RELEASE_MASK,
            )
            .await
        );
        let fresh = engine
            .live_context_token()
            .expect("same-context compatibility refocus installs");
        assert_ne!(fresh.owner.generation, old_token.owner.generation);
        assert_eq!(fresh.lineage.completeness, WordCompleteness::KnownStart);
        assert_eq!(engine.committed_tail.buffer, "retained-word");
        assert!(engine.committed_tail.autocorrect_suppression.is_some());
        assert!(!harness.adapter.revalidate(&old_token));
        assert!(bridge_fence(&mut harness).await.is_ok());
    }));
}

#[test]
fn residual_pending_get_rejects_unrelated_or_malformed_content_type_without_revival() {
    zbus::block_on(bounded(async {
        for invalid in 0..3 {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = known_engine(&mut harness).await;
            let old_token = engine.live_context_token().unwrap();
            actual_focus_out(&mut harness, &mut engine, 3_250).await;
            let focus = receive(&mut harness, 3_251, "FocusIn").await;
            bounded(engine.focus_in_callback(focus.header())).await;
            let held_get = bounded(next_peer_message(&mut harness.peer)).await;
            assert_eq!(held_get.header().member().unwrap().as_str(), "Get");
            let (generation, nonce) = {
                let reducer = harness.adapter.shared.reducer.lock().unwrap();
                let request = reducer.request.as_ref().unwrap();
                (request.generation, request.nonce)
            };
            let interface = if invalid == 0 {
                IBUS_INTERFACE
            } else {
                ENGINE_INTERFACE
            };
            let property = if invalid == 1 {
                "UnrelatedProperty"
            } else {
                "ContentType"
            };
            let value = if invalid == 2 {
                zbus::zvariant::Value::from(10u32)
            } else {
                zbus::zvariant::Value::from((10u32, 0u32))
            };
            let set = Message::method_call(TARGET_PATH, "Set")
                .unwrap()
                .interface(PROPERTIES_INTERFACE)
                .unwrap()
                .sender(DISPATCH_SENDER)
                .unwrap()
                .serial(NonZeroU32::new(3_252).unwrap())
                .build(&(interface, property, value))
                .unwrap();
            harness.peer.connection.send(&set).await.unwrap();
            assert!(bounded(harness.observer.process_next()).await.is_err());
            assert!(harness.adapter.shared.cancelled.load(Ordering::Acquire));
            assert!(!harness.adapter.revalidate(&old_token));
            let mut reducer = harness.adapter.shared.reducer.lock().unwrap();
            assert!(reducer.request.is_none());
            let context = ContextKey::new(reducer.connection, CONTEXT_PATH).unwrap();
            assert!(!reducer.context_reply(generation, nonce, context, set.recv_position()));
            assert!(!reducer.marker(generation, nonce, set.recv_position()));
            assert!(reducer.consume().is_none());
            assert!(reducer.consume_source_free_activation().is_none());
        }
    }));
}

async fn empty_factory_stale_focus_case(predecessor_bound: bool) {
    const STALE_PATH: &str = "/io/github/radislabus_star/LayIme/engine/stale_empty_factory";

    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let shared = Arc::new(Mutex::new(SharedState::default()));
    let predecessor = factory_message(3_230);
    harness.peer.connection.send(&predecessor).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    let predecessor_callback = harness
        .adapter
        .begin_factory_callback(&predecessor.header(), Instant::now(), profile("lay-us"))
        .await
        .unwrap();
    if predecessor_bound {
        assert!(harness
            .adapter
            .bind_factory_target(&predecessor_callback, engine_path(STALE_PATH)));
    }

    let successor = factory_message(3_231);
    harness.peer.connection.send(&successor).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    let successor_callback = harness
        .adapter
        .begin_factory_callback(&successor.header(), Instant::now(), profile("lay-us"))
        .await
        .expect("later authenticated empty factory supersedes predecessor");
    assert!(!harness
        .adapter
        .bind_factory_target(&predecessor_callback, engine_path(STALE_PATH)));
    assert!(harness
        .adapter
        .bind_factory_target(&successor_callback, engine_path(TARGET_PATH)));

    let mut stale = LayIbusEngine::new_from_component(
        STALE_PATH.to_string(),
        shared.clone(),
        Some(harness.adapter.clone()),
        "lay-ime-us",
        true,
        ime_config(),
    );
    let stale_focus = method_message(
        DISPATCH_SENDER,
        3_232,
        STALE_PATH,
        ENGINE_INTERFACE,
        "FocusInId",
    );
    harness.peer.connection.send(&stale_focus).await.unwrap();
    let ((), observed) = bounded(future::zip(
        stale.focus_in_id(
            stale_focus.header(),
            CONTEXT_PATH.to_string(),
            "stale-client".to_string(),
        ),
        harness.observer.process_next(),
    ))
    .await;
    assert!(observed.unwrap());
    assert!(stale.live_context_token().is_none());

    let mut current = LayIbusEngine::new_from_component(
        TARGET_PATH.to_string(),
        shared,
        Some(harness.adapter.clone()),
        "lay-ime-us",
        true,
        ime_config(),
    );
    let current_focus = receive(&mut harness, 3_233, "FocusInId").await;
    bounded(current.focus_in_id(
        current_focus.header(),
        CONTEXT_PATH.to_string(),
        "current-client".to_string(),
    ))
    .await;
    current.config = ime_config();
    forward_marker_bounded(&mut harness.peer).await;
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    assert!(!legacy_key(&mut harness, &mut current, 3_234, KEY_LEFT_SHIFT, 42, 0,).await);
    assert!(
        !legacy_key(
            &mut harness,
            &mut current,
            3_235,
            KEY_LEFT_SHIFT,
            42,
            RELEASE_MASK,
        )
        .await
    );
    let token = current
        .live_context_token()
        .expect("successor acquires after stale predecessor FocusIn");
    assert_eq!(token.lineage.completeness, WordCompleteness::UnknownStart);
    assert!(current.committed_tail.buffer.is_empty());
    assert!(bridge_fence(&mut harness).await.is_ok());
    assert!(legacy_key(&mut harness, &mut current, 3_236, KEY_SPACE, 57, 0).await);
    expect_legacy_commit(&mut harness.peer).await;
    assert!(current.context_word_is_known());

    let live_token = current.live_context_token().unwrap();
    let retained_tail = current.committed_tail.buffer.clone();
    let late_stale_focus = method_message(
        DISPATCH_SENDER,
        3_237,
        STALE_PATH,
        ENGINE_INTERFACE,
        "FocusInId",
    );
    harness
        .peer
        .connection
        .send(&late_stale_focus)
        .await
        .unwrap();
    let ((), observed) = bounded(future::zip(
        stale.focus_in_id(
            late_stale_focus.header(),
            CONTEXT_PATH.to_string(),
            "stale-client".to_string(),
        ),
        harness.observer.process_next(),
    ))
    .await;
    assert!(observed.unwrap());
    assert!(harness.adapter.revalidate(&live_token));
    assert!(current.context_word_is_known());
    assert_eq!(current.committed_tail.buffer, retained_tail);
    assert!(bridge_fence(&mut harness).await.is_ok());
}

#[test]
fn residual_later_empty_factory_isolated_from_stale_focus_before_or_after_old_bind() {
    zbus::block_on(bounded(async {
        empty_factory_stale_focus_case(false).await;
        empty_factory_stale_focus_case(true).await;
    }));
}

#[test]
fn residual_declined_authenticated_factory_is_passive_and_later_factory_recovers() {
    zbus::block_on(bounded(async {
        const RECOVERED_PATH: &str =
            "/io/github/radislabus_star/LayIme/engine/recovered_after_declined_factory";

        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let source = known_engine(&mut harness).await;
        let old_token = source.live_context_token().unwrap();

        let first_factory = factory_message(3_180);
        harness.peer.connection.send(&first_factory).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        let first_callback = harness
            .adapter
            .begin_factory_callback(&first_factory.header(), Instant::now(), profile("lay-us"))
            .await
            .unwrap();

        // This authenticated, well-formed callback reaches the reducer while
        // the first ticket is still pending. The reducer refuses it and revokes
        // the old word authority; the observer must nevertheless remain usable.
        let declined_factory = factory_message(3_181);
        harness
            .peer
            .connection
            .send(&declined_factory)
            .await
            .unwrap();
        assert!(bounded(harness.observer.process_next())
            .await
            .expect("a declined factory transition is not observer transport loss"));
        assert!(matches!(
            harness
                .adapter
                .begin_factory_callback(
                    &declined_factory.header(),
                    Instant::now(),
                    profile("lay-us"),
                )
                .await,
            Err(AdapterError::Denied)
        ));
        assert!(!harness.adapter.revalidate(&old_token));
        {
            let reducer = harness.adapter.shared.reducer.lock().unwrap();
            assert_eq!(reducer.status(), AdmissionStatus::Revoked);
            assert!(reducer.request.is_none());
        }

        let fresh_factory = factory_message(3_182);
        harness.peer.connection.send(&fresh_factory).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        let fresh_callback = harness
            .adapter
            .begin_factory_callback(&fresh_factory.header(), Instant::now(), profile("lay-us"))
            .await
            .expect("later factory receives a fresh source-free reservation");
        assert!(!harness.adapter.bind_factory_target(
            &first_callback,
            engine_path("/io/github/radislabus_star/LayIme/engine/stale_declined_factory"),
        ));
        assert!(harness
            .adapter
            .bind_factory_target(&fresh_callback, engine_path(RECOVERED_PATH)));

        let mut recovered = LayIbusEngine::new_from_component(
            RECOVERED_PATH.to_string(),
            source.shared.clone(),
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        let focus = method_message(
            DISPATCH_SENDER,
            3_183,
            RECOVERED_PATH,
            ENGINE_INTERFACE,
            "FocusInId",
        );
        harness.peer.connection.send(&focus).await.unwrap();
        let ((), observed) = bounded(future::zip(
            recovered.focus_in_id(
                focus.header(),
                CONTEXT_PATH.to_string(),
                "controlled-client".to_string(),
            ),
            harness.observer.process_next(),
        ))
        .await;
        assert!(observed.unwrap());
        recovered.config = ime_config();
        forward_marker_bounded(&mut harness.peer).await;
        assert!(bounded(harness.observer.process_next()).await.unwrap());

        assert!(
            !legacy_key_at(
                &mut harness,
                &mut recovered,
                RECOVERED_PATH,
                3_184,
                KEY_LEFT_SHIFT,
                42,
                0,
            )
            .await
        );
        let recovered_token = recovered
            .live_context_token()
            .expect("fresh focus and key install a source-free owner");
        assert_eq!(
            recovered_token.lineage.completeness,
            WordCompleteness::UnknownStart
        );
        assert!(!harness.adapter.revalidate(&old_token));
        assert!(bridge_fence(&mut harness).await.is_ok());
    }));
}

#[test]
fn residual_later_factory_retires_revoked_transfer_and_recovers_source_free() {
    zbus::block_on(bounded(async {
        const RECOVERED_PATH: &str =
            "/io/github/radislabus_star/LayIme/engine/recovered_after_revocation";

        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut source = known_engine(&mut harness).await;
        let old_token = source.live_context_token().unwrap();
        source.committed_tail.buffer = "old-tail-must-not-transfer".to_string();
        {
            let mut shared = source.shared.lock().unwrap();
            shared.handoff_tail_buffer = source.committed_tail.buffer.clone();
            shared.handoff_tail_epoch = source.committed_tail.epoch;
        }

        let old_factory = factory_message(3_200);
        harness.peer.connection.send(&old_factory).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        let old_factory_callback = harness
            .adapter
            .begin_factory_callback(&old_factory.header(), Instant::now(), profile("lay-us"))
            .await
            .unwrap();
        assert!(harness
            .adapter
            .bind_factory_target(&old_factory_callback, engine_path(SOURCE_PATH)));

        actual_focus_out(&mut harness, &mut source, 3_201).await;
        actual_disable(&mut harness, &mut source, 3_202).await;

        let mut abandoned_target = LayIbusEngine::new_from_component(
            SOURCE_PATH.to_string(),
            source.shared.clone(),
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        let old_focus = method_message(
            DISPATCH_SENDER,
            3_203,
            SOURCE_PATH,
            ENGINE_INTERFACE,
            "FocusInId",
        );
        harness.peer.connection.send(&old_focus).await.unwrap();
        let ((), observed) = bounded(future::zip(
            abandoned_target.focus_in_id(
                old_focus.header(),
                CONTEXT_PATH.to_string(),
                "controlled-client".to_string(),
            ),
            harness.observer.process_next(),
        ))
        .await;
        assert!(observed.unwrap());

        // Hold the real marker so the revoked acquisition also retains an old
        // unobserved pending fence, matching the live recurrence boundary.
        let old_marker = bounded(next_peer_message(&mut harness.peer)).await;
        assert_eq!(
            old_marker.header().member().unwrap().as_str(),
            MARKER_MEMBER
        );
        let old_nonce = BarrierNonce(old_marker.body().deserialize::<u64>().unwrap());
        let old_fence = {
            let pending = harness.adapter.shared.pending.lock().unwrap();
            let pending = pending.as_ref().expect("old acquisition fence");
            assert_eq!(pending.nonce, old_nonce);
            PendingFence {
                nonce: pending.nonce,
                deadline: pending.deadline,
            }
        };
        let old_request = harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .request
            .as_ref()
            .expect("old transfer request")
            .generation;
        assert!(harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .context_acquisition_failed(old_request));
        assert!(!harness.adapter.revalidate(&old_token));

        let fresh_factory = factory_message(3_204);
        harness.peer.connection.send(&fresh_factory).await.unwrap();
        assert!(
            bounded(harness.observer.process_next()).await.unwrap(),
            "a later valid factory must not terminate observation on a revoked transfer"
        );
        let fresh_factory_callback = harness
            .adapter
            .begin_factory_callback(&fresh_factory.header(), Instant::now(), profile("lay-us"))
            .await
            .expect("later factory receives a source-free reservation");

        assert!(
            !harness.adapter.bind_factory_target(
                &old_factory_callback,
                engine_path("/io/github/radislabus_star/LayIme/engine/stale_factory"),
            ),
            "late binding from the retired ticket is a no-op"
        );
        assert!(harness
            .adapter
            .bind_factory_target(&fresh_factory_callback, engine_path(RECOVERED_PATH)));
        assert!(
            harness.adapter.shared.pending.lock().unwrap().is_none(),
            "factory retirement also removes the held stale fence"
        );

        let mut recovered = LayIbusEngine::new_from_component(
            RECOVERED_PATH.to_string(),
            source.shared.clone(),
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        let fresh_focus = method_message(
            DISPATCH_SENDER,
            3_205,
            RECOVERED_PATH,
            ENGINE_INTERFACE,
            "FocusInId",
        );
        harness.peer.connection.send(&fresh_focus).await.unwrap();
        let ((), observed) = bounded(future::zip(
            recovered.focus_in_id(
                fresh_focus.header(),
                CONTEXT_PATH.to_string(),
                "controlled-client".to_string(),
            ),
            harness.observer.process_next(),
        ))
        .await;
        assert!(observed.unwrap());
        recovered.config = ime_config();

        let (fresh_request, fresh_nonce) = {
            let reducer = harness.adapter.shared.reducer.lock().unwrap();
            let request = reducer.request.as_ref().expect("fresh source-free request");
            (request.generation, request.nonce)
        };
        let fresh_marker = bounded(next_peer_message(&mut harness.peer)).await;
        assert_eq!(fresh_marker.header().message_type(), Type::Signal);
        assert_eq!(
            fresh_marker.header().interface().unwrap().as_str(),
            MARKER_INTERFACE
        );
        assert_eq!(
            fresh_marker.header().member().unwrap().as_str(),
            MARKER_MEMBER
        );
        let fresh_marker_nonce = fresh_marker.body().deserialize::<u64>().unwrap();
        assert_eq!(fresh_marker_nonce, fresh_nonce.0);
        {
            let mut reducer = harness.adapter.shared.reducer.lock().unwrap();
            let revision = reducer.revocation_generation();
            assert!(!reducer.context_reply(
                old_request,
                old_nonce,
                context(CONTEXT_PATH),
                fresh_focus.recv_position(),
            ));
            assert!(!reducer.context_acquisition_failed(old_request));
            assert_eq!(reducer.revocation_generation(), revision);
            assert_eq!(reducer.request.as_ref().unwrap().generation, fresh_request);
            assert_eq!(reducer.request.as_ref().unwrap().nonce, fresh_nonce);
        }
        harness.adapter.expire_fence(old_fence);
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
            fresh_nonce,
            "late old timer cannot clear the successor fence"
        );

        let late_old_marker = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
            .unwrap()
            .sender(ADAPTER_SENDER)
            .unwrap()
            .build(&old_nonce.0)
            .unwrap();
        harness
            .peer
            .connection
            .send(&late_old_marker)
            .await
            .unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
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
            fresh_nonce,
            "late old marker cannot satisfy the successor fence"
        );

        let forwarded_fresh_marker = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
            .unwrap()
            .sender(ADAPTER_SENDER)
            .unwrap()
            .build(&fresh_marker_nonce)
            .unwrap();
        harness
            .peer
            .connection
            .send(&forwarded_fresh_marker)
            .await
            .unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        assert!(
            !legacy_key_at(
                &mut harness,
                &mut recovered,
                RECOVERED_PATH,
                3_206,
                KEY_LEFT_SHIFT,
                42,
                0,
            )
            .await
        );
        assert!(
            !legacy_key_at(
                &mut harness,
                &mut recovered,
                RECOVERED_PATH,
                3_207,
                KEY_LEFT_SHIFT,
                42,
                RELEASE_MASK,
            )
            .await
        );
        assert!(recovered.committed_tail.buffer.is_empty());
        assert!(!recovered.context_word_is_known());
        assert!(!harness.adapter.revalidate(&old_token));
        assert!(bridge_fence(&mut harness).await.is_ok());

        assert!(
            legacy_key_at(
                &mut harness,
                &mut recovered,
                RECOVERED_PATH,
                3_208,
                KEY_SPACE,
                57,
                0,
            )
            .await
        );
        expect_legacy_commit(&mut harness.peer).await;
        assert!(recovered.context_word_is_known());
        assert!(bridge_fence(&mut harness).await.is_ok());
    }));
}

#[test]
fn residual_missing_or_repeated_disable_revokes_and_completes_local_reset() {
    zbus::block_on(bounded(async {
        for opened_handoff in [false, true] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = known_engine(&mut harness).await;
            let old_token = engine.live_context_token().unwrap();
            if opened_handoff {
                actual_focus_out(&mut harness, &mut engine, 3_110).await;
                actual_disable(&mut harness, &mut engine, 3_111).await;
                assert!(engine.context_handoff_sealed);
            }
            engine.composition.buffer = "stale".to_string();
            engine.composition.preedit_visible = true;
            engine.atomic.active = true;
            actual_disable(&mut harness, &mut engine, 3_112).await;
            assert!(!engine.context_handoff_sealed);
            assert!(!engine.atomic.active);
            assert!(engine.composition.buffer.is_empty());
            assert!(!engine.composition.preedit_visible);
            assert!(!engine.context_word_is_known());
            assert!(!harness.adapter.revalidate(&old_token));
        }
    }));
}

#[test]
fn residual_lifecycle_bad_sender_or_payload_still_stops_observer() {
    zbus::block_on(bounded(async {
        for (sender, member) in [
            (":1.999", "FocusOut"),
            (":1.999", "Disable"),
            (DISPATCH_SENDER, "FocusOutId"),
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let engine = known_engine(&mut harness).await;
            let old_token = engine.live_context_token().unwrap();
            // FocusOutId requires a context string, not this unit payload.
            let message = method_message(sender, 3_120, TARGET_PATH, ENGINE_INTERFACE, member);
            harness.peer.connection.send(&message).await.unwrap();
            assert!(matches!(
                bounded(harness.observer.run()).await,
                Err(AdapterError::Denied)
            ));
            assert!(!harness.adapter.revalidate(&old_token));
            assert!(matches!(
                harness
                    .adapter
                    .observe_callback(&message.header(), Instant::now())
                    .await,
                Err(AdapterError::Cancelled)
            ));
        }
    }));
}

#[test]
fn residual_delayed_old_revocation_cannot_clean_up_different_path_successor() {
    zbus::block_on(bounded(async {
        for member in ["FocusOut", "Disable"] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut old = known_engine(&mut harness).await;
            let old_owner = old.context_owner.clone().unwrap();
            let delayed = if member == "FocusOut" {
                actual_focus_out(&mut harness, &mut old, 3_130).await;
                receive(&mut harness, 3_131, member).await
            } else {
                receive(&mut harness, 3_130, member).await
            };

            let shared = old.shared.clone();
            let grant = complete_native_activation(&mut harness).await;
            assert_eq!(grant.target_owner.path.as_str(), SOURCE_PATH);
            assert_ne!(grant.target_owner, old_owner);
            let mut successor = LayIbusEngine::new_from_component(
                SOURCE_PATH.to_string(),
                shared.clone(),
                Some(harness.adapter.clone()),
                "lay-ime-us",
                true,
                ime_config(),
            );
            assert!(successor.install_context_activation(ActivationOutcome::SourceFree(grant)));
            successor.committed_tail.buffer = "successor-tail".to_string();
            successor.publish_tail_handoff();
            let successor_token = successor.live_context_token().unwrap();
            let successor_state = {
                let state = shared.lock().unwrap();
                (
                    state.active_path.clone(),
                    state.context_owner_generation,
                    state.handoff_tail_buffer.clone(),
                    state.handoff_tail_epoch,
                )
            };
            assert_eq!(successor_state.0.as_deref(), Some(SOURCE_PATH));

            old.composition.buffer = "old-local-stale".to_string();
            old.composition.preedit_visible = true;
            old.atomic.active = true;
            if member == "FocusOut" {
                bounded(old.focus_out(delayed.header())).await;
            } else {
                bounded(old.disable(delayed.header())).await;
            }

            assert_eq!(old.composition.buffer, "old-local-stale", "{member}");
            assert!(old.composition.preedit_visible, "{member}");
            assert!(old.atomic.active, "{member}");
            let state = shared.lock().unwrap();
            assert_eq!(
                (
                    state.active_path.clone(),
                    state.context_owner_generation,
                    state.handoff_tail_buffer.clone(),
                    state.handoff_tail_epoch,
                ),
                successor_state,
                "{member}"
            );
            assert!(harness.adapter.revalidate(&successor_token), "{member}");
        }
    }));
}

#[test]
fn residual_bridge_fence_refuses_received_unsettled_key_or_focus_out() {
    zbus::block_on(bounded(async {
        for member in ["ProcessKeyEvent", "FocusOut"] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = known_engine(&mut harness).await;
            assert!(
                bridge_fence(&mut harness).await.is_ok(),
                "settled positive control"
            );
            let event = receive(&mut harness, 3_010, member).await;
            assert!(
                engine.live_context_token().is_some(),
                "ordinary current-key identity survives"
            );
            assert!(
                matches!(bridge_fence(&mut harness).await, Err(AdapterError::Denied)),
                "{member}"
            );
            if member == "ProcessKeyEvent" {
                assert!(
                    !run_received_legacy_key(&harness, &mut engine, &event, KEY_LEFT_SHIFT, 42, 0)
                        .await
                );
                assert!(
                    bridge_fence(&mut harness).await.is_ok(),
                    "settlement restores bridge reads"
                );
            }
        }
    }));
}

#[test]
fn residual_bridge_consumer_refuses_ingress_after_the_fence() {
    zbus::block_on(bounded(async {
        for member in ["ProcessKeyEvent", "FocusOut"] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let engine = known_engine(&mut harness).await;
            let token = bridge_fence(&mut harness).await.unwrap();
            let bridge = bridge(&harness, &engine);
            assert!(bridge.bridge_token_is_live(&engine, Some(&token)));
            receive(&mut harness, 3_020, member).await;
            assert!(
                harness.adapter.revalidate(&token),
                "do not invalidate the executing key"
            );
            assert!(
                !bridge.bridge_token_is_live(&engine, Some(&token)),
                "{member}"
            );
            assert_eq!(engine.committed_tail.buffer, " ");
        }
    }));
}

async fn leave_context(harness: &mut Harness, engine: &mut LayIbusEngine) {
    let focus_out = receive(harness, 3_030, "FocusOut").await;
    bounded(engine.focus_out(focus_out.header())).await;
    assert!(engine.context_handoff_sealed);
    let disable = receive(harness, 3_031, "Disable").await;
    bounded(engine.disable(disable.header())).await;
}

async fn bridge_toggle_terminal(
    harness: &mut Harness,
    engine: LayIbusEngine,
    expected: &str,
    target_is_ru: bool,
) -> LayIbusEngine {
    let path = engine.path.clone();
    let bridge = bridge(harness, &engine);
    harness
        .connection
        .object_server()
        .at(path.as_str(), engine)
        .await
        .unwrap();
    let (outcome, ()) = bounded(future::zip(bridge.manual_toggle_v3_inner(), async {
        serve_ping_and_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
    }))
    .await;
    assert_eq!(outcome.unwrap(), (1, target_is_ru));
    let commit = bounded(next_peer_message(&mut harness.peer)).await;
    assert_eq!(commit.header().member().unwrap().as_str(), "CommitText");
    let body = commit.body();
    let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
    assert_eq!(
        crate::ibus_interface::ibus_text_value_to_string(&value).as_deref(),
        Some(expected),
        "one exact terminal erase/commit frame"
    );
    // Keep the actual engine; unregister before the existing controlled
    // callback choreography so the object server cannot dispatch it twice.
    let iface = harness
        .connection
        .object_server()
        .interface::<_, LayIbusEngine>(path.as_str())
        .await
        .unwrap();
    let engine = std::mem::replace(&mut *iface.get_mut().await, new_engine(harness));
    harness
        .connection
        .object_server()
        .remove::<LayIbusEngine, _>(path.as_str())
        .await
        .unwrap();
    engine
}

fn observed_callback_without_reply(serial: u32, path: &str, member: &str) -> Message {
    Message::method_call(path, member)
        .unwrap()
        .interface(ENGINE_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .with_flags(zbus::message::Flags::NoReplyExpected)
        .unwrap()
        .build(&())
        .unwrap()
}

async fn consume_detached_callback_reply(peer: &mut ControlledPeer, serial: u32) {
    // Registering the real bridge engine starts zbus's object dispatcher.
    // Later lifecycle callbacks are driven directly while that engine is
    // detached; zbus replies UnknownObject even with NoReplyExpected. Consume
    // only that exact transport reply, never a client effect or arbitrary log.
    let reply = bounded(next_peer_message(peer)).await;
    assert_eq!(reply.header().message_type(), Type::Error);
    assert_eq!(reply.header().reply_serial().map(|n| n.get()), Some(serial));
    assert_eq!(
        reply.header().error_name().map(|n| n.as_str()),
        Some("org.freedesktop.DBus.Error.UnknownObject")
    );
}

async fn repeated_terminal_metadata(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    serial: u32,
) {
    let set = Message::method_call(engine.path.as_str(), "Set")
        .unwrap()
        .interface(PROPERTIES_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .with_flags(zbus::message::Flags::NoReplyExpected)
        .unwrap()
        .build(&(
            ENGINE_INTERFACE,
            "ContentType",
            zbus::zvariant::Value::from((10u32, 0u32)),
        ))
        .unwrap();
    harness.peer.connection.send(&set).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    consume_detached_callback_reply(&mut harness.peer, serial).await;
    engine.set_content_type((10, 0), Some(set.header())).await;
    assert_eq!(engine.client_context.content_purpose, 10);
}

#[test]
fn residual_manual_toggle_bridge_repeats_across_legacy_factory_handoffs() {
    zbus::block_on(manual_toggle_bridge_round_trips(true));
}

#[test]
fn residual_manual_toggle_first_word_without_leading_boundary_round_trips() {
    zbus::block_on(manual_toggle_bridge_round_trips(false));
}

async fn manual_toggle_bridge_round_trips(leading_boundary: bool) {
    for boundary in [false, true] {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut source = if leading_boundary {
            known_engine(&mut harness).await
        } else {
            let mut engine = new_engine(&harness);
            start_source_free_unknown(&mut harness, &mut engine).await;
            engine
        };
        source.config.auto_replace = false;
        source.client_context.content_purpose = 10;
        source.client_context.cursor_cell_width = 11;
        source.client_context.surrounding_text_supported = false;
        assert!(legacy_key(&mut harness, &mut source, 8_000, 'a' as u32, 30, 0).await);
        expect_legacy_commit(&mut harness.peer).await;
        assert!(source.committed_tail.buffer.ends_with('a'));
        if boundary {
            assert!(legacy_key(&mut harness, &mut source, 8_001, KEY_SPACE, 57, 0).await);
            expect_legacy_commit(&mut harness.peer).await;
        }
        for turn in 0..12 {
            let target_is_ru = turn % 2 == 0;
            let text = if target_is_ru { "ф" } else { "a" };
            let suffix = format!("{text}{}", if boundary { " " } else { "" });
            let payload = format!("{}{suffix}", "\u{7f}".repeat(1 + usize::from(boundary)));
            source = bridge_toggle_terminal(&mut harness, source, &payload, target_is_ru).await;
            assert!(source.committed_tail.buffer.ends_with(&suffix));
            let expected_tail = source.committed_tail.buffer.clone();
            let serial = 8_010 + turn * 10;
            let target_path = format!("{TARGET_PATH}_{boundary}_{turn}");
            let factory = Message::method_call("/org/freedesktop/IBus/Factory", "CreateEngine")
                .unwrap()
                .interface(FACTORY_INTERFACE)
                .unwrap()
                .sender(DISPATCH_SENDER)
                .unwrap()
                .serial(NonZeroU32::new(serial).unwrap())
                .with_flags(zbus::message::Flags::NoReplyExpected)
                .unwrap()
                .build(&"lay-us")
                .unwrap();
            harness.peer.connection.send(&factory).await.unwrap();
            assert!(bounded(harness.observer.process_next()).await.unwrap());
            consume_detached_callback_reply(&mut harness.peer, serial).await;
            let callback = harness
                .adapter
                .begin_factory_callback(&factory.header(), Instant::now(), profile("lay-us"))
                .await
                .unwrap();
            assert!(harness
                .adapter
                .bind_factory_target(&callback, engine_path(&target_path)));
            for (offset, member) in [(1, "FocusOut"), (2, "Disable")] {
                let event = observed_callback_without_reply(serial + offset, &source.path, member);
                harness.peer.connection.send(&event).await.unwrap();
                assert!(bounded(harness.observer.process_next()).await.unwrap());
                consume_detached_callback_reply(&mut harness.peer, serial + offset).await;
                if member == "FocusOut" {
                    source.focus_out(event.header()).await;
                    assert!(
                        source.context_handoff_sealed,
                        "turn {turn}: bridge output must seal without another key"
                    );
                } else {
                    source.disable(event.header()).await;
                }
            }
            let mut target = LayIbusEngine::new_from_component(
                target_path.clone(),
                source.shared.clone(),
                Some(harness.adapter.clone()),
                if target_is_ru {
                    "lay-ime-ru"
                } else {
                    "lay-ime-us"
                },
                true,
                ime_config(),
            );
            let focus = observed_callback_without_reply(serial + 3, &target_path, "FocusIn");
            harness.peer.connection.send(&focus).await.unwrap();
            assert!(bounded(harness.observer.process_next()).await.unwrap());
            consume_detached_callback_reply(&mut harness.peer, serial + 3).await;
            target.focus_in_callback(focus.header()).await;
            let get = bounded(next_peer_message(&mut harness.peer)).await;
            assert_eq!(get.header().member().map(|m| m.as_str()), Some("Get"));
            if turn % 2 == 0 {
                repeated_terminal_metadata(&mut harness, &mut target, serial + 4).await;
            }
            let value = OwnedValue::from(ObjectPath::try_from(CONTEXT_PATH).unwrap());
            harness
                .peer
                .connection
                .reply(&get.header(), &value)
                .await
                .unwrap();
            forward_marker_bounded(&mut harness.peer).await;
            assert!(bounded(harness.observer.process_next()).await.unwrap());
            if turn % 2 != 0 {
                repeated_terminal_metadata(&mut harness, &mut target, serial + 4).await;
            }
            let outcome = harness
                .adapter
                .try_finish_activation_for(&engine_path(&target_path))
                .unwrap()
                .expect("same-context transfer ready");
            assert!(matches!(&outcome, ActivationOutcome::Transfer(_)));
            assert!(target.install_context_activation(outcome));
            assert_eq!(target.context_word_is_known(), leading_boundary || boundary);
            assert_eq!(target.committed_tail.buffer, expected_tail);
            assert_eq!(
                target.capture_input_frame_identity().is_some(),
                leading_boundary || boundary,
                "manual projection must not promote unknown word completeness, turn {turn}"
            );
            target.config.auto_replace = false;
            target.client_context.cursor_cell_width = 11;
            target.client_context.surrounding_text_supported = false;
            source = target;
        }
    }
}

#[test]
fn residual_first_word_suffix_tracks_unicode_backspace_and_rejects_retained_prefix_after_gap() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.config.auto_replace = false;
        engine.client_context.content_purpose = 10;
        engine.client_context.cursor_cell_width = 11;
        engine.layout_gesture.layout_is_ru = true;
        for (index, code) in [30, 48, 46].into_iter().enumerate() {
            assert!(legacy_key(&mut harness, &mut engine, 9_000 + index as u32, 0, code, 0).await);
            expect_legacy_commit(&mut harness.peer).await;
            assert_eq!(
                engine
                    .context_word_scope
                    .as_ref()
                    .unwrap()
                    .lineage()
                    .observed_suffix_chars,
                index as u32 + 1
            );
            assert!(engine.context_allows_manual_toggle());
            assert!(!engine.context_word_is_known());
            assert!(engine.capture_input_frame_identity().is_none());
        }
        assert_eq!(engine.committed_tail.buffer, "фис");
        assert!(!legacy_key(&mut harness, &mut engine, 9_004, KEY_BACKSPACE, 14, 0).await);
        assert_eq!(engine.committed_tail.buffer, "фи");
        assert_eq!(
            engine
                .context_word_scope
                .as_ref()
                .unwrap()
                .lineage()
                .observed_suffix_chars,
            2
        );
        assert!(engine.context_allows_manual_toggle());

        // A command may edit the client while retaining this local mirror.
        // Subsequent typing proves only the new suffix, never the old prefix.
        assert!(!legacy_key(&mut harness, &mut engine, 9_005, 'a' as u32, 30, 1 << 2).await);
        assert_eq!(engine.committed_tail.buffer, "фи");
        assert_eq!(
            engine
                .context_word_scope
                .as_ref()
                .unwrap()
                .lineage()
                .observed_suffix_chars,
            0
        );
        assert!(!engine.context_allows_manual_toggle());
        assert!(legacy_key(&mut harness, &mut engine, 9_006, 0, 46, 0).await);
        expect_legacy_commit(&mut harness.peer).await;
        assert_eq!(engine.committed_tail.buffer, "фис");
        assert_eq!(
            engine
                .context_word_scope
                .as_ref()
                .unwrap()
                .lineage()
                .observed_suffix_chars,
            1
        );
        assert!(!engine.context_allows_manual_toggle());
        let mut effects = AtomicEffectBuilder::default();
        assert_eq!(
            engine
                .manual_toggle_active_text_target(&mut EngineOutput::atomic(&mut effects))
                .await
                .unwrap(),
            None
        );
        assert_eq!(
            effects.finish(false),
            (PROPOSAL_NATIVE_UNHANDLED, Vec::new())
        );
        assert_eq!(engine.committed_tail.buffer, "фис");
    });
}

#[test]
fn residual_first_word_manual_admission_is_terminal_only_and_revoked_with_context() {
    zbus::block_on(async {
        for gap in ["reset", "focus_out", "cursor", "tab", "sensitive"] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = new_engine(&harness);
            start_source_free_unknown(&mut harness, &mut engine).await;
            engine.config.auto_replace = false;
            engine.client_context.content_purpose = 10;
            engine.client_context.cursor_cell_width = 11;
            assert!(legacy_key(&mut harness, &mut engine, 9_100, 'a' as u32, 30, 0).await);
            expect_legacy_commit(&mut harness.peer).await;
            assert!(engine.context_allows_manual_toggle());
            engine.client_context.surrounding_text_supported = true;
            assert!(
                !engine.context_allows_manual_toggle(),
                "GTK remains unchanged"
            );
            engine.client_context.surrounding_text_supported = false;
            engine.client_context.cursor_cell_width = 0;
            assert!(
                !engine.context_allows_manual_toggle(),
                "erase geometry is mandatory"
            );
            engine.client_context.cursor_cell_width = 11;
            match gap {
                "reset" => {
                    let message = receive(&mut harness, 9_101, "Reset").await;
                    engine
                        .reset(
                            message.header(),
                            zbus::object_server::SignalEmitter::new(
                                &harness.connection,
                                engine.path.clone(),
                            )
                            .unwrap(),
                        )
                        .await
                        .unwrap();
                }
                "focus_out" => actual_focus_out(&mut harness, &mut engine, 9_101).await,
                "cursor" => {
                    assert!(!legacy_key(&mut harness, &mut engine, 9_101, KEY_LEFT, 105, 0).await);
                }
                "tab" => {
                    assert!(!legacy_key(&mut harness, &mut engine, 9_101, KEY_TAB, 15, 0).await);
                }
                "sensitive" => engine.set_content_type_state(8, 0),
                _ => unreachable!(),
            }
            assert!(!engine.context_allows_manual_toggle(), "{gap}");
            assert!(!engine.context_word_is_known(), "{gap}");
            let before = engine.committed_tail.buffer.clone();
            let mut effects = AtomicEffectBuilder::default();
            assert_eq!(
                engine
                    .manual_toggle_active_text_target(&mut EngineOutput::atomic(&mut effects))
                    .await
                    .unwrap(),
                None,
                "{gap}"
            );
            assert_eq!(
                effects.finish(false),
                (PROPOSAL_NATIVE_UNHANDLED, Vec::new())
            );
            assert_eq!(engine.committed_tail.buffer, before, "{gap}");
            assert!(engine.capture_input_frame_identity().is_none(), "{gap}");
        }
    });
}

#[test]
fn residual_first_word_observation_waits_for_atomic_submission_receipt() {
    zbus::block_on(async {
        for submitted in [false, true] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = new_engine(&harness);
            start_source_free_unknown(&mut harness, &mut engine).await;
            engine.config.auto_replace = false;
            engine.client_context.content_purpose = 10;
            engine.client_context.cursor_cell_width = 11;
            let mut keys = AtomicDriver::new(&mut harness, &mut engine, 9_200);
            assert_literal_commit(&keys.press('a' as u32, 30, 0).await, "a");
            let scope = keys.engine.context_word_scope.as_ref().unwrap();
            assert_eq!(scope.lineage().observed_suffix_chars, 0);
            assert!(keys.engine.committed_tail.buffer.is_empty());
            assert!(!keys.engine.context_allows_manual_toggle());
            if !submitted {
                // Atomic V1 disposition1 is RefusedZeroEffect; keep the exact
                // transaction and digest that identify the pending proposal.
                keys.prior.0 = 1;
            }
            let modifier = keys.press(KEY_LEFT_SHIFT, 42, 0).await;
            assert_eq!(modifier.0, PROPOSAL_NATIVE_UNHANDLED);
            assert!(modifier.1.is_empty());
            let scope = keys.engine.context_word_scope.as_ref().unwrap();
            assert_eq!(scope.lineage().observed_suffix_chars, u32::from(submitted));
            assert_eq!(
                keys.engine.committed_tail.buffer,
                if submitted { "a" } else { "" }
            );
            assert!(!keys.engine.context_word_is_known());
            assert_eq!(keys.engine.context_allows_manual_toggle(), submitted);
            assert!(keys.engine.capture_input_frame_identity().is_none());
        }
    });
}

#[test]
fn residual_atomic_unsubmitted_input_revokes_an_already_observed_prefix() {
    zbus::block_on(async {
        for receipt in [5, 1, 4] {
            for leading_boundary in [false, true] {
                let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
                let mut engine = if leading_boundary {
                    known_engine(&mut harness).await
                } else {
                    let mut engine = new_engine(&harness);
                    start_source_free_unknown(&mut harness, &mut engine).await;
                    engine
                };
                engine.config.auto_replace = false;
                engine.client_context.content_purpose = 10;
                engine.client_context.cursor_cell_width = 11;
                let mut keys = AtomicDriver::new(&mut harness, &mut engine, 9_300);
                assert_literal_commit(&keys.press('a' as u32, 30, 0).await, "a");
                assert_literal_commit(&keys.press('b' as u32, 48, 0).await, "b");
                assert!(keys.engine.committed_tail.buffer.ends_with('a'));
                assert!(keys.engine.context_allows_manual_toggle());
                // 5: uncertain submission; 1: native key delivery is possible;
                // 4: focus lineage terminated. None proves an unchanged tail.
                keys.prior.0 = receipt;
                let modifier = keys.press(KEY_LEFT_SHIFT, 42, 0).await;
                assert_eq!(modifier.0, PROPOSAL_NATIVE_UNHANDLED);
                assert!(modifier.1.is_empty());
                let scope = keys.engine.context_word_scope.as_ref().unwrap();
                assert_eq!(
                    scope.lineage().observed_suffix_chars,
                    0,
                    "receipt {receipt}"
                );
                assert!(!keys.engine.context_word_is_known(), "receipt {receipt}");
                assert!(
                    !keys.engine.context_allows_manual_toggle(),
                    "receipt {receipt}"
                );
                assert!(keys.engine.capture_input_frame_identity().is_none());
            }
        }
    });
}

#[test]
fn residual_first_numeric_word_bridge_refuses_without_delegation_or_output() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.config.auto_replace = false;
        engine.client_context.content_purpose = 10;
        engine.client_context.cursor_cell_width = 11;
        for (index, code) in [2, 3, 4].into_iter().enumerate() {
            assert!(legacy_key(&mut harness, &mut engine, 9_400 + index as u32, 0, code, 0).await);
            expect_legacy_commit(&mut harness.peer).await;
        }
        assert_eq!(engine.committed_tail.buffer, "123");
        assert!(engine.context_allows_manual_toggle());
        assert!(!engine.context_word_is_known());
        let bridge = bridge(&harness, &engine);
        harness
            .connection
            .object_server()
            .at(TARGET_PATH, engine)
            .await
            .unwrap();
        let (result, ()) = bounded(future::zip(bridge.manual_toggle_v3_inner(), async {
            serve_ping_and_marker(&mut harness.peer).await;
            assert!(harness.observer.process_next().await.unwrap());
        }))
        .await;
        assert_eq!(
            result.unwrap(),
            (0, false),
            "no daemon route for an unmappable suffix"
        );
        // A proof-only transport marker makes absence of output observable
        // without a sleep or an empty-queue timing assumption.
        harness
            .connection
            .emit_signal(None::<&str>, TARGET_PATH, "org.lay.Proof", "Reached", &())
            .await
            .unwrap();
        let next = bounded(next_peer_message(&mut harness.peer)).await;
        assert_eq!(next.header().member().unwrap().as_str(), "Reached");
        let interface = harness
            .connection
            .object_server()
            .interface::<_, LayIbusEngine>(TARGET_PATH)
            .await
            .unwrap();
        assert_eq!(interface.get().await.committed_tail.buffer, "123");
    });
}

async fn native_refocus_case(next_context: &str) {
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = known_engine(&mut harness).await;
    let previous = engine.live_context_token().unwrap();
    leave_context(&mut harness, &mut engine).await;
    let focus_in = receive(&mut harness, 3_032, "FocusInId").await;
    bounded(engine.focus_in_id(
        focus_in.header(),
        next_context.to_string(),
        "test-client".to_string(),
    ))
    .await;
    // The production callback reloads the host config. Keep this controlled
    // fixture independent of the remote account's default text backend.
    engine.config = ime_config();
    assert!(
        harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .request
            .is_some(),
        "real re-focus must start acquisition, not enrich old owner"
    );
    forward_marker_bounded(&mut harness.peer).await;
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    assert!(!legacy_key(&mut harness, &mut engine, 3_033, KEY_LEFT_SHIFT, 42, 0).await);
    assert!(
        !legacy_key(
            &mut harness,
            &mut engine,
            3_035,
            KEY_LEFT_SHIFT,
            42,
            RELEASE_MASK
        )
        .await
    );
    let current = engine
        .live_context_token()
        .expect("native re-focus installs an owner");
    assert_ne!(current.owner.generation, previous.owner.generation);
    assert_ne!(
        current.activation.generation,
        previous.activation.generation
    );
    assert_eq!(current.activation.context.path.as_str(), next_context);
    assert_eq!(engine.context_word_is_known(), next_context == CONTEXT_PATH);
    assert_eq!(
        engine.committed_tail.buffer,
        if next_context == CONTEXT_PATH {
            " "
        } else {
            ""
        }
    );
    assert!(harness
        .adapter
        .shared
        .reducer
        .lock()
        .unwrap()
        .request
        .is_none());
    assert!(harness
        .adapter
        .shared
        .ready_activation
        .lock()
        .unwrap()
        .is_none());
    assert!(bridge_fence(&mut harness).await.is_ok());
    if next_context != CONTEXT_PATH {
        assert!(legacy_key(&mut harness, &mut engine, 3_034, KEY_SPACE, 57, 0).await);
        expect_legacy_commit(&mut harness.peer).await;
        assert!(
            engine.context_word_is_known(),
            "next actual boundary rearms new context"
        );
    }
}

#[test]
fn residual_native_refocus_same_context_rotates_activation() {
    zbus::block_on(bounded(native_refocus_case(CONTEXT_PATH)));
}

#[test]
fn residual_native_refocus_other_context_rearms_source_free() {
    zbus::block_on(bounded(native_refocus_case(
        "/org/freedesktop/IBus/InputContext_2",
    )));
}

async fn ready_transfer(harness: &mut Harness) -> LayIbusEngine {
    let mut engine = known_engine(harness).await;
    leave_context(harness, &mut engine).await;
    let focus_in = receive(harness, 3_040, "FocusInId").await;
    let observed = harness
        .adapter
        .observe_callback(&focus_in.header(), Instant::now())
        .await
        .unwrap();
    // Use the real acquisition independently of the separate re-focus bug.
    harness
        .adapter
        .start_native_activation(
            engine_path(TARGET_PATH),
            context(CONTEXT_PATH),
            observed.position,
        )
        .unwrap();
    forward_marker_bounded(&mut harness.peer).await;
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    assert!(matches!(
        harness
            .adapter
            .shared
            .ready_activation
            .lock()
            .unwrap()
            .as_ref()
            .map(|r| &r.outcome),
        Some(ActivationOutcome::Transfer(_))
    ));
    engine
}

async fn revoke_ready(harness: &mut Harness, owner: &EngineOwner, content_type: bool) -> Message {
    let message = if content_type {
        Message::method_call(TARGET_PATH, "Set")
            .unwrap()
            .interface(PROPERTIES_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(3_041).unwrap())
            .build(&(
                ENGINE_INTERFACE,
                "ContentType",
                // This is a real hints change; repeating (0,0) is idempotent.
                zbus::zvariant::Value::from((0u32, 1u32)),
            ))
            .unwrap()
    } else {
        method_message(
            DISPATCH_SENDER,
            3_041,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "Reset",
        )
    };
    harness.peer.connection.send(&message).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    assert_eq!(
        harness.adapter.current_owner().as_ref(),
        Some(owner),
        "revocation retains owner"
    );
    assert_eq!(
        harness
            .adapter
            .current_token()
            .unwrap()
            .lineage
            .completeness,
        WordCompleteness::UnknownStart
    );
    message
}

async fn global_engine_changed(harness: &mut Harness, serial: u32, profile: &str) {
    let signal = Message::signal(IBUS_PATH, IBUS_INTERFACE, "GlobalEngineChanged")
        .unwrap()
        .sender(IBUS_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .build(&profile)
        .unwrap();
    harness.peer.connection.send(&signal).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
}

#[test]
fn residual_reset_before_ready_install_does_not_leak_ownerless_key_settlements() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;

        let published_token = harness
            .adapter
            .current_token()
            .expect("ready source-free grant has a reducer owner");
        assert!(engine.context_owner.is_none());
        assert!(engine.live_context_token().is_none());
        assert!(harness
            .adapter
            .shared
            .ready_activation
            .lock()
            .unwrap()
            .is_some());
        engine.committed_tail.buffer = "stale-before-ready-install".to_string();

        let mut resets = Vec::new();
        for serial in 3_400..3_403 {
            let reset = method_message(
                DISPATCH_SENDER,
                serial,
                TARGET_PATH,
                ENGINE_INTERFACE,
                "Reset",
            );
            harness.peer.connection.send(&reset).await.unwrap();
            assert!(bounded(harness.observer.process_next()).await.unwrap());
            resets.push(reset);
        }
        for reset in &resets {
            bounded(
                engine.reset(
                    reset.header(),
                    zbus::object_server::SignalEmitter::new(
                        &harness.connection,
                        engine.path.clone(),
                    )
                    .unwrap(),
                ),
            )
            .await
            .unwrap();
        }
        assert!(!harness.adapter.revalidate(&published_token));
        assert!(engine.context_owner.is_none());
        assert!(!engine.context_word_is_known());
        let reset_outcome = harness
            .adapter
            .pending_activation_for(&engine_path(TARGET_PATH))
            .expect("current uninstalled reset witness");
        assert!(matches!(&reset_outcome, ActivationOutcome::ResetUnknown(_)));

        // Every callback runs to completion through the production engine and
        // observer paths. None may leak an ownerless ingress entry into the
        // reducer's bounded unsettled queue or cancel future observation.
        for index in 0..(MAX_UNSETTLED_KEYS + 2) {
            assert!(
                !legacy_key(
                    &mut harness,
                    &mut engine,
                    3_410 + index as u32,
                    KEY_LEFT_SHIFT,
                    42,
                    if index % 2 == 0 { 0 } else { RELEASE_MASK },
                )
                .await
            );
            if index == 0 {
                assert!(engine.committed_tail.buffer.is_empty());
                assert!(engine.live_context_token().is_some());
                assert!(!engine.context_word_is_known());
                assert!(harness
                    .adapter
                    .pending_activation_for(&engine_path(TARGET_PATH))
                    .is_none());
                assert!(harness
                    .adapter
                    .activation_outcome_is_current(&reset_outcome));
                assert!(
                    !harness.adapter.acknowledge_activation(&reset_outcome),
                    "a still-current token cannot acknowledge an absent/already installed witness"
                );
            }
        }
        assert!(harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .unsettled
            .is_empty());

        let recovered = engine
            .live_context_token()
            .expect("the first real key installs the reset witness");
        assert_eq!(recovered.owner, published_token.owner);
        assert_eq!(recovered.activation, published_token.activation);
        assert_ne!(recovered.revocation, published_token.revocation);
        assert_eq!(
            recovered.lineage.completeness,
            WordCompleteness::UnknownStart
        );
        assert!(engine.committed_tail.buffer.is_empty());
        assert!(!engine.context_word_is_known());
        assert!(!harness.adapter.revalidate(&published_token));
        assert!(bridge_fence(&mut harness).await.is_ok());

        assert!(legacy_key(&mut harness, &mut engine, 3_500, KEY_SPACE, 57, 0).await);
        expect_legacy_commit(&mut harness.peer).await;
        assert!(engine.context_word_is_known());
        assert!(bridge_fence(&mut harness).await.is_ok());
    }));
}

#[test]
fn residual_established_owner_reset_burst_rearms_on_first_real_boundary() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = known_engine(&mut harness).await;
        let old_token = engine.live_context_token().unwrap();

        let mut resets = Vec::new();
        for serial in 3_600..3_603 {
            let reset = method_message(
                DISPATCH_SENDER,
                serial,
                TARGET_PATH,
                ENGINE_INTERFACE,
                "Reset",
            );
            harness.peer.connection.send(&reset).await.unwrap();
            assert!(bounded(harness.observer.process_next()).await.unwrap());
            resets.push(reset);
        }
        for reset in &resets {
            bounded(
                engine.reset(
                    reset.header(),
                    zbus::object_server::SignalEmitter::new(
                        &harness.connection,
                        engine.path.clone(),
                    )
                    .unwrap(),
                ),
            )
            .await
            .unwrap();
        }

        assert!(!harness.adapter.revalidate(&old_token));
        assert!(!engine.context_word_is_known());
        assert!(harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .unsettled
            .is_empty());
        assert!(legacy_key(&mut harness, &mut engine, 3_603, KEY_SPACE, 57, 0).await);
        expect_legacy_commit(&mut harness.peer).await;
        assert!(engine.context_word_is_known());
        assert!(bridge_fence(&mut harness).await.is_ok());
    }));
}

#[test]
fn residual_ready_transfer_rejects_same_owner_revocation_before_take() {
    zbus::block_on(bounded(async {
        for content_type in [false, true] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let _source = ready_transfer(&mut harness).await;
            let old_outcome = harness
                .adapter
                .pending_activation_for(&engine_path(TARGET_PATH))
                .expect("ready transfer witness");
            assert!(matches!(&old_outcome, ActivationOutcome::Transfer(_)));
            let owner = harness.adapter.current_owner().unwrap();
            revoke_ready(&mut harness, &owner, content_type).await;
            assert!(!harness.adapter.activation_outcome_is_current(&old_outcome));
            assert!(matches!(
                harness
                    .adapter
                    .pending_activation_for(&engine_path(TARGET_PATH)),
                Some(ActivationOutcome::ResetUnknown(_))
            ));
        }
    }));
}

#[test]
fn residual_taken_transfer_rejects_same_owner_revocation_before_install() {
    zbus::block_on(bounded(async {
        for content_type in [false, true] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut source = ready_transfer(&mut harness).await;
            let outcome = harness
                .adapter
                .pending_activation_for(&engine_path(TARGET_PATH))
                .expect("runtime peeks the ready transfer before installation");
            assert!(matches!(&outcome, ActivationOutcome::Transfer(_)));
            let owner = harness.adapter.current_owner().unwrap();
            assert!(harness.adapter.activation_outcome_is_current(&outcome));
            revoke_ready(&mut harness, &owner, content_type).await;
            assert!(
                !source.install_context_activation(outcome),
                "revoked KnownStart must never install"
            );
            assert!(matches!(
                harness
                    .adapter
                    .pending_activation_for(&engine_path(TARGET_PATH)),
                Some(ActivationOutcome::ResetUnknown(_))
            ));
            assert!(
                !legacy_key(
                    &mut harness,
                    &mut source,
                    3_610 + if content_type { 1 } else { 0 },
                    KEY_LEFT_SHIFT,
                    42,
                    0,
                )
                .await
            );
            assert!(source.live_context_token().is_some());
            assert!(!source.context_word_is_known());
            assert!(source.committed_tail.buffer.is_empty());
            assert!(source.committed_tail.pending_completion_learning.is_none());
            assert!(harness
                .adapter
                .pending_activation_for(&engine_path(TARGET_PATH))
                .is_none());
        }
    }));
}

#[test]
fn residual_reset_unknown_witness_is_not_revived_by_foreign_or_focus_out() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let _source = ready_transfer(&mut harness).await;
        let owner = harness.adapter.current_owner().unwrap();
        global_engine_changed(&mut harness, 3_620, "foreign-ime").await;
        assert!(harness
            .adapter
            .pending_activation_for(&engine_path(TARGET_PATH))
            .is_none());
        revoke_ready(&mut harness, &owner, false).await;
        global_engine_changed(&mut harness, 3_621, "lay-us").await;
        assert!(
            harness
                .adapter
                .pending_activation_for(&engine_path(TARGET_PATH))
                .is_none(),
            "Foreign -> Reset -> Lay cannot recreate an invalidated witness"
        );

        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let _source = ready_transfer(&mut harness).await;
        let owner = harness.adapter.current_owner().unwrap();
        let focus_out = method_message(
            DISPATCH_SENDER,
            3_622,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "FocusOut",
        );
        harness.peer.connection.send(&focus_out).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        assert!(matches!(
            harness
                .adapter
                .pending_activation_for(&engine_path(TARGET_PATH)),
            Some(ActivationOutcome::Transfer(_))
        ));
        revoke_ready(&mut harness, &owner, false).await;
        assert!(
            harness
                .adapter
                .pending_activation_for(&engine_path(TARGET_PATH))
                .is_none(),
            "Reset after FocusOut cannot convert an unsettled transfer into ResetUnknown"
        );
    }));
}

#[test]
fn residual_legacy_enter_backspace_revokes_beyond_the_observed_mirror() {
    zbus::block_on(bounded(async {
        for enter in [KEY_ENTER, KEY_KP_ENTER] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = new_engine(&harness);
            start_source_free_unknown(&mut harness, &mut engine).await;
            assert!(legacy_key(&mut harness, &mut engine, 3_050, u32::from(b'l'), 38, 0).await);
            expect_legacy_commit(&mut harness.peer).await;
            assert!(!engine.context_word_is_known());
            assert!(!legacy_key(&mut harness, &mut engine, 3_051, enter, 28, 0).await);
            assert!(engine.committed_tail.buffer.is_empty());
            assert!(
                engine.context_word_is_known(),
                "Enter is an observed boundary"
            );
            assert!(!legacy_key(&mut harness, &mut engine, 3_052, KEY_BACKSPACE, 14, 0).await);
            assert!(
                !engine.context_word_is_known(),
                "Backspace rejoins the unobserved prefix"
            );
            assert!(!legacy_key(&mut harness, &mut engine, 3_053, KEY_TAB, 15, 0).await);
            let mut effects = AtomicEffectBuilder::default();
            let manual = engine
                .manual_toggle_active_text_target(&mut EngineOutput::atomic(&mut effects))
                .await
                .unwrap();
            assert_eq!(manual, None);
            assert_eq!(
                effects.finish(false),
                (PROPOSAL_NATIVE_UNHANDLED, Vec::new())
            );
            assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
            assert!(legacy_key(&mut harness, &mut engine, 3_054, KEY_SPACE, 57, 0).await);
            expect_legacy_commit(&mut harness.peer).await;
            assert!(
                engine.context_word_is_known(),
                "a new actual boundary restores authority"
            );
        }
    }));
}

async fn retained_boundary_literal_keys(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    serial: &mut u32,
    text: &str,
) {
    for ch in text.chars() {
        let keycode = match ch {
            'a' => 30,
            'b' => 48,
            'c' => 46,
            ' ' => 57,
            _ => unreachable!("fixture key"),
        };
        assert!(legacy_key(harness, engine, *serial, ch as u32, keycode, 0).await);
        *serial += 1;
        let commit = bounded(next_peer_message(&mut harness.peer)).await;
        assert_eq!(commit.header().member().unwrap().as_str(), "CommitText");
        let body = commit.body();
        let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
        assert_eq!(
            crate::ibus_interface::ibus_text_value_to_string(&value),
            Some(ch.to_string())
        );
        assert!(legacy_key(harness, engine, *serial, ch as u32, keycode, RELEASE_MASK).await);
        *serial += 1;
    }
}

async fn retained_boundary_backspace(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    serial: &mut u32,
) {
    assert!(!legacy_key(harness, engine, *serial, KEY_BACKSPACE, 14, 0).await);
    *serial += 1;
    let _ = legacy_key(harness, engine, *serial, KEY_BACKSPACE, 14, RELEASE_MASK).await;
    *serial += 1;
}

#[test]
fn residual_observed_boundary_backspace_keeps_start_and_retires_old_token() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = known_engine(&mut harness).await;
        engine.config.auto_replace = false;
        let mut serial = 4_000;
        retained_boundary_literal_keys(&mut harness, &mut engine, &mut serial, "ab ").await;
        assert_eq!(engine.committed_tail.buffer, " ab ");
        let closed = engine.live_context_token().unwrap();
        retained_boundary_backspace(&mut harness, &mut engine, &mut serial).await;
        assert_eq!(engine.committed_tail.buffer, " ab");
        assert!(engine.context_word_is_known());
        assert!(engine.capture_input_frame_identity().is_some());
        assert!(!harness.adapter.revalidate(&closed));
        for expected in [" a", " "] {
            retained_boundary_backspace(&mut harness, &mut engine, &mut serial).await;
            assert_eq!(engine.committed_tail.buffer, expected);
            assert!(engine.context_word_is_known());
        }
        retained_boundary_literal_keys(&mut harness, &mut engine, &mut serial, "c").await;
        assert_eq!(engine.committed_tail.buffer, " c");
        assert!(engine.capture_input_frame_identity().is_some());
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    }));
}

#[test]
fn residual_observed_boundary_backspace_refuses_an_unobserved_mirror_separator() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        engine.config.auto_replace = false;
        start_source_free_unknown(&mut harness, &mut engine).await;
        assert!(!legacy_key(&mut harness, &mut engine, 4_098, KEY_LEFT_SHIFT, 42, 0).await);
        assert!(
            !legacy_key(
                &mut harness,
                &mut engine,
                4_099,
                KEY_LEFT_SHIFT,
                42,
                RELEASE_MASK,
            )
            .await
        );
        assert!(engine.live_context_token().is_some());
        assert!(!engine.context_word_is_known());
        engine.committed_tail.buffer = "hidden prefix".into();
        assert!(!engine.context_word_is_known());
        let mut serial = 4_100;
        retained_boundary_literal_keys(&mut harness, &mut engine, &mut serial, " ").await;
        assert_eq!(engine.committed_tail.buffer, "hidden prefix ");
        assert_eq!(
            engine
                .context_word_scope
                .unwrap()
                .lineage()
                .observed_boundary_floor,
            Some(13)
        );
        assert!(engine.context_word_is_known());
        retained_boundary_backspace(&mut harness, &mut engine, &mut serial).await;
        assert_eq!(engine.committed_tail.buffer, "hidden prefix");
        assert!(!engine.context_word_is_known());
        assert!(engine.capture_input_frame_identity().is_none());
        assert_eq!(
            engine
                .context_word_scope
                .unwrap()
                .lineage()
                .observed_boundary_floor,
            None
        );
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    }));
}

#[test]
fn residual_observed_boundary_backspace_cannot_cross_a_command_input_gap() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = known_engine(&mut harness).await;
        engine.config.auto_replace = false;
        let mut serial = 4_200;
        retained_boundary_literal_keys(&mut harness, &mut engine, &mut serial, "ab ").await;
        assert!(!legacy_key(&mut harness, &mut engine, serial, b'a' as u32, 30, 1 << 2).await);
        serial += 1;
        assert!(!engine.context_word_is_known());
        assert_eq!(
            engine
                .context_word_scope
                .unwrap()
                .lineage()
                .observed_boundary_floor,
            None
        );
        // A retained mirror after an external command is not fresh evidence.
        engine.committed_tail.buffer = " ab ".into();
        retained_boundary_backspace(&mut harness, &mut engine, &mut serial).await;
        assert_eq!(engine.committed_tail.buffer, " ab");
        assert!(!engine.context_word_is_known());
        assert!(engine.capture_input_frame_identity().is_none());
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    }));
}
