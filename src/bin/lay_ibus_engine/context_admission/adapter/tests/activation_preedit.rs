//! Literal input consumed before Ready belongs to its original acquisition.
use super::*;

#[test]
fn platform_timing_wait_preserves_first_literal_through_activation() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        engine.set_client_capabilities(9);
        engine.config.nanda_precognition = false;
        engine.config.nanda_autocorrect = false;
        let release = Arc::new(event_listener::Event::new());
        let timing_ready = release.listen();
        let focus = observed_compatibility_focus(&mut harness, 165_099).await;
        harness
            .adapter
            .start_compatibility_activation_with_deadline(
                engine_path(TARGET_PATH),
                focus.position,
                async move {
                    timing_ready.await;
                    Some(Instant::now() + CALLBACK_BUDGET)
                },
            )
            .unwrap();
        assert!(harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .request
            .is_some());
        assert!(harness.adapter.current_token().is_none());

        let key = legacy_key_message_at(165_100, TARGET_PATH, b'p' as u32, 25, 0);
        harness.peer.detached_callback_reply_serials.insert(165_100);
        harness.peer.connection.send(&key).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());
        let observed = harness
            .adapter
            .observe_callback(&key.header(), Instant::now())
            .await
            .unwrap();
        assert!(harness
            .adapter
            .pending_literal_input_receipt(TARGET_PATH, &observed)
            .is_some());
        assert!(run_received_legacy_key(&harness, &mut engine, &key, b'p' as u32, 25, 0).await);
        assert_eq!(engine.composition.buffer, "p");
        assert!(engine.context_owner.is_none());
        let effects = terminal_delivery::legacy_effects(&mut harness).await;
        assert!(effects.iter().any(|message| message
            .header()
            .member()
            .is_some_and(|member| member.as_str() == "UpdatePreeditText")));
        assert!(!effects.iter().any(|message| matches!(
            message.header().member().map(|member| member.as_str()),
            Some("CommitText" | "DeleteSurroundingText")
        )));

        release.notify(1);
        serve_current_context_and_marker(&mut harness.peer).await;
        assert!(harness.adapter.current_token().is_none());
        assert!(harness.observer.process_next().await.unwrap());
        engine.try_install_pending_context_activation();
        assert_eq!(engine.composition.buffer, "p");
        assert_eq!(engine.committed_tail.buffer, "p");
        assert!(engine.live_context_token().is_some());
        assert!(!engine.context_word_is_known());
    }));
}

#[test]
fn stock_delayed_surrounding_callback_transfers_literal_only_with_live_admission() {
    for revoked in [false, true] {
        zbus::block_on(bounded(async {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = new_engine(&harness);
            start_source_free_unknown(&mut harness, &mut engine).await;
            engine.set_client_capabilities(41);
            engine.config.nanda_precognition = true;
            engine.config.nanda_autocorrect = false;
            assert!(legacy_key(&mut harness, &mut engine, 162_500, 'п' as u32, 0, 0).await);
            let initial = terminal_delivery::legacy_effects(&mut harness).await;
            assert!(!initial.iter().any(|message| matches!(
                message.header().member().map(|member| member.as_str()),
                Some("CommitText" | "DeleteSurroundingText")
            )));
            assert_eq!(engine.composition.buffer, "п");
            let before_callback = format!(
                "known={}, manual={}, pending={}, dirty={}, visible={}, token={}, tail={:?}",
                engine.context_word_is_known(),
                engine.context_owns_active_preedit_manual_toggle(),
                engine.composition.preedit_display_only_pending,
                engine.composition.preedit_dirty,
                engine.composition.preedit_visible,
                engine.live_context_token().is_some(),
                engine.committed_tail.buffer,
            );
            if revoked {
                let owner = harness.adapter.current_owner().unwrap();
                assert!(harness.adapter.revoke_current_owner(&owner));
            }
            engine
                .set_surrounding_text(
                    zbus::object_server::SignalEmitter::new(
                        &harness.connection,
                        engine.path.clone(),
                    )
                    .unwrap(),
                    crate::text::make_ibus_text("п".into()),
                    1,
                    1,
                )
                .await
                .unwrap();
            let effects = terminal_delivery::legacy_effects(&mut harness).await;
            let commits: Vec<_> = effects
                .iter()
                .filter(|message| {
                    message
                        .header()
                        .member()
                        .is_some_and(|member| member.as_str() == "CommitText")
                })
                .map(|message| {
                    let body = message.body();
                    let text = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
                    crate::ibus_interface::ibus_text_value_to_string(&text).unwrap()
                })
                .collect();
            assert!(!effects.iter().any(|message| message
                .header()
                .member()
                .is_some_and(|member| member.as_str() == "DeleteSurroundingText")));
            if revoked {
                assert!(commits.is_empty());
                assert_eq!(engine.composition.buffer, "п");
            } else {
                assert_eq!(commits, ["п"], "{before_callback}");
                assert!(engine.composition.buffer.is_empty());
                assert!(!engine.composition.legacy_word_preedit_active);
                assert!(engine.client_context.surrounding_text_snapshot.is_none());
                assert!(!engine.exact_managed_surrounding_word_is_current());
                assert!(engine.live_context_token().is_some());
            }
        }));
    }
}

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

