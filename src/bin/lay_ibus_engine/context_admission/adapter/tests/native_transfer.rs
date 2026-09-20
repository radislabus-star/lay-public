use super::*;
use crate::protocol::{KEY_BACKSPACE, KEY_SPACE, KEY_TAB};

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

async fn type_active_known_word(
    harness: &mut Harness,
    source: &mut LayIbusEngine,
    shared: &Arc<Mutex<SharedState>>,
    serial: u32,
) {
    source.committed_tail.buffer.clear();
    {
        let mut state = shared.lock().unwrap();
        state.handoff_tail_buffer.clear();
        state.handoff_tail_epoch = source.committed_tail.epoch;
    }
    for (offset, (character, keycode)) in [
        ('g', 34),
        ('h', 35),
        ('b', 48),
        ('d', 32),
        ('t', 20),
        ('n', 49),
    ]
    .into_iter()
    .enumerate()
    {
        let key = method_message(
            DISPATCH_SENDER,
            serial + u32::try_from(offset).unwrap(),
            SOURCE_PATH,
            ENGINE_INTERFACE,
            "ProcessKeyEvent",
        );
        send_observed(harness, &key).await;
        let emitter =
            zbus::object_server::SignalEmitter::new(&harness.connection, SOURCE_PATH).unwrap();
        let handled = source
            .process_key_event(key.header(), emitter, u32::from(character), keycode, 0)
            .await
            .unwrap();
        let expected = character.to_string();
        assert_exact_literal_delivery(
            &harness.connection,
            &mut harness.peer,
            SOURCE_PATH,
            handled,
            &expected,
        )
        .await;
    }
}

fn factory_message(serial: u32) -> Message {
    factory_message_for_profile(serial, "lay-us")
}

