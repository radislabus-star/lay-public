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
    retained_boundary_literal_keys(harness, &mut engine, &mut next, appended, true).await;
    assert!(engine.context_reset_rereceipt_computation_allowed());
    assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
    retained_boundary_literal_keys(harness, &mut engine, &mut next, " ", true).await;
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
                    retained_boundary_literal_keys(
                        &mut harness,
                        &mut engine,
                        &mut 25_331,
                        "a",
                        true,
                    )
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

struct Td121CompletionMemory {
    directory: std::path::PathBuf,
    previous_environment: [(&'static str, Option<std::ffi::OsString>); 4],
}

impl Td121CompletionMemory {
    fn seed() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};

        static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("fixture clock after epoch")
            .as_nanos();
        let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "lay-td121-completion-memory-{}-{stamp}-{sequence}",
            std::process::id()
        ));
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        // create(), not create_dir_all(): never take ownership of an existing
        // directory. Drop removes only the directory created by this guard.
        builder
            .create(&directory)
            .expect("create private fixture directory");
        let guard = Self {
            directory,
            previous_environment: [
                "LAY_NANDA_WORD_USAGE_EVENTS",
                "LAY_NANDA_WORD_USAGE_COUNTS",
                "LAY_NANDA_WORD_USAGE_FEEDBACK_COUNTS",
                "LAY_NANDA_USAGE_PRIOR",
            ]
            .map(|name| (name, std::env::var_os(name))),
        };

        let seed_input = guard.directory.join("accepted-completion.jsonl");
        let live_events = guard.directory.join("runtime-events.jsonl");
        let live_counts = guard.directory.join("runtime-counts.json");
        let feedback = guard.directory.join("feedback-counts.json");
        let legacy_prior = guard.directory.join("legacy-prior.json");
        // The compiler input and runtime event journal MUST differ: loading
        // both the seed JSONL and its compiled snapshot would count it twice.
        std::fs::write(
            &seed_input,
            concat!(
                "{\"ts\":1,\"kind\":\"accepted_ime\",\"word\":\"проверка\",",
                "\"context\":[],\"to\":\"проверка\",\"source\":\"ime\",",
                "\"operation\":\"completion\"}\n"
            ),
        )
        .expect("write one independently declared acceptance event");
        std::fs::write(&live_events, "").expect("create empty runtime event journal");
        let report = lay::nanda_wave::compile_usage_feedback_snapshot(&seed_input, &feedback)
            .expect("compile acceptance with existing canonical snapshot compiler");
        assert_eq!(report["status"], "ok");
        assert_eq!(report["parsed_events"], 1);
        assert_eq!(report["usage_events"], 1);
        assert_eq!(report["correction_receipts"], 0);
        assert_eq!(report["correction_events"], 0);

        // Missing private counts/prior are intentional: no inherited profile
        // may supplement this one-event fixture. These bindings precede the
        // first memory warm/readout and remain active for the whole test.
        for (name, path) in [
            ("LAY_NANDA_WORD_USAGE_EVENTS", &live_events),
            ("LAY_NANDA_WORD_USAGE_COUNTS", &live_counts),
            ("LAY_NANDA_WORD_USAGE_FEEDBACK_COUNTS", &feedback),
            ("LAY_NANDA_USAGE_PRIOR", &legacy_prior),
        ] {
            std::env::set_var(name, path);
        }
        guard
    }
}

impl Drop for Td121CompletionMemory {
    fn drop(&mut self) {
        for (name, previous) in &self.previous_environment {
            match previous {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        }
        // Best-effort teardown must not replace the original assertion failure
        // with a panic during unwinding. Canonical process isolation also
        // retires the worker and all OnceLock/cache state after this fixture.
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

fn td121_prime_completion_material() -> Td121CompletionMemory {
    use lay::typing_cpu::{LiveCompletionRequest, TypingCpu};
    let memory = Td121CompletionMemory::seed();
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
    memory
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
    let key = legacy_key_message_at(serial, TARGET_PATH, keyval, keycode, state);
    send_manually_dispatched_callback(&mut harness.peer, &key, TARGET_PATH, ENGINE_INTERFACE).await;
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
    td121_pending_current_completion_with_compute_and_reset_lineage(harness, compute, true).await
}

async fn td121_pending_current_completion_with_compute_and_reset_lineage(
    harness: &mut Harness,
    compute: bool,
    retain_reset_lineage: bool,
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
    if !retain_reset_lineage {
        engine.context_reset_rereceipt = None;
    }
    engine.config.nanda_precognition = compute;
    let _ = harness.connection.object_server();
    engine.reset_precognition_causal_counts();
    assert!(td121_readout_key(harness, &mut engine, 24_011, 'о' as u32, 36, 0).await);
    td121_expect_legacy_commit_text(&mut harness.peer, "о").await;
    assert_eq!(
        engine.composition.preedit_visible, compute,
        "enabled precognition retains its inert frame; disabling the feature hides it"
    );
    for cursor in [3, 2] {
        surrounding_receipt(harness, &mut engine, "привет", cursor, cursor).await;
        assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        assert!(engine.capture_observed_suffix_display_frame().is_none());
        assert!(engine.selected_precognition_suffix().is_none());
        assert_eq!(
            engine.composition.preedit_visible, compute,
            "enabled precognition must not blink across unconfirmed surrounding snapshots"
        );
        assert_eq!(
            engine.composition.preedit_suffix,
            if compute { "ивет" } else { "" }
        );
        assert!(engine.composition.preedit_candidates.is_empty());
        assert!(engine.composition.preedit_replacement_targets.is_empty());
        assert_eq!(engine.composition.preedit_display_only_pending, compute);
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
    let _memory = Td121CompletionMemory::seed();
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
    let _memory = td121_prime_completion_material();
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
    let _memory = td121_prime_completion_material();
    zbus::block_on(bounded(async {
        for retain_reset_lineage in [true, false] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = td121_pending_current_completion_with_compute_and_reset_lineage(
                &mut harness,
                true,
                retain_reset_lineage,
            )
            .await;
            let effects = td121_current_completion_receipt(&mut harness, &mut engine).await;
            if !retain_reset_lineage {
                assert!(engine.composition.preedit_visible);
                assert!(engine.composition.preedit_display_only_pending);
                assert!(engine.selected_precognition_suffix().is_none());
                assert!(effects.iter().all(|effect| {
                    effect.header().member().unwrap().as_str() == "UpdatePreeditText"
                }));
                assert!(effects.iter().all(|effect| {
                    !matches!(
                        effect.header().member().unwrap().as_str(),
                        "HidePreeditText" | "ShowPreeditText"
                    )
                }));
                for effect in &effects {
                    let body = effect.body();
                    let (text, _, visible, _) = body
                        .deserialize::<(zbus::zvariant::Value<'_>, u32, bool, u32)>()
                        .unwrap();
                    assert!(visible);
                    assert!(crate::ibus_interface::ibus_text_value_to_string(&text)
                        .is_some_and(|text| !text.is_empty()));
                }
                engine.cancel_precognition_display_generation();
                continue;
            }
            assert_eq!(engine.selected_precognition_suffix().as_deref(), Some("верка"), "exact client receipt must publish already-computed current material before the next key callback");
            assert!(!engine.composition.preedit_display_only_pending);
            assert_eq!(effects.len(), 1);
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
            assert!(
                !td121_readout_key(&mut harness, &mut engine, 24_020, KEY_LEFT_ALT, 56, 0).await
            );
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
        }
    }));
}

#[test]
fn td121_alt_before_exact_receipt_cannot_accept_later_cached_hint() {
    let _memory = td121_prime_completion_material();
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

fn manual_typed_key_message(
    serial: u32,
    member: &str,
    keyval: u32,
    keycode: u32,
    state: u32,
) -> Message {
    Message::method_call(TARGET_PATH, member)
        .unwrap()
        .interface(ENGINE_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(serial).unwrap())
        .with_flags(zbus::message::Flags::NoReplyExpected)
        .unwrap()
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
                let key =
                    manual_typed_key_message(14_810, "ProcessKeyEvent", keyval, keycode, state);
                {
                    let mut output_scope = engine.begin_context_bridge_output(Some(&token));
                    if bind_before_key {
                        assert!(
                            output_scope.consume_context_reset_rereceipt_for_exact_manual_handoff()
                        );
                    }
                    send_manually_dispatched_callback(
                        &mut harness.peer,
                        &key,
                        TARGET_PATH,
                        ENGINE_INTERFACE,
                    )
                    .await;
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
            // Preserve the original typed_key_message wire types. The manual
            // builder must not let integer keycodes default to signed i32.
            let (keyval, keycode, state): (u32, u32, u32) = (keyval, keycode, state);
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let engine = known_engine(&mut harness).await;
            let token = bridge_fence(&mut harness).await.unwrap();
            let key = match member {
                "malformed" => Message::method_call(TARGET_PATH, "ProcessKeyEvent")
                    .unwrap()
                    .interface(ENGINE_INTERFACE)
                    .unwrap()
                    .sender(DISPATCH_SENDER)
                    .unwrap()
                    .serial(NonZeroU32::new(14_900).unwrap())
                    .with_flags(zbus::message::Flags::NoReplyExpected)
                    .unwrap()
                    .build(&())
                    .unwrap(),
                "FocusOut" => method_message(
                    DISPATCH_SENDER,
                    14_900,
                    TARGET_PATH,
                    ENGINE_INTERFACE,
                    "FocusOut",
                ),
                _ => Message::method_call(TARGET_PATH, member)
                    .unwrap()
                    .interface(ENGINE_INTERFACE)
                    .unwrap()
                    .sender(DISPATCH_SENDER)
                    .unwrap()
                    .serial(NonZeroU32::new(14_900).unwrap())
                    .with_flags(zbus::message::Flags::NoReplyExpected)
                    .unwrap()
                    .build(&(keyval, keycode, state))
                    .unwrap(),
            };
            send_manually_dispatched_callback(
                &mut harness.peer,
                &key,
                TARGET_PATH,
                ENGINE_INTERFACE,
            )
            .await;
            let observed = harness.observer.process_next().await;
            if member == "malformed" {
                assert!(matches!(observed, Err(AdapterError::Denied)));
            } else {
                assert!(observed.unwrap());
            }
            assert!(
                !harness.adapter.revalidate_bridge(&token),
                "{member}, keyval={keyval}, state={state}"
            );
            if member == "malformed" {
                let adapter = harness.adapter.clone();
                let (fence, ()) = bounded(future::zip(
                    async {
                        let fence = adapter.begin_bridge_fence().await?;
                        adapter.complete_bridge_fence(fence).await
                    },
                    async {
                        let ping = next_peer_message(&mut harness.peer).await;
                        assert_eq!(ping.header().member().unwrap().as_str(), "Ping");
                        let value = ping.body().deserialize::<OwnedValue>().unwrap();
                        harness
                            .peer
                            .connection
                            .reply(&ping.header(), &value)
                            .await
                            .unwrap();
                        // Cancellation is checked after the actual bridge marker
                        // publication. Consume only that exact protocol signal;
                        // it is not client output and must not hide a text effect.
                        let marker = next_peer_message(&mut harness.peer).await;
                        assert_eq!(marker.header().message_type(), Type::Signal);
                        assert_eq!(marker.header().path().unwrap().as_str(), MARKER_PATH);
                        assert_eq!(
                            marker.header().interface().unwrap().as_str(),
                            MARKER_INTERFACE
                        );
                        assert_eq!(marker.header().member().unwrap().as_str(), MARKER_MEMBER);
                        assert_eq!(
                            marker.body().deserialize::<u64>().unwrap(),
                            u64::try_from(value).unwrap()
                        );
                    },
                ))
                .await;
                assert!(matches!(fence, Err(AdapterError::Cancelled)));
            } else {
                assert!(bridge_fence(&mut harness).await.is_err());
            }
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
            let key =
                manual_typed_key_message(15_000 + offset, "ProcessKeyEvent", KEY_LEFT_SHIFT, 42, 0);
            send_manually_dispatched_callback(
                &mut harness.peer,
                &key,
                TARGET_PATH,
                ENGINE_INTERFACE,
            )
            .await;
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
        let key = manual_typed_key_message(15_100, "ProcessKeyEvent", KEY_LEFT_SHIFT, 42, 0);
        send_manually_dispatched_callback(&mut harness.peer, &key, TARGET_PATH, ENGINE_INTERFACE)
            .await;
        assert!(harness.observer.process_next().await.is_err());
        assert!(harness.adapter.current_token().is_none());
    }));
}

async fn known_engine(harness: &mut Harness) -> LayIbusEngine {
    let mut engine = new_engine(harness);
    start_source_free_unknown(harness, &mut engine).await;
    let input_mode_before_key = engine.layout_gesture.layout_is_ru;
    let native_space_was_visible = engine.composition.preedit_visible;
    assert!(!legacy_key(harness, &mut engine, 3_000, KEY_SPACE, 57, 0).await);
    expect_legacy_native_space(
        harness,
        &engine,
        input_mode_before_key,
        native_space_was_visible,
    )
    .await;
    assert!(!legacy_key(harness, &mut engine, 3_001, KEY_SPACE, 57, RELEASE_MASK).await);
    assert!(engine.context_word_is_known());
    // These helpers invoke callbacks manually while the object is absent.
    // Finish the independent dispatcher before callers can register this path.
    assert!(harness
        .connection
        .object_server()
        .interface::<_, LayIbusEngine>(TARGET_PATH)
        .await
        .is_err());
    let token = engine.live_context_token().expect("known fixture token");
    bounded(complete_absent_manual_dispatch(&mut harness.peer)).await;
    assert_eq!(engine.live_context_token().as_ref(), Some(&token));
    assert!(harness.adapter.revalidate(&token));
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
    send_manually_dispatched_callback(&mut harness.peer, &message, TARGET_PATH, ENGINE_INTERFACE)
        .await;
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
    send_manually_dispatched_callback(&mut harness.peer, &focus_out, TARGET_PATH, ENGINE_INTERFACE)
        .await;
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

        let input_mode_before_key = engine.layout_gesture.layout_is_ru;
        let native_space_was_visible = engine.composition.preedit_visible;
        assert!(!legacy_key(&mut harness, &mut engine, 3_007, KEY_SPACE, 57, 0).await);
        expect_legacy_native_space(
            &mut harness,
            &engine,
            input_mode_before_key,
            native_space_was_visible,
        )
        .await;
        assert!(
            !legacy_key(
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
fn delayed_focus_out_after_later_word_revocation_preserves_successor_stamps() {
    zbus::block_on(bounded(async {
        for later_event in ["Reset", "Set"] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = known_engine(&mut harness).await;
            let old_token = engine.live_context_token().unwrap();

            let focus_out = receive(&mut harness, 30_100, "FocusOut").await;
            let focus_in = receive(&mut harness, 30_101, "FocusInId").await;
            let later = if later_event == "Set" {
                let set = Message::method_call(TARGET_PATH, "Set")
                    .unwrap()
                    .interface(PROPERTIES_INTERFACE)
                    .unwrap()
                    .sender(DISPATCH_SENDER)
                    .unwrap()
                    .serial(NonZeroU32::new(30_102).unwrap())
                    .with_flags(zbus::message::Flags::NoReplyExpected)
                    .unwrap()
                    .build(&(
                        ENGINE_INTERFACE,
                        "ContentType",
                        zbus::zvariant::Value::from((0u32, 1u32)),
                    ))
                    .unwrap();
                send_manually_dispatched_callback(
                    &mut harness.peer,
                    &set,
                    TARGET_PATH,
                    PROPERTIES_INTERFACE,
                )
                .await;
                assert!(bounded(harness.observer.process_next()).await.unwrap());
                set
            } else {
                receive(&mut harness, 30_102, "Reset").await
            };
            assert!(!harness.adapter.revalidate(&old_token));

            assert!(
                !engine
                    .observe_context_focus_out(&focus_out.header(), Instant::now())
                    .await,
                "{later_event} already retired the older FocusOut"
            );
            assert!(
                harness
                    .adapter
                    .observe_callback(&focus_in.header(), Instant::now())
                    .await
                    .is_ok(),
                "{later_event} must not erase the later FocusInId stamp"
            );
            assert!(
                harness
                    .adapter
                    .observe_callback(&later.header(), Instant::now())
                    .await
                    .is_ok(),
                "{later_event} stamp must remain available"
            );
            assert!(engine.context_owner.is_none(), "{later_event}");

            assert!(
                engine
                    .activate_context_from_header(
                        &focus_in.header(),
                        Instant::now(),
                        Some(CONTEXT_PATH),
                    )
                    .await,
                "{later_event} successor must start a fresh activation"
            );
            forward_marker_bounded(&mut harness.peer).await;
            assert!(bounded(harness.observer.process_next()).await.unwrap());
            engine.try_install_pending_context_activation();
            let new_token = engine.live_context_token().expect("successor authority");
            assert_ne!(new_token.owner, old_token.owner, "{later_event}");

            if later_event == "Set" {
                engine.set_content_type((0, 1), Some(later.header())).await;
            } else {
                assert!(
                    !engine
                        .observe_context_revocation(&later.header(), Instant::now())
                        .await
                );
            }
            assert!(
                harness.adapter.revalidate(&new_token),
                "{later_event} from the old owner must not revoke the successor"
            );
        }
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

pub(super) async fn actual_disable(harness: &mut Harness, engine: &mut LayIbusEngine, serial: u32) {
    let disable = method_message(
        DISPATCH_SENDER,
        serial,
        TARGET_PATH,
        ENGINE_INTERFACE,
        "Disable",
    );
    send_manually_dispatched_callback(&mut harness.peer, &disable, TARGET_PATH, ENGINE_INTERFACE)
        .await;
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
        .with_flags(zbus::message::Flags::NoReplyExpected)
        .unwrap()
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
    let key = legacy_key_message_at(serial, path, keyval, keycode, state);
    send_manually_dispatched_callback(&mut harness.peer, &key, path, ENGINE_INTERFACE).await;
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
            .with_flags(zbus::message::Flags::NoReplyExpected)
            .unwrap()
            .build(&(
                ENGINE_INTERFACE,
                "ContentType",
                zbus::zvariant::Value::from((10u32, 0u32)),
            ))
            .unwrap();
        send_manually_dispatched_callback(
            &mut harness.peer,
            &set,
            TARGET_PATH,
            PROPERTIES_INTERFACE,
        )
        .await;
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

    let activation_mode = engine.layout_gesture.layout_is_ru;
    assert!(!legacy_key(&mut harness, &mut engine, 3_216, KEY_LEFT_SHIFT, 42, 0,).await);
    expect_activation_input_mode_update(&mut harness, &engine, activation_mode).await;
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

    let input_mode_before_key = engine.layout_gesture.layout_is_ru;
    let native_space_was_visible = engine.composition.preedit_visible;
    let handled = legacy_key(&mut harness, &mut engine, 3_218, KEY_SPACE, 57, 0).await;
    assert!(
        !handled,
        "ordinary Legacy/terminal Space is the original native key"
    );
    expect_legacy_native_space(
        &mut harness,
        &engine,
        input_mode_before_key,
        native_space_was_visible,
    )
    .await;
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

        let activation_mode = engine.layout_gesture.layout_is_ru;
        assert!(!legacy_key(&mut harness, &mut engine, 3_222, KEY_LEFT_SHIFT, 42, 0,).await);
        expect_activation_input_mode_update(&mut harness, &engine, activation_mode).await;
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
    send_manually_dispatched_callback(&mut harness.peer, &focus_out, TARGET_PATH, ENGINE_INTERFACE)
        .await;
    send_manually_dispatched_callback(&mut harness.peer, &focus_in, TARGET_PATH, ENGINE_INTERFACE)
        .await;
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
    let activation_mode = engine.layout_gesture.layout_is_ru;
    assert!(!legacy_key(&mut harness, &mut engine, 14_300, KEY_LEFT_SHIFT, 42, 0).await);
    expect_activation_input_mode_update(&mut harness, &engine, activation_mode).await;
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
fn td121_default_bridge_accepts_authenticated_marker_after_callback_budget() {
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
        assert!(fence.deadline - began >= Duration::from_millis(15));
        // Hold the real marker beyond the 5 ms callback budget. The bridge
        // still admits it only through its nonce, ordered observer, and live
        // reducer token; the timer does not create or restore authority.
        async_io::Timer::at(began + Duration::from_millis(7)).await;
        assert!(Instant::now() < fence.deadline);
        deliver_held_marker(&mut harness, nonce).await;
        let bridged = harness.adapter.finish_bridge_fence(fence).unwrap();
        assert_eq!(bridged, token);
        assert!(engine.context_word_is_known());
        assert_eq!(engine.live_context_token().as_ref(), Some(&token));
        assert!(drain_output_to_proof(&mut harness).await.is_empty());
    }));
}

#[test]
fn td121_default_bridge_refuses_marker_after_its_own_deadline() {
    zbus::block_on(bounded(async {
        let mut harness = default_budget_harness().await;
        let engine = known_engine(&mut harness).await;
        let token = engine.live_context_token().unwrap();
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
    send_manually_dispatched_callback(
        &mut harness.peer,
        &predecessor,
        "/org/freedesktop/IBus/Factory",
        FACTORY_INTERFACE,
    )
    .await;
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
    send_manually_dispatched_callback(
        &mut harness.peer,
        &successor,
        "/org/freedesktop/IBus/Factory",
        FACTORY_INTERFACE,
    )
    .await;
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
    send_manually_dispatched_callback(
        &mut harness.peer,
        &stale_focus,
        STALE_PATH,
        ENGINE_INTERFACE,
    )
    .await;
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
    let activation_mode = current.layout_gesture.layout_is_ru;
    assert!(!legacy_key(&mut harness, &mut current, 3_234, KEY_LEFT_SHIFT, 42, 0,).await);
    expect_activation_input_mode_update(&mut harness, &current, activation_mode).await;
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
    let input_mode_before_key = current.layout_gesture.layout_is_ru;
    let native_space_was_visible = current.composition.preedit_visible;
    assert!(!legacy_key(&mut harness, &mut current, 3_236, KEY_SPACE, 57, 0).await);
    expect_legacy_native_space(
        &mut harness,
        &current,
        input_mode_before_key,
        native_space_was_visible,
    )
    .await;
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
    send_manually_dispatched_callback(
        &mut harness.peer,
        &late_stale_focus,
        STALE_PATH,
        ENGINE_INTERFACE,
    )
    .await;
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
        send_manually_dispatched_callback(
            &mut harness.peer,
            &first_factory,
            "/org/freedesktop/IBus/Factory",
            FACTORY_INTERFACE,
        )
        .await;
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
        send_manually_dispatched_callback(
            &mut harness.peer,
            &declined_factory,
            "/org/freedesktop/IBus/Factory",
            FACTORY_INTERFACE,
        )
        .await;
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
        send_manually_dispatched_callback(
            &mut harness.peer,
            &fresh_factory,
            "/org/freedesktop/IBus/Factory",
            FACTORY_INTERFACE,
        )
        .await;
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
        send_manually_dispatched_callback(
            &mut harness.peer,
            &focus,
            RECOVERED_PATH,
            ENGINE_INTERFACE,
        )
        .await;
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

        let activation_mode = recovered.layout_gesture.layout_is_ru;
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
        expect_activation_input_mode_update(&mut harness, &recovered, activation_mode).await;
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
        send_manually_dispatched_callback(
            &mut harness.peer,
            &old_factory,
            "/org/freedesktop/IBus/Factory",
            FACTORY_INTERFACE,
        )
        .await;
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
        send_manually_dispatched_callback(
            &mut harness.peer,
            &old_focus,
            SOURCE_PATH,
            ENGINE_INTERFACE,
        )
        .await;
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
        send_manually_dispatched_callback(
            &mut harness.peer,
            &fresh_factory,
            "/org/freedesktop/IBus/Factory",
            FACTORY_INTERFACE,
        )
        .await;
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
        send_manually_dispatched_callback(
            &mut harness.peer,
            &fresh_focus,
            RECOVERED_PATH,
            ENGINE_INTERFACE,
        )
        .await;
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
        let source_free_mode_before_key = recovered.layout_gesture.layout_is_ru;
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
        let property = bounded(next_peer_message(&mut harness.peer)).await;
        assert_input_mode_update(&property, &recovered, source_free_mode_before_key);
        assert!(!harness.adapter.revalidate(&old_token));
        assert!(bridge_fence(&mut harness).await.is_ok());

        let input_mode_before_key = recovered.layout_gesture.layout_is_ru;
        let native_space_was_visible = recovered.composition.preedit_visible;
        assert!(
            !legacy_key_at(
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
        expect_legacy_native_space(
            &mut harness,
            &recovered,
            input_mode_before_key,
            native_space_was_visible,
        )
        .await;
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
        let input_mode_before_key = engine.layout_gesture.layout_is_ru;
        assert!(legacy_key(harness, &mut engine, press_serial, ch as u32, code, 0).await);
        expect_legacy_commit(&mut harness.peer, &engine, input_mode_before_key).await;
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

    // This fixture models Reset as the next callback after the exact receipt.
    // Bind its production recency precondition to that callback rather than to
    // the time the controlled peer spent serving the receipt.
    assert!(engine.committed_tail.last_input_at.is_some());
    engine.committed_tail.last_input_at = Some(Instant::now());
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
    assert!(harness
        .connection
        .object_server()
        .interface::<_, LayIbusEngine>(path.as_str())
        .await
        .is_err());
    bounded(complete_absent_manual_dispatch(&mut harness.peer)).await;
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
                    let native_space_was_visible = engine.composition.preedit_visible;
                    let native_space_mode = engine.layout_gesture.layout_is_ru;
                    assert!(legacy_key(&mut harness, &mut engine, 15_010, KEY_SPACE, 57, 0).await);
                    expect_legacy_managed_space(
                        &mut harness,
                        &engine,
                        native_space_mode,
                        native_space_was_visible,
                    )
                    .await;
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
        let input_mode_before_key = engine.layout_gesture.layout_is_ru;
        assert!(legacy_key(harness, &mut engine, serial, ch as u32, code, 0).await);
        expect_legacy_commit(&mut harness.peer, &engine, input_mode_before_key).await;
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
    // This fixture models Reset at the next callback after the final key. Its
    // product recency precondition belongs to that modeled callback, not to
    // wall time spent by the test harness between callbacks.
    assert!(engine.committed_tail.last_input_at.is_some());
    engine.committed_tail.last_input_at = Some(Instant::now());
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

pub(super) async fn actual_reset(
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

                let input_mode_before_key = engine.layout_gesture.layout_is_ru;
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
                expect_legacy_commit(&mut harness.peer, &engine, input_mode_before_key).await;
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
                    let input_mode_before_key = engine.layout_gesture.layout_is_ru;
                    let native_space_was_visible = engine.composition.preedit_visible;
                    assert!(legacy_key(&mut harness, &mut engine, 9_710, KEY_SPACE, 57, 0).await);
                    expect_legacy_managed_space(
                        &mut harness,
                        &engine,
                        input_mode_before_key,
                        native_space_was_visible,
                    )
                    .await;
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
                    let input_mode_before_key = engine.layout_gesture.layout_is_ru;
                    assert!(
                        legacy_key(&mut harness, &mut engine, serial, ch as u32, code, 0).await
                    );
                    expect_legacy_commit(&mut harness.peer, &engine, input_mode_before_key).await;
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
                    .with_flags(zbus::message::Flags::NoReplyExpected)
                    .unwrap()
                    .build(&(
                        ENGINE_INTERFACE,
                        "ContentType",
                        zbus::zvariant::Value::from((8u32, 0u32)),
                    ))
                    .unwrap();
                send_manually_dispatched_callback(
                    &mut harness.peer,
                    &set,
                    TARGET_PATH,
                    PROPERTIES_INTERFACE,
                )
                .await;
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
                let input_mode_before_key = engine.layout_gesture.layout_is_ru;
                assert!(legacy_key(&mut harness, &mut engine, serial, ch as u32, code, 0).await);
                expect_legacy_commit(&mut harness.peer, &engine, input_mode_before_key).await;

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
            let input_mode_before_key = engine.layout_gesture.layout_is_ru;
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
            expect_legacy_commit(&mut harness.peer, &engine, input_mode_before_key).await;
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

#[test]
fn firefox_shortened_preedit_does_not_erase_prior_stale_surface_witness() {
    zbus::block_on(bounded(async {
        for (appended, keycode, stale_surface, matches_prior, accept_with_tab) in [
            ('d', 32, "abcdef", true, false),
            ('d', 32, "abcdef", true, true),
            ('d', 32, "abcxef", false, false),
            ('x', 45, "abcdef", false, false),
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = initial_observed_tail_reset(
                &mut harness,
                9_500,
                &[('a', 30), ('b', 48), ('c', 46)],
            )
            .await;
            exact_surrounding_receipt(&mut harness, &mut engine, "abc").await;
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            engine.record_context_reset_preedit_publication("def", 0);

            // The real legacy callback publishes the shortened frame *inside*
            // the key handler, after its commit but before the post-key Reset
            // lineage is advanced. Reproduce that interleaving here.
            let tail_before = engine.committed_tail.buffer.clone();
            let epoch_before = engine.committed_tail.epoch;
            engine.committed_tail.buffer.push(appended);
            engine.committed_tail.epoch += 1;
            engine.record_context_reset_preedit_publication("ef", 0);
            engine.committed_tail.buffer = tail_before;
            engine.committed_tail.epoch = epoch_before;

            let input_mode_before_key = engine.layout_gesture.layout_is_ru;
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    9_510,
                    appended as u32,
                    keycode,
                    0
                )
                .await
            );
            expect_legacy_commit(&mut harness.peer, &engine, input_mode_before_key).await;
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            // Firefox can still report the previous displayed completion after
            // the append shortens it. That surface remains inert until a fresh
            // exact receipt arrives; a different surface must still revoke it.
            surrounding_receipt(&mut harness, &mut engine, stale_surface, 4, 4).await;
            assert_eq!(engine.context_reset_rereceipt.is_some(), matches_prior);
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());

            exact_surrounding_receipt(&mut harness, &mut engine, &format!("abc{appended}")).await;
            assert_eq!(
                engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                matches_prior
            );
            if accept_with_tab {
                engine.composition.preedit_candidates = vec!["ef".to_string()];
                engine.composition.preedit_suffix = "ef".to_string();
                engine.composition.preedit_visible = true;
                engine.composition.preedit_display_only_pending = false;
                assert!(legacy_key(&mut harness, &mut engine, 9_511, KEY_TAB, 15, 0).await);
                td121_expect_legacy_commit_text(&mut harness.peer, "ef ").await;
            } else {
                let (_engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
                if matches_prior {
                    assert_eq!(
                        disposition,
                        Ok(
                            lay::manual_toggle::ImeManualToggleOutcome::DelegateExactImeTail
                                .as_v3()
                        )
                    );
                } else {
                    assert!(matches!(disposition, Ok((0, _)) | Err(_)));
                }
            }
        }
    }));
}

#[test]
fn firefox_zero_width_visible_completion_tab_appends_once_with_space() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = initial_observed_tail_reset(&mut harness, 9_515, &[('a', 30)]).await;
        assert!(
            engine.live_context_token().is_some(),
            "Reset must install a live settled token independently of the lost pending range"
        );
        exact_surrounding_receipt(&mut harness, &mut engine, "a").await;
        assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        engine.record_context_reset_preedit_publication("bc", 0);
        engine.config.nanda_precognition = true;
        engine.composition.preedit_candidates = vec!["bc".to_string()];
        engine.composition.preedit_suffix = "bc".to_string();
        engine.composition.preedit_visible = true;

        surrounding_receipt(&mut harness, &mut engine, "a\u{200b}", 1, 1).await;
        assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        assert!(engine.composition.preedit_visible);
        assert!(engine.composition.preedit_display_only_pending);
        assert!(engine.composition.preedit_candidates.is_empty());
        assert_eq!(
            engine
                .context_reset_rereceipt_visible_append_suffix()
                .as_deref(),
            Some("bc")
        );

        assert!(legacy_key(&mut harness, &mut engine, 9_517, KEY_TAB, 15, 0).await);
        td121_expect_legacy_commit_text(&mut harness.peer, "bc ").await;
        assert_eq!(engine.committed_tail.buffer, "abc ");
    }));
}

#[test]
fn transient_boundary_tab_rejects_changed_surface_or_authority() {
    zbus::block_on(bounded(async {
        for fault in [
            "no_publication",
            "different_suffix",
            "hidden_preedit",
            "new_tail_epoch",
            "different_right_text",
            "selection",
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = initial_observed_tail_reset(&mut harness, 9_518, &[('a', 30)]).await;
            exact_surrounding_receipt(&mut harness, &mut engine, "a").await;
            if fault != "no_publication" {
                engine.record_context_reset_preedit_publication("bc", 0);
            }
            engine.config.nanda_precognition = true;
            engine.composition.preedit_candidates = vec!["bc".to_string()];
            engine.composition.preedit_suffix = "bc".to_string();
            engine.composition.preedit_visible = true;
            let (surface, anchor) = match fault {
                "different_right_text" => ("a\u{200b}x", 1),
                "selection" => ("a\u{200b}", 0),
                _ => ("a\u{200b}", 1),
            };
            surrounding_receipt(&mut harness, &mut engine, surface, 1, anchor).await;
            match fault {
                "different_suffix" => engine.composition.preedit_suffix = "bd".to_string(),
                "hidden_preedit" => engine.composition.preedit_visible = false,
                "new_tail_epoch" => engine.committed_tail.epoch += 1,
                _ => {}
            }
            assert!(
                engine
                    .context_reset_rereceipt_visible_append_suffix()
                    .is_none(),
                "{fault}"
            );
            let emitter =
                zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                    .unwrap();
            assert!(
                !engine
                    .accept_completion(&mut crate::output::EngineOutput::legacy(&emitter), true)
                    .await
                    .unwrap(),
                "{fault}"
            );
            assert!(
                drain_output_to_proof(&mut harness)
                    .await
                    .iter()
                    .all(|member| !matches!(
                        member.as_str(),
                        "DeleteSurroundingText" | "CommitText"
                    )),
                "{fault}"
            );
            assert_eq!(engine.committed_tail.buffer, "a", "{fault}");
            engine.cancel_precognition_display_generation();
        }
    }));
}

#[test]
fn firefox_zero_width_space_after_caret_requires_fresh_exact_receipt() {
    zbus::block_on(bounded(async {
        for (surface, cursor, anchor, retain) in [
            ("a\u{200b}", 1, 1, true),
            ("ax", 1, 1, false),
            ("a\u{200b}x", 1, 1, false),
            ("a\u{200b}", 1, 0, false),
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = initial_observed_tail_reset(&mut harness, 9_520, &[('a', 30)]).await;
            exact_surrounding_receipt(&mut harness, &mut engine, "a").await;
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            engine.record_context_reset_preedit_publication("bc", 0);
            if retain {
                engine.config.nanda_precognition = true;
                engine.composition.preedit_candidates = vec!["bc".to_string()];
                engine.composition.preedit_suffix = "bc".to_string();
                engine.composition.preedit_visible = true;
            }

            surrounding_receipt(&mut harness, &mut engine, surface, cursor, anchor).await;
            assert_eq!(
                engine.context_reset_rereceipt.is_some(),
                retain,
                "{surface:?}"
            );
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "temporary or contradictory right-side text must not authorize whole-word handoff"
            );
            if retain {
                assert!(
                    engine.composition.preedit_visible,
                    "the transient editor sentinel must not hide the visible suggestion"
                );
            }
            exact_surrounding_receipt(&mut harness, &mut engine, "a").await;
            assert_eq!(
                engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                retain,
                "only the retained sentinel lineage can recover from a new exact receipt: {surface:?}"
            );
            if retain {
                engine.composition.preedit_candidates = vec!["bc".to_string()];
                engine.composition.preedit_suffix = "bc".to_string();
                engine.composition.preedit_visible = true;
                engine.composition.preedit_display_only_pending = false;
                assert!(legacy_key(&mut harness, &mut engine, 9_522, KEY_TAB, 15, 0).await);
                td121_expect_legacy_commit_text(&mut harness.peer, "bc ").await;
            }
        }
    }));
}

#[test]
fn firefox_zero_width_space_then_owned_append_requires_new_exact_token() {
    zbus::block_on(bounded(async {
        for (surface, cursor, anchor, retain) in [
            ("axbc", 2, 2, true),
            // Firefox can keep the *old* displayed completion in place while
            // the caret advances for the owned key, with one editor sentinel
            // still following that retired publication.
            ("abc\u{200b}", 2, 2, true),
            ("axbd", 2, 2, false),
            ("axbcq", 2, 2, false),
            ("abd\u{200b}", 2, 2, false),
            ("abc\u{200b}q", 2, 2, false),
            ("abc\u{200b}", 2, 1, false),
            ("axbc", 2, 1, false),
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = initial_observed_tail_reset(&mut harness, 9_530, &[('a', 30)]).await;
            exact_surrounding_receipt(&mut harness, &mut engine, "a").await;
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            engine.record_context_reset_preedit_publication("bc", 0);
            engine.config.nanda_precognition = true;
            engine.composition.preedit_candidates = vec!["bc".to_string()];
            engine.composition.preedit_suffix = "bc".to_string();
            engine.composition.preedit_visible = true;

            surrounding_receipt(&mut harness, &mut engine, "a\u{200b}", 1, 1).await;
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            let input_mode_before_key = engine.layout_gesture.layout_is_ru;
            assert!(legacy_key(&mut harness, &mut engine, 9_533, 'x' as u32, 45, 0).await);
            expect_legacy_commit(&mut harness.peer, &engine, input_mode_before_key).await;
            assert_eq!(engine.committed_tail.buffer, "ax");
            assert!(engine.context_reset_rereceipt.is_some());
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());

            surrounding_receipt(&mut harness, &mut engine, surface, cursor, anchor).await;
            assert_eq!(
                engine.context_reset_rereceipt.is_some(),
                retain,
                "{surface:?}"
            );
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            exact_surrounding_receipt(&mut harness, &mut engine, "ax").await;
            assert_eq!(
                engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                retain,
                "only a matching old publication and fresh exact appended token may recover"
            );
            if retain {
                // The next owned key may itself be followed by Firefox Reset.
                // The recovered receipt must remain a valid predecessor for
                // the new token after that Reset and its own exact snapshot.
                assert!(legacy_key(&mut harness, &mut engine, 9_534, 'd' as u32, 32, 0).await);
                td121_expect_legacy_commit_text(&mut harness.peer, "d").await;
                actual_reset(&mut harness, &mut engine, 9_535, false).await;
                exact_surrounding_receipt(&mut harness, &mut engine, "axd").await;
                assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            }
        }
    }));
}

#[test]
fn firefox_deferred_manual_refresh_retains_retired_preedit_from_previous_prefix() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine =
            initial_observed_tail_reset(&mut harness, 19_200, &[('a', 30), ('b', 48)]).await;
        engine.set_client_capabilities(1_073_741_865);
        exact_surrounding_receipt(&mut harness, &mut engine, "ab").await;
        engine.config.nanda_precognition = true;
        publish_fixture_append_completion(&mut harness, &mut engine, "cde").await;
        surrounding_receipt(&mut harness, &mut engine, "ab\u{200b}", 2, 2).await;
        let input_mode_before_key = engine.layout_gesture.layout_is_ru;
        assert!(legacy_key(&mut harness, &mut engine, 19_210, 'c' as u32, 46, 0).await);
        expect_legacy_commit(&mut harness.peer, &engine, input_mode_before_key).await;
        assert_eq!(engine.committed_tail.buffer, "abc");
        // The third owned letter shortens the displayed completion. Firefox
        // may still echo the publication made when only two letters existed.
        engine.record_context_reset_preedit_publication("de", 0);
        surrounding_receipt(&mut harness, &mut engine, "abcde\u{200b}", 3, 3).await;
        surrounding_receipt(&mut harness, &mut engine, "abc\u{200b}", 3, 3).await;
        assert!(engine.context_reset_rereceipt_manual_refresh_allowed());

        let path = engine.path.clone();
        let bridge = bridge(&harness, &engine);
        harness
            .connection
            .object_server()
            .at(path.as_str(), engine)
            .await
            .unwrap();
        let (outcome, ()) = bounded(future::zip(bridge.manual_toggle_v3_inner(), async {
            serve_ping_and_marker(&mut harness.peer).await;
            assert!(harness.observer.process_next().await.unwrap());
            loop {
                let message = bounded(next_peer_message(&mut harness.peer)).await;
                let member = message.header().member().unwrap().as_str().to_string();
                assert!(!matches!(
                    member.as_str(),
                    "DeleteSurroundingText" | "CommitText"
                ));
                if member == "RequireSurroundingText" {
                    break;
                }
            }
        }))
        .await;
        assert_eq!(outcome.unwrap(), (4, false));
        let mut current = cycle09_take_registered_engine(&harness, &path).await;
        actual_reset(&mut harness, &mut current, 19_215, false).await;
        assert!(current.layout_gesture.pending_manual_toggle);
        surrounding_receipt(&mut harness, &mut current, "abcde\u{200b}", 3, 3).await;
        assert!(
            current.context_reset_rereceipt.is_some(),
            "same-owner retired publication must remain inert until the exact client receipt"
        );
        assert!(!current.context_reset_rereceipt_exact_manual_handoff_allowed());
        exact_surrounding_receipt(&mut harness, &mut current, "abc").await;
        assert!(current.context_reset_rereceipt_exact_manual_handoff_allowed());
        harness
            .connection
            .object_server()
            .at(path.as_str(), current)
            .await
            .unwrap();
        let (outcome, ()) = bounded(future::zip(bridge.manual_toggle_v3_inner(), async {
            serve_ping_and_marker(&mut harness.peer).await;
            assert!(harness.observer.process_next().await.unwrap());
        }))
        .await;
        assert_eq!(outcome.unwrap(), (3, false));
    }));
}

#[test]
fn firefox_manual_toggle_defers_until_the_client_replies_after_rpc() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine =
            initial_observed_tail_reset(&mut harness, 19_300, &[('a', 30), ('x', 45)]).await;
        engine.set_client_capabilities(1_073_741_865);
        exact_surrounding_receipt(&mut harness, &mut engine, "ax").await;
        engine.config.nanda_precognition = true;
        publish_fixture_append_completion(&mut harness, &mut engine, "cde").await;
        surrounding_receipt(&mut harness, &mut engine, "ax\u{200b}", 2, 2).await;
        assert!(engine.context_reset_rereceipt_manual_refresh_allowed());
        assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        let path = engine.path.clone();
        let first_bridge = bridge(&harness, &engine);
        harness
            .connection
            .object_server()
            .at(path.as_str(), engine)
            .await
            .unwrap();
        let (outcome, ()) = bounded(future::zip(first_bridge.manual_toggle_v3_inner(), async {
            serve_ping_and_marker(&mut harness.peer).await;
            assert!(harness.observer.process_next().await.unwrap());
            let mut members = Vec::new();
            while !members
                .iter()
                .any(|member| member == "RequireSurroundingText")
            {
                let message = bounded(next_peer_message(&mut harness.peer)).await;
                let member = message.header().member().unwrap().as_str().to_string();
                assert!(!matches!(
                    member.as_str(),
                    "DeleteSurroundingText" | "CommitText"
                ));
                members.push(member);
                assert!(members.len() <= 5, "unexpected refresh output: {members:?}");
            }
            assert!(members.iter().any(|member| member == "HidePreeditText"));
        }))
        .await;
        // Firefox can wait for the RPC to finish before reporting the exact
        // text. No Delete/Commit is authorized by the stale preedit above.
        assert_eq!(outcome.unwrap(), (4, false));
        let mut current = cycle09_take_registered_engine(&harness, &path).await;
        assert!(current.layout_gesture.pending_manual_toggle);
        assert_eq!(current.committed_tail.buffer, "ax");
        // Firefox can Reset after HidePreeditText and before echoing the
        // retired visible completion. This is still the same pending gesture.
        actual_reset(&mut harness, &mut current, 19_315, true).await;
        assert!(current.layout_gesture.pending_manual_toggle);
        assert!(current.layout_gesture.pending_manual_refresh_at.is_some());
        let emitter =
            zbus::object_server::SignalEmitter::new(&harness.connection, path.clone()).unwrap();
        current
            .set_surrounding_text(
                emitter.clone(),
                crate::text::make_ibus_text("axcde\u{200b}".to_string()),
                2,
                2,
            )
            .await
            .unwrap();
        assert!(current.layout_gesture.pending_manual_toggle);
        assert_eq!(current.committed_tail.buffer, "ax");
        current
            .set_surrounding_text(emitter, crate::text::make_ibus_text("ax".to_string()), 2, 2)
            .await
            .unwrap();
        assert!(current.layout_gesture.pending_manual_toggle);
        assert_eq!(current.committed_tail.buffer, "ax");
        assert!(current.context_reset_rereceipt_exact_manual_handoff_allowed());
        harness
            .connection
            .object_server()
            .at(path.as_str(), current)
            .await
            .unwrap();
        let members = drain_output_to_proof(&mut harness).await;
        assert!(!members
            .iter()
            .any(|member| member == "DeleteSurroundingText"));
        assert!(!members.iter().any(|member| member == "CommitText"));
        let (outcome, ()) = bounded(future::zip(first_bridge.manual_toggle_v3_inner(), async {
            serve_ping_and_marker(&mut harness.peer).await;
            assert!(harness.observer.process_next().await.unwrap());
        }))
        .await;
        assert_eq!(outcome.unwrap(), (3, false));
        let mut leased_readouts = Vec::new();
        for _ in 0..2 {
            let (reply, ()) = bounded(future::zip(first_bridge.visible_tail_v3_inner(), async {
                serve_ping_and_marker(&mut harness.peer).await;
                assert!(harness.observer.process_next().await.unwrap());
            }))
            .await;
            leased_readouts.push(reply.unwrap());
        }
        assert_eq!(leased_readouts[0], leased_readouts[1]);
        assert_eq!(leased_readouts[0].0, "passive:committed-tail");
        assert_eq!(leased_readouts[0].1, "ax");
        let current = harness
            .connection
            .object_server()
            .interface::<_, LayIbusEngine>(path.as_str())
            .await
            .unwrap();
        let current = current.get().await;
        assert!(!current.layout_gesture.pending_manual_toggle);
        assert_eq!(current.committed_tail.buffer, "ax");
        drop(current);
        assert!(drain_output_to_proof(&mut harness)
            .await
            .iter()
            .all(|member| !matches!(member.as_str(), "DeleteSurroundingText" | "CommitText")));

        // A contradictory client receipt must cancel the same gesture without
        // using the old local tail as text authority.
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine =
            initial_observed_tail_reset(&mut harness, 19_400, &[('a', 30), ('x', 45)]).await;
        engine.set_client_capabilities(1_073_741_865);
        exact_surrounding_receipt(&mut harness, &mut engine, "ax").await;
        engine.config.nanda_precognition = true;
        publish_fixture_append_completion(&mut harness, &mut engine, "cde").await;
        surrounding_receipt(&mut harness, &mut engine, "ax\u{200b}", 2, 2).await;
        assert!(engine.context_reset_rereceipt_manual_refresh_allowed());
        let path = engine.path.clone();
        let bridge = bridge(&harness, &engine);
        harness
            .connection
            .object_server()
            .at(path.as_str(), engine)
            .await
            .unwrap();
        let (outcome, ()) = bounded(future::zip(bridge.manual_toggle_v3_inner(), async {
            serve_ping_and_marker(&mut harness.peer).await;
            assert!(harness.observer.process_next().await.unwrap());
            loop {
                let message = bounded(next_peer_message(&mut harness.peer)).await;
                let member = message.header().member().unwrap().as_str().to_string();
                assert!(!matches!(
                    member.as_str(),
                    "DeleteSurroundingText" | "CommitText"
                ));
                if member == "RequireSurroundingText" {
                    break;
                }
            }
        }))
        .await;
        assert_eq!(outcome.unwrap(), (4, false));
        let mut current = cycle09_take_registered_engine(&harness, &path).await;
        actual_reset(&mut harness, &mut current, 19_415, true).await;
        assert!(current.layout_gesture.pending_manual_toggle);
        current
            .set_surrounding_text(
                zbus::object_server::SignalEmitter::new(&harness.connection, path.clone()).unwrap(),
                crate::text::make_ibus_text("ay".to_string()),
                2,
                2,
            )
            .await
            .unwrap();
        assert!(!current.layout_gesture.pending_manual_toggle);
        assert_eq!(current.committed_tail.buffer, "ax");
        assert!(drain_output_to_proof(&mut harness)
            .await
            .iter()
            .all(|member| !matches!(member.as_str(), "DeleteSurroundingText" | "CommitText")));
    }));
}

