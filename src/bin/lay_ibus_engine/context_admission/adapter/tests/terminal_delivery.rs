//! Real legacy callback and signal contracts; native desktop delivery is a
//! separate consumer proof. No simulated keyboard encoder is used here.
use super::*;

async fn known_terminal(width: i32) -> (Harness, LayIbusEngine) {
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = new_engine(&harness);
    engine.config.auto_replace = false;
    engine.config.nanda_precognition = false;
    start_source_free_unknown(&mut harness, &mut engine).await;
    assert!(legacy_key(&mut harness, &mut engine, 20_000, KEY_SPACE, 57, 0).await);
    expect_legacy_commit(&mut harness.peer).await;
    assert!(
        legacy_key(
            &mut harness,
            &mut engine,
            20_001,
            KEY_SPACE,
            57,
            RELEASE_MASK
        )
        .await
    );
    no_legacy_output(&mut harness).await;
    assert!(engine.context_word_is_known());
    engine.set_content_type_state(10, 0);
    engine.set_client_capabilities(1 | 1 << 3);
    engine.client_context.cursor_cell_width = width;
    (harness, engine)
}

async fn armed_exact_replay(
    tail: &str,
    expected_suffix: &str,
    target_layout_is_ru: bool,
    reset_choreography: bool,
) -> (Harness, LayIbusEngine) {
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = new_engine(&harness);
    engine.config.auto_replace = false;
    engine.config.nanda_precognition = false;
    start_source_free_unknown(&mut harness, &mut engine).await;
    if !reset_choreography {
        assert!(legacy_key(&mut harness, &mut engine, 21_000, KEY_SPACE, 57, 0).await);
        expect_legacy_commit(&mut harness.peer).await;
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                21_001,
                KEY_SPACE,
                57,
                RELEASE_MASK
            )
            .await
        );
        no_legacy_output(&mut harness).await;
    }

    if reset_choreography {
        for (offset, ch) in tail.chars().enumerate() {
            let keycode = match ch {
                'g' => 34,
                'h' => 35,
                'b' => 48,
                'd' => 32,
                't' => 20,
                'n' => 49,
                _ => panic!("reset choreography fixture uses the exact ghbdtn source"),
            };
            let serial = 21_002 + offset as u32 * 2;
            assert!(legacy_key(&mut harness, &mut engine, serial, ch as u32, keycode, 0).await);
            expect_legacy_commit(&mut harness.peer).await;
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    serial + 1,
                    ch as u32,
                    keycode,
                    RELEASE_MASK,
                )
                .await
            );
            no_legacy_output(&mut harness).await;
        }
    } else {
        for ch in tail.chars() {
            engine.push_tail_char(ch);
        }
    }
    engine.set_client_capabilities(1 << 5);
    let visible_tail = engine.committed_tail.buffer.clone();
    let cursor = visible_tail.chars().count() as u32;
    if reset_choreography {
        exact_replay_surrounding_receipt(&mut harness, &mut engine, &visible_tail).await;
    } else {
        engine.client_context.surrounding_text_snapshot =
            Some(SurroundingTextSnapshot::new(visible_tail, cursor, cursor));
    }
    engine.composition.preedit_visible = true;
    engine.composition.preedit_suffix = "stale".to_string();
    engine.composition.preedit_candidates = vec!["stale".to_string()];
    engine.composition.preedit_replacement_targets = vec![Some("stale".to_string())];
    engine.composition.preedit_candidate_index = 0;
    engine.composition.preedit_display_only_pending = true;
    engine.composition.preedit_dirty = true;
    engine.composition.pending_display_frame = engine.capture_input_frame_identity();
    engine.arm_pending_ime_completion_learning(
        "seed context".to_string(),
        "seed".to_string(),
        "seeded".to_string(),
        true,
    );
    if reset_choreography {
        reset_to_unknown_start_with_one_preedit_clear(&mut harness, &mut engine).await;
        let visible_tail = engine.committed_tail.buffer.clone();
        exact_replay_surrounding_receipt(&mut harness, &mut engine, &visible_tail).await;
        assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        let emitter =
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap();
        let token = engine.live_context_token().expect("post-Reset live token");
        let mut bridge_output = engine.begin_context_bridge_output(Some(&token));
        let delegation = bridge_output
            .manual_toggle_active_text_target(&mut crate::output::EngineOutput::legacy(&emitter))
            .await
            .unwrap();
        assert_eq!(delegation, None);
        assert!(bridge_output.exact_manual_toggle_handoff_is_live());
        bridge_output.complete();
        drop(bridge_output);
        no_legacy_output(&mut harness).await;
        assert!(engine.context_reset_rereceipt.is_none());
        assert!(engine.exact_manual_toggle_handoff_is_live());
        assert!(!engine.context_word_is_known());
        engine.set_layout_is_ru(target_layout_is_ru);
    } else {
        engine.set_layout_is_ru(target_layout_is_ru);
        engine.prepare_exact_manual_toggle_layout_handoff();
    }
    let epoch = engine.committed_tail.epoch;
    assert!(engine.arm_exact_manual_toggle_autocorrect_suppression(
        expected_suffix,
        epoch,
        TARGET_PATH,
        target_layout_is_ru,
    ));
    assert!(engine.composition.preedit_suffix.is_empty());
    assert!(engine.composition.preedit_candidates.is_empty());
    assert!(engine.composition.preedit_replacement_targets.is_empty());
    assert!(engine.composition.pending_display_frame.is_none());
    assert!(engine.committed_tail.pending_completion_learning.is_none());
    (harness, engine)
}

pub(super) async fn legacy_effects(harness: &mut Harness) -> Vec<Message> {
    // FIFO transport marker proves absence without a sleep or queue polling.
    harness
        .connection
        .emit_signal(None::<&str>, TARGET_PATH, "org.lay.Proof", "Reached", &())
        .await
        .unwrap();
    bounded(async {
        let mut effects = Vec::new();
        loop {
            let next = next_peer_message(&mut harness.peer).await;
            if next
                .header()
                .interface()
                .map(|interface| interface.as_str())
                == Some("org.lay.Proof")
            {
                assert_eq!(next.header().member().unwrap().as_str(), "Reached");
                return effects;
            }
            assert_eq!(
                next.header().message_type(),
                Type::Signal,
                "unexpected peer message: {next:?}"
            );
            assert_eq!(
                next.header().interface().unwrap().as_str(),
                ENGINE_INTERFACE
            );
            effects.push(next);
        }
    })
    .await
}

async fn no_legacy_output(harness: &mut Harness) {
    assert!(legacy_effects(harness).await.is_empty());
}

async fn expect_one_empty_preedit_clear(harness: &mut Harness) {
    let effects = legacy_effects(harness).await;
    assert_eq!(effects.len(), 2, "one clear update plus one hide");
    assert_eq!(
        effects[0].header().member().unwrap().as_str(),
        "UpdatePreeditText"
    );
    let update_body = effects[0].body();
    let (text, cursor, visible, mode) = update_body
        .deserialize::<(zbus::zvariant::Value<'_>, u32, bool, u32)>()
        .unwrap();
    assert_eq!(
        crate::ibus_interface::ibus_text_value_to_string(&text),
        Some(String::new())
    );
    assert_eq!((cursor, visible, mode), (0, false, 0));
    assert_eq!(
        effects[1].header().member().unwrap().as_str(),
        "HidePreeditText"
    );
}

fn replay_keyval(ch: char) -> u32 {
    if ch.is_ascii() {
        ch as u32
    } else {
        0x0100_0000 | ch as u32
    }
}

#[test]
fn td121_native_engine_dispatch_preserves_burst_order_and_exact_replay() {
    zbus::block_on(bounded(async {
        let source = "ghbdtnghbdtn";
        let (mut harness, engine) = armed_exact_replay(source, source, true, true).await;
        let scope = match engine.committed_tail.autocorrect_suppression.as_ref() {
            Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope)) => scope.clone(),
            _ => panic!("actual exact replay must be armed"),
        };
        let path = engine.path.clone();
        let initial_epoch = engine.committed_tail.epoch;
        harness
            .connection
            .object_server()
            .at(path.as_str(), engine)
            .await
            .unwrap();
        let iface = harness
            .connection
            .object_server()
            .interface::<_, LayIbusEngine>(path.as_str())
            .await
            .unwrap();
        let held = iface.get_mut().await;
        let keys = std::iter::repeat_n((KEY_BACKSPACE, 14), source.chars().count())
            .chain(scope.replacement.chars().map(|ch| (replay_keyval(ch), 30)));
        let events: Vec<(u32, u32, u32)> = keys
            .flat_map(|(keyval, keycode)| {
                [0, RELEASE_MASK].map(move |state| (keyval, keycode, state))
            })
            .collect();
        assert_eq!(events.len(), 48);
        let mut expected_serials = Vec::new();
        for (offset, event) in events.iter().enumerate() {
            let serial = 24_000 + offset as u32;
            expected_serials.push(serial);
            let message = Message::method_call(path.as_str(), "ProcessKeyEvent")
                .unwrap()
                .interface(ENGINE_INTERFACE)
                .unwrap()
                .sender(DISPATCH_SENDER)
                .unwrap()
                .serial(std::num::NonZeroU32::new(serial).unwrap())
                .build(event)
                .unwrap();
            harness.peer.connection.send(&message).await.unwrap();
            assert!(harness.observer.process_next().await.unwrap());
        }
        // The existing object write guard creates real dispatcher backpressure.
        // Ingress has observed the entire bounded burst without executing it.
        assert_eq!(held.committed_tail.buffer, source);
        assert_eq!(
            harness
                .adapter
                .shared
                .reducer
                .lock()
                .unwrap()
                .unsettled
                .len(),
            48
        );
        drop(held);
        let mut received_serials = Vec::new();
        let mut handled = Vec::new();
        let mut effects = Vec::new();
        while received_serials.len() < expected_serials.len() {
            let message = next_peer_message(&mut harness.peer).await;
            match message.header().message_type() {
                Type::MethodReturn => {
                    received_serials.push(message.header().reply_serial().unwrap().get());
                    handled.push(message.body().deserialize::<bool>().unwrap());
                }
                Type::Signal => effects.push(message.header().member().unwrap().to_string()),
                other => panic!(
                    "unexpected dispatcher message: {other:?} serial={:?} error={:?} body={:?}",
                    message.header().reply_serial(),
                    message.header().error_name(),
                    message.body().deserialize::<String>(),
                ),
            }
        }
        let engine = iface.get().await;
        assert_eq!(
            received_serials, expected_serials,
            "actual dispatch must preserve receive order; tail={:?} effects={effects:?}",
            engine.committed_tail.buffer
        );
        assert!(
            handled.iter().all(|handled| !handled),
            "native replay is observe-only"
        );
        assert!(
            effects.is_empty(),
            "native replay must not emit IME text: {effects:?}"
        );
        assert_eq!(engine.committed_tail.buffer, scope.replacement);
        assert_eq!(engine.committed_tail.epoch, initial_epoch + 24);
        assert!(!engine.context_word_is_known());
        assert!(engine.committed_tail.pending_completion_learning.is_none());
        assert!(harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .unsettled
            .is_empty());
    }));
}

async fn exact_replay_surrounding_receipt(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    visible: &str,
) {
    let cursor = visible.chars().count() as u32;
    engine
        .set_surrounding_text(
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap(),
            crate::text::make_ibus_text(visible.to_string()),
            cursor,
            cursor,
        )
        .await
        .unwrap();
    no_legacy_output(harness).await;
}

async fn reset_to_unknown_start_with_one_preedit_clear(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
) {
    // The trace places Reset immediately after the final native edit. Bind the
    // fixture's recency precondition to this modeled callback so harness work
    // between callbacks cannot consume the production window.
    assert!(engine.committed_tail.last_input_at.is_some());
    engine.committed_tail.last_input_at = Some(Instant::now());
    let reset = method_message(
        DISPATCH_SENDER,
        21_020,
        TARGET_PATH,
        ENGINE_INTERFACE,
        "Reset",
    );
    harness.peer.connection.send(&reset).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    engine
        .reset(
            reset.header(),
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap(),
        )
        .await
        .unwrap();
    expect_one_empty_preedit_clear(harness).await;
    assert!(!engine.context_word_is_known());
    assert!(engine.context_reset_rereceipt.is_some());
}

async fn text_free_owned_soft_reset(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    serial: u32,
) {
    // This Reset is the next modeled callback after exact replay. Rebind only
    // the fixture clock; the runtime deadline and reset logic stay unchanged.
    assert!(engine.committed_tail.last_input_at.is_some());
    engine.committed_tail.last_input_at = Some(Instant::now());
    let reset = method_message(
        DISPATCH_SENDER,
        serial,
        TARGET_PATH,
        ENGINE_INTERFACE,
        "Reset",
    );
    harness.peer.connection.send(&reset).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    engine
        .reset(
            reset.header(),
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap(),
        )
        .await
        .unwrap();
    no_legacy_text_output(harness).await;
}

