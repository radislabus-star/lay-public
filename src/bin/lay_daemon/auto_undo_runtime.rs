use evdev::uinput::VirtualDevice;
use lay::text_edit::{replacement_plan_matches, TextReplacement};
use lay::word_buffer::{PendingAutoUndo, UserLearningCorrection, WordBuffer};
use std::time::Instant;

use super::{
    append_user_correction_learning_log, apply_text_replacement,
    insert_prepared_text_for_replacement_plan, log, prepare_text_insert_for_replacement_plan,
    record_recent_action, release_possible_modifiers, switch_to_target_layout, ExecutingGuard,
};

pub(super) fn handle_pending_auto_undo(
    buf: &mut WordBuffer,
    undo: PendingAutoUndo,
    virtual_kbd: Option<&mut VirtualDevice>,
    executing: &mut bool,
    started_at: Instant,
) -> Option<bool> {
    let Some(kbd) = virtual_kbd else {
        log("⚠ auto-undo: нет uinput device");
        return None;
    };

    *executing = true;
    let _executing_guard = ExecutingGuard(executing);

    if let Err(e) = release_possible_modifiers(kbd) {
        log(&format!("⚠ auto-undo modifier cleanup failed: {e}"));
    }

    let plan = pending_auto_undo_plan(&undo);
    if !replacement_plan_matches(&undo.replacement, &undo.original, &plan) {
        log("⚠ auto-undo skipped before delete: edit plan invariant failed");
        return None;
    }
    let prepared_insert = match prepare_text_insert_for_replacement_plan(&plan, true) {
        Ok(prepared) => prepared,
        Err(e) => {
            log(&format!("⚠ auto-undo skipped before delete: {e}"));
            return None;
        }
    };
    if let Err(e) = apply_text_replacement(kbd, &plan) {
        log(&format!("⚠ auto-undo delete failed: {e}"));
        return None;
    }

    let insert_outcome = match insert_prepared_text_for_replacement_plan(
        kbd,
        &plan,
        &undo.original,
        &prepared_insert,
        "auto-undo",
    ) {
        Ok(outcome) => outcome,
        Err(e) => {
            log(&format!("⚠ auto-undo {e}"));
            return None;
        }
    };
    if insert_outcome.layout_already_set {
        log("  auto-undo layout already set by text insert");
    } else {
        match switch_to_target_layout(insert_outcome.layout_is_ru) {
            Ok(layout_id) => log(&format!("  auto-undo layout → {layout_id}")),
            Err(e) => log(&format!("⚠ auto-undo layout switch failed: {e}")),
        }
    }

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
        false,
    );
    buf.clear_pending_learning();
    if !buf.remember_visible_text_for_correction(&undo.original) {
        buf.reset_all();
    }
    log(&format!(
        "✓ done: auto-undo {:?} → {:?} за {}ms",
        undo.replacement,
        undo.original,
        started_at.elapsed().as_millis()
    ));
    Some(insert_outcome.layout_is_ru)
}

pub(super) fn pending_auto_undo_plan(undo: &PendingAutoUndo) -> TextReplacement {
    TextReplacement {
        move_left: 0,
        backspaces: undo.replacement.chars().count() as u32,
        insert: undo.original.clone(),
        move_right: 0,
    }
}
