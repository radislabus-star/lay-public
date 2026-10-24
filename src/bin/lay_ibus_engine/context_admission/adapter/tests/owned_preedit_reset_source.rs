//! Retiring owned preedit may admit a new manual read, never the old word.
use super::*;
use crate::window_interaction::IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH;

async fn owned_preedit(harness: &mut Harness, serial: &mut u32, hints: u32) -> LayIbusEngine {
    owned_preedit_in_mode(harness, serial, hints, false).await
}

async fn owned_preedit_in_mode(
    harness: &mut Harness,
    serial: &mut u32,
    hints: u32,
    exact_refresh: bool,
) -> LayIbusEngine {
    let mut engine = td159_fresh_focus_source(harness).await;
    // Metadata belongs to the field before input. A privacy-mode transition
    // after typing must instead retire the source and is tested below.
    engine.set_content_type_state(0, hints);
    if exact_refresh {
        engine.set_client_capabilities(41 | IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH);
    }
    if hints != 0 || exact_refresh {
        exact_surrounding_receipt(harness, &mut engine, "prefix ").await;
    } else {
        // Stock owned mode begins before its first actual client snapshot.
        engine.config.nanda_precognition = true;
    }
    let mut commits = 0;
    for (ch, code) in [('a', 30), ('b', 48)] {
        assert!(legacy_key(harness, &mut engine, *serial, ch as u32, code, 0).await);
        *serial += 1;
        let effects = drain_output_to_proof(harness).await;
        engine.config.nanda_precognition = false;
        if harness
            .peer
            .detached_callback_reply_serials
            .contains(&(*serial - 1))
        {
            consume_detached_callback_reply(&mut harness.peer, *serial - 1).await;
        }
        assert!(!effects
            .iter()
            .any(|effect| effect == "DeleteSurroundingText"));
        commits += effects
            .iter()
            .filter(|effect| *effect == "CommitText")
            .count();
        assert!(legacy_key(harness, &mut engine, *serial, ch as u32, code, RELEASE_MASK,).await);
        *serial += 1;
        drain_output_to_proof(harness).await;
        if harness
            .peer
            .detached_callback_reply_serials
            .contains(&(*serial - 1))
        {
            consume_detached_callback_reply(&mut harness.peer, *serial - 1).await;
        }
    }
    if hints == 0 {
        exact_surrounding_receipt(harness, &mut engine, "prefix ").await;
        assert_eq!(commits, 0);
        assert!(engine.composition.legacy_word_preedit_active);
        assert_eq!(engine.composition.buffer, "ab");
    } else {
        // Private input uses the existing committed route, so it cannot
        // supply the owned-preedit prerequisite for the new admission.
        assert_eq!(commits, 2);
        assert!(!engine.composition.legacy_word_preedit_active);
        assert!(engine.composition.buffer.is_empty());
    }
    assert!(!engine.can_acquire_fresh_exact_manual_source());
    engine
}

#[test]
fn td159_owned_preedit_reset_admits_only_new_current_manual_source() {
    zbus::block_on(bounded(async {
        for repeat_reset in [false, true] {
            {
                let hints = 0;
                let mut harness = cycle09_harness().await;
                let mut serial = 9100;
                let mut engine = owned_preedit(&mut harness, &mut serial, hints).await;
                let source_token = engine.context_token.clone();
                firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
                if repeat_reset {
                    firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
                }
                assert!(engine.composition.buffer.is_empty());
                assert!(!engine.composition.legacy_word_preedit_active);
                assert!(engine.committed_tail.buffer.is_empty());
                assert!(!engine.can_acquire_fresh_exact_manual_source());
                assert!(!engine.exact_snapshot_manual_source_is_current());
                assert!(!engine.context_word_is_known());

                // This is a new actual field observation, deliberately different
                // from both the old preedit and its observed committed prefix.
                exact_surrounding_receipt(&mut harness, &mut engine, "πrefix uvwxyz").await;
                assert!(
                    engine.can_acquire_fresh_exact_manual_source(),
                    "authenticated owned-preedit retirement must permit a new current-field manual read: repeat={repeat_reset}, hints={hints}, source={source_token:?}, current={:?}, live={:?}, fresh={}, snapshot={:?}",
                    engine.context_token,
                    engine.live_context_token(),
                    harness.adapter.shared.reducer.lock().unwrap().fresh_manual_source_available,
                    engine.client_context.surrounding_text_snapshot,
                );
                let (engine, result, _) =
                    cycle09_manual_toggle_with_effects(&mut harness, engine).await;
                assert_eq!(result, Ok((3, false)));
                assert_eq!(engine.committed_tail.buffer, "uvwxyz");
                assert!(engine.exact_snapshot_manual_source_is_current());
                assert!(!engine.context_word_is_known());
                assert!(!engine.context_allows_manual_toggle());
                assert_eq!(
                    engine
                        .context_word_scope
                        .as_ref()
                        .unwrap()
                        .lineage()
                        .observed_suffix_chars,
                    0,
                );
            }
        }
    }));
}

