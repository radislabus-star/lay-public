//! Exact ordinary-document erasure after a completed closed-word replay.
use super::*;

#[test]
fn release97_kde_closed_replay_probe_erasure_restores_exact_manual_source() {
    for (prefix, boundary, appended_boundary) in [
        ("", ' ', ' '),
        ("文 🌍 ", ' ', ' '),
        ("", '\u{a0}', '\u{a0}'),
        ("文 🌍 ", '\u{a0}', '\u{a0}'),
        ("", '\u{a0}', ' '),
        ("文 🌍 ", '\u{a0}', ' '),
        ("", ' ', '\u{a0}'),
        ("文 🌍 ", ' ', '\u{a0}'),
    ] {
        for reset in [false, true] {
            for append_reset in [false, true] {
                for proof in [
                    "whole",
                    "missing",
                    "prior",
                    "selected",
                    "interior",
                    "foreign",
                    "wrong_boundary",
                    "wrong_word",
                    "wrong_prefix",
                    "extra_boundary",
                ] {
                    zbus::block_on(bounded(Box::pin(async {
                        let mut harness = cycle09_harness().await;
                        let (mut target, path, mut serial) =
                            td121_inherited_first_closed_word(&mut harness, prefix, " ", "").await;
                        target.config.layout_backend = "kde".into();
                        td121_deliver_inherited_closed_replay(
                            &mut harness,
                            &mut target,
                            &mut serial,
                        )
                        .await;
                        if reset {
                            firefox_replay_reset(&mut harness, &mut target, &mut serial).await;
                        }
                        let closed = format!("фис{boundary}");
                        let actual = format!("{prefix}{closed}");
                        let appended_actual = format!("{prefix}фис{appended_boundary}я");
                        let count = actual.chars().count() as u32;
                        cycle09_surrounding_receipt(
                            &mut harness,
                            &mut target,
                            &actual,
                            count,
                            count,
                        )
                        .await;
                        assert!(target.context_has_native_replay_reset_ack());
                        assert!(!target.client_context.exact_surrounding_refresh_available);

                        // The stock KDE prefix policy commits this ordinary letter.
                        // Only subsequent client receipts can witness its insertion
                        // and deletion; the local mirror is never their substitute.
                        let key = 0x0100_0000 | 'я' as u32;
                        let before_insert = format!(
                            "scope={:?}, receipt={:?}, revision={}",
                            target.context_word_scope,
                            target.context_reset_rereceipt,
                            target.client_context.surrounding_observation_revision
                        );
                        assert!(
                            firefox_observed_key_callback(
                                &mut harness,
                                &mut target,
                                &mut serial,
                                (key, 44, 0),
                            )
                            .await
                        );
                        let (members, committed) = drain_output_to_text_proof(&mut harness).await;
                        let after_insert = format!(
                            "scope={:?}, receipt={:?}, revision={}",
                            target.context_word_scope,
                            target.context_reset_rereceipt,
                            target.client_context.surrounding_observation_revision
                        );
                        assert_eq!(committed, ["я"]);
                        assert!(!members
                            .iter()
                            .any(|member| member == "DeleteSurroundingText"));
                        firefox_replay_callback(
                            &mut harness,
                            &mut target,
                            &mut serial,
                            (key, 44, RELEASE_MASK),
                        )
                        .await;
                        if append_reset {
                            firefox_replay_reset(&mut harness, &mut target, &mut serial).await;
                        }
                        cycle09_surrounding_receipt(
                            &mut harness,
                            &mut target,
                            &appended_actual,
                            count + 1,
                            count + 1,
                        )
                        .await;

                        assert_eq!(target.client_context.surrounding_text_snapshot.as_ref()
                            .map(|snapshot| snapshot.text.as_str()), Some(appended_actual.as_str()),
                            "the source is the actual inserted surface, not a fabricated Reset: before={before_insert}; after_commit={after_insert}");
                        let before_erase = format!("managed={}, floor={:?}, scope={:?}, receipt={:?}, layout={}, lease={}, focus={}, revision={}, epoch={}",
                            target.client_context.managed_input, target.client_context.managed_commit_snapshot_floor,
                            target.context_word_scope, target.context_reset_rereceipt,
                            target.layout_gesture.layout_generation, target.client_context.runtime_owner_lease_identity,
                            target.client_context.focus_serial, target.client_context.surrounding_observation_revision,
                            target.committed_tail.epoch);

                        assert!(
                            !firefox_observed_key_callback(
                                &mut harness,
                                &mut target,
                                &mut serial,
                                (KEY_BACKSPACE, 14, 0),
                            )
                            .await
                        );
                        let effects = drain_output_to_proof(&mut harness).await;
                        let after_erase = format!(
                            "scope={:?}, receipt={:?}, epoch={}",
                            target.context_word_scope,
                            target.context_reset_rereceipt,
                            target.committed_tail.epoch
                        );
                        assert!(!effects
                            .iter()
                            .any(|effect| effect == "DeleteSurroundingText"));
                        firefox_replay_callback(
                            &mut harness,
                            &mut target,
                            &mut serial,
                            (KEY_BACKSPACE, 14, RELEASE_MASK),
                        )
                        .await;
                        firefox_replay_reset(&mut harness, &mut target, &mut serial).await;
                        match proof {
                            "missing" => {}
                            "prior" => {
                                cycle09_surrounding_receipt(
                                    &mut harness,
                                    &mut target,
                                    &appended_actual,
                                    count + 1,
                                    count + 1,
                                )
                                .await
                            }
                            "selected" => {
                                cycle09_surrounding_receipt(
                                    &mut harness,
                                    &mut target,
                                    &actual,
                                    count,
                                    count - 1,
                                )
                                .await
                            }
                            "interior" => {
                                cycle09_surrounding_receipt(
                                    &mut harness,
                                    &mut target,
                                    &actual,
                                    count - 1,
                                    count - 1,
                                )
                                .await
                            }
                            "foreign" => {
                                cycle09_surrounding_receipt(
                                    &mut harness,
                                    &mut target,
                                    &format!("{actual}а"),
                                    count + 1,
                                    count + 1,
                                )
                                .await
                            }
                            "whole" => {
                                cycle09_surrounding_receipt(
                                    &mut harness,
                                    &mut target,
                                    &actual,
                                    count,
                                    count,
                                )
                                .await
                            }
                            "wrong_boundary" | "wrong_word" | "wrong_prefix" | "extra_boundary" => {
                                let text = match proof {
                                    "wrong_boundary" => format!("{prefix}фис\u{202f}"),
                                    "wrong_word" => format!("{prefix}фиы{boundary}"),
                                    "wrong_prefix" => format!("а{actual}"),
                                    "extra_boundary" => format!("{actual}{boundary}"),
                                    _ => unreachable!(),
                                };
                                let chars = text.chars().count() as u32;
                                cycle09_surrounding_receipt(
                                    &mut harness,
                                    &mut target,
                                    &text,
                                    chars,
                                    chars,
                                )
                                .await
                            }
                            _ => unreachable!(),
                        }

                        let state = format!(
                            "prefix={prefix:?}, reset={reset}, append_reset={append_reset}, proof={proof}, before_erase={before_erase}, after_erase={after_erase}, scope={:?}, receipt={:?}",
                        target.context_word_scope, target.context_reset_rereceipt,
                    );
                        let (target, outcome) = cycle09_manual_toggle(&mut harness, target).await;
                        if proof != "whole" {
                            // A deferred read is allowed; execution/delegation and
                            // an authoritative tail require the whole actual receipt.
                            assert!(
                                matches!(outcome, Ok((0 | 4, false))),
                                "{state}; outcome={outcome:?}"
                            );
                            let (_, tail) = cycle09_visible_tail(&mut harness, target).await;
                            assert!(
                                !matches!(&tail, Ok((state, _, _, _, _, _))
                        if state == "passive:committed-tail"),
                                "{state}; captured={tail:?}"
                            );
                            cycle09_assert_no_local_text_effect(&mut harness).await;
                            return;
                        }
                        assert_eq!(outcome, Ok((3, false)), "{state}");
                        let (_, tail) = cycle09_visible_tail(&mut harness, target).await;
                        assert!(
                            cycle09_tail_is_authoritative(&tail, &path, true, &closed),
                            "{state}; captured={tail:?}"
                        );
                    })));
                }
            }
        }
    }
}