#[test]
fn admitted_terminal_with_stock_caps_passes_letters_without_document_preedit() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.try_install_pending_context_activation();
        engine.set_client_capabilities(41);
        engine.config.nanda_precognition = true;
        engine.config.nanda_autocorrect = false;
        engine.client_context.cursor_cell_width = 8;
        let field = engine.terminal_input_focus_receipt().unwrap();
        assert!(engine.finish_terminal_focus_probe(Some(&field), true));
        assert!(!engine.context_word_is_known());
        for (index, ch) in "abc".chars().enumerate() {
            assert!(
                !legacy_key(
                    &mut harness,
                    &mut engine,
                    164_000 + index as u32,
                    ch as u32,
                    0,
                    0
                )
                .await
            );
        }
        assert!(engine.composition.buffer.is_empty());
        assert!(!engine.composition.legacy_word_preedit_active);
        assert!(!engine.composition.preedit_visible);
        assert!(engine.client_context.surrounding_text_snapshot.is_none());
        let effects = terminal_delivery::legacy_effects(&mut harness).await;
        assert!(!effects.iter().any(|m| matches!(
            m.header().member().map(|member| member.as_str()),
            Some("CommitText" | "DeleteSurroundingText" | "UpdatePreeditText")
        )));
        assert!(
            engine.capture_observed_suffix_display_frame().is_some(),
            "native keys with a current terminal receipt must retain suffix readout"
        );
        assert!(engine.precognition_causal_counts().0 > 0);
        assert!(!engine.context_word_is_known());
        assert!(engine.capture_space_autocorrect_frame_identity().is_none());
        engine.client_context.terminal_focus_receipt = None;
        assert!(engine.capture_observed_suffix_display_frame().is_none());
    }));
}

#[test]
fn admitted_stock_terminal_unknown_suffix_tab_appends_only_visible_completion() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.try_install_pending_context_activation();
        engine.set_client_capabilities(41);
        engine.config.nanda_precognition = true;
        engine.config.nanda_autocorrect = false;
        engine.client_context.cursor_cell_width = 8;
        let field = engine.terminal_input_focus_receipt().unwrap();
        assert!(engine.finish_terminal_focus_probe(Some(&field), true));
        for (index, ch) in "pri".chars().enumerate() {
            assert!(
                !legacy_key(
                    &mut harness,
                    &mut engine,
                    164_200 + index as u32,
                    ch as u32,
                    0,
                    0
                )
                .await
            );
        }
        assert_eq!(engine.committed_tail.buffer, "pri");
        assert!(!engine.context_word_is_known());
        engine.cancel_precognition_display_generation();
        engine.composition.preedit_candidates = vec!["nt".into()];
        engine.composition.preedit_replacement_targets = vec![None];
        engine.composition.preedit_suffix = "nt".into();
        engine.composition.preedit_visible = true;
        engine.composition.preedit_dirty = false;
        engine.composition.preedit_display_only_pending = false;
        let _ = terminal_delivery::legacy_effects(&mut harness).await;
        assert!(legacy_key(&mut harness, &mut engine, 164_203, KEY_TAB, 15, 0).await);
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
        assert_eq!(commits, ["nt "]);
        assert!(!effects.iter().any(|message| message
            .header()
            .member()
            .is_some_and(|member| member.as_str() == "DeleteSurroundingText")));
        assert_eq!(engine.committed_tail.buffer, "print ");
        assert!(engine.composition.buffer.is_empty());
    }));
}

