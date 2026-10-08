use std::sync::{Arc, Mutex};

use crate::engine::{LayIbusEngine, SurroundingTextSnapshot, WordInputMode};
use crate::output::{EngineOutput, TestEngineOutput};
use crate::protocol::KEY_SPACE;
use crate::space_autocorrect_prefetch::proof::install_full_lease;
use crate::window_interaction::{
    IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH, IBUS_CAP_PREEDIT_TEXT, IBUS_CAP_SURROUNDING_TEXT,
};
use lay::config::LayConfig;

#[path = "../../../tests/common/space_boundary_lexical.rs"]
mod lexical_fixture;

fn isolated_case(name: &str) -> bool {
    if std::env::var_os("LAY_SPACE_PAIR_TEST_CHILD").is_some() {
        return true;
    }
    // Canonical admission is immutable. Other binary tests may already have
    // admitted an unavailable provider; never reset it or leak fixture state.
    let fixture = std::env::temp_dir().join(format!("lay-pair-child-{}.bin", std::process::id()));
    std::fs::write(
        &fixture,
        include_bytes!("../../../tests/fixtures/space_boundary_lexical_v2.bin"),
    )
    .expect("lexical fixture");
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env("LAY_SPACE_PAIR_TEST_CHILD", "1")
        .env("LAY_L2_PACKAGE", &fixture)
        .output()
        .expect("isolated production-owner test");
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    false
}

fn engine(caps: u32, committed: &str) -> LayIbusEngine {
    lay::hot_field::set_process_policy(lay::hot_field::HotFieldPolicy::ime());
    lexical_fixture::warm_lexical_fixture();
    let mut engine = LayIbusEngine::new(
        "/space-boundary-pair/proof".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            auto_replace: true,
            typing_assist: true,
            nanda_autocorrect: true,
            correction_safety: "normal".to_string(),
            ..LayConfig::default()
        },
    );
    assert!(engine.bind_focus_path());
    engine.set_client_capabilities(caps | (1 << 3));
    for ch in committed.chars() {
        engine.push_tail_char(ch);
    }
    // push_tail_char retires the mode at a separator; this fixture represents
    // a fully committed right word, while owned_pair exercises real key input.
    engine.composition.word_input_mode = Some(WordInputMode::ManagedCommit);
    let cursor = committed.chars().count() as u32;
    engine.observe_external_surrounding_text(Some(SurroundingTextSnapshot::new(
        committed.to_string(),
        cursor,
        cursor,
    )));
    engine
}

fn press(
    engine: &mut LayIbusEngine,
    output: &mut TestEngineOutput,
    key: u32,
) -> zbus::fdo::Result<bool> {
    zbus::block_on(engine.process_pressed_key(&mut EngineOutput::test(output), key, 0, 0))
}

fn owned_pair(exact_refresh: bool) -> (LayIbusEngine, TestEngineOutput) {
    let caps = IBUS_CAP_PREEDIT_TEXT
        | IBUS_CAP_SURROUNDING_TEXT
        | if exact_refresh {
            IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH
        } else {
            0
        };
    let mut engine = engine(caps, "должн ");
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    for ch in "ыбыть".chars() {
        assert!(press(&mut engine, &mut output, 0x0100_0000 | ch as u32).unwrap());
    }
    assert_eq!(engine.composition.buffer, "ыбыть");
    assert!(engine.composition.legacy_word_preedit_active);
    (engine, output)
}

fn observe_space_commit(
    engine: &mut LayIbusEngine,
    output: &mut TestEngineOutput,
) -> zbus::fdo::Result<()> {
    observe_client(
        engine,
        output,
        SurroundingTextSnapshot::new("должн ыбыть ".into(), 12, 12),
    )
}

fn observe_client(
    engine: &mut LayIbusEngine,
    output: &mut TestEngineOutput,
    snapshot: SurroundingTextSnapshot,
) -> zbus::fdo::Result<()> {
    zbus::block_on(crate::window_interaction::WindowInteraction::observe_facts(
        engine,
        crate::window_interaction::WindowFactEvent::SurroundingText(Some(snapshot)),
        Some(&mut EngineOutput::test(output)),
    ))
    .map(|_| ())
}

