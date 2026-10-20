//! Literal input consumed before Ready belongs to its original acquisition.
use super::*;

#[test]
fn stock_ready_without_snapshot_keeps_new_letters_in_owned_preedit_only() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_client_capabilities(41);
        engine.config.nanda_precognition = true;
        engine.config.nanda_autocorrect = false;
        assert!(engine.client_context.surrounding_text_snapshot.is_none());
        assert!(!engine.context_word_is_known());
        for (serial, ch) in "abc".chars().enumerate() {
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    162_000 + serial as u32,
                    ch as u32,
                    0,
                    0
                )
                .await
            );
        }
        assert!(engine.composition.legacy_word_preedit_active);
        assert_eq!(engine.composition.buffer, "abc");
        assert!(engine.composition.legacy_preedit_start_boundary.is_none());
        assert!(engine.client_context.managed_word_start.is_none());
        assert!(!engine.context_word_is_known());
        let effects = terminal_delivery::legacy_effects(&mut harness).await;
        assert!(!effects.iter().any(|message| matches!(
            message.header().member().map(|m| m.as_str()),
            Some("CommitText" | "DeleteSurroundingText")
        )));
        // Caret movement edits only this owned preedit. Enter commits the
        // literal once without gaining deletion authority over an unknown prefix.
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                162_004,
                crate::protocol::KEY_LEFT,
                105,
                0
            )
            .await
        );
        assert_eq!(engine.composition.buffer, "abc");
        assert_eq!(engine.composition.cursor, 2);
        assert!(
            !legacy_key(
                &mut harness,
                &mut engine,
                162_005,
                crate::protocol::KEY_ENTER,
                28,
                0
            )
            .await
        );
        let effects = terminal_delivery::legacy_effects(&mut harness).await;
        assert_eq!(
            effects
                .iter()
                .filter(|m| m
                    .header()
                    .member()
                    .is_some_and(|m| m.as_str() == "CommitText"))
                .count(),
            1
        );
        assert!(!effects.iter().any(|m| m
            .header()
            .member()
            .is_some_and(|m| m.as_str() == "DeleteSurroundingText")));
        assert!(engine.client_context.surrounding_text_snapshot.is_none());
        assert!(engine.composition.buffer.is_empty());
    }));
}

#[test]
fn stock_ready_literal_preedit_preserves_exact_and_observed_text_contracts() {
    for (exact, snapshot) in [
        (true, None),
        (
            false,
            Some(SurroundingTextSnapshot::new("abc".into(), 1, 1)),
        ),
        (
            false,
            Some(SurroundingTextSnapshot::new("abc".into(), 0, 1)),
        ),
    ] {
        zbus::block_on(bounded(async {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = new_engine(&harness);
            start_source_free_unknown(&mut harness, &mut engine).await;
            engine.set_client_capabilities(41);
            engine.client_context.exact_surrounding_refresh_available = exact;
            engine.client_context.surrounding_text_snapshot = snapshot;
            engine.config.nanda_precognition = true;
            engine.config.nanda_autocorrect = false;
            assert!(legacy_key(&mut harness, &mut engine, 162_100, b'a' as u32, 0, 0).await);
            assert!(!engine.composition.legacy_word_preedit_active);
            assert!(engine.composition.buffer.is_empty());
            let effects = terminal_delivery::legacy_effects(&mut harness).await;
            assert!(!effects.iter().any(|m| m
                .header()
                .member()
                .is_some_and(|m| m.as_str() == "DeleteSurroundingText")));
        }));
    }
}

async fn pending_preedit() -> (Harness, LayIbusEngine) {
    let harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = new_engine(&harness);
    engine.set_client_capabilities(9);
    engine.config.nanda_precognition = false;
    engine.config.nanda_autocorrect = false;
    start_source_free_pending(&harness).await;
    (harness, engine)
}

async fn first_letter(harness: &mut Harness, engine: &mut LayIbusEngine) {
    assert!(legacy_key(harness, engine, 161_100, b'p' as u32, 25, 0).await);
    assert_eq!(engine.composition.buffer, "p");
}

