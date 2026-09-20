//! Final-pass adverse schedules through the production receive/callback paths.
use super::*;
use crate::bridge::LayImeBridge;
use crate::protocol::{KEY_ENTER, KEY_ISO_LEVEL3_SHIFT, KEY_KP_ENTER, KEY_LEFT_ALT, KEY_RIGHT_ALT};

async fn td121_observed_append_then_unconfirmed_space(
    harness: &mut Harness,
    serial: u32,
    appended: &str,
) -> LayIbusEngine {
    let mut engine =
        initial_observed_tail_reset(harness, serial, &[('a', 30), ('b', 48), ('c', 46)]).await;
    exact_surrounding_receipt(harness, &mut engine, "abc").await;
    assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
    let mut next = serial + 10;
    retained_boundary_literal_keys(harness, &mut engine, &mut next, appended).await;
    assert!(engine.context_reset_rereceipt_computation_allowed());
    assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
    retained_boundary_literal_keys(harness, &mut engine, &mut next, " ").await;
    assert_eq!(engine.committed_tail.buffer, format!("abc{appended} "));
    engine
}

#[test]
fn td121_literal_boundary_retains_observed_tail_through_late_reset_prefixes() {
    zbus::block_on(bounded(async {
        let mut failures = Vec::new();
        for appended in ["ab", "aabcab"] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine =
                td121_observed_append_then_unconfirmed_space(&mut harness, 25_000, appended).await;
            let expected = format!("abc{appended} ");
            let retained_after_space = engine.context_reset_rereceipt.is_some();
            for length in 4..=expected.len() {
                actual_reset(&mut harness, &mut engine, 25_100 + length as u32, false).await;
                assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                exact_surrounding_receipt(&mut harness, &mut engine, &expected[..length]).await;
                if length < expected.len() {
                    assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                    assert!(engine.capture_observed_suffix_display_frame().is_none());
                }
                assert!(super::terminal_delivery::legacy_effects(&mut harness)
                    .await
                    .is_empty());
            }
            let confirmed = engine.context_reset_rereceipt_exact_manual_handoff_allowed();
            let path = engine.path.clone();
            let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
            let (engine, tail) = cycle09_visible_tail(&mut harness, engine).await;
            if !retained_after_space
                || !confirmed
                || disposition != Ok((3, false))
                || !cycle09_tail_is_authoritative(&tail, &path, false, &expected)
            {
                failures.push(format!("{appended}: retained_after_space={retained_after_space} confirmed={confirmed} disposition={disposition:?} tail={tail:?}"));
            }
            assert_eq!(engine.committed_tail.buffer, expected);
        }
        assert!(failures.is_empty(), "{}", failures.join("; "));
    }));
}

#[test]
fn td121_late_boundary_receipt_cannot_restore_contradiction_or_intervening_input() {
    zbus::block_on(bounded(async {
        for loss in [
            "wrong_prefix",
            "selection",
            "focus_out",
            "backspace",
            "printable",
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine =
                td121_observed_append_then_unconfirmed_space(&mut harness, 25_300, "ab").await;
            actual_reset(&mut harness, &mut engine, 25_330, false).await;
            match loss {
                "wrong_prefix" => {
                    exact_surrounding_receipt(&mut harness, &mut engine, "abcz").await
                }
                "selection" => surrounding_receipt(&mut harness, &mut engine, "abca", 4, 3).await,
                "focus_out" => actual_focus_out(&mut harness, &mut engine, 25_331).await,
                "backspace" => {
                    retained_boundary_backspace(&mut harness, &mut engine, &mut 25_331).await
                }
                "printable" => {
                    retained_boundary_literal_keys(&mut harness, &mut engine, &mut 25_331, "a")
                        .await
                }
                _ => unreachable!(),
            }
            let actual_tail = engine.committed_tail.buffer.clone();
            actual_reset(&mut harness, &mut engine, 25_340, false).await;
            exact_surrounding_receipt(&mut harness, &mut engine, "abcab ").await;
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{loss}"
            );
            assert!(
                super::terminal_delivery::legacy_effects(&mut harness)
                    .await
                    .is_empty(),
                "{loss}"
            );
            let (engine, result) = cycle09_manual_toggle(&mut harness, engine).await;
            assert_ne!(result, Ok((3, false)), "{loss}");
            assert_eq!(engine.committed_tail.buffer, actual_tail, "{loss}");
            assert!(
                engine.committed_tail.pending_completion_learning.is_none(),
                "{loss}"
            );
        }
    }));
}

fn td121_prime_completion_material() {
    use lay::typing_cpu::{LiveCompletionRequest, TypingCpu};
    assert!(TypingCpu::warm_l2_for_ime());
    let candidates = TypingCpu::live_completion_candidates(LiveCompletionRequest {
        context_prefix: "",
        partial: "про",
        max_suffix_chars: 16,
        active_composition: true,
        allow_short_lexical: true,
        limit: 12,
    });
    assert_eq!(
        candidates
            .first()
            .map(|candidate| candidate.surface.as_str()),
        Some("проверка")
    );
}

async fn td121_readout_key(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    serial: u32,
    keyval: u32,
    keycode: u32,
    state: u32,
) -> bool {
    // The real worker starts zbus's dispatcher. This fixture drives the engine
    // callback itself, so consume only the exact detached-object transport reply.
    let key = legacy_message(serial);
    harness.peer.connection.send(&key).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    consume_detached_callback_reply(&mut harness.peer, serial).await;
    run_received_legacy_key(harness, engine, &key, keyval, keycode, state).await
}

async fn td121_pending_current_completion(harness: &mut Harness) -> LayIbusEngine {
    td121_pending_current_completion_with_compute(harness, true).await
}

async fn td121_pending_current_completion_with_compute(
    harness: &mut Harness,
    compute: bool,
) -> LayIbusEngine {
    let mut engine = new_engine(harness);
    start_source_free_unknown(harness, &mut engine).await;
    engine.layout_gesture.layout_is_ru = true;
    engine.config.auto_replace = false;
    engine.config.typing_assist = false;
    engine.config.nanda_precognition = false;
    engine.config.correction_safety = "normal".into();
    engine.config.ime_bracket_candidates = false;
    engine.client_context.content_purpose = 0;
    engine.client_context.cursor_cell_width = 0;
    engine.client_context.surrounding_text_supported = true;
    for (offset, (ch, code)) in [('п', 34), ('р', 35)].into_iter().enumerate() {
        let serial = 24_000 + offset as u32 * 2;
        assert!(legacy_key(harness, &mut engine, serial, ch as u32, code, 0).await);
        td121_expect_legacy_commit_text(&mut harness.peer, &ch.to_string()).await;
        assert!(
            legacy_key(
                harness,
                &mut engine,
                serial + 1,
                ch as u32,
                code,
                RELEASE_MASK
            )
            .await
        );
    }
    actual_reset(harness, &mut engine, 24_010, false).await;
    exact_surrounding_receipt(harness, &mut engine, "пр").await;
    assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
    assert_eq!(
        publish_fixture_append_completion(harness, &mut engine, "ивет").await,
        "ивет"
    );
    engine.config.nanda_precognition = compute;
    let _ = harness.connection.object_server();
    engine.reset_precognition_causal_counts();
    assert!(td121_readout_key(harness, &mut engine, 24_011, 'о' as u32, 36, 0).await);
    td121_expect_legacy_commit_text(&mut harness.peer, "о").await;
    for cursor in [3, 2] {
        surrounding_receipt(harness, &mut engine, "привет", cursor, cursor).await;
        assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        assert!(engine.capture_observed_suffix_display_frame().is_none());
        assert!(engine.selected_precognition_suffix().is_none());
        assert!(!engine.composition.preedit_visible);
        assert_eq!(engine.committed_tail.buffer, "про");
    }
    engine
}

async fn td121_current_completion_receipt(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
) -> Vec<Message> {
    engine
        .set_surrounding_text(
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap(),
            crate::text::make_ibus_text("про".into()),
            3,
            3,
        )
        .await
        .unwrap();
    super::terminal_delivery::legacy_effects(harness).await
}

#[test]
fn td121_pending_worker_fills_missing_material_before_exact_receipt() {
    use lay::typing_cpu::{LiveCompletionRequest, TypingCpu};
    assert!(TypingCpu::warm_l2_for_ime());
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = td121_pending_current_completion_with_compute(&mut harness, false).await;
        assert_eq!(engine.precognition_causal_counts(), (0, 0));
        // Both successful and unavailable-package reloads invalidate completed
        // readouts. The focused sandbox can use built-in candidate material;
        // prove the cache miss itself, not installation of an optional package.
        let _reload = lay::nanda_wave::reload_productive_l2_v1();
        let query = LiveCompletionRequest {
            context_prefix: "",
            partial: "про",
            max_suffix_chars: 16,
            active_composition: true,
            allow_short_lexical: true,
            limit: 12,
        };
        assert!(TypingCpu::cached_live_completion_candidates(query.clone()).is_none());
        engine.config.nanda_precognition = true;
        let frame = engine.capture_pending_reset_readout_frame().unwrap();
        assert!(!engine.precognition_identity_matches(&frame));
        let completion = crate::precognition_worker::completion_observation::observe(frame);
        surrounding_receipt(&mut harness, &mut engine, "привет", 3, 3).await;
        assert_eq!(
            engine.precognition_causal_counts(),
            (1, 0),
            "a changed retained-preedit callback must schedule exactly one readout"
        );
        let (stage, count, cache_hit) = completion.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(stage, "discarded");
        assert!(count > 0);
        assert!(
            !cache_hit,
            "the real worker must compute the missing material"
        );
        assert_eq!(engine.precognition_causal_counts(), (1, 0));
        assert!(TypingCpu::cached_live_completion_candidates(query).is_some());
        assert!(super::terminal_delivery::legacy_effects(&mut harness)
            .await
            .is_empty());
        assert!(engine.selected_precognition_suffix().is_none());
        let effects = td121_current_completion_receipt(&mut harness, &mut engine).await;
        assert_eq!(
            engine.selected_precognition_suffix().as_deref(),
            Some("верка")
        );
        assert_eq!(
            effects
                .iter()
                .map(|effect| effect.header().member().unwrap().as_str().to_owned())
                .collect::<Vec<_>>(),
            ["UpdatePreeditText", "ShowPreeditText"]
        );
        assert!(engine.committed_tail.pending_completion_learning.is_none());
        engine.cancel_precognition_display_generation();
    }));
}

#[test]
fn td121_pending_reset_schedules_material_without_publication_authority() {
    td121_prime_completion_material();
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = td121_pending_current_completion(&mut harness).await;
        let (scheduled, applied) = engine.precognition_causal_counts();
        assert!(
            scheduled >= 1,
            "actual legacy input must schedule inert current material"
        );
        assert_eq!(applied, 0);
        assert!(engine.composition.preedit_display_only_pending);
        assert!(engine.context_reset_rereceipt_computation_allowed());
        assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        assert!(engine.capture_observed_suffix_display_frame().is_none());
        let effects = super::terminal_delivery::legacy_effects(&mut harness).await;
        assert!(effects.is_empty());
        assert!(engine.committed_tail.pending_completion_learning.is_none());
        engine.cancel_precognition_display_generation();
    }));
}

#[test]
fn td121_exact_receipt_publishes_cached_current_suffix_before_alt() {
    td121_prime_completion_material();
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = td121_pending_current_completion(&mut harness).await;
        let effects = td121_current_completion_receipt(&mut harness, &mut engine).await;
        assert_eq!(engine.selected_precognition_suffix().as_deref(), Some("верка"), "exact client receipt must publish already-computed current material before the next key callback");
        assert!(!engine.composition.preedit_display_only_pending);
        assert_eq!(effects.len(), 2);
        assert_eq!(
            effects[0].header().member().unwrap().as_str(),
            "UpdatePreeditText"
        );
        let body = effects[0].body();
        let (text, cursor, visible, mode) = body
            .deserialize::<(zbus::zvariant::Value<'_>, u32, bool, u32)>()
            .unwrap();
        assert_eq!(
            crate::ibus_interface::ibus_text_value_to_string(&text).as_deref(),
            Some("верка")
        );
        assert_eq!((cursor, visible, mode), (0, true, 0));
        assert_eq!(
            effects[1].header().member().unwrap().as_str(),
            "ShowPreeditText"
        );
        assert!(!td121_readout_key(&mut harness, &mut engine, 24_020, KEY_LEFT_ALT, 56, 0).await);
        assert!(
            td121_readout_key(
                &mut harness,
                &mut engine,
                24_021,
                KEY_LEFT_ALT,
                56,
                RELEASE_MASK
            )
            .await
        );
        td121_expect_legacy_commit_text(&mut harness.peer, "верка ").await;
        assert_eq!(engine.committed_tail.buffer, "проверка ");
        assert!(engine.context_word_is_known());
        engine.cancel_precognition_display_generation();
    }));
}

#[test]
fn td121_alt_before_exact_receipt_cannot_accept_later_cached_hint() {
    td121_prime_completion_material();
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = td121_pending_current_completion(&mut harness).await;
        assert!(!td121_readout_key(&mut harness, &mut engine, 24_020, KEY_LEFT_ALT, 56, 0).await);
        let effects = td121_current_completion_receipt(&mut harness, &mut engine).await;
        assert!(effects.iter().all(|effect| !matches!(
            effect.header().member().unwrap().as_str(),
            "CommitText" | "DeleteSurroundingText"
        )));
        assert!(
            !td121_readout_key(
                &mut harness,
                &mut engine,
                24_021,
                KEY_LEFT_ALT,
                56,
                RELEASE_MASK
            )
            .await
        );
        let effects = super::terminal_delivery::legacy_effects(&mut harness).await;
        assert!(effects.iter().all(|effect| !matches!(
            effect.header().member().unwrap().as_str(),
            "CommitText" | "DeleteSurroundingText"
        )));
        assert_eq!(engine.committed_tail.buffer, "про");
        assert!(engine.committed_tail.pending_completion_learning.is_none());
        engine.cancel_precognition_display_generation();
    }));
}

fn typed_key_message(serial: u32, member: &str, keyval: u32, keycode: u32, state: u32) -> Message {
    Message::method_call(TARGET_PATH, member)
        .unwrap()
        .interface(ENGINE_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .build(&(keyval, keycode, state))
        .unwrap()
}

#[test]
fn firefox_bridge_accepts_shift_received_before_marker_and_dispatches_it_once() {
    zbus::block_on(bounded(async {
        for (keyval, keycode) in [(KEY_LEFT_SHIFT, 42), (crate::protocol::KEY_RIGHT_SHIFT, 54)] {
            for state in [0, RELEASE_MASK] {
                let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
                let mut engine = initial_observed_tail_reset(
                    &mut harness,
                    14_700,
                    &[('a', 30), ('b', 48), ('c', 46)],
                )
                .await;
                exact_surrounding_receipt(&mut harness, &mut engine, "abc").await;
                let bridge = bridge(&harness, &engine);
                harness
                    .connection
                    .object_server()
                    .at(TARGET_PATH, engine)
                    .await
                    .unwrap();
                let iface = harness
                    .connection
                    .object_server()
                    .interface::<_, LayIbusEngine>(TARGET_PATH)
                    .await
                    .unwrap();
                // Hold the actual interface lock to order observer ingress
                // and the bridge marker before the real key callback starts.
                let held_engine = iface.get_mut().await;
                let key = typed_key_message(14_710, "ProcessKeyEvent", keyval, keycode, state);
                harness.peer.connection.send(&key).await.unwrap();
                assert!(harness.observer.process_next().await.unwrap());
                assert_eq!(
                    harness
                        .adapter
                        .shared
                        .reducer
                        .lock()
                        .unwrap()
                        .unsettled
                        .len(),
                    1
                );
                let (result, ()) = future::zip(bridge.manual_toggle_v3_inner(), async {
                    serve_next_ping_and_marker(&mut harness.peer).await;
                    assert!(harness.observer.process_next().await.unwrap());
                    drop(held_engine);
                })
                .await;
                assert_eq!(
                    result,
                    Ok((3, false)),
                    "a legacy Shift is not a word-mutation conflict"
                );
                loop {
                    let reply = bounded(next_peer_message(&mut harness.peer)).await;
                    if reply.header().reply_serial().map(|n| n.get()) == Some(14_710) {
                        assert_eq!(reply.header().message_type(), Type::MethodReturn);
                        assert!(!reply.body().deserialize::<bool>().unwrap());
                        break;
                    }
                    assert!(!reply.header().member().is_some_and(|m| matches!(
                        m.as_str(),
                        "CommitText" | "DeleteSurroundingText"
                    )));
                }
                let engine = std::mem::replace(&mut *iface.get_mut().await, new_engine(&harness));
                harness
                    .connection
                    .object_server()
                    .remove::<LayIbusEngine, _>(TARGET_PATH)
                    .await
                    .unwrap();
                assert_eq!(engine.committed_tail.buffer, "abc");
                assert!(!engine.context_word_is_known());
                assert!(engine.exact_manual_toggle_handoff_is_live());
                assert!(harness
                    .adapter
                    .shared
                    .reducer
                    .lock()
                    .unwrap()
                    .unsettled
                    .is_empty());
                assert!(harness
                    .adapter
                    .revalidate(&engine.live_context_token().unwrap()));
                assert!(drain_output_to_proof(&mut harness).await.is_empty());
            }
        }
    }));
}

#[test]
fn firefox_bridge_publication_survives_shift_received_after_reset_suffix_binding() {
    zbus::block_on(bounded(async {
        {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let engine = initial_observed_tail_reset(
                &mut harness,
                14_780,
                &[('a', 30), ('b', 48), ('c', 46)],
            )
            .await;
            assert!(engine.context_reset_rereceipt_computation_allowed());
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert!(engine.client_context.surrounding_text_supported);
            assert!(engine.client_context.surrounding_text_snapshot.is_none());
            let layout_is_ru = engine.layout_gesture.layout_is_ru;
            let (mut engine, pending) = cycle09_manual_toggle(&mut harness, engine).await;
            assert_eq!(pending, Ok((1, layout_is_ru)));
            assert!(engine.layout_gesture.pending_manual_toggle);
            assert_eq!(engine.committed_tail.buffer, "abc");

            engine
                .set_surrounding_text(
                    zbus::object_server::SignalEmitter::new(
                        &harness.connection,
                        engine.path.clone(),
                    )
                    .unwrap(),
                    crate::text::make_ibus_text("abc".to_string()),
                    3,
                    3,
                )
                .await
                .unwrap();
            let (effects, committed) = drain_output_to_text_proof(&mut harness).await;
            assert!(
                effects
                    .iter()
                    .any(|member| member == "DeleteSurroundingText"),
                "exact pending receipt must delete the proved old tail: {effects:?}"
            );
            assert!(
                effects.iter().any(|member| member == "CommitText"),
                "exact pending receipt must commit the toggled tail: {effects:?}"
            );
            assert_eq!(committed, ["фис"]);
            assert!(!engine.layout_gesture.pending_manual_toggle);
            assert_eq!(engine.committed_tail.buffer, "фис");
            assert_eq!(
                engine.layout_gesture.layout_is_ru, layout_is_ru,
                "layout ownership stays pending until the client confirms the final surface"
            );

            engine
                .set_surrounding_text(
                    zbus::object_server::SignalEmitter::new(
                        &harness.connection,
                        engine.path.clone(),
                    )
                    .unwrap(),
                    crate::text::make_ibus_text("фис".to_string()),
                    3,
                    3,
                )
                .await
                .unwrap();
            let final_effects = drain_output_to_proof(&mut harness).await;
            assert!(
                final_effects.iter().all(|member| !matches!(
                    member.as_str(),
                    "DeleteSurroundingText" | "CommitText"
                )),
                "the exact final receipt must confirm without another mutation: {final_effects:?}"
            );
            assert_eq!(engine.layout_gesture.layout_is_ru, !layout_is_ru);
            assert_eq!(
                engine
                    .client_context
                    .surrounding_text_snapshot
                    .as_ref()
                    .map(|snapshot| snapshot.text.as_str()),
                Some("фис")
            );
        }

        {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let engine = initial_observed_tail_reset(
                &mut harness,
                14_790,
                &[('a', 30), ('b', 48), ('c', 46)],
            )
            .await;
            let layout_is_ru = engine.layout_gesture.layout_is_ru;
            let (mut engine, pending) = cycle09_manual_toggle(&mut harness, engine).await;
            assert_eq!(pending, Ok((1, layout_is_ru)));
            assert!(engine.layout_gesture.pending_manual_toggle);

            engine
                .set_surrounding_text(
                    zbus::object_server::SignalEmitter::new(
                        &harness.connection,
                        engine.path.clone(),
                    )
                    .unwrap(),
                    crate::text::make_ibus_text("xyz".to_string()),
                    3,
                    3,
                )
                .await
                .unwrap();
            let effects = drain_output_to_proof(&mut harness).await;
            assert!(
                effects.iter().all(|member| !matches!(
                    member.as_str(),
                    "DeleteSurroundingText" | "CommitText"
                )),
                "mismatched pending receipt must not mutate client text: {effects:?}"
            );
            assert!(!engine.layout_gesture.pending_manual_toggle);
            assert_eq!(engine.committed_tail.buffer, "abc");
        }

        for bind_before_key in [true, false] {
            for (keyval, keycode, state) in [
                (KEY_LEFT_SHIFT, 42, 0),
                (KEY_LEFT_SHIFT, 42, RELEASE_MASK),
                (crate::protocol::KEY_RIGHT_SHIFT, 54, 0),
                (crate::protocol::KEY_RIGHT_SHIFT, 54, RELEASE_MASK),
            ] {
                let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
                let mut engine = initial_observed_tail_reset(
                    &mut harness,
                    14_800,
                    &[('a', 30), ('b', 48), ('c', 46)],
                )
                .await;
                exact_surrounding_receipt(&mut harness, &mut engine, "abc").await;
                let token = bridge_fence(&mut harness).await.unwrap();
                let key = typed_key_message(14_810, "ProcessKeyEvent", keyval, keycode, state);
                {
                    let mut output_scope = engine.begin_context_bridge_output(Some(&token));
                    if bind_before_key {
                        assert!(
                            output_scope.consume_context_reset_rereceipt_for_exact_manual_handoff()
                        );
                    }
                    harness.peer.connection.send(&key).await.unwrap();
                    assert!(harness.observer.process_next().await.unwrap());
                    if !bind_before_key {
                        assert!(
                            output_scope.consume_context_reset_rereceipt_for_exact_manual_handoff()
                        );
                    }
                    output_scope.prepare_exact_manual_toggle_layout_handoff();
                    assert!(output_scope.exact_manual_toggle_handoff_is_live(),
                        "read-only Shift must not revoke exact-tail publication after suffix binding");
                    output_scope.complete();
                }
                let published_token = engine.live_context_token().unwrap();
                let epoch = engine.committed_tail.epoch;
                let emitter =
                    zbus::object_server::SignalEmitter::new(&harness.connection, TARGET_PATH)
                        .unwrap();
                assert!(!engine
                    .process_key_event(key.header(), emitter, keyval, keycode, state)
                    .await
                    .unwrap());
                assert_eq!(engine.committed_tail.buffer, "abc");
                assert_eq!(engine.committed_tail.epoch, epoch);
                assert_eq!(engine.live_context_token().as_ref(), Some(&published_token));
                assert!(harness.adapter.revalidate(&published_token));
                assert!(engine.exact_manual_toggle_handoff_is_live());
                assert!(!engine.context_word_is_known());
                assert!(harness
                    .adapter
                    .shared
                    .reducer
                    .lock()
                    .unwrap()
                    .unsettled
                    .is_empty());
                assert!(drain_output_to_proof(&mut harness).await.is_empty());
            }
        }
    }));
}

#[test]
fn firefox_bridge_readonly_shift_does_not_exempt_other_input_or_lifecycle() {
    zbus::block_on(bounded(async {
        for (member, keyval, keycode, state) in [
            ("ProcessKeyEvent", 'a' as u32, 30, 0),
            ("ProcessKeyEvent", 'a' as u32, 30, RELEASE_MASK),
            ("ProcessKeyEvent", KEY_LEFT_ALT, 64, 0),
            ("ProcessKeyEvent", KEY_LEFT_ALT, 64, RELEASE_MASK),
            ("ProcessKeyEvent", KEY_TAB, 15, 0),
            ("ProcessKeyEvent", KEY_LEFT, 105, 0),
            ("ProcessKeyEventAtomicV1", KEY_LEFT_SHIFT, 42, 0),
            ("ProcessKeyEventAtomicV1", KEY_LEFT_SHIFT, 42, RELEASE_MASK),
            ("malformed", KEY_LEFT_SHIFT, 42, 0),
            ("FocusOut", KEY_LEFT_SHIFT, 42, 0),
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let engine = known_engine(&mut harness).await;
            let token = bridge_fence(&mut harness).await.unwrap();
            let key = match member {
                "malformed" => method_message(
                    DISPATCH_SENDER,
                    14_900,
                    TARGET_PATH,
                    ENGINE_INTERFACE,
                    "ProcessKeyEvent",
                ),
                "FocusOut" => method_message(
                    DISPATCH_SENDER,
                    14_900,
                    TARGET_PATH,
                    ENGINE_INTERFACE,
                    "FocusOut",
                ),
                _ => typed_key_message(14_900, member, keyval, keycode, state),
            };
            harness.peer.connection.send(&key).await.unwrap();
            assert!(harness.observer.process_next().await.unwrap());
            assert!(
                !harness.adapter.revalidate_bridge(&token),
                "{member}, keyval={keyval}, state={state}"
            );
            assert!(bridge_fence(&mut harness).await.is_err());
            assert_eq!(engine.committed_tail.buffer, " ");
            assert!(drain_output_to_proof(&mut harness).await.is_empty());
        }
    }));
}

#[test]
fn firefox_readonly_shift_keeps_callback_queue_bound() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let _engine = known_engine(&mut harness).await;
        for offset in 0..64 {
            let key = typed_key_message(15_000 + offset, "ProcessKeyEvent", KEY_LEFT_SHIFT, 42, 0);
            harness.peer.connection.send(&key).await.unwrap();
            assert!(harness.observer.process_next().await.unwrap());
        }
        assert_eq!(
            harness
                .adapter
                .shared
                .reducer
                .lock()
                .unwrap()
                .unsettled
                .len(),
            64
        );
        let key = typed_key_message(15_100, "ProcessKeyEvent", KEY_LEFT_SHIFT, 42, 0);
        harness.peer.connection.send(&key).await.unwrap();
        assert!(harness.observer.process_next().await.is_err());
        assert!(harness.adapter.current_token().is_none());
    }));
}

async fn known_engine(harness: &mut Harness) -> LayIbusEngine {
    let mut engine = new_engine(harness);
    start_source_free_unknown(harness, &mut engine).await;
    assert!(legacy_key(harness, &mut engine, 3_000, KEY_SPACE, 57, 0).await);
    expect_legacy_commit(&mut harness.peer).await;
    assert!(legacy_key(harness, &mut engine, 3_001, KEY_SPACE, 57, RELEASE_MASK).await);
    assert!(engine.context_word_is_known());
    engine
}

