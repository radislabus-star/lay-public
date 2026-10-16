//! A delivered native replacement may still have a prior client text receipt.
use super::*;

#[test]
fn completed_native_closed_replay_survives_reset_after_partial_replacement() {
    for prefix in ["", "文 🌍 "] {
        for separator in [" ", "\u{a0}"] {
            for adverse in [
                "none",
                "selection",
                "foreign_prefix",
                "extra_epoch",
                "expired",
                "scope_pair",
            ] {
                zbus::block_on(bounded(async {
                    let capabilities =
                        41 | crate::window_interaction::IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH;
                    let mut harness = cycle09_harness().await;
                    let (mut target, path, mut serial) =
                        td121_inherited_first_closed_word_with_capabilities(
                            &mut harness,
                            prefix,
                            separator,
                            "",
                            Some(capabilities),
                        )
                        .await;
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
                    for (index, (ch, code)) in [('ф', 30), ('и', 48), ('с', 46), (' ', 57)]
                        .into_iter()
                        .enumerate()
                    {
                        let key = if ch == ' ' {
                            KEY_SPACE
                        } else {
                            0x0100_0000 | ch as u32
                        };
                        if index == 1 {
                            let mut received = Vec::new();
                            for state in [0, RELEASE_MASK] {
                                received.push(
                                    firefox_receive_deferred_native_key(
                                        &mut harness,
                                        &target,
                                        &mut serial,
                                        (key, code, state),
                                    )
                                    .await,
                                );
                            }
                            let reset =
                                firefox_receive_deferred_reset(&mut harness, &target, &mut serial)
                                    .await;
                            for message in &received {
                                assert!(
                                    !firefox_dispatch_deferred_native_key(
                                        &mut harness,
                                        &mut target,
                                        message,
                                    )
                                    .await
                                );
                            }
                            target
                                .reset(
                                    reset.header(),
                                    zbus::object_server::SignalEmitter::new(
                                        &harness.connection,
                                        target.path.clone(),
                                    )
                                    .unwrap(),
                                )
                                .await
                                .unwrap();
                            cycle09_assert_no_local_text_effect(&mut harness).await;
                            let old = format!("{prefix}a");
                            let chars = old.chars().count() as u32;
                            cycle09_surrounding_receipt(
                                &mut harness,
                                &mut target,
                                &old,
                                chars,
                                chars,
                            )
                            .await;
                        } else {
                            for state in [0, RELEASE_MASK] {
                                assert!(
                                    !firefox_replay_callback(
                                        &mut harness,
                                        &mut target,
                                        &mut serial,
                                        (key, code, state),
                                    )
                                    .await
                                );
                            }
                        }
                    }
                    assert_eq!(target.committed_tail.buffer, "фис ");
                    let completed_epoch = target.committed_tail.epoch;
                    if adverse == "extra_epoch" {
                        target.committed_tail.epoch += 1;
                    }
                    if adverse == "expired" {
                        let expired = Instant::now() - Duration::from_millis(1);
                        if let Some(AutocorrectSuppression::ExactReplay(scope)) =
                            target.committed_tail.autocorrect_suppression.as_mut()
                        {
                            scope.expires_at = expired;
                        }
                        if let Some(AutocorrectSuppression::ExactReplay(scope)) = target
                            .shared
                            .lock()
                            .unwrap()
                            .autocorrect_suppression
                            .as_mut()
                        {
                            scope.expires_at = expired;
                        }
                    }
                    if adverse == "scope_pair" {
                        target.shared.lock().unwrap().autocorrect_suppression = None;
                    }
                    firefox_replay_reset(&mut harness, &mut target, &mut serial).await;
                    let partial = format!("{prefix}ф");
                    let chars = partial.chars().count() as u32;
                    cycle09_surrounding_receipt(&mut harness, &mut target, &partial, chars, chars)
                        .await;
                    assert!(!target.context_reset_rereceipt_exact_manual_handoff_allowed());
                    assert!(!target.context_word_is_known());
                    firefox_replay_reset(&mut harness, &mut target, &mut serial).await;
                    let actual = if adverse == "foreign_prefix" {
                        format!("other {prefix}фис{separator}")
                    } else {
                        format!("{prefix}фис{separator}")
                    };
                    let chars = actual.chars().count() as u32;
                    let anchor = if adverse == "selection" { 0 } else { chars };
                    cycle09_surrounding_receipt(&mut harness, &mut target, &actual, chars, anchor)
                        .await;
                    if adverse != "none" {
                        assert!(
                            !target.context_reset_rereceipt_exact_manual_handoff_allowed(),
                            "{adverse}"
                        );
                        let (_, disposition, effects) =
                            cycle09_manual_toggle_with_effects(&mut harness, target).await;
                        assert_eq!(disposition, Ok((0, false)), "{adverse}");
                        assert!(
                            !effects.iter().any(|effect| {
                                effect.header().member().is_some_and(|member| {
                                    matches!(
                                        member.as_str(),
                                        "CommitText" | "DeleteSurroundingText"
                                    )
                                })
                            }),
                            "{adverse}"
                        );
                        return;
                    }
                    assert!(
                        target.context_reset_rereceipt_exact_manual_handoff_allowed(),
                        "complete same-owner native replay needs its exact client receipt",
                    );
                    assert_eq!(target.committed_tail.epoch, completed_epoch);
                    let (target, disposition, effects) =
                        cycle09_manual_toggle_with_effects(&mut harness, target).await;
                    assert_eq!(disposition, Ok((3, false)));
                    assert!(!effects.iter().any(|effect| {
                        effect.header().member().is_some_and(|member| {
                            matches!(member.as_str(), "CommitText" | "DeleteSurroundingText")
                        })
                    }));
                    assert_eq!(target.committed_tail.epoch, completed_epoch.wrapping_add(1));
                    let (_, tail) = cycle09_visible_tail(&mut harness, target).await;
                    assert!(cycle09_tail_is_authoritative(
                        &tail,
                        &path,
                        true,
                        &format!("фис{separator}"),
                    ));
                }));
            }
        }
    }
}