async fn run_exact_replay(
    tail: &str,
    expected_suffix: &str,
    target_layout_is_ru: bool,
    reset_to_unknown_start: bool,
) -> (Harness, LayIbusEngine, String) {
    let (mut harness, mut engine) = armed_exact_replay(
        tail,
        expected_suffix,
        target_layout_is_ru,
        reset_to_unknown_start,
    )
    .await;
    let scope = match engine.committed_tail.autocorrect_suppression.as_ref() {
        Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope)) => scope.clone(),
        _ => panic!("exact replay must be armed"),
    };
    let mut unhandled_callback_sink = scope.original_tail.clone();
    let base_epoch = scope.epoch;
    let mut serial = 21_030;
    let mut initial_preedit_cleared = reset_to_unknown_start;
    if reset_to_unknown_start {
        engine.config.auto_replace = true;
        engine.config.nanda_precognition = true;
        engine.reset_precognition_causal_counts();
        exact_replay_surrounding_receipt(&mut harness, &mut engine, &unhandled_callback_sink).await;
        assert!(engine.capture_observed_suffix_display_frame().is_some());
    }
    for _ in 0..scope.original_suffix.chars().count() {
        assert!(!legacy_key(&mut harness, &mut engine, serial, KEY_BACKSPACE, 14, 0).await);
        unhandled_callback_sink.pop();
        assert_eq!(engine.committed_tail.buffer, unhandled_callback_sink);
        assert!(
            !legacy_key(
                &mut harness,
                &mut engine,
                serial + 1,
                KEY_BACKSPACE,
                14,
                RELEASE_MASK,
            )
            .await
        );
        if initial_preedit_cleared {
            no_legacy_output(&mut harness).await;
        } else {
            expect_one_empty_preedit_clear(&mut harness).await;
            initial_preedit_cleared = true;
        }
        exact_replay_surrounding_receipt(&mut harness, &mut engine, &unhandled_callback_sink).await;
        if reset_to_unknown_start
            && !unhandled_callback_sink.is_empty()
            && !unhandled_callback_sink.ends_with(char::is_whitespace)
        {
            assert!(engine.capture_observed_suffix_display_frame().is_some());
        }
        serial += 2;
    }
    for ch in scope.replacement.chars() {
        let keyval = replay_keyval(ch);
        let keycode = if ch == ' ' { 57 } else { 30 };
        let shifted = ch.is_uppercase();
        if shifted {
            assert!(!legacy_key(&mut harness, &mut engine, serial, KEY_LEFT_SHIFT, 42, 0,).await);
            assert!(engine.exact_replay_quarantine_active());
            serial += 1;
        }
        let modifier_state = u32::from(shifted);
        assert!(
            !legacy_key(
                &mut harness,
                &mut engine,
                serial,
                keyval,
                keycode,
                modifier_state,
            )
            .await
        );
        unhandled_callback_sink.push(ch);
        assert_eq!(engine.committed_tail.buffer, unhandled_callback_sink);
        assert!(
            !legacy_key(
                &mut harness,
                &mut engine,
                serial + 1,
                keyval,
                keycode,
                RELEASE_MASK | modifier_state,
            )
            .await
        );
        if shifted {
            assert!(
                !legacy_key(
                    &mut harness,
                    &mut engine,
                    serial + 2,
                    KEY_LEFT_SHIFT,
                    42,
                    RELEASE_MASK,
                )
                .await
            );
            serial += 1;
        }
        no_legacy_output(&mut harness).await;
        exact_replay_surrounding_receipt(&mut harness, &mut engine, &unhandled_callback_sink).await;
        if reset_to_unknown_start
            && !unhandled_callback_sink.is_empty()
            && !unhandled_callback_sink.ends_with(char::is_whitespace)
        {
            assert!(engine.capture_observed_suffix_display_frame().is_some());
        }
        serial += 2;
    }
    assert_eq!(
        engine.committed_tail.epoch,
        base_epoch
            .wrapping_add(scope.original_suffix.chars().count() as u64)
            .wrapping_add(scope.replacement.chars().count() as u64)
    );
    assert!(engine.exact_replay_quarantine_active());
    assert!(engine.composition.preedit_suffix.is_empty());
    assert!(engine.composition.preedit_candidates.is_empty());
    assert!(engine.composition.preedit_replacement_targets.is_empty());
    assert!(engine.composition.pending_display_frame.is_none());
    assert!(engine.committed_tail.pending_completion_learning.is_none());
    assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    if reset_to_unknown_start {
        assert_eq!(engine.precognition_causal_counts(), (0, 0));
    }
    (harness, engine, unhandled_callback_sink)
}

pub(super) async fn no_legacy_text_output(harness: &mut Harness) {
    for effect in legacy_effects(harness).await {
        assert!(matches!(
            effect.header().member().unwrap().as_str(),
            "UpdatePreeditText"
                | "UpdatePreeditTextWithMode"
                | "ShowPreeditText"
                | "HidePreeditText"
        ));
    }
}

#[test]
fn ordinary_unknown_start_input_reaches_the_precognition_schedule_probe() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        engine.config.auto_replace = true;
        engine.config.nanda_precognition = true;
        start_source_free_unknown(&mut harness, &mut engine).await;
        assert!(!engine.context_word_is_known());
        engine.reset_precognition_causal_counts();

        assert!(legacy_key(&mut harness, &mut engine, 20_900, 'g' as u32, 34, 0).await);
        expect_legacy_commit(&mut harness.peer).await;
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                20_901,
                'g' as u32,
                34,
                RELEASE_MASK,
            )
            .await
        );
        no_legacy_output(&mut harness).await;
        exact_replay_surrounding_receipt(&mut harness, &mut engine, "g").await;
        assert!(engine.capture_observed_suffix_display_frame().is_some());
        assert_eq!(engine.precognition_causal_counts(), (1, 0));
        engine.cancel_precognition_display_generation();
    });
}

#[test]
fn td121_text_free_soft_reset_during_exact_replay_preserves_progress_epoch() {
    zbus::block_on(async {
        let mut observed = Vec::new();
        for (tail, target_layout_is_ru, reset_to_unknown_start, reset_count) in [
            ("ghbdtn", true, true, 2usize),
            ("привет ", false, false, 1usize),
        ] {
            let (mut harness, mut engine) =
                armed_exact_replay(tail, tail, target_layout_is_ru, reset_to_unknown_start).await;
            let scope = match engine.committed_tail.autocorrect_suppression.as_ref() {
                Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope)) => scope.clone(),
                _ => panic!("truthful exact replay scope"),
            };
            assert_eq!(scope.original_suffix, tail);
            assert_eq!(scope.original_tail, engine.committed_tail.buffer);
            exact_replay_surrounding_receipt(&mut harness, &mut engine, &scope.original_tail).await;
            if reset_to_unknown_start {
                let word_scope = engine
                    .context_word_scope
                    .as_ref()
                    .expect("exact replay has a current word scope");
                assert_eq!(
                    word_scope.lineage().completeness,
                    WordCompleteness::UnknownStart
                );
                assert_eq!(
                    word_scope.lineage().observed_suffix_chars,
                    tail.chars().count() as u32,
                );
                assert!(engine
                    .context_token
                    .as_ref()
                    .expect("exact replay has a live token")
                    .matches_word_scope(word_scope));
            }
            let base_epoch = engine.committed_tail.epoch;
            let mut expected_tail = scope.original_tail;
            let mut native_delete_presses = 0;
            let mut text_free_reset_epochs = Vec::new();
            let mut release_pairs_match = true;
            let mut serial = 21_300;

            for reset_index in 0..=reset_count {
                let press_handled =
                    legacy_key(&mut harness, &mut engine, serial, KEY_BACKSPACE, 14, 0).await;
                native_delete_presses += usize::from(!press_handled);
                no_legacy_text_output(&mut harness).await;
                if !press_handled {
                    expected_tail.pop();
                }

                if reset_index < reset_count {
                    let epoch_before_reset = engine.committed_tail.epoch;
                    text_free_owned_soft_reset(&mut harness, &mut engine, serial + 1).await;
                    let epoch_after_reset = engine.committed_tail.epoch;
                    text_free_reset_epochs.push((epoch_before_reset, epoch_after_reset));
                    exact_replay_surrounding_receipt(&mut harness, &mut engine, &expected_tail)
                        .await;
                }
                let release_handled = legacy_key(
                    &mut harness,
                    &mut engine,
                    serial + 2,
                    KEY_BACKSPACE,
                    14,
                    RELEASE_MASK,
                )
                .await;
                no_legacy_text_output(&mut harness).await;
                release_pairs_match &= release_handled == press_handled;
                serial += 3;
            }

            let expected_distance = reset_count + 1;
            let actual_distance = engine.committed_tail.epoch.wrapping_sub(base_epoch) as usize;
            observed.push((
                tail.to_string(),
                base_epoch,
                text_free_reset_epochs,
                expected_distance,
                actual_distance,
                native_delete_presses,
                release_pairs_match,
                engine.committed_tail.buffer.clone(),
                expected_tail,
            ));
        }
        assert!(
            observed.iter().all(
                |(
                    _,
                    _,
                    reset_epochs,
                    expected_distance,
                    actual_distance,
                    native,
                    releases,
                    actual,
                    expected,
                )| {
                    reset_epochs.iter().all(|(before, after)| before == after)
                        && actual_distance == expected_distance
                        && native == expected_distance
                        && *releases
                        && actual == expected
                }
            ),
            "text-free owned Resets must not consume exact replay progress: {observed:?}",
        );
    });
}

#[test]
fn td121_soft_reset_does_not_preserve_invalid_exact_replay_scopes() {
    zbus::block_on(async {
        for (invalidation, publishes) in [
            ("owner", true),
            ("path", false),
            ("selection", true),
            ("composition", true),
            ("expired", true),
        ] {
            let (mut harness, mut engine) =
                armed_exact_replay("ghbdtn", "ghbdtn", true, false).await;
            match invalidation {
                "owner" => {
                    engine.client_context.runtime_owner_lease_identity = engine
                        .client_context
                        .runtime_owner_lease_identity
                        .wrapping_add(1);
                }
                "path" => {
                    engine.path = "/io/github/radislabus_star/LayIme/engine/stale".to_string()
                }
                "selection" => {
                    engine.client_context.surrounding_text_snapshot =
                        Some(SurroundingTextSnapshot::new("ghbdtn".to_string(), 6, 5));
                }
                "composition" => engine.composition.buffer = "active".to_string(),
                "expired" => {
                    let expired = match engine.committed_tail.autocorrect_suppression.as_mut() {
                        Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope)) => {
                            scope.expires_at = Instant::now() - Duration::from_millis(1);
                            scope.clone()
                        }
                        _ => panic!("active exact replay scope"),
                    };
                    engine.shared.lock().unwrap().autocorrect_suppression = Some(
                        crate::protocol::AutocorrectSuppression::ExactReplay(expired),
                    );
                }
                _ => unreachable!(),
            }
            assert!(!engine.exact_replay_quarantine_active(), "{invalidation}");
            let epoch = engine.committed_tail.epoch;
            let tail = engine.committed_tail.buffer.clone();
            let shared_before = (invalidation == "path").then(|| {
                let shared = engine.shared.lock().unwrap();
                (
                    shared.active_path.clone(),
                    shared.handoff_tail_epoch,
                    shared.handoff_tail_buffer.clone(),
                )
            });
            text_free_owned_soft_reset(&mut harness, &mut engine, 21_400).await;
            assert_eq!(engine.committed_tail.buffer, tail, "{invalidation}");
            assert!(!engine.exact_replay_quarantine_active(), "{invalidation}");
            assert_eq!(
                engine.committed_tail.epoch,
                epoch.wrapping_add(u64::from(publishes)),
                "{invalidation}",
            );
            if let Some(expected) = shared_before {
                let shared = engine.shared.lock().unwrap();
                assert_eq!(
                    (
                        shared.active_path.clone(),
                        shared.handoff_tail_epoch,
                        shared.handoff_tail_buffer.clone(),
                    ),
                    expected,
                    "stale path cannot mutate current shared ownership",
                );
            }
        }
    });
}

#[test]
fn td121_completed_unknown_replay_exports_tail_before_the_next_manual_toggle() {
    zbus::block_on(async {
        let (mut harness, engine, replayed) =
            run_exact_replay("ghbdtn", "ghbdtn", true, true).await;
        let path = engine.path.clone();
        let (engine, visible_tail) =
            super::residuals::cycle09_visible_tail(&mut harness, engine).await;
        let visible_tail_is_authoritative =
            super::residuals::cycle09_tail_is_authoritative(&visible_tail, &path, true, &replayed);
        let (_engine, next_manual_toggle) =
            super::residuals::cycle09_manual_toggle(&mut harness, engine).await;
        assert!(
            visible_tail_is_authoritative && next_manual_toggle == Ok((3, false)),
            "completed UnknownStart replay must export its exact tail before the queued toggle: visible_tail={visible_tail:?} next_manual_toggle={next_manual_toggle:?}",
        );
    });
}

#[test]
fn td121_firefox_coarse_reset_receipts_separate_readout_from_replay_confirmation() {
    zbus::block_on(async {
        for (tail, reset_choreography) in [("ghbdtn", true), ("ghbdtn ", false)] {
            let (mut harness, mut engine) =
                armed_exact_replay(tail, tail, true, reset_choreography).await;
            let scope = match engine.committed_tail.autocorrect_suppression.as_ref() {
                Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope)) => scope.clone(),
                _ => panic!("exact replay must be armed"),
            };
            let retained_prefix = scope
                .original_tail
                .strip_suffix(&scope.original_suffix)
                .unwrap();
            let expected_tail = format!("{retained_prefix}{}", scope.replacement);
            let mut serial = 21_500;

            for index in 0..scope.original_suffix.chars().count() {
                assert!(!legacy_key(&mut harness, &mut engine, serial, KEY_BACKSPACE, 14, 0).await);
                assert!(
                    !legacy_key(
                        &mut harness,
                        &mut engine,
                        serial + 1,
                        KEY_BACKSPACE,
                        14,
                        RELEASE_MASK,
                    )
                    .await
                );
                if index == 0 && !reset_choreography {
                    expect_one_empty_preedit_clear(&mut harness).await;
                } else {
                    no_legacy_output(&mut harness).await;
                }
                serial += 2;
                if index == 2 {
                    text_free_owned_soft_reset(&mut harness, &mut engine, serial).await;
                    serial += 1;
                    exact_replay_surrounding_receipt(&mut harness, &mut engine, "ghbdt").await;
                    if reset_choreography {
                        // This is an already observed deletion surface. Its
                        // retained provenance remains inert during replay.
                        assert!(engine
                            .context_reset_rereceipt
                            .as_ref()
                            .is_some_and(|pending| !pending.confirmed));
                    } else {
                        // The fixture's unchanged leading prefix is missing.
                        assert!(engine.context_reset_rereceipt.is_none());
                    }
                    assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                    assert!(engine.capture_observed_suffix_display_frame().is_none());
                }
            }
            assert_eq!(engine.committed_tail.buffer, retained_prefix);
            text_free_owned_soft_reset(&mut harness, &mut engine, serial).await;
            serial += 1;
            exact_replay_surrounding_receipt(&mut harness, &mut engine, retained_prefix).await;

            for ch in scope.replacement.chars() {
                let keyval = replay_keyval(ch);
                let keycode = if ch == ' ' { 57 } else { 30 };
                assert!(!legacy_key(&mut harness, &mut engine, serial, keyval, keycode, 0).await);
                assert!(
                    !legacy_key(
                        &mut harness,
                        &mut engine,
                        serial + 1,
                        keyval,
                        keycode,
                        RELEASE_MASK,
                    )
                    .await
                );
                no_legacy_output(&mut harness).await;
                serial += 2;
            }
            assert_eq!(engine.committed_tail.buffer, expected_tail);

            // The live timeout may precede the final Reset. At this separate stage
            // every positive VisibleTailV3 admission lane is still false, and the
            // mirrored keys have emitted no IME text effect.
            assert_eq!(engine.context_word_is_known(), tail.ends_with(' '));
            assert!(!engine.context_exact_manual_handoff_bounds_unknown_suffix());
            assert!(!engine.context_observed_suffix_exact_manual_handoff_allowed());
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            no_legacy_output(&mut harness).await;

            text_free_owned_soft_reset(&mut harness, &mut engine, serial).await;
            assert!(engine.context_reset_rereceipt.is_some());
            assert!(!engine.context_observed_suffix_exact_manual_handoff_allowed());
            let (mut engine, passive) =
                super::residuals::cycle09_visible_tail(&mut harness, engine).await;
            assert_eq!(passive.as_ref().unwrap().0, "passive:unknown-context");
            assert!(passive.as_ref().unwrap().1.is_empty());

            exact_replay_surrounding_receipt(&mut harness, &mut engine, &expected_tail).await;
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert!(!engine.context_observed_suffix_exact_manual_handoff_allowed());
            let (engine, confirmed) =
                super::residuals::cycle09_visible_tail(&mut harness, engine).await;
            assert!(super::residuals::cycle09_tail_is_authoritative(
                &confirmed,
                &engine.path,
                true,
                &expected_tail,
            ));
            assert!(
                engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "readout must not consume the confirmed one-shot mutation receipt"
            );
            let (_engine, toggle) =
                super::residuals::cycle09_manual_toggle(&mut harness, engine).await;
            assert_eq!(
                toggle,
                Ok((3, false)),
                "confirmed rereceipt permits mutation"
            );
        }
    });
}

