use super::*;

#[test]
fn config_replace_words_is_independent_from_engine_mode() {
    let simple = LayConfig {
        mode: "simple".to_string(),
        correction_engine: Some("replay".to_string()),
        replace_words: 2,
        ..LayConfig::default()
    };
    let smart = LayConfig {
        mode: "simple".to_string(),
        correction_engine: Some("smart".to_string()),
        replace_words: 2,
        ..LayConfig::default()
    };

    assert_eq!(simple.active_replace_words(), 2);
    assert_eq!(smart.active_replace_words(), 2);
    assert_eq!(simple.active_correction_engine(), CorrectionEngine::Replay);
    assert_eq!(smart.active_correction_engine(), CorrectionEngine::Smart);
}

#[test]
fn config_allows_three_word_scope() {
    let cfg = LayConfig {
        replace_words: 3,
        ..LayConfig::default()
    };
    assert_eq!(cfg.active_replace_words(), 3);

    let too_large = LayConfig {
        replace_words: 8,
        ..LayConfig::default()
    };
    assert_eq!(too_large.active_replace_words(), 3);
}

#[test]
fn typing_assist_scope_is_independent_from_manual_scope() {
    let default_cfg = LayConfig::default();
    assert_eq!(default_cfg.active_replace_words(), 1);
    assert_eq!(default_cfg.active_typing_assist_words(), 2);

    let custom = LayConfig {
        replace_words: 1,
        typing_assist_words: 3,
        ..LayConfig::default()
    };
    assert_eq!(custom.active_replace_words(), 1);
    assert_eq!(custom.active_typing_assist_words(), 3);

    let too_large = LayConfig {
        typing_assist_words: 9,
        ..LayConfig::default()
    };
    assert_eq!(too_large.active_typing_assist_words(), 3);
}

#[test]
fn auto_switch_layout_is_enabled_by_default() {
    assert!(LayConfig::default().auto_switch_layout);
}

#[test]
fn lem_scope_flags_are_enabled_by_default() {
    let cfg = LayConfig::default();
    assert!(cfg.lem_enabled);
    assert!(!cfg.lem_enabled_for_scope(1));
    assert!(cfg.lem_enabled_for_scope(2));
    assert!(cfg.lem_enabled_for_scope(3));
    assert!(cfg.lem_enabled_for_scope(8));
    assert_eq!(cfg.active_lem_weight(), 1.0);
    assert_eq!(cfg.active_nanda_l2_weight(), 1.0);
    assert_eq!(cfg.active_nanda_l3_weight(), 1.0);
    assert_eq!(
        cfg.active_typing_assist_pipeline().len(),
        default_typing_assist_rules().len()
    );
}

#[test]
fn legacy_llm_mode_maps_to_smart_only_without_explicit_engine() {
    let legacy = LayConfig {
        mode: "llm".to_string(),
        correction_engine: None,
        ..LayConfig::default()
    };
    let explicit_replay = LayConfig {
        mode: "llm".to_string(),
        correction_engine: Some("replay".to_string()),
        ..LayConfig::default()
    };

    assert_eq!(legacy.active_correction_engine(), CorrectionEngine::Smart);
    assert_eq!(
        explicit_replay.active_correction_engine(),
        CorrectionEngine::Replay
    );
}
