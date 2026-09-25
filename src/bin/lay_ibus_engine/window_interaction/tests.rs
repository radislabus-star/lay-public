#[cfg(test)]
mod selector {
    use crate::window_interaction::{
        TextTargetCapabilityFacts, TextTargetDecision, TextTargetDecisionReason,
        TextTargetEditRoute,
    };

    #[test]
    fn capability_contract_selects_one_delete_executor() {
        assert_eq!(
            TextTargetEditRoute::select(5, true, true),
            TextTargetEditRoute::ExactSurroundingText
        );
        assert_eq!(
            TextTargetEditRoute::select(5, false, true),
            TextTargetEditRoute::TerminalErase
        );
        assert_eq!(
            TextTargetEditRoute::select(5, false, false),
            TextTargetEditRoute::Unsupported
        );
        assert_eq!(
            TextTargetEditRoute::select(0, false, false),
            TextTargetEditRoute::CommitOnly
        );

        for (facts, route, reason) in [
            (
                TextTargetCapabilityFacts {
                    backspaces: 0,
                    surrounding_text_supported: false,
                    terminal_erase_supported: false,
                },
                TextTargetEditRoute::CommitOnly,
                TextTargetDecisionReason::AppendNeedsNoDelete,
            ),
            (
                TextTargetCapabilityFacts {
                    backspaces: 5,
                    surrounding_text_supported: true,
                    terminal_erase_supported: true,
                },
                TextTargetEditRoute::ExactSurroundingText,
                TextTargetDecisionReason::SurroundingTextDelete,
            ),
            (
                TextTargetCapabilityFacts {
                    backspaces: 5,
                    surrounding_text_supported: false,
                    terminal_erase_supported: true,
                },
                TextTargetEditRoute::TerminalErase,
                TextTargetDecisionReason::ProvenTerminalErase,
            ),
            (
                TextTargetCapabilityFacts {
                    backspaces: 5,
                    surrounding_text_supported: false,
                    terminal_erase_supported: false,
                },
                TextTargetEditRoute::Unsupported,
                TextTargetDecisionReason::MissingProvenDelete,
            ),
        ] {
            assert_eq!(
                TextTargetDecision::from_facts(facts),
                TextTargetDecision { route, reason }
            );
        }
    }
}

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use lay::config::LayConfig;
use lay::text_edit::{VisibleTailSnapshot, VisibleTailSource};

use super::*;
use crate::context_admission::{ActivationOutcome, ContextAdmissionAdapter, WordCompleteness};
use crate::engine::LayIbusEngine;
use crate::output::{EngineOutput, TestEngineOutput};
use crate::state::CommittedTailReplaceRequest;

#[test]
fn kitty_focus_probe_requires_exact_window_identity() {
    use super::observation::exact_kitty_window;

    assert!(exact_kitty_window(
        r#"{"appId":"kitty.desktop","windowId":"42","stableSequence":"7"}"#
    ));
    assert!(!exact_kitty_window(
        r#"{"appId":"google-chrome.desktop","windowId":"42","stableSequence":"7"}"#
    ));
    assert!(!exact_kitty_window(
        r#"{"appId":"kitty.desktop","windowId":"","stableSequence":"7"}"#
    ));
}

#[test]
fn kitty_focus_probe_survives_activation_serial_but_not_focus_transfer() {
    let mut engine = engine();
    engine.set_client_capabilities(9);
    engine.client_context.cursor_cell_width = 8;
    engine.client_context.focus_receipt = Some("kitty-context".to_string());
    let before = engine.client_context.focus_serial;
    engine.client_context.kitty_focus_probe_serial = Some(before);
    engine.client_context.focus_serial = crate::engine::next_input_identity();
    assert!(engine
        .client_context
        .finish_kitty_focus_probe(Some("kitty-context"), true));
    assert_eq!(
        engine
            .client_context
            .kitty_terminal_focus_receipt
            .as_deref(),
        Some("kitty-context")
    );
    engine.client_context.focus_serial = crate::engine::next_input_identity();
    assert!(engine.has_proven_terminal_input());

    engine.client_context.focus_receipt = Some("other-context".to_string());
    assert!(!engine.has_proven_terminal_input());
    assert!(!engine
        .client_context
        .finish_kitty_focus_probe(Some("kitty-context"), true));
    assert!(!engine.has_proven_terminal_input());
}

