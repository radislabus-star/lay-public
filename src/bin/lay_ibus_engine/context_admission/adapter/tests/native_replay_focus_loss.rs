//! The recorded stock-IBus Reset/refocus schedule must not swallow native input.
use super::*;

async fn passive_refocus(harness: &mut Harness, engine: &mut LayIbusEngine, serial: &mut u32) {
    actual_focus_out(harness, engine, *serial).await;
    *serial += 1;
    let message = observed_callback_without_reply(*serial, &engine.path, "FocusIn");
    send_manually_dispatched_callback(&mut harness.peer, &message, &engine.path, ENGINE_INTERFACE)
        .await;
    assert!(harness.observer.process_next().await.unwrap());
    *serial += 1;
    engine.focus_in_callback(message.header()).await;
}

async fn hold_replay_acquisition_marker(harness: &mut Harness) -> Message {
    // The existing compatibility resolver can coalesce the two focus cycles.
    // Answer only its actual Gets; never manufacture a callback or a Ready.
    for _ in 0..3 {
        let message = next_peer_message(&mut harness.peer).await;
        if message.header().member().map(|m| m.as_str()) == Some("Get") {
            let value = OwnedValue::from(ObjectPath::try_from(CONTEXT_PATH).unwrap());
            harness
                .peer
                .connection
                .reply(&message.header(), &value)
                .await
                .unwrap();
            continue;
        }
        assert_eq!(
            message.header().interface().map(|m| m.as_str()),
            Some(MARKER_INTERFACE)
        );
        assert_eq!(
            message.header().member().map(|m| m.as_str()),
            Some(MARKER_MEMBER)
        );
        return message;
    }
    panic!("compatibility acquisition did not produce its bounded marker");
}

#[test]
fn native_replay_after_refocus_cannot_enter_preedit_before_a_late_reset() {
    zbus::block_on(bounded(native_replay_late_reset(false)));
}

#[test]
fn native_replay_source_free_ready_retains_delivery_without_old_word_authority() {
    zbus::block_on(bounded(native_replay_late_reset(true)));
}

async fn native_replay_late_reset(install_ready: bool) {
    let (mut harness, engine) = stock_owned_source::stock_owned_manual_fixture().await;
    let (mut engine, outcome, _) = cycle09_manual_toggle_with_effects(&mut harness, engine).await;
    assert_eq!(outcome, Ok((3, false)));
    engine.layout_gesture.layout_is_ru = false;
    let epoch = engine.committed_tail.epoch;
    let path = engine.path.clone();
    let (mut engine, armed) =
        cycle09_suppress_exact_replay(&mut harness, engine, "слово ", epoch, &path, false).await;
    assert_eq!(armed, Ok(true));
    let source = engine.live_context_token().expect("admitted replay source");
    let mut serial = 164_500;
    for state in [0, RELEASE_MASK] {
        assert!(
            !firefox_replay_callback(
                &mut harness,
                &mut engine,
                &mut serial,
                (KEY_BACKSPACE, 14, state),
            )
            .await
        );
    }
    firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
    passive_refocus(&mut harness, &mut engine, &mut serial).await;
    passive_refocus(&mut harness, &mut engine, &mut serial).await;
    engine.config.text_backend = "ime".into();
    engine.config.nanda_precognition = true;
    assert!(!harness.adapter.revalidate(&source));
    assert!(engine.live_context_token().is_none());

    let marker = hold_replay_acquisition_marker(&mut harness).await;
    if install_ready {
        let nonce = marker.body().deserialize::<u64>().unwrap();
        let forwarded = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
            .unwrap()
            .sender(ADAPTER_SENDER)
            .unwrap()
            .build(&nonce)
            .unwrap();
        harness.peer.connection.send(&forwarded).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());
        engine.try_install_pending_context_activation();
        assert!(engine.live_context_token().is_some());
        assert!(!harness.adapter.revalidate(&source));
        assert!(engine.committed_tail.buffer.is_empty());
        assert_eq!(
            engine
                .context_word_scope
                .as_ref()
                .unwrap()
                .lineage()
                .completeness,
            crate::context_admission::WordCompleteness::UnknownStart
        );
    }
    for _ in 1.."слово ".chars().count() {
        for state in [0, RELEASE_MASK] {
            assert!(
                !firefox_replay_callback(
                    &mut harness,
                    &mut engine,
                    &mut serial,
                    (KEY_BACKSPACE, 14, state),
                )
                .await
            );
        }
    }
    for (index, ch) in "ckjdj ".chars().enumerate() {
        assert!(
            !firefox_observed_key_callback(
                &mut harness,
                &mut engine,
                &mut serial,
                (ch as u32, 0, 0),
            )
            .await,
            "native replacement letter {index} must not start a cancellable composition"
        );
        assert!(engine.composition.buffer.is_empty());
        assert!(engine.committed_tail.buffer.is_empty());
        cycle09_assert_no_local_text_effect(&mut harness).await;
        if index == 2 {
            firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
        }
    }
    assert!(!harness.adapter.revalidate(&source));
    if install_ready {
        // The observed trailing Space may arm the next word. It must not
        // import the just-replayed word into the fresh empty owner.
        assert_eq!(
            engine
                .context_word_scope
                .as_ref()
                .unwrap()
                .lineage()
                .observed_suffix_chars,
            0
        );
        assert!(engine.committed_tail.buffer.is_empty());
    } else {
        assert!(engine.live_context_token().is_none());
    }
    assert!(engine.client_context.surrounding_text_snapshot.is_none());
}