fn factory_message_for_profile(serial: u32, requested_profile: &str) -> Message {
    Message::method_call("/org/freedesktop/IBus/Factory", "CreateEngine")
        .unwrap()
        .interface(FACTORY_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .build(&requested_profile)
        .unwrap()
}

async fn send_global_engine_changed(harness: &mut Harness, serial: u32, profile_name: &str) {
    let signal = Message::signal(IBUS_PATH, IBUS_INTERFACE, "GlobalEngineChanged")
        .unwrap()
        .sender(IBUS_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .build(&profile_name)
        .unwrap();
    send_observed(harness, &signal).await;
}

fn focus_out_id_message(serial: u32) -> Message {
    focus_out_id_message_for(SOURCE_PATH, serial)
}

fn focus_out_id_message_for(path: &str, serial: u32) -> Message {
    Message::method_call(path, "FocusOutId")
        .unwrap()
        .interface(ENGINE_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .build(&CONTEXT_PATH)
        .unwrap()
}

async fn send_observed(harness: &mut Harness, message: &Message) {
    harness.peer.connection.send(message).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
}

async fn open_factory_for(harness: &mut Harness, target_path: &str, serial: u32) {
    let factory = factory_message(serial);
    send_observed(harness, &factory).await;
    let callback = harness
        .adapter
        .begin_factory_callback(&factory.header(), Instant::now(), profile("lay-us"))
        .await
        .unwrap();
    assert!(harness
        .adapter
        .bind_factory_target(&callback, engine_path(target_path)));
}

async fn start_native_focus(
    harness: &mut Harness,
    target: &mut LayIbusEngine,
    target_path: &str,
    context_path: &str,
    serial: u32,
) {
    let nonce =
        start_native_focus_before_marker(harness, target, target_path, context_path, serial).await;
    forward_exact_marker(&mut harness.peer, nonce).await;
    assert!(bounded(harness.observer.process_next()).await.unwrap());
}

async fn start_native_focus_before_marker(
    harness: &mut Harness,
    target: &mut LayIbusEngine,
    target_path: &str,
    context_path: &str,
    serial: u32,
) -> BarrierNonce {
    let focus = Message::method_call(target_path, "FocusInId")
        .unwrap()
        .interface(ENGINE_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .build(&())
        .unwrap();
    send_observed(harness, &focus).await;
    target
        .focus_in_id(
            focus.header(),
            context_path.to_string(),
            "controlled-client".to_string(),
        )
        .await;
    harness
        .adapter
        .shared
        .reducer
        .lock()
        .unwrap()
        .request
        .as_ref()
        .expect("native focus starts the target acquisition")
        .nonce
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
fn td125_focus_out_refuses_transfer_of_cancelled_owned_preedit() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(NATIVE_TRANSFER_BUDGET).await;
        let shared = Arc::new(Mutex::new(SharedState::default()));
        let mut source = establish_known_source(&mut harness, shared.clone()).await;

        source.committed_tail.buffer = "l ab".to_string();
        source.composition.buffer = "ab".to_string();
        source.composition.cursor = 2;
        source.composition.legacy_word_preedit_active = true;
        source.composition.preedit_visible = true;
        source.rebuild_preedit_fast_from_tail();
        assert!(source.publish_tail_handoff());
        assert_eq!(shared.lock().unwrap().handoff_tail_buffer, "l ab");

        let factory = factory_message(7_207);
        send_observed(&mut harness, &factory).await;
        let factory_callback = harness
            .adapter
            .begin_factory_callback(&factory.header(), Instant::now(), profile("lay-us"))
            .await
            .unwrap();
        assert!(harness
            .adapter
            .bind_factory_target(&factory_callback, engine_path(TARGET_PATH)));

        let focus_out = focus_out_id_message(7_208);
        harness.peer.connection.send(&focus_out).await.unwrap();
        let (accepted, observed) = bounded(future::zip(
            source.observe_context_focus_out(&focus_out.header(), Instant::now()),
            harness.observer.process_next(),
        ))
        .await;
        assert!(observed.unwrap());
        assert!(accepted);

        assert!(!source.context_handoff_sealed);
        assert!(!source.context_word_is_known());
        assert!(source.composition.buffer.is_empty());
        assert!(!source.composition.legacy_word_preedit_active);
        assert_eq!(source.committed_tail.buffer, "l ");
        {
            let state = shared.lock().unwrap();
            assert_eq!(state.handoff_tail_buffer, "l ");
            assert_eq!(state.handoff_tail_epoch, source.committed_tail.epoch);
        }

        crate::window_interaction::WindowInteraction::finish_focus_out(&mut source);
        assert!(source.committed_tail.buffer.is_empty());
        assert!(shared.lock().unwrap().handoff_tail_buffer.is_empty());
    }));
}

#[test]
fn td125_cursor_zero_cancellation_cannot_resettle_known_start_authority() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(NATIVE_TRANSFER_BUDGET).await;
        let shared = Arc::new(Mutex::new(SharedState::default()));
        let mut source = establish_known_source(&mut harness, shared.clone()).await;

        source.set_client_capabilities(crate::window_interaction::IBUS_CAP_PREEDIT_TEXT | (1 << 3));
        source.committed_tail.buffer = "l ab".to_string();
        source.composition.buffer = "ab".to_string();
        source.composition.cursor = 0;
        source.composition.legacy_word_preedit_active = true;
        source.composition.preedit_visible = true;
        source.rebuild_preedit_fast_from_tail();
        {
            let mut state = shared.lock().unwrap();
            state.handoff_tail_buffer = "l ab".to_string();
            state.handoff_tail_epoch = source.committed_tail.epoch;
        }
        assert!(harness.adapter.test_set_word_scope(
            source.context_owner.as_ref().unwrap(),
            source.committed_tail.epoch,
            source.context_word_scope.as_ref().unwrap(),
        ));
        source.context_token = harness.adapter.current_token();
        let stale_known = source.live_context_token().expect("known preedit scope");
        assert!(source.context_word_is_known());

        let backspace = method_message(
            DISPATCH_SENDER,
            7_209,
            SOURCE_PATH,
            ENGINE_INTERFACE,
            "ProcessKeyEvent",
        );
        harness.peer.connection.send(&backspace).await.unwrap();
        let emitter = zbus::object_server::SignalEmitter::new(&harness.connection, SOURCE_PATH)
            .expect("source signal emitter");
        let (handled, observed) = bounded(future::zip(
            source.process_key_event(backspace.header(), emitter, KEY_BACKSPACE, 14, 0),
            harness.observer.process_next(),
        ))
        .await;
        assert!(observed.unwrap());
        assert!(!handled.unwrap());
        for expected in ["UpdatePreeditText", "HidePreeditText"] {
            let signal = bounded(next_peer_message(&mut harness.peer)).await;
            assert_eq!(signal.header().member().unwrap().as_str(), expected);
        }

        assert!(source.composition.buffer.is_empty());
        assert!(!source.composition.legacy_word_preedit_active);
        assert_eq!(source.committed_tail.buffer, "l");
        assert_eq!(shared.lock().unwrap().handoff_tail_buffer, "l");
        assert!(!source.context_word_is_known());
        assert!(source.capture_input_frame_identity().is_none());
        assert!(!harness.adapter.revalidate(&stale_known));
    }));
}