async fn td121_observe_reset_batch(harness: &mut Harness, count: u32) -> Vec<Message> {
    let mut messages = Vec::new();
    for index in 0..count {
        let reset = method_message(
            DISPATCH_SENDER,
            22_000 + index,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "Reset",
        );
        harness.peer.connection.send(&reset).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        messages.push(reset);
    }
    messages
}

async fn td121_deliver_observed_reset(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    reset: &Message,
) {
    engine
        .reset(
            reset.header(),
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap(),
        )
        .await
        .unwrap();
    no_legacy_text_output(harness).await;
}

#[test]
fn td121_batched_reset_callbacks_retain_provenance_until_final_exact_receipt() {
    zbus::block_on(async {
        let mut outcomes = Vec::new();
        for (tail, suffix, reset_choreography) in [
            ("ghbdtn", "ghbdtn", true),
            ("prefix ghbdtn ", "ghbdtn ", false),
        ] {
            for count in [2, 3, 5] {
                let (mut harness, mut engine, replayed) =
                    run_exact_replay(tail, suffix, true, reset_choreography).await;
                text_free_owned_soft_reset(&mut harness, &mut engine, 21_990).await;
                let length = replayed.chars().count();
                let prior: String = replayed.chars().take(length - count as usize).collect();
                exact_replay_surrounding_receipt(&mut harness, &mut engine, &prior).await;
                let resets = td121_observe_reset_batch(&mut harness, count).await;
                let mut retained = Vec::new();
                for (index, reset) in resets.iter().enumerate() {
                    td121_deliver_observed_reset(&mut harness, &mut engine, reset).await;
                    retained.push(engine.context_reset_rereceipt.is_some());
                    assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                    let visible: String = replayed
                        .chars()
                        .take(length - resets.len() + index + 1)
                        .collect();
                    exact_replay_surrounding_receipt(&mut harness, &mut engine, &visible).await;
                    if index + 1 != resets.len() {
                        assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                        let (returned, visible) =
                            super::residuals::cycle09_visible_tail(&mut harness, engine).await;
                        engine = returned;
                        assert_eq!(visible.as_ref().unwrap().0, "passive:unknown-context");
                        assert!(visible.as_ref().unwrap().1.is_empty());
                        assert!(engine.committed_tail.pending_completion_learning.is_none());
                        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
                        no_legacy_output(&mut harness).await;
                    }
                }
                let exact = engine.context_reset_rereceipt_exact_manual_handoff_allowed();
                let (engine, visible) =
                    super::residuals::cycle09_visible_tail(&mut harness, engine).await;
                let authoritative = super::residuals::cycle09_tail_is_authoritative(
                    &visible,
                    &engine.path,
                    true,
                    &replayed,
                );
                let (_engine, toggle) =
                    super::residuals::cycle09_manual_toggle(&mut harness, engine).await;
                outcomes.push((tail, count, retained, exact, authoritative, toggle));
            }
        }
        assert!(
            outcomes
                .iter()
                .all(|(_, _, retained, exact, authoritative, toggle)| retained
                    .iter()
                    .all(|value| *value)
                    && *exact
                    && *authoritative
                    && *toggle == Ok((3, false))),
            "batched callback outcomes: {outcomes:?}"
        );
    });
}

#[test]
fn td121_batched_reset_callbacks_cannot_restore_invalidated_provenance() {
    zbus::block_on(async {
        for gap in [
            "text",
            "selection",
            "focus_out",
            "owner",
            "capabilities",
            "duplicate_ingress",
        ] {
            let (mut harness, mut engine, replayed) =
                run_exact_replay("ghbdtn", "ghbdtn", true, true).await;
            text_free_owned_soft_reset(&mut harness, &mut engine, 21_990).await;
            exact_replay_surrounding_receipt(&mut harness, &mut engine, "п").await;
            let resets = td121_observe_reset_batch(&mut harness, 3).await;
            td121_deliver_observed_reset(&mut harness, &mut engine, &resets[0]).await;
            exact_replay_surrounding_receipt(&mut harness, &mut engine, "прив").await;
            assert!(engine.context_reset_rereceipt.is_some());
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            match gap {
                "text" => {
                    exact_replay_surrounding_receipt(&mut harness, &mut engine, "unrelated").await;
                }
                "selection" => {
                    super::residuals::surrounding_receipt(&mut harness, &mut engine, "прив", 4, 3)
                        .await;
                }
                "focus_out" => {
                    super::residuals::actual_focus_out(&mut harness, &mut engine, 22_010).await;
                }
                "owner" => {
                    super::residuals::global_engine_changed(&mut harness, 22_010, "foreign-ime")
                        .await;
                }
                "capabilities" => engine.set_client_capabilities(1 | 1 << 3),
                "duplicate_ingress" => {
                    harness.peer.connection.send(&resets[0]).await.unwrap();
                    assert!(matches!(
                        bounded(harness.observer.process_next()).await,
                        Err(AdapterError::Denied)
                    ));
                }
                _ => unreachable!(),
            }
            for reset in &resets[1..] {
                td121_deliver_observed_reset(&mut harness, &mut engine, reset).await;
                exact_replay_surrounding_receipt(&mut harness, &mut engine, &replayed).await;
                assert!(
                    !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                    "{gap}"
                );
                assert!(engine.committed_tail.pending_completion_learning.is_none());
                assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
            }
            if gap == "duplicate_ingress" {
                // The real duplicate ingress cancelled the observer. Exercise
                // the bridge's denial without a live-observer-only helper.
                let bridge = crate::bridge::LayImeBridge {
                    ibus_connection: harness.connection.clone(),
                    shared: engine.shared.clone(),
                    context_admission_required: true,
                    admission: Some(harness.adapter.clone()),
                };
                let (visible, ()) = bounded(future::zip(
                    bridge.visible_tail_v3_inner(),
                    serve_ping_and_marker(&mut harness.peer),
                ))
                .await;
                let (toggle, ()) = bounded(future::zip(
                    bridge.manual_toggle_v3_inner(),
                    serve_ping_and_marker(&mut harness.peer),
                ))
                .await;
                let denied = "org.freedesktop.DBus.Error.Failed: metadata observer cancelled";
                assert_eq!(
                    visible.map_err(|error| error.to_string()),
                    Err(denied.into())
                );
                assert_eq!(
                    toggle.map_err(|error| error.to_string()),
                    Err(denied.into())
                );
                no_legacy_text_output(&mut harness).await;
                continue;
            }
            let (engine, visible) =
                super::residuals::cycle09_visible_tail(&mut harness, engine).await;
            assert!(
                !super::residuals::cycle09_tail_is_authoritative(
                    &visible,
                    &engine.path,
                    true,
                    &replayed,
                ),
                "{gap}"
            );
            let (_engine, toggle) =
                super::residuals::cycle09_manual_toggle(&mut harness, engine).await;
            assert_ne!(toggle, Ok((3, false)), "{gap}");
            no_legacy_text_output(&mut harness).await;
        }
    });
}

#[test]
fn td121_completed_replay_delayed_client_surfaces_require_final_exact_receipt() {
    zbus::block_on(async {
        for (tail, suffix, reset_choreography) in [
            ("ghbdtn", "ghbdtn", true),
            ("prefix ghbdtn ", "ghbdtn ", false),
        ] {
            for source_prefixes in [true, false] {
                let (mut harness, mut engine, replayed) =
                    run_exact_replay(tail, suffix, true, reset_choreography).await;
                text_free_owned_soft_reset(&mut harness, &mut engine, 21_800).await;
                assert!(engine.context_reset_rereceipt.is_some());
                let original = match engine.committed_tail.autocorrect_suppression.as_ref() {
                    Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope)) => {
                        scope.original_tail.clone()
                    }
                    _ => panic!("completed replay retains its scope"),
                };
                let prior = if source_prefixes {
                    &original
                } else {
                    &replayed
                };
                let length = prior.chars().count();
                for count in [length - 1, length - 1, length - 2] {
                    let delayed: String = prior.chars().take(count).collect();
                    exact_replay_surrounding_receipt(&mut harness, &mut engine, &delayed).await;
                    assert!(engine.context_reset_rereceipt.is_some(),
                        "completed replay predecessor lost: source_prefixes={source_prefixes} count={count}");
                    assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                    let (returned, visible) =
                        super::residuals::cycle09_visible_tail(&mut harness, engine).await;
                    engine = returned;
                    assert_eq!(visible.as_ref().unwrap().0, "passive:unknown-context");
                    assert!(visible.as_ref().unwrap().1.is_empty());
                    no_legacy_text_output(&mut harness).await;
                }
                exact_replay_surrounding_receipt(&mut harness, &mut engine, &replayed).await;
                assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                let (engine, visible) =
                    super::residuals::cycle09_visible_tail(&mut harness, engine).await;
                assert!(super::residuals::cycle09_tail_is_authoritative(
                    &visible,
                    &engine.path,
                    true,
                    &replayed,
                ));
                let (_engine, toggle) =
                    super::residuals::cycle09_manual_toggle(&mut harness, engine).await;
                assert_eq!(toggle, Ok((3, false)));
            }
        }
    });
}

#[test]
fn td121_completed_replay_prior_surface_cannot_hide_contradiction() {
    zbus::block_on(async {
        for gap in [
            "text",
            "selection",
            "cursor",
            "owner",
            "shared_scope",
            "focus_out",
            "printable",
        ] {
            let (mut harness, mut engine, replayed) =
                run_exact_replay("ghbdtn", "ghbdtn", true, true).await;
            text_free_owned_soft_reset(&mut harness, &mut engine, 21_850).await;
            exact_replay_surrounding_receipt(&mut harness, &mut engine, "ghbdt").await;
            assert!(engine.context_reset_rereceipt.is_some());
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            match gap {
                "text" => {
                    exact_replay_surrounding_receipt(&mut harness, &mut engine, "unrelated").await;
                }
                "selection" => {
                    super::residuals::surrounding_receipt(&mut harness, &mut engine, "ghbd", 4, 3)
                        .await;
                }
                "cursor" => {
                    super::residuals::surrounding_receipt(&mut harness, &mut engine, "ghbd", 3, 3)
                        .await;
                }
                "owner" => {
                    super::residuals::global_engine_changed(&mut harness, 21_860, "foreign-ime")
                        .await;
                    exact_replay_surrounding_receipt(&mut harness, &mut engine, "ghbd").await;
                }
                "shared_scope" => {
                    engine.shared.lock().unwrap().autocorrect_suppression = None;
                    exact_replay_surrounding_receipt(&mut harness, &mut engine, "ghbd").await;
                }
                "focus_out" => {
                    super::residuals::actual_focus_out(&mut harness, &mut engine, 21_861).await;
                }
                "printable" => {
                    assert!(legacy_key(&mut harness, &mut engine, 21_862, 'ф' as u32, 30, 0).await);
                    expect_legacy_commit(&mut harness.peer).await;
                    assert!(engine.committed_tail.buffer.ends_with('ф'));
                    assert!(
                        legacy_key(
                            &mut harness,
                            &mut engine,
                            21_863,
                            'ф' as u32,
                            30,
                            RELEASE_MASK,
                        )
                        .await
                    );
                    exact_replay_surrounding_receipt(&mut harness, &mut engine, "ghbd").await;
                }
                _ => unreachable!(),
            }
            assert!(engine.context_reset_rereceipt.is_none(), "{gap}");
            exact_replay_surrounding_receipt(&mut harness, &mut engine, &replayed).await;
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{gap}"
            );
            let (_engine, toggle) =
                super::residuals::cycle09_manual_toggle(&mut harness, engine).await;
            if gap == "focus_out" {
                assert_eq!(
                    toggle,
                    Err("org.freedesktop.DBus.Error.Failed: context admission denied".to_string()),
                );
            } else {
                assert_eq!(toggle, Ok((0, false)), "{gap}");
            }
            no_legacy_text_output(&mut harness).await;
        }
    });
}

#[test]
fn td121_completed_replay_preserved_prefix_and_consumed_receipt_are_not_rearmed() {
    zbus::block_on(async {
        for gap in ["changed_prefix", "short_prefix", "confirmed", "consumed"] {
            let (mut harness, mut engine, replayed) =
                run_exact_replay("prefix ghbdtn ", "ghbdtn ", true, false).await;
            text_free_owned_soft_reset(&mut harness, &mut engine, 21_870).await;
            let prior = " prefix ghbdt";
            exact_replay_surrounding_receipt(&mut harness, &mut engine, prior).await;
            assert!(engine.context_reset_rereceipt.is_some());
            match gap {
                "changed_prefix" | "short_prefix" => {
                    let snapshot = if gap == "changed_prefix" {
                        " changed ghbd"
                    } else {
                        " prefi"
                    };
                    exact_replay_surrounding_receipt(&mut harness, &mut engine, snapshot).await;
                }
                "confirmed" | "consumed" => {
                    exact_replay_surrounding_receipt(&mut harness, &mut engine, &replayed).await;
                    assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                    if gap == "consumed" {
                        let (returned, toggle) =
                            super::residuals::cycle09_manual_toggle(&mut harness, engine).await;
                        engine = returned;
                        assert_eq!(toggle, Ok((3, false)));
                    }
                    exact_replay_surrounding_receipt(&mut harness, &mut engine, prior).await;
                }
                _ => unreachable!(),
            }
            assert!(engine.context_reset_rereceipt.is_none(), "{gap}");
            exact_replay_surrounding_receipt(&mut harness, &mut engine, &replayed).await;
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            no_legacy_text_output(&mut harness).await;
        }
    });
}

