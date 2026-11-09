use std::time::Duration;

use futures_lite::future;

use super::*;
use crate::engine::SurroundingTextSnapshot;
use crate::output::TestEngineOutput;

fn pending_native() -> (LayIbusEngine, ReplyBoundFragment, zbus::Connection) {
    let (mut engine, connection) = LayIbusEngine::fragment_reply_test_fixture();
    engine.config.fragment_layout_autoflip = true;
    engine.config.auto_switch_layout = true;
    engine.committed_tail.buffer = "ghbdtn".into();
    engine.composition.buffer.clear();
    engine.composition.cursor = 0;
    let release = FragmentLetterRelease {
        keycode: 49,
        focus_serial: engine.client_context.focus_serial,
        letter: 'n',
        native: false,
    };
    let request = engine.fragment_layout_letter_request('n').unwrap();
    let pending = ReplyBoundFragment::new(&engine, release, true, request);
    assert!(pending.is_current(&engine));
    assert!(engine.fragment_native_observation_is_pending(6));
    (engine, pending, connection)
}

#[test]
fn fragment_known_native_word_readiness_preserves_an_observed_older_prefix() {
    use crate::engine::WordInputMode;
    use crate::window_interaction::ManagedCommitSnapshotFloor;

    for (local_separator, observed_separator) in [(' ', ' '), ('\u{a0}', ' '), (' ', '\u{a0}')] {
        let (mut engine, _connection) = LayIbusEngine::fragment_reply_test_fixture();
        engine.config.fragment_layout_autoflip = true;
        engine.config.auto_switch_layout = true;
        engine.set_client_capabilities(41);
        engine.composition.word_input_mode = Some(WordInputMode::ManagedCommit);
        engine.committed_tail.buffer = format!("prior{local_separator}ghbdtn");
        engine.committed_tail.last_input_at = Some(Instant::now());
        engine.client_context.managed_commit_snapshot_floor = Some(ManagedCommitSnapshotFloor {
            focus_serial: engine.client_context.focus_serial,
            runtime_owner_lease_identity: engine.client_context.runtime_owner_lease_identity,
            layout_generation: engine.layout_gesture.layout_generation,
            tail_epoch: engine.committed_tail.epoch,
            observation_revision: engine.client_context.surrounding_observation_revision,
            keycode: 49,
            reset_echo_available: false,
        });
        let actual = format!("prior{observed_separator}ghbdtn");
        let chars = actual.chars().count() as u32;
        engine.observe_document_snapshot(Some(SurroundingTextSnapshot::new(actual, chars, chars)));
        assert!(engine.context_word_is_known());
        assert!(engine.native_document_is_current());
        assert!(engine.native_managed_readout_is_current());
        assert!(engine.capture_fragment_layout_frame().is_some());
        assert!(engine.text_target_edit_route(6).can_execute());
        let before = engine.committed_tail.buffer.clone();
        let snapshot = engine.client_context.surrounding_text_snapshot.clone();
        let epoch = engine.committed_tail.epoch;
        assert!(
            !engine.fragment_native_observation_is_pending(6),
            "a fresh known current word must not wait on an older separator: {local_separator:?}->{observed_separator:?}"
        );
        assert_eq!(engine.committed_tail.buffer, before);
        assert_eq!(engine.committed_tail.epoch, epoch);
        assert_eq!(engine.client_context.surrounding_text_snapshot, snapshot);
        if local_separator != observed_separator {
            for loss in [
                "word",
                "selection",
                "receipt",
                "focus",
                "lease",
                "epoch",
                "layout",
                "token",
                "scope",
                "binding",
                "privacy",
                "sealed",
                "gnome",
            ] {
                let mut changed = engine.clone();
                match loss {
                    "word" => changed
                        .client_context
                        .surrounding_text_snapshot
                        .as_mut()
                        .unwrap()
                        .text
                        .push('x'),
                    "selection" => {
                        changed
                            .client_context
                            .surrounding_text_snapshot
                            .as_mut()
                            .unwrap()
                            .anchor_pos -= 1
                    }
                    "receipt" => {
                        changed
                            .client_context
                            .managed_commit_snapshot_floor
                            .as_mut()
                            .unwrap()
                            .observation_revision =
                            changed.client_context.surrounding_observation_revision
                    }
                    "focus" => changed.client_context.focus_serial += 1,
                    "lease" => changed.client_context.runtime_owner_lease_identity += 1,
                    "epoch" => changed.committed_tail.epoch += 1,
                    "layout" => changed.layout_gesture.layout_generation += 1,
                    "token" => changed.context_token = None,
                    "scope" => changed.context_word_scope = None,
                    "binding" => changed.client_context.native_document_binding = None,
                    "privacy" => {
                        changed.client_context.content_purpose =
                            crate::window_interaction::IBUS_INPUT_PURPOSE_PASSWORD
                    }
                    "sealed" => changed.context_handoff_sealed = true,
                    "gnome" => changed.config.layout_backend = "gnome".into(),
                    _ => unreachable!(),
                }
                assert!(changed.fragment_native_observation_is_pending(6), "{loss}");
                assert_eq!(changed.committed_tail.buffer, before, "{loss}");
            }
            // Revoke the canonical owner, rather than only removing a local
            // cached owner value while its admission token remains valid.
            engine.revoke_context_word();
            assert!(!engine.context_word_is_known());
            assert!(engine.fragment_native_observation_is_pending(6));
        }
    }
}

