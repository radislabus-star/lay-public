use std::sync::{Arc, Mutex};

use lay::config::LayConfig;

use super::*;
use crate::atomic::td120_test_atomic_capability;
use crate::context_admission::{
    ActivationGrant, ActivationOutcome, ContextAdmissionAdapter, EngineOwner, EnginePath,
    FrameGeneration, OwnerGeneration, ReceiptOrigin, TicketId, TicketKind, TransferGrant,
};
use crate::output::{
    AtomicEffectBuilder, AtomicProposal, EngineOutput, PROPOSAL_FRAME_READY,
    PROPOSAL_NATIVE_UNHANDLED,
};
use crate::protocol::{Shared, KEY_LEFT_SHIFT, KEY_SPACE, KEY_TAB};

const CONTEXT_PATH: &str = "/org/freedesktop/IBus/InputContext_td121";
const TAIL_EPOCH: u64 = 17;

struct InstalledEngine {
    engine: LayIbusEngine,
    adapter: ContextAdmissionAdapter,
    grant: ActivationGrant,
    _peer: zbus::Connection,
}

fn config() -> LayConfig {
    LayConfig {
        auto_replace: true,
        auto_switch_layout: false,
        text_backend: "ime".to_string(),
        ..LayConfig::default()
    }
}

fn installed_engine(path: &str, completeness: WordCompleteness) -> InstalledEngine {
    let shared = Arc::new(Mutex::new(Default::default()));
    let (adapter, grant, peer) =
        ContextAdmissionAdapter::test_established(path, CONTEXT_PATH, completeness, TAIL_EPOCH);
    let mut engine = LayIbusEngine::new_from_component(
        path.to_string(),
        shared,
        Some(adapter.clone()),
        "lay-ime-us",
        true,
        config(),
    );
    assert!(engine.install_context_activation(ActivationOutcome::SourceFree(grant.clone())));
    InstalledEngine {
        engine,
        adapter,
        grant,
        _peer: peer,
    }
}

fn active_completion(engine: &mut LayIbusEngine) {
    engine.composition.buffer = "пров".to_string();
    engine.composition.cursor = 4;
    engine.composition.preedit_suffix = "ерка".to_string();
    engine.composition.preedit_candidates = vec!["ерка".to_string()];
    engine.composition.preedit_replacement_targets = vec![None];
}

#[test]
fn bridge_output_scope_clears_its_witness_on_success_and_cancellation() {
    for complete in [false, true] {
        let mut fixture = installed_engine("/engine/bridge_scope", WordCompleteness::KnownStart);
        let token = fixture.engine.live_context_token().unwrap();
        {
            let mut output = fixture.engine.begin_context_bridge_output(Some(&token));
            output.committed_tail.buffer = "text".into();
            assert!(output.publish_tail_handoff());
            if complete {
                output.complete();
            }
        }
        assert!(fixture.engine.context_bridge_token.is_none());
        assert_eq!(fixture.engine.context_word_is_known(), complete);
        assert_eq!(fixture.adapter.revalidate(&token), complete);
    }
}

fn press(engine: &mut LayIbusEngine, keyval: u32) -> (bool, AtomicProposal) {
    let mut builder = AtomicEffectBuilder::default();
    let handled = {
        let mut output = EngineOutput::atomic(&mut builder);
        zbus::block_on(engine.process_pressed_key(&mut output, keyval, 0, 0))
            .expect("actual managed key route")
    };
    (handled, builder.finish(handled))
}

fn committed_texts(proposal: &AtomicProposal) -> Vec<String> {
    proposal
        .1
        .iter()
        .filter(|(tag, _)| *tag == 1)
        .map(|(_, value)| String::try_from(value.clone()).expect("CommitText string"))
        .collect()
}

