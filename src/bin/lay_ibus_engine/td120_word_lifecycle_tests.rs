use std::sync::{Arc, Mutex};
use std::time::Instant;

use lay::config::LayConfig;
use lay::text_edit::{VisibleTailSnapshot, VisibleTailSource};

use crate::engine::{LayIbusEngine, SurroundingTextSnapshot};
use crate::output::{
    AtomicEffectBuilder, AtomicProposal, EngineOutput, PROPOSAL_CONSUMED_NO_EFFECT,
    PROPOSAL_FRAME_READY, PROPOSAL_NATIVE_UNHANDLED,
};
use crate::protocol::{
    AutocorrectSuppression, CurrentWordSuppression, ExactManualToggleSuppression, Shared,
    KEY_BACKSPACE, KEY_ENTER,
};
use crate::state::CommittedTailReplaceRequest;

const IBUS_CAP_SURROUNDING_TEXT: u32 = 1 << 5;

struct ManualEditFixture {
    source: &'static str,
    replacement: &'static str,
    target_layout_is_ru: bool,
}

const US_TO_RU: ManualEditFixture = ManualEditFixture {
    source: "ghbdtn",
    replacement: "привет",
    target_layout_is_ru: true,
};

fn unbound_engine(path: &str, shared: Shared, layout_is_ru: bool) -> LayIbusEngine {
    let mut engine = LayIbusEngine::new(
        path.to_string(),
        shared,
        layout_is_ru,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            ..LayConfig::default()
        },
    );
    engine.set_client_capabilities(IBUS_CAP_SURROUNDING_TEXT);
    engine
}

fn isolated_engine(path: &str, layout_is_ru: bool) -> LayIbusEngine {
    let mut engine = unbound_engine(path, Arc::new(Mutex::new(Default::default())), layout_is_ru);
    assert!(engine.bind_focus_path());
    engine
}

fn type_tail(engine: &mut LayIbusEngine, text: &str) {
    for ch in text.chars() {
        engine.push_tail_char(ch);
    }
}

fn erase_tail_chars(engine: &mut LayIbusEngine, count: usize) {
    for _ in 0..count {
        engine.backspace_committed_tail_only();
    }
}

fn recorded_manual_edit(
    engine: &mut LayIbusEngine,
    expected_target_layout_is_ru: bool,
) -> AtomicProposal {
    let mut builder = AtomicEffectBuilder::default();
    let target_layout_is_ru = {
        let mut output = EngineOutput::atomic(&mut builder);
        zbus::block_on(engine.toggle_committed_tail_target(&mut output))
            .expect("typed committed-tail manual edit")
    };
    assert_eq!(target_layout_is_ru, Some(expected_target_layout_is_ru));
    let proposal = builder.finish(true);
    assert_recorded_replacement(&proposal);
    proposal
}

fn recorded_replace(
    engine: &mut LayIbusEngine,
    request: CommittedTailReplaceRequest,
) -> (bool, AtomicProposal) {
    let mut builder = AtomicEffectBuilder::default();
    let handled = {
        let mut output = EngineOutput::atomic(&mut builder);
        zbus::block_on(engine.replace_committed_tail(&mut output, request))
            .expect("typed committed-tail replacement")
    };
    let proposal = builder.finish(handled);
    (handled, proposal)
}

fn assert_recorded_replacement(proposal: &AtomicProposal) {
    assert_recorded_effect_counts(proposal, 1, 1);
}

fn assert_recorded_effect_counts(
    proposal: &AtomicProposal,
    expected_deletes: usize,
    expected_commits: usize,
) {
    assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
    let tags = proposal.1.iter().map(|(tag, _)| *tag).collect::<Vec<_>>();
    assert_eq!(
        tags.iter().filter(|tag| **tag == 2).count(),
        expected_deletes
    );
    assert_eq!(
        tags.iter().filter(|tag| **tag == 1).count(),
        expected_commits
    );
}

fn exact_suppression(scope: &Option<AutocorrectSuppression>) -> ExactManualToggleSuppression {
    match scope {
        Some(AutocorrectSuppression::ExactReplay(identity)) => identity.clone(),
        other => panic!("expected exact replay suppression, got {other:?}"),
    }
}

fn current_word(scope: &Option<AutocorrectSuppression>) -> CurrentWordSuppression {
    match scope {
        Some(AutocorrectSuppression::CurrentWord(current_word)) => *current_word,
        other => panic!("expected current-word suppression, got {other:?}"),
    }
}