#[test]
fn td121_visible_tail_refuses_unqualified_unknown_suffixes_without_side_effects() {
    zbus::block_on(async {
        for invalidation in [
            "incomplete",
            "mismatched_snapshot",
            "selection",
            "stale_context",
        ] {
            let (mut harness, mut engine, replayed) =
                run_exact_replay("ghbdtn", "ghbdtn", true, true).await;
            match invalidation {
                "incomplete" => {
                    engine.committed_tail.buffer.pop();
                    let incomplete = engine.committed_tail.buffer.clone();
                    exact_replay_surrounding_receipt(&mut harness, &mut engine, &incomplete).await;
                }
                "mismatched_snapshot" => {
                    engine.client_context.surrounding_text_snapshot =
                        Some(SurroundingTextSnapshot::new("другой".to_string(), 6, 6));
                }
                "selection" => {
                    engine.client_context.surrounding_text_snapshot =
                        Some(SurroundingTextSnapshot::new(replayed.clone(), 6, 5));
                }
                "stale_context" => {
                    super::residuals::global_engine_changed(&mut harness, 21_410, "foreign-ime")
                        .await;
                }
                _ => unreachable!(),
            }
            if invalidation != "stale_context" {
                assert!(
                    !engine.context_observed_suffix_exact_manual_handoff_allowed(),
                    "{invalidation}",
                );
            }
            let local_suppression = engine.committed_tail.autocorrect_suppression.clone();
            let shared_suppression = engine
                .shared
                .lock()
                .unwrap()
                .autocorrect_suppression
                .clone();
            let pending_learning = engine.committed_tail.pending_completion_learning.is_some();
            let (engine, reply) =
                super::residuals::cycle09_visible_tail(&mut harness, engine).await;
            assert!(
                matches!(reply, Ok((ref state, ref text, _, _, _, ref focus))
                    if state == "passive:unknown-context" && text.is_empty() && focus.is_empty()),
                "{invalidation}: {reply:?}",
            );
            assert_eq!(
                engine.committed_tail.autocorrect_suppression, local_suppression,
                "{invalidation}",
            );
            assert_eq!(
                engine.shared.lock().unwrap().autocorrect_suppression,
                shared_suppression,
                "{invalidation}",
            );
            assert_eq!(
                engine.committed_tail.pending_completion_learning.is_some(),
                pending_learning,
                "{invalidation}",
            );
            assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
        }
    });
}

#[test]
fn exact_replay_unhandled_callback_preserves_six_complete_transactions() {
    zbus::block_on(async {
        for (tail, suffix, target_layout_is_ru, expected, reset_to_unknown_start) in [
            ("ghbdtn", "ghbdtn", true, "привет", true),
            ("Ghbdtn", "Ghbdtn", true, "Привет", false),
            ("привет", "привет", false, "ghbdtn", false),
            ("ghbdtn ", "ghbdtn ", true, "привет ", false),
            ("привет ", "привет ", false, "ghbdtn ", false),
            ("prefix 1ghbdtn", "1ghbdtn", true, "prefix 1привет", false),
        ] {
            let (_harness, engine, unhandled_callback_sink) =
                run_exact_replay(tail, suffix, target_layout_is_ru, reset_to_unknown_start).await;
            let expected_sink = if reset_to_unknown_start {
                expected.to_string()
            } else {
                format!(" {expected}")
            };
            assert_eq!(unhandled_callback_sink, expected_sink);
            assert_eq!(engine.committed_tail.buffer, unhandled_callback_sink);
        }
    });
}

#[test]
fn exact_replay_order_and_visible_glyph_mismatches_revoke_without_text_effect() {
    zbus::block_on(async {
        // Printable before deletion completes.
        let (mut harness, mut engine) = armed_exact_replay("gh", "gh", true, false).await;
        let before = engine.committed_tail.buffer.clone();
        assert!(legacy_key(&mut harness, &mut engine, 21_100, replay_keyval('п'), 34, 0).await);
        assert_eq!(engine.committed_tail.buffer, before);
        expect_one_empty_preedit_clear(&mut harness).await;
        assert!(engine.committed_tail.autocorrect_suppression.is_none());
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                21_101,
                replay_keyval('п'),
                34,
                RELEASE_MASK
            )
            .await
        );

        // A command chord is not printable replay authority, even with the expected keysym.
        let (mut harness, mut engine) = armed_exact_replay("g", "g", true, false).await;
        assert!(!legacy_key(&mut harness, &mut engine, 21_145, KEY_BACKSPACE, 14, 0).await);
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                21_147,
                replay_keyval('п'),
                34,
                1 << 2,
            )
            .await
        );
        expect_one_empty_preedit_clear(&mut harness).await;
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                21_148,
                replay_keyval('п'),
                34,
                RELEASE_MASK | 1 << 2,
            )
            .await
        );

        // Duplicate Backspace after the one-character deletion phase.
        let (mut harness, mut engine) = armed_exact_replay("g", "g", true, false).await;
        assert!(!legacy_key(&mut harness, &mut engine, 21_110, KEY_BACKSPACE, 14, 0).await);
        assert!(legacy_key(&mut harness, &mut engine, 21_112, KEY_BACKSPACE, 14, 0).await);
        expect_one_empty_preedit_clear(&mut harness).await;
        assert!(engine.committed_tail.autocorrect_suppression.is_none());

        // Skipped epoch cannot select a textually plausible phase.
        let (mut harness, mut engine) = armed_exact_replay("1g", "1g", true, false).await;
        engine.committed_tail.epoch = engine.committed_tail.epoch.wrapping_add(1);
        assert!(legacy_key(&mut harness, &mut engine, 21_120, KEY_BACKSPACE, 14, 0).await);
        expect_one_empty_preedit_clear(&mut harness).await;
        assert!(engine.committed_tail.autocorrect_suppression.is_none());
    });
}

#[test]
fn exact_replay_duplicate_and_identity_mismatches_revoke_without_text_effect() {
    zbus::block_on(async {
        // Duplicate first printable while the contour expects its successor.
        let (mut harness, mut engine) = armed_exact_replay("gh", "gh", true, false).await;
        for serial in [21_130, 21_132] {
            assert!(!legacy_key(&mut harness, &mut engine, serial, KEY_BACKSPACE, 14, 0).await);
        }
        assert!(!legacy_key(&mut harness, &mut engine, 21_134, replay_keyval('п'), 34, 0).await);
        let before = engine.committed_tail.buffer.clone();
        assert!(legacy_key(&mut harness, &mut engine, 21_136, replay_keyval('п'), 34, 0).await);
        assert_eq!(engine.committed_tail.buffer, before);
        expect_one_empty_preedit_clear(&mut harness).await;

        // Client-visible keysym wins over a physical keycode that maps to the target glyph.
        let (mut harness, mut engine) = armed_exact_replay("g", "g", true, false).await;
        assert!(!legacy_key(&mut harness, &mut engine, 21_140, KEY_BACKSPACE, 14, 0).await);
        assert_eq!(engine.physical_char('g' as u32, 34), Some('п'));
        assert_eq!(engine.passthrough_visible_char('g' as u32, 34), Some('g'));
        assert!(legacy_key(&mut harness, &mut engine, 21_142, 'g' as u32, 34, 0).await);
        expect_one_empty_preedit_clear(&mut harness).await;

        // Owner identity change invalidates the immutable lease.
        let (mut harness, mut engine) = armed_exact_replay("g", "g", true, false).await;
        engine.client_context.runtime_owner_lease_identity = engine
            .client_context
            .runtime_owner_lease_identity
            .wrapping_add(1);
        assert!(legacy_key(&mut harness, &mut engine, 21_150, KEY_BACKSPACE, 14, 0).await);
        expect_one_empty_preedit_clear(&mut harness).await;
        assert!(engine.committed_tail.autocorrect_suppression.is_none());
    });
}

#[test]
fn exact_replay_active_path_and_expiry_refuse_before_text_effect() {
    zbus::block_on(async {
        // A path change invalidates local replay authority without clearing the
        // still-owned scope for the original engine path.
        let (mut harness, mut engine) = armed_exact_replay("g", "g", true, false).await;
        let before = engine.committed_tail.buffer.clone();
        engine.path = "/io/github/radislabus_star/LayIme/engine/other".to_string();
        assert!(legacy_key(&mut harness, &mut engine, 21_160, KEY_BACKSPACE, 14, 0).await);
        assert_eq!(engine.committed_tail.buffer, before);
        expect_one_empty_preedit_clear(&mut harness).await;
        assert!(engine.committed_tail.autocorrect_suppression.is_none());
        assert!(matches!(
            engine
                .shared
                .lock()
                .unwrap()
                .autocorrect_suppression
                .as_ref(),
            Some(crate::protocol::AutocorrectSuppression::ExactReplay(_))
        ));
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                21_161,
                KEY_BACKSPACE,
                14,
                RELEASE_MASK,
            )
            .await
        );
        no_legacy_output(&mut harness).await;

        // Expiry while deletion is still active rejects rather than retiring
        // the contour as a completed transaction.
        let (mut harness, mut engine) = armed_exact_replay("g", "g", true, false).await;
        let before = engine.committed_tail.buffer.clone();
        let expired = match engine.committed_tail.autocorrect_suppression.as_mut() {
            Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope)) => {
                scope.expires_at = Instant::now() - Duration::from_millis(1);
                scope.clone()
            }
            _ => panic!("active exact replay scope"),
        };
        engine.shared.lock().unwrap().autocorrect_suppression = Some(
            crate::protocol::AutocorrectSuppression::ExactReplay(expired),
        );
        assert!(legacy_key(&mut harness, &mut engine, 21_170, KEY_BACKSPACE, 14, 0).await);
        assert_eq!(engine.committed_tail.buffer, before);
        expect_one_empty_preedit_clear(&mut harness).await;
        assert!(engine.committed_tail.autocorrect_suppression.is_none());
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                21_171,
                KEY_BACKSPACE,
                14,
                RELEASE_MASK,
            )
            .await
        );
        no_legacy_output(&mut harness).await;
    });
}

#[test]
fn completed_exact_replay_retires_before_ordinary_managed_input() {
    zbus::block_on(async {
        for (tail, suffix, target_layout_is_ru, next) in [
            ("ghbdtn", "ghbdtn", true, 'ф'),
            ("привет ", "привет ", false, 'a'),
        ] {
            let (mut harness, mut engine, replayed) =
                run_exact_replay(tail, suffix, target_layout_is_ru, false).await;
            let snapshot_before_shift = engine.client_context.surrounding_text_snapshot.clone();
            for (serial, state) in [
                (21_190, 0),
                (21_191, RELEASE_MASK),
                (21_192, 0),
                (21_193, RELEASE_MASK),
            ] {
                assert!(
                    !legacy_key(&mut harness, &mut engine, serial, KEY_LEFT_SHIFT, 42, state,)
                        .await
                );
                no_legacy_output(&mut harness).await;
            }
            assert_eq!(
                engine.client_context.surrounding_text_snapshot,
                snapshot_before_shift
            );
            assert!(engine.exact_replay_quarantine_active());
            let keyval = replay_keyval(next);
            assert!(legacy_key(&mut harness, &mut engine, 21_200, keyval, 30, 0).await);
            let effects = legacy_effects(&mut harness).await;
            assert_eq!(effects.len(), 1);
            assert_eq!(effects[0].header().member().unwrap().as_str(), "CommitText");
            assert_eq!(engine.committed_tail.buffer, format!("{replayed}{next}"));
            assert!(engine.committed_tail.pending_completion_learning.is_none());
            assert!(legacy_key(&mut harness, &mut engine, 21_201, keyval, 30, RELEASE_MASK).await);
            no_legacy_output(&mut harness).await;
            assert!(legacy_key(&mut harness, &mut engine, 21_202, KEY_SPACE, 57, 0).await);
            let effects = legacy_effects(&mut harness).await;
            assert_eq!(effects.len(), 1);
            assert_eq!(effects[0].header().member().unwrap().as_str(), "CommitText");
            assert_eq!(engine.committed_tail.buffer, format!("{replayed}{next} "));
            assert!(engine.committed_tail.autocorrect_suppression.is_none());
        }
    });
}

async fn replay_prefix_then_reset(
    tail: &str,
    suffix: &str,
    target_ru: bool,
) -> (
    Harness,
    LayIbusEngine,
    crate::protocol::ExactManualToggleSuppression,
    usize,
) {
    let (mut harness, mut engine) = armed_exact_replay(tail, suffix, target_ru, false).await;
    let Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope)) =
        engine.committed_tail.autocorrect_suppression.clone()
    else {
        panic!("armed exact replay");
    };
    let mut serial = 23_000;
    for _ in scope.original_suffix.chars() {
        assert!(!legacy_key(&mut harness, &mut engine, serial, KEY_BACKSPACE, 14, 0).await);
        assert!(
            !legacy_key(
                &mut harness,
                &mut engine,
                serial + 1,
                KEY_BACKSPACE,
                14,
                RELEASE_MASK,
            )
            .await
        );
        no_legacy_text_output(&mut harness).await;
        serial += 2;
    }
    text_free_owned_soft_reset(&mut harness, &mut engine, 23_100).await;
    assert!(engine.context_reset_rereceipt.is_none());
    let inserted = scope.replacement.chars().count() - 2;
    for (i, ch) in scope.replacement.chars().take(inserted).enumerate() {
        let serial = 23_110 + 2 * i as u32;
        assert!(!legacy_key(&mut harness, &mut engine, serial, replay_keyval(ch), 30, 0,).await);
        assert!(
            !legacy_key(
                &mut harness,
                &mut engine,
                serial + 1,
                replay_keyval(ch),
                30,
                RELEASE_MASK,
            )
            .await
        );
        no_legacy_text_output(&mut harness).await;
    }
    text_free_owned_soft_reset(&mut harness, &mut engine, 23_150).await;
    assert!(engine.context_reset_rereceipt.is_some());
    assert!(engine.exact_replay_quarantine_active());
    (harness, engine, scope, inserted)
}

