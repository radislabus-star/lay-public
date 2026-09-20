use std::sync::{Arc, Mutex};

use lay::config::LayConfig;

use crate::engine::LayIbusEngine;
use crate::output::{AtomicEffectBuilder, EngineOutput, TestEngineOutput, PROPOSAL_FRAME_READY};
use crate::protocol::{KEY_BACKSPACE, KEY_ENTER, KEY_LEFT, KEY_SPACE, KEY_TAB};
use crate::space_autocorrect_prefetch::proof::install_exact_lease;
use crate::window_interaction::{
    IBUS_CAP_LAY_COMMIT_ONLY_PREEDIT, IBUS_CAP_PREEDIT_TEXT, IBUS_CAP_SURROUNDING_TEXT,
    IBUS_INPUT_PURPOSE_TERMINAL,
};

const IBUS_CAP_FOCUS: u32 = 1 << 3;

fn legacy_engine(path: &str, caps: u32) -> LayIbusEngine {
    let mut engine = LayIbusEngine::new(
        path.to_string(),
        Arc::new(Mutex::new(Default::default())),
        false,
        true,
        LayConfig {
            auto_replace: true,
            auto_switch_layout: true,
            text_backend: "ime".to_string(),
            ..LayConfig::default()
        },
    );
    assert!(engine.bind_focus_path());
    engine.set_client_capabilities(caps);
    engine
}

fn press(engine: &mut LayIbusEngine, output: &mut TestEngineOutput, keyval: u32) -> bool {
    zbus::block_on(engine.process_pressed_key(&mut EngineOutput::test(output), keyval, 0, 0))
        .expect("managed key route")
}

fn shared_tail(engine: &LayIbusEngine) -> String {
    engine
        .shared
        .lock()
        .expect("shared tail state")
        .handoff_tail_buffer
        .clone()
}

#[test]
fn td125_legacy_preedit_word_commits_ready_space_correction_without_delete() {
    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("warm exact-layout authority");
    let mut engine = legacy_engine(
        "/td125/legacy-preedit/corrected",
        IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS,
    );
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };

    for ch in "ghbdtn".chars() {
        assert!(press(&mut engine, &mut output, ch as u32));
    }

    assert!(
        output.committed_texts.is_empty(),
        "the application must not own the unfinished token: {:?}",
        output.committed_texts
    );
    assert_eq!(engine.composition.buffer, "ghbdtn");
    assert_eq!(
        output
            .preedit_updates
            .last()
            .map(|update| update.0.as_str()),
        Some("ghbdtn")
    );

    let identity = engine
        .capture_input_frame_identity()
        .expect("active preedit identity");
    install_exact_lease(&identity, &engine.config);

    assert!(press(&mut engine, &mut output, KEY_SPACE));
    assert_eq!(output.committed_texts, ["привет "]);
    assert!(output.surrounding_deletes.is_empty());
    assert!(!output.effects.contains(&"forward-key"));
    assert_eq!(engine.committed_tail.buffer, "привет ");
    assert_eq!(shared_tail(&engine), "привет ");
    assert!(engine.composition.buffer.is_empty());
}

#[test]
fn td125_without_preedit_capability_keeps_per_character_managed_commits() {
    let mut engine = legacy_engine("/td125/legacy-preedit/no-cap", IBUS_CAP_FOCUS);
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };

    for ch in "ab".chars() {
        assert!(press(&mut engine, &mut output, ch as u32));
    }

    assert_eq!(output.committed_texts, ["a", "b"]);
    assert!(output.preedit_updates.is_empty());
    assert!(engine.composition.buffer.is_empty());
    assert!(!engine.composition.legacy_word_preedit_active);
}