#[test]
fn firefox_retired_preedit_survives_next_publication_until_exact_append_receipt() {
    zbus::block_on(bounded(async {
        for (appended, code, next_preedit, surface, anchor, retain) in [
            ('b', 48, "c", "abbc", 2, true),
            ('x', 45, "yz", "axbc", 2, true),
            ('x', 45, "yz", "axbd", 2, false),
            ('x', 45, "yz", "axbc", 1, false),
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = initial_observed_tail_reset(&mut harness, 9_540, &[('a', 30)]).await;
            exact_surrounding_receipt(&mut harness, &mut engine, "a").await;
            engine.record_context_reset_preedit_publication("bc", 0);
            surrounding_receipt(&mut harness, &mut engine, "a\u{200b}", 1, 1).await;
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());

            let input_mode_before_key = engine.layout_gesture.layout_is_ru;
            assert!(legacy_key(&mut harness, &mut engine, 9_543, appended as u32, code, 0).await);
            expect_legacy_commit(&mut harness.peer, &engine, input_mode_before_key).await;
            let committed = format!("a{appended}");
            assert_eq!(engine.committed_tail.buffer, committed);
            engine.record_context_reset_preedit_publication(next_preedit, 0);

            surrounding_receipt(&mut harness, &mut engine, surface, 2, anchor).await;
            assert_eq!(
                engine.context_reset_rereceipt.is_some(),
                retain,
                "new preedit {next_preedit:?}, stale surface {surface:?}"
            );
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert!(engine.capture_observed_suffix_display_frame().is_none());
            exact_surrounding_receipt(&mut harness, &mut engine, &committed).await;
            assert_eq!(
                engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                retain,
                "only a fresh exact appended token may restore authority"
            );
            assert!(
                drain_output_to_proof(&mut harness)
                    .await
                    .iter()
                    .all(|member| !matches!(
                        member.as_str(),
                        "DeleteSurroundingText" | "CommitText"
                    )),
                "a stale publication cannot authorize text mutation"
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
    let reply = bounded(next_peer_message_raw(peer)).await;
    assert_eq!(reply.header().message_type(), Type::Error);
    assert_eq!(reply.header().reply_serial().map(|n| n.get()), Some(serial));
    assert_eq!(
        reply.header().error_name().map(|n| n.as_str()),
        Some("org.freedesktop.DBus.Error.UnknownObject")
    );
    peer.detached_callback_reply_serials.remove(&serial);
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
    send_manually_dispatched_callback(
        &mut harness.peer,
        &set,
        engine.path.as_str(),
        PROPERTIES_INTERFACE,
    )
    .await;
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
        let activation_mode = source.layout_gesture.layout_is_ru;
        assert!(!legacy_key(&mut harness, &mut source, 8_000, 'a' as u32, 30, 0).await);
        if !leading_boundary {
            expect_activation_input_mode_update(&mut harness, &source, activation_mode).await;
        }
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
            send_manually_dispatched_callback(
                &mut harness.peer,
                &factory,
                "/org/freedesktop/IBus/Factory",
                FACTORY_INTERFACE,
            )
            .await;
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
                send_manually_dispatched_callback(
                    &mut harness.peer,
                    &event,
                    &source.path,
                    ENGINE_INTERFACE,
                )
                .await;
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
            send_manually_dispatched_callback(
                &mut harness.peer,
                &focus,
                &target_path,
                ENGINE_INTERFACE,
            )
            .await;
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
            let activation_mode = engine.layout_gesture.layout_is_ru;
            assert!(!legacy_key(&mut harness, &mut engine, 9_000 + index as u32, 0, code, 0).await);
            if index == 0 {
                expect_activation_input_mode_update(&mut harness, &engine, activation_mode).await;
            }
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
            let activation_mode = engine.layout_gesture.layout_is_ru;
            assert!(!legacy_key(&mut harness, &mut engine, 9_100, 'a' as u32, 30, 0).await);
            expect_activation_input_mode_update(&mut harness, &engine, activation_mode).await;
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
            let activation_mode = engine.layout_gesture.layout_is_ru;
            assert!(!legacy_key(&mut harness, &mut engine, 9_400 + index as u32, 0, code, 0).await);
            if index == 0 {
                expect_activation_input_mode_update(&mut harness, &engine, activation_mode).await;
            }
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
fn residual_absent_dispatch_registration_handoff_completes_before_served_callback() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let engine = known_engine(&mut harness).await;
        // First causal oracle: the helper must finish every absent-object
        // dispatch before a caller can register a served engine at this path.
        assert!(
            !harness
                .peer
                .detached_callback_reply_serials
                .contains(&3_001)
                && harness
                    .peer
                    .detached_callback_reply_serials
                    .counts
                    .is_empty(),
            "registration handoff retains unfinished manual callback"
        );
        let owner = engine.context_owner.clone().expect("known fixture owner");
        let token = engine.live_context_token().expect("known fixture token");
        let tail = engine.committed_tail.buffer.clone();
        let epoch = engine.committed_tail.epoch;
        assert!(engine.context_word_is_known());
        let bridge = bridge(&harness, &engine);
        harness
            .connection
            .object_server()
            .at(TARGET_PATH, engine)
            .await
            .unwrap();
        let interface = harness
            .connection
            .object_server()
            .interface::<_, LayIbusEngine>(TARGET_PATH)
            .await
            .unwrap();
        let mut held = interface.get_mut().await;
        // One actual wire packet has two deliberately controlled invocations:
        // the manual helper while this lock is held, then the real registered
        // dispatcher after it is released. No second ingress is fabricated.
        let serial = 58_000;
        let expected_wire = legacy_key_message_at(serial, TARGET_PATH, KEY_SPACE, 57, RELEASE_MASK);
        let header_key =
            HeaderKey::from_zbus_header(ConnectionGeneration(60), &expected_wire.header()).unwrap();
        assert!(!legacy_key(&mut harness, &mut held, serial, KEY_SPACE, 57, RELEASE_MASK,).await);
        assert!(held.context_word_is_known());
        assert_eq!(held.live_context_token().as_ref(), Some(&token));
        assert_eq!(held.context_owner.as_ref(), Some(&owner));
        assert_eq!(held.committed_tail.buffer, tail);
        assert_eq!(held.committed_tail.epoch, epoch);
        let observed = harness
            .adapter
            .observe_callback(&expected_wire.header(), Instant::now())
            .await
            .unwrap();
        assert_eq!(observed.header, header_key);
        assert!(
            matches!(observed.disposition, IngressDisposition::Key { owner: ref actual, .. } if actual == &owner)
        );
        assert!(harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .unsettled
            .is_empty());
        assert_eq!(
            harness
                .peer
                .detached_callback_reply_serials
                .counts
                .get(&serial),
            Some(&1),
            "one wire emission, not a second duplicated send"
        );
        // Cached stamps are cloned, not consumed. The second callback can
        // obtain this same stamp, but cannot settle its already settled header.
        let ping = standard_dispatcher_ping();
        let ping_serial = ping.primary_header().serial_num();
        assert_ne!(ping_serial.get(), serial);
        harness.peer.connection.send(&ping).await.unwrap();
        drop(held);
        // The production Engine interface is spawn=false. Its queued key
        // finishes before this next Peer call is dispatched. Raw reading must
        // reject any unexpected signal/error/normal key reply, not filter it.
        let reply = bounded(next_peer_message_raw(&mut harness.peer)).await;
        assert_standard_dispatcher_ping_reply(&reply, ping_serial);
        let engine = interface.get().await;
        assert_eq!(engine.context_owner.as_ref(), Some(&owner));
        assert_eq!(engine.committed_tail.buffer, tail);
        assert_eq!(engine.committed_tail.epoch, epoch);
        assert!(!engine.context_word_is_known());
        assert_eq!(
            engine
                .context_word_scope
                .as_ref()
                .unwrap()
                .lineage()
                .completeness,
            WordCompleteness::UnknownStart,
        );
        assert!(!harness.adapter.revalidate(&token));
        assert!(!engine.exact_manual_toggle_handoff_is_live());
        drop(engine);
        // legacy_key declared this one packet as absent for its usual manual
        // fixtures. Here registration preceded emission; the raw Ping and
        // state assertions proved its served completion, not UnknownObject.
        // Reclassify only this intentional mixed packet, after that proof.
        assert!(harness.peer.detached_callback_reply_serials.remove(&serial));
        assert!(harness
            .peer
            .detached_callback_reply_serials
            .counts
            .is_empty());
        let (outcome, ()) = bounded(future::zip(bridge.manual_toggle_v3_inner(), async {
            serve_ping_and_marker(&mut harness.peer).await;
            assert!(harness.observer.process_next().await.unwrap());
        }))
        .await;
        match outcome {
            Ok(outcome) => assert_eq!(outcome, (0, false)),
            Err(error) => assert!(error.to_string().ends_with("context admission denied")),
        }
        assert!(drain_output_to_proof(&mut harness).await.is_empty());
        assert!(!harness.adapter.revalidate(&token));
    }));
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
    let activation_mode = engine.layout_gesture.layout_is_ru;
    assert!(!legacy_key(&mut harness, &mut engine, 3_033, KEY_LEFT_SHIFT, 42, 0).await);
    expect_activation_input_mode_update(&mut harness, &engine, activation_mode).await;
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
        let input_mode_before_key = engine.layout_gesture.layout_is_ru;
        let native_space_was_visible = engine.composition.preedit_visible;
        assert!(!legacy_key(&mut harness, &mut engine, 3_034, KEY_SPACE, 57, 0).await);
        expect_legacy_native_space(
            &mut harness,
            &engine,
            input_mode_before_key,
            native_space_was_visible,
        )
        .await;
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
            .with_flags(zbus::message::Flags::NoReplyExpected)
            .unwrap()
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
    send_manually_dispatched_callback(
        &mut harness.peer,
        &message,
        TARGET_PATH,
        if content_type {
            PROPERTIES_INTERFACE
        } else {
            ENGINE_INTERFACE
        },
    )
    .await;
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
            send_manually_dispatched_callback(
                &mut harness.peer,
                &reset,
                TARGET_PATH,
                ENGINE_INTERFACE,
            )
            .await;
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
        let activation_mode = engine.layout_gesture.layout_is_ru;
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
                expect_activation_input_mode_update(&mut harness, &engine, activation_mode).await;
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

        let input_mode_before_key = engine.layout_gesture.layout_is_ru;
        let native_space_was_visible = engine.composition.preedit_visible;
        assert!(!legacy_key(&mut harness, &mut engine, 3_500, KEY_SPACE, 57, 0).await);
        expect_legacy_native_space(
            &mut harness,
            &engine,
            input_mode_before_key,
            native_space_was_visible,
        )
        .await;
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
            send_manually_dispatched_callback(
                &mut harness.peer,
                &reset,
                TARGET_PATH,
                ENGINE_INTERFACE,
            )
            .await;
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
        let input_mode_before_key = engine.layout_gesture.layout_is_ru;
        let native_space_was_visible = engine.composition.preedit_visible;
        assert!(!legacy_key(&mut harness, &mut engine, 3_603, KEY_SPACE, 57, 0).await);
        expect_legacy_native_space(
            &mut harness,
            &engine,
            input_mode_before_key,
            native_space_was_visible,
        )
        .await;
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
        send_manually_dispatched_callback(
            &mut harness.peer,
            &focus_out,
            TARGET_PATH,
            ENGINE_INTERFACE,
        )
        .await;
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
            let input_mode_before_key = engine.layout_gesture.layout_is_ru;
            assert!(legacy_key(&mut harness, &mut engine, 3_050, u32::from(b'l'), 38, 0).await);
            expect_legacy_commit(&mut harness.peer, &engine, input_mode_before_key).await;
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
            let input_mode_before_key = engine.layout_gesture.layout_is_ru;
            let native_space_was_visible = engine.composition.preedit_visible;
            assert!(!legacy_key(&mut harness, &mut engine, 3_054, KEY_SPACE, 57, 0).await);
            expect_legacy_native_space(
                &mut harness,
                &engine,
                input_mode_before_key,
                native_space_was_visible,
            )
            .await;
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
    space_committed: bool,
) {
    for ch in text.chars() {
        let keycode = match ch {
            'a' => 30,
            'b' => 48,
            'c' => 46,
            ' ' => 57,
            _ => unreachable!("fixture key"),
        };
        let input_mode_before_key = engine.layout_gesture.layout_is_ru;
        if ch == ' ' {
            // Callers declare ordinary versus opaque transport independently
            // of the production predicate. Both retain their authority oracle.
            let native_space_was_visible = engine.composition.preedit_visible;
            assert_eq!(
                legacy_key(harness, engine, *serial, ch as u32, keycode, 0).await,
                space_committed
            );
            *serial += 1;
            if space_committed {
                expect_legacy_managed_space(
                    harness,
                    engine,
                    input_mode_before_key,
                    native_space_was_visible,
                )
                .await;
            } else {
                expect_legacy_native_space(
                    harness,
                    engine,
                    input_mode_before_key,
                    native_space_was_visible,
                )
                .await;
            }
            assert_eq!(
                legacy_key(harness, engine, *serial, ch as u32, keycode, RELEASE_MASK).await,
                space_committed
            );
            *serial += 1;
            continue;
        }
        assert!(legacy_key(harness, engine, *serial, ch as u32, keycode, 0).await);
        *serial += 1;
        let commit =
            next_legacy_text_effect(&mut harness.peer, engine, input_mode_before_key).await;
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
    retained_boundary_literal_keys(harness, &mut engine, serial, "abc", false).await;
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
fn no_surrounding_browser_accepts_only_its_live_owned_preedit_completion() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_client_capabilities(9); // PREEDIT_TEXT | FOCUS, no surrounding text
        engine.config.nanda_precognition = false;
        engine.config.ime_bracket_candidates = false;

        assert!(legacy_key(&mut harness, &mut engine, 12_876, 'a' as u32, 30, 0).await);
        let initial = drain_output_to_proof(&mut harness).await;
        assert!(initial.iter().any(|member| member == "UpdatePreeditText"));
        assert!(!initial.iter().any(|member| member == "CommitText"));
        assert!(engine.composition.legacy_word_preedit_active);
        assert_eq!(engine.composition.buffer, "a");
        assert!(!engine.context_word_is_known());
        assert!(engine.live_context_token().is_some());

        set_fixture_append_completion(&mut engine, "bc");
        let emitter =
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap();
        engine
            .publish_selected_precognition_candidate(&mut crate::output::EngineOutput::legacy(
                &emitter,
            ))
            .await
            .unwrap();
        let published = bounded(next_peer_message(&mut harness.peer)).await;
        assert_eq!(
            published.header().member().unwrap().as_str(),
            "UpdatePreeditText"
        );
        let body = published.body();
        let (text, _, visible, _) = body
            .deserialize::<(zbus::zvariant::Value<'_>, u32, bool, u32)>()
            .unwrap();
        assert_eq!(
            crate::ibus_interface::ibus_text_value_to_string(&text).as_deref(),
            Some("abc")
        );
        assert!(visible);

        assert!(legacy_key(&mut harness, &mut engine, 12_877, KEY_TAB, 15, 0).await);
        td121_expect_legacy_commit_text(&mut harness.peer, "abc ").await;
        assert_eq!(engine.committed_tail.buffer, "abc ");
        assert!(engine.composition.buffer.is_empty());
    }));
}