#[test]
fn td121_interleaved_replay_reset_retains_history_through_final_exact_receipt() {
    zbus::block_on(bounded(async {
        let mut results = Vec::new();
        for (tail, suffix, target_ru) in [
            ("ghbdtn", "ghbdtn", true),
            ("ntrcn ", "ntrcn ", true),
            ("prefix ghbdtn ", "ghbdtn ", true),
            ("привет", "привет", false),
            ("текст ", "текст ", false),
            ("prefix привет ", "привет ", false),
        ] {
            let (mut harness, mut engine, scope, inserted) =
                replay_prefix_then_reset(tail, suffix, target_ru).await;
            let predecessor = engine
                .context_reset_rereceipt
                .as_ref()
                .unwrap()
                .predecessor_token
                .clone();
            exact_replay_surrounding_receipt(&mut harness, &mut engine, &scope.unchanged_prefix)
                .await;
            let retained = engine
                .context_reset_rereceipt
                .as_ref()
                .is_some_and(|pending| {
                    !pending.confirmed && pending.predecessor_token == predecessor
                });
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert!(engine.capture_observed_suffix_display_frame().is_none());
            no_legacy_text_output(&mut harness).await;
            for (i, ch) in scope.replacement.chars().skip(inserted).enumerate() {
                let serial = 23_160 + 2 * i as u32;
                let code = if ch == ' ' { 57 } else { 30 };
                assert!(
                    !legacy_key(
                        &mut harness,
                        &mut engine,
                        serial,
                        replay_keyval(ch),
                        code,
                        0,
                    )
                    .await
                );
                assert!(
                    !legacy_key(
                        &mut harness,
                        &mut engine,
                        serial + 1,
                        replay_keyval(ch),
                        code,
                        RELEASE_MASK,
                    )
                    .await
                );
                no_legacy_text_output(&mut harness).await;
            }
            let replayed = format!("{}{}", scope.unchanged_prefix, scope.replacement);
            assert_eq!(engine.committed_tail.buffer, replayed);
            text_free_owned_soft_reset(&mut harness, &mut engine, 23_180).await;
            let late = format!(
                "{}{}",
                scope.unchanged_prefix,
                scope.replacement.chars().next().unwrap()
            );
            exact_replay_surrounding_receipt(&mut harness, &mut engine, &late).await;
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            text_free_owned_soft_reset(&mut harness, &mut engine, 23_182).await;
            exact_replay_surrounding_receipt(&mut harness, &mut engine, &replayed).await;
            let allowed = engine.context_reset_rereceipt_exact_manual_handoff_allowed();
            assert!(engine.committed_tail.pending_completion_learning.is_none());
            let (returned, snapshot) =
                super::residuals::cycle09_visible_tail(&mut harness, engine).await;
            let authoritative = super::residuals::cycle09_tail_is_authoritative(
                &snapshot,
                &returned.path,
                target_ru,
                &replayed,
            );
            let (_engine, toggle) =
                super::residuals::cycle09_manual_toggle(&mut harness, returned).await;
            results.push((
                tail,
                retained,
                allowed,
                authoritative,
                toggle == Ok((3, false)),
            ));
        }
        assert!(
            results
                .iter()
                .all(|(_, retained, allowed, snapshot, toggle)| *retained
                    && *allowed
                    && *snapshot
                    && *toggle),
            "{results:?}"
        );
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    }));
}

#[test]
fn td121_interleaved_replay_history_rejects_future_or_invalid_surfaces() {
    zbus::block_on(bounded(async {
        for gap in [
            "future",
            "foreign",
            "prefix",
            "selection",
            "cursor",
            "owner",
            "shared_scope",
            "expired",
        ] {
            let (mut harness, mut engine, scope, _) =
                replay_prefix_then_reset("prefix ghbdtn ", "ghbdtn ", true).await;
            let current = engine.committed_tail.buffer.clone();
            let mut snapshot = scope.unchanged_prefix.clone();
            match gap {
                "future" => snapshot.push_str(&scope.replacement),
                "foreign" => snapshot.push_str("unrelated"),
                "prefix" => snapshot = "foreign prefix ".to_string(),
                "owner" => {
                    super::residuals::global_engine_changed(&mut harness, 23_190, "foreign-ime")
                        .await;
                }
                "shared_scope" => engine.shared.lock().unwrap().autocorrect_suppression = None,
                "expired" => {
                    let Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope)) =
                        engine.committed_tail.autocorrect_suppression.as_mut()
                    else {
                        panic!("incomplete exact replay");
                    };
                    scope.expires_at = Instant::now() - Duration::from_millis(1);
                    engine.shared.lock().unwrap().autocorrect_suppression =
                        engine.committed_tail.autocorrect_suppression.clone();
                }
                _ => {}
            }
            let chars = snapshot.chars().count() as u32;
            let (cursor, anchor) = match gap {
                "selection" => (chars, chars - 1),
                "cursor" => (chars - 1, chars - 1),
                _ => (chars, chars),
            };
            super::residuals::surrounding_receipt(
                &mut harness,
                &mut engine,
                &snapshot,
                cursor,
                anchor,
            )
            .await;
            assert!(engine.context_reset_rereceipt.is_none(), "{gap}");
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{gap}"
            );
            // A later matching snapshot cannot resurrect the rejected receipt.
            exact_replay_surrounding_receipt(&mut harness, &mut engine, &current).await;
            assert!(engine.context_reset_rereceipt.is_none(), "{gap}");
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{gap}"
            );
            no_legacy_text_output(&mut harness).await;
            assert!(engine.committed_tail.pending_completion_learning.is_none());
        }

        // During deletion, a shorter unobserved source prefix is future work.
        let (mut harness, mut engine) =
            armed_exact_replay("prefix ghbdtn ", "ghbdtn ", true, false).await;
        let original = engine.committed_tail.buffer.clone();
        assert!(!legacy_key(&mut harness, &mut engine, 23_200, KEY_BACKSPACE, 14, 0).await);
        assert!(
            !legacy_key(
                &mut harness,
                &mut engine,
                23_201,
                KEY_BACKSPACE,
                14,
                RELEASE_MASK,
            )
            .await
        );
        no_legacy_text_output(&mut harness).await;
        let reached = engine.committed_tail.buffer.clone();
        for (text, expected) in [
            (original, true),
            (reached.clone(), true),
            (
                reached.chars().take(reached.chars().count() - 1).collect(),
                false,
            ),
        ] {
            let end = text.chars().count() as u32;
            assert_eq!(
                engine.exact_replay_contains_prior_snapshot(&SurroundingTextSnapshot::new(
                    text, end, end
                ),),
                expected
            );
        }
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    }));
}

#[test]
fn td121_aborted_second_exact_capture_does_not_consume_enter() {
    zbus::block_on(bounded(async {
        for (tail, suffix, expired) in [
            ("ghbdtn", "ghbdtn", false),
            ("ntrcn ", "ntrcn ", false),
            ("prefix ghbdtn ", "ghbdtn ", false),
            ("ghbdtn", "ghbdtn", true),
        ] {
            let (mut harness, mut engine, replayed) =
                run_exact_replay(tail, suffix, true, false).await;
            text_free_owned_soft_reset(&mut harness, &mut engine, 22_600).await;
            exact_replay_surrounding_receipt(&mut harness, &mut engine, &replayed).await;
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            if expired {
                let Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope)) =
                    engine.committed_tail.autocorrect_suppression.as_mut()
                else {
                    panic!("completed exact scope");
                };
                scope.expires_at = Instant::now() - Duration::from_millis(1);
                engine.shared.lock().unwrap().autocorrect_suppression =
                    engine.committed_tail.autocorrect_suppression.clone();
            }
            assert!(engine.exact_replay_quarantine_active());
            let epoch = engine.committed_tail.epoch;
            let (returned, toggle) =
                super::residuals::cycle09_manual_toggle(&mut harness, engine).await;
            engine = returned;
            assert_eq!(toggle, Ok((3, false)));
            assert_eq!(engine.committed_tail.epoch, epoch.wrapping_add(1));
            assert_eq!(engine.committed_tail.buffer, replayed);
            assert!(engine.context_reset_rereceipt.is_none());
            no_legacy_text_output(&mut harness).await;

            // The next capture aborts before arming replay: no second
            // suppression or physical edit occurs. An ordinary Enter must
            // reach the client despite the earlier completed replay.
            let mut client = replayed.clone();
            let handled = legacy_key(
                &mut harness,
                &mut engine,
                22_602,
                crate::protocol::KEY_ENTER,
                28,
                0,
            )
            .await;
            super::residuals::consume_detached_callback_reply(&mut harness.peer, 22_602).await;
            if !handled {
                client.push('\n');
            }
            assert!(!handled, "completed replay consumed Enter for {tail:?}");
            assert_eq!(client, format!("{replayed}\n"));
            assert!(
                !legacy_key(
                    &mut harness,
                    &mut engine,
                    22_603,
                    crate::protocol::KEY_ENTER,
                    28,
                    RELEASE_MASK,
                )
                .await
            );
            super::residuals::consume_detached_callback_reply(&mut harness.peer, 22_603).await;
            no_legacy_text_output(&mut harness).await;
            assert!(engine.committed_tail.pending_completion_learning.is_none());
        }
    }));
}

#[test]
fn td121_next_exact_handoff_cannot_retire_incomplete_or_mismatched_replay() {
    zbus::block_on(bounded(async {
        for incomplete in [true, false] {
            let (mut harness, mut engine) = if incomplete {
                armed_exact_replay("ghbdtn", "ghbdtn", true, false).await
            } else {
                let (harness, engine, _) = run_exact_replay("ghbdtn", "ghbdtn", true, false).await;
                let mut state = engine.shared.lock().unwrap();
                let Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope)) =
                    state.autocorrect_suppression.as_mut()
                else {
                    panic!("completed shared exact scope");
                };
                scope.epoch = scope.epoch.wrapping_add(1);
                drop(state);
                (harness, engine)
            };
            let local = engine.committed_tail.autocorrect_suppression.clone();
            let shared = engine
                .shared
                .lock()
                .unwrap()
                .autocorrect_suppression
                .clone();
            let before = engine.committed_tail.buffer.clone();
            engine.prepare_exact_manual_toggle_layout_handoff();
            assert_eq!(engine.committed_tail.autocorrect_suppression, local);
            assert_eq!(
                engine.shared.lock().unwrap().autocorrect_suppression,
                shared
            );
            assert_eq!(engine.committed_tail.buffer, before);
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    22_620,
                    crate::protocol::KEY_ENTER,
                    28,
                    0,
                )
                .await
            );
            assert_eq!(engine.committed_tail.buffer, before);
            no_legacy_text_output(&mut harness).await;
            assert!(engine.committed_tail.pending_completion_learning.is_none());
        }
    }));
}

#[test]
fn completed_expired_exact_replay_cannot_consume_the_next_ordinary_key() {
    zbus::block_on(async {
        let (mut harness, mut engine, replayed) =
            run_exact_replay("ghbdtn", "ghbdtn", true, false).await;
        let expired = match engine.committed_tail.autocorrect_suppression.as_mut() {
            Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope)) => {
                scope.expires_at = Instant::now() - Duration::from_millis(1);
                scope.clone()
            }
            _ => panic!("completed exact replay scope"),
        };
        {
            let mut shared = engine.shared.lock().unwrap();
            shared.autocorrect_suppression = Some(
                crate::protocol::AutocorrectSuppression::ExactReplay(expired),
            );
        }
        exact_replay_surrounding_receipt(&mut harness, &mut engine, &replayed).await;
        assert!(engine.exact_replay_quarantine_active());
        assert!(engine.composition.preedit_candidates.is_empty());
        assert!(engine.composition.pending_display_frame.is_none());
        assert!(legacy_key(&mut harness, &mut engine, 21_220, replay_keyval('ф'), 30, 0,).await);
        let effects = legacy_effects(&mut harness).await;
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].header().member().unwrap().as_str(), "CommitText");
        assert_eq!(engine.committed_tail.buffer, format!("{replayed}ф"));
    });
}

#[test]
fn terminal_delivery_legacy_native_glyphs_preserve_mirror_without_commit_or_release_capture() {
    zbus::block_on(async {
        for width in [2, 11, 0, 22] {
            let (mut harness, mut engine) = known_terminal(width).await;
            let mut expected = String::from(" ");
            for (index, (keyval, keycode, glyph)) in [
                (0x06c1, 30, 'а'),
                (0x0100_042f, 21, 'Я'),
                ('Q' as u32, 16, 'Q'),
                ('b' as u32, 48, 'b'),
            ]
            .into_iter()
            .enumerate()
            {
                let serial = 20_010 + 2 * index as u32;
                assert!(
                    !legacy_key(&mut harness, &mut engine, serial, keyval, keycode, 0).await,
                    "ordinary native glyph, cursor width {width}"
                );
                expected.push(glyph);
                assert_eq!(engine.committed_tail.buffer, expected);
                assert!(engine.context_word_is_known());
                assert!(
                    !legacy_key(
                        &mut harness,
                        &mut engine,
                        serial + 1,
                        keyval,
                        keycode,
                        RELEASE_MASK,
                    )
                    .await
                );
                no_legacy_output(&mut harness).await;
            }
        }
    });
}

#[test]
fn terminal_delivery_first_atomic_frame_retires_legacy_native_word_mode() {
    zbus::block_on(async {
        let (mut harness, mut engine) = known_terminal(11).await;
        assert!(!legacy_key(&mut harness, &mut engine, 20_030, 'a' as u32, 30, 0).await);
        no_legacy_output(&mut harness).await;
        assert_eq!(engine.committed_tail.buffer, " a");
        let proposal = atomic_callback(
            &mut harness,
            &mut engine,
            (20_031, 20_031),
            'b' as u32,
            48,
            0,
            (0, 0, Vec::new()),
        )
        .await;
        assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
        assert_eq!(committed_texts(&proposal), ["b"]);
        assert!(engine.atomic.active);
        assert_eq!(engine.committed_tail.buffer, " a", "unreceipted proposal");
    });
}

