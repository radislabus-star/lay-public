use evdev::{uinput::VirtualDevice, Device, EventType, KeyCode};
use lay::keyboard::{is_typing_key, KeyEvent};
use lay::word_buffer::WordBuffer;

use super::{emit_key_taps_fast, emit_shifted_key_tap_fast, log};

pub(super) struct PhysicalInputGrab<'a> {
    device: Option<&'a mut Device>,
    active: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct ForwardedTyping {
    pub(super) keys: usize,
    pub(super) spaces: usize,
    pub(super) boundaries: usize,
    pub(super) manual_toggles: usize,
    pub(super) last_manual_toggle_layout_is_ru: Option<bool>,
}

#[derive(Debug, Default)]
struct QueuedLeftShiftTaps {
    pressed: bool,
    completed_taps: usize,
}

impl QueuedLeftShiftTaps {
    fn observe(&mut self, value: i32) -> bool {
        match value {
            1 => self.pressed = true,
            0 if self.pressed => {
                self.pressed = false;
                self.completed_taps += 1;
                if self.completed_taps == 2 {
                    self.completed_taps = 0;
                    return true;
                }
            }
            _ => {}
        }
        false
    }
}

impl<'a> PhysicalInputGrab<'a> {
    pub(super) fn new(device: Option<&'a mut Device>) -> Self {
        let Some(device) = device else {
            return Self {
                device: None,
                active: false,
            };
        };

        match device.grab() {
            Ok(()) => Self {
                device: Some(device),
                active: true,
            },
            Err(e) => {
                log(&format!(
                    "⚠ physical device grab failed: {e}; continuing without input isolation"
                ));
                Self {
                    device: Some(device),
                    active: false,
                }
            }
        }
    }

    pub(super) fn is_active(&self) -> bool {
        self.active
    }

    pub(super) fn forward_queued_typing(
        &mut self,
        virtual_kbd: &mut VirtualDevice,
        buf: &mut WordBuffer,
        layout_is_ru: bool,
        label: &str,
        skip_spaces: usize,
        forward_boundaries: bool,
    ) -> ForwardedTyping {
        self.forward_queued_input(
            virtual_kbd,
            buf,
            layout_is_ru,
            label,
            skip_spaces,
            forward_boundaries,
            None,
        )
    }

