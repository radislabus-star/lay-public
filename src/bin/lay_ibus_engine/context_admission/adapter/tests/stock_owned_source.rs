//! Explicit manual readout retains a current locally observed stock-client word.
use super::*;

#[path = "held_replay_boundary.rs"]
mod held_replay_boundary;

pub(super) async fn stock_owned_manual_fixture() -> (Harness, LayIbusEngine) {
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

async fn stock_observed_closed_word_fixture(close_word: bool) -> (Harness, LayIbusEngine) {
    stock_observed_closed_word_in(
        bootstrap_harness_with_budget(CALLBACK_BUDGET).await,
        close_word,
        true,
        &[('с', 46), ('л', 37), ('о', 36), ('в', 32), ('о', 36)],
        "",
    )
    .await
}

async fn stock_observed_closed_word_in(
    mut harness: Harness,
    close_word: bool,
    layout_is_ru: bool,
    keys: &[(char, u32)],
    prefix: &str,
) -> (Harness, LayIbusEngine) {
    let mut engine = new_engine(&harness);
    start_source_free_unknown(&mut harness, &mut engine).await;
    engine.set_client_capabilities(41);
    engine.config.nanda_precognition = true;
    engine.config.nanda_autocorrect = false;
    engine.layout_gesture.layout_is_ru = layout_is_ru;
    let mut observed = prefix.to_owned();
    for (i, (ch, code)) in keys.iter().copied().enumerate() {
        let serial = 166_000 + i as u32 * 2;
        assert!(legacy_key(&mut harness, &mut engine, serial, ch as u32, code, 0).await);
        // Match the real client: the first projection arrives after release.
        if i == 0 {
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    serial + 1,
                    ch as u32,
                    code,
                    RELEASE_MASK
                )
                .await
            );
        }
        observed.push(ch);
        engine
            .set_surrounding_text(
                zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                    .unwrap(),
                crate::text::make_ibus_text(observed.clone()),
                observed.chars().count() as u32,
                observed.chars().count() as u32,
            )
            .await
            .unwrap();
        if i != 0 {
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    serial + 1,
                    ch as u32,
                    code,
                    RELEASE_MASK
                )
                .await
            );
        }
    }
    assert!(engine.client_context.surrounding_text_callback_observed);
    assert!(engine.composition.buffer.is_empty());
    assert!(engine.current_external_snapshot_agrees_with_owned_tail());
    if close_word {
        assert!(engine.capture_stock_observed_boundary_source(' ').is_some(),
            "pre-Space: known={}, required={}, sealed={}, atomic={}, sensitive={}, exact={}, replay={}, quarantined={}",
            engine.context_word_is_known(), engine.context_admission_required,
            engine.context_handoff_sealed, engine.atomic.active, engine.content_is_sensitive(),
            engine.client_context.exact_surrounding_refresh_available,
            matches!(engine.committed_tail.autocorrect_suppression, Some(AutocorrectSuppression::ExactReplay(_))),
            engine.exact_replay_quarantine_active() || engine.exact_replay_tail_change_quarantined);
        assert!(legacy_key(&mut harness, &mut engine, 166_100, KEY_SPACE, 57, 0).await);
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                166_101,
                KEY_SPACE,
                57,
                RELEASE_MASK
            )
            .await
        );
        let word: String = keys.iter().map(|(ch, _)| *ch).collect();
        assert_eq!(engine.committed_tail.buffer, format!("{word} "));
        // CommitText clears the pre-Space snapshot. This client supplies no
        // later snapshot for the locally committed boundary.
        assert!(engine.client_context.surrounding_text_snapshot.is_none());
    }
    let _ = terminal_delivery::legacy_effects(&mut harness).await;
    (harness, engine)
}