#[test]
fn fragment_reply_has_the_identical_bool_signature_and_body() {
    for handled in [false, true] {
        let context = zbus::zvariant::serialized::Context::new_dbus(zbus::zvariant::LE, 0);
        let expected = zbus::zvariant::to_bytes(context, &handled).unwrap();
        let (notified, _listener) = ResponseDispatchNotifier::new(handled);
        for reply in [
            KeyEventReply::Plain(handled),
            KeyEventReply::Notified(notified),
        ] {
            assert_eq!(KeyEventReply::SIGNATURE, bool::SIGNATURE);
            assert_eq!(reply.handled(), handled);
            let actual = zbus::zvariant::to_bytes(context, &reply).unwrap();
            assert_eq!(actual.bytes(), expected.bytes());
            let (decoded, _) = actual.deserialize::<bool>().unwrap();
            assert_eq!(decoded, handled);
        }
    }
}

#[test]
fn fragment_reply_metadata_cannot_survive_changed_input_or_authority() {
    for change in [
        "path",
        "owner",
        "token",
        "focus",
        "lease",
        "epoch",
        "tail",
        "layout-generation",
        "layout",
        "composition",
        "desktop",
        "binding",
        "off",
        "auto-off",
        "privacy",
        "manual",
        "callback",
        "press",
        "space",
        "aba",
    ] {
        let (mut engine, pending, _connection) = pending_native();
        match change {
            "path" => engine.path.push_str("/other"),
            "owner" => engine.context_owner = None,
            "token" => engine.context_token = None,
            "focus" => engine.client_context.focus_serial += 1,
            "lease" => engine.client_context.runtime_owner_lease_identity += 1,
            "epoch" => engine.committed_tail.epoch += 1,
            "tail" => engine.committed_tail.buffer.push('x'),
            "layout-generation" => engine.layout_gesture.layout_generation += 1,
            "layout" => engine.layout_gesture.layout_is_ru = true,
            "composition" => engine.composition.buffer.push('x'),
            "desktop" => engine.config.layout_backend = "gnome".into(),
            "binding" => engine.client_context.native_document_binding = None,
            "off" => engine.config.fragment_layout_autoflip = false,
            "auto-off" => engine.config.auto_switch_layout = false,
            "privacy" => {
                engine.client_context.content_purpose =
                    crate::window_interaction::IBUS_INPUT_PURPOSE_PASSWORD
            }
            "manual" => engine.layout_gesture.pending_manual_toggle = true,
            "callback" => engine.context_callback_entered = Some(Instant::now()),
            "press" => {
                engine.layout_gesture.handled_press_keycodes.insert(49);
            }
            "space" => {
                engine.committed_tail.buffer.push(' ');
                engine.committed_tail.epoch += 1;
            }
            "aba" => {
                engine.committed_tail.buffer.push('x');
                engine.committed_tail.buffer.pop();
                engine.committed_tail.epoch += 2;
            }
            _ => unreachable!(),
        }
        assert!(!pending.is_current(&engine), "{change}");
        let text = engine.committed_tail.buffer.clone();
        let epoch = engine.committed_tail.epoch;
        let mut observed = TestEngineOutput::default();
        zbus::block_on(
            engine.fulfill_fragment_after_reply(&mut EngineOutput::test(&mut observed), pending),
        );
        assert_eq!(engine.committed_tail.buffer, text, "{change}");
        assert_eq!(engine.committed_tail.epoch, epoch, "{change}");
        assert!(observed.committed_texts.is_empty(), "{change}");
        assert!(observed.surrounding_deletes.is_empty(), "{change}");
        assert!(observed.input_mode_updates.is_empty(), "{change}");
    }
}