fn install(engine: &LayIbusEngine) {
    let frame = engine
        .capture_space_autocorrect_frame_identity()
        .expect("Space frame");
    assert_eq!(
        frame
            .space_boundary_pair
            .as_ref()
            .expect("exact pair scope")
            .text,
        "должн ыбыть"
    );
    install_full_lease(&frame, &engine.config);
}

#[test]
fn space_pair_pending_calculation_and_actual_commit_join_once_in_either_order() {
    if !isolated_case("space_boundary_pair_tests::space_pair_pending_calculation_and_actual_commit_join_once_in_either_order") { return; }
    use crate::engine::PendingSpaceBoundarySelection;
    use crate::space_autocorrect_prefetch::proof::{hold_full_lease, publish_held_full_lease};

    for snapshot_first in [false, true] {
        let (mut engine, mut output) = owned_pair(false);
        let frame = engine.capture_space_autocorrect_frame_identity().unwrap();
        let lease = hold_full_lease(&frame, &engine.config);
        let lookup = engine.take_space_autocorrect_lease(&frame);
        assert!(matches!(
            lookup.lookup,
            crate::space_autocorrect_prefetch::SpaceAutocorrectLookup::NotReady
        ));
        assert!(crate::space_autocorrect_prefetch::proof::has_current_slot(
            &frame
        ));
        let pending = engine
            .prepare_space_boundary_continuation(
                &frame,
                PendingSpaceBoundarySelection::Computing {
                    identity: Box::new(frame.clone()),
                    worker_generation: lease.worker_generation,
                },
            )
            .unwrap();
        zbus::block_on(engine.commit_legacy_space_boundary_continuation(
            &mut EngineOutput::test(&mut output),
            pending,
        ))
        .unwrap();
        assert_eq!(output.committed_texts.last().unwrap(), "ыбыть ");
        assert!(output.surrounding_deletes.is_empty());
        if snapshot_first {
            observe_space_commit(&mut engine, &mut output).unwrap();
        }
        assert!(output.surrounding_deletes.is_empty());
        assert!(publish_held_full_lease(lease));
        let handled = zbus::block_on(
            engine.apply_space_boundary_after_client_commit(&mut EngineOutput::test(&mut output)),
        )
        .unwrap();
        assert_eq!(handled, snapshot_first);
        if !snapshot_first {
            observe_space_commit(&mut engine, &mut output).unwrap();
        }
        assert_eq!(output.surrounding_deletes, [(-12, 12)]);
        assert_eq!(engine.committed_tail.buffer, "должны быть ");
        observe_space_commit(&mut engine, &mut output).unwrap();
        assert!(!zbus::block_on(
            engine.apply_space_boundary_after_client_commit(&mut EngineOutput::test(&mut output))
        )
        .unwrap());
        assert_eq!(output.surrounding_deletes.len(), 1);
    }

    for result in ["no_apply", "unsealed", "exact_lane"] {
        let (mut engine, mut output) = owned_pair(false);
        let frame = engine.capture_space_autocorrect_frame_identity().unwrap();
        let mut lease = hold_full_lease(&frame, &engine.config);
        let pending = engine
            .prepare_space_boundary_continuation(
                &frame,
                PendingSpaceBoundarySelection::Computing {
                    identity: Box::new(frame.clone()),
                    worker_generation: lease.worker_generation,
                },
            )
            .unwrap();
        zbus::block_on(engine.commit_legacy_space_boundary_continuation(
            &mut EngineOutput::test(&mut output),
            pending,
        ))
        .unwrap();
        observe_space_commit(&mut engine, &mut output).unwrap();
        match result {
            "no_apply" => {
                assert!(crate::space_autocorrect_prefetch::proof::publish_held_no_apply(lease))
            }
            "unsealed" => {
                lease.decision.action = lay::text_edit::EditAction::keep("proof_refusal", "ыбыть");
                assert!(publish_held_full_lease(lease));
            }
            "exact_lane" => {
                lease.kind = crate::space_autocorrect_prefetch::PreparedLeaseKind::ExactLayout;
                assert!(publish_held_full_lease(lease));
            }
            _ => unreachable!(),
        }
        assert!(!zbus::block_on(
            engine.apply_space_boundary_after_client_commit(&mut EngineOutput::test(&mut output))
        )
        .unwrap());
        observe_space_commit(&mut engine, &mut output).unwrap();
        assert!(output.surrounding_deletes.is_empty(), "refuse {result}");
        assert_eq!(engine.committed_tail.buffer, "должн ыбыть ");
    }

    for violation in ["next_key", "focus", "material", "expired", "text"] {
        let (mut engine, mut output) = owned_pair(false);
        let frame = engine.capture_space_autocorrect_frame_identity().unwrap();
        let lease = hold_full_lease(&frame, &engine.config);
        let pending = engine
            .prepare_space_boundary_continuation(
                &frame,
                PendingSpaceBoundarySelection::Computing {
                    identity: Box::new(frame.clone()),
                    worker_generation: lease.worker_generation,
                },
            )
            .unwrap();
        zbus::block_on(engine.commit_legacy_space_boundary_continuation(
            &mut EngineOutput::test(&mut output),
            pending,
        ))
        .unwrap();
        match violation {
            "next_key" => {
                press(&mut engine, &mut output, crate::protocol::KEY_LEFT).unwrap();
            }
            "focus" => engine.client_context.focus_serial += 1,
            "material" => {
                engine
                    .committed_tail
                    .pending_visible_postcondition
                    .as_mut()
                    .unwrap()
                    .space_boundary_commit
                    .as_mut()
                    .unwrap()
                    .material_generation += 1
            }
            "expired" => {
                engine
                    .committed_tail
                    .pending_visible_postcondition
                    .as_mut()
                    .unwrap()
                    .dispatched_at -= std::time::Duration::from_secs(2)
            }
            "text" => engine.observe_external_surrounding_text(Some(SurroundingTextSnapshot::new(
                "чужое слово ".into(),
                12,
                12,
            ))),
            _ => unreachable!(),
        }
        zbus::block_on(
            engine.apply_space_boundary_after_client_commit(&mut EngineOutput::test(&mut output)),
        )
        .unwrap();
        assert!(
            !publish_held_full_lease(lease),
            "revoked producer: {violation}"
        );
        observe_space_commit(&mut engine, &mut output).unwrap();
        assert!(output.surrounding_deletes.is_empty(), "refuse {violation}");
    }
}

