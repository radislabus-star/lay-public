//! Ordinary Space must acknowledge only the boundary emitted by this owner.
use super::*;

#[path = "../../../../../../tests/common/composition_authorization.rs"]
mod composition_authorization;

async fn compose_owned_word(prefix: &str) -> (Harness, LayIbusEngine) {
    compose_owned_word_with_initial_snapshot(Some(prefix), 1_073_741_865).await
}

async fn compose_owned_erasure(prefix: &str) -> (Harness, LayIbusEngine) {
    let (mut harness, mut engine) = compose_owned_word(prefix).await;
    engine.config.layout_backend = "kde".into();
    assert!(legacy_key(&mut harness, &mut engine, 159_900, 'd' as u32, 32, 0).await);
    let (_, commits) = drain_output_to_text_proof(&mut harness).await;
    assert!(commits.is_empty());
    assert!(
        legacy_key(
            &mut harness,
            &mut engine,
            159_901,
            'd' as u32,
            32,
            RELEASE_MASK
        )
        .await
    );
    cycle09_assert_no_local_text_effect(&mut harness).await;
    assert!(engine.prepare_owned_preedit_local_backspace().is_some());
    assert!(legacy_key(&mut harness, &mut engine, 159_902, KEY_BACKSPACE, 14, 0).await);
    let (members, commits) = drain_output_to_text_proof(&mut harness).await;
    assert!(commits.is_empty());
    assert!(members.iter().any(|m| m == "UpdatePreeditText"));
    assert!(!members.iter().any(|m| m == "DeleteSurroundingText"));
    assert_eq!(engine.composition.buffer, "abc");
    (harness, engine)
}

async fn compose_closed_word_erasure(separator: &str, backend: &str) -> (Harness, LayIbusEngine) {
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = observed_closed_word_for_owned_preedit(&mut harness, separator, true).await;
    engine.config.layout_backend = backend.into();
    assert!(legacy_key(&mut harness, &mut engine, 160_100, 'z' as u32, 44, 0).await);
    assert_eq!(engine.composition.buffer, "z");
    assert!(
        legacy_key(
            &mut harness,
            &mut engine,
            160_101,
            'z' as u32,
            44,
            RELEASE_MASK
        )
        .await
    );
    assert!(legacy_key(&mut harness, &mut engine, 160_102, KEY_BACKSPACE, 14, 0).await);
    assert!(engine.composition.buffer.is_empty());
    assert!(!engine.composition.legacy_word_preedit_active);
    assert_eq!(engine.committed_tail.buffer, format!("abc{separator}"));
    assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
    assert!(
        legacy_key(
            &mut harness,
            &mut engine,
            160_103,
            KEY_BACKSPACE,
            14,
            RELEASE_MASK
        )
        .await
    );
    cycle09_assert_no_local_text_effect(&mut harness).await;
    (harness, engine)
}

#[test]
fn kde_firefox_closed_word_erasure_echo_requires_final_actual_receipt() {
    for separator in [" ", "\u{a0}"] {
        zbus::block_on(bounded(async {
            let (mut harness, mut engine) = compose_closed_word_erasure(separator, "kde").await;
            let closed = format!("abc{separator}");
            let cursor = closed.chars().count() as u32;
            let epoch = engine.committed_tail.epoch;
            let revision = engine.client_context.surrounding_observation_revision;
            engine.observe_external_surrounding_text_for_owned_erasure(
                Some(SurroundingTextSnapshot::new(
                    format!("{closed}z"),
                    cursor,
                    cursor,
                )),
                true,
            );
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert!(!engine.context_reset_rereceipt_computation_allowed());
            assert!(!engine.context_reset_rereceipt_manual_refresh_allowed());
            assert!(
                !engine.context_word_is_known(),
                "the stale preedit echo grants no word authority"
            );
            engine.observe_external_surrounding_text_for_owned_erasure(
                Some(SurroundingTextSnapshot::new(closed.clone(), cursor, cursor)),
                true,
            );
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "the next whole actual field must reconfirm the successfully cancelled local preedit");
            assert_eq!(engine.committed_tail.epoch, epoch);
            assert_eq!(engine.committed_tail.buffer, closed);
            assert_eq!(
                engine.client_context.surrounding_observation_revision,
                revision + 2
            );
            cycle09_assert_no_local_text_effect(&mut harness).await;
            let path = engine.path.clone();
            let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
            assert_eq!(
                disposition,
                Ok(lay::manual_toggle::ImeManualToggleOutcome::DelegateExactImeTail.as_v3())
            );
            let (_, tail) = cycle09_visible_tail(&mut harness, engine).await;
            assert!(cycle09_tail_is_authoritative(&tail, &path, false, &closed));
        }));
    }
}

