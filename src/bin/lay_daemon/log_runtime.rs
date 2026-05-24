use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

static LOG_ENABLED: OnceLock<bool> = OnceLock::new();

pub(super) fn set_log_enabled(enabled: bool) {
    let env_enabled = std::env::var("LAY_DEBUG_LOG")
        .is_ok_and(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"));
    let _ = LOG_ENABLED.set(enabled || env_enabled);
}

pub(super) fn log(msg: &str) {
    if !*LOG_ENABLED.get_or_init(|| {
        std::env::var("LAY_DEBUG_LOG")
            .is_ok_and(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
    }) {
        return;
    }

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    eprintln!("[{ts}] {msg}");
}
