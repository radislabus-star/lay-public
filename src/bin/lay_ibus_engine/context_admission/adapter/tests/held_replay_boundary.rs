//! Deliver the paired final Space; source authority waits for the actual word.
use super::*;

async fn before_final_space(prefix: &str) -> (Harness, LayIbusEngine, u32) {
    let (mut harness, source) = stock_observed_closed_word_in(
        cycle09_harness().await,
        true,
        false,
        &[('a', 30), ('b', 48), ('c', 46)],
        prefix,
    )
    .await;
    let (source, outcome) = cycle09_manual_toggle(&mut harness, source).await;
    assert_eq!(outcome, Ok((3, false)));
    let (source, tail) = cycle09_visible_tail(&mut harness, source).await;
    let epoch = tail.unwrap().3;
    let path = format!("{TARGET_PATH}_held_replay_boundary");
    let target =
        cycle09_factory_handoff(&mut harness, source, 173_000, &path, None, "lay-ime-ru").await;
    let (mut target, armed) =
        cycle09_suppress_exact_replay(&mut harness, target, "abc ", epoch, &path, true).await;
    assert_eq!(armed, Ok(true));
    target.config.text_backend = "ime".into();
    target.config.typing_assist = false;
    target.config.nanda_precognition = false;
    let mut serial = 173_100;
    for (keyval, keycode) in [(KEY_BACKSPACE, 14); 4].into_iter().chain([
        (0x0100_0000 | 'ф' as u32, 30),
        (0x0100_0000 | 'и' as u32, 48),
        (0x0100_0000 | 'с' as u32, 46),
    ]) {
        for state in [0, RELEASE_MASK] {
            assert!(
                !firefox_replay_callback(
                    &mut harness,
                    &mut target,
                    &mut serial,
                    (keyval, keycode, state),
                )
                .await
            );
        }
    }
    assert_eq!(target.committed_tail.buffer, "фис");
    assert_no_stock_display_mutation(&terminal_delivery::legacy_effects(&mut harness).await);
    (harness, target, serial)
}

async fn hold_final_space(harness: &mut Harness, target: &mut LayIbusEngine, serial: &mut u32) {
    for state in [0, RELEASE_MASK] {
        assert!(
            firefox_observed_key_callback(harness, target, serial, (KEY_SPACE, 57, state),).await,
            "the original final Space is consumed after forwarding its exact pair"
        );
        let effects = terminal_delivery::legacy_effects(harness).await;
        let other_effects: Vec<_> = effects
            .iter()
            .filter(|effect| {
                !effect
                    .header()
                    .member()
                    .is_some_and(|member| member.as_str() == "ForwardKeyEvent")
            })
            .cloned()
            .collect();
        assert_no_stock_display_mutation(&other_effects);
        assert_eq!(
            forwarded_keys(&effects),
            if state == 0 {
                vec![(KEY_SPACE, 57, 0), (KEY_SPACE, 57, RELEASE_MASK)]
            } else {
                vec![]
            }
        );
        assert_eq!(
            effects
                .iter()
                .filter(|effect| effect
                    .header()
                    .member()
                    .is_some_and(|member| member.as_str() == "RequireSurroundingText"))
                .count(),
            usize::from(state == 0),
            "one surrounding request at hold entry, none on release",
        );
    }
    assert_eq!(target.committed_tail.buffer, "фис");
    assert!(!target.context_has_native_replay_reset_ack());
    assert!(target.committed_tail.observed_boundary_source.is_none());
    assert_no_stock_display_mutation(&terminal_delivery::legacy_effects(harness).await);
}

async fn actual_snapshot(
    harness: &Harness,
    engine: &mut LayIbusEngine,
    text: &str,
    cursor: u32,
    anchor: u32,
) {
    engine
        .set_surrounding_text(
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap(),
            crate::text::make_ibus_text(text.into()),
            cursor,
            anchor,
        )
        .await
        .unwrap();
}

fn committed_texts(effects: &[Message]) -> Vec<String> {
    effects
        .iter()
        .filter(|effect| {
            effect
                .header()
                .member()
                .is_some_and(|m| m.as_str() == "CommitText")
        })
        .map(|effect| {
            let body = effect.body();
            let value = body.deserialize::<zbus::zvariant::Value<'_>>().unwrap();
            crate::ibus_interface::ibus_text_value_to_string(&value).unwrap()
        })
        .collect()
}