#[test]
fn td120_successful_manual_edit_full_erase_new_word_does_not_suppress() {
    let mut engine = isolated_engine("/td120/full-erase", false);
    type_tail(&mut engine, US_TO_RU.source);

    recorded_manual_edit(&mut engine, US_TO_RU.target_layout_is_ru);
    assert_eq!(engine.committed_tail.buffer, US_TO_RU.replacement);

    erase_tail_chars(&mut engine, US_TO_RU.replacement.chars().count());
    assert!(engine.committed_tail.buffer.is_empty());
    engine.backspace_committed_tail_only();
    type_tail(&mut engine, "gjxbnfq");

    assert_eq!(engine.last_tail_token_text(), "gjxbnfq");
    assert!(!engine.take_manual_toggle_autocorrect_suppression());
}

#[test]
fn td120_successful_manual_edit_full_token_erase_with_left_context_does_not_suppress() {
    let mut engine = isolated_engine("/td120/left-context", false);
    type_tail(&mut engine, "push ltkfq");

    recorded_manual_edit(&mut engine, true);
    assert_eq!(engine.committed_tail.buffer, "push делай");

    erase_tail_chars(&mut engine, "делай".chars().count());
    assert_eq!(engine.committed_tail.buffer, "push ");
    type_tail(&mut engine, "ytn");

    assert_eq!(engine.committed_tail.buffer, "push ytn");
    assert!(!engine.take_manual_toggle_autocorrect_suppression());
}

#[test]
fn td120_full_erase_and_identical_retype_is_a_fresh_word() {
    let mut engine = isolated_engine("/td120/identical-retype", false);
    type_tail(&mut engine, US_TO_RU.source);
    recorded_manual_edit(&mut engine, US_TO_RU.target_layout_is_ru);

    erase_tail_chars(&mut engine, US_TO_RU.replacement.chars().count());
    type_tail(&mut engine, US_TO_RU.replacement);

    assert_eq!(engine.last_tail_token_text(), US_TO_RU.replacement);
    assert!(!engine.take_manual_toggle_autocorrect_suppression());
}

#[test]
fn td120_partial_erase_and_layout_letter_punctuation_preserve_guard_once() {
    let mut engine = isolated_engine("/td120/continuing-layout-token", true);
    type_tail(&mut engine, "нужен");

    recorded_manual_edit(&mut engine, false);
    assert_eq!(engine.committed_tail.buffer, "ye;ty");

    engine.backspace_committed_tail_only();
    engine.push_tail_char(']');
    engine.push_tail_char('[');
    engine.push_tail_char('\'');

    assert_eq!(engine.last_tail_token_text(), "ye;t]['");
    assert_eq!(engine.composition.preedit_fast.token(), "ye;t]['");
    assert!(engine.take_manual_toggle_autocorrect_suppression());
    assert!(!engine.take_manual_toggle_autocorrect_suppression());
}

#[test]
fn td120_accepted_candidate_with_trailing_boundary_does_not_protect_next_word() {
    let mut engine = isolated_engine("/td120/candidate-boundary", true);
    type_tail(&mut engine, "пров");

    let (handled, proposal) = recorded_replace(
        &mut engine,
        CommittedTailReplaceRequest::ime_candidate_accept(
            "пров".chars().count() as u32,
            "проверка ".to_string(),
        ),
    );

    assert!(handled);
    assert_recorded_effect_counts(&proposal, 0, 1);
    assert_eq!(engine.committed_tail.buffer, "проверка ");
    assert!(!engine.take_manual_toggle_autocorrect_suppression());
}

#[test]
fn td120_auto_undo_with_trailing_boundary_does_not_protect_next_word() {
    let mut engine = isolated_engine("/td120/undo-boundary", true);
    type_tail(&mut engine, "проверка ");

    let (handled, proposal) = recorded_replace(
        &mut engine,
        CommittedTailReplaceRequest::ime_auto_undo(
            "проверка ".chars().count() as u32,
            "проверрка ".to_string(),
        ),
    );

    assert!(handled);
    assert_recorded_replacement(&proposal);
    assert_eq!(engine.committed_tail.buffer, "проверрка ");
    assert!(!engine.take_manual_toggle_autocorrect_suppression());
}