#[test]
fn terminal_delivery_purpose_and_capability_changes_preserve_transport_and_word_lifetime() {
    zbus::block_on(async {
        let (mut harness, mut engine) = known_terminal(11).await;
        for (index, (key, code, purpose, caps, native)) in [
            ('a', 30, 10, 9, true),
            ('b', 48, 0, 9, false),
            ('c', 46, 10, 9, true),
            ('d', 32, 10, 41, false),
            // Losing SurroundingText must keep the already managed word.
            ('e', 18, 10, 9, false),
            (' ', 57, 10, 9, false),
            ('f', 33, 10, 9, true),
        ]
        .into_iter()
        .enumerate()
        {
            engine.set_content_type_state(purpose, 0);
            engine.set_client_capabilities(caps);
            assert_eq!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    20_040 + index as u32,
                    key as u32,
                    code,
                    0
                )
                .await,
                !native,
                "key {key}, purpose {purpose}, caps {caps}"
            );
            if native {
                no_legacy_output(&mut harness).await;
            } else {
                let effects = legacy_effects(&mut harness).await;
                assert_eq!(effects.len(), 1);
                let effect = &effects[0];
                assert_eq!(effect.header().member().unwrap().as_str(), "CommitText");
                let body = effect.body();
                let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
                assert_eq!(
                    crate::ibus_interface::ibus_text_value_to_string(&value),
                    Some(key.to_string())
                );
            }
        }
        assert_eq!(engine.committed_tail.buffer, " abcde f");
    });
}

#[test]
fn terminal_delivery_unknown_first_word_stays_native_and_autocorrects_exact_suffix_on_space() {
    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("exact preparation available");
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        engine.config.auto_replace = true;
        engine.config.auto_switch_layout = true;
        engine.config.nanda_precognition = false;
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_content_type_state(10, 0);
        engine.set_client_capabilities(1 | 1 << 3);
        engine.client_context.cursor_cell_width = 11;
        assert!(!engine.context_word_is_known());

        for (index, (key, code)) in [
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
            let serial = 24_500 + index as u32 * 2;
            assert!(!legacy_key(&mut harness, &mut engine, serial, key as u32, code, 0).await);
            no_legacy_output(&mut harness).await;
            assert!(
                !legacy_key(
                    &mut harness,
                    &mut engine,
                    serial + 1,
                    key as u32,
                    code,
                    RELEASE_MASK,
                )
                .await
            );
            no_legacy_output(&mut harness).await;
            assert!(engine.composition.buffer.is_empty());
            assert!(!engine.composition.legacy_word_preedit_active);
        }

        assert_eq!(engine.committed_tail.buffer, "ghbdtn");
        assert!(engine.capture_input_frame_identity().is_none());
        let frame = engine
            .capture_space_autocorrect_frame_identity()
            .expect("exact native terminal suffix frame");
        assert!(frame.display_suffix_token.is_none());
        assert!(frame.space_autocorrect_suffix_token.is_some());
        engine.client_context.cursor_cell_width = 0;
        assert!(engine.capture_space_autocorrect_frame_identity().is_none());
        engine.client_context.cursor_cell_width = 11;
        assert_eq!(
            engine.capture_space_autocorrect_frame_identity(),
            Some(frame.clone())
        );
        crate::space_autocorrect_prefetch::proof::install_exact_lease(&frame, &engine.config);

        assert!(legacy_key(&mut harness, &mut engine, 24_520, KEY_SPACE, 57, 0).await);
        let effects = legacy_effects(&mut harness).await;
        assert_eq!(effects.len(), 1, "one terminal replacement frame");
        let effect = &effects[0];
        assert_eq!(effect.header().member().unwrap().as_str(), "CommitText");
        let body = effect.body();
        let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
        assert_eq!(
            crate::ibus_interface::ibus_text_value_to_string(&value),
            Some("\u{7f}".repeat(6) + "привет ")
        );
        assert_eq!(engine.committed_tail.buffer, "привет ");
        assert!(engine.composition.buffer.is_empty());
        assert!(!engine.composition.legacy_word_preedit_active);

        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                24_521,
                KEY_SPACE,
                57,
                RELEASE_MASK,
            )
            .await
        );
        no_legacy_output(&mut harness).await;
    });
}

#[test]
fn terminal_delivery_unknown_first_word_stays_native_and_runs_full_typo_correction_on_space() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        engine.config.auto_replace = true;
        engine.config.auto_switch_layout = true;
        engine.config.nanda_autocorrect = true;
        engine.config.nanda_precognition = false;
        engine.config.correction_safety = "experimental".into();
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_content_type_state(10, 0);
        engine.set_client_capabilities(1 | 1 << 3);
        engine.client_context.cursor_cell_width = 11;
        engine.set_layout_is_ru(true);

        for (index, (ch, code)) in [
            ('р', 35),
            ('а', 33),
            ('б', 51),
            ('о', 36),
            ('а', 33),
            ('е', 20),
            ('т', 49),
        ]
        .into_iter()
        .enumerate()
        {
            let serial = 24_600 + index as u32 * 2;
            let keyval = replay_keyval(ch);
            assert!(!legacy_key(&mut harness, &mut engine, serial, keyval, code, 0).await);
            no_legacy_output(&mut harness).await;
            assert!(
                !legacy_key(
                    &mut harness,
                    &mut engine,
                    serial + 1,
                    keyval,
                    code,
                    RELEASE_MASK,
                )
                .await
            );
            no_legacy_output(&mut harness).await;
            assert!(engine.composition.buffer.is_empty());
            assert!(!engine.composition.legacy_word_preedit_active);
        }

        assert_eq!(engine.committed_tail.buffer, "рабоает");
        assert!(engine.capture_input_frame_identity().is_none());
        let frame = engine
            .capture_space_autocorrect_frame_identity()
            .expect("exact native terminal suffix frame");
        assert!(frame.display_suffix_token.is_none());
        assert!(frame.space_autocorrect_suffix_token.is_some());
        crate::space_autocorrect_prefetch::proof::install_full_lease(&frame, &engine.config);

        assert!(legacy_key(&mut harness, &mut engine, 24_620, KEY_SPACE, 57, 0).await);
        let effects = legacy_effects(&mut harness).await;
        assert_eq!(effects.len(), 1, "one terminal replacement frame");
        let effect = &effects[0];
        assert_eq!(effect.header().member().unwrap().as_str(), "CommitText");
        let body = effect.body();
        let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
        assert_eq!(
            crate::ibus_interface::ibus_text_value_to_string(&value),
            Some("\u{7f}".repeat(7) + "работает ")
        );
        assert_eq!(engine.committed_tail.buffer, "работает ");
        assert!(engine.composition.buffer.is_empty());
        assert!(!engine.composition.legacy_word_preedit_active);

        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                24_621,
                KEY_SPACE,
                57,
                RELEASE_MASK,
            )
            .await
        );
        no_legacy_output(&mut harness).await;
    });
}

#[test]
fn terminal_delivery_chrome_unknown_first_word_autocorrects_owned_preedit_on_space() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        engine.config.auto_replace = true;
        engine.config.auto_switch_layout = true;
        engine.config.nanda_autocorrect = true;
        engine.config.nanda_precognition = false;
        engine.config.correction_safety = "experimental".into();
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_content_type_state(0, 0);
        engine.set_client_capabilities(1 | 1 << 3);
        engine.set_layout_is_ru(true);

        for (index, (ch, code)) in [
            ('р', 35),
            ('а', 33),
            ('б', 51),
            ('о', 36),
            ('а', 33),
            ('е', 20),
            ('т', 49),
        ]
        .into_iter()
        .enumerate()
        {
            let serial = 24_700 + index as u32 * 2;
            let keyval = replay_keyval(ch);
            assert!(legacy_key(&mut harness, &mut engine, serial, keyval, code, 0).await);
            let effects = legacy_effects(&mut harness).await;
            assert!(effects.iter().any(|effect| {
                effect
                    .header()
                    .member()
                    .is_some_and(|member| member.as_str() == "UpdatePreeditText")
            }));
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    serial + 1,
                    keyval,
                    code,
                    RELEASE_MASK,
                )
                .await
            );
            no_legacy_output(&mut harness).await;
            if index == 0 {
                // Chromium advertises caps=9 while the first printable starts
                // the owned preedit, then caps=41 for the remaining token.
                engine.set_client_capabilities(1 | 1 << 3 | 1 << 5);
            }
        }

        assert!(!engine.context_word_is_known());
        assert!(engine.capture_input_frame_identity().is_none());
        assert!(engine.composition.legacy_word_preedit_active);
        assert_eq!(engine.composition.buffer, "рабоает");
        assert_eq!(engine.last_tail_token_text(), "рабоает");
        let frame = engine
            .capture_space_autocorrect_frame_identity()
            .expect("exact owned-preedit Space frame");
        assert!(frame.display_suffix_token.is_none());
        assert!(frame.space_autocorrect_suffix_token.is_some());
        assert!(frame.lexical_coordinates.is_some());
        let scheduled = crate::space_autocorrect_prefetch::take_with_budget(&frame, Duration::ZERO);
        assert!(
            !matches!(
                scheduled.lookup,
                crate::space_autocorrect_prefetch::SpaceAutocorrectLookup::Stale
            ),
            "owned-preedit Space work must bind the post-settlement callback frame"
        );
        crate::space_autocorrect_prefetch::proof::install_full_lease(&frame, &engine.config);

        assert!(legacy_key(&mut harness, &mut engine, 24_720, KEY_SPACE, 57, 0).await);
        let effects = legacy_effects(&mut harness).await;
        let commits = effects
            .iter()
            .filter(|effect| {
                effect
                    .header()
                    .member()
                    .is_some_and(|member| member.as_str() == "CommitText")
            })
            .map(|effect| {
                let body = effect.body();
                let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
                crate::ibus_interface::ibus_text_value_to_string(&value).unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(commits, ["работает "]);
        assert_eq!(engine.committed_tail.buffer, "работает ");
        assert!(engine.composition.buffer.is_empty());
        assert!(!engine.composition.legacy_word_preedit_active);
    });
}

#[test]
fn terminal_delivery_chrome_owned_preedit_schedules_unknown_start_suggestion() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        engine.config.nanda_precognition = true;
        engine.config.correction_safety = "experimental".into();
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_content_type_state(0, 0);
        engine.set_client_capabilities(1 | 1 << 3);
        engine.set_layout_is_ru(true);

        assert!(legacy_key(&mut harness, &mut engine, 24_740, replay_keyval('п'), 34, 0).await);
        let effects = legacy_effects(&mut harness).await;
        assert!(effects.iter().any(|effect| effect
            .header()
            .member()
            .is_some_and(|member| member.as_str() == "ShowPreeditText")));
        assert!(!engine.context_word_is_known());
        let first = engine
            .capture_space_autocorrect_frame_identity()
            .expect("owned preedit frame");
        assert!(engine.precognition_identity_matches(&first));
        assert!(
            engine.precognition_causal_counts().0 > 0,
            "owned preedit did not schedule display work"
        );

        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                24_741,
                replay_keyval('п'),
                34,
                RELEASE_MASK
            )
            .await
        );
        no_legacy_output(&mut harness).await;
        engine.set_client_capabilities(1 | 1 << 3 | 1 << 5);
        assert!(legacy_key(&mut harness, &mut engine, 24_742, replay_keyval('у'), 18, 0).await);
        let effects = legacy_effects(&mut harness).await;
        assert!(effects.iter().any(|effect| effect
            .header()
            .member()
            .is_some_and(|member| member.as_str() == "UpdatePreeditText")));
        assert!(effects.iter().all(|effect| effect
            .header()
            .member()
            .is_none_or(|member| member.as_str() != "HidePreeditText")));
        let second = engine
            .capture_space_autocorrect_frame_identity()
            .expect("advanced owned preedit frame");
        assert!(engine.precognition_identity_matches(&second));
        assert!(engine.precognition_causal_counts().0 > 1);
        let mut stale = second.clone();
        stale.tail_epoch = stale.tail_epoch.wrapping_add(1);
        assert!(!engine.precognition_identity_matches(&stale));
    });
}

