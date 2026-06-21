use super::*;
use std::sync::Mutex;

static ACTION_LOG_ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &std::path::Path) -> Self {
        let previous = std::env::var_os(key);
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.take() {
            std::env::set_var(self.key, previous);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

#[test]
fn action_log_keeps_only_last_lines() {
    let tmp = std::env::temp_dir().join(format!(
        "lay-action-log-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();
    let path = tmp.join("recent_actions.jsonl");

    for idx in 0..5 {
        let from = format!("from-{idx}");
        let to = format!("to-{idx}");
        let action = RecentAction {
            ts: idx,
            kind: "typing-assist",
            from: &from,
            to: &to,
            replace_words: 1,
            words: 1,
            elapsed_ms: idx as u128,
            decision_ms: None,
            output_ms: None,
            undo_available: true,
        };
        record_action_to_path(&path, &action, 3);
    }

    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(text.lines().count(), 3);
    assert!(!text.contains("from-1"));
    assert!(text.contains("from-2"));
    assert!(text.contains("from-4"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn action_log_writes_optional_stage_timings() {
    let tmp = std::env::temp_dir().join(format!(
        "lay-action-log-stage-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();
    let path = tmp.join("recent_actions.jsonl");

    let action = RecentAction {
        ts: 1,
        kind: "typing-assist",
        from: "кторое ",
        to: "которое ",
        replace_words: 1,
        words: 1,
        elapsed_ms: 42,
        decision_ms: Some(7),
        output_ms: Some(35),
        undo_available: true,
    };
    record_action_to_path(&path, &action, 3);

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("\"decision_ms\":7"));
    assert!(text.contains("\"output_ms\":35"));
    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn action_log_is_disabled_by_default_and_enabled_by_config() {
    let _lock = ACTION_LOG_ENV_LOCK.lock().unwrap();
    let tmp = std::env::temp_dir().join(format!(
        "lay-action-log-config-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let home = tmp.join("home");
    let config_path = tmp.join("config.json");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(&config_path, "{}").unwrap();
    let _home = EnvGuard::set("HOME", &home);
    let _config = EnvGuard::set(crate::config::CONFIG_PATH_ENV, &config_path);

    record_action("typing-assist", "bad", "good", 1, 1, 10, true);
    let log_path = home.join(ACTIONS_PATH);
    assert!(!log_path.exists());

    std::fs::write(&config_path, r#"{"debug_action_log":true}"#).unwrap();
    record_action("typing-assist", "bad", "good", 1, 1, 10, true);
    wait_for_path(&log_path);
    assert!(log_path.exists());

    let _ = std::fs::remove_dir_all(tmp);
}

fn wait_for_path(path: &std::path::Path) {
    for _ in 0..20 {
        if path.exists() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}
