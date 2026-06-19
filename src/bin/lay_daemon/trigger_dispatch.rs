use evdev::{uinput::VirtualDevice, Device, KeyCode};
use lay::word_buffer::WordBuffer;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::pending_typing_assist::PendingTypingAssist;

use super::physical_input_grab::PhysicalInputGrab;
use super::{
    active_auto_replace, active_correction_engine, active_replace_words, handle_double_shift,
    lock_virtual_keyboard, run_manual_correction_with_scope,
};
use super::{DShiftState, MultiTapPending, ShiftState};

pub(super) fn trigger_key_from_config(trigger: &str) -> KeyCode {
    match trigger {
        "double-ctrl" => KeyCode::KEY_LEFTCTRL,
        "double-alt" => KeyCode::KEY_LEFTALT,
        "caps-lock" => KeyCode::KEY_CAPSLOCK,
        "single-rshift" => KeyCode::KEY_RIGHTSHIFT,
        "single-rctrl" => KeyCode::KEY_RIGHTCTRL,
        "single-ralt" => KeyCode::KEY_RIGHTALT,
        "single-pause" => KeyCode::KEY_PAUSE,
        _ => KeyCode::KEY_LEFTSHIFT,
    }
}

pub(super) fn is_single_trigger_id(trigger: &str) -> bool {
    trigger.starts_with("single-")
}

pub(super) fn run_configured_manual_correction(
    buffer: &mut WordBuffer,
    device: &mut Device,
    virtual_kbd: &Arc<Mutex<Option<VirtualDevice>>>,
    executing: &mut bool,
) -> Option<bool> {
    let mut physical_grab = PhysicalInputGrab::new(Some(device));
    let input_isolated = physical_grab.is_active();
    let mut g = lock_virtual_keyboard(virtual_kbd);
    handle_double_shift(
        buffer,
        active_replace_words(),
        active_correction_engine(),
        active_auto_replace(),
        g.as_mut(),
        executing,
        input_isolated,
        Some(&mut physical_grab),
    )
}

pub(super) fn run_scoped_manual_correction(
    buffer: &mut WordBuffer,
    replace_words: usize,
    device: &mut Device,
    virtual_kbd: &Arc<Mutex<Option<VirtualDevice>>>,
    executing: &mut bool,
    events_since_word_start: u32,
    reason: &str,
) -> Option<bool> {
    let mut physical_grab = PhysicalInputGrab::new(Some(device));
    let input_isolated = physical_grab.is_active();
    let mut g = lock_virtual_keyboard(virtual_kbd);
    run_manual_correction_with_scope(
        buffer,
        replace_words,
        g.as_mut(),
        executing,
        events_since_word_start,
        reason,
        input_isolated,
        Some(&mut physical_grab),
    )
}

pub(super) fn apply_manual_correction_result(
    correction_result: Option<bool>,
    current_layout_is_ru: &mut bool,
    last_layout_poll: &mut Instant,
    suppress_next_typing_assist_after_manual_replay: &mut bool,
    pending_typing_assist_after_space: &mut Option<PendingTypingAssist>,
) {
    if let Some(is_ru) = correction_result {
        *current_layout_is_ru = is_ru;
        *last_layout_poll = Instant::now();
        *suppress_next_typing_assist_after_manual_replay = true;
        pending_typing_assist_after_space.take();
    }
}

pub(super) struct ManualTriggerCompletion<'a> {
    pub(super) current_layout_is_ru: &'a mut bool,
    pub(super) last_layout_poll: &'a mut Instant,
    pub(super) suppress_next_typing_assist_after_manual_replay: &'a mut bool,
    pub(super) pending_typing_assist_after_space: &'a mut Option<PendingTypingAssist>,
    pub(super) shift_state: &'a mut ShiftState,
    pub(super) dshift_state: &'a mut DShiftState,
    pub(super) pending_multi_tap: &'a mut Option<MultiTapPending>,
    pub(super) last_double_at: &'a mut Option<Instant>,
    pub(super) clear_on_next_typing: &'a mut bool,
}

pub(super) fn complete_manual_trigger(
    correction_result: Option<bool>,
    ctx: ManualTriggerCompletion<'_>,
) {
    apply_manual_correction_result(
        correction_result,
        ctx.current_layout_is_ru,
        ctx.last_layout_poll,
        ctx.suppress_next_typing_assist_after_manual_replay,
        ctx.pending_typing_assist_after_space,
    );
    ctx.shift_state.clear_shifts();
    *ctx.dshift_state = DShiftState::Idle;
    *ctx.pending_multi_tap = None;
    *ctx.last_double_at = Some(Instant::now());
    *ctx.clear_on_next_typing = true;
}
