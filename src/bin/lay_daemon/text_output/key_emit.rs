use evdev::{uinput::VirtualDevice, EventType, InputEvent, KeyCode};
use lay::keyboard::KeyEvent;
use std::time::Duration;

use super::device::VIRTUAL_KEYBOARD_KEYS;
use super::modifiers::{release_possible_modifiers, release_possible_modifiers_fast};

const KEY_PACE_MS: u64 = 1;
const BACKSPACE_PACE_MS: u64 = 2;
const BACKSPACE_SETTLE_MS: u64 = 16;
const TEXT_REPLACE_BACKSPACE_PACE_MS: u64 = 1;
const TEXT_REPLACE_BACKSPACE_SETTLE_MS: u64 = 1;
const TEXT_INSERT_KEY_PACE_MS: u64 = 1;
const TEXT_INSERT_SPACE_SETTLE_MS: u64 = 0;
const ISOLATED_REPLAY_MAX_EVENTS: usize = 32;

pub(crate) fn validate_isolated_replay_bounds(
    backspaces: u32,
    events: &[KeyEvent],
) -> std::io::Result<()> {
    let backspaces = usize::try_from(backspaces).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Backspace count does not fit usize",
        )
    })?;
    if backspaces == 0 || events.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "isolated replay requires non-empty delete and insert batches",
        ));
    }
    if backspaces > ISOLATED_REPLAY_MAX_EVENTS || events.len() > ISOLATED_REPLAY_MAX_EVENTS {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "isolated replay exceeds the bounded batch: backspaces={backspaces} events={}",
                events.len()
            ),
        ));
    }
    Ok(())
}

trait KeyFrameEmitter {
    fn emit_frame(&mut self, events: &[InputEvent]) -> std::io::Result<()>;
}

impl KeyFrameEmitter for VirtualDevice {
    fn emit_frame(&mut self, events: &[InputEvent]) -> std::io::Result<()> {
        self.emit(events)
    }
}

fn release_all_frame() -> Vec<InputEvent> {
    VIRTUAL_KEYBOARD_KEYS
        .iter()
        .map(|key| InputEvent::new(EventType::KEY.0, key.code(), 0))
        .collect()
}

fn emit_closed_frame<E: KeyFrameEmitter>(
    emitter: &mut E,
    events: &[InputEvent],
) -> std::io::Result<()> {
    match emitter.emit_frame(events) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = emitter.emit_frame(&release_all_frame());
            Err(error)
        }
    }
}

pub(crate) fn release_all_virtual_keys(dev: &mut VirtualDevice) -> std::io::Result<()> {
    dev.emit(&release_all_frame())
}

fn key_tap_frame(code: u16) -> [InputEvent; 2] {
    [
        InputEvent::new(EventType::KEY.0, code, 1),
        InputEvent::new(EventType::KEY.0, code, 0),
    ]
}

fn shifted_key_tap_frame(code: u16) -> [InputEvent; 4] {
    let shift_l = KeyCode::KEY_LEFTSHIFT.code();
    [
        InputEvent::new(EventType::KEY.0, shift_l, 1),
        InputEvent::new(EventType::KEY.0, code, 1),
        InputEvent::new(EventType::KEY.0, code, 0),
        InputEvent::new(EventType::KEY.0, shift_l, 0),
    ]
}

pub(crate) fn emit_shifted_key_tap_fast(
    dev: &mut VirtualDevice,
    key: KeyCode,
) -> std::io::Result<()> {
    emit_closed_frame(dev, &shifted_key_tap_frame(key.code()))
}

pub(crate) fn replay_keycodes(dev: &mut VirtualDevice, events: &[KeyEvent]) -> std::io::Result<()> {
    replay_keycodes_with_pace(dev, events, KEY_PACE_MS, 0, true)
}

pub(super) fn replay_text_insert_keycodes(
    dev: &mut VirtualDevice,
    events: &[KeyEvent],
) -> std::io::Result<()> {
    replay_keycodes_with_pace(
        dev,
        events,
        TEXT_INSERT_KEY_PACE_MS,
        TEXT_INSERT_SPACE_SETTLE_MS,
        true,
    )
}

pub(super) fn replay_text_insert_keycodes_fast_after_modifier_cleanup(
    dev: &mut VirtualDevice,
    events: &[KeyEvent],
) -> std::io::Result<()> {
    let key_pace_ms = if events.len() <= ISOLATED_REPLAY_MAX_EVENTS {
        0
    } else {
        TEXT_INSERT_KEY_PACE_MS
    };
    replay_keycodes_with_pace(dev, events, key_pace_ms, 0, false)
}

pub(crate) fn replay_keycodes_isolated_paced_after_modifier_cleanup(
    dev: &mut VirtualDevice,
    events: &[KeyEvent],
) -> std::io::Result<()> {
    if events.is_empty() || events.len() > ISOLATED_REPLAY_MAX_EVENTS {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "isolated replay event count must be 1..={ISOLATED_REPLAY_MAX_EVENTS}, got {}",
                events.len()
            ),
        ));
    }
    replay_keycodes_with_pace(dev, events, KEY_PACE_MS, 0, false)
}