#[test]
fn fragment_reply_dispatch_does_not_supply_a_document_receipt() {
    let (mut engine, pending, _connection) = pending_native();
    engine.client_context.surrounding_text_snapshot =
        Some(SurroundingTextSnapshot::new("ghbdt".into(), 5, 5));
    let text = engine.committed_tail.buffer.clone();
    let mut observed = TestEngineOutput::default();
    zbus::block_on(
        engine.fulfill_fragment_after_reply(&mut EngineOutput::test(&mut observed), pending),
    );
    assert_eq!(engine.committed_tail.buffer, text);
    assert!(observed.committed_texts.is_empty());
    assert!(observed.surrounding_deletes.is_empty());
}

#[test]
fn fragment_only_positive_pending_native_requests_defer_the_reply() {
    for control in ["positive", "off", "gnome", "no-proposal", "stale-focus"] {
        let (mut engine, mut pending, connection) = pending_native();
        match control {
            "off" => engine.config.fragment_layout_autoflip = false,
            "gnome" => engine.config.layout_backend = "gnome".into(),
            "no-proposal" => {
                engine.committed_tail.buffer = "hello".into();
                pending.release.letter = 'o';
            }
            "stale-focus" => engine.client_context.focus_serial += 1,
            "positive" => {}
            _ => unreachable!(),
        }
        let emitter = SignalEmitter::new(&connection, engine.path.clone()).unwrap();
        let reply =
            zbus::block_on(engine.fragment_letter_key_reply(&emitter, pending.release, true));
        assert!(reply.handled());
        assert_eq!(
            matches!(reply, KeyEventReply::Notified(_)),
            control == "positive",
            "{control}"
        );
        // No continuation can mutate the same engine before its key reply is
        // dispatched. The existing factory is deliberately not registered at
        // this test transport, so dropping the reply cannot invent an owner.
        assert_eq!(
            engine.committed_tail.buffer,
            if control == "no-proposal" {
                "hello"
            } else {
                "ghbdtn"
            }
        );
    }
}

#[test]
fn fragment_notifier_is_silent_until_the_bool_response_is_dispatched() {
    struct ReplyService(Option<KeyEventReply>);
    #[zbus::interface(name = "org.lay.FragmentReplyTest")]
    impl ReplyService {
        fn process(&mut self) -> KeyEventReply {
            self.0.take().unwrap()
        }
    }

    let (reply, dispatched) = ResponseDispatchNotifier::new(true);
    let mut dispatched = Box::pin(dispatched);
    assert!(zbus::block_on(future::poll_once(&mut dispatched)).is_none());
    let (server_socket, client_socket) = std::os::unix::net::UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        zbus::block_on(
            zbus::connection::Builder::unix_stream(server_socket)
                .server(zbus::Guid::generate())
                .unwrap()
                .p2p()
                .serve_at("/reply", ReplyService(Some(KeyEventReply::Notified(reply))))
                .unwrap()
                .build(),
        )
        .unwrap()
    });
    let client = zbus::block_on(
        zbus::connection::Builder::unix_stream(client_socket)
            .p2p()
            .build(),
    )
    .unwrap();
    let _server = server.join().unwrap();
    assert!(zbus::block_on(future::poll_once(&mut dispatched)).is_none());
    zbus::block_on(async {
        let observed = future::race(
            async {
                let response = client
                    .call_method(
                        None::<&str>,
                        "/reply",
                        Some("org.lay.FragmentReplyTest"),
                        "Process",
                        &(),
                    )
                    .await
                    .unwrap();
                assert_eq!(response.body().signature(), bool::SIGNATURE);
                assert!(response.body().deserialize::<bool>().unwrap());
                dispatched.await;
                true
            },
            async {
                async_io::Timer::after(Duration::from_secs(2)).await;
                false
            },
        )
        .await;
        assert!(
            observed,
            "private-bus bool response must dispatch before continuation"
        );
    });
}