async fn browser_managed_exact_snapshots_autocorrect_after_admission_loss(
    lose_admission_after: usize,
) {
    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("exact preparation available");
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = new_engine(&harness);
    engine.config.auto_replace = true;
    engine.config.auto_switch_layout = true;
    engine.config.nanda_autocorrect = true;
    engine.config.nanda_precognition = false;
    engine.config.correction_safety = "experimental".into();
    start_source_free_unknown(&mut harness, &mut engine).await;
    engine.set_content_type_state(0, 0);
    engine.set_client_capabilities(
        1 | 1 << 3 | 1 << 5 | crate::window_interaction::IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH,
    );
    engine.set_layout_is_ru(true);

    for (index, (ch, code)) in [
        ('р', 35),
        ('а', 33),
        ('б', 51),
        ('о', 36),
        ('а', 33),
        ('е', 20),
        ('т', 49),
    ]
    .into_iter()
    .enumerate()
    {
        let serial = 24_760 + index as u32 * 2;
        let keyval = replay_keyval(ch);
        assert!(legacy_key(&mut harness, &mut engine, serial, keyval, code, 0).await);
        let effects = legacy_effects(&mut harness).await;
        let commits = effects
            .iter()
            .filter(|effect| {
                effect
                    .header()
                    .member()
                    .is_some_and(|member| member.as_str() == "CommitText")
            })
            .map(|effect| {
                let body = effect.body();
                let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
                crate::ibus_interface::ibus_text_value_to_string(&value).unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(commits, [ch.to_string()]);
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                serial + 1,
                keyval,
                code,
                RELEASE_MASK,
            )
            .await
        );
        no_legacy_output(&mut harness).await;

        if index >= lose_admission_after {
            engine.context_token = None;
            engine.context_word_scope = None;
            engine.context_reset_rereceipt = None;
        }
        let visible = engine.committed_tail.buffer.clone();
        exact_replay_surrounding_receipt(&mut harness, &mut engine, &visible).await;
    }

    assert!(!engine.context_word_is_known());
    assert_eq!(engine.committed_tail.buffer, "рабоает");
    assert!(engine.composition.buffer.is_empty());
    assert!(engine
        .client_context
        .surrounding_text_snapshot
        .as_ref()
        .is_some_and(|snapshot| !snapshot.has_selection() && snapshot.text == "рабоает"));

    let first_exact_frame = engine
        .capture_space_autocorrect_frame_identity()
        .expect("first exact bounded managed-widget Space frame");
    super::residuals::surrounding_receipt(&mut harness, &mut engine, "xрабоает", 8, 8).await;
    assert!(
        engine.capture_space_autocorrect_frame_identity().is_none(),
        "a non-boundary character to the left must keep the suffix read-only"
    );
    super::residuals::surrounding_receipt(&mut harness, &mut engine, "рабоаетx", 7, 7).await;
    assert!(
        engine.capture_space_autocorrect_frame_identity().is_none(),
        "a non-boundary character to the right must keep the prefix read-only"
    );
    super::residuals::surrounding_receipt(&mut harness, &mut engine, "рабоает", 7, 6).await;
    assert!(
        engine.capture_space_autocorrect_frame_identity().is_none(),
        "a selection must never authorize whole-word correction"
    );
    exact_replay_surrounding_receipt(&mut harness, &mut engine, "рабоает").await;
    let frame = engine
        .capture_space_autocorrect_frame_identity()
        .expect("exact bounded managed-widget Space frame after admission loss");
    assert_ne!(
        first_exact_frame.space_autocorrect_surrounding_revision,
        frame.space_autocorrect_surrounding_revision,
        "an equal-text rereceipt must create a fresh snapshot witness"
    );
    assert!(frame.display_suffix_token.is_none());
    assert!(frame.space_autocorrect_suffix_token.is_none());
    assert_eq!(
        frame.space_autocorrect_surrounding_revision,
        Some(engine.client_context.surrounding_observation_revision)
    );
    crate::space_autocorrect_prefetch::proof::install_full_lease(&frame, &engine.config);

    assert!(legacy_key(&mut harness, &mut engine, 24_790, KEY_SPACE, 57, 0).await);
    let effects = legacy_effects(&mut harness).await;
    let members = effects
        .iter()
        .filter_map(|effect| {
            effect
                .header()
                .member()
                .map(|member| member.as_str().to_string())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        members
            .iter()
            .filter(|member| member.as_str() == "DeleteSurroundingText")
            .count(),
        1
    );
    let commits = effects
        .iter()
        .filter(|effect| {
            effect
                .header()
                .member()
                .is_some_and(|member| member.as_str() == "CommitText")
        })
        .map(|effect| {
            let body = effect.body();
            let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
            crate::ibus_interface::ibus_text_value_to_string(&value).unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(commits, ["работает "]);
    assert_eq!(engine.committed_tail.buffer, "работает ");
}

#[test]
fn terminal_delivery_browser_exact_snapshots_restore_space_frame_after_callback_admission_loss() {
    zbus::block_on(async {
        // Chrome refuses the printable callback lineage from the first key;
        // Firefox loses its Reset/rereceipt lineage later in the same word.
        browser_managed_exact_snapshots_autocorrect_after_admission_loss(0).await;
        browser_managed_exact_snapshots_autocorrect_after_admission_loss(3).await;
    });
}

#[test]
fn terminal_delivery_browser_delayed_surrounding_uses_proved_managed_word_start_on_space() {
    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("exact preparation available");
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        engine.config.auto_replace = true;
        engine.config.auto_switch_layout = true;
        engine.config.nanda_autocorrect = true;
        engine.config.nanda_precognition = false;
        engine.config.correction_safety = "experimental".into();
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_content_type_state(0, 0);
        engine.set_client_capabilities(
            1 | 1 << 3 | 1 << 5 | crate::window_interaction::IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH,
        );
        engine.set_layout_is_ru(true);
        exact_replay_surrounding_receipt(&mut harness, &mut engine, "").await;
        assert!(
            engine.client_context.managed_word_start.is_some(),
            "exact empty boundary must arm managed-word-start witness"
        );

        for (index, (ch, code)) in [
            ('р', 35),
            ('а', 33),
            ('б', 51),
            ('о', 36),
            ('а', 33),
            ('е', 20),
            ('т', 49),
        ]
        .into_iter()
        .enumerate()
        {
            let serial = 24_800 + index as u32 * 2;
            let keyval = replay_keyval(ch);
            assert!(legacy_key(&mut harness, &mut engine, serial, keyval, code, 0).await);
            let effects = legacy_effects(&mut harness).await;
            assert!(effects.iter().any(|effect| {
                effect
                    .header()
                    .member()
                    .is_some_and(|member| member.as_str() == "CommitText")
            }));
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    serial + 1,
                    keyval,
                    code,
                    RELEASE_MASK,
                )
                .await
            );
            no_legacy_output(&mut harness).await;
            engine.context_token = None;
            engine.context_word_scope = None;
            engine.context_reset_rereceipt = None;
        }

        assert_eq!(engine.committed_tail.buffer, "рабоает");
        assert!(
            engine.managed_word_start_is_current(),
            "managed word-start chain lost: witness={:?} mode={:?} epoch={} tail={:?} focus={} owner={} layout_generation={} surrounding_revision={} sealed={} atomic={} composition={:?}",
            engine.client_context.managed_word_start,
            engine.composition.word_input_mode,
            engine.committed_tail.epoch,
            engine.committed_tail.buffer,
            engine.client_context.focus_serial,
            engine.client_context.runtime_owner_lease_identity,
            engine.layout_gesture.layout_generation,
            engine.client_context.surrounding_observation_revision,
            engine.context_handoff_sealed,
            engine.atomic.active,
            engine.composition.buffer,
        );
        let frame = engine
            .capture_space_autocorrect_frame_identity()
            .expect("proved managed-word-start Space frame before delayed final snapshot");
        assert!(frame.space_autocorrect_managed_start_identity.is_some());
        assert!(frame.space_autocorrect_surrounding_revision.is_none());
        assert!(frame.space_autocorrect_suffix_token.is_none());
        assert_eq!(
            engine.managed_word_start_projected_snapshot(),
            Some(SurroundingTextSnapshot::new("рабоает".to_string(), 7, 7,))
        );

        // Any key returned to the client can mutate text or move the caret
        // before a delayed surrounding callback. All such routes must revoke
        // both the projected witness and already prepared Space work.
        for (route, keyval, keycode, state) in [
            ("command", 'v' as u32, 55, 1 << 2),
            ("tab", KEY_TAB, 15, 0),
            ("candidate_navigation", crate::protocol::KEY_UP, 103, 0),
            ("generic_navigation", 0xff50, 110, 0),
            ("cursor", KEY_LEFT, 105, 0),
        ] {
            let mut relinquished = engine.clone();
            crate::space_autocorrect_prefetch::proof::install_full_lease(
                &frame,
                &relinquished.config,
            );
            let mut key_output = crate::output::TestEngineOutput::default();
            let handled = relinquished
                .process_pressed_key(
                    &mut crate::output::EngineOutput::test(&mut key_output),
                    keyval,
                    keycode,
                    state,
                )
                .await
                .unwrap_or_else(|error| panic!("{route} key failed: {error}"));
            assert!(!handled, "{route} must remain client-owned");
            assert!(
                relinquished.client_context.managed_word_start.is_none(),
                "{route} retained projected word-start authority"
            );
            assert!(
                relinquished
                    .capture_space_autocorrect_frame_identity()
                    .is_none(),
                "{route} retained a Space correction frame"
            );
            let revoked =
                crate::space_autocorrect_prefetch::take_with_budget(&frame, Duration::ZERO);
            assert!(
                !matches!(
                    revoked.lookup,
                    crate::space_autocorrect_prefetch::SpaceAutocorrectLookup::Ready(_)
                ),
                "{route} retained prepared Space work"
            );

            let mut space_output = crate::output::TestEngineOutput::default();
            assert!(relinquished
                .process_pressed_key(
                    &mut crate::output::EngineOutput::test(&mut space_output),
                    KEY_SPACE,
                    57,
                    0,
                )
                .await
                .unwrap_or_else(|error| panic!("{route} Space failed: {error}")));
            assert!(
                space_output.surrounding_deletes.is_empty(),
                "{route} allowed projected deletion after client-owned input"
            );
            assert_eq!(space_output.committed_texts, [" "]);
        }

        let mut disabled = engine.clone();
        crate::space_autocorrect_prefetch::proof::install_full_lease(&frame, &disabled.config);
        disabled.config.text_backend = "uinput".to_string();
        let mut disabled_output = crate::output::TestEngineOutput::default();
        assert!(!disabled
            .process_key_event_with_output(
                &mut crate::output::EngineOutput::test(&mut disabled_output),
                'v' as u32,
                55,
                0,
            )
            .await
            .expect("disabled-composition client key"));
        assert!(disabled.client_context.managed_word_start.is_none());
        assert!(disabled
            .capture_space_autocorrect_frame_identity()
            .is_none());
        assert!(!matches!(
            crate::space_autocorrect_prefetch::take_with_budget(&frame, Duration::ZERO).lookup,
            crate::space_autocorrect_prefetch::SpaceAutocorrectLookup::Ready(_)
        ));
        disabled.config.text_backend = "ime".to_string();
        let mut disabled_space_output = crate::output::TestEngineOutput::default();
        assert!(disabled
            .process_key_event_with_output(
                &mut crate::output::EngineOutput::test(&mut disabled_space_output),
                KEY_SPACE,
                57,
                0,
            )
            .await
            .expect("Space after disabled-composition client key"));
        assert!(disabled_space_output.surrounding_deletes.is_empty());
        assert_eq!(disabled_space_output.committed_texts, [" "]);

        let mut alt_release = engine.clone();
        alt_release.layout_gesture.alt_completion_active = true;
        alt_release.layout_gesture.alt_used_as_modifier = false;
        crate::space_autocorrect_prefetch::proof::install_full_lease(&frame, &alt_release.config);
        let mut alt_output = crate::output::TestEngineOutput::default();
        assert!(!alt_release
            .process_key_event_with_output(
                &mut crate::output::EngineOutput::test(&mut alt_output),
                crate::protocol::KEY_LEFT_ALT,
                64,
                RELEASE_MASK,
            )
            .await
            .expect("standalone Alt completion release"));
        assert!(alt_release.client_context.managed_word_start.is_none());
        assert!(alt_release
            .capture_space_autocorrect_frame_identity()
            .is_none());
        assert!(!matches!(
            crate::space_autocorrect_prefetch::take_with_budget(&frame, Duration::ZERO).lookup,
            crate::space_autocorrect_prefetch::SpaceAutocorrectLookup::Ready(_)
        ));
        let mut alt_space_output = crate::output::TestEngineOutput::default();
        assert!(alt_release
            .process_key_event_with_output(
                &mut crate::output::EngineOutput::test(&mut alt_space_output),
                KEY_SPACE,
                57,
                0,
            )
            .await
            .expect("Space after standalone Alt completion release"));
        assert!(alt_space_output.surrounding_deletes.is_empty());
        assert_eq!(alt_space_output.committed_texts, [" "]);

        let mut equal_text_aba = engine.clone();
        equal_text_aba.committed_tail.epoch = equal_text_aba.committed_tail.epoch.wrapping_add(2);
        assert!(equal_text_aba
            .capture_space_autocorrect_frame_identity()
            .is_none());

        let mut changed_focus = engine.clone();
        changed_focus.client_context.focus_serial = crate::engine::next_input_identity();
        assert!(changed_focus
            .capture_space_autocorrect_frame_identity()
            .is_none());

        let mut changed_owner = engine.clone();
        changed_owner.client_context.runtime_owner_lease_identity =
            crate::engine::next_input_identity();
        assert!(changed_owner
            .capture_space_autocorrect_frame_identity()
            .is_none());

        let mut selected = engine.clone();
        selected.observe_external_surrounding_text(Some(SurroundingTextSnapshot::new(
            "рабоает".to_string(),
            7,
            6,
        )));
        assert!(selected.client_context.managed_word_start.is_none());

        let mut moved_cursor = engine.clone();
        moved_cursor.observe_external_surrounding_text(Some(SurroundingTextSnapshot::new(
            "рабоает".to_string(),
            6,
            6,
        )));
        assert!(moved_cursor.client_context.managed_word_start.is_none());

        crate::space_autocorrect_prefetch::proof::install_full_lease(&frame, &engine.config);

        assert!(legacy_key(&mut harness, &mut engine, 24_820, KEY_SPACE, 57, 0).await);
        let effects = legacy_effects(&mut harness).await;
        assert_eq!(
            effects
                .iter()
                .filter(|effect| effect
                    .header()
                    .member()
                    .is_some_and(|member| member.as_str() == "DeleteSurroundingText"))
                .count(),
            1
        );
        let commits = effects
            .iter()
            .filter(|effect| {
                effect
                    .header()
                    .member()
                    .is_some_and(|member| member.as_str() == "CommitText")
            })
            .map(|effect| {
                let body = effect.body();
                let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
                crate::ibus_interface::ibus_text_value_to_string(&value).unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(commits, ["работает "]);
        assert_eq!(engine.committed_tail.buffer, "работает ");
    });
}

#[test]
fn terminal_delivery_midword_exact_snapshot_survives_owned_commit_reset_echo() {
    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("exact preparation available");
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        engine.config.auto_replace = true;
        engine.config.auto_switch_layout = true;
        engine.config.nanda_autocorrect = true;
        engine.config.nanda_precognition = false;
        engine.config.correction_safety = "experimental".into();
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_content_type_state(0, 0);
        engine.set_client_capabilities(
            1 | 1 << 3 | 1 << 5 | crate::window_interaction::IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH,
        );
        engine.set_layout_is_ru(true);

        let first = ('р', 35);
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                25_000,
                replay_keyval(first.0),
                first.1,
                0
            )
            .await
        );
        let effects = legacy_effects(&mut harness).await;
        assert!(effects.iter().any(|effect| effect
            .header()
            .member()
            .is_some_and(|member| member.as_str() == "CommitText")));
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                25_001,
                replay_keyval(first.0),
                first.1,
                RELEASE_MASK
            )
            .await
        );
        no_legacy_output(&mut harness).await;
        assert!(engine.client_context.managed_word_start.is_none());

        // The first exact client receipt arrives after the word has started.
        exact_replay_surrounding_receipt(&mut harness, &mut engine, "р").await;
        assert!(engine.managed_word_start_is_current());
        assert_eq!(
            engine.managed_word_start_projected_snapshot(),
            Some(SurroundingTextSnapshot::new("р".to_string(), 1, 1))
        );

        for (index, (ch, code)) in [
            ('а', 33),
            ('б', 51),
            ('о', 36),
            ('а', 33),
            ('е', 20),
            ('т', 49),
        ]
        .into_iter()
        .enumerate()
        {
            let serial = 25_002 + index as u32 * 2;
            let keyval = replay_keyval(ch);
            assert!(legacy_key(&mut harness, &mut engine, serial, keyval, code, 0).await);
            let effects = legacy_effects(&mut harness).await;
            assert!(effects.iter().any(|effect| effect
                .header()
                .member()
                .is_some_and(|member| member.as_str() == "CommitText")));
            assert!(engine.managed_word_start_is_current());
            let frame = engine
                .capture_space_autocorrect_frame_identity()
                .expect("proved frame before Reset");
            engine.reset_for_ibus_soft_reset();
            assert!(
                engine.managed_word_start_is_current(),
                "owned CommitText Reset lost word-start proof"
            );
            assert_eq!(
                engine.capture_space_autocorrect_frame_identity(),
                Some(frame)
            );
            // Firefox's Reset can arrive before the release of the same
            // managed key. That release must remain owned by the engine.
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    serial + 1,
                    keyval,
                    code,
                    RELEASE_MASK
                )
                .await
            );
            no_legacy_output(&mut harness).await;
            assert!(
                engine.managed_word_start_is_current(),
                "release after owned Reset lost word-start proof"
            );
        }
        assert_eq!(engine.committed_tail.buffer, "рабоает");
        let frame = engine
            .capture_space_autocorrect_frame_identity()
            .expect("final Space frame");
        assert_eq!(
            engine.managed_word_start_projected_snapshot(),
            Some(SurroundingTextSnapshot::new("рабоает".to_string(), 7, 7))
        );
        let mut duplicate_reset = engine.clone();
        let mut expired_echo = engine.clone();
        crate::space_autocorrect_prefetch::proof::install_full_lease(&frame, &engine.config);
        assert!(legacy_key(&mut harness, &mut engine, 25_020, KEY_SPACE, 57, 0).await);
        let effects = legacy_effects(&mut harness).await;
        assert_eq!(
            effects
                .iter()
                .filter(|effect| effect
                    .header()
                    .member()
                    .is_some_and(|member| member.as_str() == "DeleteSurroundingText"))
                .count(),
            1
        );
        assert_eq!(engine.committed_tail.buffer, "работает ");

        // The owned echo is one-use, and a delayed Reset cannot preserve a
        // handled release or revive Space authority for an old CommitText.
        duplicate_reset.reset_for_ibus_soft_reset();
        assert!(duplicate_reset
            .capture_space_autocorrect_frame_identity()
            .is_none());
        expired_echo.arm_managed_commit_reset_echo();
        expired_echo.committed_tail.last_commit_at =
            Some(Instant::now() - Duration::from_millis(701));
        expired_echo
            .layout_gesture
            .handled_press_keycodes
            .insert(49);
        expired_echo.reset_for_ibus_soft_reset();
        assert!(!expired_echo
            .layout_gesture
            .handled_press_keycodes
            .contains(&49));
        assert!(expired_echo
            .capture_space_autocorrect_frame_identity()
            .is_none());
    });
}