async fn late_terminal_capability(exact_refresh: bool) {
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = new_engine(&harness);
    start_source_free_unknown(&mut harness, &mut engine).await;
    engine.try_install_pending_context_activation();
    engine.set_client_capabilities(9);
    engine.config.nanda_precognition = true;
    engine.config.nanda_autocorrect = false;
    engine.client_context.cursor_cell_width = 8;
    let field = engine.terminal_input_focus_receipt().unwrap();
    assert!(engine.finish_terminal_focus_probe(Some(&field), true));
    assert!(!legacy_key(&mut harness, &mut engine, 164_100, b'a' as u32, 0, 0).await);
    assert_eq!(
        engine.composition.word_input_mode,
        Some(WordInputMode::TerminalPassthrough)
    );
    let exact_capability = if exact_refresh {
        crate::window_interaction::IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH
    } else {
        0
    };
    engine.set_client_capabilities(41 | exact_capability);
    if exact_refresh {
        assert!(!engine.uses_native_terminal_input());
        assert_eq!(
            engine.composition.word_input_mode,
            Some(WordInputMode::ManagedCommit)
        );
    } else {
        assert!(engine.uses_native_terminal_input());
        assert_eq!(
            engine.composition.word_input_mode,
            Some(WordInputMode::TerminalPassthrough)
        );
        assert!(!legacy_key(&mut harness, &mut engine, 164_101, b'b' as u32, 0, 0).await);
    }
    assert!(engine.composition.buffer.is_empty());
    assert!(!engine.composition.legacy_word_preedit_active);
    assert!(!engine.composition.preedit_visible);
    assert!(engine.client_context.surrounding_text_snapshot.is_none());
    let effects = terminal_delivery::legacy_effects(&mut harness).await;
    assert!(!effects.iter().any(|m| matches!(
        m.header().member().map(|member| member.as_str()),
        Some("CommitText" | "DeleteSurroundingText" | "UpdatePreeditText")
    )));
}

#[test]
fn admitted_terminal_late_surrounding_capability_preserves_native_input() {
    zbus::block_on(bounded(late_terminal_capability(false)));
}

#[test]
fn admitted_terminal_late_exact_capability_retains_managed_fallback() {
    zbus::block_on(bounded(late_terminal_capability(true)));
}

async fn first_letter(harness: &mut Harness, engine: &mut LayIbusEngine) {
    assert!(legacy_key(harness, engine, 161_100, b'p' as u32, 25, 0).await);
    assert_eq!(engine.composition.buffer, "p");
}

async fn publish_ready(harness: &mut Harness) {
    forward_marker_bounded(&mut harness.peer).await;
    assert!(harness.observer.process_next().await.unwrap());
}

async fn cursor_before_activation_install(ready_first: bool) {
    let (mut harness, mut engine) = pending_preedit().await;
    if ready_first {
        publish_ready(&mut harness).await;
    }
    engine.remember_hint_geometry(41, 63, 8, 17);
    assert_eq!(
        engine.current_hint_rect(),
        None,
        "pending geometry is not live"
    );
    if !ready_first {
        publish_ready(&mut harness).await;
    }
    engine.try_install_pending_context_activation();
    assert!(engine.live_context_token().is_some());
    assert_eq!(engine.current_hint_rect(), Some([41, 63, 8, 17]));
    assert!(
        !engine.context_word_is_known(),
        "display is not a word receipt"
    );
    assert!(engine.composition.buffer.is_empty());
    assert!(engine.committed_tail.buffer.is_empty());
    assert!(terminal_delivery::legacy_effects(&mut harness)
        .await
        .is_empty());
}

#[test]
fn hint_geometry_observed_before_ready_survives_exact_activation_install() {
    zbus::block_on(bounded(cursor_before_activation_install(false)));
}

