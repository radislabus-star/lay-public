//! Reuse the actual native replay callbacks and Reset/SSI contract. The explicit
//! current-Firefox fact substitutes only the desktop observation in this proof;
//! it supplies neither a replay scope nor mutation authority.
use super::*;

async fn partial_replay() -> (Harness, LayIbusEngine, u64) {
    let (mut harness, mut engine) = armed_exact_replay("prefix abc ", "abc ", true, false).await;
    engine.config.layout_backend = "kde".into();
    engine.client_context.exact_surrounding_refresh_available = true;
    exact_replay_surrounding_receipt(&mut harness, &mut engine, " prefix abc ").await;
    let epoch = engine.committed_tail.epoch;
    for serial in 0..4 {
        for state in [0, RELEASE_MASK] {
            assert!(
                !legacy_key(
                    &mut harness,
                    &mut engine,
                    28_000 + serial * 2 + u32::from(state != 0),
                    KEY_BACKSPACE,
                    14,
                    state,
                )
                .await
            );
        }
    }
    for state in [0, RELEASE_MASK] {
        assert!(
            !legacy_key(
                &mut harness,
                &mut engine,
                28_010 + u32::from(state != 0),
                0x0100_0000 | 'ф' as u32,
                30,
                state,
            )
            .await
        );
    }
    no_legacy_text_output(&mut harness).await;
    assert_eq!(engine.committed_tail.buffer, " prefix ф");
    text_free_owned_soft_reset(&mut harness, &mut engine, 28_012).await;
    assert!(engine.context_reset_rereceipt.is_some());
    (harness, engine, epoch)
}

#[test]
fn kde_firefox_delayed_erased_boundary_keeps_only_inert_native_lineage() {
    zbus::block_on(bounded(async {
        for closing in [" ", "\u{a0}"] {
            let (mut harness, mut engine, epoch) = partial_replay().await;
            let snapshot = SurroundingTextSnapshot::new(" prefix\u{a0}".into(), 8, 8);
            assert!(!engine.exact_replay_contains_prior_snapshot(&snapshot));
            assert!(!engine.native_replay_processed(epoch, TARGET_PATH, true));
            engine.observe_external_surrounding_text_for_owned_erasure(Some(snapshot), true);
            assert!(
                engine.context_reset_rereceipt.is_some(),
                "delayed erased boundary must preserve inert replay lineage"
            );
            assert!(!engine.context_reset_rereceipt.as_ref().unwrap().confirmed);
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert!(!engine.context_word_is_known());
            no_legacy_text_output(&mut harness).await;

            for (offset, (key, code)) in [('и', 48), ('с', 46), (' ', 57)].into_iter().enumerate()
            {
                for state in [0, RELEASE_MASK] {
                    assert!(
                        !legacy_key(
                            &mut harness,
                            &mut engine,
                            28_020 + offset as u32 * 2 + u32::from(state != 0),
                            if key == ' ' {
                                KEY_SPACE
                            } else {
                                0x0100_0000 | key as u32
                            },
                            code,
                            state,
                        )
                        .await
                    );
                }
            }
            no_legacy_text_output(&mut harness).await;
            assert!(engine.native_replay_processed(epoch, TARGET_PATH, true));
            text_free_owned_soft_reset(&mut harness, &mut engine, 28_030).await;
            exact_replay_surrounding_receipt(
                &mut harness,
                &mut engine,
                &format!(" prefix фис{closing}"),
            )
            .await;
            // The delayed erased boundary is inert. It cannot also normalize
            // the final result: only the exact planned ASCII closing Space may
            // confirm it; a changed full result must still discard provenance.
            assert_eq!(
                engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                closing == " ",
                "closing={closing:?}"
            );
            if closing != " " {
                assert!(engine.context_reset_rereceipt.is_none());
            }
            no_legacy_text_output(&mut harness).await;
        }
    }));
}

#[test]
fn kde_firefox_erased_boundary_rejects_foreign_or_unproved_observations() {
    zbus::block_on(bounded(async {
        for case in [
            "foreign-client",
            "gnome",
            "letter",
            "count",
            "tab",
            "cursor",
            "selection",
            "shared",
            "expired",
            "capability",
        ] {
            let (mut harness, mut engine, _) = partial_replay().await;
            let mut snapshot = SurroundingTextSnapshot::new(" prefix\u{a0}".into(), 8, 8);
            let mut current_firefox = true;
            match case {
                "foreign-client" => current_firefox = false,
                "gnome" => engine.config.layout_backend = "gnome".into(),
                "letter" => snapshot.text = " prefiy\u{a0}".into(),
                "count" => {
                    snapshot = SurroundingTextSnapshot::new(" prefix\u{a0}\u{a0}".into(), 9, 9)
                }
                "tab" => snapshot.text = " prefix\t".into(),
                "cursor" => {
                    snapshot.cursor_pos = 6;
                    snapshot.anchor_pos = 6;
                }
                "selection" => snapshot.anchor_pos = 0,
                "shared" => engine.shared.lock().unwrap().autocorrect_suppression = None,
                "expired" => {
                    let Some(crate::protocol::AutocorrectSuppression::ExactReplay(scope)) =
                        engine.committed_tail.autocorrect_suppression.as_mut()
                    else {
                        panic!("scope")
                    };
                    scope.expires_at = Instant::now() - Duration::from_secs(1);
                    let expired = scope.clone();
                    engine.shared.lock().unwrap().autocorrect_suppression = Some(
                        crate::protocol::AutocorrectSuppression::ExactReplay(expired),
                    );
                }
                "capability" => engine.client_context.exact_surrounding_refresh_available = false,
                _ => unreachable!(),
            }
            engine.observe_external_surrounding_text_for_owned_erasure(
                Some(snapshot),
                current_firefox,
            );
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{case}"
            );
            assert!(engine.context_reset_rereceipt.is_none(), "{case}");
            assert!(!engine.context_word_is_known(), "{case}");
            no_legacy_text_output(&mut harness).await;
        }

        let (mut harness, mut engine) =
            armed_exact_replay("prefix abc ", "abc ", true, false).await;
        engine.config.layout_backend = "kde".into();
        engine.client_context.exact_surrounding_refresh_available = true;
        assert!(
            !engine.kde_firefox_native_erased_boundary_echo(&SurroundingTextSnapshot::new(
                " prefix\u{a0}".into(),
                8,
                8
            )),
            "unobserved future erasure cannot retain lineage"
        );
        no_legacy_text_output(&mut harness).await;
    }));
}
