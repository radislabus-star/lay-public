use std::collections::BTreeSet;
use std::num::NonZeroU32;
use std::os::unix::net::UnixStream;
use std::pin::Pin;
use std::thread;
use std::time::{Duration, Instant};

use futures_lite::future;
use zbus::connection;
use zbus::message::Type;
use zbus::zvariant::{ObjectPath, OwnedValue, StructureBuilder};

use super::*;
use crate::atomic::td120_test_atomic_capability;
use crate::context_admission::rendezvous::RendezvousBegin;
use crate::engine::{LayIbusEngine, SurroundingTextSnapshot};
use crate::output::{PROPOSAL_FRAME_READY, PROPOSAL_NATIVE_UNHANDLED};
use crate::protocol::SharedState;

#[path = "tests/native_transfer.rs"]
mod native_transfer;
#[path = "tests/word_scope.rs"]
pub(crate) mod word_scope;

const ADAPTER_SENDER: &str = ":1.2";
const IBUS_SENDER: &str = ":1.0";
const DISPATCH_SENDER: &str = ":1.42";
const SOURCE_PATH: &str = "/io/github/radislabus_star/LayIme/engine/source";
const TARGET_PATH: &str = "/io/github/radislabus_star/LayIme/engine/target";
const CONTEXT_PATH: &str = "/org/freedesktop/IBus/InputContext_1";

struct ControlledPeer {
    connection: Connection,
    incoming: MessageStream,
    detached_callback_reply_serials: BTreeSet<u32>,
}

struct Harness {
    connection: Connection,
    peer: ControlledPeer,
    adapter: ContextAdmissionAdapter,
    observer: ContextAdmissionObserver,
    identity: BootstrapIdentity,
}

fn controlled_pair() -> (Connection, ControlledPeer) {
    let (adapter_socket, peer_socket) = UnixStream::pair().unwrap();
    let guid = zbus::Guid::generate();
    let server_guid = guid.clone();
    let adapter_thread = thread::spawn(move || {
        zbus::block_on(async move {
            connection::Builder::unix_stream(adapter_socket)
                .server(server_guid)
                .unwrap()
                .p2p()
                .unique_name(ADAPTER_SENDER)
                .unwrap()
                .build()
                .await
                .unwrap()
        })
    });
    let peer_connection = zbus::block_on(async move {
        connection::Builder::unix_stream(peer_socket)
            .p2p()
            .build()
            .await
            .unwrap()
    });
    peer_connection.set_unique_name(IBUS_SENDER).unwrap();
    let adapter_connection = adapter_thread.join().unwrap();
    let incoming = MessageStream::from(&peer_connection);
    (
        adapter_connection,
        ControlledPeer {
            connection: peer_connection,
            incoming,
            detached_callback_reply_serials: BTreeSet::new(),
        },
    )
}

async fn next_peer_message_raw(peer: &mut ControlledPeer) -> Message {
    next_ordered_message(Pin::new(&mut peer.incoming))
        .await
        .expect("controlled peer stream remains open")
        .expect("controlled peer message")
}

async fn next_peer_message(peer: &mut ControlledPeer) -> Message {
    loop {
        let message = next_peer_message_raw(peer).await;
        let detached_reply = message.header().message_type() == Type::Error
            && message.header().error_name().map(|name| name.as_str())
                == Some("org.freedesktop.DBus.Error.UnknownObject")
            && message
                .header()
                .reply_serial()
                .is_some_and(|serial| peer.detached_callback_reply_serials.remove(&serial.get()));
        if !detached_reply {
            return message;
        }
    }
}

fn global_engine_value(name: &str) -> OwnedValue {
    let descriptor = StructureBuilder::new()
        .add_field(name)
        .add_field("Lay US")
        .add_field("controlled descriptor")
        .build()
        .unwrap();
    OwnedValue::try_from(descriptor).unwrap()
}

async fn serve_bootstrap(peer: &mut ControlledPeer, profile_name: &str) {
    serve_bootstrap_with_mode(peer, Some(profile_name), true).await;
}

async fn serve_bootstrap_with_mode(
    peer: &mut ControlledPeer,
    profile_name: Option<&str>,
    use_global_engine: bool,
) {
    for step in 0..4 {
        let call = next_peer_message(peer).await;
        assert_eq!(call.header().message_type(), Type::MethodCall);
        let member = call.header().member().unwrap().as_str().to_owned();
        match (step, member.as_str()) {
            (0, "GetNameOwner") | (1, "GetNameOwner") => {
                let name = call.body().deserialize::<String>().unwrap();
                let owner = match name.as_str() {
                    IBUS_NAME => IBUS_SENDER,
                    DBUS_NAME => DISPATCH_SENDER,
                    _ => panic!("unexpected bootstrap name {name}"),
                };
                peer.connection.reply(&call.header(), &owner).await.unwrap();
            }
            (2, "GetUseGlobalEngine") => {
                peer.connection
                    .reply(&call.header(), &use_global_engine)
                    .await
                    .unwrap();
            }
            (3, "Get") => {
                let (interface, property) = call.body().deserialize::<(String, String)>().unwrap();
                assert_eq!(interface, IBUS_INTERFACE);
                assert_eq!(property, "GlobalEngine");
                if let Some(profile_name) = profile_name {
                    peer.connection
                        .reply(&call.header(), &global_engine_value(profile_name))
                        .await
                        .unwrap();
                } else {
                    peer.connection
                        .reply_error(
                            &call.header(),
                            "org.freedesktop.DBus.Error.Failed",
                            &"No global engine.",
                        )
                        .await
                        .unwrap();
                }
            }
            _ => panic!("unexpected bootstrap step {step}: {member}"),
        }
    }
}