#[test]
fn td120_rejected_output_does_not_newly_arm_suppression() {
    let mut engine = isolated_engine("/td120/rejected", false);
    type_tail(&mut engine, "abc");
    let stale_tail = VisibleTailSnapshot::new(
        VisibleTailSource::ImeCommittedTail,
        "different".to_string(),
        Some(engine.path.clone()),
        engine.committed_tail.epoch,
    );

    let (handled, proposal) = recorded_replace(
        &mut engine,
        CommittedTailReplaceRequest::ime_manual_toggle(3, "фис".to_string(), true)
            .with_expected_tail(stale_tail),
    );

    assert!(!handled);
    assert_eq!(proposal.0, PROPOSAL_NATIVE_UNHANDLED);
    assert!(proposal.1.is_empty());
    assert!(!engine.take_manual_toggle_autocorrect_suppression());
}

#[test]
fn td120_duplicate_output_does_not_newly_arm_suppression() {
    let mut engine = isolated_engine("/td120/duplicate", false);
    engine.set_client_capabilities(0);
    engine.set_content_type_state(10, 0);
    engine.client_context.cursor_cell_width = 9;
    type_tail(&mut engine, "a  ");

    let (first_handled, first_proposal) = recorded_replace(
        &mut engine,
        CommittedTailReplaceRequest::daemon_bridge(1, String::new(), true),
    );
    assert!(first_handled);
    assert_eq!(first_proposal.0, PROPOSAL_FRAME_READY);
    assert_eq!(engine.committed_tail.buffer, "a ");
    assert!(!engine.take_manual_toggle_autocorrect_suppression());

    let (duplicate_handled, duplicate_proposal) = recorded_replace(
        &mut engine,
        CommittedTailReplaceRequest::daemon_bridge(1, String::new(), true),
    );

    assert!(duplicate_handled);
    assert_eq!(duplicate_proposal.0, PROPOSAL_CONSUMED_NO_EFFECT);
    assert!(duplicate_proposal.1.is_empty());
    assert_eq!(engine.committed_tail.buffer, "a ");
    assert!(!engine.take_manual_toggle_autocorrect_suppression());
}

#[test]
fn td120_duplicate_open_result_preserves_existing_guard_once() {
    let mut engine = isolated_engine("/td120/duplicate-open", false);
    type_tail(&mut engine, "abc");

    let request = CommittedTailReplaceRequest::daemon_bridge(1, "x".to_string(), true);
    let (first_handled, first_proposal) = recorded_replace(&mut engine, request.clone());
    assert!(first_handled);
    assert_recorded_effect_counts(&first_proposal, 1, 1);
    assert_eq!(engine.committed_tail.buffer, "abx");

    let (duplicate_handled, duplicate_proposal) = recorded_replace(&mut engine, request);
    assert!(duplicate_handled);
    assert_eq!(duplicate_proposal.0, PROPOSAL_CONSUMED_NO_EFFECT);
    assert!(duplicate_proposal.1.is_empty());
    assert_eq!(engine.committed_tail.buffer, "abx");
    assert!(engine.take_manual_toggle_autocorrect_suppression());
    assert!(!engine.take_manual_toggle_autocorrect_suppression());
}

#[test]
fn td120_exact_replay_suppression_survives_temporary_empty_tail_and_exact_revoke() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut source = unbound_engine("/td120/exact/us", shared.clone(), false);
    assert!(source.bind_focus_path());
    type_tail(&mut source, US_TO_RU.source);
    source.prepare_exact_manual_toggle_layout_handoff();
    let leased_epoch = source.committed_tail.epoch;

    let mut target = unbound_engine("/td120/exact/ru", shared.clone(), true);
    assert!(target.bind_focus_path());
    target.reset_for_ibus_soft_reset();
    target.client_context.surrounding_text_snapshot = Some(SurroundingTextSnapshot::new(
        US_TO_RU.source.to_string(),
        US_TO_RU.source.chars().count() as u32,
        US_TO_RU.source.chars().count() as u32,
    ));
    assert_eq!(target.committed_tail.buffer, US_TO_RU.source);
    assert!(target.arm_exact_manual_toggle_autocorrect_suppression(
        US_TO_RU.source,
        leased_epoch,
        "/td120/exact/ru",
        true,
    ));

    let armed = exact_suppression(&target.committed_tail.autocorrect_suppression);
    assert_eq!(armed.path, "/td120/exact/ru");
    assert_eq!(armed.epoch, leased_epoch);
    assert!(armed.expires_at > Instant::now());

    erase_tail_chars(&mut target, US_TO_RU.source.chars().count());
    assert!(target.committed_tail.buffer.is_empty());
    assert_eq!(
        exact_suppression(&target.committed_tail.autocorrect_suppression),
        armed
    );
    assert_eq!(
        exact_suppression(
            &shared
                .lock()
                .expect("lay ime state poisoned")
                .autocorrect_suppression
        ),
        armed
    );

    type_tail(&mut target, US_TO_RU.replacement);
    assert_eq!(
        exact_suppression(&target.committed_tail.autocorrect_suppression),
        armed
    );
    assert!(!target.revoke_exact_manual_toggle_autocorrect_suppression(
        leased_epoch.wrapping_add(1),
        "/td120/exact/ru",
    ));
    assert!(!target
        .revoke_exact_manual_toggle_autocorrect_suppression(leased_epoch, "/td120/exact/us",));
    assert!(
        target.revoke_exact_manual_toggle_autocorrect_suppression(leased_epoch, "/td120/exact/ru",)
    );
    assert!(!target.take_manual_toggle_autocorrect_suppression());
}