#[test]
fn hint_geometry_observed_after_ready_survives_exact_activation_install() {
    zbus::block_on(bounded(cursor_before_activation_install(true)));
}

#[test]
fn hint_geometry_from_revoked_ready_is_not_rebound() {
    zbus::block_on(bounded(async {
        let (mut harness, mut engine) = pending_preedit().await;
        publish_ready(&mut harness).await;
        engine.remember_hint_geometry(41, 63, 8, 17);
        let owner = harness.adapter.current_owner().unwrap();
        assert!(harness.adapter.revoke_current_owner(&owner));
        engine.try_install_pending_context_activation();
        assert!(engine.live_context_token().is_none());
        assert_eq!(engine.current_hint_rect(), None);
    }));
}

#[test]
fn hint_geometry_from_replaced_request_is_not_rebound() {
    zbus::block_on(bounded(async {
        let (mut harness, mut engine) = pending_preedit().await;
        engine.remember_hint_geometry(41, 63, 8, 17);
        publish_ready(&mut harness).await;
        let owner = harness.adapter.current_owner().unwrap();
        assert!(harness.adapter.revoke_current_owner(&owner));
        start_source_free_pending(&harness).await;
        publish_ready(&mut harness).await;
        engine.try_install_pending_context_activation();
        assert!(engine.live_context_token().is_some());
        assert_eq!(engine.current_hint_rect(), None);
    }));
}

#[test]
fn hint_geometry_cannot_survive_local_focus_replacement() {
    zbus::block_on(bounded(async {
        let (mut harness, mut engine) = pending_preedit().await;
        engine.remember_hint_geometry(41, 63, 8, 17);
        engine.bind_focus_receipt(
            "/org/freedesktop/IBus/InputContext_2".into(),
            "other-field".into(),
        );
        publish_ready(&mut harness).await;
        engine.try_install_pending_context_activation();
        assert!(engine.live_context_token().is_some());
        assert_eq!(engine.current_hint_rect(), None);
    }));
}

async fn literal_before_context_ready(
    warm: bool,
    native: bool,
    stock_no_snapshot: bool,
    pending_text: &str,
) {
    literal_before_context_ready_with_queued_tail(
        warm,
        native,
        stock_no_snapshot,
        pending_text,
        false,
    )
    .await;
}

