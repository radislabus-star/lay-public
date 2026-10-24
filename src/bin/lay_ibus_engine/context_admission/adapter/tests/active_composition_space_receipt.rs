//! Ordinary Space must acknowledge only the boundary emitted by this owner.
use super::*;

async fn compose_owned_word(prefix: &str) -> (Harness, LayIbusEngine) {
    compose_owned_word_with_initial_snapshot(Some(prefix), 1_073_741_865).await
}

async fn compose_owned_word_with_initial_snapshot(
    prefix: Option<&str>,
    capabilities: u32,
) -> (Harness, LayIbusEngine) {
    let mut harness = cycle09_harness().await;
    let mut engine = new_engine(&harness);
    start_source_free_unknown(&mut harness, &mut engine).await;
    engine.config.auto_replace = false;
    engine.config.typing_assist = false;
    // Stock clients without an initial observation enter literal preedit only
    // with the same completion setting enabled in the native failing client.
    engine.config.nanda_precognition = prefix.is_none();
    engine.config.ime_bracket_candidates = false;
    engine.set_client_capabilities(capabilities);
    engine.set_content_type_state(0, 0);
    if let Some(prefix) = prefix {
        let cursor = prefix.chars().count() as u32;
        cycle09_surrounding_receipt(&mut harness, &mut engine, prefix, cursor, cursor).await;
    }
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
    (harness, engine)
}

async fn compose_and_commit_space(prefix: &str) -> (Harness, LayIbusEngine) {
    let (harness, engine) = compose_owned_word(prefix).await;
    let (harness, engine) = commit_owned_space(harness, engine).await;
    assert!(engine
        .committed_tail
        .pending_visible_postcondition
        .is_some());
    (harness, engine)
}

async fn commit_owned_space(
    mut harness: Harness,
    mut engine: LayIbusEngine,
) -> (Harness, LayIbusEngine) {
    let serial = 159_706;
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
    (harness, engine)
}

async fn compose_and_accept_completion(prefix: &str) -> (Harness, LayIbusEngine) {
    let (harness, engine) = compose_owned_word(prefix).await;
    accept_owned_completion(harness, engine).await
}

async fn accept_owned_completion(
    mut harness: Harness,
    mut engine: LayIbusEngine,
) -> (Harness, LayIbusEngine) {
    // Inject a completed append candidate, not a still-pending worker display.
    engine.cancel_precognition_display_generation();
    set_fixture_append_completion(&mut engine, "def");
    engine.composition.preedit_display_only_pending = false;
    assert!(
        engine.context_word_is_known() || engine.context_owns_active_preedit_append_completion()
    );
    assert!(legacy_key(&mut harness, &mut engine, 159_710, KEY_TAB, 15, 0).await);
    let (members, commits) = drain_output_to_text_proof(&mut harness).await;
    assert_eq!(commits, ["abcdef "]);
    assert!(!members.iter().any(|m| m == "DeleteSurroundingText"));
    assert!(
        legacy_key(
            &mut harness,
            &mut engine,
            159_711,
            KEY_TAB,
            15,
            RELEASE_MASK
        )
        .await
    );
    cycle09_assert_no_local_text_effect(&mut harness).await;
    assert_eq!(engine.committed_tail.buffer, "abcdef ");
    (harness, engine)
}

#[test]
fn td159_stock_user_space_first_surrounding_retains_v4_tail() {
    zbus::block_on(bounded(async {
        for boundary in ['\u{a0}', ' '] {
            let (harness, engine) = compose_owned_word_with_initial_snapshot(None, 41).await;
            assert!(engine.client_context.surrounding_text_snapshot.is_none());
            assert!(engine.composition.legacy_preedit_start_boundary.is_none());
            let (mut harness, mut engine) = commit_owned_space(harness, engine).await;
            let epoch = engine.committed_tail.epoch;
            let text = format!("abc{boundary}");
            cycle09_surrounding_receipt(&mut harness, &mut engine, &text, 4, 4).await;
            assert_eq!(engine.committed_tail.buffer, text);
            assert_eq!(engine.committed_tail.epoch, epoch);
            assert!(engine
                .committed_tail
                .pending_visible_postcondition
                .is_none());
            assert!(engine.committed_tail.recent_replace.is_none());
            assert!(engine.committed_tail.pending_completion_learning.is_none());
            engine = bridge_delegate_exact_without_gui_edit(&mut harness, engine).await;
            let (_, visible) = private_visible_tail_v4(&mut harness, engine).await;
            let (state, tail, _, _, path, field_receipt, _) = visible.unwrap();
            assert_eq!(tail, text);
            assert_eq!(state, "passive:committed-tail");
            assert!(!path.is_empty());
            assert!(!field_receipt.is_empty());
        }
    }));
}