#[test]
fn native_replay_delivery_after_revocation_rejects_expiry_layout_modifiers_and_order() {
    zbus::block_on(bounded(async {
        for invalid in ["expiry", "layout", "modifier", "order"] {
            let (mut harness, engine) = stock_owned_source::stock_owned_manual_fixture().await;
            let (mut engine, outcome, _) =
                cycle09_manual_toggle_with_effects(&mut harness, engine).await;
            assert_eq!(outcome, Ok((3, false)));
            engine.layout_gesture.layout_is_ru = false;
            let epoch = engine.committed_tail.epoch;
            let path = engine.path.clone();
            let (mut engine, armed) =
                cycle09_suppress_exact_replay(&mut harness, engine, "слово ", epoch, &path, false)
                    .await;
            assert_eq!(armed, Ok(true));
            let source = engine.live_context_token().unwrap();
            let mut serial = 165_000;
            assert!(
                !firefox_replay_callback(
                    &mut harness,
                    &mut engine,
                    &mut serial,
                    (KEY_BACKSPACE, 14, 0),
                )
                .await
            );
            firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
            passive_refocus(&mut harness, &mut engine, &mut serial).await;
            passive_refocus(&mut harness, &mut engine, &mut serial).await;
            engine.config.text_backend = "ime".into();
            engine.config.nanda_precognition = true;
            let _marker = hold_replay_acquisition_marker(&mut harness).await;
            if invalid == "expiry" {
                let mut shared = engine.shared.lock().unwrap();
                let Some(AutocorrectSuppression::ExactReplay(scope)) =
                    shared.autocorrect_suppression.as_mut()
                else {
                    panic!("missing replay")
                };
                scope.expires_at = std::time::Instant::now() - std::time::Duration::from_millis(1);
            } else if invalid == "layout" {
                engine.layout_gesture.layout_is_ru = true;
            }
            let keyval = if invalid == "order" {
                b'q' as u32
            } else {
                KEY_BACKSPACE
            };
            let state = if invalid == "modifier" { 1 << 2 } else { 0 };
            assert!(
                firefox_observed_key_callback(
                    &mut harness,
                    &mut engine,
                    &mut serial,
                    (keyval, 14, state),
                )
                .await,
                "invalid {invalid} must not pass as exact replay delivery"
            );
            assert!(engine
                .shared
                .lock()
                .unwrap()
                .autocorrect_suppression
                .is_none());
            assert!(!harness.adapter.revalidate(&source));
            assert!(engine.live_context_token().is_none());
            assert!(engine.composition.buffer.is_empty());
            assert!(engine.committed_tail.buffer.is_empty());
            assert!(engine.client_context.surrounding_text_snapshot.is_none());
        }
    }));
}

