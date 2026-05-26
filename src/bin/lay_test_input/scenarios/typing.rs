use super::super::input_device::{hold_tap, tap};
use evdev::{uinput::VirtualDevice, KeyCode};
use std::thread::sleep;
use std::time::Duration;

pub(super) fn type_physical(
    dev: &mut VirtualDevice,
    physical_text: &str,
    pause_ms: u64,
) -> std::io::Result<()> {
    for ch in physical_text.chars() {
        tap_physical_char(dev, ch)?;
        sleep(Duration::from_millis(pause_ms));
    }
    Ok(())
}

pub(super) fn double_shift_manual(dev: &mut VirtualDevice, settle_ms: u64) -> std::io::Result<()> {
    double_shift_manual_after(dev, 200, settle_ms)
}

pub(super) fn double_shift_manual_after(
    dev: &mut VirtualDevice,
    before_ms: u64,
    settle_ms: u64,
) -> std::io::Result<()> {
    sleep(Duration::from_millis(before_ms));
    tap(dev, KeyCode::KEY_LEFTSHIFT.code())?;
    sleep(Duration::from_millis(80));
    tap(dev, KeyCode::KEY_LEFTSHIFT.code())?;
    sleep(Duration::from_millis(settle_ms));
    Ok(())
}

fn tap_physical_char(dev: &mut VirtualDevice, ch: char) -> std::io::Result<()> {
    let (key, shifted) = physical_key_for_char(ch)?;
    if shifted {
        hold_tap(dev, KeyCode::KEY_LEFTSHIFT.code(), key.code())
    } else {
        tap(dev, key.code())
    }
}

fn physical_key_for_char(ch: char) -> std::io::Result<(KeyCode, bool)> {
    let shifted = ch.is_ascii_uppercase();
    let key = match ch.to_ascii_lowercase() {
        'a' => KeyCode::KEY_A,
        'b' => KeyCode::KEY_B,
        'c' => KeyCode::KEY_C,
        'd' => KeyCode::KEY_D,
        'e' => KeyCode::KEY_E,
        'f' => KeyCode::KEY_F,
        'g' => KeyCode::KEY_G,
        'h' => KeyCode::KEY_H,
        'i' => KeyCode::KEY_I,
        'j' => KeyCode::KEY_J,
        'k' => KeyCode::KEY_K,
        'l' => KeyCode::KEY_L,
        'm' => KeyCode::KEY_M,
        'n' => KeyCode::KEY_N,
        'o' => KeyCode::KEY_O,
        'p' => KeyCode::KEY_P,
        'q' => KeyCode::KEY_Q,
        'r' => KeyCode::KEY_R,
        's' => KeyCode::KEY_S,
        't' => KeyCode::KEY_T,
        'u' => KeyCode::KEY_U,
        'v' => KeyCode::KEY_V,
        'w' => KeyCode::KEY_W,
        'x' => KeyCode::KEY_X,
        'y' => KeyCode::KEY_Y,
        'z' => KeyCode::KEY_Z,
        '0' => KeyCode::KEY_0,
        '1' => KeyCode::KEY_1,
        '2' => KeyCode::KEY_2,
        '3' => KeyCode::KEY_3,
        '4' => KeyCode::KEY_4,
        '5' => KeyCode::KEY_5,
        '6' => KeyCode::KEY_6,
        '7' => KeyCode::KEY_7,
        '8' => KeyCode::KEY_8,
        '9' => KeyCode::KEY_9,
        ' ' => KeyCode::KEY_SPACE,
        '-' => KeyCode::KEY_MINUS,
        '=' => KeyCode::KEY_EQUAL,
        '[' => KeyCode::KEY_LEFTBRACE,
        ']' => KeyCode::KEY_RIGHTBRACE,
        ',' => KeyCode::KEY_COMMA,
        '.' => KeyCode::KEY_DOT,
        ';' => KeyCode::KEY_SEMICOLON,
        '\'' => KeyCode::KEY_APOSTROPHE,
        '`' => KeyCode::KEY_GRAVE,
        '/' => KeyCode::KEY_SLASH,
        '\\' => KeyCode::KEY_BACKSLASH,
        other => return Err(unsupported_physical_char(other)),
    };
    Ok((key, shifted))
}

fn unsupported_physical_char(ch: char) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("unsupported physical scenario char: {ch:?}"),
    )
}
