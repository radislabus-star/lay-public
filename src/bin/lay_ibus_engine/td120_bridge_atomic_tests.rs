use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

use lay::config::LayConfig;

use crate::atomic::td120_test_atomic_capability;
use crate::bridge::LayImeBridge;
use crate::engine::{LayIbusEngine, SurroundingTextSnapshot};
use crate::output::{PROPOSAL_FRAME_READY, PROPOSAL_NATIVE_UNHANDLED};
use crate::protocol::{AutocorrectSuppression, Shared, KEY_LEFT_SHIFT, KEY_SPACE};

const RECEIPT_NONE: (u8, u64, Vec<u8>) = (0, 0, Vec::new());

#[test]
fn legacy_replay_suppression_survives_only_the_live_controlled_layout_handoff() {
    zbus::block_on(async {
        for transition in ["controlled", "expired", "other_field", "unprepared"] {
            let shared = Arc::new(Mutex::new(Default::default()));
            let isolated = isolated_p2p_bridge(shared.clone()).await;
            let source_path = "/io/github/lay/replay_source";
            let mut source = engine(source_path, shared.clone(), true);
            source.bind_focus_receipt("/input/original".into(), "ibus".into());
            source.push_tail_char('a');
            if transition != "unprepared" {
                source.prepare_exact_manual_toggle_layout_handoff();
            }
            register_engine(&isolated.bridge, source_path, source).await;
            assert!(isolated
                .bridge
                .suppress_next_autocorrect_inner()
                .await
                .unwrap());
            if transition == "expired" {
                shared.lock().unwrap().preserve_active_path_until =
                    Some(std::time::Instant::now() - std::time::Duration::from_secs(1));
            }
            // IBus can activate the selected engine only after the daemon's
            // suppression RPC returns. No sleeps or synthetic widget receipts.
            let mut target = engine("/io/github/lay/replay_target", shared.clone(), false);
            if transition == "other_field" {
                assert!(target.bind_focus_receipt("/input/other".into(), "ibus".into()));
            }
            assert_eq!(
                target.take_manual_toggle_autocorrect_suppression(),
                transition == "controlled",
                "transition={transition}"
            );
            assert!(!target.take_manual_toggle_autocorrect_suppression());
            assert!(shared.lock().unwrap().autocorrect_suppression.is_none());
        }
    });
}

struct IsolatedBridge {
    bridge: LayImeBridge,
    _peer: zbus::Connection,
}

async fn isolated_p2p_bridge(shared: Shared) -> IsolatedBridge {
    isolated_p2p_bridge_configured(shared, None, None).await
}

async fn isolated_p2p_bridge_configured(
    shared: Shared,
    write_gate: Option<Arc<ObservedWriteGate>>,
    timeout: Option<std::time::Duration>,
) -> IsolatedBridge {
    let (server_stream, client_stream) = UnixStream::pair().expect("p2p stream pair");
    let guid = zbus::Guid::generate();
    let server = std::thread::spawn(move || {
        let builder = if let Some(gate) = write_gate {
            let split: zbus::connection::socket::BoxedSplit =
                async_io::Async::new(server_stream).unwrap().into();
            let (read, write) = split.take();
            zbus::connection::Builder::socket(zbus::connection::socket::BoxedSplit::new(
                read,
                Box::new(ObservedPartialWriter { inner: write, gate }),
            ))
        } else {
            zbus::connection::Builder::unix_stream(server_stream)
        };
        zbus::block_on(
            builder
                .server(guid)
                .expect("p2p server identity")
                .p2p()
                .build(),
        )
        .expect("p2p server connection")
    });
    let builder = zbus::connection::Builder::unix_stream(client_stream).p2p();
    let builder = if let Some(timeout) = timeout {
        builder.method_timeout(timeout)
    } else {
        builder
    };
    let peer = builder.build().await.expect("p2p client connection");
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

struct ObservedGlobalEnginePeer {
    name: Arc<Mutex<String>>,
    delay: Arc<std::sync::atomic::AtomicU64>,
}
#[zbus::interface(name = "org.freedesktop.IBus")]
impl ObservedGlobalEnginePeer {
    #[zbus(name = "GetGlobalEngine")]
    async fn engine(&self) -> zbus::fdo::Result<zbus::zvariant::OwnedValue> {
        use std::sync::atomic::Ordering;
        use zbus::zvariant::{OwnedValue, Structure, Value};
        let name = self.name.lock().unwrap().clone();
        let delay = self.delay.swap(0, Ordering::SeqCst);
        if delay != 0 {
            async_io::Timer::after(std::time::Duration::from_millis(delay)).await;
        }
        if name == "backend-error" {
            return Err(zbus::fdo::Error::Failed("backend unavailable".into()));
        }
        let value = Value::from(Structure::from((
            "IBusEngineDesc",
            std::collections::HashMap::<String, OwnedValue>::new(),
            name,
        )));
        value
            .try_to_owned()
            .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))
    }
}