pub(super) fn forwarded_keys(effects: &[Message]) -> Vec<(u32, u32, u32)> {
    effects
        .iter()
        .filter(|effect| {
            effect
                .header()
                .member()
                .is_some_and(|m| m.as_str() == "ForwardKeyEvent")
        })
        .map(|effect| effect.body().deserialize().unwrap())
        .collect()
}

#[test]
fn held_replay_boundary_partial_word_cannot_block_the_authorized_space() {
    zbus::block_on(bounded(async {
        let (mut harness, mut target, mut serial) = before_final_space("").await;
        actual_snapshot(&harness, &mut target, "фи", 2, 2).await;
        assert_no_stock_display_mutation(&terminal_delivery::legacy_effects(&mut harness).await);
        let mut effects = Vec::new();
        for state in [0, RELEASE_MASK] {
            assert!(
                firefox_observed_key_callback(
                    &mut harness,
                    &mut target,
                    &mut serial,
                    (KEY_SPACE, 57, state)
                )
                .await
            );
            effects.extend(terminal_delivery::legacy_effects(&mut harness).await);
        }
        let forwarded: Vec<(u32, u32, u32)> = effects
            .iter()
            .filter(|effect| {
                effect
                    .header()
                    .member()
                    .is_some_and(|m| m.as_str() == "ForwardKeyEvent")
            })
            .map(|effect| effect.body().deserialize().unwrap())
            .collect();
        assert_eq!(
            forwarded,
            [(KEY_SPACE, 57, 0), (KEY_SPACE, 57, RELEASE_MASK)],
            "the exact authorized key pair must progress without a future whole-word callback"
        );
        assert!(committed_texts(&effects).is_empty());
        assert_eq!(target.committed_tail.buffer, "фис");
        assert!(!target.context_stock_owned_manual_source_is_current());
        assert!(!target.context_has_native_replay_reset_ack());
        assert!(target.committed_tail.observed_boundary_source.is_none());
        target.cancel_held_replay_boundary();
        actual_snapshot(&harness, &mut target, "фис", 3, 3).await;
        assert_no_stock_display_mutation(&terminal_delivery::legacy_effects(&mut harness).await);
        assert!(!target.context_stock_owned_manual_source_is_current());
    }));
}

#[test]
fn held_replay_boundary_dispatches_once_then_requires_actual_word_for_source() {
    for prefix in ["", "文 🌍 "] {
        zbus::block_on(bounded(async {
            let (mut harness, mut target, mut serial) = before_final_space(prefix).await;
            hold_final_space(&mut harness, &mut target, &mut serial).await;
            let field = target.context_token.as_ref().unwrap().exact_field_receipt();
            let (mut target, pending) = private_visible_tail_v4(&mut harness, target).await;
            let pending = pending.unwrap();
            assert_eq!(pending.0, "passive:unknown-context");
            assert!(
                pending.1.is_empty(),
                "held native mirror is not a text source"
            );
            assert_eq!(
                pending.5, field,
                "hold must retain only current field identity"
            );
            assert_eq!(pending.6, target.content_allows_personalization());
            let partial = format!("{prefix}фи");
            let cursor = partial.chars().count() as u32;
            actual_snapshot(&harness, &mut target, &partial, cursor, cursor).await;
            assert_no_stock_display_mutation(
                &terminal_delivery::legacy_effects(&mut harness).await,
            );
            assert_eq!(target.committed_tail.buffer, "фис");
            let actual = format!("{prefix}фис");
            let cursor = actual.chars().count() as u32;
            actual_snapshot(&harness, &mut target, &actual, cursor, cursor).await;
            let effects = terminal_delivery::legacy_effects(&mut harness).await;
            assert_no_stock_display_mutation(&effects);
            assert!(effects
                .iter()
                .all(|e| e.header().member().is_none_or(|m| !matches!(
                    m.as_str(),
                    "DeleteSurroundingText" | "ForwardKeyEvent"
                ))));
            assert_eq!(target.committed_tail.buffer, "фис ");
            assert!(target.context_word_is_known());
            assert!(target.context_stock_owned_manual_source_is_current());
            assert!(!target.context_has_native_replay_reset_ack());
            assert!(target
                .committed_tail
                .pending_visible_postcondition
                .is_none());
            assert!(target.committed_tail.pending_completion_learning.is_none());
            assert!(target.client_context.surrounding_text_snapshot.is_none());
            let source = target
                .committed_tail
                .observed_boundary_source
                .as_ref()
                .unwrap();
            assert_eq!(source.observed_external_prefix.as_deref(), Some(prefix));
            assert_eq!(source.tail, "фис ");
            // A duplicate callback must not commit again or claim a result ACK.
            actual_snapshot(&harness, &mut target, &actual, cursor, cursor).await;
            assert_no_stock_display_mutation(
                &terminal_delivery::legacy_effects(&mut harness).await,
            );
            assert!(!target.context_has_native_replay_reset_ack());
        }));
    }
}

