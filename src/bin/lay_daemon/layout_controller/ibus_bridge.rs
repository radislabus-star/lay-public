use std::process::Command;

use lay::desktop::is_ru_layout_id;

use super::super::log;

pub(super) fn read_current_layout_is_ru() -> Result<bool, String> {
    read_engine().map(|engine| is_ru_layout_id(&engine))
}

pub(super) fn verify_engine_once(expected_engine: &str) -> Result<(), String> {
    verify_engine_with(
        expected_engine,
        super::ime_bridge::observed_ibus_engine,
        read_engine,
    )
}

fn verify_engine_with(
    expected_engine: &str,
    observed: impl FnOnce() -> Result<String, String>,
    legacy: impl FnOnce() -> Result<String, String>,
) -> Result<(), String> {
    let observed_engine = observed().or_else(|_| legacy())?;
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

#[cfg(test)]
mod observed_engine_tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn exact_engine_fast_success_does_not_invoke_legacy_reader() {
        assert!(verify_engine_with(
            "lay-ime-ru",
            || Ok("lay-ime-ru".into()),
            || panic!("unexpected CLI")
        )
        .is_ok());
    }

    #[test]
    fn exact_engine_read_error_invokes_legacy_reader_once() {
        let calls = Cell::new(0);
        assert!(verify_engine_with(
            "lay-ime-ru",
            || Err("UnknownMethod".into()),
            || {
                calls.set(calls.get() + 1);
                Ok("lay-ime-ru".into())
            }
        )
        .is_ok());
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn exact_engine_valid_mismatch_cannot_be_overridden_by_legacy_reader() {
        let error = verify_engine_with(
            "lay-ime-ru",
            || Ok("lay-ime-us".into()),
            || panic!("mismatch must not use CLI"),
        )
        .unwrap_err();
        assert!(error.contains("expected=lay-ime-ru actual=lay-ime-us"));
    }

    #[test]
    fn exact_engine_legacy_failure_is_propagated_after_one_attempt() {
        let calls = Cell::new(0);
        let error = verify_engine_with(
            "lay-ime-ru",
            || Err("disconnected".into()),
            || {
                calls.set(calls.get() + 1);
                Err("original CLI failure".into())
            },
        )
        .unwrap_err();
        assert_eq!(calls.get(), 1);
        assert_eq!(error, "original CLI failure");
    }
}