async fn publish_ready(harness: &mut Harness) {
    forward_marker_bounded(&mut harness.peer).await;
    assert!(harness.observer.process_next().await.unwrap());
}

async fn literal_before_context_ready(warm: bool, native: bool) {
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = new_engine(&harness);
    engine.set_client_capabilities(9);
    engine.config.nanda_precognition = false;
    engine.config.nanda_autocorrect = false;
    if warm {
        start_source_free_pending(&harness).await;
        publish_ready(&mut harness).await;
        engine.try_install_pending_context_activation();
        assert!(engine.live_context_token().is_some());
        let focus_out = method_message(
            DISPATCH_SENDER,
            161_400,
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
        assert!(
            engine
                .observe_context_focus_out(&focus_out.header(), Instant::now())
                .await
        );
    }
    let next_context = "/org/freedesktop/IBus/InputContext_2";
    let focus = if native {
        Message::method_call(TARGET_PATH, "FocusInId")
            .unwrap()
            .interface(ENGINE_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(161_401).unwrap())
            .with_flags(zbus::message::Flags::NoReplyExpected)
            .unwrap()
            .build(&(next_context, "preedit-client"))
            .unwrap()
    } else {
        method_message(
            DISPATCH_SENDER,
            161_401,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "FocusIn",
        )
    };
    send_manually_dispatched_callback(&mut harness.peer, &focus, TARGET_PATH, ENGINE_INTERFACE)
        .await;
    assert!(harness.observer.process_next().await.unwrap());
    assert!(
        engine
            .activate_context_from_header(
                &focus.header(),
                Instant::now(),
                native.then_some(next_context),
            )
            .await
    );
    assert!(engine.context_owner.is_none());
    let held = next_peer_message(&mut harness.peer).await;
    if native {
        // FocusInId already supplies the context. Hold its real marker rather
        // than inventing the property Get used only by ordinary FocusIn.
        assert_eq!(held.header().message_type(), Type::Signal);
        assert_eq!(
            held.header().interface().unwrap().as_str(),
            MARKER_INTERFACE
        );
        assert_eq!(held.header().member().unwrap().as_str(), MARKER_MEMBER);
    } else {
        assert_eq!(held.header().member().unwrap().as_str(), "Get");
        assert_eq!(
            held.body().deserialize::<(String, String)>().unwrap(),
            (IBUS_INTERFACE.into(), "CurrentInputContext".into())
        );
    }
    assert!(legacy_key(&mut harness, &mut engine, 161_402, b'p' as u32, 25, 0).await);
    assert_eq!(engine.composition.buffer, "p");
    let _ = terminal_delivery::legacy_effects(&mut harness).await;
    if native {
        let nonce = held.body().deserialize::<u64>().unwrap();
        let forwarded = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
            .unwrap()
            .sender(ADAPTER_SENDER)
            .unwrap()
            .build(&nonce)
            .unwrap();
        harness.peer.connection.send(&forwarded).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());
    } else {
        let value = OwnedValue::from(ObjectPath::try_from(next_context).unwrap());
        harness
            .peer
            .connection
            .reply(&held.header(), &value)
            .await
            .unwrap();
        publish_ready(&mut harness).await;
    }
    engine.try_install_pending_context_activation();
    assert_eq!(
        engine.composition.buffer, "p",
        "Ready discarded a consumed first letter after a held context message"
    );
    assert_eq!(engine.committed_tail.buffer, "p");
    assert!(engine.live_context_token().is_some());
    assert!(!engine.context_word_is_known());
    assert_eq!(
        harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .activation()
            .unwrap()
            .context,
        context(next_context)
    );
}

#[test]
fn c08_activation_ready_warm_compatibility_reply_preserves_first_letter() {
    zbus::block_on(bounded(literal_before_context_ready(true, false)));
}

#[test]
fn c08_activation_ready_warm_native_marker_preserves_first_letter() {
    zbus::block_on(bounded(literal_before_context_ready(true, true)));
}