#[test]
fn td120_active_manual_and_candidate_open_results_arm_current_word() {
    let mut manual = isolated_engine("/td120/active-manual", false);
    for ch in US_TO_RU.source.chars() {
        manual.insert_composition_char(ch);
    }
    let mut manual_builder = AtomicEffectBuilder::default();
    let manual_target = {
        let mut output = EngineOutput::atomic(&mut manual_builder);
        zbus::block_on(manual.manual_toggle_active_text_target(&mut output))
            .expect("active manual edit")
    };
    assert_eq!(manual_target, Some(true));
    assert_recorded_effect_counts(&manual_builder.finish(true), 0, 1);
    let manual_guard = current_word(&manual.committed_tail.autocorrect_suppression);
    assert_eq!(
        manual_guard.open_token_chars,
        US_TO_RU.replacement.chars().count()
    );

    let mut candidate = isolated_engine("/td120/active-candidate", true);
    for ch in "пров".chars() {
        candidate.insert_composition_char(ch);
    }
    candidate.composition.preedit_suffix = "ерка".to_string();
    candidate.composition.preedit_candidates = vec!["ерка".to_string()];
    candidate.composition.preedit_replacement_targets = vec![None];
    let mut candidate_builder = AtomicEffectBuilder::default();
    let accepted = {
        let mut output = EngineOutput::atomic(&mut candidate_builder);
        zbus::block_on(candidate.accept_completion(&mut output, false))
            .expect("active candidate acceptance")
    };
    assert!(accepted);
    assert_recorded_effect_counts(&candidate_builder.finish(true), 0, 1);
    assert_eq!(candidate.committed_tail.buffer, "проверка");
    assert_eq!(
        current_word(&candidate.committed_tail.autocorrect_suppression).open_token_chars,
        "проверка".chars().count()
    );
}

#[test]
fn td120_accepted_completion_edit_boundary_publishes_feedback_once_with_attribution() {
    assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
    let mut candidate = isolated_engine("/td120/accepted-feedback", true);
    type_tail(&mut candidate, "контекст ");
    for ch in "пров".chars() {
        candidate.insert_composition_char(ch);
    }
    candidate.composition.preedit_suffix = "ерка".to_string();
    candidate.composition.preedit_candidates = vec!["ерка".to_string()];
    candidate.composition.preedit_replacement_targets = vec![None];

    let mut accept_builder = AtomicEffectBuilder::default();
    let accepted = {
        let mut output = EngineOutput::atomic(&mut accept_builder);
        zbus::block_on(candidate.accept_completion(&mut output, true))
            .expect("active candidate acceptance with boundary")
    };
    assert!(accepted);
    assert_recorded_effect_counts(&accept_builder.finish(true), 0, 1);
    assert_eq!(candidate.committed_tail.buffer, "контекст проверка ");
    assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());

    let mut backspace_builder = AtomicEffectBuilder::default();
    let backspace_handled = {
        let mut output = EngineOutput::atomic(&mut backspace_builder);
        zbus::block_on(candidate.process_pressed_key(&mut output, KEY_BACKSPACE, 0, 0))
            .expect("completion edit Backspace")
    };
    assert!(!backspace_handled);
    assert_eq!(candidate.committed_tail.buffer, "контекст проверка");
    assert!(candidate
        .committed_tail
        .pending_completion_learning
        .as_ref()
        .is_some_and(|pending| pending.editing));
    candidate.rebuild_preedit_fast_from_tail();
    assert!(candidate.arm_current_word_autocorrect_suppression());
    current_word(&candidate.committed_tail.autocorrect_suppression);

    let mut boundary_builder = AtomicEffectBuilder::default();
    let boundary_handled = {
        let mut output = EngineOutput::atomic(&mut boundary_builder);
        zbus::block_on(candidate.process_pressed_key(&mut output, '!' as u32, 0, 0))
            .expect("completion edit hard boundary")
    };
    assert!(boundary_handled);
    assert_eq!(candidate.committed_tail.buffer, "контекст проверка!");
    assert!(candidate.committed_tail.autocorrect_suppression.is_none());
    assert_eq!(
        crate::tail_memory::take_accepted_completion_feedback(),
        [("контекст".to_string(), "проверка".to_string())]
    );
    assert!(candidate
        .committed_tail
        .pending_completion_learning
        .is_none());

    let mut duplicate_boundary_builder = AtomicEffectBuilder::default();
    let duplicate_boundary_handled = {
        let mut output = EngineOutput::atomic(&mut duplicate_boundary_builder);
        zbus::block_on(candidate.process_pressed_key(&mut output, '?' as u32, 0, 0))
            .expect("second hard boundary")
    };
    assert!(duplicate_boundary_handled);
    assert!(crate::tail_memory::take_accepted_completion_feedback().is_empty());
}