fn callback_message(member: &str) -> zbus::Message {
    zbus::Message::method_call("/engine/window_interaction", member)
        .unwrap()
        .interface("org.freedesktop.IBus.Engine")
        .unwrap()
        .build(&())
        .unwrap()
}

struct AdmittedEngine {
    engine: LayIbusEngine,
    adapter: ContextAdmissionAdapter,
    _peer: zbus::Connection,
}

fn engine() -> LayIbusEngine {
    LayIbusEngine::new(
        "/engine/window-interaction".to_string(),
        Arc::new(Mutex::new(Default::default())),
        false,
        true,
        LayConfig::default(),
    )
}

fn admitted_engine() -> AdmittedEngine {
    let path = "/engine/window-interaction-admitted";
    let (adapter, grant, peer) = ContextAdmissionAdapter::test_established(
        path,
        "/org/freedesktop/IBus/InputContext_window_interaction",
        WordCompleteness::KnownStart,
        17,
    );
    let mut engine = LayIbusEngine::new_from_component(
        path.to_string(),
        Arc::new(Mutex::new(Default::default())),
        Some(adapter.clone()),
        "lay-ime-us",
        true,
        LayConfig::default(),
    );
    assert!(engine.install_context_activation(ActivationOutcome::SourceFree(grant)));
    AdmittedEngine {
        engine,
        adapter,
        _peer: peer,
    }
}

fn exact_request(engine: &LayIbusEngine, before: &str, after: &str) -> CommittedTailReplaceRequest {
    CommittedTailReplaceRequest::ime_manual_toggle(
        before.chars().count() as u32,
        after.to_string(),
        true,
    )
    .with_expected_tail(VisibleTailSnapshot::new(
        VisibleTailSource::ImeCommittedTail,
        before,
        Some(engine.path.clone()),
        engine.committed_tail.epoch,
    ))
}

#[test]
fn window_interaction_gui_uses_existing_later_postcondition_receipt() {
    fn observed(age_ms: u64, external: &str) -> OutcomeProof {
        let mut engine = engine();
        let focus = callback_message("FocusIn");
        assert_eq!(
            zbus::block_on(WindowInteraction::observe_lifecycle(
                &mut engine,
                WindowLifecycleEvent::FocusIn {
                    header: &focus.header(),
                },
                None,
            ))
            .unwrap(),
            LifecycleReceipt::Focused { changed: true }
        );
        assert_eq!(
            zbus::block_on(WindowInteraction::observe_facts(
                &mut engine,
                WindowFactEvent::Capabilities(IBUS_CAP_SURROUNDING_TEXT),
                None,
            ))
            .unwrap(),
            ObservationReceipt::Capabilities
        );
        let mut callback_output = TestEngineOutput::default();
        assert_eq!(
            zbus::block_on(WindowInteraction::observe_facts(
                &mut engine,
                WindowFactEvent::ContentType {
                    value: (0, 0),
                    header: None,
                },
                None,
            ))
            .unwrap(),
            ObservationReceipt::ContentType
        );
        engine.committed_tail.buffer = "ghbdtn".into();
        assert!(matches!(
            zbus::block_on(WindowInteraction::observe_facts(
                &mut engine,
                WindowFactEvent::SurroundingText(Some(SurroundingTextSnapshot::new(
                    "ghbdtn".into(),
                    6,
                    6,
                ))),
                Some(&mut EngineOutput::test(&mut callback_output)),
            ))
            .unwrap(),
            ObservationReceipt::SurroundingText(OutcomeProof::Rejected)
        ));
        let authority = WindowInteraction::admit_replace(&engine, 6);
        assert_eq!(authority, TextTargetAuthority::LocalSurroundingDeleteCommit);
        let request = exact_request(&engine, "ghbdtn", "привет");
        let mut local_output = TestEngineOutput::default();
        assert_eq!(
            zbus::block_on(WindowInteraction::execute_local(
                &mut engine,
                authority,
                request,
                &mut EngineOutput::test(&mut local_output),
            ))
            .unwrap(),
            ExecutionReceipt::LocalComplete
        );
        assert_eq!(local_output.surrounding_deletes, [(-6, 6)]);
        assert_eq!(local_output.committed_texts, ["привет"]);
        engine
            .committed_tail
            .pending_visible_postcondition
            .as_mut()
            .expect("local execution arms the existing postcondition")
            .dispatched_at = Instant::now() - Duration::from_millis(age_ms);

        let chars = external.chars().count() as u32;
        let receipt = zbus::block_on(WindowInteraction::observe_facts(
            &mut engine,
            WindowFactEvent::SurroundingText(Some(SurroundingTextSnapshot::new(
                external.into(),
                chars,
                chars,
            ))),
            Some(&mut EngineOutput::test(&mut callback_output)),
        ))
        .unwrap();
        let ObservationReceipt::SurroundingText(outcome) = receipt else {
            panic!("later SetSurroundingText must return its existing outcome")
        };
        outcome
    }

    assert_eq!(
        observed(0, "привет"),
        OutcomeProof::ExistingPostconditionConfirmed
    );
    assert_eq!(
        observed(0, "старое"),
        OutcomeProof::ExistingPostconditionPending
    );
    assert_eq!(
        observed(501, "старое"),
        OutcomeProof::ExistingPostconditionMismatch
    );
}