#[test]
fn kde_firefox_closed_word_erasure_echo_refuses_changed_or_interrupted_receipts() {
    for separator in [" ", "\u{a0}"] {
        for invalid in [
            "wrong_erased",
            "selected_echo",
            "moved_caret_echo",
            "foreign_first",
            "gnome",
            "selected_final",
            "changed_prefix",
            "duplicate_echo",
            "foreign_final",
            "epoch",
            "focus",
            "lease",
            "layout",
            "word_revoked",
            "inactive_path",
            "reset",
            "intervening",
            "missing_proof",
        ] {
            zbus::block_on(bounded(async {
                let backend = if invalid == "gnome" { "gnome" } else { "kde" };
                let (mut harness, mut engine) =
                    compose_closed_word_erasure(separator, backend).await;
                let closed = format!("abc{separator}");
                let cursor = closed.chars().count() as u32;
                if invalid == "missing_proof" {
                    engine.context_reset_rereceipt = None;
                }
                let echo = SurroundingTextSnapshot::new(
                    format!(
                        "{closed}{}",
                        if invalid == "wrong_erased" { 'x' } else { 'z' }
                    ),
                    cursor + u32::from(invalid == "moved_caret_echo"),
                    cursor + u32::from(matches!(invalid, "selected_echo" | "moved_caret_echo")),
                );
                engine.observe_external_surrounding_text_for_owned_erasure(
                    Some(echo.clone()),
                    invalid != "foreign_first",
                );
                assert!(
                    !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                    "{invalid}"
                );
                match invalid {
                    "duplicate_echo" => {
                        engine.observe_external_surrounding_text_for_owned_erasure(Some(echo), true)
                    }
                    "epoch" => engine.committed_tail.epoch += 1,
                    "focus" => engine.client_context.focus_serial += 1,
                    "lease" => engine.client_context.runtime_owner_lease_identity += 1,
                    "layout" => engine.layout_gesture.layout_generation += 1,
                    "word_revoked" => engine.revoke_context_word(),
                    "inactive_path" => {
                        engine.shared.lock().unwrap().active_path = Some("/other".into())
                    }
                    "reset" => {
                        actual_reset(&mut harness, &mut engine, 160_104, false).await;
                    }
                    "intervening" => engine.observe_external_surrounding_text_for_owned_erasure(
                        Some(SurroundingTextSnapshot::new("other".into(), 5, 5)),
                        true,
                    ),
                    _ => (),
                }
                let final_text = if invalid == "changed_prefix" {
                    format!("other {closed}")
                } else {
                    closed.clone()
                };
                let final_cursor = final_text.chars().count() as u32;
                engine.observe_external_surrounding_text_for_owned_erasure(
                    Some(SurroundingTextSnapshot::new(
                        final_text,
                        final_cursor,
                        final_cursor - u32::from(invalid == "selected_final"),
                    )),
                    invalid != "foreign_final",
                );
                assert!(
                    !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                    "{invalid}"
                );
                // Even another exact suffix cannot revive discarded provenance.
                engine.observe_external_surrounding_text_for_owned_erasure(
                    Some(SurroundingTextSnapshot::new(closed, cursor, cursor)),
                    true,
                );
                assert!(
                    !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                    "{invalid}"
                );
                assert!(!engine.context_word_is_known(), "{invalid}");
                cycle09_assert_no_local_text_effect(&mut harness).await;
            }));
        }
    }
}

#[test]
fn kde_firefox_closed_word_erasure_echo_preserves_direct_exact_receipts() {
    for backend in ["kde", "gnome"] {
        for separator in [" ", "\u{a0}"] {
            zbus::block_on(bounded(async {
                let (mut harness, mut engine) =
                    compose_closed_word_erasure(separator, backend).await;
                let closed = format!("abc{separator}");
                let cursor = closed.chars().count() as u32;
                engine.observe_external_surrounding_text_for_owned_erasure(
                    Some(SurroundingTextSnapshot::new(closed.clone(), cursor, cursor)),
                    true,
                );
                assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                cycle09_assert_no_local_text_effect(&mut harness).await;
                let path = engine.path.clone();
                let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
                assert_eq!(
                    disposition,
                    Ok(lay::manual_toggle::ImeManualToggleOutcome::DelegateExactImeTail.as_v3())
                );
                let (_, tail) = cycle09_visible_tail(&mut harness, engine).await;
                assert!(cycle09_tail_is_authoritative(&tail, &path, false, &closed));
            }));
        }
    }
}