#[test]
fn surrounding_without_refresh_accepts_live_owned_preedit_completion() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_client_capabilities(41);
        engine.observe_external_surrounding_text(Some(SurroundingTextSnapshot::new(
            String::new(),
            0,
            0,
        )));
        engine.config.nanda_precognition = false;
        engine.config.ime_bracket_candidates = false;

        assert!(legacy_key(&mut harness, &mut engine, 12_877, 'a' as u32, 30, 0).await);
        let initial = drain_output_to_proof(&mut harness).await;
        assert!(initial.iter().any(|member| member == "UpdatePreeditText"));
        assert!(!initial.iter().any(|member| member == "CommitText"));
        assert!(engine.composition.legacy_word_preedit_active);
        set_fixture_append_completion(&mut engine, "bc");
        let emitter =
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap();
        engine
            .publish_selected_precognition_candidate(&mut EngineOutput::legacy(&emitter))
            .await
            .unwrap();
        let _ = bounded(next_peer_message(&mut harness.peer)).await;

        assert!(engine.context_owns_active_preedit_append_completion());
        assert!(legacy_key(&mut harness, &mut engine, 12_878, KEY_TAB, 15, 0).await);
        td121_expect_legacy_commit_text(&mut harness.peer, "abc ").await;
        assert_eq!(engine.committed_tail.buffer, "abc ");
    }));
}

async fn c06_exact_refresh_owned_second_word(harness: &mut Harness, serial: u32) -> LayIbusEngine {
    let mut engine =
        initial_observed_tail_reset(harness, serial, &[('a', 30), ('b', 48), ('c', 46)]).await;
    engine.set_client_capabilities(1_073_741_865);
    engine.config.ime_bracket_candidates = false;
    exact_surrounding_receipt(harness, &mut engine, "abc").await;
    let native_space_was_visible = engine.composition.preedit_visible;
    let native_space_mode = engine.layout_gesture.layout_is_ru;
    assert!(legacy_key(harness, &mut engine, serial + 10, KEY_SPACE, 57, 0).await);
    expect_legacy_managed_space(
        harness,
        &engine,
        native_space_mode,
        native_space_was_visible,
    )
    .await;
    // Model Reset as the next callback after that local Space, not after
    // wall time spent by the controlled peer serving its output.
    engine.committed_tail.last_input_at = Some(Instant::now());
    actual_reset(harness, &mut engine, serial + 11, false).await;
    drain_output_to_proof(harness).await;
    exact_surrounding_receipt(harness, &mut engine, "abc ").await;
    assert!(!engine.context_word_is_known());
    assert_eq!(engine.committed_tail.buffer, "abc ");

    assert!(legacy_key(harness, &mut engine, serial + 12, 'a' as u32, 30, 0).await);
    let effects = drain_output_to_proof(harness).await;
    assert!(effects.iter().any(|member| member == "UpdatePreeditText"));
    assert!(!effects.iter().any(|member| member == "CommitText"));
    assert_eq!(engine.composition.buffer, "a");
    assert_eq!(engine.committed_tail.buffer, "abc a");
    assert!(engine.composition.legacy_word_preedit_active);
    assert!(engine.composition.legacy_preedit_start_boundary.is_some());
    assert!(engine.client_context.surrounding_text_snapshot.is_none());
    assert!(engine.client_context.managed_word_start.is_none());
    assert!(engine.live_context_token().is_some());
    assert!(!engine.context_word_is_known());

    // The candidate is controlled; publication and Tab use the actual legacy
    // callback/output transport and existing backend verifier.
    set_fixture_append_completion(&mut engine, "bc");
    let emitter =
        zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone()).unwrap();
    engine
        .publish_selected_precognition_candidate(&mut EngineOutput::legacy(&emitter))
        .await
        .unwrap();
    let effects = super::terminal_delivery::legacy_effects(harness).await;
    let updates = effects
        .iter()
        .filter(|effect| effect.header().member().unwrap().as_str() == "UpdatePreeditText")
        .collect::<Vec<_>>();
    assert_eq!(updates.len(), 1);
    let body = updates[0].body();
    let (text, cursor, visible, _) = body
        .deserialize::<(zbus::zvariant::Value<'_>, u32, bool, u32)>()
        .unwrap();
    assert_eq!(
        crate::ibus_interface::ibus_text_value_to_string(&text).as_deref(),
        Some("abc")
    );
    assert_eq!((cursor, visible), (1, true));
    assert_eq!(
        crate::text::preedit_attribute_geometry(&text),
        [(1, 0, 0, 1), (2, 0x888888, 1, 3), (1, 1, 1, 3)],
        "the actual legacy signal must leave its owned typed prefix neutral"
    );
    assert!(effects.iter().all(|effect| !matches!(
        effect.header().member().unwrap().as_str(),
        "CommitText" | "DeleteSurroundingText"
    )));
    engine
}

#[test]
fn c06_exact_refresh_observed_second_word_accepts_live_owned_append_once() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = c06_exact_refresh_owned_second_word(&mut harness, 28_100).await;
        // This grant remains specific to explicit append acceptance.
        assert!(!engine.context_owns_active_preedit_manual_toggle());

        assert!(legacy_key(&mut harness, &mut engine, 28_120, KEY_TAB, 15, 0).await);
        let effects = super::terminal_delivery::legacy_effects(&mut harness).await;
        let commits = effects
            .iter()
            .filter(|effect| effect.header().member().unwrap().as_str() == "CommitText")
            .map(|effect| {
                let body = effect.body();
                let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
                crate::ibus_interface::ibus_text_value_to_string(&value).unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(commits, ["abc "]);
        assert!(effects.iter().all(|effect| {
            effect.header().member().unwrap().as_str() != "DeleteSurroundingText"
        }));
        assert_eq!(engine.committed_tail.buffer, "abc abc ");
        assert!(engine.composition.buffer.is_empty());
        assert!(engine.composition.legacy_preedit_start_boundary.is_none());
        // Append acceptance does not promote the preexisting UnknownStart
        // lineage into completion-learning authority during this edit.
        assert!(engine.committed_tail.pending_completion_learning.is_none());
        assert!(!legacy_key(&mut harness, &mut engine, 28_121, KEY_TAB, 15, 0).await);
        let after = drain_output_to_proof(&mut harness).await;
        assert!(after
            .iter()
            .all(|member| !matches!(member.as_str(), "CommitText" | "DeleteSurroundingText")));
        assert_eq!(engine.committed_tail.buffer, "abc abc ");
    }));
}

#[test]
fn c06_exact_refresh_owned_append_refuses_unproved_or_revoked_start() {
    for gap in [
        "missing_boundary",
        "stale_token",
        "focus",
        "owner",
        "layout",
        "tail_epoch",
        "tail_prefix",
        "observation_revision",
        "selection",
        "reset",
        "replacement",
        "dirty_hint",
        "display_only",
        "caret",
        "sensitive",
    ] {
        zbus::block_on(bounded(async {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = c06_exact_refresh_owned_second_word(&mut harness, 28_200).await;
            match gap {
                "missing_boundary" => engine.composition.legacy_preedit_start_boundary = None,
                "stale_token" => engine.context_token = None,
                "focus" => engine.client_context.focus_serial += 1,
                "owner" => engine.client_context.runtime_owner_lease_identity += 1,
                "layout" => engine.layout_gesture.layout_generation += 1,
                "tail_epoch" => engine.committed_tail.epoch += 1,
                "tail_prefix" => engine.committed_tail.buffer = "xyz a".into(),
                "observation_revision" => {
                    exact_surrounding_receipt(&mut harness, &mut engine, "abc ").await;
                }
                "selection" => {
                    surrounding_receipt(&mut harness, &mut engine, "abc ", 4, 0).await;
                }
                "reset" => {
                    actual_reset(&mut harness, &mut engine, 28_220, false).await;
                    drain_output_to_proof(&mut harness).await;
                }
                "replacement" => {
                    engine.composition.preedit_replacement_targets[0] = Some("xyz".into());
                }
                "dirty_hint" => engine.composition.preedit_dirty = true,
                "display_only" => engine.composition.preedit_display_only_pending = true,
                "caret" => engine.composition.cursor = 0,
                "sensitive" => {
                    engine.client_context.content_purpose =
                        crate::window_interaction::IBUS_INPUT_PURPOSE_PASSWORD;
                }
                _ => unreachable!(),
            }
            assert!(
                !engine.context_owns_active_preedit_append_completion(),
                "{gap}"
            );
            let tail_before_tab = engine.committed_tail.buffer.clone();
            assert!(
                !legacy_key(&mut harness, &mut engine, 28_221, KEY_TAB, 15, 0).await,
                "{gap}"
            );
            let effects = drain_output_to_proof(&mut harness).await;
            assert!(
                effects.iter().all(|member| !matches!(
                    member.as_str(),
                    "CommitText" | "DeleteSurroundingText"
                )),
                "{gap}: {effects:?}"
            );
            assert_eq!(engine.committed_tail.buffer, tail_before_tab, "{gap}");
        }));
    }
}

#[test]
fn c06_exact_refresh_capability_alone_cannot_admit_an_unproved_owned_start() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.config.nanda_precognition = false;
        engine.set_client_capabilities(9);
        assert!(legacy_key(&mut harness, &mut engine, 28_300, 'a' as u32, 30, 0).await);
        drain_output_to_proof(&mut harness).await;
        assert!(engine.composition.legacy_word_preedit_active);
        // A later capability notification supplies neither a field boundary
        // nor authority to replace the already displayed word.
        engine.set_client_capabilities(1_073_741_865);
        set_fixture_append_completion(&mut engine, "bc");
        assert!(engine.composition.legacy_preedit_start_boundary.is_none());
        assert!(!engine.context_owns_active_preedit_append_completion());
        assert!(!legacy_key(&mut harness, &mut engine, 28_301, KEY_TAB, 15, 0).await);
        let effects = drain_output_to_proof(&mut harness).await;
        assert!(effects
            .iter()
            .all(|member| !matches!(member.as_str(), "CommitText" | "DeleteSurroundingText")));
        assert_eq!(engine.composition.buffer, "a");
    }));
}

#[test]
fn c06_owned_backspace_retype_accepts_live_append_without_client_edit() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = c06_exact_refresh_owned_second_word(&mut harness, 28_400).await;
        assert!(legacy_key(&mut harness, &mut engine, 28_420, 'b' as u32, 48, 0).await);
        drain_output_to_proof(&mut harness).await;
        let before_epoch = engine.committed_tail.epoch;
        assert!(legacy_key(&mut harness, &mut engine, 28_421, KEY_BACKSPACE, 14, 0).await);
        let edits = drain_output_to_proof(&mut harness).await;
        assert!(edits
            .iter()
            .all(|member| !matches!(member.as_str(), "CommitText" | "DeleteSurroundingText")));
        assert_eq!(engine.composition.buffer, "a");
        assert_eq!(engine.committed_tail.buffer, "abc a");
        assert_eq!(engine.committed_tail.epoch, before_epoch.wrapping_add(1));
        assert!(legacy_key(&mut harness, &mut engine, 28_422, 'b' as u32, 48, 0).await);
        drain_output_to_proof(&mut harness).await;
        set_fixture_append_completion(&mut engine, "c");
        let emitter =
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap();
        engine
            .publish_selected_precognition_candidate(&mut EngineOutput::legacy(&emitter))
            .await
            .unwrap();
        let visible = super::terminal_delivery::legacy_effects(&mut harness).await;
        assert!(visible
            .iter()
            .any(|message| message.header().member().unwrap().as_str() == "UpdatePreeditText"));
        // Exercise the real accept callback; a missing local-edit receipt must
        // fail here, even if the same append hint has been displayed.
        assert!(legacy_key(&mut harness, &mut engine, 28_423, KEY_TAB, 15, 0).await);
        let effects = super::terminal_delivery::legacy_effects(&mut harness).await;
        let commits = effects
            .iter()
            .filter(|message| message.header().member().unwrap().as_str() == "CommitText")
            .map(|message| {
                let body = message.body();
                let text = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
                crate::ibus_interface::ibus_text_value_to_string(&text).unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(commits, ["abc "]);
        assert!(effects
            .iter()
            .all(|message| message.header().member().unwrap().as_str() != "DeleteSurroundingText"));
        assert_eq!(engine.committed_tail.buffer, "abc abc ");
        assert!(engine.composition.buffer.is_empty());
        assert!(engine.composition.legacy_preedit_start_boundary.is_none());
        assert!(engine.committed_tail.pending_completion_learning.is_none());
    }));
}

#[test]
fn c06_owned_backspace_cannot_repair_external_epoch_or_revoked_start() {
    for gap in [
        "external_epoch",
        "missing_boundary",
        "owner",
        "layout",
        "surrounding_revision",
        "selection",
    ] {
        zbus::block_on(bounded(async {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = c06_exact_refresh_owned_second_word(&mut harness, 28_500).await;
            assert!(legacy_key(&mut harness, &mut engine, 28_520, 'b' as u32, 48, 0).await);
            drain_output_to_proof(&mut harness).await;
            match gap {
                "external_epoch" => engine.committed_tail.epoch += 1,
                "missing_boundary" => engine.composition.legacy_preedit_start_boundary = None,
                "owner" => engine.client_context.runtime_owner_lease_identity += 1,
                "layout" => engine.layout_gesture.layout_generation += 1,
                "surrounding_revision" => {
                    exact_surrounding_receipt(&mut harness, &mut engine, "abc ").await
                }
                "selection" => surrounding_receipt(&mut harness, &mut engine, "abc ", 4, 0).await,
                _ => unreachable!(),
            }
            let _ = legacy_key(&mut harness, &mut engine, 28_521, KEY_BACKSPACE, 14, 0).await;
            let edits = drain_output_to_proof(&mut harness).await;
            assert!(
                edits.iter().all(|member| !matches!(
                    member.as_str(),
                    "CommitText" | "DeleteSurroundingText"
                )),
                "{gap}: {edits:?}"
            );
            assert!(
                engine.composition.legacy_preedit_start_boundary.is_none(),
                "{gap}"
            );
            let _ = legacy_key(&mut harness, &mut engine, 28_522, 'b' as u32, 48, 0).await;
            drain_output_to_proof(&mut harness).await;
            set_fixture_append_completion(&mut engine, "c");
            assert!(
                !engine.context_owns_active_preedit_append_completion(),
                "{gap}"
            );
            assert!(
                !legacy_key(&mut harness, &mut engine, 28_523, KEY_TAB, 15, 0).await,
                "{gap}"
            );
            let effects = drain_output_to_proof(&mut harness).await;
            assert!(
                effects.iter().all(|member| !matches!(
                    member.as_str(),
                    "CommitText" | "DeleteSurroundingText"
                )),
                "{gap}: {effects:?}"
            );
        }));
    }
}

#[test]
fn surrounding_without_refresh_owned_preedit_rejects_stale_token_and_selection() {
    zbus::block_on(bounded(async {
        for stale_token in [false, true] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = new_engine(&harness);
            start_source_free_unknown(&mut harness, &mut engine).await;
            engine.set_client_capabilities(41);
            engine.observe_external_surrounding_text(Some(SurroundingTextSnapshot::new(
                String::new(),
                0,
                0,
            )));
            engine.config.nanda_precognition = false;
            assert!(legacy_key(&mut harness, &mut engine, 12_879, 'a' as u32, 30, 0).await);
            drain_output_to_proof(&mut harness).await;
            assert!(engine.composition.legacy_word_preedit_active);
            if stale_token {
                engine.context_token = None;
            } else {
                engine.observe_external_surrounding_text(Some(SurroundingTextSnapshot::new(
                    "selected".to_string(),
                    8,
                    0,
                )));
            }
            set_fixture_append_completion(&mut engine, "bc");
            assert!(!engine.context_owns_active_preedit_append_completion());
            assert!(!engine.context_owns_active_preedit_manual_toggle());
        }
    }));
}

#[test]
fn no_surrounding_owned_preedit_cannot_accept_after_context_token_loss() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_client_capabilities(9);
        engine.config.nanda_precognition = false;
        assert!(legacy_key(&mut harness, &mut engine, 12_878, 'a' as u32, 30, 0).await);
        drain_output_to_proof(&mut harness).await;
        assert!(engine.composition.legacy_word_preedit_active);
        assert!(engine.live_context_token().is_some());
        set_fixture_append_completion(&mut engine, "bc");
        engine.context_token = None;

        assert!(!engine.context_owns_active_preedit_append_completion());
        assert!(!legacy_key(&mut harness, &mut engine, 12_879, KEY_TAB, 15, 0).await);
        let effects = drain_output_to_proof(&mut harness).await;
        assert!(!effects
            .iter()
            .any(|member| member == "CommitText" || member == "DeleteSurroundingText"));
    }));
}

#[test]
fn no_surrounding_browser_manual_toggle_rejects_stale_preedit_owner() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_client_capabilities(9);
        engine.config.nanda_precognition = false;
        assert!(legacy_key(&mut harness, &mut engine, 12_880, 'a' as u32, 30, 0).await);
        drain_output_to_proof(&mut harness).await;
        assert!(engine.composition.preedit_visible);
        engine.context_token = None;

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
        assert_eq!(engine.composition.buffer, "a");
    }));
}

#[test]
fn no_surrounding_owned_preedit_publishes_each_of_eight_modes_without_committing() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_client_capabilities(9);
        engine.config.nanda_precognition = false;
        assert!(legacy_key(&mut harness, &mut engine, 12_882, 'a' as u32, 30, 0).await);
        drain_output_to_proof(&mut harness).await;
        assert_eq!(engine.composition.buffer, "a");

        for gesture in 0..8 {
            let expected_ru = gesture % 2 == 0;
            let expected = if expected_ru { "ф" } else { "a" };
            let mut effects = crate::output::TestEngineOutput {
                legacy_transport: true,
                ..Default::default()
            };
            let target = engine
                .manual_toggle_active_text_target(&mut EngineOutput::test(&mut effects))
                .await
                .expect("exact owned preedit gesture");
            assert_eq!(target, Some(expected_ru), "gesture {}", gesture + 1);
            assert_eq!(
                effects.input_mode_updates,
                [expected_ru],
                "gesture {}",
                gesture + 1
            );
            assert_eq!(
                effects
                    .preedit_updates
                    .last()
                    .map(|update| update.0.as_str()),
                Some(expected),
                "gesture {}",
                gesture + 1
            );
            assert!(effects.committed_texts.is_empty());
            assert!(effects.surrounding_deletes.is_empty());
            assert_eq!(engine.composition.buffer, expected);
            assert_eq!(engine.committed_tail.buffer, expected);
            assert_eq!(engine.layout_gesture.layout_is_ru, expected_ru);
        }

        let mut failed = crate::output::TestEngineOutput {
            legacy_transport: true,
            fail_input_mode_publication: true,
            ..Default::default()
        };
        assert!(engine
            .manual_toggle_active_text_target(&mut EngineOutput::test(&mut failed))
            .await
            .is_err());
        assert!(failed.input_mode_updates.is_empty());
        assert_eq!(engine.composition.buffer, "a");
        assert_eq!(engine.committed_tail.buffer, "a");
        assert!(!engine.layout_gesture.layout_is_ru);
    }));
}

#[test]
fn surrounding_without_refresh_owned_preedit_publishes_eight_modes_without_committing() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_client_capabilities(41);
        engine.observe_external_surrounding_text(Some(SurroundingTextSnapshot::new(
            String::new(),
            0,
            0,
        )));
        engine.config.nanda_precognition = false;
        assert!(legacy_key(&mut harness, &mut engine, 12_882, 'a' as u32, 30, 0).await);
        drain_output_to_proof(&mut harness).await;
        assert_eq!(engine.composition.buffer, "a");

        for gesture in 0..8 {
            let expected_ru = gesture % 2 == 0;
            let expected = if expected_ru { "ф" } else { "a" };
            let mut effects = crate::output::TestEngineOutput {
                legacy_transport: true,
                ..Default::default()
            };
            let target = engine
                .manual_toggle_active_text_target(&mut EngineOutput::test(&mut effects))
                .await
                .expect("exact owned preedit gesture");
            assert_eq!(target, Some(expected_ru), "gesture {}", gesture + 1);
            assert_eq!(effects.input_mode_updates, [expected_ru]);
            assert_eq!(
                effects
                    .preedit_updates
                    .last()
                    .map(|update| update.0.as_str()),
                Some(expected)
            );
            assert!(effects.committed_texts.is_empty());
            assert!(effects.surrounding_deletes.is_empty());
            assert_eq!(engine.composition.buffer, expected);
            assert_eq!(engine.committed_tail.buffer, expected);
            assert_eq!(engine.layout_gesture.layout_is_ru, expected_ru);
        }
    }));
}

#[test]
fn no_surrounding_owned_preedit_emits_native_ibus_input_mode_signal() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_client_capabilities(9);
        engine.config.nanda_precognition = false;
        assert!(legacy_key(&mut harness, &mut engine, 12_883, 'a' as u32, 30, 0).await);
        drain_output_to_proof(&mut harness).await;
        let emitter =
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap();
        // Initial registration remains a property list; neither physical
        // gesture may re-register that list on the same engine transport.
        LayIbusEngine::register_properties(
            &emitter,
            crate::text::make_ibus_input_mode_properties(false),
        )
        .await
        .unwrap();
        let registration = bounded(next_peer_message(&mut harness.peer)).await;
        assert_eq!(
            registration.header().member().unwrap().as_str(),
            "RegisterProperties"
        );
        let body = registration.body();
        let (value,): (zbus::zvariant::Value<'_>,) = body.deserialize().unwrap();
        assert_eq!(value.value_signature().to_string(), "(sa{sv}av)");

        for (expected_ru, expected_text, expected_symbol) in [(true, "ф", "RU"), (false, "a", "EN")]
        {
            let result = engine
                .manual_toggle_active_text_target(&mut EngineOutput::legacy(&emitter))
                .await
                .unwrap();
            assert_eq!(result, Some(expected_ru));
            let preedit = bounded(next_peer_message(&mut harness.peer)).await;
            assert_eq!(
                preedit.header().member().unwrap().as_str(),
                "UpdatePreeditText"
            );
            let body = preedit.body();
            let (text, cursor, visible, _mode): (zbus::zvariant::Value<'_>, u32, bool, u32) =
                body.deserialize().unwrap();
            assert_eq!(
                crate::ibus_interface::ibus_text_value_to_string(&text).as_deref(),
                Some(expected_text)
            );
            assert_eq!(cursor, 1);
            assert!(visible);

            let property = bounded(next_peer_message(&mut harness.peer)).await;
            let header = property.header();
            assert_eq!(header.member().unwrap().as_str(), "UpdateProperty");
            assert_eq!(
                header.interface().unwrap().as_str(),
                "org.freedesktop.IBus.Engine"
            );
            assert_eq!(header.path().unwrap().as_str(), engine.path);
            let body = property.body();
            let (value,): (zbus::zvariant::Value<'_>,) = body.deserialize().unwrap();
            let zbus::zvariant::Value::Structure(property) = value else {
                panic!("UpdateProperty must carry a single IBusProperty");
            };
            let fields = property.fields();
            assert_eq!(fields.len(), 12);
            assert!(
                matches!(&fields[0], zbus::zvariant::Value::Str(name) if name.as_str() == "IBusProperty")
            );
            assert!(
                matches!(&fields[2], zbus::zvariant::Value::Str(key) if key.as_str() == "InputMode")
            );
            let zbus::zvariant::Value::Value(symbol) = &fields[11] else {
                panic!("InputMode symbol must be an IBusText variant");
            };
            assert_eq!(
                crate::ibus_interface::ibus_text_value_to_string(symbol.as_ref()).as_deref(),
                Some(expected_symbol)
            );
            assert_eq!(engine.composition.buffer, expected_text);
            assert_eq!(engine.committed_tail.buffer, expected_text);
            assert_eq!(engine.layout_gesture.layout_is_ru, expected_ru);
            assert!(drain_output_to_proof(&mut harness).await.is_empty());
        }
    }));
}

#[test]
fn no_surrounding_owned_preedit_next_key_uses_published_ru_and_en_modes() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_client_capabilities(9);
        engine.config.nanda_precognition = false;
        assert!(legacy_key(&mut harness, &mut engine, 12_884, 'a' as u32, 30, 0).await);
        drain_output_to_proof(&mut harness).await;

        let mut ru = crate::output::TestEngineOutput {
            legacy_transport: true,
            ..Default::default()
        };
        assert_eq!(
            engine
                .manual_toggle_active_text_target(&mut EngineOutput::test(&mut ru))
                .await
                .unwrap(),
            Some(true)
        );
        assert_eq!(ru.input_mode_updates, [true]);
        assert!(legacy_key(&mut harness, &mut engine, 12_885, 'b' as u32, 48, 0).await);
        drain_output_to_proof(&mut harness).await;
        assert_eq!(engine.composition.buffer, "фи");

        let mut en = crate::output::TestEngineOutput {
            legacy_transport: true,
            ..Default::default()
        };
        assert_eq!(
            engine
                .manual_toggle_active_text_target(&mut EngineOutput::test(&mut en))
                .await
                .unwrap(),
            Some(false)
        );
        assert_eq!(en.input_mode_updates, [false]);
        assert_eq!(engine.composition.buffer, "ab");
        assert!(legacy_key(&mut harness, &mut engine, 12_886, 'c' as u32, 46, 0).await);
        drain_output_to_proof(&mut harness).await;
        assert_eq!(engine.composition.buffer, "abc");
    }));
}

#[test]
fn late_surrounding_capability_keeps_newly_published_native_completion_for_tab() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.config.auto_replace = false;
        engine.config.nanda_precognition = false;
        engine.client_context.content_purpose = 10;
        engine.client_context.cursor_cell_width = 11;

        let activation_mode = engine.layout_gesture.layout_is_ru;
        assert!(!legacy_key(&mut harness, &mut engine, 12_880, 'a' as u32, 30, 0).await);
        expect_activation_input_mode_update(&mut harness, &engine, activation_mode).await;
        super::terminal_delivery::no_legacy_text_output(&mut harness).await;
        assert_eq!(engine.committed_tail.buffer, "a");
        assert!(!engine.composition.preedit_visible);
        assert_eq!(
            engine.composition.word_input_mode,
            Some(WordInputMode::TerminalPassthrough)
        );

        // Qt announces the widget capabilities only after the first native
        // key. The new visible candidate is published after that transition.
        engine.set_client_capabilities(41);
        assert_eq!(
            engine.composition.word_input_mode,
            Some(WordInputMode::ManagedCommit)
        );
        exact_surrounding_receipt(&mut harness, &mut engine, "a").await;
        assert!(engine.context_observed_suffix_is_current());
        publish_fixture_append_completion(&mut harness, &mut engine, "bc").await;
        assert!(engine.composition.preedit_visible);

        assert!(legacy_key(&mut harness, &mut engine, 12_881, KEY_TAB, 15, 0).await);
        td121_expect_legacy_commit_text(&mut harness.peer, "bc ").await;
        assert_eq!(engine.committed_tail.buffer, "abc ");
    }));
}

