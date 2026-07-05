use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

#[test]
fn writes_stats_with_private_permissions() {
    let tmp = std::env::temp_dir().join(format!(
        "lay-stats-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();
    let path = tmp.join("stats.json");

    write_private_stats(
        &path,
        &LayStats {
            llm_calls: 1,
            learning_log_entries: 2,
            user_corrections: 1,
            promoted_rules: 1,
            last_llm_ts: 10,
            last_learning_ts: 20,
            last_promotion_ts: 30,
        },
    )
    .unwrap();

    let stats = load(&path);
    assert_eq!(stats.llm_calls, 1);
    assert_eq!(stats.learning_log_entries, 2);
    assert_eq!(stats.user_corrections, 1);
    assert_eq!(stats.promoted_rules, 1);

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
