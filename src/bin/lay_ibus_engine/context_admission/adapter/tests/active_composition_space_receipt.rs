//! Ordinary Space must acknowledge only the boundary emitted by this owner.
use super::*;

async fn compose_and_commit_space(prefix: &str) -> (Harness, LayIbusEngine) {
    let mut harness = cycle09_harness().await;
    let mut engine = new_engine(&harness);
    start_source_free_unknown(&mut harness, &mut engine).await;
    engine.config.auto_replace = false;
    engine.config.typing_assist = false;
    engine.config.nanda_precognition = false;
    engine.config.ime_bracket_candidates = false;
    engine.set_client_capabilities(1_073_741_865);
    engine.set_content_type_state(0, 0);
    let cursor = prefix.chars().count() as u32;
    cycle09_surrounding_receipt(&mut harness, &mut engine, prefix, cursor, cursor).await;
    let mut serial = 159_700;
    for (ch, keycode) in [('a', 30), ('b', 48), ('c', 46)] {
        assert!(legacy_key(&mut harness, &mut engine, serial, ch as u32, keycode, 0).await);
        serial += 1;
        let (members, commits) = drain_output_to_text_proof(&mut harness).await;
        assert!(
            commits.is_empty(),
            "fixture must use live preedit: {members:?}"
        );
        assert!(!members.iter().any(|m| m == "DeleteSurroundingText"));
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                serial,
                ch as u32,
                keycode,
                RELEASE_MASK,
            )
            .await
        );
        serial += 1;
        cycle09_assert_no_local_text_effect(&mut harness).await;
    }
    assert!(engine.composition.legacy_word_preedit_active);
    assert_eq!(engine.composition.buffer, "abc");
    assert!(legacy_key(&mut harness, &mut engine, serial, KEY_SPACE, 57, 0).await);
    let (members, commits) = drain_output_to_text_proof(&mut harness).await;
    assert_eq!(commits, ["abc "]);
    assert!(!members.iter().any(|m| m == "DeleteSurroundingText"));
    assert!(
        legacy_key(
            &mut harness,
            &mut engine,
            serial + 1,
            KEY_SPACE,
            57,
            RELEASE_MASK,
        )
        .await
    );
    cycle09_assert_no_local_text_effect(&mut harness).await;
    assert_eq!(engine.committed_tail.buffer, "abc ");
    assert!(engine
        .committed_tail
        .pending_visible_postcondition
        .is_some());
    (harness, engine)
}

async fn assert_observed_owned_space(prefix: &str) {
    let (mut harness, mut engine) = compose_and_commit_space(prefix).await;
    let epoch = engine.committed_tail.epoch;
    let known = engine.context_word_is_known();
    assert!(engine
        .committed_tail
        .pending_visible_postcondition
        .as_ref()
        .unwrap()
        .feedback
        .is_none());
    let text = format!("{prefix}abc\u{a0}");
    let cursor = text.chars().count() as u32;
    cycle09_surrounding_receipt(&mut harness, &mut engine, &text, cursor, cursor).await;
    assert_eq!(
        engine.committed_tail.buffer, "abc\u{a0}",
        "active composition Space must acknowledge its exact observed boundary",
    );
    assert_eq!(engine.committed_tail.epoch, epoch);
    assert!(engine
        .committed_tail
        .pending_visible_postcondition
        .is_none());
    assert!(engine.current_external_snapshot_agrees_with_owned_tail());
    assert!(engine.composition.buffer.is_empty());
    assert_eq!(engine.context_word_is_known(), known);
    assert!(engine.committed_tail.pending_completion_learning.is_none());
    assert!(engine.committed_tail.recent_replace.is_none());
}

#[test]
fn td159_active_composition_space_receipt_empty_prefix() {
    zbus::block_on(bounded(assert_observed_owned_space("")));
}

#[test]
fn td159_active_composition_space_receipt_unicode_prefix() {
    zbus::block_on(bounded(assert_observed_owned_space("πrefix ")));
}

#[test]
fn td159_active_composition_space_receipt_exact_ascii_and_old_nbsp() {
    zbus::block_on(bounded(async {
        let prefix = "πrefix\u{a0}";
        let (mut harness, mut engine) = compose_and_commit_space(prefix).await;
        let text = format!("{prefix}abc ");
        let cursor = text.chars().count() as u32;
        cycle09_surrounding_receipt(&mut harness, &mut engine, &text, cursor, cursor).await;
        assert_eq!(engine.committed_tail.buffer, "abc ");
        assert!(engine
            .committed_tail
            .pending_visible_postcondition
            .is_none());
        assert!(engine.current_external_snapshot_agrees_with_owned_tail());
    }));
}

#[test]
fn td159_active_composition_space_receipt_rejects_changed_client_state() {
    zbus::block_on(bounded(async {
        for (text, cursor, anchor) in [
            ("changed abc\u{a0}", 12, 12),
            ("πrefix\u{a0}abd\u{a0}", 11, 11),
            ("πrefix\u{a0}abc\u{a0}", 11, 10),
            ("πrefix\u{a0}abc\u{a0}", 9, 9),
            // An older NBSP must never become equivalent to ordinary Space.
            ("πrefix abc\u{a0}", 11, 11),
        ] {
            let (mut harness, mut engine) = compose_and_commit_space("πrefix\u{a0}").await;
            let epoch = engine.committed_tail.epoch;
            cycle09_surrounding_receipt(&mut harness, &mut engine, text, cursor, anchor).await;
            assert_eq!(
                engine.committed_tail.buffer, "abc ",
                "{text:?}/{cursor}/{anchor}"
            );
            assert_eq!(engine.committed_tail.epoch, epoch);
            assert!(!engine.current_external_snapshot_agrees_with_owned_tail());
            assert!(engine.committed_tail.recent_replace.is_none());
        }
    }));
}

#[test]
fn td159_active_composition_space_receipt_rejects_expired_epoch_and_owner() {
    zbus::block_on(bounded(async {
        for invalid in ["expired", "epoch", "owner"] {
            let (mut harness, mut engine) = compose_and_commit_space("").await;
            match invalid {
                "expired" => {
                    engine
                        .committed_tail
                        .pending_visible_postcondition
                        .as_mut()
                        .unwrap()
                        .dispatched_at -= std::time::Duration::from_millis(1501);
                }
                "epoch" => {
                    assert!(
                        legacy_key(&mut harness, &mut engine, 159_790, 'd' as u32, 32, 0).await
                    );
                    let (members, _) = drain_output_to_text_proof(&mut harness).await;
                    assert!(!members.iter().any(|m| m == "DeleteSurroundingText"));
                    assert!(
                        legacy_key(
                            &mut harness,
                            &mut engine,
                            159_791,
                            'd' as u32,
                            32,
                            RELEASE_MASK
                        )
                        .await
                    );
                    cycle09_assert_no_local_text_effect(&mut harness).await;
                }
                "owner" => {
                    // Fault injection at the shared single-owner boundary.
                    engine.shared.lock().unwrap().active_path = Some("/foreign".into());
                }
                _ => unreachable!(),
            }
            let tail = engine.committed_tail.buffer.clone();
            cycle09_surrounding_receipt(&mut harness, &mut engine, "abc\u{a0}", 4, 4).await;
            assert_eq!(engine.committed_tail.buffer, tail, "{invalid}");
            assert!(engine.committed_tail.pending_completion_learning.is_none());
            assert!(engine.committed_tail.recent_replace.is_none());
        }
    }));
}
