use evdev::{uinput::VirtualDevice, EventType, InputEvent, KeyCode};
use lay::keyboard::KeyEvent;
use std::time::Duration;

use super::modifiers::{release_possible_modifiers, release_possible_modifiers_fast};

const KEY_PACE_MS: u64 = 1;
const BACKSPACE_DOWN_MS: u64 = 1;
const BACKSPACE_PACE_MS: u64 = 2;
const BACKSPACE_SETTLE_MS: u64 = 16;
const TEXT_REPLACE_BACKSPACE_DOWN_MS: u64 = 1;
const TEXT_REPLACE_BACKSPACE_PACE_MS: u64 = 1;
const TEXT_REPLACE_BACKSPACE_SETTLE_MS: u64 = 1;
const TEXT_INSERT_KEY_PACE_MS: u64 = 1;
const TEXT_INSERT_SPACE_SETTLE_MS: u64 = 0;
const ISOLATED_ZERO_PACE_MAX_EVENTS: usize = 32;

pub(crate) fn replay_keycodes(dev: &mut VirtualDevice, events: &[KeyEvent]) -> std::io::Result<()> {
    replay_keycodes_with_pace(dev, events, KEY_PACE_MS, 0, true)
}

pub(crate) fn replay_keycodes_fast_after_modifier_cleanup(
    dev: &mut VirtualDevice,
    events: &[KeyEvent],
) -> std::io::Result<()> {
    let key_pace_ms = if events.len() <= ISOLATED_ZERO_PACE_MAX_EVENTS {
        0
    } else {
        KEY_PACE_MS
    };
    replay_keycodes_with_pace(dev, events, key_pace_ms, 0, false)
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
    let key_pace_ms = if events.len() <= ISOLATED_ZERO_PACE_MAX_EVENTS {
        0
    } else {
        TEXT_INSERT_KEY_PACE_MS
    };
    replay_keycodes_with_pace(dev, events, key_pace_ms, 0, false)
}

fn replay_keycodes_with_pace(
    dev: &mut VirtualDevice,
    events: &[KeyEvent],
    key_pace_ms: u64,
    space_settle_ms: u64,
    cleanup_modifiers: bool,
) -> std::io::Result<()> {
    let shift_l = KeyCode::KEY_LEFTSHIFT.code();

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
            dev.emit(&[InputEvent::new(EventType::KEY.0, shift_l, 1)])?;
            dev.emit(&[InputEvent::new(EventType::KEY.0, ev.keycode, 1)])?;
            dev.emit(&[InputEvent::new(EventType::KEY.0, ev.keycode, 0)])?;
            dev.emit(&[InputEvent::new(EventType::KEY.0, shift_l, 0)])?;
        } else {
            dev.emit(&[InputEvent::new(EventType::KEY.0, ev.keycode, 1)])?;
            dev.emit(&[InputEvent::new(EventType::KEY.0, ev.keycode, 0)])?;
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
        dev.emit(&[InputEvent::new(EventType::KEY.0, code, 1)])?;
        dev.emit(&[InputEvent::new(EventType::KEY.0, code, 0)])?;
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
        dev.emit(&[InputEvent::new(EventType::KEY.0, bs, 1)])?;
        std::thread::sleep(Duration::from_millis(BACKSPACE_DOWN_MS));
        dev.emit(&[InputEvent::new(EventType::KEY.0, bs, 0)])?;
        std::thread::sleep(Duration::from_millis(BACKSPACE_PACE_MS));
    }
    std::thread::sleep(Duration::from_millis(BACKSPACE_SETTLE_MS));
    Ok(())
}

pub(crate) fn emit_backspaces_fast(dev: &mut VirtualDevice, n: u32) -> std::io::Result<()> {
    if n as usize > ISOLATED_ZERO_PACE_MAX_EVENTS {
        return emit_backspaces(dev, n);
    }
    let bs = KeyCode::KEY_BACKSPACE.code();
    for _ in 0..n {
        dev.emit(&[
            InputEvent::new(EventType::KEY.0, bs, 1),
            InputEvent::new(EventType::KEY.0, bs, 0),
        ])?;
    }
    Ok(())
}

pub(super) fn emit_backspaces_for_text_replace(
    dev: &mut VirtualDevice,
    n: u32,
) -> std::io::Result<()> {
    let bs = KeyCode::KEY_BACKSPACE.code();
    for _ in 0..n {
        dev.emit(&[InputEvent::new(EventType::KEY.0, bs, 1)])?;
        std::thread::sleep(Duration::from_millis(TEXT_REPLACE_BACKSPACE_DOWN_MS));
        dev.emit(&[InputEvent::new(EventType::KEY.0, bs, 0)])?;
        std::thread::sleep(Duration::from_millis(TEXT_REPLACE_BACKSPACE_PACE_MS));
    }
    std::thread::sleep(Duration::from_millis(TEXT_REPLACE_BACKSPACE_SETTLE_MS));
    Ok(())
}

pub(super) fn emit_backspaces_for_text_replace_fast(
    dev: &mut VirtualDevice,
    n: u32,
) -> std::io::Result<()> {
    if n as usize > ISOLATED_ZERO_PACE_MAX_EVENTS {
        return emit_backspaces_for_text_replace(dev, n);
    }
    let bs = KeyCode::KEY_BACKSPACE.code();
    for _ in 0..n {
        dev.emit(&[
            InputEvent::new(EventType::KEY.0, bs, 1),
            InputEvent::new(EventType::KEY.0, bs, 0),
        ])?;
    }
    Ok(())
}