#[test]
fn stock_observed_boundary_handoff_preserves_real_prefix_without_target_receipt() {
    for prefix in ["", "文 🌍 "] {
        zbus::block_on(bounded(async {
            let (mut harness, source) = stock_observed_closed_word_in(
                cycle09_harness().await,
                true,
                false,
                &[('a', 30), ('b', 48), ('c', 46)],
                prefix,
            )
            .await;
            assert!(source.context_stock_owned_manual_source_is_current());
            let (source, outcome) = cycle09_manual_toggle(&mut harness, source).await;
            assert_eq!(outcome, Ok((3, false)));
            let (source, visible) = cycle09_visible_tail(&mut harness, source).await;
            let epoch = visible.unwrap().3;
            let path = format!("{TARGET_PATH}_stock_observed_prefix");
            let target =
                cycle09_factory_handoff(&mut harness, source, 167_000, &path, None, "lay-ime-ru")
                    .await;
            let revision = target.client_context.surrounding_observation_revision;
            assert_eq!(
                target
                    .exact_manual_target_snapshot
                    .as_ref()
                    .unwrap()
                    .source
                    .provenance,
                crate::context_admission::ExactManualSourceProvenance::ObservedWordAndLocalSpace,
            );
            let (target, armed) =
                cycle09_suppress_exact_replay(&mut harness, target, "abc ", epoch, &path, true)
                    .await;
            assert_eq!(armed, Ok(true));
            assert!(target.client_context.surrounding_text_snapshot.is_none());
            assert_eq!(
                target.client_context.surrounding_observation_revision,
                revision
            );
            assert!(!target.context_has_native_replay_reset_ack());
            assert!(!target.exact_managed_surrounding_word_is_current());
            let Some(AutocorrectSuppression::ExactReplay(scope)) =
                target.committed_tail.autocorrect_suppression.as_ref()
            else {
                panic!("authenticated native replay must retain its scoped source");
            };
            assert_eq!(scope.observed_external_prefix.as_deref(), Some(prefix));
        }));
    }
}

#[test]
fn stock_observed_word_space_manual_handoff_keeps_local_boundary_source() {
    zbus::block_on(bounded(async {
        let (mut harness, engine) = stock_observed_closed_word_fixture(true).await;
        assert!(!engine.exact_managed_surrounding_word_is_current());
        assert!(!engine.context_has_native_replay_reset_ack());
        let revision = engine.client_context.surrounding_observation_revision;
        let (engine, outcome, _) = cycle09_manual_toggle_with_effects(&mut harness, engine).await;
        assert_eq!(outcome, Ok((3, false)));
        let (engine, visible) = private_visible_tail_v4(&mut harness, engine).await;
        let visible = visible.unwrap();
        assert_eq!(visible.0, "passive:committed-tail",
            "local boundary source: known={}, replay_wait={}, bound_handoff={}, snapshot_current={}, sealed={}",
            engine.context_word_is_known(), engine.completed_native_replay_awaits_exact_receipt(),
            engine.exact_manual_toggle_handoff_is_bound_to_current_owner(),
            engine.current_external_snapshot_agrees_with_owned_tail(), engine.context_handoff_sealed);
        assert_eq!(visible.1, "слово ");
        assert!(
            !visible.5.is_empty(),
            "manual replay remains bound to the current field"
        );
        assert!(engine.client_context.surrounding_text_snapshot.is_none());
        assert_eq!(
            engine.client_context.surrounding_observation_revision,
            revision
        );
        assert!(!engine.exact_managed_surrounding_word_is_current());
        assert!(!engine.context_has_native_replay_reset_ack());
        assert_no_stock_display_mutation(&terminal_delivery::legacy_effects(&mut harness).await);
    }));
}

#[test]
fn stock_observed_boundary_requires_successful_commit_and_current_word_observation() {
    use crate::output::TestEngineOutput;
    zbus::block_on(bounded(async {
        let (_, mut engine) = stock_observed_closed_word_fixture(false).await;
        let tail = engine.committed_tail.buffer.clone();
        let epoch = engine.committed_tail.epoch;
        let mut observed = TestEngineOutput {
            legacy_transport: true,
            fail_commit: true,
            ..Default::default()
        };
        assert!(engine
            .commit_managed_passthrough_char(&mut EngineOutput::test(&mut observed), ' ', 57)
            .await
            .is_err());
        assert_eq!(engine.committed_tail.buffer, tail);
        assert_eq!(engine.committed_tail.epoch, epoch);
        assert!(engine.committed_tail.observed_boundary_source.is_none());
        assert!(!engine.context_stock_owned_manual_source_is_current());
    }));
    zbus::block_on(bounded(async {
        let (mut harness, mut engine) = stock_observed_closed_word_fixture(true).await;
        for (i, (keyval, code)) in [('а' as u32, 30), (KEY_SPACE, 57)].into_iter().enumerate() {
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    166_300 + i as u32 * 2,
                    keyval,
                    code,
                    0
                )
                .await
            );
            assert!(
                legacy_key(
                    &mut harness,
                    &mut engine,
                    166_301 + i as u32 * 2,
                    keyval,
                    code,
                    RELEASE_MASK
                )
                .await
            );
        }
        assert!(engine.committed_tail.buffer.ends_with(' '));
        assert!(engine.context_word_is_known());
        assert!(engine.committed_tail.observed_boundary_source.is_none());
        assert!(!engine.context_stock_owned_manual_source_is_current());
        assert!(!engine.context_has_native_replay_reset_ack());
    }));
}