#[test]
fn space_pair_owned_preedit_deletes_only_client_prefix_and_commits_once() {
    if !isolated_case("space_boundary_pair_tests::space_pair_owned_preedit_deletes_only_client_prefix_and_commits_once") { return; }
    for exact in [false, true] {
        let (mut engine, mut output) = owned_pair(exact);
        install(&engine);
        assert!(press(&mut engine, &mut output, KEY_SPACE).unwrap());
        assert!(
            output.surrounding_deletes.is_empty(),
            "composition must finish before deleting any client text"
        );
        assert_eq!(output.committed_texts, ["ыбыть "]);
        observe_space_commit(&mut engine, &mut output).unwrap();
        assert_eq!(output.surrounding_deletes, [(-12, 12)]);
        assert_eq!(output.committed_texts, ["ыбыть ", "должны быть "]);
        assert_eq!(engine.committed_tail.buffer, "должны быть ");
        assert!(engine.composition.buffer.is_empty());
        assert!(!engine.composition.legacy_word_preedit_active);
        let pending = engine.shared.lock().unwrap();
        let undo = pending
            .pending_auto_undo
            .as_ref()
            .expect("paired undo record");
        assert_eq!(undo.original, "должн ыбыть ");
        assert!(undo.boundary_forward_action.is_some());
    }
}