#[test]
fn unhandled_command_selection_deletion_starts_a_fresh_observed_word() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.config.auto_replace = false;
        engine.config.nanda_precognition = false;
        engine.client_context.surrounding_text_supported = true;
        engine.client_context.content_purpose = 0;
        engine.client_context.cursor_cell_width = 0;

        for (i, (ch, code)) in [('a', 30), ('b', 48), ('c', 46)].into_iter().enumerate() {
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    12_890 + i as u32,
                    ch as u32,
                    code,
                    0
                )
                .await
            );
            td121_expect_legacy_commit_text(&mut harness.peer, &ch.to_string()).await;
            exact_surrounding_receipt(&mut harness, &mut engine, &"abc"[..i + 1]).await;
        }
        assert_eq!(engine.committed_tail.buffer, "abc");

        // The widget handles Ctrl+A, then Backspace deletes its selection.
        // The IME has no right to retain a one-character-backspace mirror.
        assert!(!legacy_key(&mut harness, &mut engine, 12_893, 'a' as u32, 30, 1 << 2).await);
        assert_eq!(engine.committed_tail.buffer, "");
        assert!(!legacy_key(&mut harness, &mut engine, 12_894, KEY_BACKSPACE, 14, 0).await);
        exact_surrounding_receipt(&mut harness, &mut engine, "").await;

        for (i, (ch, code)) in [('d', 32), ('e', 18), ('f', 33)].into_iter().enumerate() {
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    12_895 + i as u32,
                    ch as u32,
                    code,
                    0
                )
                .await
            );
            td121_expect_legacy_commit_text(&mut harness.peer, &ch.to_string()).await;
            exact_surrounding_receipt(&mut harness, &mut engine, &"def"[..i + 1]).await;
        }
        assert_eq!(engine.committed_tail.buffer, "def");
        assert!(engine.context_observed_suffix_exact_manual_handoff_allowed());
    }));
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
        retained_boundary_literal_keys(&mut harness, &mut engine, &mut serial, "ab ", false).await;
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
        retained_boundary_literal_keys(&mut harness, &mut engine, &mut serial, "c", false).await;
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
        retained_boundary_literal_keys(&mut harness, &mut engine, &mut serial, " ", false).await;
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
        retained_boundary_literal_keys(&mut harness, &mut engine, &mut serial, "ab ", false).await;
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
        let input_mode_before_key = source.layout_gesture.layout_is_ru;
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
        expect_legacy_commit(&mut harness.peer, &source, input_mode_before_key).await;
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
    let input_mode_before_key = source.layout_gesture.layout_is_ru;
    let native_space_was_visible = source.composition.preedit_visible;
    assert!(legacy_key(harness, source, 11_910, KEY_SPACE, 57, 0).await);
    expect_legacy_managed_space(
        harness,
        source,
        input_mode_before_key,
        native_space_was_visible,
    )
    .await;
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
    send_manually_dispatched_callback(
        &mut harness.peer,
        &factory,
        "/org/freedesktop/IBus/Factory",
        FACTORY_INTERFACE,
    )
    .await;
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
        send_manually_dispatched_callback(
            &mut harness.peer,
            &event,
            &source.path,
            ENGINE_INTERFACE,
        )
        .await;
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
    send_manually_dispatched_callback(&mut harness.peer, &focus, target_path, ENGINE_INTERFACE)
        .await;
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
        let input_mode_before_key = source.layout_gesture.layout_is_ru;
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
        expect_legacy_commit(&mut harness.peer, &source, input_mode_before_key).await;
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
        .with_flags(zbus::message::Flags::NoReplyExpected)
        .unwrap()
        .build(&key)
        .unwrap();
    send_manually_dispatched_callback(
        &mut harness.peer,
        &message,
        engine.path.as_str(),
        ENGINE_INTERFACE,
    )
    .await;
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
    send_manually_dispatched_callback(&mut harness.peer, &message, &engine.path, ENGINE_INTERFACE)
        .await;
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
    external_prefix: &str,
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
    let original = format!("{external_prefix}abc");
    let original_cursor = original.chars().count() as u32;
    cycle09_surrounding_receipt(
        &mut harness,
        &mut engine,
        &original,
        original_cursor,
        original_cursor,
    )
    .await;
    if !external_prefix.is_empty() {
        assert!(matches!(
            engine.committed_tail.autocorrect_suppression.as_ref(),
            Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope))
                if scope.observed_external_prefix.as_deref() == Some(external_prefix)
        ));
    }
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
    let text = format!("{external_prefix}{received}");
    let cursor = text.chars().count() as u32;
    if prefix_chars == 1 && delayed_receipt {
        assert!(
            !engine.exact_replay_contains_prior_snapshot(&SurroundingTextSnapshot::new(
                "foreign ".to_string(),
                8,
                8,
            ))
        );
    }
    cycle09_surrounding_receipt(&mut harness, &mut engine, &text, cursor, cursor).await;
    assert!(engine.context_reset_rereceipt.is_some());
    assert_eq!(
        engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
        !delayed_receipt
    );
    assert!(!engine.context_word_is_known());
    (harness, engine)
}

// Private NOT_RUN insertion in adapter/tests/residuals.rs. No runtime edits.
async fn firefox_empty_reset_expect_commit(
    harness: &mut Harness,
    engine: &LayIbusEngine,
    expected_mode: bool,
    expected_text: &str,
) {
    let message = next_legacy_text_effect(&mut harness.peer, engine, expected_mode).await;
    assert_eq!(message.header().message_type(), zbus::message::Type::Signal);
    assert_eq!(message.header().path().unwrap().as_str(), engine.path);
    assert_eq!(
        message.header().interface().unwrap().as_str(),
        ENGINE_INTERFACE
    );
    assert_eq!(message.header().member().unwrap().as_str(), "CommitText");
    let body = message.body();
    let text = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
    assert_eq!(
        crate::ibus_interface::ibus_text_value_to_string(&text).as_deref(),
        Some(expected_text),
    );
    cycle09_assert_no_local_text_effect(harness).await;
}

async fn firefox_empty_reset_expect_probe_commit(
    harness: &mut Harness,
    engine: &LayIbusEngine,
    expected_mode: bool,
    expected_text: &str,
    serial: u32,
) {
    // The production handler and detached dispatcher may write in either
    // order. Keep every client effect and filter only the registered exact
    // UnknownObject reply, then fence the dispatcher before the next key.
    firefox_empty_reset_expect_commit(harness, engine, expected_mode, expected_text).await;
    bounded(complete_absent_manual_dispatch(&mut harness.peer)).await;
    assert!(!harness
        .peer
        .detached_callback_reply_serials
        .contains(&serial));
    assert!(harness
        .peer
        .detached_callback_reply_serials
        .counts
        .is_empty());
}

async fn firefox_detached_probe_transport_orders() {
    for signal_first in [true, false] {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let engine = new_engine(&harness);
        let serial = 30_900;
        let callback = observed_callback_without_reply(serial, &engine.path, "Reset");
        let mut reply_observer = MessageStream::from(&harness.peer.connection);
        if signal_first {
            harness
                .connection
                .emit_signal(
                    None::<&str>,
                    engine.path.as_str(),
                    ENGINE_INTERFACE,
                    "CommitText",
                    &crate::text::make_ibus_text("probe".to_string()),
                )
                .await
                .unwrap();
        }
        send_manually_dispatched_callback(
            &mut harness.peer,
            &callback,
            &engine.path,
            ENGINE_INTERFACE,
        )
        .await;
        if !signal_first {
            // This independent stream fences Error arrival while the primary
            // peer stream still contains that exact packet for its filter.
            let reply = bounded(next_ordered_message(Pin::new(&mut reply_observer)))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(reply.header().message_type(), Type::Error);
            assert_eq!(reply.header().reply_serial().map(|n| n.get()), Some(serial));
            assert_eq!(
                reply.header().error_name().map(|n| n.as_str()),
                Some("org.freedesktop.DBus.Error.UnknownObject")
            );
            harness
                .connection
                .emit_signal(
                    None::<&str>,
                    engine.path.as_str(),
                    ENGINE_INTERFACE,
                    "CommitText",
                    &crate::text::make_ibus_text("probe".to_string()),
                )
                .await
                .unwrap();
        }
        firefox_empty_reset_expect_probe_commit(
            &mut harness,
            &engine,
            engine.layout_gesture.layout_is_ru,
            "probe",
            serial,
        )
        .await;
    }
}

async fn firefox_empty_reset_six_scalar_handoff(
    serial: u32,
) -> (Harness, LayIbusEngine, String, u64) {
    let mut harness = cycle09_harness().await;
    let mut source = new_engine(&harness);
    start_source_free_unknown(&mut harness, &mut source).await;
    source.config.auto_replace = false;
    source.config.typing_assist = false;
    source.config.nanda_precognition = false;
    source.set_client_capabilities(41);
    source.set_content_type_state(0, 0);
    for (offset, (ch, code)) in [
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
        let mode = source.layout_gesture.layout_is_ru;
        assert!(!mode);
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
        firefox_empty_reset_expect_commit(&mut harness, &source, mode, &ch.to_string()).await;
    }
    assert_eq!(source.committed_tail.buffer, "ghbdtn");
    assert!(!source.context_word_is_known());
    cycle09_surrounding_receipt(&mut harness, &mut source, "prefix ghbdtn", 13, 13).await;
    let source_owner = source
        .context_owner
        .clone()
        .expect("actual admitted source owner");
    let (source, disposition) = cycle09_manual_toggle(&mut harness, source).await;
    assert_eq!(disposition, Ok((3, false)));
    let (source, source_tail) = cycle09_visible_tail(&mut harness, source).await;
    let source_path = source.path.clone();
    assert!(cycle09_tail_is_authoritative(
        &source_tail,
        &source_path,
        false,
        "ghbdtn"
    ));
    let source_epoch = source_tail.as_ref().unwrap().3;
    let target_path = format!("{TARGET_PATH}_firefox_empty_reset_six_{serial}");
    let target = cycle09_factory_handoff(
        &mut harness,
        source,
        serial + 20,
        &target_path,
        None,
        "lay-ime-ru",
    )
    .await;
    eprintln!("EMPTY_RESET_REPLAY_STAGE source_owner={source_owner:?} target_owner={:?} target_path={target_path} source_epoch={source_epoch}", target.context_owner);
    (harness, target, target_path, source_epoch)
}

fn firefox_empty_reset_assert_replay_phase(engine: &LayIbusEngine, distance: u64) {
    let local = engine.committed_tail.autocorrect_suppression.clone();
    let shared = engine
        .shared
        .lock()
        .unwrap()
        .autocorrect_suppression
        .clone();
    assert_eq!(
        local, shared,
        "native replay must retain equal local/shared scopes"
    );
    let Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope)) = local else {
        panic!("actual bridge must admit an ExactReplay scope");
    };
    assert_eq!(scope.original_suffix, "ghbdtn");
    assert_eq!(scope.replacement, "привет");
    assert_eq!(scope.path, engine.path);
    assert_eq!(
        engine.committed_tail.epoch.wrapping_sub(scope.epoch),
        distance
    );
    assert!(engine.exact_replay_quarantine_active());
}

fn firefox_empty_reset_assert_unknown_lineage(engine: &LayIbusEngine, count: u32, stage: &str) {
    let scope = engine
        .context_word_scope
        .as_ref()
        .expect("actual local word scope");
    let token = engine.live_context_token();
    let reducer_token = engine.context_admission.as_ref().unwrap().current_token();
    eprintln!("EMPTY_RESET_REPLAY_STAGE stage={stage} epoch={} local={:?} reducer={:?} live={} pending={}",
        engine.committed_tail.epoch, scope.lineage(),
        reducer_token.as_ref().map(|token| token.word_scope().lineage()),
        token.is_some(), engine.context_reset_rereceipt.is_some());
    assert_eq!(
        scope.lineage().completeness,
        WordCompleteness::UnknownStart,
        "{stage}"
    );
    assert_eq!(scope.lineage().observed_suffix_chars, count, "{stage}");
    let token = token.expect("received callback must retain its settled token");
    assert_eq!(token.word_scope().lineage(), scope.lineage(), "{stage}");
    assert_eq!(Some(token.clone()), reducer_token, "{stage}");
    assert!(token.matches_owner(engine.context_owner.as_ref().unwrap()));
    assert!(!engine.context_word_is_known());
}

#[test]
fn firefox_native_replay_empty_reset_six_inserts_probe_and_manual_handoff() {
    zbus::block_on(bounded(async {
        firefox_detached_probe_transport_orders().await;
        let mut serial = 31_000;
        let (mut harness, target, path, epoch) =
            firefox_empty_reset_six_scalar_handoff(serial).await;
        serial += 40;
        let (mut engine, suppression) =
            cycle09_suppress_exact_replay(&mut harness, target, "ghbdtn", epoch, &path, true).await;
        assert_eq!(suppression, Ok(true));
        engine.config.text_backend = "ime".into();
        engine.config.typing_assist = false;
        engine.config.nanda_precognition = false;
        let owner = engine
            .context_owner
            .clone()
            .expect("actual factory target owner");
        assert!(engine.live_composition_enabled());
        cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix ghbdtn", 13, 13).await;
        firefox_empty_reset_assert_replay_phase(&engine, 0);

        for erased in 1..=6usize {
            let before_epoch = engine.committed_tail.epoch;
            assert!(
                !firefox_replay_callback(
                    &mut harness,
                    &mut engine,
                    &mut serial,
                    (KEY_BACKSPACE, 14, 0),
                )
                .await
            );
            assert_eq!(engine.committed_tail.epoch, before_epoch.wrapping_add(1));
            assert_eq!(engine.committed_tail.buffer, &"ghbdtn"[..6 - erased]);
            firefox_empty_reset_assert_replay_phase(&engine, erased as u64);
            assert!(
                !firefox_replay_callback(
                    &mut harness,
                    &mut engine,
                    &mut serial,
                    (KEY_BACKSPACE, 14, RELEASE_MASK),
                )
                .await
            );
            assert_eq!(engine.committed_tail.epoch, before_epoch.wrapping_add(1));
            if erased == 2 {
                // Measured ordering: partial Reset, prior echo, exact shortened echo.
                firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
                eprintln!(
                    "EMPTY_RESET_REPLAY_STAGE stage=partial_delete_reset pending={} token={}",
                    engine.context_reset_rereceipt.is_some(),
                    engine.live_context_token().is_some()
                );
                assert!(engine.context_reset_rereceipt.is_some());
                cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix ghbdtn", 13, 13)
                    .await;
                assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix ghbd", 11, 11).await;
                assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            }
        }

        let predecessor = engine
            .context_token
            .clone()
            .expect("actual pre-empty-Reset token");
        firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
        assert_eq!(engine.committed_tail.buffer, "");
        assert!(engine.context_reset_rereceipt.is_none());
        assert_ne!(engine.context_token.as_ref(), Some(&predecessor));
        firefox_empty_reset_assert_unknown_lineage(&engine, 0, "empty_reset_before_inserts");
        cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix ", 7, 7).await;
        assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        assert!(engine.context_reset_rereceipt.is_none());
        firefox_empty_reset_assert_replay_phase(&engine, 6);

        for (offset, (ch, code)) in [
            ('п', 34),
            ('р', 35),
            ('и', 48),
            ('в', 32),
            ('е', 20),
            ('т', 49),
        ]
        .into_iter()
        .enumerate()
        {
            firefox_empty_reset_assert_replay_phase(&engine, 6 + offset as u64);
            let before_epoch = engine.committed_tail.epoch;
            let keyval = 0x0100_0000 | ch as u32;
            assert!(!firefox_replay_callback(
                &mut harness, &mut engine, &mut serial, (keyval, code, 0),
            ).await);
            assert_eq!(engine.committed_tail.epoch, before_epoch.wrapping_add(1));
            let expected: String = "привет".chars().take(offset + 1).collect();
            assert_eq!(engine.committed_tail.buffer, expected);
            firefox_empty_reset_assert_replay_phase(&engine, 7 + offset as u64);
            firefox_empty_reset_assert_unknown_lineage(
                &engine,
                offset as u32 + 1,
                "native_insert_press",
            );
            assert!(
                !firefox_replay_callback(
                    &mut harness,
                    &mut engine,
                    &mut serial,
                    (keyval, code, RELEASE_MASK),
                )
                .await
            );
            assert_eq!(engine.committed_tail.epoch, before_epoch.wrapping_add(1));
            firefox_empty_reset_assert_unknown_lineage(
                &engine,
                offset as u32 + 1,
                "native_insert_release",
            );
            assert_eq!(engine.context_owner.as_ref(), Some(&owner));
        }

        firefox_empty_reset_assert_unknown_lineage(&engine, 6, "before_nonempty_reset");
        firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
        firefox_empty_reset_assert_unknown_lineage(&engine, 0, "after_nonempty_reset");
        assert!(
            engine.context_reset_rereceipt.is_some(),
            "count6 was proved before actual Reset; inspect capture/post-token if absent"
        );
        assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix привет", 13, 13).await;
        assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());

        // A handled ManagedCommit probe uses the transferred dynamic path,
        // not the Native helper's no-local-text-effect assertion.
        let before_epoch = engine.committed_tail.epoch;
        let mode = engine.layout_gesture.layout_is_ru;
        assert!(mode);
        assert!(
            legacy_key_at(
                &mut harness,
                &mut engine,
                &path,
                serial,
                0x0100_0000 | 'а' as u32,
                33,
                0,
            )
            .await
        );
        firefox_empty_reset_expect_probe_commit(&mut harness, &engine, mode, "а", serial).await;
        serial += 1;
        assert_eq!(engine.committed_tail.buffer, "привета");
        assert_eq!(engine.committed_tail.epoch, before_epoch.wrapping_add(1));
        assert!(
            firefox_replay_callback(
                &mut harness,
                &mut engine,
                &mut serial,
                (0x0100_0000 | 'а' as u32, 33, RELEASE_MASK),
            )
            .await
        );
        firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
        cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix привета", 14, 14).await;
        assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());

        let before_epoch = engine.committed_tail.epoch;
        assert!(
            !firefox_replay_callback(
                &mut harness,
                &mut engine,
                &mut serial,
                (KEY_BACKSPACE, 14, 0),
            )
            .await
        );
        assert_eq!(engine.committed_tail.buffer, "привет");
        assert_eq!(engine.committed_tail.epoch, before_epoch.wrapping_add(1));
        assert!(
            !firefox_replay_callback(
                &mut harness,
                &mut engine,
                &mut serial,
                (KEY_BACKSPACE, 14, RELEASE_MASK),
            )
            .await
        );
        firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
        assert!(engine.live_context_token().is_some());
        cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix привет", 13, 13).await;
        assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        assert_eq!(engine.context_owner.as_ref(), Some(&owner));
        assert_eq!(engine.path, path);
        assert!(!engine.context_word_is_known());
        assert!(engine.committed_tail.pending_completion_learning.is_none());
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());

        let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
        assert_eq!(
            disposition,
            Ok(lay::manual_toggle::ImeManualToggleOutcome::DelegateExactImeTail.as_v3())
        );
        assert!(engine.layout_gesture.layout_is_ru);
        let (engine, tail) = cycle09_visible_tail(&mut harness, engine).await;
        assert!(cycle09_tail_is_authoritative(&tail, &path, true, "привет"));
        assert_eq!(engine.context_owner.as_ref(), Some(&owner));
        assert!(!engine.context_word_is_known());
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    }));
}

// Private NOT_RUN addition after FULL_V2_EMPTY_RESET_REPLAY_REGRESSION.patch.
// Real messages are observed first and dispatched later on one executor.
// No test_set_word_scope, token assignment, forged ready state, or new authority.
async fn firefox_receive_deferred_native_key(
    harness: &mut Harness,
    engine: &LayIbusEngine,
    serial: &mut u32,
    key: (u32, u32, u32),
) -> Message {
    let message = Message::method_call(engine.path.as_str(), "ProcessKeyEvent")
        .unwrap()
        .interface(ENGINE_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(*serial).unwrap())
        .with_flags(zbus::message::Flags::NoReplyExpected)
        .unwrap()
        .build(&key)
        .unwrap();
    send_manually_dispatched_callback(
        &mut harness.peer,
        &message,
        engine.path.as_str(),
        ENGINE_INTERFACE,
    )
    .await;
    assert!(harness.observer.process_next().await.unwrap());
    consume_detached_callback_reply(&mut harness.peer, *serial).await;
    *serial += 1;
    message
}

async fn firefox_receive_deferred_reset(
    harness: &mut Harness,
    engine: &LayIbusEngine,
    serial: &mut u32,
) -> Message {
    let message = observed_callback_without_reply(*serial, &engine.path, "Reset");
    send_manually_dispatched_callback(&mut harness.peer, &message, &engine.path, ENGINE_INTERFACE)
        .await;
    assert!(harness.observer.process_next().await.unwrap());
    consume_detached_callback_reply(&mut harness.peer, *serial).await;
    *serial += 1;
    message
}

async fn firefox_dispatch_deferred_native_key(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    message: &Message,
) -> bool {
    let key: (u32, u32, u32) = message.body().deserialize().unwrap();
    let emitter =
        zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone()).unwrap();
    let handled = engine
        .process_key_event(message.header(), emitter, key.0, key.1, key.2)
        .await
        .unwrap();
    // UnknownObject replies were already consumed by exact serial at ingress.
    // A validated native replay must not emit CommitText/DeleteSurroundingText.
    cycle09_assert_no_local_text_effect(harness).await;
    handled
}

fn firefox_observer_ahead_lineage_metadata(engine: &LayIbusEngine, stage: &str) {
    let token = engine.context_token.as_ref();
    let scope = engine.context_word_scope.as_ref();
    let owner = engine.context_owner.as_ref();
    let pending = engine.context_reset_rereceipt.as_ref();
    eprintln!(
        "OBSERVER_AHEAD_INSERT stage={stage} epoch={} tail_chars={} scope_chars={:?} scope_known={} token_present={} token_owner={} token_scope={} token_live={} pending={} pending_chars={:?} pending_confirmed={:?} exact_replay={}",
        engine.committed_tail.epoch,
        engine.committed_tail.buffer.chars().count(),
        scope.map(|scope| scope.lineage().observed_suffix_chars),
        engine.context_word_is_known(),
        token.is_some(),
        token.zip(owner).is_some_and(|(token, owner)| token.matches_owner(owner)),
        token.zip(scope).is_some_and(|(token, scope)| token.matches_word_scope(scope)),
        engine.live_context_token().is_some(),
        pending.is_some(),
        pending.map(|pending| pending.observed_suffix_chars),
        pending.map(|pending| pending.confirmed),
        engine.exact_replay_quarantine_active(),
    );
}

async fn firefox_observer_ahead_six_inserts(
    serial: &mut u32,
) -> (Harness, LayIbusEngine, String, bool, bool) {
    let (mut harness, target, path, epoch) = firefox_empty_reset_six_scalar_handoff(*serial).await;
    *serial += 40;
    let (mut engine, suppression) =
        cycle09_suppress_exact_replay(&mut harness, target, "ghbdtn", epoch, &path, true).await;
    assert_eq!(suppression, Ok(true));
    engine.config.text_backend = "ime".into();
    engine.config.typing_assist = false;
    engine.config.nanda_precognition = false;
    let owner = engine
        .context_owner
        .clone()
        .expect("actual factory target owner");
    assert!(engine.live_composition_enabled());
    cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix ghbdtn", 13, 13).await;

    // First reproduce the real replay deletion and genuine empty Reset.
    for erased in 1..=6usize {
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
        assert_eq!(engine.committed_tail.buffer, &"ghbdtn"[..6 - erased]);
        firefox_empty_reset_assert_replay_phase(&engine, erased as u64);
    }
    firefox_replay_reset(&mut harness, &mut engine, serial).await;
    firefox_empty_reset_assert_unknown_lineage(&engine, 0, "observer_ahead_empty_reset");
    cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix ", 7, 7).await;
    assert!(engine.context_reset_rereceipt.is_none());

    // The first three insert press/releases and fourth press settle normally.
    // The fourth release is deferred, as in accepted PRESS375 -> RELEASE376.
    for (offset, (ch, code)) in [('п', 34), ('р', 35), ('и', 48), ('в', 32)]
        .into_iter()
        .enumerate()
    {
        let keyval = 0x0100_0000 | ch as u32;
        assert!(
            !firefox_replay_callback(&mut harness, &mut engine, serial, (keyval, code, 0),).await
        );
        firefox_empty_reset_assert_unknown_lineage(&engine, offset as u32 + 1, "settled_insert");
        if offset < 3 {
            assert!(
                !firefox_replay_callback(
                    &mut harness,
                    &mut engine,
                    serial,
                    (keyval, code, RELEASE_MASK),
                )
                .await
            );
        }
    }
    assert_eq!(engine.committed_tail.buffer, "прив");
    let predecessor = engine
        .context_token
        .clone()
        .expect("actual settled fourth-insert token");
    assert!(engine
        .context_admission
        .as_ref()
        .unwrap()
        .revalidate(&predecessor));
    let prefix_epoch = engine.committed_tail.epoch;
    firefox_observer_ahead_lineage_metadata(&engine, "settled_fourth_press");

    // These are actual X11 Cyrillic_ie / Cyrillic_te values from the failure.
    // Preserve physical E/T codes: Unicode keysyms are not substituted here.
    let queue_keys = [
        (0x0100_0000 | 'в' as u32, 32, RELEASE_MASK),
        (1733, 20, 0),
        (1733, 20, RELEASE_MASK),
        (1748, 49, 0),
        (1748, 49, RELEASE_MASK),
    ];
    let mut messages = Vec::new();
    for key in queue_keys {
        messages
            .push(firefox_receive_deferred_native_key(&mut harness, &engine, serial, key).await);
    }
    let reset = firefox_receive_deferred_reset(&mut harness, &engine, serial).await;
    // The observer, not a fixture setter, revokes the settled predecessor.
    assert!(!engine
        .context_admission
        .as_ref()
        .unwrap()
        .revalidate(&predecessor));
    assert_eq!(engine.committed_tail.buffer, "прив");
    assert_eq!(engine.committed_tail.epoch, prefix_epoch);
    firefox_observer_ahead_lineage_metadata(&engine, "reset_ingress_before_queued_callbacks");

    let mut fifth_retained = false;
    let mut sixth_retained = false;
    for (index, message) in messages.iter().enumerate() {
        assert!(!firefox_dispatch_deferred_native_key(&mut harness, &mut engine, message).await);
        let expected = match index {
            0 => "прив",
            1 | 2 => "приве",
            3 | 4 => "привет",
            _ => unreachable!(),
        };
        assert_eq!(engine.committed_tail.buffer, expected);
        assert_eq!(engine.context_owner.as_ref(), Some(&owner));
        assert!(!engine.context_word_is_known());
        assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        assert!(!engine
            .context_admission
            .as_ref()
            .unwrap()
            .revalidate(&predecessor));
        if index == 1 {
            fifth_retained = engine.context_reset_rereceipt.is_some();
        }
        if index == 3 {
            sixth_retained = engine.context_reset_rereceipt.is_some();
        }
        firefox_observer_ahead_lineage_metadata(
            &engine,
            match index {
                0 => "queued_fourth_release",
                1 => "queued_fifth_press",
                2 => "queued_fifth_release",
                3 => "queued_sixth_press",
                4 => "queued_sixth_release",
                _ => unreachable!(),
            },
        );
    }
    assert_eq!(engine.committed_tail.epoch, prefix_epoch.wrapping_add(2));
    engine
        .reset(
            reset.header(),
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap(),
        )
        .await
        .unwrap();
    cycle09_assert_no_local_text_effect(&mut harness).await;
    assert_eq!(engine.context_owner.as_ref(), Some(&owner));
    assert_eq!(engine.committed_tail.buffer, "привет");
    assert!(!engine.context_word_is_known());
    assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
    assert!(engine.committed_tail.pending_completion_learning.is_none());
    assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    firefox_observer_ahead_lineage_metadata(&engine, "queued_received_reset");
    (harness, engine, path, fifth_retained, sixth_retained)
}

#[test]
fn firefox_observer_first_native_insert_reset_recovers_only_after_exact_receipt() {
    zbus::block_on(bounded(async {
        let mut serial = 32_000;
        let (mut harness, mut engine, path, fifth_retained, sixth_retained) =
            firefox_observer_ahead_six_inserts(&mut serial).await;
        let owner = engine.context_owner.clone().unwrap();
        cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix привет", 13, 13).await;
        firefox_observer_ahead_lineage_metadata(&engine, "exact_six_receipt");
        // Expected failure on unchanged production. Earlier metadata isolates
        // the first loss at the fifth native append, before the received Reset.
        assert!(
            engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
            "observer-ahead Reset lost validated replay provenance: fifth_pending={fifth_retained}, sixth_pending={sixth_retained}; exact six-character receipt must authorize only the existing handoff gate",
        );
        assert!(!engine.context_word_is_known());

        let mode = engine.layout_gesture.layout_is_ru;
        assert!(mode);
        assert!(
            legacy_key_at(
                &mut harness,
                &mut engine,
                &path,
                serial,
                0x0100_0000 | 'а' as u32,
                33,
                0,
            )
            .await
        );
        firefox_empty_reset_expect_probe_commit(&mut harness, &engine, mode, "а", serial).await;
        serial += 1;
        assert_eq!(engine.committed_tail.buffer, "привета");
        assert!(
            firefox_replay_callback(
                &mut harness,
                &mut engine,
                &mut serial,
                (0x0100_0000 | 'а' as u32, 33, RELEASE_MASK),
            )
            .await
        );
        firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
        cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix привета", 14, 14).await;
        assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());

        assert!(
            !firefox_replay_callback(
                &mut harness,
                &mut engine,
                &mut serial,
                (KEY_BACKSPACE, 14, 0),
            )
            .await
        );
        assert_eq!(engine.committed_tail.buffer, "привет");
        assert!(
            !firefox_replay_callback(
                &mut harness,
                &mut engine,
                &mut serial,
                (KEY_BACKSPACE, 14, RELEASE_MASK),
            )
            .await
        );
        firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
        cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix привет", 13, 13).await;
        assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        assert_eq!(engine.context_owner.as_ref(), Some(&owner));
        assert_eq!(engine.path, path);

        let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
        assert_eq!(
            disposition,
            Ok(lay::manual_toggle::ImeManualToggleOutcome::DelegateExactImeTail.as_v3(),)
        );
        let (engine, tail) = cycle09_visible_tail(&mut harness, engine).await;
        assert!(cycle09_tail_is_authoritative(&tail, &path, true, "привет"));
        assert_eq!(engine.context_owner.as_ref(), Some(&owner));
        assert!(!engine.context_word_is_known());
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    }));
}