#[test]
fn td121_false_global_mode_refuses_authority_and_literal_delivery_is_exact() {
    zbus::block_on(async {
        let (connection, mut peer) = controlled_pair();
        let config = AdapterConfig::new(ConnectionGeneration(60), vec![profile("lay-us")]).unwrap();
        let pending = PendingContextAdapter::subscribe(connection.clone(), config)
            .await
            .unwrap();
        let (result, ()) = future::zip(
            pending.bootstrap(),
            serve_bootstrap_with_mode(&mut peer, Some("lay-us"), false),
        )
        .await;
        let (adapter, mut observer, identity) = result.unwrap();
        assert_eq!(identity.mode, GlobalEngineMode::UnsupportedOrFalse);
        assert!(adapter.current_owner().is_none());
        assert!(adapter.current_token().is_none());

        let shared = Arc::new(Mutex::new(SharedState::default()));
        let mut engine = LayIbusEngine::new_from_component(
            TARGET_PATH.to_string(),
            shared,
            Some(adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        let key = method_message(
            DISPATCH_SENDER,
            590,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "ProcessKeyEvent",
        );
        peer.connection.send(&key).await.unwrap();
        let emitter = zbus::object_server::SignalEmitter::new(&connection, TARGET_PATH).unwrap();
        let (handled, observed) = future::zip(
            engine.process_key_event(key.header(), emitter, u32::from(b'a'), 30, 0),
            observer.process_next(),
        )
        .await;
        assert!(observed.unwrap());
        let handled = handled.unwrap();
        assert_exact_literal_delivery(&connection, &mut peer, TARGET_PATH, handled, "a").await;
        assert_eq!(engine.committed_tail.buffer, if handled { "a" } else { "" });
        assert!(engine.composition.buffer.is_empty());
        assert!(adapter.current_owner().is_none());
        assert!(adapter.current_token().is_none());
    });
}

#[test]
fn no_global_engine_bootstrap_keeps_observer_for_later_verified_lay_profile() {
    zbus::block_on(async {
        let (connection, mut peer) = controlled_pair();
        let config = AdapterConfig::new(ConnectionGeneration(60), vec![profile("lay-us")]).unwrap();
        let pending = PendingContextAdapter::subscribe(connection.clone(), config)
            .await
            .unwrap();
        let (result, ()) = future::zip(
            pending.bootstrap(),
            serve_bootstrap_with_mode(&mut peer, None, true),
        )
        .await;
        let (adapter, mut observer, identity) = result.expect("unset engine is a startup state");
        assert!(adapter.current_owner().is_none());
        assert!(adapter.current_token().is_none());

        let factory = Message::method_call("/org/freedesktop/IBus/Factory", "CreateEngine")
            .unwrap()
            .interface(FACTORY_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(5_900).unwrap())
            .build(&"lay-us")
            .unwrap();
        peer.connection.send(&factory).await.unwrap();
        assert!(observer.process_next().await.unwrap());
        let callback = adapter
            .begin_factory_callback(&factory.header(), Instant::now(), profile("lay-us"))
            .await
            .unwrap();
        assert!(adapter.bind_factory_target(&callback, engine_path(SOURCE_PATH)));
        assert!(adapter.current_token().is_none());

        let changed = Message::signal(IBUS_PATH, IBUS_INTERFACE, "GlobalEngineChanged")
            .unwrap()
            .sender(IBUS_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(5_901).unwrap())
            .build(&"lay-us")
            .unwrap();
        peer.connection.send(&changed).await.unwrap();
        assert!(observer.process_next().await.unwrap());
        let reducer = adapter.shared.reducer.lock().unwrap();
        assert!(matches!(reducer.profile, GlobalProfile::Lay(_)));
        assert!(reducer
            .source_free_factory
            .as_ref()
            .is_some_and(|reservation| {
                reservation.status == TicketStatus::Pending
                    && reservation.target_path.as_ref() == Some(&engine_path(SOURCE_PATH))
            }));
        drop(reducer);
        assert!(adapter.current_token().is_none());
        assert_eq!(identity.mode, GlobalEngineMode::Verified);
    });
}

#[test]
fn td121_unverified_bootstrap_failure_is_bounded_and_fresh_literal_has_no_transfer_authority() {
    zbus::block_on(async {
        let (connection, mut peer) = controlled_pair();
        let config = AdapterConfig::new(ConnectionGeneration(59), vec![profile("lay-us")]).unwrap();
        let pending = PendingContextAdapter::subscribe(connection, config)
            .await
            .unwrap();
        let malformed_peer = async {
            for step in 0..3 {
                let call = next_peer_message(&mut peer).await;
                match step {
                    0 | 1 => {
                        let name = call.body().deserialize::<String>().unwrap();
                        let owner = if name == IBUS_NAME {
                            IBUS_SENDER
                        } else {
                            DISPATCH_SENDER
                        };
                        peer.connection.reply(&call.header(), &owner).await.unwrap();
                    }
                    2 => peer
                        .connection
                        .reply(&call.header(), &"not-a-boolean")
                        .await
                        .unwrap(),
                    _ => unreachable!(),
                }
            }
        };
        let completed = future::race(
            async {
                let (result, ()) = future::zip(pending.bootstrap(), malformed_peer).await;
                assert!(matches!(result, Err(AdapterError::InvalidBootstrap(_))));
                true
            },
            async {
                async_io::Timer::after(Duration::from_millis(50)).await;
                false
            },
        )
        .await;
        assert!(
            completed,
            "malformed bootstrap must fail without an async hang"
        );

        let mut fresh = bootstrap_harness().await;
        assert!(fresh.adapter.current_owner().is_none());
        assert!(fresh.adapter.current_token().is_none());
        let shared = Arc::new(Mutex::new(SharedState::default()));
        let mut engine = LayIbusEngine::new_from_component(
            TARGET_PATH.to_string(),
            shared,
            Some(fresh.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        let key = method_message(
            DISPATCH_SENDER,
            591,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "ProcessKeyEvent",
        );
        fresh.peer.connection.send(&key).await.unwrap();
        let emitter =
            zbus::object_server::SignalEmitter::new(&fresh.connection, TARGET_PATH).unwrap();
        let (handled, observed) = future::zip(
            engine.process_key_event(key.header(), emitter, u32::from(b'a'), 30, 0),
            fresh.observer.process_next(),
        )
        .await;
        assert!(observed.unwrap());
        let handled = handled.unwrap();
        assert_exact_literal_delivery(
            &fresh.connection,
            &mut fresh.peer,
            TARGET_PATH,
            handled,
            "a",
        )
        .await;
        assert_eq!(engine.committed_tail.buffer, if handled { "a" } else { "" });
        assert!(engine.composition.buffer.is_empty());
        assert!(fresh.adapter.current_owner().is_none());
        assert!(fresh.adapter.current_token().is_none());
    });
}

async fn bootstrap_harness() -> Harness {
    bootstrap_harness_with_budget(Duration::from_millis(250)).await
}

async fn bootstrap_harness_with_budget(acquisition_budget: Duration) -> Harness {
    bootstrap_harness_with_profile_and_budget("lay-us", acquisition_budget).await
}

async fn bootstrap_harness_with_profile_and_budget(
    profile_name: &str,
    acquisition_budget: Duration,
) -> Harness {
    bootstrap_harness_with_profiles_and_budget(
        profile_name,
        vec![profile("lay-us")],
        acquisition_budget,
    )
    .await
}

async fn bootstrap_harness_with_profiles_and_budget(
    profile_name: &str,
    profiles: Vec<EngineProfile>,
    acquisition_budget: Duration,
) -> Harness {
    let config = AdapterConfig::new(ConnectionGeneration(60), profiles)
        .unwrap()
        .with_acquisition_budget(acquisition_budget);
    bootstrap_harness_with_config(profile_name, config).await
}

async fn bootstrap_harness_with_config(profile_name: &str, config: AdapterConfig) -> Harness {
    let (connection, mut peer) = controlled_pair();
    let pending = PendingContextAdapter::subscribe(connection.clone(), config)
        .await
        .unwrap();
    let (result, ()) = future::zip(
        pending.bootstrap(),
        serve_bootstrap(&mut peer, profile_name),
    )
    .await;
    let (adapter, observer, identity) = result.unwrap();
    Harness {
        connection,
        peer,
        adapter,
        observer,
        identity,
    }
}

fn profile(name: &str) -> EngineProfile {
    EngineProfile::new(name).unwrap()
}

fn engine_path(path: &str) -> EnginePath {
    EnginePath::new(path).unwrap()
}

fn context(path: &str) -> ContextKey {
    ContextKey::new(ConnectionGeneration(60), path).unwrap()
}

fn ime_config() -> lay::config::LayConfig {
    lay::config::LayConfig {
        text_backend: "ime".to_string(),
        ..lay::config::LayConfig::default()
    }
}

async fn forward_marker(peer: &mut ControlledPeer) {
    let marker = next_peer_message(peer).await;
    assert_eq!(marker.header().message_type(), Type::Signal);
    assert_eq!(
        marker.header().interface().unwrap().as_str(),
        MARKER_INTERFACE
    );
    assert_eq!(marker.header().member().unwrap().as_str(), MARKER_MEMBER);
    let nonce = marker.body().deserialize::<u64>().unwrap();
    let forwarded = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
        .unwrap()
        .sender(ADAPTER_SENDER)
        .unwrap()
        .build(&nonce)
        .unwrap();
    peer.connection.send(&forwarded).await.unwrap();
}

async fn literal_effects(
    connection: &Connection,
    peer: &mut ControlledPeer,
    path: &str,
) -> Vec<Message> {
    connection
        .emit_signal(None::<&str>, path, "org.lay.Proof", "LiteralSettled", &())
        .await
        .unwrap();
    future::race(
        async {
            let mut effects = Vec::new();
            loop {
                let message = next_peer_message(peer).await;
                let header = message.header();
                assert_eq!(header.message_type(), Type::Signal);
                assert_eq!(header.path().map(|value| value.as_str()), Some(path));
                let interface = header.interface().unwrap().as_str().to_owned();
                let member = header.member().unwrap().as_str().to_owned();
                if interface == "org.lay.Proof" {
                    assert_eq!(member, "LiteralSettled");
                    return effects;
                }
                assert_eq!(interface, ENGINE_INTERFACE);
                match member.as_str() {
                    "UpdatePreeditText" => {
                        let body = message.body();
                        let (text, cursor, visible, mode) = body
                            .deserialize::<(zbus::zvariant::Value<'_>, u32, bool, u32)>()
                            .unwrap();
                        assert_eq!(
                            crate::ibus_interface::ibus_text_value_to_string(&text).as_deref(),
                            Some("")
                        );
                        assert_eq!((cursor, visible, mode), (0, false, 0));
                    }
                    "HidePreeditText" => {}
                    "CommitText" | "DeleteSurroundingText" | "ForwardKeyEvent" => {
                        effects.push(message);
                    }
                    member => panic!("unexpected literal-path engine effect {member}"),
                }
            }
        },
        async {
            async_io::Timer::after(Duration::from_millis(10)).await;
            panic!("literal effect FIFO marker timed out")
        },
    )
    .await
}

async fn assert_exact_literal_delivery(
    connection: &Connection,
    peer: &mut ControlledPeer,
    path: &str,
    handled: bool,
    expected: &str,
) {
    let effects = literal_effects(connection, peer, path).await;
    if handled {
        assert_eq!(
            effects.len(),
            1,
            "handled literal requires exactly one effect"
        );
        assert_eq!(effects[0].header().member().unwrap().as_str(), "CommitText");
        let body = effects[0].body();
        let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
        assert_eq!(
            crate::ibus_interface::ibus_text_value_to_string(&value).as_deref(),
            Some(expected)
        );
    } else {
        assert!(
            effects.is_empty(),
            "native literal must not emit an engine effect"
        );
    }
}

async fn serve_current_context_and_marker(peer: &mut ControlledPeer) {
    let call = next_peer_message(peer).await;
    assert_eq!(
        call.header().member().map(|m| m.as_str()),
        Some("Get"),
        "{:?}",
        call.header()
    );
    let (interface, property) = call.body().deserialize::<(String, String)>().unwrap();
    assert_eq!(interface, IBUS_INTERFACE);
    assert_eq!(property, "CurrentInputContext");
    let value = OwnedValue::from(ObjectPath::try_from(CONTEXT_PATH).unwrap());
    peer.connection.reply(&call.header(), &value).await.unwrap();
    forward_marker(peer).await;
}

async fn serve_ping_and_marker(peer: &mut ControlledPeer) {
    let call = next_peer_message(peer).await;
    assert_eq!(call.header().member().unwrap().as_str(), "Ping");
    let value = call.body().deserialize::<OwnedValue>().unwrap();
    peer.connection.reply(&call.header(), &value).await.unwrap();
    forward_marker(peer).await;
}

async fn complete_native_activation(harness: &mut Harness) -> ActivationGrant {
    let begin = harness.adapter.begin_native_activation(
        engine_path(SOURCE_PATH),
        context(CONTEXT_PATH),
        Default::default(),
    );
    let (fence, ()) = future::zip(begin, forward_marker(&mut harness.peer)).await;
    let fence = fence.unwrap();
    assert!(harness.observer.process_next().await.unwrap());
    match harness.adapter.finish_activation(fence).unwrap() {
        ActivationOutcome::SourceFree(grant) => grant,
        ActivationOutcome::Transfer(_) => panic!("initial activation cannot transfer text"),
        ActivationOutcome::ResetUnknown(_) => panic!("initial activation was not reset"),
    }
}

fn method_message(sender: &str, serial: u32, path: &str, interface: &str, member: &str) -> Message {
    Message::method_call(path, member)
        .unwrap()
        .interface(interface)
        .unwrap()
        .sender(sender)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .with_flags(zbus::message::Flags::NoReplyExpected)
        .unwrap()
        .build(&())
        .unwrap()
}

#[test]
fn controlled_p2p_bootstrap_and_compatibility_get_marker_are_real_zbus() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness().await;
        assert_eq!(harness.identity.ibus_owner, IBUS_SENDER);
        assert_eq!(harness.identity.own_sender, ADAPTER_SENDER);
        assert_eq!(harness.identity.engine_callback_sender, DISPATCH_SENDER);
        assert_eq!(harness.identity.mode, GlobalEngineMode::Verified);
        assert_eq!(
            harness.identity.profile,
            GlobalProfile::Lay(profile("lay-us"))
        );

        let begin = harness
            .adapter
            .begin_compatibility_activation(engine_path(SOURCE_PATH), Default::default());
        let (fence, ()) =
            future::zip(begin, serve_current_context_and_marker(&mut harness.peer)).await;
        let fence = fence.unwrap();
        assert!(harness.observer.process_next().await.unwrap());
        let ActivationOutcome::SourceFree(grant) =
            harness.adapter.finish_activation(fence).unwrap()
        else {
            panic!("initial compatibility receipt cannot transfer text")
        };
        assert_eq!(grant.lineage.completeness, WordCompleteness::UnknownStart);
        assert_eq!(grant.receipt_origin, ReceiptOrigin::CompatibilityProperty);
    });
}

#[test]
fn controlled_p2p_native_marker_has_zero_get_and_ping_echo_returns_token() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness().await;
        let grant = complete_native_activation(&mut harness).await;
        assert_eq!(grant.lineage.completeness, WordCompleteness::UnknownStart);
        assert_eq!(grant.receipt_origin, ReceiptOrigin::Native);

        let begin = harness.adapter.begin_bridge_fence();
        let (fence, ()) = future::zip(begin, serve_ping_and_marker(&mut harness.peer)).await;
        let fence = fence.unwrap();
        assert!(harness.observer.process_next().await.unwrap());
        let token = harness.adapter.finish_bridge_fence(fence).unwrap();
        assert!(harness.adapter.revalidate(&token));
    });
}