#[test]
fn space_pair_committed_client_deletes_exact_full_pair() {
    if !isolated_case(
        "space_boundary_pair_tests::space_pair_committed_client_deletes_exact_full_pair",
    ) {
        return;
    }
    let mut engine = engine(
        IBUS_CAP_SURROUNDING_TEXT | IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH,
        "должн ыбыть",
    );
    install(&engine);
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    assert!(press(&mut engine, &mut output, KEY_SPACE).unwrap());
    assert_eq!(output.surrounding_deletes, [(-11, 11)]);
    assert_eq!(output.committed_texts, ["должны быть "]);
    assert_eq!(engine.committed_tail.buffer, "должны быть ");
}

#[test]
fn space_pair_stale_preedit_witness_refuses_the_prior_text_edit() {
    if !isolated_case(
        "space_boundary_pair_tests::space_pair_stale_preedit_witness_refuses_the_prior_text_edit",
    ) {
        return;
    }
    let (mut engine, mut output) = owned_pair(false);
    install(&engine);
    engine.advance_surrounding_observation_revision();
    assert!(press(&mut engine, &mut output, KEY_SPACE).unwrap());
    assert!(output.surrounding_deletes.is_empty());
    assert_eq!(output.committed_texts, ["ыбыть "]);
    assert_eq!(engine.committed_tail.buffer, "должн ыбыть ");
}

#[test]
fn space_pair_partial_commit_failure_never_falls_back_or_arms_undo() {
    if !isolated_case("space_boundary_pair_tests::space_pair_partial_commit_failure_never_falls_back_or_arms_undo") { return; }
    let (mut engine, mut output) = owned_pair(false);
    install(&engine);
    assert!(press(&mut engine, &mut output, KEY_SPACE).unwrap());
    output.fail_commit = true;
    assert!(observe_space_commit(&mut engine, &mut output).is_err());
    assert_eq!(output.surrounding_deletes, [(-12, 12)]);
    assert_eq!(output.committed_texts, ["ыбыть ", "должны быть "]);
    output.fail_commit = false;
    observe_space_commit(&mut engine, &mut output).unwrap();
    assert_eq!(
        output.surrounding_deletes.len(),
        1,
        "partial output consumes the continuation"
    );
    assert!(engine.shared.lock().unwrap().pending_auto_undo.is_none());
}

#[test]
fn space_pair_owned_and_committed_corrections_undo_the_exact_original_pair() {
    if !isolated_case("space_boundary_pair_tests::space_pair_owned_and_committed_corrections_undo_the_exact_original_pair") { return; }
    for owned in [false, true] {
        let (mut engine, mut output) = if owned {
            owned_pair(false)
        } else {
            (
                engine(
                    IBUS_CAP_SURROUNDING_TEXT | IBUS_CAP_LAY_EXACT_SURROUNDING_REFRESH,
                    "должн ыбыть",
                ),
                TestEngineOutput {
                    legacy_transport: true,
                    ..Default::default()
                },
            )
        };
        install(&engine);
        assert!(press(&mut engine, &mut output, KEY_SPACE).unwrap());
        if owned {
            observe_space_commit(&mut engine, &mut output).unwrap();
        }
        engine.observe_external_surrounding_text(Some(SurroundingTextSnapshot::new(
            "должны быть ".to_string(),
            12,
            12,
        )));
        engine.observe_visible_postcondition();
        let handled =
            zbus::block_on(engine.undo_last_ime_autocorrect(&mut EngineOutput::test(&mut output)))
                .expect("existing undo owner");
        assert_eq!(handled, Some(true));
        assert_eq!(output.surrounding_deletes.last(), Some(&(-12, 12)));
        assert_eq!(
            output.committed_texts.last().map(String::as_str),
            Some("должн ыбыть ")
        );
        assert_eq!(engine.committed_tail.buffer, "должн ыбыть ");
    }
}