async fn receive(harness: &mut Harness, serial: u32, member: &str) -> Message {
    let message = method_message(
        DISPATCH_SENDER,
        serial,
        TARGET_PATH,
        ENGINE_INTERFACE,
        member,
    );
    harness.peer.connection.send(&message).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    message
}

async fn bridge_fence(harness: &mut Harness) -> Result<AdmissionToken, AdapterError> {
    let (fence, ()) = bounded(future::zip(
        harness.adapter.begin_bridge_fence(),
        serve_ping_and_marker(&mut harness.peer),
    ))
    .await;
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    harness.adapter.finish_bridge_fence(fence.unwrap())
}

fn bridge(harness: &Harness, engine: &LayIbusEngine) -> LayImeBridge {
    LayImeBridge {
        ibus_connection: harness.connection.clone(),
        shared: engine.shared.clone(),
        context_admission_required: true,
        admission: Some(harness.adapter.clone()),
    }
}

pub(super) async fn actual_focus_out(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    serial: u32,
) {
    let focus_out = method_message(
        DISPATCH_SENDER,
        serial,
        TARGET_PATH,
        ENGINE_INTERFACE,
        "FocusOut",
    );
    harness.peer.connection.send(&focus_out).await.unwrap();
    let ((), observed) = bounded(future::zip(
        engine.focus_out(focus_out.header()),
        harness.observer.process_next(),
    ))
    .await;
    assert!(
        observed.expect("repeated FocusOut must not cancel the observer"),
        "observer stream remains live after repeated FocusOut"
    );
}

#[test]
fn residual_repeated_focus_out_revokes_locally_and_recovers_fresh_unknown_start() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = known_engine(&mut harness).await;
        let old_token = engine.live_context_token().unwrap();

        actual_focus_out(&mut harness, &mut engine, 3_002).await;
        assert!(engine.context_handoff_sealed);
        {
            let state = engine.shared.lock().unwrap();
            assert_eq!(state.active_path.as_deref(), Some(TARGET_PATH));
            assert_eq!(
                state.context_owner_generation,
                Some(old_token.owner.generation.0)
            );
        }

        // Model stale local state retained after the first sealed handoff. The
        // repeated callback is a scoped revocation and must still run the real
        // FocusOut cleanup rather than terminate the observer.
        engine.composition.buffer = "stale".to_string();
        engine.composition.preedit_visible = true;
        actual_focus_out(&mut harness, &mut engine, 3_003).await;
        assert!(!engine.context_handoff_sealed);
        assert!(engine.composition.buffer.is_empty());
        assert!(!engine.composition.preedit_visible);
        {
            let state = engine.shared.lock().unwrap();
            assert!(state.active_path.is_none());
            assert!(state.context_owner_generation.is_none());
        }
        assert!(!harness.adapter.revalidate(&old_token));

        let focus_in = receive(&mut harness, 3_004, "FocusInId").await;
        bounded(engine.focus_in_id(
            focus_in.header(),
            CONTEXT_PATH.to_string(),
            "test-client".to_string(),
        ))
        .await;
        engine.config = ime_config();
        forward_marker_bounded(&mut harness.peer).await;
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        assert!(!legacy_key(&mut harness, &mut engine, 3_005, KEY_LEFT_SHIFT, 42, 0).await);
        assert!(
            !legacy_key(
                &mut harness,
                &mut engine,
                3_006,
                KEY_LEFT_SHIFT,
                42,
                RELEASE_MASK,
            )
            .await
        );

        let fresh_token = engine.live_context_token().expect("fresh FocusIn owner");
        assert_ne!(fresh_token.owner.generation, old_token.owner.generation);
        assert_eq!(
            fresh_token.lineage.completeness,
            WordCompleteness::UnknownStart
        );
        assert!(!harness.adapter.revalidate(&old_token));

        assert!(legacy_key(&mut harness, &mut engine, 3_007, KEY_SPACE, 57, 0).await);
        expect_legacy_commit(&mut harness.peer).await;
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                3_008,
                KEY_SPACE,
                57,
                RELEASE_MASK,
            )
            .await
        );
        assert!(engine.context_word_is_known());
        assert!(bridge_fence(&mut harness).await.is_ok());
    }));
}

#[test]
fn residual_repeated_focus_out_after_reset_or_content_type_revocation_survives() {
    zbus::block_on(bounded(async {
        for content_type in [false, true] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = known_engine(&mut harness).await;
            let old_token = engine.live_context_token().unwrap();
            actual_focus_out(&mut harness, &mut engine, 3_100).await;
            assert!(engine.context_handoff_sealed);
            if content_type {
                revoke_ready(&mut harness, &old_token.owner, true).await;
            } else {
                let reset = receive(&mut harness, 3_101, "Reset").await;
                bounded(
                    engine.reset(
                        reset.header(),
                        zbus::object_server::SignalEmitter::new(
                            &harness.connection,
                            engine.path.clone(),
                        )
                        .unwrap(),
                    ),
                )
                .await
                .unwrap();
            }
            actual_focus_out(&mut harness, &mut engine, 3_102).await;
            assert!(!harness.adapter.revalidate(&old_token));
            assert!(!engine.context_handoff_sealed);
            assert!(!engine.context_word_is_known());
            let state = engine.shared.lock().unwrap();
            assert!(state.active_path.is_none());
            assert!(state.context_owner_generation.is_none());
        }
    }));
}

async fn actual_disable(harness: &mut Harness, engine: &mut LayIbusEngine, serial: u32) {
    let disable = method_message(
        DISPATCH_SENDER,
        serial,
        TARGET_PATH,
        ENGINE_INTERFACE,
        "Disable",
    );
    harness.peer.connection.send(&disable).await.unwrap();
    let ((), observed) = bounded(future::zip(
        engine.disable(disable.header()),
        harness.observer.process_next(),
    ))
    .await;
    assert!(observed.expect("lifecycle Disable refusal must not cancel observation"));
}

fn factory_message(serial: u32) -> Message {
    Message::method_call("/org/freedesktop/IBus/Factory", "CreateEngine")
        .unwrap()
        .interface(FACTORY_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .build(&"lay-us")
        .unwrap()
}

async fn legacy_key_at(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    path: &str,
    serial: u32,
    keyval: u32,
    keycode: u32,
    state: u32,
) -> bool {
    let key = method_message(
        DISPATCH_SENDER,
        serial,
        path,
        ENGINE_INTERFACE,
        "ProcessKeyEvent",
    );
    harness.peer.connection.send(&key).await.unwrap();
    let emitter = zbus::object_server::SignalEmitter::new(&harness.connection, path)
        .expect("legacy signal emitter");
    let (handled, observed) = bounded(future::zip(
        engine.process_key_event(key.header(), emitter, keyval, keycode, state),
        harness.observer.process_next(),
    ))
    .await;
    assert!(observed.expect("legacy key observer result"));
    handled.expect("legacy ProcessKeyEvent result")
}

#[derive(Clone, Copy, Debug)]
enum PendingWordLoss {
    EarlyKey,
    EarlyKeyThenTerminalContentType,
    Reset,
    ResetAfterReply,
    TerminalContentType,
    TerminalContentTypeAfterReply,
}

async fn actual_pending_word_reset(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    terminal_content_type: bool,
    serial: u32,
) {
    if terminal_content_type {
        let set = Message::method_call(TARGET_PATH, "Set")
            .unwrap()
            .interface(PROPERTIES_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(serial).unwrap())
            .build(&(
                ENGINE_INTERFACE,
                "ContentType",
                zbus::zvariant::Value::from((10u32, 0u32)),
            ))
            .unwrap();
        harness.peer.connection.send(&set).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        bounded(engine.set_content_type((10, 0), Some(set.header()))).await;
        assert_eq!(engine.client_context.content_purpose, 10);
    } else {
        let reset = receive(harness, serial, "Reset").await;
        bounded(
            engine.reset(
                reset.header(),
                zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                    .unwrap(),
            ),
        )
        .await
        .unwrap();
    }
}

async fn actual_pending_early_key(harness: &mut Harness, engine: &mut LayIbusEngine) {
    assert!(!legacy_key(harness, engine, 3_212, KEY_LEFT_SHIFT, 42, 0,).await);
    assert!(!legacy_key(harness, engine, 3_213, KEY_LEFT_SHIFT, 42, RELEASE_MASK,).await);
}

async fn held_compatibility_get_word_loss_case(word_loss: PendingWordLoss) {
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = known_engine(&mut harness).await;
    let old_token = engine.live_context_token().unwrap();
    engine.committed_tail.buffer = "old-tail-must-not-return".to_string();
    engine.rebuild_preedit_fast_from_tail();
    {
        let mut shared = engine.shared.lock().unwrap();
        shared.handoff_tail_buffer = engine.committed_tail.buffer.clone();
        shared.handoff_tail_epoch = engine.committed_tail.epoch;
    }
    assert!(engine.arm_current_word_autocorrect_suppression());

    actual_focus_out(&mut harness, &mut engine, 3_210).await;
    assert!(engine.context_handoff_sealed);

    let focus_in = receive(&mut harness, 3_211, "FocusIn").await;
    bounded(engine.focus_in_callback(focus_in.header())).await;
    engine.config = ime_config();

    let held_get = bounded(next_peer_message(&mut harness.peer)).await;
    assert_eq!(held_get.header().member().unwrap().as_str(), "Get");
    let (interface, property) = held_get.body().deserialize::<(String, String)>().unwrap();
    assert_eq!(interface, IBUS_INTERFACE);
    assert_eq!(property, "CurrentInputContext");
    let (original_request, original_nonce) = {
        let reducer = harness.adapter.shared.reducer.lock().unwrap();
        let request = reducer.request.as_ref().expect("original held-Get request");
        assert_eq!(request.origin, ReceiptOrigin::CompatibilityProperty);
        assert!(request.reply.is_none());
        (request.generation, request.nonce)
    };

    if matches!(
        word_loss,
        PendingWordLoss::EarlyKey | PendingWordLoss::EarlyKeyThenTerminalContentType
    ) {
        actual_pending_early_key(&mut harness, &mut engine).await;
    }
    match word_loss {
        PendingWordLoss::Reset => {
            actual_pending_word_reset(&mut harness, &mut engine, false, 3_214).await;
        }
        PendingWordLoss::TerminalContentType | PendingWordLoss::EarlyKeyThenTerminalContentType => {
            actual_pending_word_reset(&mut harness, &mut engine, true, 3_215).await;
        }
        PendingWordLoss::EarlyKey
        | PendingWordLoss::ResetAfterReply
        | PendingWordLoss::TerminalContentTypeAfterReply => {}
    }

    let reset_after_reply = matches!(
        word_loss,
        PendingWordLoss::ResetAfterReply | PendingWordLoss::TerminalContentTypeAfterReply
    );
    if !reset_after_reply {
        let reducer = harness.adapter.shared.reducer.lock().unwrap();
        let request = reducer
            .request
            .as_ref()
            .expect("word loss retains the original context acquisition");
        assert_eq!(request.generation, original_request);
        assert_eq!(request.nonce, original_nonce);
        assert!(request.reply.is_none());
        assert_eq!(
            reducer.lineage().completeness,
            WordCompleteness::UnknownStart
        );
    }

    let value = OwnedValue::from(ObjectPath::try_from(CONTEXT_PATH).unwrap());
    harness
        .peer
        .connection
        .reply(&held_get.header(), &value)
        .await
        .unwrap();
    let marker = bounded(next_peer_message(&mut harness.peer)).await;
    assert_eq!(marker.header().member().unwrap().as_str(), MARKER_MEMBER);
    assert_eq!(
        marker.body().deserialize::<u64>().unwrap(),
        original_nonce.0,
        "the word-loss path must retain the original marker fence"
    );
    if reset_after_reply {
        actual_pending_word_reset(
            &mut harness,
            &mut engine,
            matches!(word_loss, PendingWordLoss::TerminalContentTypeAfterReply),
            3_219,
        )
        .await;
        let reducer = harness.adapter.shared.reducer.lock().unwrap();
        let request = reducer
            .request
            .as_ref()
            .expect("marker-window reset retains the original request");
        assert_eq!(request.generation, original_request);
        assert_eq!(request.nonce, original_nonce);
        assert!(request.reply.is_some());
        assert!(request.marker_position.is_none());
        assert_eq!(
            reducer.lineage().completeness,
            WordCompleteness::UnknownStart
        );
    }
    assert!(!harness.adapter.revalidate(&old_token));
    let forwarded = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
        .unwrap()
        .sender(ADAPTER_SENDER)
        .unwrap()
        .build(&original_nonce.0)
        .unwrap();
    harness.peer.connection.send(&forwarded).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());

    assert!(!legacy_key(&mut harness, &mut engine, 3_216, KEY_LEFT_SHIFT, 42, 0,).await);
    assert!(
        !legacy_key(
            &mut harness,
            &mut engine,
            3_217,
            KEY_LEFT_SHIFT,
            42,
            RELEASE_MASK,
        )
        .await
    );
    let fresh = engine
        .live_context_token()
        .expect("the original Get and marker install a fresh context");
    assert_ne!(fresh.owner.generation, old_token.owner.generation);
    assert_eq!(fresh.lineage.completeness, WordCompleteness::UnknownStart);
    assert!(engine.committed_tail.buffer.is_empty());
    assert!(engine.committed_tail.autocorrect_suppression.is_none());
    assert!(engine.committed_tail.pending_completion_learning.is_none());
    {
        let shared = engine.shared.lock().unwrap();
        assert!(shared.handoff_tail_buffer.is_empty());
        assert!(shared.autocorrect_suppression.is_none());
    }
    assert!(engine
        .atomic
        .settlement_feedback_events
        .lock()
        .unwrap()
        .is_empty());
    assert!(bridge_fence(&mut harness).await.is_ok());

    let handled = legacy_key(&mut harness, &mut engine, 3_218, KEY_SPACE, 57, 0).await;
    if handled {
        expect_legacy_commit(&mut harness.peer).await;
    }
    assert!(engine.context_word_is_known());
    assert!(bridge_fence(&mut harness).await.is_ok());
}

#[test]
fn residual_held_compatibility_get_survives_early_key_as_empty_unknown_start() {
    zbus::block_on(bounded(held_compatibility_get_word_loss_case(
        PendingWordLoss::EarlyKey,
    )));
}

#[test]
fn residual_held_compatibility_get_survives_actual_reset_as_empty_unknown_start() {
    zbus::block_on(bounded(held_compatibility_get_word_loss_case(
        PendingWordLoss::Reset,
    )));
}

#[test]
fn residual_held_compatibility_get_survives_actual_terminal_content_type_as_empty_unknown_start() {
    zbus::block_on(bounded(held_compatibility_get_word_loss_case(
        PendingWordLoss::TerminalContentType,
    )));
}

#[test]
fn residual_held_compatibility_get_survives_early_key_then_terminal_content_type() {
    zbus::block_on(bounded(held_compatibility_get_word_loss_case(
        PendingWordLoss::EarlyKeyThenTerminalContentType,
    )));
}

#[test]
fn residual_compatibility_marker_window_survives_actual_reset_or_terminal_content_type() {
    zbus::block_on(bounded(async {
        held_compatibility_get_word_loss_case(PendingWordLoss::ResetAfterReply).await;
        held_compatibility_get_word_loss_case(PendingWordLoss::TerminalContentTypeAfterReply).await;
    }));
}

#[test]
fn residual_compatibility_refocus_without_disable_keeps_original_get_and_word() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = known_engine(&mut harness).await;
        let old_token = engine.live_context_token().unwrap();
        engine.committed_tail.buffer = "retained-word".to_string();
        engine.rebuild_preedit_fast_from_tail();
        {
            let mut shared = engine.shared.lock().unwrap();
            shared.handoff_tail_buffer = engine.committed_tail.buffer.clone();
            shared.handoff_tail_epoch = engine.committed_tail.epoch;
        }
        assert!(engine.arm_current_word_autocorrect_suppression());

        actual_focus_out(&mut harness, &mut engine, 3_220).await;
        let focus_in = receive(&mut harness, 3_221, "FocusIn").await;
        bounded(engine.focus_in_callback(focus_in.header())).await;
        engine.config = ime_config();

        let held_get = bounded(next_peer_message(&mut harness.peer)).await;
        assert_eq!(held_get.header().member().unwrap().as_str(), "Get");
        let (request_generation, nonce) = {
            let reducer = harness.adapter.shared.reducer.lock().unwrap();
            let request = reducer
                .request
                .as_ref()
                .expect("compatibility refocus request");
            assert_eq!(request.origin, ReceiptOrigin::CompatibilityProperty);
            (request.generation, request.nonce)
        };
        let value = OwnedValue::from(ObjectPath::try_from(CONTEXT_PATH).unwrap());
        harness
            .peer
            .connection
            .reply(&held_get.header(), &value)
            .await
            .unwrap();
        let marker = bounded(next_peer_message(&mut harness.peer)).await;
        assert_eq!(marker.header().member().unwrap().as_str(), MARKER_MEMBER);
        assert_eq!(marker.body().deserialize::<u64>().unwrap(), nonce.0);
        {
            let reducer = harness.adapter.shared.reducer.lock().unwrap();
            let request = reducer.request.as_ref().expect("original request retained");
            assert_eq!(request.generation, request_generation);
            assert_eq!(request.nonce, nonce);
            assert!(request.reply.is_some());
        }
        let forwarded = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
            .unwrap()
            .sender(ADAPTER_SENDER)
            .unwrap()
            .build(&nonce.0)
            .unwrap();
        harness.peer.connection.send(&forwarded).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());

        assert!(!legacy_key(&mut harness, &mut engine, 3_222, KEY_LEFT_SHIFT, 42, 0,).await);
        assert!(
            !legacy_key(
                &mut harness,
                &mut engine,
                3_223,
                KEY_LEFT_SHIFT,
                42,
                RELEASE_MASK,
            )
            .await
        );
        let fresh = engine
            .live_context_token()
            .expect("same-context compatibility refocus installs");
        assert_ne!(fresh.owner.generation, old_token.owner.generation);
        assert_eq!(fresh.lineage.completeness, WordCompleteness::KnownStart);
        assert_eq!(engine.committed_tail.buffer, "retained-word");
        assert!(engine.committed_tail.autocorrect_suppression.is_some());
        assert!(!harness.adapter.revalidate(&old_token));
        assert!(bridge_fence(&mut harness).await.is_ok());
    }));
}

async fn published_source_free_then_compatibility_refocus_case(
    reply_context: &str,
    expect_transfer: bool,
) {
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = new_engine(&harness);

    // Owner 2 is published, but its target has not consumed it yet.
    start_source_free_pending(&harness).await;
    forward_marker_bounded(&mut harness.peer).await;
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    assert!(engine.context_owner.is_none());
    assert!(harness
        .adapter
        .shared
        .ready_activation
        .lock()
        .unwrap()
        .is_some());

    let focus_out = method_message(
        DISPATCH_SENDER,
        3_224,
        TARGET_PATH,
        ENGINE_INTERFACE,
        "FocusOut",
    );
    let focus_in = method_message(
        DISPATCH_SENDER,
        3_225,
        TARGET_PATH,
        ENGINE_INTERFACE,
        "FocusIn",
    );
    harness.peer.connection.send(&focus_out).await.unwrap();
    harness.peer.connection.send(&focus_in).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    assert!(bounded(harness.observer.process_next()).await.unwrap());

    assert!(
        engine
            .observe_context_focus_out(&focus_out.header(), Instant::now())
            .await
    );
    let sealed_owner = engine
        .context_owner
        .clone()
        .expect("FocusOut installs the published owner before sealing it");
    bounded(engine.focus_in_callback(focus_in.header())).await;
    engine.config = ime_config();

    let get = bounded(next_peer_message(&mut harness.peer)).await;
    assert_eq!(get.header().member().unwrap().as_str(), "Get");
    let (request_generation, nonce) = {
        let reducer = harness.adapter.shared.reducer.lock().unwrap();
        let request = reducer.request.as_ref().expect("compatibility request 3");
        assert_eq!(request.origin, ReceiptOrigin::CompatibilityProperty);
        assert!(reducer.unsettled.is_empty());
        (request.generation, request.nonce)
    };

    harness
        .peer
        .connection
        .reply(
            &get.header(),
            &OwnedValue::from(ObjectPath::try_from(reply_context).unwrap()),
        )
        .await
        .unwrap();
    let marker = bounded(next_peer_message(&mut harness.peer)).await;
    assert_eq!(marker.header().member().unwrap().as_str(), MARKER_MEMBER);
    assert_eq!(marker.body().deserialize::<u64>().unwrap(), nonce.0);
    {
        let reducer = harness.adapter.shared.reducer.lock().unwrap();
        let request = reducer
            .request
            .as_ref()
            .expect("same compatibility request");
        assert_eq!(request.generation, request_generation);
        assert_eq!(request.nonce, nonce);
        assert!(request.reply.is_some());
        assert!(request.marker_position.is_none());
        assert!(reducer.unsettled.is_empty());
    }
    {
        let pending = harness.adapter.shared.pending.lock().unwrap();
        let pending = pending.as_ref().expect("compatibility fence");
        assert_eq!(pending.nonce, nonce);
        assert!(pending.deadline > Instant::now());
    }
    let forwarded = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
        .unwrap()
        .sender(ADAPTER_SENDER)
        .unwrap()
        .build(&nonce.0)
        .unwrap();
    harness.peer.connection.send(&forwarded).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());

    let outcome = harness
        .adapter
        .pending_activation_for(&engine_path(TARGET_PATH))
        .expect("prompt compatibility reply and marker publish one outcome");
    assert_eq!(
        matches!(&outcome, ActivationOutcome::Transfer(_)),
        expect_transfer
    );
    assert!(outcome.token().owner.generation > sealed_owner.generation);
    if let ActivationOutcome::Transfer(grant) = &outcome {
        assert_eq!(grant.source_owner, sealed_owner);
    }
    assert!(harness.adapter.shared.pending.lock().unwrap().is_none());
    assert!(harness
        .adapter
        .shared
        .reducer
        .lock()
        .unwrap()
        .request
        .is_none());
    engine.try_install_pending_context_activation();
    assert!(engine.live_context_token().is_some());
    assert!(harness
        .adapter
        .shared
        .ready_activation
        .lock()
        .unwrap()
        .is_none());
}

#[test]
fn td121_published_source_free_refocus_prompt_compatibility_reply_is_ready() {
    zbus::block_on(bounded(async {
        published_source_free_then_compatibility_refocus_case(CONTEXT_PATH, true).await;
        published_source_free_then_compatibility_refocus_case(
            "/org/freedesktop/IBus/InputContext_2",
            false,
        )
        .await;
    }));
}

#[test]
fn td121_compatibility_request_fits_five_milliseconds_then_forced_expiry_clears() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(Duration::from_millis(5)).await;
        let acquisition = harness
            .adapter
            .begin_compatibility_activation(engine_path(TARGET_PATH), Default::default());
        let peer = async {
            let get = next_peer_message(&mut harness.peer).await;
            assert_eq!(get.header().member().unwrap().as_str(), "Get");
            harness
                .peer
                .connection
                .reply(
                    &get.header(),
                    &OwnedValue::from(ObjectPath::try_from(CONTEXT_PATH).unwrap()),
                )
                .await
                .unwrap();
            let marker = next_peer_message(&mut harness.peer).await;
            assert_eq!(marker.header().member().unwrap().as_str(), MARKER_MEMBER);
        };
        let (fence, ()) = future::zip(acquisition, peer).await;
        let fence = fence.expect("controlled Get reply and marker emission fit the 5 ms budget");

        // Force the existing deadline owner without a sleep. The branch proves
        // expiry cleanup semantics; the completed acquisition above separately
        // measures that this controlled Get/marker exchange fit within 5 ms.
        harness.adapter.expire_fence(fence);
        assert!(harness.adapter.shared.pending.lock().unwrap().is_none());
        assert!(harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .request
            .is_none());
        assert!(harness.adapter.current_owner().is_none());
        assert!(harness.adapter.current_token().is_none());
        assert!(harness
            .adapter
            .shared
            .ready_activation
            .lock()
            .unwrap()
            .is_none());
    }));
}

async fn default_budget_harness() -> Harness {
    bootstrap_harness_with_config(
        "lay-us",
        AdapterConfig::new(ConnectionGeneration(60), vec![profile("lay-us")]).unwrap(),
    )
    .await
}

async fn hold_default_activation_marker(harness: &mut Harness, native: bool) -> (u64, Instant) {
    let old_deadline = Instant::now() + Duration::from_millis(5);
    if native {
        harness.adapter.start_native_activation(
            engine_path(TARGET_PATH),
            context(CONTEXT_PATH),
            Default::default(),
        )
    } else {
        harness
            .adapter
            .start_compatibility_activation(engine_path(TARGET_PATH), Default::default())
    }
    .unwrap();
    if !native {
        let get = next_peer_message(&mut harness.peer).await;
        assert_eq!(get.header().member().unwrap().as_str(), "Get");
        harness
            .peer
            .connection
            .reply(
                &get.header(),
                &OwnedValue::from(ObjectPath::try_from(CONTEXT_PATH).unwrap()),
            )
            .await
            .unwrap();
    }
    let marker = next_peer_message(&mut harness.peer).await;
    assert_eq!(marker.header().member().unwrap().as_str(), MARKER_MEMBER);
    (marker.body().deserialize::<u64>().unwrap(), old_deadline)
}

async fn deliver_held_marker(harness: &mut Harness, nonce: u64) {
    let forwarded = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
        .unwrap()
        .sender(ADAPTER_SENDER)
        .unwrap()
        .build(&nonce)
        .unwrap();
    harness.peer.connection.send(&forwarded).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
}

async fn default_activation_survives_delayed_marker(native: bool) {
    let mut harness = default_budget_harness().await;
    let (nonce, old_deadline) = hold_default_activation_marker(&mut harness, native).await;
    // Release a held protocol event past the measured failing deadline. This
    // is a causal deadline violation, not preparation time before user input.
    async_io::Timer::at(old_deadline + Duration::from_millis(1)).await;
    assert!(Instant::now() > old_deadline);
    deliver_held_marker(&mut harness, nonce).await;
    assert!(
        harness
            .adapter
            .pending_activation_for(&engine_path(TARGET_PATH))
            .is_some(),
        "authenticated activation must remain publishable beyond the 5 ms bridge budget"
    );
    let mut engine = new_engine(&harness);
    assert!(!legacy_key(&mut harness, &mut engine, 14_300, KEY_LEFT_SHIFT, 42, 0).await);
    let token = engine
        .live_context_token()
        .expect("real legacy callback consumes the published activation");
    assert!(harness.adapter.revalidate(&token));
    assert_eq!(token.activation.context, context(CONTEXT_PATH));
    assert!(!engine.context_word_is_known());
    assert!(engine.committed_tail.buffer.is_empty());
    assert!(engine.capture_input_frame_identity().is_none());
    assert!(harness
        .adapter
        .pending_activation_for(&engine_path(TARGET_PATH))
        .is_none());
    assert!(drain_output_to_proof(&mut harness).await.is_empty());
}

