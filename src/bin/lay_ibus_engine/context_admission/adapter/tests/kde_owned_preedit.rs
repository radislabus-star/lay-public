//! KDE group is a postcondition of the existing owned-preedit operation.
use super::*;
use crate::layout_sync::{LayoutStackProbe, LAYOUT_STACK_PROBE};
use crate::output::{EngineOutput, TestEngineOutput};

fn install_probe(probe: LayoutStackProbe) {
    LAYOUT_STACK_PROBE.with(|slot| {
        assert!(slot.borrow().is_none());
        *slot.borrow_mut() = Some(probe);
    });
}

fn take_probe() -> LayoutStackProbe {
    LAYOUT_STACK_PROBE.with(|slot| slot.borrow_mut().take().unwrap())
}

#[test]
fn td159_kde_owned_preedit_updates_group_without_switching_engine() {
    zbus::block_on(bounded(async {
        for gestures in [1, 8] {
            let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
            let (mut engine, mut serial) =
                td121_owned_legacy_word_with_closing_space_reset(&mut harness, 41, false).await;
            engine.config.layout_backend = "kde".into();
            install_probe(LayoutStackProbe {
                active_engine: Some("lay-ime-us".into()),
                ..Default::default()
            });
            for gesture in 0..gestures {
                let target_is_ru = gesture % 2 == 0;
                let expected = if target_is_ru {
                    "привет"
                } else {
                    "ghbdtn"
                };
                let mut output = TestEngineOutput {
                    legacy_transport: true,
                    ..Default::default()
                };
                assert_eq!(
                    engine
                        .manual_toggle_active_text_target(&mut EngineOutput::test(&mut output))
                        .await
                        .unwrap(),
                    Some(target_is_ru)
                );
                assert_eq!(engine.composition.buffer, expected);
                assert_eq!(engine.committed_tail.buffer, format!("abc {expected}"));
                assert_eq!(engine.layout_gesture.layout_is_ru, target_is_ru);
                assert_eq!(output.input_mode_updates, [target_is_ru]);
                assert!(output.committed_texts.is_empty());
                assert!(output.surrounding_deletes.is_empty());
                assert!(engine.context_owns_active_preedit_manual_toggle());
            }
            let observed = take_probe();
            assert_eq!(
                observed.kde_activations,
                (0..gestures).map(|n| n % 2 == 0).collect::<Vec<_>>()
            );
            assert!(observed.activations.is_empty());
            assert!(observed.ibus_switches.is_empty());
            assert_eq!(observed.active_engine.as_deref(), Some("lay-ime-us"));
            serial += 1;
            assert!(legacy_key(&mut harness, &mut engine, serial, 'g' as u32, 34, 0).await);
            assert_eq!(
                engine.composition.buffer,
                if gestures == 1 {
                    "приветп"
                } else {
                    "ghbdtng"
                }
            );
        }
    }));
}

#[test]
fn td159_kde_owned_preedit_group_failure_restores_live_word_and_mode() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let (mut engine, _) =
            td121_owned_legacy_word_with_closing_space_reset(&mut harness, 41, false).await;
        engine.config.layout_backend = "kde".into();
        install_probe(LayoutStackProbe {
            active_engine: Some("lay-ime-us".into()),
            fail_kde_activation: true,
            ..Default::default()
        });
        let mut output = TestEngineOutput {
            legacy_transport: true,
            ..Default::default()
        };
        let result = engine
            .manual_toggle_active_text_target_with_disposition(&mut EngineOutput::test(&mut output))
            .await;
        let observed = take_probe();
        let error = result.expect_err("failed KDE group must not report a completed gesture");
        assert_eq!(
            error.receipt(),
            crate::window_interaction::ExecutionReceipt::LocalIndeterminatePartial
        );
        assert_eq!(engine.composition.buffer, "ghbdtn");
        assert_eq!(engine.committed_tail.buffer, "abc ghbdtn");
        assert!(!engine.layout_gesture.layout_is_ru);
        assert_eq!(output.input_mode_updates, [true, false]);
        assert_eq!(output.preedit_updates.last().unwrap().0, "ghbdtn");
        assert!(output.committed_texts.is_empty());
        assert!(output.surrounding_deletes.is_empty());
        assert_eq!(observed.kde_activations, [true]);
        assert!(observed.ibus_switches.is_empty());
    }));
}

#[test]
fn td159_kde_owned_preedit_owner_loss_does_not_rollback_into_new_context() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let (mut engine, _) =
            td121_owned_legacy_word_with_closing_space_reset(&mut harness, 41, false).await;
        engine.config.layout_backend = "kde".into();
        let admission = harness.adapter.clone();
        let owner = engine.context_owner.clone().unwrap();
        install_probe(LayoutStackProbe {
            active_engine: Some("lay-ime-us".into()),
            after_kde_activation: Some(Box::new(move || {
                assert!(admission.revoke_current_owner(&owner));
            })),
            ..Default::default()
        });
        let mut output = TestEngineOutput {
            legacy_transport: true,
            ..Default::default()
        };
        let result = engine
            .manual_toggle_active_text_target_with_disposition(&mut EngineOutput::test(&mut output))
            .await;
        let observed = take_probe();
        assert!(result.is_err());
        assert!(!engine.layout_gesture.layout_is_ru);
        assert_eq!(output.input_mode_updates, [true]);
        assert_eq!(output.preedit_updates.len(), 1);
        assert_eq!(output.preedit_updates[0].0, "привет");
        assert!(output.committed_texts.is_empty());
        assert!(output.surrounding_deletes.is_empty());
        assert_eq!(observed.kde_activations, [true]);
        assert!(observed.ibus_switches.is_empty());
    }));
}