fn native_engine(tail: &str, prove_boundary: bool) -> (LayIbusEngine, zbus::Connection) {
    use crate::context_admission::{ActivationOutcome, ContextAdmissionAdapter, WordCompleteness};
    lay::hot_field::set_process_policy(lay::hot_field::HotFieldPolicy::ime());
    lexical_fixture::warm_lexical_fixture();
    let path = "/space-boundary-pair/native";
    let (adapter, grant, peer) = ContextAdmissionAdapter::test_established(
        path,
        "/org/freedesktop/IBus/InputContext_pair",
        WordCompleteness::KnownStart,
        17,
    );
    let mut engine = LayIbusEngine::new_from_component(
        path.to_string(),
        Arc::new(Mutex::new(Default::default())),
        Some(adapter.clone()),
        "lay-ime-ru",
        true,
        LayConfig {
            auto_replace: true,
            typing_assist: true,
            nanda_autocorrect: true,
            text_backend: "ime".to_string(),
            ..LayConfig::default()
        },
    );
    assert!(engine.install_context_activation(ActivationOutcome::SourceFree(grant)));
    engine.set_content_type_state(10, 0);
    engine.client_context.cursor_cell_width = 8;
    engine.composition.word_input_mode = Some(WordInputMode::TerminalPassthrough);
    engine.committed_tail.buffer = tail.to_string();
    let mut scope = *engine.context_word_scope.as_ref().unwrap();
    if prove_boundary {
        scope.close_at_observed_boundary(17, Some(0));
    }
    assert!(adapter.test_set_word_scope(
        engine.context_owner.as_ref().unwrap(),
        engine.committed_tail.epoch,
        &scope
    ));
    engine.context_word_scope = Some(scope);
    engine.context_token = adapter.current_token();
    (engine, peer)
}

#[test]
fn space_pair_native_observed_scope_uses_one_terminal_frame_and_exact_undo() {
    if !isolated_case("space_boundary_pair_tests::space_pair_native_observed_scope_uses_one_terminal_frame_and_exact_undo") { return; }
    let (mut engine, _peer) = native_engine(" должн ыбыть", true);
    install(&engine);
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    assert!(press(&mut engine, &mut output, KEY_SPACE).unwrap());
    assert!(output.surrounding_deletes.is_empty());
    assert_eq!(
        output.committed_texts,
        [format!("{}должны быть ", "\u{7f}".repeat(11))]
    );
    assert_eq!(engine.committed_tail.buffer, " должны быть ");
    assert_eq!(
        zbus::block_on(engine.undo_last_ime_autocorrect(&mut EngineOutput::test(&mut output)))
            .unwrap(),
        Some(true)
    );
    assert_eq!(
        output.committed_texts.last(),
        Some(&format!("{}должн ыбыть ", "\u{7f}".repeat(12)))
    );
    assert_eq!(engine.committed_tail.buffer, " должн ыбыть ");
}

#[test]
fn space_pair_native_mirror_cannot_manufacture_an_earlier_word_start() {
    if !isolated_case("space_boundary_pair_tests::space_pair_native_mirror_cannot_manufacture_an_earlier_word_start") { return; }
    for (tail, observed) in [
        ("должн ыбыть", false),
        ("должн ыбыть", true),
        (" должн ыбыть", false),
    ] {
        let (engine, _peer) = native_engine(tail, observed);
        assert!(
            engine.capture_space_boundary_pair_scope().is_none(),
            "unproved prefix: {tail:?}"
        );
    }
}