async fn compose_owned_append_before_caret_echo(backend: &str) -> (Harness, LayIbusEngine, String) {
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = known_engine(&mut harness).await;
    engine.config.layout_backend = backend.into();
    engine.config.auto_replace = false;
    engine.config.typing_assist = false;
    engine.config.nanda_precognition = false;
    engine.client_context.content_purpose = 0;
    engine.client_context.cursor_cell_width = 0;
    engine.set_client_capabilities(0x4000_0029);
    assert_eq!(engine.committed_tail.buffer, " ");
    for (index, (ch, code)) in [('a', 30), ('b', 48), ('c', 46)].into_iter().enumerate() {
        strict_prefix_v10_literal_key(
            &mut harness,
            &mut engine,
            161_000 + index as u32 * 2,
            ch,
            code,
        )
        .await;
    }
    actual_reset(&mut harness, &mut engine, 161_010, false).await;
    exact_surrounding_receipt(&mut harness, &mut engine, " abc").await;
    assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
    strict_prefix_v10_literal_key(&mut harness, &mut engine, 161_012, 'd', 32).await;
    let pending = engine.context_reset_rereceipt.as_ref().unwrap();
    assert!(pending.confirmed && pending.awaiting_owned_append_receipt);
    assert_eq!(engine.committed_tail.buffer, " abcd");
    (harness, engine, " abcd".into())
}