async fn literal_before_context_ready_with_queued_tail(
    warm: bool,
    native: bool,
    stock_no_snapshot: bool,
    pending_text: &str,
    queued_tail: bool,
) {
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
    if stock_no_snapshot {
        // Match clients which advertise SurroundingText but provide no snapshot.
        // FocusIn reloads configuration; restore only the enabled IME settings.
        engine.set_client_capabilities(41);
        engine.config.text_backend = "ime".into();
        engine.config.nanda_precognition = true;
        assert!(engine.client_context.surrounding_text_snapshot.is_none());
    }
    let mut held = next_peer_message(&mut harness.peer).await;
    if stock_no_snapshot && !native {
        assert_eq!(held.header().member().unwrap().as_str(), "Get");
        let value = OwnedValue::from(ObjectPath::try_from(next_context).unwrap());
        harness
            .peer
            .connection
            .reply(&held.header(), &value)
            .await
            .unwrap();
        held = next_peer_message(&mut harness.peer).await;
    }
    if native || stock_no_snapshot {
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
    let effects = terminal_delivery::legacy_effects(&mut harness).await;
    if stock_no_snapshot {
        assert!(engine.context_owner.is_none());
        assert!(!effects.iter().any(|message| matches!(
            message.header().member().map(|member| member.as_str()),
            Some("CommitText" | "DeleteSurroundingText")
        )));
    }
    assert!(pending_text.starts_with('p'));
    let mut queued = Vec::new();
    for (offset, ch) in pending_text.chars().skip(1).enumerate() {
        if queued_tail {
            let serial = 161_410 + offset as u32;
            let key = legacy_key_message_at(serial, TARGET_PATH, ch as u32, 0, 0);
            harness.peer.detached_callback_reply_serials.insert(serial);
            harness.peer.connection.send(&key).await.unwrap();
            assert!(harness.observer.process_next().await.unwrap());
            queued.push((key, ch as u32));
            continue;
        }
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                161_410 + offset as u32,
                ch as u32,
                0,
                0,
            )
            .await
        );
    }
    let unfinished = pending_text.rsplit(' ').next().unwrap();
    if !queued_tail {
        assert_eq!(engine.composition.buffer, unfinished);
        assert_eq!(engine.committed_tail.buffer, pending_text);
    }
    assert!(engine.context_owner.is_none());
    if native || stock_no_snapshot {
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
    if queued_tail {
        assert!(
            engine.context_owner.is_none(),
            "Ready overtook queued literal input"
        );
    }
    for (key, keyval) in queued {
        assert!(run_received_legacy_key(&harness, &mut engine, &key, keyval, 0, 0).await);
    }
    engine.try_install_pending_context_activation();
    assert_eq!(
        engine.composition.buffer, unfinished,
        "Ready discarded a consumed first letter after a held context message"
    );
    assert_eq!(
        engine.committed_tail.buffer, pending_text,
        "Ready discarded literal input committed before its acquisition marker"
    );
    assert!(engine.live_context_token().is_some());
    assert_eq!(engine.context_word_is_known(), pending_text.contains(' '));
    if pending_text.ends_with(' ') {
        assert!(engine.context_stock_owned_manual_source_is_current());
        assert_eq!(
            engine
                .context_word_scope
                .as_ref()
                .unwrap()
                .lineage()
                .observed_first_word,
            Some((0, 1)),
        );
    }
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
fn stock_pending_literal_space_callback_queued_across_ready_retains_word_boundary() {
    for native in [false, true] {
        zbus::block_on(bounded(literal_before_context_ready_with_queued_tail(
            true, native, true, "p ", true,
        )));
    }
}

#[test]
fn stock_pending_literal_continuation_queued_across_ready_retains_word_boundary() {
    for native in [false, true] {
        zbus::block_on(bounded(literal_before_context_ready_with_queued_tail(
            true, native, true, "p q", true,
        )));
    }
}

#[test]
fn c08_activation_ready_warm_compatibility_reply_preserves_first_letter() {
    zbus::block_on(bounded(literal_before_context_ready(
        true, false, false, "p",
    )));
}

#[test]
fn c08_activation_ready_warm_native_marker_preserves_first_letter() {
    zbus::block_on(bounded(literal_before_context_ready(
        true, true, false, "p",
    )));
}

#[test]
fn c08_activation_ready_cold_compatibility_reply_preserves_first_letter() {
    zbus::block_on(bounded(literal_before_context_ready(
        false, false, false, "p",
    )));
}

#[test]
fn stock_pending_marker_without_snapshot_keeps_new_letters_uncommitted() {
    for native in [false, true] {
        zbus::block_on(bounded(literal_before_context_ready(
            true, native, true, "p",
        )));
    }
}

#[test]
fn stock_pending_marker_preserves_word_committed_with_space_before_ready() {
    for native in [false, true] {
        zbus::block_on(bounded(literal_before_context_ready(
            true, native, true, "p ",
        )));
    }
}

#[test]
fn stock_pending_marker_preserves_literal_continuation_after_space_before_ready() {
    for native in [false, true] {
        zbus::block_on(bounded(literal_before_context_ready(
            true, native, true, "p q",
        )));
    }
}

#[test]
fn pending_literal_space_rejects_focus_request_reset_gap_epoch_and_sensitive_changes() {
    for change in ["focus", "request", "reset", "gap", "epoch", "sensitive"] {
        zbus::block_on(bounded(async {
            let (mut harness, mut engine) = pending_preedit().await;
            first_letter(&mut harness, &mut engine).await;
            assert!(legacy_key(&mut harness, &mut engine, 163_001, KEY_SPACE, 0, 0).await);
            assert_eq!(engine.committed_tail.buffer, "p ");
            assert!(engine.composition.buffer.is_empty());
            assert!(engine.context_owner.is_none());
            match change {
                "focus" => {
                    engine.bind_focus_receipt(
                        "/org/freedesktop/IBus/InputContext_2".into(),
                        "other-field".into(),
                    );
                    publish_ready(&mut harness).await;
                }
                "request" => {
                    publish_ready(&mut harness).await;
                    let owner = harness.adapter.current_owner().unwrap();
                    assert!(harness.adapter.revoke_current_owner(&owner));
                    // The first real marker must be consumed before this
                    // helper drains Engine signals and the next request starts.
                    let initial = terminal_delivery::legacy_effects(&mut harness).await;
                    assert_eq!(
                        initial
                            .iter()
                            .filter(|message| {
                                message
                                    .header()
                                    .member()
                                    .is_some_and(|member| member.as_str() == "CommitText")
                            })
                            .count(),
                        1
                    );
                    start_source_free_pending(&harness).await;
                    publish_ready(&mut harness).await;
                }
                "reset" => {
                    publish_ready(&mut harness).await;
                    let reset = method_message(
                        DISPATCH_SENDER,
                        163_002,
                        TARGET_PATH,
                        ENGINE_INTERFACE,
                        "Reset",
                    );
                    send_manually_dispatched_callback(
                        &mut harness.peer,
                        &reset,
                        TARGET_PATH,
                        ENGINE_INTERFACE,
                    )
                    .await;
                    assert!(harness.observer.process_next().await.unwrap());
                }
                "gap" => {
                    let _ = legacy_key(&mut harness, &mut engine, 163_003, KEY_LEFT, 0, 0).await;
                    publish_ready(&mut harness).await;
                }
                "epoch" => {
                    // Controlled unobserved mutation: matching visible text
                    // cannot substitute for uninterrupted callback provenance.
                    engine.committed_tail.epoch += 1;
                    publish_ready(&mut harness).await;
                }
                "sensitive" => {
                    engine.client_context.content_purpose = 8;
                    assert!(engine.content_is_sensitive());
                    publish_ready(&mut harness).await;
                }
                _ => unreachable!(),
            }
            engine.try_install_pending_context_activation();
            assert!(engine.committed_tail.buffer.is_empty(), "{change}");
            assert!(!engine.context_word_is_known(), "{change}");
            let effects = terminal_delivery::legacy_effects(&mut harness).await;
            assert_eq!(
                effects
                    .iter()
                    .filter(|message| {
                        message
                            .header()
                            .member()
                            .is_some_and(|member| member.as_str() == "CommitText")
                    })
                    .count(),
                usize::from(change != "request"),
                "Ready emitted a second commit: {change}"
            );
            assert!(
                !effects.iter().any(|message| {
                    message
                        .header()
                        .member()
                        .is_some_and(|member| member.as_str() == "DeleteSurroundingText")
                }),
                "Ready emitted a delete: {change}"
            );
        }));
    }
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
        assert!(
            engine.context_owner.is_none(),
            "Ready overtook its literal callback"
        );
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
fn c08_activation_ready_ingress_race_with_early_install_attempt() {
    zbus::block_on(bounded(ready_during_literal_rendezvous(true)));
}

#[test]
fn pending_literal_completion_rejects_foreign_header_receipt_and_duplicate() {
    zbus::block_on(bounded(async {
        let (mut harness, mut engine) = pending_preedit().await;
        let key = legacy_key_message_at(164_000, TARGET_PATH, b'p' as u32, 25, 0);
        harness.peer.detached_callback_reply_serials.insert(164_000);
        harness.peer.connection.send(&key).await.unwrap();
        assert!(harness.observer.process_next().await.unwrap());
        let observed = harness
            .adapter
            .observe_callback(&key.header(), Instant::now())
            .await
            .unwrap();
        let receipt = harness
            .adapter
            .pending_literal_input_receipt(TARGET_PATH, &observed)
            .unwrap();
        let foreign = legacy_key_message_at(164_001, TARGET_PATH, b'p' as u32, 25, 0);
        assert!(!harness
            .adapter
            .complete_pending_literal_callback(&foreign.header(), &receipt));
        let mut wrong = receipt.clone();
        wrong.request = RequestGeneration(next_generation(wrong.request.0));
        assert!(!harness
            .adapter
            .complete_pending_literal_callback(&key.header(), &wrong));
        publish_ready(&mut harness).await;
        engine.try_install_pending_context_activation();
        assert!(engine.context_owner.is_none());
        assert!(run_received_legacy_key(&harness, &mut engine, &key, b'p' as u32, 25, 0).await);
        assert_eq!(engine.composition.buffer, "p");
        assert_eq!(engine.committed_tail.buffer, "p");
        assert!(engine.live_context_token().is_some());
        assert!(!engine.context_word_is_known());
        assert!(!harness
            .adapter
            .complete_pending_literal_callback(&key.header(), &receipt));
    }));
}

async fn pending_literal_store_fixture(
    capacity: usize,
) -> (
    CallbackStampStore<u64>,
    HeaderKey,
    PendingLiteralInputReceipt,
) {
    let (mut harness, _) = pending_preedit().await;
    let key = legacy_key_message_at(164_100, TARGET_PATH, b'p' as u32, 25, 0);
    harness.peer.detached_callback_reply_serials.insert(164_100);
    harness.peer.connection.send(&key).await.unwrap();
    assert!(harness.observer.process_next().await.unwrap());
    let observed = harness
        .adapter
        .observe_callback(&key.header(), Instant::now())
        .await
        .unwrap();
    let receipt = harness
        .adapter
        .pending_literal_input_receipt(TARGET_PATH, &observed)
        .unwrap();
    let store = CallbackStampStore::with_capacity(
        observed.header.connection,
        observed.guard.owner,
        capacity,
    );
    assert!(store.publish(IngressStamp {
        header: observed.header.clone(),
        position: 1,
        disposition: IngressDisposition::PendingLiteral(receipt.clone()),
    }));
    (store, observed.header, receipt)
}

#[test]
fn pending_literal_capacity_eviction_cannot_fake_callback_completion() {
    zbus::block_on(bounded(async {
        let (store, key, receipt) = pending_literal_store_fixture(2).await;
        for serial in [164_101, 164_102] {
            let mut unrelated = key.clone();
            unrelated.serial = serial;
            assert!(store.publish(IngressStamp {
                header: unrelated,
                position: u64::from(serial),
                disposition: IngressDisposition::Passive,
            }));
        }
        assert!(!store.pending_literal_callbacks_completed(|r| r == &receipt));
        assert!(store.complete_pending_literal(&key, &receipt));
        assert!(store.pending_literal_callbacks_completed(|r| r == &receipt));
    }));
}

#[test]
fn pending_literal_full_store_rejects_new_metadata_without_losing_obligation() {
    zbus::block_on(bounded(async {
        let (store, key, receipt) = pending_literal_store_fixture(1).await;
        let mut extra = key.clone();
        extra.serial += 1;
        assert!(!store.publish(IngressStamp {
            header: extra,
            position: 2,
            disposition: IngressDisposition::Passive,
        }));
        assert!(!store.pending_literal_callbacks_completed(|r| r == &receipt));
        assert!(store.complete_pending_literal(&key, &receipt));
    }));
}

#[test]
fn pending_literal_release_completion_cannot_evict_unfinished_literal() {
    zbus::block_on(bounded(async {
        let (store, key, receipt) = pending_literal_store_fixture(1).await;
        let guard = store.guard().unwrap();
        let mut release_header = key.clone();
        release_header.serial += 1;
        assert!(!store.record_completed_release(
            release_header,
            (b'p' as u32, 25, RELEASE_MASK),
            guard,
        ));
        assert!(!store.pending_literal_callbacks_completed(|r| r == &receipt));
        assert!(store.complete_pending_literal(&key, &receipt));
    }));
}

#[test]
fn pending_literal_invalid_publication_cannot_fake_callback_completion() {
    zbus::block_on(bounded(async {
        for foreign_connection in [false, true] {
            let (store, key, receipt) = pending_literal_store_fixture(2).await;
            let mut invalid = key.clone();
            if foreign_connection {
                invalid.connection = ConnectionGeneration(next_generation(key.connection.0));
            }
            assert!(!store.publish(IngressStamp {
                header: invalid,
                position: 2,
                disposition: IngressDisposition::Passive,
            }));
            assert!(!store.pending_literal_callbacks_completed(|r| r == &receipt));
            assert!(store.complete_pending_literal(&key, &receipt));
            store.terminate_observer();
            assert!(!store.pending_literal_callbacks_completed(|_| true));
        }
    }));
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