#[test]
fn td121_default_native_activation_survives_marker_beyond_bridge_deadline() {
    zbus::block_on(bounded(default_activation_survives_delayed_marker(true)));
}

#[test]
fn td121_default_compatibility_activation_survives_marker_beyond_bridge_deadline() {
    zbus::block_on(bounded(default_activation_survives_delayed_marker(false)));
}

#[test]
fn td121_default_bridge_still_refuses_the_same_delayed_marker() {
    zbus::block_on(bounded(async {
        let mut harness = default_budget_harness().await;
        let engine = known_engine(&mut harness).await;
        let token = engine.live_context_token().unwrap();
        let began = Instant::now();
        let (fence, nonce) = future::zip(harness.adapter.begin_bridge_fence(), async {
            let ping = next_peer_message(&mut harness.peer).await;
            assert_eq!(ping.header().member().unwrap().as_str(), "Ping");
            let value = ping.body().deserialize::<OwnedValue>().unwrap();
            harness
                .peer
                .connection
                .reply(&ping.header(), &value)
                .await
                .unwrap();
            let marker = next_peer_message(&mut harness.peer).await;
            assert_eq!(marker.header().member().unwrap().as_str(), MARKER_MEMBER);
            marker.body().deserialize::<u64>().unwrap()
        })
        .await;
        let fence = fence.expect("immediate Ping/marker emission fits the bridge budget");
        assert!(fence.deadline - began < Duration::from_millis(6));
        async_io::Timer::at(fence.deadline + Duration::from_millis(1)).await;
        deliver_held_marker(&mut harness, nonce).await;
        assert!(harness.adapter.finish_bridge_fence(fence).is_err());
        assert!(engine.context_word_is_known());
        assert_eq!(engine.live_context_token().as_ref(), Some(&token));
        assert!(drain_output_to_proof(&mut harness).await.is_empty());
    }));
}

#[test]
fn td121_delayed_activation_revalidates_nonce_and_lifecycle() {
    zbus::block_on(bounded(async {
        for native in [false, true] {
            for revoke in [false, true] {
                let mut harness = default_budget_harness().await;
                let (nonce, old_deadline) =
                    hold_default_activation_marker(&mut harness, native).await;
                if revoke {
                    let _ = receive(&mut harness, 14_310, "FocusOut").await;
                }
                async_io::Timer::at(old_deadline + Duration::from_millis(1)).await;
                deliver_held_marker(&mut harness, nonce + 1).await;
                assert!(harness
                    .adapter
                    .pending_activation_for(&engine_path(TARGET_PATH))
                    .is_none());
                deliver_held_marker(&mut harness, nonce).await;
                assert_eq!(
                    harness
                        .adapter
                        .pending_activation_for(&engine_path(TARGET_PATH))
                        .is_some(),
                    !revoke,
                    "native={native}, revoke={revoke}"
                );
            }
        }
    }));
}

#[test]
fn residual_pending_get_rejects_unrelated_or_malformed_content_type_without_revival() {
    zbus::block_on(bounded(async {
        for invalid in 0..3 {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = known_engine(&mut harness).await;
            let old_token = engine.live_context_token().unwrap();
            actual_focus_out(&mut harness, &mut engine, 3_250).await;
            let focus = receive(&mut harness, 3_251, "FocusIn").await;
            bounded(engine.focus_in_callback(focus.header())).await;
            let held_get = bounded(next_peer_message(&mut harness.peer)).await;
            assert_eq!(held_get.header().member().unwrap().as_str(), "Get");
            let (generation, nonce) = {
                let reducer = harness.adapter.shared.reducer.lock().unwrap();
                let request = reducer.request.as_ref().unwrap();
                (request.generation, request.nonce)
            };
            let interface = if invalid == 0 {
                IBUS_INTERFACE
            } else {
                ENGINE_INTERFACE
            };
            let property = if invalid == 1 {
                "UnrelatedProperty"
            } else {
                "ContentType"
            };
            let value = if invalid == 2 {
                zbus::zvariant::Value::from(10u32)
            } else {
                zbus::zvariant::Value::from((10u32, 0u32))
            };
            let set = Message::method_call(TARGET_PATH, "Set")
                .unwrap()
                .interface(PROPERTIES_INTERFACE)
                .unwrap()
                .sender(DISPATCH_SENDER)
                .unwrap()
                .serial(NonZeroU32::new(3_252).unwrap())
                .build(&(interface, property, value))
                .unwrap();
            harness.peer.connection.send(&set).await.unwrap();
            assert!(bounded(harness.observer.process_next()).await.is_err());
            assert!(harness.adapter.shared.cancelled.load(Ordering::Acquire));
            assert!(!harness.adapter.revalidate(&old_token));
            let mut reducer = harness.adapter.shared.reducer.lock().unwrap();
            assert!(reducer.request.is_none());
            let context = ContextKey::new(reducer.connection, CONTEXT_PATH).unwrap();
            assert!(!reducer.context_reply(generation, nonce, context, set.recv_position()));
            assert!(!reducer.marker(generation, nonce, set.recv_position()));
            assert!(reducer.consume().is_none());
            assert!(reducer.consume_source_free_activation().is_none());
        }
    }));
}

async fn empty_factory_stale_focus_case(predecessor_bound: bool) {
    const STALE_PATH: &str = "/io/github/radislabus_star/LayIme/engine/stale_empty_factory";

    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let shared = Arc::new(Mutex::new(SharedState::default()));
    let predecessor = factory_message(3_230);
    harness.peer.connection.send(&predecessor).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    let predecessor_callback = harness
        .adapter
        .begin_factory_callback(&predecessor.header(), Instant::now(), profile("lay-us"))
        .await
        .unwrap();
    if predecessor_bound {
        assert!(harness
            .adapter
            .bind_factory_target(&predecessor_callback, engine_path(STALE_PATH)));
    }

    let successor = factory_message(3_231);
    harness.peer.connection.send(&successor).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    let successor_callback = harness
        .adapter
        .begin_factory_callback(&successor.header(), Instant::now(), profile("lay-us"))
        .await
        .expect("later authenticated empty factory supersedes predecessor");
    assert!(!harness
        .adapter
        .bind_factory_target(&predecessor_callback, engine_path(STALE_PATH)));
    assert!(harness
        .adapter
        .bind_factory_target(&successor_callback, engine_path(TARGET_PATH)));

    let mut stale = LayIbusEngine::new_from_component(
        STALE_PATH.to_string(),
        shared.clone(),
        Some(harness.adapter.clone()),
        "lay-ime-us",
        true,
        ime_config(),
    );
    let stale_focus = method_message(
        DISPATCH_SENDER,
        3_232,
        STALE_PATH,
        ENGINE_INTERFACE,
        "FocusInId",
    );
    harness.peer.connection.send(&stale_focus).await.unwrap();
    let ((), observed) = bounded(future::zip(
        stale.focus_in_id(
            stale_focus.header(),
            CONTEXT_PATH.to_string(),
            "stale-client".to_string(),
        ),
        harness.observer.process_next(),
    ))
    .await;
    assert!(observed.unwrap());
    assert!(stale.live_context_token().is_none());

    let mut current = LayIbusEngine::new_from_component(
        TARGET_PATH.to_string(),
        shared,
        Some(harness.adapter.clone()),
        "lay-ime-us",
        true,
        ime_config(),
    );
    let current_focus = receive(&mut harness, 3_233, "FocusInId").await;
    bounded(current.focus_in_id(
        current_focus.header(),
        CONTEXT_PATH.to_string(),
        "current-client".to_string(),
    ))
    .await;
    current.config = ime_config();
    forward_marker_bounded(&mut harness.peer).await;
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    assert!(!legacy_key(&mut harness, &mut current, 3_234, KEY_LEFT_SHIFT, 42, 0,).await);
    assert!(
        !legacy_key(
            &mut harness,
            &mut current,
            3_235,
            KEY_LEFT_SHIFT,
            42,
            RELEASE_MASK,
        )
        .await
    );
    let token = current
        .live_context_token()
        .expect("successor acquires after stale predecessor FocusIn");
    assert_eq!(token.lineage.completeness, WordCompleteness::UnknownStart);
    assert!(current.committed_tail.buffer.is_empty());
    assert!(bridge_fence(&mut harness).await.is_ok());
    assert!(legacy_key(&mut harness, &mut current, 3_236, KEY_SPACE, 57, 0).await);
    expect_legacy_commit(&mut harness.peer).await;
    assert!(current.context_word_is_known());

    let live_token = current.live_context_token().unwrap();
    let retained_tail = current.committed_tail.buffer.clone();
    let late_stale_focus = method_message(
        DISPATCH_SENDER,
        3_237,
        STALE_PATH,
        ENGINE_INTERFACE,
        "FocusInId",
    );
    harness
        .peer
        .connection
        .send(&late_stale_focus)
        .await
        .unwrap();
    let ((), observed) = bounded(future::zip(
        stale.focus_in_id(
            late_stale_focus.header(),
            CONTEXT_PATH.to_string(),
            "stale-client".to_string(),
        ),
        harness.observer.process_next(),
    ))
    .await;
    assert!(observed.unwrap());
    assert!(harness.adapter.revalidate(&live_token));
    assert!(current.context_word_is_known());
    assert_eq!(current.committed_tail.buffer, retained_tail);
    assert!(bridge_fence(&mut harness).await.is_ok());
}

#[test]
fn residual_later_empty_factory_isolated_from_stale_focus_before_or_after_old_bind() {
    zbus::block_on(bounded(async {
        empty_factory_stale_focus_case(false).await;
        empty_factory_stale_focus_case(true).await;
    }));
}

#[test]
fn residual_declined_authenticated_factory_is_passive_and_later_factory_recovers() {
    zbus::block_on(bounded(async {
        const RECOVERED_PATH: &str =
            "/io/github/radislabus_star/LayIme/engine/recovered_after_declined_factory";

        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let source = known_engine(&mut harness).await;
        let old_token = source.live_context_token().unwrap();

        let first_factory = factory_message(3_180);
        harness.peer.connection.send(&first_factory).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        let first_callback = harness
            .adapter
            .begin_factory_callback(&first_factory.header(), Instant::now(), profile("lay-us"))
            .await
            .unwrap();

        // This authenticated, well-formed callback reaches the reducer while
        // the first ticket is still pending. The reducer refuses it and revokes
        // the old word authority; the observer must nevertheless remain usable.
        let declined_factory = factory_message(3_181);
        harness
            .peer
            .connection
            .send(&declined_factory)
            .await
            .unwrap();
        assert!(bounded(harness.observer.process_next())
            .await
            .expect("a declined factory transition is not observer transport loss"));
        assert!(matches!(
            harness
                .adapter
                .begin_factory_callback(
                    &declined_factory.header(),
                    Instant::now(),
                    profile("lay-us"),
                )
                .await,
            Err(AdapterError::Denied)
        ));
        assert!(!harness.adapter.revalidate(&old_token));
        {
            let reducer = harness.adapter.shared.reducer.lock().unwrap();
            assert_eq!(reducer.status(), AdmissionStatus::Revoked);
            assert!(reducer.request.is_none());
        }

        let fresh_factory = factory_message(3_182);
        harness.peer.connection.send(&fresh_factory).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        let fresh_callback = harness
            .adapter
            .begin_factory_callback(&fresh_factory.header(), Instant::now(), profile("lay-us"))
            .await
            .expect("later factory receives a fresh source-free reservation");
        assert!(!harness.adapter.bind_factory_target(
            &first_callback,
            engine_path("/io/github/radislabus_star/LayIme/engine/stale_declined_factory"),
        ));
        assert!(harness
            .adapter
            .bind_factory_target(&fresh_callback, engine_path(RECOVERED_PATH)));

        let mut recovered = LayIbusEngine::new_from_component(
            RECOVERED_PATH.to_string(),
            source.shared.clone(),
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        let focus = method_message(
            DISPATCH_SENDER,
            3_183,
            RECOVERED_PATH,
            ENGINE_INTERFACE,
            "FocusInId",
        );
        harness.peer.connection.send(&focus).await.unwrap();
        let ((), observed) = bounded(future::zip(
            recovered.focus_in_id(
                focus.header(),
                CONTEXT_PATH.to_string(),
                "controlled-client".to_string(),
            ),
            harness.observer.process_next(),
        ))
        .await;
        assert!(observed.unwrap());
        recovered.config = ime_config();
        forward_marker_bounded(&mut harness.peer).await;
        assert!(bounded(harness.observer.process_next()).await.unwrap());

        assert!(
            !legacy_key_at(
                &mut harness,
                &mut recovered,
                RECOVERED_PATH,
                3_184,
                KEY_LEFT_SHIFT,
                42,
                0,
            )
            .await
        );
        let recovered_token = recovered
            .live_context_token()
            .expect("fresh focus and key install a source-free owner");
        assert_eq!(
            recovered_token.lineage.completeness,
            WordCompleteness::UnknownStart
        );
        assert!(!harness.adapter.revalidate(&old_token));
        assert!(bridge_fence(&mut harness).await.is_ok());
    }));
}

#[test]
fn residual_later_factory_retires_revoked_transfer_and_recovers_source_free() {
    zbus::block_on(bounded(async {
        const RECOVERED_PATH: &str =
            "/io/github/radislabus_star/LayIme/engine/recovered_after_revocation";

        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut source = known_engine(&mut harness).await;
        let old_token = source.live_context_token().unwrap();
        source.committed_tail.buffer = "old-tail-must-not-transfer".to_string();
        {
            let mut shared = source.shared.lock().unwrap();
            shared.handoff_tail_buffer = source.committed_tail.buffer.clone();
            shared.handoff_tail_epoch = source.committed_tail.epoch;
        }

        let old_factory = factory_message(3_200);
        harness.peer.connection.send(&old_factory).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        let old_factory_callback = harness
            .adapter
            .begin_factory_callback(&old_factory.header(), Instant::now(), profile("lay-us"))
            .await
            .unwrap();
        assert!(harness
            .adapter
            .bind_factory_target(&old_factory_callback, engine_path(SOURCE_PATH)));

        actual_focus_out(&mut harness, &mut source, 3_201).await;
        actual_disable(&mut harness, &mut source, 3_202).await;

        let mut abandoned_target = LayIbusEngine::new_from_component(
            SOURCE_PATH.to_string(),
            source.shared.clone(),
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        let old_focus = method_message(
            DISPATCH_SENDER,
            3_203,
            SOURCE_PATH,
            ENGINE_INTERFACE,
            "FocusInId",
        );
        harness.peer.connection.send(&old_focus).await.unwrap();
        let ((), observed) = bounded(future::zip(
            abandoned_target.focus_in_id(
                old_focus.header(),
                CONTEXT_PATH.to_string(),
                "controlled-client".to_string(),
            ),
            harness.observer.process_next(),
        ))
        .await;
        assert!(observed.unwrap());

        // Hold the real marker so the revoked acquisition also retains an old
        // unobserved pending fence, matching the live recurrence boundary.
        let old_marker = bounded(next_peer_message(&mut harness.peer)).await;
        assert_eq!(
            old_marker.header().member().unwrap().as_str(),
            MARKER_MEMBER
        );
        let old_nonce = BarrierNonce(old_marker.body().deserialize::<u64>().unwrap());
        let old_fence = {
            let pending = harness.adapter.shared.pending.lock().unwrap();
            let pending = pending.as_ref().expect("old acquisition fence");
            assert_eq!(pending.nonce, old_nonce);
            PendingFence {
                nonce: pending.nonce,
                deadline: pending.deadline,
            }
        };
        let old_request = harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .request
            .as_ref()
            .expect("old transfer request")
            .generation;
        assert!(harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .context_acquisition_failed(old_request));
        assert!(!harness.adapter.revalidate(&old_token));

        let fresh_factory = factory_message(3_204);
        harness.peer.connection.send(&fresh_factory).await.unwrap();
        assert!(
            bounded(harness.observer.process_next()).await.unwrap(),
            "a later valid factory must not terminate observation on a revoked transfer"
        );
        let fresh_factory_callback = harness
            .adapter
            .begin_factory_callback(&fresh_factory.header(), Instant::now(), profile("lay-us"))
            .await
            .expect("later factory receives a source-free reservation");

        assert!(
            !harness.adapter.bind_factory_target(
                &old_factory_callback,
                engine_path("/io/github/radislabus_star/LayIme/engine/stale_factory"),
            ),
            "late binding from the retired ticket is a no-op"
        );
        assert!(harness
            .adapter
            .bind_factory_target(&fresh_factory_callback, engine_path(RECOVERED_PATH)));
        assert!(
            harness.adapter.shared.pending.lock().unwrap().is_none(),
            "factory retirement also removes the held stale fence"
        );

        let mut recovered = LayIbusEngine::new_from_component(
            RECOVERED_PATH.to_string(),
            source.shared.clone(),
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        let fresh_focus = method_message(
            DISPATCH_SENDER,
            3_205,
            RECOVERED_PATH,
            ENGINE_INTERFACE,
            "FocusInId",
        );
        harness.peer.connection.send(&fresh_focus).await.unwrap();
        let ((), observed) = bounded(future::zip(
            recovered.focus_in_id(
                fresh_focus.header(),
                CONTEXT_PATH.to_string(),
                "controlled-client".to_string(),
            ),
            harness.observer.process_next(),
        ))
        .await;
        assert!(observed.unwrap());
        recovered.config = ime_config();

        let (fresh_request, fresh_nonce) = {
            let reducer = harness.adapter.shared.reducer.lock().unwrap();
            let request = reducer.request.as_ref().expect("fresh source-free request");
            (request.generation, request.nonce)
        };
        let fresh_marker = bounded(next_peer_message(&mut harness.peer)).await;
        assert_eq!(fresh_marker.header().message_type(), Type::Signal);
        assert_eq!(
            fresh_marker.header().interface().unwrap().as_str(),
            MARKER_INTERFACE
        );
        assert_eq!(
            fresh_marker.header().member().unwrap().as_str(),
            MARKER_MEMBER
        );
        let fresh_marker_nonce = fresh_marker.body().deserialize::<u64>().unwrap();
        assert_eq!(fresh_marker_nonce, fresh_nonce.0);
        {
            let mut reducer = harness.adapter.shared.reducer.lock().unwrap();
            let revision = reducer.revocation_generation();
            assert!(!reducer.context_reply(
                old_request,
                old_nonce,
                context(CONTEXT_PATH),
                fresh_focus.recv_position(),
            ));
            assert!(!reducer.context_acquisition_failed(old_request));
            assert_eq!(reducer.revocation_generation(), revision);
            assert_eq!(reducer.request.as_ref().unwrap().generation, fresh_request);
            assert_eq!(reducer.request.as_ref().unwrap().nonce, fresh_nonce);
        }
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
            fresh_nonce,
            "late old timer cannot clear the successor fence"
        );

        let late_old_marker = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
            .unwrap()
            .sender(ADAPTER_SENDER)
            .unwrap()
            .build(&old_nonce.0)
            .unwrap();
        harness
            .peer
            .connection
            .send(&late_old_marker)
            .await
            .unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
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
            fresh_nonce,
            "late old marker cannot satisfy the successor fence"
        );

        let forwarded_fresh_marker = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
            .unwrap()
            .sender(ADAPTER_SENDER)
            .unwrap()
            .build(&fresh_marker_nonce)
            .unwrap();
        harness
            .peer
            .connection
            .send(&forwarded_fresh_marker)
            .await
            .unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        assert!(
            !legacy_key_at(
                &mut harness,
                &mut recovered,
                RECOVERED_PATH,
                3_206,
                KEY_LEFT_SHIFT,
                42,
                0,
            )
            .await
        );
        assert!(
            !legacy_key_at(
                &mut harness,
                &mut recovered,
                RECOVERED_PATH,
                3_207,
                KEY_LEFT_SHIFT,
                42,
                RELEASE_MASK,
            )
            .await
        );
        assert!(recovered.committed_tail.buffer.is_empty());
        assert!(!recovered.context_word_is_known());
        assert!(!harness.adapter.revalidate(&old_token));
        assert!(bridge_fence(&mut harness).await.is_ok());

        assert!(
            legacy_key_at(
                &mut harness,
                &mut recovered,
                RECOVERED_PATH,
                3_208,
                KEY_SPACE,
                57,
                0,
            )
            .await
        );
        expect_legacy_commit(&mut harness.peer).await;
        assert!(recovered.context_word_is_known());
        assert!(bridge_fence(&mut harness).await.is_ok());
    }));
}

#[test]
fn residual_missing_or_repeated_disable_revokes_and_completes_local_reset() {
    zbus::block_on(bounded(async {
        for opened_handoff in [false, true] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = known_engine(&mut harness).await;
            let old_token = engine.live_context_token().unwrap();
            if opened_handoff {
                actual_focus_out(&mut harness, &mut engine, 3_110).await;
                actual_disable(&mut harness, &mut engine, 3_111).await;
                assert!(engine.context_handoff_sealed);
            }
            engine.composition.buffer = "stale".to_string();
            engine.composition.preedit_visible = true;
            engine.atomic.active = true;
            actual_disable(&mut harness, &mut engine, 3_112).await;
            assert!(!engine.context_handoff_sealed);
            assert!(!engine.atomic.active);
            assert!(engine.composition.buffer.is_empty());
            assert!(!engine.composition.preedit_visible);
            assert!(!engine.context_word_is_known());
            assert!(!harness.adapter.revalidate(&old_token));
        }
    }));
}

#[test]
fn residual_lifecycle_bad_sender_or_payload_still_stops_observer() {
    zbus::block_on(bounded(async {
        for (sender, member) in [
            (":1.999", "FocusOut"),
            (":1.999", "Disable"),
            (DISPATCH_SENDER, "FocusOutId"),
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let engine = known_engine(&mut harness).await;
            let old_token = engine.live_context_token().unwrap();
            // FocusOutId requires a context string, not this unit payload.
            let message = method_message(sender, 3_120, TARGET_PATH, ENGINE_INTERFACE, member);
            harness.peer.connection.send(&message).await.unwrap();
            assert!(matches!(
                bounded(harness.observer.run()).await,
                Err(AdapterError::Denied)
            ));
            assert!(!harness.adapter.revalidate(&old_token));
            assert!(matches!(
                harness
                    .adapter
                    .observe_callback(&message.header(), Instant::now())
                    .await,
                Err(AdapterError::Cancelled)
            ));
        }
    }));
}

#[test]
fn residual_delayed_old_revocation_cannot_clean_up_different_path_successor() {
    zbus::block_on(bounded(async {
        for member in ["FocusOut", "Disable"] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut old = known_engine(&mut harness).await;
            let old_owner = old.context_owner.clone().unwrap();
            let delayed = if member == "FocusOut" {
                actual_focus_out(&mut harness, &mut old, 3_130).await;
                receive(&mut harness, 3_131, member).await
            } else {
                receive(&mut harness, 3_130, member).await
            };

            let shared = old.shared.clone();
            let grant = complete_native_activation(&mut harness).await;
            assert_eq!(grant.target_owner.path.as_str(), SOURCE_PATH);
            assert_ne!(grant.target_owner, old_owner);
            let mut successor = LayIbusEngine::new_from_component(
                SOURCE_PATH.to_string(),
                shared.clone(),
                Some(harness.adapter.clone()),
                "lay-ime-us",
                true,
                ime_config(),
            );
            assert!(successor.install_context_activation(ActivationOutcome::SourceFree(grant)));
            successor.committed_tail.buffer = "successor-tail".to_string();
            successor.publish_tail_handoff();
            let successor_token = successor.live_context_token().unwrap();
            let successor_state = {
                let state = shared.lock().unwrap();
                (
                    state.active_path.clone(),
                    state.context_owner_generation,
                    state.handoff_tail_buffer.clone(),
                    state.handoff_tail_epoch,
                )
            };
            assert_eq!(successor_state.0.as_deref(), Some(SOURCE_PATH));

            old.composition.buffer = "old-local-stale".to_string();
            old.composition.preedit_visible = true;
            old.atomic.active = true;
            if member == "FocusOut" {
                bounded(old.focus_out(delayed.header())).await;
            } else {
                bounded(old.disable(delayed.header())).await;
            }

            assert_eq!(old.composition.buffer, "old-local-stale", "{member}");
            assert!(old.composition.preedit_visible, "{member}");
            assert!(old.atomic.active, "{member}");
            let state = shared.lock().unwrap();
            assert_eq!(
                (
                    state.active_path.clone(),
                    state.context_owner_generation,
                    state.handoff_tail_buffer.clone(),
                    state.handoff_tail_epoch,
                ),
                successor_state,
                "{member}"
            );
            assert!(harness.adapter.revalidate(&successor_token), "{member}");
        }
    }));
}

#[test]
fn residual_bridge_fence_refuses_received_unsettled_key_or_focus_out() {
    zbus::block_on(bounded(async {
        for member in ["ProcessKeyEvent", "FocusOut"] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = known_engine(&mut harness).await;
            assert!(
                bridge_fence(&mut harness).await.is_ok(),
                "settled positive control"
            );
            let event = receive(&mut harness, 3_010, member).await;
            assert!(
                engine.live_context_token().is_some(),
                "ordinary current-key identity survives"
            );
            assert!(
                matches!(bridge_fence(&mut harness).await, Err(AdapterError::Denied)),
                "{member}"
            );
            if member == "ProcessKeyEvent" {
                assert!(
                    !run_received_legacy_key(&harness, &mut engine, &event, KEY_LEFT_SHIFT, 42, 0)
                        .await
                );
                assert!(
                    bridge_fence(&mut harness).await.is_ok(),
                    "settlement restores bridge reads"
                );
            }
        }
    }));
}

#[test]
fn td121_visible_tail_denial_is_passive_until_the_exact_callback_settles() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let engine = known_engine(&mut harness).await;
        let (mut engine, initial) = cycle09_visible_tail(&mut harness, engine).await;
        let initial = initial.expect("settled positive readout");

        let key = receive(&mut harness, 3_015, "ProcessKeyEvent").await;
        consume_detached_callback_reply(&mut harness.peer, 3_015).await;
        let path = engine.path.clone();
        let read_bridge = bridge(&harness, &engine);
        harness
            .connection
            .object_server()
            .at(path.as_str(), engine)
            .await
            .unwrap();
        let (denied, ()) = bounded(future::zip(read_bridge.visible_tail_v3_inner(), async {
            serve_ping_and_marker(&mut harness.peer).await;
            assert!(harness.observer.process_next().await.unwrap());
        }))
        .await;
        cycle09_assert_no_local_text_effect(&mut harness).await;
        engine = cycle09_take_registered_engine(&harness, &path).await;
        assert_eq!(
            denied.expect("an unsettled callback is a passive readout"),
            (
                "passive:unknown-context".to_string(),
                String::new(),
                false,
                0,
                String::new(),
                String::new(),
            )
        );

        let mutation_bridge = bridge(&harness, &engine);
        harness
            .connection
            .object_server()
            .at(path.as_str(), engine)
            .await
            .unwrap();
        let (mutation, ()) = bounded(future::zip(
            mutation_bridge.manual_toggle_v3_inner(),
            async {
                serve_ping_and_marker(&mut harness.peer).await;
                assert!(harness.observer.process_next().await.unwrap());
            },
        ))
        .await;
        cycle09_assert_no_local_text_effect(&mut harness).await;
        engine = cycle09_take_registered_engine(&harness, &path).await;
        assert_eq!(
            mutation.map_err(|error| error.to_string()),
            Err("org.freedesktop.DBus.Error.Failed: context admission denied".to_string()),
            "the same denial remains a hard error for mutation"
        );

        assert!(!run_received_legacy_key(&harness, &mut engine, &key, KEY_LEFT_SHIFT, 42, 0).await);
        cycle09_assert_no_local_text_effect(&mut harness).await;
        let (_engine, settled) = cycle09_visible_tail(&mut harness, engine).await;
        assert_eq!(
            settled.expect("a fresh fence reads the settled authority"),
            initial
        );
    }));
}

