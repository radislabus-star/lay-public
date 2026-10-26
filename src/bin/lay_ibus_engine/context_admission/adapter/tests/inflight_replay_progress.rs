//! Ordered client progress is inert until an exact final replay receipt.
use super::*;

async fn inflight_target_before_delayed_snapshot() -> (Harness, LayIbusEngine, String, u32) {
    let mut serial = 91_000;
    let (mut harness, target, path, epoch) = firefox_empty_reset_six_scalar_handoff(serial).await;
    serial += 40;
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
                    &mut serial,
                    (KEY_BACKSPACE, 14, state),
                )
                .await
            );
        }
    }
    firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
    cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix ", 7, 7).await;
    for (index, (ch, code)) in [('п', 34), ('р', 35), ('и', 48), ('в', 32), ('е', 20)]
        .into_iter()
        .enumerate()
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
        if index == 0 {
            firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
            cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix п", 8, 8).await;
        }
    }
    firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
    assert!(engine.exact_replay_reset_provenance(false).is_some());
    assert!(engine.exact_replay_reset_provenance(true).is_none());
    (harness, engine, path, serial)
}

#[test]
fn inflight_native_replay_growing_surface_with_lagging_caret_waits_for_exact_final() {
    zbus::block_on(bounded(async {
        let (mut harness, mut engine, path, mut serial) =
            inflight_target_before_delayed_snapshot().await;
        // Five presses are observed; the actual client reports four characters
        // with its preceding caret after one. This cannot authorize a text effect.
        cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix прив", 8, 8).await;
        assert!(
            engine
                .context_reset_rereceipt
                .as_ref()
                .is_some_and(|p| !p.confirmed),
            "actual growing native surface must retain inert lineage during replay"
        );
        assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        assert!(!engine.context_word_is_known());
        cycle09_assert_no_local_text_effect(&mut harness).await;
        for state in [0, RELEASE_MASK] {
            assert!(
                !firefox_replay_callback(
                    &mut harness,
                    &mut engine,
                    &mut serial,
                    (0x0100_0000 | 'т' as u32, 49, state),
                )
                .await
            );
        }
        firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
        cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix привет", 13, 13).await;
        assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
        assert!(!engine.context_word_is_known());
        let (engine, outcome) = cycle09_manual_toggle(&mut harness, engine).await;
        assert_eq!(
            outcome,
            Ok(lay::manual_toggle::ImeManualToggleOutcome::DelegateExactImeTail.as_v3())
        );
        let (_, tail) = cycle09_visible_tail(&mut harness, engine).await;
        assert!(cycle09_tail_is_authoritative(&tail, &path, true, "привет"));
        cycle09_assert_no_local_text_effect(&mut harness).await;
    }));
}

#[test]
fn inflight_native_replay_contradictory_progress_cannot_revive_from_final_surface() {
    for (surface, cursor, anchor) in [
        ("prefix прив", 8, 9),   // Selection is never inert replay progress.
        ("prefix прив", 7, 7),   // A growing body cannot move the caret backwards.
        ("prefix привет", 8, 8), // The sixth replacement press has not been observed.
        ("otherx прив", 8, 8),   // An equal-length external prefix is still foreign.
    ] {
        zbus::block_on(bounded(async {
            let (mut harness, mut engine, _path, _serial) =
                inflight_target_before_delayed_snapshot().await;
            cycle09_surrounding_receipt(&mut harness, &mut engine, surface, cursor, anchor).await;
            assert!(
                engine.context_reset_rereceipt.is_none(),
                "{surface:?} {cursor} {anchor}"
            );
            cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix привет", 13, 13).await;
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert!(!engine.context_word_is_known());
            assert!(engine.committed_tail.pending_completion_learning.is_none());
            cycle09_assert_no_local_text_effect(&mut harness).await;
        }));
    }
}
