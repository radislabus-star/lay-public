use std::time::{SystemTime, UNIX_EPOCH};

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
            input_gate: None,
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
        input_gate: None,
        undo_available: true,
    };
    record_action_to_path(&path, &action, 3);

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("\"decision_ms\":7"));
    assert!(text.contains("\"output_ms\":35"));
    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn action_log_writes_input_gate_trace() {
    let tmp = std::env::temp_dir().join(format!(
        "lay-action-log-gate-test-{}",
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
        from: "lfdfq ",
        to: "давай ",
        replace_words: 1,
        words: 1,
        elapsed_ms: 12,
        decision_ms: Some(3),
        output_ms: Some(9),
        input_gate: Some(RecentActionGateTrace {
            stage: "word_boundary".to_string(),
            input_class: Some("wrong_layout".to_string()),
            candidate_count: 1,
            selected_source: Some("deterministic".to_string()),
            selected_source_id: Some("exact_layout".to_string()),
            selected_error_class: Some("wrong_layout".to_string()),
            selected_gate_action: Some("apply".to_string()),
            reason: "apply_selected_candidate".to_string(),
        }),
        undo_available: true,
    };
    record_action_to_path(&path, &action, 3);

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("\"input_gate\""));
    assert!(text.contains("\"stage\":\"word_boundary\""));
    assert!(text.contains("\"input_class\":\"wrong_layout\""));
    assert!(text.contains("\"selected_gate_action\":\"apply\""));
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
    assert!(log_path.exists());

    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn action_log_writes_dirty_task_for_applied_gate() {
    let _lock = ACTION_LOG_ENV_LOCK.lock().unwrap();
    let tmp = std::env::temp_dir().join(format!(
        "lay-action-log-dirty-task-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let home = tmp.join("home");
    let config_path = tmp.join("config.json");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(&config_path, r#"{"debug_action_log":true}"#).unwrap();
    let _home = EnvGuard::set("HOME", &home);
    let _config = EnvGuard::set(crate::config::CONFIG_PATH_ENV, &config_path);

    record_action_with_stages_and_gate(
        "typing-assist",
        "проверк ",
        "проверка ",
        1,
        1,
        12,
        Some(3),
        Some(9),
        Some(RecentActionGateTrace {
            stage: "word_boundary".to_string(),
            input_class: Some("composite-typo".to_string()),
            candidate_count: 2,
            selected_source: Some("nanda".to_string()),
            selected_source_id: Some("L2SurfaceMotifCell32".to_string()),
            selected_error_class: Some("composite-typo".to_string()),
            selected_gate_action: Some("apply".to_string()),
            reason: "apply_selected_candidate".to_string(),
        }),
        true,
    );

    let path = home.join(NANDA_DIRTY_TASKS_PATH);
    let text = std::fs::read_to_string(path).unwrap();
    assert!(text.contains("\"kind\":\"lay_dirty_task_v1\""));
    assert!(text.contains("\"from\":\"проверк \""));
    assert!(text.contains("\"to\":\"проверка \""));
    assert!(text.contains("\"selected_source_id\":\"L2SurfaceMotifCell32\""));

    let _ = std::fs::remove_dir_all(tmp);
}
