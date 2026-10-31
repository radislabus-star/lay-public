//! Failure-only evidence in the existing trace; never input or edit authority.
use lay::desktop::LayoutBackend;
use std::fmt::Display;

pub(super) fn record_refusal(stage: &str, error: &dyn Display, may_retain: bool) {
    if !refusal_debug_enabled(crate::active_layout_backend()) {
        return;
    }
    lay::debug_log::append_private_line(
        lay::debug_log::ibus_runtime_trace_path().with_file_name("kde_exact_replay_debug.jsonl"),
        refusal_record(stage, error, may_retain).to_string(),
    );
}

fn refusal_debug_enabled(backend: LayoutBackend) -> bool {
    backend == LayoutBackend::Kde && lay::config::runtime_debug_action_log()
}

fn refusal_record(stage: &str, error: &dyn Display, may_retain: bool) -> serde_json::Value {
    serde_json::json!({
        "kind": "kde_exact_replay_refusal",
        "ts": lay::time::unix_timestamp(),
        "pid": std::process::id(),
        "stage": stage,
        "reason": may_retain.then(|| error.to_string().chars().take(512).collect::<String>()),
        "payload_retention_allowed": may_retain,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Published flags are process-global; use the canonical isolated runner.
    #[test]
    fn refusal_debug_probe_preserves_published_config_until_explicit_reload() {
        let path = std::path::PathBuf::from(
            std::env::var_os(lay::config::CONFIG_PATH_ENV)
                .expect("run this test through the canonical hermetic runner"),
        );
        let private_config = std::path::PathBuf::from(
            std::env::var_os("XDG_CONFIG_HOME").expect("private hermetic config directory"),
        );
        assert_eq!(path, private_config.join("lay/config.toml"));
        assert!(!path.exists(), "the isolated config must start empty");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();

        for enabled in [false, true] {
            let published = serde_json::json!({
                "debug_action_log": enabled,
                "fragment_layout_autoflip": enabled,
            });
            std::fs::write(&path, published.to_string()).unwrap();
            let _ = lay::config::LayConfig::load();
            let pending = serde_json::json!({
                "debug_action_log": !enabled,
                "fragment_layout_autoflip": !enabled,
            });
            std::fs::write(&path, pending.to_string()).unwrap();

            assert!(!refusal_debug_enabled(LayoutBackend::Gnome));
            assert_eq!(lay::config::runtime_debug_action_log(), enabled);
            assert_eq!(lay::config::runtime_fragment_layout_enabled(), enabled);
            let decision = refusal_debug_enabled(LayoutBackend::Kde);
            assert_eq!(
                lay::config::runtime_debug_action_log(),
                enabled,
                "a refusal debug probe must not publish pending disk configuration"
            );
            assert_eq!(lay::config::runtime_fragment_layout_enabled(), enabled);
            assert_eq!(decision, enabled);

            let _ = lay::config::LayConfig::load();
            assert_eq!(lay::config::runtime_debug_action_log(), !enabled);
            assert_eq!(lay::config::runtime_fragment_layout_enabled(), !enabled);
            assert_eq!(refusal_debug_enabled(LayoutBackend::Kde), !enabled);
        }
    }

    #[test]
    fn refusal_trace_preserves_private_payload_boundary_and_valid_json() {
        let error = format!("private-word \"quoted\"\n{}", "文".repeat(600));
        let private = refusal_record("before-layout", &error, false);
        assert!(private["reason"].is_null());
        assert!(!private.to_string().contains("private-word"));
        assert_eq!(private["payload_retention_allowed"], false);
        let permitted = refusal_record("before-layout", &error, true);
        let encoded = permitted.to_string();
        let decoded: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, permitted);
        assert_eq!(decoded["reason"].as_str().unwrap().chars().count(), 512);
        assert!(decoded["reason"]
            .as_str()
            .unwrap()
            .starts_with("private-word \"quoted\"\n"));
    }
}
