use zbus::fdo;

use lay::manual_toggle::{plan_manual_toggle, ManualToggleRequest};
use lay::text_edit::{
    authorize_backend_edit, plan_ime_manual_toggle_edit, plan_text_replacement, TextEditBackend,
    VisibleTail,
};

use super::engine::{LayIbusEngine, ManualToggleAuthority};
use super::output::EngineOutput;
use super::trace;

impl LayIbusEngine {
    pub(super) async fn manual_toggle_active_text_target(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
    ) -> fdo::Result<Option<bool>> {
        // PROTECTED USER CONTRACT: an immediate double Shift after autocorrect
        // restores the exact recorded input before any layout/manual toggle.
        // Keep this first; typing_transition_authority_contract enforces order.
        if self.defer_pending_ime_auto_undo_until_visible() {
            trace::record_auto_undo_retry("requested_exact_snapshot");
            if let Some(connection) = emitter.connection() {
                connection
                    .emit_signal(
                        None::<&str>,
                        self.path.as_str(),
                        "org.freedesktop.IBus.Engine",
                        "RequireSurroundingText",
                        &(),
                    )
                    .await
                    .map_err(|error| fdo::Error::Failed(error.to_string()))?;
            } else {
                trace::record_auto_undo_retry("atomic_waiting_exact_snapshot");
            }
            // The IME owns the pending rollback. Keep the daemon from replaying
            // a second route while SetSurroundingText confirms the exact tail.
            return Ok(Some(self.layout_gesture.layout_is_ru));
        }
        if let Some(target_layout_is_ru) = self.undo_last_ime_autocorrect(emitter).await? {
            return Ok(Some(target_layout_is_ru));
        }
        let authority = self.manual_toggle_authority();
        match authority {
            ManualToggleAuthority::ImeCommittedTail => {
                if self.terminal_committed_tail_executor_available() {
                    trace::record(
                        r#"{"kind":"ibus_manual_toggle_dispatch","source":"ime_committed_tail","executor":"terminal_erase_commit"}"#,
                    );
                    return self.toggle_committed_tail_target(emitter).await;
                }
                self.prepare_exact_manual_toggle_layout_handoff();
                trace::record(
                    r#"{"kind":"ibus_manual_toggle_delegation","source":"ime_committed_tail"}"#,
                );
                self.trace_key("double_shift_defer_exact_ime_tail", 0, 0, false, None);
                return Ok(None);
            }
            ManualToggleAuthority::DaemonWordBuffer => {
                self.defer_committed_tail_manual_toggle_to_daemon();
                trace::record(
                    r#"{"kind":"ibus_manual_toggle_delegation","source":"daemon_word_buffer"}"#,
                );
                self.trace_key("double_shift_defer_to_daemon", 0, 0, false, None);
                return Ok(None);
            }
            ManualToggleAuthority::ImeActiveComposition => {}
        }
        let Some(plan) = plan_manual_toggle(ManualToggleRequest {
            visible_tail: VisibleTail::ime_active_composition(&self.composition.buffer),
            current_layout_is_ru: self.layout_gesture.layout_is_ru,
            preserve_trailing_whitespace: false,
        }) else {
            return Ok(None);
        };
        trace::record_manual_toggle_plan(&plan);
        let original = self.composition.buffer.clone();
        let Some(replacement_plan) = plan_text_replacement(&original, &plan.replacement) else {
            return Ok(None);
        };
        let action = plan_ime_manual_toggle_edit(&original, &plan.replacement, replacement_plan);
        lay::action_log::record_candidate_edit_action_before_apply(
            &action,
            lay::action_log::MutationLogRoute::IME_ACTIVE_COMPOSITION,
            None,
        );
        let Some(authorized_edit) =
            authorize_backend_edit(TextEditBackend::Ime, action).into_authorized()
        else {
            trace::record(r#"{"kind":"ibus_manual_toggle_authorization_blocked"}"#);
            return Ok(None);
        };
        self.commit_verified_active_composition(emitter, authorized_edit)
            .await?;
        self.committed_tail.suppress_next_autocorrect = plan.suppress_next_autocorrect;
        self.committed_tail.exact_manual_toggle_suppression = None;
        self.sync_layout_after_manual_toggle(&plan.replacement);
        self.trace_key("double_shift_commit", 0, 0, true, None);
        Ok(Some(plan.target_layout_is_ru))
    }

    fn defer_committed_tail_manual_toggle_to_daemon(&mut self) {
        self.committed_tail.suppress_next_autocorrect = true;
        self.committed_tail.exact_manual_toggle_suppression = None;
    }
}

#[cfg(test)]
mod tests {
    use super::LayIbusEngine;
    use crate::output::{AtomicEffectBuilder, EngineOutput, PROPOSAL_FRAME_READY};
    use lay::config::LayConfig;
    use std::sync::{Arc, Mutex};

    #[test]
    fn deferring_committed_tail_toggle_suppresses_next_boundary_autocorrect() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );

        engine.defer_committed_tail_manual_toggle_to_daemon();

        assert!(engine.committed_tail.suppress_next_autocorrect);
    }

    #[test]
    fn pending_undo_precedes_unproven_committed_tail_delegation() {
        let source = include_str!("shift.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        let delegation = production
            .find("ManualToggleAuthority::DaemonWordBuffer =>")
            .expect("proven-output delegation");
        let pending_undo = production
            .find("defer_pending_ime_auto_undo_until_visible")
            .expect("pending undo route");

        assert!(pending_undo < delegation);
    }

    #[test]
    fn physical_double_shift_splits_terminal_commit_from_surrounding_text_replay() {
        let source = include_str!("shift.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        let committed_tail_arm = production
            .split("ManualToggleAuthority::ImeCommittedTail =>")
            .nth(1)
            .expect("IME committed-tail authority arm")
            .split("ManualToggleAuthority::DaemonWordBuffer =>")
            .next()
            .expect("daemon authority arm follows committed-tail arm");

        assert!(committed_tail_arm.contains("terminal_committed_tail_executor_available"));
        assert!(committed_tail_arm.contains("toggle_committed_tail_target(emitter).await"));
        assert!(committed_tail_arm.contains("prepare_exact_manual_toggle_layout_handoff"));
        assert!(!committed_tail_arm.contains("defer_committed_tail_manual_toggle_to_daemon"));
        assert!(committed_tail_arm.contains("ime_committed_tail"));
    }

    #[test]
    fn terminal_committed_tail_double_shift_uses_one_ime_output_frame() {
        let mut engine = LayIbusEngine::new(
            "/test/terminal".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );
        engine.client_context.content_purpose = 10;
        engine.client_context.cursor_cell_width = 11;
        engine.client_context.surrounding_text_supported = false;
        engine.committed_tail.buffer = "rjvvbn".to_string();

        let mut builder = AtomicEffectBuilder::default();
        let mut output = EngineOutput::atomic(&mut builder);
        let target_is_ru = zbus::block_on(engine.manual_toggle_active_text_target(&mut output))
            .expect("terminal Double Shift");
        let (status, effects) = builder.finish(true);

        assert_eq!(target_is_ru, Some(true));
        assert_eq!(status, PROPOSAL_FRAME_READY);
        assert_eq!(
            effects.len(),
            1,
            "terminal replacement must be one commit frame"
        );
        assert_eq!(engine.committed_tail.buffer, "коммит");
        assert!(engine.layout_gesture.layout_is_ru);
        assert!(engine
            .committed_tail
            .pending_visible_postcondition
            .is_none());
    }

    #[test]
    fn terminal_committed_tail_double_shift_round_trips_and_preserves_boundary() {
        let mut engine = LayIbusEngine::new(
            "/test/terminal-round-trip".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );
        engine.client_context.content_purpose = 10;
        engine.client_context.cursor_cell_width = 11;
        engine.client_context.surrounding_text_supported = false;
        engine.committed_tail.buffer = "rjvvbn ".to_string();

        for (expected_text, expected_layout_is_ru) in [("коммит ", true), ("rjvvbn ", false)]
        {
            let mut builder = AtomicEffectBuilder::default();
            let mut output = EngineOutput::atomic(&mut builder);
            let target_is_ru = zbus::block_on(engine.manual_toggle_active_text_target(&mut output))
                .expect("terminal Double Shift round trip");
            let (status, effects) = builder.finish(true);

            assert_eq!(target_is_ru, Some(expected_layout_is_ru));
            assert_eq!(status, PROPOSAL_FRAME_READY);
            assert_eq!(effects.len(), 1);
            assert_eq!(engine.committed_tail.buffer, expected_text);
            assert_eq!(engine.layout_gesture.layout_is_ru, expected_layout_is_ru);
        }
    }

    #[test]
    fn active_composition_toggle_defers_state_mutation_to_commit_owner() {
        let source = include_str!("shift.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(!production.contains("self.composition.buffer = plan.replacement"));
        assert!(!production.contains("replace_last_tail_token_text(&plan.replacement"));
        assert!(production.contains("commit_verified_active_composition(emitter, authorized_edit)"));
    }

    #[test]
    fn active_composition_toggle_uses_one_typed_ime_authority() {
        let plan = lay::text_edit::plan_text_replacement("ghbdtn", "привет")
            .expect("manual toggle replacement plan");
        let action = lay::text_edit::plan_ime_manual_toggle_edit("ghbdtn", "привет", plan);
        let edit =
            lay::text_edit::authorize_backend_edit(lay::text_edit::TextEditBackend::Ime, action)
                .into_authorized()
                .expect("manual toggle must be authorized");

        assert_eq!(edit.backend(), lay::text_edit::TextEditBackend::Ime);
        assert_eq!(edit.action().from_text(), "ghbdtn");
        assert_eq!(edit.action().to_text(), "привет");
    }
}