#[test]
fn window_interaction_exact_tail_is_delegation_without_client_receipt() {
    let mut engine = engine();
    engine.committed_tail.buffer = "ghbdtn".into();
    engine.set_client_capabilities(1 << 5);
    engine.client_context.surrounding_text_snapshot =
        Some(SurroundingTextSnapshot::new("ghbdtn".into(), 6, 6));
    let authority = WindowInteraction::admit_manual_toggle(&engine);
    assert_eq!(authority, TextTargetAuthority::DelegateExactImeTail);
    let mut output = TestEngineOutput::default();
    let (target_layout, execution) = zbus::block_on(WindowInteraction::execute_manual_toggle(
        &mut engine,
        authority,
        &mut EngineOutput::test(&mut output),
    ))
    .unwrap();
    assert_eq!(target_layout, None);
    assert_eq!(execution, ExecutionReceipt::DelegatedExactImeTail);
    assert!(output.effects.is_empty());
    assert!(engine.exact_manual_toggle_handoff_is_live());
    let shared = engine.shared.lock().unwrap();
    assert_eq!(
        shared.exact_manual_toggle_handoff_epoch,
        Some(engine.committed_tail.epoch)
    );
    assert_eq!(
        shared.exact_manual_toggle_handoff_path.as_deref(),
        Some(engine.path.as_str())
    );
    assert_eq!(shared.handoff_tail_buffer, "ghbdtn");
    drop(shared);
    assert!(engine
        .committed_tail
        .pending_visible_postcondition
        .is_none());
    assert_eq!(
        WindowInteraction::observe_existing_postcondition(&mut engine),
        OutcomeProof::Rejected
    );
}

