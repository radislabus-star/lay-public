//! Actual native callback transcript, not a replacement keyboard encoder.
use super::*;

async fn native_word(backend: &str, initial_ru: bool, word: &str) -> (Harness, LayIbusEngine) {
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = new_engine(&harness);
    engine.config.layout_backend = backend.into();
    engine.config.auto_replace = false;
    engine.config.nanda_precognition = false;
    start_source_free_unknown(&mut harness, &mut engine).await;
    engine.set_content_type_state(crate::engine::IBUS_INPUT_PURPOSE_TERMINAL, 0);
    engine.set_client_capabilities(1 | 1 << 3);
    engine.client_context.cursor_cell_width = 11;
    engine.set_layout_is_ru(initial_ru);
    assert!(engine.uses_native_terminal_input());
    let mut serial = 31_000;
    type_observed_native_text(&mut harness, &mut engine, word, &mut serial).await;
    assert_eq!(engine.committed_tail.buffer, word);
    assert!(engine.composition.buffer.is_empty());
    (harness, engine)
}

#[test]
fn kde_terminal_native_literal_restores_existing_manual_plan_direction() {
    zbus::block_on(bounded(async {
        for (initial_ru, word, expected_ru, target) in [
            (false, "привет", true, "ghbdtn"),
            (false, "привет ", true, "ghbdtn "),
            (true, "ghbdtn", false, "привет"),
            (true, "ghbdtn ", false, "привет "),
        ] {
            let (mut harness, engine) = native_word("kde", initial_ru, word).await;
            assert_eq!(engine.layout_gesture.layout_is_ru, expected_ru,
                "KDE native literal must select the actual observed script before planning: {word:?}");
            let plan =
                lay::manual_toggle::plan_manual_toggle(lay::manual_toggle::ManualToggleRequest {
                    visible_tail: lay::manual_toggle::VisibleTail::ime_committed_tail(
                        &engine.committed_tail.buffer,
                    ),
                    current_layout_is_ru: engine.layout_gesture.layout_is_ru,
                    preserve_trailing_whitespace: true,
                })
                .expect("existing planner must retain the observed opposite-layout target");
            assert_eq!(plan.replacement, target);
            assert_eq!(plan.target_layout_is_ru, !expected_ru);
            no_legacy_text_output(&mut harness).await;
        }
    }));
}

#[test]
fn kde_terminal_native_mode_keeps_other_clients_and_neutral_input_unchanged() {
    zbus::block_on(bounded(async {
        for (initial_ru, word) in [(false, "привет"), (true, "ghbdtn")] {
            let (mut harness, engine) = native_word("gnome", initial_ru, word).await;
            assert_eq!(engine.layout_gesture.layout_is_ru, initial_ru);
            no_legacy_text_output(&mut harness).await;
        }
        for backend in ["kde", "gnome"] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let mut engine = new_engine(&harness);
            engine.config.layout_backend = backend.into();
            engine.set_layout_is_ru(false);
            assert!(!engine.uses_native_terminal_input());
            engine.observe_kde_native_terminal_input_mode('п');
            assert!(!engine.layout_gesture.layout_is_ru);
            no_legacy_text_output(&mut harness).await;
        }
        for initial_ru in [false, true] {
            let (mut harness, engine) = native_word("kde", initial_ru, "123").await;
            assert_eq!(engine.layout_gesture.layout_is_ru, initial_ru);
            no_legacy_text_output(&mut harness).await;
        }
        let (mut harness, mut engine) = native_word("kde", false, "abc").await;
        engine.composition.buffer = "owned".into();
        engine.observe_kde_native_terminal_input_mode('п');
        assert!(!engine.layout_gesture.layout_is_ru);
        no_legacy_text_output(&mut harness).await;
    }));
}
