use evdev::uinput::VirtualDevice;
use lay::keyboard::{map_events_to_layout, text_to_key_events, KeyEvent};
use lay::manual_toggle::{ManualTogglePlan, ManualToggleRoute};
use lay::text_edit::{
    authorize_backend_edit, plan_manual_edit, AuthorizedEdit, TextEditBackend, TextReplacement,
    VisibleTailSource,
};
use lay::word_buffer::WordBuffer;

use super::{
    activate_target_layout_once_for_exact_replay, cancel_exact_ime_autocorrect_suppression,
    cancel_exact_ime_manual_toggle_handoff_v2, capture_exact_focused_window_identity,
    emit_backspaces, log, release_possible_modifiers_fast,
    replay_keycodes_isolated_paced_after_modifier_cleanup,
    suppress_next_ime_autocorrect_for_exact_replay, validate_isolated_replay_bounds,
    verify_exact_focused_window_identity, ExecutingGuard, ImeCommittedTailReplay,
};

struct PreparedExactReplay {
    authorized_edit: AuthorizedEdit,
    events: Vec<KeyEvent>,
    visible_after: String,
}

enum ExactImeCleanupState {
    Handoff {
        epoch: u64,
        path: String,
    },
    SuppressionOrHandoff {
        epoch: u64,
        target_path: String,
        source_path: String,
    },
    Disarmed,
}

struct ExactImeHandoffCleanup {
    state: ExactImeCleanupState,
}

impl ExactImeHandoffCleanup {
    fn new(epoch: u64, path: String) -> Self {
        Self {
            state: ExactImeCleanupState::Handoff { epoch, path },
        }
    }

    fn arm_suppression(&mut self, epoch: u64, target_path: String, source_path: String) {
        self.state = ExactImeCleanupState::SuppressionOrHandoff {
            epoch,
            target_path,
            source_path,
        };
    }

    fn disarm(&mut self) {
        self.state = ExactImeCleanupState::Disarmed;
    }
}

impl Drop for ExactImeHandoffCleanup {
    fn drop(&mut self) {
        match &self.state {
            ExactImeCleanupState::Handoff { epoch, path } => {
                if let Err(error) = cancel_exact_ime_manual_toggle_handoff_v2(*epoch, path) {
                    log(&format!(
                        "warning: exact IME handoff cleanup failed: {error}"
                    ));
                }
            }
            ExactImeCleanupState::SuppressionOrHandoff {
                epoch,
                target_path,
                source_path,
            } => {
                if let Err(suppression_error) =
                    cancel_exact_ime_autocorrect_suppression(*epoch, target_path)
                {
                    if let Err(handoff_error) =
                        cancel_exact_ime_manual_toggle_handoff_v2(*epoch, source_path)
                    {
                        log(&format!(
                            "warning: exact IME cleanup failed: suppression={suppression_error}; handoff={handoff_error}"
                        ));
                    }
                }
            }
            ExactImeCleanupState::Disarmed => {}
        }
    }
}

