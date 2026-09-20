use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

use lay::config::LayConfig;

use crate::atomic::td120_test_atomic_capability;
use crate::bridge::LayImeBridge;
use crate::engine::{LayIbusEngine, SurroundingTextSnapshot};
use crate::output::{PROPOSAL_FRAME_READY, PROPOSAL_NATIVE_UNHANDLED};
use crate::protocol::{AutocorrectSuppression, Shared, KEY_LEFT_SHIFT, KEY_SPACE};

const RECEIPT_NONE: (u8, u64, Vec<u8>) = (0, 0, Vec::new());

struct IsolatedBridge {
    bridge: LayImeBridge,
    _peer: zbus::Connection,
}

async fn isolated_p2p_bridge(shared: Shared) -> IsolatedBridge {
    let (server_stream, client_stream) = UnixStream::pair().expect("p2p stream pair");
    let guid = zbus::Guid::generate();
    let server = std::thread::spawn(move || {
        zbus::block_on(
            zbus::connection::Builder::unix_stream(server_stream)
                .server(guid)
                .expect("p2p server identity")
                .p2p()
                .build(),
        )
        .expect("p2p server connection")
    });
    let peer = zbus::connection::Builder::unix_stream(client_stream)
        .p2p()
        .build()
        .await
        .expect("p2p client connection");
    let connection = server.join().expect("p2p server handshake");
    IsolatedBridge {
        bridge: LayImeBridge {
            ibus_connection: connection,
            shared,
            context_admission_required: false,
            admission: None,
        },
        _peer: peer,
    }
}

fn engine(path: &str, shared: Shared, layout_is_ru: bool) -> LayIbusEngine {
    let mut engine = LayIbusEngine::new(
        path.to_string(),
        shared,
        layout_is_ru,
        true,
        LayConfig {
            auto_replace: false,
            auto_switch_layout: false,
            text_backend: "ime".to_string(),
            ..LayConfig::default()
        },
    );
    assert!(engine.bind_focus_path());
    engine
}

fn envelope(transaction: u64, focus_epoch: u64) -> crate::atomic::AtomicEnvelope {
    (transaction, 2, 12, focus_epoch, 5, 13, vec![8; 32])
}

async fn register_engine(bridge: &LayImeBridge, path: &str, engine: LayIbusEngine) {
    assert!(bridge
        .ibus_connection
        .object_server()
        .at(path, engine)
        .await
        .expect("register isolated engine"));
}

async fn prepare_atomic_space(
    bridge: &LayImeBridge,
    path: &str,
    transaction: u64,
    focus_epoch: u64,
) {
    let iface = bridge
        .ibus_connection
        .object_server()
        .interface::<_, LayIbusEngine>(path)
        .await
        .expect("registered engine");
    let mut engine = iface.get_mut().await;
    let proposal = engine
        .process_atomic_key_event(
            KEY_SPACE,
            65,
            0,
            envelope(transaction, focus_epoch),
            td120_test_atomic_capability(),
            RECEIPT_NONE,
        )
        .await
        .expect("real atomic Space proposal");
    assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
    assert_eq!(proposal.1.iter().filter(|(tag, _)| *tag == 1).count(), 1);
}

async fn settle_atomic_space(
    bridge: &LayImeBridge,
    path: &str,
    transaction: u64,
    next_transaction: u64,
    focus_epoch: u64,
) {
    let iface = bridge
        .ibus_connection
        .object_server()
        .interface::<_, LayIbusEngine>(path)
        .await
        .expect("registered engine");
    let mut engine = iface.get_mut().await;
    let next = engine
        .process_atomic_key_event(
            KEY_LEFT_SHIFT,
            42,
            0,
            envelope(next_transaction, focus_epoch),
            td120_test_atomic_capability(),
            (2, transaction, vec![9; 32]),
        )
        .await
        .expect("settle atomic Space");
    assert_eq!(next.0, PROPOSAL_NATIVE_UNHANDLED);
    assert!(next.1.is_empty());
}