#[test]
fn td125_surrounding_text_capability_does_not_prove_applied_delete_and_uses_preedit() {
    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("warm exact-layout authority");
    let mut unmarked = legacy_engine(
        "/td125/legacy-preedit/surrounding-unmarked",
        IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS | IBUS_CAP_SURROUNDING_TEXT,
    );
    let mut unmarked_output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    assert!(press(&mut unmarked, &mut unmarked_output, 'a' as u32));
    assert_eq!(unmarked_output.committed_texts, ["a"]);
    assert!(!unmarked.composition.legacy_word_preedit_active);

    let mut engine = legacy_engine(
        "/td125/legacy-preedit/surrounding",
        IBUS_CAP_PREEDIT_TEXT
            | IBUS_CAP_FOCUS
            | IBUS_CAP_SURROUNDING_TEXT
            | IBUS_CAP_LAY_COMMIT_ONLY_PREEDIT,
    );
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };

    for ch in "ghbdtn".chars() {
        assert!(press(&mut engine, &mut output, ch as u32));
    }

    assert!(output.committed_texts.is_empty());
    assert_eq!(engine.composition.buffer, "ghbdtn");
    assert!(engine.composition.legacy_word_preedit_active);

    let identity = engine
        .capture_input_frame_identity()
        .expect("caps41 active preedit identity");
    install_exact_lease(&identity, &engine.config);

    assert!(press(&mut engine, &mut output, KEY_SPACE));
    assert_eq!(output.committed_texts, ["привет "]);
    assert!(output.surrounding_deletes.is_empty());
    assert!(!output.effects.contains(&"forward-key"));
    assert_eq!(engine.committed_tail.buffer, "привет ");

    assert!(press(&mut engine, &mut output, 'a' as u32));
    assert_eq!(output.committed_texts, ["привет "]);
    assert_eq!(engine.composition.buffer, "a");
    assert!(engine.composition.legacy_word_preedit_active);
}

#[test]
fn td125_capability_gain_does_not_claim_an_already_committed_token_suffix() {
    let mut engine = legacy_engine("/td125/legacy-preedit/cap-gain", IBUS_CAP_FOCUS);
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };

    assert!(press(&mut engine, &mut output, 'a' as u32));
    engine.set_client_capabilities(IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS);
    assert!(press(&mut engine, &mut output, 'b' as u32));

    assert_eq!(output.committed_texts, ["a", "b"]);
    assert!(engine.composition.buffer.is_empty());
    assert!(!engine.composition.legacy_word_preedit_active);
}

#[test]
fn td125_surrounding_capability_gain_finishes_owned_preedit_before_managed_commits() {
    let mut engine = legacy_engine(
        "/td125/legacy-preedit/surrounding-gain",
        IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS,
    );
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };

    assert!(press(&mut engine, &mut output, 'a' as u32));
    assert_eq!(engine.composition.buffer, "a");
    assert!(engine.composition.legacy_word_preedit_active);
    assert!(output.committed_texts.is_empty());

    engine.set_client_capabilities(
        IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS | IBUS_CAP_SURROUNDING_TEXT,
    );
    assert_eq!(engine.composition.buffer, "a");
    assert!(engine.composition.legacy_word_preedit_active);

    assert!(press(&mut engine, &mut output, 'b' as u32));
    assert!(press(&mut engine, &mut output, KEY_SPACE));
    assert_eq!(output.committed_texts, ["ab "]);
    assert!(engine.composition.buffer.is_empty());
    assert!(!engine.composition.legacy_word_preedit_active);

    assert!(press(&mut engine, &mut output, 'c' as u32));
    assert_eq!(output.committed_texts, ["ab ", "c"]);
    assert!(engine.composition.buffer.is_empty());
    assert!(!engine.composition.legacy_word_preedit_active);
}

#[test]
fn td125_capability_loss_discards_only_the_uncommitted_preedit_word() {
    let mut engine = legacy_engine(
        "/td125/legacy-preedit/cap-loss",
        IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS,
    );
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    for ch in "ab".chars() {
        assert!(press(&mut engine, &mut output, ch as u32));
    }

    engine.set_client_capabilities(IBUS_CAP_FOCUS);

    assert!(engine.composition.buffer.is_empty());
    assert!(!engine.composition.legacy_word_preedit_active);
    assert!(engine.committed_tail.buffer.is_empty());
    assert!(shared_tail(&engine).is_empty());
    assert!(press(&mut engine, &mut output, 'c' as u32));
    assert_eq!(output.committed_texts, ["c"]);
}

#[test]
fn td125_atomic_output_does_not_enter_legacy_preedit_ownership() {
    let mut engine = legacy_engine(
        "/td125/legacy-preedit/atomic",
        IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS,
    );
    engine.atomic.active = true;
    let mut builder = AtomicEffectBuilder::default();

    assert!(zbus::block_on(engine.process_pressed_key(
        &mut EngineOutput::atomic(&mut builder),
        'a' as u32,
        0,
        0,
    ))
    .expect("atomic key route"));

    assert!(engine.composition.buffer.is_empty());
    assert!(!engine.composition.legacy_word_preedit_active);
    let proposal = builder.finish(true);
    assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
}

