use evdev::uinput::VirtualDevice;
use lay::text_edit::replacement_plan_matches;
use lay::word_buffer::{PendingAutoUndo, UserLearningCorrection, WordBuffer};
use std::time::Instant;

use super::action_log_runtime::RecentActionRecord;
use super::{
    append_user_correction_learning_log, apply_text_replacement_pipeline, log,
    record_recent_action, release_possible_modifiers, should_try_ime_text_backend,
    switch_or_restore_layout_after_text_edit, try_ime_replace_tail, ExecutingGuard,
};

pub(super) fn handle_pending_auto_undo(
    buf: &mut WordBuffer,
    undo: PendingAutoUndo,
    virtual_kbd: Option<&mut VirtualDevice>,
    executing: &mut bool,
    started_at: Instant,
) -> Option<bool> {
    let plan = undo.replacement_plan();
    if !replacement_plan_matches(&undo.replacement, &undo.original, &plan) {
        log("⚠ auto-undo skipped before delete: edit plan invariant failed");
        return None;
    }
    let edit_action = lay::text_edit::plan_recorded_undo_edit(
        &undo.replacement,
        &undo.original,
        plan.clone(),
        undo.words,
    );
    lay::action_log::record_candidate_edit_action_before_apply(
        &edit_action,
        lay::action_log::MutationLogRoute::AUTO_UNDO,
        None,
    );
    if should_try_ime_text_backend() {
        let ime_backend_action = lay::text_edit::authorize_backend_edit(
            lay::text_edit::TextEditBackend::Ime,
            edit_action.clone(),
        );
        let ime_reason = ime_backend_action.reason;
        let Some(ime_authorized) = ime_backend_action.into_authorized() else {
            log(&format!(
                "⚠ auto-undo IME blocked before dispatch: {ime_reason}; secondary backend blocked"
            ));
            return None;
        };
        let dispatch = try_ime_replace_tail(ime_authorized, "auto-undo");
        if dispatch.was_dispatched() {
            let target_layout = lay::keyboard::preferred_layout_for_text(&undo.original, true);
            switch_or_restore_layout_after_text_edit(true, target_layout, None, "auto-undo", false);
            remember_auto_undo(buf, &undo, started_at);
            log(&format!(
                "✓ done: auto-undo {:?} → {:?} через IME за {}ms",
                undo.replacement,
                undo.original,
                started_at.elapsed().as_millis()
            ));
            return Some(target_layout);
        }
        if !dispatch.permits_backend_reselection() {
            log(&format!(
                "⚠ auto-undo IME dispatch ended without apply: {}; secondary backend blocked",
                dispatch.reason()
            ));
            return None;
        }
    }

    let Some(kbd) = virtual_kbd else {
        log("⚠ auto-undo: нет uinput device");
        return None;
    };
    let daemon_backend_action = lay::text_edit::authorize_backend_edit(
        lay::text_edit::TextEditBackend::Daemon,
        edit_action,
    );
    let backend = daemon_backend_action.backend;
    let reason = daemon_backend_action.reason;
    let Some(daemon_authorized) = daemon_backend_action.into_authorized() else {
        log(&format!(
            "⚠ auto-undo daemon output blocked by executor contract: reason={} backend={} original={:?} replacement={:?}",
            reason,
            backend.as_str(),
            undo.replacement,
            undo.original
        ));
        return None;
    };

    *executing = true;
    let _executing_guard = ExecutingGuard(executing);

    if let Err(e) = release_possible_modifiers(kbd) {
        log(&format!("⚠ auto-undo modifier cleanup failed: {e}"));
    }

    let insert_outcome = match apply_text_replacement_pipeline(
        kbd,
        daemon_authorized,
        true,
        None,
        "auto-undo",
        false,
    ) {
        Ok(outcome) => outcome,
        Err(e) => {
            e.log("auto-undo", "delete failed");
            return None;
        }
    };
    switch_or_restore_layout_after_text_edit(
        true,
        insert_outcome.layout_is_ru,
        None,
        "auto-undo",
        insert_outcome.layout_already_set,
    );

    remember_auto_undo(buf, &undo, started_at);
    log(&format!(
        "✓ done: auto-undo {:?} → {:?} за {}ms",
        undo.replacement,
        undo.original,
        started_at.elapsed().as_millis()
    ));
    Some(insert_outcome.layout_is_ru)
}

fn remember_auto_undo(buf: &mut WordBuffer, undo: &PendingAutoUndo, started_at: Instant) {
    append_user_correction_learning_log(&UserLearningCorrection {
        lay_kind: undo.lay_kind.clone(),
        lay_from: undo.original.clone(),
        lay_to: undo.replacement.clone(),
        from: undo.replacement.clone(),
        to: undo.original.clone(),
        replace_words: undo.replace_words,
        words: undo.words,
    });
    record_recent_action(RecentActionRecord {
        kind: "auto-undo",
        from: &undo.replacement,
        to: &undo.original,
        replace_words: undo.replace_words,
        words: undo.words,
        started_at,
        input_gate: None,
        undo_available: false,
    });
    buf.clear_pending_learning();
    if !buf.remember_visible_text_for_correction(&undo.original) {
        buf.reset_all();
    }
}