#[test]
fn window_interaction_terminal_execution_stays_client_unverified() {
    let mut engine = engine();
    engine.committed_tail.buffer = "ghbdtn".into();
    engine.set_content_type_state(10, 0);
    engine.client_context.cursor_cell_width = 8;
    let authority = WindowInteraction::admit_manual_toggle(&engine);
    assert_eq!(authority, TextTargetAuthority::LocalTerminalErase);
    let mut test_output = TestEngineOutput::default();
    let (target_layout, receipt) = zbus::block_on(WindowInteraction::execute_manual_toggle(
        &mut engine,
        authority,
        &mut EngineOutput::test(&mut test_output),
    ))
    .expect("terminal local execution");
    assert_eq!(target_layout, Some(true));
    assert_eq!(receipt, ExecutionReceipt::LocalComplete);
    assert_eq!(test_output.surrounding_deletes, []);
    assert_eq!(
        test_output.committed_texts,
        ["\u{7f}\u{7f}\u{7f}\u{7f}\u{7f}\u{7f}привет"]
    );
    assert_eq!(engine.committed_tail.buffer, "привет");
    assert!(engine.layout_gesture.layout_is_ru);
    assert!(engine
        .committed_tail
        .pending_visible_postcondition
        .is_none());
    assert_eq!(
        WindowInteraction::observe_existing_postcondition(&mut engine),
        OutcomeProof::Rejected
    );
}

#[test]
fn window_interaction_daemon_buffer_delegates_with_zero_local_effects() {
    let mut engine = engine();
    let authority = WindowInteraction::admit_manual_toggle(&engine);
    assert_eq!(authority, TextTargetAuthority::DelegateDaemonBuffer);
    let mut output = TestEngineOutput::default();
    let (target_layout, receipt) = zbus::block_on(WindowInteraction::execute_manual_toggle(
        &mut engine,
        authority,
        &mut EngineOutput::test(&mut output),
    ))
    .unwrap();
    assert_eq!(target_layout, None);
    assert_eq!(receipt, ExecutionReceipt::DelegatedDaemonBuffer);
    assert!(output.effects.is_empty());
    assert!(matches!(
        engine.committed_tail.autocorrect_suppression,
        Some(crate::protocol::AutocorrectSuppression::LegacyReplayV1)
    ));
}

#[test]
fn window_interaction_sensitive_selection_and_stale_authority_refuse() {
    let mut sensitive = engine();
    sensitive.committed_tail.buffer = "secret".into();
    sensitive.composition.buffer = "draft".into();
    sensitive.set_client_capabilities(IBUS_CAP_SURROUNDING_TEXT);
    sensitive.client_context.surrounding_text_snapshot =
        Some(SurroundingTextSnapshot::new("secret".into(), 6, 6));
    sensitive.set_content_type_state(8, 0);
    let sensitive_authority = WindowInteraction::admit_replace(&sensitive, 6);
    assert_eq!(
        sensitive_authority,
        TextTargetAuthority::Reject(WindowRejectReason::SensitiveContent)
    );
    assert!(sensitive.committed_tail.buffer.is_empty());
    assert!(sensitive.composition.buffer.is_empty());
    assert!(sensitive.client_context.surrounding_text_snapshot.is_none());
    let mut sensitive_output = TestEngineOutput::default();
    assert_eq!(
        zbus::block_on(WindowInteraction::execute_local(
            &mut sensitive,
            sensitive_authority,
            CommittedTailReplaceRequest::ime_manual_toggle(6, "public".into(), true),
            &mut EngineOutput::test(&mut sensitive_output),
        ))
        .unwrap(),
        ExecutionReceipt::Rejected
    );
    assert!(sensitive_output.effects.is_empty());

    let mut selected = engine();
    selected.committed_tail.buffer = "abc".into();
    selected.set_client_capabilities(IBUS_CAP_SURROUNDING_TEXT);
    selected.client_context.surrounding_text_snapshot =
        Some(SurroundingTextSnapshot::new("abc".into(), 3, 0));
    let selection_authority = WindowInteraction::admit_replace(&selected, 3);
    assert_eq!(
        selection_authority,
        TextTargetAuthority::Reject(WindowRejectReason::SelectionPresent)
    );
    let mut selection_output = TestEngineOutput::default();
    let selection_request = exact_request(&selected, "abc", "фис");
    assert_eq!(
        zbus::block_on(WindowInteraction::execute_local(
            &mut selected,
            selection_authority,
            selection_request,
            &mut EngineOutput::test(&mut selection_output),
        ))
        .unwrap(),
        ExecutionReceipt::Rejected
    );
    assert!(selection_output.effects.is_empty());
    assert_eq!(selected.committed_tail.buffer, "abc");

    let mut stale = engine();
    stale.committed_tail.buffer = "abc".into();
    stale.set_client_capabilities(1 << 5);
    stale.client_context.surrounding_text_snapshot =
        Some(SurroundingTextSnapshot::new("abc".into(), 3, 3));
    let authority = WindowInteraction::admit_replace(&stale, 3);
    stale.set_client_capabilities(0);
    let request = exact_request(&stale, "abc", "фис");
    let mut test_output = TestEngineOutput::default();
    let receipt = zbus::block_on(WindowInteraction::execute_local(
        &mut stale,
        authority,
        request,
        &mut EngineOutput::test(&mut test_output),
    ))
    .expect("stale authority is a typed refusal");
    assert_eq!(receipt, ExecutionReceipt::Rejected);
    assert!(test_output.effects.is_empty());
}

