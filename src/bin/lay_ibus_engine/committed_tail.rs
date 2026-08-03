use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::engine::{LayIbusEngine, PendingSystemOutcomeFeedback, SystemOutcomeKind};
use super::state::CommittedTailReplaceRequest;
use super::text::make_ibus_text;
use super::trace;
use lay::manual_toggle::{plan_manual_toggle, ManualToggleRequest, VisibleTail};
use lay::text_edit::{VisibleTailSnapshot, VisibleTailSource};

impl LayIbusEngine {
    /// Applies only a verified current-token correction after Space.
    ///
    /// This is the autocorrect route, not the IME/preedit route:
    /// `BoundaryCell32 + shared L2/L3/L4/Bayes signals -> DecisionCore ->
    /// AuthorizedEdit`.
    /// Completion acceptance remains in `accept_stuck_tail()` and is never
    /// routed here.
    pub(super) async fn autocorrect_committed_token_on_space(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<bool> {
        if !self.config.auto_replace {
            return Ok(false);
        }
        let token = self.last_tail_token_text();
        if token.is_empty() {
            return Ok(false);
        }
        let boundary_text = format!("{token} ");
        let Some(decision) = lay::ime_correction::decide_active_composition_autocorrect(
            lay::ime_correction::ActiveCompositionAutocorrectRequest {
                text: &boundary_text,
                committed_tail: &self.tail_buffer,
                config: &self.config,
                active_layout_is_ru: Some(self.layout_is_ru),
            },
        ) else {
            trace::record(r#"{"kind":"ibus_space_autocorrect","status":"no_decision"}"#);
            return Ok(false);
        };
        let layout_transition = decision
            .input_gate
            .as_ref()
            .and_then(|trace| trace.selected_error_class.as_deref())
            == Some("wrong_layout");
        if !committed_tail_autocorrect_decision_is_authorized(&decision) {
            trace::record(format!(
                r#"{{"kind":"ibus_space_autocorrect","status":"not_authorized","allow_apply":{}}}"#,
                decision.action.allow_apply(),
            ));
            return Ok(false);
        }
        trace::record(r#"{"kind":"ibus_space_autocorrect","status":"authorized"}"#);

        lay::action_log::record_candidate_edit_action_before_apply(
            &decision.action,
            lay::action_log::MutationLogRoute::IME_COMMITTED_TAIL,
            decision.input_gate,
        );
        let replacement = decision.replacement;
        let expected_tail = VisibleTailSnapshot::new(
            VisibleTailSource::ImeCommittedTail,
            token.clone(),
            Some(self.path.clone()),
            self.tail_epoch,
        );
        let handled = self
            .replace_committed_tail(
                emitter,
                CommittedTailReplaceRequest::ime_autocorrect(
                    token.chars().count() as u32,
                    replacement.clone(),
                )
                .with_expected_tail(expected_tail)
                .with_winner_action(decision.action)
                .with_outcome_feedback(PendingSystemOutcomeFeedback {
                    original: token.clone(),
                    replacement: replacement.clone(),
                    source: VisibleTailSource::ImeCommittedTail,
                    kind: if layout_transition {
                        SystemOutcomeKind::LayoutProjection
                    } else {
                        SystemOutcomeKind::Correction
                    },
                }),
            )
            .await?;
        if handled {
            self.remember_pending_ime_auto_undo(boundary_text, replacement);
        }
        Ok(handled)
    }

    pub(super) async fn accept_stuck_tail(
        &mut self,
        emitter: &SignalEmitter<'_>,
        with_space: bool,
    ) -> fdo::Result<bool> {
        if self.tail_buffer.trim().is_empty() {
            return Ok(false);
        }
        let mut committed_suffix = self.selected_visible_completion_suffix();
        if committed_suffix.is_empty() {
            return Ok(false);
        }
        let tail_token = self.last_tail_token_text();
        if tail_token.is_empty() {
            return Ok(false);
        }
        let context_tail = self
            .tail_buffer
            .strip_suffix(&tail_token)
            .unwrap_or_default()
            .trim_end()
            .to_string();
        let typed_prefix = tail_token.clone();
        let suffix_chars = committed_suffix.chars().count();
        trace::record_completion_accept("stuck_tail", suffix_chars, with_space);
        let mut accepted_text = format!("{}{}", tail_token, committed_suffix.trim());
        if with_space {
            committed_suffix.push(' ');
            accepted_text.push(' ');
        }
        let action = lay::text_edit::plan_ime_completion_edit(
            "ibus-committed-tail-completion",
            900,
            tail_token,
            accepted_text.clone(),
        );
        lay::action_log::record_candidate_edit_action_before_apply(
            &action,
            lay::action_log::MutationLogRoute::IME_COMMITTED_TAIL,
            None,
        );
        let authorization =
            lay::text_edit::authorize_backend_edit(lay::text_edit::TextEditBackend::Ime, action);
        let Some(authorized_edit) = authorization.into_authorized() else {
            trace::record(r#"{"kind":"ibus_stuck_completion_blocked"}"#);
            return Ok(false);
        };
        if authorized_edit.action().to_text() != accepted_text {
            trace::record(r#"{"kind":"ibus_stuck_completion_authorized_text_mismatch"}"#);
            return Ok(false);
        }
        let Some(authorized_plan) = authorized_edit.action().plan() else {
            trace::record(r#"{"kind":"ibus_stuck_completion_authorized_plan_missing"}"#);
            return Ok(false);
        };
        if authorized_plan.backspaces != 0
            || authorized_plan.move_left != 0
            || authorized_plan.move_right != 0
            || authorized_plan.insert != committed_suffix
        {
            trace::record(r#"{"kind":"ibus_stuck_completion_authorized_plan_mismatch"}"#);
            return Ok(false);
        }
        self.clear_preedit(emitter).await?;
        Self::commit_text(emitter, make_ibus_text(authorized_plan.insert.clone()))
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        self.arm_pending_ime_completion_learning(
            context_tail,
            typed_prefix,
            accepted_text.trim().to_string(),
            with_space,
        );
        self.sync_tail_after_stuck_completion(&committed_suffix);
        Ok(true)
    }

    fn sync_tail_after_stuck_completion(&mut self, text: &str) {
        for ch in text.chars() {
            self.push_tail_char(ch);
        }
    }

    pub(super) async fn toggle_committed_tail_target(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<Option<bool>> {
        let Some(plan) = self.committed_tail_toggle_plan() else {
            return Ok(None);
        };
        trace::record_manual_toggle_plan(&plan);
        let handled = self
            .replace_committed_tail(
                emitter,
                CommittedTailReplaceRequest::ime_manual_toggle(
                    plan.backspaces,
                    plan.replacement.clone(),
                    plan.suppress_next_autocorrect,
                ),
            )
            .await?;
        if handled {
            lay::typing_cpu::TypingCpu::record_accepted_layout_projection(
                &plan.edit.original_token,
                &plan.replacement,
            );
            self.sync_layout_after_manual_toggle(&plan.replacement);
            self.trace_key("double_shift_committed_tail", 0, 0, true, None);
        }
        Ok(handled.then_some(plan.target_layout_is_ru))
    }

    pub(super) async fn undo_last_ime_autocorrect(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<Option<bool>> {
        let Some(pending) = self.take_pending_ime_auto_undo() else {
            return Ok(None);
        };
        let (rejected_context, accepted_context) =
            ime_auto_undo_contexts(&self.tail_buffer, &pending.original, &pending.replacement);
        let target_layout_is_ru =
            lay::keyboard::preferred_layout_for_text(&pending.original, self.layout_is_ru);
        let expected_tail = VisibleTailSnapshot::new(
            VisibleTailSource::ImeCommittedTail,
            pending.replacement.clone(),
            Some(self.path.clone()),
            self.tail_epoch,
        );
        let handled = self
            .replace_committed_tail(
                emitter,
                CommittedTailReplaceRequest::ime_auto_undo(
                    pending.replacement.chars().count() as u32,
                    pending.original.clone(),
                )
                .with_expected_tail(expected_tail),
            )
            .await?;
        if handled {
            lay::typing_cpu::TypingCpu::record_user_correction(
                &rejected_context,
                &rejected_context,
                &accepted_context,
                "ime_auto_undo",
            );
            trace::record(r#"{"kind":"ibus_auto_undo","status":"restored_exact_original"}"#);
            self.trace_key("double_shift_auto_undo", 0, 0, true, None);
            return Ok(Some(target_layout_is_ru));
        }
        self.restore_pending_ime_auto_undo(pending);
        Ok(None)
    }

    fn committed_tail_toggle_plan(&self) -> Option<lay::manual_toggle::ManualTogglePlan> {
        plan_manual_toggle(ManualToggleRequest {
            visible_tail: VisibleTail::ime_committed_tail(&self.tail_buffer),
            current_layout_is_ru: self.layout_is_ru,
            preserve_trailing_whitespace: true,
        })
    }

    #[cfg(test)]
    fn take_manual_toggle_autocorrect_suppression(&mut self) -> bool {
        let shared_suppression = self.take_autocorrect_suppression_handoff();
        let suppress = self.suppress_next_committed_tail_autocorrect || shared_suppression;
        self.suppress_next_committed_tail_autocorrect = false;
        suppress
    }
}

fn ime_auto_undo_contexts(
    visible_tail: &str,
    original: &str,
    replacement: &str,
) -> (String, String) {
    let rejected = visible_tail.to_string();
    let accepted = rejected
        .strip_suffix(replacement)
        .map(|prefix| format!("{prefix}{original}"))
        .unwrap_or_else(|| original.to_string());
    (rejected, accepted)
}

#[cfg(test)]
mod auto_undo_feedback_tests {
    use super::ime_auto_undo_contexts;

    #[test]
    fn undo_feedback_keeps_phrase_context_and_restores_only_tail() {
        let (rejected, accepted) =
            ime_auto_undo_contexts("обновлять модель по ход ", "ходу ", "ход ");
        assert_eq!(rejected, "обновлять модель по ход ");
        assert_eq!(accepted, "обновлять модель по ходу ");
    }
}

fn committed_tail_autocorrect_decision_is_authorized(
    decision: &lay::ime_correction::ActiveCompositionAutocorrectDecision,
) -> bool {
    decision.action.allow_apply()
}

#[cfg(test)]
mod tests {
    use super::{committed_tail_autocorrect_decision_is_authorized, LayIbusEngine};
    use lay::config::LayConfig;
    use lay::ime_correction::{
        decide_active_composition_autocorrect, ActiveCompositionAutocorrectRequest,
    };
    use std::sync::{Arc, Mutex};

    fn engine() -> LayIbusEngine {
        LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                auto_replace: true,
                typing_assist: true,
                auto_switch_layout: true,
                correction_safety: "experimental".to_string(),
                nanda_autocorrect: true,
                nanda_precognition: true,
                ..LayConfig::default()
            },
        )
    }

    #[test]
    fn stuck_completion_appends_suffix_to_tail_memory() {
        let mut engine = engine();
        for ch in "пров".chars() {
            engine.push_tail_char(ch);
        }

        engine.sync_tail_after_stuck_completion("ерка ");

        assert_eq!(engine.tail_buffer, "проверка ");
        assert_eq!(engine.preedit_fast.token(), "");
    }

    #[test]
    fn stuck_completion_is_an_authorized_ime_edit() {
        let source = include_str!("committed_tail.rs");
        assert!(
            source.contains("ibus-committed-tail-completion")
                && source.contains("accepted_text.clone()")
                && source.contains("make_ibus_text(committed_suffix.clone())"),
            "the proof must cover full token completion while CommitText emits only its suffix"
        );
    }

    #[test]
    fn manual_toggle_suppresses_next_boundary_autocorrect_once() {
        let mut engine = engine();
        engine.suppress_next_committed_tail_autocorrect = true;

        assert!(engine.take_manual_toggle_autocorrect_suppression());
        assert!(!engine.take_manual_toggle_autocorrect_suppression());
    }

    #[test]
    fn manual_toggle_suppression_survives_engine_handoff() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let engine_a = LayIbusEngine::new(
            "/test/a".to_string(),
            Arc::clone(&shared),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                ..LayConfig::default()
            },
        );
        engine_a.publish_autocorrect_suppression_handoff();

        let mut engine_b = LayIbusEngine::new(
            "/test/b".to_string(),
            shared,
            false,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                ..LayConfig::default()
            },
        );

        assert!(engine_b.take_manual_toggle_autocorrect_suppression());
        assert!(!engine_b.take_manual_toggle_autocorrect_suppression());
    }

    #[test]
    fn committed_tail_toggle_plan_uses_visible_ime_tail_not_old_daemon_buffer() {
        let mut engine = engine();
        engine.tail_buffer.push_str("вот ");
        engine.layout_is_ru = true;

        let plan = engine.committed_tail_toggle_plan().expect("toggle plan");

        assert_eq!(plan.replacement, "djn ");
        assert_eq!(plan.backspaces, 4);
    }

    #[test]
    fn committed_tail_preserves_winner_action_across_backend_boundary() {
        let source = include_str!("committed_tail.rs");

        assert!(
            source.contains("with_winner_action(decision.action)")
                && source.contains("with_expected_tail(expected_tail)"),
            "the IBus backend must carry the decision winner and its snapshot lease"
        );
    }

    #[test]
    fn committed_tail_space_autocorrect_keeps_decision_core_authority() {
        let cfg = LayConfig {
            text_backend: "ime".to_string(),
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: "experimental".to_string(),
            nanda_autocorrect: false,
            nanda_precognition: true,
            // This test owns the IBus/DecisionCore authority boundary, not the
            // availability or contents of the separately tested live L2 field.
            nanda_l2_phase_apply: false,
            ..LayConfig::default()
        };
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text: "прохоил ",
            committed_tail: "прохоил",
            config: &cfg,
            active_layout_is_ru: None,
        })
        .expect("shared decision");

        assert!(
            committed_tail_autocorrect_decision_is_authorized(&decision),
            "Space autocorrect must not locally narrow DecisionCore authority to boundary/layout only"
        );
    }
}