#[test]
fn residual_bridge_consumer_refuses_ingress_after_the_fence() {
    zbus::block_on(bounded(async {
        for member in ["ProcessKeyEvent", "FocusOut"] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let engine = known_engine(&mut harness).await;
            let token = bridge_fence(&mut harness).await.unwrap();
            let bridge = bridge(&harness, &engine);
            assert!(bridge.bridge_token_is_live(&engine, Some(&token)));
            receive(&mut harness, 3_020, member).await;
            assert!(
                harness.adapter.revalidate(&token),
                "do not invalidate the executing key"
            );
            assert!(
                !bridge.bridge_token_is_live(&engine, Some(&token)),
                "{member}"
            );
            assert_eq!(engine.committed_tail.buffer, " ");
        }
    }));
}

async fn leave_context(harness: &mut Harness, engine: &mut LayIbusEngine) {
    let focus_out = receive(harness, 3_030, "FocusOut").await;
    bounded(engine.focus_out(focus_out.header())).await;
    assert!(engine.context_handoff_sealed);
    let disable = receive(harness, 3_031, "Disable").await;
    bounded(engine.disable(disable.header())).await;
}

async fn bridge_toggle_terminal(
    harness: &mut Harness,
    engine: LayIbusEngine,
    expected: &str,
    target_is_ru: bool,
) -> LayIbusEngine {
    let path = engine.path.clone();
    let bridge = bridge(harness, &engine);
    harness
        .connection
        .object_server()
        .at(path.as_str(), engine)
        .await
        .unwrap();
    let (outcome, ()) = bounded(future::zip(bridge.manual_toggle_v3_inner(), async {
        serve_ping_and_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
    }))
    .await;
    assert_eq!(outcome.unwrap(), (1, target_is_ru));
    let commit = bounded(next_peer_message(&mut harness.peer)).await;
    assert_eq!(commit.header().member().unwrap().as_str(), "CommitText");
    let body = commit.body();
    let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
    assert_eq!(
        crate::ibus_interface::ibus_text_value_to_string(&value).as_deref(),
        Some(expected),
        "one exact terminal erase/commit frame"
    );
    // Keep the actual engine; unregister before the existing controlled
    // callback choreography so the object server cannot dispatch it twice.
    let iface = harness
        .connection
        .object_server()
        .interface::<_, LayIbusEngine>(path.as_str())
        .await
        .unwrap();
    let engine = std::mem::replace(&mut *iface.get_mut().await, new_engine(harness));
    harness
        .connection
        .object_server()
        .remove::<LayIbusEngine, _>(path.as_str())
        .await
        .unwrap();
    engine
}

async fn drain_output_to_proof(harness: &mut Harness) -> Vec<String> {
    harness
        .connection
        .emit_signal(None::<&str>, TARGET_PATH, "org.lay.Proof", "Reached", &())
        .await
        .unwrap();
    let mut members = Vec::new();
    loop {
        let message = bounded(next_peer_message(&mut harness.peer)).await;
        let Some(member) = message
            .header()
            .member()
            .map(|member| member.as_str().to_string())
        else {
            continue;
        };
        if member == "Reached" {
            return members;
        }
        members.push(member);
    }
}

async fn drain_output_to_text_proof(harness: &mut Harness) -> (Vec<String>, Vec<String>) {
    harness
        .connection
        .emit_signal(None::<&str>, TARGET_PATH, "org.lay.Proof", "Reached", &())
        .await
        .unwrap();
    let mut members = Vec::new();
    let mut committed = Vec::new();
    loop {
        let message = bounded(next_peer_message(&mut harness.peer)).await;
        let Some(member) = message
            .header()
            .member()
            .map(|member| member.as_str().to_string())
        else {
            continue;
        };
        if member == "Reached" {
            return (members, committed);
        }
        if member == "CommitText" {
            let body = message.body();
            let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
            committed.push(
                crate::ibus_interface::ibus_text_value_to_string(&value)
                    .expect("CommitText string"),
            );
        }
        members.push(member);
    }
}

async fn serve_next_ping_and_marker(peer: &mut ControlledPeer) {
    loop {
        let message = bounded(next_peer_message(peer)).await;
        let header = message.header();
        let Some(member) = header.member() else {
            assert!(matches!(
                header.message_type(),
                Type::MethodReturn | Type::Error
            ));
            continue;
        };
        assert_eq!(member.as_str(), "Ping");
        let value = message.body().deserialize::<OwnedValue>().unwrap();
        peer.connection
            .reply(&message.header(), &value)
            .await
            .unwrap();
        forward_marker(peer).await;
        return;
    }
}

async fn bridge_delegate_exact_without_gui_edit(
    harness: &mut Harness,
    engine: LayIbusEngine,
) -> LayIbusEngine {
    let path = engine.path.clone();
    let tail_before = engine.committed_tail.buffer.clone();
    let epoch_before = engine.committed_tail.epoch;
    let bridge = bridge(harness, &engine);
    harness
        .connection
        .object_server()
        .at(path.as_str(), engine)
        .await
        .unwrap();
    let (outcome, ()) = bounded(future::zip(bridge.manual_toggle_v3_inner(), async {
        serve_next_ping_and_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
    }))
    .await;
    assert_eq!(outcome.unwrap(), (3, false));
    assert!(drain_output_to_proof(harness)
        .await
        .iter()
        .all(|member| !matches!(member.as_str(), "DeleteSurroundingText" | "CommitText")));

    let iface = harness
        .connection
        .object_server()
        .interface::<_, LayIbusEngine>(path.as_str())
        .await
        .unwrap();
    let engine = std::mem::replace(&mut *iface.get_mut().await, new_engine(harness));
    harness
        .connection
        .object_server()
        .remove::<LayIbusEngine, _>(path.as_str())
        .await
        .unwrap();
    assert_eq!(engine.committed_tail.buffer, tail_before);
    assert_eq!(engine.committed_tail.epoch, epoch_before.wrapping_add(1));
    assert!(engine.exact_manual_toggle_handoff_is_live());
    engine
}

pub(super) async fn surrounding_receipt(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    text: &str,
    cursor_pos: u32,
    anchor_pos: u32,
) {
    engine
        .set_surrounding_text(
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap(),
            crate::text::make_ibus_text(text.to_string()),
            cursor_pos,
            anchor_pos,
        )
        .await
        .unwrap();
    assert!(drain_output_to_proof(harness)
        .await
        .iter()
        .all(|member| !matches!(member.as_str(), "DeleteSurroundingText" | "CommitText")));
}

async fn exact_surrounding_receipt(harness: &mut Harness, engine: &mut LayIbusEngine, text: &str) {
    let chars = text.chars().count() as u32;
    surrounding_receipt(harness, engine, text, chars, chars).await;
}

async fn observer_first_reset_unknown_tail(harness: &mut Harness, serial: u32) -> LayIbusEngine {
    let mut engine = new_engine(harness);
    start_source_free_unknown(harness, &mut engine).await;
    engine.config.auto_replace = false;
    engine.config.typing_assist = false;
    engine.config.nanda_precognition = false;
    engine.client_context.content_purpose = 0;
    engine.client_context.cursor_cell_width = 0;
    engine.client_context.surrounding_text_supported = true;

    for (offset, (ch, code)) in [('a', 30), ('b', 48), ('c', 46), ('d', 32), ('e', 18)]
        .into_iter()
        .enumerate()
    {
        let press_serial = serial + (offset as u32 * 2);
        assert!(legacy_key(harness, &mut engine, press_serial, ch as u32, code, 0).await);
        expect_legacy_commit(&mut harness.peer).await;
        assert!(
            legacy_key(
                harness,
                &mut engine,
                press_serial + 1,
                ch as u32,
                code,
                RELEASE_MASK,
            )
            .await
        );
    }
    assert_eq!(engine.committed_tail.buffer, "abcde");
    exact_surrounding_receipt(harness, &mut engine, "abcde").await;

    let reset = receive(harness, serial + 10, "Reset").await;
    engine
        .reset(
            reset.header(),
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        engine.context_word_scope.as_ref().unwrap().lineage(),
        engine
            .context_token
            .as_ref()
            .unwrap()
            .word_scope()
            .lineage(),
        "Reset must install exactly the reducer's post-Reset lineage"
    );
    assert!(engine.context_reset_rereceipt.is_some());
    engine
}

async fn bridge_toggle_refused_without_text_effect(
    harness: &mut Harness,
    engine: LayIbusEngine,
) -> LayIbusEngine {
    let path = engine.path.clone();
    let tail_before = engine.committed_tail.buffer.clone();
    let epoch_before = engine.committed_tail.epoch;
    let bridge = bridge(harness, &engine);
    harness
        .connection
        .object_server()
        .at(path.as_str(), engine)
        .await
        .unwrap();
    let (outcome, ()) = bounded(future::zip(bridge.manual_toggle_v3_inner(), async {
        serve_next_ping_and_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
    }))
    .await;
    match outcome {
        Ok(outcome) => assert_eq!(outcome, (0, false)),
        Err(error) => assert!(error.to_string().ends_with("context admission denied")),
    }
    assert!(drain_output_to_proof(harness)
        .await
        .iter()
        .all(|member| !matches!(member.as_str(), "DeleteSurroundingText" | "CommitText")));

    let iface = harness
        .connection
        .object_server()
        .interface::<_, LayIbusEngine>(path.as_str())
        .await
        .unwrap();
    let engine = std::mem::replace(&mut *iface.get_mut().await, new_engine(harness));
    harness
        .connection
        .object_server()
        .remove::<LayIbusEngine, _>(path.as_str())
        .await
        .unwrap();
    assert_eq!(engine.committed_tail.buffer, tail_before);
    assert_eq!(engine.committed_tail.epoch, epoch_before);
    engine
}

#[test]
fn residual_reset_rereceipt_rejoins_typed_daemon_exact_route() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = observer_first_reset_unknown_tail(&mut harness, 8_600).await;
        assert_eq!(engine.committed_tail.buffer, "abcde");
        exact_surrounding_receipt(&mut harness, &mut engine, "abcde").await;
        assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());

        engine = bridge_delegate_exact_without_gui_edit(&mut harness, engine).await;
        assert_eq!(engine.committed_tail.buffer, "abcde");
    }));
}

#[test]
fn firefox_reset_retains_known_word_or_closed_observed_tail() {
    zbus::block_on(bounded(async {
        let keys = [('a', 30), ('b', 48), ('c', 46)];
        let mut failures = Vec::new();
        for (mode, release_state, allows_handoff) in [
            ("known_word", 0, true),
            ("space_boundary", 0, true),
            ("accepted_append_boundary", 0, true),
            ("accepted_alt_boundary", RELEASE_MASK, true),
            ("accepted_alt_boundary", RELEASE_MASK | (1 << 3), true),
            (
                "accepted_alt_boundary",
                RELEASE_MASK | (1 << 3) | (1 << 2),
                false,
            ),
            (
                "accepted_alt_boundary",
                RELEASE_MASK | (1 << 3) | (1 << 6),
                false,
            ),
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = if mode == "known_word" {
                let mut engine = known_engine(&mut harness).await;
                engine.config.auto_replace = false;
                engine.config.typing_assist = false;
                engine.config.nanda_precognition = false;
                engine.client_context.content_purpose = 0;
                engine.client_context.cursor_cell_width = 0;
                engine.client_context.surrounding_text_supported = true;
                for (offset, &(ch, code)) in keys.iter().enumerate() {
                    let serial = 15_000 + offset as u32 * 2;
                    assert!(
                        legacy_key(&mut harness, &mut engine, serial, ch as u32, code, 0).await
                    );
                    td121_expect_legacy_commit_text(&mut harness.peer, &ch.to_string()).await;
                    assert!(
                        legacy_key(
                            &mut harness,
                            &mut engine,
                            serial + 1,
                            ch as u32,
                            code,
                            RELEASE_MASK
                        )
                        .await
                    );
                }
                assert!(engine.context_word_is_known());
                engine
            } else {
                let mut engine = initial_observed_tail_reset(&mut harness, 15_000, &keys).await;
                exact_surrounding_receipt(&mut harness, &mut engine, "abc").await;
                assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                if mode == "space_boundary" {
                    assert!(legacy_key(&mut harness, &mut engine, 15_010, KEY_SPACE, 57, 0).await);
                    td121_expect_legacy_commit_text(&mut harness.peer, " ").await;
                } else {
                    set_fixture_append_completion(&mut engine, "xyz");
                    if mode == "accepted_alt_boundary" {
                        assert!(
                            !legacy_key(&mut harness, &mut engine, 15_010, KEY_LEFT_ALT, 64, 0)
                                .await
                        );
                        assert!(
                            legacy_key(
                                &mut harness,
                                &mut engine,
                                15_011,
                                KEY_LEFT_ALT,
                                64,
                                release_state,
                            )
                            .await
                        );
                    } else {
                        assert!(
                            legacy_key(&mut harness, &mut engine, 15_010, KEY_TAB, 15, 0).await
                        );
                    }
                    td121_expect_legacy_commit_text(&mut harness.peer, "xyz ").await;
                }
                assert!(engine.context_word_is_known());
                engine
            };
            let expected = match mode {
                "known_word" => " abc",
                "space_boundary" => "abc ",
                _ => "abcxyz ",
            };
            assert_eq!(engine.committed_tail.buffer, expected);
            actual_reset(&mut harness, &mut engine, 15_020, false).await;
            assert!(
                !engine.context_word_is_known(),
                "Reset still revokes generic authority"
            );
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "a local predecessor alone cannot authorize an edit"
            );
            exact_surrounding_receipt(&mut harness, &mut engine, expected).await;
            let display = engine.capture_observed_suffix_display_frame().is_some();
            let path = engine.path.clone();
            let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
            let (_, tail) = cycle09_visible_tail(&mut harness, engine).await;
            let expected_disposition = if allows_handoff {
                (3, false)
            } else {
                (0, false)
            };
            if disposition != Ok(expected_disposition)
                || cycle09_tail_is_authoritative(&tail, &path, false, expected) != allows_handoff
                || display != (mode == "known_word")
            {
                failures.push(format!(
                    "{mode}/{release_state}: disposition={disposition:?}, display={display}, exact_tail={}",
                    cycle09_tail_is_authoritative(&tail, &path, false, expected)
                ));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("; "));
    }));
}

#[test]
fn residual_reset_rereceipt_failures_never_delete_gui_text() {
    zbus::block_on(bounded(async {
        for gap in [
            "mismatch",
            "selection",
            "second_receipt",
            "navigation",
            "focus_out",
            "caps9",
            "sensitive",
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = observer_first_reset_unknown_tail(&mut harness, 8_700).await;

            match gap {
                "mismatch" => {
                    surrounding_receipt(&mut harness, &mut engine, "abcdf", 5, 5).await;
                }
                "selection" => {
                    surrounding_receipt(&mut harness, &mut engine, "abcde", 5, 0).await;
                }
                "second_receipt" => {
                    exact_surrounding_receipt(&mut harness, &mut engine, "abcde").await;
                    assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                    exact_surrounding_receipt(&mut harness, &mut engine, "abcde").await;
                }
                "navigation" => {
                    exact_surrounding_receipt(&mut harness, &mut engine, "abcde").await;
                    assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                    assert!(!legacy_key(&mut harness, &mut engine, 8_711, KEY_LEFT, 105, 0).await);
                }
                "focus_out" => {
                    exact_surrounding_receipt(&mut harness, &mut engine, "abcde").await;
                    assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                    actual_focus_out(&mut harness, &mut engine, 8_711).await;
                }
                "caps9" => {
                    exact_surrounding_receipt(&mut harness, &mut engine, "abcde").await;
                    assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                    engine.set_client_capabilities(1 | 1 << 3);
                }
                "sensitive" => {
                    exact_surrounding_receipt(&mut harness, &mut engine, "abcde").await;
                    assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                    engine.set_content_type_state(8, 0);
                }
                _ => unreachable!(),
            }

            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{gap} must revoke exact GUI authority"
            );
            engine = bridge_toggle_refused_without_text_effect(&mut harness, engine).await;
            assert!(
                engine.context_reset_rereceipt.is_none(),
                "{gap} must not retain a reusable receipt"
            );
        }
    }));
}

async fn initial_observed_tail_reset(
    harness: &mut Harness,
    serial: u32,
    keys: &[(char, u32)],
) -> LayIbusEngine {
    let mut engine = new_engine(harness);
    start_source_free_unknown(harness, &mut engine).await;
    engine.config.auto_replace = false;
    engine.config.typing_assist = false;
    engine.config.nanda_precognition = false;
    engine.client_context.content_purpose = 0;
    engine.client_context.cursor_cell_width = 0;
    engine.client_context.surrounding_text_supported = true;
    for (offset, &(ch, code)) in keys.iter().enumerate() {
        let serial = serial + offset as u32 * 2;
        assert!(legacy_key(harness, &mut engine, serial, ch as u32, code, 0).await);
        expect_legacy_commit(&mut harness.peer).await;
        assert!(
            legacy_key(
                harness,
                &mut engine,
                serial + 1,
                ch as u32,
                code,
                RELEASE_MASK
            )
            .await
        );
    }
    assert_eq!(
        engine.committed_tail.buffer,
        keys.iter().map(|key| key.0).collect::<String>()
    );
    actual_reset(harness, &mut engine, serial + keys.len() as u32 * 2, false).await;
    assert!(
        engine.context_reset_rereceipt.is_some(),
        "initial Reset serial={serial} token={:?} owner={:?} scope={:?} current_owner={:?} epoch={} tail={:?}",
        engine.context_token,
        engine.context_owner,
        engine.context_word_scope,
        harness.adapter.current_owner(),
        engine.committed_tail.epoch,
        engine.committed_tail.buffer,
    );
    engine
}

async fn actual_reset(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    serial: u32,
    detached: bool,
) {
    let reset = receive(harness, serial, "Reset").await;
    if detached {
        consume_detached_callback_reply(&mut harness.peer, serial).await;
    }
    engine
        .reset(
            reset.header(),
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap(),
        )
        .await
        .unwrap();
}

#[test]
fn firefox_initial_delayed_prefix_recovers_only_after_append_reset_and_exact_receipt() {
    zbus::block_on(bounded(async {
        let keys = [('a', 30), ('b', 48), ('c', 46), ('d', 32), ('e', 18)];
        for initial_len in [2, 5] {
            for delayed in [false, true] {
                let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
                let mut engine =
                    initial_observed_tail_reset(&mut harness, 9_600, &keys[..initial_len]).await;
                let initial = engine.committed_tail.buffer.clone();
                let first = if delayed {
                    &initial[..initial_len - 1]
                } else {
                    &initial
                };
                exact_surrounding_receipt(&mut harness, &mut engine, first).await;
                let retained_after_first = engine.context_reset_rereceipt.is_some();
                assert_eq!(
                    engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                    !delayed
                );
                if delayed {
                    let (returned, visible) = cycle09_visible_tail(&mut harness, engine).await;
                    engine = returned;
                    assert_eq!(visible.as_ref().unwrap().0, "passive:unknown-context");
                    assert!(visible.as_ref().unwrap().1.is_empty());
                    // ManualToggleV3 is a mutating user command after the
                    // browser-Reset pending contract. This test owns only the
                    // reset/rereceipt lifecycle; pending execution is proven
                    // separately at bridge level.
                }

                if delayed {
                    // cycle09_visible_tail starts zbus's dispatcher. Drive the
                    // now-detached callback explicitly in this branch.
                    let key = receive(&mut harness, 9_620, "ProcessKeyEvent").await;
                    consume_detached_callback_reply(&mut harness.peer, 9_620).await;
                    assert!(
                        run_received_legacy_key(&harness, &mut engine, &key, 'f' as u32, 33, 0)
                            .await
                    );
                } else {
                    assert!(legacy_key(&mut harness, &mut engine, 9_620, 'f' as u32, 33, 0).await);
                }
                expect_legacy_commit(&mut harness.peer).await;
                let appended = format!("{initial}f");
                assert_eq!(engine.committed_tail.buffer, appended);
                let retained_after_append = engine.context_reset_rereceipt.is_some();
                if let Some(pending) = engine.context_reset_rereceipt.as_ref() {
                    assert_eq!(pending.confirmed, !delayed);
                    assert_eq!(pending.token_text, appended);
                }
                assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());

                actual_reset(&mut harness, &mut engine, 9_621, delayed).await;
                let retained_after_reset = engine.context_reset_rereceipt.is_some();
                assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                exact_surrounding_receipt(&mut harness, &mut engine, &appended).await;
                assert_eq!(
                    engine
                        .context_word_scope
                        .as_ref()
                        .unwrap()
                        .lineage()
                        .completeness,
                    WordCompleteness::UnknownStart
                );
                let recovered = engine.context_reset_rereceipt_exact_manual_handoff_allowed();
                assert_eq!(
                    (retained_after_first, retained_after_append, retained_after_reset, recovered),
                    (true, true, true, true),
                    "initial_len={initial_len} delayed={delayed}: candidate loss is distinct from premature authority"
                );
                let (engine, visible) = cycle09_visible_tail(&mut harness, engine).await;
                assert!(cycle09_tail_is_authoritative(
                    &visible,
                    &engine.path,
                    false,
                    &appended
                ));
                let (_engine, toggle) = cycle09_manual_toggle(&mut harness, engine).await;
                assert_eq!(toggle, Ok((3, false)));
            }
        }
    }));
}

#[test]
fn firefox_initial_delayed_prefix_cannot_survive_contradiction_or_input_gap() {
    zbus::block_on(bounded(async {
        for gap in [
            "wrong_text",
            "selection",
            "empty",
            "cursor_inside",
            "duplicate_prefix",
            "navigation",
            "backspace",
            "boundary",
            "focus_out",
            "caps9",
            "sensitive",
            "stale_owner",
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = initial_observed_tail_reset(
                &mut harness,
                9_700,
                &[('a', 30), ('b', 48), ('c', 46)],
            )
            .await;
            if !matches!(gap, "wrong_text" | "selection" | "empty" | "cursor_inside") {
                exact_surrounding_receipt(&mut harness, &mut engine, "ab").await;
                assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            }
            match gap {
                "wrong_text" => exact_surrounding_receipt(&mut harness, &mut engine, "ax").await,
                "selection" => surrounding_receipt(&mut harness, &mut engine, "ab", 2, 0).await,
                "empty" => exact_surrounding_receipt(&mut harness, &mut engine, "").await,
                "cursor_inside" => {
                    surrounding_receipt(&mut harness, &mut engine, "abc", 2, 2).await
                }
                "duplicate_prefix" => {
                    exact_surrounding_receipt(&mut harness, &mut engine, "ab").await
                }
                "navigation" => {
                    let _ = legacy_key(&mut harness, &mut engine, 9_710, KEY_LEFT, 105, 0).await;
                }
                "backspace" => {
                    let _ =
                        legacy_key(&mut harness, &mut engine, 9_710, KEY_BACKSPACE, 14, 0).await;
                }
                "boundary" => {
                    assert!(legacy_key(&mut harness, &mut engine, 9_710, KEY_SPACE, 57, 0).await);
                    expect_legacy_commit(&mut harness.peer).await;
                }
                "focus_out" => actual_focus_out(&mut harness, &mut engine, 9_710).await,
                "caps9" => engine.set_client_capabilities(1 | 1 << 3),
                "sensitive" => engine.set_content_type_state(8, 0),
                "stale_owner" => {
                    engine.context_owner.as_mut().unwrap().generation.0 += 1;
                }
                _ => unreachable!(),
            }
            actual_reset(&mut harness, &mut engine, 9_711, false).await;
            exact_surrounding_receipt(&mut harness, &mut engine, "abc").await;
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{gap}"
            );
            engine = bridge_toggle_refused_without_text_effect(&mut harness, engine).await;
            assert!(engine.context_reset_rereceipt.is_none(), "{gap}");
        }
    }));
}

fn set_fixture_append_completion(engine: &mut LayIbusEngine, suffix: &str) {
    engine.composition.preedit_suffix = suffix.into();
    engine.composition.preedit_candidates = vec![suffix.into()];
    engine.composition.preedit_replacement_targets = vec![None];
    engine.composition.preedit_visible = true;
}