fn observed_suffix_completion() -> InstalledEngine {
    let mut fixture = installed_engine("/engine/observed_suffix", WordCompleteness::UnknownStart);
    let engine = &mut fixture.engine;
    for ch in "пров".chars() {
        engine.push_tail_char(ch);
        engine
            .context_word_scope
            .as_mut()
            .unwrap()
            .observe_tail_append(engine.committed_tail.buffer.chars().count() as u32);
    }
    assert!(fixture.adapter.test_set_word_scope(
        engine.context_owner.as_ref().unwrap(),
        engine.committed_tail.epoch,
        engine.context_word_scope.as_ref().unwrap(),
    ));
    engine.context_token = fixture.adapter.current_token();
    engine.observe_external_surrounding_text(Some(crate::engine::SurroundingTextSnapshot::new(
        "пров".into(),
        4,
        4,
    )));
    engine.composition.preedit_candidates = vec!["ерка".into()];
    engine.composition.preedit_replacement_targets = vec![None];
    engine.composition.preedit_suffix = "ерка".into();
    engine.composition.preedit_visible = true;
    fixture
}

#[test]
fn first_word_suffix_has_display_identity_but_no_whole_word_edit_identity() {
    let fixture = observed_suffix_completion();
    let engine = &fixture.engine;
    let frame = engine
        .capture_observed_suffix_display_frame()
        .expect("observed display frame");
    assert!(engine.precognition_identity_matches(&frame));
    assert!(!engine.input_frame_identity_matches(&frame));
    assert!(engine.capture_input_frame_identity().is_none());
    assert!(!engine.context_word_is_known());
    assert_eq!(frame.observed_token, "пров");
}

#[test]
fn first_word_tab_appends_only_selected_suffix_without_whole_word_feedback() {
    assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    let mut fixture = observed_suffix_completion();
    let (handled, proposal) = press(&mut fixture.engine, KEY_TAB);
    assert!(handled);
    assert_eq!(committed_texts(&proposal), ["ерка "]);
    assert!(proposal.1.iter().all(|(tag, _)| *tag != 2));
    assert_eq!(fixture.engine.committed_tail.buffer, "проверка ");
    assert!(!fixture.engine.context_word_is_known());
    assert!(fixture
        .engine
        .committed_tail
        .pending_completion_learning
        .is_none());
    assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
}

#[test]
fn first_word_snapshot_contradictions_refuse_display_and_tab_without_text_effects() {
    for (text, cursor, anchor) in [
        ("запров", 6, 6),
        ("проверь", 4, 4),
        ("пров", 4, 0),
        ("друг", 4, 4),
        ("пров", 9, 9),
    ] {
        let mut fixture = observed_suffix_completion();
        fixture.engine.observe_external_surrounding_text(Some(
            crate::engine::SurroundingTextSnapshot::new(text.into(), cursor, anchor),
        ));
        assert!(fixture
            .engine
            .capture_observed_suffix_display_frame()
            .is_none());
        let (handled, proposal) = press(&mut fixture.engine, KEY_TAB);
        assert!(!handled, "{text} {cursor} {anchor}");
        assert!(committed_texts(&proposal).is_empty());
        assert!(proposal.1.iter().all(|(tag, _)| *tag != 2));
        assert_eq!(fixture.engine.committed_tail.buffer, "пров");
        assert!(fixture
            .engine
            .committed_tail
            .pending_completion_learning
            .is_none());
    }
}

#[test]
fn first_word_repeated_surrounding_ack_keeps_visible_tab_completion() {
    let mut fixture = observed_suffix_completion();
    let emitter =
        zbus::object_server::SignalEmitter::new(&fixture._peer, fixture.engine.path.clone())
            .unwrap();
    zbus::block_on(fixture.engine.set_surrounding_text(
        emitter,
        crate::text::make_ibus_text("пров".into()),
        4,
        4,
    ))
    .unwrap();
    assert!(fixture.engine.composition.preedit_visible);
    assert!(!fixture.engine.composition.preedit_display_only_pending);
    assert_eq!(fixture.engine.composition.preedit_suffix, "ерка");
    let (handled, proposal) = press(&mut fixture.engine, KEY_TAB);
    assert!(handled);
    assert_eq!(committed_texts(&proposal), ["ерка "]);
    assert!(proposal.1.iter().all(|(tag, _)| *tag != 2));
}

