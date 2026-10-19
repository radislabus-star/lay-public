//! IBus blur policy belongs to the current typed composition, never a hint.
use super::*;
use crate::output::{EngineOutput, TestEngineOutput};

#[test]
fn td159_owned_preedit_blur_preserves_typed_word_without_accepting_hint() {
    zbus::block_on(bounded(async {
        for unknown_start in [false, true] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let (mut engine, _) =
                td121_owned_legacy_word_with_closing_space_reset(&mut harness, 41, unknown_start)
                    .await;
            let typed = engine.composition.buffer.clone();
            let tail = engine.committed_tail.buffer.clone();
            engine.composition.preedit_candidates = vec!["ка".into()];
            engine.composition.preedit_replacement_targets = vec![None];
            let mut output = TestEngineOutput {
                legacy_transport: true,
                ..Default::default()
            };
            engine
                .publish_selected_precognition_candidate(&mut EngineOutput::test(&mut output))
                .await
                .unwrap();
            assert_eq!(
                output.preedit_updates,
                [(typed.clone(), typed.chars().count() as u32, true, 1)],
                "IBus must preserve exactly the currently owned typed word on blur"
            );
            assert_eq!(output.auxiliary_updates, [("ка".to_owned(), true)]);
            assert_eq!(engine.composition.buffer, typed);
            assert_eq!(engine.committed_tail.buffer, tail);
            assert_eq!(engine.context_word_is_known(), !unknown_start);
            assert!(output.committed_texts.is_empty());
            assert!(output.surrounding_deletes.is_empty());
        }
    }));
}

#[test]
fn td159_owned_preedit_blur_retains_clear_for_nonowners_and_atomic() {
    zbus::block_on(bounded(async {
        for refusal in [
            "revoked",
            "sealed",
            "atomic",
            "unowned",
            "tail",
            "token",
            "sensitive",
        ] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let (mut engine, _) =
                td121_owned_legacy_word_with_closing_space_reset(&mut harness, 41, false).await;
            match refusal {
                "revoked" => assert!(harness
                    .adapter
                    .revoke_current_owner(engine.context_owner.as_ref().unwrap())),
                "sealed" => engine.context_handoff_sealed = true,
                "atomic" => engine.atomic.active = true,
                "unowned" => engine.composition.legacy_word_preedit_active = false,
                "tail" => engine.committed_tail.buffer = "another".into(),
                "token" => engine.context_token = None,
                "sensitive" => engine.set_content_type_state(8, 0),
                _ => unreachable!(),
            }
            let mut output = TestEngineOutput {
                legacy_transport: refusal != "atomic",
                ..Default::default()
            };
            engine
                .publish_selected_precognition_candidate(&mut EngineOutput::test(&mut output))
                .await
                .unwrap();
            assert!(
                !output.preedit_updates.is_empty(),
                "{refusal}: exercise publication"
            );
            assert!(
                output
                    .preedit_updates
                    .iter()
                    .all(|(_, _, _, mode)| *mode == 0),
                "{refusal}"
            );
            assert!(output.committed_texts.is_empty(), "{refusal}");
            assert!(output.surrounding_deletes.is_empty(), "{refusal}");
        }
    }));
}

#[test]
fn td159_owned_preedit_blur_explicit_clear_cannot_commit_or_accept_completion() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let (mut engine, _) =
            td121_owned_legacy_word_with_closing_space_reset(&mut harness, 41, false).await;
        let mut output = TestEngineOutput {
            legacy_transport: true,
            ..Default::default()
        };
        engine
            .publish_selected_precognition_candidate(&mut EngineOutput::test(&mut output))
            .await
            .unwrap();
        assert_eq!(output.preedit_updates.last().unwrap().3, 1);
        output.preedit_updates.clear();
        engine
            .clear_preedit(&mut EngineOutput::test(&mut output))
            .await
            .unwrap();
        assert_eq!(output.preedit_updates, [(String::new(), 0, false, 0)]);
        assert!(output.committed_texts.is_empty());
        assert!(output.surrounding_deletes.is_empty());
        assert!(!engine.composition.preedit_visible);
    }));
}
