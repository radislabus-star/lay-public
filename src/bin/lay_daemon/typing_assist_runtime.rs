use evdev::{uinput::VirtualDevice, Device};
use lay::word_buffer::WordBuffer;
use std::sync::atomic::Ordering;
use std::time::Instant;

#[path = "typing_assist_runtime/candidate.rs"]
mod candidate;
#[path = "typing_assist_runtime/output.rs"]
mod output;

use candidate::find_typing_assist_correction;
use output::{apply_typing_assist_correction, TypingAssistApplyContext};

use super::{active_auto_switch_layout, log, TYPING_ASSIST_RUNTIME_READY};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TypingAssistOutcome {
    Applied,
    NoCorrection,
    Deferred,
}

pub(super) fn handle_typing_assist_after_space(
    buf: &mut WordBuffer,
    virtual_kbd: Option<&mut VirtualDevice>,
    physical_device: Option<&mut Device>,
    executing: &mut bool,
    cursor_offset: u32,
) -> TypingAssistOutcome {
    if !TYPING_ASSIST_RUNTIME_READY.load(Ordering::Relaxed) {
        log("· typing-assist skipped: warmup pending");
        return TypingAssistOutcome::NoCorrection;
    }

    let started_at = Instant::now();
    let Some(correction) = find_typing_assist_correction(buf, active_auto_switch_layout()) else {
        return TypingAssistOutcome::NoCorrection;
    };

    apply_typing_assist_correction(TypingAssistApplyContext {
        buf,
        virtual_kbd,
        physical_device,
        executing,
        cursor_offset,
        started_at,
        correction,
    })
}
