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

async fn legacy_effects(harness: &mut Harness) -> Vec<Message> {
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
            if next.header().interface().unwrap().as_str() == "org.lay.Proof" {
                assert_eq!(next.header().member().unwrap().as_str(), "Reached");
                return effects;
            }
            assert_eq!(next.header().message_type(), Type::Signal);
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
                .expect("known current word");
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