#[test]
fn td121_slow_compatibility_get_does_not_hold_actual_key_callback() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(Duration::from_millis(80)).await;
        let shared = Arc::new(Mutex::new(SharedState::default()));
        let mut engine = LayIbusEngine::new_from_component(
            TARGET_PATH.to_string(),
            shared,
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        assert!(engine.live_composition_enabled());
        let focus = method_message(
            DISPATCH_SENDER,
            610,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "FocusIn",
        );
        harness.peer.connection.send(&focus).await.unwrap();
        let activation = async {
            let (started, observed) = future::zip(
                engine.activate_context_from_header(&focus.header(), Instant::now(), None),
                harness.observer.process_next(),
            )
            .await;
            assert!(observed.unwrap());
            assert!(started);
        };
        assert!(
            future::race(
                async {
                    activation.await;
                    true
                },
                async {
                    async_io::Timer::after(Duration::from_millis(25)).await;
                    false
                },
            )
            .await
        );

        let held_get = next_peer_message(&mut harness.peer).await;
        assert_eq!(held_get.header().member().unwrap().as_str(), "Get");

        let key = method_message(
            DISPATCH_SENDER,
            611,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "ProcessKeyEventAtomicV1",
        );
        harness.peer.connection.send(&key).await.unwrap();
        let key_callback_completed = future::race(
            async {
                let (proposal, observed) = future::zip(
                    crate::window_interaction::WindowInteraction::process_atomic_key(
                        &mut engine,
                        &key.header(),
                        u32::from(b'a'),
                        38,
                        0,
                        (1, 2, 12, 3, 5, 13, vec![8; 32]),
                        td120_test_atomic_capability(),
                        (0, 0, Vec::new()),
                    ),
                    harness.observer.process_next(),
                )
                .await;
                assert!(observed.unwrap());
                assert_eq!(proposal.unwrap(), (PROPOSAL_NATIVE_UNHANDLED, Vec::new()));
                true
            },
            async {
                async_io::Timer::after(Duration::from_millis(25)).await;
                false
            },
        )
        .await;
        assert!(
            key_callback_completed,
            "the key callback must not wait for the held CurrentInputContext Get"
        );
        assert!(!engine.context_word_is_known());

        let value = OwnedValue::from(ObjectPath::try_from(CONTEXT_PATH).unwrap());
        harness
            .peer
            .connection
            .reply(&held_get.header(), &value)
            .await
            .unwrap();
        forward_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
        async_io::Timer::after(Duration::from_millis(90)).await;

        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(SOURCE_PATH))
            .unwrap()
            .is_none());
        let outcome = harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .expect("matching target consumes the ready result");
        let ActivationOutcome::SourceFree(grant) = outcome else {
            panic!("initial slow compatibility acquisition is source-free")
        };
        assert_eq!(grant.lineage.completeness, WordCompleteness::UnknownStart);
        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .is_none());
    });
}