#[test]
fn kde_firefox_queued_owned_append_requires_final_whole_actual_receipt() {
    for intermediate_caret in [false, true] {
        zbus::block_on(bounded(async {
            let (mut harness, mut engine, earlier) =
                compose_owned_append_before_caret_echo("kde").await;
            strict_prefix_v10_literal_key(&mut harness, &mut engine, 161_014, 'e', 18).await;
            let full = format!("{earlier}e");
            let epoch = engine.committed_tail.epoch;
            let old_cursor = earlier.chars().count() as u32 - 1;
            let echo = SurroundingTextSnapshot::new(earlier.clone(), old_cursor, old_cursor);
            assert!(
                engine.kde_firefox_owned_append_caret_observation_pending(&echo),
                "a delayed caret from the earlier owned append must survive a second handled append"
            );
            engine.observe_external_surrounding_text_for_owned_erasure(Some(echo), true);
            assert!(engine.context_reset_rereceipt.is_some());
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert!(!engine.context_allows_manual_toggle());
            assert!(!engine.context_observed_suffix_exact_manual_handoff_allowed());
            assert!(!engine.context_owns_active_preedit_manual_toggle());
            cycle09_assert_no_local_text_effect(&mut harness).await;
            if intermediate_caret {
                let cursor = earlier.chars().count() as u32;
                let actual = SurroundingTextSnapshot::new(earlier, cursor, cursor);
                assert!(engine.kde_firefox_owned_append_caret_observation_pending(&actual));
                engine.observe_external_surrounding_text_for_owned_erasure(Some(actual), true);
                assert!(engine.context_reset_rereceipt.is_some());
                assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
                assert!(!engine.context_allows_manual_toggle());
                cycle09_assert_no_local_text_effect(&mut harness).await;
            }
            let cursor = full.chars().count() as u32;
            engine.observe_external_surrounding_text_for_owned_erasure(
                Some(SurroundingTextSnapshot::new(full.clone(), cursor, cursor)),
                true,
            );
            assert!(engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert_eq!(engine.committed_tail.epoch, epoch);
            assert_eq!(engine.committed_tail.buffer, full);
            cycle09_assert_no_local_text_effect(&mut harness).await;
            let path = engine.path.clone();
            let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
            assert_eq!(
                disposition,
                Ok(lay::manual_toggle::ImeManualToggleOutcome::DelegateExactImeTail.as_v3())
            );
            let (_, tail) = cycle09_visible_tail(&mut harness, engine).await;
            assert!(cycle09_tail_is_authoritative(&tail, &path, false, &full));
        }));
    }
}

#[test]
fn kde_firefox_queued_owned_append_rejects_contradictory_receipts() {
    for invalid in [
        "gnome",
        "missing_receipt",
        "unconfirmed",
        "inactive_path",
        "foreign_first",
        "selected_echo",
        "wrong_echo",
        "duplicate_echo",
        "regressing_prefix",
        "changed_prefix",
        "selected_prefix",
        "duplicate_prefix",
        "changed_final_after_prefix",
        "changed_final",
        "selected_final",
        "foreign_final",
        "focus_loss",
    ] {
        zbus::block_on(bounded(async {
            let (mut harness, mut engine, earlier) =
                compose_owned_append_before_caret_echo(if invalid == "gnome" {
                    "gnome"
                } else {
                    "kde"
                })
                .await;
            strict_prefix_v10_literal_key(&mut harness, &mut engine, 161_014, 'e', 18).await;
            let full = format!("{earlier}e");
            if invalid == "missing_receipt" {
                engine.context_reset_rereceipt = None;
            }
            if invalid == "unconfirmed" {
                engine.context_reset_rereceipt.as_mut().unwrap().confirmed = false;
            }
            if invalid == "inactive_path" {
                engine.shared.lock().unwrap().active_path = Some("/other".into());
            }
            let old = earlier.chars().count() as u32 - 1;
            let echo = SurroundingTextSnapshot::new(
                if invalid == "wrong_echo" {
                    " abcx".into()
                } else {
                    earlier.clone()
                },
                old,
                old + u32::from(invalid == "selected_echo"),
            );
            engine.observe_external_surrounding_text_for_owned_erasure(
                Some(echo.clone()),
                invalid != "foreign_first",
            );
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{invalid}"
            );
            if invalid == "duplicate_echo" {
                engine.observe_external_surrounding_text_for_owned_erasure(Some(echo), true);
            }
            if matches!(
                invalid,
                "regressing_prefix"
                    | "changed_prefix"
                    | "selected_prefix"
                    | "duplicate_prefix"
                    | "changed_final_after_prefix"
            ) {
                let text = match invalid {
                    "regressing_prefix" => " abc".into(),
                    "changed_prefix" => format!("other{earlier}"),
                    _ => earlier.clone(),
                };
                let end = text.chars().count() as u32;
                let actual = SurroundingTextSnapshot::new(
                    text,
                    end,
                    end - u32::from(invalid == "selected_prefix"),
                );
                engine.observe_external_surrounding_text_for_owned_erasure(
                    Some(actual.clone()),
                    true,
                );
                if invalid == "duplicate_prefix" {
                    engine.observe_external_surrounding_text_for_owned_erasure(Some(actual), true);
                }
            }
            if invalid == "focus_loss" {
                actual_focus_out(&mut harness, &mut engine, 161_016).await;
            }
            assert!(!engine.context_allows_manual_toggle(), "{invalid}");
            assert!(
                !engine.context_observed_suffix_exact_manual_handoff_allowed(),
                "{invalid}"
            );
            assert!(
                !engine.context_owns_active_preedit_manual_toggle(),
                "{invalid}"
            );
            let final_text = if matches!(invalid, "changed_final" | "changed_final_after_prefix") {
                format!("other{full}")
            } else {
                full.clone()
            };
            let end = final_text.chars().count() as u32;
            engine.observe_external_surrounding_text_for_owned_erasure(
                Some(SurroundingTextSnapshot::new(
                    final_text,
                    end,
                    end - u32::from(invalid == "selected_final"),
                )),
                invalid != "foreign_final",
            );
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{invalid}"
            );
            let end = full.chars().count() as u32;
            engine.observe_external_surrounding_text_for_owned_erasure(
                Some(SurroundingTextSnapshot::new(full, end, end)),
                true,
            );
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{invalid}"
            );
            cycle09_assert_no_local_text_effect(&mut harness).await;
        }));
    }
}

#[test]
fn kde_firefox_queued_owned_append_preserves_unicode_whole_field() {
    for external in ["", "π ", "до\u{a0}"] {
        for token in ["abcde", "абвгд"] {
            let mut chars = token.chars();
            let prefix: String = chars.by_ref().take(3).collect();
            let first = chars.next().unwrap();
            let before = format!("{external}{prefix}");
            let earlier = format!("{before}{first}");
            let old = before.chars().count() as u32;
            let actual_before = SurroundingTextSnapshot::new(before, old, old);
            let echo = SurroundingTextSnapshot::new(earlier.clone(), old, old);
            assert!(LayIbusEngine::kde_firefox_owned_append_caret_echo_matches(
                &actual_before,
                &echo,
                token
            ));
            for text in [earlier, format!("{external}{token}")] {
                let end = text.chars().count() as u32;
                let actual = SurroundingTextSnapshot::new(text, end, end);
                assert!(LayIbusEngine::kde_firefox_owned_append_caret_echo_settles(
                    &echo, &actual, token
                ));
                let mut changed = actual.clone();
                changed.text.insert_str(0, "other ");
                changed.cursor_pos += 6;
                changed.anchor_pos += 6;
                assert!(!LayIbusEngine::kde_firefox_owned_append_caret_echo_settles(
                    &echo, &changed, token
                ));
                assert!(!LayIbusEngine::kde_firefox_owned_append_caret_echo_matches(
                    &actual_before,
                    &actual,
                    token
                ));
            }
        }
    }
}

