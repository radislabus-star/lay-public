/// Cursor-edit provenance through real receive/callback and manual bridge paths.
mod native_cursor_edit_tests {
    use super::*;

    async fn cursor_reset_with_order(
        harness: &mut Harness,
        engine: &mut LayIbusEngine,
        serial: u32,
        handler_first: bool,
    ) {
        if !handler_first {
            actual_reset(harness, engine, serial, false).await;
            return;
        }
        let path = engine.path.clone();
        let reset = method_message(DISPATCH_SENDER, serial, &path, ENGINE_INTERFACE, "Reset");
        send_manually_dispatched_callback(&mut harness.peer, &reset, &path, ENGINE_INTERFACE).await;
        let emitter = zbus::object_server::SignalEmitter::new(&harness.connection, path).unwrap();
        // Establish handler-first order before the actual observer supplies
        // its canonical stamp. This tests causal order, not transport latency
        // against the production callback deadline.
        let mut reply = Box::pin(engine.reset(reset.header(), emitter));
        assert!(
            future::poll_once(&mut reply).await.is_none(),
            "handler-first Reset must await its actual canonical observation"
        );
        assert!(bounded(harness.observer.process_next()).await.unwrap());
        bounded(reply).await.unwrap();
    }

    async fn cursor_at_interior(harness: &mut Harness) -> LayIbusEngine {
        let keys = [
            ('a', 30),
            ('b', 48),
            ('c', 46),
            ('d', 32),
            ('e', 18),
            ('f', 33),
        ];
        let mut engine = initial_observed_tail_reset_with_content_type(
            harness,
            96_000,
            &keys,
            None,
            Some((0, 0)),
        )
        .await;
        exact_surrounding_receipt(harness, &mut engine, "prefix abcdef").await;
        assert!(
            !legacy_key(
                harness,
                &mut engine,
                96_030,
                crate::protocol::KEY_LEFT,
                105,
                0
            )
            .await
        );
        actual_reset(harness, &mut engine, 96_031, false).await;
        cycle09_surrounding_receipt(harness, &mut engine, "prefix abcdef", 12, 12).await;
        assert!(
            !td121_release_with_order(
                harness,
                &mut engine,
                96_032,
                crate::protocol::KEY_LEFT,
                105,
                true
            )
            .await
        );
        assert!(engine
            .context_reset_rereceipt
            .as_ref()
            .is_some_and(|pending| !pending.confirmed));
        assert!(engine.committed_tail.buffer.is_empty());
        cycle09_assert_no_local_text_effect(harness).await;
        // A local output marker does not fence zbus's independent dispatcher.
        // Metadata fixtures below read their exact next callback reply.
        bounded(complete_absent_manual_dispatch(&mut harness.peer)).await;
        engine
    }

    #[test]
    fn td121_cursor_edit_receipt_is_manual_only_until_explicit_consumption() {
        zbus::block_on(bounded(async {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = cursor_at_interior(&mut harness).await;
            assert!(!legacy_key(&mut harness, &mut engine, 96_033, 65_367, 107, 0).await);
            exact_surrounding_receipt(&mut harness, &mut engine, "prefix abcdef").await;
            actual_reset(&mut harness, &mut engine, 96_034, false).await;
            assert!(
                !td121_release_with_order(&mut harness, &mut engine, 96_035, 65_367, 107, true)
                    .await
            );
            assert!(engine.native_cursor_manual_source_is_current());
            assert!(!engine.context_word_is_known());
            assert!(!engine.context_observed_suffix_exact_manual_handoff_allowed());
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert!(!engine.context_reset_rereceipt_computation_allowed());
            assert!(engine.capture_observed_suffix_display_frame().is_none());
            assert!(engine.capture_pending_reset_readout_frame().is_none());
            assert!(engine.capture_pending_reset_space_frame().is_none());
            set_fixture_append_completion(&mut engine, "xyz");
            let (mut engine, visible) = cycle09_visible_tail(&mut harness, engine).await;
            assert!(
                visible
                    .as_ref()
                    .is_ok_and(|tail| tail.0.starts_with("passive:")),
                "generic readout must not expose this phase: {visible:?}"
            );
            assert!(
                engine.native_cursor_manual_source_is_current(),
                "readout does not consume the manual proof"
            );
            assert!(!legacy_key(&mut harness, &mut engine, 96_036, KEY_TAB, 15, 0).await);
            assert!(!engine.native_cursor_manual_source_is_current());
            assert!(engine.committed_tail.pending_completion_learning.is_none());
            assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
            cycle09_assert_no_local_text_effect(&mut harness).await;
        }));
    }