#[test]
fn td159_stock_user_space_first_receipt_rejects_other_field_text_and_caret() {
    zbus::block_on(bounded(async {
        for (text, cursor, anchor) in [
            ("old abc\u{a0}", 8, 8),
            ("old\u{a0}abc\u{a0}", 8, 8),
            ("abd\u{a0}", 4, 4),
            ("abc\u{a0}\u{a0}", 5, 5),
            ("abc\u{a0}", 3, 3),
            ("abc\u{a0}", 4, 3),
            ("abc\u{a0}later", 4, 4),
        ] {
            let (harness, engine) = compose_owned_word_with_initial_snapshot(None, 41).await;
            let (mut harness, mut engine) = commit_owned_space(harness, engine).await;
            let epoch = engine.committed_tail.epoch;
            cycle09_surrounding_receipt(&mut harness, &mut engine, text, cursor, anchor).await;
            assert_eq!(
                engine.committed_tail.buffer, "abc ",
                "{text:?}/{cursor}/{anchor}"
            );
            assert_eq!(engine.committed_tail.epoch, epoch);
            assert!(!engine.current_external_snapshot_agrees_with_owned_tail());
            assert!(engine.committed_tail.recent_replace.is_none());
            assert!(engine.committed_tail.pending_completion_learning.is_none());
        }
    }));
}

#[test]
fn td159_stock_user_space_first_receipt_rejects_expired_epoch_and_owner() {
    zbus::block_on(bounded(async {
        for invalid in ["expired", "epoch", "owner"] {
            let (harness, engine) = compose_owned_word_with_initial_snapshot(None, 41).await;
            let (mut harness, mut engine) = commit_owned_space(harness, engine).await;
            match invalid {
                "expired" => {
                    engine
                        .committed_tail
                        .pending_visible_postcondition
                        .as_mut()
                        .unwrap()
                        .dispatched_at -= std::time::Duration::from_millis(1501)
                }
                "epoch" => engine.committed_tail.epoch += 1,
                "owner" => engine.shared.lock().unwrap().active_path = Some("/foreign".into()),
                _ => unreachable!(),
            }
            cycle09_surrounding_receipt(&mut harness, &mut engine, "abc\u{a0}", 4, 4).await;
            assert_eq!(engine.committed_tail.buffer, "abc ", "{invalid}");
            assert!(engine.committed_tail.recent_replace.is_none());
            assert!(engine.committed_tail.pending_completion_learning.is_none());
        }
    }));
}

#[test]
fn td159_stock_completion_first_surrounding_after_commit_retains_v4_tail() {
    zbus::block_on(bounded(async {
        let (harness, engine) = compose_owned_word_with_initial_snapshot(None, 41).await;
        assert!(engine.client_context.surrounding_text_supported);
        assert!(!engine.client_context.exact_surrounding_refresh_available);
        assert!(engine.client_context.surrounding_text_snapshot.is_none());
        assert!(engine.composition.legacy_preedit_start_boundary.is_none());
        let (mut harness, mut engine) = accept_owned_completion(harness, engine).await;
        let epoch = engine.committed_tail.epoch;
        cycle09_surrounding_receipt(&mut harness, &mut engine, "abcdef\u{a0}", 7, 7).await;
        assert_eq!(engine.committed_tail.buffer, "abcdef\u{a0}");
        assert_eq!(engine.committed_tail.epoch, epoch);
        assert!(engine
            .committed_tail
            .pending_visible_postcondition
            .is_none());
        assert!(engine.committed_tail.recent_replace.is_none());
        assert!(engine.committed_tail.pending_completion_learning.is_none());
        engine = bridge_delegate_exact_without_gui_edit(&mut harness, engine).await;
        let (_, visible) = private_visible_tail_v4(&mut harness, engine).await;
        let (state, tail, _, _, path, field_receipt, _) = visible.unwrap();
        assert_eq!(tail, "abcdef\u{a0}");
        assert_eq!(state, "passive:committed-tail");
        assert!(!path.is_empty());
        assert!(!field_receipt.is_empty());
    }));
}

