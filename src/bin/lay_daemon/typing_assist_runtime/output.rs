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
#[path = "output/whitespace.rs"]
mod whitespace;

use defer::{defer_complex_edit, should_defer_immediate_typing_edit};
use ime::try_apply_ime_replacement;
use minimal::apply_minimal_typing_replacement;
use whitespace::try_apply_whitespace_insertions;

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
    let original = edit.original.clone();
    let replacement = edit.replacement.clone();
    let defer_complex_live_edit = cursor_offset == 0
        && !physical_grab.is_active()
        && should_defer_immediate_typing_edit(&edit);

    if cursor_offset == 0 {
        if let Some(outcome) = try_apply_ime_replacement(
            buf,
            &mut virtual_kbd,
            &mut physical_grab,
            &events,
            &original,
            &replacement,
            started_at,
        ) {
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
    if let Some(outcome) = try_apply_whitespace_insertions(
        buf,
        &events,
        &edit,
        &original,
        &replacement,
        cursor_offset,
        started_at,
        &mut physical_grab,
        kbd,
        original_layout,
    ) {
        return outcome;
    }

    apply_minimal_typing_replacement(
        buf,
        &events,
        &edit,
        &original,
        &replacement,
        cursor_offset,
        started_at,
        &mut physical_grab,
        kbd,
        original_layout,
    )
}
