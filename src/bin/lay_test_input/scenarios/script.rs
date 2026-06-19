use super::super::desktop_probe::activate_layout;
use super::super::input_device::{double_alt, double_shift, double_shift_enter, tap};
use super::typing::type_physical;
use evdev::{uinput::VirtualDevice, KeyCode};
use std::fs;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

pub(super) fn run_script(dev: &mut VirtualDevice, path: &Path) -> std::io::Result<()> {
    let text = fs::read_to_string(path)?;
    run_script_text(dev, &text, &path.display().to_string())
}

pub(super) fn run_script_text(
    dev: &mut VirtualDevice,
    text: &str,
    source_name: &str,
) -> std::io::Result<()> {
    for (idx, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        match fields.as_slice() {
            ["layout", id] => {
                activate_layout(id);
                sleep(Duration::from_millis(250));
            }
            ["layout", id, settle_ms] => {
                activate_layout(id);
                sleep(Duration::from_millis(parse_u64(
                    settle_ms,
                    source_name,
                    idx,
                )?));
            }
            ["sleep", ms] => sleep(Duration::from_millis(parse_u64(ms, source_name, idx)?)),
            ["type", physical_text] => type_physical(dev, &decode_script_text(physical_text), 35)?,
            ["type", physical_text, pause_ms] => type_physical(
                dev,
                &decode_script_text(physical_text),
                parse_u64(pause_ms, source_name, idx)?,
            )?,
            ["space"] => tap(dev, KeyCode::KEY_SPACE.code())?,
            ["enter"] => tap(dev, KeyCode::KEY_ENTER.code())?,
            ["left"] => tap(dev, KeyCode::KEY_LEFT.code())?,
            ["right"] => tap(dev, KeyCode::KEY_RIGHT.code())?,
            ["up"] => tap(dev, KeyCode::KEY_UP.code())?,
            ["down"] => tap(dev, KeyCode::KEY_DOWN.code())?,
            ["backspace"] => tap(dev, KeyCode::KEY_BACKSPACE.code())?,
            ["double_shift"] => double_shift(dev, 900)?,
            ["double_shift_enter"] => double_shift_enter(dev, 900)?,
            ["double_alt"] => double_alt(dev, 900)?,
            _ => return Err(bad_script_line(source_name, idx, raw_line)),
        }
    }
    Ok(())
}

fn parse_u64(value: &str, source_name: &str, idx: usize) -> std::io::Result<u64> {
    value.parse::<u64>().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("bad number at {source_name}:{}: {e}", idx + 1),
        )
    })
}

fn decode_script_text(text: &str) -> String {
    text.replace("\\s", " ")
}

fn bad_script_line(source_name: &str, idx: usize, raw_line: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("bad script line {} in {source_name}: {raw_line:?}", idx + 1),
    )
}