#[test]
fn td121_legacy_letters_and_space_settle_exactly_before_held_get_release() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(Duration::from_millis(80)).await;
        let shared = Arc::new(Mutex::new(SharedState::default()));
        let mut engine = LayIbusEngine::new_from_component(
            TARGET_PATH.to_string(),
            shared,
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        let focus = method_message(
            DISPATCH_SENDER,
            620,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "FocusIn",
        );
        harness.peer.connection.send(&focus).await.unwrap();
        let (started, observed) = future::zip(
            engine.activate_context_from_header(&focus.header(), Instant::now(), None),
            harness.observer.process_next(),
        )
        .await;
        assert!(observed.unwrap());
        assert!(started);
        let held_get = next_peer_message(&mut harness.peer).await;
        assert_eq!(held_get.header().member().unwrap().as_str(), "Get");

        let completed_before_get_budget = future::race(
            async {
                let mut managed_literal = String::new();
                for (serial, keyval, keycode) in [
                    (621, u32::from(b'a'), 30),
                    (622, u32::from(b'b'), 48),
                    (623, crate::protocol::KEY_SPACE, 57),
                ] {
                    let key = method_message(
                        DISPATCH_SENDER,
                        serial,
                        TARGET_PATH,
                        ENGINE_INTERFACE,
                        "ProcessKeyEvent",
                    );
                    harness.peer.connection.send(&key).await.unwrap();
                    let emitter =
                        zbus::object_server::SignalEmitter::new(&harness.connection, TARGET_PATH)
                            .unwrap();
                    let (handled, observed) = future::zip(
                        engine.process_key_event(key.header(), emitter, keyval, keycode, 0),
                        harness.observer.process_next(),
                    )
                    .await;
                    assert!(observed.unwrap());
                    let handled = handled.unwrap();
                    let expected = char::from_u32(keyval).unwrap().to_string();
                    assert_exact_literal_delivery(
                        &harness.connection,
                        &mut harness.peer,
                        TARGET_PATH,
                        handled,
                        &expected,
                    )
                    .await;
                    if handled {
                        managed_literal.push_str(&expected);
                    }
                    assert_eq!(engine.committed_tail.buffer, managed_literal);
                }
                true
            },
            async {
                async_io::Timer::after(Duration::from_millis(25)).await;
                false
            },
        )
        .await;
        assert!(completed_before_get_budget);
        assert!(engine.composition.buffer.is_empty());

        let value = OwnedValue::from(ObjectPath::try_from(CONTEXT_PATH).unwrap());
        harness
            .peer
            .connection
            .reply(&held_get.header(), &value)
            .await
            .unwrap();
        forward_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
    });
}

