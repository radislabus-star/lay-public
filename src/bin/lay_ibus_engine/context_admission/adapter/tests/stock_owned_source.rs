//! Explicit manual readout retains a current locally observed stock-client word.
use super::*;

async fn stock_owned_manual_fixture() -> (Harness, LayIbusEngine) {
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = new_engine(&harness);
    start_source_free_unknown(&mut harness, &mut engine).await;
    engine.set_client_capabilities(41);
    engine.config.nanda_precognition = true;
    engine.config.nanda_autocorrect = false;
    for (serial, ch) in "слово ".chars().enumerate() {
        legacy_key(
            &mut harness,
            &mut engine,
            163_000 + serial as u32,
            ch as u32,
            0,
            0,
        )
        .await;
    }
    let _ = terminal_delivery::legacy_effects(&mut harness).await;
    (harness, engine)
}

#[test]
fn stock_owned_word_space_manual_handoff_keeps_current_field_source() {
    zbus::block_on(bounded(async {
        let (mut harness, engine) = stock_owned_manual_fixture().await;
        assert_eq!(engine.committed_tail.buffer, "слово ");
        assert!(engine.context_word_is_known());
        assert!(engine.client_context.surrounding_text_snapshot.is_none());
        assert!(!engine.client_context.surrounding_text_callback_observed);
        let (engine, outcome, _) = cycle09_manual_toggle_with_effects(&mut harness, engine).await;
        assert_eq!(outcome, Ok((3, false)));
        let (mut engine, visible) = private_visible_tail_v4(&mut harness, engine).await;
        let visible = visible.unwrap();
        assert_eq!(
            visible.0, "passive:committed-tail",
            "locally observed current word must survive its explicit manual handoff"
        );
        assert_eq!(visible.1, "слово ");
        assert!(!visible.5.is_empty(), "same-field receipt is mandatory");
        engine.composition.buffer.push('x');
        assert!(!engine.context_stock_owned_manual_source_is_current());
        engine.composition.buffer.clear();
        // The same field retains its KnownStart scope across the explicit
        // layout handoff. Suppression must consume that same bounded source,
        // without requiring a surrounding callback the client cannot send.
        engine.layout_gesture.layout_is_ru = false;
        let epoch = engine.committed_tail.epoch;
        let path = engine.path.clone();
        let (mut engine, suppression) =
            cycle09_suppress_exact_replay(&mut harness, engine, "слово ", epoch, &path, false)
                .await;
        assert_eq!(suppression, Ok(true));
        assert!(cycle09_exact_suppression_is_armed(&engine));
        assert!(!cycle09_exact_handoff_is_live(&engine));
        assert!(engine.client_context.surrounding_text_snapshot.is_none());
        // Stock clients cannot add an SSI acknowledgement after native replay.
        // Preserve the locally tracked manual source, without inventing an ACK.
        let mut serial = 164_000;
        for _ in "слово ".chars() {
            assert!(
                !firefox_replay_callback(
                    &mut harness,
                    &mut engine,
                    &mut serial,
                    (KEY_BACKSPACE, 14, 0),
                )
                .await
            );
        }
        for ch in "ckjdj ".chars() {
            assert!(!firefox_replay_callback(
                &mut harness, &mut engine, &mut serial, (ch as u32, 0, 0),
            ).await);
        }
        let _ = terminal_delivery::legacy_effects(&mut harness).await;
        assert_eq!(engine.committed_tail.buffer, "ckjdj ");
        assert!(engine.context_word_is_known());
        assert!(!engine.context_has_native_replay_reset_ack());
        assert!(!engine.completed_native_replay_awaits_exact_receipt());
        let (mut engine, visible) = private_visible_tail_v4(&mut harness, engine).await;
        let visible = visible.unwrap();
        assert_eq!(visible.0, "passive:committed-tail");
        assert_eq!(visible.1, "ckjdj ");
        assert!(!visible.5.is_empty());
        // Clients that do supply surrounding text retain the receipt boundary.
        engine.client_context.surrounding_text_callback_observed = true;
        assert!(engine.completed_native_replay_awaits_exact_receipt());
        engine.client_context.surrounding_text_callback_observed = false;
        let (_engine, outcome, _) = cycle09_manual_toggle_with_effects(&mut harness, engine).await;
        assert_eq!(outcome, Ok((3, false)));
    }));
}

#[test]
fn stock_owned_manual_source_rejects_stale_unknown_and_observed_contexts() {
    zbus::block_on(bounded(async {
        for invalidation in [
            "exact-client",
            "observed-client",
            "sensitive",
            "sealed",
            "atomic",
            "no-scope",
            "no-token",
            "changed-tail",
            "expired",
        ] {
            let (mut harness, engine) = stock_owned_manual_fixture().await;
            let (mut engine, outcome, _) =
                cycle09_manual_toggle_with_effects(&mut harness, engine).await;
            assert_eq!(outcome, Ok((3, false)), "{invalidation}");
            match invalidation {
                "exact-client" => engine.client_context.exact_surrounding_refresh_available = true,
                "observed-client" => {
                    engine.client_context.surrounding_text_callback_observed = true
                }
                "sensitive" => engine.client_context.content_purpose = 8,
                "sealed" => engine.context_handoff_sealed = true,
                "atomic" => engine.atomic.active = true,
                "no-scope" => engine.context_word_scope = None,
                "no-token" => engine.context_token = None,
                "changed-tail" => engine.committed_tail.buffer.push('x'),
                "expired" => {
                    engine.shared.lock().unwrap().preserve_active_path_until =
                        Some(Instant::now() - Duration::from_secs(1))
                }
                _ => unreachable!(),
            }
            let (mut engine, visible) = private_visible_tail_v4(&mut harness, engine).await;
            let visible = visible.unwrap();
            assert_eq!(visible.0, "passive:unknown-context", "{invalidation}");
            assert!(visible.1.is_empty(), "{invalidation}");
            assert!(visible.5.is_empty(), "{invalidation}");
            engine.layout_gesture.layout_is_ru = false;
            let epoch = engine.committed_tail.epoch;
            let path = engine.path.clone();
            assert!(
                !engine.arm_exact_manual_toggle_autocorrect_suppression(
                    "слово ",
                    epoch,
                    &path,
                    false
                ),
                "{invalidation}"
            );
        }
    }));
}