#[test]
fn space_pair_native_atomic_output_has_exactly_one_edit_frame() {
    if !isolated_case(
        "space_boundary_pair_tests::space_pair_native_atomic_output_has_exactly_one_edit_frame",
    ) {
        return;
    }
    let (mut engine, _peer) = native_engine(" должн ыбыть", true);
    install(&engine);
    let mut builder = crate::output::AtomicEffectBuilder::default();
    let handled = zbus::block_on(engine.process_pressed_key(
        &mut EngineOutput::atomic(&mut builder),
        KEY_SPACE,
        0,
        0,
    ))
    .unwrap();
    let (status, effects) = builder.finish(handled);
    assert_eq!(status, crate::output::PROPOSAL_FRAME_READY);
    assert!(!effects.iter().any(|(tag, _)| *tag == 2));
    let commits = effects
        .into_iter()
        .filter(|(tag, _)| *tag == 1)
        .map(|(_, value)| String::try_from(value).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(commits, [format!("{}должны быть ", "\u{7f}".repeat(11))]);
    assert_eq!(engine.committed_tail.buffer, " должны быть ");
}

#[test]
fn space_pair_partial_inverse_consumes_the_record_without_retry() {
    if !isolated_case(
        "space_boundary_pair_tests::space_pair_partial_inverse_consumes_the_record_without_retry",
    ) {
        return;
    }
    let (mut engine, mut output) = owned_pair(false);
    install(&engine);
    assert!(press(&mut engine, &mut output, KEY_SPACE).unwrap());
    observe_space_commit(&mut engine, &mut output).unwrap();
    engine.observe_external_surrounding_text(Some(SurroundingTextSnapshot::new(
        "должны быть ".to_string(),
        12,
        12,
    )));
    engine.observe_visible_postcondition();
    output.fail_commit = true;
    assert!(
        zbus::block_on(engine.undo_last_ime_autocorrect(&mut EngineOutput::test(&mut output)))
            .is_err()
    );
    assert_eq!(output.surrounding_deletes, [(-12, 12), (-12, 12)]);
    assert!(engine.shared.lock().unwrap().pending_auto_undo.is_none());
    assert_eq!(
        zbus::block_on(engine.undo_last_ime_autocorrect(&mut EngineOutput::test(&mut output)))
            .unwrap(),
        None
    );
    assert_eq!(output.surrounding_deletes.len(), 2);
}

#[test]
fn space_pair_both_spaces_preserve_left_then_apply_the_verified_pair() {
    if !isolated_case("space_boundary_pair_tests::space_pair_both_spaces_preserve_left_then_apply_the_verified_pair") { return; }
    let mut engine = engine(IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_SURROUNDING_TEXT, "");
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    for ch in "должн".chars() {
        assert!(press(&mut engine, &mut output, 0x0100_0000 | ch as u32).unwrap());
    }
    // The first boundary has no editable pair and cannot transfer a letter.
    assert!(engine.capture_space_boundary_pair_scope().is_none());
    assert!(press(&mut engine, &mut output, KEY_SPACE).unwrap());
    assert_eq!(engine.committed_tail.buffer, "должн ");
    assert_eq!(output.committed_texts, ["должн "]);
    engine.observe_external_surrounding_text(Some(SurroundingTextSnapshot::new(
        "должн ".to_string(),
        6,
        6,
    )));
    for ch in "ыбыть".chars() {
        assert!(press(&mut engine, &mut output, 0x0100_0000 | ch as u32).unwrap());
    }
    install(&engine);
    assert!(press(&mut engine, &mut output, KEY_SPACE).unwrap());
    assert_eq!(output.committed_texts, ["должн ", "ыбыть "]);
    assert!(output.surrounding_deletes.is_empty());
    observe_space_commit(&mut engine, &mut output).unwrap();
    assert_eq!(output.committed_texts, ["должн ", "ыбыть ", "должны быть "]);
    assert_eq!(output.surrounding_deletes, [(-12, 12)]);
    assert_eq!(engine.committed_tail.buffer, "должны быть ");
}

#[test]
fn space_pair_client_commit_continuation_is_exact_one_shot_and_revocable() {
    if !isolated_case("space_boundary_pair_tests::space_pair_client_commit_continuation_is_exact_one_shot_and_revocable") { return; }
    let (mut engine, mut output) = owned_pair(false);
    install(&engine);
    assert!(press(&mut engine, &mut output, KEY_SPACE).unwrap());
    assert!(
        output.surrounding_deletes.is_empty(),
        "controlled old synchronous dispatch would fail this assertion"
    );
    assert!(engine.shared.lock().unwrap().pending_auto_undo.is_none());
    observe_client(
        &mut engine,
        &mut output,
        SurroundingTextSnapshot::new("должн ".into(), 6, 6),
    )
    .unwrap();
    assert!(
        output.surrounding_deletes.is_empty(),
        "partial pre-commit receipt grants no delete"
    );
    observe_space_commit(&mut engine, &mut output).unwrap();
    observe_space_commit(&mut engine, &mut output).unwrap();
    assert_eq!(
        output.surrounding_deletes,
        [(-12, 12)],
        "duplicate callback cannot repeat correction"
    );

    // Missing Qt post-correction readout may use the actual full original
    // receipt causally in the unchanged lease, never a fabricated snapshot.
    for violation in ["none", "owner", "focus", "contradictory"] {
        let (mut engine, mut output) = owned_pair(false);
        install(&engine);
        assert!(press(&mut engine, &mut output, KEY_SPACE).unwrap());
        observe_space_commit(&mut engine, &mut output).unwrap();
        assert!(engine.client_context.surrounding_text_snapshot.is_none());
        match violation {
            "owner" => engine.client_context.runtime_owner_lease_identity += 1,
            "focus" => engine.client_context.focus_serial += 1,
            "contradictory" => engine.observe_external_surrounding_text(Some(
                SurroundingTextSnapshot::new("чужое слово ".into(), 12, 12),
            )),
            _ => {}
        }
        if violation == "none" {
            assert!(engine.pending_ime_auto_undo_uses_causal_precondition_snapshot());
            assert_eq!(
                zbus::block_on(
                    engine.undo_last_ime_autocorrect(&mut EngineOutput::test(&mut output))
                )
                .unwrap(),
                Some(true)
            );
            assert_eq!(engine.committed_tail.buffer, "должн ыбыть ");
            assert_eq!(output.surrounding_deletes, [(-12, 12), (-12, 12)]);
        } else {
            assert!(
                !engine.pending_ime_auto_undo_uses_causal_precondition_snapshot(),
                "refuse stale causal receipt: {violation}"
            );
            assert!(engine.defer_pending_ime_auto_undo_until_visible());
            assert_eq!(output.surrounding_deletes, [(-12, 12)]);
        }
    }

    for violation in [
        "selection",
        "caret",
        "text",
        "focus",
        "owner",
        "config",
        "material",
        "expired",
        "next_key",
        "closed",
    ] {
        let (mut engine, mut output) = owned_pair(false);
        install(&engine);
        assert!(press(&mut engine, &mut output, KEY_SPACE).unwrap());
        let mut snapshot = SurroundingTextSnapshot::new("должн ыбыть ".into(), 12, 12);
        match violation {
            "selection" => snapshot.anchor_pos = 0,
            "caret" => {
                snapshot.cursor_pos = 11;
                snapshot.anchor_pos = 11;
            }
            "text" => snapshot.text = "чужое слово ".into(),
            "focus" => engine.client_context.focus_serial += 1,
            "owner" => engine.client_context.runtime_owner_lease_identity += 1,
            "config" => engine.config.auto_replace = false,
            "material" => {
                engine
                    .committed_tail
                    .pending_visible_postcondition
                    .as_mut()
                    .unwrap()
                    .space_boundary_commit
                    .as_mut()
                    .unwrap()
                    .material_generation += 1
            }
            "expired" => {
                engine
                    .committed_tail
                    .pending_visible_postcondition
                    .as_mut()
                    .unwrap()
                    .dispatched_at -= std::time::Duration::from_secs(2)
            }
            "next_key" => {
                press(&mut engine, &mut output, crate::protocol::KEY_LEFT).unwrap();
            }
            "closed" => engine.close_committed_tail_field(),
            _ => unreachable!(),
        }
        observe_client(&mut engine, &mut output, snapshot).unwrap();
        // An equal later receipt cannot resurrect the revoked continuation.
        observe_space_commit(&mut engine, &mut output).unwrap();
        assert!(
            output.surrounding_deletes.is_empty(),
            "must refuse {violation}"
        );
        assert!(
            engine.shared.lock().unwrap().pending_auto_undo.is_none(),
            "no undo from refused {violation}"
        );
    }
}