#[test]
fn firefox_observed_reset_after_release_preserves_prefix_in_both_callback_orders() {
    zbus::block_on(bounded(async {
        let keys = [('a', 30), ('b', 48), ('c', 46)];
        for prefix_len in [1, 3] {
            for reset_handler_first in [false, true] {
                let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
                let mut engine = new_engine(&harness);
                start_source_free_unknown(&mut harness, &mut engine).await;
                engine.config.auto_replace = false;
                engine.config.typing_assist = false;
                engine.config.nanda_precognition = false;
                engine.client_context.content_purpose = 0;
                engine.client_context.cursor_cell_width = 0;
                engine.client_context.surrounding_text_supported = true;
                for (i, &(ch, code)) in keys[..prefix_len].iter().enumerate() {
                    let serial = 13_300 + i as u32 * 2;
                    assert!(
                        legacy_key(&mut harness, &mut engine, serial, ch as u32, code, 0).await
                    );
                    expect_legacy_commit(&mut harness.peer).await;
                    if i + 1 < prefix_len {
                        let _ = legacy_key(
                            &mut harness,
                            &mut engine,
                            serial + 1,
                            ch as u32,
                            code,
                            RELEASE_MASK,
                        )
                        .await;
                    }
                }
                let prefix = keys[..prefix_len]
                    .iter()
                    .map(|key| key.0)
                    .collect::<String>();
                let predecessor = engine.live_context_token().unwrap();
                let epoch = engine.committed_tail.epoch;
                let release = receive(&mut harness, 13_310, "ProcessKeyEvent").await;
                let reset = receive(&mut harness, 13_311, "Reset").await;
                assert!(!harness.adapter.revalidate(&predecessor));
                if reset_handler_first {
                    engine
                        .reset(
                            reset.header(),
                            zbus::object_server::SignalEmitter::new(
                                &harness.connection,
                                engine.path.clone(),
                            )
                            .unwrap(),
                        )
                        .await
                        .unwrap();
                    assert!(engine.context_reset_rereceipt.is_some());
                }
                let &(ch, code) = &keys[prefix_len - 1];
                let _ = run_received_legacy_key(
                    &harness,
                    &mut engine,
                    &release,
                    ch as u32,
                    code,
                    RELEASE_MASK,
                )
                .await;
                assert_eq!(engine.committed_tail.buffer, prefix);
                assert_eq!(engine.committed_tail.epoch, epoch);
                if !reset_handler_first {
                    assert_eq!(engine.context_token.as_ref(), Some(&predecessor),
                        "a zero-effect release must retain its non-authoritative predecessor for the pending Reset");
                    engine
                        .reset(
                            reset.header(),
                            zbus::object_server::SignalEmitter::new(
                                &harness.connection,
                                engine.path.clone(),
                            )
                            .unwrap(),
                        )
                        .await
                        .unwrap();
                }
                assert!(
                    engine.context_reset_rereceipt.is_some(),
                    "prefix_len={prefix_len} reset_handler_first={reset_handler_first}"
                );
                assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                assert!(!engine.context_word_is_known());
                assert!(drain_output_to_proof(&mut harness)
                    .await
                    .iter()
                    .all(|member| !matches!(
                        member.as_str(),
                        "DeleteSurroundingText" | "CommitText"
                    )));
                exact_surrounding_receipt(&mut harness, &mut engine, &prefix).await;
                assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                let (_engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
                assert_eq!(disposition, Ok((3, false)));
            }
        }
    }));
}

#[test]
fn firefox_revoked_release_retirement_cannot_hide_effects_gaps_or_foreign_content() {
    zbus::block_on(bounded(async {
        for gap in [
            "effectful",
            "same_revocation_missing_key",
            "foreign_owner",
            "sensitive",
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = initial_observed_tail_reset(
                &mut harness,
                13_400,
                &[('a', 30), ('b', 48), ('c', 46)],
            )
            .await;
            exact_surrounding_receipt(&mut harness, &mut engine, "abc").await;
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            let old_owner = harness.adapter.current_owner().unwrap();
            let key = receive(&mut harness, 13_410, "ProcessKeyEvent").await;
            let content = if gap == "sensitive" {
                let set = Message::method_call(TARGET_PATH, "Set")
                    .unwrap()
                    .interface(PROPERTIES_INTERFACE)
                    .unwrap()
                    .sender(DISPATCH_SENDER)
                    .unwrap()
                    .serial(NonZeroU32::new(13_411).unwrap())
                    .build(&(
                        ENGINE_INTERFACE,
                        "ContentType",
                        zbus::zvariant::Value::from((8u32, 0u32)),
                    ))
                    .unwrap();
                harness.peer.connection.send(&set).await.unwrap();
                assert!(bounded(harness.observer.process_next()).await.unwrap());
                Some(set)
            } else {
                if gap == "same_revocation_missing_key" {
                    // Controlled invalid settlement, without a lifecycle
                    // revocation: it must retain the ordinary refusal path.
                    harness
                        .adapter
                        .shared
                        .reducer
                        .lock()
                        .unwrap()
                        .unsettled
                        .clear();
                } else {
                    let _ = receive(&mut harness, 13_411, "Reset").await;
                }
                None
            };
            let successor = if gap == "foreign_owner" {
                let owner = harness
                    .adapter
                    .shared
                    .reducer
                    .lock()
                    .unwrap()
                    .establish_source(
                        engine_path(TARGET_PATH),
                        context(CONTEXT_PATH),
                        WordCompleteness::UnknownStart,
                        engine.committed_tail.epoch,
                    )
                    .unwrap();
                assert_ne!(owner, old_owner);
                Some(owner)
            } else {
                None
            };
            let handled = run_received_legacy_key(
                &harness,
                &mut engine,
                &key,
                'd' as u32,
                32,
                if gap == "effectful" { 0 } else { RELEASE_MASK },
            )
            .await;
            if gap == "effectful" {
                assert!(handled);
                td121_expect_legacy_commit_text(&mut harness.peer, "d").await;
                assert_eq!(engine.committed_tail.buffer, "abcd");
            }
            if let Some(set) = content {
                engine.set_content_type((8, 0), Some(set.header())).await;
                assert!(engine.content_is_sensitive());
                assert!(engine.committed_tail.buffer.is_empty());
            }
            if let Some(successor) = successor {
                assert_eq!(harness.adapter.current_owner(), Some(successor));
            }
            assert!(engine.context_reset_rereceipt.is_none(), "{gap}");
            assert!(!engine.context_word_is_known(), "{gap}");
            assert!(
                engine.capture_observed_suffix_display_frame().is_none(),
                "{gap}"
            );
            assert!(
                drain_output_to_proof(&mut harness)
                    .await
                    .iter()
                    .all(|member| !matches!(
                        member.as_str(),
                        "DeleteSurroundingText" | "CommitText"
                    )),
                "{gap}"
            );
            let _ = bridge_toggle_refused_without_text_effect(&mut harness, engine).await;
        }
    }));
}

#[test]
fn firefox_retired_published_preedit_preserves_only_lineage_until_exact_client_text() {
    zbus::block_on(bounded(async {
        let keys = [
            ('a', 30),
            ('b', 48),
            ('c', 46),
            ('d', 32),
            ('e', 18),
            ('f', 33),
        ];
        for prefix_len in [2, 4] {
            for append_count in [1, 2] {
                let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
                let mut engine =
                    initial_observed_tail_reset(&mut harness, 14_000, &keys[..prefix_len]).await;
                let prefix = engine.committed_tail.buffer.clone();
                let left = if prefix_len == 4 { "left " } else { "" };
                let right = if prefix_len == 4 { " right" } else { "" };
                let before = format!("{left}{prefix}{right}");
                let cursor = (left.chars().count() + prefix_len) as u32;
                surrounding_receipt(&mut harness, &mut engine, &before, cursor, cursor).await;
                assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                engine.config.ime_bracket_candidates = prefix_len == 4;
                let published =
                    publish_fixture_append_completion(&mut harness, &mut engine, "жλyz").await;
                let presentation = format!("{left}{prefix}{published}{right}");
                for (i, &(ch, code)) in keys[prefix_len..prefix_len + append_count]
                    .iter()
                    .enumerate()
                {
                    assert!(
                        legacy_key(
                            &mut harness,
                            &mut engine,
                            14_020 + i as u32,
                            ch as u32,
                            code,
                            0
                        )
                        .await
                    );
                    td121_expect_legacy_commit_text(&mut harness.peer, &ch.to_string()).await;
                }
                let token = engine.live_context_token().unwrap();
                let epoch = engine.committed_tail.epoch;
                let current = engine.committed_tail.buffer.clone();
                let cursor = (left.chars().count() + current.chars().count()) as u32;
                let published_cursor = (left.chars().count() + prefix_len) as u32;
                for receipt_cursor in [cursor, published_cursor, cursor] {
                    surrounding_receipt(
                        &mut harness,
                        &mut engine,
                        &presentation,
                        receipt_cursor,
                        receipt_cursor,
                    )
                    .await;
                    assert!(
                        engine.context_reset_rereceipt.is_some(),
                        "published presentation must not erase observed appends"
                    );
                    assert!(!engine.context_reset_rereceipt.as_ref().unwrap().confirmed);
                    assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                    assert!(engine.capture_observed_suffix_display_frame().is_none());
                    assert!(!engine.context_word_is_known());
                    assert_eq!(engine.live_context_token().as_ref(), Some(&token));
                    assert_eq!(engine.committed_tail.epoch, epoch);
                    assert_eq!(engine.committed_tail.buffer, current);
                }
                let settled = format!("{left}{current}{right}");
                surrounding_receipt(&mut harness, &mut engine, &settled, cursor, cursor).await;
                assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                assert!(engine.capture_observed_suffix_display_frame().is_some());
                assert!(engine.committed_tail.pending_completion_learning.is_none());
                assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
                let path = engine.path.clone();
                let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
                assert_eq!(disposition, Ok((3, false)));
                let (_engine, tail) = cycle09_visible_tail(&mut harness, engine).await;
                assert!(cycle09_tail_is_authoritative(&tail, &path, false, &current));
            }
        }
    }));
}

async fn publish_fixture_append_completion(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    suffix: &str,
) -> String {
    set_fixture_append_completion(engine, suffix);
    engine.composition.preedit_visible = false;
    let expected = if engine.config.ime_bracket_candidates {
        format!("[{suffix}]")
    } else {
        suffix.to_string()
    };
    let emitter =
        zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone()).unwrap();
    engine
        .publish_selected_precognition_candidate(&mut crate::output::EngineOutput::legacy(&emitter))
        .await
        .unwrap();
    let message = bounded(next_peer_message(&mut harness.peer)).await;
    assert_eq!(
        message.header().member().unwrap().as_str(),
        "UpdatePreeditText"
    );
    let body = message.body();
    let (text, cursor, visible, mode) = body
        .deserialize::<(zbus::zvariant::Value<'_>, u32, bool, u32)>()
        .unwrap();
    assert_eq!(
        crate::ibus_interface::ibus_text_value_to_string(&text).as_deref(),
        Some(expected.as_str())
    );
    assert_eq!((cursor, visible, mode), (0, true, 0));
    assert_eq!(drain_output_to_proof(harness).await, ["ShowPreeditText"]);
    expected
}

#[test]
fn firefox_published_preedit_retention_refuses_unproved_or_contradictory_snapshots() {
    zbus::block_on(bounded(async {
        for gap in [
            "unpublished",
            "over_bound",
            "wrong_surface",
            "wrong_prefix",
            "selection",
            "wrong_cursor",
            "left_continuation",
            "stale_owner",
            "sensitive",
            "tab_while_stale",
            "reset_after_publication",
            "replaced_publication",
            "unchanged_receipt",
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = initial_observed_tail_reset(
                &mut harness,
                14_100,
                &[('a', 30), ('b', 48), ('c', 46)],
            )
            .await;
            exact_surrounding_receipt(&mut harness, &mut engine, "abc").await;
            engine.config.ime_bracket_candidates = false;
            let suffix = if gap == "over_bound" {
                "x".repeat(crate::preedit::PREEDIT_TAIL_LIMIT)
            } else {
                "xyz".to_string()
            };
            if gap == "unpublished" {
                set_fixture_append_completion(&mut engine, &suffix);
            } else {
                publish_fixture_append_completion(&mut harness, &mut engine, &suffix).await;
            }
            if gap == "replaced_publication" {
                publish_fixture_append_completion(&mut harness, &mut engine, "uvw").await;
            }
            if gap != "unchanged_receipt" {
                assert!(legacy_key(&mut harness, &mut engine, 14_120, 'd' as u32, 32, 0).await);
                td121_expect_legacy_commit_text(&mut harness.peer, "d").await;
            }
            let mut presentation = format!("abc{suffix}");
            let mut cursor = 4;
            let mut anchor = cursor;
            match gap {
                "wrong_surface" => presentation = "abcwyz".into(),
                "wrong_prefix" => presentation = "abxxyz".into(),
                "selection" => anchor = 3,
                "wrong_cursor" => {
                    cursor = 2;
                    anchor = cursor;
                }
                "left_continuation" => {
                    presentation.insert(0, 'z');
                    cursor += 1;
                    anchor = cursor;
                }
                "stale_owner" => engine.context_owner.as_mut().unwrap().generation.0 += 1,
                "sensitive" => engine.set_content_type_state(8, 0),
                "reset_after_publication" => {
                    actual_reset(&mut harness, &mut engine, 14_121, false).await
                }
                "unchanged_receipt" => {
                    presentation = "abc".into();
                    cursor = 3;
                    anchor = cursor;
                }
                _ => {}
            }
            surrounding_receipt(&mut harness, &mut engine, &presentation, cursor, anchor).await;
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{gap}"
            );
            assert!(
                engine.capture_observed_suffix_display_frame().is_none(),
                "{gap}"
            );
            if gap == "tab_while_stale" {
                set_fixture_append_completion(&mut engine, "xyz");
                assert!(!legacy_key(&mut harness, &mut engine, 14_121, KEY_TAB, 15, 0).await);
                assert!(drain_output_to_proof(&mut harness)
                    .await
                    .iter()
                    .all(|member| !matches!(
                        member.as_str(),
                        "DeleteSurroundingText" | "CommitText"
                    )));
            }
            assert!(engine.context_reset_rereceipt.is_none(), "{gap}");
            exact_surrounding_receipt(&mut harness, &mut engine, "abcd").await;
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{gap}"
            );
            let _ = bridge_toggle_refused_without_text_effect(&mut harness, engine).await;
        }
    }));
}

#[test]
fn firefox_exact_client_receipt_wins_when_typed_token_equals_retired_presentation() {
    zbus::block_on(bounded(async {
        let keys = [('a', 30), ('b', 48), ('c', 46), ('d', 32), ('e', 18)];
        for prefix_len in [2, 4] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine =
                initial_observed_tail_reset(&mut harness, 14_200, &keys[..prefix_len]).await;
            let prefix = engine.committed_tail.buffer.clone();
            exact_surrounding_receipt(&mut harness, &mut engine, &prefix).await;
            engine.config.ime_bracket_candidates = false;
            let (ch, code) = keys[prefix_len];
            publish_fixture_append_completion(&mut harness, &mut engine, &ch.to_string()).await;
            assert!(legacy_key(&mut harness, &mut engine, 14_220, ch as u32, code, 0).await);
            td121_expect_legacy_commit_text(&mut harness.peer, &ch.to_string()).await;
            let current = engine.committed_tail.buffer.clone();
            exact_surrounding_receipt(&mut harness, &mut engine, &current).await;
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            let path = engine.path.clone();
            let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
            assert_eq!(disposition, Ok((3, false)));
            let (_engine, tail) = cycle09_visible_tail(&mut harness, engine).await;
            assert!(cycle09_tail_is_authoritative(&tail, &path, false, &current));
        }
    }));
}

#[test]
fn firefox_confirmed_reset_allows_suffix_frame_tab_effect_and_exact_manual_tail() {
    zbus::block_on(bounded(async {
        let keys = [('a', 30), ('b', 48), ('c', 46), ('d', 32), ('e', 18)];
        for prefix_len in [3, 5] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine =
                initial_observed_tail_reset(&mut harness, 9_800, &keys[..prefix_len]).await;
            let prefix = keys[..prefix_len]
                .iter()
                .map(|key| key.0)
                .collect::<String>();
            exact_surrounding_receipt(&mut harness, &mut engine, &prefix).await;
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            let reset_token = engine.live_context_token().unwrap();
            assert!(
                engine.capture_observed_suffix_display_frame().is_some(),
                "a confirmed full Reset receipt must reach suffix readout: {prefix}"
            );
            assert_eq!(engine.live_context_token().as_ref(), Some(&reset_token));
            assert_eq!(
                engine
                    .context_word_scope
                    .as_ref()
                    .unwrap()
                    .lineage()
                    .observed_suffix_chars,
                0,
                "readout must not settle or promote the Reset lineage"
            );
            assert!(!engine.context_word_is_known());
            set_fixture_append_completion(&mut engine, "xyz");
            assert!(legacy_key(&mut harness, &mut engine, 9_820, KEY_TAB, 15, 0).await);
            td121_expect_legacy_commit_text(&mut harness.peer, "xyz ").await;
            let accepted = format!("{prefix}xyz ");
            assert_eq!(engine.committed_tail.buffer, accepted);
            assert!(
                engine.context_word_is_known(),
                "only Tab's actual space establishes a boundary"
            );
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert!(engine.capture_observed_suffix_display_frame().is_none());
            assert!(engine.committed_tail.pending_completion_learning.is_none());
            assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
            exact_surrounding_receipt(&mut harness, &mut engine, &accepted).await;
            let path = engine.path.clone();
            let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
            assert_eq!(disposition, Ok((3, false)));
            let (_engine, tail) = cycle09_visible_tail(&mut harness, engine).await;
            assert!(cycle09_tail_is_authoritative(
                &tail, &path, false, &accepted
            ));
        }
    }));
}

#[test]
fn firefox_confirmed_reset_alt_preparation_preserves_receipt_until_actual_append() {
    zbus::block_on(bounded(async {
        let keys = [('a', 30), ('b', 48), ('c', 46), ('d', 32), ('e', 18)];
        for prefix_len in [3, 5] {
            for (alt, code) in [
                (KEY_LEFT_ALT, 64),
                (KEY_RIGHT_ALT, 108),
                (KEY_ISO_LEVEL3_SHIFT, 108),
            ] {
                let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
                let mut engine =
                    initial_observed_tail_reset(&mut harness, 14_400, &keys[..prefix_len]).await;
                let prefix: String = keys[..prefix_len].iter().map(|key| key.0).collect();
                exact_surrounding_receipt(&mut harness, &mut engine, &prefix).await;
                publish_fixture_append_completion(&mut harness, &mut engine, "жλyz").await;
                let token = engine.live_context_token().unwrap();
                assert!(!legacy_key(&mut harness, &mut engine, 14_420, alt, code, 0).await);
                assert!(drain_output_to_proof(&mut harness).await.is_empty());
                assert_eq!(engine.committed_tail.buffer, prefix);
                assert_eq!(engine.live_context_token().as_ref(), Some(&token));
                assert!(
                    engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                    "zero-effect Alt preparation must preserve the confirmed Reset receipt"
                );
                assert!(!engine.context_word_is_known());
                assert!(engine.layout_gesture.alt_completion_active);
                assert!(!engine.layout_gesture.alt_used_as_modifier);
                assert!(engine.context_observed_suffix_is_current());
                assert_eq!(engine.selected_visible_completion_suffix(), "жλyz");
                let accepted_release =
                    legacy_key(&mut harness, &mut engine, 14_421, alt, code, RELEASE_MASK).await;
                assert!(accepted_release,
                    "prefix={prefix}, alt={alt}, before={token:?}, after={:?}, reset={:?}, visible={}, suffix={:?}, gesture=({}, {}), snapshot={:?}",
                    engine.live_context_token(), engine.context_reset_rereceipt,
                    engine.composition.preedit_visible, engine.composition.preedit_suffix,
                    engine.layout_gesture.alt_completion_active, engine.layout_gesture.alt_used_as_modifier,
                    engine.client_context.surrounding_text_snapshot);
                td121_expect_legacy_commit_text(&mut harness.peer, "жλyz ").await;
                let accepted = format!("{prefix}жλyz ");
                assert_eq!(engine.committed_tail.buffer, accepted);
                assert!(engine.context_word_is_known());
                assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                assert!(engine.capture_observed_suffix_display_frame().is_none());
                assert!(engine.committed_tail.pending_completion_learning.is_none());
                assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
                exact_surrounding_receipt(&mut harness, &mut engine, &accepted).await;
                let path = engine.path.clone();
                let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
                assert_eq!(disposition, Ok((3, false)));
                let (_engine, tail) = cycle09_visible_tail(&mut harness, engine).await;
                assert!(cycle09_tail_is_authoritative(
                    &tail, &path, false, &accepted
                ));
            }
        }
    }));
}

#[test]
fn firefox_reset_alt_preparation_cannot_accept_across_intervening_gaps() {
    zbus::block_on(bounded(async {
        for (alt, code) in [
            (KEY_LEFT_ALT, 64),
            (KEY_RIGHT_ALT, 108),
            (KEY_ISO_LEVEL3_SHIFT, 108),
        ] {
            for gap in [
                "command",
                "navigation",
                "focus_out",
                "sensitive",
                "stale_owner",
                "wrong_text",
                "reset",
            ] {
                let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
                let mut engine = initial_observed_tail_reset(
                    &mut harness,
                    14_500,
                    &[('a', 30), ('b', 48), ('c', 46)],
                )
                .await;
                exact_surrounding_receipt(&mut harness, &mut engine, "abc").await;
                publish_fixture_append_completion(&mut harness, &mut engine, "xyz").await;
                assert!(!legacy_key(&mut harness, &mut engine, 14_510, alt, code, 0).await);
                match gap {
                    "command" => {
                        let _ =
                            legacy_key(&mut harness, &mut engine, 14_511, 'a' as u32, 30, 1 << 2)
                                .await;
                    }
                    "navigation" => {
                        let _ =
                            legacy_key(&mut harness, &mut engine, 14_511, KEY_LEFT, 105, 0).await;
                    }
                    "focus_out" => actual_focus_out(&mut harness, &mut engine, 14_511).await,
                    "sensitive" => engine.set_content_type_state(8, 0),
                    "stale_owner" => engine.context_owner.as_mut().unwrap().generation.0 += 1,
                    "wrong_text" => {
                        exact_surrounding_receipt(&mut harness, &mut engine, "abd").await
                    }
                    "reset" => actual_reset(&mut harness, &mut engine, 14_511, false).await,
                    _ => unreachable!(),
                }
                let tail_before = engine.committed_tail.buffer.clone();
                assert!(
                    !legacy_key(&mut harness, &mut engine, 14_512, alt, code, RELEASE_MASK).await,
                    "{gap}"
                );
                assert_eq!(engine.committed_tail.buffer, tail_before, "{gap}");
                assert!(!engine.context_word_is_known(), "{gap}");
                assert!(
                    drain_output_to_proof(&mut harness)
                        .await
                        .iter()
                        .all(|member| !matches!(
                            member.as_str(),
                            "CommitText" | "DeleteSurroundingText"
                        )),
                    "{gap}"
                );
                assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
            }
        }
    }));
}

#[test]
fn firefox_reset_suffix_frame_and_tab_refuse_unconfirmed_or_invalid_receipts() {
    zbus::block_on(bounded(async {
        for gap in [
            "unconfirmed",
            "wrong_text",
            "selection",
            "duplicate",
            "navigation",
            "focus_out",
            "caps9",
            "sensitive",
            "stale_owner",
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = initial_observed_tail_reset(
                &mut harness,
                9_900,
                &[('a', 30), ('b', 48), ('c', 46)],
            )
            .await;
            match gap {
                "unconfirmed" => exact_surrounding_receipt(&mut harness, &mut engine, "ab").await,
                "wrong_text" => exact_surrounding_receipt(&mut harness, &mut engine, "abd").await,
                "selection" => surrounding_receipt(&mut harness, &mut engine, "abc", 3, 1).await,
                _ => exact_surrounding_receipt(&mut harness, &mut engine, "abc").await,
            }
            match gap {
                "duplicate" => exact_surrounding_receipt(&mut harness, &mut engine, "abc").await,
                "navigation" => {
                    let _ = legacy_key(&mut harness, &mut engine, 9_910, KEY_LEFT, 105, 0).await;
                }
                "focus_out" => actual_focus_out(&mut harness, &mut engine, 9_910).await,
                "caps9" => engine.set_client_capabilities(1 | 1 << 3),
                "sensitive" => engine.set_content_type_state(8, 0),
                "stale_owner" => engine.context_owner.as_mut().unwrap().generation.0 += 1,
                _ => {}
            }
            assert!(
                engine.capture_observed_suffix_display_frame().is_none(),
                "{gap}"
            );
            set_fixture_append_completion(&mut engine, "xyz");
            let tail_before = engine.committed_tail.buffer.clone();
            assert!(
                !legacy_key(&mut harness, &mut engine, 9_911, KEY_TAB, 15, 0).await,
                "{gap}"
            );
            assert_eq!(engine.committed_tail.buffer, tail_before, "{gap}");
            assert!(!engine.context_word_is_known(), "{gap}");
            assert!(
                drain_output_to_proof(&mut harness)
                    .await
                    .iter()
                    .all(|member| !matches!(
                        member.as_str(),
                        "DeleteSurroundingText" | "CommitText"
                    )),
                "{gap}"
            );
        }
    }));
}

#[test]
fn firefox_fresh_surrounding_after_confirmed_append_rearms_next_reset() {
    zbus::block_on(bounded(async {
        for (base, confirmed_prefix, appended, appended_code) in
            [(9_100, "abcd", 'e', 18), (9_300, "a", 'b', 48)]
        {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = new_engine(&harness);
            start_source_free_unknown(&mut harness, &mut engine).await;
            engine.config.auto_replace = false;
            engine.config.typing_assist = false;
            engine.config.nanda_precognition = false;
            engine.client_context.content_purpose = 0;
            engine.client_context.cursor_cell_width = 0;
            engine.client_context.surrounding_text_supported = true;

            for (offset, (ch, code)) in confirmed_prefix.chars().zip([30, 48, 46, 32]).enumerate() {
                let serial = base + offset as u32 * 10;
                assert!(legacy_key(&mut harness, &mut engine, serial, ch as u32, code, 0).await);
                expect_legacy_commit(&mut harness.peer).await;

                let reset = receive(&mut harness, serial + 1, "Reset").await;
                engine
                    .reset(
                        reset.header(),
                        zbus::object_server::SignalEmitter::new(
                            &harness.connection,
                            engine.path.clone(),
                        )
                        .unwrap(),
                    )
                    .await
                    .unwrap();
                assert!(engine.context_reset_rereceipt.is_some());
                exact_surrounding_receipt(&mut harness, &mut engine, &confirmed_prefix[..=offset])
                    .await;
                assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                let _ = legacy_key(
                    &mut harness,
                    &mut engine,
                    serial + 2,
                    ch as u32,
                    code,
                    RELEASE_MASK,
                )
                .await;
            }

            let append_serial = base + 50;
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    append_serial,
                    appended as u32,
                    appended_code,
                    0,
                )
                .await
            );
            expect_legacy_commit(&mut harness.peer).await;
            assert_eq!(
                engine.committed_tail.buffer,
                format!("{confirmed_prefix}{appended}")
            );
            assert!(engine.context_reset_rereceipt.is_some());
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());

            let pending = engine.context_reset_rereceipt.as_ref().unwrap();
            assert!(pending.confirmed, "the verified append retains its witness");
            assert_eq!(pending.token_text, format!("{confirmed_prefix}{appended}"));
            assert_eq!(
                pending.armed_revision,
                engine.client_context.surrounding_observation_revision
            );

            let advanced = format!("{confirmed_prefix}{appended}");
            exact_surrounding_receipt(&mut harness, &mut engine, &advanced).await;
            assert!(
                engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "fresh exact SurroundingText for the appended token must not be rejected as a duplicate: {advanced}"
            );

            let reset = receive(&mut harness, append_serial + 1, "Reset").await;
            engine
                .reset(
                    reset.header(),
                    zbus::object_server::SignalEmitter::new(
                        &harness.connection,
                        engine.path.clone(),
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            assert!(
                engine.context_reset_rereceipt.is_some(),
                "the next Reset must arm from the fresh advanced receipt"
            );
            exact_surrounding_receipt(&mut harness, &mut engine, &advanced).await;
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            exact_surrounding_receipt(&mut harness, &mut engine, &advanced).await;
            assert!(
                engine.context_reset_rereceipt.is_none(),
                "an unchanged-token second receipt must still revoke"
            );
        }
    }));
}

