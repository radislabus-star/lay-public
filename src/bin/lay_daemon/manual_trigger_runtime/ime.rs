use super::super::{log, try_ime_manual_toggle};

pub(crate) fn run_ime_manual_toggle() -> Option<bool> {
    match try_ime_manual_toggle() {
        Ok(Some(target_is_ru)) => {
            log("· manual trigger handled by focused IME engine");
            Some(target_is_ru)
        }
        Ok(None) => {
            log("· manual trigger skipped: focused IME engine had no editable tail");
            None
        }
        Err(error) => {
            log(&format!("⚠ IME manual trigger failed: {error}"));
            None
        }
    }
}