pub(super) fn execute_exact_ime_tail_replay(
    buffer: &mut WordBuffer,
    virtual_keyboard: Option<&mut VirtualDevice>,
    executing: &mut bool,
    input_isolated: bool,
    replay: ImeCommittedTailReplay,
) -> Option<bool> {
    let (plan, lease) = replay.into_parts();
    let source_path = lease.expected_source_path().to_string();
    let mut handoff_cleanup =
        ExactImeHandoffCleanup::new(lease.expected_epoch(), source_path.clone());
    if !input_isolated {
        log("warning: exact IME tail replay blocked because physical input is not isolated");
        return None;
    }
    let Some(virtual_keyboard) = virtual_keyboard else {
        log("warning: exact IME tail replay blocked because uinput is unavailable");
        return None;
    };
    let focused_window = match capture_exact_focused_window_identity() {
        Ok(identity) => identity,
        Err(error) => {
            log(&format!(
                "warning: exact IME tail replay has no stable focused-window lease: {error}"
            ));
            return None;
        }
    };

    let prepared = match prepare_exact_replay(&plan, lease.expected_suffix()) {
        Ok(prepared) => prepared,
        Err(error) => {
            log(&format!(
                "warning: exact IME tail replay plan rejected: {error}"
            ));
            return None;
        }
    };

    *executing = true;
    let _executing_guard = ExecutingGuard(executing);
    if let Err(error) = lease.validate_current(plan.backspaces) {
        log(&format!(
            "warning: exact IME tail replay lease changed before layout: {error}"
        ));
        return None;
    }

    let layout_handoff = match activate_target_layout_once_for_exact_replay(
        lease.initial_layout_is_ru(),
        plan.target_layout_is_ru,
    ) {
        Ok(handoff) => handoff,
        Err(error) => {
            log(&format!(
                "warning: exact IME tail replay one-shot layout handoff failed: {error}"
            ));
            return None;
        }
    };
    let target_path = match lease
        .validate_after_controlled_layout_handoff(plan.backspaces, plan.target_layout_is_ru)
    {
        Ok(path) => path,
        Err(error) => {
            layout_handoff.restore_initial_best_effort("exact IME tail replay lease");
            log(&format!(
                "warning: exact IME tail replay lease changed after layout handoff: {error}"
            ));
            return None;
        }
    };
    if let Err(error) = verify_exact_focused_window_identity(&focused_window) {
        layout_handoff.restore_initial_best_effort("exact IME tail replay focus lease");
        log(&format!(
            "warning: exact IME tail replay focus changed after layout handoff: {error}"
        ));
        return None;
    }
    if let Err(error) = release_possible_modifiers_fast(virtual_keyboard) {
        layout_handoff.restore_initial_best_effort("exact IME tail replay modifier cleanup");
        log(&format!(
            "warning: exact IME tail replay modifier cleanup failed: {error}"
        ));
        return None;
    }

    let action = prepared.authorized_edit.action();
    let Some(authorized_plan) = action.plan() else {
        layout_handoff.restore_initial_best_effort("exact IME tail replay authorization");
        log("warning: exact IME tail replay authorization lost its replacement plan");
        return None;
    };
    if prepared.authorized_edit.backend() != TextEditBackend::Daemon
        || authorized_plan.backspaces != plan.backspaces
        || action.to_text() != plan.replacement
    {
        layout_handoff.restore_initial_best_effort("exact IME tail replay authorization");
        log("warning: exact IME tail replay authorization no longer matches its plan");
        return None;
    }

    handoff_cleanup.arm_suppression(lease.expected_epoch(), target_path.clone(), source_path);
    if let Err(error) = suppress_next_ime_autocorrect_for_exact_replay(
        lease.expected_suffix(),
        lease.expected_epoch(),
        &target_path,
        plan.target_layout_is_ru,
    ) {
        layout_handoff.restore_initial_best_effort("exact IME tail replay suppression");
        log(&format!(
            "warning: exact IME tail replay autocorrect suppression failed: {error}"
        ));
        return None;
    }
    if let Err(error) = verify_exact_focused_window_identity(&focused_window) {
        log(&format!(
            "warning: exact IME tail replay focus changed before delete: {error}"
        ));
        return None;
    }
    if let Err(error) = emit_backspaces(virtual_keyboard, plan.backspaces) {
        log(&format!(
            "warning: exact IME tail replay delete became indeterminate: {error}"
        ));
        return None;
    }
    if let Err(error) = verify_exact_focused_window_identity(&focused_window) {
        log(&format!(
            "warning: exact IME tail replay focus changed between delete and insert: {error}"
        ));
        return None;
    }
    if let Err(error) =
        replay_keycodes_isolated_paced_after_modifier_cleanup(virtual_keyboard, &prepared.events)
    {
        log(&format!(
            "warning: exact IME tail replay insert became indeterminate after delete: {error}"
        ));
        return None;
    }
    if let Err(error) = verify_exact_focused_window_identity(&focused_window) {
        log(&format!(
            "warning: exact IME tail replay focus changed during insert: {error}"
        ));
        return None;
    }
    handoff_cleanup.disarm();
    if !buffer.remember_visible_text_for_correction(&prepared.visible_after) {
        log(
            "warning: exact IME tail replay succeeded but daemon tail memory was not representable",
        );
    }
    log(&format!(
        "exact IME tail replay complete: backspaces={} replacement={:?}",
        plan.backspaces, plan.replacement
    ));
    Some(plan.target_layout_is_ru)
}

