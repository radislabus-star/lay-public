use super::*;

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