#[test]
fn td125_terminal_purpose_keeps_native_passthrough() {
    let mut engine = legacy_engine(
        "/td125/legacy-preedit/terminal",
        IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS,
    );
    engine.set_content_type_state(IBUS_INPUT_PURPOSE_TERMINAL, 0);
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };

    assert!(!press(&mut engine, &mut output, 'a' as u32));
    assert!(output.committed_texts.is_empty());
    assert!(engine.composition.buffer.is_empty());
    assert!(!engine.composition.legacy_word_preedit_active);
}

#[test]
fn td125_not_ready_space_commits_original_preedit_once() {
    let mut engine = legacy_engine(
        "/td125/legacy-preedit/not-ready",
        IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS,
    );
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    for ch in "qxzv".chars() {
        assert!(press(&mut engine, &mut output, ch as u32));
    }
    engine.invalidate_space_autocorrect_path();

    assert!(press(&mut engine, &mut output, KEY_SPACE));
    assert_eq!(output.committed_texts, ["qxzv "]);
    assert!(output.surrounding_deletes.is_empty());
    assert_eq!(engine.committed_tail.buffer, "qxzv ");
    assert_eq!(shared_tail(&engine), "qxzv ");
    assert!(engine.composition.buffer.is_empty());
    assert!(!engine.composition.legacy_word_preedit_active);
}

#[test]
fn td125_punctuation_finishes_owned_word_in_one_commit() {
    let mut engine = legacy_engine(
        "/td125/legacy-preedit/punctuation",
        IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS,
    );
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    for ch in "ab".chars() {
        assert!(press(&mut engine, &mut output, ch as u32));
    }

    assert!(press(&mut engine, &mut output, '.' as u32));
    assert_eq!(output.committed_texts, ["ab."]);
    assert_eq!(engine.committed_tail.buffer, "ab.");
    assert_eq!(shared_tail(&engine), "ab.");
    assert!(engine.composition.buffer.is_empty());
    assert!(!engine.composition.legacy_word_preedit_active);
}

#[test]
fn td125_backspace_and_arrow_edit_only_the_owned_preedit() {
    let mut engine = legacy_engine(
        "/td125/legacy-preedit/edit",
        IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS,
    );
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    for ch in "ab".chars() {
        assert!(press(&mut engine, &mut output, ch as u32));
    }

    assert!(press(&mut engine, &mut output, KEY_BACKSPACE));
    assert_eq!(engine.composition.buffer, "a");
    assert_eq!(engine.composition.cursor, 1);
    assert!(press(&mut engine, &mut output, KEY_LEFT));
    assert_eq!(engine.composition.cursor, 0);
    assert!(output.committed_texts.is_empty());
    assert!(engine.composition.legacy_word_preedit_active);
}

#[test]
fn td125_enter_commits_owned_preedit_and_passes_enter_through() {
    let mut engine = legacy_engine(
        "/td125/legacy-preedit/enter",
        IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS,
    );
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    for ch in "ab".chars() {
        assert!(press(&mut engine, &mut output, ch as u32));
    }

    assert!(!press(&mut engine, &mut output, KEY_ENTER));
    assert_eq!(output.committed_texts, ["ab"]);
    assert_eq!(engine.committed_tail.buffer, "ab");
    assert_eq!(shared_tail(&engine), "ab");
    assert!(engine.composition.buffer.is_empty());
    assert!(!engine.composition.legacy_word_preedit_active);
}

#[test]
fn td125_tab_accepts_existing_candidate_and_retires_preedit_ownership() {
    let mut engine = legacy_engine(
        "/td125/legacy-preedit/tab",
        IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS,
    );
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    for ch in "hel".chars() {
        assert!(press(&mut engine, &mut output, ch as u32));
    }
    engine.composition.preedit_candidates = vec!["lo".to_string()];
    engine.composition.preedit_replacement_targets = vec![None];
    engine.composition.preedit_candidate_index = 0;
    engine.composition.preedit_display_only_pending = false;

    assert!(press(&mut engine, &mut output, KEY_TAB));

    assert_eq!(output.committed_texts, ["hello "]);
    assert!(engine.composition.buffer.is_empty());
    assert!(!engine.composition.legacy_word_preedit_active);
}