fn replay_keycodes_with_pace(
    dev: &mut VirtualDevice,
    events: &[KeyEvent],
    key_pace_ms: u64,
    space_settle_ms: u64,
    cleanup_modifiers: bool,
) -> std::io::Result<()> {
    // Fast double-Shift can leave physical Shift in kernel/Mutter modifier
    // state for a moment. Release modifiers before replay so output is not
    // accidentally capitalized.
    if cleanup_modifiers {
        if key_pace_ms == 0 && space_settle_ms == 0 {
            release_possible_modifiers_fast(dev)?;
        } else {
            release_possible_modifiers(dev)?;
        }
    }

    for ev in events {
        if ev.shift {
            emit_closed_frame(dev, &shifted_key_tap_frame(ev.keycode))?;
        } else {
            emit_closed_frame(dev, &key_tap_frame(ev.keycode))?;
        }
        let settle_ms = if ev.keycode == KeyCode::KEY_SPACE.code() && space_settle_ms > 0 {
            space_settle_ms
        } else {
            key_pace_ms
        };
        if settle_ms > 0 {
            std::thread::sleep(Duration::from_millis(settle_ms));
        }
    }
    Ok(())
}

pub(crate) fn emit_key_taps_fast(
    dev: &mut VirtualDevice,
    key: KeyCode,
    n: u32,
) -> std::io::Result<()> {
    emit_key_taps(dev, key, n, 0)
}

pub(super) fn emit_key_taps(
    dev: &mut VirtualDevice,
    key: KeyCode,
    n: u32,
    pace_ms: u64,
) -> std::io::Result<()> {
    let code = key.code();
    for _ in 0..n {
        emit_closed_frame(dev, &key_tap_frame(code))?;
        if pace_ms > 0 {
            std::thread::sleep(Duration::from_millis(pace_ms));
        }
    }
    Ok(())
}

pub(crate) fn emit_backspaces(dev: &mut VirtualDevice, n: u32) -> std::io::Result<()> {
    let bs = KeyCode::KEY_BACKSPACE.code();

    // Long batches can partially drop in Mutter/GTK for large deletes. Small
    // pacing keeps manual replay deletion deterministic.
    for _ in 0..n {
        emit_closed_frame(dev, &key_tap_frame(bs))?;
        std::thread::sleep(Duration::from_millis(BACKSPACE_PACE_MS));
    }
    std::thread::sleep(Duration::from_millis(BACKSPACE_SETTLE_MS));
    Ok(())
}

pub(super) fn emit_backspaces_for_text_replace(
    dev: &mut VirtualDevice,
    n: u32,
) -> std::io::Result<()> {
    let bs = KeyCode::KEY_BACKSPACE.code();
    for _ in 0..n {
        emit_closed_frame(dev, &key_tap_frame(bs))?;
        std::thread::sleep(Duration::from_millis(TEXT_REPLACE_BACKSPACE_PACE_MS));
    }
    std::thread::sleep(Duration::from_millis(TEXT_REPLACE_BACKSPACE_SETTLE_MS));
    Ok(())
}

pub(super) fn emit_backspaces_for_text_replace_fast(
    dev: &mut VirtualDevice,
    n: u32,
) -> std::io::Result<()> {
    if n as usize > ISOLATED_REPLAY_MAX_EVENTS {
        return emit_backspaces_for_text_replace(dev, n);
    }
    let bs = KeyCode::KEY_BACKSPACE.code();
    for _ in 0..n {
        emit_closed_frame(dev, &key_tap_frame(bs))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[derive(Default)]
    struct RecordingEmitter {
        frames: Vec<Vec<(u16, i32)>>,
        fail_first: bool,
    }

    impl KeyFrameEmitter for RecordingEmitter {
        fn emit_frame(&mut self, events: &[InputEvent]) -> io::Result<()> {
            self.frames.push(
                events
                    .iter()
                    .map(|event| (event.code(), event.value()))
                    .collect(),
            );
            if std::mem::take(&mut self.fail_first) {
                Err(io::Error::other("injected emit failure"))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn tap_frames_close_every_pressed_key_before_syn_report() {
        let plain = key_tap_frame(KeyCode::KEY_Y.code());
        assert_eq!(
            plain.map(|event| (event.code(), event.value())),
            [(KeyCode::KEY_Y.code(), 1), (KeyCode::KEY_Y.code(), 0)]
        );

        let shifted = shifted_key_tap_frame(KeyCode::KEY_Y.code());
        assert_eq!(
            shifted.map(|event| (event.code(), event.value())),
            [
                (KeyCode::KEY_LEFTSHIFT.code(), 1),
                (KeyCode::KEY_Y.code(), 1),
                (KeyCode::KEY_Y.code(), 0),
                (KeyCode::KEY_LEFTSHIFT.code(), 0),
            ]
        );
    }

    #[test]
    fn failed_frame_attempts_release_of_every_virtual_key() {
        let mut emitter = RecordingEmitter {
            fail_first: true,
            ..RecordingEmitter::default()
        };

        let error = emit_closed_frame(&mut emitter, &key_tap_frame(KeyCode::KEY_Y.code()))
            .expect_err("first frame must fail");

        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert_eq!(emitter.frames.len(), 2);
        assert_eq!(emitter.frames[1].len(), VIRTUAL_KEYBOARD_KEYS.len());
        assert!(emitter.frames[1].iter().all(|(_, value)| *value == 0));
        assert!(emitter.frames[1]
            .iter()
            .any(|(code, _)| *code == KeyCode::KEY_Y.code()));
    }

    #[test]
    fn isolated_replay_is_bounded_before_emission() {
        let one = [KeyEvent {
            keycode: KeyCode::KEY_Y.code(),
            shift: false,
            layout_is_ru: false,
        }];
        assert!(validate_isolated_replay_bounds(1, &one).is_ok());
        assert!(validate_isolated_replay_bounds(0, &one).is_err());
        assert!(validate_isolated_replay_bounds(33, &one).is_err());
        assert!(validate_isolated_replay_bounds(1, &[]).is_err());
        assert!(validate_isolated_replay_bounds(1, &vec![one[0]; 33]).is_err());
    }
}