#[test]
fn td120_real_bridge_v1_arm_and_live_consume_survive_atomic_settlement() {
    zbus::block_on(async {
        let shared = Arc::new(Mutex::new(Default::default()));
        let isolated = isolated_p2p_bridge(shared.clone()).await;
        let path = "/io/github/lay/td120/bridge_v1_arm";
        let mut live = engine(path, shared.clone(), false);
        live.push_tail_char('a');
        register_engine(&isolated.bridge, path, live).await;

        prepare_atomic_space(&isolated.bridge, path, 501, 81).await;
        assert!(isolated
            .bridge
            .suppress_next_autocorrect_inner()
            .await
            .expect("real V1 bridge admission"));
        let armed_revision = shared.lock().expect("shared state").suppression_revision;
        settle_atomic_space(&isolated.bridge, path, 501, 502, 81).await;
        {
            let state = shared.lock().expect("shared state");
            assert_eq!(state.suppression_revision, armed_revision);
            assert!(matches!(
                state.autocorrect_suppression.as_ref(),
                Some(AutocorrectSuppression::LegacyReplayV1)
            ));
        }

        let consumed_path = "/io/github/lay/td120/bridge_v1_consume";
        let consumed_shared = Arc::new(Mutex::new(Default::default()));
        let consumed_bridge = isolated_p2p_bridge(consumed_shared.clone()).await;
        let mut live = engine(consumed_path, consumed_shared.clone(), false);
        live.push_tail_char('b');
        register_engine(&consumed_bridge.bridge, consumed_path, live).await;
        prepare_atomic_space(&consumed_bridge.bridge, consumed_path, 503, 82).await;
        assert!(consumed_bridge
            .bridge
            .suppress_next_autocorrect_inner()
            .await
            .expect("real V1 bridge admission"));
        {
            let iface = consumed_bridge
                .bridge
                .ibus_connection
                .object_server()
                .interface::<_, LayIbusEngine>(consumed_path)
                .await
                .expect("registered engine");
            assert!(iface
                .get_mut()
                .await
                .take_manual_toggle_autocorrect_suppression());
        }
        let consumed_revision = consumed_shared
            .lock()
            .expect("shared state")
            .suppression_revision;
        settle_atomic_space(&consumed_bridge.bridge, consumed_path, 503, 504, 82).await;
        let state = consumed_shared.lock().expect("shared state");
        assert_eq!(state.suppression_revision, consumed_revision);
        assert!(state.autocorrect_suppression.is_none());
    });
}

#[test]
fn td120_real_bridge_v2_arm_and_revoke_survive_atomic_settlement() {
    zbus::block_on(async {
        let shared = Arc::new(Mutex::new(Default::default()));
        let isolated = isolated_p2p_bridge(shared.clone()).await;
        let path = "/io/github/lay/td120/bridge_v2_arm";
        let mut live = engine(path, shared.clone(), false);
        live.push_tail_char('ч');
        live.set_client_capabilities(1 << 5);
        live.client_context.surrounding_text_snapshot =
            Some(SurroundingTextSnapshot::new("ч".to_string(), 1, 1));
        live.prepare_exact_manual_toggle_layout_handoff();
        let epoch = live.committed_tail.epoch;
        register_engine(&isolated.bridge, path, live).await;

        prepare_atomic_space(&isolated.bridge, path, 511, 83).await;
        assert!(isolated
            .bridge
            .suppress_next_autocorrect_v2_inner("ч".to_string(), epoch, path.to_string(), false)
            .await
            .expect("real V2 bridge admission"));
        let armed_revision = shared.lock().expect("shared state").suppression_revision;
        settle_atomic_space(&isolated.bridge, path, 511, 512, 83).await;
        {
            let state = shared.lock().expect("shared state");
            assert_eq!(state.suppression_revision, armed_revision);
            assert!(matches!(
                state.autocorrect_suppression.as_ref(),
                Some(AutocorrectSuppression::ExactReplay(_))
            ));
        }

        prepare_atomic_space(&isolated.bridge, path, 513, 84).await;
        assert!(isolated
            .bridge
            .cancel_exact_manual_toggle_suppression_v2_inner(epoch, path.to_string())
            .await
            .expect("real V2 bridge revoke"));
        let revoked_revision = shared.lock().expect("shared state").suppression_revision;
        settle_atomic_space(&isolated.bridge, path, 513, 514, 84).await;
        let state = shared.lock().expect("shared state");
        assert_eq!(state.suppression_revision, revoked_revision);
        assert!(state.autocorrect_suppression.is_none());
    });
}