#[test]
fn td159_stock_completion_first_receipt_rejects_other_field_text_and_caret() {
    zbus::block_on(bounded(async {
        for (text, cursor, anchor) in [
            ("old abcdef\u{a0}", 11, 11),
            ("old\u{a0}abcdef\u{a0}", 11, 11),
            ("abcdeg\u{a0}", 7, 7),
            ("abcdef\u{a0}\u{a0}", 8, 8),
            ("abcdef\u{a0}", 6, 6),
            ("abcdef\u{a0}", 7, 6),
            ("abcdef\u{a0}later", 7, 7),
        ] {
            let (harness, engine) = compose_owned_word_with_initial_snapshot(None, 41).await;
            let (mut harness, mut engine) = accept_owned_completion(harness, engine).await;
            let epoch = engine.committed_tail.epoch;
            cycle09_surrounding_receipt(&mut harness, &mut engine, text, cursor, anchor).await;
            assert_eq!(
                engine.committed_tail.buffer, "abcdef ",
                "{text:?}/{cursor}/{anchor}"
            );
            assert_eq!(engine.committed_tail.epoch, epoch);
            assert!(!engine.current_external_snapshot_agrees_with_owned_tail());
            assert!(engine.committed_tail.recent_replace.is_none());
            assert!(engine.committed_tail.pending_completion_learning.is_none());
        }
    }));
}

#[test]
fn td159_stock_completion_first_receipt_rejects_expired_epoch_and_owner() {
    zbus::block_on(bounded(async {
        for invalid in ["expired", "epoch", "owner"] {
            let (harness, engine) = compose_owned_word_with_initial_snapshot(None, 41).await;
            let (mut harness, mut engine) = accept_owned_completion(harness, engine).await;
            match invalid {
                "expired" => {
                    engine
                        .committed_tail
                        .pending_visible_postcondition
                        .as_mut()
                        .unwrap()
                        .dispatched_at -= std::time::Duration::from_millis(1501)
                }
                "epoch" => engine.committed_tail.epoch += 1,
                "owner" => engine.shared.lock().unwrap().active_path = Some("/foreign".into()),
                _ => unreachable!(),
            }
            cycle09_surrounding_receipt(&mut harness, &mut engine, "abcdef\u{a0}", 7, 7).await;
            assert_eq!(engine.committed_tail.buffer, "abcdef ", "{invalid}");
            assert!(engine.committed_tail.recent_replace.is_none());
            assert!(engine.committed_tail.pending_completion_learning.is_none());
        }
    }));
}

#[test]
fn td159_stock_completion_expectation_requires_current_exact_owned_edit() {
    zbus::block_on(bounded(async {
        for invalid in [
            "wrong_user_input",
            "exact_refresh",
            "mode",
            "preedit",
            "tail",
            "cursor",
            "token",
            "sealed",
            "atomic",
            "sensitive",
            "wrong_from",
            "wrong_to",
        ] {
            let (_harness, mut engine) = compose_owned_word_with_initial_snapshot(None, 41).await;
            let from = if invalid == "wrong_from" {
                "abd"
            } else {
                "abc"
            };
            let to = if invalid == "wrong_to" {
                "abcdeg "
            } else {
                "abcdef "
            };
            let action = lay::text_edit::plan_ime_candidate_accept_edit("fixture", 900, from, to);
            let edit = lay::text_edit::authorize_backend_edit(
                lay::text_edit::TextEditBackend::Ime,
                action,
            )
            .into_authorized()
            .unwrap();
            match invalid {
                "exact_refresh" => engine.client_context.exact_surrounding_refresh_available = true,
                "mode" => {
                    engine.composition.word_input_mode = Some(WordInputMode::TerminalPassthrough)
                }
                "preedit" => engine.composition.legacy_word_preedit_active = false,
                "tail" => engine.committed_tail.buffer = "old abc".into(),
                "cursor" => engine.composition.cursor -= 1,
                "token" => engine.context_token = None,
                "sealed" => engine.context_handoff_sealed = true,
                "atomic" => engine.atomic.active = true,
                "sensitive" => engine.set_content_type_state(8, 0),
                _ => {}
            }
            let edit = (invalid != "wrong_user_input").then_some(&edit);
            let outgoing = if invalid == "wrong_user_input" {
                "abd "
            } else {
                "abcdef "
            };
            assert!(
                engine
                    .owned_preedit_space_commit_expectation(outgoing, edit)
                    .is_none(),
                "{invalid}"
            );
        }
    }));
}