#[derive(Debug, Default)]
struct ObservedWriteGate {
    armed: std::sync::atomic::AtomicBool,
    partial: std::sync::atomic::AtomicBool,
    released: std::sync::atomic::AtomicBool,
}
struct ObservedWriteRelease(Arc<ObservedWriteGate>);
impl Drop for ObservedWriteRelease {
    fn drop(&mut self) {
        self.0
            .released
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}
#[derive(Debug)]
struct ObservedPartialWriter {
    inner: Box<dyn zbus::connection::socket::WriteHalf>,
    gate: Arc<ObservedWriteGate>,
}
#[zbus::export::async_trait::async_trait]
impl zbus::connection::socket::WriteHalf for ObservedPartialWriter {
    async fn sendmsg(
        &mut self,
        buffer: &[u8],
        fds: &[std::os::fd::BorrowedFd<'_>],
    ) -> std::io::Result<usize> {
        use std::sync::atomic::Ordering;
        if self.gate.armed.swap(false, Ordering::SeqCst) {
            let n = self.inner.sendmsg(&buffer[..1], fds).await?;
            self.gate.partial.store(true, Ordering::SeqCst);
            return Ok(n);
        }
        while self.gate.partial.load(Ordering::SeqCst) && !self.gate.released.load(Ordering::SeqCst)
        {
            async_io::Timer::after(std::time::Duration::from_millis(1)).await;
        }
        self.inner.sendmsg(buffer, fds).await
    }
    async fn close(&mut self) -> std::io::Result<()> {
        self.inner.close().await
    }
    fn can_pass_unix_fd(&self) -> bool {
        self.inner.can_pass_unix_fd()
    }
    async fn peer_credentials(&mut self) -> std::io::Result<zbus::fdo::ConnectionCredentials> {
        self.inner.peer_credentials().await
    }
}
struct ObservedFixture {
    client: zbus::Connection,
    _service: zbus::Connection,
    _backend_peer: zbus::Connection,
    name: Arc<Mutex<String>>,
    delay: Arc<std::sync::atomic::AtomicU64>,
}
async fn observed_global_engine_fixture(
    shared: Shared,
    gate: Option<Arc<ObservedWriteGate>>,
) -> ObservedFixture {
    let backend = isolated_p2p_bridge_configured(shared.clone(), gate, None).await;
    let session =
        isolated_p2p_bridge_configured(shared, None, Some(std::time::Duration::from_millis(250)))
            .await;
    let name = Arc::new(Mutex::new("lay-ime-ru".to_string()));
    let delay = Arc::new(std::sync::atomic::AtomicU64::new(0));
    backend
        ._peer
        .object_server()
        .at(
            "/org/freedesktop/IBus",
            ObservedGlobalEnginePeer {
                name: name.clone(),
                delay: delay.clone(),
            },
        )
        .await
        .unwrap();
    let service = session.bridge.ibus_connection.clone();
    service
        .object_server()
        .at("/io/github/radislabus_star/LayIme", backend.bridge)
        .await
        .unwrap();
    ObservedFixture {
        client: session._peer,
        _service: service,
        _backend_peer: backend._peer,
        name,
        delay,
    }
}
async fn read_observed_engine_rpc(client: &zbus::Connection) -> zbus::Result<String> {
    let reply = client
        .call_method(
            Some("io.github.radislabus_star.LayIme"),
            "/io/github/radislabus_star/LayIme",
            Some("io.github.radislabus_star.LayIme"),
            "ObservedIbusEngine",
            &(),
        )
        .await?;
    reply.body().deserialize::<String>()
}
async fn wait_observed_read_completion() {
    let wait = async {
        while crate::observed_ibus_engine::test_read_in_flight() {
            async_io::Timer::after(std::time::Duration::from_millis(1)).await;
        }
    };
    futures_lite::future::race(wait, async {
        async_io::Timer::after(std::time::Duration::from_secs(2)).await;
        panic!("backend observation did not finish");
    })
    .await;
}
#[test]
fn observed_engine_rpc_reads_actual_backend_without_shared_state_access() {
    let shared: Shared = Arc::new(Mutex::new(Default::default()));
    let poisoned = shared.clone();
    assert!(std::thread::spawn(move || {
        let _guard = poisoned.lock().unwrap();
        panic!("poison projection to prove no getter dependence");
    })
    .join()
    .is_err());
    zbus::block_on(async {
        let f = observed_global_engine_fixture(shared.clone(), None).await;
        assert_eq!(
            read_observed_engine_rpc(&f.client).await.unwrap(),
            "lay-ime-ru"
        );
        *f.name.lock().unwrap() = "xkb:us::eng".into();
        assert_eq!(
            read_observed_engine_rpc(&f.client).await.unwrap(),
            "xkb:us::eng"
        );
        assert!(shared.is_poisoned());
        f.name.lock().unwrap().clear();
        assert!(read_observed_engine_rpc(&f.client).await.is_err());
        *f.name.lock().unwrap() = "backend-error".into();
        assert!(read_observed_engine_rpc(&f.client).await.is_err());
        assert!(!crate::observed_ibus_engine::test_read_in_flight());
    });
}
#[test]
fn observed_engine_rpc_client_timeout_does_not_cancel_backend_read() {
    use std::sync::atomic::Ordering;
    zbus::block_on(async {
        let f =
            observed_global_engine_fixture(Arc::new(Mutex::new(Default::default())), None).await;
        f.delay.store(350, Ordering::SeqCst);
        assert!(read_observed_engine_rpc(&f.client).await.is_err());
        assert!(crate::observed_ibus_engine::test_read_in_flight());
        assert!(read_observed_engine_rpc(&f.client)
            .await
            .unwrap_err()
            .to_string()
            .contains("already pending"));
        *f.name.lock().unwrap() = "lay-ime-us".into();
        wait_observed_read_completion().await;
        assert_eq!(
            read_observed_engine_rpc(&f.client).await.unwrap(),
            "lay-ime-us"
        );
    });
}
#[test]
fn observed_engine_rpc_partial_write_survives_caller_timeout_and_backpressure() {
    use std::sync::atomic::Ordering;
    zbus::block_on(async {
        let gate = Arc::new(ObservedWriteGate::default());
        let _release_on_failure = ObservedWriteRelease(gate.clone());
        let f = observed_global_engine_fixture(
            Arc::new(Mutex::new(Default::default())),
            Some(gate.clone()),
        )
        .await;
        gate.armed.store(true, Ordering::SeqCst);
        assert!(read_observed_engine_rpc(&f.client).await.is_err());
        assert!(gate.partial.load(Ordering::SeqCst));
        assert!(crate::observed_ibus_engine::test_read_in_flight());
        assert!(read_observed_engine_rpc(&f.client)
            .await
            .unwrap_err()
            .to_string()
            .contains("already pending"));
        gate.released.store(true, Ordering::SeqCst);
        wait_observed_read_completion().await;
        *f.name.lock().unwrap() = "lay-ime-us".into();
        assert_eq!(
            read_observed_engine_rpc(&f.client).await.unwrap(),
            "lay-ime-us"
        );
    });
}