#[test]
fn c08_activation_ready_cold_compatibility_reply_preserves_first_letter() {
    zbus::block_on(bounded(literal_before_context_ready(false, false)));
}

async fn ready_during_literal_rendezvous(install_before_callback: bool) {
    use std::future::Future;
    use std::task::Poll;

    let (mut harness, mut engine) = pending_preedit().await;
    let key = legacy_key_message_at(161_300, TARGET_PATH, b'p' as u32, 25, 0);
    let header = key.header();
    let admission = harness.adapter.clone();
    let mut observation = Box::pin(admission.observe_callback(&header, Instant::now()));
    std::future::poll_fn(|cx| {
        assert!(matches!(observation.as_mut().poll(cx), Poll::Pending));
        Poll::Ready(())
    })
    .await;
    assert_eq!(admission.shared.stamps.waiter_count(), 1);

    harness.peer.detached_callback_reply_serials.insert(161_300);
    harness.peer.connection.send(&key).await.unwrap();
    assert!(harness.observer.process_next().await.unwrap());
    publish_ready(&mut harness).await;
    if install_before_callback {
        engine.try_install_pending_context_activation();
        assert!(engine.context_owner.is_some());
    }
    let observed = observation
        .await
        .expect("Ready revoked an authenticated pending-literal ingress stamp");
    assert!(admission
        .pending_literal_input_receipt(TARGET_PATH, &observed)
        .is_some());
    assert!(run_received_legacy_key(&harness, &mut engine, &key, b'p' as u32, 25, 0).await);
    assert_eq!(engine.composition.buffer, "p");
    engine.try_install_pending_context_activation();
    assert_eq!(engine.composition.buffer, "p");
    assert_eq!(engine.committed_tail.buffer, "p");
    assert!(engine.live_context_token().is_some());
    assert!(!engine.context_word_is_known());
    assert!(legacy_key(&mut harness, &mut engine, 161_301, b'r' as u32, 19, 0).await);
    assert!(legacy_key(&mut harness, &mut engine, 161_302, b'i' as u32, 23, 0).await);
    assert_eq!(engine.composition.buffer, "pri");
    engine.composition.preedit_candidates = vec!["nt".into()];
    engine.composition.preedit_replacement_targets = vec![None];
    engine.composition.preedit_suffix = "nt".into();
    engine.composition.preedit_visible = true;
    engine.composition.preedit_dirty = false;
    engine.composition.preedit_display_only_pending = false;
    let _ = terminal_delivery::legacy_effects(&mut harness).await;
    assert!(legacy_key(&mut harness, &mut engine, 161_303, KEY_TAB, 15, 0).await);
    let effects = terminal_delivery::legacy_effects(&mut harness).await;
    let commits: Vec<String> = effects
        .iter()
        .filter(|message| {
            message
                .header()
                .member()
                .is_some_and(|m| m.as_str() == "CommitText")
        })
        .map(|message| {
            let body = message.body();
            let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
            crate::ibus_interface::ibus_text_value_to_string(&value).unwrap()
        })
        .collect();
    assert_eq!(commits, ["print "]);
}

#[test]
fn c08_activation_ready_ingress_race_before_install() {
    zbus::block_on(bounded(ready_during_literal_rendezvous(false)));
}

#[test]
fn c08_activation_ready_ingress_race_after_install() {
    zbus::block_on(bounded(ready_during_literal_rendezvous(true)));
}

