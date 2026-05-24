use evdev::{uinput::VirtualDevice, Device, EventType, InputEvent, KeyCode};
use lay::keyboard::{is_typing_key, KeyEvent};
use lay::word_buffer::WordBuffer;

use super::{emit_key_taps_fast, log};

pub(super) struct PhysicalInputGrab<'a> {
    device: Option<&'a mut Device>,
    active: bool,
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
    ) {
        if !self.active {
            return;
        }

        let Some(device) = self.device.as_deref_mut() else {
            return;
        };

        let mut shift_active = false;
        let mut forwarded = 0usize;
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
                    KeyCode::KEY_LEFTSHIFT | KeyCode::KEY_RIGHTSHIFT => {
                        shift_active = value != 0;
                        continue;
                    }
                    _ => {}
                }

                if value != 1 && value != 2 {
                    continue;
                }

                if key == KeyCode::KEY_SPACE {
                    if let Err(e) = emit_key_taps_fast(virtual_kbd, KeyCode::KEY_SPACE, 1) {
                        log(&format!("⚠ {label} passthrough space failed: {e}"));
                        continue;
                    }
                    buf.handle_space();
                    forwarded += 1;
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
                forwarded += 1;
            }
        }

        if forwarded > 0 {
            log(&format!(
                "· {label} passthrough forwarded {forwarded} queued keys"
            ));
        }
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
        dev.emit(&[
            InputEvent::new(EventType::KEY.0, KeyCode::KEY_LEFTSHIFT.code(), 1),
            InputEvent::new(EventType::KEY.0, key.code(), 1),
            InputEvent::new(EventType::KEY.0, key.code(), 0),
            InputEvent::new(EventType::KEY.0, KeyCode::KEY_LEFTSHIFT.code(), 0),
        ])
    } else {
        emit_key_taps_fast(dev, key, 1)
    }
}