async fn firefox_observer_ahead_refuses_receipt_gap(gap: &str) {
    let mut serial = 33_000;
    let (mut harness, mut engine, path, _, _) =
        firefox_observer_ahead_six_inserts(&mut serial).await;
    match gap {
        "selection" => {
            cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix привет", 13, 7).await
        }
        "wrong_surface" => {
            cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix приветx", 14, 14).await
        }
        "focus_out" => {
            let message = observed_callback_without_reply(serial, &path, "FocusOut");
            send_manually_dispatched_callback(&mut harness.peer, &message, &path, ENGINE_INTERFACE)
                .await;
            assert!(harness.observer.process_next().await.unwrap());
            consume_detached_callback_reply(&mut harness.peer, serial).await;
            engine.focus_out(message.header()).await;
            cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix привет", 13, 13).await;
        }
        _ => unreachable!(),
    }
    assert!(
        !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
        "{gap}"
    );
    assert!(!engine.context_word_is_known());
    assert!(engine.committed_tail.pending_completion_learning.is_none());
    assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
    assert!(
        matches!(disposition, Ok((0, _)) | Err(_)),
        "{gap}: {disposition:?}"
    );
    assert!(!cycle09_exact_handoff_is_live(&engine), "{gap}");
    cycle09_assert_no_local_text_effect(&mut harness).await;
}

#[test]
fn firefox_observer_first_native_insert_reset_rejects_selection() {
    zbus::block_on(bounded(firefox_observer_ahead_refuses_receipt_gap(
        "selection",
    )));
}

#[test]
fn firefox_observer_first_native_insert_reset_rejects_wrong_surface() {
    zbus::block_on(bounded(firefox_observer_ahead_refuses_receipt_gap(
        "wrong_surface",
    )));
}

#[test]
fn firefox_observer_first_native_insert_reset_rejects_focus_out() {
    zbus::block_on(bounded(firefox_observer_ahead_refuses_receipt_gap(
        "focus_out",
    )));
}

#[test]
fn firefox_native_replay_append_preserves_reset_lineage_until_exact_tail() {
    zbus::block_on(bounded(async {
        for (prefix_chars, delayed_receipt, external_prefix) in [
            (1, false, "prefix "),
            (1, true, "prefix "),
            (2, false, "prefix "),
            (2, true, "prefix "),
            (1, false, ""),
            (1, true, ""),
            (2, false, ""),
            (2, true, ""),
        ] {
            let mut serial = 15_000;
            let (mut harness, mut engine) = firefox_replay_prefix_with_reset(
                &mut serial,
                prefix_chars,
                delayed_receipt,
                external_prefix,
            )
            .await;
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
            // A completed native replay can still receive one delayed
            // client surface from deletion or insertion after this Reset.
            let stale = if prefix_chars == 1 {
                format!("{external_prefix}a")
            } else {
                format!("{external_prefix}ф")
            };
            let stale_cursor = stale.chars().count() as u32;
            cycle09_surrounding_receipt(
                &mut harness,
                &mut engine,
                &stale,
                stale_cursor,
                stale_cursor,
            )
            .await;
            assert!(engine.context_reset_rereceipt.is_some());
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            let exact = format!("{external_prefix}фис");
            let exact_cursor = exact.chars().count() as u32;
            cycle09_surrounding_receipt(
                &mut harness,
                &mut engine,
                &exact,
                exact_cursor,
                exact_cursor,
            )
            .await;
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
fn firefox_completed_boundary_replay_waits_for_final_exact_receipt() {
    zbus::block_on(bounded(async {
        let mut harness = cycle09_harness().await;
        let mut source = cycle09_source(&mut harness).await;
        cycle09_add_trailing_boundary(&mut harness, &mut source).await;
        cycle09_surrounding_receipt(&mut harness, &mut source, "prefix abc ", 11, 11).await;
        let (source, disposition) = cycle09_manual_toggle(&mut harness, source).await;
        assert_eq!(disposition, Ok((3, false)));
        let (source, tail) = cycle09_visible_tail(&mut harness, source).await;
        assert!(cycle09_tail_is_authoritative(
            &tail,
            &source.path,
            false,
            " abc ",
        ));
        let epoch = tail.as_ref().unwrap().3;
        let target_path = format!("{TARGET_PATH}_completed_boundary_replay");
        let target = cycle09_factory_handoff(
            &mut harness,
            source,
            26_000,
            &target_path,
            None,
            "lay-ime-ru",
        )
        .await;
        let (mut target, suppression) =
            cycle09_suppress_exact_replay(&mut harness, target, "abc ", epoch, &target_path, true)
                .await;
        assert_eq!(suppression, Ok(true));
        target.config.text_backend = "ime".into();
        target.config.typing_assist = false;
        target.config.nanda_precognition = false;
        cycle09_surrounding_receipt(&mut harness, &mut target, "prefix abc ", 11, 11).await;
        let mut serial = 26_040;
        for _ in 0..4 {
            for state in [0, RELEASE_MASK] {
                assert!(
                    !firefox_replay_callback(
                        &mut harness,
                        &mut target,
                        &mut serial,
                        (KEY_BACKSPACE, 14, state),
                    )
                    .await
                );
            }
        }
        assert_eq!(target.committed_tail.buffer, " ");
        for (ch, code) in [('ф', 30), ('и', 48), ('с', 46), (' ', 57)] {
            let keyval = if ch == ' ' {
                KEY_SPACE
            } else {
                0x0100_0000 | ch as u32
            };
            for state in [0, RELEASE_MASK] {
                assert!(
                    !firefox_replay_callback(
                        &mut harness,
                        &mut target,
                        &mut serial,
                        (keyval, code, state),
                    )
                    .await
                );
            }
        }
        assert_eq!(target.committed_tail.buffer, " фис ");
        firefox_replay_reset(&mut harness, &mut target, &mut serial).await;
        assert!(target.context_reset_rereceipt.is_some());

        // Firefox reports the surface after deletion even though replacement
        // keys have already completed. It cannot yet authorize another edit.
        cycle09_surrounding_receipt(&mut harness, &mut target, "prefix ", 7, 7).await;
        assert!(target.context_reset_rereceipt.is_some());
        assert!(!target.context_reset_rereceipt_exact_manual_handoff_allowed());
        assert!(!target.context_word_is_known());

        cycle09_surrounding_receipt(&mut harness, &mut target, "prefix фис ", 11, 11).await;
        assert!(target.context_reset_rereceipt_exact_manual_handoff_allowed());
        let (target, disposition) = cycle09_manual_toggle(&mut harness, target).await;
        assert_eq!(
            disposition,
            Ok(lay::manual_toggle::ImeManualToggleOutcome::DelegateExactImeTail.as_v3()),
        );
        let (target, tail) = cycle09_visible_tail(&mut harness, target).await;
        assert!(cycle09_tail_is_authoritative(
            &tail,
            &target.path,
            true,
            " фис ",
        ));
    }));
}

#[test]
fn firefox_source_free_first_word_after_space_delegates_exact_tail() {
    zbus::block_on(bounded(async {
        for reset_before_toggle in [false, true] {
            let mut harness = cycle09_harness().await;
            let mut source = new_engine(&harness);
            start_source_free_unknown(&mut harness, &mut source).await;
            source.config.auto_replace = false;
            source.config.typing_assist = false;
            source.config.nanda_precognition = false;
            source.set_client_capabilities(41);
            source.set_content_type_state(0, 0);
            for (offset, (ch, code)) in [('a', 30), ('b', 48), ('c', 46)].into_iter().enumerate() {
                let input_mode_before_key = source.layout_gesture.layout_is_ru;
                assert!(
                    legacy_key(
                        &mut harness,
                        &mut source,
                        27_000 + offset as u32,
                        ch as u32,
                        code,
                        0,
                    )
                    .await
                );
                expect_legacy_commit(&mut harness.peer, &source, input_mode_before_key).await;
            }
            assert_eq!(source.committed_tail.buffer, "abc");
            assert!(!source.context_word_is_known());
            cycle09_surrounding_receipt(&mut harness, &mut source, "abc", 3, 3).await;
            let input_mode_before_key = source.layout_gesture.layout_is_ru;
            let native_space_was_visible = source.composition.preedit_visible;
            assert!(legacy_key(&mut harness, &mut source, 27_010, KEY_SPACE, 57, 0).await);
            expect_legacy_managed_space(
                &mut harness,
                &source,
                input_mode_before_key,
                native_space_was_visible,
            )
            .await;
            assert_eq!(source.committed_tail.buffer, "abc ");
            cycle09_surrounding_receipt(&mut harness, &mut source, "abc ", 4, 4).await;
            if reset_before_toggle {
                actual_reset(&mut harness, &mut source, 27_020, false).await;
                cycle09_surrounding_receipt(&mut harness, &mut source, "abc ", 4, 4).await;
            }
            let (source, disposition) = cycle09_manual_toggle(&mut harness, source).await;
            assert_eq!(
                disposition,
                Ok(lay::manual_toggle::ImeManualToggleOutcome::DelegateExactImeTail.as_v3()),
                "first word after Space must delegate, reset_before_toggle={reset_before_toggle}"
            );
            let (source, tail) = cycle09_visible_tail(&mut harness, source).await;
            assert!(cycle09_tail_is_authoritative(
                &tail,
                &source.path,
                false,
                "abc ",
            ));
        }
    }));
}

#[test]
fn firefox_first_word_space_retires_published_preedit_before_exact_receipt() {
    zbus::block_on(bounded(async {
        for (retired_surface, cursor, expected_retention) in [
            ("abcdefgh", 1, true),
            ("abcxefgh", 1, false),
            ("abcdefgh", 0, false),
        ] {
            let mut harness = cycle09_harness().await;
            let mut source = new_engine(&harness);
            start_source_free_unknown(&mut harness, &mut source).await;
            source.config.auto_replace = false;
            source.config.typing_assist = false;
            source.config.nanda_precognition = false;
            source.set_client_capabilities(41);
            source.set_content_type_state(0, 0);
            let input_mode_before_key = source.layout_gesture.layout_is_ru;
            assert!(legacy_key(&mut harness, &mut source, 27_100, 'a' as u32, 30, 0).await);
            expect_legacy_commit(&mut harness.peer, &source, input_mode_before_key).await;
            assert_eq!(source.committed_tail.buffer, "a");
            actual_reset(&mut harness, &mut source, 27_101, false).await;
            cycle09_surrounding_receipt(&mut harness, &mut source, "a", 1, 1).await;
            assert!(source.context_reset_rereceipt_exact_manual_handoff_allowed());
            let published =
                publish_fixture_append_completion(&mut harness, &mut source, "bcdefgh").await;
            assert_eq!(published, "bcdefgh");

            let native_space_was_visible = source.composition.preedit_visible;
            let native_space_mode = source.layout_gesture.layout_is_ru;
            assert!(legacy_key(&mut harness, &mut source, 27_102, KEY_SPACE, 57, 0).await);
            expect_legacy_managed_space(
                &mut harness,
                &source,
                native_space_mode,
                native_space_was_visible,
            )
            .await;
            assert_eq!(source.committed_tail.buffer, "a ");
            assert!(
                source.context_reset_rereceipt.is_some(),
                "Space must retain the first-word predecessor"
            );
            // Firefox first reports the retired preedit at the old insertion
            // cursor, then resets and reports the actual committed word + Space.
            cycle09_surrounding_receipt(&mut harness, &mut source, retired_surface, cursor, cursor)
                .await;
            if !expected_retention {
                assert!(source.context_reset_rereceipt.is_none());
                continue;
            }
            assert!(
                source.context_reset_rereceipt.is_some(),
                "the retired preedit is not an authority-bearing contradiction"
            );
            assert!(!source.context_reset_rereceipt_exact_manual_handoff_allowed());
            actual_reset(&mut harness, &mut source, 27_103, false).await;
            assert!(source.context_reset_rereceipt.is_some());
            cycle09_surrounding_receipt(&mut harness, &mut source, "a ", 2, 2).await;
            assert!(source.context_reset_rereceipt_exact_manual_handoff_allowed());
            let (source, disposition) = cycle09_manual_toggle(&mut harness, source).await;
            assert_eq!(
                disposition,
                Ok(lay::manual_toggle::ImeManualToggleOutcome::DelegateExactImeTail.as_v3()),
                "the retired preedit must not erase first-word authority before exact text"
            );
            let (source, tail) = cycle09_visible_tail(&mut harness, source).await;
            assert!(cycle09_tail_is_authoritative(
                &tail,
                &source.path,
                false,
                "a "
            ));
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
                firefox_replay_prefix_with_reset(&mut serial, 2, true, "prefix ").await;
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
                    send_manually_dispatched_callback(
                        &mut harness.peer,
                        &message,
                        &engine.path,
                        ENGINE_INTERFACE,
                    )
                    .await;
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

#[test]
fn managed_no_surrounding_command_retires_mirror_before_fresh_owned_word_toggle() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_client_capabilities(9);
        engine.config.auto_replace = false;
        engine.config.auto_switch_layout = true;
        engine.config.nanda_precognition = false;
        assert!(!engine.client_context.surrounding_text_supported);

        for (index, (key, code)) in [('a', 30), ('b', 48), ('c', 46)].into_iter().enumerate() {
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    40_000 + index as u32,
                    key as u32,
                    code,
                    0,
                )
                .await
            );
        }
        let (members, committed) = drain_output_to_text_proof(&mut harness).await;
        assert!(committed.is_empty());
        assert!(!members
            .iter()
            .any(|member| member == "DeleteSurroundingText"));
        assert_eq!(engine.composition.buffer, "abc");
        assert!(engine.composition.legacy_word_preedit_active);

        // Control_L itself commits the owned preedit through the existing
        // non-printable callback. The client then owns Ctrl+A and Backspace;
        // no surrounding receipt proves what those two commands deleted.
        assert!(!legacy_key(&mut harness, &mut engine, 40_003, 0xffe3, 29, 0).await);
        let (members, committed) = drain_output_to_text_proof(&mut harness).await;
        assert_eq!(committed, ["abc"]);
        assert!(!members
            .iter()
            .any(|member| member == "DeleteSurroundingText"));
        assert!(engine.composition.buffer.is_empty());
        assert!(!engine.composition.legacy_word_preedit_active);
        assert_eq!(engine.committed_tail.buffer, "abc");
        assert_eq!(
            engine.composition.word_input_mode,
            Some(WordInputMode::ManagedCommit)
        );

        assert!(!legacy_key(&mut harness, &mut engine, 40_004, 'a' as u32, 30, 1 << 2).await);
        // The old SurroundingText-only guard retains "abc" here.
        assert!(engine.committed_tail.buffer.is_empty());
        assert!(engine.composition.word_input_mode.is_none());
        assert!(engine.client_context.managed_word_start.is_none());
        assert!(engine.client_context.surrounding_text_snapshot.is_none());
        assert!(engine.shared.lock().unwrap().handoff_tail_buffer.is_empty());
        assert!(!legacy_key(&mut harness, &mut engine, 40_005, KEY_BACKSPACE, 14, 0).await);
        let (members, committed) = drain_output_to_text_proof(&mut harness).await;
        assert!(committed.is_empty());
        assert!(!members
            .iter()
            .any(|member| member == "DeleteSurroundingText"));
        assert!(engine.committed_tail.buffer.is_empty());
        assert!(!engine.context_word_is_known());
        assert!(!engine.context_allows_manual_toggle());
        assert!(!engine.context_owns_active_preedit_manual_toggle());
        let (mut engine, outcome) = cycle09_manual_toggle(&mut harness, engine).await;
        assert_eq!(
            outcome,
            Ok(lay::manual_toggle::ImeManualToggleOutcome::NotHandled.as_v3())
        );

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
                legacy_key(
                    &mut harness,
                    &mut engine,
                    40_006 + index as u32,
                    key as u32,
                    code,
                    0,
                )
                .await
            );
        }
        let (members, committed) = drain_output_to_text_proof(&mut harness).await;
        assert!(committed.is_empty());
        assert!(!members
            .iter()
            .any(|member| member == "DeleteSurroundingText"));
        assert_eq!(engine.composition.buffer, "ghbdtn");
        assert_eq!(engine.committed_tail.buffer, "ghbdtn");
        assert!(engine.composition.legacy_word_preedit_active);
        assert!(!engine.context_word_is_known());
        assert!(!engine.context_allows_manual_toggle());
        assert!(engine.context_owns_active_preedit_manual_toggle());

        // Actual bridge admission and the existing exact owned-preedit edit
        // route must succeed; the helper forbids CommitText/DeleteSurroundingText.
        for (expected_ru, expected) in [(true, "привет"), (false, "ghbdtn")] {
            let (toggled, outcome) = cycle09_manual_toggle(&mut harness, engine).await;
            engine = toggled;
            assert_eq!(
                outcome,
                Ok(lay::manual_toggle::ImeManualToggleOutcome::handled(expected_ru).as_v3())
            );
            assert_eq!(engine.composition.buffer, expected);
            assert_eq!(engine.committed_tail.buffer, expected);
            assert_eq!(engine.composition.cursor, expected.chars().count());
            assert_eq!(engine.layout_gesture.layout_is_ru, expected_ru);
            assert!(engine.composition.preedit_visible);
            assert!(engine.composition.legacy_word_preedit_active);
            assert!(engine.context_owns_active_preedit_manual_toggle());
            assert!(!engine.context_word_is_known());
        }
    }));
}

#[test]
fn terminal_passthrough_command_preserves_native_mirror_without_owned_preedit_grant() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_content_type_state(10, 0);
        engine.set_client_capabilities(9);
        engine.client_context.cursor_cell_width = 11;
        engine.config.auto_replace = false;
        engine.config.nanda_precognition = false;

        for (index, (key, code)) in [('a', 30), ('b', 48), ('c', 46)].into_iter().enumerate() {
            assert!(
                !legacy_key(
                    &mut harness,
                    &mut engine,
                    40_100 + index as u32,
                    key as u32,
                    code,
                    0,
                )
                .await
            );
        }
        assert_eq!(engine.committed_tail.buffer, "abc");
        assert_eq!(
            engine.composition.word_input_mode,
            Some(WordInputMode::TerminalPassthrough)
        );
        assert!(engine.composition.buffer.is_empty());
        assert!(!engine.composition.legacy_word_preedit_active);
        assert!(!legacy_key(&mut harness, &mut engine, 40_103, 'a' as u32, 30, 1 << 2).await);
        assert_eq!(engine.committed_tail.buffer, "abc");
        assert_eq!(
            engine.composition.word_input_mode,
            Some(WordInputMode::TerminalPassthrough)
        );
        assert!(!engine.context_word_is_known());
        assert!(!engine.context_allows_manual_toggle());
        assert!(!engine.context_owns_active_preedit_manual_toggle());

        assert!(!legacy_key(&mut harness, &mut engine, 40_104, KEY_BACKSPACE, 14, 0).await);
        assert_eq!(engine.committed_tail.buffer, "ab");
        assert!(!legacy_key(&mut harness, &mut engine, 40_105, 'd' as u32, 32, 0).await);
        assert_eq!(engine.committed_tail.buffer, "abd");
        assert_eq!(
            engine.composition.word_input_mode,
            Some(WordInputMode::TerminalPassthrough)
        );
        assert!(!engine.context_owns_active_preedit_manual_toggle());
        let (members, committed) = drain_output_to_text_proof(&mut harness).await;
        assert!(committed.is_empty());
        assert!(!members
            .iter()
            .any(|member| member == "DeleteSurroundingText"));
        assert!(!members.iter().any(|member| member == "UpdatePreeditText"));
    }));
}

async fn c09_caps9_installed_mode_release_case(transfer: bool) {
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = new_engine(&harness);
    start_source_free_unknown(&mut harness, &mut engine).await;
    engine.set_client_capabilities(9);
    engine.config.nanda_precognition = false;
    if transfer {
        // ADR native-space-observed-boundary: metadata still precedes the
        // native boundary. Require exactly the real InputMode publication and
        // no text effect; no arbitrary signal or property-count waiver.
        let input_mode_before_key = engine.layout_gesture.layout_is_ru;
        assert!(!engine.composition.preedit_visible);
        assert!(!legacy_key(&mut harness, &mut engine, 41_000, KEY_SPACE, 57, 0).await);
        let initial = super::terminal_delivery::legacy_effects(&mut harness).await;
        let members: Vec<_> = initial
            .iter()
            .map(|message| message.header().member().unwrap().as_str().to_string())
            .collect();
        assert_eq!(members, ["UpdateProperty"]);
        assert_input_mode_update(&initial[0], &engine, input_mode_before_key);
        assert_eq!(engine.committed_tail.buffer, " ");
        assert!(engine.context_word_is_known());
        legacy_key(
            &mut harness,
            &mut engine,
            41_003,
            KEY_SPACE,
            57,
            RELEASE_MASK,
        )
        .await;
        let released = super::terminal_delivery::legacy_effects(&mut harness).await;
        assert!(!released
            .iter()
            .any(|message| message.header().member().unwrap().as_str() == "UpdateProperty"));
        assert!(engine.context_word_is_known());
        leave_context(&mut harness, &mut engine).await;
        let focus_in = receive(&mut harness, 41_004, "FocusInId").await;
        let observed = harness
            .adapter
            .observe_callback(&focus_in.header(), Instant::now())
            .await
            .unwrap();
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
                .map(|ready| &ready.outcome),
            Some(ActivationOutcome::Transfer(_))
        ));
    }
    assert!(!engine.client_context.surrounding_text_supported);
    assert!(engine.client_context.surrounding_text_snapshot.is_none());
    let before_mode = engine.layout_gesture.layout_is_ru;
    let before_tail = engine.committed_tail.buffer.clone();
    let expected_owner = harness.adapter.current_owner().unwrap();
    let _ = super::terminal_delivery::legacy_effects(&mut harness).await;

    // The already sent Space release is sufficient. This is the actual
    // production legacy callback with an observed stamp and native emitter;
    // no SetSurroundingText or next press is supplied.
    legacy_key(
        &mut harness,
        &mut engine,
        41_001,
        KEY_SPACE,
        57,
        RELEASE_MASK,
    )
    .await;
    let effects = super::terminal_delivery::legacy_effects(&mut harness).await;
    let updates: Vec<_> = effects
        .iter()
        .filter(|message| message.header().member().unwrap().as_str() == "UpdateProperty")
        .collect();
    assert_eq!(updates.len(), 1, "installed owner must publish InputMode");
    let header = updates[0].header();
    assert_eq!(header.path().unwrap().as_str(), engine.path);
    assert_eq!(header.interface().unwrap().as_str(), ENGINE_INTERFACE);
    let body = updates[0].body();
    let (value,): (zbus::zvariant::Value<'_>,) = body.deserialize().unwrap();
    let zbus::zvariant::Value::Structure(property) = value else {
        panic!("UpdateProperty must carry a single IBusProperty");
    };
    let fields = property.fields();
    assert_eq!(fields.len(), 12);
    assert!(matches!(&fields[2], zbus::zvariant::Value::Str(key) if key.as_str() == "InputMode"));
    let zbus::zvariant::Value::Value(symbol) = &fields[11] else {
        panic!("InputMode symbol must be an IBusText variant");
    };
    assert_eq!(
        crate::ibus_interface::ibus_text_value_to_string(symbol.as_ref()).as_deref(),
        Some(if before_mode { "RU" } else { "EN" })
    );
    assert!(!effects.iter().any(|message| matches!(
        message.header().member().unwrap().as_str(),
        "CommitText" | "DeleteSurroundingText" | "ForwardKeyEvent"
    )));
    assert_eq!(engine.context_owner.as_ref(), Some(&expected_owner));
    assert_eq!(engine.layout_gesture.layout_is_ru, before_mode);
    assert_eq!(engine.committed_tail.buffer, before_tail);
    assert!(engine.client_context.surrounding_text_snapshot.is_none());
    assert!(!engine.client_context.input_mode_property_refresh_pending);

    legacy_key(
        &mut harness,
        &mut engine,
        41_002,
        KEY_SPACE,
        57,
        RELEASE_MASK,
    )
    .await;
    let repeated = super::terminal_delivery::legacy_effects(&mut harness).await;
    assert!(!repeated
        .iter()
        .any(|message| { message.header().member().unwrap().as_str() == "UpdateProperty" }));
}

#[test]
fn c09_source_free_caps9_release_publishes_installed_mode_without_surrounding() {
    zbus::block_on(bounded(c09_caps9_installed_mode_release_case(false)));
}

#[test]
fn c09_transfer_caps9_release_publishes_installed_mode_without_surrounding() {
    zbus::block_on(bounded(c09_caps9_installed_mode_release_case(true)));
}

#[test]
fn c09_post_install_mode_refuses_revoked_or_mismatched_owner() {
    zbus::block_on(bounded(async {
        for loss in ["revoked", "path", "shared_path", "shared_generation"] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = new_engine(&harness);
            start_source_free_unknown(&mut harness, &mut engine).await;
            engine.set_client_capabilities(9);
            engine.config.nanda_precognition = false;
            // Receive the actual release callback and install its authenticated
            // owner before the production publisher can consume pending mode.
            let release = receive(&mut harness, 41_100, "ProcessKeyEvent").await;
            assert!(engine
                .begin_context_key_callback(&release.header(), Instant::now(), false)
                .await
                .is_some());
            let owner = engine
                .context_owner
                .clone()
                .expect("installed release owner");
            assert_eq!(harness.adapter.current_owner().as_ref(), Some(&owner));
            assert!(engine.live_context_token().is_some());
            assert!(engine.client_context.input_mode_property_refresh_pending);
            let before_mode = engine.layout_gesture.layout_is_ru;
            match loss {
                "revoked" => assert!(harness.adapter.revoke_current_owner(&owner)),
                "path" => engine.path = "/engine/foreign".to_string(),
                "shared_path" => {
                    engine.shared.lock().unwrap().active_path = Some("/engine/foreign".into());
                }
                "shared_generation" => {
                    engine.shared.lock().unwrap().context_owner_generation =
                        Some(owner.generation.0 + 1);
                }
                _ => unreachable!(),
            }
            // Do not begin a new activation after corruption. Exercise the
            // same production publisher directly on the installed owner.
            let emitter = zbus::object_server::SignalEmitter::new(&harness.connection, TARGET_PATH)
                .expect("legacy signal emitter");
            engine
                .publish_pending_input_mode_property(&mut EngineOutput::legacy(&emitter))
                .await;
            let effects = super::terminal_delivery::legacy_effects(&mut harness).await;
            assert!(
                !effects.iter().any(|message| {
                    message.header().member().unwrap().as_str() == "UpdateProperty"
                }),
                "must not publish stale mode: {loss}"
            );
            assert_eq!(engine.layout_gesture.layout_is_ru, before_mode);
        }
    }));
}

#[test]
fn c09_post_install_mode_transport_failure_preserves_authority_and_pending() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_client_capabilities(9);
        // The existing install path establishes pending mode without running
        // a successful publisher before transport-failure injection.
        engine.try_install_pending_context_activation();
        assert!(engine.context_owner.is_some());
        assert_eq!(engine.context_owner, harness.adapter.current_owner());
        assert!(engine.live_context_token().is_some());
        assert!(engine.client_context.input_mode_property_refresh_pending);
        let owner = engine.context_owner.clone();
        let token = engine.context_token.clone();
        let tail = engine.committed_tail.buffer.clone();
        let mode = engine.layout_gesture.layout_is_ru;
        let mut failed = crate::output::TestEngineOutput {
            legacy_transport: true,
            fail_input_mode_publication: true,
            ..Default::default()
        };
        // Use the production passive callback route; zero-width geometry
        // does not request any external window probe or text observation.
        crate::window_interaction::WindowInteraction::observe_facts(
            &mut engine,
            crate::window_interaction::WindowFactEvent::CursorGeometry {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            Some(&mut EngineOutput::test(&mut failed)),
        )
        .await
        .unwrap();
        assert!(failed.input_mode_updates.is_empty());
        assert!(engine.client_context.input_mode_property_refresh_pending);
        assert_eq!(engine.context_owner, owner);
        assert_eq!(engine.context_token, token);
        assert_eq!(engine.committed_tail.buffer, tail);
        assert_eq!(engine.layout_gesture.layout_is_ru, mode);
        let mut output = crate::output::TestEngineOutput {
            legacy_transport: true,
            ..Default::default()
        };
        engine
            .publish_pending_input_mode_property(&mut EngineOutput::test(&mut output))
            .await;
        assert_eq!(output.input_mode_updates, [mode]);
        assert!(!engine.client_context.input_mode_property_refresh_pending);
    }));
}

