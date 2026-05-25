use evdev::uinput::VirtualDevice;
use lay::decoder::DecoderEditPlan;
use lay::keyboard::KeyEvent;
use lay::text_edit::offset_replacement_plan_for_cursor;
use lay::word_buffer::WordBuffer;
use std::time::Instant;

use super::super::super::physical_input_grab::PhysicalInputGrab;
use super::super::super::{
    active_auto_switch_layout, active_replace_words, apply_text_replacement,
    insert_prepared_text_for_replacement_plan, log, prepare_text_insert_for_replacement_plan,
    switch_or_restore_layout_after_text_edit,
};
use super::super::{find_typing_assist_correction, TypingAssistOutcome};
use super::memory::remember_typing_assist_correction;

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_minimal_typing_replacement(
    buf: &mut WordBuffer,
    events: &[KeyEvent],
    edit: &DecoderEditPlan,
    original: &str,
    replacement: &str,
    cursor_offset: u32,
    started_at: Instant,
    physical_grab: &mut PhysicalInputGrab<'_>,
    kbd: &mut VirtualDevice,
    original_layout: Option<bool>,
) -> TypingAssistOutcome {
    let plan = offset_replacement_plan_for_cursor(&edit.plan, cursor_offset);
    if cursor_offset == 0 && !edit.plan_matches_replacement() {
        log("⚠ typing-assist skipped before delete: edit plan invariant failed");
        return TypingAssistOutcome::NoCorrection;
    }
    log(&format!(
        "  typing-assist plan: left={} bs={} insert={:?} right={}",
        plan.move_left, plan.backspaces, plan.insert, plan.move_right
    ));
    let prepared_insert = match prepare_text_insert_for_replacement_plan(&plan, true) {
        Ok(prepared) => prepared,
        Err(e) => {
            log(&format!("⚠ typing-assist skipped before delete: {e}"));
            return TypingAssistOutcome::NoCorrection;
        }
    };
    if let Err(e) = apply_text_replacement(kbd, &plan) {
        log(&format!("⚠ typing-assist minimal replace failed: {e}"));
        return TypingAssistOutcome::NoCorrection;
    }
    let insert_outcome = match insert_prepared_text_for_replacement_plan(
        kbd,
        &plan,
        replacement,
        &prepared_insert,
        "typing-assist",
    ) {
        Ok(outcome) => outcome,
        Err(e) => {
            log(&format!("⚠ typing-assist {e}"));
            return TypingAssistOutcome::NoCorrection;
        }
    };
    remember_typing_assist_correction(
        buf,
        events,
        &plan,
        original,
        replacement,
        cursor_offset,
        started_at,
    );
    switch_or_restore_layout_after_text_edit(
        active_auto_switch_layout(),
        insert_outcome.layout_is_ru,
        original_layout,
        "typing-assist",
        insert_outcome.layout_already_set,
    );
    let forwarded =
        physical_grab.forward_queued_typing(kbd, buf, insert_outcome.layout_is_ru, "typing-assist");
    log(&format!(
        "✓ done: помощь при наборе {:?} → {:?} за {}ms",
        original,
        replacement,
        started_at.elapsed().as_millis()
    ));
    if forwarded.spaces > 0 {
        if let Some(next) =
            find_typing_assist_correction(buf, active_auto_switch_layout(), active_replace_words())
        {
            let (next_original, next_replacement) =
                (next.edit.original.clone(), next.edit.replacement.clone());
            return apply_minimal_typing_replacement(
                buf,
                &next.events,
                &next.edit,
                &next_original,
                &next_replacement,
                0,
                Instant::now(),
                physical_grab,
                kbd,
                original_layout,
            );
        }
    }
    TypingAssistOutcome::Applied {
        layout_is_ru: insert_outcome.layout_is_ru,
    }
}