#[test]
fn window_interaction_reset_rereceipt_uses_existing_one_shot_token() {
    zbus::block_on(crate::context_admission::assert_window_interaction_reset_rereceipt_contract());
}

#[test]
fn window_interaction_bridge_raii_revokes_on_cancel_or_drop() {
    let mut fixture = admitted_engine();
    fixture.engine.committed_tail.buffer = "ghbdtn".into();
    fixture
        .engine
        .set_client_capabilities(IBUS_CAP_SURROUNDING_TEXT);
    fixture.engine.client_context.surrounding_text_snapshot =
        Some(SurroundingTextSnapshot::new("ghbdtn".into(), 6, 6));
    let token = fixture.engine.live_context_token().expect("live token");
    let authority = WindowInteraction::admit_replace(&fixture.engine, 6);
    let request = exact_request(&fixture.engine, "ghbdtn", "привет");
    let mut output = TestEngineOutput {
        pause_before_commit: true,
        ..Default::default()
    };
    let mut cancelled = Box::pin(async {
        let mut bridge = fixture.engine.begin_context_bridge_output(Some(&token));
        let receipt = WindowInteraction::execute_local(
            &mut bridge,
            authority,
            request,
            &mut EngineOutput::test(&mut output),
        )
        .await?;
        bridge.complete();
        Ok::<_, LocalExecutionFailure>(receipt)
    });
    assert!(
        zbus::block_on(futures_lite::future::poll_once(cancelled.as_mut())).is_none(),
        "execution must be cancellable after delete and before commit"
    );
    drop(cancelled);
    assert_eq!(output.surrounding_deletes, [(-6, 6)]);
    assert!(output.committed_texts.is_empty());
    assert!(fixture.engine.context_bridge_token.is_none());
    assert!(!fixture.engine.context_word_is_known());
    assert!(!fixture.adapter.revalidate(&token));
}

#[test]
fn window_interaction_commit_failure_after_delete_is_indeterminate() {
    let mut fixture = admitted_engine();
    fixture.engine.committed_tail.buffer = "ghbdtn".into();
    fixture.engine.set_client_capabilities(1 << 5);
    fixture.engine.client_context.surrounding_text_snapshot =
        Some(SurroundingTextSnapshot::new("ghbdtn".into(), 6, 6));
    let token = fixture.engine.live_context_token().expect("live token");
    let authority = WindowInteraction::admit_replace(&fixture.engine, 6);
    let request = exact_request(&fixture.engine, "ghbdtn", "привет");
    let mut test_output = TestEngineOutput {
        fail_commit: true,
        ..Default::default()
    };
    {
        let mut bridge = fixture.engine.begin_context_bridge_output(Some(&token));
        let failure = zbus::block_on(WindowInteraction::execute_local(
            &mut bridge,
            authority,
            request,
            &mut EngineOutput::test(&mut test_output),
        ))
        .expect_err("commit failure after accepted delete");
        assert_eq!(failure.progress(), LocalEffectProgress::DeleteDispatched);
        assert_eq!(
            failure.receipt(),
            ExecutionReceipt::LocalIndeterminatePartial
        );
    }
    let delete = test_output
        .effects
        .iter()
        .position(|effect| *effect == "delete")
        .unwrap();
    let commit = test_output
        .effects
        .iter()
        .position(|effect| *effect == "commit")
        .unwrap();
    assert!(delete < commit);
    assert_eq!(test_output.surrounding_deletes, [(-6, 6)]);
    assert_eq!(test_output.committed_texts, ["привет"]);
    assert!(fixture.engine.context_bridge_token.is_none());
    assert!(!fixture.engine.context_word_is_known());
    assert!(!fixture.adapter.revalidate(&token));
}