#[test]
fn td120_committed_candidate_undo_and_bridge_open_results_arm_current_word() {
    let mut candidate = isolated_engine("/td120/committed-candidate-open", true);
    type_tail(&mut candidate, "пров");
    let (handled, proposal) = recorded_replace(
        &mut candidate,
        CommittedTailReplaceRequest::ime_candidate_accept(4, "проверка".to_string()),
    );
    assert!(handled);
    assert_recorded_effect_counts(&proposal, 0, 1);
    current_word(&candidate.committed_tail.autocorrect_suppression);

    let mut undo = isolated_engine("/td120/undo-open", true);
    type_tail(&mut undo, "проверка ");
    let (handled, proposal) = recorded_replace(
        &mut undo,
        CommittedTailReplaceRequest::ime_auto_undo(9, "проверрка".to_string()),
    );
    assert!(handled);
    assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
    current_word(&undo.committed_tail.autocorrect_suppression);

    let mut bridge = isolated_engine("/td120/bridge-open", false);
    type_tail(&mut bridge, "abc");
    let (handled, proposal) = recorded_replace(
        &mut bridge,
        CommittedTailReplaceRequest::daemon_bridge(1, "x".to_string(), true),
    );
    assert!(handled);
    assert_recorded_effect_counts(&proposal, 1, 1);
    assert_eq!(bridge.committed_tail.buffer, "abx");
    current_word(&bridge.committed_tail.autocorrect_suppression);
}

#[test]
fn td120_bridge_replacement_after_hard_punctuation_tracks_only_open_token() {
    let mut engine = isolated_engine("/td120/bridge-hard-punctuation", false);
    type_tail(&mut engine, "abc!q");
    let (handled, proposal) = recorded_replace(
        &mut engine,
        CommittedTailReplaceRequest::daemon_bridge(1, "x".to_string(), true),
    );

    assert!(handled);
    assert_recorded_effect_counts(&proposal, 1, 1);
    assert_eq!(engine.committed_tail.buffer, "abc!x");
    assert_eq!(
        current_word(&engine.committed_tail.autocorrect_suppression).open_token_chars,
        1
    );

    erase_tail_chars(&mut engine, 1);
    assert_eq!(engine.committed_tail.buffer, "abc!");
    assert!(engine.committed_tail.autocorrect_suppression.is_none());
    type_tail(&mut engine, "y");
    assert_eq!(engine.committed_tail.buffer, "abc!y");
    assert!(!engine.take_manual_toggle_autocorrect_suppression());
}