pub(crate) async fn assert_window_interaction_reset_rereceipt_contract() {
    let mut positive_harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut positive = observer_first_reset_unknown_tail(&mut positive_harness, 8_800).await;
    exact_surrounding_receipt(&mut positive_harness, &mut positive, "abcde").await;
    assert!(positive.context_reset_rereceipt_exact_manual_handoff_allowed());
    let token = positive.live_context_token().expect("post-Reset token");
    {
        let mut bridge = positive.begin_context_bridge_output(Some(&token));
        assert!(bridge.consume_context_reset_rereceipt_for_exact_manual_handoff());
        bridge.complete();
    }
    assert!(positive.context_bridge_token.is_none());
    assert!(
        !positive.context_word_is_known(),
        "Reset re-receipt settles the observed suffix without inventing a known start"
    );
    let settled_token = positive
        .live_context_token()
        .expect("consumption keeps one reducer-owned current token");
    assert_ne!(settled_token, token);
    assert!(positive_harness.adapter.revalidate(&settled_token));
    assert!(positive.context_reset_rereceipt.is_none());
    assert!(
        !positive.consume_context_reset_rereceipt_for_exact_manual_handoff(),
        "the one-shot Reset receipt cannot be consumed twice"
    );

    let mut mismatch_harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut mismatch = observer_first_reset_unknown_tail(&mut mismatch_harness, 8_900).await;
    surrounding_receipt(&mut mismatch_harness, &mut mismatch, "abcdf", 5, 5).await;
    assert!(!mismatch.context_reset_rereceipt_exact_manual_handoff_allowed());
    assert!(!mismatch.consume_context_reset_rereceipt_for_exact_manual_handoff());
    assert_eq!(mismatch.committed_tail.buffer, "abcde");

    let mut second_harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut second = observer_first_reset_unknown_tail(&mut second_harness, 9_000).await;
    exact_surrounding_receipt(&mut second_harness, &mut second, "abcde").await;
    assert!(second.context_reset_rereceipt_exact_manual_handoff_allowed());
    exact_surrounding_receipt(&mut second_harness, &mut second, "abcde").await;
    assert!(!second.context_reset_rereceipt_exact_manual_handoff_allowed());
    assert!(!second.consume_context_reset_rereceipt_for_exact_manual_handoff());
    assert_eq!(second.committed_tail.buffer, "abcde");
}

fn observed_callback_without_reply(serial: u32, path: &str, member: &str) -> Message {
    Message::method_call(path, member)
        .unwrap()
        .interface(ENGINE_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .with_flags(zbus::message::Flags::NoReplyExpected)
        .unwrap()
        .build(&())
        .unwrap()
}

pub(super) async fn consume_detached_callback_reply(peer: &mut ControlledPeer, serial: u32) {
    // Registering the real bridge engine starts zbus's object dispatcher.
    // Later lifecycle callbacks are driven directly while that engine is
    // detached; zbus replies UnknownObject even with NoReplyExpected. Consume
    // only that exact transport reply, never a client effect or arbitrary log.
    let reply = bounded(next_peer_message(peer)).await;
    assert_eq!(reply.header().message_type(), Type::Error);
    assert_eq!(reply.header().reply_serial().map(|n| n.get()), Some(serial));
    assert_eq!(
        reply.header().error_name().map(|n| n.as_str()),
        Some("org.freedesktop.DBus.Error.UnknownObject")
    );
}

async fn repeated_terminal_metadata(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    serial: u32,
) {
    let set = Message::method_call(engine.path.as_str(), "Set")
        .unwrap()
        .interface(PROPERTIES_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .with_flags(zbus::message::Flags::NoReplyExpected)
        .unwrap()
        .build(&(
            ENGINE_INTERFACE,
            "ContentType",
            zbus::zvariant::Value::from((10u32, 0u32)),
        ))
        .unwrap();
    harness.peer.connection.send(&set).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    consume_detached_callback_reply(&mut harness.peer, serial).await;
    engine.set_content_type((10, 0), Some(set.header())).await;
    assert_eq!(engine.client_context.content_purpose, 10);
}

#[test]
fn residual_manual_toggle_bridge_repeats_across_legacy_factory_handoffs() {
    zbus::block_on(manual_toggle_bridge_round_trips(true));
}

#[test]
fn residual_manual_toggle_first_word_without_leading_boundary_round_trips() {
    zbus::block_on(manual_toggle_bridge_round_trips(false));
}

async fn manual_toggle_bridge_round_trips(leading_boundary: bool) {
    for boundary in [false, true] {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut source = if leading_boundary {
            known_engine(&mut harness).await
        } else {
            let mut engine = new_engine(&harness);
            start_source_free_unknown(&mut harness, &mut engine).await;
            engine
        };
        source.config.auto_replace = false;
        source.client_context.content_purpose = 10;
        source.client_context.cursor_cell_width = 11;
        source.client_context.surrounding_text_supported = false;
        assert!(!legacy_key(&mut harness, &mut source, 8_000, 'a' as u32, 30, 0).await);
        super::terminal_delivery::no_legacy_text_output(&mut harness).await;
        assert!(source.committed_tail.buffer.ends_with('a'));
        if boundary {
            assert!(!legacy_key(&mut harness, &mut source, 8_001, KEY_SPACE, 57, 0).await);
            super::terminal_delivery::no_legacy_text_output(&mut harness).await;
        }
        for turn in 0..12 {
            let target_is_ru = turn % 2 == 0;
            let text = if target_is_ru { "ф" } else { "a" };
            let suffix = format!("{text}{}", if boundary { " " } else { "" });
            let payload = format!("{}{suffix}", "\u{7f}".repeat(1 + usize::from(boundary)));
            source = bridge_toggle_terminal(&mut harness, source, &payload, target_is_ru).await;
            assert!(source.committed_tail.buffer.ends_with(&suffix));
            let expected_tail = source.committed_tail.buffer.clone();
            let serial = 8_010 + turn * 10;
            let target_path = format!("{TARGET_PATH}_{boundary}_{turn}");
            let factory = Message::method_call("/org/freedesktop/IBus/Factory", "CreateEngine")
                .unwrap()
                .interface(FACTORY_INTERFACE)
                .unwrap()
                .sender(DISPATCH_SENDER)
                .unwrap()
                .serial(NonZeroU32::new(serial).unwrap())
                .with_flags(zbus::message::Flags::NoReplyExpected)
                .unwrap()
                .build(&"lay-us")
                .unwrap();
            harness.peer.connection.send(&factory).await.unwrap();
            assert!(bounded(harness.observer.process_next()).await.unwrap());
            consume_detached_callback_reply(&mut harness.peer, serial).await;
            let callback = harness
                .adapter
                .begin_factory_callback(&factory.header(), Instant::now(), profile("lay-us"))
                .await
                .unwrap();
            assert!(harness
                .adapter
                .bind_factory_target(&callback, engine_path(&target_path)));
            for (offset, member) in [(1, "FocusOut"), (2, "Disable")] {
                let event = observed_callback_without_reply(serial + offset, &source.path, member);
                harness.peer.connection.send(&event).await.unwrap();
                assert!(bounded(harness.observer.process_next()).await.unwrap());
                consume_detached_callback_reply(&mut harness.peer, serial + offset).await;
                if member == "FocusOut" {
                    source.focus_out(event.header()).await;
                    assert!(
                        source.context_handoff_sealed,
                        "turn {turn}: bridge output must seal without another key"
                    );
                } else {
                    source.disable(event.header()).await;
                }
            }
            let mut target = LayIbusEngine::new_from_component(
                target_path.clone(),
                source.shared.clone(),
                Some(harness.adapter.clone()),
                if target_is_ru {
                    "lay-ime-ru"
                } else {
                    "lay-ime-us"
                },
                true,
                ime_config(),
            );
            let focus = observed_callback_without_reply(serial + 3, &target_path, "FocusIn");
            harness.peer.connection.send(&focus).await.unwrap();
            assert!(bounded(harness.observer.process_next()).await.unwrap());
            consume_detached_callback_reply(&mut harness.peer, serial + 3).await;
            target.focus_in_callback(focus.header()).await;
            let get = bounded(next_peer_message(&mut harness.peer)).await;
            assert_eq!(get.header().member().map(|m| m.as_str()), Some("Get"));
            if turn % 2 == 0 {
                repeated_terminal_metadata(&mut harness, &mut target, serial + 4).await;
            }
            let value = OwnedValue::from(ObjectPath::try_from(CONTEXT_PATH).unwrap());
            harness
                .peer
                .connection
                .reply(&get.header(), &value)
                .await
                .unwrap();
            forward_marker_bounded(&mut harness.peer).await;
            assert!(bounded(harness.observer.process_next()).await.unwrap());
            if turn % 2 != 0 {
                repeated_terminal_metadata(&mut harness, &mut target, serial + 4).await;
            }
            let outcome = harness
                .adapter
                .try_finish_activation_for(&engine_path(&target_path))
                .unwrap()
                .expect("same-context transfer ready");
            assert!(matches!(&outcome, ActivationOutcome::Transfer(_)));
            assert!(target.install_context_activation(outcome));
            assert_eq!(target.context_word_is_known(), leading_boundary || boundary);
            assert_eq!(target.committed_tail.buffer, expected_tail);
            assert_eq!(
                target.capture_input_frame_identity().is_some(),
                leading_boundary || boundary,
                "manual projection must not promote unknown word completeness, turn {turn}"
            );
            target.config.auto_replace = false;
            target.client_context.cursor_cell_width = 11;
            target.client_context.surrounding_text_supported = false;
            source = target;
        }
    }
}

#[test]
fn residual_first_word_suffix_tracks_unicode_backspace_and_rejects_retained_prefix_after_gap() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.config.auto_replace = false;
        engine.client_context.content_purpose = 10;
        engine.client_context.cursor_cell_width = 11;
        engine.layout_gesture.layout_is_ru = true;
        for (index, code) in [30, 48, 46].into_iter().enumerate() {
            assert!(!legacy_key(&mut harness, &mut engine, 9_000 + index as u32, 0, code, 0).await);
            super::terminal_delivery::no_legacy_text_output(&mut harness).await;
            assert_eq!(
                engine
                    .context_word_scope
                    .as_ref()
                    .unwrap()
                    .lineage()
                    .observed_suffix_chars,
                index as u32 + 1
            );
            assert!(engine.context_allows_manual_toggle());
            assert!(!engine.context_word_is_known());
            assert!(engine.capture_input_frame_identity().is_none());
        }
        assert_eq!(engine.committed_tail.buffer, "фис");
        assert!(!legacy_key(&mut harness, &mut engine, 9_004, KEY_BACKSPACE, 14, 0).await);
        assert_eq!(engine.committed_tail.buffer, "фи");
        assert_eq!(
            engine
                .context_word_scope
                .as_ref()
                .unwrap()
                .lineage()
                .observed_suffix_chars,
            2
        );
        assert!(engine.context_allows_manual_toggle());

        // A command may edit the client while retaining this local mirror.
        // Subsequent typing proves only the new suffix, never the old prefix.
        assert!(!legacy_key(&mut harness, &mut engine, 9_005, 'a' as u32, 30, 1 << 2).await);
        assert_eq!(engine.committed_tail.buffer, "фи");
        assert_eq!(
            engine
                .context_word_scope
                .as_ref()
                .unwrap()
                .lineage()
                .observed_suffix_chars,
            0
        );
        assert!(!engine.context_allows_manual_toggle());
        assert!(!legacy_key(&mut harness, &mut engine, 9_006, 0, 46, 0).await);
        super::terminal_delivery::no_legacy_text_output(&mut harness).await;
        assert_eq!(engine.committed_tail.buffer, "фис");
        assert_eq!(
            engine
                .context_word_scope
                .as_ref()
                .unwrap()
                .lineage()
                .observed_suffix_chars,
            1
        );
        assert!(!engine.context_allows_manual_toggle());
        let mut effects = AtomicEffectBuilder::default();
        assert_eq!(
            engine
                .manual_toggle_active_text_target(&mut EngineOutput::atomic(&mut effects))
                .await
                .unwrap(),
            None
        );
        assert_eq!(
            effects.finish(false),
            (PROPOSAL_NATIVE_UNHANDLED, Vec::new())
        );
        assert_eq!(engine.committed_tail.buffer, "фис");
    });
}

#[test]
fn residual_first_word_manual_admission_is_terminal_only_and_revoked_with_context() {
    zbus::block_on(async {
        for gap in ["reset", "focus_out", "cursor", "tab", "sensitive"] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = new_engine(&harness);
            start_source_free_unknown(&mut harness, &mut engine).await;
            engine.config.auto_replace = false;
            engine.client_context.content_purpose = 10;
            engine.client_context.cursor_cell_width = 11;
            assert!(!legacy_key(&mut harness, &mut engine, 9_100, 'a' as u32, 30, 0).await);
            super::terminal_delivery::no_legacy_text_output(&mut harness).await;
            assert!(engine.context_allows_manual_toggle());
            engine.client_context.surrounding_text_supported = true;
            assert!(
                !engine.context_allows_manual_toggle(),
                "GTK remains unchanged"
            );
            engine.client_context.surrounding_text_supported = false;
            engine.client_context.cursor_cell_width = 0;
            assert!(
                !engine.context_allows_manual_toggle(),
                "erase geometry is mandatory"
            );
            engine.client_context.cursor_cell_width = 11;
            match gap {
                "reset" => {
                    let message = receive(&mut harness, 9_101, "Reset").await;
                    engine
                        .reset(
                            message.header(),
                            zbus::object_server::SignalEmitter::new(
                                &harness.connection,
                                engine.path.clone(),
                            )
                            .unwrap(),
                        )
                        .await
                        .unwrap();
                }
                "focus_out" => actual_focus_out(&mut harness, &mut engine, 9_101).await,
                "cursor" => {
                    assert!(!legacy_key(&mut harness, &mut engine, 9_101, KEY_LEFT, 105, 0).await);
                }
                "tab" => {
                    assert!(!legacy_key(&mut harness, &mut engine, 9_101, KEY_TAB, 15, 0).await);
                }
                "sensitive" => engine.set_content_type_state(8, 0),
                _ => unreachable!(),
            }
            assert!(!engine.context_allows_manual_toggle(), "{gap}");
            assert!(!engine.context_word_is_known(), "{gap}");
            let before = engine.committed_tail.buffer.clone();
            let mut effects = AtomicEffectBuilder::default();
            assert_eq!(
                engine
                    .manual_toggle_active_text_target(&mut EngineOutput::atomic(&mut effects))
                    .await
                    .unwrap(),
                None,
                "{gap}"
            );
            assert_eq!(
                effects.finish(false),
                (PROPOSAL_NATIVE_UNHANDLED, Vec::new())
            );
            assert_eq!(engine.committed_tail.buffer, before, "{gap}");
            assert!(engine.capture_input_frame_identity().is_none(), "{gap}");
        }
    });
}

#[test]
fn residual_first_word_observation_waits_for_atomic_submission_receipt() {
    zbus::block_on(async {
        for submitted in [false, true] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = new_engine(&harness);
            start_source_free_unknown(&mut harness, &mut engine).await;
            engine.config.auto_replace = false;
            engine.client_context.content_purpose = 10;
            engine.client_context.cursor_cell_width = 11;
            let mut keys = AtomicDriver::new(&mut harness, &mut engine, 9_200);
            assert_literal_commit(&keys.press('a' as u32, 30, 0).await, "a");
            let scope = keys.engine.context_word_scope.as_ref().unwrap();
            assert_eq!(scope.lineage().observed_suffix_chars, 0);
            assert!(keys.engine.committed_tail.buffer.is_empty());
            assert!(!keys.engine.context_allows_manual_toggle());
            if !submitted {
                // Atomic V1 disposition1 is RefusedZeroEffect; keep the exact
                // transaction and digest that identify the pending proposal.
                keys.prior.0 = 1;
            }
            let modifier = keys.press(KEY_LEFT_SHIFT, 42, 0).await;
            assert_eq!(modifier.0, PROPOSAL_NATIVE_UNHANDLED);
            assert!(modifier.1.is_empty());
            let scope = keys.engine.context_word_scope.as_ref().unwrap();
            assert_eq!(scope.lineage().observed_suffix_chars, u32::from(submitted));
            assert_eq!(
                keys.engine.committed_tail.buffer,
                if submitted { "a" } else { "" }
            );
            assert!(!keys.engine.context_word_is_known());
            assert_eq!(keys.engine.context_allows_manual_toggle(), submitted);
            assert!(keys.engine.capture_input_frame_identity().is_none());
        }
    });
}

#[test]
fn residual_atomic_unsubmitted_input_revokes_an_already_observed_prefix() {
    zbus::block_on(async {
        for receipt in [5, 1, 4] {
            for leading_boundary in [false, true] {
                let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
                let mut engine = if leading_boundary {
                    known_engine(&mut harness).await
                } else {
                    let mut engine = new_engine(&harness);
                    start_source_free_unknown(&mut harness, &mut engine).await;
                    engine
                };
                engine.config.auto_replace = false;
                engine.client_context.content_purpose = 10;
                engine.client_context.cursor_cell_width = 11;
                let mut keys = AtomicDriver::new(&mut harness, &mut engine, 9_300);
                assert_literal_commit(&keys.press('a' as u32, 30, 0).await, "a");
                assert_literal_commit(&keys.press('b' as u32, 48, 0).await, "b");
                assert!(keys.engine.committed_tail.buffer.ends_with('a'));
                assert!(keys.engine.context_allows_manual_toggle());
                // 5: uncertain submission; 1: native key delivery is possible;
                // 4: focus lineage terminated. None proves an unchanged tail.
                keys.prior.0 = receipt;
                let modifier = keys.press(KEY_LEFT_SHIFT, 42, 0).await;
                assert_eq!(modifier.0, PROPOSAL_NATIVE_UNHANDLED);
                assert!(modifier.1.is_empty());
                let scope = keys.engine.context_word_scope.as_ref().unwrap();
                assert_eq!(
                    scope.lineage().observed_suffix_chars,
                    0,
                    "receipt {receipt}"
                );
                assert!(!keys.engine.context_word_is_known(), "receipt {receipt}");
                assert!(
                    !keys.engine.context_allows_manual_toggle(),
                    "receipt {receipt}"
                );
                assert!(keys.engine.capture_input_frame_identity().is_none());
            }
        }
    });
}

#[test]
fn residual_first_numeric_word_bridge_refuses_without_delegation_or_output() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.config.auto_replace = false;
        engine.client_context.content_purpose = 10;
        engine.client_context.cursor_cell_width = 11;
        for (index, code) in [2, 3, 4].into_iter().enumerate() {
            assert!(!legacy_key(&mut harness, &mut engine, 9_400 + index as u32, 0, code, 0).await);
            super::terminal_delivery::no_legacy_text_output(&mut harness).await;
        }
        assert_eq!(engine.committed_tail.buffer, "123");
        assert!(engine.context_allows_manual_toggle());
        assert!(!engine.context_word_is_known());
        let bridge = bridge(&harness, &engine);
        harness
            .connection
            .object_server()
            .at(TARGET_PATH, engine)
            .await
            .unwrap();
        let (result, ()) = bounded(future::zip(bridge.manual_toggle_v3_inner(), async {
            serve_ping_and_marker(&mut harness.peer).await;
            assert!(harness.observer.process_next().await.unwrap());
        }))
        .await;
        assert_eq!(
            result.unwrap(),
            (0, false),
            "no daemon route for an unmappable suffix"
        );
        // A proof-only transport marker makes absence of output observable
        // without a sleep or an empty-queue timing assumption.
        harness
            .connection
            .emit_signal(None::<&str>, TARGET_PATH, "org.lay.Proof", "Reached", &())
            .await
            .unwrap();
        let next = bounded(next_peer_message(&mut harness.peer)).await;
        assert_eq!(next.header().member().unwrap().as_str(), "Reached");
        let interface = harness
            .connection
            .object_server()
            .interface::<_, LayIbusEngine>(TARGET_PATH)
            .await
            .unwrap();
        assert_eq!(interface.get().await.committed_tail.buffer, "123");
    });
}

#[test]
fn residual_known_numeric_word_bridge_refuses_without_delegation_or_output() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = known_engine(&mut harness).await;
        engine.config.auto_replace = false;
        engine.client_context.content_purpose = 10;
        engine.client_context.cursor_cell_width = 11;
        for (index, code) in [2, 3, 4].into_iter().enumerate() {
            assert!(!legacy_key(&mut harness, &mut engine, 9_410 + index as u32, 0, code, 0).await);
            super::terminal_delivery::no_legacy_text_output(&mut harness).await;
        }
        assert!(engine.committed_tail.buffer.ends_with("123"));
        assert!(engine.context_allows_manual_toggle());
        assert!(engine.context_word_is_known());
        let bridge = bridge(&harness, &engine);
        harness
            .connection
            .object_server()
            .at(TARGET_PATH, engine)
            .await
            .unwrap();
        let (result, ()) = bounded(future::zip(bridge.manual_toggle_v3_inner(), async {
            serve_ping_and_marker(&mut harness.peer).await;
            assert!(harness.observer.process_next().await.unwrap());
        }))
        .await;
        assert_eq!(
            result.unwrap(),
            (0, false),
            "a refused local terminal plan must not become exact-tail delegation"
        );
        harness
            .connection
            .emit_signal(None::<&str>, TARGET_PATH, "org.lay.Proof", "Reached", &())
            .await
            .unwrap();
        let next = bounded(next_peer_message(&mut harness.peer)).await;
        assert_eq!(next.header().member().unwrap().as_str(), "Reached");
        let interface = harness
            .connection
            .object_server()
            .interface::<_, LayIbusEngine>(TARGET_PATH)
            .await
            .unwrap();
        assert!(interface.get().await.committed_tail.buffer.ends_with("123"));
    });
}

#[test]
fn residual_pending_auto_undo_bridge_keeps_ime_ownership_without_delegation() {
    zbus::block_on(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = known_engine(&mut harness).await;
        engine.committed_tail.buffer = "собака ".to_string();
        engine.remember_pending_ime_auto_undo(
            "cj,frf ".to_string(),
            "собака ".to_string(),
            lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
        );
        engine.client_context.surrounding_text_supported = true;
        engine.client_context.surrounding_text_snapshot = None;
        let target_layout_is_ru = engine.layout_gesture.layout_is_ru;
        let bridge = bridge(&harness, &engine);
        harness
            .connection
            .object_server()
            .at(TARGET_PATH, engine)
            .await
            .unwrap();
        let (result, ()) = bounded(future::zip(bridge.manual_toggle_v3_inner(), async {
            serve_ping_and_marker(&mut harness.peer).await;
            assert!(harness.observer.process_next().await.unwrap());
        }))
        .await;
        assert_eq!(result.unwrap(), (1, target_layout_is_ru));
        let members = drain_output_to_proof(&mut harness).await;
        assert!(members
            .iter()
            .any(|member| member == "RequireSurroundingText"));
        assert!(members
            .iter()
            .all(|member| !matches!(member.as_str(), "DeleteSurroundingText" | "CommitText")));
        let interface = harness
            .connection
            .object_server()
            .interface::<_, LayIbusEngine>(TARGET_PATH)
            .await
            .unwrap();
        let engine = interface.get().await;
        assert_eq!(
            engine.pending_ime_auto_undo_retry_status(),
            "waiting_exact_snapshot"
        );
        assert!(!engine.exact_manual_toggle_handoff_is_live());
        assert!(engine.context_word_is_known());
    });
}

async fn native_refocus_case(next_context: &str) {
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = known_engine(&mut harness).await;
    let previous = engine.live_context_token().unwrap();
    leave_context(&mut harness, &mut engine).await;
    let focus_in = receive(&mut harness, 3_032, "FocusInId").await;
    bounded(engine.focus_in_id(
        focus_in.header(),
        next_context.to_string(),
        "test-client".to_string(),
    ))
    .await;
    // The production callback reloads the host config. Keep this controlled
    // fixture independent of the remote account's default text backend.
    engine.config = ime_config();
    assert!(
        harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .request
            .is_some(),
        "real re-focus must start acquisition, not enrich old owner"
    );
    forward_marker_bounded(&mut harness.peer).await;
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    assert!(!legacy_key(&mut harness, &mut engine, 3_033, KEY_LEFT_SHIFT, 42, 0).await);
    assert!(
        !legacy_key(
            &mut harness,
            &mut engine,
            3_035,
            KEY_LEFT_SHIFT,
            42,
            RELEASE_MASK
        )
        .await
    );
    let current = engine
        .live_context_token()
        .expect("native re-focus installs an owner");
    assert_ne!(current.owner.generation, previous.owner.generation);
    assert_ne!(
        current.activation.generation,
        previous.activation.generation
    );
    assert_eq!(current.activation.context.path.as_str(), next_context);
    assert_eq!(engine.context_word_is_known(), next_context == CONTEXT_PATH);
    assert_eq!(
        engine.committed_tail.buffer,
        if next_context == CONTEXT_PATH {
            " "
        } else {
            ""
        }
    );
    assert!(harness
        .adapter
        .shared
        .reducer
        .lock()
        .unwrap()
        .request
        .is_none());
    assert!(harness
        .adapter
        .shared
        .ready_activation
        .lock()
        .unwrap()
        .is_none());
    assert!(bridge_fence(&mut harness).await.is_ok());
    if next_context != CONTEXT_PATH {
        assert!(legacy_key(&mut harness, &mut engine, 3_034, KEY_SPACE, 57, 0).await);
        expect_legacy_commit(&mut harness.peer).await;
        assert!(
            engine.context_word_is_known(),
            "next actual boundary rearms new context"
        );
    }
}

#[test]
fn residual_native_refocus_same_context_rotates_activation() {
    zbus::block_on(bounded(native_refocus_case(CONTEXT_PATH)));
}

#[test]
fn residual_native_refocus_other_context_rearms_source_free() {
    zbus::block_on(bounded(native_refocus_case(
        "/org/freedesktop/IBus/InputContext_2",
    )));
}

async fn ready_transfer(harness: &mut Harness) -> LayIbusEngine {
    let mut engine = known_engine(harness).await;
    leave_context(harness, &mut engine).await;
    let focus_in = receive(harness, 3_040, "FocusInId").await;
    let observed = harness
        .adapter
        .observe_callback(&focus_in.header(), Instant::now())
        .await
        .unwrap();
    // Use the real acquisition independently of the separate re-focus bug.
    harness
        .adapter
        .start_native_activation(
            engine_path(TARGET_PATH),
            context(CONTEXT_PATH),
            observed.position,
        )
        .unwrap();
    forward_marker_bounded(&mut harness.peer).await;
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    assert!(matches!(
        harness
            .adapter
            .shared
            .ready_activation
            .lock()
            .unwrap()
            .as_ref()
            .map(|r| &r.outcome),
        Some(ActivationOutcome::Transfer(_))
    ));
    engine
}

