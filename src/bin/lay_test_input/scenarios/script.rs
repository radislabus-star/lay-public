use super::super::desktop_probe::activate_layout;
use super::super::input_device::{double_shift, double_shift_enter, tap};
use super::typing::type_physical;
use evdev::{uinput::VirtualDevice, KeyCode};
use std::fs;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

pub(super) fn run_script(dev: &mut VirtualDevice, path: &Path) -> std::io::Result<()> {
    let text = fs::read_to_string(path)?;
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
            ["sleep", ms] => sleep(Duration::from_millis(parse_u64(ms, path, idx)?)),
            ["type", physical_text] => type_physical(dev, &decode_script_text(physical_text), 35)?,
            ["type", physical_text, pause_ms] => type_physical(
                dev,
                &decode_script_text(physical_text),
                parse_u64(pause_ms, path, idx)?,
            )?,
            ["enter"] => tap(dev, KeyCode::KEY_ENTER.code())?,
            ["double_shift"] => double_shift(dev, 900)?,
            ["double_shift_enter"] => double_shift_enter(dev, 900)?,
            _ => return Err(bad_script_line(path, idx, raw_line)),
        }
    }
    Ok(())
}

fn parse_u64(value: &str, path: &Path, idx: usize) -> std::io::Result<u64> {
    value.parse::<u64>().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("bad number at {}:{}: {e}", path.display(), idx + 1),
        )
    })
}

fn decode_script_text(text: &str) -> String {
    text.replace("\\s", " ")
}

fn bad_script_line(path: &Path, idx: usize, raw_line: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!(
            "bad script line {} in {}: {raw_line:?}",
            idx + 1,
            path.display()
        ),
    )
}