#[test]
fn delayed_focus_in_id_enriches_pending_compatibility_request_and_cancels_get() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness().await;
        let mut engine = LayIbusEngine::new_from_component(
            TARGET_PATH.to_string(),
            Arc::new(Mutex::new(SharedState::default())),
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        let focus = method_message(
            DISPATCH_SENDER,
            680,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "FocusIn",
        );
        harness.peer.connection.send(&focus).await.unwrap();
        let (started, observed) = future::zip(
            engine.activate_context_from_header(&focus.header(), Instant::now(), None),
            harness.observer.process_next(),
        )
        .await;
        assert!(observed.unwrap());
        assert!(started);
        let held_get = next_peer_message(&mut harness.peer).await;
        assert_eq!(held_get.header().member().unwrap().as_str(), "Get");
        let request_before = harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .request
            .as_ref()
            .unwrap()
            .generation;

        let delayed = method_message(
            DISPATCH_SENDER,
            681,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "FocusInId",
        );
        harness.peer.connection.send(&delayed).await.unwrap();
        let (enriched, observed) = future::zip(
            engine.activate_context_from_header(
                &delayed.header(),
                Instant::now(),
                Some(CONTEXT_PATH),
            ),
            harness.observer.process_next(),
        )
        .await;
        assert!(observed.unwrap());
        assert!(enriched);
        {
            let reducer = harness.adapter.shared.reducer.lock().unwrap();
            let request = reducer.request.as_ref().unwrap();
            assert_eq!(request.generation, request_before);
            assert_eq!(request.origin, ReceiptOrigin::Native);
            assert_eq!(request.reply.as_ref().unwrap().0, context(CONTEXT_PATH));
        }

        forward_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
        let outcome = harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .expect("the enriched original request produces one ready outcome");
        assert!(engine.install_context_activation(outcome));
        assert!(engine.live_context_token().is_some());
        assert!(!engine.context_word_is_known());
    });
}

#[test]
fn delayed_focus_in_id_consumes_completed_compatibility_activation_without_rearm() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness().await;
        let mut engine = LayIbusEngine::new_from_component(
            TARGET_PATH.to_string(),
            Arc::new(Mutex::new(SharedState::default())),
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        let focus = method_message(
            DISPATCH_SENDER,
            683,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "FocusIn",
        );
        harness.peer.connection.send(&focus).await.unwrap();
        let (started, observed) = future::zip(
            engine.activate_context_from_header(&focus.header(), Instant::now(), None),
            harness.observer.process_next(),
        )
        .await;
        assert!(observed.unwrap());
        assert!(started);
        let get = next_peer_message(&mut harness.peer).await;
        let value = OwnedValue::from(ObjectPath::try_from(CONTEXT_PATH).unwrap());
        harness
            .peer
            .connection
            .reply(&get.header(), &value)
            .await
            .unwrap();
        forward_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
        let ready_owner = harness
            .adapter
            .shared
            .ready_activation
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .owner
            .clone();

        let delayed = method_message(
            DISPATCH_SENDER,
            684,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "FocusInId",
        );
        harness.peer.connection.send(&delayed).await.unwrap();
        let (enriched, observed) = future::zip(
            engine.activate_context_from_header(
                &delayed.header(),
                Instant::now(),
                Some(CONTEXT_PATH),
            ),
            harness.observer.process_next(),
        )
        .await;
        assert!(observed.unwrap());
        assert!(enriched);
        assert_eq!(engine.context_owner.as_ref(), Some(&ready_owner));
        assert!(engine.live_context_token().is_some());
        assert!(!engine.context_word_is_known());
        assert!(harness
            .adapter
            .shared
            .ready_activation
            .lock()
            .unwrap()
            .is_none());
        assert!(harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .request
            .is_none());
    });
}

#[test]
fn td121_marker_ready_owner_survives_deadline_until_first_atomic_key() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(Duration::from_millis(20)).await;
        let shared = Arc::new(Mutex::new(SharedState::default()));
        let mut engine = LayIbusEngine::new_from_component(
            TARGET_PATH.to_string(),
            shared,
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        assert!(engine.live_composition_enabled());
        harness
            .adapter
            .start_native_activation(
                engine_path(TARGET_PATH),
                context(CONTEXT_PATH),
                Default::default(),
            )
            .unwrap();
        forward_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
        async_io::Timer::after(Duration::from_millis(30)).await;

        let key = method_message(
            DISPATCH_SENDER,
            612,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "ProcessKeyEventAtomicV1",
        );
        harness.peer.connection.send(&key).await.unwrap();
        let (proposal, observed) = future::zip(
            crate::window_interaction::WindowInteraction::process_atomic_key(
                &mut engine,
                &key.header(),
                u32::from(b'a'),
                38,
                0,
                (2, 3, 12, 4, 6, 13, vec![8; 32]),
                td120_test_atomic_capability(),
                (0, 0, Vec::new()),
            ),
            harness.observer.process_next(),
        )
        .await;
        assert!(observed.unwrap());
        assert_eq!(proposal.unwrap().0, PROPOSAL_FRAME_READY);
        assert!(engine.live_context_token().is_some());
        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .is_none());
    });
}

#[test]
fn td121_marker_ready_owner_is_installed_before_focus_out_and_disable() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness().await;
        let shared = Arc::new(Mutex::new(SharedState::default()));
        let old_tail_epoch = 41;
        shared.lock().unwrap().handoff_tail_epoch = old_tail_epoch;
        let mut engine = LayIbusEngine::new_from_component(
            TARGET_PATH.to_string(),
            shared.clone(),
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        harness
            .adapter
            .start_native_activation(
                engine_path(TARGET_PATH),
                context(CONTEXT_PATH),
                Default::default(),
            )
            .unwrap();
        forward_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
        assert!(engine.context_owner.is_none());

        let focus_out = method_message(
            DISPATCH_SENDER,
            613,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "FocusOut",
        );
        harness.peer.connection.send(&focus_out).await.unwrap();
        let (accepted, observed) = future::zip(
            engine.observe_context_focus_out(&focus_out.header(), Instant::now()),
            harness.observer.process_next(),
        )
        .await;
        assert!(observed.unwrap());
        assert!(accepted);
        let installed_tail_epoch = engine.committed_tail.epoch;
        assert!(installed_tail_epoch > old_tail_epoch);
        assert_eq!(
            shared.lock().unwrap().handoff_tail_epoch,
            installed_tail_epoch
        );

        let disable = method_message(
            DISPATCH_SENDER,
            614,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "Disable",
        );
        harness.peer.connection.send(&disable).await.unwrap();
        let (accepted, observed) = future::zip(
            engine.observe_context_disable(&disable.header(), Instant::now()),
            harness.observer.process_next(),
        )
        .await;
        assert!(observed.unwrap());
        assert!(accepted);
        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .is_none());

        engine.committed_tail.buffer = "ч".to_string();
        engine.set_client_capabilities(1 << 5);
        engine.client_context.surrounding_text_snapshot =
            Some(SurroundingTextSnapshot::new("ч".to_string(), 1, 1));
        {
            let mut state = shared.lock().unwrap();
            state.handoff_tail_buffer = "ч".to_string();
            state.preserve_active_path_until = Some(Instant::now() + Duration::from_secs(1));
            state.exact_manual_toggle_handoff_epoch = Some(old_tail_epoch);
            state.exact_manual_toggle_handoff_path = Some(TARGET_PATH.to_string());
        }
        assert!(!engine.arm_exact_manual_toggle_autocorrect_suppression(
            "ч",
            old_tail_epoch,
            TARGET_PATH,
            false,
        ));
        shared.lock().unwrap().exact_manual_toggle_handoff_epoch = Some(installed_tail_epoch);
        assert!(engine.arm_exact_manual_toggle_autocorrect_suppression(
            "ч",
            installed_tail_epoch,
            TARGET_PATH,
            false,
        ));
    });
}

