use evdev::uinput::VirtualDevice;
use lay::text_edit::{replacement_plan_matches, TransitionAudit};
use lay::word_buffer::{PendingAutoUndo, UserLearningCorrection, WordBuffer};
use std::time::Instant;

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
    let edit_action = lay::text_edit::authorize_replacement_with_transition(
        "auto-undo",
        1000,
        &undo.replacement,
        &undo.original,
        plan.clone(),
        Some("auto_undo"),
        None,
        TransitionAudit::proven(
            "auto_undo_restore",
            "pending_auto_undo_recorded",
            true,
            false,
            undo.words.max(1),
        ),
    );
    lay::action_log::record_candidate_edit_action_before_apply(&edit_action, None);
    if !edit_action.allow_apply() {
        log(&format!(
            "⚠ auto-undo blocked by EditAction safety: reason={} original={:?} replacement={:?}",
            edit_action.safety_reason(),
            undo.replacement,
            undo.original
        ));
        return None;
    }

    if should_try_ime_text_backend()
        && try_ime_replace_tail(&undo.replacement, &undo.original, "auto-undo").unwrap_or(false)
    {
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

    let Some(kbd) = virtual_kbd else {
        log("⚠ auto-undo: нет uinput device");
        return None;
    };

    *executing = true;
    let _executing_guard = ExecutingGuard(executing);

    if let Err(e) = release_possible_modifiers(kbd) {
        log(&format!("⚠ auto-undo modifier cleanup failed: {e}"));
    }

    let insert_outcome = match apply_text_replacement_pipeline(
        kbd,
        &plan,
        &undo.original,
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
    record_recent_action(
        "auto-undo",
        &undo.replacement,
        &undo.original,
        undo.replace_words,
        undo.words,
        started_at,
        None,
        false,
    );
    buf.clear_pending_learning();
    if !buf.remember_visible_text_for_correction(&undo.original) {
        buf.reset_all();
    }
}