async fn revoke_ready(harness: &mut Harness, owner: &EngineOwner, content_type: bool) -> Message {
    let message = if content_type {
        Message::method_call(TARGET_PATH, "Set")
            .unwrap()
            .interface(PROPERTIES_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(3_041).unwrap())
            .build(&(
                ENGINE_INTERFACE,
                "ContentType",
                // This is a real hints change; repeating (0,0) is idempotent.
                zbus::zvariant::Value::from((0u32, 1u32)),
            ))
            .unwrap()
    } else {
        method_message(
            DISPATCH_SENDER,
            3_041,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "Reset",
        )
    };
    harness.peer.connection.send(&message).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    assert_eq!(
        harness.adapter.current_owner().as_ref(),
        Some(owner),
        "revocation retains owner"
    );
    assert_eq!(
        harness
            .adapter
            .current_token()
            .unwrap()
            .lineage
            .completeness,
        WordCompleteness::UnknownStart
    );
    message
}

pub(super) async fn global_engine_changed(harness: &mut Harness, serial: u32, profile: &str) {
    let signal = Message::signal(IBUS_PATH, IBUS_INTERFACE, "GlobalEngineChanged")
        .unwrap()
        .sender(IBUS_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .build(&profile)
        .unwrap();
    harness.peer.connection.send(&signal).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
}

#[test]
fn residual_reset_before_ready_install_does_not_leak_ownerless_key_settlements() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;

        let published_token = harness
            .adapter
            .current_token()
            .expect("ready source-free grant has a reducer owner");
        assert!(engine.context_owner.is_none());
        assert!(engine.live_context_token().is_none());
        assert!(harness
            .adapter
            .shared
            .ready_activation
            .lock()
            .unwrap()
            .is_some());
        engine.committed_tail.buffer = "stale-before-ready-install".to_string();

        let mut resets = Vec::new();
        for serial in 3_400..3_403 {
            let reset = method_message(
                DISPATCH_SENDER,
                serial,
                TARGET_PATH,
                ENGINE_INTERFACE,
                "Reset",
            );
            harness.peer.connection.send(&reset).await.unwrap();
            assert!(bounded(harness.observer.process_next()).await.unwrap());
            resets.push(reset);
        }
        for reset in &resets {
            bounded(
                engine.reset(
                    reset.header(),
                    zbus::object_server::SignalEmitter::new(
                        &harness.connection,
                        engine.path.clone(),
                    )
                    .unwrap(),
                ),
            )
            .await
            .unwrap();
        }
        assert!(!harness.adapter.revalidate(&published_token));
        assert!(engine.context_owner.is_none());
        assert!(!engine.context_word_is_known());
        let reset_outcome = harness
            .adapter
            .pending_activation_for(&engine_path(TARGET_PATH))
            .expect("current uninstalled reset witness");
        assert!(matches!(&reset_outcome, ActivationOutcome::ResetUnknown(_)));

        // Every callback runs to completion through the production engine and
        // observer paths. None may leak an ownerless ingress entry into the
        // reducer's bounded unsettled queue or cancel future observation.
        for index in 0..(MAX_UNSETTLED_KEYS + 2) {
            assert!(
                !legacy_key(
                    &mut harness,
                    &mut engine,
                    3_410 + index as u32,
                    KEY_LEFT_SHIFT,
                    42,
                    if index % 2 == 0 { 0 } else { RELEASE_MASK },
                )
                .await
            );
            if index == 0 {
                assert!(engine.committed_tail.buffer.is_empty());
                assert!(engine.live_context_token().is_some());
                assert!(!engine.context_word_is_known());
                assert!(harness
                    .adapter
                    .pending_activation_for(&engine_path(TARGET_PATH))
                    .is_none());
                assert!(harness
                    .adapter
                    .activation_outcome_is_current(&reset_outcome));
                assert!(
                    !harness.adapter.acknowledge_activation(&reset_outcome),
                    "a still-current token cannot acknowledge an absent/already installed witness"
                );
            }
        }
        assert!(harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .unsettled
            .is_empty());

        let recovered = engine
            .live_context_token()
            .expect("the first real key installs the reset witness");
        assert_eq!(recovered.owner, published_token.owner);
        assert_eq!(recovered.activation, published_token.activation);
        assert_ne!(recovered.revocation, published_token.revocation);
        assert_eq!(
            recovered.lineage.completeness,
            WordCompleteness::UnknownStart
        );
        assert!(engine.committed_tail.buffer.is_empty());
        assert!(!engine.context_word_is_known());
        assert!(!harness.adapter.revalidate(&published_token));
        assert!(bridge_fence(&mut harness).await.is_ok());

        assert!(legacy_key(&mut harness, &mut engine, 3_500, KEY_SPACE, 57, 0).await);
        expect_legacy_commit(&mut harness.peer).await;
        assert!(engine.context_word_is_known());
        assert!(bridge_fence(&mut harness).await.is_ok());
    }));
}

#[test]
fn residual_established_owner_reset_burst_rearms_on_first_real_boundary() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = known_engine(&mut harness).await;
        let old_token = engine.live_context_token().unwrap();

        let mut resets = Vec::new();
        for serial in 3_600..3_603 {
            let reset = method_message(
                DISPATCH_SENDER,
                serial,
                TARGET_PATH,
                ENGINE_INTERFACE,
                "Reset",
            );
            harness.peer.connection.send(&reset).await.unwrap();
            assert!(bounded(harness.observer.process_next()).await.unwrap());
            resets.push(reset);
        }
        for reset in &resets {
            bounded(
                engine.reset(
                    reset.header(),
                    zbus::object_server::SignalEmitter::new(
                        &harness.connection,
                        engine.path.clone(),
                    )
                    .unwrap(),
                ),
            )
            .await
            .unwrap();
        }

        assert!(!harness.adapter.revalidate(&old_token));
        assert!(!engine.context_word_is_known());
        assert!(harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .unsettled
            .is_empty());
        assert!(legacy_key(&mut harness, &mut engine, 3_603, KEY_SPACE, 57, 0).await);
        expect_legacy_commit(&mut harness.peer).await;
        assert!(engine.context_word_is_known());
        assert!(bridge_fence(&mut harness).await.is_ok());
    }));
}

#[test]
fn residual_ready_transfer_rejects_same_owner_revocation_before_take() {
    zbus::block_on(bounded(async {
        for content_type in [false, true] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let _source = ready_transfer(&mut harness).await;
            let old_outcome = harness
                .adapter
                .pending_activation_for(&engine_path(TARGET_PATH))
                .expect("ready transfer witness");
            assert!(matches!(&old_outcome, ActivationOutcome::Transfer(_)));
            let owner = harness.adapter.current_owner().unwrap();
            revoke_ready(&mut harness, &owner, content_type).await;
            assert!(!harness.adapter.activation_outcome_is_current(&old_outcome));
            assert!(matches!(
                harness
                    .adapter
                    .pending_activation_for(&engine_path(TARGET_PATH)),
                Some(ActivationOutcome::ResetUnknown(_))
            ));
        }
    }));
}

#[test]
fn residual_taken_transfer_rejects_same_owner_revocation_before_install() {
    zbus::block_on(bounded(async {
        for content_type in [false, true] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut source = ready_transfer(&mut harness).await;
            let outcome = harness
                .adapter
                .pending_activation_for(&engine_path(TARGET_PATH))
                .expect("runtime peeks the ready transfer before installation");
            assert!(matches!(&outcome, ActivationOutcome::Transfer(_)));
            let owner = harness.adapter.current_owner().unwrap();
            assert!(harness.adapter.activation_outcome_is_current(&outcome));
            revoke_ready(&mut harness, &owner, content_type).await;
            assert!(
                !source.install_context_activation(outcome),
                "revoked KnownStart must never install"
            );
            assert!(matches!(
                harness
                    .adapter
                    .pending_activation_for(&engine_path(TARGET_PATH)),
                Some(ActivationOutcome::ResetUnknown(_))
            ));
            assert!(
                !legacy_key(
                    &mut harness,
                    &mut source,
                    3_610 + if content_type { 1 } else { 0 },
                    KEY_LEFT_SHIFT,
                    42,
                    0,
                )
                .await
            );
            assert!(source.live_context_token().is_some());
            assert!(!source.context_word_is_known());
            assert!(source.committed_tail.buffer.is_empty());
            assert!(source.committed_tail.pending_completion_learning.is_none());
            assert!(harness
                .adapter
                .pending_activation_for(&engine_path(TARGET_PATH))
                .is_none());
        }
    }));
}

#[test]
fn residual_reset_unknown_witness_is_not_revived_by_foreign_or_focus_out() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let _source = ready_transfer(&mut harness).await;
        let owner = harness.adapter.current_owner().unwrap();
        global_engine_changed(&mut harness, 3_620, "foreign-ime").await;
        assert!(harness
            .adapter
            .pending_activation_for(&engine_path(TARGET_PATH))
            .is_none());
        revoke_ready(&mut harness, &owner, false).await;
        global_engine_changed(&mut harness, 3_621, "lay-us").await;
        assert!(
            harness
                .adapter
                .pending_activation_for(&engine_path(TARGET_PATH))
                .is_none(),
            "Foreign -> Reset -> Lay cannot recreate an invalidated witness"
        );

        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let _source = ready_transfer(&mut harness).await;
        let owner = harness.adapter.current_owner().unwrap();
        let focus_out = method_message(
            DISPATCH_SENDER,
            3_622,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "FocusOut",
        );
        harness.peer.connection.send(&focus_out).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        assert!(matches!(
            harness
                .adapter
                .pending_activation_for(&engine_path(TARGET_PATH)),
            Some(ActivationOutcome::Transfer(_))
        ));
        revoke_ready(&mut harness, &owner, false).await;
        assert!(
            harness
                .adapter
                .pending_activation_for(&engine_path(TARGET_PATH))
                .is_none(),
            "Reset after FocusOut cannot convert an unsettled transfer into ResetUnknown"
        );
    }));
}

#[test]
fn residual_legacy_enter_backspace_revokes_beyond_the_observed_mirror() {
    zbus::block_on(bounded(async {
        for enter in [KEY_ENTER, KEY_KP_ENTER] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = new_engine(&harness);
            start_source_free_unknown(&mut harness, &mut engine).await;
            assert!(legacy_key(&mut harness, &mut engine, 3_050, u32::from(b'l'), 38, 0).await);
            expect_legacy_commit(&mut harness.peer).await;
            assert!(!engine.context_word_is_known());
            assert!(!legacy_key(&mut harness, &mut engine, 3_051, enter, 28, 0).await);
            assert!(engine.committed_tail.buffer.is_empty());
            assert!(
                engine.context_word_is_known(),
                "Enter is an observed boundary"
            );
            assert!(!legacy_key(&mut harness, &mut engine, 3_052, KEY_BACKSPACE, 14, 0).await);
            assert!(
                !engine.context_word_is_known(),
                "Backspace rejoins the unobserved prefix"
            );
            assert!(!legacy_key(&mut harness, &mut engine, 3_053, KEY_TAB, 15, 0).await);
            let mut effects = AtomicEffectBuilder::default();
            let manual = engine
                .manual_toggle_active_text_target(&mut EngineOutput::atomic(&mut effects))
                .await
                .unwrap();
            assert_eq!(manual, None);
            assert_eq!(
                effects.finish(false),
                (PROPOSAL_NATIVE_UNHANDLED, Vec::new())
            );
            assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
            assert!(legacy_key(&mut harness, &mut engine, 3_054, KEY_SPACE, 57, 0).await);
            expect_legacy_commit(&mut harness.peer).await;
            assert!(
                engine.context_word_is_known(),
                "a new actual boundary restores authority"
            );
        }
    }));
}

async fn retained_boundary_literal_keys(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    serial: &mut u32,
    text: &str,
) {
    for ch in text.chars() {
        let keycode = match ch {
            'a' => 30,
            'b' => 48,
            'c' => 46,
            ' ' => 57,
            _ => unreachable!("fixture key"),
        };
        assert!(legacy_key(harness, engine, *serial, ch as u32, keycode, 0).await);
        *serial += 1;
        let commit = bounded(next_peer_message(&mut harness.peer)).await;
        assert_eq!(commit.header().member().unwrap().as_str(), "CommitText");
        let body = commit.body();
        let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
        assert_eq!(
            crate::ibus_interface::ibus_text_value_to_string(&value),
            Some(ch.to_string())
        );
        assert!(legacy_key(harness, engine, *serial, ch as u32, keycode, RELEASE_MASK).await);
        *serial += 1;
    }
}

async fn retained_boundary_backspace(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    serial: &mut u32,
) {
    assert!(!legacy_key(harness, engine, *serial, KEY_BACKSPACE, 14, 0).await);
    *serial += 1;
    let _ = legacy_key(harness, engine, *serial, KEY_BACKSPACE, 14, RELEASE_MASK).await;
    *serial += 1;
}

async fn td121_unknown_start_completion_on_alt_release(
    harness: &mut Harness,
    serial: &mut u32,
    suffix: &str,
    snapshot: Option<(&str, u32, u32)>,
) -> LayIbusEngine {
    let mut engine = new_engine(harness);
    start_source_free_unknown(harness, &mut engine).await;
    retained_boundary_literal_keys(harness, &mut engine, serial, "abc").await;
    assert_eq!(engine.committed_tail.buffer, "abc");
    assert!(!engine.context_word_is_known());
    if let Some((text, cursor, anchor)) = snapshot {
        cycle09_surrounding_receipt(harness, &mut engine, text, cursor, anchor).await;
    }
    engine.composition.buffer.clear();
    engine.composition.cursor = 0;
    engine.composition.preedit_suffix = suffix.into();
    engine.composition.preedit_candidates = (!suffix.is_empty())
        .then(|| suffix.into())
        .into_iter()
        .collect();
    engine.composition.preedit_replacement_targets =
        (!suffix.is_empty()).then_some(None).into_iter().collect();
    engine.composition.preedit_visible = !suffix.is_empty();
    engine
}

async fn td121_expect_legacy_commit_text(peer: &mut ControlledPeer, expected: &str) {
    loop {
        let message = bounded(next_peer_message(peer)).await;
        let member = message.header().member().unwrap().as_str().to_string();
        assert_ne!(member, "DeleteSurroundingText");
        if member == "CommitText" {
            let body = message.body();
            let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
            assert_eq!(
                crate::ibus_interface::ibus_text_value_to_string(&value).as_deref(),
                Some(expected)
            );
            return;
        }
    }
}

#[test]
fn td121_successful_completion_release_settles_its_append_and_boundary_effect() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut serial = 12_900;
        let mut engine = td121_unknown_start_completion_on_alt_release(
            &mut harness,
            &mut serial,
            "def",
            Some(("abc", 3, 3)),
        )
        .await;

        assert!(!legacy_key(&mut harness, &mut engine, serial, KEY_LEFT_ALT, 64, 0).await);
        cycle09_surrounding_receipt(&mut harness, &mut engine, "abc", 3, 3).await;
        assert!(engine.composition.preedit_visible);
        assert_eq!(engine.composition.preedit_suffix, "def");
        assert!(engine.capture_observed_suffix_display_frame().is_some());
        serial += 1;
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                serial,
                KEY_LEFT_ALT,
                64,
                RELEASE_MASK,
            )
            .await
        );
        td121_expect_legacy_commit_text(&mut harness.peer, "def ").await;
        assert_eq!(engine.committed_tail.buffer, "abcdef ");
        assert!(
            engine.context_word_is_known(),
            "the successful append plus boundary happened on release and must settle that callback"
        );
        cycle09_surrounding_receipt(&mut harness, &mut engine, "abcdef ", 7, 7).await;
        let path = engine.path.clone();
        let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
        assert_eq!(
            disposition,
            Ok((3, false)),
            "ManualToggleV3 exact delegation"
        );
        let (_engine, tail) = cycle09_visible_tail(&mut harness, engine).await;
        assert!(cycle09_tail_is_authoritative(
            &tail, &path, false, "abcdef "
        ));
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    }));
}

#[test]
fn td121_non_effectful_completion_routes_cannot_promote_unknown_start() {
    zbus::block_on(bounded(async {
        for route in ["no-change", "unhandled", "rejected"] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut serial = 13_000;
            let (suffix, snapshot) = match route {
                "no-change" => ("", Some(("abc", 3, 3))),
                "unhandled" => ("def", Some(("abc", 3, 3))),
                "rejected" => ("def", Some(("other", 5, 5))),
                _ => unreachable!(),
            };
            let mut engine = td121_unknown_start_completion_on_alt_release(
                &mut harness,
                &mut serial,
                suffix,
                snapshot,
            )
            .await;
            assert!(!legacy_key(&mut harness, &mut engine, serial, KEY_LEFT_ALT, 64, 0).await);
            if route != "unhandled" {
                if route == "rejected" {
                    cycle09_surrounding_receipt(&mut harness, &mut engine, "other", 5, 5).await;
                } else {
                    cycle09_surrounding_receipt(&mut harness, &mut engine, "abc", 3, 3).await;
                }
                serial += 1;
                assert!(
                    !legacy_key(
                        &mut harness,
                        &mut engine,
                        serial,
                        KEY_LEFT_ALT,
                        64,
                        RELEASE_MASK,
                    )
                    .await
                );
            }
            assert_eq!(engine.committed_tail.buffer, "abc", "{route}");
            assert!(!engine.context_word_is_known(), "{route}");
            assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
        }
    }));
}

#[test]
fn td121_completion_boundary_backspace_cannot_resurrect_the_completed_word() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut serial = 13_100;
        let mut engine = td121_unknown_start_completion_on_alt_release(
            &mut harness,
            &mut serial,
            "def",
            Some(("abc", 3, 3)),
        )
        .await;
        assert!(!legacy_key(&mut harness, &mut engine, serial, KEY_LEFT_ALT, 64, 0).await);
        cycle09_surrounding_receipt(&mut harness, &mut engine, "abc", 3, 3).await;
        serial += 1;
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                serial,
                KEY_LEFT_ALT,
                64,
                RELEASE_MASK,
            )
            .await
        );
        td121_expect_legacy_commit_text(&mut harness.peer, "def ").await;
        assert!(engine.context_word_is_known());

        // The client applies native Backspace after the unhandled press. Its
        // next real surrounding-text receipt must retire the closed word.
        serial += 1;
        assert!(!legacy_key(&mut harness, &mut engine, serial, KEY_BACKSPACE, 14, 0).await);
        cycle09_surrounding_receipt(&mut harness, &mut engine, "abcdef", 6, 6).await;
        assert!(!engine.context_word_is_known());
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    }));
}

#[test]
fn residual_observed_boundary_backspace_keeps_start_and_retires_old_token() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = known_engine(&mut harness).await;
        engine.config.auto_replace = false;
        let mut serial = 4_000;
        retained_boundary_literal_keys(&mut harness, &mut engine, &mut serial, "ab ").await;
        assert_eq!(engine.committed_tail.buffer, " ab ");
        let closed = engine.live_context_token().unwrap();
        retained_boundary_backspace(&mut harness, &mut engine, &mut serial).await;
        assert_eq!(engine.committed_tail.buffer, " ab");
        assert!(engine.context_word_is_known());
        assert!(engine.capture_input_frame_identity().is_some());
        assert!(!harness.adapter.revalidate(&closed));
        for expected in [" a", " "] {
            retained_boundary_backspace(&mut harness, &mut engine, &mut serial).await;
            assert_eq!(engine.committed_tail.buffer, expected);
            assert!(engine.context_word_is_known());
        }
        retained_boundary_literal_keys(&mut harness, &mut engine, &mut serial, "c").await;
        assert_eq!(engine.committed_tail.buffer, " c");
        assert!(engine.capture_input_frame_identity().is_some());
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    }));
}

#[test]
fn residual_observed_boundary_backspace_refuses_an_unobserved_mirror_separator() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        engine.config.auto_replace = false;
        start_source_free_unknown(&mut harness, &mut engine).await;
        assert!(!legacy_key(&mut harness, &mut engine, 4_098, KEY_LEFT_SHIFT, 42, 0).await);
        assert!(
            !legacy_key(
                &mut harness,
                &mut engine,
                4_099,
                KEY_LEFT_SHIFT,
                42,
                RELEASE_MASK,
            )
            .await
        );
        assert!(engine.live_context_token().is_some());
        assert!(!engine.context_word_is_known());
        engine.committed_tail.buffer = "hidden prefix".into();
        assert!(!engine.context_word_is_known());
        let mut serial = 4_100;
        retained_boundary_literal_keys(&mut harness, &mut engine, &mut serial, " ").await;
        assert_eq!(engine.committed_tail.buffer, "hidden prefix ");
        assert_eq!(
            engine
                .context_word_scope
                .unwrap()
                .lineage()
                .observed_boundary_floor,
            Some(13)
        );
        assert!(engine.context_word_is_known());
        retained_boundary_backspace(&mut harness, &mut engine, &mut serial).await;
        assert_eq!(engine.committed_tail.buffer, "hidden prefix");
        assert!(!engine.context_word_is_known());
        assert!(engine.capture_input_frame_identity().is_none());
        assert_eq!(
            engine
                .context_word_scope
                .unwrap()
                .lineage()
                .observed_boundary_floor,
            None
        );
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    }));
}

#[test]
fn residual_observed_boundary_backspace_cannot_cross_a_command_input_gap() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = known_engine(&mut harness).await;
        engine.config.auto_replace = false;
        let mut serial = 4_200;
        retained_boundary_literal_keys(&mut harness, &mut engine, &mut serial, "ab ").await;
        assert!(!legacy_key(&mut harness, &mut engine, serial, b'a' as u32, 30, 1 << 2).await);
        serial += 1;
        assert!(!engine.context_word_is_known());
        assert_eq!(
            engine
                .context_word_scope
                .unwrap()
                .lineage()
                .observed_boundary_floor,
            None
        );
        // A retained mirror after an external command is not fresh evidence.
        engine.committed_tail.buffer = " ab ".into();
        retained_boundary_backspace(&mut harness, &mut engine, &mut serial).await;
        assert_eq!(engine.committed_tail.buffer, " ab");
        assert!(!engine.context_word_is_known());
        assert!(engine.capture_input_frame_identity().is_none());
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    }));
}

#[derive(Clone, Copy, Debug)]
enum Cycle09ExternalTailCase {
    ExactMatch,
    TrailingBoundaryExactMatch,
    SourceMismatch,
    TargetMismatch,
    LateTargetMismatch,
    BoundaryMismatch,
    Selection,
    NoSnapshot,
    StaleContext,
}

async fn cycle09_assert_no_local_text_effect(harness: &mut Harness) {
    harness
        .connection
        .emit_signal(
            None::<&str>,
            TARGET_PATH,
            "org.lay.Proof",
            "Cycle09Reached",
            &(),
        )
        .await
        .unwrap();
    loop {
        let message = bounded(next_peer_message(&mut harness.peer)).await;
        let member = message
            .header()
            .member()
            .map(|member| member.as_str().to_string());
        assert!(
            !matches!(
                member.as_deref(),
                Some("DeleteSurroundingText" | "CommitText")
            ),
            "exact replay admission must not mutate client text locally"
        );
        if member.as_deref() == Some("Cycle09Reached") {
            break;
        }
    }
}

async fn cycle09_surrounding_receipt(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    text: &str,
    cursor_pos: u32,
    anchor_pos: u32,
) {
    engine
        .set_surrounding_text(
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap(),
            crate::text::make_ibus_text(text.to_string()),
            cursor_pos,
            anchor_pos,
        )
        .await
        .unwrap();
    cycle09_assert_no_local_text_effect(harness).await;
}

async fn cycle09_take_registered_engine(harness: &Harness, path: &str) -> LayIbusEngine {
    let iface = harness
        .connection
        .object_server()
        .interface::<_, LayIbusEngine>(path)
        .await
        .unwrap();
    let engine = std::mem::replace(&mut *iface.get_mut().await, new_engine(harness));
    harness
        .connection
        .object_server()
        .remove::<LayIbusEngine, _>(path)
        .await
        .unwrap();
    engine
}

pub(super) async fn cycle09_manual_toggle(
    harness: &mut Harness,
    engine: LayIbusEngine,
) -> (LayIbusEngine, Result<(u8, bool), String>) {
    let path = engine.path.clone();
    let bridge = bridge(harness, &engine);
    harness
        .connection
        .object_server()
        .at(path.as_str(), engine)
        .await
        .unwrap();
    let (result, ()) = bounded(future::zip(bridge.manual_toggle_v3_inner(), async {
        serve_ping_and_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
    }))
    .await;
    cycle09_assert_no_local_text_effect(harness).await;
    let engine = cycle09_take_registered_engine(harness, &path).await;
    (engine, result.map_err(|error| error.to_string()))
}

pub(super) async fn cycle09_visible_tail(
    harness: &mut Harness,
    engine: LayIbusEngine,
) -> (
    LayIbusEngine,
    Result<(String, String, bool, u64, String, String), String>,
) {
    let path = engine.path.clone();
    let bridge = bridge(harness, &engine);
    harness
        .connection
        .object_server()
        .at(path.as_str(), engine)
        .await
        .unwrap();
    let (result, ()) = bounded(future::zip(bridge.visible_tail_v3_inner(), async {
        serve_ping_and_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
    }))
    .await;
    cycle09_assert_no_local_text_effect(harness).await;
    let engine = cycle09_take_registered_engine(harness, &path).await;
    (engine, result.map_err(|error| error.to_string()))
}

async fn cycle09_suppress_exact_replay(
    harness: &mut Harness,
    engine: LayIbusEngine,
    suffix: &str,
    epoch: u64,
    path: &str,
    layout_is_ru: bool,
) -> (LayIbusEngine, Result<bool, String>) {
    let engine_path = engine.path.clone();
    let bridge = bridge(harness, &engine);
    harness
        .connection
        .object_server()
        .at(engine_path.as_str(), engine)
        .await
        .unwrap();
    let (result, ()) = bounded(future::zip(
        bridge.suppress_next_autocorrect_v2_inner(
            suffix.to_string(),
            epoch,
            path.to_string(),
            layout_is_ru,
        ),
        async {
            serve_ping_and_marker(&mut harness.peer).await;
            assert!(harness.observer.process_next().await.unwrap());
        },
    ))
    .await;
    cycle09_assert_no_local_text_effect(harness).await;
    let engine = cycle09_take_registered_engine(harness, &engine_path).await;
    (engine, result.map_err(|error| error.to_string()))
}

fn cycle09_exact_handoff_is_live(engine: &LayIbusEngine) -> bool {
    let state = engine.shared.lock().unwrap();
    state.exact_manual_toggle_handoff_epoch.is_some()
        || state.exact_manual_toggle_handoff_path.is_some()
        || state.preserve_active_path_until.is_some()
}

fn cycle09_exact_suppression_is_armed(engine: &LayIbusEngine) -> bool {
    engine.committed_tail.autocorrect_suppression.is_some()
        || engine
            .shared
            .lock()
            .unwrap()
            .autocorrect_suppression
            .is_some()
}