#[test]
fn td125_soft_reset_discards_owned_preedit_from_local_and_shared_authority() {
    let mut engine = legacy_engine(
        "/td125/legacy-preedit/reset",
        IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS,
    );
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    for ch in "ab".chars() {
        assert!(press(&mut engine, &mut output, ch as u32));
    }
    assert!(engine.composition.legacy_word_preedit_active);
    assert_eq!(engine.committed_tail.buffer, "ab");
    assert_eq!(shared_tail(&engine), "ab");

    engine.reset_for_ibus_soft_reset();

    assert!(engine.composition.buffer.is_empty());
    assert!(!engine.composition.legacy_word_preedit_active);
    assert!(engine.committed_tail.buffer.is_empty());
    assert!(shared_tail(&engine).is_empty());
    engine.set_client_capabilities(IBUS_CAP_FOCUS);
    assert!(press(&mut engine, &mut output, 'c' as u32));
    assert!(press(&mut engine, &mut output, KEY_SPACE));
    assert_eq!(output.committed_texts, ["c", " "]);
    assert!(output.surrounding_deletes.is_empty());
    assert_eq!(engine.committed_tail.buffer, "c ");
    assert_eq!(shared_tail(&engine), "c ");
}

#[test]
fn td125_focus_reset_cannot_transfer_owned_preedit_as_committed_tail() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut focus_engine = LayIbusEngine::new(
        "/td125/legacy-preedit/focus-reset".to_string(),
        shared.clone(),
        false,
        true,
        LayConfig {
            auto_replace: true,
            auto_switch_layout: true,
            text_backend: "ime".to_string(),
            ..LayConfig::default()
        },
    );
    assert!(focus_engine.bind_focus_path());
    focus_engine.set_client_capabilities(IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS);
    let mut focus_output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    for ch in "ab".chars() {
        assert!(press(&mut focus_engine, &mut focus_output, ch as u32));
    }
    assert!(focus_engine.should_preserve_focus_handoff());
    assert_eq!(shared_tail(&focus_engine), "ab");

    focus_engine.reset_for_ibus_focus_change();

    assert!(focus_engine.composition.buffer.is_empty());
    assert!(!focus_engine.composition.legacy_word_preedit_active);
    assert!(focus_engine.committed_tail.buffer.is_empty());
    assert!(shared_tail(&focus_engine).is_empty());

    let inherited = LayIbusEngine::new(
        "/td125/legacy-preedit/focus-target".to_string(),
        shared,
        false,
        true,
        LayConfig::default(),
    );
    assert!(inherited.committed_tail.buffer.is_empty());
}

#[test]
fn td125_cursor_zero_backspace_cancels_owned_preedit_and_tracks_native_delete() {
    let mut engine = legacy_engine("/td125/legacy-preedit/cursor-zero", IBUS_CAP_FOCUS);
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    assert!(press(&mut engine, &mut output, 'x' as u32));
    assert!(press(&mut engine, &mut output, KEY_SPACE));
    assert_eq!(engine.committed_tail.buffer, "x ");
    engine.set_client_capabilities(IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS);
    for ch in "ab".chars() {
        assert!(press(&mut engine, &mut output, ch as u32));
    }
    assert_eq!(engine.committed_tail.buffer, "x ab");
    assert!(press(&mut engine, &mut output, KEY_LEFT));
    assert!(press(&mut engine, &mut output, KEY_LEFT));
    assert_eq!(engine.composition.cursor, 0);

    assert!(!press(&mut engine, &mut output, KEY_BACKSPACE));

    assert!(engine.composition.buffer.is_empty());
    assert!(!engine.composition.legacy_word_preedit_active);
    assert_eq!(engine.committed_tail.buffer, "x");
    assert_eq!(shared_tail(&engine), "x");
    assert!(output.surrounding_deletes.is_empty());
}

#[test]
fn td125_disabling_live_composition_clears_client_preedit_before_state_reset() {
    let mut engine = legacy_engine(
        "/td125/legacy-preedit/disabled",
        IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS,
    );
    let mut output = TestEngineOutput {
        legacy_transport: true,
        ..Default::default()
    };
    assert!(press(&mut engine, &mut output, 'a' as u32));
    assert!(engine.composition.preedit_visible);
    let updates_before = output.preedit_updates.len();
    engine.config.text_backend = "uinput".to_string();

    let handled = zbus::block_on(engine.process_key_event_with_output(
        &mut EngineOutput::test(&mut output),
        'z' as u32,
        0,
        0,
    ))
    .expect("disabled-composition route");

    assert!(!handled);
    assert_eq!(output.preedit_updates.len(), updates_before + 1);
    assert_eq!(
        output
            .preedit_updates
            .last()
            .map(|update| update.0.as_str()),
        Some("")
    );
    assert!(engine.composition.buffer.is_empty());
    assert!(engine.committed_tail.buffer.is_empty());
    assert!(shared_tail(&engine).is_empty());
}