fn prepare_exact_replay(
    plan: &ManualTogglePlan,
    expected_suffix: &str,
) -> Result<PreparedExactReplay, String> {
    if plan.route != ManualToggleRoute::ImeCommittedTail
        || plan.edit.source != VisibleTailSource::ImeCommittedTail
    {
        return Err("plan is not owned by the exact IME committed-tail route".to_string());
    }
    if expected_suffix.is_empty()
        || expected_suffix.chars().count() != plan.backspaces as usize
        || plan.edit.delete_chars != plan.backspaces
        || plan.edit.insert_text != plan.replacement
        || plan.edit.target_layout_is_ru != plan.target_layout_is_ru
        || !plan.suppress_next_autocorrect
    {
        return Err("plan fields do not match the exact leased suffix".to_string());
    }
    let trailing = expected_suffix
        .strip_prefix(&plan.edit.original_token)
        .ok_or_else(|| "leased suffix does not begin with the exact original token".to_string())?;
    if !trailing.chars().all(|ch| ch == ' ') || !plan.replacement.ends_with(trailing) {
        return Err("exact replay supports only byte-preserved trailing spaces".to_string());
    }
    let prefix = plan
        .edit
        .original_tail
        .strip_suffix(expected_suffix)
        .ok_or_else(|| "leased suffix is absent from the observed tail".to_string())?;
    let events = text_to_key_events(&plan.replacement, plan.target_layout_is_ru)
        .ok_or_else(|| "replacement cannot be represented by exact uinput keys".to_string())?;
    if events.is_empty()
        || map_events_to_layout(&events, plan.target_layout_is_ru) != plan.replacement
    {
        return Err("prepared uinput keys do not reproduce the exact replacement".to_string());
    }
    validate_isolated_replay_bounds(plan.backspaces, &events).map_err(|error| error.to_string())?;

    let replacement = TextReplacement {
        move_left: 0,
        backspaces: plan.backspaces,
        insert: plan.replacement.clone(),
        move_right: 0,
    };
    let action = plan_manual_edit(
        "manual-exact-ime-tail-replay",
        1000,
        expected_suffix,
        &plan.replacement,
        replacement,
        1,
    );
    lay::action_log::record_candidate_edit_action_before_apply(
        &action,
        lay::action_log::MutationLogRoute::MANUAL_TEXT_REPLACE,
        None,
    );
    let authorization = authorize_backend_edit(TextEditBackend::Daemon, action);
    let backend = authorization.backend;
    let reason = authorization.reason;
    let authorized_edit = authorization.into_authorized().ok_or_else(|| {
        format!(
            "daemon edit authorization rejected: backend={} reason={reason}",
            backend.as_str()
        )
    })?;
    Ok(PreparedExactReplay {
        authorized_edit,
        events,
        visible_after: format!("{prefix}{}", plan.replacement),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use lay::manual_toggle::{plan_manual_toggle, ManualToggleRequest, VisibleTail};

    fn plan(tail: &str, current_layout_is_ru: bool) -> ManualTogglePlan {
        plan_manual_toggle(ManualToggleRequest {
            visible_tail: VisibleTail::ime_committed_tail(tail),
            current_layout_is_ru,
            preserve_trailing_whitespace: true,
        })
        .expect("exact toggle plan")
    }

    #[test]
    fn exact_replay_preserves_the_observed_prefix_and_autocomplete_tail() {
        let prepared = prepare_exact_replay(&plan("file ghjdthrf ", false), "ghjdthrf ")
            .expect("prepared replay");

        assert_eq!(prepared.visible_after, "file проверка ");
        assert_eq!(map_events_to_layout(&prepared.events, true), "проверка ");
        assert_eq!(prepared.authorized_edit.backend(), TextEditBackend::Daemon);
    }

    #[test]
    fn exact_replay_rejects_a_stale_or_misclassified_lease() {
        let exact = plan("ghbdtn", false);
        assert!(prepare_exact_replay(&exact, "hbdtn").is_err());

        let mut wrong_source = exact;
        wrong_source.route = ManualToggleRoute::Daemon;
        wrong_source.edit.source = VisibleTailSource::DaemonWordBuffer;
        assert!(prepare_exact_replay(&wrong_source, "ghbdtn").is_err());
    }

    #[test]
    fn exact_replay_source_has_one_mutation_and_orders_both_lease_checks_before_it() {
        let source = include_str!("exact_ime_tail_replay.rs");
        let production = source.split("#[cfg(test)]").next().expect("production");
        let before = production.find("validate_current").expect("first lease");
        let after = production
            .find("validate_after_controlled_layout_handoff")
            .expect("second lease");
        let delete = production.find("emit_backspaces(").expect("delete");
        let suppress = production
            .find("suppress_next_ime_autocorrect_for_exact_replay(")
            .expect("checked suppression");
        let insert = production
            .find("replay_keycodes_isolated_paced_after_modifier_cleanup(")
            .expect("insert");

        assert!(before < after && after < suppress && suppress < delete && delete < insert);
        assert_eq!(production.matches("emit_backspaces(").count(), 1);
        assert_eq!(
            production
                .matches("suppress_next_ime_autocorrect_for_exact_replay(")
                .count(),
            1
        );
        assert!(!production.contains("suppress_next_ime_autocorrect();"));
        assert!(production.contains("ExactImeHandoffCleanup"));
        let arm_cleanup = production
            .find("handoff_cleanup.arm_suppression")
            .expect("suppression rollback arm");
        let disarm_cleanup = production
            .find("handoff_cleanup.disarm()")
            .expect("suppression rollback disarm");
        assert!(arm_cleanup < suppress && suppress < delete && insert < disarm_cleanup);
        assert_eq!(
            production
                .matches("replay_keycodes_isolated_paced_after_modifier_cleanup(")
                .count(),
            1
        );
        assert!(!production.contains("retry_exact_ime_tail"));
        assert!(!production.contains("verify_target_layout_ready_for_replay"));
        assert!(!production.contains("LayoutCapabilityPreflight"));
        assert!(!production.contains("std::thread::sleep"));
    }

    #[test]
    fn exact_replay_rejects_long_batches_before_layout_or_text_mutation() {
        let long_tail = "a".repeat(33);
        let long_plan = plan(&long_tail, false);

        assert!(prepare_exact_replay(&long_plan, &long_tail).is_err());
    }

    #[test]
    fn exact_replay_rejects_non_space_trailing_whitespace_before_layout() {
        let mut tab_plan = plan("ghbdtn ", false);
        tab_plan.edit.original_tail = "ghbdtn\t".to_string();

        assert!(prepare_exact_replay(&tab_plan, "ghbdtn\t").is_err());
    }
}