#[test]
fn first_word_stale_lineage_cannot_reuse_equal_text_display_frame() {
    let mut fixture = observed_suffix_completion();
    let frame = fixture
        .engine
        .capture_observed_suffix_display_frame()
        .unwrap();
    let scope = fixture.engine.context_word_scope.as_mut().unwrap();
    scope.revoke_for_input_gap();
    for count in 1..=4 {
        scope.observe_tail_append(count);
    }
    assert!(fixture.adapter.test_set_word_scope(
        fixture.engine.context_owner.as_ref().unwrap(),
        fixture.engine.committed_tail.epoch,
        fixture.engine.context_word_scope.as_ref().unwrap(),
    ));
    fixture.engine.context_token = fixture.adapter.current_token();
    let next = fixture
        .engine
        .capture_observed_suffix_display_frame()
        .unwrap();
    assert_eq!(frame.committed_tail, next.committed_tail);
    assert!(!fixture.engine.precognition_identity_matches(&frame));
    assert!(fixture.engine.precognition_identity_matches(&next));
    assert!(fixture.engine.capture_input_frame_identity().is_none());
}

#[test]
fn first_word_refuses_unobserved_sensitive_missing_snapshot_and_replacement_paths() {
    for route in [
        "unobserved",
        "sensitive",
        "snapshot",
        "handoff",
        "atomic",
        "replacement",
    ] {
        let mut fixture = observed_suffix_completion();
        match route {
            "unobserved" => fixture
                .engine
                .context_word_scope
                .as_mut()
                .unwrap()
                .revoke_for_input_gap(),
            "sensitive" => fixture.engine.client_context.content_purpose = 8,
            "snapshot" => fixture.engine.client_context.surrounding_text_snapshot = None,
            "handoff" => fixture.engine.context_handoff_sealed = true,
            "atomic" => fixture.engine.atomic.active = true,
            "replacement" => {
                fixture.engine.composition.preedit_replacement_targets = vec![Some("другое".into())]
            }
            _ => unreachable!(),
        }
        let (handled, proposal) = press(&mut fixture.engine, KEY_TAB);
        assert!(!handled, "{route}");
        assert!(committed_texts(&proposal).is_empty(), "{route}");
        assert!(proposal.1.iter().all(|(tag, _)| *tag != 2));
        assert_eq!(fixture.engine.committed_tail.buffer, "пров");
    }
}

#[test]
fn td121_unknown_start_actual_tab_refuses_active_composition_without_output_or_feedback() {
    assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    let mut fixture = installed_engine(
        "/io/github/radislabus_star/LayIme/engine/td121_unknown_tab",
        WordCompleteness::UnknownStart,
    );
    active_completion(&mut fixture.engine);

    let (handled, proposal) = press(&mut fixture.engine, KEY_TAB);

    assert!(!handled);
    assert_eq!(proposal, (PROPOSAL_NATIVE_UNHANDLED, Vec::new()));
    assert_eq!(fixture.engine.composition.buffer, "пров");
    assert!(fixture.engine.composition.preedit_suffix.is_empty());
    assert!(fixture
        .engine
        .committed_tail
        .pending_completion_learning
        .is_none());
    assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
}

#[test]
fn td121_known_start_actual_tab_accepts_complete_word() {
    assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    let mut fixture = installed_engine(
        "/io/github/radislabus_star/LayIme/engine/td121_known_tab",
        WordCompleteness::KnownStart,
    );
    active_completion(&mut fixture.engine);

    let (handled, proposal) = press(&mut fixture.engine, KEY_TAB);

    assert!(handled);
    assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
    assert_eq!(committed_texts(&proposal), ["проверка "]);
    assert_eq!(fixture.engine.committed_tail.buffer, "проверка ");
    assert!(fixture.engine.composition.buffer.is_empty());
    assert!(fixture
        .engine
        .committed_tail
        .pending_completion_learning
        .is_some());
    assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
}