    pub(super) fn forward_queued_typing_with_manual_toggles(
        &mut self,
        virtual_kbd: &mut VirtualDevice,
        buf: &mut WordBuffer,
        layout_is_ru: bool,
        label: &str,
        skip_spaces: usize,
        forward_boundaries: bool,
        replay_manual_toggle: &mut dyn FnMut(&mut VirtualDevice, &mut WordBuffer) -> Option<bool>,
    ) -> ForwardedTyping {
        self.forward_queued_input(
            virtual_kbd,
            buf,
            layout_is_ru,
            label,
            skip_spaces,
            forward_boundaries,
            Some(replay_manual_toggle),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn forward_queued_input(
        &mut self,
        virtual_kbd: &mut VirtualDevice,
        buf: &mut WordBuffer,
        mut layout_is_ru: bool,
        label: &str,
        mut skip_spaces: usize,
        forward_boundaries: bool,
        mut replay_manual_toggle: Option<
            &mut dyn FnMut(&mut VirtualDevice, &mut WordBuffer) -> Option<bool>,
        >,
    ) -> ForwardedTyping {
        if !self.active {
            return ForwardedTyping::default();
        }

        let Some(device) = self.device.as_deref_mut() else {
            return ForwardedTyping::default();
        };

        let mut shift_active = false;
        let mut left_shift_taps = QueuedLeftShiftTaps::default();
        let mut forwarded = ForwardedTyping::default();
        loop {
            let events = match device.fetch_events() {
                Ok(events) => events.collect::<Vec<_>>(),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    log(&format!("⚠ {label} passthrough read failed: {e}"));
                    break;
                }
            };
            if events.is_empty() {
                break;
            }

            for event in events {
                if event.event_type() != EventType::KEY {
                    continue;
                }
                let key = KeyCode::new(event.code());
                let value = event.value();

                match key {
                    KeyCode::KEY_LEFTSHIFT => {
                        shift_active = value != 0;
                        if left_shift_taps.observe(value) {
                            if let Some(replay) = replay_manual_toggle.as_deref_mut() {
                                if let Some(target_layout_is_ru) = replay(virtual_kbd, buf) {
                                    layout_is_ru = target_layout_is_ru;
                                    forwarded.manual_toggles += 1;
                                    forwarded.last_manual_toggle_layout_is_ru =
                                        Some(target_layout_is_ru);
                                }
                            }
                        }
                        continue;
                    }
                    KeyCode::KEY_RIGHTSHIFT => {
                        shift_active = value != 0;
                        continue;
                    }
                    _ => {}
                }

                if value != 1 && value != 2 {
                    continue;
                }

                if key == KeyCode::KEY_SPACE {
                    if skip_spaces > 0 {
                        skip_spaces -= 1;
                        continue;
                    }
                    if let Err(e) = emit_key_taps_fast(virtual_kbd, KeyCode::KEY_SPACE, 1) {
                        log(&format!("⚠ {label} passthrough space failed: {e}"));
                        continue;
                    }
                    buf.handle_space();
                    forwarded.spaces += 1;
                    continue;
                }

                if forward_boundaries
                    && value == 1
                    && matches!(
                        key,
                        KeyCode::KEY_ENTER
                            | KeyCode::KEY_TAB
                            | KeyCode::KEY_BACKSPACE
                            | KeyCode::KEY_DELETE
                            | KeyCode::KEY_LEFT
                            | KeyCode::KEY_RIGHT
                            | KeyCode::KEY_UP
                            | KeyCode::KEY_DOWN
                            | KeyCode::KEY_HOME
                            | KeyCode::KEY_END
                            | KeyCode::KEY_ESC
                    )
                {
                    if let Err(e) = emit_key_taps_fast(virtual_kbd, key, 1) {
                        log(&format!(
                            "warning: {label} passthrough boundary failed: {e}"
                        ));
                        continue;
                    }
                    buf.reset_all();
                    forwarded.boundaries += 1;
                    continue;
                }

                if !is_typing_key(key) {
                    continue;
                }

                if let Err(e) = emit_forwarded_key_tap(virtual_kbd, key, shift_active) {
                    log(&format!("⚠ {label} passthrough key failed: {e}"));
                    continue;
                }
                buf.push(KeyEvent {
                    keycode: event.code(),
                    shift: shift_active,
                    layout_is_ru,
                });
                forwarded.keys += 1;
            }
        }

        if forwarded.keys + forwarded.spaces + forwarded.boundaries + forwarded.manual_toggles > 0 {
            log(&format!(
                "· {label} passthrough forwarded {} queued keys, {} spaces, {} boundaries, {} manual toggles",
                forwarded.keys,
                forwarded.spaces,
                forwarded.boundaries,
                forwarded.manual_toggles
            ));
        }
        forwarded
    }
}

impl Drop for PhysicalInputGrab<'_> {
    fn drop(&mut self) {
        if self.active {
            if let Some(device) = self.device.as_deref_mut() {
                if let Err(e) = device.ungrab() {
                    log(&format!("⚠ physical device ungrab failed: {e}"));
                }
            }
        }
    }
}

fn emit_forwarded_key_tap(
    dev: &mut VirtualDevice,
    key: KeyCode,
    shift: bool,
) -> std::io::Result<()> {
    if shift {
        emit_shifted_key_tap_fast(dev, key)
    } else {
        emit_key_taps_fast(dev, key, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::QueuedLeftShiftTaps;

    #[test]
    fn queued_left_shift_pairs_remain_exact_inverse_gestures() {
        let mut taps = QueuedLeftShiftTaps::default();
        let mut toggles = 0;

        assert!(!taps.observe(0));
        for _ in 0..4 {
            assert!(!taps.observe(1));
            toggles += usize::from(taps.observe(0));
        }

        assert_eq!(toggles, 2);
    }
}
