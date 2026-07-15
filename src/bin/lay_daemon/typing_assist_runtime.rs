use evdev::{uinput::VirtualDevice, Device};
use lay::word_buffer::WordBuffer;
use std::sync::atomic::Ordering;
use std::time::Instant;

#[path = "typing_assist_runtime/candidate.rs"]
mod candidate;
#[path = "typing_assist_runtime/output.rs"]
mod output;

pub(crate) use candidate::{find_typing_assist_correction, TypingAssistCorrection};
use output::{apply_typing_assist_correction, TypingAssistApplyContext};

use super::{
    active_auto_switch_layout, active_typing_assist_words, log, DaemonTextObservation,
    TYPING_ASSIST_RUNTIME_READY,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TypingAssistOutcome {
    Applied { layout_is_ru: bool },
    NoCorrection,
    Deferred,
}

pub(super) fn prepare_typing_assist_after_space(
    buf: &WordBuffer,
) -> Option<TypingAssistCorrection> {
    if !TYPING_ASSIST_RUNTIME_READY.load(Ordering::Relaxed) {
        log("· typing-assist skipped: warmup pending");
        return None;
    }

    find_typing_assist_correction(
        buf,
        active_auto_switch_layout(),
        active_typing_assist_words(),
    )
}

pub(super) fn apply_prepared_typing_assist_after_space(
    buf: &mut WordBuffer,
    virtual_kbd: Option<&mut VirtualDevice>,
    physical_device: Option<&mut Device>,
    executing: &mut bool,
    cursor_offset: u32,
    correction: TypingAssistCorrection,
    text_observation: DaemonTextObservation<'_>,
) -> TypingAssistOutcome {
    apply_typing_assist_correction(TypingAssistApplyContext {
        buf,
        virtual_kbd,
        physical_device,
        executing,
        cursor_offset,
        started_at: Instant::now(),
        correction,
        text_observation,
    })
}