#[test]
fn c08_activation_ready_preserves_consumed_literal_preedit() {
    zbus::block_on(bounded(async {
        let (mut harness, mut engine) = pending_preedit().await;
        assert!(legacy_key(&mut harness, &mut engine, 161_001, b'p' as u32, 25, 0).await);
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                161_002,
                b'p' as u32,
                25,
                RELEASE_MASK
            )
            .await
        );
        assert_eq!(engine.composition.buffer, "p");
        assert!(engine.composition.legacy_word_preedit_active);
        assert!(engine.context_owner.is_none());
        forward_marker_bounded(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
        engine.try_install_pending_context_activation();
        assert!(engine.live_context_token().is_some());
        assert_eq!(
            engine.composition.buffer, "p",
            "Ready discarded a consumed first letter"
        );
        assert_eq!(engine.committed_tail.buffer, "p");
        assert_eq!(engine.composition.cursor, 1);
        assert!(
            !engine.context_word_is_known(),
            "literal preservation cannot invent a boundary"
        );
        assert!(legacy_key(&mut harness, &mut engine, 161_003, b'r' as u32, 19, 0).await);
        assert!(legacy_key(&mut harness, &mut engine, 161_004, b'i' as u32, 23, 0).await);
        assert_eq!(engine.composition.buffer, "pri");
        engine.composition.preedit_candidates = vec!["nt".into()];
        engine.composition.preedit_replacement_targets = vec![None];
        engine.composition.preedit_suffix = "nt".into();
        engine.composition.preedit_visible = true;
        engine.composition.preedit_dirty = false;
        engine.composition.preedit_display_only_pending = false;
        let _ = terminal_delivery::legacy_effects(&mut harness).await;
        assert!(legacy_key(&mut harness, &mut engine, 161_005, KEY_TAB, 15, 0).await);
        let effects = terminal_delivery::legacy_effects(&mut harness).await;
        let commits: Vec<String> = effects
            .iter()
            .filter(|message| {
                message
                    .header()
                    .member()
                    .is_some_and(|member| member.as_str() == "CommitText")
            })
            .map(|message| {
                let body = message.body();
                let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
                crate::ibus_interface::ibus_text_value_to_string(&value).unwrap()
            })
            .collect();
        assert_eq!(
            commits,
            ["print "],
            "Tab must retain the entire typed prefix exactly once"
        );
    }));
}

#[test]
fn c08_activation_ready_preserves_multiple_early_letters_and_late_release() {
    zbus::block_on(bounded(async {
        let (mut harness, mut engine) = pending_preedit().await;
        assert!(legacy_key(&mut harness, &mut engine, 161_110, b'd' as u32, 32, 0).await);
        assert!(legacy_key(&mut harness, &mut engine, 161_111, b'o' as u32, 24, 0).await);
        assert_eq!(engine.composition.buffer, "do");
        publish_ready(&mut harness).await;
        engine.try_install_pending_context_activation();
        assert_eq!(engine.composition.buffer, "do");
        assert_eq!(engine.committed_tail.buffer, "do");
        assert_eq!(engine.composition.cursor, 2);
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                161_112,
                b'd' as u32,
                32,
                RELEASE_MASK
            )
            .await
        );
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                161_113,
                b'o' as u32,
                24,
                RELEASE_MASK
            )
            .await
        );
        assert_eq!(engine.composition.buffer, "do");
        assert!(!engine.context_word_is_known());
    }));
}

#[test]
fn c08_activation_ready_drops_unproven_preedit() {
    zbus::block_on(bounded(async {
        let (mut harness, mut engine) = pending_preedit().await;
        // Matching text and an ownership flag alone are never provenance.
        engine.composition.buffer = "p".into();
        engine.composition.cursor = 1;
        engine.composition.legacy_word_preedit_active = true;
        engine.composition.preedit_visible = true;
        engine.committed_tail.buffer = "p".into();
        publish_ready(&mut harness).await;
        engine.try_install_pending_context_activation();
        assert!(engine.composition.buffer.is_empty());
        assert!(engine.committed_tail.buffer.is_empty());
        assert!(!engine.context_word_is_known());
    }));
}

#[test]
fn c08_activation_ready_drops_preedit_after_focus_replacement() {
    zbus::block_on(bounded(async {
        let (mut harness, mut engine) = pending_preedit().await;
        first_letter(&mut harness, &mut engine).await;
        engine.bind_focus_receipt(
            "/org/freedesktop/IBus/InputContext_2".into(),
            "other-field".into(),
        );
        // Initial receipt binding does not itself erase this buffer. Ready
        // must reject the old local focus identity even with identical text.
        assert_eq!(engine.composition.buffer, "p");
        publish_ready(&mut harness).await;
        engine.try_install_pending_context_activation();
        assert!(engine.composition.buffer.is_empty());
        assert!(engine.committed_tail.buffer.is_empty());
    }));
}