#[test]
fn td121_expired_background_activation_clears_for_next_target_bound_fence() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(Duration::from_millis(20)).await;
        harness
            .adapter
            .start_native_activation(
                engine_path(TARGET_PATH),
                context(CONTEXT_PATH),
                Default::default(),
            )
            .unwrap();
        let expired_marker = next_peer_message(&mut harness.peer).await;
        assert_eq!(
            expired_marker.header().member().unwrap().as_str(),
            MARKER_MEMBER
        );
        async_io::Timer::after(Duration::from_millis(30)).await;
        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .is_none());

        harness
            .adapter
            .start_native_activation(
                engine_path(TARGET_PATH),
                context(CONTEXT_PATH),
                Default::default(),
            )
            .unwrap();
        forward_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
        assert!(matches!(
            harness
                .adapter
                .try_finish_activation_for(&engine_path(TARGET_PATH))
                .unwrap(),
            Some(ActivationOutcome::SourceFree(_))
        ));
    });
}

#[test]
fn td121_marker_before_final_source_settlement_promotes_after_deadline() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(Duration::from_millis(20)).await;
        let source = complete_native_activation(&mut harness).await;

        let factory = Message::method_call("/org/freedesktop/IBus/Factory", "CreateEngine")
            .unwrap()
            .interface(FACTORY_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(620).unwrap())
            .build(&"lay-us")
            .unwrap();
        harness.peer.connection.send(&factory).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());
        let factory_callback = harness
            .adapter
            .begin_factory_callback(&factory.header(), Instant::now(), profile("lay-us"))
            .await
            .unwrap();
        assert!(harness
            .adapter
            .bind_factory_target(&factory_callback, engine_path(TARGET_PATH)));

        let source_key = method_message(
            DISPATCH_SENDER,
            621,
            SOURCE_PATH,
            ENGINE_INTERFACE,
            "ProcessKeyEvent",
        );
        harness.peer.connection.send(&source_key).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());

        let focus_out = method_message(
            DISPATCH_SENDER,
            622,
            SOURCE_PATH,
            ENGINE_INTERFACE,
            "FocusOut",
        );
        harness.peer.connection.send(&focus_out).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());
        let observed_focus_out = harness
            .adapter
            .observe_callback(&focus_out.header(), Instant::now())
            .await
            .unwrap();
        assert!(harness
            .adapter
            .focus_out(&source.target_owner, &observed_focus_out));

        let disable = method_message(
            DISPATCH_SENDER,
            623,
            SOURCE_PATH,
            ENGINE_INTERFACE,
            "Disable",
        );
        harness.peer.connection.send(&disable).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());
        let observed_disable = harness
            .adapter
            .observe_callback(&disable.header(), Instant::now())
            .await
            .unwrap();
        assert!(harness
            .adapter
            .disable(&source.target_owner, &observed_disable));

        let focus_in = method_message(
            DISPATCH_SENDER,
            624,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "FocusInId",
        );
        harness.peer.connection.send(&focus_in).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());
        let observed_focus_in = harness
            .adapter
            .observe_callback(&focus_in.header(), Instant::now())
            .await
            .unwrap();
        harness
            .adapter
            .start_native_activation(
                engine_path(TARGET_PATH),
                context(CONTEXT_PATH),
                observed_focus_in.position,
            )
            .unwrap();
        forward_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
        async_io::Timer::after(Duration::from_millis(30)).await;
        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .is_none());

        let source_key_callback = harness
            .adapter
            .begin_key_callback(&source.target_owner, &source_key.header(), Instant::now())
            .await
            .unwrap();
        let scope = WordScope::new(source.lineage);
        assert!(harness.adapter.settle_key_callback(
            &source_key_callback,
            SettledWordState::from_scope(0, &scope),
        ));
        assert!(harness
            .adapter
            .seal_source(&source.target_owner, 0, &observed_focus_out));

        let ActivationOutcome::Transfer(transfer) = harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .expect("marker-complete request promotes after final settlement")
        else {
            panic!("factory handoff produces a transfer")
        };
        assert_eq!(transfer.target_owner.path.as_str(), TARGET_PATH);
    });
}

#[test]
fn controlled_p2p_callback_rendezvous_stamps_factory_before_path_binding() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness().await;
        complete_native_activation(&mut harness).await;
        let mut capture = MessageStream::for_match_rule(
            method_rule(FACTORY_INTERFACE, "CreateEngine").unwrap(),
            &harness.connection,
            Some(4),
        )
        .await
        .unwrap();
        let call = Message::method_call("/org/freedesktop/IBus/Factory", "CreateEngine")
            .unwrap()
            .interface(FACTORY_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(700).unwrap())
            .build(&"lay-us")
            .unwrap();
        harness.peer.connection.send(&call).await.unwrap();
        let captured = next_ordered_message(Pin::new(&mut capture))
            .await
            .unwrap()
            .unwrap();
        let header = captured.header();
        let begin =
            harness
                .adapter
                .begin_factory_callback(&header, Instant::now(), profile("lay-us"));
        let (callback, observed) = future::zip(begin, harness.observer.process_next()).await;
        assert!(observed.unwrap());
        let callback = callback.unwrap();
        assert_eq!(callback.position, captured.recv_position());
        {
            let reducer = harness.adapter.shared.reducer.lock().unwrap();
            let ticket = reducer.ticket.as_ref().unwrap();
            assert_eq!(ticket.id, callback.ticket);
            assert_eq!(ticket.factory_position, captured.recv_position());
            assert!(ticket.target_path.is_none());
        }
        assert!(harness
            .adapter
            .bind_factory_target(&callback, engine_path(TARGET_PATH)));
        assert_eq!(
            harness
                .adapter
                .shared
                .reducer
                .lock()
                .unwrap()
                .ticket
                .as_ref()
                .unwrap()
                .target_path
                .as_ref()
                .unwrap()
                .as_str(),
            TARGET_PATH
        );
    });
}