    #[test]
    fn td121_cursor_edit_cannot_survive_identity_or_metadata_changes() {
        zbus::block_on(bounded(async {
            for scenario in 0..6 {
                let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
                let mut engine = cursor_at_interior(&mut harness).await;
                match scenario {
                    0 => {
                        engine.client_context.focus_serial =
                            engine.client_context.focus_serial.wrapping_add(1)
                    }
                    1 => {
                        engine.client_context.runtime_owner_lease_identity = engine
                            .client_context
                            .runtime_owner_lease_identity
                            .wrapping_add(1)
                    }
                    2 => engine.set_layout_is_ru(!engine.layout_gesture.layout_is_ru),
                    3 => {
                        let capabilities = u32::from(engine.client_context.preedit_text_supported)
                            | (u32::from(engine.client_context.surrounding_text_supported) << 5)
                            | (u32::from(
                                engine.client_context.exact_surrounding_refresh_available,
                            ) << 30);
                        engine.set_client_capabilities(capabilities ^ 1);
                        engine.set_client_capabilities(capabilities);
                    }
                    4 => {
                        let mut serial = 96_040;
                        for value in [(8, 0), (0, 0)] {
                            let message = firefox_receive_deferred_content_type(
                                &mut harness,
                                &engine,
                                &mut serial,
                                value,
                            )
                            .await;
                            engine.set_content_type(value, Some(message.header())).await;
                        }
                    }
                    5 => {
                        let owner = engine.context_owner.as_ref().unwrap();
                        harness.adapter.revoke_current_owner(owner);
                    }
                    _ => unreachable!(),
                }
                exact_surrounding_receipt(&mut harness, &mut engine, "prefix abcdef").await;
                assert!(
                    !engine.native_cursor_manual_source_is_current(),
                    "scenario={scenario}"
                );
                assert!(engine.capture_pending_reset_readout_frame().is_none());
                assert!(engine.capture_pending_reset_space_frame().is_none());
                assert!(!engine.context_word_is_known());
                cycle09_assert_no_local_text_effect(&mut harness).await;
            }
        }));
    }

