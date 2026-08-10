use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn writes_learning_log_as_jsonl() {
    let tmp = std::env::temp_dir().join(format!(
        "lay-learn-log-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();

    let path = tmp.join("corrections.jsonl");
    append_learning_log_to_path(&path, "layout-replay", "ghbdtn", "привет", 1, 1);
    let line = std::fs::read_to_string(&path).unwrap();
    let value: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(value["kind"], "layout-replay");
    assert_eq!(value["from"], "ghbdtn");
    assert_eq!(value["to"], "привет");
    assert!(value.get("lay_kind").is_none());
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
fn learning_feedback_records_user_fix_after_lay_correction() {
    let mut buffer = WordBuffer::new();
    buffer.remember_pending_learning_correction("typing-assist", "смотри ", "смотрин ", 1, 1);
    for _ in 0.."смотрин ".chars().count() {
        buffer.note_learning_backspace();
    }
    for key in [
        KeyCode::KEY_C,
        KeyCode::KEY_V,
        KeyCode::KEY_J,
        KeyCode::KEY_N,
        KeyCode::KEY_H,
        KeyCode::KEY_B,
    ] {
        buffer.note_learning_typed(key_event(key, true));
    }

    let correction = buffer
        .take_user_learning_correction(true)
        .expect("user correction should be captured");

    assert_eq!(
        correction,
        UserLearningCorrection {
            lay_kind: "typing-assist".to_string(),
            lay_from: "смотри ".to_string(),
            lay_to: "смотрин ".to_string(),
            from: "смотрин ".to_string(),
            to: "смотри ".to_string(),
            replace_words: 1,
            words: 1,
        }
    );
}

#[test]
fn learning_feedback_ignores_lay_output_without_user_edit() {
    let mut buffer = WordBuffer::new();
    buffer.remember_pending_learning_correction("typing-assist", "смотри ", "смотрин ", 1, 1);
    buffer.note_learning_typed(key_event(KeyCode::KEY_G, true));

    assert!(buffer.take_user_learning_correction(true).is_none());
}

#[test]
fn learning_feedback_does_not_attach_space_to_non_space_correction() {
    let mut buffer = WordBuffer::new();
    buffer.remember_pending_learning_correction("smart-text", "abc", "abd", 1, 1);
    buffer.note_learning_backspace();
    buffer.note_learning_typed(key_event(KeyCode::KEY_C, false));

    let correction = buffer
        .take_user_learning_correction(true)
        .expect("user correction should be captured");

    assert_eq!(correction.from, "d");
    assert_eq!(correction.to, "c");
    assert_eq!(correction.user_target().as_deref(), Some("abc"));
}

#[test]
fn learning_feedback_reconstructs_the_full_user_target_from_a_suffix_edit() {
    let correction = UserLearningCorrection {
        lay_kind: "typing-assist".to_string(),
        lay_from: "Праивльно? ".to_string(),
        lay_to: "Правило? ".to_string(),
        from: "ло? ".to_string(),
        to: "льно? ".to_string(),
        replace_words: 1,
        words: 1,
    };

    assert_eq!(correction.user_target().as_deref(), Some("Правильно? "));
}

#[test]
fn writes_user_correction_learning_log_with_lay_context() {
    let tmp = std::env::temp_dir().join(format!(
        "lay-user-learn-log-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();

    let path = tmp.join("corrections.jsonl");
    append_user_correction_learning_log_to_path(
        &path,
        &UserLearningCorrection {
            lay_kind: "typing-assist".to_string(),
            lay_from: "смотри ".to_string(),
            lay_to: "смотрин ".to_string(),
            from: "смотрин ".to_string(),
            to: "смотри ".to_string(),
            replace_words: 1,
            words: 1,
        },
    );

    let line = std::fs::read_to_string(&path).unwrap();
    let value: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(value["kind"], "user-correction");
    assert_eq!(value["from"], "смотрин ");
    assert_eq!(value["to"], "смотри ");
    assert_eq!(value["lay_kind"], "typing-assist");
    assert_eq!(value["lay_from"], "смотри ");
    assert_eq!(value["lay_to"], "смотрин ");
    assert_eq!(value["user_target"], "смотри ");

    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn writes_system_rollback_as_a_distinct_causal_receipt() {
    let tmp = std::env::temp_dir().join(format!(
        "lay-system-revert-log-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();

    let path = tmp.join("corrections.jsonl");
    append_reverted_system_apply_learning_log_to_path(
        &path,
        &UserLearningCorrection {
            lay_kind: "typing-assist".to_string(),
            lay_from: "проверрка ".to_string(),
            lay_to: "проверка ".to_string(),
            from: "проверка ".to_string(),
            to: "проверрка ".to_string(),
            replace_words: 1,
            words: 1,
        },
    );

    let line = std::fs::read_to_string(&path).unwrap();
    let value: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(value["kind"], "system-apply-reverted");
    assert_eq!(value["lay_from"], "проверрка ");
    assert_eq!(value["lay_to"], "проверка ");
    assert_eq!(value["user_target"], "проверрка ");

    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn repeated_user_correction_promotes_exact_rule() {
    let tmp = std::env::temp_dir().join(format!(
        "lay-learn-promote-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();

    let candidates = tmp.join("learning_candidates.json");
    let replacements = tmp.join("replacements.json");
    let correction = UserLearningCorrection {
        lay_kind: "typing-assist".to_string(),
        lay_from: "смотри ".to_string(),
        lay_to: "смотриии ".to_string(),
        from: "смотриии ".to_string(),
        to: "смотри ".to_string(),
        replace_words: 1,
        words: 1,
    };

    assert_eq!(
        promote_user_correction_if_repeated(&candidates, &replacements, &correction),
        LearningPromotion::Recorded {
            from: "смотриии".to_string(),
            to: "смотри".to_string(),
            count: 1,
        }
    );
    assert!(!replacements.exists());

    assert_eq!(
        promote_user_correction_if_repeated(&candidates, &replacements, &correction),
        LearningPromotion::Promoted {
            from: "смотриии".to_string(),
            to: "смотри".to_string(),
        }
    );

    let rules: BTreeMap<String, String> =
        serde_json::from_str(&std::fs::read_to_string(&replacements).unwrap()).unwrap();
    assert_eq!(rules.get("смотриии"), Some(&"смотри".to_string()));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&candidates).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(&replacements)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    assert_eq!(
        promoted_replacement_for_token("Смотриии"),
        Some("Смотри".to_string())
    );

    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn learning_promotion_skips_unsafe_short_edits() {
    let tmp = std::env::temp_dir().join(format!(
        "lay-learn-skip-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();

    let correction = UserLearningCorrection {
        lay_kind: "auto-replace".to_string(),
        lay_from: "b ".to_string(),
        lay_to: "в ".to_string(),
        from: "в ".to_string(),
        to: "и ".to_string(),
        replace_words: 1,
        words: 1,
    };

    assert_eq!(
        promote_user_correction_if_repeated(
            &tmp.join("learning_candidates.json"),
            &tmp.join("replacements.json"),
            &correction,
        ),
        LearningPromotion::Skipped
    );
    assert!(!tmp.join("replacements.json").exists());

    let _ = std::fs::remove_dir_all(tmp);
}