#[test]
fn td159_stock_user_space_expectation_requires_current_single_owned_word() {
    zbus::block_on(bounded(async {
        for invalid in [
            "exact_refresh",
            "mode",
            "preedit",
            "tail",
            "cursor",
            "token",
            "sealed",
            "atomic",
            "sensitive",
            "surrounding",
            "preedit_capability",
            "wrong_word",
            "extra_word",
            "empty_word",
        ] {
            let (_harness, mut engine) = compose_owned_word_with_initial_snapshot(None, 41).await;
            let mut outgoing = "abc ";
            match invalid {
                "exact_refresh" => engine.client_context.exact_surrounding_refresh_available = true,
                "mode" => {
                    engine.composition.word_input_mode = Some(WordInputMode::TerminalPassthrough)
                }
                "preedit" => engine.composition.legacy_word_preedit_active = false,
                "tail" => engine.committed_tail.buffer = "old abc".into(),
                "cursor" => engine.composition.cursor -= 1,
                "token" => engine.context_token = None,
                "sealed" => engine.context_handoff_sealed = true,
                "atomic" => engine.atomic.active = true,
                "sensitive" => engine.set_content_type_state(8, 0),
                "surrounding" => engine.client_context.surrounding_text_supported = false,
                "preedit_capability" => engine.client_context.preedit_text_supported = false,
                "wrong_word" => outgoing = "abd ",
                "extra_word" => {
                    engine.composition.buffer = "abc def".into();
                    engine.composition.cursor = 7;
                    engine.committed_tail.buffer = "abc def".into();
                    outgoing = "abc def ";
                }
                "empty_word" => {
                    engine.composition.buffer.clear();
                    engine.composition.cursor = 0;
                    engine.committed_tail.buffer.clear();
                    outgoing = " ";
                }
                _ => unreachable!(),
            }
            assert!(
                engine
                    .owned_preedit_space_commit_expectation(outgoing, None)
                    .is_none(),
                "{invalid}"
            );
        }
    }));
}

#[test]
fn td159_completion_expectation_never_downgrades_known_or_partial_start() {
    zbus::block_on(bounded(async {
        for invalid in ["snapshot_missing", "witness_missing", "revision_stale"] {
            let (_harness, mut engine) = compose_owned_word("old ").await;
            match invalid {
                "snapshot_missing" => engine.client_context.surrounding_text_snapshot = None,
                "witness_missing" => engine.composition.legacy_preedit_start_boundary = None,
                "revision_stale" => engine.client_context.surrounding_observation_revision += 1,
                _ => unreachable!(),
            }
            let action =
                lay::text_edit::plan_ime_candidate_accept_edit("fixture", 900, "abc", "abcdef ");
            let edit = lay::text_edit::authorize_backend_edit(
                lay::text_edit::TextEditBackend::Ime,
                action,
            )
            .into_authorized()
            .unwrap();
            assert!(
                engine
                    .owned_preedit_space_commit_expectation("abcdef ", Some(&edit))
                    .is_none(),
                "{invalid}"
            );
        }
    }));
}

#[test]
fn td159_accepted_completion_space_receipt_retains_exact_tail_after_nbsp() {
    zbus::block_on(bounded(async {
        for (prefix, space) in [
            ("", ' '),
            ("", '\u{a0}'),
            ("πrefix ", '\u{a0}'),
            ("πrefix\u{a0}", '\u{a0}'),
        ] {
            let (mut harness, mut engine) = compose_and_accept_completion(prefix).await;
            let epoch = engine.committed_tail.epoch;
            assert!(engine
                .committed_tail
                .pending_visible_postcondition
                .as_ref()
                .unwrap()
                .feedback
                .is_none());
            let text = format!("{prefix}abcdef{space}");
            let cursor = text.chars().count() as u32;
            cycle09_surrounding_receipt(&mut harness, &mut engine, &text, cursor, cursor).await;
            assert_eq!(engine.committed_tail.buffer, format!("abcdef{space}"));
            assert_eq!(engine.committed_tail.epoch, epoch);
            assert!(engine.current_external_snapshot_agrees_with_owned_tail());
            assert!(engine
                .committed_tail
                .pending_visible_postcondition
                .is_none());
            assert!(engine.committed_tail.recent_replace.is_none());
        }
    }));
}