#[test]
fn native_replay_refocus_cycles_do_not_swallow_backspace_after_owner_revocation() {
    zbus::block_on(bounded(async {
        for invalid_binding in [None, Some("path"), Some("lease"), Some("delivery_lease")] {
            let (mut harness, engine) = stock_owned_source::stock_owned_manual_fixture().await;
            let (mut engine, outcome, _) =
                cycle09_manual_toggle_with_effects(&mut harness, engine).await;
            assert_eq!(outcome, Ok((3, false)));
            engine.layout_gesture.layout_is_ru = false;
            let epoch = engine.committed_tail.epoch;
            let path = engine.path.clone();
            let (mut engine, armed) =
                cycle09_suppress_exact_replay(&mut harness, engine, "слово ", epoch, &path, false)
                    .await;
            assert_eq!(armed, Ok(true));
            assert!(!engine.client_context.surrounding_text_callback_observed);
            let mut serial = 164_000;
            let source = engine.live_context_token().expect("admitted replay source");
            for state in [0, RELEASE_MASK] {
                assert!(
                    !firefox_replay_callback(
                        &mut harness,
                        &mut engine,
                        &mut serial,
                        (KEY_BACKSPACE, 14, state),
                    )
                    .await
                );
            }
            // Chrome's client Reset/FocusOut/FocusIn becomes two engine focus
            // cycles on stock IBus 1.5.26. No reply is invented for the pending
            // compatibility Get, and no token is installed behind the reducer.
            firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
            passive_refocus(&mut harness, &mut engine, &mut serial).await;
            passive_refocus(&mut harness, &mut engine, &mut serial).await;
            assert!(!harness.adapter.revalidate(&source));
            assert!(engine.live_context_token().is_none());
            // FocusIn reloads the persisted user configuration. This isolated
            // harness has no user config; restore the recorded enabled-IME mode,
            // without changing the reducer, owner, tail, or replay scope.
            engine.config.text_backend = "ime".into();
            engine.config.nanda_precognition = true;
            assert!(engine.live_composition_enabled());
            let marker_before_key = {
                let mut shared = engine.shared.lock().unwrap();
                if let Some(binding) = invalid_binding {
                    let Some(AutocorrectSuppression::ExactReplay(scope)) =
                        shared.autocorrect_suppression.as_mut()
                    else {
                        panic!("recorded refocus must retain the orphaned marker");
                    };
                    match binding {
                        "path" => scope.path.push_str("_foreign"),
                        "lease" => {
                            scope.owner_lease_identity = scope.owner_lease_identity.wrapping_add(1)
                        }
                        "delivery_lease" => {
                            scope.delivery_lease_identity =
                                Some(scope.owner_lease_identity.wrapping_add(1))
                        }
                        _ => unreachable!(),
                    }
                }
                shared.autocorrect_suppression.clone()
            };

            // Leave the compatibility Get and detached-dispatch replies pending.
            // They are not field evidence and must not be manufactured to admit
            // the next real key callback.
            let message = Message::method_call(engine.path.as_str(), "ProcessKeyEvent")
                .unwrap()
                .interface(ENGINE_INTERFACE)
                .unwrap()
                .sender(DISPATCH_SENDER)
                .unwrap()
                .serial(NonZeroU32::new(serial).unwrap())
                .with_flags(zbus::message::Flags::NoReplyExpected)
                .unwrap()
                .build(&(KEY_BACKSPACE, 14u32, 0u32))
                .unwrap();
            send_manually_dispatched_callback(
                &mut harness.peer,
                &message,
                &engine.path,
                ENGINE_INTERFACE,
            )
            .await;
            assert!(harness.observer.process_next().await.unwrap());
            let emitter =
                zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                    .unwrap();
            let handled = engine
                .process_key_event(message.header(), emitter, KEY_BACKSPACE, 14, 0)
                .await
                .unwrap();
            if invalid_binding.is_none() {
                assert!(
                !handled,
                "a revoked word lease must not make an orphaned replay veto eat physical Backspace"
            );
                let shared = engine.shared.lock().unwrap();
                let Some(AutocorrectSuppression::ExactReplay(after)) =
                    shared.autocorrect_suppression.as_ref()
                else {
                    panic!("unfinished replay must retain only delivery classification");
                };
                let Some(AutocorrectSuppression::ExactReplay(before)) = marker_before_key else {
                    panic!("fixture lost its replay descriptor");
                };
                assert_eq!(after.native_press_count, before.native_press_count + 1);
                assert!(after.source_token.is_none());
                assert!(after.observed_external_prefix.is_none());
                assert!(engine.committed_tail.buffer.is_empty());
            } else {
                assert!(
                    handled,
                    "a foreign replay binding must not become native passage"
                );
                assert_eq!(
                    engine.shared.lock().unwrap().autocorrect_suppression,
                    marker_before_key,
                    "an old instance must not retire another path or lease's marker"
                );
            }
            assert!(!harness.adapter.revalidate(&source));
            assert!(engine.live_context_token().is_none());
        }
    }));
}
