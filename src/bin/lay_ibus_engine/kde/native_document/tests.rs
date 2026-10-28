use super::*;

#[test]
fn native_owned_append_resamples_one_incoherent_read_without_extending_deadline() {
    let name = "window_interaction::native_document::tests::native_owned_append_resamples_one_incoherent_read_without_extending_deadline";
    if !crate::space_boundary_pair_tests::isolated_case(name) {
        return;
    }
    use std::cell::RefCell;
    use std::collections::VecDeque;
    let (mut engine, binding, output) = native_owned_pair_waiting_for_boundary();
    assert!(engine.native_observation_failed("native_document_changed"));
    assert!(!engine.native_document_is_current());
    assert!(engine.native_owned_append_preparation_is_current());
    let expected = engine.managed_word_start_preparation_snapshot().unwrap();
    let token = engine.context_token.clone();
    let scope = engine.context_word_scope;
    let epoch = engine.committed_tail.epoch;
    let attempts = RefCell::new(VecDeque::from([
        Err("native_document_changed"),
        Ok(Observation {
            binding: binding.for_result(&engine),
            snapshot: expected.clone(),
        }),
    ]));
    let seen = RefCell::new(Vec::new());
    let deadline = Instant::now() + OBSERVATION_BUDGET;
    let result = zbus::block_on(read_with_one_owned_append_resample(
        deadline,
        |actual_deadline, geometry| {
            seen.borrow_mut().push((actual_deadline, geometry));
            std::future::ready(
                attempts
                    .borrow_mut()
                    .pop_front()
                    .expect("at most one source resample"),
            )
        },
        |snapshot| engine.native_owned_append_observation_is_pending(snapshot),
        engine.native_owned_append_preparation_is_current(),
    ));
    assert!(
        result.is_ok(),
        "a current owned append must reacquire a coherent source: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap().snapshot, expected);
    assert_eq!(*seen.borrow(), [(deadline, true), (deadline, false)]);
    assert!(
        !engine.native_document_is_current(),
        "read coordination cannot install a projected or returned observation"
    );
    assert_eq!(engine.context_token, token);
    assert_eq!(engine.context_word_scope, scope);
    assert_eq!(engine.committed_tail.epoch, epoch);
    assert!(output.committed_texts.is_empty());
    assert!(output.surrounding_deletes.is_empty());
}

#[test]
fn native_owned_append_resample_stops_on_other_errors_and_after_one_resample() {
    use std::cell::RefCell;
    use std::collections::VecDeque;
    let deadline = Instant::now() + OBSERVATION_BUDGET;
    for (first, allowed, expected_calls) in [
        ("native_document_changed", false, 1),
        ("native_document_changed", true, 2),
        ("native_selection_present", true, 1),
        ("native_private_or_terminal", true, 1),
        ("native_no_admission", true, 1),
        ("native_deadline", true, 1),
        ("native_focused_target_changed", true, 1),
    ] {
        let seen = RefCell::new(Vec::new());
        let attempts: RefCell<VecDeque<NativeResult<Observation>>> =
            RefCell::new(VecDeque::from([Err(first), Err("native_document_changed")]));
        let result = zbus::block_on(read_with_one_owned_append_resample(
            deadline,
            |actual_deadline, geometry| {
                seen.borrow_mut().push((actual_deadline, geometry));
                std::future::ready(
                    attempts
                        .borrow_mut()
                        .pop_front()
                        .expect("never retry a second error"),
                )
            },
            |_| false,
            allowed,
        ));
        assert!(result.is_err());
        assert_eq!(
            seen.borrow().len(),
            expected_calls,
            "{first}, allowed={allowed}"
        );
        assert!(seen.borrow().iter().all(|(actual, _)| *actual == deadline));
        assert!(seen.borrow()[0].1);
        if expected_calls == 2 {
            assert!(!seen.borrow()[1].1);
        }
    }
    let (engine, binding) = observed_native_engine();
    for pending in [false, true] {
        let seen = RefCell::new(Vec::new());
        let result = zbus::block_on(read_with_one_owned_append_resample(
            deadline,
            |actual_deadline, geometry| {
                seen.borrow_mut().push((actual_deadline, geometry));
                std::future::ready(Ok(Observation {
                    binding: binding.clone(),
                    snapshot: SurroundingTextSnapshot::new("пров".into(), 4, 4),
                }))
            },
            |_| pending,
            false,
        ));
        assert!(result.is_ok());
        assert_eq!(
            *seen.borrow(),
            if pending {
                vec![(deadline, true), (deadline, false)]
            } else {
                vec![(deadline, true)]
            }
        );
    }
    assert!(engine.native_document_is_current());
}

#[test]
fn chrome_activation_policy_does_not_probe_gnome() {
    assert!(
        zbus::block_on(LayIbusEngine::kde_chrome_activation_deadline(
            lay::desktop::LayoutBackend::Gnome,
        ))
        .is_none()
    );
}
use crate::window_interaction::{
    TextTargetCapabilityFacts, TextTargetDecision, TextTargetDecisionReason, TextTargetEditRoute,
};

#[test]
fn native_text_notification_registration_supports_atspi_2_44() {
    // This is the deployed registry's public wire ABI, not its empty reply.
    let message = zbus::Message::method_call("/org/a11y/atspi/registry", "RegisterEvent")
        .unwrap()
        .build(&bus::text_event_registration("object:text-changed"))
        .unwrap();
    assert_eq!(message.body().signature().to_string_no_parens(), "sas");
    let (event, properties): (String, Vec<String>) = message.body().deserialize().unwrap();
    assert_eq!(event, "object:text-changed");
    assert!(properties.is_empty());
}

#[test]
fn native_selection_notification_registration_supports_atspi_2_44() {
    let message = zbus::Message::method_call("/org/a11y/atspi/registry", "RegisterEvent")
        .unwrap()
        .build(&bus::text_event_registration(
            "object:text-selection-changed",
        ))
        .unwrap();
    assert_eq!(message.body().signature().to_string_no_parens(), "sas");
    let (event, properties): (String, Vec<String>) = message.body().deserialize().unwrap();
    assert_eq!(event, "object:text-selection-changed");
    assert!(properties.is_empty());
}

fn observed_native_engine() -> (LayIbusEngine, NativeDocumentBinding) {
    observed_native_engine_with_completeness(
        crate::context_admission::WordCompleteness::UnknownStart,
    )
}

fn observed_native_engine_with_completeness(
    completeness: crate::context_admission::WordCompleteness,
) -> (LayIbusEngine, NativeDocumentBinding) {
    observed_native_engine_with_component(completeness, "lay-ime-us")
}

fn observed_native_engine_with_component(
    completeness: crate::context_admission::WordCompleteness,
    component: &str,
) -> (LayIbusEngine, NativeDocumentBinding) {
    use crate::context_admission::{ActivationOutcome, ContextAdmissionAdapter};
    use std::sync::{Arc, Mutex};

    let path = "/test/native_document_observer";
    let (adapter, grant, peer) = ContextAdmissionAdapter::test_established(
        path,
        "/org/freedesktop/IBus/InputContext_native_observer",
        completeness,
        17,
    );
    let mut engine = LayIbusEngine::new_from_component(
        path.into(),
        Arc::new(Mutex::new(Default::default())),
        Some(adapter),
        component,
        true,
        lay::config::LayConfig {
            layout_backend: "kde".into(),
            ..Default::default()
        },
    );
    assert!(engine.install_context_activation(ActivationOutcome::SourceFree(grant)));
    engine.shared.lock().unwrap().active_path = Some(path.into());
    let binding = NativeDocumentBinding {
        identity: NativeFieldIdentity {
            desktop: FocusedWindow {
                identity: "test-window".into(),
                pid: 1,
            },
            bus: peer.server_guid().to_string(),
            owner: ":1.1".into(),
            application: "/application".try_into().unwrap(),
            field: "/field".try_into().unwrap(),
        },
        connection: peer,
        token: engine.context_token.clone().unwrap(),
        focus_serial: engine.client_context.focus_serial,
        owner_lease: engine.client_context.runtime_owner_lease_identity,
        layout_generation: engine.layout_gesture.layout_generation,
        tail_epoch: engine.committed_tail.epoch,
        observer: None,
        last_display_frame: None,
        hint_geometry: None,
    };
    engine.client_context.native_document_binding = Some(binding.clone());
    engine.observe_document_snapshot(Some(SurroundingTextSnapshot::new("пров".into(), 4, 4)));
    (engine, binding)
}

fn managed_native_acquisition_suffix_without_receipt() -> LayIbusEngine {
    use crate::engine::WordInputMode;
    use crate::output::{EngineOutput, TestEngineOutput};

    let (mut engine, binding) = observed_native_engine();
    engine.config.text_backend = "ime".into();
    engine.layout_gesture.layout_is_ru = true;
    engine.set_client_capabilities(41);
    engine.composition.word_input_mode = Some(WordInputMode::ManagedCommit);
    // Acquisition retained three literal letters; this lineage sees only the
    // next actual CommitText. It must remain UnknownStart after the readout.
    engine.committed_tail.buffer = "про".into();
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    zbus::block_on(engine.commit_managed_passthrough_char(
        &mut EngineOutput::test(&mut output),
        'в',
        32,
    ))
    .unwrap();
    assert_eq!(output.committed_texts, ["в"]);
    engine.client_context.native_document_binding = Some(binding.for_result(&engine));
    engine
}

fn managed_native_acquisition_suffix() -> LayIbusEngine {
    let mut engine = managed_native_acquisition_suffix_without_receipt();
    engine.observe_document_snapshot(Some(SurroundingTextSnapshot::new("пров".into(), 4, 4)));
    assert!(engine.native_manual_word_is_current());
    assert!(engine.exact_managed_surrounding_word_is_current());
    assert!(
        engine
            .context_word_scope
            .unwrap()
            .lineage()
            .observed_suffix_chars
            < 4
    );
    engine
}

fn short_function_native_word(
    source: &str,
    completeness: crate::context_admission::WordCompleteness,
) -> (LayIbusEngine, NativeDocumentBinding) {
    use crate::engine::WordInputMode;
    use crate::output::{EngineOutput, TestEngineOutput};

    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("warm exact authority");
    let (mut engine, binding) = observed_native_engine_with_completeness(completeness);
    engine.config.text_backend = "ime".into();
    engine.config.auto_replace = true;
    engine.config.auto_switch_layout = true;
    engine.layout_gesture.layout_is_ru = false;
    engine.set_client_capabilities(41);
    engine.composition.word_input_mode = Some(WordInputMode::ManagedCommit);
    engine.committed_tail.buffer.clear();
    engine.composition.preedit_fast.reset();
    engine.observe_document_snapshot(Some(SurroundingTextSnapshot::new(String::new(), 0, 0)));
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    for ch in source.chars() {
        zbus::block_on(engine.commit_managed_passthrough_char(
            &mut EngineOutput::test(&mut output),
            ch,
            32,
        ))
        .unwrap();
    }
    let binding = binding.for_result(&engine);
    engine.client_context.native_document_binding = Some(binding.clone());
    (engine, binding)
}

fn native_owned_pair_waiting_for_boundary() -> (
    LayIbusEngine,
    NativeDocumentBinding,
    crate::output::TestEngineOutput,
) {
    use crate::engine::WordInputMode;
    use crate::output::{EngineOutput, TestEngineOutput};
    crate::space_boundary_pair_tests::warm_pair_lexical_fixture();
    let (mut engine, binding) = observed_native_engine_with_component(
        crate::context_admission::WordCompleteness::UnknownStart,
        "lay-ime-ru",
    );
    engine.config.text_backend = "ime".into();
    engine.config.auto_replace = false;
    engine.config.typing_assist = true;
    engine.config.nanda_autocorrect = true;
    engine.config.correction_safety = "normal".into();
    engine.set_client_capabilities(41);
    engine.composition.word_input_mode = Some(WordInputMode::ManagedCommit);
    engine.committed_tail.buffer = "должн".into();
    engine.client_context.native_document_binding = Some(binding.for_result(&engine));
    engine.observe_document_snapshot(Some(SurroundingTextSnapshot::new("должн".into(), 5, 5)));
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    zbus::block_on(engine.commit_managed_passthrough_char(
        &mut EngineOutput::test(&mut output),
        ' ',
        57,
    ))
    .unwrap();
    assert!(engine.install_native_document(Observation {
        binding: binding.for_result(&engine),
        snapshot: SurroundingTextSnapshot::new("должн ".into(), 6, 6),
    }));
    engine.composition.word_input_mode = Some(WordInputMode::ManagedCommit);
    for ch in "ыбыть".chars() {
        zbus::block_on(engine.commit_managed_passthrough_char(
            &mut EngineOutput::test(&mut output),
            ch,
            32,
        ))
        .unwrap();
    }
    assert!(engine.install_native_document(Observation {
        binding: binding.for_result(&engine),
        snapshot: SurroundingTextSnapshot::new("должн ыбыть".into(), 11, 11),
    }));
    assert!(engine.native_word_preparation_edit_is_current());
    let pair = engine.capture_space_boundary_pair_scope().unwrap();
    assert_eq!(pair.text, "должн ыбыть");
    assert!(matches!(
        pair.provenance,
        crate::engine::SpaceBoundaryPairProvenance::NativePrepared { .. }
    ));
    assert!(engine.composition.buffer.is_empty());
    engine.config.auto_replace = true;
    output.committed_texts.clear();
    (engine, binding, output)
}

fn settle_native_space_scope(engine: &mut LayIbusEngine, binding: &mut NativeDocumentBinding) {
    // Mirror ProcessKeyEvent settlement through the production scope operation
    // and the existing fixture's reducer-state publisher.
    let epoch = engine.committed_tail.epoch;
    let boundary = u32::try_from(engine.committed_tail.buffer.chars().count() - 1).unwrap();
    let scope = engine.context_word_scope.as_mut().unwrap();
    scope.close_at_observed_boundary(epoch, Some(boundary));
    let admission = engine.context_admission.as_ref().unwrap();
    assert!(admission.test_set_word_scope(engine.context_owner.as_ref().unwrap(), epoch, scope));
    engine.context_token = admission.current_token();
    assert_ne!(engine.context_token.as_ref(), Some(&binding.token));
    binding.token = engine.context_token.clone().unwrap();
}

#[test]
fn native_pair_pending_work_joins_only_after_actual_space_in_either_order() {
    let name = "window_interaction::native_document::tests::native_pair_pending_work_joins_only_after_actual_space_in_either_order";
    if !crate::space_boundary_pair_tests::isolated_case(name) {
        return;
    }
    use crate::engine::PendingSpaceBoundarySelection;
    use crate::output::EngineOutput;
    use crate::space_autocorrect_prefetch::proof::{
        has_current_slot, hold_full_lease, publish_held_full_lease,
    };
    for (actual_first, pre_commit_echo, settled_boundary) in [
        (false, false, false),
        (true, false, false),
        (false, true, false),
        (true, true, false),
        (false, false, true),
        (true, false, true),
        (false, true, true),
        (true, true, true),
    ] {
        let (mut engine, mut binding, mut output) = native_owned_pair_waiting_for_boundary();
        let frame = engine.capture_space_autocorrect_frame_identity().unwrap();
        let lease = hold_full_lease(&frame, &engine.config);
        assert_eq!(lease.decision.replacement, "должны быть ");
        let lookup = engine.take_space_autocorrect_lease(&frame);
        assert!(matches!(
            lookup.lookup,
            crate::space_autocorrect_prefetch::SpaceAutocorrectLookup::NotReady
        ));
        assert!(
            has_current_slot(&frame),
            "native pending lookup must retain its current work"
        );
        let pending = engine
            .prepare_space_boundary_continuation(
                &frame,
                PendingSpaceBoundarySelection::Computing {
                    identity: Box::new(frame.clone()),
                    worker_generation: lease.worker_generation,
                },
            )
            .expect("the exact native pair supplies the existing continuation");
        assert!(zbus::block_on(
            engine
                .commit_space_boundary_continuation(&mut EngineOutput::test(&mut output), pending,)
        )
        .unwrap());
        assert_eq!(output.committed_texts, [" "]);
        assert_eq!(engine.committed_tail.buffer, "должн ыбыть ");
        assert!(engine.space_boundary_is_pending());
        if settled_boundary {
            settle_native_space_scope(&mut engine, &mut binding);
        }
        if pre_commit_echo {
            assert!(engine.install_native_document(Observation {
                binding: binding.for_result(&engine),
                snapshot: SurroundingTextSnapshot::new("должн ыбыть".into(), 11, 11),
            }));
            assert!(
                !zbus::block_on(engine.apply_space_boundary_after_client_commit(
                    &mut EngineOutput::test(&mut output),
                ))
                .unwrap()
            );
            assert!(
                engine.space_boundary_is_pending(),
                "retain the pre-Space echo"
            );
            assert!(has_current_slot(&frame));
        }
        if actual_first {
            assert!(engine.install_native_document(Observation {
                binding: binding.for_result(&engine),
                snapshot: SurroundingTextSnapshot::new("должн ыбыть ".into(), 12, 12),
            }));
        }
        assert!(!zbus::block_on(
            engine.apply_space_boundary_after_client_commit(&mut EngineOutput::test(&mut output),)
        )
        .unwrap());
        assert!(has_current_slot(&frame));
        assert!(publish_held_full_lease(lease));
        if !actual_first {
            assert!(
                !zbus::block_on(engine.apply_space_boundary_after_client_commit(
                    &mut EngineOutput::test(&mut output),
                ))
                .unwrap(),
                "a computed action without actual Space cannot edit"
            );
            assert!(has_current_slot(&frame));
            assert!(engine.install_native_document(Observation {
                binding: binding.for_result(&engine),
                snapshot: SurroundingTextSnapshot::new("должн ыбыть ".into(), 12, 12),
            }));
        }
        // The fixture deliberately has no real native client. Reaching the
        // existing executor consumes exactly this decision and must refuse
        // without mutation; live acceptance proves actual range execution.
        let receipt = engine
            .committed_tail
            .pending_visible_postcondition
            .as_ref()
            .unwrap();
        let pending = receipt.space_boundary_commit.as_ref().unwrap();
        assert!(engine.native_space_boundary_owner_is_current(
            receipt.native_document.as_ref(),
            &pending.frame
        ));
        assert!(engine.input_frame_authority_matches(&pending.frame));
        assert!(pending
            .observed_pair_commit(
                engine
                    .client_context
                    .surrounding_text_snapshot
                    .as_ref()
                    .unwrap()
            )
            .is_some());
        let _ = zbus::block_on(
            engine.apply_space_boundary_after_client_commit(&mut EngineOutput::test(&mut output)),
        );
        assert!(!has_current_slot(&frame));
        assert!(engine
            .committed_tail
            .pending_visible_postcondition
            .as_ref()
            .is_none_or(|receipt| receipt.space_boundary_commit.is_none()));
        assert_eq!(output.committed_texts, [" "]);
        assert!(output.surrounding_deletes.is_empty());
        assert!(!zbus::block_on(
            engine.apply_space_boundary_after_client_commit(&mut EngineOutput::test(&mut output),)
        )
        .unwrap());
    }
}

#[test]
fn native_pair_pending_work_is_cancelled_on_input_or_changed_authority() {
    let name = "window_interaction::native_document::tests::native_pair_pending_work_is_cancelled_on_input_or_changed_authority";
    if !crate::space_boundary_pair_tests::isolated_case(name) {
        return;
    }
    use crate::engine::PendingSpaceBoundarySelection;
    use crate::output::EngineOutput;
    use crate::space_autocorrect_prefetch::proof::{
        has_current_slot, hold_full_lease, publish_held_full_lease,
    };
    for settled_boundary in [false, true] {
        for changed in [
            "focus",
            "lease",
            "layout",
            "epoch",
            "field",
            "text",
            "selection",
            "cursor",
            "expired",
            "material",
            "gnome",
            "atomic",
            "next_key",
            "second_boundary",
            "input_gap_scope",
        ] {
            let (mut engine, mut binding, mut output) = native_owned_pair_waiting_for_boundary();
            let frame = engine.capture_space_autocorrect_frame_identity().unwrap();
            let lease = hold_full_lease(&frame, &engine.config);
            let lookup = engine.take_space_autocorrect_lease(&frame);
            assert!(matches!(
                lookup.lookup,
                crate::space_autocorrect_prefetch::SpaceAutocorrectLookup::NotReady
            ));
            assert!(has_current_slot(&frame));
            let pending = engine
                .prepare_space_boundary_continuation(
                    &frame,
                    PendingSpaceBoundarySelection::Computing {
                        identity: Box::new(frame.clone()),
                        worker_generation: lease.worker_generation,
                    },
                )
                .unwrap();
            assert!(zbus::block_on(engine.commit_space_boundary_continuation(
                &mut EngineOutput::test(&mut output),
                pending,
            ))
            .unwrap());
            if settled_boundary {
                settle_native_space_scope(&mut engine, &mut binding);
            }
            let mut actual = SurroundingTextSnapshot::new("должн ыбыть ".into(), 12, 12);
            match changed {
                "text" => actual.text = "чужой ыбыть ".into(),
                "selection" => actual.anchor_pos = 0,
                "cursor" => {
                    actual.cursor_pos = 11;
                    actual.anchor_pos = 11;
                }
                _ => {}
            }
            assert!(engine.install_native_document(Observation {
                binding: binding.for_result(&engine),
                snapshot: actual
            }));
            assert!(publish_held_full_lease(lease));
            match changed {
                "focus" => engine.client_context.focus_serial += 1,
                "lease" => engine.client_context.runtime_owner_lease_identity += 1,
                "layout" => engine.layout_gesture.layout_generation += 1,
                "epoch" => engine.committed_tail.epoch += 1,
                "field" => {
                    engine
                        .client_context
                        .native_document_binding
                        .as_mut()
                        .unwrap()
                        .identity
                        .field = "/other".try_into().unwrap()
                }
                "expired" => {
                    engine
                        .committed_tail
                        .pending_visible_postcondition
                        .as_mut()
                        .unwrap()
                        .dispatched_at = Instant::now() - std::time::Duration::from_millis(1501)
                }
                "material" => {
                    engine
                        .committed_tail
                        .pending_visible_postcondition
                        .as_mut()
                        .unwrap()
                        .space_boundary_commit
                        .as_mut()
                        .unwrap()
                        .material_generation = u64::MAX
                }
                "gnome" => engine.config.layout_backend = "gnome".into(),
                "atomic" => engine.atomic.active = true,
                "next_key" => {
                    let _ = zbus::block_on(engine.process_pressed_key(
                        &mut EngineOutput::test(&mut output),
                        crate::protocol::KEY_LEFT,
                        0,
                        0,
                    ));
                }
                "second_boundary" | "input_gap_scope" => {
                    let epoch = engine.committed_tail.epoch;
                    let mut scope = crate::context_admission::WordScope::new(
                        engine.context_word_scope.as_ref().unwrap().lineage(),
                    );
                    if changed == "second_boundary" {
                        scope.close_at_observed_boundary(epoch, Some(11));
                        scope.close_at_observed_boundary(epoch, Some(11));
                    } else {
                        scope.revoke_for_input_gap();
                    }
                    let admission = engine.context_admission.as_ref().unwrap();
                    assert!(admission.test_set_word_scope(
                        engine.context_owner.as_ref().unwrap(),
                        epoch,
                        &scope
                    ));
                    engine.context_word_scope = Some(scope);
                    engine.context_token = admission.current_token();
                }
                _ => {}
            }
            let _ = zbus::block_on(
                engine
                    .apply_space_boundary_after_client_commit(&mut EngineOutput::test(&mut output)),
            );
            assert!(!has_current_slot(&frame), "retire {changed}");
            assert!(
                output.surrounding_deletes.is_empty(),
                "no delete for {changed}"
            );
            assert_eq!(
                output.committed_texts,
                [" "],
                "only the original Space for {changed}"
            );
        }
    }
}

fn pending_native_us_append() -> LayIbusEngine {
    use crate::context_admission::WordCompleteness;
    use crate::output::{EngineOutput, TestEngineOutput};
    let (mut engine, binding) = short_function_native_word("y", WordCompleteness::KnownStart);
    assert!(engine.install_native_document(Observation {
        binding: binding.clone(),
        snapshot: SurroundingTextSnapshot::new("y".into(), 1, 1),
    }));
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    zbus::block_on(engine.commit_managed_passthrough_char(
        &mut EngineOutput::test(&mut output),
        'f',
        32,
    ))
    .unwrap();
    assert_eq!(output.committed_texts, ["f"]);
    engine.client_context.native_document_binding = Some(binding.for_result(&engine));
    engine
}

#[test]
fn native_us_append_observation_only_resamples_earlier_owned_prefix() {
    use crate::context_admission::WordCompleteness;

    let engine = pending_native_us_append();
    let snapshot =
        |text: &str, cursor, anchor| SurroundingTextSnapshot::new(text.into(), cursor, anchor);
    for actual in [snapshot("", 0, 0), snapshot("y", 1, 1)] {
        assert!(
            engine.native_owned_append_observation_is_pending(&actual),
            "activity={} preparation={:?} chain={:?} native={} managed={} witness={:?}",
            engine.has_unfinished_us_managed_input(),
            engine.native_word_preparation_identity(),
            engine.managed_word_start_preparation_identity(),
            engine.native_document_is_current(),
            engine.client_context.managed_input,
            engine.client_context.managed_word_start
        );
    }
    for actual in [
        snapshot("yf", 2, 2),
        snapshot("yx", 2, 2),
        snapshot("z", 1, 1),
        snapshot("y ", 1, 1),
        snapshot("yf", 1, 1),
        snapshot("y", 1, 0),
    ] {
        assert!(
            !engine.native_owned_append_observation_is_pending(&actual),
            "{actual:?}"
        );
    }
    assert!(!engine.native_word_preparation_edit_is_current());
    assert_ne!(
        engine.client_context.surrounding_text_snapshot,
        engine.managed_word_start_preparation_snapshot(),
        "classifying a stale source must not manufacture an actual receipt"
    );

    // A later word keeps both the earlier Unicode text and the right suffix.
    // An observation before this word's boundary is not an owned append.
    use crate::output::{EngineOutput, TestEngineOutput};
    let (mut engine, binding) = short_function_native_word("y", WordCompleteness::KnownStart);
    engine.committed_tail.buffer = "пишу y".into();
    assert!(engine.install_native_document(Observation {
        binding: binding.for_result(&engine),
        snapshot: snapshot("😀 пишу y хвост", 8, 8),
    }));
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    zbus::block_on(engine.commit_managed_passthrough_char(
        &mut EngineOutput::test(&mut output),
        'f',
        32,
    ))
    .unwrap();
    engine.client_context.native_document_binding = Some(binding.for_result(&engine));
    assert!(engine.native_owned_append_observation_is_pending(&snapshot("😀 пишу y хвост", 8, 8)));
    for actual in [
        snapshot("😀 пишу хвост", 6, 6),
        snapshot("😀 пишу yf хвост", 9, 9),
        snapshot("😀 пишу y иной", 8, 8),
    ] {
        assert!(
            !engine.native_owned_append_observation_is_pending(&actual),
            "{actual:?}"
        );
    }
    assert_eq!(output.committed_texts, ["f"]);
    assert!(output.surrounding_deletes.is_empty());
    assert!(output.forwarded_keys.is_empty());
}

#[test]
fn native_us_append_observation_respects_owner_and_input_boundaries() {
    let stale = SurroundingTextSnapshot::new("y".into(), 1, 1);
    for changed in [
        "focus",
        "lease",
        "layout",
        "gnome",
        "ru",
        "atomic",
        "privacy",
        "epoch",
        "unmanaged",
        "boundary",
    ] {
        let mut engine = pending_native_us_append();
        assert!(engine.native_owned_append_observation_is_pending(&stale));
        match changed {
            "focus" => engine.client_context.focus_serial += 1,
            "lease" => engine.client_context.runtime_owner_lease_identity += 1,
            "layout" => engine.layout_gesture.layout_generation += 1,
            "gnome" => engine.config.layout_backend = "gnome".into(),
            "ru" => engine.layout_gesture.layout_is_ru = true,
            "atomic" => engine.atomic.active = true,
            "privacy" => {
                engine.client_context.content_purpose = super::super::IBUS_INPUT_PURPOSE_PASSWORD
            }
            "epoch" => engine.committed_tail.epoch += 1,
            "unmanaged" => engine.client_context.managed_input = false,
            "boundary" => engine.committed_tail.buffer.push(' '),
            _ => unreachable!(),
        }
        assert!(
            !engine.native_owned_append_observation_is_pending(&stale),
            "{changed}"
        );
    }
}

#[test]
fn short_function_space_fresh_native_read_prepares_at_boundary() {
    use crate::space_autocorrect_prefetch::{self, PreparedLeaseKind, SpaceAutocorrectLookup};
    for (source, target) in [("d", "в "), ("b", "и "), ("j,", "об ")] {
        let (mut engine, binding) = short_function_native_word(
            source,
            crate::context_admission::WordCompleteness::UnknownStart,
        );
        space_autocorrect_prefetch::invalidate_path(&engine.path);
        let scope = engine.context_word_scope;
        assert!(engine.install_native_document(Observation {
            binding,
            snapshot: SurroundingTextSnapshot::new(
                source.into(),
                source.chars().count() as u32,
                source.chars().count() as u32
            ),
        }));
        let frame = engine
            .capture_space_autocorrect_frame_identity()
            .expect("actual complete-word proof");
        let lookup = engine.take_space_autocorrect_lease(&frame);
        let SpaceAutocorrectLookup::Ready(lease) = lookup.lookup else {
            panic!("fresh Space proof was not prepared for {source:?}");
        };
        assert_eq!(lease.kind, PreparedLeaseKind::ExactLayout);
        assert_eq!(lease.decision.replacement, target);
        assert!(lease.decision.action.allow_apply());
        assert!(lease.exact_certificate.is_some());
        assert_eq!(engine.context_word_scope, scope);
        assert!(!engine.context_word_is_known());
        space_autocorrect_prefetch::invalidate(&frame);
    }
}

#[test]
fn short_function_space_activity_is_independent_of_display_cache() {
    use crate::space_autocorrect_prefetch::{self, SpaceAutocorrectLookup};
    for source in ["d", "b", "j,"] {
        let (mut engine, binding) = short_function_native_word(
            source,
            crate::context_admission::WordCompleteness::UnknownStart,
        );
        engine.composition.preedit_fast.reset();
        assert!(engine.install_native_document(Observation {
            binding,
            snapshot: SurroundingTextSnapshot::new(
                source.into(),
                source.chars().count() as u32,
                source.chars().count() as u32
            ),
        }));
        assert!(engine.capture_input_frame_identity().is_none());
        let frame = engine
            .capture_space_autocorrect_frame_identity()
            .expect("actual complete-word proof");
        assert!(
            frame.active_composition,
            "unfinished owned word {source:?} lost activity with its display cache"
        );
        assert_eq!(frame.observed_token, source);
        assert!(!engine.context_word_is_known());
        let last_input = engine.committed_tail.last_input_at.take();
        assert!(
            !engine
                .capture_space_autocorrect_frame_identity()
                .expect("inert source frame")
                .active_composition
        );
        engine.committed_tail.last_input_at = last_input;
        use lay::exact_layout_authority::FactoryEngineProfile;
        for (profile, ru) in [
            (FactoryEngineProfile::Ru, false),
            (FactoryEngineProfile::Unknown, false),
            (FactoryEngineProfile::UsQwerty, true),
        ] {
            engine.client_context.factory_engine_profile = profile;
            engine.layout_gesture.layout_is_ru = ru;
            let other = engine
                .capture_space_autocorrect_frame_identity()
                .expect("inert source frame");
            assert!(!other.active_composition);
            space_autocorrect_prefetch::invalidate_path(&engine.path);
            assert!(matches!(
                engine.take_space_autocorrect_lease(&other).lookup,
                SpaceAutocorrectLookup::NotReady
            ));
        }
        engine.client_context.factory_engine_profile = FactoryEngineProfile::UsQwerty;
        engine.layout_gesture.layout_is_ru = false;
        engine.composition.word_input_mode =
            Some(crate::engine::WordInputMode::TerminalPassthrough);
        assert!(!engine.has_unfinished_us_managed_input());
        engine.composition.word_input_mode = Some(crate::engine::WordInputMode::ManagedCommit);
        engine.atomic.active = true;
        assert!(!engine.has_unfinished_us_managed_input());
    }
}

#[test]
fn short_function_space_known_word_validator_uses_the_same_activity() {
    use crate::space_autocorrect_prefetch::{self, PreparedLeaseKind, SpaceAutocorrectLookup};
    for (source, target) in [("d", "в "), ("b", "и "), ("j,", "об ")] {
        let (mut engine, binding) = short_function_native_word(
            source,
            crate::context_admission::WordCompleteness::KnownStart,
        );
        engine.composition.preedit_fast.reset();
        space_autocorrect_prefetch::invalidate_path(&engine.path);
        assert!(engine.install_native_document(Observation {
            binding,
            snapshot: SurroundingTextSnapshot::new(
                source.into(),
                source.chars().count() as u32,
                source.chars().count() as u32
            ),
        }));
        assert!(engine.context_word_is_known());
        let frame = engine
            .capture_space_autocorrect_frame_identity()
            .expect("known Space frame");
        assert!(frame.active_composition);
        let completion = engine
            .capture_input_frame_identity()
            .expect("known completion frame");
        assert!(!completion.active_composition);
        assert_ne!(frame.frame_fingerprint, completion.frame_fingerprint);
        let lookup = engine.take_space_autocorrect_lease(&frame);
        let SpaceAutocorrectLookup::Ready(lease) = lookup.lookup else {
            panic!("known Space frame rejected by its validator: {source:?}");
        };
        assert_eq!(lease.kind, PreparedLeaseKind::ExactLayout);
        assert_eq!(lease.decision.replacement, target);
        assert!(lease.decision.action.allow_apply());
        space_autocorrect_prefetch::invalidate(&frame);
    }
}

#[test]
fn native_document_same_text_after_commit_observes_the_new_effect_only_once() {
    let mut engine = managed_native_acquisition_suffix_without_receipt();
    // The managed commit already discards its old cached snapshot. An actual
    // same-surface read must restore it once, then remain stable for Tab.
    assert!(engine.client_context.surrounding_text_snapshot.is_none());
    let snapshot = SurroundingTextSnapshot::new("пров".into(), 4, 4);
    let binding = engine
        .client_context
        .native_document_binding
        .clone()
        .unwrap();
    let revision = engine.client_context.surrounding_observation_revision;
    let scope = engine.context_word_scope;
    let token = engine.context_token.clone();
    let epoch = engine.committed_tail.epoch;
    assert_eq!(snapshot.text, "пров");
    assert!(binding.is_current(&engine));
    assert!(!engine.exact_managed_surrounding_word_is_current());
    assert!(engine.capture_space_autocorrect_frame_identity().is_none());
    assert!(engine.install_native_document(Observation {
        binding,
        snapshot: snapshot.clone(),
    }));
    assert!(
        engine.client_context.surrounding_observation_revision > revision,
        "the actual post-CommitText read was mistaken for its pre-commit cache"
    );
    assert!(engine.exact_managed_surrounding_word_is_current());
    let frame = engine.capture_space_autocorrect_frame_identity().unwrap();
    let observed_revision = engine.client_context.surrounding_observation_revision;
    assert!(engine.install_native_document(Observation {
        binding: engine
            .client_context
            .native_document_binding
            .clone()
            .unwrap(),
        snapshot,
    }));
    assert_eq!(
        engine.client_context.surrounding_observation_revision,
        observed_revision
    );
    assert_eq!(
        engine.capture_space_autocorrect_frame_identity(),
        Some(frame)
    );
    assert_eq!(engine.context_word_scope, scope);
    assert_eq!(engine.context_token, token);
    assert_eq!(engine.committed_tail.epoch, epoch);
    assert!(!engine.context_word_is_known());
}

#[test]
fn native_document_space_word_proof_does_not_require_manual_authority_over_older_text() {
    use crate::output::{EngineOutput, TestEngineOutput};
    use crate::space_autocorrect_prefetch::{self, SpaceAutocorrectLookup};

    let mut engine = managed_native_acquisition_suffix();
    engine.config.auto_replace = true;
    engine.committed_tail.buffer = "проверка\u{a0}пров".into();
    engine.observe_document_snapshot(Some(SurroundingTextSnapshot::new(
        "проверка пров".into(),
        13,
        13,
    )));
    assert!(!engine.native_manual_word_is_current());
    assert!(engine.exact_managed_surrounding_word_is_current());
    let frame = engine.capture_space_autocorrect_frame_identity().unwrap();
    let token = engine.context_token.clone();
    let scope = engine.context_word_scope;
    let epoch = engine.committed_tail.epoch;
    let before =
        space_autocorrect_prefetch::take_with_budget(&frame, Duration::ZERO).worker_generation;
    let mut output = TestEngineOutput::default();
    zbus::block_on(
        engine.refresh_native_document_precognition(&mut EngineOutput::test(&mut output)),
    )
    .unwrap();
    let queued = space_autocorrect_prefetch::take_with_budget(&frame, Duration::ZERO);
    assert!(
        queued.worker_generation > before,
        "the independent canonical Space word proof was hidden behind the manual-prefix gate"
    );
    assert!(!matches!(queued.lookup, SpaceAutocorrectLookup::Stale));
    assert_eq!(engine.context_token, token);
    assert_eq!(engine.context_word_scope, scope);
    assert_eq!(engine.committed_tail.epoch, epoch);
    assert!(!engine.native_manual_word_is_current());
    assert!(!engine.context_word_is_known());
    assert!(output.committed_texts.is_empty());
    space_autocorrect_prefetch::invalidate(&frame);
}

#[test]
fn native_document_actual_read_queues_current_space_frame_without_promoting_scope() {
    use crate::output::{EngineOutput, TestEngineOutput};
    use crate::space_autocorrect_prefetch::{self, SpaceAutocorrectLookup};

    let mut engine = managed_native_acquisition_suffix();
    engine.config.auto_replace = true;
    let scope = engine.context_word_scope;
    let token = engine.context_token.clone();
    let epoch = engine.committed_tail.epoch;
    assert!(!engine.context_word_is_known());
    let frame = engine.capture_space_autocorrect_frame_identity().unwrap();
    assert!(frame.space_autocorrect_surrounding_revision.is_some());
    assert!(frame.display_suffix_token.is_none());
    let before =
        space_autocorrect_prefetch::take_with_budget(&frame, Duration::ZERO).worker_generation;
    let mut output = TestEngineOutput::default();
    zbus::block_on(
        engine.refresh_native_document_precognition(&mut EngineOutput::test(&mut output)),
    )
    .unwrap();
    let queued = space_autocorrect_prefetch::take_with_budget(&frame, Duration::ZERO);
    assert!(
        queued.worker_generation > before,
        "fresh native receipt did not queue Space work"
    );
    assert!(!matches!(queued.lookup, SpaceAutocorrectLookup::Stale));
    assert_eq!(engine.context_word_scope, scope);
    assert_eq!(engine.context_token, token);
    assert_eq!(engine.committed_tail.epoch, epoch);
    assert!(!engine.context_word_is_known());
    assert!(output.committed_texts.is_empty());
    space_autocorrect_prefetch::invalidate(&frame);
}

#[test]
fn native_space_preparation_continues_owned_append_without_reusing_edit_receipt() {
    use crate::output::{EngineOutput, TestEngineOutput};

    let mut engine = managed_native_acquisition_suffix();
    engine.config.auto_replace = true;
    let original_scope = engine.context_word_scope;
    let original_token = engine.context_token.clone();
    let before = engine.capture_space_autocorrect_frame_identity().unwrap();
    let binding = engine
        .client_context
        .native_document_binding
        .clone()
        .unwrap();
    assert_eq!(before.observed_token, "пров");
    assert!(
        engine.client_context.managed_word_start.is_some(),
        "native={} managed={} mode={:?} snapshot={:?}",
        engine.native_document_is_current(),
        engine.client_context.managed_input,
        engine.composition.word_input_mode,
        engine.client_context.surrounding_text_snapshot,
    );

    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    zbus::block_on(Box::pin(engine.commit_managed_passthrough_char(
        &mut EngineOutput::test(&mut output),
        'е',
        18,
    )))
    .unwrap();
    assert_eq!(output.committed_texts, ["е"]);
    assert!(!engine.native_document_is_current());
    assert_eq!(
        engine.text_target_edit_route(5),
        TextTargetEditRoute::Unsupported
    );
    assert!(
        engine
            .client_context
            .surrounding_text_snapshot
            .as_ref()
            .is_none_or(|snapshot| snapshot.text == "пров"),
        "a local commit must not manufacture a new actual observation"
    );
    let prepared = engine
        .capture_space_autocorrect_frame_identity()
        .unwrap_or_else(|| {
            panic!(
                "owned append preparation missing: chain={:?} native={:?} witness={:?} snapshot={}",
                engine.managed_word_start_preparation_identity(),
                engine.native_word_preparation_identity(),
                engine.client_context.managed_word_start,
                engine.client_context.surrounding_text_snapshot.is_some(),
            )
        });
    assert_eq!(prepared.observed_token, "прове");
    assert_ne!(
        prepared, before,
        "the old four-letter calculation stays stale"
    );

    // A native read can complete while Chrome still exposes the preceding
    // owned prefix. That actual document cannot edit the new five-letter word,
    // but must not erase its inert preparation or invent a different owner.
    engine.client_context.native_document_binding = Some(binding.for_result(&engine));
    engine.observe_document_snapshot(Some(SurroundingTextSnapshot::new("пров".into(), 4, 4)));
    assert!(!engine.native_word_preparation_edit_is_current());
    assert_eq!(
        engine.capture_space_autocorrect_frame_identity().as_ref(),
        Some(&prepared),
        "an earlier owned-prefix observation must not replace the new input material"
    );

    engine.client_context.native_document_binding = Some(binding.for_result(&engine));
    engine.observe_document_snapshot(Some(SurroundingTextSnapshot::new("прове".into(), 5, 5)));
    assert_eq!(
        engine.capture_space_autocorrect_frame_identity().as_ref(),
        Some(&prepared),
        "the actual matching receipt corroborates the same owned input material"
    );
    assert_eq!(
        engine.text_target_edit_route(5),
        TextTargetEditRoute::NativeRangeCommit
    );
    assert_eq!(engine.context_word_scope, original_scope);
    assert_eq!(engine.context_token, original_token);
    assert!(!engine.context_word_is_known());
}

#[test]
fn native_space_pair_preparation_survives_prior_prefix_without_edit_permission() {
    use crate::engine::WordInputMode;
    use crate::output::{EngineOutput, TestEngineOutput};

    let (mut engine, binding) = observed_native_engine();
    engine.config.text_backend = "ime".into();
    engine.layout_gesture.layout_is_ru = true;
    engine.set_client_capabilities(41);
    engine.composition.word_input_mode = Some(WordInputMode::ManagedCommit);
    engine.committed_tail.buffer = "тест про".into();
    engine.client_context.native_document_binding = Some(binding.for_result(&engine));
    engine.observe_document_snapshot(Some(SurroundingTextSnapshot::new("тест про".into(), 8, 8)));
    let scope = engine.context_word_scope;
    let token = engine.context_token.clone();
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    zbus::block_on(Box::pin(engine.commit_managed_passthrough_char(
        &mut EngineOutput::test(&mut output),
        'в',
        32,
    )))
    .unwrap();
    let prepared = engine.capture_space_autocorrect_frame_identity().unwrap();
    assert_eq!(
        prepared
            .space_boundary_pair
            .as_ref()
            .map(|pair| pair.text.as_str()),
        Some("тест пров"),
        "the complete pair is preparation data before the native receipt"
    );
    let pair = prepared.space_boundary_pair.as_ref().unwrap();
    assert!(engine.space_pair_client_snapshot(pair).is_none());
    assert!(!engine.native_word_preparation_edit_is_current());
    engine.client_context.native_document_binding = Some(binding.for_result(&engine));
    engine.observe_document_snapshot(Some(SurroundingTextSnapshot::new("тест про".into(), 8, 8)));
    assert_eq!(
        engine.capture_space_autocorrect_frame_identity().as_ref(),
        Some(&prepared)
    );
    assert!(engine.space_pair_client_snapshot(pair).is_none());
    engine.client_context.native_document_binding = Some(binding.for_result(&engine));
    engine.observe_document_snapshot(Some(SurroundingTextSnapshot::new("тест пров".into(), 9, 9)));
    assert_eq!(
        engine.capture_space_autocorrect_frame_identity().as_ref(),
        Some(&prepared)
    );
    assert!(engine.space_pair_client_snapshot(pair).is_some());
    assert_eq!(engine.context_word_scope, scope);
    assert_eq!(engine.context_token, token);
    assert!(!engine.context_word_is_known());
    assert_eq!(output.committed_texts, ["в"]);
    assert!(output.surrounding_deletes.is_empty());
    assert!(output.forwarded_keys.is_empty());
}

#[test]
fn native_space_preparation_rejects_changed_owner_and_nonappend_input() {
    use crate::output::{EngineOutput, TestEngineOutput};

    for changed in [
        "focus",
        "lease",
        "layout",
        "token",
        "scope",
        "handoff",
        "gnome",
        "atomic",
        "selection",
        "snapshot",
        "tail_aba",
        "text",
        "mode",
    ] {
        let mut engine = managed_native_acquisition_suffix();
        let mut output = TestEngineOutput {
            legacy_transport: true,
            ..Default::default()
        };
        zbus::block_on(Box::pin(engine.commit_managed_passthrough_char(
            &mut EngineOutput::test(&mut output),
            'е',
            18,
        )))
        .unwrap();
        assert!(engine.native_word_preparation_identity().is_some());
        assert!(!engine.native_word_preparation_edit_is_current());
        match changed {
            "focus" => engine.client_context.focus_serial += 1,
            "lease" => engine.client_context.runtime_owner_lease_identity += 1,
            "layout" => engine.layout_gesture.layout_generation += 1,
            "token" => engine.context_token = None,
            "scope" => engine.context_word_scope = None,
            "handoff" => engine.context_handoff_sealed = true,
            "gnome" => engine.config.layout_backend = "gnome".into(),
            "atomic" => engine.atomic.active = true,
            "selection" => {
                engine.observe_document_snapshot(Some(SurroundingTextSnapshot::new(
                    "прове".into(),
                    5,
                    0,
                )));
            }
            "snapshot" => engine.observe_document_snapshot(None),
            "tail_aba" => engine.committed_tail.epoch += 2,
            "text" => engine.committed_tail.buffer.push('и'),
            "mode" => engine.composition.word_input_mode = None,
            _ => unreachable!(),
        }
        assert!(
            engine.native_word_preparation_identity().is_none(),
            "{changed}"
        );
        assert!(
            !engine.native_word_preparation_edit_is_current(),
            "{changed}"
        );
    }
}

#[test]
fn native_document_space_refresh_rejects_changed_source_or_authority() {
    use crate::output::{EngineOutput, TestEngineOutput};
    use crate::space_autocorrect_prefetch;

    for changed in [
        "snapshot",
        "text",
        "selection",
        "revision",
        "focus",
        "lease",
        "layout",
        "sensitive",
        "gnome",
        "left_boundary",
        "right_boundary",
        "disabled",
    ] {
        let mut engine = managed_native_acquisition_suffix();
        engine.config.auto_replace = true;
        let frame = engine.capture_space_autocorrect_frame_identity().unwrap();
        match changed {
            "snapshot" => engine.client_context.surrounding_text_snapshot = None,
            "text" => {
                engine.client_context.surrounding_text_snapshot =
                    Some(SurroundingTextSnapshot::new("друг".into(), 4, 4))
            }
            "selection" => {
                engine
                    .client_context
                    .surrounding_text_snapshot
                    .as_mut()
                    .unwrap()
                    .anchor_pos = 0
            }
            "revision" => {
                engine.client_context.surrounding_observation_revision = engine
                    .client_context
                    .managed_commit_snapshot_floor
                    .unwrap()
                    .observation_revision
            }
            "focus" => engine.client_context.focus_serial += 1,
            "lease" => engine.client_context.runtime_owner_lease_identity += 1,
            "layout" => engine.layout_gesture.layout_generation += 1,
            "sensitive" => engine.client_context.content_purpose = 8,
            "gnome" => engine.config.layout_backend = "gnome".into(),
            "left_boundary" => {
                engine.client_context.surrounding_text_snapshot =
                    Some(SurroundingTextSnapshot::new("апров".into(), 5, 5))
            }
            "right_boundary" => {
                engine.client_context.surrounding_text_snapshot =
                    Some(SurroundingTextSnapshot::new("прова".into(), 4, 4))
            }
            "disabled" => engine.config.auto_replace = false,
            _ => unreachable!(),
        }
        let before =
            space_autocorrect_prefetch::take_with_budget(&frame, Duration::ZERO).worker_generation;
        let mut output = TestEngineOutput::default();
        zbus::block_on(
            engine.refresh_native_document_precognition(&mut EngineOutput::test(&mut output)),
        )
        .unwrap();
        let after =
            space_autocorrect_prefetch::take_with_budget(&frame, Duration::ZERO).worker_generation;
        assert_eq!(after, before, "{changed}");
        assert!(output.committed_texts.is_empty(), "{changed}");
    }
}

#[test]
fn native_document_managed_acquisition_suffix_is_readout_only() {
    let engine = managed_native_acquisition_suffix();
    let scope = engine.context_word_scope;
    let token = engine.context_token.clone();
    assert!(engine.context_observed_suffix_readout_is_current());
    let frame = engine.capture_observed_suffix_display_frame().unwrap();
    assert!(engine.precognition_identity_matches(&frame));
    assert_eq!(engine.context_word_scope, scope);
    assert_eq!(engine.context_token, token);
    assert!(!engine.context_word_is_known());
    assert!(!engine.context_observed_suffix_is_exact_current());
    assert!(engine.capture_input_frame_identity().is_none());
}

#[test]
fn native_document_managed_acquisition_readout_needs_fresh_actual_word() {
    for guard in [
        "snapshot",
        "text",
        "selection",
        "revision",
        "focus",
        "lease",
        "layout",
        "sensitive",
        "gnome",
    ] {
        let mut engine = managed_native_acquisition_suffix();
        match guard {
            "snapshot" => engine.client_context.surrounding_text_snapshot = None,
            "text" => {
                engine.client_context.surrounding_text_snapshot =
                    Some(SurroundingTextSnapshot::new("друг".into(), 4, 4))
            }
            "selection" => {
                engine
                    .client_context
                    .surrounding_text_snapshot
                    .as_mut()
                    .unwrap()
                    .anchor_pos = 0
            }
            "revision" => {
                engine.client_context.surrounding_observation_revision = engine
                    .client_context
                    .managed_commit_snapshot_floor
                    .unwrap()
                    .observation_revision
            }
            "focus" => engine.client_context.focus_serial += 1,
            "lease" => engine.client_context.runtime_owner_lease_identity += 1,
            "layout" => engine.layout_gesture.layout_generation += 1,
            "sensitive" => engine.client_context.content_purpose = 8,
            "gnome" => engine.config.layout_backend = "gnome".into(),
            _ => unreachable!(),
        }
        assert!(
            !engine.context_observed_suffix_readout_is_current(),
            "{guard}"
        );
        assert!(
            engine.capture_observed_suffix_display_frame().is_none(),
            "{guard}"
        );
        let mut observed = crate::output::TestEngineOutput::default();
        engine.remember_hint_geometry(68, 286, 0, 20);
        zbus::block_on(engine.prepare_inline_terminal_hint(
            &mut crate::output::EngineOutput::test(&mut observed),
            "ерка",
        ))
        .unwrap();
        assert!(observed.effects.is_empty(), "{guard}");
    }
}

#[test]
fn native_document_current_word_readout_does_not_authorize_an_older_prefix() {
    let mut engine = managed_native_acquisition_suffix();
    engine.committed_tail.buffer = "проверка\u{a0}пров".into();
    let actual = SurroundingTextSnapshot::new("проверка пров".into(), 13, 13);
    engine.observe_document_snapshot(Some(actual.clone()));
    engine
        .client_context
        .native_document_binding
        .as_mut()
        .unwrap()
        .hint_geometry = Some(NativeHintGeometry {
        snapshot: actual,
        rect: [168, 285, 0, 21],
    });
    let token = engine.context_token.clone();
    let scope = engine.context_word_scope;
    assert!(!engine.current_external_snapshot_agrees_with_owned_tail());
    assert!(!engine.native_manual_word_is_current());
    assert!(engine.exact_managed_surrounding_word_is_current());
    assert!(engine.native_managed_readout_is_current());
    assert!(engine.context_observed_suffix_readout_is_current());
    assert_eq!(engine.current_hint_rect(), Some([168, 285, 0, 21]));
    assert!(engine.capture_observed_suffix_display_frame().is_some());
    assert_eq!(engine.context_token, token);
    assert_eq!(engine.context_word_scope, scope);
    assert!(!engine.context_word_is_known());
    assert!(!engine.context_observed_suffix_is_exact_current());
    assert!(engine.capture_input_frame_identity().is_none());
    assert_eq!(engine.committed_tail.buffer, "проверка\u{a0}пров");
}

#[test]
fn native_document_known_active_word_manual_keeps_its_exact_span_and_live_owner() {
    use crate::context_admission::WordCompleteness;
    for changed in [
        "current",
        "unknown",
        "word",
        "left",
        "right",
        "selection",
        "epoch",
        "focus",
        "lease",
        "layout",
        "privacy",
        "sealed",
        "closed",
        "gnome",
    ] {
        let completeness = if changed == "unknown" {
            WordCompleteness::UnknownStart
        } else {
            WordCompleteness::KnownStart
        };
        let (mut engine, binding) = observed_native_engine_with_completeness(completeness);
        engine.set_client_capabilities(41);
        engine.committed_tail.buffer = "проверка\u{a0}привет".into();
        assert!(engine.install_native_document(Observation {
            binding: binding.for_result(&engine),
            snapshot: SurroundingTextSnapshot::new("проверка привет".into(), 15, 15),
        }));
        match changed {
            "current" | "unknown" => {}
            "word" => {
                engine.client_context.surrounding_text_snapshot = Some(
                    SurroundingTextSnapshot::new("проверка привел".into(), 15, 15),
                )
            }
            "left" => {
                engine.client_context.surrounding_text_snapshot = Some(
                    SurroundingTextSnapshot::new("проверкаxпривет".into(), 15, 15),
                )
            }
            "right" => {
                engine.client_context.surrounding_text_snapshot = Some(
                    SurroundingTextSnapshot::new("проверка приветx".into(), 15, 15),
                )
            }
            "selection" => {
                engine
                    .client_context
                    .surrounding_text_snapshot
                    .as_mut()
                    .unwrap()
                    .anchor_pos = 0
            }
            "epoch" => engine.committed_tail.epoch += 1,
            "focus" => engine.client_context.focus_serial += 1,
            "lease" => engine.client_context.runtime_owner_lease_identity += 1,
            "layout" => engine.layout_gesture.layout_generation += 1,
            "privacy" => {
                engine.client_context.content_purpose = super::super::IBUS_INPUT_PURPOSE_PASSWORD
            }
            "sealed" => engine.context_handoff_sealed = true,
            "closed" => {
                engine.committed_tail.buffer.push(' ');
                engine.client_context.surrounding_text_snapshot = Some(
                    SurroundingTextSnapshot::new("проверка привет ".into(), 16, 16),
                );
            }
            "gnome" => engine.config.layout_backend = "gnome".into(),
            _ => unreachable!(),
        }
        let scope = engine.context_word_scope;
        let token = engine.context_token.clone();
        let epoch = engine.committed_tail.epoch;
        assert!(
            !engine.current_external_snapshot_agrees_with_owned_tail(),
            "{changed}"
        );
        assert_eq!(
            engine.native_manual_word_is_current(),
            changed == "current",
            "{changed}"
        );
        assert_eq!(engine.context_word_scope, scope, "{changed}");
        assert_eq!(engine.context_token, token, "{changed}");
        assert_eq!(engine.committed_tail.epoch, epoch, "{changed}");
    }
}

fn native_layout_space_result(prefix: &str, right: &str) -> LayIbusEngine {
    use crate::engine::{PendingSystemOutcomeFeedback, SystemOutcomeKind};
    use lay::text_edit::VisibleTailSource;
    let (mut engine, binding) = observed_native_engine();
    engine.config.auto_switch_layout = true;
    engine.layout_gesture.layout_is_ru = true;
    engine.committed_tail.buffer = "ghjd ".into();
    let token = engine.live_context_token().unwrap();
    {
        let mut bridge = engine.begin_context_bridge_output(Some(&token));
        assert!(bridge.publish_tail_handoff());
        bridge.complete();
    }
    engine.client_context.native_document_binding = Some(binding.for_result(&engine));
    let caret = (prefix.chars().count() + 5) as u32;
    engine.arm_exact_visible_postcondition_from_surrounding_dispatch(
        Instant::now(),
        Some(PendingSystemOutcomeFeedback {
            source: VisibleTailSource::ImeCommittedTail,
            kind: SystemOutcomeKind::LayoutProjection,
            original: "пров ".into(),
            replacement: "ghjd ".into(),
        }),
        Some("ghjd ".into()),
        SurroundingTextSnapshot::new(format!("{prefix}ghjd {right}"), caret, caret),
    );
    assert!(engine.install_native_document(Observation {
        binding: binding.for_result(&engine),
        snapshot: SurroundingTextSnapshot::new(format!("{prefix}ghjd\u{a0}{right}"), caret, caret),
    }));
    engine
}

#[test]
fn native_document_layout_space_presentation_uses_the_existing_owner_and_full_verifier() {
    for (prefix, right, feedback) in [
        ("", "", true),
        ("", "", false),
        ("старое ", "\n", true),
        ("старое ", "\n", false),
    ] {
        let mut engine = native_layout_space_result(prefix, right);
        if !feedback {
            engine
                .committed_tail
                .pending_visible_postcondition
                .as_mut()
                .unwrap()
                .feedback = None;
        }
        let token = engine.context_token.clone();
        let scope = engine.context_word_scope;
        let epoch = engine.committed_tail.epoch;
        assert!(engine.acknowledge_native_layout_space());
        assert_eq!(engine.committed_tail.buffer, "ghjd\u{a0}");
        assert_eq!(
            engine.shared.lock().unwrap().handoff_tail_buffer,
            "ghjd\u{a0}"
        );
        assert_eq!(engine.committed_tail.epoch, epoch);
        assert_eq!(engine.context_token, token);
        assert_eq!(engine.context_word_scope, scope);
        assert!(!engine.acknowledge_native_layout_space());
        engine.observe_native_document_postcondition();
        assert!(engine
            .committed_tail
            .pending_visible_postcondition
            .is_none());
        assert_eq!(engine.committed_tail.epoch, epoch + 1);
        assert!(!engine.layout_gesture.layout_is_ru);
        assert!(!engine.context_word_is_known());
    }
}

#[test]
fn native_document_layout_space_presentation_rejects_other_field_or_effect_changes() {
    use crate::engine::SystemOutcomeKind;
    for changed in [
        "text",
        "prefix",
        "right",
        "selection",
        "caret",
        "epoch",
        "field",
        "focus",
        "lease",
        "layout",
        "privacy",
        "sealed",
        "gnome",
        "feedback",
        "no_layout",
        "no_native",
        "shared",
    ] {
        let mut engine = native_layout_space_result("старое ", "\n");
        match changed {
            "no_layout" => {
                engine
                    .committed_tail
                    .pending_visible_postcondition
                    .as_mut()
                    .unwrap()
                    .layout_sync_text = None
            }
            "text" => engine
                .client_context
                .surrounding_text_snapshot
                .as_mut()
                .unwrap()
                .text
                .replace_range(13..14, "x"),
            "prefix" => engine
                .client_context
                .surrounding_text_snapshot
                .as_mut()
                .unwrap()
                .text
                .insert(0, 'x'),
            "right" => engine
                .client_context
                .surrounding_text_snapshot
                .as_mut()
                .unwrap()
                .text
                .push('x'),
            "selection" => {
                engine
                    .client_context
                    .surrounding_text_snapshot
                    .as_mut()
                    .unwrap()
                    .anchor_pos = 0
            }
            "caret" => {
                engine
                    .client_context
                    .surrounding_text_snapshot
                    .as_mut()
                    .unwrap()
                    .cursor_pos -= 1
            }
            "epoch" => engine.committed_tail.epoch += 1,
            "field" => {
                engine
                    .client_context
                    .native_document_binding
                    .as_mut()
                    .unwrap()
                    .identity
                    .field = "/other".try_into().unwrap()
            }
            "focus" => engine.client_context.focus_serial += 1,
            "lease" => engine.client_context.runtime_owner_lease_identity += 1,
            "layout" => engine.layout_gesture.layout_generation += 1,
            "privacy" => {
                engine.client_context.content_purpose = super::super::IBUS_INPUT_PURPOSE_PASSWORD
            }
            "sealed" => engine.context_handoff_sealed = true,
            "gnome" => engine.config.layout_backend = "gnome".into(),
            "feedback" => {
                engine
                    .committed_tail
                    .pending_visible_postcondition
                    .as_mut()
                    .unwrap()
                    .feedback
                    .as_mut()
                    .unwrap()
                    .kind = SystemOutcomeKind::Correction
            }
            "no_native" => {
                engine
                    .committed_tail
                    .pending_visible_postcondition
                    .as_mut()
                    .unwrap()
                    .native_document = None
            }
            "shared" => engine.shared.lock().unwrap().handoff_tail_buffer.push('x'),
            _ => unreachable!(),
        }
        let suffix = engine
            .committed_tail
            .pending_visible_postcondition
            .as_ref()
            .unwrap()
            .expected_suffix
            .clone();
        let expected = engine
            .committed_tail
            .pending_visible_postcondition
            .as_ref()
            .unwrap()
            .expected_external_snapshot
            .clone();
        assert!(!engine.acknowledge_native_layout_space(), "{changed}");
        assert_eq!(engine.committed_tail.buffer, "ghjd ", "{changed}");
        assert_eq!(
            engine
                .committed_tail
                .pending_visible_postcondition
                .as_ref()
                .unwrap()
                .expected_suffix,
            suffix,
            "{changed}"
        );
        assert_eq!(
            engine
                .committed_tail
                .pending_visible_postcondition
                .as_ref()
                .unwrap()
                .expected_external_snapshot,
            expected,
            "{changed}"
        );
    }
}

#[test]
fn native_document_owned_closing_space_keeps_scope_and_enables_the_same_manual_range() {
    let mut engine = native_layout_space_result("старое ", "\n");
    // Actual Chrome advertises capability41 in the retained native wire. A
    // native document observation itself must never manufacture that flag.
    engine.set_client_capabilities(41);
    assert!(!engine.acknowledge_native_closing_space());
    engine.committed_tail.pending_visible_postcondition = None;
    let token = engine.context_token.clone();
    let scope = engine.context_word_scope;
    let epoch = engine.committed_tail.epoch;
    assert!(!engine.native_manual_word_is_current());
    assert!(engine.acknowledge_native_closing_space());
    assert_eq!(engine.committed_tail.buffer, "ghjd\u{a0}");
    assert_eq!(
        engine.shared.lock().unwrap().handoff_tail_buffer,
        "ghjd\u{a0}"
    );
    assert_eq!(engine.context_token, token);
    assert_eq!(engine.context_word_scope, scope);
    assert_eq!(engine.committed_tail.epoch, epoch);
    assert!(engine.layout_gesture.layout_is_ru);
    assert!(!engine.context_word_is_known());
    assert!(engine.native_manual_word_is_current());
    assert!(engine
        .committed_tail
        .pending_visible_postcondition
        .is_none());
    assert!(!engine.acknowledge_native_closing_space());
}

fn native_owned_append_after_closing_space(
    closing: char,
    observed: bool,
    settle_in_observer: bool,
) -> (
    LayIbusEngine,
    NativeDocumentBinding,
    crate::output::TestEngineOutput,
) {
    use crate::engine::WordInputMode;
    use crate::output::{EngineOutput, TestEngineOutput};
    let mut engine = managed_native_acquisition_suffix();
    engine.config.auto_replace = false;
    let binding = engine
        .client_context
        .native_document_binding
        .clone()
        .unwrap();
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    zbus::block_on(engine.commit_managed_passthrough_char(
        &mut EngineOutput::test(&mut output),
        ' ',
        57,
    ))
    .unwrap();
    if observed {
        assert!(engine.install_native_document(Observation {
            binding: binding.for_result(&engine),
            snapshot: SurroundingTextSnapshot::new(format!("пров{closing}"), 5, 5),
        }));
        if settle_in_observer {
            engine.observe_native_document_postcondition();
        }
        assert_eq!(engine.committed_tail.buffer, format!("пров{closing}"));
        assert_eq!(
            engine
                .client_context
                .managed_word_start
                .as_ref()
                .map(|witness| witness.anchored_token_chars()),
            Some(0),
            "an actual boundary must be captured before the first next-word key"
        );
    }
    engine.composition.word_input_mode = Some(WordInputMode::ManagedCommit);
    zbus::block_on(engine.commit_managed_passthrough_char(
        &mut EngineOutput::test(&mut output),
        'е',
        18,
    ))
    .unwrap();
    (engine, binding, output)
}

#[test]
fn native_owned_word_separator_reflow_preserves_the_exact_pair_scope() {
    use crate::output::EngineOutput;

    for (closing, internal, partial, delayed, settle_in_observer) in [
        (' ', ' ', false, false, true),
        ('\u{a0}', '\u{a0}', false, false, true),
        ('\u{a0}', ' ', false, false, true),
        ('\u{a0}', ' ', true, false, true),
        ('\u{a0}', ' ', true, true, true),
        (' ', ' ', false, false, false),
        ('\u{a0}', ' ', false, false, false),
    ] {
        let (mut engine, binding, mut output) =
            native_owned_append_after_closing_space(closing, !delayed, settle_in_observer);
        let epoch = engine.committed_tail.epoch;
        let token = engine.context_token.clone();
        let scope = engine.context_word_scope;
        if partial {
            let boundary = if delayed { closing } else { internal };
            let actual = SurroundingTextSnapshot::new(format!("пров{boundary}"), 5, 5);
            assert!(engine.install_native_document(Observation {
                binding: binding.for_result(&engine),
                snapshot: actual.clone(),
            }));
            engine.observe_native_document_postcondition();
            assert_eq!(
                engine.client_context.surrounding_text_snapshot,
                Some(actual)
            );
            assert!(
                engine.native_word_preparation_identity().is_some(),
                "delayed={delayed} witness={:?} scope={:?} floor={:?}",
                engine.client_context.managed_word_start,
                engine.context_word_scope,
                engine.client_context.managed_commit_snapshot_floor
            );
            assert!(!engine.native_word_preparation_edit_is_current());
        }
        let actual = SurroundingTextSnapshot::new(format!("пров{internal}е"), 6, 6);
        assert!(engine.install_native_document(Observation {
            binding: binding.for_result(&engine),
            snapshot: actual.clone(),
        }));
        engine.observe_native_document_postcondition();
        assert_eq!(output.committed_texts, [" ", "е"]);
        assert_eq!(
            engine.client_context.surrounding_text_snapshot,
            Some(actual.clone())
        );
        assert_eq!(engine.committed_tail.epoch, epoch);
        assert_eq!(engine.context_token, token);
        assert_eq!(engine.context_word_scope, scope);
        assert_eq!(
            engine.shared.lock().unwrap().handoff_tail_buffer,
            engine.committed_tail.buffer
        );
        let pair = engine.capture_space_boundary_pair_scope().unwrap_or_else(|| {
            panic!("pair lost for {closing:?}->{internal:?}, partial={partial}: tail={:?}, snapshot={:?}", engine.committed_tail.buffer, engine.client_context.surrounding_text_snapshot)
        });
        assert_eq!(pair.text, format!("пров{internal}е"));

        // Space can read this same field one owned letter behind. Reuse the
        // existing source-only resample, never a projected edit permission.
        assert!(!engine.native_owned_append_observation_is_pending(&actual));
        zbus::block_on(engine.commit_managed_passthrough_char(
            &mut EngineOutput::test(&mut output),
            'щ',
            32,
        ))
        .unwrap();
        assert!(engine.capture_space_boundary_pair_scope().is_some());
        assert!(!engine.native_word_preparation_edit_is_current());
        assert!(engine.native_owned_append_observation_is_pending(&actual));
        for rejected in [
            SurroundingTextSnapshot::new(format!("пров{internal}ещ"), 7, 7),
            SurroundingTextSnapshot::new(format!("пров{internal}е"), 6, 5),
            SurroundingTextSnapshot::new(format!("пров{internal}е хвост"), 6, 6),
            SurroundingTextSnapshot::new(format!("иной{internal}е"), 6, 6),
        ] {
            assert!(!engine.native_owned_append_observation_is_pending(&rejected));
        }
        assert_eq!(output.committed_texts, [" ", "е", "щ"]);
        assert!(output.surrounding_deletes.is_empty());
        assert!(output.forwarded_keys.is_empty());

        let witness_identity = engine.native_word_preparation_identity().unwrap();
        let pair = engine.capture_space_boundary_pair_scope().unwrap();
        for reason in [
            "native_document_changed",
            "native_deadline",
            "native_document_shape",
            "native_desktop_unobserved",
        ] {
            let epoch = engine.committed_tail.epoch;
            let token = engine.context_token.clone();
            let scope = engine.context_word_scope;
            assert!(engine.native_observation_failed(reason));
            assert!(engine.client_context.surrounding_text_snapshot.is_none());
            assert!(!engine.native_word_preparation_edit_is_current());
            assert!(engine.space_pair_client_snapshot(&pair).is_none());
            assert_eq!(
                engine.text_target_edit_route(7),
                TextTargetEditRoute::Unsupported
            );
            assert_eq!(
                engine
                    .client_context
                    .managed_word_start
                    .as_ref()
                    .map(|w| w.identity),
                Some(witness_identity),
                "same-owner transient read lost inert append provenance: {reason}"
            );
            assert_eq!(engine.committed_tail.epoch, epoch);
            assert_eq!(engine.context_token, token);
            assert_eq!(engine.context_word_scope, scope);
            assert!(engine.install_native_document(Observation {
                binding: binding.for_result(&engine),
                snapshot: SurroundingTextSnapshot::new(format!("пров{internal}ещ"), 7, 7),
            }));
            engine.observe_native_document_postcondition();
            assert_eq!(
                engine.native_word_preparation_identity(),
                Some(witness_identity)
            );
            assert!(engine.native_word_preparation_edit_is_current());
            assert_eq!(
                engine.capture_space_boundary_pair_scope(),
                Some(pair.clone())
            );
        }
        assert!(engine.native_observation_failed("native_document_changed"));
        assert!(engine.install_native_document(Observation {
            binding: binding.for_result(&engine),
            snapshot: SurroundingTextSnapshot::new(format!("иной{internal}ещ"), 7, 7),
        }));
        assert!(engine.native_word_preparation_identity().is_none());
        assert!(engine.capture_space_boundary_pair_scope().is_none());
        assert!(engine.space_pair_client_snapshot(&pair).is_none());
        assert_eq!(output.committed_texts, [" ", "е", "щ"]);
        assert!(output.surrounding_deletes.is_empty());
    }
}

#[test]
fn native_owned_word_separator_reflow_rejects_foreign_or_unproven_changes() {
    for late in [false, true] {
        for changed in [
            "word",
            "prefix",
            "separator_position",
            "multi_space",
            "newline",
            "selection",
            "caret",
            "invalid_caret",
            "no_witness",
            "no_floor",
            "floor_epoch",
            "floor_owner",
            "floor_layout",
            "future_revision",
            "no_native_owner",
            "field",
            "focus",
            "layout",
            "shared",
            "sensitive",
            "sealed",
            "composition",
            "mode",
            "unmanaged",
            "pending",
            "gnome",
        ] {
            // Missing prior witness is the positive late-boundary case above.
            if late && changed == "no_witness" {
                continue;
            }
            let (mut engine, mut binding, output) =
                native_owned_append_after_closing_space('\u{a0}', !late, true);
            let mut actual = if late {
                SurroundingTextSnapshot::new("пров\u{a0}".into(), 5, 5)
            } else {
                SurroundingTextSnapshot::new("пров е".into(), 6, 6)
            };
            match changed {
                "word" => actual.text = "пров ж".into(),
                "prefix" => actual.text = "тров е".into(),
                "separator_position" => actual.text = "про ве".into(),
                "multi_space" => actual = SurroundingTextSnapshot::new("пров  е".into(), 7, 7),
                "newline" => actual.text = "пров\nе".into(),
                "selection" => actual.anchor_pos = 0,
                "caret" => {
                    actual.cursor_pos -= 1;
                    actual.anchor_pos = actual.cursor_pos;
                }
                "invalid_caret" => {
                    actual.cursor_pos = 99;
                    actual.anchor_pos = 99;
                }
                "no_witness" => engine.client_context.managed_word_start = None,
                "no_floor" => engine.client_context.managed_commit_snapshot_floor = None,
                "floor_epoch" => {
                    engine
                        .client_context
                        .managed_commit_snapshot_floor
                        .as_mut()
                        .unwrap()
                        .tail_epoch = 0
                }
                "floor_owner" => {
                    engine
                        .client_context
                        .managed_commit_snapshot_floor
                        .as_mut()
                        .unwrap()
                        .runtime_owner_lease_identity += 1
                }
                "floor_layout" => {
                    engine
                        .client_context
                        .managed_commit_snapshot_floor
                        .as_mut()
                        .unwrap()
                        .layout_generation += 1
                }
                "future_revision" => {
                    engine
                        .client_context
                        .managed_commit_snapshot_floor
                        .as_mut()
                        .unwrap()
                        .observation_revision =
                        engine.client_context.surrounding_observation_revision + 1
                }
                "no_native_owner" => engine.client_context.native_document_binding = None,
                "field" => binding.identity.field = "/foreign_field".try_into().unwrap(),
                "focus" => engine.client_context.focus_serial += 1,
                "layout" => engine.layout_gesture.layout_generation += 1,
                "shared" => engine.shared.lock().unwrap().handoff_tail_buffer = "foreign".into(),
                "sensitive" => engine.client_context.content_purpose = 8,
                "sealed" => engine.context_handoff_sealed = true,
                "composition" => engine.composition.buffer = "е".into(),
                "mode" => engine.composition.word_input_mode = None,
                "unmanaged" => engine.client_context.managed_input = false,
                "pending" => engine.arm_exact_visible_postcondition_from_surrounding_dispatch(
                    Instant::now(),
                    None,
                    None,
                    SurroundingTextSnapshot::new("пров\u{a0}е".into(), 6, 6),
                ),
                "gnome" => engine.config.layout_backend = "gnome".into(),
                _ => unreachable!(),
            }
            engine.install_native_document(Observation {
                binding: binding.for_result(&engine),
                snapshot: actual.clone(),
            });
            engine.observe_native_document_postcondition();
            let normalized = if late {
                "пров\u{a0}е"
            } else {
                "пров е"
            };
            assert_ne!(
                engine.committed_tail.buffer, normalized,
                "late={late} {changed}"
            );
            assert!(
                engine.capture_space_boundary_pair_scope().is_none(),
                "{changed}"
            );
            assert!(
                !engine.native_owned_append_observation_is_pending(&actual),
                "a rejected pair cannot request a reread: late={late} {changed}"
            );
            assert_eq!(output.committed_texts, [" ", "е"], "{changed}");
        }
    }
}

#[test]
fn native_document_owned_closing_space_rejects_other_slots_and_stale_ownership() {
    for changed in [
        "pending",
        "word",
        "older_slot",
        "selection",
        "epoch",
        "focus",
        "lease",
        "layout",
        "privacy",
        "sealed",
        "gnome",
        "shared",
        "no_binding",
    ] {
        let mut engine = native_layout_space_result("", "");
        if changed != "pending" {
            engine.committed_tail.pending_visible_postcondition = None;
        }
        match changed {
            "pending" => {}
            "word" => engine
                .client_context
                .surrounding_text_snapshot
                .as_mut()
                .unwrap()
                .text
                .replace_range(0..1, "x"),
            "older_slot" => {
                engine.committed_tail.buffer = "g h ".into();
                engine.shared.lock().unwrap().handoff_tail_buffer = "g h ".into();
                engine.client_context.surrounding_text_snapshot =
                    Some(SurroundingTextSnapshot::new("g\u{a0}h\u{a0}".into(), 4, 4));
            }
            "selection" => {
                engine
                    .client_context
                    .surrounding_text_snapshot
                    .as_mut()
                    .unwrap()
                    .anchor_pos = 0
            }
            "epoch" => engine.committed_tail.epoch += 1,
            "focus" => engine.client_context.focus_serial += 1,
            "lease" => engine.client_context.runtime_owner_lease_identity += 1,
            "layout" => engine.layout_gesture.layout_generation += 1,
            "privacy" => {
                engine.client_context.content_purpose = super::super::IBUS_INPUT_PURPOSE_PASSWORD
            }
            "sealed" => engine.context_handoff_sealed = true,
            "gnome" => engine.config.layout_backend = "gnome".into(),
            "shared" => engine.shared.lock().unwrap().handoff_tail_buffer.push('x'),
            "no_binding" => engine.client_context.native_document_binding = None,
            _ => unreachable!(),
        }
        let before = engine.committed_tail.buffer.clone();
        assert!(!engine.acknowledge_native_closing_space(), "{changed}");
        assert_eq!(engine.committed_tail.buffer, before, "{changed}");
    }
}

#[test]
fn native_document_managed_hint_requires_actual_current_character_geometry() {
    let mut engine = managed_native_acquisition_suffix();
    engine.remember_hint_geometry(68, 286, 0, 20);
    assert!(
        engine.native_display_observation_needed(),
        "a real callback may read now-valid bounds"
    );
    assert_eq!(
        engine.current_hint_rect(),
        None,
        "a previous key's IBus caret must not cover the new letter"
    );
    let token = engine.context_token.clone();
    let scope = engine.context_word_scope;
    let snapshot = engine
        .client_context
        .surrounding_text_snapshot
        .clone()
        .unwrap();
    let rect = bus::character_end_rect([67, 285, 11, 21]).unwrap();
    engine
        .client_context
        .native_document_binding
        .as_mut()
        .unwrap()
        .hint_geometry = Some(NativeHintGeometry {
        snapshot: snapshot.clone(),
        rect,
    });
    assert_eq!(engine.current_hint_rect(), Some([78, 285, 0, 21]));
    assert!(
        !engine.native_display_observation_needed(),
        "current readout and bounds need no extra read"
    );
    assert_eq!(engine.context_token, token);
    assert_eq!(engine.context_word_scope, scope);
    assert!(!engine.context_word_is_known());
    assert!(engine.capture_input_frame_identity().is_none());
    assert!(!engine.context_observed_suffix_is_exact_current());
    engine
        .client_context
        .surrounding_text_snapshot
        .as_mut()
        .unwrap()
        .cursor_pos = 3;
    assert_eq!(
        engine.current_hint_rect(),
        None,
        "a later caret cannot reuse old native bounds"
    );
    engine.client_context.surrounding_text_snapshot = Some(snapshot);
    let binding = engine
        .client_context
        .native_document_binding
        .clone()
        .unwrap();
    engine.client_context.native_document_binding = Some(binding.for_result(&engine));
    assert_eq!(
        engine.current_hint_rect(),
        None,
        "transport reuse must discard character geometry"
    );
    assert!(engine.native_display_observation_needed());
    engine.config.layout_backend = "gnome".into();
    assert!(!engine.native_display_observation_needed());
    assert_eq!(
        engine.current_hint_rect(),
        Some([68, 286, 0, 20]),
        "GNOME keeps the existing geometry path"
    );
}

#[test]
fn native_document_character_geometry_rejects_unavailable_or_invalid_bounds() {
    for rect in [
        [4, 92, 0, 0],
        [4, 92, 0, 21],
        [1, 2, -1, 20],
        [1, 2, 12, 513],
        [i32::MAX, 2, 1, 20],
        [100_000, 2, 1, 20],
    ] {
        assert_eq!(bus::character_end_rect(rect), None, "{rect:?}");
    }
    assert_eq!(
        bus::character_end_rect([-30, 12, 10, 20]),
        Some([-20, 12, 0, 20])
    );
}

#[test]
fn native_document_duplicate_known_word_read_preserves_the_display_frame() {
    use crate::context_admission::WordCompleteness;
    use crate::engine::WordInputMode;
    use crate::output::{EngineOutput, TestEngineOutput};

    let (mut engine, binding) =
        observed_native_engine_with_completeness(WordCompleteness::KnownStart);
    engine.committed_tail.buffer = "тест про".into();
    engine.config.nanda_precognition = true;
    engine.config.text_backend = "ime".into();
    engine.layout_gesture.layout_is_ru = true;
    engine.set_client_capabilities(41);
    engine.composition.word_input_mode = Some(WordInputMode::ManagedCommit);
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    zbus::block_on(engine.commit_managed_passthrough_char(
        &mut EngineOutput::test(&mut output),
        'в',
        32,
    ))
    .unwrap();
    assert_eq!(output.committed_texts, ["в"]);
    let snapshot = SurroundingTextSnapshot::new("тест пров".into(), 9, 9);
    engine.observe_document_snapshot(Some(snapshot.clone()));
    engine.client_context.native_document_binding = Some(binding.for_result(&engine));
    let frame = engine.capture_input_frame_identity().unwrap();
    assert!(frame.space_boundary_pair.is_some());
    engine
        .client_context
        .native_document_binding
        .as_mut()
        .unwrap()
        .last_display_frame = Some(frame.clone());
    let revision = engine.client_context.surrounding_observation_revision;
    let token = engine.context_token.clone();
    let current = engine
        .client_context
        .native_document_binding
        .clone()
        .unwrap();

    assert!(engine.install_native_document(Observation {
        binding: current.clone(),
        snapshot: snapshot.clone(),
    }));
    assert_eq!(
        engine.client_context.surrounding_observation_revision,
        revision
    );
    assert_eq!(engine.capture_input_frame_identity(), Some(frame.clone()));
    assert_eq!(engine.context_token, token);
    assert!(engine.native_document_is_current());

    // Identical text after input changed is a new receipt, not a duplicate.
    engine.committed_tail.epoch += 1;
    assert!(engine.install_native_document(Observation {
        binding: current.for_result(&engine),
        snapshot,
    }));
    assert!(engine.client_context.surrounding_observation_revision > revision);
    assert_ne!(engine.capture_input_frame_identity(), Some(frame));
}

#[test]
fn native_document_deferred_result_keeps_the_next_layout_handoff_epoch() {
    use crate::context_admission::ExactManualSourceProvenance;

    for already_in_bridge in [false, true] {
        // The first exact edit has dispatched. Its actual native result arrives
        // later, outside that RPC's guard, without another physical key.
        let (mut engine, binding) = observed_native_engine();
        engine.config.auto_switch_layout = true;
        engine.layout_gesture.layout_is_ru = true;
        engine.committed_tail.buffer = "ghjd".into();
        let actual = SurroundingTextSnapshot::new("ghjd".into(), 4, 4);
        engine.arm_exact_visible_postcondition_from_surrounding_dispatch(
            Instant::now(),
            None,
            Some("ghjd".into()),
            actual.clone(),
        );
        assert!(engine.install_native_document(Observation {
            binding: binding.for_result(&engine),
            snapshot: actual,
        }));
        let token = engine.live_context_token().unwrap();
        let scope = engine.context_word_scope;
        let epoch = engine.committed_tail.epoch;
        assert!(engine.context_bridge_token.is_none());
        if already_in_bridge {
            let mut bridge = engine.begin_context_bridge_output(Some(&token));
            bridge.observe_native_document_postcondition();
            assert_eq!(bridge.context_bridge_token.as_ref(), Some(&token));
            bridge.complete();
        } else {
            engine.observe_native_document_postcondition();
        }
        assert!(engine
            .committed_tail
            .pending_visible_postcondition
            .is_none());
        assert_eq!(engine.committed_tail.epoch, epoch + 1);
        assert_eq!(engine.context_word_scope, scope);
        assert!(!engine.context_word_is_known());
        assert!(engine.context_bridge_token.is_none());
        // The reducer must recognize exactly the epoch that the source will
        // seal on FocusOut. Matching text cannot repair an unreported epoch.
        assert!(
            engine
                .context_admission
                .as_ref()
                .unwrap()
                .publish_exact_manual_snapshot(
                    &token,
                    engine.committed_tail.epoch,
                    engine.committed_tail.buffer.clone(),
                    Some(String::new()),
                    engine.client_context.surrounding_observation_revision,
                    Instant::now() + Duration::from_secs(1),
                    ExactManualSourceProvenance::NativeDocument,
                ),
            "deferred native result left the published epoch outside admission"
        );
    }
}

#[test]
fn native_document_deferred_result_cannot_publish_from_a_stale_observation() {
    for changed in [
        "epoch",
        "focus",
        "lease",
        "layout",
        "field",
        "privacy",
        "text",
        "selection",
        "gnome",
    ] {
        let (mut engine, binding) = observed_native_engine();
        engine.config.auto_switch_layout = true;
        engine.layout_gesture.layout_is_ru = true;
        engine.committed_tail.buffer = "ghjd".into();
        let actual = SurroundingTextSnapshot::new("ghjd".into(), 4, 4);
        engine.arm_exact_visible_postcondition_from_surrounding_dispatch(
            Instant::now(),
            None,
            Some("ghjd".into()),
            actual.clone(),
        );
        assert!(engine.install_native_document(Observation {
            binding: binding.for_result(&engine),
            snapshot: actual,
        }));
        match changed {
            "epoch" => engine.committed_tail.epoch += 1,
            "focus" => engine.client_context.focus_serial += 1,
            "lease" => engine.client_context.runtime_owner_lease_identity += 1,
            "layout" => engine.layout_gesture.layout_generation += 1,
            "field" => {
                engine
                    .client_context
                    .native_document_binding
                    .as_mut()
                    .unwrap()
                    .identity
                    .field = "/other".try_into().unwrap()
            }
            "privacy" => {
                engine.client_context.content_purpose = super::super::IBUS_INPUT_PURPOSE_PASSWORD
            }
            "text" => {
                engine
                    .client_context
                    .surrounding_text_snapshot
                    .as_mut()
                    .unwrap()
                    .text = "друг".into()
            }
            "selection" => {
                engine
                    .client_context
                    .surrounding_text_snapshot
                    .as_mut()
                    .unwrap()
                    .anchor_pos = 0
            }
            "gnome" => engine.config.layout_backend = "gnome".into(),
            _ => unreachable!(),
        }
        let epoch = engine.committed_tail.epoch;
        let scope = engine.context_word_scope;
        engine.observe_native_document_postcondition();
        assert_eq!(engine.committed_tail.epoch, epoch, "{changed}");
        assert_eq!(engine.context_word_scope, scope, "{changed}");
        assert!(engine.context_bridge_token.is_none(), "{changed}");
        assert!(engine.layout_gesture.layout_is_ru, "{changed}");
    }
}

#[test]
fn native_document_late_known_word_starts_existing_hint_worker_once() {
    use crate::context_admission::WordCompleteness;
    use crate::output::EngineOutput;

    zbus::block_on(async {
        let (mut engine, binding) =
            observed_native_engine_with_completeness(WordCompleteness::KnownStart);
        engine.committed_tail.buffer = "пров".into();
        engine.config.nanda_precognition = true;
        engine.config.text_backend = "ime".into();
        engine.layout_gesture.layout_is_ru = true;
        engine.set_client_capabilities(41);
        assert!(engine.context_word_is_known());
        assert!(engine.capture_input_frame_identity().is_some());
        assert!(
            engine.native_document_is_current(),
            "native binding must be current before the wake"
        );
        assert!(
            engine.native_manual_word_is_current(),
            "the actual snapshot must match the current word"
        );
        assert!(
            engine.input_frame_identity_matches(&engine.capture_input_frame_identity().unwrap())
        );
        assert!(!engine.composition.preedit_display_only_pending);
        let token = engine.context_token.clone();
        let revision = engine.client_context.surrounding_observation_revision;
        let emitter =
            zbus::object_server::SignalEmitter::new(&binding.connection, engine.path.clone())
                .unwrap();
        let mut output = EngineOutput::legacy(&emitter);
        engine.reset_precognition_causal_counts();
        engine
            .refresh_native_document_precognition(&mut output)
            .await
            .unwrap();
        assert_eq!(engine.precognition_causal_counts(), (1, 0));
        assert!(engine.composition.preedit_display_only_pending);
        assert!(engine.composition.buffer.is_empty());
        assert_eq!(engine.committed_tail.buffer, "пров");
        assert_eq!(engine.context_token, token);
        assert_eq!(
            engine.client_context.surrounding_observation_revision,
            revision
        );
        engine
            .refresh_native_document_precognition(&mut output)
            .await
            .unwrap();
        assert_eq!(engine.precognition_causal_counts(), (1, 0));
        engine.cancel_precognition_display_generation();
        engine.composition.preedit_display_only_pending = false;
        engine
            .refresh_native_document_precognition(&mut output)
            .await
            .unwrap();
        assert_eq!(
            engine.precognition_causal_counts(),
            (1, 0),
            "an unchanged native event must not resurrect cancelled display work"
        );
        engine.committed_tail.epoch += 1;
        engine.client_context.native_document_binding = Some(binding.for_result(&engine));
        engine
            .refresh_native_document_precognition(&mut output)
            .await
            .unwrap();
        assert_eq!(engine.precognition_causal_counts(), (2, 0));
        engine.cancel_precognition_display_generation();
    });
}

#[test]
fn native_document_late_known_hint_needs_actual_current_unselected_word() {
    use crate::context_admission::WordCompleteness;
    use crate::output::EngineOutput;
    for guard in [
        "snapshot",
        "different_text",
        "selection",
        "focus",
        "lease",
        "layout",
        "inactive",
        "sensitive",
        "composition",
        "gnome",
    ] {
        zbus::block_on(async {
            let (mut engine, binding) =
                observed_native_engine_with_completeness(WordCompleteness::KnownStart);
            engine.committed_tail.buffer = "пров".into();
            engine.config.nanda_precognition = true;
            engine.config.text_backend = "ime".into();
            engine.layout_gesture.layout_is_ru = true;
            engine.set_client_capabilities(41);
            match guard {
                "snapshot" => engine.client_context.surrounding_text_snapshot = None,
                "different_text" => {
                    engine.client_context.surrounding_text_snapshot =
                        Some(SurroundingTextSnapshot::new("друг".into(), 4, 4))
                }
                "selection" => {
                    engine
                        .client_context
                        .surrounding_text_snapshot
                        .as_mut()
                        .unwrap()
                        .anchor_pos = 0
                }
                "focus" => engine.client_context.focus_serial += 1,
                "lease" => engine.client_context.runtime_owner_lease_identity += 1,
                "layout" => engine.layout_gesture.layout_generation += 1,
                "inactive" => engine.shared.lock().unwrap().active_path = Some("/other".into()),
                "sensitive" => {
                    engine.client_context.content_purpose =
                        super::super::IBUS_INPUT_PURPOSE_PASSWORD
                }
                "composition" => engine.composition.buffer = "пров".into(),
                "gnome" => engine.config.layout_backend = "gnome".into(),
                _ => unreachable!(),
            }
            let token = engine.context_token.clone();
            let revision = engine.client_context.surrounding_observation_revision;
            let emitter =
                zbus::object_server::SignalEmitter::new(&binding.connection, engine.path.clone())
                    .unwrap();
            engine.reset_precognition_causal_counts();
            engine
                .refresh_native_document_precognition(&mut EngineOutput::legacy(&emitter))
                .await
                .unwrap();
            assert_eq!(engine.precognition_causal_counts(), (0, 0), "{guard}");
            assert_eq!(engine.context_token, token, "{guard}");
            assert_eq!(
                engine.client_context.surrounding_observation_revision, revision,
                "{guard}"
            );
        });
    }
}

#[test]
fn native_document_preserved_literal_rejoins_managed_input_and_commits_once() {
    use crate::engine::WordInputMode;
    use crate::output::{EngineOutput, TestEngineOutput};

    let (mut engine, _) = observed_native_engine();
    engine.set_client_capabilities(41);
    engine.committed_tail.buffer = "пров".into();
    engine.composition.buffer = "пров".into();
    engine.composition.cursor = 4;
    engine.composition.legacy_word_preedit_active = true;
    engine.composition.preedit_visible = true;
    // Ready preserves this exact literal preedit but clears word_input_mode.
    engine.composition.word_input_mode = None;
    let token = engine.context_token.clone();
    let revision = engine.client_context.surrounding_observation_revision;
    let mut observed = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    engine.settle_native_literal_input_mode();
    assert_eq!(
        engine.composition.word_input_mode,
        Some(WordInputMode::ManagedCommit)
    );
    assert!(zbus::block_on(
        engine.commit_observed_literal_preedit(&mut EngineOutput::test(&mut observed))
    )
    .unwrap());
    assert!(!zbus::block_on(
        engine.commit_observed_literal_preedit(&mut EngineOutput::test(&mut observed))
    )
    .unwrap());
    assert_eq!(observed.committed_texts, ["пров"]);
    assert!(observed.surrounding_deletes.is_empty());
    assert!(engine.composition.buffer.is_empty());
    assert!(!engine.composition.legacy_word_preedit_active);
    assert_eq!(engine.context_token, token);
    assert!(engine.client_context.surrounding_observation_revision >= revision);
    assert!(engine.client_context.surrounding_text_snapshot.is_none());
    assert_eq!(
        engine.text_target_edit_route(4),
        TextTargetEditRoute::Unsupported
    );
}

#[test]
fn native_document_missing_mode_needs_actual_current_owned_literal() {
    use crate::engine::WordInputMode;
    for guard in [
        "no_snapshot",
        "different_text",
        "selection",
        "focus",
        "unowned",
        "terminal_mode",
        "gnome",
    ] {
        let (mut engine, _) = observed_native_engine();
        engine.set_client_capabilities(41);
        engine.committed_tail.buffer = "пров".into();
        engine.composition.buffer = "пров".into();
        engine.composition.cursor = 4;
        engine.composition.legacy_word_preedit_active = true;
        engine.composition.preedit_visible = true;
        engine.composition.word_input_mode = None;
        match guard {
            "no_snapshot" => engine.client_context.surrounding_text_snapshot = None,
            "different_text" => {
                engine.client_context.surrounding_text_snapshot =
                    Some(SurroundingTextSnapshot::new("друг".into(), 4, 4))
            }
            "selection" => {
                engine
                    .client_context
                    .surrounding_text_snapshot
                    .as_mut()
                    .unwrap()
                    .anchor_pos = 0
            }
            "focus" => engine.client_context.focus_serial += 1,
            "unowned" => engine.composition.legacy_word_preedit_active = false,
            "terminal_mode" => {
                engine.composition.word_input_mode = Some(WordInputMode::TerminalPassthrough)
            }
            "gnome" => engine.config.layout_backend = "gnome".into(),
            _ => unreachable!(),
        }
        let before = engine.composition.word_input_mode;
        let token = engine.context_token.clone();
        let revision = engine.client_context.surrounding_observation_revision;
        engine.settle_native_literal_input_mode();
        assert_eq!(engine.composition.word_input_mode, before, "{guard}");
        assert_eq!(engine.context_token, token, "{guard}");
        assert_eq!(
            engine.client_context.surrounding_observation_revision, revision,
            "{guard}"
        );
    }
}

#[test]
fn native_document_transient_read_clears_edit_proof_and_actual_read_recovers() {
    for reason in [
        "native_deadline",
        "native_document_changed",
        "native_document_shape",
        "native_desktop_unobserved",
    ] {
        let (mut engine, binding) = observed_native_engine();
        let token = engine.context_token.clone();
        let revision = engine.client_context.surrounding_observation_revision;
        assert!(engine.native_document_is_current());
        assert_eq!(
            engine.text_target_edit_route(4),
            TextTargetEditRoute::NativeRangeCommit
        );
        assert!(engine.native_observation_failed(reason));
        assert_eq!(engine.context_token, token);
        assert!(observer::current(&engine, &binding));
        assert!(engine.client_context.surrounding_text_snapshot.is_none());
        assert!(engine.client_context.surrounding_observation_revision > revision);
        assert!(!engine.native_document_is_current());
        assert_eq!(
            engine.text_target_edit_route(4),
            TextTargetEditRoute::Unsupported
        );
        assert!(!engine.client_context.surrounding_text_callback_observed);
        // Only a new complete authenticated observation restores edit evidence.
        assert!(engine.install_native_document(Observation {
            binding,
            snapshot: SurroundingTextSnapshot::new("пров".into(), 4, 4),
        }));
        assert!(engine.native_document_is_current());
        assert_eq!(
            engine.text_target_edit_route(4),
            TextTargetEditRoute::NativeRangeCommit
        );
    }
}

#[test]
fn native_document_unobserved_desktop_keeps_literal_keys_without_edit_permission() {
    use crate::output::{EngineOutput, TestEngineOutput};

    let mut engine = managed_native_acquisition_suffix();
    let token = engine.context_token.clone();
    let scope = engine.context_word_scope;
    assert!(engine.native_observation_failed("native_desktop_unobserved"));
    assert_eq!(engine.context_token, token);
    assert_eq!(engine.context_word_scope, scope);
    assert!(engine.client_context.surrounding_text_snapshot.is_none());
    assert!(!engine.native_manual_word_is_current());
    assert!(!engine.native_managed_readout_is_current());
    assert_eq!(engine.native_document_hint_rect(), Some(None));
    assert_eq!(
        engine.text_target_edit_route(4),
        TextTargetEditRoute::Unsupported
    );

    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    zbus::block_on(engine.commit_managed_passthrough_char(
        &mut EngineOutput::test(&mut output),
        'е',
        20,
    ))
    .unwrap();
    assert_eq!(output.committed_texts, ["е"]);
    assert!(output.surrounding_deletes.is_empty());
    assert!(output.forwarded_keys.is_empty());
    assert_eq!(engine.last_tail_token_text(), "прове");
    assert!(!engine.context_word_is_known());
    assert!(engine.consume_handled_release(20));
    assert!(!engine.consume_handled_release(20));
    assert!(!engine.native_manual_word_is_current());

    // An older document cannot authorize an edit of the newly committed word.
    let binding = engine
        .client_context
        .native_document_binding
        .clone()
        .unwrap();
    assert!(engine.install_native_document(Observation {
        binding: binding.for_result(&engine),
        snapshot: SurroundingTextSnapshot::new("пров".into(), 4, 4),
    }));
    assert!(!engine.native_manual_word_is_current());
    assert!(!engine.native_managed_readout_is_current());
    assert!(engine.install_native_document(Observation {
        binding: binding.for_result(&engine),
        snapshot: SurroundingTextSnapshot::new("прове".into(), 5, 5),
    }));
    assert!(engine.native_manual_word_is_current());
    assert!(engine.native_managed_readout_is_current());
    assert!(!engine.context_word_is_known());
    assert_eq!(engine.context_token, token);
}

#[test]
fn native_document_unobserved_desktop_cannot_preserve_a_changed_input_owner() {
    for changed in [
        "focus", "lease", "layout", "absent", "active", "atomic", "privacy", "gnome", "handoff",
        "token", "scope",
    ] {
        let mut engine = managed_native_acquisition_suffix();
        match changed {
            "focus" => engine.client_context.focus_serial += 1,
            "lease" => engine.client_context.runtime_owner_lease_identity += 1,
            "layout" => engine.layout_gesture.layout_generation += 1,
            "absent" => engine.client_context.native_document_binding = None,
            "active" => engine.shared.lock().unwrap().active_path = Some("/other".into()),
            "atomic" => engine.atomic.active = true,
            "privacy" => {
                engine.client_context.content_purpose = super::super::IBUS_INPUT_PURPOSE_PASSWORD
            }
            "gnome" => engine.config.layout_backend = "gnome".into(),
            "handoff" => engine.context_handoff_sealed = true,
            "token" => engine.context_token = None,
            "scope" => engine.context_word_scope = None,
            _ => unreachable!(),
        }
        assert!(
            !engine.native_observation_failed("native_desktop_unobserved"),
            "{changed}"
        );
        assert!(
            engine.client_context.native_document_binding.is_none(),
            "{changed}"
        );
        assert!(!engine.native_document_is_current(), "{changed}");
    }
}

fn native_window_receipt(lifetime: u64, window: u32) -> String {
    format!("kde-window:bus1::1.5:{lifetime}:{{00000000-0000-0000-0000-{window:012x}}}")
}

#[test]
fn native_document_fresh_same_field_survives_forward_provider_reconnect() {
    use crate::output::{EngineOutput, TestEngineOutput};

    let mut engine = managed_native_acquisition_suffix();
    engine
        .client_context
        .native_document_binding
        .as_mut()
        .unwrap()
        .identity
        .desktop
        .identity = native_window_receipt(7, 1);
    assert!(engine.native_observation_failed("native_desktop_unobserved"));
    assert!(!engine.native_document_is_current());
    let token = engine.context_token.clone();
    let scope = engine.context_word_scope;
    let epoch = engine.committed_tail.epoch;
    let previous = engine
        .client_context
        .native_document_binding
        .clone()
        .unwrap();
    let mut fresh = previous.for_result(&engine);
    fresh.identity.desktop.identity = native_window_receipt(9, 1);
    assert!(fresh.is_current(&engine));
    assert!(engine.install_native_document(Observation {
        binding: fresh,
        snapshot: SurroundingTextSnapshot::new("пров".into(), 4, 4),
    }));
    assert_eq!(engine.context_token, token);
    assert_eq!(engine.context_word_scope, scope);
    assert_eq!(engine.committed_tail.epoch, epoch);
    assert!(engine.native_manual_word_is_current());
    assert!(!engine.context_word_is_known());
    assert!(!observer::current(&engine, &previous));

    // Exercise the production key router after the fresh read, not just its
    // commit helper: the key following a provider failure must not be skipped.
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    assert!(zbus::block_on(engine.process_key_event_with_output(
        &mut EngineOutput::test(&mut output),
        0x06c5,
        20,
        0,
    ))
    .unwrap());
    assert_eq!(output.committed_texts, ["е"]);
    assert_eq!(engine.last_tail_token_text(), "прове");
    assert!(output.surrounding_deletes.is_empty());
    assert!(output.forwarded_keys.is_empty());
    assert!(!engine.context_word_is_known());

    // The same physical field can also normalize an already owned separator
    // while KWin's query connection reconnects. Its input lease continues;
    // the old transport subscription must still be retired.
    let (mut engine, _, output) = native_owned_append_after_closing_space('\u{a0}', true, true);
    engine
        .client_context
        .native_document_binding
        .as_mut()
        .unwrap()
        .identity
        .desktop
        .identity = native_window_receipt(7, 1);
    let witness = engine.native_word_preparation_identity().unwrap();
    assert!(engine.native_observation_failed("native_desktop_unobserved"));
    let token = engine.context_token.clone();
    let scope = engine.context_word_scope;
    let epoch = engine.committed_tail.epoch;
    let previous = engine
        .client_context
        .native_document_binding
        .clone()
        .unwrap();
    let mut fresh = previous.for_result(&engine);
    fresh.identity.desktop.identity = native_window_receipt(9, 1);
    assert!(!previous.same_observation_owner(&fresh));
    assert!(fresh.identity.continues_from(&previous.identity));
    assert!(engine.install_native_document(Observation {
        binding: fresh,
        snapshot: SurroundingTextSnapshot::new("пров е".into(), 6, 6),
    }));
    assert_eq!(engine.context_token, token);
    assert_eq!(engine.context_word_scope, scope);
    assert_eq!(engine.committed_tail.epoch, epoch);
    assert_eq!(engine.native_word_preparation_identity(), Some(witness));
    assert_eq!(
        engine
            .capture_space_boundary_pair_scope()
            .as_ref()
            .map(|pair| pair.text.as_str()),
        Some("пров е")
    );
    assert!(engine.native_word_preparation_edit_is_current());
    assert!(!observer::current(&engine, &previous));
    assert_eq!(output.committed_texts, [" ", "е"]);
    assert!(output.surrounding_deletes.is_empty());
    assert!(output.forwarded_keys.is_empty());
}

#[test]
fn native_document_reconnect_cannot_accept_older_or_different_field_identity() {
    for changed in [
        "older_transport",
        "window",
        "compositor_owner",
        "session_bus",
        "pid",
        "a11y_bus",
        "a11y_owner",
        "application",
        "field",
    ] {
        let mut engine = managed_native_acquisition_suffix();
        engine
            .client_context
            .native_document_binding
            .as_mut()
            .unwrap()
            .identity
            .desktop
            .identity = native_window_receipt(7, 1);
        let mut fresh = engine
            .client_context
            .native_document_binding
            .clone()
            .unwrap()
            .for_result(&engine);
        fresh.identity.desktop.identity = native_window_receipt(9, 1);
        match changed {
            "older_transport" => fresh.identity.desktop.identity = native_window_receipt(6, 1),
            "window" => fresh.identity.desktop.identity = native_window_receipt(9, 2),
            "compositor_owner" => {
                fresh.identity.desktop.identity =
                    native_window_receipt(9, 1).replace(":1.5", ":1.6")
            }
            "session_bus" => {
                fresh.identity.desktop.identity =
                    native_window_receipt(9, 1).replace("bus1", "bus2")
            }
            "pid" => fresh.identity.desktop.pid += 1,
            "a11y_bus" => fresh.identity.bus = "other-bus".into(),
            "a11y_owner" => fresh.identity.owner = ":1.99".into(),
            "application" => fresh.identity.application = "/other_application".try_into().unwrap(),
            "field" => fresh.identity.field = "/other_field".try_into().unwrap(),
            _ => unreachable!(),
        }
        assert!(fresh.is_current(&engine), "{changed}");
        if changed == "older_transport" {
            assert_eq!(
                fresh.identity,
                engine
                    .client_context
                    .native_document_binding
                    .as_ref()
                    .unwrap()
                    .identity
            );
        }
        let token = engine.context_token.clone();
        let scope = engine.context_word_scope;
        let epoch = engine.committed_tail.epoch;
        assert!(
            !engine.install_native_document(Observation {
                binding: fresh,
                snapshot: SurroundingTextSnapshot::new("пров".into(), 4, 4),
            }),
            "{changed}"
        );
        if changed == "older_transport" {
            assert!(engine.client_context.native_document_binding.is_some());
            assert!(engine.native_document_is_current());
            assert_eq!(engine.context_token, token);
            assert_eq!(engine.context_word_scope, scope);
            assert_eq!(engine.committed_tail.epoch, epoch);
        } else {
            assert!(
                engine.client_context.native_document_binding.is_none(),
                "{changed}"
            );
            assert!(!engine.native_document_is_current(), "{changed}");
        }
    }
}

#[test]
fn native_document_identity_or_selection_loss_cannot_retain_subscription() {
    for reason in [
        "native_field_peer",
        "native_field_changed",
        "native_no_desktop",
        "native_desktop_changed",
        "native_not_chrome",
        "native_no_admission",
        "native_private_or_terminal",
        "native_selection_present",
    ] {
        let (mut engine, binding) = observed_native_engine();
        assert!(!engine.native_observation_failed(reason));
        assert!(engine.client_context.native_document_binding.is_none());
        assert!(!engine.native_document_is_current());
        assert!(!observer::current(&engine, &binding));
    }
    let (mut engine, binding) = observed_native_engine();
    engine.client_context.focus_serial += 1;
    assert!(!engine.native_observation_failed("native_deadline"));
    assert!(!observer::current(&engine, &binding));
}

#[test]
fn native_document_pending_result_waits_for_its_intermediate_selection_without_edit_proof() {
    let (mut engine, binding) = observed_native_engine();
    engine.config.auto_switch_layout = true;
    engine.layout_gesture.layout_is_ru = true;
    engine.committed_tail.buffer = "ghjd".into();
    let actual = SurroundingTextSnapshot::new("ghjd".into(), 4, 4);
    engine.arm_exact_visible_postcondition_from_surrounding_dispatch(
        Instant::now(),
        None,
        Some("ghjd".into()),
        actual.clone(),
    );
    let token = engine.context_token.clone();
    let epoch = engine.committed_tail.epoch;
    assert!(engine.native_observation_failed("native_selection_present"));
    assert_eq!(engine.context_token, token);
    assert!(observer::current(&engine, &binding));
    assert!(engine
        .committed_tail
        .pending_visible_postcondition
        .is_some());
    assert!(engine.client_context.surrounding_text_snapshot.is_none());
    assert!(!engine.native_document_is_current());
    assert_eq!(
        engine.text_target_edit_route(4),
        TextTargetEditRoute::Unsupported
    );
    assert!(!engine.context_word_is_known());
    // A fresh exact result, not the intermediate selection, settles the edit
    // and publishes its next mode through the existing admission contract.
    assert!(engine.install_native_document(Observation {
        binding: binding.for_result(&engine),
        snapshot: actual,
    }));
    engine.observe_native_document_postcondition();
    assert!(engine
        .committed_tail
        .pending_visible_postcondition
        .is_none());
    assert_eq!(engine.committed_tail.epoch, epoch + 1);
    assert!(!engine.context_word_is_known());
}

#[test]
fn native_document_intermediate_selection_requires_the_current_exact_native_result() {
    for changed in [
        "no_pending",
        "no_native",
        "no_exact_result",
        "epoch",
        "focus",
        "lease",
        "layout",
        "field",
        "privacy",
        "sealed",
        "gnome",
    ] {
        let (mut engine, binding) = observed_native_engine();
        engine.committed_tail.buffer = "ghjd".into();
        engine.arm_exact_visible_postcondition_from_surrounding_dispatch(
            Instant::now(),
            None,
            None,
            SurroundingTextSnapshot::new("ghjd".into(), 4, 4),
        );
        match changed {
            "no_pending" => engine.committed_tail.pending_visible_postcondition = None,
            "no_native" => {
                engine
                    .committed_tail
                    .pending_visible_postcondition
                    .as_mut()
                    .unwrap()
                    .native_document = None
            }
            "no_exact_result" => {
                engine
                    .committed_tail
                    .pending_visible_postcondition
                    .as_mut()
                    .unwrap()
                    .expected_external_snapshot = None
            }
            "epoch" => engine.committed_tail.epoch += 1,
            "focus" => engine.client_context.focus_serial += 1,
            "lease" => engine.client_context.runtime_owner_lease_identity += 1,
            "layout" => engine.layout_gesture.layout_generation += 1,
            "field" => {
                engine
                    .client_context
                    .native_document_binding
                    .as_mut()
                    .unwrap()
                    .identity
                    .field = "/other".try_into().unwrap()
            }
            "privacy" => {
                engine.client_context.content_purpose = super::super::IBUS_INPUT_PURPOSE_PASSWORD
            }
            "sealed" => engine.context_handoff_sealed = true,
            "gnome" => engine.config.layout_backend = "gnome".into(),
            _ => unreachable!(),
        }
        assert!(
            !engine.native_observation_failed("native_selection_present"),
            "{changed}"
        );
        assert!(
            engine.client_context.native_document_binding.is_none(),
            "{changed}"
        );
        assert!(!observer::current(&engine, &binding), "{changed}");
        assert_eq!(
            engine.text_target_edit_route(4),
            TextTargetEditRoute::Unsupported,
            "{changed}"
        );
    }
}

#[test]
fn native_text_subscription_survives_a_commit_without_reusing_its_edit_receipt() {
    use crate::output::{EngineOutput, TestEngineOutput};

    let mut engine = managed_native_acquisition_suffix();
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    zbus::block_on(engine.commit_managed_passthrough_char(
        &mut EngineOutput::test(&mut output),
        'е',
        32,
    ))
    .unwrap();
    let binding = engine
        .client_context
        .native_document_binding
        .clone()
        .unwrap();
    assert_eq!(engine.committed_tail.buffer, "прове");
    assert_eq!(output.committed_texts, ["е"]);
    assert!(observer::current(&engine, &binding));
    assert!(!binding.is_current(&engine));
    let token = engine.context_token.clone();
    let scope = engine.context_word_scope;
    let epoch = engine.committed_tail.epoch;
    assert!(
        observer::needs_subscription(&engine, &binding),
        "ordinary CommitText must not suppress the notification needed to read its actual result"
    );
    assert_eq!(engine.context_token, token);
    assert_eq!(engine.context_word_scope, scope);
    assert_eq!(engine.committed_tail.epoch, epoch);
    assert!(!binding.is_current(&engine));
    assert_eq!(output.committed_texts, ["е"]);
}

#[test]
fn native_text_subscription_still_rejects_a_changed_observation_owner() {
    for changed in [
        "focus", "lease", "layout", "field", "absent", "active", "atomic", "privacy", "gnome",
    ] {
        let (mut engine, binding) = observed_native_engine();
        match changed {
            "focus" => engine.client_context.focus_serial += 1,
            "lease" => engine.client_context.runtime_owner_lease_identity += 1,
            "layout" => engine.layout_gesture.layout_generation += 1,
            "field" => {
                engine
                    .client_context
                    .native_document_binding
                    .as_mut()
                    .unwrap()
                    .identity
                    .field = "/other".try_into().unwrap()
            }
            "absent" => engine.client_context.native_document_binding = None,
            "active" => engine.shared.lock().unwrap().active_path = Some("/other".into()),
            "atomic" => engine.atomic.active = true,
            "privacy" => {
                engine.client_context.content_purpose = super::super::IBUS_INPUT_PURPOSE_PASSWORD
            }
            "gnome" => engine.config.layout_backend = "gnome".into(),
            _ => unreachable!(),
        }
        assert!(
            !observer::needs_subscription(&engine, &binding),
            "{changed}"
        );
    }
}

#[test]
fn native_text_notification_cannot_cross_focus_lease_layout_or_field() {
    let (mut engine, binding) = observed_native_engine();
    let path = engine.path.clone();
    assert!(observer::current(&engine, &binding));
    // An ordinary key may wake a fresh read; it does not extend an edit token.
    engine.committed_tail.epoch += 1;
    assert!(observer::current(&engine, &binding));
    assert!(!binding.is_current(&engine));
    engine.committed_tail.epoch -= 1;

    engine.client_context.focus_serial += 1;
    assert!(!observer::current(&engine, &binding));
    engine.client_context.focus_serial -= 1;
    engine.client_context.runtime_owner_lease_identity += 1;
    assert!(!observer::current(&engine, &binding));
    engine.client_context.runtime_owner_lease_identity -= 1;
    engine.layout_gesture.layout_generation += 1;
    assert!(!observer::current(&engine, &binding));
    engine.layout_gesture.layout_generation -= 1;
    engine.shared.lock().unwrap().active_path = Some("/other".into());
    assert!(!observer::current(&engine, &binding));
    engine.shared.lock().unwrap().active_path = Some(path);
    let mut changed = binding.clone();
    changed.identity.field = "/other".try_into().unwrap();
    assert!(!observer::current(&engine, &changed));
    engine.config.layout_backend = "gnome".into();
    assert!(!observer::current(&engine, &binding));
}

#[test]
fn native_document_non_kde_backends_never_read_or_emit() {
    use crate::output::{EngineOutput, TestEngineOutput};
    use std::sync::{Arc, Mutex};

    for backend in ["gnome", "niri", "x11"] {
        let mut engine = LayIbusEngine::new(
            "/test/native-document/desktop".into(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            lay::config::LayConfig {
                layout_backend: backend.into(),
                ..Default::default()
            },
        );
        let read = engine.read_native_document();
        assert!(std::mem::size_of_val(&read) < 1024);
        assert!(matches!(zbus::block_on(read), Err("native_not_eligible")));
        let snapshot = SurroundingTextSnapshot::new("abc".into(), 3, 3);
        let select = engine.select_native_document_range(&snapshot, 1);
        assert!(std::mem::size_of_val(&select) < 1024);
        assert_eq!(zbus::block_on(select), Err("native_stale_binding"));
        let mut observed = TestEngineOutput::default();
        assert!(
            zbus::block_on(super::super::WindowInteraction::refresh_native_document(
                &mut engine,
                &mut EngineOutput::test(&mut observed),
            ))
            .unwrap()
        );
        let mut output = EngineOutput::test(&mut observed);
        let release = super::super::WindowInteraction::refresh_native_document_for_display(
            &mut engine,
            &mut output,
        );
        assert!(std::mem::size_of_val(&release) < 1024);
        zbus::block_on(release).unwrap();
        assert!(observed.effects.is_empty());
        assert!(engine.client_context.native_document_binding.is_none());
        assert!(engine.client_context.surrounding_text_snapshot.is_none());
        assert!(!engine.client_context.surrounding_text_callback_observed);
    }
}

#[test]
fn native_document_snapshot_and_edit_range_use_unicode_scalar_offsets() {
    let actual = checked_snapshot("а😀е\u{301}".into(), 4, 3, 0).unwrap();
    assert_eq!(actual.cursor_pos, 3);
    assert_eq!(actual.anchor_pos, 3);
    assert_eq!(exact_range(&actual, 2), Ok((1, 3)));
    assert_eq!(exact_range(&actual, 3), Ok((0, 3)));
    assert!(exact_range(&actual, 4).is_err());
    assert!(exact_range(&actual, 0).is_err());
    assert!(exact_range(&SurroundingTextSnapshot::new("abc".into(), 2, 1), 1).is_err());
    assert!(exact_range(&SurroundingTextSnapshot::new("abc".into(), 4, 4), 1).is_err());
}

#[test]
fn native_document_rejects_selection_invalid_counts_and_oversized_sources() {
    for (text, count, caret, selections) in [
        ("abc".to_string(), 3, 3, 1),
        ("abc".to_string(), 3, 3, -1),
        ("abc".to_string(), 3, 4, 0),
        ("abc".to_string(), 3, -1, 0),
        ("😀".to_string(), 2, 1, 0),
        ("a".to_string(), -1, 0, 0),
        (
            "x".repeat(MAX_DOCUMENT_CHARS as usize + 1),
            MAX_DOCUMENT_CHARS + 1,
            0,
            0,
        ),
    ] {
        assert!(checked_snapshot(text, count, caret, selections).is_err());
    }
    assert_eq!(
        checked_snapshot(String::new(), 0, 0, 0).unwrap().cursor_pos,
        0
    );
    assert!(checked_snapshot(
        "x".repeat(MAX_DOCUMENT_CHARS as usize),
        MAX_DOCUMENT_CHARS,
        MAX_DOCUMENT_CHARS,
        0
    )
    .is_ok());
}

#[test]
fn native_document_selects_one_local_executor_without_claiming_surrounding_text() {
    let mut facts = TextTargetCapabilityFacts {
        backspaces: 4,
        surrounding_text_supported: false,
        native_range_supported: true,
        terminal_erase_supported: false,
    };
    assert_eq!(
        TextTargetDecision::from_facts(facts),
        TextTargetDecision {
            route: TextTargetEditRoute::NativeRangeCommit,
            reason: TextTargetDecisionReason::QualifiedNativeRange,
        }
    );
    facts.backspaces = 0;
    assert_eq!(
        TextTargetDecision::from_facts(facts).route,
        TextTargetEditRoute::CommitOnly
    );
    facts.backspaces = 4;
    facts.native_range_supported = false;
    assert_eq!(
        TextTargetDecision::from_facts(facts).route,
        TextTargetEditRoute::Unsupported
    );
}

#[test]
fn native_document_deadline_does_not_propagate_transport_frame() {
    let payload = [0u8; 64 * 1024];
    let operation = bounded(Instant::now() + IO_BUDGET, async move {
        std::future::pending::<()>().await;
        std::hint::black_box(payload);
        Ok(())
    });
    // The deadline wrapper must not copy a transport's frame into each
    // enclosing key/focus future, even before the first poll.
    assert!(std::mem::size_of_val(&operation) < 1024);
}

#[test]
fn native_document_expired_deadline_never_polls_transport() {
    let polled = std::cell::Cell::new(false);
    assert_eq!(
        zbus::block_on(bounded(Instant::now(), async {
            polled.set(true);
            Ok(())
        })),
        Err("native_deadline")
    );
    assert!(!polled.get());
}

#[test]
fn native_document_deadline_drops_unsettled_transport_without_retry() {
    struct PendingRequest(std::rc::Rc<std::cell::Cell<u32>>);
    impl Drop for PendingRequest {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let drops = std::rc::Rc::new(std::cell::Cell::new(0));
    let request = PendingRequest(drops.clone());
    let result = zbus::block_on(bounded(
        Instant::now() + Duration::from_millis(1),
        async move {
            let _request = request;
            std::future::pending::<NativeResult<()>>().await
        },
    ));
    assert_eq!(result, Err("native_deadline"));
    assert_eq!(drops.get(), 1);
}