#[test]
fn td159_owned_preedit_reset_rejects_unconfirmed_or_unsafe_source() {
    zbus::block_on(bounded(async {
        for gap in [
            "unobserved_reset",
            "selection",
            "interior",
            "sensitive",
            "revoked",
            "privacy_change",
            "private_before_input",
        ] {
            let mut harness = cycle09_harness().await;
            let mut serial = 9200;
            let hints = if gap == "private_before_input" {
                2048
            } else {
                0
            };
            let mut engine = owned_preedit(&mut harness, &mut serial, hints).await;
            if gap == "privacy_change" {
                engine.set_content_type_state(0, 2048);
            }
            if gap == "unobserved_reset" {
                engine.reset_for_ibus_soft_reset();
            } else {
                firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
            }
            if gap == "sensitive" {
                engine.set_content_type_state(8, 0);
            }
            if gap == "revoked" {
                assert!(harness
                    .adapter
                    .revoke_current_owner(engine.context_owner.as_ref().unwrap()));
            }
            let (cursor, anchor) = match gap {
                "selection" => (6, 0),
                "interior" => (3, 3),
                _ => (6, 6),
            };
            cycle09_surrounding_receipt(&mut harness, &mut engine, "uvwxyz", cursor, anchor).await;
            assert!(!engine.can_acquire_fresh_exact_manual_source(), "{gap}");
            let (engine, result, _) =
                cycle09_manual_toggle_with_effects(&mut harness, engine).await;
            assert!(matches!(result, Ok((0, _)) | Err(_)), "{gap}: {result:?}");
            assert!(!engine.exact_snapshot_manual_source_is_current(), "{gap}");
            assert!(!engine.context_word_is_known(), "{gap}");
        }
    }));
}

#[test]
fn td159_owned_preedit_reset_after_surrounding_requests_new_same_gesture_read() {
    zbus::block_on(bounded(async {
        let mut harness = cycle09_harness().await;
        let mut serial = 9300;
        // This contract requires ownership to survive a matching snapshot.
        // Declare the explicit exact-refresh mode before its first key. Stock
        // delayed projections now finish literal input (covered separately).
        let mut engine = owned_preedit_in_mode(&mut harness, &mut serial, 0, true).await;
        // Actual Qt ordering: the last snapshot is delivered between two
        // Reset callbacks, so the second Reset retires it before the gesture.
        exact_surrounding_receipt(&mut harness, &mut engine, "prefix ab").await;
        firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
        exact_surrounding_receipt(&mut harness, &mut engine, "").await;
        exact_surrounding_receipt(&mut harness, &mut engine, "uvwxyz").await;
        firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
        assert!(engine.client_context.surrounding_text_snapshot.is_none());
        assert!(engine.composition.buffer.is_empty());
        assert!(engine.committed_tail.buffer.is_empty());
        assert!(!engine.can_acquire_fresh_exact_manual_source());
        let fresh = harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .fresh_manual_source_available;
        let (mut engine, result, effects) =
            cycle09_manual_toggle_with_effects(&mut harness, engine).await;
        assert_eq!(
            result,
            Ok(lay::manual_toggle::ImeManualToggleOutcome::AwaitingExactSnapshot.as_v3()),
            "final Reset must allow one new actual read for the same gesture, never the retired snapshot; fresh={fresh}",
        );
        assert_eq!(
            effects
                .iter()
                .filter(|message| message
                    .header()
                    .member()
                    .is_some_and(|member| member.as_str() == "RequireSurroundingText"))
                .count(),
            1,
        );
        assert!(engine.layout_gesture.pending_manual_toggle);
        assert!(engine.layout_gesture.pending_manual_refresh_at.is_some());
        assert!(!engine.context_word_is_known());
        assert!(!engine.exact_snapshot_manual_source_is_current());

        let (next, result, effects) =
            cycle09_manual_toggle_with_effects(&mut harness, engine).await;
        engine = next;
        assert_eq!(
            result,
            Ok(lay::manual_toggle::ImeManualToggleOutcome::AwaitingExactSnapshot.as_v3())
        );
        assert!(!effects.iter().any(|message| message
            .header()
            .member()
            .is_some_and(|member| member.as_str() == "RequireSurroundingText")));

        // The read response is for a deliberately different current word.
        // The callback must remain read-only; the same daemon gesture owns
        // the existing exact-tail handoff.
        exact_surrounding_receipt(&mut harness, &mut engine, "πrefix qwerty").await;
        let (engine, result, _) = cycle09_manual_toggle_with_effects(&mut harness, engine).await;
        assert_eq!(
            result,
            Ok(lay::manual_toggle::ImeManualToggleOutcome::DelegateExactImeTail.as_v3())
        );
        assert_eq!(engine.committed_tail.buffer, "qwerty");
        assert!(engine.exact_snapshot_manual_source_is_current());
        assert!(!engine.context_word_is_known());
        assert!(!engine.layout_gesture.pending_manual_toggle);
        assert!(engine.layout_gesture.pending_manual_refresh_at.is_none());
    }));
}

