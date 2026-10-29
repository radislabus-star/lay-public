//! Failure-only evidence in the existing trace; never input or edit authority.
use std::fmt::Display;

pub(super) fn record_refusal(stage: &str, error: &dyn Display, may_retain: bool) {
    if crate::active_layout_backend() != lay::desktop::LayoutBackend::Kde
        || !lay::config::LayConfig::load().debug_action_log
    {
        return;
    }
    lay::debug_log::append_private_line(
        lay::debug_log::ibus_runtime_trace_path().with_file_name("kde_exact_replay_debug.jsonl"),
        refusal_record(stage, error, may_retain).to_string(),
    );
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