#[test]
fn stock_observed_local_boundary_retains_source_and_delivery_refusals() {
    for refusal in [
        "snapshot",
        "selection",
        "no-commit",
        "focus",
        "owner",
        "epoch",
        "revision",
        "next-letter",
        "exact-refresh",
        "sensitive",
        "sealed",
        "revoked",
        "replay",
    ] {
        zbus::block_on(bounded(async {
            let (mut harness, mut engine) = stock_observed_closed_word_fixture(true).await;
            assert!(engine.context_stock_owned_manual_source_is_current());
            match refusal {
                "snapshot" | "selection" => {
                    let anchor = if refusal == "selection" { 0 } else { 6 };
                    engine
                        .set_surrounding_text(
                            zbus::object_server::SignalEmitter::new(
                                &harness.connection,
                                engine.path.clone(),
                            )
                            .unwrap(),
                            crate::text::make_ibus_text("другое".into()),
                            6,
                            anchor,
                        )
                        .await
                        .unwrap();
                }
                "no-commit" => engine.committed_tail.last_commit_at = None,
                "focus" => engine.client_context.focus_serial += 1,
                "owner" => engine.client_context.runtime_owner_lease_identity += 1,
                "epoch" => engine.committed_tail.epoch += 1,
                "revision" => engine.advance_surrounding_observation_revision(),
                "next-letter" => {
                    assert!(
                        legacy_key(&mut harness, &mut engine, 166_200, 'а' as u32, 30, 0).await
                    );
                }
                "exact-refresh" => engine.client_context.exact_surrounding_refresh_available = true,
                "sensitive" => engine.set_content_type_state(
                    crate::window_interaction::IBUS_INPUT_PURPOSE_PASSWORD,
                    0,
                ),
                "sealed" => engine.context_handoff_sealed = true,
                "revoked" => engine.revoke_context_word(),
                "replay" => {
                    let (next, outcome, _) =
                        cycle09_manual_toggle_with_effects(&mut harness, engine).await;
                    engine = next;
                    assert_eq!(outcome, Ok((3, false)));
                    let (next, visible) = private_visible_tail_v4(&mut harness, engine).await;
                    engine = next;
                    assert_eq!(visible.unwrap().1, "слово ");
                    // The daemon has selected the target decoder before it
                    // arms the replay, as in the existing stock-source fixture.
                    engine.layout_gesture.layout_is_ru = false;
                    let epoch = engine.committed_tail.epoch;
                    let path = engine.path.clone();
                    let (next, suppression) = cycle09_suppress_exact_replay(
                        &mut harness,
                        engine,
                        "слово ",
                        epoch,
                        &path,
                        false,
                    )
                    .await;
                    engine = next;
                    assert_eq!(suppression, Ok(true));
                }
                _ => unreachable!(),
            }
            assert!(
                !engine.context_stock_owned_manual_source_is_current(),
                "{refusal}"
            );
            assert!(!engine.context_has_native_replay_reset_ack(), "{refusal}");
        }));
    }
    zbus::block_on(bounded(async {
        let (_, mut engine) = stock_observed_closed_word_fixture(false).await;
        // An unobserved last ordinary character is not a completed boundary.
        engine.client_context.surrounding_text_snapshot = None;
        assert!(!engine.context_stock_owned_manual_source_is_current());
    }));
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

async fn stock_pending_second_word_completion() -> (Harness, LayIbusEngine, String) {
    use crate::output::{EngineOutput, TestEngineOutput};
    use lay::typing_cpu::{ImeCandidateProposal, ImeCandidateSource};
    let (mut harness, mut engine) = stock_owned_manual_fixture().await;
    assert!(engine.context_word_is_known());
    assert!(
        legacy_key(
            &mut harness,
            &mut engine,
            165_000,
            KEY_SPACE,
            0,
            RELEASE_MASK
        )
        .await
    );
    engine
        .set_surrounding_text(
            zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                .unwrap(),
            crate::text::make_ibus_text("слово ".into()),
            6,
            6,
        )
        .await
        .unwrap();
    let mut observed = "слово ".to_string();
    for (i, (ch, code)) in [('п', 34), ('р', 35), ('о', 36), ('в', 32)]
        .into_iter()
        .enumerate()
    {
        let serial = 165_010 + i as u32 * 2;
        assert!(legacy_key(&mut harness, &mut engine, serial, ch as u32, code, 0).await);
        observed.push(ch);
        assert!(
            engine.composition.buffer.is_empty(),
            "literal managed commit expected"
        );
        if ch == 'в' {
            break;
        }
        engine
            .set_surrounding_text(
                zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                    .unwrap(),
                crate::text::make_ibus_text(observed.clone()),
                observed.chars().count() as u32,
                observed.chars().count() as u32,
            )
            .await
            .unwrap();
        assert!(
            legacy_key(
                &mut harness,
                &mut engine,
                serial + 1,
                ch as u32,
                code,
                RELEASE_MASK
            )
            .await
        );
    }
    let _ = terminal_delivery::legacy_effects(&mut harness).await;
    engine.cancel_precognition_display_generation();
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    assert!(!engine
        .apply_background_precognition(
            &mut EngineOutput::test(&mut output),
            vec![ImeCandidateProposal::new(
                "ерка",
                1.0,
                ImeCandidateSource::L2Completion
            )],
        )
        .await
        .unwrap());
    assert!(engine.composition.preedit_dirty);
    assert!(engine.selected_precognition_suffix().is_none());
    (harness, engine, observed)
}

fn assert_no_stock_display_mutation(effects: &[zbus::Message]) {
    assert!(!effects.iter().any(|message| matches!(
        message.header().member().map(|member| member.as_str()),
        Some("CommitText" | "DeleteSurroundingText" | "ForwardKeyEvent")
    )));
}

fn assert_retained_boundary_delivery(effects: &[zbus::Message], dispatched: bool) {
    assert_eq!(
        held_replay_boundary::forwarded_keys(effects),
        if dispatched {
            vec![(KEY_SPACE, 57, 0), (KEY_SPACE, 57, RELEASE_MASK)]
        } else {
            vec![]
        },
        "only one authorized final Space pair may be forwarded"
    );
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
}

#[test]
fn stock_deferred_completion_tracks_the_actual_second_word_receipt() {
    let _memory = td121_prime_completion_material();
    zbus::block_on(bounded(async {
        let (mut harness, mut engine, observed) = stock_pending_second_word_completion().await;
        let previous = engine.composition.pending_display_frame.clone();
        engine
            .set_surrounding_text(
                zbus::object_server::SignalEmitter::new(&harness.connection, engine.path.clone())
                    .unwrap(),
                crate::text::make_ibus_text(observed.clone()),
                observed.chars().count() as u32,
                observed.chars().count() as u32,
            )
            .await
            .unwrap();
        assert!(engine.exact_managed_surrounding_word_is_current());
        let current = engine.capture_input_frame_identity().unwrap();
        assert_ne!(
            previous.as_ref(),
            Some(&current),
            "the actual pair receipt advances display identity"
        );
        assert_eq!(
            engine.composition.pending_display_frame.as_ref(),
            Some(&current),
            "a fresh second-word receipt must preserve pending current display material"
        );
        assert!(engine.selected_precognition_suffix().is_none());
        assert_no_stock_display_mutation(&terminal_delivery::legacy_effects(&mut harness).await);
        td121_prepare_current_completion_material(&engine);
        let snapshot = engine.client_context.surrounding_text_snapshot.clone();
        let revision = engine.client_context.surrounding_observation_revision;
        let _ = td121_readout_key(
            &mut harness,
            &mut engine,
            165_030,
            'в' as u32,
            31,
            RELEASE_MASK,
        )
        .await;
        assert!(engine.layout_gesture.handled_press_keycodes.contains(&32));
        assert!(
            engine.selected_precognition_suffix().is_none(),
            "unrelated release cannot publish"
        );
        assert!(
            td121_readout_key(
                &mut harness,
                &mut engine,
                165_031,
                'в' as u32,
                32,
                RELEASE_MASK
            )
            .await
        );
        assert_eq!(
            engine.selected_precognition_suffix().as_deref(),
            Some("ерка")
        );
        assert!(engine.composition.preedit_visible || engine.composition.auxiliary_visible);
        assert!(!engine.composition.preedit_dirty);
        assert!(!engine.composition.preedit_display_only_pending);
        assert_eq!(engine.client_context.surrounding_text_snapshot, snapshot);
        assert_eq!(
            engine.client_context.surrounding_observation_revision,
            revision
        );
        assert!(engine.committed_tail.pending_completion_learning.is_none());
        assert_no_stock_display_mutation(&terminal_delivery::legacy_effects(&mut harness).await);
        engine.cancel_precognition_display_generation();
    }));
}

#[test]
fn stock_deferred_completion_does_not_refresh_from_inexact_receipts() {
    let _memory = td121_prime_completion_material();
    zbus::block_on(bounded(async {
        for (text, cursor, anchor) in [
            ("слово прон", 10, 10),
            ("слово пров", 9, 9),
            ("слово пров", 10, 9),
        ] {
            let (mut harness, mut engine, _) = stock_pending_second_word_completion().await;
            let previous = engine.composition.pending_display_frame.clone();
            engine
                .set_surrounding_text(
                    zbus::object_server::SignalEmitter::new(
                        &harness.connection,
                        engine.path.clone(),
                    )
                    .unwrap(),
                    crate::text::make_ibus_text(text.into()),
                    cursor,
                    anchor,
                )
                .await
                .unwrap();
            assert!(!engine.exact_managed_surrounding_word_is_current());
            assert!(
                engine.composition.pending_display_frame.is_none()
                    || engine.composition.pending_display_frame == previous,
                "an inexact receipt must not rebind pending display work"
            );
            assert!(engine.selected_precognition_suffix().is_none());
            assert_no_stock_display_mutation(
                &terminal_delivery::legacy_effects(&mut harness).await,
            );
            let _ = td121_readout_key(
                &mut harness,
                &mut engine,
                165_032,
                'в' as u32,
                32,
                RELEASE_MASK,
            )
            .await;
            // Existing KnownStart display may still be shown. This observation
            // must neither rebind its computation nor itself accept any text.
            assert!(engine.committed_tail.pending_completion_learning.is_none());
            assert_no_stock_display_mutation(
                &terminal_delivery::legacy_effects(&mut harness).await,
            );
            engine.cancel_precognition_display_generation();
        }
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

#[test]
fn stock_native_replay_never_injects_hybrid_final_literal() {
    for invalidation in [
        "live",
        "no-source",
        "no-prefix",
        "no-adapter",
        "revoked",
        "delivery-only",
        "expired",
        "pair-mismatch",
        "sensitive",
        "selection",
        "shared-path",
    ] {
        zbus::block_on(bounded(async {
            let (mut harness, source) = stock_observed_closed_word_in(
                cycle09_harness().await,
                true,
                false,
                &[('a', 30), ('b', 48), ('c', 46)],
                "文 🌍 ",
            )
            .await;
            let (source, outcome) = cycle09_manual_toggle(&mut harness, source).await;
            assert_eq!(outcome, Ok((3, false)));
            let (source, tail) = cycle09_visible_tail(&mut harness, source).await;
            let epoch = tail.unwrap().3;
            let path = format!("{TARGET_PATH}_stock_literal_provenance");
            let mut target =
                cycle09_factory_handoff(&mut harness, source, 169_000, &path, None, "lay-ime-ru")
                    .await;
            target.config.text_backend = "ime".into();
            target.config.typing_assist = false;
            target.config.nanda_precognition = false;
            let (mut target, armed) =
                cycle09_suppress_exact_replay(&mut harness, target, "abc ", epoch, &path, true)
                    .await;
            assert_eq!(armed, Ok(true));
            let mut serial = 169_100;
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
            let Some(AutocorrectSuppression::ExactReplay(mut scope)) =
                target.committed_tail.autocorrect_suppression.clone()
            else {
                panic!("missing final-key plan")
            };
            assert!(scope.source_token.is_some());
            assert_eq!(scope.observed_external_prefix.as_deref(), Some("文 🌍 "));
            match invalidation {
                "no-source" | "delivery-only" => scope.source_token = None,
                "no-prefix" => scope.observed_external_prefix = None,
                "no-adapter" => target.context_admission = None,
                "revoked" => assert!(target
                    .context_admission
                    .as_ref()
                    .unwrap()
                    .revoke_current_owner(target.context_owner.as_ref().unwrap())),
                "expired" => scope.expires_at = Instant::now() - Duration::from_secs(1),
                "sensitive" => target.client_context.content_purpose = 8,
                "selection" => {
                    target.client_context.surrounding_text_snapshot = Some(
                        crate::engine::SurroundingTextSnapshot::new("фис".into(), 3, 0),
                    );
                }
                "live" | "pair-mismatch" | "shared-path" => {}
                _ => unreachable!(),
            }
            target.committed_tail.autocorrect_suppression =
                Some(AutocorrectSuppression::ExactReplay(scope.clone()));
            if invalidation == "delivery-only" {
                scope.observed_external_prefix = None;
                target.committed_tail.autocorrect_suppression = None;
                target.context_owner = None;
            }
            let mut shared = target.shared.lock().unwrap();
            if invalidation == "pair-mismatch" {
                scope.target_layout_is_ru = !scope.target_layout_is_ru;
            }
            if invalidation == "shared-path" {
                shared.active_path = None;
            }
            shared.autocorrect_suppression = Some(AutocorrectSuppression::ExactReplay(scope));
            drop(shared);
            for state in [0, RELEASE_MASK] {
                let handled = firefox_observed_key_callback(
                    &mut harness,
                    &mut target,
                    &mut serial,
                    (KEY_SPACE, 57, state),
                )
                .await;
                let effects = terminal_delivery::legacy_effects(&mut harness).await;
                // Only the live paired boundary forwards one exact key pair.
                // Invalid scope or the original release must emit no keys;
                // no case may mix CommitText with pending native letters.
                if invalidation == "live" {
                    assert!(handled, "original boundary must be consumed state={state}");
                }
                assert_retained_boundary_delivery(&effects, invalidation == "live" && state == 0);
                assert!(
                    !target.context_has_native_replay_reset_ack(),
                    "{invalidation}"
                );
                assert_eq!(
                    target.held_replay_boundary_awaits_source(),
                    invalidation == "live",
                    "{invalidation}"
                );
                assert!(target
                    .completed_exact_replay_boundary_provenance()
                    .is_none());
            }
        }));
    }
}

#[test]
fn stock_held_native_replay_preserves_field_identity_without_text_authority() {
    for prefix in ["", "文 🌍 "] {
        for invalidation in [
            "live",
            "expired",
            "pair-mismatch",
            "no-provenance",
            "shared-path",
            "sensitive",
            "selection",
            "no-token",
        ] {
            zbus::block_on(bounded(async {
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
                let path = format!("{TARGET_PATH}_stock_completed_pending_receipt");
                let mut target = cycle09_factory_handoff(
                    &mut harness,
                    source,
                    168_000,
                    &path,
                    None,
                    "lay-ime-ru",
                )
                .await;
                target.config.text_backend = "ime".into();
                target.config.typing_assist = false;
                target.config.nanda_precognition = false;
                let (mut target, armed) =
                    cycle09_suppress_exact_replay(&mut harness, target, "abc ", epoch, &path, true)
                        .await;
                assert_eq!(armed, Ok(true));
                let Some(AutocorrectSuppression::ExactReplay(scope)) =
                    target.committed_tail.autocorrect_suppression.as_ref()
                else {
                    panic!("missing paired replay");
                };
                assert!(
                    scope.source_token.is_some(),
                    "missing original reducer source"
                );
                assert_eq!(scope.observed_external_prefix.as_deref(), Some(prefix));
                let mut serial = 168_100;
                let mut keys = vec![(KEY_BACKSPACE, 14); 4];
                keys.extend([
                    (0x0100_0000 | 'ф' as u32, 30),
                    (0x0100_0000 | 'и' as u32, 48),
                    (0x0100_0000 | 'с' as u32, 46),
                    (KEY_SPACE, 57),
                ]);
                for (index, (keyval, keycode)) in keys.into_iter().enumerate() {
                    // Native client order: an old deletion prefix arrives after
                    // all deletes; empty text and a partial target then lag replay.
                    let actual = match index {
                        4 => Some(format!("{prefix}a")),
                        5 => Some(prefix.to_owned()),
                        7 => Some(format!("{prefix}фи")),
                        _ => None,
                    };
                    if let Some(actual) = actual {
                        let count = actual.chars().count() as u32;
                        cycle09_surrounding_receipt(
                            &mut harness,
                            &mut target,
                            &actual,
                            count,
                            count,
                        )
                        .await;
                        assert_no_stock_display_mutation(
                            &terminal_delivery::legacy_effects(&mut harness).await,
                        );
                    }
                    for state in [0, RELEASE_MASK] {
                        let handled = firefox_observed_key_callback(
                            &mut harness,
                            &mut target,
                            &mut serial,
                            (keyval, keycode, state),
                        )
                        .await;
                        assert_eq!(
                            handled,
                            keyval == KEY_SPACE,
                            "only the original final boundary is consumed: index={index} state={state}"
                        );
                        let effects = terminal_delivery::legacy_effects(&mut harness).await;
                        assert_retained_boundary_delivery(
                            &effects,
                            keyval == KEY_SPACE && state == 0,
                        );
                        if keyval == KEY_SPACE {
                            assert_eq!(
                                effects.iter().filter(|effect| effect.header().member()
                                    .is_some_and(|member| member.as_str() == "RequireSurroundingText")
                                ).count(),
                                usize::from(state == 0),
                                "one surrounding request at held entry: index={index} state={state}",
                            );
                        }
                        assert!(
                            target.exact_replay_quarantine_active(),
                            "lost replay index={index} state={state} tail={:?} epoch={} local={:?}",
                            target.committed_tail.buffer,
                            target.committed_tail.epoch,
                            target.committed_tail.autocorrect_suppression
                        );
                    }
                }
                assert_eq!(target.committed_tail.buffer, "фис");
                assert!(target.held_replay_boundary_awaits_source());
                assert!(target
                    .completed_exact_replay_boundary_provenance()
                    .is_none());
                assert!(!target.context_has_native_replay_reset_ack());
                let expected_field = target.context_token.as_ref().unwrap().exact_field_receipt();
                match invalidation {
                    "live" => {}
                    "expired" | "no-provenance" => {
                        let Some(AutocorrectSuppression::ExactReplay(scope)) =
                            target.committed_tail.autocorrect_suppression.as_mut()
                        else {
                            panic!("missing held replay")
                        };
                        if invalidation == "expired" {
                            scope.expires_at = Instant::now() - Duration::from_secs(1);
                        } else {
                            scope.source_token = None;
                        }
                        target.shared.lock().unwrap().autocorrect_suppression =
                            target.committed_tail.autocorrect_suppression.clone();
                    }
                    "pair-mismatch" => {
                        let mut shared = target.shared.lock().unwrap();
                        let Some(AutocorrectSuppression::ExactReplay(scope)) =
                            shared.autocorrect_suppression.as_mut()
                        else {
                            panic!("missing shared completed replay")
                        };
                        scope.target_layout_is_ru = !scope.target_layout_is_ru;
                    }
                    "shared-path" => target.shared.lock().unwrap().active_path = None,
                    "sensitive" => target.client_context.content_purpose = 8,
                    "selection" => {
                        let actual = format!("{prefix}фис");
                        let count = actual.chars().count() as u32;
                        cycle09_surrounding_receipt(&mut harness, &mut target, &actual, count, 0)
                            .await;
                    }
                    "no-token" => target.context_token = None,
                    _ => unreachable!(),
                }
                let (mut target, reply) = private_visible_tail_v4(&mut harness, target).await;
                let reply = reply.unwrap();
                assert_eq!(reply.0, "passive:unknown-context", "{invalidation}");
                assert!(reply.1.is_empty(), "mirrored replay is not a text receipt");
                assert_eq!(reply.4, path);
                if invalidation == "live" {
                    // The existing daemon lease binds this actual policy bit.
                    // Empty text with unchanged privacy is not a learning ACK.
                    assert_eq!(
                        reply.6,
                        target.content_allows_personalization(),
                        "pending settlement must not fabricate a privacy-policy change"
                    );
                    assert!(target.committed_tail.pending_completion_learning.is_none());
                    assert_eq!(reply.5, expected_field,
                    "pending exact settlement must retain the proved field identity without exposing a text source: {reply:?}");
                    assert!(!reply.5.is_empty());
                    assert!(target.held_replay_boundary_awaits_source());
                    // Actual whole-word observation permits the one retained
                    // Space. It publishes source evidence, never a result ACK.
                    let actual = format!("{prefix}фис");
                    let count = actual.chars().count() as u32;
                    target
                        .set_surrounding_text(
                            zbus::object_server::SignalEmitter::new(
                                &harness.connection,
                                target.path.clone(),
                            )
                            .unwrap(),
                            crate::text::make_ibus_text(actual.clone()),
                            count,
                            count,
                        )
                        .await
                        .unwrap();
                    let effects = terminal_delivery::legacy_effects(&mut harness).await;
                    assert_no_stock_display_mutation(&effects);
                    assert_eq!(target.committed_tail.buffer, "фис ");
                    assert!(target.context_stock_owned_manual_source_is_current());
                    assert!(!target.has_held_replay_boundary());
                    assert!(target
                        .committed_tail
                        .pending_visible_postcondition
                        .is_none());
                    assert!(target.committed_tail.pending_completion_learning.is_none());
                    assert!(!target.context_has_native_replay_reset_ack());
                    target
                        .set_surrounding_text(
                            zbus::object_server::SignalEmitter::new(
                                &harness.connection,
                                target.path.clone(),
                            )
                            .unwrap(),
                            crate::text::make_ibus_text(actual),
                            count,
                            count,
                        )
                        .await
                        .unwrap();
                    let duplicate = terminal_delivery::legacy_effects(&mut harness).await;
                    assert!(
                        !duplicate.iter().any(|effect| effect
                            .header()
                            .member()
                            .is_some_and(|member| member.as_str() == "RequireSurroundingText")),
                        "identical actual receipts must not cause a request loop"
                    );
                    assert_no_stock_display_mutation(&duplicate);
                    let actual_final = format!("{prefix}фис ");
                    let count = actual_final.chars().count() as u32;
                    cycle09_surrounding_receipt(
                        &mut harness,
                        &mut target,
                        &actual_final,
                        count,
                        count,
                    )
                    .await;
                    let (next, settled) = private_visible_tail_v4(&mut harness, target).await;
                    target = next;
                    let settled = settled.unwrap();
                    assert_eq!(
                        settled.0,
                        lay::text_edit::VisibleTailSource::ImeCommittedTail.bridge_state()
                    );
                    assert_eq!(settled.1, "фис ");
                    assert_eq!(settled.5, expected_field);
                } else {
                    assert!(!reply.6, "unproved context must not permit retention");
                    assert!(reply.5.is_empty(), "{invalidation}: {reply:?}");
                    assert!(target
                        .completed_exact_replay_boundary_provenance()
                        .is_none());
                    let actual = format!("{prefix}фис");
                    let count = actual.chars().count() as u32;
                    cycle09_surrounding_receipt(&mut harness, &mut target, &actual, count, count)
                        .await;
                    assert_no_stock_display_mutation(
                        &terminal_delivery::legacy_effects(&mut harness).await,
                    );
                    assert!(
                        !target.context_stock_owned_manual_source_is_current(),
                        "{invalidation}"
                    );
                }
                if invalidation != "live" {
                    assert!(!target.context_has_native_replay_reset_ack());
                }
            }));
        }
    }
}