#[test]
fn c06_native_backspace_preserves_only_inert_confirmed_reset_predecessor() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine =
            initial_observed_tail_reset(&mut harness, 21_000, &[('a', 30), ('b', 48)]).await;
        exact_surrounding_receipt(&mut harness, &mut engine, "ab").await;
        assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        assert_eq!(
            engine
                .context_word_scope
                .as_ref()
                .unwrap()
                .lineage()
                .observed_suffix_chars,
            0
        );
        let prior = engine.context_reset_rereceipt.as_ref().unwrap().clone();
        assert!(prior.confirmed);
        assert_eq!(prior.observed_suffix_chars, 2);
        assert!(engine.composition.buffer.is_empty());
        publish_fixture_append_completion(&mut harness, &mut engine, "cde").await;
        let epoch = engine.committed_tail.epoch;
        assert!(!legacy_key(&mut harness, &mut engine, 21_020, KEY_BACKSPACE, 14, 0).await);
        assert_eq!(engine.committed_tail.buffer, "a");
        assert_eq!(engine.committed_tail.epoch, epoch.wrapping_add(1));
        let effects = super::terminal_delivery::legacy_effects(&mut harness).await;
        assert_eq!(
            effects.len(),
            2,
            "published native hint must clear exactly once"
        );
        assert_eq!(
            effects[0].header().member().unwrap().as_str(),
            "UpdatePreeditText"
        );
        assert_eq!(
            effects[1].header().member().unwrap().as_str(),
            "HidePreeditText"
        );
        for effect in &effects {
            let header = effect.header();
            let member = header.member().unwrap().as_str();
            assert!(
                matches!(member, "UpdatePreeditText" | "HidePreeditText"),
                "native BSP emitted {member}"
            );
            if member == "UpdatePreeditText" {
                let body = effect.body();
                let (text, _, visible, _) = body
                    .deserialize::<(zbus::zvariant::Value<'_>, u32, bool, u32)>()
                    .unwrap();
                assert_eq!(
                    crate::ibus_interface::ibus_text_value_to_string(&text).as_deref(),
                    Some("")
                );
                assert!(!visible);
            }
        }
        eprintln!(
            "NATIVE_BSP old_count={} scope={} pending={:?}",
            prior.observed_suffix_chars,
            engine
                .context_word_scope
                .as_ref()
                .unwrap()
                .lineage()
                .observed_suffix_chars,
            engine.context_reset_rereceipt
        );
        actual_reset(&mut harness, &mut engine, 21_021, false).await;
        eprintln!(
            "NATIVE_RESET scope={} pending={:?}",
            engine
                .context_word_scope
                .as_ref()
                .unwrap()
                .lineage()
                .observed_suffix_chars,
            engine.context_reset_rereceipt
        );
        assert!(
            engine.live_context_token().is_some(),
            "Reset must install a live settled token independently of the lost pending range"
        );
        exact_surrounding_receipt(&mut harness, &mut engine, "a").await;
        assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed(), "fresh exact shortened receipt lost confirmed predecessor (old_count=2, settled scope=0)");
        publish_fixture_append_completion(&mut harness, &mut engine, "bcde").await;
        assert!(legacy_key(&mut harness, &mut engine, 21_022, KEY_TAB, 15, 0).await);
        let effects = super::terminal_delivery::legacy_effects(&mut harness).await;
        let commits: Vec<_> = effects
            .iter()
            .filter(|effect| effect.header().member().unwrap().as_str() == "CommitText")
            .map(|effect| {
                let body = effect.body();
                let (text,) = body.deserialize::<(zbus::zvariant::Value<'_>,)>().unwrap();
                crate::ibus_interface::ibus_text_value_to_string(&text).unwrap()
            })
            .collect();
        assert_eq!(commits, ["bcde "]);
        assert!(effects
            .iter()
            .all(|effect| effect.header().member().unwrap().as_str() != "DeleteSurroundingText"));
        assert_eq!(engine.committed_tail.buffer, "abcde ");
    }));
}

async fn c06_native_confirmed_prefix_fixture(
    harness: &mut Harness,
    serial: u32,
    later_word: bool,
    russian: bool,
) -> (LayIbusEngine, String) {
    let mut engine = new_engine(harness);
    engine.layout_gesture.layout_is_ru = russian;
    start_source_free_unknown(harness, &mut engine).await;
    engine.config.auto_replace = false;
    engine.config.typing_assist = false;
    engine.config.nanda_precognition = false;
    engine.client_context.surrounding_text_supported = true;
    let keys: &[(char, u32)] = if later_word {
        &[('c', 46), (' ', 57), ('a', 30), ('b', 48)]
    } else {
        &[('a', 30), ('b', 48)]
    };
    for (i, &(ch, code)) in keys.iter().enumerate() {
        let mode = engine.layout_gesture.layout_is_ru;
        let native_space_was_visible = engine.composition.preedit_visible;
        let handled = legacy_key(
            harness,
            &mut engine,
            serial + i as u32 * 2,
            ch as u32,
            code,
            0,
        )
        .await;
        if ch == ' ' {
            assert!(
                handled,
                "ordinary ManagedCommit separator stays in the commit stream"
            );
            expect_legacy_managed_space(harness, &engine, mode, native_space_was_visible).await;
        } else {
            assert!(handled);
            expect_legacy_commit(&mut harness.peer, &engine, mode).await;
        }
        let _ = legacy_key(
            harness,
            &mut engine,
            serial + i as u32 * 2 + 1,
            ch as u32,
            code,
            RELEASE_MASK,
        )
        .await;
    }
    let prefix = match (later_word, russian) {
        (false, false) => "ab",
        (true, false) => "c ab",
        (false, true) => "фи",
        (true, true) => "с фи",
    }
    .to_string();
    assert_eq!(engine.committed_tail.buffer, prefix);
    engine.committed_tail.last_input_at = Some(Instant::now());
    actual_reset(harness, &mut engine, serial + 10, false).await;
    exact_surrounding_receipt(harness, &mut engine, &prefix).await;
    assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
    assert_eq!(
        engine
            .context_reset_rereceipt
            .as_ref()
            .unwrap()
            .observed_suffix_chars,
        2
    );
    assert_eq!(
        engine
            .context_word_scope
            .as_ref()
            .unwrap()
            .lineage()
            .observed_suffix_chars,
        0
    );
    (engine, prefix)
}

async fn c06_native_backspace_clear_only(harness: &mut Harness) {
    let effects = super::terminal_delivery::legacy_effects(harness).await;
    assert_eq!(
        effects.len(),
        2,
        "one native empty update and one hide, no work republished during deletion"
    );
    assert_eq!(
        effects[0].header().member().unwrap().as_str(),
        "UpdatePreeditText"
    );
    assert_eq!(
        effects[1].header().member().unwrap().as_str(),
        "HidePreeditText"
    );
    let body = effects[0].body();
    let (text, cursor, visible, mode) = body
        .deserialize::<(zbus::zvariant::Value<'_>, u32, bool, u32)>()
        .unwrap();
    assert_eq!(
        crate::ibus_interface::ibus_text_value_to_string(&text).as_deref(),
        Some("")
    );
    assert_eq!((cursor, visible, mode), (0, false, 0));
}

#[test]
fn c06_native_backspace_retype_first_and_later_words_uses_fresh_client_receipts() {
    zbus::block_on(bounded(async {
        for (later_word, russian) in [(false, false), (true, false), (false, true), (true, true)] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let (mut engine, prefix) =
                c06_native_confirmed_prefix_fixture(&mut harness, 22_000, later_word, russian)
                    .await;
            let predecessor = engine
                .context_reset_rereceipt
                .as_ref()
                .unwrap()
                .predecessor_token
                .clone();
            let epoch = engine.committed_tail.epoch;
            publish_fixture_append_completion(&mut harness, &mut engine, "xyz").await;
            engine.config.nanda_precognition = true;
            assert!(!legacy_key(&mut harness, &mut engine, 22_020, KEY_BACKSPACE, 14, 0).await);
            c06_native_backspace_clear_only(&mut harness).await;
            assert!(!engine.composition.preedit_display_only_pending);
            assert!(engine.composition.pending_display_frame.is_none());
            engine.config.nanda_precognition = false;
            let shortened: String = prefix.chars().take(prefix.chars().count() - 1).collect();
            assert_eq!(engine.committed_tail.buffer, shortened);
            assert_eq!(engine.committed_tail.epoch, epoch + 1);
            let pending = engine.context_reset_rereceipt.as_ref().unwrap();
            assert!(!pending.confirmed);
            assert_eq!(pending.predecessor_token, predecessor);
            assert_eq!(pending.observed_suffix_chars, 1);
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            let _ = legacy_key(
                &mut harness,
                &mut engine,
                22_021,
                KEY_BACKSPACE,
                14,
                RELEASE_MASK,
            )
            .await;
            assert!(super::terminal_delivery::legacy_effects(&mut harness)
                .await
                .is_empty());
            actual_reset(&mut harness, &mut engine, 22_022, false).await;
            assert!(engine.live_context_token().is_some());
            exact_surrounding_receipt(&mut harness, &mut engine, &shortened).await;
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            let mode = engine.layout_gesture.layout_is_ru;
            assert!(legacy_key(&mut harness, &mut engine, 22_023, 'b' as u32, 48, 0).await);
            expect_legacy_commit(&mut harness.peer, &engine, mode).await;
            assert_eq!(engine.committed_tail.buffer, prefix);
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            actual_reset(&mut harness, &mut engine, 22_024, false).await;
            exact_surrounding_receipt(&mut harness, &mut engine, &prefix).await;
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            publish_fixture_append_completion(&mut harness, &mut engine, "xyz").await;
            assert!(legacy_key(&mut harness, &mut engine, 22_025, KEY_TAB, 15, 0).await);
            let effects = super::terminal_delivery::legacy_effects(&mut harness).await;
            let commits: Vec<_> = effects
                .iter()
                .filter(|e| e.header().member().unwrap().as_str() == "CommitText")
                .map(|e| {
                    let body = e.body();
                    let (text,) = body.deserialize::<(zbus::zvariant::Value<'_>,)>().unwrap();
                    crate::ibus_interface::ibus_text_value_to_string(&text).unwrap()
                })
                .collect();
            assert_eq!(commits, ["xyz "]);
            assert!(effects
                .iter()
                .all(|e| e.header().member().unwrap().as_str() != "DeleteSurroundingText"));
            assert_eq!(engine.committed_tail.buffer, format!("{prefix}xyz "));
            assert!(engine.composition.preedit_suffix.is_empty());
        }
    }));
}

#[test]
fn c06_native_backspace_carry_cannot_accept_before_fresh_exact_client_confirmation() {
    zbus::block_on(bounded(async {
        for fresh_before_tab in [true, false] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let (mut engine, _) =
                c06_native_confirmed_prefix_fixture(&mut harness, 23_000, false, false).await;
            publish_fixture_append_completion(&mut harness, &mut engine, "cde").await;
            assert!(!legacy_key(&mut harness, &mut engine, 23_020, KEY_BACKSPACE, 14, 0).await);
            c06_native_backspace_clear_only(&mut harness).await;
            assert!(engine
                .context_reset_rereceipt
                .as_ref()
                .is_some_and(|p| !p.confirmed));
            assert!(!engine.context_word_is_known());
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            if fresh_before_tab {
                // The actual client receipt supplies the shortened range. A
                // separate Reset is tested elsewhere but is not mandatory.
                exact_surrounding_receipt(&mut harness, &mut engine, "a").await;
                assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                assert!(!engine.context_word_is_known());
            } else {
                assert!(!legacy_key(&mut harness, &mut engine, 23_021, KEY_TAB, 15, 0).await);
                let effects = super::terminal_delivery::legacy_effects(&mut harness).await;
                assert!(effects.iter().all(|e| !matches!(
                    e.header().member().unwrap().as_str(),
                    "CommitText" | "DeleteSurroundingText"
                )));
                assert_eq!(engine.committed_tail.buffer, "a");
                assert!(engine.committed_tail.pending_completion_learning.is_none());
                assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
                // A refused client-side Tab is an input gap; the old shortened
                // provenance cannot be revived by matching text after that gap.
                exact_surrounding_receipt(&mut harness, &mut engine, "a").await;
                assert!(engine.context_reset_rereceipt.is_none());
                assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            }
        }
    }));
}

#[test]
fn c06_native_backspace_refuses_unconfirmed_drift_selection_and_whole_erase() {
    zbus::block_on(bounded(async {
        for gap in [
            "unconfirmed",
            "wrong_text",
            "selection",
            "middle_caret",
            "epoch",
            "focus",
            "capability",
            "whole",
            "repeat",
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let keys: &[(char, u32)] = if gap == "whole" {
                &[('a', 30)]
            } else {
                &[('a', 30), ('b', 48), ('c', 46)]
            };
            let mut engine = initial_observed_tail_reset(&mut harness, 24_000, keys).await;
            let prefix: String = keys.iter().map(|(ch, _)| *ch).collect();
            if gap != "unconfirmed" {
                exact_surrounding_receipt(&mut harness, &mut engine, &prefix).await;
            }
            match gap {
                "wrong_text" => exact_surrounding_receipt(&mut harness, &mut engine, "abx").await,
                "selection" => surrounding_receipt(&mut harness, &mut engine, &prefix, 3, 1).await,
                "middle_caret" => {
                    surrounding_receipt(&mut harness, &mut engine, "abc rest", 3, 3).await
                }
                "epoch" => engine.committed_tail.epoch += 1,
                "focus" => actual_focus_out(&mut harness, &mut engine, 24_010).await,
                "capability" => engine.set_client_capabilities(1 | 1 << 3),
                _ => (),
            }
            let _ = legacy_key(&mut harness, &mut engine, 24_020, KEY_BACKSPACE, 14, 0).await;
            let effects = super::terminal_delivery::legacy_effects(&mut harness).await;
            assert!(
                effects.iter().all(|e| !matches!(
                    e.header().member().unwrap().as_str(),
                    "CommitText" | "DeleteSurroundingText"
                )),
                "gap {gap}"
            );
            if gap == "repeat" {
                assert!(engine
                    .context_reset_rereceipt
                    .as_ref()
                    .is_some_and(|p| !p.confirmed));
                let _ = legacy_key(&mut harness, &mut engine, 24_021, KEY_BACKSPACE, 14, 0).await;
                let effects = super::terminal_delivery::legacy_effects(&mut harness).await;
                assert!(effects.iter().all(|e| !matches!(
                    e.header().member().unwrap().as_str(),
                    "CommitText" | "DeleteSurroundingText"
                )));
            }
            assert!(
                engine.context_reset_rereceipt.is_none(),
                "gap {gap} must retire native carried provenance"
            );
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "gap {gap}"
            );
        }
    }));
}

// Private NOT_RUN append to the already-combined observer-first fixture.
// All source/token/lineage effects enter through actual adapter/native handlers.
async fn firefox_receive_deferred_content_type(
    harness: &mut Harness,
    engine: &LayIbusEngine,
    serial: &mut u32,
    value: (u32, u32),
) -> Message {
    let message = Message::method_call(engine.path.as_str(), "Set")
        .unwrap()
        .interface(PROPERTIES_INTERFACE)
        .unwrap()
        .sender(DISPATCH_SENDER)
        .unwrap()
        .serial(NonZeroU32::new(*serial).unwrap())
        .with_flags(zbus::message::Flags::NoReplyExpected)
        .unwrap()
        .build(&(
            ENGINE_INTERFACE,
            "ContentType",
            zbus::zvariant::Value::from(value),
        ))
        .unwrap();
    send_manually_dispatched_callback(
        &mut harness.peer,
        &message,
        engine.path.as_str(),
        PROPERTIES_INTERFACE,
    )
    .await;
    assert!(harness.observer.process_next().await.unwrap());
    consume_detached_callback_reply(&mut harness.peer, *serial).await;
    *serial += 1;
    message
}

async fn firefox_assert_no_recovered_manual_authority(
    harness: &mut Harness,
    engine: LayIbusEngine,
    stage: &str,
) {
    assert!(
        !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
        "{stage}"
    );
    assert!(!engine.context_word_is_known(), "{stage}");
    assert!(
        engine.committed_tail.pending_completion_learning.is_none(),
        "{stage}"
    );
    assert!(
        crate::tail_memory::take_accepted_completion_feedback().is_empty(),
        "{stage}"
    );
    let (engine, disposition) = cycle09_manual_toggle(harness, engine).await;
    assert!(
        matches!(disposition, Ok((0, _)) | Err(_)),
        "{stage}: {disposition:?}"
    );
    assert!(!cycle09_exact_handoff_is_live(&engine), "{stage}");
    cycle09_assert_no_local_text_effect(harness).await;
}

async fn firefox_completed_replay_content_type_gap(changed_back: bool) {
    let mut serial = 34_000;
    let (mut harness, mut engine, _, _, _) = firefox_observer_ahead_six_inserts(&mut serial).await;
    assert_eq!(engine.client_context.content_purpose, 0);
    let old = engine.committed_tail.autocorrect_suppression.clone();
    let Some(crate::protocol::AutocorrectSuppression::ExactReplay(replay)) = old else {
        panic!("fixture requires the actual complete replay");
    };
    let source = replay
        .source_token
        .clone()
        .expect("actual admitted replay seed");
    let first =
        firefox_receive_deferred_content_type(&mut harness, &engine, &mut serial, (1, 0)).await;
    let second = if changed_back {
        Some(
            firefox_receive_deferred_content_type(&mut harness, &engine, &mut serial, (0, 0)).await,
        )
    } else {
        None
    };
    // ContentType observer ingress invalidates provenance before local setters.
    assert_eq!(engine.client_context.content_purpose, 0);
    assert!(!harness
        .adapter
        .exact_replay_reset_provenance_is_current(&source));
    let reset = firefox_receive_deferred_reset(&mut harness, &engine, &mut serial).await;
    engine
        .reset(
            reset.header(),
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap(),
        )
        .await
        .unwrap();
    cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix привет", 13, 13).await;
    assert!(
        !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
        "completed replay crossed observer-ahead changed ContentType, changed_back={changed_back}",
    );
    // Dispatch the delayed setters using their original received headers too.
    engine.set_content_type((1, 0), Some(first.header())).await;
    if let Some(second) = second {
        engine.set_content_type((0, 0), Some(second.header())).await;
        assert_eq!(engine.client_context.content_purpose, 0);
    }
    cycle09_assert_no_local_text_effect(&mut harness).await;
    firefox_assert_no_recovered_manual_authority(&mut harness, engine, "content_type_gap").await;
}

#[test]
fn firefox_completed_replay_reset_rejects_observer_ahead_content_type_change() {
    zbus::block_on(bounded(firefox_completed_replay_content_type_gap(false)));
}

#[test]
fn firefox_completed_replay_reset_rejects_content_type_change_away_and_back() {
    zbus::block_on(bounded(firefox_completed_replay_content_type_gap(true)));
}

#[test]
fn firefox_completed_replay_reset_rejects_malformed_key_ingress_and_later_reset() {
    zbus::block_on(bounded(async {
        let mut serial = 35_000;
        let (mut harness, mut engine, path, _, _) =
            firefox_observer_ahead_six_inserts(&mut serial).await;
        eprintln!("MALFORMED_REPLAY stage=fixture_complete serial={serial}");
        let malformed = Message::method_call(path.as_str(), "ProcessKeyEvent")
            .unwrap()
            .interface(ENGINE_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(serial).unwrap())
            .with_flags(zbus::message::Flags::NoReplyExpected)
            .unwrap()
            .build(&())
            .unwrap();
        eprintln!("MALFORMED_REPLAY stage=before_malformed_send serial={serial}");
        send_manually_dispatched_callback(
            &mut harness.peer,
            &malformed,
            path.as_str(),
            ENGINE_INTERFACE,
        )
        .await;
        eprintln!("MALFORMED_REPLAY stage=after_malformed_send serial={serial}");
        assert!(matches!(
            harness.observer.process_next().await,
            Err(AdapterError::Denied)
        ));
        eprintln!("MALFORMED_REPLAY stage=malformed_observer_denied serial={serial}");
        consume_detached_callback_reply(&mut harness.peer, serial).await;
        eprintln!("MALFORMED_REPLAY stage=malformed_reply_consumed serial={serial}");
        serial += 1;
        assert!(harness.adapter.current_owner().is_none());
        assert!(harness.adapter.current_token().is_none());

        let reset = observed_callback_without_reply(serial, &path, "Reset");
        eprintln!("MALFORMED_REPLAY stage=before_reset_send serial={serial}");
        send_manually_dispatched_callback(&mut harness.peer, &reset, &path, ENGINE_INTERFACE).await;
        eprintln!("MALFORMED_REPLAY stage=after_reset_send serial={serial}");
        assert!(matches!(
            harness.observer.process_next().await,
            Err(AdapterError::Cancelled)
        ));
        eprintln!("MALFORMED_REPLAY stage=reset_observer_cancelled serial={serial}");
        consume_detached_callback_reply(&mut harness.peer, serial).await;
        eprintln!("MALFORMED_REPLAY stage=reset_reply_consumed serial={serial}");
        engine
            .reset(
                reset.header(),
                zbus::object_server::SignalEmitter::new(&harness.connection, path.clone()).unwrap(),
            )
            .await
            .unwrap();
        eprintln!("MALFORMED_REPLAY stage=reset_handler_complete serial={serial}");
        cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix привет", 13, 13).await;
        eprintln!("MALFORMED_REPLAY stage=surrounding_complete serial={serial}");
        assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        assert!(!engine.context_word_is_known());
        assert!(engine.committed_tail.pending_completion_learning.is_none());
        cycle09_assert_no_local_text_effect(&mut harness).await;

        // Production begins with a real Ping before its cancelled-fence
        // refusal. Echo only that exact Ping; do not wait for a marker or
        // drive the already-cancelled metadata observer. The actual marker,
        // if emitted, is drained by the unchanged no-text-effect FIFO proof.
        eprintln!("MALFORMED_REPLAY stage=negative_assertions_complete serial={serial}");
        let bridge = bridge(&harness, &engine);
        harness
            .connection
            .object_server()
            .at(path.as_str(), engine)
            .await
            .unwrap();
        eprintln!("MALFORMED_REPLAY stage=before_cancelled_bridge serial={serial}");
        let (disposition, ()) = bounded(future::zip(bridge.manual_toggle_v3_inner(), async {
            let call = bounded(next_peer_message(&mut harness.peer)).await;
            let header = call.header();
            assert_eq!(header.message_type(), Type::MethodCall);
            assert_eq!(header.path().map(|path| path.as_str()), Some(IBUS_PATH));
            assert_eq!(
                header.interface().map(|interface| interface.as_str()),
                Some(IBUS_INTERFACE)
            );
            assert_eq!(header.member().map(|member| member.as_str()), Some("Ping"));
            let value = call.body().deserialize::<OwnedValue>().unwrap();
            harness
                .peer
                .connection
                .reply(&header, &value)
                .await
                .unwrap();
            eprintln!("MALFORMED_REPLAY stage=actual_ping_echoed serial={serial}");
        }))
        .await;
        eprintln!("MALFORMED_REPLAY stage=after_cancelled_bridge serial={serial}");
        assert!(
            matches!(&disposition, Err(zbus::fdo::Error::Failed(reason))
                if reason == &AdapterError::Cancelled.to_string()),
            "malformed observer gap must produce the actual cancelled-fence refusal: {disposition:?}"
        );
        assert!(harness.adapter.current_owner().is_none());
        assert!(harness.adapter.current_token().is_none());
        assert!(harness.adapter.shared.pending.lock().unwrap().is_none());
        let engine = cycle09_take_registered_engine(&harness, &path).await;
        eprintln!("MALFORMED_REPLAY stage=engine_taken serial={serial}");
        assert!(!cycle09_exact_handoff_is_live(&engine));
        assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
        cycle09_assert_no_local_text_effect(&mut harness).await;
        eprintln!("MALFORMED_REPLAY stage=complete serial={serial}");
    }));
}

// Returns the actual deferred Reset before its native handler. This allows
// incomplete/expiry/wrong-key evidence to be inserted before fallback capture.
async fn firefox_replay_before_deferred_reset(
    serial: &mut u32,
    include_sixth: bool,
    wrong_sixth: bool,
) -> (Harness, LayIbusEngine, Message) {
    let (mut harness, target, path, epoch) = firefox_empty_reset_six_scalar_handoff(*serial).await;
    *serial += 40;
    let (mut engine, suppression) =
        cycle09_suppress_exact_replay(&mut harness, target, "ghbdtn", epoch, &path, true).await;
    assert_eq!(suppression, Ok(true));
    engine.config.text_backend = "ime".into();
    engine.config.typing_assist = false;
    engine.config.nanda_precognition = false;
    cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix ghbdtn", 13, 13).await;
    for _ in 0..6 {
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
    firefox_replay_reset(&mut harness, &mut engine, serial).await;
    cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix ", 7, 7).await;
    for (offset, (ch, code)) in [('п', 34), ('р', 35), ('и', 48), ('в', 32)]
        .into_iter()
        .enumerate()
    {
        let keyval = 0x0100_0000 | ch as u32;
        assert!(
            !firefox_replay_callback(&mut harness, &mut engine, serial, (keyval, code, 0),).await
        );
        firefox_empty_reset_assert_unknown_lineage(&engine, offset as u32 + 1, "adverse_prefix");
        if offset < 3 {
            assert!(
                !firefox_replay_callback(
                    &mut harness,
                    &mut engine,
                    serial,
                    (keyval, code, RELEASE_MASK),
                )
                .await
            );
        }
    }
    let mut keys = vec![
        (0x0100_0000 | 'в' as u32, 32, RELEASE_MASK),
        (1733, 20, 0),
        (1733, 20, RELEASE_MASK),
    ];
    if include_sixth {
        let (keyval, code) = if wrong_sixth {
            (b'x' as u32, 45)
        } else {
            (1748, 49)
        };
        keys.push((keyval, code, 0));
        keys.push((keyval, code, RELEASE_MASK));
    }
    let mut messages = Vec::new();
    for key in keys {
        messages
            .push(firefox_receive_deferred_native_key(&mut harness, &engine, serial, key).await);
    }
    let reset = firefox_receive_deferred_reset(&mut harness, &engine, serial).await;
    for (index, message) in messages.iter().enumerate() {
        if wrong_sixth && index >= 3 {
            // A wrong replay key may use the ordinary exact append route.
            // Observe that actual result; do not pretend it remained Native.
            let key: (u32, u32, u32) = message.body().deserialize().unwrap();
            engine
                .process_key_event(
                    message.header(),
                    zbus::object_server::SignalEmitter::new(&harness.connection, path.clone())
                        .unwrap(),
                    key.0,
                    key.1,
                    key.2,
                )
                .await
                .unwrap();
            let effects = super::terminal_delivery::legacy_effects(&mut harness).await;
            assert!(
                effects.iter().all(|effect| {
                    effect
                        .header()
                        .member()
                        .is_none_or(|member| member.as_str() != "DeleteSurroundingText")
                }),
                "wrong replay key must not authorize a deletion"
            );
        } else {
            assert!(
                !firefox_dispatch_deferred_native_key(&mut harness, &mut engine, message).await
            );
        }
    }
    if !wrong_sixth {
        assert_eq!(
            engine.committed_tail.buffer,
            if include_sixth {
                "привет"
            } else {
                "приве"
            }
        );
    }
    assert!(!engine.context_word_is_known());
    assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
    (harness, engine, reset)
}

async fn firefox_finish_adverse_received_reset(
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
    // Exact actual mirror, not a mismatching future expected word.
    let text = format!("prefix {}", engine.committed_tail.buffer);
    let cursor = text.chars().count() as u32;
    cycle09_surrounding_receipt(harness, engine, &text, cursor, cursor).await;
    cycle09_assert_no_local_text_effect(harness).await;
}

#[test]
fn firefox_completed_replay_reset_rejects_incomplete_projection() {
    zbus::block_on(bounded(async {
        let mut serial = 36_000;
        let (mut harness, mut engine, reset) =
            firefox_replay_before_deferred_reset(&mut serial, false, false).await;
        assert_eq!(engine.committed_tail.buffer, "приве");
        firefox_finish_adverse_received_reset(&mut harness, &mut engine, &reset).await;
        firefox_assert_no_recovered_manual_authority(&mut harness, engine, "incomplete_projection")
            .await;
    }));
}

#[test]
fn firefox_completed_replay_reset_rejects_expired_completed_projection() {
    zbus::block_on(bounded(async {
        let mut serial = 37_000;
        let (mut harness, mut engine, reset) =
            firefox_replay_before_deferred_reset(&mut serial, true, false).await;
        let Some(crate::protocol::AutocorrectSuppression::ExactReplay(mut replay)) =
            engine.committed_tail.autocorrect_suppression.clone()
        else {
            panic!("actual complete native replay must still be quarantined");
        };
        // Explicit clock fault injection as existing expiry tests use. Only
        // the deadline changes; source token, epoch, tail and owner are intact.
        replay.expires_at = Instant::now() - Duration::from_millis(1);
        engine.committed_tail.autocorrect_suppression = Some(
            crate::protocol::AutocorrectSuppression::ExactReplay(replay.clone()),
        );
        engine.shared.lock().unwrap().autocorrect_suppression =
            Some(crate::protocol::AutocorrectSuppression::ExactReplay(replay));
        firefox_finish_adverse_received_reset(&mut harness, &mut engine, &reset).await;
        firefox_assert_no_recovered_manual_authority(
            &mut harness,
            engine,
            "expired_complete_projection",
        )
        .await;
    }));
}

#[test]
fn firefox_completed_replay_reset_rejects_wrong_native_replacement_key() {
    zbus::block_on(bounded(async {
        let mut serial = 38_000;
        let (mut harness, mut engine, reset) =
            firefox_replay_before_deferred_reset(&mut serial, true, true).await;
        assert!(!engine.exact_replay_quarantine_active());
        firefox_finish_adverse_received_reset(&mut harness, &mut engine, &reset).await;
        firefox_assert_no_recovered_manual_authority(
            &mut harness,
            engine,
            "wrong_native_replacement_key",
        )
        .await;
    }));
}

// Private prospective append: raw PREEDIT changes only the completed replay seed.
// The independent ordinary deferred-refresh positives remain unchanged.
#[test]
fn firefox_completed_replay_reset_rejects_raw_preedit_only_capability_change() {
    zbus::block_on(bounded(async {
        let mut serial = 39_000;
        let (mut harness, mut engine, path, _, _) =
            firefox_observer_ahead_six_inserts(&mut serial).await;
        assert_eq!(engine.committed_tail.buffer, "привет");
        assert!(!engine.context_word_is_known());
        assert_eq!(
            engine
                .context_word_scope
                .as_ref()
                .unwrap()
                .lineage()
                .observed_suffix_chars,
            0
        );
        assert!(engine.client_context.preedit_text_supported);
        assert!(engine.client_context.surrounding_text_supported);
        assert!(!engine.client_context.exact_surrounding_refresh_available);
        let revision = engine.client_context.surrounding_observation_revision;
        let owner = engine.context_owner.clone();
        let Some(crate::protocol::AutocorrectSuppression::ExactReplay(replay)) =
            engine.committed_tail.autocorrect_suppression.as_ref()
        else {
            panic!("fixture requires actual completed native replay");
        };
        let seed = replay
            .source_token
            .as_deref()
            .expect("actual admitted seed")
            .clone();
        let pending = engine.context_reset_rereceipt.as_ref().unwrap();
        assert_eq!(pending.predecessor_token, seed);
        assert_eq!(pending.token_text, "привет");
        assert_eq!(pending.observed_suffix_chars, 6);
        assert!(!pending.confirmed);

        // Factory handoff sets 41. Remove only PREEDIT: SURROUNDING and FOCUS
        // remain present; derived legacy-preedit and exact-refresh stay false.
        let caps = crate::window_interaction::IBUS_CAP_SURROUNDING_TEXT | (1 << 3);
        assert_eq!(
            41u32 ^ caps,
            crate::window_interaction::IBUS_CAP_PREEDIT_TEXT
        );
        harness
            .connection
            .object_server()
            .at(path.as_str(), engine)
            .await
            .unwrap();
        let message = Message::method_call(path.as_str(), "SetCapabilities")
            .unwrap()
            .interface(ENGINE_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(serial).unwrap())
            .build(&caps)
            .unwrap();
        harness.peer.connection.send(&message).await.unwrap();
        // SetCapabilities is not in ENGINE_CALLBACK_MEMBERS. Dispatch the real
        // object method and consume its actual reply, without an observer wait.
        let reply = bounded(next_peer_message_raw(&mut harness.peer)).await;
        assert_eq!(reply.header().message_type(), Type::MethodReturn);
        assert_eq!(reply.header().reply_serial().map(|n| n.get()), Some(serial));
        reply.body().deserialize::<()>().unwrap();
        serial += 1;
        engine = cycle09_take_registered_engine(&harness, &path).await;
        assert!(!engine.client_context.preedit_text_supported);
        assert!(engine.client_context.surrounding_text_supported);
        assert!(!engine.client_context.exact_surrounding_refresh_available);
        assert_eq!(
            engine.client_context.surrounding_observation_revision,
            revision
        );
        assert_eq!(engine.context_owner, owner);
        assert!(engine.context_reset_rereceipt.is_none());
        assert!(matches!(
            engine.committed_tail.autocorrect_suppression.as_ref(),
            Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope))
                if scope.source_token.is_none()
        ));
        assert!(matches!(
            engine.shared.lock().unwrap().autocorrect_suppression.as_ref(),
            Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope))
                if scope.source_token.is_none()
        ));
        cycle09_assert_no_local_text_effect(&mut harness).await;

        let reset = firefox_receive_deferred_reset(&mut harness, &engine, &mut serial).await;
        engine
            .reset(
                reset.header(),
                zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                    .unwrap(),
            )
            .await
            .unwrap();
        cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix привет", 13, 13).await;
        assert!(engine.context_reset_rereceipt.is_none());
        let (engine, readout) = cycle09_visible_tail(&mut harness, engine).await;
        assert!(
            !cycle09_tail_is_authoritative(&readout, &path, true, "привет"),
            "raw PREEDIT change must not regenerate replay authority: {readout:?}"
        );
        firefox_assert_no_recovered_manual_authority(
            &mut harness,
            engine,
            "raw_preedit_only_completed_seed_gap",
        )
        .await;
    }));
}

