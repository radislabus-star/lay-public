use evdev::uinput::VirtualDevice;
use lay::decoder::{CorrectionTrigger, DecoderEditPlan};
use lay::keyboard::KeyEvent;
use lay::word_buffer::WordBuffer;
use std::time::Instant;

use super::super::super::physical_input_grab::PhysicalInputGrab;
use super::super::super::{
    active_auto_switch_layout, apply_text_replacement_pipeline, log,
    switch_or_restore_layout_after_text_edit,
};
use super::super::TypingAssistOutcome;
use super::memory::{
    remember_typing_assist_correction, TypingAssistMemoryContext, TypingAssistTiming,
};
use super::queued::next_correction_after_forwarded_spaces;

pub(crate) struct MinimalTypingReplacementContext<'a, 'grab> {
    pub(crate) buf: &'a mut WordBuffer,
    pub(crate) events: &'a [KeyEvent],
    pub(crate) edit: &'a DecoderEditPlan,
    pub(crate) original: &'a str,
    pub(crate) replacement: &'a str,
    pub(crate) rule_id: Option<&'a str>,
    pub(crate) cursor_offset: u32,
    pub(crate) timing: TypingAssistTiming,
    pub(crate) physical_grab: &'a mut PhysicalInputGrab<'grab>,
    pub(crate) kbd: &'a mut VirtualDevice,
    pub(crate) original_layout: Option<bool>,
    pub(crate) prefer_full_token_plan: bool,
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
        rule_id,
        cursor_offset,
        timing,
        physical_grab,
        kbd,
        original_layout,
        prefer_full_token_plan,
    } = ctx;
    let plan = if prefer_full_token_plan && matches!(edit.trigger, CorrectionTrigger::AfterSpace) {
        edit.verified_full_token_plan_for_cursor(cursor_offset)
    } else {
        edit.verified_plan_for_cursor(cursor_offset)
    };
    let Some(plan) = plan else {
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
    remember_typing_assist_correction(TypingAssistMemoryContext {
        buf,
        events,
        plan: &plan,
        original,
        replacement,
        rule_id,
        cursor_offset,
        timing,
    });
    switch_or_restore_layout_after_text_edit(
        active_auto_switch_layout(),
        insert_outcome.layout_is_ru,
        original_layout,
        "typing-assist",
        insert_outcome.layout_already_set,
    );
    let forwarded = physical_grab.forward_queued_typing(
        kbd,
        buf,
        insert_outcome.layout_is_ru,
        "typing-assist",
        trailing_space_count(replacement),
    );
    log(&format!(
        "✓ done: помощь при наборе {:?} → {:?} за {}ms",
        original,
        replacement,
        timing.started_at.elapsed().as_millis()
    ));
    if let Some(next) = next_correction_after_forwarded_spaces(buf, forwarded.spaces) {
        let (next_original, next_replacement) =
            (next.edit.original.clone(), next.edit.replacement.clone());
        return apply_minimal_typing_replacement(MinimalTypingReplacementContext {
            buf,
            events: &next.events,
            edit: &next.edit,
            original: &next_original,
            replacement: &next_replacement,
            rule_id: next.rule_id.as_deref(),
            cursor_offset: 0,
            timing: TypingAssistTiming {
                decision_ms: next.decision_ms,
                started_at: Instant::now(),
            },
            physical_grab,
            kbd,
            original_layout,
            prefer_full_token_plan,
        });
    }
    TypingAssistOutcome::Applied {
        layout_is_ru: insert_outcome.layout_is_ru,
    }
}

fn trailing_space_count(text: &str) -> usize {
    text.chars().rev().take_while(|ch| *ch == ' ').count()
}