#[test]
fn td120_failed_and_noop_replacements_preserve_prior_guard_without_rearming() {
    let mut engine = isolated_engine("/td120/failure-preserves", false);
    type_tail(&mut engine, "abc");
    let (handled, _) = recorded_replace(
        &mut engine,
        CommittedTailReplaceRequest::daemon_bridge(1, "x".to_string(), true),
    );
    assert!(handled);
    let before = current_word(&engine.committed_tail.autocorrect_suppression);

    let stale = VisibleTailSnapshot::new(
        VisibleTailSource::ImeCommittedTail,
        "foreign".to_string(),
        Some(engine.path.clone()),
        engine.committed_tail.epoch,
    );
    let (handled, proposal) = recorded_replace(
        &mut engine,
        CommittedTailReplaceRequest::ime_manual_toggle(2, "фис".to_string(), true)
            .with_expected_tail(stale),
    );
    assert!(!handled);
    assert_eq!(proposal.0, PROPOSAL_NATIVE_UNHANDLED);
    assert_eq!(
        current_word(&engine.committed_tail.autocorrect_suppression).incarnation,
        before.incarnation
    );

    engine.composition.preedit_visible = true;
    let mut failed_builder = AtomicEffectBuilder::default();
    failed_builder.fail_preedit_publication();
    let failed = {
        let mut output = EngineOutput::atomic(&mut failed_builder);
        zbus::block_on(engine.replace_committed_tail(
            &mut output,
            CommittedTailReplaceRequest::ime_manual_toggle(2, "фис".to_string(), true),
        ))
    };
    assert!(failed.is_err());
    assert_eq!(
        current_word(&engine.committed_tail.autocorrect_suppression).incarnation,
        before.incarnation
    );

    let (handled, proposal) = recorded_replace(
        &mut engine,
        CommittedTailReplaceRequest::ime_manual_toggle(0, String::new(), true),
    );
    assert!(!handled);
    assert_eq!(proposal.0, PROPOSAL_NATIVE_UNHANDLED);
    assert_eq!(
        current_word(&engine.committed_tail.autocorrect_suppression).incarnation,
        before.incarnation
    );
}

#[test]
fn td120_composition_cursor_full_erase_retires_current_word() {
    let mut engine = isolated_engine("/td120/composition-full-erase", false);
    for ch in US_TO_RU.source.chars() {
        engine.insert_composition_char(ch);
    }
    let mut builder = AtomicEffectBuilder::default();
    {
        let mut output = EngineOutput::atomic(&mut builder);
        zbus::block_on(engine.manual_toggle_active_text_target(&mut output))
            .expect("active manual edit");
    }
    current_word(&engine.committed_tail.autocorrect_suppression);
    engine.composition.buffer = US_TO_RU.replacement.to_string();
    engine.composition.cursor = engine.composition.buffer.chars().count();

    for _ in 0..US_TO_RU.replacement.chars().count() {
        let mut backspace_builder = AtomicEffectBuilder::default();
        let handled = {
            let mut output = EngineOutput::atomic(&mut backspace_builder);
            zbus::block_on(engine.backspace(&mut output)).expect("composition backspace")
        };
        assert!(handled);
    }
    assert!(engine.composition.buffer.is_empty());
    assert!(engine.committed_tail.autocorrect_suppression.is_none());
}

#[test]
fn td120_partial_composition_edit_preserves_current_word_incarnation() {
    let mut engine = isolated_engine("/td120/composition-partial", false);
    type_tail(&mut engine, "abc");
    assert!(engine.arm_current_word_autocorrect_suppression());
    let before = current_word(&engine.committed_tail.autocorrect_suppression);
    engine.composition.buffer = "abc".to_string();
    engine.composition.cursor = 2;
    engine.insert_composition_char('x');

    let after = current_word(&engine.committed_tail.autocorrect_suppression);
    assert_eq!(after.incarnation, before.incarnation);
    assert_eq!(after.open_token_chars, 4);
}

#[test]
fn td120_backspace_after_tail_trim_preserves_until_the_tracked_token_is_empty() {
    let mut engine = isolated_engine("/td120/trimmed-token", false);
    type_tail(&mut engine, &"a".repeat(200));
    assert!(engine.arm_current_word_autocorrect_suppression());
    let before = current_word(&engine.committed_tail.autocorrect_suppression);
    assert_eq!(
        before.open_token_chars,
        engine.committed_tail.buffer.chars().count()
    );

    engine.push_tail_char('b');
    let after_trimmed_append = current_word(&engine.committed_tail.autocorrect_suppression);
    assert_eq!(after_trimmed_append.incarnation, before.incarnation);
    assert_eq!(
        after_trimmed_append.open_token_chars,
        engine.committed_tail.buffer.chars().count()
    );

    erase_tail_chars(&mut engine, after_trimmed_append.open_token_chars / 2);
    let partial = current_word(&engine.committed_tail.autocorrect_suppression);
    assert_eq!(partial.incarnation, before.incarnation);
    assert_eq!(
        partial.open_token_chars,
        after_trimmed_append.open_token_chars / 2
    );

    erase_tail_chars(&mut engine, partial.open_token_chars);
    assert!(engine.committed_tail.buffer.is_empty());
    assert!(engine.committed_tail.autocorrect_suppression.is_none());
}