#[test]
fn completed_native_replay_prior_receipt_refreshes_same_manual_gesture_without_edit() {
    for prefix in ["", "文 🌍 "] {
        for separator in [" ", "\u{a0}"] {
            zbus::block_on(bounded(async {
                let capabilities =
                    41 | crate::window_interaction::IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH;
                let mut harness = cycle09_harness().await;
                let (mut target, path, mut serial) =
                    td121_inherited_first_closed_word_with_capabilities(
                        &mut harness,
                        prefix,
                        separator,
                        "",
                        Some(capabilities),
                    )
                    .await;
                td121_deliver_inherited_closed_replay(&mut harness, &mut target, &mut serial).await;
                firefox_replay_reset(&mut harness, &mut target, &mut serial).await;
                let old_surface = format!("{prefix}abc{separator}");
                let chars = old_surface.chars().count() as u32;
                cycle09_surrounding_receipt(&mut harness, &mut target, &old_surface, chars, chars)
                    .await;
                assert!(!target.context_reset_rereceipt_exact_manual_handoff_allowed());
                assert!(!target.context_word_is_known());
                assert!(!target.composition.preedit_visible);
                assert_eq!(target.committed_tail.buffer, "фис ");
                assert!(target
                    .completed_exact_replay_boundary_provenance()
                    .or_else(|| target.exact_replay_reset_provenance(true))
                    .is_some());
                let epoch = target.committed_tail.epoch;
                let (mut target, disposition, effects) =
                    cycle09_manual_toggle_with_effects(&mut harness, target).await;
                assert_eq!(
                    disposition,
                    Ok(lay::manual_toggle::ImeManualToggleOutcome::AwaitingExactSnapshot.as_v3()),
                    "same physical gesture must request a fresh receipt before editing",
                );
                assert_eq!(target.committed_tail.epoch, epoch);
                assert_eq!(target.committed_tail.buffer, "фис ");
                assert!(effects.iter().any(|effect| {
                    effect
                        .header()
                        .member()
                        .is_some_and(|m| m.as_str() == "RequireSurroundingText")
                }));
                assert!(!effects.iter().any(|effect| {
                    effect.header().member().is_some_and(|m| {
                        matches!(m.as_str(), "CommitText" | "DeleteSurroundingText")
                    })
                }));
                let actual = format!("{prefix}фис{separator}");
                let chars = actual.chars().count() as u32;
                cycle09_surrounding_receipt(&mut harness, &mut target, &actual, chars, chars).await;
                assert!(
                    target.context_reset_rereceipt_exact_manual_handoff_allowed()
                        || target.context_word_is_known()
                );
                let (target, disposition) = cycle09_manual_toggle(&mut harness, target).await;
                assert_eq!(disposition, Ok((3, false)));
                let (_, tail) = cycle09_visible_tail(&mut harness, target).await;
                assert!(cycle09_tail_is_authoritative(
                    &tail,
                    &path,
                    true,
                    &format!("фис{separator}"),
                ));
            }));
        }
    }
}