#[test]
fn kde_firefox_owned_append_caret_echo_requires_final_actual_receipt() {
    for reset in [true, false] {
        zbus::block_on(bounded(async {
            let (mut harness, mut engine, full) =
                compose_owned_append_before_caret_echo("kde").await;
            let cursor = full.chars().count() as u32;
            let epoch = engine.committed_tail.epoch;
            let source_state = format!(
                "mode={:?}, active={:?}, exact={}, before={:?}, scope={:?}, token_live={}",
                engine.composition.word_input_mode,
                engine.shared.lock().unwrap().active_path,
                engine.client_context.exact_surrounding_refresh_available,
                engine.client_context.surrounding_text_snapshot,
                engine.context_word_scope.as_ref().map(|scope| (
                    scope.lineage().completeness,
                    scope.lineage().observed_suffix_chars
                )),
                engine.live_context_token().is_some()
            );
            let echo = SurroundingTextSnapshot::new(full.clone(), cursor - 1, cursor - 1);
            assert!(engine.kde_firefox_owned_append_caret_observation_pending(&echo),
                "the actual SSI selector must recognize the proved pre-effect snapshot: {source_state}");
            engine.observe_external_surrounding_text_for_owned_erasure(Some(echo), true);
            assert!(
                engine.context_reset_rereceipt.is_some(),
                "an owned append caret echo must retain only inert lineage: {source_state}"
            );
            assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            assert!(!engine.context_allows_manual_toggle());
            assert!(!engine.context_observed_suffix_exact_manual_handoff_allowed());
            assert!(!engine.context_owns_active_preedit_manual_toggle());
            cycle09_assert_no_local_text_effect(&mut harness).await;
            if reset {
                actual_reset(&mut harness, &mut engine, 161_014, false).await;
                assert!(!engine.context_reset_rereceipt_exact_manual_handoff_allowed());
            }
            engine.observe_external_surrounding_text_for_owned_erasure(
                Some(SurroundingTextSnapshot::new(full.clone(), cursor, cursor)),
                true,
            );
            assert!(
                engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "the full actual caret must settle the same owned append"
            );
            assert_eq!(engine.committed_tail.epoch, epoch);
            assert_eq!(engine.committed_tail.buffer, full);
            cycle09_assert_no_local_text_effect(&mut harness).await;
            let path = engine.path.clone();
            let (engine, disposition) = cycle09_manual_toggle(&mut harness, engine).await;
            assert_eq!(
                disposition,
                Ok(lay::manual_toggle::ImeManualToggleOutcome::DelegateExactImeTail.as_v3())
            );
            let (_, tail) = cycle09_visible_tail(&mut harness, engine).await;
            assert!(cycle09_tail_is_authoritative(&tail, &path, false, &full));
        }));
    }
}