#[test]
fn c08_activation_ready_drops_preedit_from_replaced_request() {
    zbus::block_on(bounded(async {
        let (mut harness, mut engine) = pending_preedit().await;
        first_letter(&mut harness, &mut engine).await;
        publish_ready(&mut harness).await;
        let owner = harness.adapter.current_owner().unwrap();
        assert!(harness.adapter.revoke_current_owner(&owner));
        // Drain the first request's preedit signals before forwarding the
        // replacement request's marker on this deliberately ordered peer.
        let _ = terminal_delivery::legacy_effects(&mut harness).await;
        start_source_free_pending(&harness).await;
        publish_ready(&mut harness).await;
        engine.try_install_pending_context_activation();
        assert!(engine.live_context_token().is_some());
        assert!(engine.composition.buffer.is_empty());
        assert!(engine.committed_tail.buffer.is_empty());
    }));
}

#[test]
fn c08_activation_ready_reset_cannot_restore_consumed_preedit() {
    zbus::block_on(bounded(async {
        let (mut harness, mut engine) = pending_preedit().await;
        first_letter(&mut harness, &mut engine).await;
        publish_ready(&mut harness).await;
        let reset = method_message(
            DISPATCH_SENDER,
            161_120,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "Reset",
        );
        send_manually_dispatched_callback(&mut harness.peer, &reset, TARGET_PATH, ENGINE_INTERFACE)
            .await;
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        // The real receive reducer replaced Ready; do not invoke local Reset
        // first, since that could hide an invalid restore by clearing the text.
        assert_eq!(engine.composition.buffer, "p");
        engine.try_install_pending_context_activation();
        assert!(engine.live_context_token().is_some());
        assert!(engine.composition.buffer.is_empty());
        assert!(engine.committed_tail.buffer.is_empty());
        assert!(!engine.context_word_is_known());
    }));
}

#[test]
fn c08_activation_ready_sensitive_client_cannot_restore_preedit() {
    zbus::block_on(bounded(async {
        let (mut harness, mut engine) = pending_preedit().await;
        first_letter(&mut harness, &mut engine).await;
        engine.set_content_type_state(crate::window_interaction::IBUS_INPUT_PURPOSE_PASSWORD, 0);
        publish_ready(&mut harness).await;
        engine.try_install_pending_context_activation();
        assert!(engine.composition.buffer.is_empty());
        assert!(engine.committed_tail.buffer.is_empty());
        assert!(!engine.context_word_is_known());
    }));
}

#[test]
fn c08_activation_ready_without_preedit_keeps_already_committed_input() {
    zbus::block_on(bounded(async {
        let (mut harness, mut engine) = pending_preedit().await;
        engine.set_client_capabilities(8);
        assert!(legacy_key(&mut harness, &mut engine, 161_130, b'p' as u32, 25, 0).await);
        assert!(engine.composition.buffer.is_empty());
        publish_ready(&mut harness).await;
        engine.try_install_pending_context_activation();
        assert!(!legacy_key(&mut harness, &mut engine, 161_131, KEY_TAB, 15, 0).await);
        let effects = terminal_delivery::legacy_effects(&mut harness).await;
        let commits: Vec<String> = effects
            .iter()
            .filter(|message| {
                message
                    .header()
                    .member()
                    .is_some_and(|member| member.as_str() == "CommitText")
            })
            .map(|message| {
                let body = message.body();
                let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
                crate::ibus_interface::ibus_text_value_to_string(&value).unwrap()
            })
            .collect();
        assert_eq!(
            commits,
            ["p"],
            "literal delivery remains exactly once without preedit or a completion"
        );
    }));
}
