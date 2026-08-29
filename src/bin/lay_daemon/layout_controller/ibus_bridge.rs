use std::process::Command;

use lay::desktop::is_ru_layout_id;

use super::super::log;

pub(super) fn read_current_layout_is_ru() -> Result<bool, String> {
    read_engine().map(|engine| is_ru_layout_id(&engine))
}

pub(super) fn verify_engine_once(expected_engine: &str) -> Result<(), String> {
    let observed_engine = read_engine()?;
    if observed_engine == expected_engine {
        Ok(())
    } else {
        Err(format!(
            "IBus engine readback mismatch: expected={expected_engine} actual={observed_engine}"
        ))
    }
}

pub(super) fn ensure_engine(ibus_engine: &str, target_is_ru: bool) -> Result<(), String> {
    if read_engine().is_ok_and(|engine| engine == ibus_engine) {
        return Ok(());
    }
    let already_target = read_current_layout_is_ru().is_ok_and(|current| current == target_is_ru);
    if let Err(error) = switch_engine(ibus_engine) {
        if already_target {
            log(&format!(
                "⚠ IBus refresh failed but engine is already target: {error}"
            ));
            return Ok(());
        }
        return Err(error);
    }
    if read_current_layout_is_ru().is_ok_and(|current| current == target_is_ru) {
        Ok(())
    } else {
        Err("IBus engine verify failed".to_string())
    }
}

fn switch_engine(engine: &str) -> Result<(), String> {
    let out = Command::new("ibus")
        .args(["engine", engine])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(())
}

fn read_engine() -> Result<String, String> {
    let out = Command::new("ibus")
        .arg("engine")
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