#[test]
fn kde_firefox_owned_append_caret_echo_refuses_unowned_or_changed_observations() {
    for invalid in [
        "gnome",
        "foreign_first",
        "unconfirmed",
        "not_owned_append",
        "selection",
        "interior",
        "wrong_text",
        "inactive_path",
        "duplicate",
        "changed_final_prefix",
        "selected_final",
        "foreign_final",
        "focus_loss",
        "missing_receipt",
    ] {
        zbus::block_on(bounded(async {
            let (mut harness, mut engine, full) =
                compose_owned_append_before_caret_echo(if invalid == "gnome" {
                    "gnome"
                } else {
                    "kde"
                })
                .await;
            let cursor = full.chars().count() as u32;
            if invalid == "unconfirmed" {
                engine.context_reset_rereceipt.as_mut().unwrap().confirmed = false;
            }
            if invalid == "not_owned_append" {
                engine
                    .context_reset_rereceipt
                    .as_mut()
                    .unwrap()
                    .awaiting_owned_append_receipt = false;
            }
            if invalid == "inactive_path" {
                engine.shared.lock().unwrap().active_path = Some("/other".into());
            }
            if invalid == "missing_receipt" {
                engine.context_reset_rereceipt = None;
            }
            let echo = SurroundingTextSnapshot::new(
                if invalid == "wrong_text" {
                    " abcx".into()
                } else {
                    full.clone()
                },
                cursor - if invalid == "interior" { 2 } else { 1 },
                cursor
                    - if invalid == "selection" {
                        0
                    } else if invalid == "interior" {
                        2
                    } else {
                        1
                    },
            );
            engine.observe_external_surrounding_text_for_owned_erasure(
                Some(echo.clone()),
                invalid != "foreign_first",
            );
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{invalid}"
            );
            assert!(
                !engine.context_allows_manual_toggle()
                    && !engine.context_observed_suffix_exact_manual_handoff_allowed()
                    && !engine.context_owns_active_preedit_manual_toggle(),
                "{invalid}"
            );
            if invalid == "duplicate" {
                engine.observe_external_surrounding_text_for_owned_erasure(Some(echo), true);
            }
            if invalid == "focus_loss" {
                actual_focus_out(&mut harness, &mut engine, 161_015).await;
            }
            let final_text = if invalid == "changed_final_prefix" {
                format!("other{full}")
            } else {
                full.clone()
            };
            let end = final_text.chars().count() as u32;
            engine.observe_external_surrounding_text_for_owned_erasure(
                Some(SurroundingTextSnapshot::new(
                    final_text,
                    end,
                    end - u32::from(invalid == "selected_final"),
                )),
                invalid != "foreign_final",
            );
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{invalid}"
            );
            engine.observe_external_surrounding_text_for_owned_erasure(
                Some(SurroundingTextSnapshot::new(full, cursor, cursor)),
                true,
            );
            assert!(
                !engine.context_reset_rereceipt_exact_manual_handoff_allowed(),
                "{invalid}"
            );
            cycle09_assert_no_local_text_effect(&mut harness).await;
        }));
    }
}

#[test]
fn kde_firefox_owned_append_caret_echo_preserves_unicode_external_context() {
    for external in ["", "π ", "до\u{a0}"] {
        for token in ["abcd", "абвг"] {
            let full = format!("{external}{token}");
            let (last, _) = full.char_indices().next_back().unwrap();
            let before = &full[..last];
            let old = before.chars().count() as u32;
            let previous = SurroundingTextSnapshot::new(before.into(), old, old);
            let echo = SurroundingTextSnapshot::new(full.clone(), old, old);
            let exact = SurroundingTextSnapshot::new(full, old + 1, old + 1);
            assert!(LayIbusEngine::kde_firefox_owned_append_caret_echo_matches(
                &previous, &echo, token
            ));
            assert!(LayIbusEngine::kde_firefox_owned_append_caret_echo_settles(
                &echo, &exact, token
            ));
            assert!(!LayIbusEngine::kde_firefox_owned_append_caret_echo_matches(
                &previous, &exact, token
            ));
            let mut foreign = exact.clone();
            foreign.text.insert_str(0, "other ");
            foreign.cursor_pos += 6;
            foreign.anchor_pos += 6;
            assert!(!LayIbusEngine::kde_firefox_owned_append_caret_echo_settles(
                &echo, &foreign, token
            ));
        }
    }
}

#[test]
fn kde_firefox_owned_erasure_echo_requires_final_actual_receipt() {
    zbus::block_on(bounded(async {
        let prefix = "π ";
        let (mut harness, mut engine) = compose_owned_erasure(prefix).await;
        let epoch = engine.committed_tail.epoch;
        let known = engine.context_word_is_known();
        let start = prefix.chars().count() as u32;
        assert!(!engine.context_owns_active_preedit_manual_toggle());
        engine.observe_external_surrounding_text_for_owned_erasure(
            Some(SurroundingTextSnapshot::new(
                format!("{prefix}d"),
                start,
                start,
            )),
            true,
        );
        assert!(
            !engine.context_owns_active_preedit_manual_toggle(),
            "the output echo is not authority"
        );
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                159_903,
                KEY_BACKSPACE,
                14,
                RELEASE_MASK
            )
            .await
        );
        cycle09_assert_no_local_text_effect(&mut harness).await;
        assert!(
            !engine.context_owns_active_preedit_manual_toggle(),
            "a key release is not a client receipt"
        );
        engine.observe_external_surrounding_text_for_owned_erasure(
            Some(SurroundingTextSnapshot::new(prefix.into(), start, start)),
            true,
        );
        assert!(
            engine.context_owns_active_preedit_manual_toggle(),
            "same-owner erasure must settle only at the final exact client receipt"
        );
        assert_eq!(
            engine
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .unwrap()
                .text,
            prefix
        );
        assert_eq!(engine.committed_tail.epoch, epoch);
        assert_eq!(engine.committed_tail.buffer, "abc");
        assert_eq!(engine.context_word_is_known(), known);
        assert!(engine.client_context.native_document_binding.is_none());
        assert!(engine.committed_tail.pending_completion_learning.is_none());
        assert!(engine.committed_tail.recent_replace.is_none());
    }));
}