#[test]
fn held_replay_boundary_cancellation_never_commits_from_a_late_matching_word() {
    for invalidation in [
        "owner",
        "reset",
        "focus",
        "layout",
        "revision",
        "epoch",
        "expired",
        "selection",
        "new-key",
        "pair",
        "sensitive",
        "shared-path",
        "no-token",
        "no-source-provenance",
        "snapshot-provenance",
    ] {
        zbus::block_on(bounded(async {
            let (mut harness, mut target, mut serial) = before_final_space("").await;
            hold_final_space(&mut harness, &mut target, &mut serial).await;
            match invalidation {
                "owner" => assert!(target
                    .context_admission
                    .as_ref()
                    .unwrap()
                    .revoke_current_owner(target.context_owner.as_ref().unwrap())),
                "reset" => target.revoke_context_word(),
                "focus" => target.client_context.focus_serial += 1,
                "layout" => target.layout_gesture.layout_generation += 1,
                "revision" => {
                    actual_snapshot(&harness, &mut target, "other", 5, 5).await;
                }
                "epoch" => target.committed_tail.epoch += 1,
                "expired" | "no-source-provenance" | "snapshot-provenance" => {
                    let Some(AutocorrectSuppression::ExactReplay(mut scope)) =
                        target.committed_tail.autocorrect_suppression.clone()
                    else {
                        panic!("missing held replay")
                    };
                    match invalidation {
                        "expired" => scope.expires_at = Instant::now() - Duration::from_secs(1),
                        "no-source-provenance" => scope.source_provenance = None,
                        "snapshot-provenance" => scope.source_provenance = Some(
                            crate::context_admission::ExactManualSourceProvenance::ClientSnapshot,
                        ),
                        _ => unreachable!(),
                    }
                    target.committed_tail.autocorrect_suppression =
                        Some(AutocorrectSuppression::ExactReplay(scope.clone()));
                    target.shared.lock().unwrap().autocorrect_suppression =
                        Some(AutocorrectSuppression::ExactReplay(scope));
                }
                "selection" => actual_snapshot(&harness, &mut target, "фис", 3, 0).await,
                "new-key" => {
                    let _ = firefox_observed_key_callback(
                        &mut harness,
                        &mut target,
                        &mut serial,
                        ('x' as u32, 45, 0),
                    )
                    .await;
                    let _ = terminal_delivery::legacy_effects(&mut harness).await;
                }
                "pair" => target.shared.lock().unwrap().autocorrect_suppression = None,
                "sensitive" => target.client_context.content_purpose = 8,
                "shared-path" => target.shared.lock().unwrap().active_path = None,
                "no-token" => target.context_token = None,
                _ => unreachable!(),
            }
            actual_snapshot(&harness, &mut target, "фис", 3, 3).await;
            assert_no_stock_display_mutation(
                &terminal_delivery::legacy_effects(&mut harness).await,
            );
            assert!(
                !target.context_stock_owned_manual_source_is_current(),
                "{invalidation}"
            );
            assert!(
                !target.context_has_native_replay_reset_ack(),
                "{invalidation}"
            );
            assert!(
                target.committed_tail.observed_boundary_source.is_none(),
                "{invalidation}"
            );
        }));
    }
}

#[test]
fn held_replay_boundary_caret_contradiction_cancels_before_late_matching_word() {
    for prefix in ["", "文 🌍 "] {
        for surface in ["фи", "фис", "фис x"] {
            zbus::block_on(bounded(async {
                let (mut harness, mut target, mut serial) = before_final_space(prefix).await;
                hold_final_space(&mut harness, &mut target, &mut serial).await;
                let text = format!("{prefix}{surface}");
                let cursor = text.chars().count() as u32 - 1;
                actual_snapshot(&harness, &mut target, &text, cursor, cursor).await;
                assert!(
                    !target.has_held_replay_boundary(),
                    "a non-end caret must cancel the actionable boundary: {text:?}"
                );
                let actual = format!("{prefix}фис");
                let cursor = actual.chars().count() as u32;
                actual_snapshot(&harness, &mut target, &actual, cursor, cursor).await;
                assert_no_stock_display_mutation(
                    &terminal_delivery::legacy_effects(&mut harness).await,
                );
                assert!(!target.context_stock_owned_manual_source_is_current());
                assert!(target.committed_tail.observed_boundary_source.is_none());
                assert!(!target.context_has_native_replay_reset_ack());
                assert!(target
                    .committed_tail
                    .pending_visible_postcondition
                    .is_none());
                assert!(target.committed_tail.pending_completion_learning.is_none());
            }));
        }
    }
}