// Causal regression through the production receive/callback paths.
// Controlled FIRST_SHARED_MECHANISM, not the exact physical interleaving:
// actual FocusOut callback expires before observer ingress; a later real Reset
// revokes the already-observed ticket before the delayed Disable callback.
// Reset is a controlled analogue of late FocusOut cleanup, not a claim that
// the physical textarea trace contained Reset. No reducer/stamp/owner writes.
// Reviewed checkout HEAD: 8c5cb6093802f98915dd5260869e2de26d099441.
// Baseline adapter.rs SHA256:
// bf99fc05a018d24f12c4d68d2111af3d579ea2df86327fba108e52babdf97a0c
// Baseline observation.rs SHA256:
// 787bc1061b3d2a6642d5ee896998e14926554cff8f4204bd8dba650f303c0a2b
// Baseline residuals.rs SHA256:
// 559fd1936b73614cfd200e97373d304d35e11e46975f5d5d79ce8bac05dbf80f
// Reuse the existing module imports, boxed bounded(), and real P2P helpers.

#[test]
fn expired_focus_out_then_retired_disable_preserves_successor_focus_in_stamp() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = known_engine(&mut harness).await;
        let old_token = engine.live_context_token().unwrap();

        let focus_out = method_message(
            DISPATCH_SENDER,
            46_605,
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
        // Deliberately do not poll the observer. The production rendezvous
        // deadline expires normally; there is no sleep or synthetic expiry.
        assert!(
            !bounded(engine.observe_context_focus_out(&focus_out.header(), Instant::now())).await
        );
        assert!(!engine.context_handoff_sealed);
        assert!(!harness.adapter.revalidate(&old_token));

        assert!(bounded(harness.observer.process_next()).await.unwrap());
        let disable = receive(&mut harness, 46_606, "Disable").await;
        let focus_in = Message::method_call(TARGET_PATH, "FocusInId")
            .unwrap()
            .interface(ENGINE_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(46_608).unwrap())
            .with_flags(zbus::message::Flags::NoReplyExpected)
            .unwrap()
            .build(&(
                CONTEXT_PATH.to_string(),
                "textarea-owned-fixture".to_string(),
            ))
            .unwrap();
        send_manually_dispatched_callback(
            &mut harness.peer,
            &focus_in,
            TARGET_PATH,
            ENGINE_INTERFACE,
        )
        .await;
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        let _enable = receive(&mut harness, 46_609, "Enable").await;
        let _later_reset = receive(&mut harness, 46_610, "Reset").await;

        let disable_stamp = harness
            .adapter
            .observe_callback(&disable.header(), Instant::now())
            .await
            .expect("actual Disable stamp before its delayed callback");
        let focus_before = harness
            .adapter
            .observe_callback(&focus_in.header(), Instant::now())
            .await
            .expect("successor FocusInId stamp was actually published");
        {
            let reducer = harness.adapter.shared.reducer.lock().unwrap();
            let ticket = reducer.ticket.as_ref().expect("observed lifecycle ticket");
            assert_eq!(ticket.status, TicketStatus::Revoked);
            assert_eq!(ticket.source_owner, old_token.owner);
            assert_eq!(
                ticket.disable_position.as_ref(),
                Some(&disable_stamp.position)
            );
            assert!(ticket.source_seal.is_none());
        }

        // This is the production lifecycle entrypoint, including local cleanup.
        bounded(engine.disable(disable.header())).await;
        let focus_after = harness
            .adapter
            .observe_callback(&focus_in.header(), Instant::now())
            .await
            .expect("retired Disable erased successor FocusIn stamp");
        assert_eq!(focus_after.header, focus_before.header);
        assert_eq!(focus_after.position, focus_before.position);
        assert_eq!(focus_after.disposition, focus_before.disposition);
        assert!(!engine.context_handoff_sealed);
        assert!(engine.context_owner.is_none());
        assert!(engine.live_context_token().is_none());
        assert!(engine.committed_tail.buffer.is_empty());
        {
            let shared = engine.shared.lock().unwrap();
            assert!(shared.active_path.is_none());
            assert!(shared.context_owner_generation.is_none());
        }
        cycle09_assert_no_local_text_effect(&mut harness).await;

        // An observer stamp alone is inert. Only the actual FocusIn callback
        // starts the source-free native marker acquisition of a fresh owner.
        // Use the actual wire payload, not separately invented callback args.
        let (context_path, client) = focus_in.body().deserialize::<(String, String)>().unwrap();
        assert_eq!(context_path, CONTEXT_PATH);
        bounded(engine.focus_in_id(focus_in.header(), context_path, client)).await;
        assert!(engine.live_context_token().is_none());
        forward_marker_bounded(&mut harness.peer).await;
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        engine.try_install_pending_context_activation();
        let fresh = engine
            .live_context_token()
            .expect("fresh source-free owner");
        assert_ne!(fresh.owner, old_token.owner);
        assert_eq!(fresh.lineage.completeness, WordCompleteness::UnknownStart);
        assert_eq!(fresh.lineage.observed_suffix_chars, 0);
        assert!(!harness.adapter.revalidate(&old_token));
        assert!(!engine.context_word_is_known());
        assert!(!engine.context_handoff_sealed);
        assert!(engine.committed_tail.buffer.is_empty());
        assert!(engine.context_reset_rereceipt.is_none());
        assert!(engine.committed_tail.pending_completion_learning.is_none());
        cycle09_assert_no_local_text_effect(&mut harness).await;
    }));
}

// Causal RED reproduced on unchanged production v10; actual callback controls.
async fn strict_prefix_v10_literal_key(
    harness: &mut Harness,
    engine: &mut LayIbusEngine,
    serial: u32,
    ch: char,
    code: u32,
) {
    assert!(legacy_key(harness, engine, serial, ch as u32, code, 0).await);
    td121_expect_legacy_commit_text(&mut harness.peer, &ch.to_string()).await;
    assert!(legacy_key(harness, engine, serial + 1, ch as u32, code, RELEASE_MASK).await);
}

async fn strict_prefix_v10_confirmed_appends(
    harness: &mut Harness,
    serial: u32,
    keys: &[(char, u32); 4],
    originally_confirmed: bool,
) -> (LayIbusEngine, String, String) {
    let mut engine = known_engine(harness).await;
    engine.config.auto_replace = false;
    engine.config.typing_assist = false;
    engine.config.nanda_precognition = false;
    engine.client_context.content_purpose = 0;
    engine.client_context.cursor_cell_width = 0;
    engine.client_context.surrounding_text_supported = true;
    assert_eq!(engine.committed_tail.buffer, " ");
    for (i, &(ch, code)) in keys[..2].iter().enumerate() {
        strict_prefix_v10_literal_key(harness, &mut engine, serial + 2 * i as u32, ch, code).await;
    }
    actual_reset(harness, &mut engine, serial + 4, false).await;
    let initial: String = keys[..2].iter().map(|key| key.0).collect();
    let first = if originally_confirmed {
        format!(" {initial}")
    } else {
        format!(" {}", keys[0].0)
    };
    exact_surrounding_receipt(harness, &mut engine, &first).await;
    assert_eq!(
        engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
        originally_confirmed
    );
    for (i, &(ch, code)) in keys[2..].iter().enumerate() {
        strict_prefix_v10_literal_key(harness, &mut engine, serial + 10 + 2 * i as u32, ch, code)
            .await;
    }
    let full: String = keys.iter().map(|key| key.0).collect();
    let stale: String = keys[..3].iter().map(|key| key.0).collect();
    assert_eq!(engine.committed_tail.buffer, format!(" {full}"));
    let pending = engine
        .context_reset_rereceipt
        .as_ref()
        .expect("authenticated append retains predecessor");
    assert_eq!(pending.token_text, full);
    assert_eq!(pending.confirmed, originally_confirmed);
    assert!(engine.committed_tail.autocorrect_suppression.is_none());
    assert!(!engine.exact_replay_quarantine_active());
    exact_surrounding_receipt(harness, &mut engine, &format!(" {stale}")).await;
    assert!(engine.context_reset_rereceipt.is_some());
    assert!(!engine.context_reset_rereceipt.as_ref().unwrap().confirmed);
    assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
    // surrounding_receipt already checks actual wire for no Commit/Delete.
    (engine, full, stale)
}

#[test]
fn confirmed_native_append_delayed_prefix_next_exact_receipt_restores_manual_route() {
    zbus::block_on(bounded(async {
        for keys in [
            [('a', 30), ('b', 48), ('c', 46), ('d', 32)],
            [('ф', 0), ('и', 0), ('с', 0), ('в', 0)],
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let (mut engine, full, _) =
                strict_prefix_v10_confirmed_appends(&mut harness, 48_000, &keys, true).await;
            // The controlled gap contains no Reset between prefix and full receipt.
            exact_surrounding_receipt(&mut harness, &mut engine, &format!(" {full}")).await;
            assert!(
                engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "confirmed native append delayed prefix lost next exact receipt authority"
            );
            // First causal RED is above, before dependent bridge/RPC preparation.
            let expected_tail = format!(" {full}");
            assert_eq!(engine.committed_tail.buffer, expected_tail);
            let (engine, result) = cycle09_manual_toggle(&mut harness, engine).await;
            assert_eq!(result, Ok((3, false)));
            assert_eq!(engine.committed_tail.buffer, expected_tail);
            let (engine, visible) = cycle09_visible_tail(&mut harness, engine).await;
            assert!(cycle09_tail_is_authoritative(
                &visible,
                &engine.path,
                false,
                &expected_tail
            ));
            assert_eq!(engine.committed_tail.buffer, expected_tail);
        }
    }));
}

#[test]
fn retained_strict_prefix_next_receipt_refuses_contradiction_or_identity_gap() {
    zbus::block_on(bounded(async {
        for gap in [
            "wrong_surface",
            "selection",
            "revision_gap",
            "new_focus",
            "sensitive",
            "key_gap",
            "no_original_confirmation",
        ] {
            let keys = [('a', 30), ('b', 48), ('c', 46), ('d', 32)];
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let (mut engine, full, stale) = strict_prefix_v10_confirmed_appends(
                &mut harness,
                49_000,
                &keys,
                gap != "no_original_confirmation",
            )
            .await;
            let old = engine.live_context_token().unwrap();
            let exact = format!(" {full}");
            match gap {
                "wrong_surface" => {
                    exact_surrounding_receipt(&mut harness, &mut engine, " abce").await
                }
                "selection" => surrounding_receipt(&mut harness, &mut engine, &exact, 5, 1).await,
                "revision_gap" => {
                    exact_surrounding_receipt(&mut harness, &mut engine, &format!(" {stale}"))
                        .await;
                    exact_surrounding_receipt(&mut harness, &mut engine, &exact).await;
                }
                "new_focus" => {
                    actual_focus_out(&mut harness, &mut engine, 49_020).await;
                    let focus = Message::method_call(TARGET_PATH, "FocusInId")
                        .unwrap()
                        .interface(ENGINE_INTERFACE)
                        .unwrap()
                        .sender(DISPATCH_SENDER)
                        .unwrap()
                        .serial(NonZeroU32::new(49_021).unwrap())
                        .with_flags(zbus::message::Flags::NoReplyExpected)
                        .unwrap()
                        .build(&(
                            CONTEXT_PATH.to_string(),
                            "strict-prefix-owned-fixture".to_string(),
                        ))
                        .unwrap();
                    send_manually_dispatched_callback(
                        &mut harness.peer,
                        &focus,
                        TARGET_PATH,
                        ENGINE_INTERFACE,
                    )
                    .await;
                    assert!(bounded(harness.observer.process_next()).await.unwrap());
                    let (context, client) = focus.body().deserialize::<(String, String)>().unwrap();
                    bounded(engine.focus_in_id(focus.header(), context, client)).await;
                    forward_marker_bounded(&mut harness.peer).await;
                    assert!(bounded(harness.observer.process_next()).await.unwrap());
                    engine.try_install_pending_context_activation();
                    assert!(!harness.adapter.revalidate(&old));
                    exact_surrounding_receipt(&mut harness, &mut engine, &exact).await;
                }
                "sensitive" => {
                    engine.set_content_type_state(8, 0);
                    exact_surrounding_receipt(&mut harness, &mut engine, &exact).await;
                }
                "key_gap" => {
                    assert!(!legacy_key(&mut harness, &mut engine, 49_020, KEY_LEFT, 105, 0).await);
                    exact_surrounding_receipt(&mut harness, &mut engine, &exact).await;
                }
                "no_original_confirmation" => {
                    exact_surrounding_receipt(&mut harness, &mut engine, &exact).await
                }
                _ => unreachable!(),
            }
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{gap}"
            );
            assert!(
                engine.context_reset_rereceipt.is_none(),
                "{gap} must discard the predecessor"
            );
            let engine = bridge_toggle_refused_without_text_effect(&mut harness, engine).await;
            assert!(engine.context_reset_rereceipt.is_none(), "{gap}");
        }
    }));
}

#[test]
fn test29_detached_reset_returns_exact_unknown_object_reply() {
    zbus::block_on(bounded(async {
        for serial in [19_315, 19_415] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine =
                initial_observed_tail_reset(&mut harness, 19_300, &[('a', 30), ('x', 45)]).await;
            exact_surrounding_receipt(&mut harness, &mut engine, "ax").await;
            // Finish setup output before the controlled unregister boundary.
            // No drain/filter runs between the Reset and the raw witness.
            let _ = drain_output_to_proof(&mut harness).await;
            let path = engine.path.clone();
            harness
                .connection
                .object_server()
                .at(path.as_str(), engine)
                .await
                .unwrap();
            let mut detached = cycle09_take_registered_engine(&harness, &path).await;
            assert!(harness
                .connection
                .object_server()
                .interface::<_, LayIbusEngine>(path.as_str())
                .await
                .is_err());
            assert!(!harness
                .peer
                .detached_callback_reply_serials
                .contains(&serial));

            // Literal old helper flag, actual observer ingress and direct
            // Reset callback: the transport reply has not been consumed.
            actual_reset(&mut harness, &mut detached, serial, false).await;
            let reply = bounded(next_peer_message_raw(&mut harness.peer)).await;
            eprintln!(
                "C09_TEST29_DETACHED_RESET_WITNESS sent_serial={} type={:?} reply_serial={:?} error_name={:?} member={:?}",
                serial,
                reply.header().message_type(),
                reply.header().reply_serial().map(|value| value.get()),
                reply.header().error_name().map(|name| name.as_str()),
                reply.header().member().map(|member| member.as_str()),
            );
            assert_eq!(reply.header().message_type(), Type::Error);
            assert_eq!(
                reply.header().reply_serial().map(|value| value.get()),
                Some(serial)
            );
            assert_eq!(
                reply.header().error_name().map(|name| name.as_str()),
                Some("org.freedesktop.DBus.Error.UnknownObject")
            );
            assert!(reply.header().member().is_none());
        }
    }));
}

// PRIVATE append-only fragment for adapter::tests::word_scope::residuals.
// NOT_RUN. The served engine must install from the real acquisition task;
// no target key, direct try_install, fabricated token or owner is supplied.
async fn c09_acquisition_completion_source_free_wire_case(native: bool) {
    let mut harness = default_budget_harness().await;
    let mut engine = new_engine(&harness);
    engine.set_client_capabilities(9);
    engine.config.nanda_precognition = false;
    let expected_ru = engine.layout_gesture.layout_is_ru;
    harness
        .connection
        .object_server()
        .at(TARGET_PATH, engine)
        .await
        .unwrap();
    let interface = harness
        .connection
        .object_server()
        .interface::<_, LayIbusEngine>(TARGET_PATH)
        .await
        .unwrap();

    // Actual dispatcher backpressure lets the existing observer stamp the
    // real typed calls before their callbacks start, without a second thread.
    let held = interface.get_mut().await;
    let focus = if native {
        Message::method_call(TARGET_PATH, "FocusInId")
            .unwrap()
            .interface(ENGINE_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(61_100).unwrap())
            .with_flags(zbus::message::Flags::NoReplyExpected)
            .unwrap()
            .build(&(CONTEXT_PATH, "c09-owned-fixture"))
            .unwrap()
    } else {
        method_message(
            DISPATCH_SENDER,
            61_100,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "FocusIn",
        )
    };
    harness.peer.connection.send(&focus).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    let enable = method_message(
        DISPATCH_SENDER,
        61_101,
        TARGET_PATH,
        ENGINE_INTERFACE,
        "Enable",
    );
    harness.peer.connection.send(&enable).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    drop(held);

    let mut signals = Vec::new();
    let mut got_compatibility_reply = false;
    let nonce = bounded(async {
        loop {
            let message = next_peer_message(&mut harness.peer).await;
            let header = message.header();
            if header.message_type() == Type::MethodCall {
                assert!(
                    !native,
                    "native acquisition must not issue compatibility Get"
                );
                assert!(
                    !got_compatibility_reply,
                    "only the original Get is serviced"
                );
                assert_eq!(header.interface().unwrap().as_str(), PROPERTIES_INTERFACE);
                assert_eq!(header.member().unwrap().as_str(), "Get");
                assert_eq!(header.path().unwrap().as_str(), IBUS_PATH);
                assert_eq!(
                    message.body().deserialize::<(String, String)>().unwrap(),
                    (
                        IBUS_INTERFACE.to_string(),
                        "CurrentInputContext".to_string()
                    )
                );
                harness
                    .peer
                    .connection
                    .reply(
                        &header,
                        &OwnedValue::from(ObjectPath::try_from(CONTEXT_PATH).unwrap()),
                    )
                    .await
                    .unwrap();
                got_compatibility_reply = true;
                continue;
            }
            assert_eq!(header.message_type(), Type::Signal);
            if header.interface().unwrap().as_str() == MARKER_INTERFACE {
                assert_eq!(header.path().unwrap().as_str(), MARKER_PATH);
                assert_eq!(header.member().unwrap().as_str(), MARKER_MEMBER);
                return message.body().deserialize::<u64>().unwrap();
            }
            assert_eq!(header.path().unwrap().as_str(), TARGET_PATH);
            assert_eq!(header.interface().unwrap().as_str(), ENGINE_INTERFACE);
            assert!(matches!(
                header.member().unwrap().as_str(),
                "RegisterProperties" | "RequireSurroundingText"
            ));
            signals.push(message);
        }
    })
    .await;
    assert_eq!(got_compatibility_reply, !native);

    // Hold the real target mutex only while the observer publishes the grant.
    // Both old and prospective production can publish, but only completion
    // delivery can install/ACK it when this guard is released without a key.
    let held = interface.get_mut().await;
    let deadline = {
        let pending = harness.adapter.shared.pending.lock().unwrap();
        let pending = pending.as_ref().expect("actual emitted acquisition fence");
        assert_eq!(pending.nonce.0, nonce);
        pending.deadline
    };
    let forwarded = Message::signal(MARKER_PATH, MARKER_INTERFACE, MARKER_MEMBER)
        .unwrap()
        .sender(ADAPTER_SENDER)
        .unwrap()
        .build(&nonce)
        .unwrap();
    harness.peer.connection.send(&forwarded).await.unwrap();
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    let expected_owner = harness
        .adapter
        .current_owner()
        .expect("published admitted owner");
    {
        let ready = harness.adapter.shared.ready_activation.lock().unwrap();
        let ready = ready
            .as_ref()
            .expect("published grant retained until install");
        assert_eq!(ready.owner, expected_owner);
        assert_eq!(ready.fence.nonce.0, nonce);
        assert!(matches!(&ready.outcome, ActivationOutcome::SourceFree(_)));
    }
    assert!(held.context_owner.is_none());
    drop(held);

    // The read uses the SAME actual fence deadline. Old production cannot
    // hang or accidentally fail only at the outer choreography timeout.
    let update = future::race(
        async {
            loop {
                let message = next_peer_message(&mut harness.peer).await;
                let header = message.header();
                assert_eq!(header.message_type(), Type::Signal);
                assert_eq!(header.path().unwrap().as_str(), TARGET_PATH);
                assert_eq!(header.interface().unwrap().as_str(), ENGINE_INTERFACE);
                if header.member().unwrap().as_str() == "UpdateProperty" {
                    return Some(message);
                }
                assert!(matches!(
                    header.member().unwrap().as_str(),
                    "RegisterProperties" | "RequireSurroundingText"
                ));
                signals.push(message);
            }
        },
        async {
            async_io::Timer::at(deadline).await;
            None
        },
    )
    .await;
    assert!(
        update.is_some(),
        "C09_ACQUISITION_COMPLETION_WITHOUT_KEY_MISSING_INPUT_MODE native={native}"
    );
    let update = update.unwrap();
    let body = update.body();
    let (property,): (zbus::zvariant::Value<'_>,) = body.deserialize().unwrap();
    assert_eq!(
        property,
        crate::text::make_ibus_input_mode_property(expected_ru)
    );
    signals.push(update);
    signals.extend(super::terminal_delivery::legacy_effects(&mut harness).await);
    assert_eq!(
        signals
            .iter()
            .filter(|message| message.header().member().unwrap().as_str() == "UpdateProperty")
            .count(),
        1
    );
    assert!(!signals.iter().any(|message| matches!(
        message.header().member().unwrap().as_str(),
        "CommitText" | "DeleteSurroundingText" | "ForwardKeyEvent" | "UpdatePreeditText"
    )));
    assert!(signals.iter().any(|message| {
        message.header().member().unwrap().as_str() == "RequireSurroundingText"
    }));
    let engine = interface.get().await;
    assert_eq!(engine.context_owner.as_ref(), Some(&expected_owner));
    let token = engine
        .live_context_token()
        .expect("completion installs live exact grant");
    assert!(token.matches_owner(&expected_owner));
    assert_eq!(token.activation.context, context(CONTEXT_PATH));
    assert!(!engine.context_word_is_known());
    assert!(engine.committed_tail.buffer.is_empty());
    assert!(engine.composition.buffer.is_empty());
    assert_eq!(engine.layout_gesture.layout_is_ru, expected_ru);
    assert!(!engine.client_context.input_mode_property_refresh_pending);
    assert!(harness
        .adapter
        .shared
        .ready_activation
        .lock()
        .unwrap()
        .is_none());
}

#[test]
fn c09_native_source_free_acquisition_completion_publishes_without_key() {
    zbus::block_on(bounded(c09_acquisition_completion_source_free_wire_case(
        true,
    )));
}

#[test]
fn c09_compatibility_source_free_acquisition_completion_publishes_without_key() {
    zbus::block_on(bounded(c09_acquisition_completion_source_free_wire_case(
        false,
    )));
}

// PRIVATE / NOT_RUN / GREEN-only. Append in adapter::tests::word_scope::residuals.
// Calls the V2 acquisition-specific production wait, unavailable in old source.
// Actual pending metadata supplies request/target/fence; no synthetic request.
// Original next_factory... control and all existing residual assertions remain.
// Scope is real delayed-predecessor callback ACK/notification, not a served
// property/decoder result or deterministic pause inside a mutex critical section.

#[test]
fn c09_acquisition_completion_waits_for_delayed_predecessor_ack() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness().await;
        let shared = Arc::new(Mutex::new(SharedState::default()));
        shared.lock().unwrap().handoff_tail_epoch = 41;
        let mut source = LayIbusEngine::new_from_component(
            TARGET_PATH.to_string(),
            shared.clone(),
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        start_source_free_pending(&harness).await;
        forward_marker_bounded(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
        let predecessor = harness
            .adapter
            .shared
            .ready_activation
            .lock()
            .unwrap()
            .clone()
            .unwrap();
        assert!(source.context_owner.is_none());
        let next_path = "/io/github/radislabus_star/LayIme/engine/next";
        let factory = Message::method_call("/org/freedesktop/IBus/Factory", "CreateEngine")
            .unwrap()
            .interface(FACTORY_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(4_100).unwrap())
            .with_flags(zbus::message::Flags::NoReplyExpected)
            .unwrap()
            .build(&"lay-us")
            .unwrap();
        send_manually_dispatched_callback(
            &mut harness.peer,
            &factory,
            "/org/freedesktop/IBus/Factory",
            FACTORY_INTERFACE,
        )
        .await;
        assert!(harness.observer.process_next().await.unwrap());
        let callback = harness
            .adapter
            .begin_factory_callback(&factory.header(), Instant::now(), profile("lay-us"))
            .await
            .unwrap();
        assert!(harness
            .adapter
            .bind_factory_target(&callback, engine_path(next_path)));
        let focus_out = method_message(
            DISPATCH_SENDER,
            4_101,
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
        let disable = method_message(
            DISPATCH_SENDER,
            4_102,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "Disable",
        );
        send_manually_dispatched_callback(
            &mut harness.peer,
            &disable,
            TARGET_PATH,
            ENGINE_INTERFACE,
        )
        .await;
        assert!(harness.observer.process_next().await.unwrap());
        let focus_in = Message::method_call(next_path, "FocusInId")
            .unwrap()
            .interface(ENGINE_INTERFACE)
            .unwrap()
            .sender(DISPATCH_SENDER)
            .unwrap()
            .serial(NonZeroU32::new(4_103).unwrap())
            .with_flags(zbus::message::Flags::NoReplyExpected)
            .unwrap()
            .build(&(CONTEXT_PATH, "c09-delayed-predecessor-owned-fixture"))
            .unwrap();
        send_manually_dispatched_callback(
            &mut harness.peer,
            &focus_in,
            next_path,
            ENGINE_INTERFACE,
        )
        .await;
        assert!(harness.observer.process_next().await.unwrap());
        let observed = harness
            .adapter
            .observe_callback(&focus_in.header(), Instant::now())
            .await
            .unwrap();
        harness
            .adapter
            .start_native_activation(
                engine_path(next_path),
                context(CONTEXT_PATH),
                observed.position,
            )
            .unwrap();
        forward_marker_bounded(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
        let pending = harness
            .adapter
            .shared
            .pending
            .lock()
            .unwrap()
            .clone()
            .unwrap();
        assert_ne!(pending.nonce, predecessor.fence.nonce);
        assert_eq!(
            harness
                .adapter
                .shared
                .ready_activation
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .owner,
            predecessor.owner
        );
        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(next_path))
            .unwrap()
            .is_none());
        // Acquisition marker observation does NOT set the Bridge-only ready
        // bit. The predecessor must first be installed/acknowledged before
        // this exact successor can occupy the single existing ready slot.
        assert!(pending.marker_observed);
        assert!(!pending.ready);
        let successor_fence = PendingFence {
            nonce: pending.nonce,
            deadline: pending.deadline,
        };
        let (successor_request, successor_target) = match &pending.kind {
            FenceKind::Acquisition {
                request,
                target_path,
            } => (*request, target_path.clone()),
            FenceKind::Bridge { .. } => panic!("actual native acquisition, never Bridge"),
        };
        assert_eq!(successor_target, engine_path(next_path));
        let mut waiting = Box::pin(harness.adapter.wait_for_acquisition_completion(
            &successor_target,
            successor_request,
            successor_fence,
        ));
        future::poll_fn(|cx| {
            assert!(
                matches!(
                    std::future::Future::poll(waiting.as_mut(), cx),
                    std::task::Poll::Pending
                ),
                "completion wait returned before the exact successor was published"
            );
            std::task::Poll::Ready(())
        })
        .await;
        // The listener is now registered and suspended in the actual wait.
        // Subsequent notifications may occur before its next poll; they must
        // remain observable rather than requiring a second key or a sleep.
        // An old completed fence's timer cannot erase its successor's work.
        harness.adapter.expire_fence(predecessor.fence);
        assert!(pending.marker_observed);
        assert!(matches!(pending.kind, FenceKind::Acquisition { .. }));
        // Matching observed Acquisition retains its separate ready owner.
        harness.adapter.expire_fence(PendingFence {
            nonce: pending.nonce,
            deadline: pending.deadline,
        });
        assert_eq!(
            harness
                .adapter
                .shared
                .ready_activation
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .owner,
            predecessor.owner
        );
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
            pending.nonce
        );
        assert!(
            source
                .observe_context_focus_out(&focus_out.header(), Instant::now())
                .await
        );
        assert!(source.context_owner.is_some());
        assert!(source.committed_tail.epoch > 41);
        assert!(source.committed_tail.buffer.is_empty());
        assert!(!source.context_word_is_known());
        assert!(harness.adapter.shared.pending.lock().unwrap().is_none());
        assert!(
            bounded(waiting).await.is_ok(),
            "completion listener lost delayed predecessor ACK/publication"
        );
        let outcome = harness
            .adapter
            .try_finish_activation_for(&engine_path(next_path))
            .unwrap()
            .expect("successor becomes ready after exact source installation/seal");
        let mut target = LayIbusEngine::new_from_component(
            next_path.to_string(),
            shared.clone(),
            Some(harness.adapter.clone()),
            "lay-ime-us",
            true,
            ime_config(),
        );
        assert!(target.install_context_activation(outcome));
        assert!(target.live_context_token().is_some());
        assert!(!target.context_word_is_known());
        assert!(target.committed_tail.buffer.is_empty());
        assert_eq!(
            shared.lock().unwrap().active_path.as_deref(),
            Some(next_path)
        );
        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(TARGET_PATH))
            .unwrap()
            .is_none());
        assert!(harness
            .adapter
            .try_finish_activation_for(&engine_path(next_path))
            .unwrap()
            .is_none());
    }));
}