#[test]
fn td121_unknown_start_actual_space_delivers_literal_without_correction_or_feedback() {
    assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    let mut fixture = installed_engine(
        "/io/github/radislabus_star/LayIme/engine/td121_unknown_literal",
        WordCompleteness::UnknownStart,
    );
    fixture.engine.composition.buffer = "ljv".to_string();
    fixture.engine.composition.cursor = 3;

    let (handled, proposal) = press(&mut fixture.engine, KEY_SPACE);

    assert!(handled);
    assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
    assert_eq!(committed_texts(&proposal), ["ljv "]);
    assert!(proposal.1.iter().all(|(tag, _)| *tag != 2));
    assert_eq!(fixture.engine.committed_tail.buffer, "ljv ");
    assert!(fixture.engine.composition.buffer.is_empty());

    fixture.engine.composition.buffer = "ghbdtn".to_string();
    fixture.engine.composition.cursor = 6;
    let mut manual_builder = AtomicEffectBuilder::default();
    let manual = {
        let mut output = EngineOutput::atomic(&mut manual_builder);
        zbus::block_on(fixture.engine.manual_toggle_active_text_target(&mut output))
            .expect("actual manual route")
    };
    assert_eq!(manual, None);
    assert_eq!(
        manual_builder.finish(false),
        (PROPOSAL_NATIVE_UNHANDLED, Vec::new())
    );
    assert_eq!(fixture.engine.composition.buffer, "ghbdtn");
    assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
}

#[test]
fn td121_unknown_space_then_boundary_backspace_refuses_tab_completion() {
    let mut fixture = installed_engine(
        "/io/github/radislabus_star/LayIme/engine/td121_boundary_backspace",
        WordCompleteness::UnknownStart,
    );
    fixture.engine.composition.buffer = "ljv".to_string();
    fixture.engine.composition.cursor = 3;

    let before_space = fixture.engine.committed_tail.buffer.clone();
    let (space_handled, _) = press(&mut fixture.engine, KEY_SPACE);
    assert!(space_handled);
    fixture
        .engine
        .advance_context_word_scope(KEY_SPACE, 0, 0, &before_space, true);
    let rearmed_scope = *fixture.engine.context_word_scope.as_ref().unwrap();
    assert!(fixture.adapter.test_set_word_scope(
        &fixture.grant.target_owner,
        fixture.engine.committed_tail.epoch,
        &rearmed_scope,
    ));
    fixture.engine.context_token = fixture.adapter.current_token();
    assert!(fixture.engine.context_word_is_known());

    let before_backspace = fixture.engine.committed_tail.buffer.clone();
    let (backspace_handled, _) = press(&mut fixture.engine, crate::protocol::KEY_BACKSPACE);
    assert!(!backspace_handled);
    assert!(before_backspace.ends_with(' '));
    fixture.engine.advance_context_word_scope(
        crate::protocol::KEY_BACKSPACE,
        0,
        0,
        &before_backspace,
        false,
    );
    assert!(!fixture.engine.context_word_is_known());

    active_completion(&mut fixture.engine);
    let (tab_handled, proposal) = press(&mut fixture.engine, KEY_TAB);
    assert!(!tab_handled);
    assert_eq!(proposal, (PROPOSAL_NATIVE_UNHANDLED, Vec::new()));
    assert!(fixture
        .engine
        .committed_tail
        .pending_completion_learning
        .is_none());
}

fn transfer_fixture(
    target_path: &str,
    source_path: &str,
    active_path: &str,
    transferred_tail: &str,
    shared_epoch: u64,
) -> InstalledEngine {
    let shared: Shared = Arc::new(Mutex::new(Default::default()));
    {
        let mut state = shared.lock().expect("TD-121 transfer shared state");
        state.active_path = Some(active_path.to_string());
        state.context_owner_generation = Some(OwnerGeneration(77).0);
        state.handoff_tail_buffer = transferred_tail.to_string();
        state.handoff_tail_epoch = shared_epoch;
    }
    let (adapter, grant, peer) = ContextAdmissionAdapter::test_established(
        target_path,
        CONTEXT_PATH,
        WordCompleteness::KnownStart,
        TAIL_EPOCH,
    );
    let mut engine = LayIbusEngine::new_from_component(
        target_path.to_string(),
        shared,
        Some(adapter.clone()),
        "lay-ime-us",
        true,
        config(),
    );
    let transfer = TransferGrant {
        revocation: grant.revocation,
        ticket: TicketId(41),
        kind: TicketKind::Factory,
        source_owner: EngineOwner {
            path: EnginePath::new(source_path).expect("TD-121 source path"),
            generation: OwnerGeneration(77),
        },
        target_owner: grant.target_owner.clone(),
        target_activation: grant.target_activation.clone(),
        lineage: grant.lineage,
        source_tail_epoch: TAIL_EPOCH,
        frame_generation: FrameGeneration(9),
        invalidate_prior_authority: true,
        receipt_origin: ReceiptOrigin::Native,
    };
    let outcome = ActivationOutcome::Transfer(transfer);
    adapter.test_bind_activation_outcome(&outcome);
    let installed = engine.install_context_activation(outcome);
    assert_eq!(
        installed,
        active_path == source_path && shared_epoch == TAIL_EPOCH
    );
    InstalledEngine {
        engine,
        adapter,
        grant,
        _peer: peer,
    }
}

