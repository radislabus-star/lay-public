use evdev::{uinput::VirtualDevice, Device, KeyCode};
use lay::word_buffer::WordBuffer;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::{
    active_auto_replace, active_correction_engine, active_replace_words,
    grab_physical_device_for_correction, handle_double_shift, lock_virtual_keyboard,
    run_manual_correction_with_scope,
};

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
    let _physical_grab = grab_physical_device_for_correction(device);
    let mut g = lock_virtual_keyboard(virtual_kbd);
    handle_double_shift(
        buffer,
        active_replace_words(),
        active_correction_engine(),
        active_auto_replace(),
        g.as_mut(),
        executing,
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
    let _physical_grab = grab_physical_device_for_correction(device);
    let mut g = lock_virtual_keyboard(virtual_kbd);
    run_manual_correction_with_scope(
        buffer,
        replace_words,
        g.as_mut(),
        executing,
        events_since_word_start,
        reason,
    )
}

pub(super) fn apply_manual_correction_result(
    correction_result: Option<bool>,
    current_layout_is_ru: &mut bool,
    last_layout_poll: &mut Instant,
    suppress_next_typing_assist_after_manual_replay: &mut bool,
) {
    if let Some(is_ru) = correction_result {
        *current_layout_is_ru = is_ru;
        *last_layout_poll = Instant::now();
        *suppress_next_typing_assist_after_manual_replay = true;
    }
}