#[test]
fn td120_uncapped_length_uses_the_same_capped_layout_symbol_classification() {
    let mut engine = isolated_engine("/td120/capped-layout-classification", false);
    type_tail(&mut engine, &format!("я{}[", "a".repeat(40)));
    assert!(engine.composition.preedit_fast.has_open_token());
    assert!(engine.arm_current_word_autocorrect_suppression());
    assert_eq!(
        current_word(&engine.committed_tail.autocorrect_suppression).open_token_chars,
        42
    );
}

#[test]
fn td120_space_enter_and_hard_boundary_retire_current_word() {
    for (path, boundary) in [
        ("/td120/space-boundary", ' '),
        ("/td120/hard-boundary", '!'),
    ] {
        let mut engine = isolated_engine(path, false);
        type_tail(&mut engine, "abc");
        assert!(engine.arm_current_word_autocorrect_suppression());
        engine.push_tail_char(boundary);
        assert!(engine.committed_tail.autocorrect_suppression.is_none());
    }

    let mut enter = isolated_engine("/td120/enter-boundary", false);
    type_tail(&mut enter, "abc");
    assert!(enter.arm_current_word_autocorrect_suppression());
    let mut builder = AtomicEffectBuilder::default();
    let handled = {
        let mut output = EngineOutput::atomic(&mut builder);
        zbus::block_on(enter.process_pressed_key(&mut output, KEY_ENTER, 0, 0))
            .expect("committed Enter")
    };
    assert!(!handled);
    assert!(enter.committed_tail.autocorrect_suppression.is_none());
}

#[test]
fn td120_active_composition_enter_retires_guard_after_one_commit() {
    let mut engine = isolated_engine("/td120/active-enter", false);
    for ch in "abc".chars() {
        engine.insert_composition_char(ch);
    }
    assert!(engine.arm_current_word_autocorrect_suppression());

    let mut builder = AtomicEffectBuilder::default();
    let handled = {
        let mut output = EngineOutput::atomic(&mut builder);
        zbus::block_on(engine.process_pressed_key(&mut output, KEY_ENTER, 0, 0))
            .expect("active composition Enter")
    };
    let proposal = builder.finish(handled);
    assert!(!handled);
    assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
    assert_eq!(proposal.1.iter().filter(|(tag, _)| *tag == 1).count(), 1);
    assert!(engine.composition.buffer.is_empty());
    assert!(engine.committed_tail.autocorrect_suppression.is_none());
    assert!(engine.atomic.deferred_learning_actions.is_empty());
}

#[test]
fn td120_terminal_passthrough_boundary_retires_guard_without_ime_commit() {
    let mut engine = isolated_engine("/td120/terminal-boundary", false);
    engine.set_client_capabilities(0);
    engine.client_context.cursor_cell_width = 1;
    type_tail(&mut engine, "abc");
    assert!(engine.arm_current_word_autocorrect_suppression());

    let mut builder = AtomicEffectBuilder::default();
    let handled = {
        let mut output = EngineOutput::atomic(&mut builder);
        zbus::block_on(engine.process_pressed_key(&mut output, '!' as u32, 0, 0))
            .expect("terminal passthrough punctuation")
    };
    let proposal = builder.finish(handled);
    assert!(!handled);
    assert!(proposal.1.iter().all(|(tag, _)| *tag != 1));
    assert_eq!(engine.committed_tail.buffer, "abc!");
    assert!(engine.committed_tail.autocorrect_suppression.is_none());
    assert!(engine.atomic.deferred_learning_actions.is_empty());
}

#[test]
fn td120_existing_handoff_rebinds_same_incarnation_and_rejects_old_owner() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut source = unbound_engine("/td120/handoff/source", shared.clone(), false);
    assert!(source.bind_focus_path());
    type_tail(&mut source, "abc");
    source.publish_active_path_preserve_handoff(
        Instant::now() + std::time::Duration::from_millis(700),
    );
    assert!(source.arm_current_word_autocorrect_suppression());
    let source_guard = current_word(&source.committed_tail.autocorrect_suppression);

    let mut target = unbound_engine("/td120/handoff/target", shared.clone(), true);
    assert!(target.bind_focus_path());
    let target_guard = current_word(&target.committed_tail.autocorrect_suppression);
    assert_eq!(target_guard.incarnation, source_guard.incarnation);
    assert_ne!(
        target_guard.owner_lease_identity,
        source_guard.owner_lease_identity
    );
    assert!(!source.take_manual_toggle_autocorrect_suppression());
    source.clear_autocorrect_suppression_handoff();
    assert!(shared
        .lock()
        .expect("shared handoff")
        .autocorrect_suppression
        .is_some());
    assert!(target.take_manual_toggle_autocorrect_suppression());
    assert!(shared
        .lock()
        .expect("shared handoff")
        .autocorrect_suppression
        .is_none());
}