#[test]
fn kde_firefox_owned_erasure_echo_refuses_stale_or_foreign_observations() {
    zbus::block_on(bounded(async {
        for invalid in [
            "owner",
            "focus",
            "lease",
            "layout",
            "epoch",
            "revision",
            "prefix",
            "suffix",
            "character",
            "cursor",
            "selection",
            "missing",
            "sealed",
            "atomic",
            "backend",
            "client",
        ] {
            let (mut harness, mut engine) = compose_owned_erasure("π ").await;
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    159_903,
                    KEY_BACKSPACE,
                    14,
                    RELEASE_MASK
                )
                .await
            );
            cycle09_assert_no_local_text_effect(&mut harness).await;
            let mut echo = Some(SurroundingTextSnapshot::new("π d".into(), 2, 2));
            match invalid {
                "owner" => engine.shared.lock().unwrap().active_path = Some("/foreign".into()),
                "focus" => engine.client_context.focus_serial += 1,
                "lease" => engine.client_context.runtime_owner_lease_identity += 1,
                "layout" => engine.layout_gesture.layout_generation += 1,
                "epoch" => engine.committed_tail.epoch += 1,
                "revision" => engine.client_context.surrounding_observation_revision += 1,
                "prefix" => echo.as_mut().unwrap().text = "χ d".into(),
                "suffix" => echo.as_mut().unwrap().text.push('x'),
                "character" => echo.as_mut().unwrap().text = "π e".into(),
                "cursor" => {
                    echo.as_mut().unwrap().cursor_pos = 3;
                    echo.as_mut().unwrap().anchor_pos = 3;
                }
                "selection" => echo.as_mut().unwrap().anchor_pos = 3,
                "missing" => echo = None,
                "sealed" => engine.context_handoff_sealed = true,
                "atomic" => engine.atomic.active = true,
                "backend" => engine.config.layout_backend = "gnome".into(),
                _ => {}
            }
            engine.observe_external_surrounding_text_for_owned_erasure(echo, invalid != "client");
            engine.observe_external_surrounding_text_for_owned_erasure(
                Some(SurroundingTextSnapshot::new("π ".into(), 2, 2)),
                true,
            );
            assert!(
                !engine.context_owns_active_preedit_manual_toggle(),
                "{invalid}"
            );
        }
    }));
}

#[test]
fn kde_firefox_owned_erasure_echo_does_not_survive_an_intervening_receipt() {
    zbus::block_on(bounded(async {
        let (mut harness, mut engine) = compose_owned_erasure("").await;
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                159_903,
                KEY_BACKSPACE,
                14,
                RELEASE_MASK
            )
            .await
        );
        cycle09_assert_no_local_text_effect(&mut harness).await;
        for text in ["d", "foreign", ""] {
            engine.observe_external_surrounding_text_for_owned_erasure(
                Some(SurroundingTextSnapshot::new(text.into(), 0, 0)),
                true,
            );
        }
        assert!(!engine.context_owns_active_preedit_manual_toggle());
    }));
}