async fn cycle09_harness() -> Harness {
    let (connection, mut peer) = controlled_pair();
    let config = AdapterConfig::new(
        ConnectionGeneration(60),
        vec![profile("lay-ime-us"), profile("lay-ime-ru")],
    )
    .unwrap()
    .with_acquisition_budget(CALLBACK_BUDGET);
    let pending = PendingContextAdapter::subscribe(connection.clone(), config)
        .await
        .unwrap();
    let (result, ()) = future::zip(
        pending.bootstrap(),
        serve_bootstrap(&mut peer, "lay-ime-us"),
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

async fn cycle09_source(harness: &mut Harness) -> LayIbusEngine {
    let mut source = known_engine(harness).await;
    source.config.auto_replace = false;
    source.config.typing_assist = false;
    source.config.nanda_precognition = false;
    source.set_client_capabilities(41);
    source.set_content_type_state(0, 0);
    for (offset, (ch, code)) in [('a', 30), ('b', 48), ('c', 46)].into_iter().enumerate() {
        assert!(
            legacy_key(
                harness,
                &mut source,
                11_900 + offset as u32,
                ch as u32,
                code,
                0,
            )
            .await
        );
        expect_legacy_commit(&mut harness.peer).await;
    }
    assert_eq!(source.committed_tail.buffer, " abc");
    assert!(source.context_word_is_known());
    let scope = source.context_word_scope.as_ref().unwrap();
    assert_eq!(scope.lineage().completeness, WordCompleteness::KnownStart);
    assert!(source
        .live_context_token()
        .is_some_and(|token| token.matches_word_scope(scope)));
    source
}

async fn cycle09_add_trailing_boundary(harness: &mut Harness, source: &mut LayIbusEngine) {
    assert!(legacy_key(harness, source, 11_910, KEY_SPACE, 57, 0).await);
    expect_legacy_commit(&mut harness.peer).await;
    assert_eq!(source.committed_tail.buffer, " abc ");
}

async fn cycle09_factory_handoff(
    harness: &mut Harness,
    mut source: LayIbusEngine,
    serial: u32,
    target_path: &str,
    preinstall_snapshot: Option<Option<SurroundingTextSnapshot>>,
    target_component: &str,
) -> LayIbusEngine {
    let factory = Message::method_call("/org/freedesktop/IBus/Factory", "CreateEngine")
        .unwrap()
        .interface(FACTORY_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .with_flags(zbus::message::Flags::NoReplyExpected)
        .unwrap()
        .build(&target_component)
        .unwrap();
    harness.peer.connection.send(&factory).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    consume_detached_callback_reply(&mut harness.peer, serial).await;
    let callback = harness
        .adapter
        .begin_factory_callback(&factory.header(), Instant::now(), profile(target_component))
        .await
        .unwrap();
    assert!(harness
        .adapter
        .bind_factory_target(&callback, engine_path(target_path)));
    global_engine_changed(harness, serial + 1, target_component).await;

    for (offset, member) in [(2, "FocusOut"), (3, "Disable")] {
        let event = observed_callback_without_reply(serial + offset, &source.path, member);
        harness.peer.connection.send(&event).await.unwrap();
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        consume_detached_callback_reply(&mut harness.peer, serial + offset).await;
        if member == "FocusOut" {
            source.focus_out(event.header()).await;
        } else {
            source.disable(event.header()).await;
        }
    }

    let mut target = LayIbusEngine::new_from_component(
        target_path.to_string(),
        source.shared.clone(),
        Some(harness.adapter.clone()),
        target_component,
        true,
        ime_config(),
    );
    // C18 production order: target capability/content metadata arrives before
    // the marker publishes and installs the same-context transfer.
    target.set_client_capabilities(41);
    target.set_content_type_state(0, 0);
    if let Some(snapshot) = preinstall_snapshot {
        target.observe_external_surrounding_text(snapshot);
    }
    let focus = observed_callback_without_reply(serial + 4, target_path, "FocusIn");
    harness.peer.connection.send(&focus).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    consume_detached_callback_reply(&mut harness.peer, serial + 4).await;
    target.focus_in_callback(focus.header()).await;
    let get = bounded(next_peer_message(&mut harness.peer)).await;
    assert_eq!(
        get.header().member().map(|member| member.as_str()),
        Some("Get")
    );
    harness
        .peer
        .connection
        .reply(
            &get.header(),
            &OwnedValue::from(ObjectPath::try_from(CONTEXT_PATH).unwrap()),
        )
        .await
        .unwrap();
    forward_marker_bounded(&mut harness.peer).await;
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    let outcome = harness
        .adapter
        .try_finish_activation_for(&engine_path(target_path))
        .unwrap()
        .expect("actual factory handoff must become ready");
    assert!(matches!(&outcome, ActivationOutcome::Transfer(_)));
    assert!(target.install_context_activation(outcome));
    target.config.auto_replace = false;
    target
}

pub(super) fn cycle09_tail_is_authoritative(
    reply: &Result<(String, String, bool, u64, String, String), String>,
    path: &str,
    layout_is_ru: bool,
    expected_tail: &str,
) -> bool {
    matches!(reply, Ok((state, text, layout, _, reply_path, focus))
        if state == "passive:committed-tail"
            && text == expected_tail
            && *layout == layout_is_ru
            && reply_path == path
            && !focus.is_empty())
}

async fn td121_no_target_snapshot_handoff(
    serial: u32,
    preinstall_snapshot: Option<Option<SurroundingTextSnapshot>>,
) -> (Harness, LayIbusEngine, String, u64) {
    let mut harness = cycle09_harness().await;
    let mut source = new_engine(&harness);
    start_source_free_unknown(&mut harness, &mut source).await;
    source.config.auto_replace = false;
    source.config.typing_assist = false;
    source.config.nanda_precognition = false;
    source.set_client_capabilities(41);
    source.set_content_type_state(0, 0);
    for (offset, (ch, code)) in [('a', 30), ('b', 48), ('c', 46)].into_iter().enumerate() {
        assert!(
            legacy_key(
                &mut harness,
                &mut source,
                serial + offset as u32,
                ch as u32,
                code,
                0,
            )
            .await
        );
        expect_legacy_commit(&mut harness.peer).await;
    }
    assert_eq!(source.committed_tail.buffer, "abc");
    assert!(!source.context_word_is_known());
    cycle09_surrounding_receipt(&mut harness, &mut source, "prefix abc", 10, 10).await;

    let (source, disposition) = cycle09_manual_toggle(&mut harness, source).await;
    assert_eq!(disposition, Ok((3, false)));
    let (source, source_tail) = cycle09_visible_tail(&mut harness, source).await;
    let source_path = source.path.clone();
    assert!(cycle09_tail_is_authoritative(
        &source_tail,
        &source_path,
        false,
        "abc"
    ));
    let source_epoch = source_tail.as_ref().unwrap().3;

    let target_path = format!("{TARGET_PATH}_td121_no_target_snapshot_{serial}");
    let target = cycle09_factory_handoff(
        &mut harness,
        source,
        serial + 20,
        &target_path,
        preinstall_snapshot,
        "lay-ime-ru",
    )
    .await;
    (harness, target, target_path, source_epoch)
}

async fn firefox_replay_callback(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    serial: &mut u32,
    key: (u32, u32, u32),
) -> bool {
    let message = Message::method_call(engine.path.as_str(), "ProcessKeyEvent")
        .unwrap()
        .interface(ENGINE_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(*serial).unwrap())
        .build(&key)
        .unwrap();
    harness.peer.connection.send(&message).await.unwrap();
    assert!(harness.observer.process_next().await.unwrap());
    consume_detached_callback_reply(&mut harness.peer, *serial).await;
    *serial += 1;
    let emitter =
        zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone()).unwrap();
    let handled = engine
        .process_key_event(message.header(), emitter, key.0, key.1, key.2)
        .await
        .unwrap();
    cycle09_assert_no_local_text_effect(harness).await;
    handled
}

async fn firefox_replay_reset(harness: &mut Harness, engine: &mut LayIbusEngine, serial: &mut u32) {
    let message = observed_callback_without_reply(*serial, &engine.path, "Reset");
    harness.peer.connection.send(&message).await.unwrap();
    assert!(harness.observer.process_next().await.unwrap());
    consume_detached_callback_reply(&mut harness.peer, *serial).await;
    *serial += 1;
    engine
        .reset(
            message.header(),
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap(),
        )
        .await
        .unwrap();
    cycle09_assert_no_local_text_effect(harness).await;
}

async fn firefox_replay_prefix_with_reset(
    serial: &mut u32,
    prefix_chars: usize,
    delayed_receipt: bool,
) -> (Harness, LayIbusEngine) {
    let (mut harness, target, path, epoch) = td121_no_target_snapshot_handoff(*serial, None).await;
    *serial += 40;
    let (mut engine, suppression) =
        cycle09_suppress_exact_replay(&mut harness, target, "abc", epoch, &path, true).await;
    assert_eq!(suppression, Ok(true));
    engine.config.text_backend = "ime".into();
    engine.config.typing_assist = false;
    engine.config.nanda_precognition = false;
    assert!(engine.live_composition_enabled());
    assert!(engine.exact_replay_quarantine_active());
    for _ in 0..3 {
        for state in [0, RELEASE_MASK] {
            assert!(
                !firefox_replay_callback(
                    &mut harness,
                    &mut engine,
                    serial,
                    (KEY_BACKSPACE, 14, state),
                )
                .await
            );
        }
    }
    assert_eq!(engine.committed_tail.buffer, "");
    for (ch, code) in [('ф', 30), ('и', 48), ('с', 46)]
        .into_iter()
        .take(prefix_chars)
    {
        for state in [0, RELEASE_MASK] {
            assert!(
                !firefox_replay_callback(
                    &mut harness,
                    &mut engine,
                    serial,
                    (0x0100_0000 | ch as u32, code, state),
                )
                .await
            );
        }
    }
    firefox_replay_reset(&mut harness, &mut engine, serial).await;
    assert!(engine.context_reset_rereceipt.is_some());
    let prefix: String = "фис".chars().take(prefix_chars).collect();
    assert_eq!(engine.committed_tail.buffer, prefix);
    let received: String = prefix
        .chars()
        .take(prefix_chars - usize::from(delayed_receipt))
        .collect();
    let text = format!("prefix {received}");
    let cursor = text.chars().count() as u32;
    cycle09_surrounding_receipt(&mut harness, &mut engine, &text, cursor, cursor).await;
    assert!(engine.context_reset_rereceipt.is_some());
    assert_eq!(
        engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
        !delayed_receipt
    );
    assert!(!engine.context_word_is_known());
    (harness, engine)
}

#[test]
fn firefox_native_replay_append_preserves_reset_lineage_until_exact_tail() {
    zbus::block_on(bounded(async {
        for (prefix_chars, delayed_receipt) in [(1, false), (2, false), (2, true)] {
            let mut serial = 15_000;
            let (mut harness, mut engine) =
                firefox_replay_prefix_with_reset(&mut serial, prefix_chars, delayed_receipt).await;
            for (ch, code) in [('ф', 30), ('и', 48), ('с', 46)]
                .into_iter()
                .skip(prefix_chars)
            {
                for state in [0, RELEASE_MASK] {
                    assert!(
                        !firefox_replay_callback(
                            &mut harness,
                            &mut engine,
                            &mut serial,
                            (0x0100_0000 | ch as u32, code, state),
                        )
                        .await
                    );
                }
                assert!(
                    engine.context_reset_rereceipt.is_some(),
                    "validated native append must retain the Reset predecessor"
                );
                assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                assert!(!engine.context_word_is_known());
            }
            assert_eq!(engine.committed_tail.buffer, "фис");
            firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
            cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix фис", 10, 10).await;
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert!(!engine.context_word_is_known());
            assert!(engine.committed_tail.pending_completion_learning.is_none());
            assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
            let path = engine.path.clone();
            let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
            assert_eq!(
                disposition,
                Ok(lay::manual_toggle::ImeManualToggleOutcome::DelegateExactImeTail.as_v3())
            );
            assert!(
                engine.layout_gesture.layout_is_ru,
                "exact Firefox handoff must retain the verified Russian target layout"
            );
            let (engine, tail) = cycle09_visible_tail(&mut harness, engine).await;
            assert!(cycle09_tail_is_authoritative(&tail, &path, true, "фис"));
            assert!(!engine.context_word_is_known());
        }
    }));
}

#[test]
fn firefox_replay_reset_receipt_rejects_revoked_scope_and_context() {
    zbus::block_on(bounded(async {
        for gap in [
            "wrong_key",
            "command",
            "local_scope_removed",
            "shared_scope_removed",
            "expired",
            "selection",
            "capability_loss",
            "focus_out",
        ] {
            let mut serial = 15_200;
            let (mut harness, mut engine) =
                firefox_replay_prefix_with_reset(&mut serial, 2, true).await;
            let mut key = (0x0100_0000 | 'с' as u32, 46, 0);
            match gap {
                "wrong_key" => key = (u32::from(b'x'), 45, 0),
                "command" => key.2 = 1 << 2,
                "local_scope_removed" => engine.committed_tail.autocorrect_suppression = None,
                "shared_scope_removed" => {
                    engine.shared.lock().unwrap().autocorrect_suppression = None;
                }
                "expired" => {
                    let mut suppression = engine
                        .committed_tail
                        .autocorrect_suppression
                        .clone()
                        .unwrap();
                    let crate::protocol::AutocorrectSuppression::ExactReplay(ref mut scope) =
                        suppression
                    else {
                        panic!("fixture must arm the actual replay lease");
                    };
                    scope.expires_at = Instant::now() - Duration::from_millis(1);
                    engine.committed_tail.autocorrect_suppression = Some(suppression.clone());
                    engine.shared.lock().unwrap().autocorrect_suppression = Some(suppression);
                }
                "selection" => {
                    cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix фи", 9, 7).await;
                }
                "capability_loss" => engine.set_client_capabilities(9),
                "focus_out" => {
                    let message = observed_callback_without_reply(serial, &engine.path, "FocusOut");
                    harness.peer.connection.send(&message).await.unwrap();
                    assert!(harness.observer.process_next().await.unwrap());
                    consume_detached_callback_reply(&mut harness.peer, serial).await;
                    engine.focus_out(message.header()).await;
                }
                _ => unreachable!(),
            }
            if gap != "focus_out" {
                assert!(firefox_replay_callback(&mut harness, &mut engine, &mut serial, key).await);
                assert_eq!(engine.committed_tail.buffer, "фи", "{gap}");
            }
            assert!(engine.context_reset_rereceipt.is_none(), "{gap}");
            assert!(!engine.context_word_is_known());
            cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix фис", 10, 10).await;
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert!(engine.committed_tail.pending_completion_learning.is_none());
            assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
            let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
            assert!(
                matches!(disposition, Ok((0, _)) | Err(_)),
                "{gap}: {disposition:?}"
            );
            assert!(!cycle09_exact_handoff_is_live(&engine), "{gap}");
        }
    }));
}

#[test]
fn td121_captured_target_snapshot_cannot_survive_layout_a_b_a() {
    zbus::block_on(bounded(async {
        let (mut harness, target_b, _target_b_path, _) =
            td121_no_target_snapshot_handoff(12_720, None).await;
        let old_target_token = target_b
            .exact_manual_target_snapshot
            .as_ref()
            .expect("B inherited target receipt")
            .target_token
            .clone();
        let target_a_path = format!("{TARGET_PATH}_td121_return_a");
        let target_a = cycle09_factory_handoff(
            &mut harness,
            target_b,
            12_760,
            &target_a_path,
            None,
            "lay-ime-us",
        )
        .await;
        assert!(!harness.adapter.revalidate(&old_target_token));
        assert!(target_a
            .exact_manual_target_snapshot
            .as_ref()
            .is_none_or(|receipt| receipt.target_token != old_target_token));
    }));
}

#[test]
fn td121_same_context_target_without_a_fresh_snapshot_keeps_the_controlled_handoff_lease() {
    zbus::block_on(bounded(async {
        let (mut harness, target, target_path, source_epoch) =
            td121_no_target_snapshot_handoff(12_480, None).await;

        // This is the C18 production order: the same canonical input context
        // has transferred and the exact handoff is still live, but the fresh
        // engine has not received SetSurroundingText yet.
        assert!(target.client_context.surrounding_text_snapshot.is_none());
        assert!(!target.context_word_is_known());
        assert!(target.context_exact_manual_handoff_bounds_unknown_suffix());
        assert!(target.exact_manual_toggle_handoff_is_bound_to_current_owner());
        assert!(target.exact_manual_toggle_handoff_is_live());
        assert!(!target.current_external_snapshot_agrees_with_owned_tail());

        let (target, target_tail) = cycle09_visible_tail(&mut harness, target).await;
        assert!(
            cycle09_tail_is_authoritative(&target_tail, &target_path, true, "abc"),
            "same-context controlled handoff must retain its exact lease before the target's first optional surrounding-text callback: {target_tail:?}"
        );
        assert!(target.client_context.surrounding_text_snapshot.is_none());
        assert!(target.exact_manual_target_snapshot.is_some());
        let (target, suppression) = cycle09_suppress_exact_replay(
            &mut harness,
            target,
            "abc",
            source_epoch,
            &target_path,
            true,
        )
        .await;
        assert_eq!(suppression, Ok(true));
        assert!(target.exact_manual_target_snapshot.is_none());
        assert!(cycle09_exact_suppression_is_armed(&target));
    }));
}

#[test]
fn td121_target_capability_change_after_transfer_invalidates_the_inherited_snapshot() {
    zbus::block_on(bounded(async {
        let (mut harness, mut target, target_path, _) =
            td121_no_target_snapshot_handoff(12_540, None).await;
        assert!(target.exact_manual_target_snapshot.is_some());
        target.set_client_capabilities(0);
        assert!(target.exact_manual_target_snapshot.is_none());
        let (target, target_tail) = cycle09_visible_tail(&mut harness, target).await;
        assert!(!cycle09_tail_is_authoritative(
            &target_tail,
            &target_path,
            true,
            "abc"
        ));
        assert!(!cycle09_exact_suppression_is_armed(&target));
    }));
}

#[test]
fn td121_preinstall_target_observation_never_inherits_the_source_snapshot() {
    zbus::block_on(bounded(async {
        let cases = [
            ("none", None),
            (
                "mismatch",
                Some(SurroundingTextSnapshot::new("prefix ab".into(), 9, 9)),
            ),
            (
                "selection",
                Some(SurroundingTextSnapshot::new("prefix abc".into(), 10, 7)),
            ),
        ];
        for (index, (name, snapshot)) in cases.into_iter().enumerate() {
            let (mut harness, target, target_path, _) =
                td121_no_target_snapshot_handoff(12_600 + index as u32 * 60, Some(snapshot)).await;
            assert!(target.client_context.surrounding_text_callback_observed);
            assert!(
                target.exact_manual_target_snapshot.is_none(),
                "{name}: pre-install target observation must block inherited proof"
            );
            let (target, target_tail) = cycle09_visible_tail(&mut harness, target).await;
            assert!(
                !cycle09_tail_is_authoritative(&target_tail, &target_path, true, "abc"),
                "{name}: contradictory target observation authorized exact tail: {target_tail:?}"
            );
            assert!(!cycle09_exact_suppression_is_armed(&target));
            cycle09_assert_no_local_text_effect(&mut harness).await;
        }
    }));
}

#[test]
fn td121_expired_inherited_snapshot_cannot_authorize_visible_tail_or_suppression() {
    zbus::block_on(bounded(async {
        let (mut harness, mut target, target_path, source_epoch) =
            td121_no_target_snapshot_handoff(12_800, None).await;
        target
            .exact_manual_target_snapshot
            .as_mut()
            .expect("fixture installs typed inherited receipt")
            .source
            .expires_at = Instant::now();
        assert!(!target.inherited_exact_manual_snapshot_agrees_with_owned_tail());
        let (target, target_tail) = cycle09_visible_tail(&mut harness, target).await;
        assert!(!cycle09_tail_is_authoritative(
            &target_tail,
            &target_path,
            true,
            "abc"
        ));
        let (target, suppression) = cycle09_suppress_exact_replay(
            &mut harness,
            target,
            "abc",
            source_epoch,
            &target_path,
            true,
        )
        .await;
        assert_eq!(suppression, Ok(false));
        assert!(!cycle09_exact_suppression_is_armed(&target));
    }));
}

#[test]
fn exact_replay_requires_current_unselected_external_tail_at_each_lease() {
    zbus::block_on(bounded(async {
        let cases = [
            Cycle09ExternalTailCase::ExactMatch,
            Cycle09ExternalTailCase::TrailingBoundaryExactMatch,
            Cycle09ExternalTailCase::SourceMismatch,
            Cycle09ExternalTailCase::TargetMismatch,
            Cycle09ExternalTailCase::LateTargetMismatch,
            Cycle09ExternalTailCase::BoundaryMismatch,
            Cycle09ExternalTailCase::Selection,
            Cycle09ExternalTailCase::NoSnapshot,
            Cycle09ExternalTailCase::StaleContext,
        ];
        let mut false_accepts = Vec::new();

        for (index, case) in cases.into_iter().enumerate() {
            let mut harness = cycle09_harness().await;
            let mut source = cycle09_source(&mut harness).await;
            if matches!(case, Cycle09ExternalTailCase::TrailingBoundaryExactMatch) {
                cycle09_add_trailing_boundary(&mut harness, &mut source).await;
            }
            let expected_tail =
                if matches!(case, Cycle09ExternalTailCase::TrailingBoundaryExactMatch) {
                    " abc "
                } else {
                    " abc"
                };
            match case {
                Cycle09ExternalTailCase::SourceMismatch => {
                    cycle09_surrounding_receipt(&mut harness, &mut source, "prefix ab", 9, 9).await;
                }
                Cycle09ExternalTailCase::BoundaryMismatch => {
                    cycle09_surrounding_receipt(&mut harness, &mut source, "prefixxabc", 10, 10)
                        .await;
                }
                Cycle09ExternalTailCase::Selection => {
                    cycle09_surrounding_receipt(&mut harness, &mut source, "prefix abc", 10, 7)
                        .await;
                }
                Cycle09ExternalTailCase::NoSnapshot => {
                    assert!(source.client_context.surrounding_text_snapshot.is_none());
                }
                Cycle09ExternalTailCase::TrailingBoundaryExactMatch => {
                    cycle09_surrounding_receipt(&mut harness, &mut source, "prefix abc ", 11, 11)
                        .await;
                }
                _ => {
                    cycle09_surrounding_receipt(&mut harness, &mut source, "prefix abc", 10, 10)
                        .await;
                }
            }

            if matches!(case, Cycle09ExternalTailCase::StaleContext) {
                let stale = source.live_context_token().unwrap();
                global_engine_changed(&mut harness, 12_000 + index as u32 * 20, "foreign-ime")
                    .await;
                assert!(!harness.adapter.revalidate(&stale));
            }

            let (source_after_toggle, disposition) =
                cycle09_manual_toggle(&mut harness, source).await;
            let (source_after_capture, source_tail) =
                cycle09_visible_tail(&mut harness, source_after_toggle).await;
            let source_path = source_after_capture.path.clone();
            let source_authoritative =
                cycle09_tail_is_authoritative(&source_tail, &source_path, false, expected_tail);

            if matches!(case, Cycle09ExternalTailCase::StaleContext) {
                if source_authoritative
                    || cycle09_exact_handoff_is_live(&source_after_capture)
                    || cycle09_exact_suppression_is_armed(&source_after_capture)
                {
                    false_accepts.push(format!("{case:?}: stale source remained authoritative"));
                }
                continue;
            }

            assert_eq!(
                disposition,
                Ok((3, false)),
                "{case:?}: focused GUI route must remain typed exact until source capture"
            );
            if matches!(
                case,
                Cycle09ExternalTailCase::SourceMismatch
                    | Cycle09ExternalTailCase::BoundaryMismatch
                    | Cycle09ExternalTailCase::Selection
                    | Cycle09ExternalTailCase::NoSnapshot
            ) {
                if source_authoritative
                    || cycle09_exact_handoff_is_live(&source_after_capture)
                    || cycle09_exact_suppression_is_armed(&source_after_capture)
                {
                    false_accepts.push(format!(
                            "{case:?}: source capture accepted absent, selected, or boundary-mismatched external text"
                    ));
                }
                continue;
            }

            assert!(
                source_authoritative,
                "{case:?}: exact source snapshot must capture: {source_tail:?}"
            );
            assert!(cycle09_exact_handoff_is_live(&source_after_capture));
            assert!(!cycle09_exact_suppression_is_armed(&source_after_capture));
            let source_epoch = source_tail.as_ref().unwrap().3;
            let target_path = format!("{TARGET_PATH}_cycle09_{index}");
            let mut target = cycle09_factory_handoff(
                &mut harness,
                source_after_capture,
                12_100 + index as u32 * 20,
                &target_path,
                None,
                "lay-ime-ru",
            )
            .await;
            let target_text = match case {
                Cycle09ExternalTailCase::TargetMismatch => "prefix ab",
                Cycle09ExternalTailCase::TrailingBoundaryExactMatch => "prefix abc ",
                _ => "prefix abc",
            };
            let target_cursor = target_text.chars().count() as u32;
            cycle09_surrounding_receipt(
                &mut harness,
                &mut target,
                target_text,
                target_cursor,
                target_cursor,
            )
            .await;
            let (mut target_after_validation, target_tail) =
                cycle09_visible_tail(&mut harness, target).await;
            let target_authoritative =
                cycle09_tail_is_authoritative(&target_tail, &target_path, true, expected_tail);
            if matches!(case, Cycle09ExternalTailCase::LateTargetMismatch) {
                assert!(
                    target_authoritative,
                    "late target mismatch must first pass target VisibleTailV3"
                );
                cycle09_surrounding_receipt(
                    &mut harness,
                    &mut target_after_validation,
                    "prefix ab",
                    9,
                    9,
                )
                .await;
            }
            let (target, suppression) = cycle09_suppress_exact_replay(
                &mut harness,
                target_after_validation,
                if matches!(case, Cycle09ExternalTailCase::TrailingBoundaryExactMatch) {
                    "abc "
                } else {
                    "abc"
                },
                source_epoch,
                &target_path,
                true,
            )
            .await;
            let suppression_accepted =
                suppression == Ok(true) || cycle09_exact_suppression_is_armed(&target);

            match case {
                Cycle09ExternalTailCase::ExactMatch
                | Cycle09ExternalTailCase::TrailingBoundaryExactMatch => {
                    assert!(target_authoritative, "exact target snapshot must validate");
                    assert_eq!(suppression, Ok(true));
                    assert!(cycle09_exact_suppression_is_armed(&target));
                    assert!(!cycle09_exact_handoff_is_live(&target));
                }
                Cycle09ExternalTailCase::TargetMismatch => {
                    if target_authoritative
                        || suppression_accepted
                        || cycle09_exact_handoff_is_live(&target)
                    {
                        false_accepts.push(format!(
                            "{case:?}: target lease accepted boundary-mismatched external text"
                        ));
                    }
                }
                Cycle09ExternalTailCase::LateTargetMismatch => {
                    if suppression_accepted || cycle09_exact_handoff_is_live(&target) {
                        false_accepts.push(format!(
                            "{case:?}: suppression arm accepted changed external text"
                        ));
                    }
                }
                _ => unreachable!(),
            }
        }

        assert!(
            false_accepts.is_empty(),
            "current external-tail authority violations: {}",
            false_accepts.join("; ")
        );
    }));
}