#[test]
fn window_interaction_manual_toggle_terminal_failure_preserves_progress_and_revokes() {
    let mut fixture = admitted_engine();
    fixture.engine.committed_tail.buffer = "ghbdtn".into();
    fixture.engine.set_content_type_state(10, 0);
    fixture.engine.client_context.cursor_cell_width = 8;
    let token = fixture.engine.live_context_token().expect("live token");
    let authority = WindowInteraction::admit_manual_toggle(&fixture.engine);
    assert_eq!(authority, TextTargetAuthority::LocalTerminalErase);
    let mut test_output = TestEngineOutput {
        fail_commit: true,
        ..Default::default()
    };
    {
        let mut bridge = fixture.engine.begin_context_bridge_output(Some(&token));
        let failure = zbus::block_on(WindowInteraction::execute_manual_toggle(
            &mut bridge,
            authority,
            &mut EngineOutput::test(&mut test_output),
        ))
        .expect_err("terminal CommitText failure");
        assert_eq!(failure.progress(), LocalEffectProgress::CursorOrPreedit);
        assert_eq!(
            failure.receipt(),
            ExecutionReceipt::LocalIndeterminatePartial
        );
    }
    assert!(test_output.effects.contains(&"commit"));
    assert!(fixture.engine.context_bridge_token.is_none());
    assert!(!fixture.engine.context_word_is_known());
    assert!(!fixture.adapter.revalidate(&token));

    // The auto-undo wrapper must preserve the destructive delete receipt when
    // the following commit fails; the terminal case above has no separate
    // DeleteSurroundingText effect and therefore cannot prove this boundary.
    let mut undo = admitted_engine();
    undo.engine.committed_tail.buffer = "собака ".into();
    undo.engine
        .set_client_capabilities(IBUS_CAP_SURROUNDING_TEXT);
    undo.engine.client_context.surrounding_text_snapshot =
        Some(SurroundingTextSnapshot::new("собака ".into(), 7, 7));
    undo.engine.remember_pending_ime_auto_undo(
        "cj,frf ".into(),
        "собака ".into(),
        lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
    );
    let undo_token = undo.engine.live_context_token().expect("live undo token");
    let undo_authority = WindowInteraction::admit_manual_toggle(&undo.engine);
    assert_eq!(undo_authority, TextTargetAuthority::DelegateExactImeTail);
    let mut undo_output = TestEngineOutput {
        fail_commit: true,
        ..Default::default()
    };
    {
        let mut bridge = undo.engine.begin_context_bridge_output(Some(&undo_token));
        let failure = zbus::block_on(WindowInteraction::execute_manual_toggle(
            &mut bridge,
            undo_authority,
            &mut EngineOutput::test(&mut undo_output),
        ))
        .expect_err("auto-undo CommitText failure after accepted delete");
        assert_eq!(failure.progress(), LocalEffectProgress::DeleteDispatched);
        assert_eq!(
            failure.receipt(),
            ExecutionReceipt::LocalIndeterminatePartial
        );
    }
    assert_eq!(undo_output.surrounding_deletes, [(-7, 7)]);
    assert_eq!(undo_output.committed_texts, ["cj,frf "]);
    assert!(undo.engine.context_bridge_token.is_none());
    assert!(!undo.engine.context_word_is_known());
    assert!(!undo.adapter.revalidate(&undo_token));
}

