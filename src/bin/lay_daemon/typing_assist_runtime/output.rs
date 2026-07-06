use evdev::{uinput::VirtualDevice, Device};
use lay::word_buffer::WordBuffer;
use std::time::Instant;

#[path = "output/defer.rs"]
mod defer;
#[path = "output/ime.rs"]
mod ime;
#[path = "output/memory.rs"]
mod memory;
#[path = "output/minimal.rs"]
mod minimal;
#[path = "output/nanda_trace.rs"]
mod nanda_trace;
#[path = "output/queued.rs"]
mod queued;

use defer::{defer_complex_edit, should_defer_immediate_typing_edit};
use ime::{try_apply_ime_replacement, ImeTypingReplacementContext};
use memory::TypingAssistTiming;
use minimal::{apply_minimal_typing_replacement, MinimalTypingReplacementContext};

use super::super::physical_input_grab::PhysicalInputGrab;
use super::super::{
    log, read_current_layout_is_ru, release_possible_modifiers_fast, ExecutingGuard,
};
use super::candidate::TypingAssistCorrection;
use super::TypingAssistOutcome;

pub(crate) struct TypingAssistApplyContext<'a> {
    pub(crate) buf: &'a mut WordBuffer,
    pub(crate) virtual_kbd: Option<&'a mut VirtualDevice>,
    pub(crate) physical_device: Option<&'a mut Device>,
    pub(crate) executing: &'a mut bool,
    pub(crate) cursor_offset: u32,
    pub(crate) started_at: Instant,
    pub(crate) correction: TypingAssistCorrection,
}

pub(crate) fn apply_typing_assist_correction(
    ctx: TypingAssistApplyContext<'_>,
) -> TypingAssistOutcome {
    let TypingAssistApplyContext {
        buf,
        mut virtual_kbd,
        physical_device,
        executing,
        cursor_offset,
        started_at,
        correction,
    } = ctx;
    let mut physical_grab = PhysicalInputGrab::new(physical_device);
    let events = correction.events;
    let edit = correction.edit;
    let rule_id = correction.rule_id;
    let input_gate = correction.input_gate;
    let timing = TypingAssistTiming {
        decision_ms: correction.decision_ms,
        started_at,
    };
    let original = edit.original.clone();
    let replacement = edit.replacement.clone();
    let prefer_full_token_plan = true;
    let defer_complex_live_edit = cursor_offset == 0
        && !physical_grab.is_active()
        && should_defer_immediate_typing_edit(&edit);

    if cursor_offset == 0 {
        if let Some(outcome) = try_apply_ime_replacement(ImeTypingReplacementContext {
            buf,
            virtual_kbd: &mut virtual_kbd,
            physical_grab: &mut physical_grab,
            events: &events,
            original: &original,
            replacement: &replacement,
            rule_id: rule_id.as_deref(),
            input_gate: input_gate.clone(),
            timing,
        }) {
            return outcome;
        }
        if defer_complex_live_edit {
            return defer_complex_edit();
        }
    }

    if defer_complex_live_edit {
        return defer_complex_edit();
    }

    let Some(kbd) = virtual_kbd else {
        log("⚠ typing-assist: нет uinput device");
        return TypingAssistOutcome::NoCorrection;
    };

    *executing = true;
    let _executing_guard = ExecutingGuard(executing);

    if let Err(e) = release_possible_modifiers_fast(kbd) {
        log(&format!("⚠ typing-assist modifier cleanup failed: {e}"));
    }

    let original_layout = read_current_layout_is_ru().ok();
    apply_minimal_typing_replacement(MinimalTypingReplacementContext {
        buf,
        events: &events,
        edit: &edit,
        original: &original,
        replacement: &replacement,
        rule_id: rule_id.as_deref(),
        input_gate,
        cursor_offset,
        timing,
        physical_grab: &mut physical_grab,
        kbd,
        original_layout,
        prefer_full_token_plan,
    })
}