#[test]
fn td159_accepted_completion_space_receipt_rejects_changed_client_state() {
    zbus::block_on(bounded(async {
        for (text, selection, moved_cursor) in [
            ("changed abcdef\u{a0}", false, false),
            ("πrefix\u{a0}abcdeg\u{a0}", false, false),
            ("πrefix\u{a0}abcdef\u{a0}", true, false),
            ("πrefix\u{a0}abcdef\u{a0}", false, true),
            ("πrefix abcdef\u{a0}", false, false),
        ] {
            let (mut harness, mut engine) = compose_and_accept_completion("πrefix\u{a0}").await;
            let epoch = engine.committed_tail.epoch;
            let cursor = text.chars().count() as u32 - u32::from(moved_cursor);
            let anchor = cursor - u32::from(selection);
            cycle09_surrounding_receipt(&mut harness, &mut engine, text, cursor, anchor).await;
            assert_eq!(
                engine.committed_tail.buffer, "abcdef ",
                "{text:?}/{cursor}/{anchor}"
            );
            assert_eq!(engine.committed_tail.epoch, epoch);
            assert!(!engine.current_external_snapshot_agrees_with_owned_tail());
            assert!(engine.committed_tail.recent_replace.is_none());
        }
    }));
}

#[test]
fn td159_accepted_completion_space_receipt_rejects_expired_epoch_and_owner() {
    zbus::block_on(bounded(async {
        for invalid in ["expired", "epoch", "owner"] {
            let (mut harness, mut engine) = compose_and_accept_completion("").await;
            match invalid {
                "expired" => {
                    engine
                        .committed_tail
                        .pending_visible_postcondition
                        .as_mut()
                        .unwrap()
                        .dispatched_at -= std::time::Duration::from_millis(1501)
                }
                "epoch" => engine.committed_tail.epoch += 1,
                "owner" => engine.shared.lock().unwrap().active_path = Some("/foreign".into()),
                _ => unreachable!(),
            }
            cycle09_surrounding_receipt(&mut harness, &mut engine, "abcdef\u{a0}", 7, 7).await;
            assert_eq!(engine.committed_tail.buffer, "abcdef ", "{invalid}");
            assert!(engine.committed_tail.recent_replace.is_none());
        }
    }));
}

#[test]
fn td159_accepted_completion_space_receipt_requires_exact_authorized_edit() {
    zbus::block_on(bounded(async {
        let (_harness, engine) = compose_owned_word("").await;
        for (from, authorized_to, outgoing) in [
            ("abcd", "abcdef ", "abcdef "),
            ("abc", "abcdef ", "abcdeg "),
            ("abc", "abcdef", "abcdef"),
            ("abc", "abcdef  ", "abcdef  "),
        ] {
            let action =
                lay::text_edit::plan_ime_candidate_accept_edit("fixture", 900, from, authorized_to);
            let edit = lay::text_edit::authorize_backend_edit(
                lay::text_edit::TextEditBackend::Ime,
                action,
            )
            .into_authorized()
            .unwrap();
            assert!(
                engine
                    .owned_preedit_space_commit_expectation(outgoing, Some(&edit))
                    .is_none(),
                "{from:?}/{authorized_to:?}/{outgoing:?}"
            );
        }
    }));
}

#[test]
fn td159_accepted_completion_nbsp_keeps_real_v4_delegated_tail() {
    zbus::block_on(bounded(async {
        let (mut harness, mut engine) = compose_and_accept_completion("").await;
        cycle09_surrounding_receipt(&mut harness, &mut engine, "abcdef\u{a0}", 7, 7).await;
        engine = bridge_delegate_exact_without_gui_edit(&mut harness, engine).await;
        let (_, visible) = private_visible_tail_v4(&mut harness, engine).await;
        let (state, tail, _, _, path, field_receipt, _) = visible.unwrap();
        assert_eq!(tail, "abcdef\u{a0}");
        assert_eq!(state, "passive:committed-tail");
        assert!(!path.is_empty());
        assert!(!field_receipt.is_empty());
    }));
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