#[test]
fn terminal_delivery_firefox_exact_refresh_keeps_prefix_committed_and_schedules_space() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        engine.config.auto_replace = true;
        engine.config.auto_switch_layout = true;
        engine.config.nanda_autocorrect = true;
        engine.config.nanda_precognition = false;
        engine.config.correction_safety = "experimental".into();
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_content_type_state(0, 0);
        engine.set_client_capabilities(
            1 | 1 << 3 | 1 << 5 | crate::window_interaction::IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH,
        );
        engine.set_layout_is_ru(true);

        let mut visible = String::new();
        let mut pending_final_frame = None;
        for (index, (ch, code)) in [
            ('р', 35),
            ('а', 33),
            ('б', 51),
            ('о', 36),
            ('а', 33),
            ('е', 20),
            ('т', 49),
        ]
        .into_iter()
        .enumerate()
        {
            let serial = 24_800 + index as u32 * 4;
            let keyval = replay_keyval(ch);
            assert!(legacy_key(&mut harness, &mut engine, serial, keyval, code, 0).await);
            visible.push(ch);
            let effects = legacy_effects(&mut harness).await;
            let commits = effects
                .iter()
                .filter(|effect| {
                    effect
                        .header()
                        .member()
                        .is_some_and(|member| member.as_str() == "CommitText")
                })
                .map(|effect| {
                    let body = effect.body();
                    let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
                    crate::ibus_interface::ibus_text_value_to_string(&value).unwrap()
                })
                .collect::<Vec<_>>();
            assert_eq!(commits, [ch.to_string()]);
            assert!(effects.iter().all(|effect| {
                if effect
                    .header()
                    .member()
                    .is_none_or(|member| member.as_str() != "UpdatePreeditText")
                {
                    return true;
                }
                let body = effect.body();
                let (text, _, _, _) = body
                    .deserialize::<(zbus::zvariant::Value<'_>, u32, bool, u32)>()
                    .unwrap();
                crate::ibus_interface::ibus_text_value_to_string(&text)
                    .as_deref()
                    .is_none_or(str::is_empty)
            }));
            assert!(engine.composition.buffer.is_empty());
            assert!(!engine.composition.legacy_word_preedit_active);

            let final_printable = index == 6;
            if final_printable {
                // The prior strict receipt advances with the final managed
                // append before Firefox publishes its updated snapshot. Space
                // computation must already use the identity that the later
                // exact receipt will authorize.
                assert!(engine.context_reset_rereceipt_computation_allowed());
                assert!(!engine.exact_marked_surrounding_suffix_is_current());
                let pending = engine
                    .capture_pending_reset_space_frame()
                    .expect("pending Reset computation frame");
                assert!(pending.space_autocorrect_suffix_token.is_some());
                assert!(crate::space_autocorrect_prefetch::proof::has_current_slot(
                    &pending
                ));
                crate::space_autocorrect_prefetch::proof::install_full_lease(
                    &pending,
                    &engine.config,
                );

                // Physical Firefox can deliver a delayed strict prefix from
                // its retired presentation before the final key release. It
                // removes exact authority, but must retain the current Reset
                // lineage for the authenticated Reset/full-receipt sequence.
                let delayed_prefix = visible.chars().take(3).collect::<String>();
                exact_replay_surrounding_receipt(&mut harness, &mut engine, &delayed_prefix).await;
                assert!(
                    engine.context_reset_rereceipt.is_some(),
                    "delayed strict prefix must retain inert Reset lineage"
                );
                assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                assert!(
                    legacy_key(
                        &mut harness,
                        &mut engine,
                        serial + 2,
                        keyval,
                        code,
                        RELEASE_MASK,
                    )
                    .await
                );
                no_legacy_output(&mut harness).await;
                let lookup =
                    crate::space_autocorrect_prefetch::take_with_budget(&pending, Duration::ZERO);
                assert!(matches!(
                    &lookup.lookup,
                    crate::space_autocorrect_prefetch::SpaceAutocorrectLookup::Ready(_)
                ));
                let emitter = zbus::object_server::SignalEmitter::new(
                    &harness.connection,
                    engine.path.clone(),
                )
                .unwrap();
                let mut output = crate::output::EngineOutput::legacy(&emitter);
                assert!(
                    !engine
                        .autocorrect_committed_token_on_space(&mut output, &pending, lookup)
                        .await
                        .unwrap(),
                    "pending Reset computation must not grant edit authority"
                );
                no_legacy_output(&mut harness).await;
                assert_eq!(engine.committed_tail.buffer, visible);
                crate::space_autocorrect_prefetch::proof::install_full_lease(
                    &pending,
                    &engine.config,
                );

                for (offset, prefix_chars) in [(3_u32, 4_usize), (4, 6)] {
                    text_free_owned_soft_reset(&mut harness, &mut engine, serial + offset).await;
                    assert!(engine.context_reset_rereceipt.is_some());
                    assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                    assert!(
                        crate::space_autocorrect_prefetch::proof::has_current_terminal_full_slot(
                            &pending
                        ),
                        "authenticated Reset must preserve only the identity-equal full slot"
                    );
                    assert_eq!(
                        engine.capture_pending_reset_space_frame().as_ref(),
                        Some(&pending),
                        "authenticated Reset token rotation must retain the current job identity"
                    );
                    let delayed_prefix = visible.chars().take(prefix_chars).collect::<String>();
                    exact_replay_surrounding_receipt(&mut harness, &mut engine, &delayed_prefix)
                        .await;
                    assert!(engine.context_reset_rereceipt.is_some());
                    assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                }
                text_free_owned_soft_reset(&mut harness, &mut engine, serial + 5).await;
                assert!(engine.context_reset_rereceipt.is_some());
                assert!(
                    crate::space_autocorrect_prefetch::proof::has_current_terminal_full_slot(
                        &pending
                    ),
                    "the final Reset must retain the completed early full-worker result"
                );
                assert_eq!(
                    engine.capture_pending_reset_space_frame().as_ref(),
                    Some(&pending),
                    "the final Reset must preserve the early full-worker identity"
                );
                pending_final_frame = Some(pending);
            } else {
                // Firefox retires the managed commit's client composition
                // before delivering the key release. Its first exact receipt
                // arms the strict witness used by later managed appends.
                text_free_owned_soft_reset(&mut harness, &mut engine, serial + 1).await;
                assert!(engine.context_reset_rereceipt.is_some());
            }

            let cursor = visible.chars().count() as u32;
            let emitter =
                zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                    .unwrap();
            let mut output = crate::output::EngineOutput::legacy(&emitter);
            crate::window_interaction::WindowInteraction::observe_facts(
                &mut engine,
                crate::window_interaction::WindowFactEvent::SurroundingText(Some(
                    SurroundingTextSnapshot::new(visible.clone(), cursor, cursor),
                )),
                Some(&mut output),
            )
            .await
            .unwrap();
            let snapshot_effects = legacy_effects(&mut harness).await;
            assert!(snapshot_effects.iter().all(|effect| {
                if effect
                    .header()
                    .member()
                    .is_none_or(|member| member.as_str() != "UpdatePreeditText")
                {
                    return true;
                }
                let body = effect.body();
                let (text, _, _, _) = body
                    .deserialize::<(zbus::zvariant::Value<'_>, u32, bool, u32)>()
                    .unwrap();
                crate::ibus_interface::ibus_text_value_to_string(&text)
                    .as_deref()
                    .is_none_or(str::is_empty)
            }));
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            if final_printable {
                let exact = engine
                    .capture_space_autocorrect_frame_identity()
                    .expect("final exact marked surrounding Space frame");
                assert_eq!(Some(&exact), pending_final_frame.as_ref());
                assert!(
                    crate::space_autocorrect_prefetch::proof::has_current_terminal_full_slot(
                        &exact
                    ),
                    "the exact receipt must preserve the completed equal-identity lease"
                );
            }

            if !final_printable {
                assert!(
                    !legacy_key(
                        &mut harness,
                        &mut engine,
                        serial + 2,
                        keyval,
                        code,
                        RELEASE_MASK,
                    )
                    .await
                );
                no_legacy_output(&mut harness).await;
            }
        }

        assert!(!engine.context_word_is_known());
        assert_eq!(engine.committed_tail.buffer, "рабоает");
        assert!(engine.capture_input_frame_identity().is_none());
        let frame = engine
            .capture_space_autocorrect_frame_identity()
            .expect("exact marked surrounding Space frame");
        assert!(frame.display_suffix_token.is_none());
        assert!(frame.space_autocorrect_suffix_token.is_some());
        assert!(frame.lexical_coordinates.is_some());
        assert_eq!(Some(&frame), pending_final_frame.as_ref());
        assert!(crate::space_autocorrect_prefetch::proof::has_current_terminal_full_slot(&frame));

        assert!(legacy_key(&mut harness, &mut engine, 24_840, KEY_SPACE, 57, 0).await);
        let effects = legacy_effects(&mut harness).await;
        let members = effects
            .iter()
            .filter_map(|effect| {
                effect
                    .header()
                    .member()
                    .map(|member| member.as_str().to_string())
            })
            .collect::<Vec<_>>();
        assert_eq!(
            members
                .iter()
                .filter(|member| member.as_str() == "DeleteSurroundingText")
                .count(),
            1
        );
        let commits = effects
            .iter()
            .filter(|effect| {
                effect
                    .header()
                    .member()
                    .is_some_and(|member| member.as_str() == "CommitText")
            })
            .map(|effect| {
                let body = effect.body();
                let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
                crate::ibus_interface::ibus_text_value_to_string(&value).unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(commits, ["работает "]);
        assert_eq!(engine.committed_tail.buffer, "работает ");
        assert!(engine.composition.buffer.is_empty());
        assert!(!engine.composition.legacy_word_preedit_active);
    });
}

#[test]
fn terminal_delivery_space_ready_and_refusal_outcomes_have_one_text_owner() {
    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("exact preparation available");
    zbus::block_on(async {
        for outcome in [
            "ready",
            "not_ready",
            "stale",
            "disabled",
            "suppressed",
            "no_geometry",
        ] {
            let width = if outcome == "no_geometry" { 0 } else { 11 };
            let (mut harness, mut engine) = known_terminal(width).await;
            for (index, (key, code)) in [
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
                assert!(
                    !legacy_key(
                        &mut harness,
                        &mut engine,
                        20_060 + index as u32,
                        key as u32,
                        code,
                        0
                    )
                    .await
                );
                no_legacy_output(&mut harness).await;
            }
            assert_eq!(engine.committed_tail.buffer, " ghbdtn");
            engine.config.auto_replace = true;
            engine.config.auto_switch_layout = true;
            let frame = engine
                .capture_input_frame_identity()
                .unwrap_or_else(|| panic!("known current word for outcome={outcome}"));
            crate::space_autocorrect_prefetch::proof::install_exact_lease(&frame, &engine.config);
            match outcome {
                "not_ready" => engine.invalidate_space_autocorrect_path(),
                "stale" => engine.config.nanda_autocorrect = !engine.config.nanda_autocorrect,
                "disabled" => engine.config.auto_replace = false,
                "suppressed" => assert!(engine.arm_current_word_autocorrect_suppression()),
                _ => {}
            }
            let handled = legacy_key(&mut harness, &mut engine, 20_070, KEY_SPACE, 57, 0).await;
            assert_eq!(handled, outcome == "ready", "{outcome}");
            let effects = legacy_effects(&mut harness).await;
            if outcome == "ready" {
                assert_eq!(effects.len(), 1, "exactly one replacement frame");
                let effect = &effects[0];
                assert_eq!(effect.header().member().unwrap().as_str(), "CommitText");
                let body = effect.body();
                let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
                assert_eq!(
                    crate::ibus_interface::ibus_text_value_to_string(&value),
                    Some("\u{7f}".repeat(6) + "привет ")
                );
                assert_eq!(engine.committed_tail.buffer, " привет ");
            } else {
                assert!(
                    effects.is_empty(),
                    "native Space, no second text owner: {outcome}"
                );
                assert_eq!(engine.committed_tail.buffer, " ghbdtn ", "{outcome}");
            }
            assert!(engine.committed_tail.autocorrect_suppression.is_none());
            assert_eq!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    20_071,
                    KEY_SPACE,
                    57,
                    RELEASE_MASK
                )
                .await,
                handled,
                "Space release belongs to this press: {outcome}"
            );
            no_legacy_output(&mut harness).await;
        }
    });
}