#[test]
fn c09_manual_noreply_repeated_serial_counts_exact_unknown_object() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        // Establish the lazy-dispatch topology once, before the modeled batch.
        // This fixture target has no registered engine and no served callback.
        assert!(harness
            .connection
            .object_server()
            .interface::<_, LayIbusEngine>(TARGET_PATH)
            .await
            .is_err());
        let serial = 47_100;
        let reset = method_message(
            DISPATCH_SENDER,
            serial,
            TARGET_PATH,
            ENGINE_INTERFACE,
            "Reset",
        );
        send_manually_dispatched_callback(&mut harness.peer, &reset, TARGET_PATH, ENGINE_INTERFACE)
            .await;
        send_manually_dispatched_callback(&mut harness.peer, &reset, TARGET_PATH, ENGINE_INTERFACE)
            .await;
        assert_eq!(
            harness
                .peer
                .detached_callback_reply_serials
                .counts
                .get(&serial),
            Some(&2)
        );
        // Same serial, two actual emissions: no wait or observer step was added
        // between sends. The existing raw helper proves each exact error name.
        consume_detached_callback_reply(&mut harness.peer, serial).await;
        assert!(harness
            .peer
            .detached_callback_reply_serials
            .contains(&serial));
        consume_detached_callback_reply(&mut harness.peer, serial).await;
        assert!(!harness
            .peer
            .detached_callback_reply_serials
            .contains(&serial));
        let proof = Message::signal(CONTEXT_PATH, IBUS_INTERFACE, "ControlledManualReplyProof")
            .unwrap()
            .sender(ADAPTER_SENDER)
            .unwrap()
            .build(&())
            .unwrap();
        harness.connection.send(&proof).await.unwrap();
        let received = bounded(next_peer_message(&mut harness.peer)).await;
        assert_eq!(received.header().message_type(), Type::Signal);
        assert_eq!(
            received.header().member().map(|member| member.as_str()),
            Some("ControlledManualReplyProof")
        );
        assert_eq!(
            received.header().path().map(|path| path.as_str()),
            Some(CONTEXT_PATH)
        );
    }));
}

// PRIVATE / GREEN-only / NOT_RUN: append in adapter::tests::word_scope::residuals.
// The test-side begin_native_activation calls the production finish/marker
// path without starting a competing automatic delivery task. This checks the
// served method's guards after its interface await, not a client/atomic-wire
// positive. Every owner/token/scope and request comes from the real reducer.
async fn c09_acquisition_completion_delivery_guard_case(case: &'static str) {
    let mut harness = default_budget_harness().await;
    let target = engine_path(TARGET_PATH);
    let mut engine = new_engine(&harness);
    engine.set_client_capabilities(9);
    engine.config.nanda_precognition = false;

    let fence = bounded(harness.adapter.begin_native_activation(
        target.clone(),
        context(CONTEXT_PATH),
        Default::default(),
    ))
    .await
    .unwrap();
    forward_marker_bounded(&mut harness.peer).await;
    assert!(bounded(harness.observer.process_next()).await.unwrap());
    let ready = harness
        .adapter
        .shared
        .ready_activation
        .lock()
        .unwrap()
        .clone()
        .expect("actual published native acquisition");
    assert_eq!(ready.fence, fence);
    assert_eq!(ready.target_path, target);
    assert!(matches!(&ready.outcome, ActivationOutcome::SourceFree(_)));
    assert!(harness
        .adapter
        .activation_outcome_is_current(&ready.outcome));
    assert_eq!(harness.adapter.current_owner().as_ref(), Some(&ready.owner));
    assert!(harness.adapter.shared.pending.lock().unwrap().is_none());

    harness
        .connection
        .object_server()
        .at(TARGET_PATH, engine)
        .await
        .unwrap();
    let interface = harness
        .connection
        .object_server()
        .interface::<_, LayIbusEngine>(TARGET_PATH)
        .await
        .unwrap();
    let mut held = interface.get_mut().await;
    assert!(held.context_owner.is_none());
    assert!(held.live_context_token().is_none());

    // Keep this genuinely bootstrapped second adapter/observer alive throughout
    // the foreign case. Its different Arc instance is not a fabricated token.
    let foreign_harness = if case == "foreign_adapter" {
        Some(default_budget_harness().await)
    } else {
        None
    };
    let delivery_admission = foreign_harness
        .as_ref()
        .map(|foreign| foreign.adapter.clone())
        .unwrap_or_else(|| harness.adapter.clone());
    let mut delivery = Box::pin(LayIbusEngine::deliver_ready_context_activation(
        &harness.connection,
        &delivery_admission,
        &target,
        ready.request,
        ready.fence,
    ));
    future::poll_fn(|cx| {
        assert!(
            matches!(
                std::future::Future::poll(delivery.as_mut(), cx),
                std::task::Poll::Pending
            ),
            "delivery must wait for the actual held target interface: {case}"
        );
        std::task::Poll::Ready(())
    })
    .await;

    match case {
        "atomic_active" => {
            // Production atomic activation, not a direct active=true write.
            // With no installed owner it must remain native-unhandled and
            // produce no effects; the guard must still preserve this route.
            held.discard_atomic_pending();
            let proposal = held
                .process_atomic_key_event(
                    KEY_LEFT_SHIFT,
                    42,
                    RELEASE_MASK,
                    (
                        1,
                        ENVELOPE_MUTTER_FRAME,
                        ENVELOPE_CLIENT,
                        ENVELOPE_FOCUS_EPOCH,
                        ENVELOPE_CONTEXT,
                        ENVELOPE_LEASE,
                        vec![8; 32],
                    ),
                    td120_test_atomic_capability(),
                    (0, 0, Vec::new()),
                )
                .await
                .unwrap();
            assert_eq!(proposal.0, PROPOSAL_NATIVE_UNHANDLED);
            assert!(proposal.1.is_empty());
            assert!(held.atomic.active);
            assert!(held.context_owner.is_none());
        }
        "foreign_adapter" => {
            assert!(!delivery_admission.same_instance(&harness.adapter));
            assert!(held
                .context_admission
                .as_ref()
                .unwrap()
                .same_instance(&harness.adapter));
            assert!(held.context_owner.is_none());
        }
        "consumed_grant" => {
            // Actual existing production install/ACK consumes the grant while
            // delivery is suspended. No owner/token/scope is assigned by test.
            held.try_install_pending_context_activation();
            assert_eq!(held.context_owner.as_ref(), Some(&ready.owner));
            assert_eq!(
                held.live_context_token().as_ref(),
                Some(&ready.outcome.token())
            );
            assert!(held.client_context.input_mode_property_refresh_pending);
            assert!(harness
                .adapter
                .shared
                .ready_activation
                .lock()
                .unwrap()
                .is_none());
        }
        _ => panic!("undeclared delivery guard case"),
    }

    let local_before = (
        held.context_owner.clone(),
        held.context_token.clone(),
        held.context_word_scope,
        held.committed_tail.buffer.clone(),
        held.committed_tail.epoch,
        held.composition.buffer.clone(),
        held.composition.cursor,
        held.composition.preedit_visible,
        held.layout_gesture.layout_is_ru,
        held.client_context.input_mode_property_refresh_pending,
        held.client_context.runtime_owner_lease_identity,
        held.atomic.active,
    );
    let shared_before = {
        let shared = held.shared.lock().unwrap();
        (
            shared.active_path.clone(),
            shared.context_owner_generation,
            shared.handoff_tail_buffer.clone(),
            shared.handoff_tail_epoch,
            shared.handoff_focus_receipt.clone(),
            shared.suppression_revision,
        )
    };
    let token_before = harness.adapter.current_token();
    assert!(token_before
        .as_ref()
        .is_some_and(|token| harness.adapter.revalidate(token)));
    drop(held);
    bounded(delivery).await;

    // FIFO signal marker proves that delivery emitted NOTHING. No arbitrary
    // skip or timing sleep hides property, text or preedit effects.
    assert!(
        super::terminal_delivery::legacy_effects(&mut harness)
            .await
            .is_empty(),
        "guarded delivery emitted a native signal: {case}"
    );
    let after = interface.get().await;
    assert_eq!(
        (
            after.context_owner.clone(),
            after.context_token.clone(),
            after.context_word_scope,
            after.committed_tail.buffer.clone(),
            after.committed_tail.epoch,
            after.composition.buffer.clone(),
            after.composition.cursor,
            after.composition.preedit_visible,
            after.layout_gesture.layout_is_ru,
            after.client_context.input_mode_property_refresh_pending,
            after.client_context.runtime_owner_lease_identity,
            after.atomic.active,
        ),
        local_before,
        "guarded delivery changed local state: {case}"
    );
    let shared_after = {
        let shared = after.shared.lock().unwrap();
        (
            shared.active_path.clone(),
            shared.context_owner_generation,
            shared.handoff_tail_buffer.clone(),
            shared.handoff_tail_epoch,
            shared.handoff_focus_receipt.clone(),
            shared.suppression_revision,
        )
    };
    assert_eq!(shared_after, shared_before, "Shared changed: {case}");
    assert_eq!(
        harness.adapter.current_token(),
        token_before,
        "reducer token changed: {case}"
    );
    assert!(harness.adapter.shared.pending.lock().unwrap().is_none());
    let slot = harness.adapter.shared.ready_activation.lock().unwrap();
    if case == "consumed_grant" {
        assert!(slot.is_none(), "old delivery revived an acknowledged grant");
    } else {
        let remaining = slot
            .as_ref()
            .expect("guard cannot consume or discard ready grant");
        assert_eq!(remaining.request, ready.request);
        assert_eq!(remaining.fence, ready.fence);
        assert_eq!(remaining.target_path, ready.target_path);
        assert_eq!(remaining.owner, ready.owner);
        assert_eq!(remaining.outcome, ready.outcome);
        assert!(harness
            .adapter
            .activation_outcome_is_current(&remaining.outcome));
    }
}

#[test]
fn c09_acquisition_completion_delivery_preserves_guarded_state_after_interface_wait() {
    zbus::block_on(bounded(async {
        for case in ["atomic_active", "foreign_adapter", "consumed_grant"] {
            c09_acquisition_completion_delivery_guard_case(case).await;
        }
    }));
}

#[test]
fn native_space_observed_boundary_retains_strict_predecessor_until_exact_reset_receipt() {
    zbus::block_on(bounded(async {
        for (keys, suffix) in [
            (vec![('a', 30), ('b', 48), ('c', 46)], "de"),
            (vec![('ф', 0), ('и', 0), ('с', 0)], "ка"),
        ] {
            for suppressed in [false, true] {
                let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
                let mut engine = initial_observed_tail_reset(&mut harness, 62_000, &keys).await;
                let word = keys.iter().map(|key| key.0).collect::<String>();
                exact_surrounding_receipt(&mut harness, &mut engine, &word).await;
                assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                publish_fixture_append_completion(&mut harness, &mut engine, suffix).await;
                assert!(super::terminal_delivery::legacy_effects(&mut harness)
                    .await
                    .is_empty());
                let predecessor = engine
                    .context_reset_rereceipt
                    .as_ref()
                    .unwrap()
                    .predecessor_token
                    .clone();
                let owner = engine.context_owner.clone();
                let before_token = engine.live_context_token().unwrap();
                let before_epoch = engine.committed_tail.epoch;
                let before_revision = engine.client_context.surrounding_observation_revision;
                assert!(engine.composition.buffer.is_empty());
                assert!(!engine.uses_native_terminal_input());
                if suppressed {
                    assert!(engine.arm_current_word_autocorrect_suppression());
                }
                assert!(legacy_key(&mut harness, &mut engine, 62_020, KEY_SPACE, 57, 0).await);
                let effects = super::terminal_delivery::legacy_effects(&mut harness).await;
                let members: Vec<_> = effects
                    .iter()
                    .map(|effect| effect.header().member().unwrap().as_str().to_string())
                    .collect();
                assert_eq!(
                    members,
                    ["UpdatePreeditText", "HidePreeditText", "CommitText"]
                );
                for effect in &effects {
                    assert_eq!(effect.header().message_type(), Type::Signal);
                    assert_eq!(effect.header().path().unwrap().as_str(), engine.path);
                    assert_eq!(
                        effect.header().interface().unwrap().as_str(),
                        ENGINE_INTERFACE
                    );
                }
                let body = effects[0].body();
                let (text, cursor, visible, mode) = body
                    .deserialize::<(zbus::zvariant::Value<'_>, u32, bool, u32)>()
                    .unwrap();
                assert_eq!(
                    crate::ibus_interface::ibus_text_value_to_string(&text),
                    Some(String::new())
                );
                assert_eq!((cursor, visible, mode), (0, false, 0));
                let body = effects[2].body();
                let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
                assert_eq!(
                    crate::ibus_interface::ibus_text_value_to_string(&value),
                    Some(" ".into())
                );
                let expected = format!("{word} ");
                assert_eq!(engine.committed_tail.buffer, expected);
                assert_eq!(
                    engine.committed_tail.epoch,
                    before_epoch.checked_add(1).unwrap()
                );
                assert_eq!(engine.context_owner, owner);
                assert!(engine.context_word_is_known());
                assert_ne!(engine.live_context_token().as_ref(), Some(&before_token));
                let pending = engine
                    .context_reset_rereceipt
                    .as_ref()
                    .expect("native Space erased eligible boundary predecessor");
                assert!(!pending.confirmed);
                // The private replay/echo payload is not fixture authority:
                // prove no live replay seed or inherited display grant through
                // existing public state and the actual Reset/receipt consumer.
                assert!(!matches!(
                    engine.committed_tail.autocorrect_suppression.as_ref(),
                    Some(crate::protocol::AutocorrectSuppression::ExactReplay(_))
                ));
                assert!(!matches!(
                    engine
                        .shared
                        .lock()
                        .unwrap()
                        .autocorrect_suppression
                        .as_ref(),
                    Some(crate::protocol::AutocorrectSuppression::ExactReplay(_))
                ));
                assert!(!engine.context_reset_rereceipt_manual_refresh_allowed());
                assert!(engine
                    .context_reset_rereceipt_visible_append_suffix()
                    .is_none());
                assert_eq!(pending.predecessor_token, predecessor);
                assert_eq!(pending.token_text, expected);
                assert_eq!(
                    pending.observed_suffix_chars as usize,
                    expected.chars().count()
                );
                assert_eq!(pending.tail_epoch, engine.committed_tail.epoch);
                assert_eq!(pending.armed_revision, before_revision);
                assert_eq!(engine.context_token.as_ref(), Some(&pending.token));
                assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                assert!(engine.client_context.surrounding_text_snapshot.is_none());
                assert!(engine.composition.word_input_mode.is_none());
                assert!(!engine.composition.preedit_visible);
                assert!(engine.composition.preedit_suffix.is_empty());
                assert!(
                    legacy_key(
                        &mut harness,
                        &mut engine,
                        62_021,
                        KEY_SPACE,
                        57,
                        RELEASE_MASK
                    )
                    .await
                );
                assert!(super::terminal_delivery::legacy_effects(&mut harness)
                    .await
                    .is_empty());
                assert!(engine.context_reset_rereceipt.is_some());
                assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                actual_reset(&mut harness, &mut engine, 62_022, false).await;
                assert!(!engine.context_word_is_known());
                assert!(engine.context_reset_rereceipt.is_some());
                assert!(!engine.context_reset_rereceipt.as_ref().unwrap().confirmed);
                assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                assert!(super::terminal_delivery::legacy_effects(&mut harness)
                    .await
                    .is_empty());
                exact_surrounding_receipt(&mut harness, &mut engine, &expected).await;
                assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                assert!(super::terminal_delivery::legacy_effects(&mut harness)
                    .await
                    .is_empty());
                let path = engine.path.clone();
                let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
                assert_eq!(
                    disposition,
                    Ok(lay::manual_toggle::ImeManualToggleOutcome::DelegateExactImeTail.as_v3())
                );
                let (_, tail) = cycle09_visible_tail(&mut harness, engine).await;
                assert!(cycle09_tail_is_authoritative(
                    &tail, &path, false, &expected
                ));
            }
        }
    }));
}

// Private native Space transport discriminator. This fixture does not prove
// that a browser applies the returned physical key or that a later Tab succeeds.
#[test]
fn native_space_legacy_no_apply_and_manual_suppression_close_scope_without_edit() {
    zbus::block_on(bounded(async {
        for suppressed in [false, true] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = initial_observed_tail_reset(
                &mut harness,
                61_000,
                &[('a', 30), ('b', 48), ('c', 46)],
            )
            .await;
            exact_surrounding_receipt(&mut harness, &mut engine, "abc").await;
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            publish_fixture_append_completion(&mut harness, &mut engine, "de").await;
            assert!(super::terminal_delivery::legacy_effects(&mut harness)
                .await
                .is_empty());
            let predecessor = engine
                .context_reset_rereceipt
                .as_ref()
                .unwrap()
                .predecessor_token
                .clone();
            let owner = engine.context_owner.clone();
            let before_revision = engine.client_context.surrounding_observation_revision;
            let before_token = engine.live_context_token().unwrap();
            let before_epoch = engine.committed_tail.epoch;
            assert!(engine.composition.buffer.is_empty());
            assert!(!engine.uses_native_terminal_input());
            if suppressed {
                assert!(engine.arm_current_word_autocorrect_suppression());
            }
            assert!(
                legacy_key(&mut harness, &mut engine, 61_020, KEY_SPACE, 57, 0).await,
                "ordinary ManagedCommit Space must commit once, suppressed={suppressed}"
            );
            let effects = super::terminal_delivery::legacy_effects(&mut harness).await;
            let members: Vec<_> = effects
                .iter()
                .map(|message| message.header().member().unwrap().as_str().to_string())
                .collect();
            assert_eq!(
                members,
                ["UpdatePreeditText", "HidePreeditText", "CommitText"]
            );
            let body = effects[0].body();
            let (text, cursor, visible, mode) = body
                .deserialize::<(zbus::zvariant::Value<'_>, u32, bool, u32)>()
                .unwrap();
            assert_eq!(
                crate::ibus_interface::ibus_text_value_to_string(&text),
                Some(String::new())
            );
            assert_eq!((cursor, visible, mode), (0, false, 0));
            let body = effects[2].body();
            let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
            assert_eq!(
                crate::ibus_interface::ibus_text_value_to_string(&value),
                Some(" ".into())
            );
            assert_eq!(engine.committed_tail.buffer, "abc ");
            assert_eq!(
                engine.committed_tail.epoch,
                before_epoch.checked_add(1).unwrap()
            );
            assert_eq!(engine.context_owner, owner);
            assert!(engine.context_word_is_known());
            assert_ne!(engine.live_context_token().as_ref(), Some(&before_token));
            // Replaces A's pending=None oracle explicitly: ADR retains only
            // inert provenance. CommitText dispatch cannot confirm or grant edit.
            let pending = engine
                .context_reset_rereceipt
                .as_ref()
                .expect("native Space must retain the independently eligible predecessor");
            assert!(!pending.confirmed);
            assert_eq!(pending.predecessor_token, predecessor);
            assert_eq!(pending.token_text, "abc ");
            assert_eq!(pending.observed_suffix_chars, 4);
            assert_eq!(pending.tail_epoch, engine.committed_tail.epoch);
            assert_eq!(pending.armed_revision, before_revision);
            assert_eq!(engine.context_token.as_ref(), Some(&pending.token));
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert!(!engine.context_reset_rereceipt_manual_refresh_allowed());
            assert!(engine
                .context_reset_rereceipt_visible_append_suffix()
                .is_none());
            assert!(engine.client_context.surrounding_text_snapshot.is_none());
            assert!(engine.composition.word_input_mode.is_none());
            assert!(!engine.composition.preedit_visible);
            assert!(engine.composition.preedit_suffix.is_empty());
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    61_021,
                    KEY_SPACE,
                    57,
                    RELEASE_MASK
                )
                .await
            );
            assert!(super::terminal_delivery::legacy_effects(&mut harness)
                .await
                .is_empty());
            // Even an exact early ACK has no UnknownStart handoff identity.
            // The old KnownStart consumer remains distinct and unchanged.
            exact_surrounding_receipt(&mut harness, &mut engine, "abc ").await;
            assert!(engine.context_reset_rereceipt.as_ref().unwrap().confirmed);
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            actual_reset(&mut harness, &mut engine, 61_022, false).await;
            assert!(engine.context_reset_rereceipt.is_some());
            assert_eq!(
                engine
                    .context_reset_rereceipt
                    .as_ref()
                    .unwrap()
                    .predecessor_token,
                predecessor
            );
            assert!(!engine.context_reset_rereceipt.as_ref().unwrap().confirmed);
            assert!(!engine.context_word_is_known());
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert!(super::terminal_delivery::legacy_effects(&mut harness)
                .await
                .is_empty());
            exact_surrounding_receipt(&mut harness, &mut engine, "abc ").await;
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert!(engine.capture_observed_suffix_display_frame().is_none());
            assert!(super::terminal_delivery::legacy_effects(&mut harness)
                .await
                .is_empty());
            let path = engine.path.clone();
            let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
            assert_eq!(
                disposition,
                Ok(lay::manual_toggle::ImeManualToggleOutcome::DelegateExactImeTail.as_v3())
            );
            let (_, tail) = cycle09_visible_tail(&mut harness, engine).await;
            assert!(cycle09_tail_is_authoritative(&tail, &path, false, "abc "));
        }
    }));
}

#[test]
fn native_space_legacy_clear_failure_does_not_observe_boundary() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine =
            initial_observed_tail_reset(&mut harness, 61_100, &[('a', 30), ('b', 48), ('c', 46)])
                .await;
        exact_surrounding_receipt(&mut harness, &mut engine, "abc").await;
        publish_fixture_append_completion(&mut harness, &mut engine, "de").await;
        let before_tail = engine.committed_tail.buffer.clone();
        let before_epoch = engine.committed_tail.epoch;
        let before_token = engine.context_token.clone();
        let emitter =
            zbus::object_server::SignalEmitter::new(&harness.connection, TARGET_PATH).unwrap();
        // Real transport failure at Clear, not a simulated CommitText failure.
        // The direct production method isolates this boundary after ingress;
        // receiver admission and recovery of a failed wire are outside this test.
        harness.connection.clone().close().await.unwrap();
        let result = engine
            .process_pressed_key(&mut EngineOutput::legacy(&emitter), KEY_SPACE, 57, 0)
            .await;
        assert!(result.is_err());
        assert_eq!(engine.committed_tail.buffer, before_tail);
        assert_eq!(engine.committed_tail.epoch, before_epoch);
        assert_eq!(engine.context_token, before_token);
    }));
}

// The native output outcome is never an ACK. Exercise the existing consumers,
// with no fixture writes to scope/token/revision/confirmation fields.
#[test]
fn native_space_observed_boundary_refuses_missing_or_contradictory_receipts() {
    for guard in [
        "missing",
        "surface",
        "selection",
        "cursor",
        "revision_gap",
        "native_key_gap",
        "focus",
        "content",
    ] {
        zbus::block_on(bounded(async {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = initial_observed_tail_reset(
                &mut harness,
                63_000,
                &[('a', 30), ('b', 48), ('c', 46)],
            )
            .await;
            exact_surrounding_receipt(&mut harness, &mut engine, "abc").await;
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            publish_fixture_append_completion(&mut harness, &mut engine, "de").await;
            let old_token = engine.live_context_token().unwrap();
            let before_epoch = engine.committed_tail.epoch;
            let mode = engine.layout_gesture.layout_is_ru;
            assert!(engine.composition.preedit_visible);
            assert!(legacy_key(&mut harness, &mut engine, 63_020, KEY_SPACE, 57, 0).await);
            expect_legacy_managed_space(&mut harness, &engine, mode, true).await;
            assert_eq!(engine.committed_tail.buffer, "abc ");
            assert_eq!(
                engine.committed_tail.epoch,
                before_epoch.checked_add(1).unwrap()
            );
            assert!(!harness.adapter.revalidate(&old_token));
            assert!(engine.context_reset_rereceipt.is_some());
            assert!(!engine.context_reset_rereceipt.as_ref().unwrap().confirmed);
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            actual_reset(&mut harness, &mut engine, 63_021, false).await;
            assert!(!engine.context_word_is_known());
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            match guard {
                "missing" => {}
                "surface" => surrounding_receipt(&mut harness, &mut engine, "abd ", 4, 4).await,
                "selection" => surrounding_receipt(&mut harness, &mut engine, "abc ", 4, 3).await,
                "cursor" => surrounding_receipt(&mut harness, &mut engine, "abc ", 3, 3).await,
                "revision_gap" => {
                    // An initially unconfirmed prefix cannot arm the separate
                    // managed confirmed-prefix echo or bridge a revision gap.
                    exact_surrounding_receipt(&mut harness, &mut engine, "abc").await;
                    assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                    exact_surrounding_receipt(&mut harness, &mut engine, "abc ").await;
                }
                "native_key_gap" => {
                    let mode = engine.layout_gesture.layout_is_ru;
                    assert!(legacy_key(&mut harness, &mut engine, 63_022, 'd' as u32, 32, 0).await);
                    expect_legacy_commit(&mut harness.peer, &engine, mode).await;
                    exact_surrounding_receipt(&mut harness, &mut engine, "abc ").await;
                }
                "focus" => {
                    actual_focus_out(&mut harness, &mut engine, 63_022).await;
                    exact_surrounding_receipt(&mut harness, &mut engine, "abc ").await;
                }
                "content" => {
                    engine.set_content_type_state(8, 0);
                    exact_surrounding_receipt(&mut harness, &mut engine, "abc ").await;
                }
                _ => unreachable!(),
            }
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{guard}"
            );
            assert!(
                engine
                    .context_reset_rereceipt_visible_append_suffix()
                    .is_none(),
                "{guard}"
            );
            assert!(
                super::terminal_delivery::legacy_effects(&mut harness)
                    .await
                    .iter()
                    .all(|effect| {
                        !matches!(
                            effect.header().member().unwrap().as_str(),
                            "CommitText" | "DeleteSurroundingText"
                        )
                    }),
                "{guard}"
            );
            let tail_before = engine.committed_tail.buffer.clone();
            let epoch_before = engine.committed_tail.epoch;
            let engine = bridge_toggle_refused_without_text_effect(&mut harness, engine).await;
            assert_eq!(engine.committed_tail.buffer, tail_before, "{guard}");
            assert_eq!(engine.committed_tail.epoch, epoch_before, "{guard}");
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{guard}"
            );
        }));
    }
}

#[test]
fn native_space_without_eligible_predecessor_cannot_create_reset_authority() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.config.auto_replace = false;
        engine.config.nanda_precognition = false;
        assert!(engine.context_reset_rereceipt.is_none());
        assert!(engine.committed_tail.buffer.is_empty());
        assert!(!engine.composition.preedit_visible);
        let mode = engine.layout_gesture.layout_is_ru;
        assert!(!legacy_key(&mut harness, &mut engine, 63_100, KEY_SPACE, 57, 0).await);
        expect_legacy_native_space(&mut harness, &engine, mode, false).await;
        assert_eq!(engine.committed_tail.buffer, " ");
        assert!(engine.context_word_is_known());
        assert!(engine.context_reset_rereceipt.is_none());
        actual_reset(&mut harness, &mut engine, 63_101, false).await;
        exact_surrounding_receipt(&mut harness, &mut engine, " ").await;
        assert!(engine.context_reset_rereceipt.is_none());
        assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        let engine = bridge_toggle_refused_without_text_effect(&mut harness, engine).await;
        assert_eq!(engine.committed_tail.buffer, " ");
        assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
    }));
}
