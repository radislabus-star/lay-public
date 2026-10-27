//! A delivered native replacement may still have a prior client text receipt.
use super::*;

#[test]
fn completed_native_closed_replay_survives_reset_after_partial_replacement() {
    for prefix in ["", "文 🌍 "] {
        for separator in [
            " ",
            "\u{a0}",
            "\u{a0}\u{a0}",
            " \u{a0}",
            "\u{a0} \u{a0} ",
            "\u{a0}\u{a0}\u{a0}\u{a0}\u{a0}\u{a0}\u{a0}\u{a0}\u{a0}\u{a0}",
        ] {
            let count = separator.chars().count();
            let mut observed_boundaries = vec![
                " ".repeat(count),
                format!("{}\u{a0}", " ".repeat(count - 1)),
                separator.to_string(),
                (0..count)
                    .map(|index| if index % 2 == 0 { '\u{a0}' } else { ' ' })
                    .collect(),
            ];
            observed_boundaries.sort();
            observed_boundaries.dedup();
            for observed_separator in observed_boundaries {
                for adverse in [
                    "none",
                    "selection",
                    "foreign_prefix",
                    "extra_epoch",
                    "expired",
                    "scope_pair",
                    "scope_mismatch",
                    "scope_prefix",
                    "wrong_word",
                    "missing_boundary",
                    "extra_boundary",
                    "other_whitespace",
                    "interior_caret",
                    "incomplete",
                ] {
                    if adverse != "none" && observed_separator != separator {
                        continue;
                    }
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
                        let boundary_count = separator.chars().count();
                        for _ in 0..3 + boundary_count {
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
                        let mut replacement_keys = vec![('ф', 30), ('и', 48), ('с', 46)];
                        replacement_keys.extend(std::iter::repeat_n((' ', 57), boundary_count));
                        let replacement_len = replacement_keys.len();
                        for (index, (ch, code)) in replacement_keys.into_iter().enumerate() {
                            if adverse == "incomplete" && index + 1 == replacement_len {
                                break;
                            }
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
                                let reset = firefox_receive_deferred_reset(
                                    &mut harness,
                                    &target,
                                    &mut serial,
                                )
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
                                    if index + 1 == replacement_len {
                                        firefox_completed_replay_space_callback(
                                            &mut harness,
                                            &mut target,
                                            &mut serial,
                                            state,
                                        )
                                        .await;
                                    } else {
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
                        }
                        assert_eq!(
                            target.committed_tail.buffer,
                            format!(
                                "фис{}",
                                " ".repeat(boundary_count - usize::from(adverse == "incomplete")),
                            ),
                        );
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
                        if adverse == "scope_mismatch" {
                            if let Some(AutocorrectSuppression::ExactReplay(scope)) = target
                                .shared
                                .lock()
                                .unwrap()
                                .autocorrect_suppression
                                .as_mut()
                            {
                                scope.replacement.push(' ');
                            }
                        }
                        if adverse == "scope_prefix" {
                            if let Some(AutocorrectSuppression::ExactReplay(scope)) =
                                target.committed_tail.autocorrect_suppression.as_mut()
                            {
                                scope.unchanged_prefix.push(' ');
                            }
                            if let Some(AutocorrectSuppression::ExactReplay(scope)) = target
                                .shared
                                .lock()
                                .unwrap()
                                .autocorrect_suppression
                                .as_mut()
                            {
                                scope.unchanged_prefix.push(' ');
                            }
                        }
                        firefox_replay_reset(&mut harness, &mut target, &mut serial).await;
                        let partial = format!("{prefix}ф");
                        let chars = partial.chars().count() as u32;
                        cycle09_surrounding_receipt(
                            &mut harness,
                            &mut target,
                            &partial,
                            chars,
                            chars,
                        )
                        .await;
                        assert!(!target.context_reset_rereceipt_exact_manual_handoff_allowed());
                        assert!(!target.context_word_is_known());
                        firefox_replay_reset(&mut harness, &mut target, &mut serial).await;
                        let actual = match adverse {
                            "foreign_prefix" => format!("other {prefix}фис{observed_separator}"),
                            "wrong_word" => format!("{prefix}фир{observed_separator}"),
                            "missing_boundary" => format!(
                                "{prefix}фис{}",
                                observed_separator
                                    .chars()
                                    .take(boundary_count - 1)
                                    .collect::<String>(),
                            ),
                            "extra_boundary" => format!("{prefix}фис{observed_separator} "),
                            "other_whitespace" => format!(
                                "{prefix}фис\t{}",
                                observed_separator.chars().skip(1).collect::<String>(),
                            ),
                            _ => format!("{prefix}фис{observed_separator}"),
                        };
                        let chars = actual.chars().count() as u32;
                        let cursor = if adverse == "interior_caret" {
                            chars - 1
                        } else {
                            chars
                        };
                        let anchor = if adverse == "selection" { 0 } else { cursor };
                        cycle09_surrounding_receipt(
                            &mut harness,
                            &mut target,
                            &actual,
                            cursor,
                            anchor,
                        )
                        .await;
                        if adverse != "none" {
                            assert!(
                                !target.context_reset_rereceipt_exact_manual_handoff_allowed(),
                                "{adverse}"
                            );
                            let (_, disposition, effects) =
                                cycle09_manual_toggle_with_effects(&mut harness, target).await;
                            if matches!(
                                adverse,
                                "selection"
                                    | "foreign_prefix"
                                    | "extra_epoch"
                                    | "expired"
                                    | "scope_pair"
                            ) {
                                assert_eq!(disposition, Ok((0, false)), "{adverse}");
                            } else {
                                assert!(
                                    matches!(disposition, Ok((0 | 4, false))),
                                    "contradiction cannot edit or delegate: {adverse}: {disposition:?}",
                                );
                            }
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
                        "complete replay needs its exact receipt: prefix={prefix:?}, source={separator:?}, observed={observed_separator:?}, boundaries={boundary_count}, adverse={adverse}, tail={:?}",
                        target.committed_tail.buffer,
                    );
                        assert_eq!(target.committed_tail.epoch, completed_epoch);
                        assert_eq!(
                            target.committed_tail.buffer,
                            format!("фис{observed_separator}")
                        );
                        {
                            let shared = target.shared.lock().unwrap();
                            assert_eq!(shared.handoff_tail_buffer, target.committed_tail.buffer);
                            assert_eq!(
                                shared.autocorrect_suppression,
                                target.committed_tail.autocorrect_suppression,
                            );
                            let Some(AutocorrectSuppression::ExactReplay(scope)) =
                                &target.committed_tail.autocorrect_suppression
                            else {
                                panic!("completed replay keeps its paired acknowledgement witness");
                            };
                            assert_eq!(scope.replacement, format!("фис{observed_separator}"));
                        }
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
                            &format!("фис{observed_separator}"),
                        ));
                    }));
                }
            }
        }
    }
}