#[test]
fn completed_native_replay_refresh_refuses_incomplete_or_contradictory_provenance() {
    for adverse in [
        "unmatched_projection",
        "incomplete",
        "selection",
        "foreign_prefix",
        "extra_epoch",
        "expired",
    ] {
        zbus::block_on(bounded(async {
            let capabilities =
                41 | crate::window_interaction::IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH;
            let mut harness = cycle09_harness().await;
            let (mut target, _, mut serial) = td121_inherited_first_closed_word_with_capabilities(
                &mut harness,
                "文 🌍 ",
                " ",
                "",
                Some(capabilities),
            )
            .await;
            if adverse == "incomplete" {
                for state in [0, RELEASE_MASK] {
                    assert!(
                        !firefox_replay_callback(
                            &mut harness,
                            &mut target,
                            &mut serial,
                            (KEY_BACKSPACE, 14, state)
                        )
                        .await
                    );
                }
                firefox_replay_reset(&mut harness, &mut target, &mut serial).await;
            } else if adverse == "unmatched_projection" {
                td121_deliver_inherited_closed_replay(&mut harness, &mut target, &mut serial).await;
            } else {
                td121_finish_inherited_closed_replay(&mut harness, &mut target, &mut serial).await;
            }
            let surface = if adverse == "foreign_prefix" {
                "other abc "
            } else {
                "文 🌍 abc "
            };
            let chars = surface.chars().count() as u32;
            cycle09_surrounding_receipt(
                &mut harness,
                &mut target,
                surface,
                chars,
                if adverse == "selection" { 0 } else { chars },
            )
            .await;
            if adverse == "extra_epoch" {
                target.committed_tail.epoch = target.committed_tail.epoch.wrapping_add(1);
            }
            if adverse == "expired" {
                let expired = Instant::now() - Duration::from_millis(1);
                if let Some(AutocorrectSuppression::ExactReplay(replay)) =
                    target.committed_tail.autocorrect_suppression.as_mut()
                {
                    replay.expires_at = expired;
                }
                if let Some(AutocorrectSuppression::ExactReplay(replay)) = target
                    .shared
                    .lock()
                    .unwrap()
                    .autocorrect_suppression
                    .as_mut()
                {
                    replay.expires_at = expired;
                }
            }
            assert!(
                !target.context_reset_rereceipt_manual_refresh_allowed(),
                "{adverse}"
            );
            let epoch = target.committed_tail.epoch;
            let buffer = target.committed_tail.buffer.clone();
            let (target, outcome, effects) =
                cycle09_manual_toggle_with_effects(&mut harness, target).await;
            assert_eq!(outcome, Ok((0, false)), "{adverse}");
            assert_eq!(target.committed_tail.epoch, epoch, "{adverse}");
            assert_eq!(target.committed_tail.buffer, buffer, "{adverse}");
            assert!(
                !effects
                    .iter()
                    .any(|effect| effect.header().member().is_some_and(|m| matches!(
                        m.as_str(),
                        "CommitText" | "DeleteSurroundingText" | "RequireSurroundingText"
                    ))),
                "{adverse}"
            );
        }));
    }
}