#[test]
fn foreign_bootstrap_factory_focus_waits_for_ordered_matching_lay_evidence() {
    zbus::block_on(async {
        let mut harness =
            bootstrap_harness_with_profile_and_budget("foreign-ime", Duration::from_millis(250))
                .await;
        assert_eq!(
            harness.identity.profile,
            GlobalProfile::Foreign(profile("foreign-ime"))
        );
        let call = Message::method_call("/org/freedesktop/IBus/Factory", "CreateEngine")
            .unwrap()
            .interface(FACTORY_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(705).unwrap())
            .build(&"lay-us")
            .unwrap();
        harness.peer.connection.send(&call).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());
        let factory = harness
            .adapter
            .begin_factory_callback(&call.header(), Instant::now(), profile("lay-us"))
            .await
            .unwrap();
        assert!(harness
            .adapter
            .bind_factory_target(&factory, engine_path(TARGET_PATH),));

        let focus = method_message(
            DISPATCH_SENDER,
            706,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "FocusInId",
        );
        harness.peer.connection.send(&focus).await.unwrap();
        let shared = Arc::new(Mutex::new(SharedState::default()));
        let mut engine = LayIbusEngine::new_from_component(
            TARGET_PATH.to_string(),
            shared,
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        let (started, observed) = future::zip(
            engine.activate_context_from_header(
                &focus.header(),
                Instant::now(),
                Some(CONTEXT_PATH),
            ),
            harness.observer.process_next(),
        )
        .await;
        assert!(observed.unwrap());
        assert!(started);
        forward_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .is_none());

        let signal = Message::signal(IBUS_PATH, IBUS_INTERFACE, "GlobalEngineChanged")
            .unwrap()
            .sender(IBUS_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(707).unwrap())
            .build(&"lay-us")
            .unwrap();
        harness.peer.connection.send(&signal).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());
        let ActivationOutcome::SourceFree(grant) = harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .expect("matching ordered Lay evidence promotes the reserved target")
        else {
            panic!("foreign bootstrap has no transferable source")
        };
        assert_eq!(grant.lineage.completeness, WordCompleteness::UnknownStart);
        assert_eq!(grant.target_owner.path.as_str(), TARGET_PATH);
        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .is_none());
    });
}

#[test]
fn foreign_bootstrap_factory_focus_final_foreign_profile_is_hard_negative() {
    zbus::block_on(async {
        let mut harness =
            bootstrap_harness_with_profile_and_budget("foreign-ime", Duration::from_millis(250))
                .await;
        let call = Message::method_call("/org/freedesktop/IBus/Factory", "CreateEngine")
            .unwrap()
            .interface(FACTORY_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(708).unwrap())
            .build(&"lay-us")
            .unwrap();
        harness.peer.connection.send(&call).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());
        let factory = harness
            .adapter
            .begin_factory_callback(&call.header(), Instant::now(), profile("lay-us"))
            .await
            .unwrap();
        assert!(harness
            .adapter
            .bind_factory_target(&factory, engine_path(TARGET_PATH),));

        let focus = method_message(
            DISPATCH_SENDER,
            709,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "FocusInId",
        );
        harness.peer.connection.send(&focus).await.unwrap();
        let observed = harness.observer.process_next().await.unwrap();
        assert!(observed);
        let focus_callback = harness
            .adapter
            .observe_callback(&focus.header(), Instant::now())
            .await
            .unwrap();
        harness
            .adapter
            .start_native_activation(
                engine_path(TARGET_PATH),
                context(CONTEXT_PATH),
                focus_callback.position,
            )
            .unwrap();
        forward_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());

        let signal = Message::signal(IBUS_PATH, IBUS_INTERFACE, "GlobalEngineChanged")
            .unwrap()
            .sender(IBUS_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(710).unwrap())
            .build(&"foreign-final")
            .unwrap();
        harness.peer.connection.send(&signal).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());
        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .is_none());
        assert!(harness.adapter.current_owner().is_none());
    });
}

#[test]
fn matching_lay_signal_does_not_discard_received_unsettled_key() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness().await;
        let grant = complete_native_activation(&mut harness).await;
        let key = method_message(
            DISPATCH_SENDER,
            706,
            SOURCE_PATH,
            ENGINE_INTERFACE,
            "ProcessKeyEvent",
        );
        harness.peer.connection.send(&key).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());

        let signal = Message::signal(IBUS_PATH, IBUS_INTERFACE, "GlobalEngineChanged")
            .unwrap()
            .sender(IBUS_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(707).unwrap())
            .build(&"lay-us")
            .unwrap();
        harness.peer.connection.send(&signal).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());

        let callback = harness
            .adapter
            .begin_key_callback(&grant.target_owner, &key.header(), Instant::now())
            .await
            .unwrap();
        let scope = WordScope::new(grant.lineage);
        assert!(harness
            .adapter
            .settle_key_callback(&callback, SettledWordState::from_scope(0, &scope),));
    });
}

#[test]
fn receive_order_key_blocks_focus_out_seal_until_handler_settlement() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness().await;
        let grant = complete_native_activation(&mut harness).await;
        let owner = grant.target_owner;

        let factory = Message::method_call("/org/freedesktop/IBus/Factory", "CreateEngine")
            .unwrap()
            .interface(FACTORY_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(710).unwrap())
            .build(&"lay-us")
            .unwrap();
        harness.peer.connection.send(&factory).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());

        let key = method_message(
            DISPATCH_SENDER,
            711,
            SOURCE_PATH,
            ENGINE_INTERFACE,
            "ProcessKeyEvent",
        );
        harness.peer.connection.send(&key).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());

        let focus_out = method_message(
            DISPATCH_SENDER,
            712,
            SOURCE_PATH,
            ENGINE_INTERFACE,
            "FocusOut",
        );
        harness.peer.connection.send(&focus_out).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());
        let observed_focus = harness
            .adapter
            .observe_callback(&focus_out.header(), Instant::now())
            .await
            .unwrap();
        assert!(harness.adapter.focus_out(&owner, &observed_focus));
        assert!(
            !harness.adapter.seal_source(&owner, 0, &observed_focus),
            "the earlier received key is unsettled even though its handler has not entered"
        );

        let key_callback = harness
            .adapter
            .begin_key_callback(&owner, &key.header(), Instant::now())
            .await
            .unwrap();
        let scope = WordScope::new(grant.lineage);
        assert!(harness
            .adapter
            .settle_key_callback(&key_callback, SettledWordState::from_scope(0, &scope),));
        assert!(harness.adapter.seal_source(&owner, 0, &observed_focus));
    });
}

#[test]
fn delayed_focus_out_id_for_old_context_does_not_touch_current_owner() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness().await;
        let grant = complete_native_activation(&mut harness).await;
        let token = harness.adapter.current_token().unwrap();
        let stale = Message::method_call(SOURCE_PATH, "FocusOutId")
            .unwrap()
            .interface(ENGINE_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(720).unwrap())
            .build(&"/org/freedesktop/IBus/InputContext_old")
            .unwrap();
        harness.peer.connection.send(&stale).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());

        let observed = harness
            .adapter
            .observe_callback(&stale.header(), Instant::now())
            .await
            .unwrap();
        assert!(harness.adapter.callback_is_stale(&observed));
        assert!(!harness.adapter.focus_out(&grant.target_owner, &observed));
        assert!(harness.adapter.revalidate(&token));
        assert!(harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .ticket
            .is_none());
    });
}

