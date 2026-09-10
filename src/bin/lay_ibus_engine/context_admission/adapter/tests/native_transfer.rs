use super::*;
use crate::protocol::KEY_SPACE;

const NATIVE_TRANSFER_BUDGET: Duration = Duration::from_secs(1);
const OTHER_CONTEXT_PATH: &str = "/org/freedesktop/IBus/InputContext_2";

async fn bounded<T>(work: impl std::future::Future<Output = T>) -> T {
    future::race(work, async {
        async_io::Timer::after(NATIVE_TRANSFER_BUDGET).await;
        panic!("bounded native-transfer P2P choreography timed out")
    })
    .await
}

async fn establish_known_source(
    harness: &mut Harness,
    shared: Arc<Mutex<SharedState>>,
) -> LayIbusEngine {
    let mut source = LayIbusEngine::new_from_component(
        SOURCE_PATH.to_string(),
        shared.clone(),
        Some(harness.adapter.clone()),
        "lay-ime-us",
        true,
        ime_config(),
    );
    let grant = complete_native_activation(harness).await;
    assert!(source.install_context_activation(ActivationOutcome::SourceFree(grant)));

    let space = method_message(
        DISPATCH_SENDER,
        7_200,
        SOURCE_PATH,
        ENGINE_INTERFACE,
        "ProcessKeyEvent",
    );
    harness.peer.connection.send(&space).await.unwrap();
    let emitter = zbus::object_server::SignalEmitter::new(&harness.connection, SOURCE_PATH)
        .expect("source signal emitter");
    let (handled, observed) = bounded(future::zip(
        source.process_key_event(space.header(), emitter, KEY_SPACE, 57, 0),
        harness.observer.process_next(),
    ))
    .await;
    assert!(observed.unwrap());
    assert!(handled.unwrap());
    let commit = bounded(next_peer_message(&mut harness.peer)).await;
    assert_eq!(commit.header().member().unwrap().as_str(), "CommitText");
    assert!(source.context_word_is_known());

    source.committed_tail.buffer = "l".to_string();
    source.committed_tail.pending_completion_learning = None;
    source.committed_tail.autocorrect_suppression = None;
    {
        let mut state = shared.lock().unwrap();
        state.handoff_tail_buffer = "l".to_string();
        state.handoff_tail_epoch = source.committed_tail.epoch;
        state.autocorrect_suppression = None;
        state.exact_manual_toggle_handoff_epoch = None;
        state.exact_manual_toggle_handoff_path = None;
    }
    source
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

fn focus_out_id_message(serial: u32) -> Message {
    Message::method_call(SOURCE_PATH, "FocusOutId")
        .unwrap()
        .interface(ENGINE_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .build(&CONTEXT_PATH)
        .unwrap()
}

async fn forward_exact_marker(peer: &mut ControlledPeer, expected_nonce: BarrierNonce) {
    let marker = bounded(next_peer_message(peer)).await;
    assert_eq!(marker.header().message_type(), Type::Signal);
    assert_eq!(
        marker.header().interface().unwrap().as_str(),
        MARKER_INTERFACE
    );
    assert_eq!(marker.header().member().unwrap().as_str(), MARKER_MEMBER);
    assert_eq!(
        marker.body().deserialize::<u64>().unwrap(),
        expected_nonce.0
    );
    let forwarded = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
        .unwrap()
        .sender(ADAPTER_SENDER)
        .unwrap()
        .build(&expected_nonce.0)
        .unwrap();
    peer.connection.send(&forwarded).await.unwrap();
}

async fn pending_transfer_late_native_case(native_context: &str) {
    let mut harness = bootstrap_harness_with_budget(NATIVE_TRANSFER_BUDGET).await;
    let shared = Arc::new(Mutex::new(SharedState::default()));
    let mut source = establish_known_source(&mut harness, shared.clone()).await;

    let factory = factory_message(7_201);
    harness.peer.connection.send(&factory).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    let factory_callback = harness
        .adapter
        .begin_factory_callback(&factory.header(), Instant::now(), profile("lay-us"))
        .await
        .unwrap();
    assert!(harness
        .adapter
        .bind_factory_target(&factory_callback, engine_path(TARGET_PATH)));

    let focus_out = focus_out_id_message(7_202);
    harness.peer.connection.send(&focus_out).await.unwrap();
    let (accepted, observed) = bounded(future::zip(
        source.observe_context_focus_out(&focus_out.header(), Instant::now()),
        harness.observer.process_next(),
    ))
    .await;
    assert!(observed.unwrap());
    assert!(accepted);

    let disable = method_message(
        DISPATCH_SENDER,
        7_203,
        SOURCE_PATH,
        ENGINE_INTERFACE,
        "Disable",
    );
    harness.peer.connection.send(&disable).await.unwrap();
    let (accepted, observed) = bounded(future::zip(
        source.observe_context_disable(&disable.header(), Instant::now()),
        harness.observer.process_next(),
    ))
    .await;
    assert!(observed.unwrap());
    assert!(accepted);

    let mut target = LayIbusEngine::new_from_component(
        TARGET_PATH.to_string(),
        shared.clone(),
        Some(harness.adapter.clone()),
        "lay-ime-us",
        true,
        ime_config(),
    );
    let focus_in = method_message(
        DISPATCH_SENDER,
        7_204,
        TARGET_PATH,
        ENGINE_INTERFACE,
        "FocusIn",
    );
    harness.peer.connection.send(&focus_in).await.unwrap();
    let (started, observed) = bounded(future::zip(
        target.activate_context_from_header(&focus_in.header(), Instant::now(), None),
        harness.observer.process_next(),
    ))
    .await;
    assert!(observed.unwrap());
    assert!(started);

    let held_get = bounded(next_peer_message(&mut harness.peer)).await;
    assert_eq!(held_get.header().member().unwrap().as_str(), "Get");
    let (request_generation, request_nonce) = {
        let reducer = harness.adapter.shared.reducer.lock().unwrap();
        let request = reducer.request.as_ref().expect("compatibility request");
        assert_eq!(request.origin, ReceiptOrigin::CompatibilityProperty);
        assert!(request.reply.is_none());
        (request.generation, request.nonce)
    };
    assert!(
        harness.adapter.shared.pending.lock().unwrap().is_none(),
        "the held Get keeps acquisition in the pre-fence window"
    );

    let focus_in_id = method_message(
        DISPATCH_SENDER,
        7_205,
        TARGET_PATH,
        ENGINE_INTERFACE,
        "FocusInId",
    );
    harness.peer.connection.send(&focus_in_id).await.unwrap();
    let ((), observed) = bounded(future::zip(
        target.focus_in_id(
            focus_in_id.header(),
            native_context.to_string(),
            "controlled-client".to_string(),
        ),
        harness.observer.process_next(),
    ))
    .await;
    assert!(observed.unwrap());
    {
        let reducer = harness.adapter.shared.reducer.lock().unwrap();
        let request = reducer.request.as_ref().expect("same enriched request");
        assert_eq!(request.generation, request_generation);
        assert_eq!(request.nonce, request_nonce);
        assert_eq!(request.origin, ReceiptOrigin::Native);
        assert_eq!(request.reply.as_ref().unwrap().0, context(native_context));
    }

    forward_exact_marker(&mut harness.peer, request_nonce).await;
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    let outcome = harness
        .adapter
        .try_finish_activation_for(&engine_path(TARGET_PATH))
        .unwrap()
        .expect("the original request publishes one target-bound outcome");
    if native_context == CONTEXT_PATH {
        assert!(matches!(&outcome, ActivationOutcome::Transfer(_)));
    } else {
        assert!(matches!(&outcome, ActivationOutcome::SourceFree(_)));
    }
    assert!(target.install_context_activation(outcome));

    assert_eq!(
        target.committed_tail.buffer,
        if native_context == CONTEXT_PATH {
            "l"
        } else {
            ""
        }
    );
    assert_eq!(
        target.context_word_is_known(),
        native_context == CONTEXT_PATH
    );
    assert!(target.committed_tail.pending_completion_learning.is_none());
    assert!(target.committed_tail.autocorrect_suppression.is_none());
    {
        let state = shared.lock().unwrap();
        assert!(state.autocorrect_suppression.is_none());
        assert!(state.exact_manual_toggle_handoff_epoch.is_none());
        assert!(state.exact_manual_toggle_handoff_path.is_none());
    }

    let enable = method_message(
        DISPATCH_SENDER,
        7_206,
        TARGET_PATH,
        ENGINE_INTERFACE,
        "Enable",
    );
    harness.peer.connection.send(&enable).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    drop(held_get);
}

#[test]
fn pending_transfer_focus_in_then_focus_in_id_enriches_one_prearm_request() {
    zbus::block_on(bounded(async {
        pending_transfer_late_native_case(CONTEXT_PATH).await;
        pending_transfer_late_native_case(OTHER_CONTEXT_PATH).await;
    }));
}

#[test]
fn superseded_source_free_cannot_publish_its_stale_fence() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(NATIVE_TRANSFER_BUDGET).await;
        let stale_nonce = BarrierNonce(7_300);
        let stale_request = harness
            .adapter
            .begin_activation_request(
                engine_path(SOURCE_PATH),
                stale_nonce,
                ReceiptOrigin::CompatibilityProperty,
                Default::default(),
            )
            .expect("old empty source-free request");
        assert!(harness.adapter.shared.pending.lock().unwrap().is_none());

        let factory = factory_message(7_301);
        harness.peer.connection.send(&factory).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        let factory_callback = harness
            .adapter
            .begin_factory_callback(&factory.header(), Instant::now(), profile("lay-us"))
            .await
            .expect("later factory supersedes the empty acquisition");
        assert!(harness
            .adapter
            .bind_factory_target(&factory_callback, engine_path(TARGET_PATH)));
        assert!(harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .request
            .is_none());

        let stale = PendingFenceState {
            nonce: stale_nonce,
            deadline: Instant::now() + NATIVE_TRANSFER_BUDGET,
            kind: FenceKind::Acquisition {
                request: stale_request,
                target_path: engine_path(SOURCE_PATH),
            },
            ready_token: None,
            marker_observed: false,
            ready: false,
        };
        assert!(matches!(
            harness.adapter.arm_fence(stale),
            Err(AdapterError::Denied)
        ));
        assert!(harness.adapter.shared.pending.lock().unwrap().is_none());

        let bridge_nonce = BarrierNonce(7_302);
        harness
            .adapter
            .arm_fence(PendingFenceState {
                nonce: bridge_nonce,
                deadline: Instant::now() + NATIVE_TRANSFER_BUDGET,
                kind: FenceKind::Bridge {
                    ping_position: Default::default(),
                },
                ready_token: None,
                marker_observed: false,
                ready: false,
            })
            .expect("bridge fencing does not require an acquisition request");
        harness.adapter.clear_fence(bridge_nonce);

        let focus_in = method_message(
            DISPATCH_SENDER,
            7_303,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "FocusInId",
        );
        harness.peer.connection.send(&focus_in).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        let observed = harness
            .adapter
            .observe_callback(&focus_in.header(), Instant::now())
            .await
            .unwrap();
        let current_nonce = BarrierNonce(7_304);
        let current_request = harness
            .adapter
            .begin_activation_request(
                engine_path(TARGET_PATH),
                current_nonce,
                ReceiptOrigin::Native,
                observed.position,
            )
            .expect("the factory-bound successor activation can start");

        let acquisition = |nonce| PendingFenceState {
            nonce,
            deadline: Instant::now() + NATIVE_TRANSFER_BUDGET,
            kind: FenceKind::Acquisition {
                request: current_request,
                target_path: engine_path(TARGET_PATH),
            },
            ready_token: None,
            marker_observed: false,
            ready: false,
        };
        assert!(matches!(
            harness
                .adapter
                .arm_fence(acquisition(BarrierNonce(current_nonce.0 + 1))),
            Err(AdapterError::Denied)
        ));
        assert!(harness.adapter.shared.pending.lock().unwrap().is_none());
        let mut wrong_target = acquisition(current_nonce);
        wrong_target.kind = FenceKind::Acquisition {
            request: current_request,
            target_path: engine_path(SOURCE_PATH),
        };
        assert!(matches!(
            harness.adapter.arm_fence(wrong_target),
            Err(AdapterError::Denied)
        ));
        assert!(harness.adapter.shared.pending.lock().unwrap().is_none());
        harness
            .adapter
            .arm_fence(acquisition(current_nonce))
            .expect("the exact live request may occupy the fence slot");
        assert!(harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .context_reply(
                current_request,
                current_nonce,
                context(CONTEXT_PATH),
                observed.position,
            ));
        let marker = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
            .unwrap()
            .sender(ADAPTER_SENDER)
            .unwrap()
            .build(&current_nonce.0)
            .unwrap();
        harness.peer.connection.send(&marker).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        let outcome = harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .expect("successor completes after stale arm rejection");
        let ActivationOutcome::SourceFree(grant) = outcome else {
            panic!("empty successor cannot inherit transfer authority")
        };
        assert_eq!(grant.lineage.completeness, WordCompleteness::UnknownStart);
        assert!(harness.adapter.shared.pending.lock().unwrap().is_none());
        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .is_none());
    }));
}
