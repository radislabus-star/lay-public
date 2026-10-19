//! A different native field must retire only its revoked acquisition fence.
use super::*;

const RESTART_BUDGET: Duration = Duration::from_secs(1);
const OTHER_CONTEXT: &str = "/org/freedesktop/IBus/InputContext_2";

async fn bounded<T>(work: impl std::future::Future<Output = T>) -> T {
    future::race(Box::pin(work), async {
        async_io::Timer::after(RESTART_BUDGET).await;
        panic!("bounded native-focus restart choreography timed out")
    })
    .await
}

async fn observed_focus(harness: &mut Harness, serial: u32) -> Message {
    let message = method_message(
        DISPATCH_SENDER,
        serial,
        TARGET_PATH,
        ENGINE_INTERFACE,
        "FocusInId",
    );
    send_manually_dispatched_callback(&mut harness.peer, &message, TARGET_PATH, ENGINE_INTERFACE)
        .await;
    assert!(harness.observer.process_next().await.unwrap());
    message
}

async fn pending_native(harness: &mut Harness, reply_received: bool) -> PendingFence {
    let focus = observed_focus(harness, 159_800).await;
    let observed = harness
        .adapter
        .observe_callback(&focus.header(), Instant::now())
        .await
        .unwrap();
    let nonce = BarrierNonce(159_801);
    let request = harness
        .adapter
        .begin_activation_request(
            engine_path(TARGET_PATH),
            nonce,
            ReceiptOrigin::Native,
            observed.position,
            Some(&context(CONTEXT_PATH)),
        )
        .unwrap();
    if reply_received {
        assert!(harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .context_reply(request, nonce, context(CONTEXT_PATH), observed.position,));
    }
    let deadline = Instant::now() + RESTART_BUDGET;
    harness
        .adapter
        .arm_fence(PendingFenceState {
            nonce,
            deadline,
            kind: FenceKind::Acquisition {
                request,
                target_path: engine_path(TARGET_PATH),
            },
            ready_token: None,
            marker_observed: false,
            ready: false,
        })
        .unwrap();
    PendingFence { nonce, deadline }
}

async fn different_field_can_restart(reply_received: bool) {
    let mut harness = bootstrap_harness_with_budget(RESTART_BUDGET).await;
    let old_fence = pending_native(&mut harness, reply_received).await;
    let shared = Arc::new(Mutex::new(SharedState::default()));
    shared.lock().unwrap().handoff_tail_buffer = "previous field".to_string();
    let mut engine = LayIbusEngine::new_from_component(
        TARGET_PATH.to_string(),
        shared.clone(),
        Some(harness.adapter.clone()),
        "lay-ime-us",
        true,
        ime_config(),
    );
    let focus = observed_focus(&mut harness, 159_802).await;
    assert!(
        engine
            .activate_context_from_header(&focus.header(), Instant::now(), Some(OTHER_CONTEXT))
            .await,
        "different native context must start after revoking its predecessor"
    );
    assert!(harness.adapter.current_owner().is_none());
    forward_marker(&mut harness.peer).await;
    let current_nonce = harness
        .adapter
        .shared
        .pending
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .nonce;
    assert_ne!(current_nonce, old_fence.nonce);
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
        current_nonce,
        "old expiry must not clear the new field's fence"
    );
    assert!(harness.observer.process_next().await.unwrap());
    engine.try_install_pending_context_activation();
    let token = engine
        .context_token
        .as_ref()
        .expect("new field admitted")
        .clone();
    assert!(harness.adapter.revalidate(&token));
    assert_eq!(token.activation.context, context(OTHER_CONTEXT));
    assert_eq!(token.lineage.completeness, WordCompleteness::UnknownStart);
    assert_eq!(token.lineage.observed_suffix_chars, 0);
    assert!(!engine.context_word_is_known());
    assert!(engine.committed_tail.buffer.is_empty());
    assert!(shared.lock().unwrap().handoff_tail_buffer.is_empty());

    let stale_marker = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
        .unwrap()
        .sender(ADAPTER_SENDER)
        .unwrap()
        .build(&old_fence.nonce.0)
        .unwrap();
    harness.peer.connection.send(&stale_marker).await.unwrap();
    assert!(harness.observer.process_next().await.unwrap());
    harness.adapter.expire_fence(old_fence);
    assert!(harness.adapter.revalidate(&token));
    assert_eq!(engine.context_owner, harness.adapter.current_owner());
}

#[test]
fn td159_native_focus_restart_before_context_reply() {
    zbus::block_on(bounded(different_field_can_restart(false)));
}

#[test]
fn td159_native_focus_restart_after_context_reply_before_marker() {
    zbus::block_on(bounded(different_field_can_restart(true)));
}

#[test]
fn td159_native_focus_restart_preserves_current_acquisition_and_bridge() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(RESTART_BUDGET).await;
        let fence = pending_native(&mut harness, true).await;
        let before = harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .revocation_generation();
        let generation = harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .request
            .as_ref()
            .unwrap()
            .generation;
        let duplicate = observed_focus(&mut harness, 159_803).await;
        let observed = harness
            .adapter
            .observe_callback(&duplicate.header(), Instant::now())
            .await
            .unwrap();
        assert!(harness.adapter.enrich_pending_native_activation(
            &engine_path(TARGET_PATH),
            context(CONTEXT_PATH),
            &observed,
        ));
        assert!(matches!(
            harness.adapter.begin_activation_request(
                engine_path(TARGET_PATH),
                BarrierNonce(159_804),
                ReceiptOrigin::Native,
                observed.position,
                Some(&context(CONTEXT_PATH)),
            ),
            Err(AdapterError::Busy)
        ));
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
            fence.nonce
        );
        {
            let reducer = harness.adapter.shared.reducer.lock().unwrap();
            assert_eq!(reducer.revocation_generation(), before);
            assert_eq!(reducer.request.as_ref().unwrap().generation, generation);
        }
        harness.adapter.expire_fence(fence);
        let bridge_nonce = BarrierNonce(159_805);
        harness
            .adapter
            .arm_fence(PendingFenceState {
                nonce: bridge_nonce,
                deadline: Instant::now() + RESTART_BUDGET,
                kind: FenceKind::Bridge {
                    ping_position: Default::default(),
                },
                ready_token: None,
                marker_observed: false,
                ready: false,
            })
            .unwrap();
        assert!(matches!(
            harness.adapter.begin_activation_request(
                engine_path(TARGET_PATH),
                BarrierNonce(159_806),
                ReceiptOrigin::Native,
                observed.position,
                Some(&context(OTHER_CONTEXT)),
            ),
            Err(AdapterError::Busy)
        ));
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
            bridge_nonce
        );
        assert!(harness.adapter.current_owner().is_none());
        harness.adapter.clear_fence(bridge_nonce);
    }));
}
