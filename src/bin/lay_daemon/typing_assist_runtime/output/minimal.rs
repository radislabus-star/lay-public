use evdev::uinput::VirtualDevice;
use lay::decoder::DecoderEditPlan;
use lay::keyboard::KeyEvent;
use lay::word_buffer::WordBuffer;
use std::time::Instant;

use super::super::super::physical_input_grab::PhysicalInputGrab;
use super::super::super::{
    active_auto_switch_layout, active_replace_words, apply_text_replacement_pipeline, log,
    switch_or_restore_layout_after_text_edit,
};
use super::super::{find_typing_assist_correction, TypingAssistOutcome};
use super::memory::remember_typing_assist_correction;

pub(crate) struct MinimalTypingReplacementContext<'a, 'grab> {
    pub(crate) buf: &'a mut WordBuffer,
    pub(crate) events: &'a [KeyEvent],
    pub(crate) edit: &'a DecoderEditPlan,
    pub(crate) original: &'a str,
    pub(crate) replacement: &'a str,
    pub(crate) cursor_offset: u32,
    pub(crate) started_at: Instant,
    pub(crate) physical_grab: &'a mut PhysicalInputGrab<'grab>,
    pub(crate) kbd: &'a mut VirtualDevice,
    pub(crate) original_layout: Option<bool>,
}

pub(crate) fn apply_minimal_typing_replacement(
    ctx: MinimalTypingReplacementContext<'_, '_>,
) -> TypingAssistOutcome {
    let MinimalTypingReplacementContext {
        buf,
        events,
        edit,
        original,
        replacement,
        cursor_offset,
        started_at,
        physical_grab,
        kbd,
        original_layout,
    } = ctx;
    let Some(plan) = edit.verified_plan_for_cursor(cursor_offset) else {
        log("⚠ typing-assist skipped before delete: edit plan invariant failed");
        return TypingAssistOutcome::NoCorrection;
    };
    log(&format!(
        "  typing-assist plan: left={} bs={} insert={:?} right={}",
        plan.move_left, plan.backspaces, plan.insert, plan.move_right
    ));
    let fast_output = physical_grab.is_active();
    let insert_outcome = match apply_text_replacement_pipeline(
        kbd,
        &plan,
        replacement,
        true,
        "typing-assist",
        fast_output,
    ) {
        Ok(outcome) => outcome,
        Err(e) => {
            e.log("typing-assist", "minimal replace failed");
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
            return apply_minimal_typing_replacement(MinimalTypingReplacementContext {
                buf,
                events: &next.events,
                edit: &next.edit,
                original: &next_original,
                replacement: &next_replacement,
                cursor_offset: 0,
                started_at: Instant::now(),
                physical_grab,
                kbd,
                original_layout,
            });
        }
    }
    TypingAssistOutcome::Applied {
        layout_is_ru: insert_outcome.layout_is_ru,
    }
}