#[test]
fn held_replay_boundary_accepts_actual_word_already_observed_before_space() {
    zbus::block_on(bounded(async {
        let (mut harness, mut target, mut serial) = before_final_space("").await;
        actual_snapshot(&harness, &mut target, "фис", 3, 3).await;
        assert_no_stock_display_mutation(&terminal_delivery::legacy_effects(&mut harness).await);
        assert!(
            firefox_observed_key_callback(
                &mut harness,
                &mut target,
                &mut serial,
                (KEY_SPACE, 57, 0),
            )
            .await
        );
        let effects = terminal_delivery::legacy_effects(&mut harness).await;
        assert!(committed_texts(&effects).is_empty());
        assert_eq!(
            forwarded_keys(&effects),
            [(KEY_SPACE, 57, 0), (KEY_SPACE, 57, RELEASE_MASK)]
        );
        assert!(target.context_stock_owned_manual_source_is_current());
        assert!(
            firefox_observed_key_callback(
                &mut harness,
                &mut target,
                &mut serial,
                (KEY_SPACE, 57, RELEASE_MASK),
            )
            .await
        );
        assert_no_stock_display_mutation(&terminal_delivery::legacy_effects(&mut harness).await);
        assert!(!target.context_has_native_replay_reset_ack());
    }));
}

#[test]
fn held_replay_boundary_output_failure_drop_and_owner_race_never_publish_source() {
    use crate::output::TestEngineOutput;
    use crate::window_interaction::WindowInteraction;
    for phase in ["request", "press", "release"] {
        for failure in ["output", "drop", "owner-during-output"] {
            zbus::block_on(bounded(async {
                let (mut harness, mut target, mut serial) = before_final_space("").await;
                let message = firefox_observed_key_ingress(
                    &mut harness,
                    &target.path,
                    &mut serial,
                    (KEY_SPACE, 57, 0),
                )
                .await;
                let adapter = target.context_admission.clone().unwrap();
                let owner = target.context_owner.clone().unwrap();
                let index = match phase {
                    "press" => Some(1),
                    "release" => Some(2),
                    _ => None,
                };
                let mut observed = TestEngineOutput {
                    legacy_transport: true,
                    fail_require_surrounding: phase == "request" && failure == "output",
                    pause_before_require_surrounding: phase == "request" && failure != "output",
                    fail_forward_at: (failure == "output").then_some(index).flatten(),
                    pause_before_forward_at: (failure != "output").then_some(index).flatten(),
                    ..Default::default()
                };
                let mut output = EngineOutput::test(&mut observed);
                let header = message.header();
                let mut pending = Box::pin(WindowInteraction::process_legacy_key(
                    &mut target,
                    &header,
                    &mut output,
                    KEY_SPACE,
                    57,
                    0,
                ));
                if failure == "output" {
                    assert!(pending.as_mut().await.is_err());
                } else {
                    assert!(futures_lite::future::poll_once(pending.as_mut())
                        .await
                        .is_none());
                    if failure == "owner-during-output" {
                        assert!(adapter.revoke_current_owner(&owner));
                        assert!(pending.as_mut().await.is_err());
                    }
                }
                drop(pending);
                let attempts = index
                    .unwrap_or(0)
                    .saturating_sub(usize::from(failure == "drop"));
                assert_eq!(observed.forwarded_keys.len(), attempts, "{phase}/{failure}");
                assert!(observed.committed_texts.is_empty());
                assert_eq!(target.committed_tail.buffer, "фис");
                assert!(!target.context_stock_owned_manual_source_is_current());
                assert!(target.committed_tail.observed_boundary_source.is_none());
                actual_snapshot(&harness, &mut target, "фис", 3, 3).await;
                assert_no_stock_display_mutation(
                    &terminal_delivery::legacy_effects(&mut harness).await,
                );
                assert!(!target.context_has_native_replay_reset_ack());
                assert!(target
                    .committed_tail
                    .pending_visible_postcondition
                    .is_none());
                assert!(target.committed_tail.pending_completion_learning.is_none());
            }));
        }
    }
}