#[test]
fn delayed_reset_handler_cannot_revoke_owner_installed_after_ingress() {
    zbus::block_on(before_deadline(
        async {
            let mut harness = bootstrap_harness().await;
            let first = complete_native_activation(&mut harness).await;
            let reset =
                method_message(DISPATCH_SENDER, 730, SOURCE_PATH, ENGINE_INTERFACE, "Reset");
            harness.peer.connection.send(&reset).await.unwrap();
            assert!(harness.observer.process_next().await.unwrap());

            let begin = harness.adapter.begin_native_activation(
                // An authenticated full Reset permits fresh, empty recovery;
                // its delayed callback must not revoke the replacement owner.
                engine_path(TARGET_PATH),
                context("/org/freedesktop/IBus/InputContext_2"),
                Default::default(),
            );
            let (fence, ()) = future::zip(begin, forward_marker(&mut harness.peer)).await;
            let fence = fence.unwrap();
            assert!(harness.observer.process_next().await.unwrap());
            let ActivationOutcome::SourceFree(second) =
                harness.adapter.finish_activation(fence).unwrap()
            else {
                panic!("post-reset activation is source-free")
            };
            assert_ne!(first.target_owner, second.target_owner);
            let token = harness.adapter.current_token().unwrap();

            let observed_reset = harness
                .adapter
                .observe_callback(&reset.header(), Instant::now())
                .await
                .unwrap();
            assert!(harness
                .adapter
                .callback_is_stale_for(&second.target_owner, &observed_reset));
            assert!(!harness
                .adapter
                .revocation_matches(&second.target_owner, &observed_reset));
            assert_eq!(harness.adapter.current_owner(), Some(second.target_owner));
            assert!(harness.adapter.revalidate(&token));
            Ok(())
        },
        Instant::now() + Duration::from_secs(3),
    ))
    .expect("bounded delayed-Reset owner replacement");
}

#[test]
fn controlled_p2p_observer_cancellation_wakes_and_drops_cleanly() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness().await;
        let cancel = async {
            async_io::Timer::after(Duration::from_millis(1)).await;
            harness.adapter.cancel_observer();
        };
        let (result, ()) = future::zip(harness.observer.process_next(), cancel).await;
        assert!(matches!(result, Err(AdapterError::Cancelled)));
        drop(harness.observer);
    });
}

#[test]
fn td121_pending_acquisition_owner_loss_rejects_late_completion_and_fresh_literal_is_exact() {
    zbus::block_on(async {
        let mut old = bootstrap_harness().await;
        old.adapter
            .start_native_activation(
                engine_path(TARGET_PATH),
                context(CONTEXT_PATH),
                Default::default(),
            )
            .unwrap();
        let outbound_marker = next_peer_message(&mut old.peer).await;
        assert_eq!(
            outbound_marker.header().member().unwrap().as_str(),
            MARKER_MEMBER
        );
        let old_nonce = BarrierNonce(outbound_marker.body().deserialize::<u64>().unwrap());
        assert_eq!(
            old.adapter
                .shared
                .pending
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .nonce,
            old_nonce
        );
        let owner_loss = Message::signal(DBUS_PATH, DBUS_INTERFACE, "NameOwnerChanged")
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .build(&(IBUS_NAME, IBUS_SENDER, ""))
            .unwrap();
        old.peer.connection.send(&owner_loss).await.unwrap();
        assert!(old.observer.process_next().await.unwrap());
        assert!(old.adapter.shared.pending.lock().unwrap().is_none());
        assert!(old
            .adapter
            .shared
            .ready_activation
            .lock()
            .unwrap()
            .is_none());

        let late_marker = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
            .unwrap()
            .sender(ADAPTER_SENDER)
            .unwrap()
            .build(&old_nonce.0)
            .unwrap();
        old.peer.connection.send(&late_marker).await.unwrap();
        assert!(matches!(
            old.observer.process_next().await,
            Err(AdapterError::Cancelled)
        ));
        assert!(old
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .is_none());
        assert!(old.adapter.current_owner().is_none());
        assert!(old.adapter.current_token().is_none());

        let (connection, mut peer) = controlled_pair();
        let config = AdapterConfig::new(ConnectionGeneration(61), vec![profile("lay-us")]).unwrap();
        let pending = PendingContextAdapter::subscribe(connection.clone(), config)
            .await
            .unwrap();
        let (result, ()) =
            future::zip(pending.bootstrap(), serve_bootstrap(&mut peer, "lay-us")).await;
        let (adapter, observer, identity) = result.unwrap();
        let mut fresh = Harness {
            connection,
            peer,
            adapter,
            observer,
            identity,
        };
        assert_eq!(
            fresh.adapter.shared.connection_generation,
            ConnectionGeneration(61)
        );
        let shared = Arc::new(Mutex::new(SharedState::default()));
        let mut engine = LayIbusEngine::new_from_component(
            TARGET_PATH.to_string(),
            shared,
            Some(fresh.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        let key = method_message(
            DISPATCH_SENDER,
            780,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "ProcessKeyEvent",
        );
        fresh.peer.connection.send(&key).await.unwrap();
        let emitter =
            zbus::object_server::SignalEmitter::new(&fresh.connection, TARGET_PATH).unwrap();
        let (handled, observed) = future::zip(
            engine.process_key_event(key.header(), emitter, u32::from(b'a'), 30, 0),
            fresh.observer.process_next(),
        )
        .await;
        assert!(observed.unwrap());
        let handled = handled.unwrap();
        assert_exact_literal_delivery(
            &fresh.connection,
            &mut fresh.peer,
            TARGET_PATH,
            handled,
            "a",
        )
        .await;
        assert_eq!(engine.committed_tail.buffer, if handled { "a" } else { "" });
        assert!(engine.composition.buffer.is_empty());
        assert!(old.adapter.current_owner().is_none());
        assert!(old.adapter.current_token().is_none());
    });
}

#[test]
fn controlled_p2p_bounded_callback_burst_is_drained_without_loss() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness().await;
        complete_native_activation(&mut harness).await;
        for serial in 800..832 {
            let call = method_message(
                DISPATCH_SENDER,
                serial,
                SOURCE_PATH,
                ENGINE_INTERFACE,
                "ProcessKeyEvent",
            );
            harness.peer.connection.send(&call).await.unwrap();
        }
        for _ in 0..32 {
            assert!(harness.observer.process_next().await.unwrap());
        }
        let guard = harness.observer.shared.stamps.guard().unwrap();
        for serial in [800, 831] {
            let key = HeaderKey::new(
                ConnectionGeneration(60),
                DISPATCH_SENDER,
                serial,
                SOURCE_PATH,
                "ProcessKeyEvent",
            )
            .unwrap();
            assert!(matches!(
                harness.observer.shared.stamps.begin(key, guard),
                RendezvousBegin::Ready(RendezvousOutcome::Stamp(_))
            ));
        }
    });
}