#[test]
fn kde_firefox_owned_erasure_echo_requires_the_successful_local_effect() {
    zbus::block_on(bounded(async {
        let (_, mut engine) = compose_owned_word("").await;
        engine.config.layout_backend = "kde".into();
        for text in ["d", ""] {
            engine.observe_external_surrounding_text_for_owned_erasure(
                Some(SurroundingTextSnapshot::new(text.into(), 0, 0)),
                true,
            );
        }
        assert!(!engine.context_owns_active_preedit_manual_toggle());
    }));
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
            let edit = composition_authorization::accepted_edit(from, to);
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
            let edit = composition_authorization::accepted_edit("abc", "abcdef ");
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
            let edit = composition_authorization::accepted_edit(from, authorized_to);
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

#[test]
fn firefox_preedit_space_reflow_retains_the_same_owner_and_exact_field() {
    zbus::block_on(bounded(async {
        for retain_prefix in [false, true] {
            let prefix = "πrefix\u{a0}";
            let (mut harness, mut engine) = compose_and_accept_completion(prefix).await;
            let epoch = engine.committed_tail.epoch;
            let known = engine.context_word_is_known();
            let actual = "πrefix abcdef\u{a0}";
            let cursor = actual.chars().count() as u32;
            cycle09_surrounding_receipt(&mut harness, &mut engine, actual, cursor, cursor).await;
            let mut pending = engine
                .committed_tail
                .pending_visible_postcondition
                .take()
                .unwrap();
            assert!(pending.expected_external_snapshot.is_some());
            if retain_prefix {
                // Model this same pending result with locally retained earlier
                // text. The native fixture proves acquisition/key history;
                // this contract test checks the owner/result transformation.
                engine.committed_tail.buffer.insert_str(0, prefix);
                engine.shared.lock().unwrap().handoff_tail_buffer =
                    engine.committed_tail.buffer.clone();
                pending.expected_suffix = engine.committed_tail.buffer.clone();
                pending.snapshot = lay::text_edit::VisibleTailSnapshot::new(
                    lay::text_edit::VisibleTailSource::ImeActiveComposition,
                    pending.expected_suffix.clone(),
                    Some(engine.path.clone()),
                    epoch,
                )
                .identity();
            }
            assert!(
                engine.acknowledge_owned_preedit_space_reflow(&mut pending),
                "owned prefix={retain_prefix}"
            );
            let wanted = if retain_prefix {
                actual
            } else {
                "abcdef\u{a0}"
            };
            assert_eq!(engine.committed_tail.buffer, wanted);
            assert_eq!(engine.shared.lock().unwrap().handoff_tail_buffer, wanted);
            assert_eq!(engine.committed_tail.epoch, epoch);
            assert_eq!(engine.context_word_is_known(), known);
            assert!(engine.current_external_snapshot_agrees_with_owned_tail());
            engine.committed_tail.pending_visible_postcondition = Some(pending);
            assert_eq!(
                engine.observe_visible_postcondition(),
                crate::window_interaction::OutcomeProof::ExistingPostconditionConfirmed
            );
            assert!(engine
                .committed_tail
                .pending_visible_postcondition
                .is_none());
            assert!(engine.committed_tail.recent_replace.is_none());
        }
    }));
}

#[test]
fn firefox_preedit_space_reflow_refuses_unrelated_or_unowned_changes() {
    zbus::block_on(bounded(async {
        for (actual, selection, moved, invalid) in [
            ("changed abcdef\u{a0}", false, false, ""),
            ("πrefix abcdeg\u{a0}", false, false, ""),
            ("πrefix abcdef\u{a0}", true, false, ""),
            ("πrefix abcdef\u{a0}", false, true, ""),
            ("πrefix abcdef ", false, false, ""),
            ("πrefix abcdef\u{a0}", false, false, "epoch"),
            ("πrefix abcdef\u{a0}", false, false, "field"),
            ("πrefix abcdef\u{a0}", false, false, "source"),
            ("πrefix abcdef\u{a0}", false, false, "owner"),
            ("πrefix abcdef\u{a0}", false, false, "sealed"),
            ("πrefix abcdef\u{a0}", false, false, "atomic"),
            ("πrefix abcdef\u{a0}", false, false, "missing_snapshot"),
        ] {
            let (mut harness, mut engine) = compose_and_accept_completion("πrefix\u{a0}").await;
            let cursor = actual.chars().count() as u32 - u32::from(moved);
            cycle09_surrounding_receipt(
                &mut harness,
                &mut engine,
                actual,
                cursor,
                cursor - u32::from(selection),
            )
            .await;
            let epoch = engine.committed_tail.epoch;
            let old = engine.committed_tail.buffer.clone();
            let mut pending = engine
                .committed_tail
                .pending_visible_postcondition
                .take()
                .unwrap();
            match invalid {
                "epoch" => pending.dispatched_epoch += 1,
                "field" => pending.snapshot.focus_id = Some("/other/field".into()),
                "source" => {
                    pending.snapshot.source = lay::text_edit::VisibleTailSource::ImeCommittedTail
                }
                "owner" => engine.shared.lock().unwrap().active_path = Some("/other/owner".into()),
                "sealed" => engine.context_handoff_sealed = true,
                "atomic" => engine.atomic.active = true,
                "missing_snapshot" => pending.expected_external_snapshot = None,
                _ => {}
            }
            assert!(
                !engine.acknowledge_owned_preedit_space_reflow(&mut pending),
                "{actual:?}/{invalid}"
            );
            assert_eq!(engine.committed_tail.buffer, old);
            assert_eq!(engine.shared.lock().unwrap().handoff_tail_buffer, old);
            assert_eq!(engine.committed_tail.epoch, epoch);
        }
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