#[test]
fn td121_e0_e1_e2_delayed_handlers_install_only_latest_and_reject_late_e1_frame() {
    zbus::block_on(bounded(async {
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
        const E1: &str = "/io/github/radislabus_star/LayIme/engine/e1";
        const E2: &str = "/io/github/radislabus_star/LayIme/engine/e2";
        let mut harness = bootstrap_harness_with_budget(NATIVE_TRANSFER_BUDGET).await;
        let shared = Arc::new(Mutex::new(SharedState::default()));
        let mut e0 = establish_known_source(&mut harness, shared.clone()).await;
        e0.committed_tail.buffer = "пров".to_string();
        {
            let mut state = shared.lock().unwrap();
            state.handoff_tail_buffer = "пров".to_string();
            state.handoff_tail_epoch = e0.committed_tail.epoch;
        }

        let mut e1 = LayIbusEngine::new_from_component(
            E1.to_string(),
            shared.clone(),
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        open_factory_for(&mut harness, E1, 7_500).await;
        let e0_focus_out = focus_out_id_message_for(SOURCE_PATH, 7_501);
        send_observed(&mut harness, &e0_focus_out).await;
        let e0_disable = method_message(
            DISPATCH_SENDER,
            7_502,
            SOURCE_PATH,
            ENGINE_INTERFACE,
            "Disable",
        );
        send_observed(&mut harness, &e0_disable).await;
        start_native_focus(&mut harness, &mut e1, E1, CONTEXT_PATH, 7_503).await;
        assert!(
            e0.observe_context_focus_out(&e0_focus_out.header(), Instant::now())
                .await
        );
        e1.try_install_pending_context_activation();
        assert_eq!(e1.committed_tail.buffer, "пров");
        assert!(e1.context_word_is_known());
        let e1_owner = e1.context_owner.clone().expect("E1 installed owner");
        let e1_epoch = e1.committed_tail.epoch;
        let late_e1_frame = e1
            .capture_input_frame_identity()
            .expect("E1 full tail frame before E2 overlap");

        let mut e2 = LayIbusEngine::new_from_component(
            E2.to_string(),
            shared.clone(),
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        open_factory_for(&mut harness, E2, 7_504).await;
        let e1_focus_out = focus_out_id_message_for(E1, 7_505);
        send_observed(&mut harness, &e1_focus_out).await;
        let e1_disable = method_message(DISPATCH_SENDER, 7_506, E1, ENGINE_INTERFACE, "Disable");
        send_observed(&mut harness, &e1_disable).await;
        start_native_focus(&mut harness, &mut e2, E2, CONTEXT_PATH, 7_507).await;
        assert!(harness.adapter.shared.pending.lock().unwrap().is_some());

        assert!(
            !e0.observe_context_disable(&e0_disable.header(), Instant::now())
                .await
        );
        assert_eq!(harness.adapter.current_owner(), Some(e1_owner.clone()));
        assert!(
            e1.observe_context_focus_out(&e1_focus_out.header(), Instant::now())
                .await
        );
        e2.try_install_pending_context_activation();
        let e2_owner = e2.context_owner.clone().expect("E2 installed owner");
        assert_ne!(e2_owner, e1_owner);
        assert_eq!(e2_owner.path.as_str(), E2);
        assert_eq!(e2.committed_tail.buffer, "пров");
        assert!(e2.context_word_is_known());
        assert!(e2.committed_tail.epoch >= e1_epoch);
        assert_eq!(
            e2.live_context_token().unwrap().exact_field_receipt(),
            format!("context-admission:60\u{1f}{CONTEXT_PATH}")
        );
        assert_eq!(harness.adapter.current_owner(), Some(e2_owner.clone()));
        assert!(
            !e1.observe_context_disable(&e1_disable.header(), Instant::now())
                .await
        );

        e1.composition.buffer = "пров".to_string();
        e1.composition.cursor = 4;
        e1.composition.preedit_suffix = "ерка".to_string();
        e1.composition.preedit_candidates = vec!["ерка".to_string()];
        e1.composition.preedit_replacement_targets = vec![None];
        e1.composition.pending_display_frame = Some(late_e1_frame);
        let tab = method_message(
            DISPATCH_SENDER,
            7_508,
            E1,
            ENGINE_INTERFACE,
            "ProcessKeyEvent",
        );
        send_observed(&mut harness, &tab).await;
        let emitter = zbus::object_server::SignalEmitter::new(&harness.connection, E1).unwrap();
        let handled = e1
            .process_key_event(tab.header(), emitter, KEY_TAB, 15, 0)
            .await
            .unwrap();
        assert!(!handled);
        assert!(literal_effects(&harness.connection, &mut harness.peer, E1)
            .await
            .is_empty());
        assert_eq!(e2.committed_tail.buffer, "пров");
        assert_eq!(harness.adapter.current_owner(), Some(e2_owner));
        {
            let state = shared.lock().unwrap();
            assert_eq!(state.active_path.as_deref(), Some(E2));
            assert_eq!(state.handoff_tail_buffer, "пров");
            assert_eq!(state.handoff_tail_epoch, e2.committed_tail.epoch);
        }
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    }));
}

async fn controlled_material_transfer_case(native_context: &str, serial: u32) {
    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("warm exact-layout authority before the controlled source frame");
    let mut harness = bootstrap_harness_with_budget(NATIVE_TRANSFER_BUDGET).await;
    let shared = Arc::new(Mutex::new(SharedState::default()));
    let mut source = establish_known_source(&mut harness, shared.clone()).await;
    source.config.auto_replace = true;
    source.config.auto_switch_layout = true;
    assert!(source.config.auto_replace && source.config.auto_switch_layout);
    type_active_known_word(&mut harness, &mut source, &shared, serial - 10).await;
    assert_eq!(source.committed_tail.buffer, "ghbdtn");
    let stale_frame = source
        .capture_input_frame_identity()
        .expect("KnownStart source frame before pending transfer");
    assert!(stale_frame.active_composition);
    assert!(stale_frame.exact_authority_snapshot.is_some());
    assert!(stale_frame.config_matches(&source.config));
    assert!(source.input_frame_identity_matches(&stale_frame));
    assert!(harness.adapter.current_owner().is_some());
    crate::space_autocorrect_prefetch::proof::install_exact_lease_with_material_generation(
        &stale_frame,
        &source.config,
        lay::nanda_wave::candidate_material_generation().wrapping_add(1),
    );

    let target_path = if native_context == CONTEXT_PATH {
        "/io/github/radislabus_star/LayIme/engine/c28_known"
    } else {
        "/io/github/radislabus_star/LayIme/engine/c28_unknown"
    };
    let mut target = LayIbusEngine::new_from_component(
        target_path.to_string(),
        shared.clone(),
        Some(harness.adapter.clone()),
        "lay-ime-us",
        true,
        ime_config(),
    );
    open_factory_for(&mut harness, target_path, serial).await;
    let focus_out = focus_out_id_message_for(SOURCE_PATH, serial + 1);
    send_observed(&mut harness, &focus_out).await;
    let disable = method_message(
        DISPATCH_SENDER,
        serial + 2,
        SOURCE_PATH,
        ENGINE_INTERFACE,
        "Disable",
    );
    send_observed(&mut harness, &disable).await;

    source.config.auto_replace = !source.config.auto_replace;
    assert!(!source.input_frame_identity_matches(&stale_frame));
    assert!(
        source
            .observe_context_focus_out(&focus_out.header(), Instant::now())
            .await
    );
    let request_nonce = start_native_focus_before_marker(
        &mut harness,
        &mut target,
        target_path,
        native_context,
        serial + 3,
    )
    .await;
    let request_generation = {
        let reducer = harness.adapter.shared.reducer.lock().unwrap();
        let request = reducer.request.as_ref().expect("pending native request");
        assert_eq!(request.nonce, request_nonce);
        request.generation
    };
    assert!(harness
        .adapter
        .try_finish_activation_for(&engine_path(target_path))
        .unwrap()
        .is_none());
    {
        let reducer = harness.adapter.shared.reducer.lock().unwrap();
        let request = reducer
            .request
            .as_ref()
            .expect("source seal retains request");
        assert_eq!(
            (request.generation, request.nonce),
            (request_generation, request_nonce)
        );
    }
    forward_exact_marker(&mut harness.peer, request_nonce).await;
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    target.try_install_pending_context_activation();
    let expected_known = native_context == CONTEXT_PATH;
    assert_eq!(target.context_word_is_known(), expected_known);
    assert_eq!(
        target.committed_tail.buffer,
        if expected_known { "ghbdtn" } else { "" }
    );
    assert_eq!(
        target.capture_input_frame_identity().is_some(),
        expected_known,
        "UnknownStart must be checked before any ordinary Space can rearm the next word"
    );

    assert!(source.composition.buffer.is_empty());
    source.composition.preedit_suffix = "hello".to_string();
    source.composition.preedit_candidates = vec!["hello".to_string()];
    source.composition.preedit_replacement_targets = vec![None];
    source.composition.pending_display_frame = Some(stale_frame);
    let tab = method_message(
        DISPATCH_SENDER,
        serial + 4,
        SOURCE_PATH,
        ENGINE_INTERFACE,
        "ProcessKeyEvent",
    );
    send_observed(&mut harness, &tab).await;
    let emitter =
        zbus::object_server::SignalEmitter::new(&harness.connection, SOURCE_PATH).unwrap();
    let tab_handled = source
        .process_key_event(tab.header(), emitter, KEY_TAB, 15, 0)
        .await
        .unwrap();
    assert!(!tab_handled);
    assert!(
        literal_effects(&harness.connection, &mut harness.peer, SOURCE_PATH)
            .await
            .is_empty()
    );

    let space = method_message(
        DISPATCH_SENDER,
        serial + 5,
        SOURCE_PATH,
        ENGINE_INTERFACE,
        "ProcessKeyEvent",
    );
    send_observed(&mut harness, &space).await;
    let emitter =
        zbus::object_server::SignalEmitter::new(&harness.connection, SOURCE_PATH).unwrap();
    let space_handled = source
        .process_key_event(space.header(), emitter, KEY_SPACE, 57, 0)
        .await
        .unwrap();
    assert_exact_literal_delivery(
        &harness.connection,
        &mut harness.peer,
        SOURCE_PATH,
        space_handled,
        " ",
    )
    .await;
    assert_eq!(
        target.committed_tail.buffer,
        if expected_known { "ghbdtn" } else { "" }
    );
    assert_eq!(target.context_word_is_known(), expected_known);
    assert!(target.composition.preedit_suffix.is_empty());
    assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
}

#[test]
fn td121_pending_transfer_config_and_controlled_material_change_refuse_stale_frame_authority() {
    zbus::block_on(bounded(async {
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
        controlled_material_transfer_case(CONTEXT_PATH, 7_600).await;
        controlled_material_transfer_case(OTHER_CONTEXT_PATH, 7_700).await;
    }));
}

#[derive(Clone, Copy, Debug)]
enum PendingCrossProfileWordLoss {
    Reset,
    ContentType,
}

#[derive(Clone, Copy, Debug)]
enum PendingCrossProfileEvidence {
    Matching,
    MismatchedLay,
    Foreign,
}

async fn pending_cross_profile_word_loss_case(
    word_loss: PendingCrossProfileWordLoss,
    evidence: PendingCrossProfileEvidence,
    serial: u32,
) {
    let mut harness = bootstrap_harness_with_profiles_and_budget(
        "lay-us",
        vec![profile("lay-us"), profile("lay-ru")],
        NATIVE_TRANSFER_BUDGET,
    )
    .await;
    let shared = Arc::new(Mutex::new(SharedState::default()));
    let mut source = establish_known_source(&mut harness, shared.clone()).await;

    let factory = factory_message_for_profile(serial, "lay-ru");
    send_observed(&mut harness, &factory).await;
    let factory_callback = harness
        .adapter
        .begin_factory_callback(&factory.header(), Instant::now(), profile("lay-ru"))
        .await
        .unwrap();
    assert!(harness
        .adapter
        .bind_factory_target(&factory_callback, engine_path(TARGET_PATH)));

    let focus_out = focus_out_id_message_for(SOURCE_PATH, serial + 1);
    send_observed(&mut harness, &focus_out).await;
    let disable = method_message(
        DISPATCH_SENDER,
        serial + 2,
        SOURCE_PATH,
        ENGINE_INTERFACE,
        "Disable",
    );
    send_observed(&mut harness, &disable).await;
    let mut target = LayIbusEngine::new_from_component(
        TARGET_PATH.to_string(),
        shared,
        Some(harness.adapter.clone()),
        "lay-ime-ru",
        true,
        ime_config(),
    );
    start_native_focus(
        &mut harness,
        &mut target,
        TARGET_PATH,
        CONTEXT_PATH,
        serial + 3,
    )
    .await;
    assert!(
        source
            .observe_context_focus_out(&focus_out.header(), Instant::now())
            .await
    );
    let (request_generation, request_nonce) = {
        let reducer = harness.adapter.shared.reducer.lock().unwrap();
        let request = reducer
            .request
            .as_ref()
            .expect("pending cross-profile request");
        assert!(matches!(request.target, RequestTarget::Transfer(_)));
        (request.generation, request.nonce)
    };

    let unrelated = method_message(
        DISPATCH_SENDER,
        serial + 4,
        "/io/github/radislabus_star/LayIme/engine/unrelated",
        ENGINE_INTERFACE,
        "Reset",
    );
    send_observed(&mut harness, &unrelated).await;
    {
        let reducer = harness.adapter.shared.reducer.lock().unwrap();
        let request = reducer
            .request
            .as_ref()
            .expect("unrelated Reset preserves request");
        assert_eq!(
            (request.generation, request.nonce),
            (request_generation, request_nonce)
        );
    }

    let word_loss = match word_loss {
        PendingCrossProfileWordLoss::Reset => method_message(
            DISPATCH_SENDER,
            serial + 5,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "Reset",
        ),
        PendingCrossProfileWordLoss::ContentType => Message::method_call(TARGET_PATH, "Set")
            .unwrap()
            .interface(PROPERTIES_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(serial + 5).unwrap())
            .build(&(
                ENGINE_INTERFACE,
                "ContentType",
                zbus::zvariant::Value::from((10u32, 0u32)),
            ))
            .unwrap(),
    };
    send_observed(&mut harness, &word_loss).await;
    {
        let reducer = harness.adapter.shared.reducer.lock().unwrap();
        let request = reducer
            .request
            .as_ref()
            .expect("word loss must preserve the exact pending request");
        assert_eq!(
            (request.generation, request.nonce),
            (request_generation, request_nonce)
        );
        assert!(matches!(request.target, RequestTarget::SourceFree));
        assert_eq!(
            reducer.lineage().completeness,
            WordCompleteness::UnknownStart
        );
    }
    assert!(harness
        .adapter
        .try_finish_activation_for(&engine_path(TARGET_PATH))
        .unwrap()
        .is_none());

    let observed_profile = match evidence {
        PendingCrossProfileEvidence::Matching => "lay-ru",
        PendingCrossProfileEvidence::MismatchedLay => "lay-us",
        PendingCrossProfileEvidence::Foreign => "foreign-ime",
    };
    send_global_engine_changed(&mut harness, serial + 6, observed_profile).await;
    if !matches!(evidence, PendingCrossProfileEvidence::Matching) {
        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .is_none());
        assert!(harness.adapter.current_owner().is_none());
        assert!(harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .request
            .is_none());
        return;
    }
    let ActivationOutcome::SourceFree(grant) = harness
        .adapter
        .try_finish_activation_for(&engine_path(TARGET_PATH))
        .unwrap()
        .expect("matching ordered profile evidence completes the retained request")
    else {
        panic!("word loss must remove transfer authority")
    };
    assert_eq!(grant.lineage.completeness, WordCompleteness::UnknownStart);
    assert_eq!(grant.target_owner.path.as_str(), TARGET_PATH);
    assert!(harness
        .adapter
        .try_finish_activation_for(&engine_path(TARGET_PATH))
        .unwrap()
        .is_none());
}

#[test]
fn td121_pending_cross_profile_reset_or_content_type_retains_source_free_unknown_request() {
    zbus::block_on(bounded(async {
        pending_cross_profile_word_loss_case(
            PendingCrossProfileWordLoss::Reset,
            PendingCrossProfileEvidence::Matching,
            7_800,
        )
        .await;
        pending_cross_profile_word_loss_case(
            PendingCrossProfileWordLoss::ContentType,
            PendingCrossProfileEvidence::Matching,
            7_900,
        )
        .await;
        pending_cross_profile_word_loss_case(
            PendingCrossProfileWordLoss::Reset,
            PendingCrossProfileEvidence::MismatchedLay,
            8_000,
        )
        .await;
        pending_cross_profile_word_loss_case(
            PendingCrossProfileWordLoss::ContentType,
            PendingCrossProfileEvidence::Foreign,
            8_100,
        )
        .await;
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
