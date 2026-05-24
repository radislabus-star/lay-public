use evdev::{uinput::VirtualDevice, KeyCode};
use lay::decoder::DecoderEditPlan;
use lay::keyboard::KeyEvent;
use lay::text_edit::plan_committed_whitespace_insertions;
use lay::word_buffer::WordBuffer;
use std::time::Instant;

use super::super::super::physical_input_grab::PhysicalInputGrab;
use super::super::super::{
    active_auto_switch_layout, apply_text_replacement, emit_key_taps_fast, log,
    switch_or_restore_layout_after_text_edit,
};
use super::super::TypingAssistOutcome;
use super::memory::remember_typing_assist_correction;

#[allow(clippy::too_many_arguments)]
pub(crate) fn try_apply_whitespace_insertions(
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
) -> Option<TypingAssistOutcome> {
    let space_plans = plan_committed_whitespace_insertions(original, replacement, cursor_offset)
        .filter(|plans| plans.len() == 1)?;
    log(&format!(
        "  typing-assist whitespace plans: count={}",
        space_plans.len()
    ));
    for plan in &space_plans {
        if let Err(e) = apply_text_replacement(kbd, plan) {
            log(&format!("⚠ typing-assist space insert move failed: {e}"));
            return Some(TypingAssistOutcome::NoCorrection);
        }
        if let Err(e) = emit_key_taps_fast(kbd, KeyCode::KEY_SPACE, 1) {
            log(&format!("⚠ typing-assist space insert failed: {e}"));
            return Some(TypingAssistOutcome::NoCorrection);
        }
        if let Err(e) = emit_key_taps_fast(kbd, KeyCode::KEY_RIGHT, plan.move_right) {
            log(&format!("⚠ typing-assist cursor restore failed: {e}"));
            return Some(TypingAssistOutcome::NoCorrection);
        }
    }
    switch_or_restore_layout_after_text_edit(
        active_auto_switch_layout(),
        true,
        original_layout,
        "typing-assist",
        false,
    );
    physical_grab.forward_queued_typing(kbd, buf, true, "typing-assist");
    remember_typing_assist_correction(
        buf,
        events,
        &edit.plan,
        original,
        replacement,
        cursor_offset,
        started_at,
    );
    log(&format!(
        "✓ done: помощь при наборе {:?} → {:?} через whitespace insertions за {}ms",
        original,
        replacement,
        started_at.elapsed().as_millis()
    ));
    Some(TypingAssistOutcome::Applied { layout_is_ru: true })
}