    #[test]
    fn td121_cursor_edit_rejects_unexplained_receipts_and_repeated_reset() {
        zbus::block_on(bounded(async {
            for invalid in [
                Some(("other abcdef", 11, 11)),
                Some(("prefix abXdef", 12, 12)),
                Some(("prefix abcdef right", 12, 12)),
                Some(("prefix abcdef", 12, 11)),
                Some(("prefix abcdef", 0, 0)),
                Some(("", 0, 0)),
                None,
            ] {
                let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
                let mut engine = cursor_at_interior(&mut harness).await;
                if let Some((text, cursor, anchor)) = invalid {
                    cycle09_surrounding_receipt(&mut harness, &mut engine, text, cursor, anchor)
                        .await;
                } else {
                    actual_reset(&mut harness, &mut engine, 96_033, false).await;
                }
                assert!(
                    engine.context_reset_rereceipt.is_none(),
                    "invalid={invalid:?}"
                );
                exact_surrounding_receipt(&mut harness, &mut engine, "prefix abcdef").await;
                assert!(!engine.native_cursor_manual_source_is_current());
                assert!(!engine.context_word_is_known());
                let (_engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
                assert_eq!(disposition, Ok((0, false)), "invalid={invalid:?}");
                cycle09_assert_no_local_text_effect(&mut harness).await;
            }
        }));
    }

    #[test]
    fn td121_native_cursor_reedit_end_reset_preserves_only_exact_manual_source() {
        zbus::block_on(bounded(cursor_reedit_matrix(false, false, false, false)));
    }

    #[test]
    fn td121_cursor_published_hint_and_native_shifts_preserve_manual_source() {
        zbus::block_on(bounded(cursor_reedit_matrix(true, true, false, false)));
    }

    #[test]
    fn td121_cursor_owned_insert_receipt_before_reset_preserves_manual_source() {
        zbus::block_on(bounded(cursor_reedit_matrix(true, true, true, false)));
    }

    #[test]
    fn td121_cursor_handler_first_reset_preserves_inert_source_until_matched_receipt() {
        zbus::block_on(bounded(cursor_reedit_matrix(true, true, true, true)));
    }

    #[test]
    fn td121_cursor_unobserved_reset_cannot_restore_tentative_source() {
        zbus::block_on(bounded(async {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = cursor_at_interior(&mut harness).await;
            assert!(!legacy_key(&mut harness, &mut engine, 97_000, 65_367, 107, 0).await);
            exact_surrounding_receipt(&mut harness, &mut engine, "prefix abcdef").await;
            assert!(
                !td121_release_with_order(&mut harness, &mut engine, 97_001, 65_367, 107, true,)
                    .await
            );
            assert!(engine.native_cursor_manual_source_is_current());
            let old = engine.context_token.clone().unwrap();
            // Adverse control: a plausible header without an actually received
            // Reset cannot turn the tentative pre-Reset clone into authority.
            let reset = method_message(
                DISPATCH_SENDER,
                97_002,
                &engine.path,
                ENGINE_INTERFACE,
                "Reset",
            );
            let emitter =
                zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                    .unwrap();
            engine.reset(reset.header(), emitter).await.unwrap();
            assert!(!harness.adapter.revalidate(&old));
            assert!(!engine.native_cursor_manual_source_is_current());
            assert!(!engine.context_word_is_known());
            assert!(engine.capture_pending_reset_space_frame().is_none());
            assert!(engine.committed_tail.pending_completion_learning.is_none());
            assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
            cycle09_assert_no_local_text_effect(&mut harness).await;
            let (_engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
            assert_eq!(disposition, Ok((0, false)));
            cycle09_assert_no_local_text_effect(&mut harness).await;
        }));
    }

    async fn cursor_reedit_matrix(
        published_hint: bool,
        native_shifts: bool,
        insert_receipt_before_reset: bool,
        handler_first_reset: bool,
    ) {
        let mut failures = Vec::new();
        for prefix in ["", "prefix "] {
            for inserted in [('c', 46), ('z', 44)] {
                for observer_first in [true, false] {
                    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
                    let keys = [
                        ('a', 30),
                        ('b', 48),
                        ('c', 46),
                        ('d', 32),
                        ('e', 18),
                        ('f', 33),
                    ];
                    let mut engine = initial_observed_tail_reset_with_content_type(
                        &mut harness,
                        95_000,
                        &keys,
                        None,
                        Some((0, 0)),
                    )
                    .await;
                    let source = format!("{prefix}abcdef");
                    let prefix_chars = prefix.chars().count() as u32;
                    exact_surrounding_receipt(&mut harness, &mut engine, &source).await;
                    assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                    assert!(!engine.context_word_is_known());
                    assert_eq!(engine.committed_tail.buffer, "abcdef");
                    if published_hint {
                        set_fixture_append_completion(&mut engine, "xyz");
                        let emitter = zbus::object_server::SignalEmitter::new(
                            &harness.connection,
                            engine.path.clone(),
                        )
                        .unwrap();
                        engine
                            .publish_selected_precognition_candidate(
                                &mut crate::output::EngineOutput::legacy(&emitter),
                            )
                            .await
                            .unwrap();
                        let effects = cycle09_collect_no_local_text_effects(&mut harness).await;
                        assert!(effects.iter().any(|message| message
                            .header()
                            .member()
                            .is_some_and(|member| member.as_str() == "UpdateAuxiliaryText")));
                        assert!(!engine.composition.preedit_visible);
                        assert!(engine.composition.auxiliary_visible);
                        assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                    }
                    let mut serial = 95_030;

                    // The client's cursor movement is external. Keep the
                    // ordinary tail invalidation; it must not remain a
                    // writable six-character mirror at the interior caret.
                    for caret in [5, 4, 3] {
                        assert!(
                            !legacy_key(
                                &mut harness,
                                &mut engine,
                                serial,
                                crate::protocol::KEY_LEFT,
                                105,
                                0,
                            )
                            .await
                        );
                        serial += 1;
                        cycle09_assert_no_local_text_effect(&mut harness).await;
                        assert!(engine.committed_tail.buffer.is_empty());
                        let old = engine.context_token.clone().unwrap();
                        cursor_reset_with_order(
                            &mut harness,
                            &mut engine,
                            serial,
                            handler_first_reset,
                        )
                        .await;
                        serial += 1;
                        assert!(!harness.adapter.revalidate(&old));
                        if handler_first_reset {
                            assert!(engine.context_reset_rereceipt.as_ref().is_some_and(|pending| !pending.confirmed),
                                "first handler-first cursor Reset lost its inert phase before the expected actual SSI");
                        }
                        cycle09_surrounding_receipt(
                            &mut harness,
                            &mut engine,
                            &source,
                            prefix_chars + caret,
                            prefix_chars + caret,
                        )
                        .await;
                        assert!(
                            !td121_release_with_order(
                                &mut harness,
                                &mut engine,
                                serial,
                                crate::protocol::KEY_LEFT,
                                105,
                                observer_first,
                            )
                            .await
                        );
                        serial += 1;
                        assert!(!engine.context_word_is_known());
                        if engine.context_reset_rereceipt.is_none() {
                            failures.push(format!("phase lost after Left caret={caret}, observer_first={observer_first}"));
                        }
                        cycle09_assert_no_local_text_effect(&mut harness).await;
                    }

                    assert!(
                        !legacy_key(&mut harness, &mut engine, serial, KEY_BACKSPACE, 14, 0,).await
                    );
                    serial += 1;
                    cycle09_assert_no_local_text_effect(&mut harness).await;
                    if engine.context_reset_rereceipt.is_none() {
                        failures.push(format!(
                            "phase lost after Backspace, observer_first={observer_first}"
                        ));
                    }
                    cursor_reset_with_order(&mut harness, &mut engine, serial, handler_first_reset)
                        .await;
                    serial += 1;
                    cycle09_surrounding_receipt(
                        &mut harness,
                        &mut engine,
                        &format!("{prefix}abdef"),
                        prefix_chars + 2,
                        prefix_chars + 2,
                    )
                    .await;
                    assert!(
                        !td121_release_with_order(
                            &mut harness,
                            &mut engine,
                            serial,
                            KEY_BACKSPACE,
                            14,
                            observer_first,
                        )
                        .await
                    );
                    serial += 1;
                    cycle09_assert_no_local_text_effect(&mut harness).await;

                    let mode = engine.layout_gesture.layout_is_ru;
                    assert!(
                        legacy_key(
                            &mut harness,
                            &mut engine,
                            serial,
                            inserted.0 as u32,
                            inserted.1,
                            0,
                        )
                        .await
                    );
                    serial += 1;
                    expect_legacy_commit(&mut harness.peer, &engine, mode).await;
                    let edited_word = format!("ab{}def", inserted.0);
                    let edited = format!("{prefix}{edited_word}");
                    if insert_receipt_before_reset {
                        cycle09_surrounding_receipt(
                            &mut harness,
                            &mut engine,
                            &edited,
                            prefix_chars + 3,
                            prefix_chars + 3,
                        )
                        .await;
                        assert!(engine.context_reset_rereceipt.is_some(),
                            "actual owned-insert SSI before Reset must retain its inert cursor phase");
                    }
                    cursor_reset_with_order(&mut harness, &mut engine, serial, handler_first_reset)
                        .await;
                    serial += 1;
                    if !insert_receipt_before_reset {
                        cycle09_surrounding_receipt(
                            &mut harness,
                            &mut engine,
                            &edited,
                            prefix_chars + 3,
                            prefix_chars + 3,
                        )
                        .await;
                    }
                    assert!(
                        td121_release_with_order(
                            &mut harness,
                            &mut engine,
                            serial,
                            inserted.0 as u32,
                            inserted.1,
                            observer_first,
                        )
                        .await
                    );
                    serial += 1;
                    assert!(!engine.context_word_is_known());
                    if engine.context_reset_rereceipt.is_none() {
                        failures.push(format!(
                            "phase lost after owned insert, observer_first={observer_first}"
                        ));
                    }
                    cycle09_assert_no_local_text_effect(&mut harness).await;

                    // Captured native order: End, actual full SSI, late
                    // Reset, release. There is no new SSI after that Reset.
                    const END_KEYVAL: u32 = 65_367;
                    assert!(
                        !legacy_key(&mut harness, &mut engine, serial, END_KEYVAL, 107, 0,).await
                    );
                    serial += 1;
                    cycle09_assert_no_local_text_effect(&mut harness).await;
                    exact_surrounding_receipt(&mut harness, &mut engine, &edited).await;
                    let old = engine.context_token.clone().unwrap();
                    cursor_reset_with_order(&mut harness, &mut engine, serial, handler_first_reset)
                        .await;
                    serial += 1;
                    assert!(!harness.adapter.revalidate(&old));
                    assert!(
                        !td121_release_with_order(
                            &mut harness,
                            &mut engine,
                            serial,
                            END_KEYVAL,
                            107,
                            observer_first,
                        )
                        .await
                    );
                    assert!(!engine.context_word_is_known());
                    if !engine.native_cursor_manual_source_is_current() {
                        failures.push(format!("End source not current, observer_first={observer_first}: pending={:?}, local={:?}, reducer={:?}", engine.context_reset_rereceipt, engine.context_token, harness.adapter.current_token()));
                    }
                    assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
                    cycle09_assert_no_local_text_effect(&mut harness).await;

                    if native_shifts {
                        assert!(engine.native_cursor_manual_source_is_current(),
                            "before first Shift: prefix={prefix:?}, observer_first={observer_first}, pending={:?}, local={:?}, reducer={:?}", engine.context_reset_rereceipt, engine.context_token, harness.adapter.current_token());
                        for shift_index in 0..2 {
                            serial += 1;
                            assert!(
                                !legacy_key(
                                    &mut harness,
                                    &mut engine,
                                    serial,
                                    crate::protocol::KEY_LEFT_SHIFT,
                                    42,
                                    0,
                                )
                                .await
                            );
                            assert!(engine.native_cursor_manual_source_is_current(),
                                "after Shift press{shift_index}: prefix={prefix:?}, observer_first={observer_first}, pending={:?}, local={:?}, reducer={:?}", engine.context_reset_rereceipt, engine.context_token, harness.adapter.current_token());
                            serial += 1;
                            assert!(
                                !legacy_key(
                                    &mut harness,
                                    &mut engine,
                                    serial,
                                    crate::protocol::KEY_LEFT_SHIFT,
                                    42,
                                    RELEASE_MASK,
                                )
                                .await
                            );
                            assert!(engine.native_cursor_manual_source_is_current(),
                                "after Shift release{shift_index}: prefix={prefix:?}, observer_first={observer_first}, pending={:?}, local={:?}, reducer={:?}", engine.context_reset_rereceipt, engine.context_token, harness.adapter.current_token());
                        }
                        assert!(
                            engine.native_cursor_manual_source_is_current(),
                            "after Shift releases: prefix={prefix:?}, observer_first={observer_first}, pending={:?}, local={:?}, reducer={:?}", engine.context_reset_rereceipt, engine.context_token, harness.adapter.current_token()
                        );
                        cycle09_assert_no_local_text_effect(&mut harness).await;
                    }
                    let path = engine.path.clone();
                    let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
                    if disposition
                        != Ok(
                            lay::manual_toggle::ImeManualToggleOutcome::DelegateExactImeTail
                                .as_v3(),
                        )
                    {
                        failures.push(format!(
                                "prefix={prefix:?}, inserted={inserted:?}, observer_first={observer_first}: {disposition:?}"
                            ));
                        continue;
                    }
                    let (_engine, tail) = cycle09_visible_tail(&mut harness, engine).await;
                    assert!(cycle09_tail_is_authoritative(
                        &tail,
                        &path,
                        false,
                        &edited_word
                    ));
                }
            }
        }
        assert!(
            failures.is_empty(),
            "exact manual source failures: {failures:?}"
        );
    }
}