#[test]
fn window_interaction_priority_routes_keep_existing_owners() {
    let mut active = engine();
    active.composition.buffer = "ghbdtn".into();
    active.composition.cursor = 6;
    let active_authority = WindowInteraction::admit_manual_toggle(&active);
    assert_eq!(
        active_authority,
        TextTargetAuthority::ActiveCompositionOwned
    );
    let mut active_output = TestEngineOutput::default();
    let (active_layout, active_receipt) = zbus::block_on(WindowInteraction::execute_manual_toggle(
        &mut active,
        active_authority,
        &mut EngineOutput::test(&mut active_output),
    ))
    .unwrap();
    assert_eq!(active_layout, Some(true));
    assert_eq!(active_receipt, ExecutionReceipt::LocalComplete);
    assert_eq!(active_output.committed_texts, ["привет"]);
    assert!(active.composition.buffer.is_empty());

    let mut atomic = engine();
    atomic.composition.buffer = "ghbdtn".into();
    atomic.atomic.active = true;
    let atomic_authority = WindowInteraction::admit_manual_toggle(&atomic);
    assert_eq!(atomic_authority, TextTargetAuthority::AtomicOwned);
    let mut atomic_output = TestEngineOutput::default();
    assert_eq!(
        zbus::block_on(WindowInteraction::execute_manual_toggle(
            &mut atomic,
            atomic_authority,
            &mut EngineOutput::test(&mut atomic_output),
        ))
        .unwrap(),
        (None, ExecutionReceipt::Rejected)
    );
    assert!(atomic_output.effects.is_empty());
    assert!(atomic.atomic.active);
    assert_eq!(atomic.composition.buffer, "ghbdtn");

    let mut undo = engine();
    assert!(undo.bind_focus_path());
    undo.set_client_capabilities(IBUS_CAP_SURROUNDING_TEXT);
    undo.committed_tail.buffer = "собака ".into();
    undo.client_context.surrounding_text_snapshot =
        Some(SurroundingTextSnapshot::new("собака ".into(), 7, 7));
    undo.remember_pending_ime_auto_undo(
        "cj,frf ".into(),
        "собака ".into(),
        lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
    );
    let undo_authority = WindowInteraction::admit_manual_toggle(&undo);
    assert_eq!(undo_authority, TextTargetAuthority::DelegateExactImeTail);
    let mut undo_output = TestEngineOutput::default();
    let (undo_layout, undo_receipt) = zbus::block_on(WindowInteraction::execute_manual_toggle(
        &mut undo,
        undo_authority,
        &mut EngineOutput::test(&mut undo_output),
    ))
    .unwrap();
    assert_eq!(undo_layout, Some(false));
    assert_eq!(undo_receipt, ExecutionReceipt::LocalComplete);
    assert_eq!(undo_output.surrounding_deletes, [(-7, 7)]);
    assert_eq!(undo_output.committed_texts, ["cj,frf "]);
    assert_eq!(undo.committed_tail.buffer, "cj,frf ");

    let mut terminal = engine();
    terminal.committed_tail.buffer = "abc".into();
    terminal.set_content_type_state(10, 0);
    terminal.client_context.cursor_cell_width = 8;
    assert_eq!(
        WindowInteraction::admit_manual_toggle(&terminal),
        TextTargetAuthority::LocalTerminalErase
    );

    let mut exact = engine();
    exact.committed_tail.buffer = "abc".into();
    exact.set_client_capabilities(1 << 5);
    assert_eq!(
        WindowInteraction::admit_manual_toggle(&exact),
        TextTargetAuthority::DelegateExactImeTail
    );
    assert_eq!(
        WindowInteraction::admit_manual_toggle(&engine()),
        TextTargetAuthority::DelegateDaemonBuffer
    );
}