#[test]
fn td120_real_bridge_handoff_cancel_censors_real_pending_frame_and_feedback() {
    zbus::block_on(async {
        let shared = Arc::new(Mutex::new(Default::default()));
        let isolated = isolated_p2p_bridge(shared.clone()).await;
        let path = "/io/github/lay/td120/bridge_handoff_cancel";
        let mut live = engine(path, shared.clone(), false);
        live.push_tail_char('x');
        live.prepare_exact_manual_toggle_layout_handoff();
        let epoch = live.committed_tail.epoch;
        register_engine(&isolated.bridge, path, live).await;

        prepare_atomic_space(&isolated.bridge, path, 521, 85).await;
        {
            let iface = isolated
                .bridge
                .ibus_connection
                .object_server()
                .interface::<_, LayIbusEngine>(path)
                .await
                .expect("registered engine");
            let engine = iface.get().await;
            crate::atomic::td120_test_defer_reverted_feedback_on_pending(&engine);
        }
        assert!(
            isolated
                .bridge
                .cancel_exact_manual_toggle_handoff_v2_inner(epoch, path.to_string())
                .await
        );
        settle_atomic_space(&isolated.bridge, path, 521, 522, 85).await;

        {
            let state = shared.lock().expect("shared state");
            assert_eq!(state.active_path.as_deref(), Some(path));
            assert!(state.handoff_tail_buffer.is_empty());
            assert!(state.autocorrect_suppression.is_none());
        }
        let iface = isolated
            .bridge
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(path)
            .await
            .expect("registered engine");
        let engine = iface.get().await;
        assert_eq!(
            *engine
                .atomic
                .settlement_feedback_events
                .lock()
                .expect("atomic feedback events"),
            ["censored"]
        );
        drop(engine);
        settle_atomic_space(&isolated.bridge, path, 521, 523, 85).await;
        let engine = iface.get().await;
        assert_eq!(
            *engine
                .atomic
                .settlement_feedback_events
                .lock()
                .expect("atomic feedback events"),
            ["censored"]
        );
    });
}

#[test]
fn td120_real_bridge_refuses_v1_without_active_owner() {
    zbus::block_on(async {
        let shared = Arc::new(Mutex::new(Default::default()));
        let isolated = isolated_p2p_bridge(shared.clone()).await;
        assert!(!isolated
            .bridge
            .suppress_next_autocorrect_inner()
            .await
            .expect("ownerless V1 refusal"));
        assert!(shared
            .lock()
            .expect("shared state")
            .autocorrect_suppression
            .is_none());
    });
}