#[test]
fn completed_native_replay_prior_receipt_refreshes_same_manual_gesture_without_edit() {
    for capabilities in [
        41,
        41 | crate::window_interaction::IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH,
    ] {
        for prefix in ["", "文 🌍 "] {
            for separator in [" ", "\u{a0}"] {
                for stale_surface in ["original", "last_replacement"] {
                    zbus::block_on(bounded(async {
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
                        td121_deliver_inherited_closed_replay(
                            &mut harness,
                            &mut target,
                            &mut serial,
                        )
                        .await;
                        firefox_replay_reset(&mut harness, &mut target, &mut serial).await;
                        let old_surface = if stale_surface == "original" {
                            format!("{prefix}abc{separator}")
                        } else {
                            format!("{prefix}фис")
                        };
                        let chars = old_surface.chars().count() as u32;
                        cycle09_surrounding_receipt(
                            &mut harness,
                            &mut target,
                            &old_surface,
                            chars,
                            chars,
                        )
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
                            Ok(
                                lay::manual_toggle::ImeManualToggleOutcome::AwaitingExactSnapshot
                                    .as_v3()
                            ),
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
                        cycle09_surrounding_receipt(
                            &mut harness,
                            &mut target,
                            &actual,
                            chars,
                            chars,
                        )
                        .await;
                        assert!(
                            target.context_reset_rereceipt_exact_manual_handoff_allowed()
                                || target.context_word_is_known()
                        );
                        let (target, disposition) =
                            cycle09_manual_toggle(&mut harness, target).await;
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
    }
}

#[test]
fn completed_native_replay_refresh_refuses_incomplete_or_contradictory_provenance() {
    for capabilities in [
        41,
        41 | crate::window_interaction::IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH,
    ] {
        for adverse in [
            "unmatched_projection",
            "incomplete",
            "selection",
            "foreign_prefix",
            "extra_epoch",
            "expired",
            "atomic",
            "sealed",
            "no_surrounding",
        ] {
            zbus::block_on(bounded(async {
                let mut harness = cycle09_harness().await;
                let (mut target, _, mut serial) =
                    td121_inherited_first_closed_word_with_capabilities(
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
                    td121_deliver_inherited_closed_replay(&mut harness, &mut target, &mut serial)
                        .await;
                } else {
                    td121_finish_inherited_closed_replay(&mut harness, &mut target, &mut serial)
                        .await;
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
                if adverse == "atomic" {
                    target.atomic.active = true;
                }
                if adverse == "sealed" {
                    target.context_handoff_sealed = true;
                }
                if adverse == "no_surrounding" {
                    target.client_context.surrounding_text_supported = false;
                }
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
}

#[test]
fn completed_native_replay_requested_read_rejects_invalid_or_late_response() {
    for prefix in ["", "文 🌍 "] {
        for adverse in [
            "partial",
            "selection",
            "foreign",
            "no_response",
            "late_valid",
            "expired_rpc",
            "privacy_change",
            "capability_loss",
        ] {
            zbus::block_on(bounded(async {
                let mut harness = cycle09_harness().await;
                let (mut target, _, mut serial) =
                    td121_inherited_first_closed_word_with_capabilities(
                        &mut harness,
                        prefix,
                        " ",
                        "",
                        Some(41),
                    )
                    .await;
                td121_deliver_inherited_closed_replay(&mut harness, &mut target, &mut serial).await;
                firefox_replay_reset(&mut harness, &mut target, &mut serial).await;
                let stale = format!("{prefix}фис");
                let chars = stale.chars().count() as u32;
                cycle09_surrounding_receipt(&mut harness, &mut target, &stale, chars, chars).await;
                let (mut target, disposition, effects) =
                    cycle09_manual_toggle_with_effects(&mut harness, target).await;
                assert_eq!(disposition, Ok((4, false)), "{adverse}");
                assert!(
                    effects.iter().any(|effect| effect
                        .header()
                        .member()
                        .is_some_and(|m| m.as_str() == "RequireSurroundingText")),
                    "{adverse}"
                );
                assert!(target.layout_gesture.pending_manual_refresh_at.is_some());
                match adverse {
                    "late_valid" | "expired_rpc" => {
                        target.layout_gesture.pending_manual_refresh_at =
                            Some(Instant::now() - Duration::from_millis(701))
                    }
                    "privacy_change" => target.set_content_type_state(0, 2048),
                    "capability_loss" => target.set_client_capabilities(0),
                    _ => {}
                }
                if !matches!(adverse, "no_response" | "expired_rpc") {
                    let actual = match adverse {
                        "partial" => format!("{prefix}ф"),
                        "foreign" => "other фис ".to_string(),
                        _ => format!("{prefix}фис "),
                    };
                    let chars = actual.chars().count() as u32;
                    cycle09_surrounding_receipt(
                        &mut harness,
                        &mut target,
                        &actual,
                        chars,
                        if adverse == "selection" { 0 } else { chars },
                    )
                    .await;
                }
                for _ in 0..2 {
                    let (next, disposition, effects) =
                        cycle09_manual_toggle_with_effects(&mut harness, target).await;
                    target = next;
                    assert!(
                        matches!(disposition, Ok((0 | 4, false)) | Err(_)),
                        "{adverse}: {disposition:?}"
                    );
                    assert!(
                        !effects
                            .iter()
                            .any(|effect| effect.header().member().is_some_and(|m| matches!(
                                m.as_str(),
                                "CommitText" | "DeleteSurroundingText" | "RequireSurroundingText"
                            ))),
                        "{adverse}"
                    );
                }
            }));
        }
    }
}