#[test]
fn window_interaction_only_boundary_has_full_callsite_closure() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ibus = std::fs::read_to_string(root.join("src/bin/lay_ibus_engine/ibus_interface.rs"))
        .expect("IBus wire adapter");
    let ibus_production = ibus.split("#[cfg(test)]").next().unwrap();
    for boundary in [
        "WindowInteraction::process_legacy_key",
        "WindowInteraction::process_atomic_key",
        "WindowInteraction::observe_lifecycle",
        "WindowInteraction::observe_facts",
    ] {
        assert!(
            ibus_production.contains(boundary),
            "missing boundary call {boundary}"
        );
    }
    for bypass in [
        ".begin_context_key_callback(",
        ".observe_context_focus_out(",
        ".observe_context_disable(",
        ".observe_context_revocation(",
        ".set_client_capabilities(",
        ".set_content_type_state(",
        ".observe_external_surrounding_text(",
    ] {
        assert!(
            !ibus_production.contains(bypass),
            "wire adapter bypass {bypass}"
        );
    }

    let bridge = std::fs::read_to_string(root.join("src/bin/lay_ibus_engine/bridge.rs"))
        .expect("bridge wire adapter");
    for (entry, expected_calls) in [
        ("self.owns_active_text_inner()", 1),
        ("self.input_state_inner()", 1),
        ("self.visible_tail_v1_inner()", 1),
        ("self.visible_tail_v2_inner()", 1),
        ("self.visible_tail_v3_inner()", 1),
        ("self.can_replace_committed_tail_inner(backspaces)", 1),
        ("self.suppress_next_autocorrect_inner()", 1),
        ("self.suppress_next_autocorrect_v2_inner(", 1),
        ("self.cancel_exact_manual_toggle_suppression_v2_inner(", 1),
        ("self.cancel_exact_manual_toggle_handoff_v2_inner(", 1),
        ("self.manual_toggle_inner()", 1),
        ("self.manual_toggle_v2_inner()", 1),
        ("self.manual_toggle_v3_inner()", 1),
        ("self.replace_tail_inner(", 4),
    ] {
        assert_eq!(
            bridge.matches(entry).count(),
            expected_calls,
            "bridge entry {entry} must delegate exactly once per wire method"
        );
    }
    for bypass in [
        ".object_server()",
        "begin_context_bridge_output",
        "WindowInteraction::execute_local",
        "WindowInteraction::execute_manual_toggle",
        ".replace_committed_tail(",
        "delete_surrounding_text",
        "commit_text",
        "context_reset_rereceipt",
    ] {
        assert!(!bridge.contains(bypass), "bridge wire bypass {bypass}");
    }

    assert!(!root.join("src/bin/lay_ibus_engine/text_target.rs").exists());
    assert!(!root
        .join("src/bin/lay_ibus_engine/bridge_actions.rs")
        .exists());
    assert!(!root
        .join("src/bin/lay_ibus_engine/context_runtime.rs")
        .exists());
    let module = ["mod.rs", "authority.rs", "execution.rs", "observation.rs"]
        .into_iter()
        .map(|name| {
            std::fs::read_to_string(
                root.join("src/bin/lay_ibus_engine/window_interaction")
                    .join(name),
            )
            .unwrap()
        })
        .collect::<String>();
    for required in [
        "WindowInteraction::execute_local",
        "WindowInteraction::execute_manual_toggle",
        "pub(crate) fn observe_existing_postcondition",
        "begin_context_bridge_output",
        "settle_context_bridge_output",
        "cancel_exact_manual_toggle_handoff_state",
        "cancel_exact_manual_toggle_suppression_v2_inner",
    ] {
        assert!(
            module.contains(required),
            "module omits boundary owner {required}"
        );
    }
    assert!(!module.contains("app_name"));
    assert!(!module.contains("application_name"));
    assert_eq!(
        module
            .matches("pub(crate) struct WindowInteraction;")
            .count(),
        1,
        "the boundary must remain one zero-storage owner"
    );
    for second_owner in [
        "ContextAdmissionReducer",
        "struct WindowInteraction {",
        "AtomicU64",
        "HashMap<",
        "BTreeMap<",
        "VecDeque<",
        "OnceLock<",
        "static mut ",
    ] {
        assert!(
            !module.contains(second_owner),
            "window module added authority storage {second_owner}"
        );
    }
}