#[test]
fn td120_v1_receiver_schedule_freezes_before_only_and_delayed_residuals() {
    zbus::block_on(async {
        let shared = Arc::new(Mutex::new(Default::default()));
        let isolated = isolated_p2p_bridge(shared.clone()).await;
        let path = "/io/github/lay/td120/v1_schedule";
        let mut live = engine(path, shared.clone(), false);
        live.push_tail_char('a');
        register_engine(&isolated.bridge, path, live).await;

        // The daemon's first/before call is accepted for the original word.
        assert!(isolated
            .bridge
            .suppress_next_autocorrect_inner()
            .await
            .expect("V1 before call"));
        let first_revision = shared.lock().expect("shared state").suppression_revision;
        assert!(isolated
            .bridge
            .suppress_next_autocorrect_inner()
            .await
            .expect("duplicate V1 call"));
        assert_eq!(
            shared.lock().expect("shared state").suppression_revision,
            first_revision.wrapping_add(1)
        );
        let iface = isolated
            .bridge
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(path)
            .await
            .expect("registered engine");
        let mut engine_guard = iface.get_mut().await;
        assert!(engine_guard.take_manual_toggle_autocorrect_suppression());
        engine_guard.push_tail_char(' ');
        engine_guard.push_tail_char('b');
        drop(engine_guard);

        // KNOWN_RESIDUAL: a delayed after-call is an actual V1 RPC with no
        // request identity, so it arms the new word rather than the replayed one.
        assert!(isolated
            .bridge
            .suppress_next_autocorrect_inner()
            .await
            .expect("delayed V1 after call"));
        let mut engine_guard = iface.get_mut().await;
        assert!(matches!(
            engine_guard.committed_tail.autocorrect_suppression.as_ref(),
            Some(AutocorrectSuppression::LegacyReplayV1)
        ));
        assert!(engine_guard.take_manual_toggle_autocorrect_suppression());
        assert!(!engine_guard.take_manual_toggle_autocorrect_suppression());
        drop(engine_guard);

        // KNOWN_RESIDUAL: the same delayed RPC resolves the new active engine.
        let path_b = "/io/github/lay/td120/v1_schedule_b";
        let mut engine_b = engine(path_b, shared.clone(), false);
        engine_b.push_tail_char('c');
        register_engine(&isolated.bridge, path_b, engine_b).await;
        assert!(isolated
            .bridge
            .suppress_next_autocorrect_inner()
            .await
            .expect("delayed V1 after owner change"));
        let iface_b = isolated
            .bridge
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(path_b)
            .await
            .expect("new owner engine");
        let mut engine_b = iface_b.get_mut().await;
        assert!(matches!(
            engine_b.committed_tail.autocorrect_suppression.as_ref(),
            Some(AutocorrectSuppression::LegacyReplayV1)
        ));
        assert!(engine_b.take_manual_toggle_autocorrect_suppression());
        let engine_a = iface.get().await;
        assert!(engine_a.committed_tail.autocorrect_suppression.is_none());
    });
}

#[test]
fn td120_v1_receiver_accepts_successful_replay_schedules_once() {
    zbus::block_on(async {
        for (case, replayed_tail) in [("open", "abc"), ("boundary", "abc ")] {
            let shared = Arc::new(Mutex::new(Default::default()));
            let isolated = isolated_p2p_bridge(shared.clone()).await;
            let path = format!("/io/github/lay/td120/v1_success_{case}");
            let mut live = engine(&path, shared, false);
            live.push_tail_char('x');
            register_engine(&isolated.bridge, &path, live).await;

            assert!(isolated
                .bridge
                .suppress_next_autocorrect_inner()
                .await
                .expect("V1 before replay"));
            {
                let iface = isolated
                    .bridge
                    .ibus_connection
                    .object_server()
                    .interface::<_, LayIbusEngine>(path.as_str())
                    .await
                    .expect("registered engine");
                let mut engine = iface.get_mut().await;
                while !engine.committed_tail.buffer.is_empty() {
                    engine.backspace_committed_tail_only();
                }
                for ch in replayed_tail.chars() {
                    engine.push_tail_char(ch);
                }
            }
            assert!(isolated
                .bridge
                .suppress_next_autocorrect_inner()
                .await
                .expect("V1 after successful replay"));
            let iface = isolated
                .bridge
                .ibus_connection
                .object_server()
                .interface::<_, LayIbusEngine>(path.as_str())
                .await
                .expect("registered engine");
            let mut engine = iface.get_mut().await;
            assert!(engine.take_manual_toggle_autocorrect_suppression());
            assert!(!engine.take_manual_toggle_autocorrect_suppression());
        }
    });
}