#[test]
fn td159_owned_preedit_reset_new_read_cancels_without_text_authority() {
    zbus::block_on(bounded(async {
        for gap in [
            "selection",
            "interior",
            "sensitive",
            "revoked",
            "privacy_change",
            "capability_loss",
            "reset",
            "expired",
            "expired_rpc",
        ] {
            let mut harness = cycle09_harness().await;
            let mut serial = 9400;
            let mut engine = owned_preedit(&mut harness, &mut serial, 0).await;
            firefox_replay_reset(&mut harness, &mut engine, &mut serial).await;
            let (mut engine, result, _) =
                cycle09_manual_toggle_with_effects(&mut harness, engine).await;
            assert_eq!(
                result,
                Ok(lay::manual_toggle::ImeManualToggleOutcome::AwaitingExactSnapshot.as_v3()),
                "{gap}"
            );
            match gap {
                "sensitive" => engine.set_content_type_state(8, 0),
                "privacy_change" => engine.set_content_type_state(0, 2048),
                "capability_loss" => engine.set_client_capabilities(0),
                "revoked" => {
                    assert!(harness
                        .adapter
                        .revoke_current_owner(engine.context_owner.as_ref().unwrap()));
                }
                "reset" => firefox_replay_reset(&mut harness, &mut engine, &mut serial).await,
                "expired" | "expired_rpc" => {
                    engine.layout_gesture.pending_manual_refresh_at =
                        Some(Instant::now() - Duration::from_millis(701))
                }
                _ => {}
            }
            let (cursor, anchor) = match gap {
                "selection" => (6, 0),
                "interior" => (3, 3),
                _ => (6, 6),
            };
            if gap != "expired_rpc" {
                cycle09_surrounding_receipt(&mut harness, &mut engine, "qwerty", cursor, anchor)
                    .await;
            }
            let (mut engine, result, effects) =
                cycle09_manual_toggle_with_effects(&mut harness, engine).await;
            assert!(
                matches!(result, Ok((0 | 4, false)) | Err(_)),
                "{gap}: {result:?}"
            );
            assert!(
                !effects.iter().any(|message| message
                    .header()
                    .member()
                    .is_some_and(|member| member.as_str() == "RequireSurroundingText")),
                "{gap}: must not renew a cancelled gesture"
            );
            assert!(engine.committed_tail.buffer.is_empty(), "{gap}");
            assert!(!engine.exact_snapshot_manual_source_is_current(), "{gap}");
            assert!(!engine.context_word_is_known(), "{gap}");
            // A later poll is still the cancelled physical gesture. It must
            // neither reacquire text nor issue a second read request.
            let (next, result, effects) =
                cycle09_manual_toggle_with_effects(&mut harness, engine).await;
            engine = next;
            assert!(
                matches!(result, Ok((0 | 4, false)) | Err(_)),
                "{gap}: {result:?}"
            );
            assert!(
                !effects.iter().any(|message| message
                    .header()
                    .member()
                    .is_some_and(|member| member.as_str() == "RequireSurroundingText")),
                "{gap}: later poll must not renew a cancelled gesture"
            );
            assert!(engine.committed_tail.buffer.is_empty(), "{gap}");
        }
    }));
}
