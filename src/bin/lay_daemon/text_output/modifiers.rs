use evdev::{uinput::VirtualDevice, EventType, InputEvent, KeyCode};
use std::time::Duration;

const MODIFIER_RELEASE_ROUNDS: usize = 2;
const MODIFIER_RELEASE_PACE_MS: u64 = 3;
const MODIFIER_RELEASE_SETTLE_MS: u64 = 4;
const FAST_MODIFIER_RELEASE_PACE_MS: u64 = 0;
const FAST_MODIFIER_RELEASE_SETTLE_MS: u64 = 0;

pub(crate) fn release_possible_modifiers(dev: &mut VirtualDevice) -> std::io::Result<()> {
    release_possible_modifiers_with_pace(dev, MODIFIER_RELEASE_PACE_MS, MODIFIER_RELEASE_SETTLE_MS)
}

pub(crate) fn release_possible_modifiers_fast(dev: &mut VirtualDevice) -> std::io::Result<()> {
    release_possible_modifiers_with_pace(
        dev,
        FAST_MODIFIER_RELEASE_PACE_MS,
        FAST_MODIFIER_RELEASE_SETTLE_MS,
    )
}

fn release_possible_modifiers_with_pace(
    dev: &mut VirtualDevice,
    pace_ms: u64,
    settle_ms: u64,
) -> std::io::Result<()> {
    let modifiers = [
        KeyCode::KEY_LEFTSHIFT.code(),
        KeyCode::KEY_RIGHTSHIFT.code(),
        KeyCode::KEY_LEFTCTRL.code(),
        KeyCode::KEY_RIGHTCTRL.code(),
        KeyCode::KEY_LEFTALT.code(),
        KeyCode::KEY_RIGHTALT.code(),
    ];
    let events: Vec<_> = modifiers
        .iter()
        .map(|code| InputEvent::new(EventType::KEY.0, *code, 0))
        .collect();

    for _ in 0..MODIFIER_RELEASE_ROUNDS {
        dev.emit(&events)?;
        if pace_ms > 0 {
            std::thread::sleep(Duration::from_millis(pace_ms));
        }
    }
    if settle_ms > 0 {
        std::thread::sleep(Duration::from_millis(settle_ms));
    }
    Ok(())
}