#[test]
fn td120_legacy_v1_is_typed_and_retains_compatibility_residual() {
    let mut engine = isolated_engine("/td120/legacy-v1", false);
    assert!(engine.arm_legacy_replay_autocorrect_suppression());
    assert!(matches!(
        engine.committed_tail.autocorrect_suppression.as_ref(),
        Some(AutocorrectSuppression::LegacyReplayV1)
    ));
    engine.push_tail_char(' ');
    assert!(matches!(
        engine.committed_tail.autocorrect_suppression.as_ref(),
        Some(AutocorrectSuppression::LegacyReplayV1)
    ));
    assert!(engine.take_manual_toggle_autocorrect_suppression());
    assert!(!engine.take_manual_toggle_autocorrect_suppression());
}

#[test]
fn td120_exact_v2_admission_revoke_and_expiry_keep_transport_scope() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut source = unbound_engine("/td120/exact-v2/source", shared.clone(), false);
    assert!(source.bind_focus_path());
    type_tail(&mut source, "abc");
    source.prepare_exact_manual_toggle_layout_handoff();
    let epoch = source.committed_tail.epoch;

    let mut target = unbound_engine("/td120/exact-v2/target", shared.clone(), true);
    assert!(target.bind_focus_path());
    target.reset_for_ibus_soft_reset();
    target.client_context.surrounding_text_snapshot =
        Some(SurroundingTextSnapshot::new("abc".to_string(), 3, 3));
    assert!(!target.arm_exact_manual_toggle_autocorrect_suppression(
        "bc",
        epoch,
        "/td120/exact-v2/target",
        true,
    ));
    assert!(target.arm_exact_manual_toggle_autocorrect_suppression(
        "abc",
        epoch,
        "/td120/exact-v2/target",
        true,
    ));
    assert!(!target.revoke_exact_manual_toggle_autocorrect_suppression(
        epoch.wrapping_add(1),
        "/td120/exact-v2/target",
    ));

    let expired = ExactManualToggleSuppression {
        path: "/td120/exact-v2/target".to_string(),
        epoch,
        expires_at: Instant::now() - std::time::Duration::from_millis(1),
        owner_lease_identity: target.client_context.runtime_owner_lease_identity,
        target_layout_is_ru: target.layout_gesture.layout_is_ru,
        original_tail: "abc".to_string(),
        original_suffix: "abc".to_string(),
        unchanged_prefix: String::new(),
        replacement: "фис".to_string(),
    };
    target.committed_tail.autocorrect_suppression =
        Some(AutocorrectSuppression::ExactReplay(expired.clone()));
    {
        let mut state = shared.lock().expect("shared exact suppression");
        state.autocorrect_suppression = Some(AutocorrectSuppression::ExactReplay(expired));
        state.suppression_revision = state.suppression_revision.wrapping_add(1);
    }
    assert!(!target.take_manual_toggle_autocorrect_suppression());
    assert!(target.committed_tail.autocorrect_suppression.is_none());
    assert!(shared
        .lock()
        .expect("shared exact suppression")
        .autocorrect_suppression
        .is_none());
}

#[test]
fn td120_public_atomic_and_physical_double_shift_contracts_are_unchanged() {
    let envelope: crate::atomic::AtomicEnvelope = (1, 2, 3, 4, 5, 6, vec![0; 32]);
    let receipt: crate::atomic::AtomicPriorReceipt = (0, 0, Vec::new());
    assert_eq!(envelope.0, 1);
    assert_eq!(receipt.0, 0);

    let interface = include_str!("ibus_interface.rs");
    let legacy_observer = interface
        .split("fn observe_daemon_owned_legacy_shift")
        .nth(1)
        .expect("legacy physical Shift observer")
        .split("async fn process_atomic_shift_gesture")
        .next()
        .expect("exclusive atomic Shift route");
    assert!(!legacy_observer.contains("manual_toggle_active_text_target"));
    assert!(!legacy_observer.contains("replace_committed_tail"));
    assert!(!legacy_observer.contains("EngineOutput"));
}