#[test]
fn td121_same_context_transfer_preserves_full_and_mixed_partial_words() {
    for (suffix, tail) in [("full", "ljv"), ("mixed", "lом")] {
        let source_path =
            format!("/io/github/radislabus_star/LayIme/engine/td121_transfer_source_{suffix}");
        let target_path =
            format!("/io/github/radislabus_star/LayIme/engine/td121_transfer_target_{suffix}");
        let fixture = transfer_fixture(&target_path, &source_path, &source_path, tail, TAIL_EPOCH);
        assert_eq!(fixture.engine.committed_tail.buffer, tail);
        assert_eq!(fixture.engine.committed_tail.epoch, TAIL_EPOCH);
        assert!(fixture.engine.context_word_is_known());
        assert!(fixture.engine.capture_input_frame_identity().is_some());
    }
}

#[test]
fn td121_runtime_rejects_different_field_aba_epoch_and_stale_owner_generation() {
    let different_field = transfer_fixture(
        "/io/github/radislabus_star/LayIme/engine/td121_different_target",
        "/io/github/radislabus_star/LayIme/engine/td121_expected_source",
        "/io/github/radislabus_star/LayIme/engine/td121_foreign_source",
        "ljv",
        TAIL_EPOCH,
    );
    assert!(different_field.engine.committed_tail.buffer.is_empty());
    assert!(!different_field.engine.context_word_is_known());

    let stale_epoch = transfer_fixture(
        "/io/github/radislabus_star/LayIme/engine/td121_aba_target",
        "/io/github/radislabus_star/LayIme/engine/td121_aba_source",
        "/io/github/radislabus_star/LayIme/engine/td121_aba_source",
        "lом",
        TAIL_EPOCH + 1,
    );
    assert!(stale_epoch.engine.committed_tail.buffer.is_empty());
    assert!(!stale_epoch.engine.context_word_is_known());

    let mut stale_owner = installed_engine(
        "/io/github/radislabus_star/LayIme/engine/td121_stale_owner",
        WordCompleteness::KnownStart,
    );
    stale_owner.engine.committed_tail.buffer = "пров".to_string();
    assert!(stale_owner.engine.context_word_is_known());
    assert!(stale_owner
        .adapter
        .revoke_current_owner(&stale_owner.grant.target_owner));
    assert!(!stale_owner.engine.context_word_is_known());
    assert!(stale_owner.engine.capture_input_frame_identity().is_none());
}

fn atomic_envelope(transaction: u64) -> crate::atomic::AtomicEnvelope {
    (transaction, 2, 12, 91, 5, 13, vec![8; 32])
}

