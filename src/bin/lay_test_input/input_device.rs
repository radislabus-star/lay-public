use evdev::{uinput::VirtualDevice, AttributeSet, EventType, InputEvent, KeyCode};
use std::thread::sleep;
use std::time::Duration;

pub(crate) fn build_virtual_keyboard() -> std::io::Result<VirtualDevice> {
    let mut keys = AttributeSet::new();
    let all = [
        KeyCode::KEY_A,
        KeyCode::KEY_B,
        KeyCode::KEY_C,
        KeyCode::KEY_D,
        KeyCode::KEY_E,
        KeyCode::KEY_F,
        KeyCode::KEY_G,
        KeyCode::KEY_H,
        KeyCode::KEY_I,
        KeyCode::KEY_J,
        KeyCode::KEY_K,
        KeyCode::KEY_L,
        KeyCode::KEY_M,
        KeyCode::KEY_N,
        KeyCode::KEY_O,
        KeyCode::KEY_P,
        KeyCode::KEY_Q,
        KeyCode::KEY_R,
        KeyCode::KEY_S,
        KeyCode::KEY_T,
        KeyCode::KEY_U,
        KeyCode::KEY_V,
        KeyCode::KEY_W,
        KeyCode::KEY_X,
        KeyCode::KEY_Y,
        KeyCode::KEY_Z,
        KeyCode::KEY_SPACE,
        KeyCode::KEY_LEFTSHIFT,
        KeyCode::KEY_LEFTALT,
        KeyCode::KEY_RIGHTALT,
        KeyCode::KEY_LEFTCTRL,
        KeyCode::KEY_BACKSPACE,
        KeyCode::KEY_ENTER,
        KeyCode::KEY_LEFT,
        KeyCode::KEY_RIGHT,
        KeyCode::KEY_UP,
        KeyCode::KEY_DOWN,
        KeyCode::KEY_0,
        KeyCode::KEY_1,
        KeyCode::KEY_2,
        KeyCode::KEY_3,
        KeyCode::KEY_4,
        KeyCode::KEY_5,
        KeyCode::KEY_6,
        KeyCode::KEY_7,
        KeyCode::KEY_8,
        KeyCode::KEY_9,
        KeyCode::KEY_MINUS,
        KeyCode::KEY_EQUAL,
        KeyCode::KEY_LEFTBRACE,
        KeyCode::KEY_RIGHTBRACE,
        KeyCode::KEY_COMMA,
        KeyCode::KEY_DOT,
        KeyCode::KEY_SEMICOLON,
        KeyCode::KEY_APOSTROPHE,
        KeyCode::KEY_GRAVE,
        KeyCode::KEY_SLASH,
        KeyCode::KEY_BACKSLASH,
    ];
    for key in all {
        keys.insert(key);
    }
    VirtualDevice::builder()?
        .name("lay-test-virtual-keyboard")
        .with_keys(&keys)?
        .build()
}

pub(crate) fn tap(dev: &mut VirtualDevice, code: u16) -> std::io::Result<()> {
    dev.emit(&[InputEvent::new(EventType::KEY.0, code, 1)])?;
    sleep(Duration::from_millis(10));
    dev.emit(&[InputEvent::new(EventType::KEY.0, code, 0)])?;
    Ok(())
}

pub(crate) fn double_shift_enter(dev: &mut VirtualDevice, settle_ms: u64) -> std::io::Result<()> {
    double_shift(dev, settle_ms)?;
    tap(dev, KeyCode::KEY_ENTER.code())?;
    Ok(())
}

pub(crate) fn double_shift(dev: &mut VirtualDevice, settle_ms: u64) -> std::io::Result<()> {
    sleep(Duration::from_millis(220));
    tap(dev, KeyCode::KEY_LEFTSHIFT.code())?;
    sleep(Duration::from_millis(80));
    tap(dev, KeyCode::KEY_LEFTSHIFT.code())?;
    sleep(Duration::from_millis(settle_ms));
    Ok(())
}

pub(crate) fn double_alt(dev: &mut VirtualDevice, settle_ms: u64) -> std::io::Result<()> {
    sleep(Duration::from_millis(220));
    tap(dev, KeyCode::KEY_LEFTALT.code())?;
    sleep(Duration::from_millis(80));
    tap(dev, KeyCode::KEY_LEFTALT.code())?;
    sleep(Duration::from_millis(settle_ms));
    Ok(())
}

pub(crate) fn double_shift_fast(dev: &mut VirtualDevice, settle_ms: u64) -> std::io::Result<()> {
    tap_with_hold(dev, KeyCode::KEY_LEFTSHIFT.code(), 2)?;
    sleep(Duration::from_millis(8));
    tap_with_hold(dev, KeyCode::KEY_LEFTSHIFT.code(), 2)?;
    sleep(Duration::from_millis(settle_ms));
    Ok(())
}

pub(crate) fn extra_fast_lshift_taps(
    dev: &mut VirtualDevice,
    settle_ms: u64,
) -> std::io::Result<()> {
    for _ in 0..4 {
        tap_with_hold(dev, KeyCode::KEY_LEFTSHIFT.code(), 2)?;
        sleep(Duration::from_millis(8));
    }
    sleep(Duration::from_millis(settle_ms));
    Ok(())
}

pub(crate) fn tap_with_hold(
    dev: &mut VirtualDevice,
    code: u16,
    hold_ms: u64,
) -> std::io::Result<()> {
    dev.emit(&[InputEvent::new(EventType::KEY.0, code, 1)])?;
    sleep(Duration::from_millis(hold_ms));
    dev.emit(&[InputEvent::new(EventType::KEY.0, code, 0)])?;
    Ok(())
}

pub(crate) fn hold_tap(
    dev: &mut VirtualDevice,
    hold_code: u16,
    tap_code: u16,
) -> std::io::Result<()> {
    dev.emit(&[InputEvent::new(EventType::KEY.0, hold_code, 1)])?;
    sleep(Duration::from_millis(20));
    tap(dev, tap_code)?;
    sleep(Duration::from_millis(20));
    dev.emit(&[InputEvent::new(EventType::KEY.0, hold_code, 0)])?;
    Ok(())
}

pub(crate) fn hold_two_tap(
    dev: &mut VirtualDevice,
    first_hold_code: u16,
    second_hold_code: u16,
    tap_code: u16,
) -> std::io::Result<()> {
    dev.emit(&[InputEvent::new(EventType::KEY.0, first_hold_code, 1)])?;
    sleep(Duration::from_millis(10));
    dev.emit(&[InputEvent::new(EventType::KEY.0, second_hold_code, 1)])?;
    sleep(Duration::from_millis(10));
    tap(dev, tap_code)?;
    sleep(Duration::from_millis(10));
    dev.emit(&[InputEvent::new(EventType::KEY.0, second_hold_code, 0)])?;
    sleep(Duration::from_millis(10));
    dev.emit(&[InputEvent::new(EventType::KEY.0, first_hold_code, 0)])?;
    Ok(())
}