#[test]
fn td121_atomic_known_commit_and_stale_token_refusal() {
    let mut normal = installed_engine(
        "/io/github/radislabus_star/LayIme/engine/td121_atomic_normal",
        WordCompleteness::KnownStart,
    );
    normal.engine.committed_tail.buffer = "abc".to_string();
    normal.engine.rebuild_preedit_fast_from_tail();
    let proposal = zbus::block_on(normal.engine.process_atomic_key_event(
        KEY_SPACE,
        65,
        0,
        atomic_envelope(701),
        td120_test_atomic_capability(),
        (0, 0, Vec::new()),
    ))
    .expect("known-context atomic proposal");
    assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
    let settled = zbus::block_on(normal.engine.process_atomic_key_event(
        KEY_LEFT_SHIFT,
        42,
        0,
        atomic_envelope(702),
        td120_test_atomic_capability(),
        (2, 701, vec![9; 32]),
    ))
    .expect("known-context atomic settlement");
    assert_eq!(settled.0, PROPOSAL_NATIVE_UNHANDLED);
    assert_eq!(normal.engine.committed_tail.buffer, "abc ");

    let mut stale = installed_engine(
        "/io/github/radislabus_star/LayIme/engine/td121_atomic_stale",
        WordCompleteness::KnownStart,
    );
    stale.engine.committed_tail.buffer = "xyz".to_string();
    stale.engine.rebuild_preedit_fast_from_tail();
    let proposal = zbus::block_on(stale.engine.process_atomic_key_event(
        KEY_SPACE,
        65,
        0,
        atomic_envelope(711),
        td120_test_atomic_capability(),
        (0, 0, Vec::new()),
    ))
    .expect("pre-revocation atomic proposal");
    assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
    assert!(stale
        .adapter
        .revoke_current_owner(&stale.grant.target_owner));
    let refused = zbus::block_on(stale.engine.process_atomic_key_event(
        KEY_LEFT_SHIFT,
        42,
        0,
        atomic_envelope(712),
        td120_test_atomic_capability(),
        (2, 711, vec![9; 32]),
    ))
    .expect("stale-token atomic refusal");
    assert_eq!(refused, (PROPOSAL_NATIVE_UNHANDLED, Vec::new()));
    assert_eq!(stale.engine.committed_tail.buffer, "xyz");
}

#[test]
fn td121_late_activation_failure_cannot_overwrite_newer_shared_owner() {
    let path = "/io/github/radislabus_star/LayIme/engine/td121_late_failure";
    let mut fixture = installed_engine(path, WordCompleteness::KnownStart);
    let newer_path = "/io/github/radislabus_star/LayIme/engine/td121_newer_owner";
    let newer_generation = fixture.grant.target_owner.generation.0 + 1;
    {
        let mut shared = fixture.engine.shared.lock().unwrap();
        shared.active_path = Some(newer_path.to_string());
        shared.context_owner_generation = Some(newer_generation);
        shared.handoff_tail_buffer = "newer-tail".to_string();
        shared.handoff_tail_epoch = 99;
    }

    fixture.engine.fail_context_activation();

    let shared = fixture.engine.shared.lock().unwrap();
    assert_eq!(shared.active_path.as_deref(), Some(newer_path));
    assert_eq!(shared.context_owner_generation, Some(newer_generation));
    assert_eq!(shared.handoff_tail_buffer, "newer-tail");
    assert_eq!(shared.handoff_tail_epoch, 99);
}

#[test]
fn td121_stale_source_free_grant_cannot_overwrite_newer_shared_owner() {
    let path = "/io/github/radislabus_star/LayIme/engine/td121_stale_source_free";
    let shared: Shared = Arc::new(Mutex::new(Default::default()));
    let (adapter, grant, peer) = ContextAdmissionAdapter::test_established(
        path,
        CONTEXT_PATH,
        WordCompleteness::UnknownStart,
        TAIL_EPOCH,
    );
    let newer_path = "/io/github/radislabus_star/LayIme/engine/td121_newer_source_free";
    let newer_generation = grant.target_owner.generation.0 + 1;
    {
        let mut state = shared.lock().unwrap();
        state.active_path = Some(newer_path.to_string());
        state.context_owner_generation = Some(newer_generation);
        state.handoff_tail_buffer = "newer-source-free-tail".to_string();
        state.handoff_tail_epoch = 101;
    }
    let mut engine = LayIbusEngine::new_from_component(
        path.to_string(),
        shared.clone(),
        Some(adapter),
        "lay-ime-us",
        true,
        config(),
    );

    assert!(!engine.install_context_activation(ActivationOutcome::SourceFree(grant)));

    let state = shared.lock().unwrap();
    assert_eq!(state.active_path.as_deref(), Some(newer_path));
    assert_eq!(state.context_owner_generation, Some(newer_generation));
    assert_eq!(state.handoff_tail_buffer, "newer-source-free-tail");
    assert_eq!(state.handoff_tail_epoch, 101);
    drop(peer);
}
